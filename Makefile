# Runtime data root (SQLite + media). Override: make run DATA_DIR=/srv/medioss
DATA_DIR ?= $(CURDIR)/.data
UI_DIR := src/apps/web-ui
STATIC_DIR := src/apps/web-api/static

# `cargo fmt`/`cargo clippy` ship with rustup on CI runners but are separate
# packages on Nix, so run them directly when present and pull them in otherwise.
# $(1) = cargo subcommand line, $(2) = package providing it
cargo-gate = if cargo $(firstword $(1)) --version >/dev/null 2>&1; then cargo $(1); else nix-shell -p $(2) --run "cargo $(1)"; fi

.PHONY: format ui build run dev up down logs test

# Format all supported files in-place
# The pattern stays quoted so prettier expands `**` itself; /bin/sh has no
# globstar and would silently degrade it to `*/*.md`, skipping README.md.
format:
	prettier --write '**/*.md'
	@$(call cargo-gate,fmt --all,rustfmt)

# Build the Astro bundle into the API's static directory
ui:
	cd $(UI_DIR) && { [ -d node_modules ] || npm ci; } && npm run build
	rm -rf $(STATIC_DIR)
	mkdir -p $(STATIC_DIR)
	cp -r $(UI_DIR)/dist/. $(STATIC_DIR)/

# Build the release binary with the UI bundle baked into static/
build: ui
	cargo build --release

# Start the whole app locally from one process (http://localhost:3000)
run: ui
	MEDIOSS_DATA_DIR=$(DATA_DIR) cargo run -p web-api

# Frontend dev server with hot reload; run `make run` alongside it for the API
dev:
	cd $(UI_DIR) && { [ -d node_modules ] || npm ci; } && npm run dev

# Start the app in Docker exactly as deployed (http://localhost:3000)
up:
	docker compose up -d --build

down:
	docker compose down

logs:
	docker compose logs -f

# Run the same gates as CI, in the same order
test:
	@$(call cargo-gate,fmt --all --check,rustfmt)
	@$(call cargo-gate,clippy --workspace --all-targets -- -D warnings,clippy)
	cargo test --workspace
	./scripts/check-boundaries.sh --self-test && ./scripts/check-boundaries.sh
