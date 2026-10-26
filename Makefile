# Runtime data root (SQLite + media). Override: make run DATA_DIR=/srv/medioss
# Server port. Override: make run PORT=3001
PORT ?= 3000
DATA_DIR ?= $(CURDIR)/.data
UI_DIR := src/apps/web-ui
STATIC_DIR := src/apps/web-api/static
UI_SETUP := cd $(UI_DIR) && bun install --frozen-lockfile
PRETTIER := bunx --bun prettier
# Two prettier runs: web-ui sources with its plugins (web-ui/.prettierrc), and
# every Markdown, YAML and JSON file in the rest of the repo from the repo root
# (docs/, specs/, .github/, root files; see .prettierignore).
PRETTIER_UI_FILES := '**/*.{astro,svelte,ts,tsx,css,md}'
PRETTIER_REPO_FILES := '**/*.{md,yml,yaml,json}' '!src/apps/web-ui/**'

# `cargo fmt`/`cargo clippy` ship with rustup on CI runners but are separate
# packages on Nix, so run them directly when present and pull them in otherwise.
# $(1) = cargo subcommand line, $(2) = package providing it
cargo-gate = if cargo $(firstword $(1)) --version >/dev/null 2>&1; then cargo $(1); else nix-shell -p $(2) --run "cargo $(1)"; fi

.PHONY: help format build-ui build-service build dev-ui dev-service dev run lint test docker-build up down restart logs clean demo demo-seed demo-screenshots

help:
	@printf '%s\n' \
		'make format         Format Rust, Astro, Svelte, TypeScript, CSS, and Markdown' \
		'make dev            Hot-reload dev stack: Astro on :4321 (proxies /api to Axum on :3000) + Axum with cargo-watch' \
		'make build          Build the frontend and release Rust binary' \
		'make lint           Check formatting, frontend files, Clippy, and boundaries' \
		'make test           Run frontend and workspace Rust tests' \
		'make demo           Seed a throwaway instance and write screenshots to docs/screenshots (see docs/demo.md)' \
		'make docker-build   Build the production Docker image' \
		'make docker-up      Build and start the Docker Compose service' \
		'make docker-down    Stop and remove Docker Compose containers' \
		'make docker-restart Restart the Docker Compose service' \
		'make docker-logs    Follow Docker Compose logs'

format:
	$(UI_SETUP) && $(PRETTIER) --write $(PRETTIER_UI_FILES)
	$(PRETTIER) --write $(PRETTIER_REPO_FILES)
	@$(call cargo-gate,fmt --all,rustfmt)

build-ui:
	$(UI_SETUP) && bun run build
	rm -rf $(STATIC_DIR)
	mkdir -p $(STATIC_DIR)
	cp -r $(UI_DIR)/dist/. $(STATIC_DIR)/

build-service:
	cargo build --release --workspace

build: build-ui build-service

# Run both servers in one terminal; Ctrl-C stops both processes.
# The Astro dev server on :4321 proxies /api to the Axum server on $(PORT),
# so the live UI in the browser talks to the real backend (see
# astro.config.mjs `server.proxy` and scripts/dev.sh `MEDIOSS_DEV_ORIGIN`).
dev-ui:
	$(UI_SETUP) && ASTRO_API_TARGET=http://localhost:$(PORT) bun run dev

dev-service:
	@if ss -ltn "sport = :$(PORT)" | grep -q ":$(PORT)"; then \
		echo "make: port $(PORT) is busy — stop the process (ss -ltnp 'sport = :$(PORT)') or run: make dev PORT=<other>"; \
		exit 1; \
	fi
	@if command -v cargo-watch >/dev/null 2>&1; then \
		MEDIOSS_DATA_DIR=$(DATA_DIR) cargo watch -x 'run -p web-api'; \
	else \
		MEDIOSS_DATA_DIR=$(DATA_DIR) nix-shell -p cargo-watch --run "cargo watch -x 'run -p web-api'"; \
	fi

dev:
	./scripts/dev.sh

run: build
	@if ss -ltn "sport = :$(PORT)" | grep -q ":$(PORT)"; then \
		echo "make: port $(PORT) is busy — stop the process (ss -ltnp 'sport = :$(PORT)') or run: make run PORT=<other>"; \
		exit 1; \
	fi
	MEDIOSS_DATA_DIR=$(DATA_DIR) PORT=$(PORT) cargo run -p web-api

lint:
	$(UI_SETUP) && $(PRETTIER) --check $(PRETTIER_UI_FILES)
	$(PRETTIER) --check $(PRETTIER_REPO_FILES)
	cargo metadata --locked --format-version 1 > /dev/null
	@$(call cargo-gate,fmt --all --check,rustfmt)
	@$(call cargo-gate,clippy --workspace --all-targets -- -D warnings,clippy)
	./scripts/check-boundaries.sh

test:
	$(UI_SETUP) && bun test
	cargo test --workspace
	./scripts/check-boundaries.sh --self-test

docker-build:
	docker compose -f docker-compose.dev.yml build
docker-up:
	docker compose -f docker-compose.dev.yml up -d --build
docker-down:
	docker compose -f docker-compose.dev.yml down
docker-restart:
	docker compose -f docker-compose.dev.yml up -d --build --force-recreate
docker-logs:
	docker compose -f docker-compose.dev.yml logs -f

# Demo content and screenshots (docs/demo.md). demo-seed and demo-screenshots
# need a running empty instance on DEMO_PORT and SESSION_SECRET in the environment.
DEMO_PORT ?= 3917
demo:
	./scripts/demo.sh
demo-seed:
	cd $(UI_DIR) && E2E_PORT=$(DEMO_PORT) bun demo/seed-demo.ts
demo-screenshots:
	cd $(UI_DIR) && E2E_PORT=$(DEMO_PORT) bun demo/screenshots.ts

clean:
	rm -rf target $(STATIC_DIR) $(UI_DIR)/dist
