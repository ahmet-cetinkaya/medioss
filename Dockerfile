# Stage 1: Build the Astro + Svelte frontend
FROM node:22-slim@sha256:83f487e0a63425e5b4d146fb5e5be574bcbe1b7b843d3ebafdd95eaf7767a7e5 AS frontend
WORKDIR /app
COPY src/apps/web-ui/package*.json ./
RUN npm ci
COPY src/apps/web-ui .
RUN npm run build

# Stage 2: Build the Rust workspace with the frontend output embedded as static assets
# The -bookworm suffix matches the runtime image's glibc (2.36); default tags are trixie-based (2.41).
# Base images are digest-pinned so a given commit always builds the same runtime; bump via Renovate/Dependabot.
FROM rust:1.98.0-bookworm@sha256:82150a52ec202c1b14d7817e14516c392bb7f5cfebd88f1ed531cb37ebd39922 AS builder
WORKDIR /app
COPY . .
COPY --from=frontend /app/dist src/apps/web-api/static
RUN cargo build --release --workspace

# Stage 3: Runtime image serving both the API and the UI from a single process
FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171
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
