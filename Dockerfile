# syntax=docker/dockerfile:1

# Stage 1: Build the Astro + Svelte frontend (bun, matching local and CI tooling)
FROM oven/bun:1.4-slim@sha256:debbe76858f2e398d2937c1eceeb82c571ac1fcd78aadf00e9634578ac2b5ef7 AS frontend
# Repo-relative layout: the About page imports NOTICE via `$repo`.
WORKDIR /repo/src/apps/web-ui
COPY NOTICE /repo/NOTICE
COPY src/apps/web-ui/package.json src/apps/web-ui/bun.lock ./
RUN bun install --frozen-lockfile
COPY src/apps/web-ui .
RUN bun run build

# Stage 2: Build the Rust workspace with the frontend output embedded as static assets
# The -bookworm suffix matches the runtime image's glibc (2.36); default tags are trixie-based (2.41).
# Base images are digest-pinned so a given commit always builds the same runtime; bump via Renovate/Dependabot.
FROM rust:1.98.0-bookworm@sha256:82150a52ec202c1b14d7817e14516c392bb7f5cfebd88f1ed531cb37ebd39922 AS builder
WORKDIR /app
COPY . .
COPY --from=frontend /repo/src/apps/web-ui/dist src/apps/web-api/static
# Cache mounts keep the ~91 locked crates and their artifacts across builds, but their
# contents never land in a layer -- hence copying the binary out to a real path.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --workspace && cp target/release/medioss /medioss

# Stage 3: Runtime image serving both the API and the UI from a single process
FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl ffmpeg && rm -rf /var/lib/apt/lists/*
COPY --from=builder /medioss /usr/local/bin/medioss
COPY --from=builder /app/src/apps/web-api/static /srv/medioss/static
COPY --from=builder /app/assets/sounds/soundscape-builtin /srv/medioss/builtin-sounds
ENV MEDIOSS_STATIC_DIR=/srv/medioss/static
RUN useradd --system --create-home --uid 10001 medioss
WORKDIR /data
# A fresh named volume inherits this ownership, so the startup writability probe
# in $MEDIOSS_DATA_DIR succeeds without the process holding uid 0.
RUN chown medioss:medioss /data
USER medioss
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1
CMD ["medioss"]
