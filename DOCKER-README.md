# 🐳 Nosdesk Docker Deployment Guide

This guide will help you run the complete Nosdesk application stack using Docker Compose, including:

- **Frontend**: Vue.js application with Nginx
- **Backend**: Rust/Actix-web API server
- **Database**: PostgreSQL 15
- **Cache**: Redis 7 (for rate limiting)

## 🚀 Quick Start

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) (20.10+)
- [Docker Compose](https://docs.docker.com/compose/install/) (2.0+)
- 4GB+ RAM available
- 10GB+ disk space

### 1. Environment Setup

```bash
# Copy the environment template
cp docker.env.example .env

# Generate a secure JWT secret
openssl rand -base64 32

# Edit .env file with your configurations
nano .env
```

**Required Configuration:**

```bash
# Update JWT_SECRET with the generated value
JWT_SECRET=your-generated-32-character-secret-key-here

# Set environment (development/production)
ENVIRONMENT=development
```

### 2. Start the Application

```bash
# Build and start all services
docker-compose up -d

# View logs (optional)
docker-compose logs -f
```

### 3. Access the Application

- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:8080
- **PostgreSQL**: localhost:5432
- **Redis**: localhost:6379

## 📋 Services Overview

### Frontend (Port 3000)
- **Technology**: Vue.js + Nginx
- **Features**: 
  - Production-optimized build
  - Gzip compression
  - Static asset caching
  - Security headers
  - API proxy to backend

### Backend (Port 8080)
- **Technology**: Rust + Actix-web
- **Features**:
  - JWT authentication
  - Rate limiting with Redis
  - File upload handling
  - Microsoft Graph integration
  - WebSocket collaboration

### PostgreSQL (Port 5432)
- **Version**: PostgreSQL 15 Alpine
- **Default Credentials**:
  - Database: `helpdesk`
  - User: `nosdesk`
  - Password: `nosdesk_password`

### Redis (Port 6379)
- **Version**: Redis 7 Alpine
- **Purpose**: Rate limiting cache
- **Password**: `nosdesk_redis_password`

## 🔧 Management Commands

### View Service Status
```bash
docker-compose ps
```

### View Logs
```bash
# All services
docker-compose logs

# Specific service
docker-compose logs backend
docker-compose logs frontend
docker-compose logs postgres
docker-compose logs redis
```

### Stop Services
```bash
# Stop all services
docker-compose down

# Stop and remove volumes (DANGER: This deletes all data)
docker-compose down -v
```

### Restart Services
```bash
# Restart all services
docker-compose restart

# Restart specific service
docker-compose restart backend
```

### Database Migrations
```bash
# Run migrations manually
docker-compose run --rm migrate

# Or access backend container
docker-compose exec backend /bin/bash
# Then run: diesel migration run
```

### Update Application
```bash
# Rebuild and restart with latest code
docker-compose build --no-cache
docker-compose up -d
```

## 🛠️ Development Workflow

### Local Development with Hot Reload

For active development, you may want to run services individually:

```bash
# Start only database and redis
docker-compose up -d postgres redis

# Run backend locally
cd backend
cargo run

# Run frontend locally (in another terminal)
cd frontend
npm run dev
```

### Debugging

#### Check Service Health
```bash
# Backend health
curl http://localhost:8080/health

# Frontend health
curl http://localhost:3000/health

# Database connection
docker-compose exec postgres psql -U nosdesk -d helpdesk -c "SELECT version();"

# Redis connection
docker-compose exec redis redis-cli ping
```

#### Access Service Shells
```bash
# Backend container
docker-compose exec backend /bin/bash

# Database shell
docker-compose exec postgres psql -U nosdesk -d helpdesk

# Redis CLI
docker-compose exec redis redis-cli
```

## 🔒 Security Features

### Rate Limiting
- **Frontend**: Nginx rate limiting (10 req/s API, 5 req/m login)
- **Backend**: Redis-based IP rate limiting (60 req/min default)

### Security Headers
- X-Frame-Options: SAMEORIGIN
- X-Content-Type-Options: nosniff
- X-XSS-Protection: 1; mode=block
- Referrer-Policy: strict-origin-when-cross-origin

### Authentication
- JWT tokens with configurable expiration
- Microsoft Entra ID integration support
- Secure password hashing with bcrypt

## 🚨 Production Deployment

### Required Changes for Production

1. **Environment Variables**:
```bash
ENVIRONMENT=production
JWT_SECRET=your-super-secure-64-character-production-secret
```

2. **HTTPS Configuration**:
   - Set up reverse proxy (Cloudflare, nginx, etc.)
   - Update CORS origins to match your domain
   - Configure SSL certificates

3. **Database Security**:
   - Use strong passwords
   - Enable SSL connections
   - Set up database backups

4. **Redis Security**:
   - Use strong password
   - Enable SSL/TLS
   - Configure persistence

### Performance Optimization

1. **Resource Limits**:
```yaml
# Add to docker-compose.yml services
deploy:
  resources:
    limits:
      memory: 512M
      cpus: '0.5'
```

2. **Database Tuning**:
   - Adjust PostgreSQL configuration
   - Set up connection pooling
   - Configure proper indexes

3. **Monitoring**:
   - Set up health checks
   - Configure log aggregation
   - Monitor resource usage

## 🐛 Troubleshooting

### Common Issues

#### Port Conflicts
```bash
# Check if ports are in use
netstat -tulpn | grep :3000
netstat -tulpn | grep :8080

# Stop conflicting services
sudo lsof -ti:3000 | xargs kill -9
```

#### Database Connection Issues
```bash
# Check database logs
docker-compose logs postgres

# Verify database is accessible
docker-compose exec postgres pg_isready -U nosdesk
```

#### Build Failures
```bash
# Clean build cache
docker system prune -f
docker-compose build --no-cache

# Check available disk space
df -h
```

#### Redis Connection Issues
```bash
# Test Redis connectivity
docker-compose exec redis redis-cli ping

# Check Redis logs
docker-compose logs redis
```

### Log Analysis

#### Backend API Errors
```bash
# View backend logs with timestamps
docker-compose logs -f --timestamps backend

# Filter for errors
docker-compose logs backend 2>&1 | grep ERROR
```

#### Frontend Issues
```bash
# Check nginx logs
docker-compose logs frontend

# Access nginx error logs
docker-compose exec frontend tail -f /var/log/nginx/error.log
```

## 📊 Monitoring & Maintenance

### Health Checks

All services include health checks:
- **Backend**: HTTP health endpoint
- **Frontend**: Nginx health endpoint  
- **PostgreSQL**: pg_isready check
- **Redis**: Redis ping command

### Backup Strategy

```bash
# Database backup
docker-compose exec postgres pg_dump -U nosdesk helpdesk > backup_$(date +%Y%m%d_%H%M%S).sql

# Volume backup
docker run --rm -v nosdesk_postgres_data:/data -v $(pwd):/backup alpine tar czf /backup/postgres_backup.tar.gz /data
```

### Updates

```bash
# Update to latest images
docker-compose pull

# Rebuild with latest code
git pull
docker-compose build --no-cache
docker-compose up -d
```

## 🆘 Support

If you encounter issues:

1. Check the [troubleshooting section](#troubleshooting)
2. Review service logs: `docker-compose logs [service-name]`
3. Verify environment configuration
4. Check Docker and system resources

For additional support, please refer to the main project documentation. 