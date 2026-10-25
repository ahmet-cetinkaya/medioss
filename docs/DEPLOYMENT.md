# Deployment Guide: MediOSS

This guide covers deploying MediOSS with its Rust backend and Astro frontend.

## Prerequisites

- Docker with the Compose plugin (`docker compose`) installed
- **ffmpeg** on the host PATH (meditation uploads are converted to Ogg Opus through it; without it the server starts but uploads answer 503)
- 128MB RAM minimum (256MB recommended for multiple users)
- 5GB storage space (10GB for multiple users)
- Basic terminal familiarity

## Deployment Options

### Option 1: Quick Start (Local Development)

Best for testing and personal use on a single machine.

```bash
# Clone the repository
git clone https://github.com/ahmet-cetinkaya/medioss.git
cd medioss

# Set SESSION_SECRET and MEDIOSS_ORIGIN first (see Configuration below)
cp .env.example .env

# Start the application (or: make docker-up)
docker compose up -d

# Access the app
open http://localhost:3000
```

To skip the local build, pull the prebuilt multi-arch image (`linux/amd64`, `linux/arm64`) and point the `backend` service at it, for example with `image: ghcr.io/ahmet-cetinkaya/medioss:1.0.0` in place of `build: .`:

```bash
docker pull ghcr.io/ahmet-cetinkaya/medioss:1.0.0   # a pinned release
docker pull ghcr.io/ahmet-cetinkaya/medioss:latest  # the newest stable release
```

The Makefile wraps the Compose commands: `make docker-build`, `make docker-up`, `make docker-down`, `make docker-restart` and `make docker-logs`.

### Option 2: Home Server Deployment

Best for families or small groups (2-5 users).

```bash
# Clone to your home server
git clone https://github.com/ahmet-cetinkaya/medioss.git
cd medioss

# Configure for home use (optional)
cp docker-compose.yml compose.home.yml
# Edit compose.home.yml to adjust resource limits

# Start with custom configuration
docker compose -f compose.home.yml up -d

# Setup reverse proxy (recommended)
# Add Nginx/Caddy configuration for HTTPS
```

### Option 3: Remote Server Deployment

Best for remote access and multiple users.

#### 3.1. DigitalOcean/VPS Deployment

```bash
# SSH into your server
ssh user@your-server-ip

# Install Docker (if not already installed)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Clone and deploy
git clone https://github.com/ahmet-cetinkaya/medioss.git
cd medioss
docker compose up -d

# Configure firewall
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
```

#### 3.2. Setup Domain Name and HTTPS

```bash
# Install Caddy (automatic HTTPS)
sudo apt install caddy

# Configure Caddy
echo "yourdomain.com {
    reverse_proxy localhost:3000
}" | sudo tee /etc/caddy/Caddyfile

# Restart Caddy
sudo systemctl restart caddy
```

### Option 4: Single Binary Deployment

For maximum performance and minimal resource usage.

```bash
# Build the Rust binary
cargo build --release

# Run directly (MEDIOSS_DATA_DIR, SESSION_SECRET and MEDIOSS_ORIGIN are required;
# build the UI first with `make build-ui` or set MEDIOSS_STATIC_DIR)
./target/release/medioss

# Or install system-wide
sudo cp target/release/medioss /usr/local/bin/
medioss
```

## Configuration

### Environment Variables

Create a `.env` file in the project root:

```env
# Application
RUST_LOG=info
PORT=3000
# MEDIOSS_BIND=0.0.0.0   # listen address; 127.0.0.1 keeps the server local

# Runtime data (see ARCHITECTURE.md: Runtime Data and Media Security).
# Everything mutable — the SQLite file and uploaded media — lives under
# this one directory: $MEDIOSS_DATA_DIR/medioss.db and $MEDIOSS_DATA_DIR/media/
MEDIOSS_DATA_DIR=/data

# Session and request security
SESSION_SECRET=<strong-random-secret>
MEDIOSS_ORIGIN=https://medioss.example
SESSION_TIMEOUT=7d
SESSION_COOKIE_SECURE=true

# Optional paths
# MEDIOSS_STATIC_DIR=/srv/medioss/static               # built UI; the Docker image sets this
# MEDIOSS_BUILTIN_SOUNDS_DIR=/srv/medioss/builtin-sounds  # built-in sound set (see below)
```

