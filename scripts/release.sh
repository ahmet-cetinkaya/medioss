#!/usr/bin/env bash
# Release helper for MediOSS.
#
#   scripts/release.sh prepare <patch|minor|major|X.Y.Z>
#       Run from an up-to-date, clean main. Creates branch release/vX.Y.Z, bumps
#       the workspace version, refreshes Cargo.lock, closes the [Unreleased]
#       section of CHANGELOG.md under the new version and today's date, moves the
#       version pins in the docs, verifies the lockfile and makes one commit.
#       It never pushes: open the PR from that branch and merge it as usual.
#
#   scripts/release.sh tag
#       Run from main after the release PR is merged. Tags the current commit
#       vX.Y.Z (X.Y.Z read from Cargo.toml) and, after a confirmation, pushes the
#       tag, which triggers the Docker publish workflow.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

MAIN_BRANCH=main
REMOTE=origin
SEMVER_PATTERN='^[0-9]+\.[0-9]+\.[0-9]+$'

die() { echo "release: $*" >&2; exit 1; }

current_version() {
  grep -m1 '^version = ' Cargo.toml | sed -E 's/^version = "([^"]+)"/\1/'
}

bump_version() {
  local current=$1 kind=$2 major minor patch
  IFS=. read -r major minor patch <<<"$current"
  case "$kind" in
    patch) echo "$major.$minor.$((patch + 1))" ;;
    minor) echo "$major.$((minor + 1)).0" ;;
    major) echo "$((major + 1)).0.0" ;;
    *) [[ $kind =~ $SEMVER_PATTERN ]] || die "not patch|minor|major|X.Y.Z: $kind"; echo "$kind" ;;
  esac
}

require_clean_tree() {
  [[ -z "$(git status --porcelain --untracked-files=no)" ]] || die "working tree has uncommitted changes"
}

require_main_in_sync() {
  [[ "$(git branch --show-current)" == "$MAIN_BRANCH" ]] || die "run from $MAIN_BRANCH"
  git fetch --quiet "$REMOTE" "$MAIN_BRANCH"
  [[ "$(git rev-parse HEAD)" == "$(git rev-parse "$REMOTE/$MAIN_BRANCH")" ]] \
    || die "$MAIN_BRANCH is not at $REMOTE/$MAIN_BRANCH; pull first"
}

# Moves the pinned release examples and "current release" mentions in the docs.
move_doc_pins() {
  local old=$1 new=$2 old_re=${1//./\\.}
  sed -i -E \
    -e "s/(medioss:)$old_re/\1$new/g" \
    -e "s/(MEDIOSS_VERSION=)$old_re/\1$new/g" \
    -e "s/([Aa]s of v)$old_re/\1$new/g" \
    -e "s/v$old_re( is the current release)/v$new\1/g" \
    README.md docs/DEPLOYMENT.md docs/docker.md docs/ARCHITECTURE.md docs/ROADMAP.md
}

close_changelog() {
  local new=$1 date=$2
  grep -q '^## \[Unreleased\]' CHANGELOG.md || die "CHANGELOG.md has no [Unreleased] section"
  awk '/^## \[Unreleased\]/{f=1;next} /^## \[/{f=0} f && NF' CHANGELOG.md | grep -q . \
    || die "the [Unreleased] section is empty: nothing to release"
  sed -i "s/^## \[Unreleased\].*/## [$new] — $date/" CHANGELOG.md
}

prepare() {
  local kind=${1:?usage: release.sh prepare <patch|minor|major|X.Y.Z>}
  require_clean_tree
  require_main_in_sync

  local old new
  old=$(current_version)
  new=$(bump_version "$old" "$kind")
  [[ "$new" != "$old" ]] || die "version is already $old"
  git rev-parse -q --verify "refs/tags/v$new" >/dev/null && die "tag v$new already exists"

  git checkout -b "release/v$new"
  sed -i -E "0,/^version = \"$old\"/s//version = \"$new\"/" Cargo.toml
  cargo update -p web-api --offline
  close_changelog "$new" "$(date +%F)"
  move_doc_pins "$old" "$new"
  cargo metadata --locked --format-version 1 >/dev/null

  git add Cargo.toml Cargo.lock CHANGELOG.md README.md docs
  git commit -m "chore: release $new"
  echo "release: v$new prepared on branch release/v$new ($old -> $new)."
  echo "Next: git push -u $REMOTE release/v$new, open the PR, merge it, then run: scripts/release.sh tag"
}

tag() {
  require_clean_tree
  require_main_in_sync

  local version
  version=$(current_version)
  git rev-parse -q --verify "refs/tags/v$version" >/dev/null && die "tag v$version already exists"
  grep -q "^## \[$version\]" CHANGELOG.md || die "CHANGELOG.md has no [$version] section: merge the release PR first"

  git tag -a "v$version" -m "v$version"
  echo "release: tagged v$version at $(git rev-parse --short HEAD)."
  read -r -p "Push v$version to $REMOTE (starts the Docker publish)? [y/N] " answer
  if [[ $answer == y || $answer == Y ]]; then
    git push "$REMOTE" "v$version"
  else
    echo "Not pushed. Push later with: git push $REMOTE v$version (or delete it: git tag -d v$version)"
  fi
}

case "${1:-}" in
  prepare) shift; prepare "$@" ;;
  tag) tag ;;
  *) die "usage: release.sh prepare <patch|minor|major|X.Y.Z> | tag" ;;
esac
