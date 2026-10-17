#!/usr/bin/env bash
# Builds the release binary, starts a throwaway local instance, seeds demo
# content, captures screenshots into docs/screenshots/, then stops the server.
# Local only: the server listens on 127.0.0.1, its data dir is a mktemp dir in
# TMPDIR, and only the test auth state and media cache live under .agents/qa/ (gitignored).
# On NixOS run it inside `nix develop` so Playwright finds its browser libraries.
set -euo pipefail
cd "$(dirname "$0")/.."

export E2E_PORT="${DEMO_PORT:-3917}"
export SESSION_SECRET="${SESSION_SECRET:-$(openssl rand -base64 32)}"
health_url="http://127.0.0.1:$E2E_PORT/api/health"
if curl -sf "$health_url" >/dev/null; then
    echo "port $E2E_PORT already answers; stop it or set DEMO_PORT=<other>" >&2
    exit 1
fi

data_dir="$(mktemp -d "${TMPDIR:-/tmp}/medioss-demo.XXXXXX")"

make build
server_pid=""
cleanup() {
    if [ -n "$server_pid" ]; then
        kill "$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
    rm -rf "$data_dir"
}
trap cleanup EXIT

MEDIOSS_BIND=127.0.0.1 MEDIOSS_DATA_DIR="$data_dir" PORT="$E2E_PORT" MEDIOSS_ORIGIN="http://localhost:$E2E_PORT" \
    SESSION_COOKIE_SECURE=false MEDIOSS_BUILTIN_SOUNDS_DIR="$PWD/assets/sounds/soundscape-builtin" \
    MEDIOSS_STATIC_DIR="$PWD/src/apps/web-api/static" ./target/release/medioss &
server_pid=$!

for _ in $(seq 1 60); do
    kill -0 "$server_pid" 2>/dev/null || { echo "server exited during startup" >&2; exit 1; }
    curl -sf "$health_url" >/dev/null && break
    sleep 1
done
curl -sf "$health_url" >/dev/null || { echo "server did not start" >&2; exit 1; }

make demo-seed demo-screenshots