The application reads these variables: `MEDIOSS_BIND`, `MEDIOSS_BUILTIN_SOUNDS_DIR`, `MEDIOSS_DATA_DIR`, `MEDIOSS_DEV_ORIGIN` (set by `make dev`; not for production), `MEDIOSS_ORIGIN`, `MEDIOSS_STATIC_DIR`, `PORT`, `RUST_LOG`, `SESSION_COOKIE_SECURE`, `SESSION_SECRET` and `SESSION_TIMEOUT`. Anything else is ignored. There is no backup setting and no upload-size setting: the upload limit is fixed at 100MB. See _Backup Strategy_ for backups.

`MEDIOSS_BUILTIN_SOUNDS_DIR` points at the directory with the built-in sound set (`manifest.json` and `LICENSE`). When it is unset the server probes `/srv/medioss/builtin-sounds` (where the Docker image ships the set) and then `./builtin-sounds`. `make run` and `make dev` do not set it, so a source checkout has no built-in sounds until you set it (the `.env` copied from `.env.example` does, with a path relative to the repo root), for example `MEDIOSS_BUILTIN_SOUNDS_DIR=$PWD/assets/sounds/soundscape-builtin make run`.

`MEDIOSS_DATA_DIR` has no default and the application refuses to start without it. There is no separate `DATABASE_URL` or `MEDIA_PATH`: both are derived from `MEDIOSS_DATA_DIR` so the runtime state stays a single directory to mount and back up.

`SESSION_SECRET` is required and must be nonblank. Generate it with `openssl rand -base64 32`. It is actively used for constant-time first-run setup host-possession authorization. It is **not** a cookie-signing key: opaque session cookies contain random server-side session IDs.

`MEDIOSS_ORIGIN` is required and must be one absolute `http://` or `https://` origin, such as `https://medioss.example` or `http://localhost:3000`. Wildcards, credentials, paths, queries, and fragments are rejected. Default ports and host casing are normalized for strict origin comparison.

`SESSION_TIMEOUT` defaults to `7d` and accepts a positive whole number followed by `d`, `h`, `m`, or `s`. `SESSION_COOKIE_SECURE` defaults to `true` and accepts only literal `true` or `false`; setting it to `false` is intended only for deliberate local HTTP/LAN deployments and emits a startup warning.

### Docker Configuration

The image is defined in [`Dockerfile`](../Dockerfile) at the repository root. Read that file for the exact instructions; this section explains the reasoning behind its shape, which the file itself cannot tell you.

It builds in three stages:

1. **Frontend build.** A Node image installs the `src/apps/web-ui` dependencies from the lockfile and runs the Astro production build. Package manifests are copied before the rest of the source so dependency installation stays cached across source-only edits.
2. **Rust build.** A Rust image compiles the workspace, with the Astro output from stage 1 copied into `src/apps/web-api/static` first, so the frontend is embedded in the same build that produces the binary. The build is `--workspace`, not just `-p web-api`: the workspace crates are compiled and checked together, so a break in a sibling crate fails the image build instead of surfacing later at runtime.
3. **Runtime.** A slim Debian image carries only the compiled binary, the static assets, and the CA certificates plus `curl` that the healthcheck needs. Neither toolchain survives into the shipped image.

Two constraints are worth calling out, because breaking them produces confusing failures:

- **The builder image is pinned to a `-bookworm` tag.** The binary is dynamically linked against glibc, so the builder's glibc must be no newer than the runtime's. Default Rust tags track a newer Debian than `bookworm-slim`, and mixing them yields a `GLIBC_x.yz not found` crash at container start, long after the build reported success. The pinned toolchain version also matches `rust-toolchain.toml`.
- **The static assets ship at `MEDIOSS_STATIC_DIR`.** The runtime image sets that variable to the path it copied the Astro output to, which is why one process serves both the API and the UI without a second web server or a separate frontend container.

