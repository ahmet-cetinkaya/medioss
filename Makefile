# Runtime data root (SQLite + media). Override: make run DATA_DIR=/srv/medioss
DATA_DIR ?= $(CURDIR)/.data
UI_DIR := src/apps/web-ui
STATIC_DIR := src/apps/web-api/static

.PHONY: format ui run dev up down logs test

# Format all supported files in-place
format:
	prettier --write **/*.md

# Build the Astro bundle into the API's static directory
ui:
	cd $(UI_DIR) && { [ -d node_modules ] || npm ci; } && npm run build
	rm -rf $(STATIC_DIR)
	mkdir -p $(STATIC_DIR)
	cp -r $(UI_DIR)/dist/. $(STATIC_DIR)/

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

test:
	cargo test --workspace
	./scripts/check-boundaries.sh
