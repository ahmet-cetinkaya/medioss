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
# every logic-bearing file. Hard-failing: every logic-bearing file needs a sibling
# .tests.rs with a #[test], wired by #[path], unless it is listed in
# scripts/layout-test-exemptions.txt (a list that may only shrink). Files holding no logic are exempt:
# re-export mod.rs/lib.rs, trait-only declarations, data-only structs, and
# generated/fixture trees (migrations, entities, test support, crate tests/).
# Upper bound the exemption list's `# count: N` header may never exceed. Only
# lower it together with the list; raising it needs a visible, reviewed edit.
EXEMPTION_CEILING=51

# True when $1 holds a non-ignored #[test]/#[tokio::test] fn and a token that
# asserts something, so commented-out, #[ignore]d or empty tests do not count.
has_asserting_test() {
  local sibling="$1" code
  code="$(grep -vE '^[[:space:]]*//' "$sibling")"
  printf '%s\n' "$code" | awk '
    /^[[:space:]]*#\[(tokio::)?test([^a-zA-Z_]|$)/ { pending = 1 }
    /^[[:space:]]*#\[ignore/ { ignored = 1 }
    /^[[:space:]]*(pub )?(async )?fn / { if (pending && !ignored) found++; pending = 0; ignored = 0 }
    END { exit found ? 0 : 1 }' &&
    printf '%s\n' "$code" | grep -qE 'assert(_eq|_ne)?!|\.expect\(|\.unwrap\(|should_panic'
}

# True when $1 declares `mod $2;` with a literal `#[cfg(test)]` among the
# attribute lines directly above it. cfg(not(test)), cfg(all(..)) and
# cfg(any(..)) are different literals and do not count.
declares_under_cfg_test() {
  local parent="$1" module="$2"
  grep -vE '^[[:space:]]*//' "$parent" | awk -v module="$module" '
    /^[[:space:]]*#\[/ { if ($0 ~ /^[[:space:]]*#\[cfg\(test\)\][[:space:]]*$/) gated = 1; next }
    $0 ~ "^[[:space:]]*(pub(\\([a-z]+\\))? )?mod " module ";[[:space:]]*$" { if (gated) found = 1 }
    { gated = 0 }
    END { exit found ? 0 : 1 }'
}

# Kind A: a file inside a router_tests/ directory, declared by name from
# router_tests/mod.rs, where lib.rs gates router_tests itself by #[cfg(test)].
is_cfg_test_module_file() {
  local file="$1" dir stem crate_dir
  dir="$(dirname "$file")"
  stem="$(basename "$file" .rs)"
  [ "$(basename "$dir")" = "router_tests" ] && [ "$stem" != "mod" ] || return 1
  [ -f "$dir/mod.rs" ] && grep -qE "^[[:space:]]*mod $stem;[[:space:]]*$" "$dir/mod.rs" || return 1
  crate_dir="$(dirname "$dir")"
  [ -f "$crate_dir/lib.rs" ] && declares_under_cfg_test "$crate_dir/lib.rs" router_tests
}

# Kind B: a SeaORM entity file holding data only: no fn, no macro_rules!, and
# no static or closure initialiser, and no impl other than the empty
# ActiveModelBehavior marker.
is_data_only_entity_file() {
  local file="$1" code
  case "$file" in */persistence/entities/*.rs | */entity.rs) ;; *) return 1 ;; esac
  code="$(grep -vE '^[[:space:]]*//' "$file")"
  printf '%s\n' "$code" | grep -qE '\bfn\b|macro_rules!|\bstatic\b|\|\|' && return 1
  ! printf '%s\n' "$code" | grep -E '\bimpl\b' |
    grep -qvE '^[[:space:]]*impl ActiveModelBehavior for ActiveModel \{\}[[:space:]]*$'
}