### Docker Compose Setup

The stack is defined in [`docker-compose.yml`](../docker-compose.yml) at the repository root, and `make docker-up` / `make docker-down` drive it. It declares one service and one named volume.

The service builds the image from the `Dockerfile` above, publishes the app on port 3000, restarts unless you stop it explicitly, and polls `/api/health` so an unhealthy container is visible in `docker ps` rather than silently serving errors. Its environment sets `MEDIOSS_DATA_DIR` to the mount point and passes required `SESSION_SECRET` and `MEDIOSS_ORIGIN` values through from your shell or the `.env` file described above; the secret is never baked into the image or committed. Session timeout and secure-cookie settings retain their documented defaults unless explicitly overridden.

`/data` is the single read-write mount for everything mutable: the `medioss.db` SQLite file and the `media/` directory where users' own meditation and white noise audio is stored. There is no separate media volume, so backing up the app means backing up one volume. Because the container runs as an unprivileged user, that mount is also the only path the process needs to write; the rest of the filesystem can stay untouched.

## Resource Requirements

Runtime data lives under `$MEDIOSS_DATA_DIR`:

```text
$MEDIOSS_DATA_DIR/
├── medioss.db                  # SQLite: identity + meditation tables
└── media/
    ├── meditation/             # program folders: audio (Ogg Opus) + covers
    ├── soundscape/             # built-in sound audio and icons
    └── identity/               # per-user avatars
```

Backups capture everything when they capture this directory. Restoring a backup restores the whole instance.

### Performance Characteristics

**Rust Backend Advantages:**

- **Memory**: 64-128MB (vs 512MB+ for Node.js)
- **CPU**: Minimal utilization, efficient async I/O
- **Startup**: <1 second cold start time
- **Concurrency**: Handle 1000+ concurrent connections

### Resource Scaling

| Deployment Type | RAM   | Storage | CPU      | Users |
| --------------- | ----- | ------- | -------- | ----- |
| Personal        | 64MB  | 5GB     | Minimal  | 1     |
| Family          | 128MB | 10GB    | Low      | 2-5   |
| Community       | 256MB | 20GB    | Moderate | 5-20  |

## Volume Management

Compose names the data volume `<project>_app_data`. The project name defaults to the directory name, so a clone in `medioss/` gives `medioss_app_data`. Check the real name before you copy it into a command:

```bash
docker volume ls | grep app_data
```

## Backup Strategy

All state is in `$MEDIOSS_DATA_DIR`: the SQLite file `medioss.db` and the `media/` directory. Back up both together; a database without its media (or the reverse) is not a consistent copy. The application has no built-in backup job, and no backup script ships with the repository. Use the manual flow below and schedule it with host cron if you want it automated.

The runtime image contains no `sqlite3` binary, so the Docker flow uses a short-lived helper container that mounts the same volume.

### Manual Backup (Docker)

The service name is `backend` and the volume is `medioss_app_data` (see _Volume Management_). Stop the service for a copy that is guaranteed consistent:

```bash
mkdir -p backups
docker compose stop backend
docker run --rm -v medioss_app_data:/data:ro -v "$PWD/backups":/backup \
  alpine tar czf "/backup/medioss-$(date +%Y%m%d).tar.gz" -C /data .
docker compose start backend
```

To keep the service running, copy the database with the SQLite online backup command and archive the media next to it:

```bash
mkdir -p backups
docker run --rm -v medioss_app_data:/data:ro -v "$PWD/backups":/backup alpine sh -c '
  apk add --no-cache sqlite >/dev/null &&
  sqlite3 /data/medioss.db ".backup /backup/medioss.db" &&
  tar czf /backup/media.tar.gz -C /data media'
```

### Manual Backup (without Docker)

Run these on the host that has `$MEDIOSS_DATA_DIR`:

```bash
mkdir -p /path/to/backups
sqlite3 "$MEDIOSS_DATA_DIR/medioss.db" ".backup /path/to/backups/medioss.db"
tar czf /path/to/backups/media.tar.gz -C "$MEDIOSS_DATA_DIR" media
```

### Restore

