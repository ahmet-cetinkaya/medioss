#!/usr/bin/env bash
# Enforces the MediOSS dependency boundary rules (docs/ARCHITECTURE.md) over
# `cargo metadata`, so a layering mistake fails CI instead of review.
set -euo pipefail

# Normal dependencies carry `"kind": null` in cargo metadata; dev-dependencies
# carry "dev" and build-dependencies "build". Filtering on the string "normal"
# matches nothing and makes every rule pass vacuously.
readonly NORMAL_DEP_FILTER='select(.kind == null)'

# Shared jq helpers. Module and layer come from `.manifest_path`, never from the
# crate name: a crate misnamed `identity_domain` still sits in a domain
# directory and must obey the domain rules. The name suffix is only a fallback
# for crates that live outside `src/modules/`.
readonly JQ_PRELUDE='
def members: .workspace_members as $ws
  | [.packages[] | select(.id as $i | $ws | index($i))];
def module_of: [scan("/src/modules/([^/]+)/")][0][0];
def layer_of:
  ([.manifest_path | scan("/src/modules/[^/]+/(domain|application|infrastructure|presentation)/")][0][0])
  // ([.name | scan("-(domain|application|infrastructure|presentation)$")][0][0]);
def rank($layer):
  if $layer == null then 99
  else {"domain": 0, "application": 1, "infrastructure": 2, "presentation": 2}[$layer] // 99
  end;
def crate_index: members
  | map({key: .name, value: {module: (.manifest_path | module_of), layer: layer_of, path: .manifest_path}})
  | from_entries;
'

# Domain and application crates are governed by an ALLOWLIST, so an unreviewed
# dependency fails closed. Anything outside this list, `kernel`, and the crate's
# own lower layer is a violation (ARCHITECTURE.md "Dependency Rules").
readonly LAYER_NEUTRAL_DEPS='["serde","thiserror","async-trait","chrono","uuid","futures-io","unicode-normalization"]'
readonly PRESENTATION_FORBIDDEN='["sea-orm","sqlx","rusqlite","diesel","walkdir","fs_extra","fs-err","tempfile"]'
# `kernel` and `media-store` sit beside the module graph and beside each other.
readonly SHARED_FORBIDDEN='["kernel","media-store","web-api","axum","sea-orm"]'

# Prints a headed violation block and returns 1 when $1 is non-empty; prints
# nothing and returns 0 otherwise.
report_violations() {
  local violations="$1" header="$2"
  if [ -z "$violations" ]; then
    return 0
  fi
  echo "BOUNDARY VIOLATION: $header:"
  echo "$violations"
  return 1
}