# Fails when the exemption list's `# count: N` is missing, below the real
# entry count, or above the ceiling.
check_exemption_budget() {
  local exemptions="$1" ceiling="$2" declared actual
  declared="$(sed -n 's/^# count: \([0-9][0-9]*\)$/\1/p' "$exemptions")"
  actual="$(grep -cvE '^(#|$)' "$exemptions")"
  if [ -z "$declared" ]; then
    echo "  exemption list lacks a '# count: N' header"
  elif [ "$actual" -gt "$declared" ]; then
    echo "  exemption list grew: $actual entries > declared count $declared"
  elif [ "$declared" -gt "$ceiling" ]; then
    echo "  exemption count $declared exceeds the ceiling $ceiling; the list may only shrink"
  fi
}

check_file_layout() {
  local root="$1" ceiling="${2:-$EXEMPTION_CEILING}" file sibling name relative
  local exemptions="$root/scripts/layout-test-exemptions.txt"
  local violations="" exempt_listed
  while IFS= read -r file; do
    case "$file" in
      */mod.rs | */lib.rs | *.tests.rs | */test_support/* | */tests/* | */migrations/* | *_entity.rs | */entities.rs) continue ;;
    esac
    relative="${file#"$root"/}"
    if grep -q 'mod tests {' "$file"; then
      violations+="  inline test module: $relative"$'\n'
      continue
    fi
    if is_cfg_test_module_file "$file" || is_data_only_entity_file "$file"; then
      if [ -f "$exemptions" ] && grep -qxF "$relative" "$exemptions"; then
        violations+="  stale exemption (exempt by kind; remove it from layout-test-exemptions.txt): $relative"$'\n'
      fi
      continue
    fi
    # Trait-only files hold signatures, not logic; data-only files hold neither
    # functions nor impls. Everything else needs a real sibling .tests.rs.
    if ! grep -qE '\bimpl\b' "$file" &&
      ! { grep -qE '\bfn\b' "$file" && ! grep -qE '\btrait\b' "$file"; }; then
      continue
    fi
    sibling="${file%.rs}.tests.rs"
    name="$(basename "$sibling")"
    exempt_listed=0
    if [ -f "$exemptions" ] && grep -qxF "$relative" "$exemptions"; then
      exempt_listed=1
    fi
    if [ -f "$sibling" ] &&
      has_asserting_test "$sibling" &&
      grep -qE "^[[:space:]]*#\[path = \"$name\"\]" "$file" &&
      grep -qE '^[[:space:]]*mod tests;' "$file"; then
      if [ "$exempt_listed" -eq 1 ]; then
        violations+="  stale exemption (it has real tests now; remove it from layout-test-exemptions.txt): $relative"$'\n'
      fi
    elif [ "$exempt_listed" -eq 0 ]; then
      violations+="  needs a wired sibling $name with at least one #[test]: $relative"$'\n'
    fi
  done < <(find "$root/src" -name '*.rs')
  if [ -f "$exemptions" ]; then
    local budget_problem
    budget_problem="$(check_exemption_budget "$exemptions" "$ceiling")"
    [ -z "$budget_problem" ] || violations+="$budget_problem"$'\n'
    while IFS= read -r relative; do
      case "$relative" in '' | '#'*) continue ;; esac
      [ -f "$root/$relative" ] || violations+="  exemption names a missing file: $relative"$'\n'
    done <"$exemptions"
  fi
  if [ -n "$violations" ]; then
    echo "LAYOUT VIOLATIONS:"
    printf '%s' "$violations"
    return 1
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

# Writes $4 as the sibling of the fixture's add.rs (with a valid hook) and
# checks the layout verdict equals $2 (accept|reject); $3 names the case.
expect_layout() {
  local dir="$1" verdict="$2" label="$3" sibling_body="$4" accepted=reject
  printf 'pub fn add(a: u8, b: u8) -> u8 { a + b }\n#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' >"$dir/src/add.rs"
  printf "$sibling_body" >"$dir/src/add.tests.rs"
  check_file_layout "$dir" >/dev/null && accepted=accept
  if [ "$accepted" = "$verdict" ]; then
    echo "  ok   ${verdict%e}ed: $label"
  else
    echo "  FAIL wrongly ${accepted%e}ed: $label"
    return 1
  fi
}

