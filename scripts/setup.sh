#!/usr/bin/env bash
# First-run setup for MediOSS local development.
#
# - Creates .env from .env.example if missing (never overwrites an existing one)
# - Generates a fresh SESSION_SECRET into it (openssl, 32 random bytes)
# - Creates the data directory
# - Optionally builds the web UI if it has not been built yet
# - Prints the next steps (start the server, complete first-run setup)
#
# Idempotent: safe to re-run; existing values are left untouched.
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ ! -f .env.example ]]; then
  echo "error: .env.example not found (run from the repository root)" >&2
  exit 1
fi

# 1. .env
if [[ -f .env ]]; then
  echo "• .env already exists — leaving it as is"
else
  cp .env.example .env
  echo "• Created .env from .env.example"
fi

# 2. SESSION_SECRET: generate only when empty or absent
secret_line() { grep -E '^SESSION_SECRET=' .env || true; }
current=$(secret_line | cut -d= -f2-)
if [[ -z "${current// /}" ]]; then
  if command -v openssl >/dev/null 2>&1; then
    secret=$(openssl rand -base64 32)
  else
    # Fallback for hosts without openssl: hex from /dev/urandom (256 bits).
    secret=$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')
  fi
  if [[ -n "$(secret_line)" ]]; then
    sed -i.bak "s|^SESSION_SECRET=.*|SESSION_SECRET=${secret}|" .env && rm -f .env.bak
  else
    printf '\nSESSION_SECRET=%s\n' "$secret" >> .env
  fi
  echo "• Generated SESSION_SECRET ($(command -v openssl >/dev/null 2>&1 && echo openssl || echo urandom), 256-bit)"
else
  echo "• SESSION_SECRET already set — keeping it"
fi

# 3. Data directory (must match MEDIOSS_DATA_DIR in .env)
data_dir=$(grep -E '^MEDIOSS_DATA_DIR=' .env | head -1 | cut -d= -f2-)
data_dir=${data_dir:-./data}
mkdir -p "$data_dir"
echo "• Data directory ready: $data_dir"

# 4. Web UI build (skip when already built)
if [[ -d src/apps/web-ui/dist ]]; then
  echo "• Web UI already built (src/apps/web-ui/dist)"
else
  if command -v bun >/dev/null 2>&1; then
    echo "• Building web UI…"
    (cd src/apps/web-ui && bun install --frozen-lockfile && bun run build)
  else
    echo "! bun not found — skipping UI build; run it later:" >&2
    echo "    cd src/apps/web-ui && bun install && bun run build" >&2
  fi
fi

echo
echo "Setup complete. Next:"
echo "  1. Start the server:   cargo run -p web-api"
echo "     (.env is loaded automatically; real env vars still take priority)"
echo "  2. Open the app:       http://localhost:3000"
echo "  3. First-run setup registers the initial admin account in the UI"
echo "     (needs the SESSION_SECRET from .env)."