# Prints "BOUNDARY VIOLATION" lines and returns 1 when the workspace rooted at
# $1 breaks a rule; prints nothing and returns 0 when it is clean. Every rule is
# evaluated so one run reports every problem, not just the first.
# $2.. are extra flags for cargo metadata (e.g. --offline).
check_workspace() {
  local manifest="$1"
  shift
  local metadata violations failed=0
  metadata="$(mktemp)"
  # shellcheck disable=SC2064
  trap "rm -f '$metadata'" RETURN
  cargo metadata --format-version 1 --manifest-path "$manifest" "$@" >"$metadata"

  violations="$(jq -r --argjson neutral "$LAYER_NEUTRAL_DEPS" "$JQ_PRELUDE"'(
    crate_index as $crates
    | members[]
    | . as $pkg
    | ($pkg | layer_of) as $layer
    | select($layer == "domain" or $layer == "application")
    | ($pkg.manifest_path | module_of) as $module
    | $pkg.dependencies[]
    | '"$NORMAL_DEP_FILTER"'
    | . as $dep
    | ($crates[$dep.name]) as $target
    | (
        $module != null and $target != null
        and $target.module == $module
        and rank($target.layer) < rank($layer)
      ) as $own_lower_layer
    | select(($neutral | index($dep.name)) == null and $dep.name != "kernel" and ($own_lower_layer | not))
    | "  \($pkg.name) [\($layer)] -> \($dep.name)"
  ) // empty' "$metadata")"
  report_violations "$violations" \
    "domain/application crates may only depend on kernel, their own lower layer, and framework-neutral libraries" ||
    failed=1

  violations="$(jq -r --argjson forbidden "$PRESENTATION_FORBIDDEN" "$JQ_PRELUDE"'(
    members[]
    | . as $pkg
    | select(($pkg | layer_of) == "presentation")
    | .dependencies[]
    | '"$NORMAL_DEP_FILTER"'
    | select(.name as $n | $forbidden | index($n))
    | "  \($pkg.name) -> \(.name)"
  ) // empty' "$metadata")"
  report_violations "$violations" \
    "presentation crates must not depend on the ORM or filesystem crates" ||
    failed=1

  violations="$(jq -r "$JQ_PRELUDE"'(
    crate_index as $crates
    | members[]
    | . as $pkg
    | ($pkg.manifest_path | module_of) as $own
    | select($own != null)
    | .dependencies[]
    | '"$NORMAL_DEP_FILTER"'
    | . as $dep
    | ($crates[$dep.name]) as $target
    | select($target != null and $target.module != null and $target.module != $own)
    | "  \($pkg.name) (module \($own)) -> \($dep.name) (module \($target.module))"
  ) // empty' "$metadata")"
  report_violations "$violations" "modules must not depend on each other directly" || failed=1

  violations="$(jq -r --argjson forbidden "$SHARED_FORBIDDEN" "$JQ_PRELUDE"'(
    crate_index as $crates
    | members[]
    | . as $pkg
    | select($pkg.name == "kernel" or $pkg.name == "media-store")
    | .dependencies[]
    | '"$NORMAL_DEP_FILTER"'
    | . as $dep
    | select(
        ((($crates[$dep.name].path // "") | contains("/src/modules/"))
         or (($forbidden | index($dep.name)) != null))
        and $dep.name != $pkg.name
      )
    | "  \($pkg.name) -> \($dep.name)"
  ) // empty' "$metadata")"
  report_violations "$violations" \
    "kernel and media-store must not depend on any module, on each other, or on a framework" ||
    failed=1

  # Inside a module dependencies point strictly downward. infrastructure and
  # presentation share a rank, so a sibling dependency between them is a
  # violation too.
  violations="$(jq -r "$JQ_PRELUDE"'(
    crate_index as $crates
    | members[]
    | . as $pkg
    | ($pkg.manifest_path | module_of) as $module
    | select($module != null)
    | ($pkg | layer_of) as $layer
    | .dependencies[]
    | '"$NORMAL_DEP_FILTER"'
    | . as $dep
    | ($crates[$dep.name]) as $target
    | select($target != null and $target.module == $module)
    | select(rank($target.layer) >= rank($layer))
    | "  \($pkg.name) [\($layer)] -> \($dep.name) [\($target.layer)]"
  ) // empty' "$metadata")"
  report_violations "$violations" \
    "within a module dependencies must point downward (presentation/infrastructure -> application -> domain)" ||
    failed=1

  return "$failed"
}

member_count() {
  cargo metadata --format-version 1 --no-deps --manifest-path "$1" |
    jq -r '.workspace_members | length'
}

# Reports file-layout drift against docs/ARCHITECTURE.md "File Organization
# Inside a Crate": no inline test modules, and a sibling <name>.tests.rs beside
# every logic-bearing file. Advisory (WARN, never fails) until the layout
# migration completes; flip to failing then. Files holding no logic are exempt:
# re-export mod.rs/lib.rs, trait-only declarations, data-only structs, and
# generated/fixture trees (migrations, entities, test support, crate tests/).
check_file_layout() {
  local root="$1" file warnings=""
  while IFS= read -r file; do
    case "$file" in
      */mod.rs | */lib.rs | *.tests.rs | */test_support/* | */tests/* | */migrations/* | *_entity.rs | */entities.rs) continue ;;
    esac
    if grep -q 'mod tests {' "$file"; then
      warnings+="  inline test module: $file"$'\n'
      continue
    fi
    # Trait-only files hold signatures, not logic; data-only files hold neither
    # functions nor impls. Everything else needs a sibling .tests.rs.
    if grep -qE '\bimpl\b' "$file"; then
      [ -f "${file%.rs}.tests.rs" ] || warnings+="  missing tests: $file"$'\n'
    elif grep -qE '\bfn\b' "$file" && ! grep -qE '\btrait\b' "$file"; then
      [ -f "${file%.rs}.tests.rs" ] || warnings+="  missing tests: $file"$'\n'
    fi
  done < <(find "$root/src" -name '*.rs')
  if [ -n "$warnings" ]; then
    echo "LAYOUT WARNINGS (advisory until the layout migration completes):"
    printf '%s' "$warnings"
  fi
}

