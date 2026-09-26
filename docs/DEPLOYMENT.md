# Deployment Guide: MediOSS Wellness App

This guide covers deploying the MediOSS wellness app with its Rust backend and Astro frontend.

## Prerequisites

- Docker and docker-compose installed
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

# Start the application
docker-compose up -d

# Access the app
open http://localhost:3000
```

### Option 2: Home Server Deployment

Best for families or small groups (2-5 users).

```bash
# Clone to your home server
git clone https://github.com/ahmet-cetinkaya/medioss.git
cd medioss

# Configure for home use (optional)
cp docker-compose.yml docker-compose.home.yml
# Edit docker-compose.home.yml to adjust resource limits

# Start with custom configuration
docker-compose -f docker-compose.home.yml up -d

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
docker-compose up -d

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

# Run directly
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

# Runtime data (see ARCHITECTURE.md: Runtime Data and Media Security).
# Everything mutable — the SQLite file and uploaded media — lives under
# this one directory: $MEDIOSS_DATA_DIR/medioss.db and $MEDIOSS_DATA_DIR/media/
MEDIOSS_DATA_DIR=/data
MAX_UPLOAD_SIZE=100MB

# Session and request security
SESSION_SECRET=<strong-random-secret>
MEDIOSS_ORIGIN=https://medioss.example
SESSION_TIMEOUT=7d
SESSION_COOKIE_SECURE=true

# Backup Configuration
BACKUP_ENABLED=true
BACKUP_SCHEDULE="0 2 * * *"  # Daily at 2 AM
BACKUP_PATH=/data/backups
```

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

The stack is defined in [`docker-compose.yml`](../docker-compose.yml) at the repository root, and `make up` / `make down` drive it. It declares one service and one named volume.

The service builds the image from the `Dockerfile` above, publishes the app on port 3000, restarts unless you stop it explicitly, and polls `/api/health` so an unhealthy container is visible in `docker ps` rather than silently serving errors. Its environment sets `MEDIOSS_DATA_DIR` to the mount point and passes required `SESSION_SECRET` and `MEDIOSS_ORIGIN` values through from your shell or the `.env` file described above; the secret is never baked into the image or committed. Session timeout and secure-cookie settings retain their documented defaults unless explicitly overridden.

`/data` is the single read-write mount for everything mutable: the `medioss.db` SQLite file and the `media/` directory where users' own meditation and white noise audio is stored. There is no separate media volume, so backing up the app means backing up one volume. Because the container runs as an unprivileged user, that mount is also the only path the process needs to write; the rest of the filesystem can stay untouched.

## Resource Requirements

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

```bash
# List volumes
docker volume ls

# Backup data volume
docker run --rm -v medioss_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/medioss-data-backup.tar.gz /data

# Restore from backup
docker run --rm -v medioss_data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/medioss-data-backup.tar.gz -C /
```

## Backup Strategy

### Automated Backups

```bash
# Add to crontab for daily backups
0 2 * * * /path/to/backup-script.sh
```

`backup-script.sh`:

```bash
#!/bin/bash
BACKUP_DIR="/var/backups/wellness"
DATE=$(date +%Y%m%d)
docker exec medioss-backend sqlite3 /data/medioss.db ".backup /tmp/backup_$DATE.db"
docker cp medioss-backend:/tmp/backup_$DATE.db $BACKUP_DIR/
find $BACKUP_DIR -name "backup_*.db" -mtime +30 -delete
```

### Manual Backup

```bash
# Full backup including database and media
./scripts/backup.sh /path/to/backup/directory
```

## Monitoring

### Health Checks

```bash
# Check if service is running
curl http://localhost:3000/api/health

# Check logs
docker-compose logs -f backend

# Monitor resource usage
docker stats medioss-backend
```

### Rust-Specific Monitoring

```bash
# Check memory usage (very low for Rust)
docker stats --no-stream medioss-backend

# View application logs
docker-compose logs -f backend | grep "INFO"

# Monitor database performance
docker exec medioss-backend sqlite3 /data/medioss.db "PRAGMA integrity_check;"
```

## Troubleshooting

### Common Issues

**Application won't start:**

```bash
# Check logs
docker-compose logs backend

# Verify database permissions
docker exec medioss-backend ls -la /data/

# Check environment variables
docker-compose config
```

**Database errors:**

```bash
# Reinitialize database
docker-compose down -v
docker-compose up -d
```

**Audio playback issues:**

- Verify browser supports Web Audio API
- Check audio permissions in browser settings
- Test with different audio formats

### Performance Tuning

```bash
# Adjust Rust logging level
export RUST_LOG=warn  # Reduce logging overhead

# Optimize SQLite performance
docker exec medioss-backend sqlite3 /data/medioss.db "PRAGMA optimize;"

# Monitor memory usage (should be very low)
docker stats medioss-backend
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

```bash
# Stop current deployment
docker-compose down

# Pull latest changes
git pull origin main

# Rebuild and start
docker-compose up -d --build

# Database migrations (automatic)
# The Rust app handles migrations on startup
```

### Rollbacks

Database migrations are irreversible by design: the identity migration's
`down` refuses to run because reverting would destroy every account and
the setup-claim state. To roll back a deployment, restore the database
volume from a backup taken **before** the upgrade (see *Backup
Configuration*), then start the previous image version.

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

### Database Optimization

```bash
# Run SQLite optimization
docker exec medioss-backend sqlite3 /data/medioss.db "
PRAGMA optimize;
ANALYZE;
VACUUM;
"
```

## Support

For issues and questions:

- GitHub Issues: [github.com/ahmet-cetinkaya/medioss/issues]
- Documentation: [docs/](./)
- Rust Community: [rust-lang.org](https://www.rust-lang.org)
