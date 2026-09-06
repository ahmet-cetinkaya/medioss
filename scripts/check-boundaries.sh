#!/usr/bin/env bash
# Enforces the MediOSS dependency boundary rules (docs/ARCHITECTURE.md) over
# `cargo metadata`, so a layering mistake fails CI instead of review.
set -euo pipefail

# Normal dependencies carry `"kind": null` in cargo metadata; dev-dependencies
# carry "dev" and build-dependencies "build". Filtering on the string "normal"
# matches nothing and makes every rule pass vacuously.
readonly NORMAL_DEP='select(.kind == null)'

# Shared jq helpers: workspace members only, and the module a manifest lives in.
readonly JQ_PRELUDE='
def members: .workspace_members as $ws
  | [.packages[] | select(.id as $i | $ws | index($i))];
def modof: [scan("/src/modules/([^/]+)/")][0][0];
'

readonly DOMAIN_FORBIDDEN='["axum","sea-orm","walkdir","fs_extra","tempfile"]'
readonly PRESENTATION_FORBIDDEN='["sea-orm","walkdir","fs_extra","tempfile"]'

# Prints "BOUNDARY VIOLATION" lines and returns 1 when the workspace rooted at
# $1 breaks a rule; prints nothing and returns 0 when it is clean.
# $2.. are extra flags for cargo metadata (e.g. --offline).
check_workspace() {
  local manifest="$1"
  shift
  local metadata violations
  metadata="$(mktemp)"
  # shellcheck disable=SC2064
  trap "rm -f '$metadata'" RETURN
  cargo metadata --format-version 1 --manifest-path "$manifest" "$@" >"$metadata"

  violations="$(jq -r --argjson forbidden "$DOMAIN_FORBIDDEN" "$JQ_PRELUDE"'(
    members[]
    | select(.name | test("-(domain|application)$"))
    | . as $pkg
    | .dependencies[]
    | '"$NORMAL_DEP"'
    | select(.name as $n | $forbidden | index($n))
    | "  \($pkg.name) -> \(.name)"
  ) // empty' "$metadata")"
  if [ -n "$violations" ]; then
    echo "BOUNDARY VIOLATION: domain/application crates must not depend on frameworks, ORMs, or filesystem crates:"
    echo "$violations"
    return 1
  fi

  violations="$(jq -r --argjson forbidden "$PRESENTATION_FORBIDDEN" "$JQ_PRELUDE"'(
    members[]
    | select(.name | test("-presentation$"))
    | . as $pkg
    | .dependencies[]
    | '"$NORMAL_DEP"'
    | select(.name as $n | $forbidden | index($n))
    | "  \($pkg.name) -> \(.name)"
  ) // empty' "$metadata")"
  if [ -n "$violations" ]; then
    echo "BOUNDARY VIOLATION: presentation crates must not depend on the ORM or filesystem crates:"
    echo "$violations"
    return 1
  fi

  violations="$(jq -r "$JQ_PRELUDE"'(
    members as $all
    | ($all | map({key: .name, value: (.manifest_path | modof)}) | from_entries) as $module
    | $all[]
    | select(.manifest_path | modof)
    | . as $pkg
    | ($pkg.manifest_path | modof) as $own
    | .dependencies[]
    | '"$NORMAL_DEP"'
    | select($module[.name] != null and $module[.name] != $own)
    | "  \($pkg.name) (module \($own)) -> \(.name) (module \($module[.name]))"
  ) // empty' "$metadata")"
  if [ -n "$violations" ]; then
    echo "BOUNDARY VIOLATION: modules must not depend on each other directly:"
    echo "$violations"
    return 1
  fi

  violations="$(jq -r "$JQ_PRELUDE"'(
    members as $all
    | ($all | map({key: .name, value: .manifest_path}) | from_entries) as $path
    | $all[]
    | select(.name == "kernel" or .name == "media-store")
    | . as $pkg
    | .dependencies[]
    | select(($path[.name] // "") | contains("/src/modules/"))
    | "  \($pkg.name) -> \(.name)"
  ) // empty' "$metadata")"
  if [ -n "$violations" ]; then
    echo "BOUNDARY VIOLATION: kernel and media-store must not depend on any module:"
    echo "$violations"
    return 1
  fi

  return 0
}

member_count() {
  cargo metadata --format-version 1 --no-deps --manifest-path "$1" |
    jq -r '.workspace_members | length'
}

# Builds a throwaway workspace in $1 whose foo-domain crate has a forbidden
# normal dependency (fake local `axum`) and a harmless dev-dependency (fake
# local `tempfile`). Both fakes are path deps, so no network is required.
write_fixture() {
  local root="$1"
  mkdir -p "$root/foo-domain/src" "$root/axum/src" "$root/tempfile/src"

  cat >"$root/Cargo.toml" <<'EOF'
[workspace]
resolver = "2"
members = ["foo-domain"]
EOF

  cat >"$root/foo-domain/Cargo.toml" <<'EOF'
[package]
name = "foo-domain"
version = "0.0.0"
edition = "2021"

[dependencies]
axum = { path = "../axum" }

[dev-dependencies]
tempfile = { path = "../tempfile" }
EOF

  local fake
  for fake in axum tempfile; do
    cat >"$root/$fake/Cargo.toml" <<EOF
[package]
name = "$fake"
version = "0.0.0"
edition = "2021"
EOF
    : >"$root/$fake/src/lib.rs"
  done
  : >"$root/foo-domain/src/lib.rs"
}

self_test() {
  local root output
  root="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$root'" EXIT
  write_fixture "$root"

  if output="$(check_workspace "$root/Cargo.toml" --offline)"; then
    echo "self-test FAILED: fixture with a forbidden axum dependency was not rejected"
    return 1
  fi

  echo "self-test: fixture correctly rejected, reported as:"
  echo "$output" | sed 's/^/  | /'
  case "$output" in
    *"foo-domain -> axum"*) ;;
    *)
      echo "self-test FAILED: expected the axum normal-dependency violation to be reported"
      return 1
      ;;
  esac
  case "$output" in
    *tempfile*)
      echo "self-test FAILED: dev-dependency on tempfile was flagged; the kind filter is broken"
      return 1
      ;;
  esac

  echo "self-test: PASS (violation caught, dev-dep correctly ignored)"
}

main() {
  if [ "${1:-}" = "--self-test" ]; then
    self_test
    return
  fi

  local root
  root="$(git rev-parse --show-toplevel)"
  check_workspace "$root/Cargo.toml"
  echo "boundary check: OK ($(member_count "$root/Cargo.toml") workspace members)"
}

main "$@"
