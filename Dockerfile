# Stage 1: Build the Astro + Svelte frontend
FROM node:22-slim AS frontend
WORKDIR /app
COPY src/apps/web-ui/package*.json ./
RUN npm ci
COPY src/apps/web-ui .
RUN npm run build

# Stage 2: Build the Rust workspace with the frontend output embedded as static assets
# The -bookworm suffix matches the runtime image's glibc (2.36); default tags are trixie-based (2.41).
FROM rust:1.98.0-bookworm AS builder
WORKDIR /app
COPY . .
COPY --from=frontend /app/dist src/apps/web-api/static
RUN cargo build --release --workspace

# Stage 3: Runtime image serving both the API and the UI from a single process
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/medioss /usr/local/bin/
COPY --from=builder /app/src/apps/web-api/static /srv/medioss/static
ENV MEDIOSS_STATIC_DIR=/srv/medioss/static
RUN useradd --system --create-home --uid 10001 medioss
WORKDIR /data
# A fresh named volume inherits this ownership, so the startup writability probe
# in $MEDIOSS_DATA_DIR succeeds without the process holding uid 0.
RUN chown medioss:medioss /data
USER medioss
EXPOSE 3000
CMD ["medioss"]
