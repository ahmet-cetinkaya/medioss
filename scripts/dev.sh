#!/usr/bin/env bash
# Runs the Astro dev server and the Axum API together; one Ctrl-C stops both.
set -u
cd "$(dirname "$0")/.."

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