# Checks check_exemption_budget on file $1 with ceiling $2 yields verdict $3.
expect_budget() {
  local verdict="$3" label="$4" accepted=reject
  [ -z "$(check_exemption_budget "$1" "$2")" ] && accepted=accept
  if [ "$accepted" = "$verdict" ]; then
    echo "  ok   $verdict: $label"
  else
    echo "  FAIL wrongly $accepted: $label"
    return 1
  fi
}

# Proves check_file_layout hard-fails on a logic file with no real test, and
# passes once a wired sibling with a #[test] exists. Uses a throwaway root.
layout_self_test() {
  local dir ok=0
  dir="$(new_fixture_dir)"
  mkdir -p "$dir/src"
  printf 'pub fn add(a: u8, b: u8) -> u8 { a + b }\n' >"$dir/src/add.rs"
  if check_file_layout "$dir" >/dev/null; then
    echo "  FAIL wrongly accepted: 11. logic file without a sibling test"
    ok=1
  else
    echo "  ok   rejected: 11. logic file without a sibling test"
  fi
  : >"$dir/src/add.tests.rs"
  printf '#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' >>"$dir/src/add.rs"
  if check_file_layout "$dir" >/dev/null; then
    echo "  FAIL wrongly accepted: 12. hollow sibling with no #[test]"
    ok=1
  else
    echo "  ok   rejected: 12. hollow sibling with no #[test]"
  fi
  expect_layout "$dir" accept "13. wired sibling with an asserting #[test]" \
    '#[test]\nfn adds() { assert_eq!(super::add(1, 1), 2); }\n' || ok=1
  expect_layout "$dir" reject "15. commented-out #[test]" \
    '// #[test]\nfn adds() { assert!(true); }\n' || ok=1
  expect_layout "$dir" reject "16. ignored-only test" \
    '#[test]\n#[ignore]\nfn adds() { assert!(true); }\n' || ok=1
  expect_layout "$dir" reject "17. empty-body test" \
    '#[test]\nfn adds() {}\n' || ok=1
  expect_layout "$dir" accept "13b. restored asserting test" \
    '#[test]\nfn adds() { assert_eq!(super::add(1, 1), 2); }\n' || ok=1
  printf '#[cfg(test)]\n// #[path = "add.tests.rs"]\n// mod tests;\n' >"$dir/src/add.rs"
  printf 'pub fn add(a: u8, b: u8) -> u8 { a + b }\n' | cat - "$dir/src/add.rs" >"$dir/src/add.rs.new" &&
    mv "$dir/src/add.rs.new" "$dir/src/add.rs"
  if check_file_layout "$dir" >/dev/null; then
    echo "  FAIL wrongly accepted: 18. commented-out #[path] hook"
    ok=1
  else
    echo "  ok   rejected: 18. commented-out #[path] hook"
  fi
  printf '# count: 1\nsrc/a.rs\nsrc/b.rs\n' >"$dir/budget.txt"
  expect_budget "$dir/budget.txt" 5 reject "19. exemption list grew past its count" || ok=1
  printf '# count: 9\nsrc/a.rs\n' >"$dir/budget.txt"
  expect_budget "$dir/budget.txt" 5 reject "20. count raised above the ceiling" || ok=1
  printf '# count: 4\nsrc/a.rs\n' >"$dir/budget.txt"
  expect_budget "$dir/budget.txt" 5 accept "21. count lowered and entry removed" || ok=1
  mkdir -p "$dir/scripts"
  printf 'src/add.rs\n' >"$dir/scripts/layout-test-exemptions.txt"
  if check_file_layout "$dir" >/dev/null; then
    echo "  FAIL wrongly accepted: 14. stale exemption for a tested file"
    ok=1
  else
    echo "  ok   rejected: 14. stale exemption for a tested file"
  fi
  remove_fixture_dir "$dir"
  return "$ok"
}

