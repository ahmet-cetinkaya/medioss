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
# .tests.rs with a #[test], wired by #[path]. There is no exemption list.
# Files holding no logic are exempt: re-export mod.rs/lib.rs (see the ratchet
# below), trait-only declarations, data-only structs, and generated/fixture
# trees (migrations, entities, crate tests/), plus four proven kinds: A
# router_tests files, B data-only entities, C gated test_support, D bin entry.

# Ratchet for logic in mod.rs/lib.rs. A
# listed file may keep its fn/impl/static/macro_rules!; an unlisted one may not
# gain any. Same shrink-only discipline: `# count: N` <= MODRS_CEILING.
MODRS_CEILING=14

# Most code lines (no blanks or comments) a crate's src/main.rs may hold and
# still count as a bin entry; the real one is 76, so this is a backstop, not a target.
BIN_ENTRY_MAX_CODE_LINES=80

# Prints $1 without /* ... */ blocks (non-nested) and // line comments, so a
# hook or test hidden inside a comment is dead code to every check below.
code_without_comments() {
  perl -0pe 's{(?<![A-Za-z0-9_])r(#*)".*?"\1}{}gs; s{/\*.*?\*/}{}gs' "$1" | grep -vE '^[[:space:]]*//'
}

# Joins lines, then drops asserts that cannot fail: assert!(true),
# assert!(!false) and assert!/assert_eq! comparing one literal with itself
# (true, false or a number). Comparisons of variables stay review-enforced.
strip_tautologies() {
  perl -0pe 's/\s+/ /g;
    s/assert!\(\s*(?:true|!\s*false)\s*[,)]//g;
    s/assert(?:_eq)?!\(\s*(true|false|-?\d+(?:\.\d+)?(?:_?[a-z]\w*)?)\s*(?:,|==)\s*\1\s*[,)]//g'
}

# True when $1 holds a non-ignored #[test]/#[tokio::test] fn and a token that
# asserts something, so commented-out, #[ignore]d or empty tests do not count.
has_asserting_test() {
  local sibling="$1" code stripped
  code="$(code_without_comments "$sibling")"
  printf '%s\n' "$code" | awk '
    /^[[:space:]]*#\[(tokio::)?test([^a-zA-Z_]|$)/ { pending = 1 }
    /^[[:space:]]*#\[ignore/ { ignored = 1 }
    /^[[:space:]]*(pub )?(async )?fn / { if (pending && !ignored) found++; pending = 0; ignored = 0 }
    END { exit found ? 0 : 1 }' || return 1
  # Captured first: `grep -q` closing a pipe early made this flaky under
  # pipefail when the writer got SIGPIPE.
  stripped="$(printf '%s\n' "$code" | strip_tautologies)"
  grep -qE 'assert(_eq|_ne)?!|\.expect\(|\.unwrap\(|should_panic' <<<"$stripped"
}

# True when $1 hooks its sibling $2 as `#[cfg(test)]` at line start, then
# `#[path = "$2"]`, then `mod tests;`, as consecutive non-comment lines.
# cfg(any(test, ..)) and cfg(not(test)) are different literals and fail.
has_wired_hook() {
  local file="$1" name="$2"
  code_without_comments "$file" | awk -v name="$name" '
    /^#\[cfg\(test\)\][[:space:]]*$/ { cfg = 1; path = 0; next }
    cfg && $0 == "#[path = \"" name "\"]" { path = 1; next }
    cfg && path && /^mod tests;[[:space:]]*$/ { found = 1 }
    { cfg = 0; path = 0 }
    END { exit found ? 0 : 1 }'
}