Stop the service, then put the files back into an empty data directory or volume:

```bash
docker compose down
docker run --rm -v medioss_app_data:/data -v "$PWD/backups":/backup alpine \
  sh -c 'rm -rf /data/* && tar xzf /backup/medioss-YYYYMMDD.tar.gz -C /data && chown -R 10001:10001 /data'
docker compose up -d
```

For the online-backup pair, copy `medioss.db` to `/data/medioss.db`, extract `media.tar.gz` into `/data`, and keep the `10001:10001` ownership.

### Scheduled Backups

Put the manual commands in a script of your own and call it from the host crontab, for example daily at 02:00 (`BACKUP_DIR` is whatever directory you choose, such as `/var/backups/medioss`):

```bash
0 2 * * * cd /path/to/medioss && /path/to/your-backup-script.sh
```

Delete old archives in that script (for example `find "$BACKUP_DIR" -name 'medioss-*.tar.gz' -mtime +30 -delete`) and copy the archives off the host.

## Monitoring

### Health Checks

```bash
# Check if service is running
curl http://localhost:3000/api/health

# Check logs
docker compose logs -f backend

# Monitor resource usage
docker stats "$(docker compose ps -q backend)"
```

### Rust-Specific Monitoring

```bash
# Check memory usage (very low for Rust)
docker stats --no-stream "$(docker compose ps -q backend)"

# View application logs
docker compose logs -f backend | grep "INFO"

# Monitor database performance
docker run --rm -v medioss_app_data:/data:ro alpine sh -c 'apk add --no-cache sqlite >/dev/null && sqlite3 /data/medioss.db "PRAGMA integrity_check;"'
```

## Troubleshooting

### Common Issues

**Application won't start:**

```bash
# Check logs
docker compose logs backend

# Verify database permissions
docker compose exec backend ls -la /data/

# Check environment variables
docker compose config
```

**Database errors:**

```bash
# Reinitialize database
docker compose down -v
docker compose up -d
```

**Audio playback issues:**

- Verify browser supports Web Audio API
- Check audio permissions in browser settings
- Test with different audio formats

### Performance Tuning

```bash
# Adjust Rust logging level
export RUST_LOG=warn  # Reduce logging overhead

# Monitor memory usage (should be very low)
docker stats "$(docker compose ps -q backend)"
```

## Security Considerations

### Basic Security

1. **Use HTTPS** - Setup with Caddy or Nginx
2. **Firewall** - Only open necessary ports
3. **Updates** - Keep Rust dependencies updated
4. **Backups** - Regular, encrypted backup strategy

### Rust Security Features

- **Memory Safety**: Compile-time guarantees
- **SQL Injection Prevention**: SeaORM parameterizes all generated queries
- **Type Safety**: Strong type system prevents vulnerabilities
- **No Runtime**: Fewer attack surfaces than interpreted languages

### Advanced Security

```bash
# Restrict network access
# Add to docker-compose.yml:
networks:
  default:
    internal: true
```

## Upgrading

Back up first (see _Backup Strategy_).

```bash
# Stop current deployment
docker compose down

# Pull latest changes
git pull origin main

# Rebuild and start (or: make docker-up)
docker compose up -d --build

# Prebuilt image instead: change the image tag, then
docker compose pull && docker compose up -d

# Database migrations (automatic)
# The Rust app handles migrations on startup
```

### Rollbacks

Database migrations are irreversible by design: the identity migration's `down` refuses to run because reverting would destroy every account and the setup-claim state. To roll back a deployment, restore the database volume from a backup taken **before** the upgrade (see _Backup Strategy_), then start the previous image version.

## Performance Optimization

### Rust-Specific Optimizations

```bash
# Build with optimizations
cargo build --release

# Enable CPU-specific optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Reduce binary size
strip target/release/medioss

# Profile memory usage
valgrind --tool=massif ./target/release/medioss
```

## Support

For issues and questions:

- GitHub Issues: [github.com/ahmet-cetinkaya/medioss/issues]
- Documentation: [docs/](./)
- Rust Community: [rust-lang.org](https://www.rust-lang.org)
