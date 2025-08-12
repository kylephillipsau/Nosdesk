# Nosdesk

A modern helpdesk and IT management system built with Rust and Vue.js, designed for efficient ticket management and IT operations.

> This project is still in active development. The software is provided as-is with absolutely no warranty. Use at your own risk!

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

### Docker Development (Recommended)
Run the full stack in Docker, with the backend serving the frontend and all dependencies:

#### **Development with Hot Reloading**
For live code and frontend hot reloading:
```bash
docker compose --profile dev up --build
```
- `backend-dev`: Rust backend with live code reload
- `frontend-watch`: Vue dev server with hot reload
- Access the app at [http://localhost:8080](http://localhost:8080)

#### **Production Environment**
For production-like deployment:
```bash
docker compose --profile prod up --build
```
- The Rust backend serves the Vue.js frontend and API endpoints
- All services (Postgres, Redis, backend) are started automatically

---

## 🐳 Docker Compose Services

- **postgres**: PostgreSQL database with persistent storage
- **redis**: Redis cache for real-time features and rate limiting
- **backend**: Rust API server (production, serves built frontend)
- **backend-dev**: Rust API server (development, with live reload and auto-migrations)
- **frontend-watch**: Vue.js dev server for hot reloading

---

## 📋 Prerequisites

- [Docker](https://www.docker.com/) and Docker Compose

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

Environment variables are managed automatically:
- **Docker**: `docker.env` file
- **Native**: Auto-generated `backend/.env`

Key integrations:
- Microsoft Entra ID for SSO
- Microsoft Intune for device management
- Redis for real-time features and caching

## 📚 Documentation

- **API Documentation**: Import `api-insomnia.json` into Insomnia

## 📄 License

Licensed under the terms in the LICENSE file.