# True when $1 declares `mod $2;` with a literal `#[cfg(test)]` among the
# attribute lines directly above it. cfg(not(test)), cfg(all(..)) and
# cfg(any(..)) are different literals and do not count.
declares_under_cfg_test() {
  local parent="$1" module="$2"
  code_without_comments "$parent" | awk -v module="$module" '
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

# Kind C: test_support.rs / test_support/ is exempt only when its crate root
# declares it exactly once as `mod test_support;`, directly under a literal
# `#[cfg(test)]` (or the cross-crate fakes gate `#[cfg(any(test, feature =
# "test-support"))]`) at line start, with no #[path] redirect, no other #[cfg*]
# attribute (stacked cfg(not(test)), cfg_attr), and nothing but cfg(test) code
# uses it. Other cfg(any(..)), cfg(not(test)) and comments fail. The feature
# form also needs test_support_feature_is_dev_only.
test_support_declaration_is_gated() {
  local parent="$1"
  code_without_comments "$parent" | awk '
    /^#\[cfg\(test\)\][[:space:]]*$/ { gated = 1; next }
    /^#\[cfg\(any\(test, feature = "test-support"\)\)\][[:space:]]*$/ { gated = 1; next }
    /^#\[(path|cfg)/ { redirected = 1; next }
    /^#\[/ { next }
    /^(pub(\([a-z]+\))? )?mod test_support;[[:space:]]*$/ { decls++; if (gated && !redirected) ok++ }
    { gated = 0; redirected = 0 }
    END { exit (decls == 1 && ok == 1) ? 0 : 1 }'
}

# True when the crate manifest $1 defines `test-support = []` in [features]
# with no other feature (such as `default`) listing it, and no non-dev
# dependency table anywhere under $LAYOUT_ROOT enables it, so release builds
# never compile the shared fakes.
test_support_feature_is_dev_only() {
  local manifest="$1" other
  [ -f "$manifest" ] || return 1
  awk '
    /^\[/ { in_features = ($0 == "[features]") }
    in_features && /^test-support[[:space:]]*=[[:space:]]*\[\][[:space:]]*$/ { defined = 1; next }
    in_features && /test-support/ { leaked = 1 }
    END { exit (defined && !leaked) ? 0 : 1 }' "$manifest" || return 1
  while IFS= read -r other; do
    awk '
      /^\[/ { prod = ($0 ~ /dependencies/ && $0 !~ /dev-dependencies/) }
      prod && /"test-support"/ { bad = 1 }
      END { exit bad ? 0 : 1 }' "$other" && return 1
  done < <(find "$LAYOUT_ROOT" -name Cargo.toml -not -path '*/node_modules/*' -not -path '*/target/*')
  return 0
}

# True when $1 is a test-only module: a .tests.rs, a router_tests/ file, or a
# file its sibling lib.rs/mod.rs declares directly under #[cfg(test)].
is_test_only_module() {
  local file="$1" dir stem
  dir="$(dirname "$file")"
  stem="$(basename "$file" .rs)"
  case "$file" in *.tests.rs | */router_tests/* | */test_support/*) return 0 ;; esac
  local parent
  for parent in "$dir/lib.rs" "$dir/mod.rs"; do
    [ -f "$parent" ] && declares_under_cfg_test "$parent" "$stem" && return 0
  done
  return 1
}

is_gated_test_support() {
  local file="$1" dir root parent user normalised
  dir="$(dirname "$file")"
  case "$file" in
    */test_support.rs) root="$file" ;;
    *)
      while [ "$(basename "$dir")" != "test_support" ]; do
        [ "$dir" = "/" ] || [ "$dir" = "." ] && return 1
        dir="$(dirname "$dir")"
      done
      root="$dir"
      dir="$(dirname "$dir")"
      ;;
  esac
  case "$file" in */test_support.rs) dir="$(dirname "$file")" ;; esac
  parent=""
  for user in lib.rs mod.rs main.rs; do
    [ -f "$dir/$user" ] && { parent="$dir/$user"; break; }
  done
  [ -n "$parent" ] || return 1
  test_support_declaration_is_gated "$parent" || return 1
  if code_without_comments "$parent" | grep -qF 'feature = "test-support"'; then
    test_support_feature_is_dev_only "$dir/../Cargo.toml" || return 1
  fi
  [ "$(code_without_comments "$parent" | grep -c 'test_support')" -eq 1 ] || return 1
  [ -f "$root" ] && grep -q 'fn main' "$root" && return 1
  while IFS= read -r user; do
    [ "$user" = "$parent" ] && continue
    is_test_only_module "$user" && continue
    normalised="$(code_without_comments "$user" | perl -0pe 's/\s+/ /g; s/\s*::\s*/::/g')"
    grep -qE '(crate|self|super)::(\{[^;]*)?test_support|mod test_support|(^|[^:A-Za-z_])test_support::' <<<"$normalised" && return 1
  done < <(find "$dir" -name '*.rs' -not -path '*/test_support/*' -not -name test_support.rs)
  return 0
}

