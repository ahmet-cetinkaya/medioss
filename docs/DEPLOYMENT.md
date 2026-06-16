# Deployment Guide: MediOSS Wellness App

This guide covers deploying the MediOSS wellness app with Rust backend and React frontend.

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

# Database (default: SQLite)
DATABASE_URL=sqlite:///data/wellness.db

# Media Storage
MEDIA_PATH=/media
MAX_UPLOAD_SIZE=50MB

# Optional Authentication
ENABLE_AUTH=false
JWT_SECRET=your-secret-key-here

# Backup Configuration
BACKUP_ENABLED=true
BACKUP_SCHEDULE="0 2 * * *"  # Daily at 2 AM
BACKUP_PATH=/data/backups
```

### Docker Configuration

```dockerfile
# Dockerfile for Rust backend
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl
COPY --from=builder /app/target/release/medioss /usr/local/bin/
COPY --from=builder /app/media /media
COPY --from=builder /app/migrations /migrations
WORKDIR /data
EXPOSE 3000
CMD ["medioss"]
```

### Docker Compose Setup

```yaml
# docker-compose.yml
version: "3.8"
services:
  backend:
    build: .
    ports: ["3000:3000"]
    volumes:
      - app_data:/data
      - ./media:/media:ro
    environment:
      - RUST_LOG=info
      - DATABASE_URL=sqlite:///data/wellness.db
      - MEDIA_PATH=/media
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  app_data:
    driver: local
```

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
docker run --rm -v medioss_app_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/wellness-data-backup.tar.gz /data

# Restore from backup
docker run --rm -v medioss_app_data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/wellness-data-backup.tar.gz -C /
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
docker exec medioss-backend sqlite3 /data/wellness.db ".backup /tmp/backup_$DATE.db"
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
docker exec medioss-backend sqlite3 /data/wellness.db "PRAGMA integrity_check;"
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
docker exec medioss-backend sqlite3 /data/wellness.db "PRAGMA optimize;"

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
- **SQL Injection Prevention**: SQLx compile-time verification
- **Type Safety**: Strong type system prevents vulnerabilities
- **No Runtime**: Fewer attack surfaces than interpreted languages

### Advanced Security

```bash
# Enable authentication (coming in Phase 2)
# Add to .env:
ENABLE_AUTH=true
JWT_SECRET=<strong-random-secret>

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
docker exec medioss-backend sqlite3 /data/wellness.db "
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