# Writes a crate named $1 at directory $2; its manifest body (dependency
# sections) is read from stdin. Path dependencies are absolute so a fixture
# manifest never has to count `../` hops.
write_crate() {
  local name="$1" dir="$2"
  mkdir -p "$dir/src"
  {
    printf '[package]\nname = "%s"\nversion = "0.0.0"\nedition = "2021"\n' "$name"
    cat
  } >"$dir/Cargo.toml"
  : >"$dir/src/lib.rs"
}

# Local stand-ins for third-party crates, so the self-test never needs network.
write_stubs() {
  local root="$1" name
  shift
  for name in "$@"; do
    write_crate "$name" "$root/stubs/$name" </dev/null
  done
}

# Builds a workspace that breaks every rule this script must enforce, plus the
# historic axum-normal-dep / tempfile-dev-dep pair.
write_violating_fixture() {
  local root="$1"
  write_stubs "$root" axum sea-orm tokio sqlx rusqlite tempfile

  cat >"$root/Cargo.toml" <<'EOF'
[workspace]
resolver = "2"
members = [
  "src/kernel",
  "src/shared/media-store",
  "src/modules/meditation/domain",
  "src/modules/meditation/application",
  "src/modules/identity/domain",
  "src/modules/identity/infrastructure",
  "src/modules/identity/presentation",
  "foo-domain",
]
EOF

  write_crate kernel "$root/src/kernel" </dev/null

  # Case 1: media-store must not depend on kernel.
  write_crate media-store "$root/src/shared/media-store" <<EOF

[dependencies]
kernel = { path = "$root/src/kernel" }
EOF

  # Cases 2, 4 and 5: a domain crate reaching for runtime/database crates, for
  # its own application layer, for media-store, and across module lines.
  write_crate meditation-domain "$root/src/modules/meditation/domain" <<EOF

[dependencies]
tokio = { path = "$root/stubs/tokio" }
sqlx = { path = "$root/stubs/sqlx" }
rusqlite = { path = "$root/stubs/rusqlite" }
meditation-application = { path = "$root/src/modules/meditation/application" }
media-store = { path = "$root/src/shared/media-store" }
EOF

  write_crate meditation-application "$root/src/modules/meditation/application" <<EOF

[dependencies]
identity_domain = { path = "$root/src/modules/identity/domain" }
EOF

  # Case 3: an underscore-named crate at a domain path is still a domain crate.
  write_crate identity_domain "$root/src/modules/identity/domain" <<EOF

[dependencies]
axum = { path = "$root/stubs/axum" }
sea-orm = { path = "$root/stubs/sea-orm" }
EOF

  write_crate identity-infrastructure "$root/src/modules/identity/infrastructure" </dev/null

  # Case 6: presentation must not reach sideways into infrastructure.
  write_crate identity-presentation "$root/src/modules/identity/presentation" <<EOF

[dependencies]
identity-infrastructure = { path = "$root/src/modules/identity/infrastructure" }
EOF

  # Historic case: forbidden normal dependency plus a harmless dev-dependency.
  write_crate foo-domain "$root/foo-domain" <<EOF

[dependencies]
axum = { path = "$root/stubs/axum" }

[dev-dependencies]
tempfile = { path = "$root/stubs/tempfile" }
EOF
}

# Builds a workspace that exercises every layer legally, so the self-test also
# proves the checker does not simply reject everything.
write_clean_fixture() {
  local root="$1"
  write_stubs "$root" serde thiserror async-trait axum sea-orm tokio

  cat >"$root/Cargo.toml" <<'EOF'
[workspace]
resolver = "2"
members = [
  "src/kernel",
  "src/shared/media-store",
  "src/modules/meditation/domain",
  "src/modules/meditation/application",
  "src/modules/meditation/infrastructure",
  "src/modules/meditation/presentation",
]
EOF

  write_crate kernel "$root/src/kernel" <<EOF

[dependencies]
serde = { path = "$root/stubs/serde" }
EOF

  write_crate media-store "$root/src/shared/media-store" <<EOF

[dependencies]
tokio = { path = "$root/stubs/tokio" }
EOF

  write_crate meditation-domain "$root/src/modules/meditation/domain" <<EOF

[dependencies]
kernel = { path = "$root/src/kernel" }
serde = { path = "$root/stubs/serde" }
thiserror = { path = "$root/stubs/thiserror" }
EOF

  write_crate meditation-application "$root/src/modules/meditation/application" <<EOF

[dependencies]
kernel = { path = "$root/src/kernel" }
meditation-domain = { path = "$root/src/modules/meditation/domain" }
async-trait = { path = "$root/stubs/async-trait" }
EOF

  write_crate meditation-infrastructure "$root/src/modules/meditation/infrastructure" <<EOF

[dependencies]
kernel = { path = "$root/src/kernel" }
meditation-domain = { path = "$root/src/modules/meditation/domain" }
meditation-application = { path = "$root/src/modules/meditation/application" }
media-store = { path = "$root/src/shared/media-store" }
sea-orm = { path = "$root/stubs/sea-orm" }
EOF

  write_crate meditation-presentation "$root/src/modules/meditation/presentation" <<EOF

[dependencies]
kernel = { path = "$root/src/kernel" }
meditation-domain = { path = "$root/src/modules/meditation/domain" }
meditation-application = { path = "$root/src/modules/meditation/application" }
axum = { path = "$root/stubs/axum" }
serde = { path = "$root/stubs/serde" }
EOF
}