# Kind D: a binary entry point (src/main.rs of a crate) that is only process
# glue: `fn main` plus `fn run`, declarations and data. Any other fn, an impl,
# a static, macro_rules! or an inline `mod x { }` means logic lives there and it
# needs a sibling test like any other file.
is_bin_entry_file() {
  local file="$1" code names lines
  [ "$(basename "$file")" = "main.rs" ] || return 1
  [ "$(basename "$(dirname "$file")")" = "src" ] || return 1
  [ -f "$(dirname "$(dirname "$file")")/Cargo.toml" ] || return 1
  code="$(code_without_comments "$file")"
  # Backstop: process glue stays small. Counted before whitespace collapse.
  lines="$(grep -cv '^[[:space:]]*$' <<<"$code" || true)"
  [ "$lines" -le "$BIN_ENTRY_MAX_CODE_LINES" ] || return 1
  # Collapsed so `fn\nname()` and `mod a\n{` read like their one-line forms.
  code="$(perl -0pe 's/\s+/ /g' <<<"$code")"
  grep -qE '\bfn +main\b' <<<"$code" || return 1
  grep -qE '\bimpl\b|\bstatic\b|macro_rules!|\bmod +[A-Za-z0-9_]+ *\{' <<<"$code" && return 1
  # Item-level blocks run code at compile time without a fn: const X = { loop {} };
  grep -qE '\b(const|static)\b[^;]*= *\{|\b(loop|while)\b' <<<"$code" && return 1
  names="$(grep -oE '\bfn +[A-Za-z0-9_]+' <<<"$code" | awk '{print $2}' | grep -vxE 'main|run' || true)"
  [ -z "$names" ]
}

# True when directory $1 is matched by a `members` entry (globs allowed) of
# the workspace manifest at $LAYOUT_ROOT/Cargo.toml, so a planted Cargo.toml
# cannot turn src/x/tests/ into a trusted integration-test directory.
is_workspace_member_dir() {
  local member pattern candidate
  [ -f "$LAYOUT_ROOT/Cargo.toml" ] || return 1
  while IFS= read -r pattern; do
    for candidate in "$LAYOUT_ROOT"/$pattern; do
      [ "$candidate" = "$1" ] && return 0
    done
  done < <(sed -n '/^members[[:space:]]*=/,/\]/p' "$LAYOUT_ROOT/Cargo.toml" | grep -o '"[^"]*"' | tr -d '"')
  return 1
}

# True when $1 sits under a tests/ directory directly beside a Cargo.toml, i.e.
# a cargo integration test; a tests/ folder inside src/ is production code.
is_crate_integration_test() {
  local dir
  dir="$(dirname "$1")"
  while [ "$dir" != "/" ] && [ "$dir" != "." ]; do
    [ "$(basename "$dir")" = "tests" ] && [ -f "$(dirname "$dir")/Cargo.toml" ] &&
      is_workspace_member_dir "$(dirname "$dir")" && return 0
    dir="$(dirname "$dir")"
  done
  return 1
}