# Exempt-by-kind cases: router_tests files (kind A) and data-only SeaORM
# entities (kind B) pass only while provably logic-free; a listed kind file is stale.
kind_self_test() {
  local dir ok=0 gate
  for gate in '#[cfg(test)]' '#[cfg(not(test))]' '#[cfg(any(test, unix))]' ''; do
    dir="$(new_fixture_dir)"
    mkdir -p "$dir/src/router_tests"
    printf '%s\n#[path = "router_tests/mod.rs"]\nmod router_tests;\n' "$gate" >"$dir/src/lib.rs"
    printf 'mod auth;\n' >"$dir/src/router_tests/mod.rs"
    printf 'pub fn helper() {}\n' >"$dir/src/router_tests/auth.rs"
    if [ "$gate" = '#[cfg(test)]' ]; then
      kind_expect "$dir" accept "22. router_tests file under #[cfg(test)]" || ok=1
    else
      kind_expect "$dir" reject "23. router_tests file under gate '${gate:-none}'" || ok=1
    fi
    remove_fixture_dir "$dir"
  done
  dir="$(new_fixture_dir)"
  mkdir -p "$dir/src/persistence/entities"
  local entity="$dir/src/persistence/entities/users.rs"
  printf 'pub struct Model;\nimpl ActiveModelBehavior for ActiveModel {}\n' >"$entity"
  kind_expect "$dir" accept "24. data-only entity" || ok=1
  printf 'impl Model { pub fn f() {} }\n' >>"$entity"
  kind_expect "$dir" reject "25. entity with a fn" || ok=1
  printf 'pub struct Model;\nimpl Model {}\nimpl ActiveModelBehavior for ActiveModel {}\n' >"$entity"
  kind_expect "$dir" reject "26. entity with a non-empty impl" || ok=1
  printf 'pub struct Model;\nmacro_rules! m { () => {}; }\nimpl ActiveModelBehavior for ActiveModel {}\n' >"$entity"
  kind_expect "$dir" reject "27. entity with macro_rules!" || ok=1
  printf 'pub static F: LazyLock<u8> = LazyLock::new(|| 1);\nimpl ActiveModelBehavior for ActiveModel {}\n' >"$entity"
  kind_expect "$dir" reject "29. entity with a static closure initialiser" || ok=1
  printf 'pub struct Model;\nimpl ActiveModelBehavior for ActiveModel {}\n' >"$entity"
  mkdir -p "$dir/scripts"
  printf '# count: 1\nsrc/persistence/entities/users.rs\n' >"$dir/scripts/layout-test-exemptions.txt"
  kind_expect "$dir" reject "28. stale exemption for a kind file" || ok=1
  remove_fixture_dir "$dir"
  dir="$(new_fixture_dir)"
  mkdir -p "$dir/src/router_tests"
  printf '#[cfg(test)]\n#[path = "router_tests/mod.rs"]\nmod router_tests;\n' >"$dir/src/lib.rs"
  printf 'mod auth;\n' >"$dir/src/router_tests/mod.rs"
  printf 'pub fn helper() {}\n' >"$dir/src/router_tests/auth.rs"
  printf 'pub fn orphan() {}\n' >"$dir/src/router_tests/orphan.rs"
  kind_expect "$dir" reject "30. router_tests file never declared in mod.rs" || ok=1
  remove_fixture_dir "$dir"
  return "$ok"
}

# Verdict of check_file_layout on fixture $1 must equal $2 (accept|reject).
kind_expect() {
  local dir="$1" verdict="$2" label="$3" accepted=reject
  check_file_layout "$dir" >/dev/null && accepted=accept
  if [ "$accepted" = "$verdict" ]; then
    echo "  ok   ${verdict%e}ed: $label"
  else
    echo "  FAIL wrongly ${accepted%e}ed: $label"
    return 1
  fi
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

  layout_self_test || failed=1
  kind_self_test || failed=1

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
  check_file_layout "$root" || return 1
}

main "$@"
