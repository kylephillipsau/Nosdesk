# Nosdesk

A modern helpdesk and IT management system built with Rust and Vue.js, designed for efficient ticket management and IT operations.

> [!CAUTION]
> This project is in active development, do not use it until it is stable and production ready. I am moving fast and breaking things!
> This software is provided as-is with absolutely no warranty. Use at your own risk!

## ✨ Key Features

### 🎫 **Ticket Management**
- **Real-time collaborative editing** with markdown support for ticket notes
- **Linked tickets** to track related issues and dependencies
- **Comments and attachments** for comprehensive issue documentation
- **Live updates** across all connected users for seamless collaboration

### 📋 **Project Management**
- **Ticket-based projects** to organize multiple related requests
- **Project dashboards** to track progress across multiple tickets
- **Team collaboration** on complex multi-ticket initiatives

### 👥 **User & Device Management**
- **Comprehensive user management** with role-based access control
- **Device tracking and assignment** to users and tickets
- **Detailed user and device profiles** for complete IT asset management

### 🔐 **Authentication & Integration**
- **Microsoft Intune & Entra ID integration** for OAuth SSO
- **Automated user and device import** from Microsoft services
- **Local user accounts** with TOTP-based multi-factor authentication
- **Flexible authentication options** for different organizational needs

### 🚀 **Real-time Collaboration**
- **WebSocket-powered live editing** for ticket notes and documentation
- **Server-sent events** for instant notifications and updates
- **Collaborative markdown editor** with ProseMirror and Yjs integration

## 🚀 Quick Start

**Prerequisites:** Docker and Docker Compose

```bash
# 1. Clone the repository
git clone https://github.com/kylephillipsau/Nosdesk.git
cd Nosdesk

# 2. Create environment configuration
cp docker.env.example docker.env

# 3. Update required environment variables in docker.env:
# - JWT_SECRET: Generate with `openssl rand -base64 32`
# - MFA_ENCRYPTION_KEY: Generate with `openssl rand -hex 32`
# - POSTGRES_PASSWORD: Change from default
# - REDIS_PASSWORD: Change from default

# 4. Start the application (production)
docker compose up -d --build
```

Access the app at [http://localhost:8080](http://localhost:8080)

**First-time setup:** The application will automatically create a default admin user. Check the Docker logs for credentials:
```bash
docker compose logs backend | grep "Default admin"
```

## 🏗️ Technology Stack

- **Backend**: Rust with Actix-web
- **Frontend**: Vue.js 3 with TypeScript and Tailwind CSS
- **Database**: PostgreSQL with Redis caching
- **Real-time**: WebSockets and Server-Sent Events
- **Authentication**: JWT with optional TOTP MFA
- **Integrations**: Microsoft Graph API for Intune/Entra ID

## 📦 Project Structure

```
Nosdesk/
├── backend/           # Rust API server
│   ├── src/
│   ├── migrations/    # Database schema
│   └── public/        # Built frontend (production)
├── frontend/          # Vue.js frontend
│   ├── src/
│   └── public/
└── compose.yaml # Docker orchestration
```

## 🔧 Configuration

All configuration is managed through `docker.env` which includes:

- **Security**: JWT_SECRET, MFA_ENCRYPTION_KEY (generate secure keys for production)
- **Database**: PostgreSQL connection settings (default: `helpdesk` database)
- **Redis**: Cache and session storage configuration
- **Microsoft Integration** (Optional): Entra ID SSO and Intune device management
- **SMTP**: Email notifications for tickets and alerts

For local development outside Docker, the backend will auto-generate `.env` from the example file.

## 📚 Documentation

- **API Documentation**: Import `api-insomnia.json` into Insomnia

## 🤝 Contributing

### Development Environment

For active development with hot reloading:

```bash
# Start development environment
docker compose -f compose.yaml -f compose.dev.yaml up -d --build

# Access services:
# - Application: http://localhost:8080
# - Backend logs: docker compose -f compose.yaml -f compose.dev.yaml logs -f backend
# - Frontend logs: docker compose -f compose.yaml -f compose.dev.yaml logs -f frontend-watch
```

**Development stack includes:**
- **postgres**: PostgreSQL database with persistent storage
- **redis**: Redis for caching and real-time features
- **backend**: Rust API with hot reload and automatic migrations (dev mode)
- **frontend-watch**: Vue.js dev server with Hot Module Replacement (HMR)

**Database migrations:**
```bash
# Apply migrations
docker compose -f compose.yaml -f compose.dev.yaml exec backend diesel migration run

# Regenerate schema
docker compose -f compose.yaml -f compose.dev.yaml exec backend sh -c 'diesel print-schema > src/schema.rs'
```

For more detailed development instructions, see [CLAUDE.md](./CLAUDE.md).

## 📄 License

Licensed under the terms in the LICENSE file.