# True when $1 (a mod.rs or lib.rs) holds fn, impl, static or macro_rules!.
modrs_has_logic() {
  local code
  code="$(code_without_comments "$1")"
  grep -qE '\bfn\b|\bimpl\b|\bstatic\b|macro_rules!' <<<"$code"
}

# Prints violations for the mod.rs/lib.rs ratchet under $1 using baseline $2.
check_modrs_ratchet() {
  local root="$1" baseline="$2" ceiling="${3:-$MODRS_CEILING}" file relative declared actual
  while IFS= read -r file; do
    case "$file" in */test_support/* | */router_tests/*) continue ;; esac
    modrs_has_logic "$file" || continue
    relative="${file#"$root"/}"
    if [ ! -f "$baseline" ] || ! grep -qxF "$relative" "$baseline"; then
      echo "  logic in mod.rs/lib.rs must be extracted to a named file or added to the baseline in a reviewed edit: $relative"
    fi
  done < <(find "$root/src" \( -name mod.rs -o -name lib.rs \))
  [ -f "$baseline" ] || return 0
  declared="$(sed -n 's/^# count: \([0-9][0-9]*\)$/\1/p' "$baseline")"
  actual="$(grep -cvE '^(#|$)' "$baseline")"
  if [ -z "$declared" ]; then
    echo "  mod.rs baseline lacks a '# count: N' header"
  elif [ "$actual" -gt "$declared" ]; then
    echo "  mod.rs baseline grew: $actual entries > declared count $declared"
  elif [ "$declared" -gt "$ceiling" ]; then
    echo "  mod.rs baseline count $declared exceeds the ceiling $ceiling; it may only shrink"
  fi
  while IFS= read -r relative; do
    case "$relative" in '' | '#'*) continue ;; esac
    if [ ! -f "$root/$relative" ]; then
      echo "  mod.rs baseline names a missing file: $relative"
    elif ! modrs_has_logic "$root/$relative"; then
      echo "  stale mod.rs baseline entry (no logic left; remove it): $relative"
    fi
  done <"$baseline"
}

# */mod.rs and */lib.rs are skipped below for the sibling-test rule; the
# check_modrs_ratchet baseline stops new logic from landing in them.
check_file_layout() {
  local root="$1" file sibling name relative
  local violations=""
  LAYOUT_ROOT="$root"
  while IFS= read -r file; do
    relative="${file#"$root"/}"
    case "$file" in
      */test_support/* | */test_support.rs)
        if ! is_gated_test_support "$file"; then
          violations+="  test_support not gated by a single literal #[cfg(test)] declaration: $relative"$'\n'
        fi
        continue
        ;;
    esac
    case "$file" in
      */mod.rs | */lib.rs | *.tests.rs | */migrations/* | *_entity.rs | */entities.rs) continue ;;
    esac
    is_crate_integration_test "$file" && continue
    relative="${file#"$root"/}"
    if grep -q 'mod tests {' "$file"; then
      violations+="  inline test module: $relative"$'\n'
      continue
    fi
    if is_cfg_test_module_file "$file" || is_data_only_entity_file "$file" || is_bin_entry_file "$file"; then
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
    if ! { [ -f "$sibling" ] && has_asserting_test "$sibling" && has_wired_hook "$file" "$name"; }; then
      violations+="  needs a wired sibling $name with at least one #[test]: $relative"$'\n'
    fi
  done < <(find "$root/src" -name '*.rs')
  local ratchet
  ratchet="$(check_modrs_ratchet "$root" "$root/scripts/layout-modrs-baseline.txt")"
  [ -z "$ratchet" ] || violations+="$ratchet"$'\n'
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

