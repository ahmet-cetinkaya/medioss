#!/usr/bin/env bash
# Runs the Astro dev server and the Axum API together; one Ctrl-C stops both.
#
# The Astro dev server (http://localhost:4321) proxies /api to the Axum server
# (http://localhost:3000), so the live UI talks to the real backend during
# development. MEDIOSS_DEV_ORIGIN lets the API accept the dev server's origin
# for browser-origin-checked mutations; it is unset in production.
set -u
cd "$(dirname "$0")/.."

export MEDIOSS_DEV_ORIGIN="${MEDIOSS_DEV_ORIGIN:-http://localhost:4321}"

cleanup() {
    trap - INT TERM EXIT
    pkill -f 'cargo watch -x run -p web-api' 2>/dev/null
    pkill -x medioss 2>/dev/null
    pkill -f 'astro dev' 2>/dev/null
    kill 0 2>/dev/null
    exit 0
}
trap cleanup INT TERM EXIT

make dev-ui &
make dev-service &
wait
