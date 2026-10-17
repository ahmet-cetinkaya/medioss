#!/usr/bin/env bash
# Builds the release binary, starts a throwaway local instance, seeds demo
# content, captures screenshots into docs/screenshots/, then stops the server.
# Local only: the data dir and test credentials live under .agents/qa/ (gitignored).
# On NixOS run it inside `nix develop` so Playwright finds its browser libraries.
set -euo pipefail
cd "$(dirname "$0")/.."

export E2E_PORT="${DEMO_PORT:-3917}"
export SESSION_SECRET="${SESSION_SECRET:-$(openssl rand -base64 32)}"
data_dir="$(mktemp -d "${TMPDIR:-/tmp}/medioss-demo.XXXXXX")"

make build
server_pid=""
cleanup() {
    [ -n "$server_pid" ] && kill "$server_pid" 2>/dev/null
    rm -rf "$data_dir"
}
trap cleanup EXIT

MEDIOSS_DATA_DIR="$data_dir" PORT="$E2E_PORT" MEDIOSS_ORIGIN="http://localhost:$E2E_PORT" \
    SESSION_COOKIE_SECURE=false MEDIOSS_BUILTIN_SOUNDS_DIR="$PWD/assets/sounds/soundscape-builtin" \
    MEDIOSS_STATIC_DIR="$PWD/src/apps/web-api/static" ./target/release/medioss &
server_pid=$!

for _ in $(seq 1 60); do
    curl -sf "localhost:$E2E_PORT/api/health" >/dev/null && break
    sleep 1
done
curl -sf "localhost:$E2E_PORT/api/health" >/dev/null || { echo "server did not start" >&2; exit 1; }

make demo-seed demo-screenshots