# Checks check_file_layout on a fixture whose add.rs ends in hook text $4
# and whose sibling body is $5 yields verdict $2.
expect_hook() {
  local dir="$1" verdict="$2" label="$3" hook="$4" body="$5" accepted=reject
  printf 'pub fn add(a: u8, b: u8) -> u8 { a + b }\n' >"$dir/src/add.rs"
  printf "$hook" >>"$dir/src/add.rs"
  printf "$body" >"$dir/src/add.tests.rs"
  check_file_layout "$dir" >/dev/null && accepted=accept
  if [ "$accepted" = "$verdict" ]; then
    echo "  ok   ${verdict%e}ed: $label"
  else
    echo "  FAIL wrongly ${accepted%e}ed: $label"
    return 1
  fi
}

# Checks check_file_layout on a fresh fixture crate whose lib.rs is $4, whose
# test_support/shipped.rs holds branching logic, and whose extra file $5 (path
# under src/, may be empty) has content $6; yields verdict $2.
expect_support() {
  local verdict="$2" label="$3" lib="$4" extra="${5:-}" extra_body="${6:-}" manifest="${7:-}" dir accepted=reject
  dir="$(new_fixture_dir)"
  mkdir -p "$dir/src/test_support"
  printf "$lib" >"$dir/src/lib.rs"
  printf 'pub fn pick(a: bool) -> u8 { if a { 1 } else { 2 } }\n' >"$dir/src/test_support/shipped.rs"
  [ -z "$extra" ] || { mkdir -p "$(dirname "$dir/src/$extra")"; printf "$extra_body" >"$dir/src/$extra"; }
  [ -z "$manifest" ] || printf "$manifest" >"$dir/Cargo.toml"
  check_file_layout "$dir" >/dev/null && accepted=accept
  if [ "$accepted" = "$verdict" ]; then
    echo "  ok   ${verdict%e}ed: $label"
  else
    echo "  FAIL wrongly ${accepted%e}ed: $label"
    return 1
  fi
}

# Checks check_file_layout on a fixture whose src/<file> is $4 and whose mod.rs
# baseline is $5 (empty: no baseline file); yields verdict $2.
expect_modrs() {
  local verdict="$2" label="$3" file="$4" body="$5" baseline="${6:-}" dir accepted=reject
  dir="$(new_fixture_dir)"
  mkdir -p "$dir/src/topic" "$dir/scripts"
  printf "$body" >"$dir/src/$file"
  [ -z "$baseline" ] || printf "$baseline" >"$dir/scripts/layout-modrs-baseline.txt"
  check_file_layout "$dir" >/dev/null && accepted=accept
  if [ "$accepted" = "$verdict" ]; then
    echo "  ok   ${verdict%e}ed: $label"
  else
    echo "  FAIL wrongly ${accepted%e}ed: $label"
    return 1
  fi
}

