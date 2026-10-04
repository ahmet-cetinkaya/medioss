# Runtime data root (SQLite + media). Override: make run DATA_DIR=/srv/medioss
# Server port. Override: make run PORT=3001
PORT ?= 3000
DATA_DIR ?= $(CURDIR)/.data
UI_DIR := src/apps/web-ui
STATIC_DIR := src/apps/web-api/static
UI_SETUP := cd $(UI_DIR) && bun install --frozen-lockfile
PRETTIER := bunx --bun --package prettier --package prettier-plugin-svelte prettier

# `cargo fmt`/`cargo clippy` ship with rustup on CI runners but are separate
# packages on Nix, so run them directly when present and pull them in otherwise.
# $(1) = cargo subcommand line, $(2) = package providing it
cargo-gate = if cargo $(firstword $(1)) --version >/dev/null 2>&1; then cargo $(1); else nix-shell -p $(2) --run "cargo $(1)"; fi

.PHONY: help format build-ui build-service build dev-ui dev-service dev run lint test docker-build up down restart logs clean

help:
	@printf '%s\n' \
		'make format         Format Rust, Astro, Svelte, TypeScript, CSS, and Markdown' \
		'make dev            Run Astro and Axum with hot reload' \
		'make build          Build the frontend and release Rust binary' \
		'make lint           Check formatting, frontend files, Clippy, and boundaries' \
		'make test           Run frontend and workspace Rust tests' \
		'make docker-build   Build the production Docker image' \
		'make docker-up      Build and start the Docker Compose service' \
		'make docker-down    Stop and remove Docker Compose containers' \
		'make docker-restart Restart the Docker Compose service' \
		'make docker-logs    Follow Docker Compose logs'

format:
	$(UI_SETUP) && $(PRETTIER) --write '**/*.{astro,svelte,ts,tsx,css,md}'
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
dev-ui:
	$(UI_SETUP) && bun run dev

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
	$(UI_SETUP) && $(PRETTIER) --check '**/*.{astro,svelte,ts,tsx,css,md}'
	@$(call cargo-gate,fmt --all --check,rustfmt)
	@$(call cargo-gate,clippy --workspace --all-targets -- -D warnings,clippy)
	./scripts/check-boundaries.sh

test:
	$(UI_SETUP) && bun test
	cargo test --workspace
	./scripts/check-boundaries.sh --self-test

docker-build:
	docker compose build
docker-up:
	docker compose up -d --build
docker-down:
	docker compose down
docker-restart:
	docker compose up -d --build --force-recreate
docker-logs:
	docker compose logs -f

clean:
	rm -rf target $(STATIC_DIR) $(UI_DIR)/dist