new_fixture_dir() {
  mktemp -d "${TMPDIR:-/tmp}/medioss-boundary-XXXXXXXX"
}

# Only ever removes a directory new_fixture_dir created.
remove_fixture_dir() {
  local dir="${1:-}"
  case "$dir" in
    */medioss-boundary-*) rm -rf "$dir" ;;
    *) echo "refusing to remove unexpected fixture directory: '$dir'" >&2 ;;
  esac
}

expect_reported() {
  local output="$1" needle="$2" label="$3"
  case "$output" in
    *"$needle"*)
      echo "  ok   rejected: $label"
      ;;
    *)
      echo "  FAIL not rejected: $label (expected \"$needle\")"
      return 1
      ;;
  esac
}

expect_absent() {
  local output="$1" needle="$2" label="$3"
  case "$output" in
    *"$needle"*)
      echo "  FAIL wrongly reported: $label"
      return 1
      ;;
    *)
      echo "  ok   accepted: $label"
      ;;
  esac
}

self_test() {
  local bad_root clean_root output failed=0
  bad_root="$(new_fixture_dir)"
  clean_root="$(new_fixture_dir)"
  # shellcheck disable=SC2064
  trap "remove_fixture_dir '$bad_root'; remove_fixture_dir '$clean_root'" EXIT
  write_violating_fixture "$bad_root"
  write_clean_fixture "$clean_root"

  if output="$(check_workspace "$bad_root/Cargo.toml" --offline)"; then
    echo "self-test FAILED: the violating fixture was not rejected"
    return 1
  fi

  echo "self-test: violating fixture rejected, reported as:"
  echo "$output" | sed 's/^/  | /'
  echo "self-test: assertions"

  expect_reported "$output" "media-store -> kernel" \
    "1. media-store depends on kernel" || failed=1
  expect_reported "$output" "meditation-domain [domain] -> tokio" \
    "2a. domain depends on tokio" || failed=1
  expect_reported "$output" "meditation-domain [domain] -> sqlx" \
    "2b. domain depends on sqlx" || failed=1
  expect_reported "$output" "meditation-domain [domain] -> rusqlite" \
    "2c. domain depends on rusqlite" || failed=1
  expect_reported "$output" "identity_domain [domain] -> axum" \
    "3a. underscore-named domain crate depends on axum" || failed=1
  expect_reported "$output" "identity_domain [domain] -> sea-orm" \
    "3b. underscore-named domain crate depends on sea-orm" || failed=1
  expect_reported "$output" "meditation-domain [domain] -> meditation-application" \
    "4. domain depends on its own application layer" || failed=1
  expect_reported "$output" "meditation-domain [domain] -> media-store" \
    "5. domain depends on media-store" || failed=1
  expect_reported "$output" "identity-presentation [presentation] -> identity-infrastructure" \
    "6. presentation depends on its sibling infrastructure" || failed=1
  expect_reported "$output" "meditation-application (module meditation) -> identity_domain (module identity)" \
    "7. module depends on another module" || failed=1
  expect_reported "$output" "foo-domain [domain] -> axum" \
    "8. domain depends on axum" || failed=1
  expect_absent "$output" "tempfile" \
    "9. dev-dependency on tempfile is not a violation" || failed=1

  if output="$(check_workspace "$clean_root/Cargo.toml" --offline)"; then
    echo "  ok   accepted: 10. a fully layered legal workspace"
  else
    echo "  FAIL wrongly reported: 10. a fully layered legal workspace"
    echo "$output" | sed 's/^/  | /'
    failed=1
  fi

  if [ "$failed" -ne 0 ]; then
    echo "self-test: FAILED"
    return 1
  fi
  echo "self-test: PASS (all violations caught, dev-dep and legal workspace accepted)"
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
  check_file_layout "$root"
}

main "$@"