# Checks check_file_layout on a fixture crate whose src/main.rs is $4 yields
# verdict $2.
expect_bin_entry() {
  local verdict="$2" label="$3" body="$4" dir accepted=reject
  dir="$(new_fixture_dir)"
  mkdir -p "$dir/src"
  : >"$dir/Cargo.toml"
  printf "$body" >"$dir/src/main.rs"
  check_file_layout "$dir" >/dev/null && accepted=accept
  if [ "$accepted" = "$verdict" ]; then
    echo "  ok   ${verdict%e}ed: $label"
  else
    echo "  FAIL wrongly ${accepted%e}ed: $label"
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
  expect_hook "$dir" reject "31. hook without #[cfg(test)]" \
    '#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert_eq!(super::add(1, 1), 2); }\n' || ok=1
  expect_hook "$dir" reject "32. hook gated by cfg(any(test, debug_assertions))" \
    '#[cfg(any(test, debug_assertions))]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert_eq!(super::add(1, 1), 2); }\n' || ok=1
  expect_hook "$dir" reject "33. hook gated by cfg(not(test))" \
    '#[cfg(not(test))]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert_eq!(super::add(1, 1), 2); }\n' || ok=1
  expect_hook "$dir" reject "34. indented cfg(test) hook" \
    '    #[cfg(test)]\n    #[path = "add.tests.rs"]\n    mod tests;\n' '#[test]\nfn adds() { assert_eq!(super::add(1, 1), 2); }\n' || ok=1
  expect_hook "$dir" reject "35. tests file of only assert!(true)" \
    '#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert!(true); }\n' || ok=1
  expect_hook "$dir" accept "36. literal cfg(test) hook, real assertion" \
    '#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert_eq!(super::add(1, 1), 2); }\n' || ok=1
  expect_support "$dir" accept "37. test_support under literal cfg(test), declared once" \
    '#[cfg(test)]\nmod test_support;\n' || ok=1
  expect_support "$dir" accept "38. test_support under the cross-crate test-support gate" \
    '#[cfg(any(test, feature = "test-support"))]\npub mod test_support;\n' '' '' '[features]\ntest-support = []\n' || ok=1
  expect_support "$dir" reject "39. test_support declared without cfg" \
    'pub mod test_support;\n' || ok=1
  expect_support "$dir" reject "40. cfg(test) only in a comment" \
    '// #[cfg(test)]\nmod test_support;\n' || ok=1
  expect_support "$dir" reject "41. test_support under cfg(any(test, debug_assertions))" \
    '#[cfg(any(test, debug_assertions))]\nmod test_support;\n' || ok=1
  expect_support "$dir" reject "42. test_support under cfg(not(test))" \
    '#[cfg(not(test))]\nmod test_support;\n' || ok=1
  expect_support "$dir" reject "43. pub use leaks test_support" \
    '#[cfg(test)]\nmod test_support;\npub use test_support::shipped::pick;\n' || ok=1
  expect_support "$dir" reject "44. #[path] redirect on test_support" \
    '#[cfg(test)]\n#[path = "elsewhere.rs"]\nmod test_support;\n' || ok=1
  expect_support "$dir" reject "45. test_support declared twice" \
    '#[cfg(test)]\nmod test_support;\n#[cfg(test)]\nmod test_support;\n' || ok=1
  expect_support "$dir" reject "46. production file uses crate::test_support" \
    '#[cfg(test)]\nmod test_support;\nmod prod;\n' prod.rs 'use crate::test_support::shipped::pick;\npub fn run() -> u8 { pick(true) }\n' || ok=1
  expect_hook "$dir" reject "47. hook inside a block comment" \
    '/*\n#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n*/\n' '#[test]\nfn adds() { assert_eq!(super::add(1, 1), 2); }\n' || ok=1
  expect_hook "$dir" reject "48. cfg(test) only inside a one-line block comment" \
    ' /* #[cfg(test)] */\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert_eq!(super::add(1, 1), 2); }\n' || ok=1
  expect_hook "$dir" reject "49. multi-line assert!(true)" \
    '#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() {\n    assert!(\n        true\n    );\n}\n' || ok=1
  expect_hook "$dir" reject "50. assert_eq!(1, 1) literal tautology" \
    '#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert_eq!(1, 1); }\n' || ok=1
  expect_hook "$dir" reject "51. assert!(true == true) tautology" \
    '#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert!(true == true); }\n' || ok=1
  expect_hook "$dir" reject "52. assert!(!false) tautology" \
    '#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert!(!false); }\n' || ok=1
  expect_hook "$dir" accept "53. assert_eq!(1, 2 - 1) is not a literal tautology" \
    '#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert_eq!(1, 2 - 1); }\n' || ok=1
  expect_hook "$dir" accept "54. assert_eq!(v, v) on a variable stays review-enforced" \
    '#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { let v = super::add(1, 1); assert_eq!(v, v); }\n' || ok=1
  dir="$(new_fixture_dir)"
  mkdir -p "$dir/src/tests"
  printf 'pub fn pick(a: bool) -> u8 { if a { 1 } else { 2 } }\n' >"$dir/src/tests/shipped.rs"
  if check_file_layout "$dir" >/dev/null; then
    echo "  FAIL wrongly accepted: 55. production logic hidden in src/tests/"
    ok=1
  else
    echo "  ok   rejected: 55. production logic hidden in src/tests/"
  fi
  : >"$dir/src/Cargo.toml"
  if check_file_layout "$dir" >/dev/null; then
    echo "  FAIL wrongly accepted: 57a. tests/ beside a Cargo.toml that is not a workspace member"
    ok=1
  else
    echo "  ok   rejected: 57a. tests/ beside a Cargo.toml that is not a workspace member"
  fi
  printf '[workspace]\nmembers = ["sr*"]\n' >"$dir/Cargo.toml"
  if check_file_layout "$dir" >/dev/null; then
    echo "  ok   accepted: 56. tests/ directly beside a Cargo.toml"
  else
    echo "  FAIL wrongly rejected: 56. tests/ directly beside a Cargo.toml"
    ok=1
  fi
  expect_support "$dir" reject "57. feature gate without a [features] entry" \
    '#[cfg(any(test, feature = "test-support"))]\npub mod test_support;\n' '' '' '[package]\nname = "x"\n' || ok=1
  expect_support "$dir" reject "58. feature gate with test-support in default" \
    '#[cfg(any(test, feature = "test-support"))]\npub mod test_support;\n' '' '' '[features]\ndefault = ["test-support"]\ntest-support = []\n' || ok=1
  expect_support "$dir" reject "59. non-dev dependency enables test-support" \
    '#[cfg(any(test, feature = "test-support"))]\npub mod test_support;\n' 'consumer/Cargo.toml' '[dependencies]\nx = { path = "..", features = ["test-support"] }\n' '[features]\ntest-support = []\n' || ok=1
  expect_support "$dir" accept "60. dev-dependency enables test-support" \
    '#[cfg(any(test, feature = "test-support"))]\npub mod test_support;\n' 'consumer/Cargo.toml' '[dev-dependencies]\nx = { path = "..", features = ["test-support"] }\n' '[features]\ntest-support = []\n' || ok=1
  expect_support "$dir" reject "61. production use crate::{test_support::x}" \
    '#[cfg(test)]\nmod test_support;\nmod prod;\n' 'prod.rs' 'use crate::{test_support::shipped::pick};\npub fn run() -> u8 { 1 }\n' '' || ok=1
  expect_support "$dir" reject "62. production use crate :: test_support (spaces)" \
    '#[cfg(test)]\nmod test_support;\nmod prod;\n' 'prod.rs' 'use crate\n    :: test_support::shipped::pick;\npub fn run() -> u8 { 1 }\n' '' || ok=1
  expect_support "$dir" reject "63. production glob import then bare test_support::" \
    '#[cfg(test)]\nmod test_support;\nmod prod;\n' 'prod.rs' 'use crate::*;\npub fn run() -> u8 { test_support::shipped::pick(true) }\n' '' || ok=1
  expect_support "$dir" reject "64. test_support declared inside an inline module" \
    'mod inner {\n    #[cfg(test)]\n    mod test_support;\n}\n' '' '' '' || ok=1
  expect_support "$dir" reject "65. stacked cfg(test) and cfg(not(test))" \
    '#[cfg(test)]\n#[cfg(not(test))]\nmod test_support;\n' '' '' '' || ok=1
  expect_support "$dir" reject "66. cfg_attr path redirect" \
    '#[cfg(test)]\n#[cfg_attr(test, path = "elsewhere.rs")]\nmod test_support;\n' '' '' '' || ok=1
  expect_modrs "$dir" reject "67. new mod.rs with a fn and no baseline" topic/mod.rs 'pub fn pick() -> u8 { 1 }\n' || ok=1
  expect_modrs "$dir" accept "68. baselined mod.rs keeps its fn" topic/mod.rs 'pub fn pick() -> u8 { 1 }\n' '# count: 1\nsrc/topic/mod.rs\n' || ok=1
  expect_modrs "$dir" reject "69. baseline entry whose mod.rs has no logic left is stale" topic/mod.rs 'pub mod a;\n' '# count: 1\nsrc/topic/mod.rs\n' || ok=1
  expect_modrs "$dir" reject "70. baseline grew past its declared count" topic/mod.rs 'pub fn pick() -> u8 { 1 }\n' '# count: 0\nsrc/topic/mod.rs\n' || ok=1
  expect_modrs "$dir" reject "71. new lib.rs with an impl" lib.rs 'struct A;\nimpl A {}\n' || ok=1
  expect_modrs "$dir" accept "72. re-export-only mod.rs" topic/mod.rs 'pub mod a;\npub use a::B;\n' || ok=1
  expect_hook "$dir" reject "73. hook inside a raw string" \
    'const X: &str = r#"\n#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n"#;\n' '#[test]\nfn adds() { assert_eq!(super::add(1, 1), 2); }\n' || ok=1
  expect_hook "$dir" reject "74. typed literal tautology assert_eq!(1u8, 1u8)" \
    '#[cfg(test)]\n#[path = "add.tests.rs"]\nmod tests;\n' '#[test]\nfn adds() { assert_eq!(1u8, 1u8); }\n' || ok=1
  expect_bin_entry "$dir" accept "75. main.rs holding only main and run" 'fn main() { run(); }\nasync fn run() {}\n' || ok=1
  expect_bin_entry "$dir" reject "76. main.rs with an extra fn" 'fn main() { run(); }\nasync fn run() {}\nfn helper() -> u8 { 1 }\n' || ok=1
  expect_bin_entry "$dir" reject "77. main.rs with an impl" 'fn main() { run(); }\nasync fn run() {}\nstruct A;\nimpl A {}\n' || ok=1
  expect_bin_entry "$dir" reject "78. main.rs with a static" 'fn main() { run(); }\nasync fn run() {}\nstatic X: u8 = 1;\n' || ok=1
  expect_bin_entry "$dir" reject "79. main.rs with macro_rules!" 'fn main() { run(); }\nasync fn run() {}\nmacro_rules! m { () => {} }\n' || ok=1
  expect_bin_entry "$dir" reject "80. main.rs with an inline module" 'fn main() { run(); }\nasync fn run() {}\nmod logic { pub const A: u8 = 1; }\n' || ok=1
  expect_bin_entry "$dir" reject "81. main.rs without fn main" 'fn run() {}\n' || ok=1
  expect_bin_entry "$dir" accept "82. main.rs declaring an out-of-line module" 'fn main() { run(); }\nasync fn run() {}\nmod operator;\n' || ok=1
  expect_bin_entry "$dir" reject "83. main.rs with a fn name on its own line" 'fn main() { run(); }\nasync fn run() {}\nfn\nhelper() -> u8 { 1 }\n' || ok=1
  expect_bin_entry "$dir" reject "84. main.rs with an inline module brace on the next line" 'fn main() { run(); }\nasync fn run() {}\nmod logic\n{ pub const A: u8 = 1; }\n' || ok=1
  expect_bin_entry "$dir" reject "85. main.rs with a const block initialiser" 'fn main() { run(); }\nasync fn run() {}\nconst X: u8 = { let mut i = 0; while i < 3 { i += 1; } i };\n' || ok=1
  expect_bin_entry "$dir" reject "86. main.rs with a loop in an item-level block" 'fn main() { run(); }\nasync fn run() {}\nconst Y: u8 = { loop { break 1; } };\n' || ok=1
  expect_bin_entry "$dir" reject "87. main.rs over the code-line cap" "$(printf 'fn main() {}\\n'; for i in $(seq 1 85); do printf 'use a%s;\\n' "$i"; done)" || ok=1
  remove_fixture_dir "$dir"
  return "$ok"
}

# Exempt-by-kind cases: router_tests files (kind A) and data-only SeaORM
# entities (kind B) pass only while provably logic-free.
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
