# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nosdesk is a modern helpdesk and IT management system with real-time collaborative editing capabilities. It uses a Rust backend (Actix-web) serving a Vue.js 3 frontend, with PostgreSQL and Redis for data persistence and real-time features.

## Development Commands

### Docker Development (Recommended)

**Development mode with hot reloading:**
```bash
docker compose --profile dev up --build
```
- Backend: Rust with live reload on http://localhost:8080
- Frontend: Vue dev server with hot reload, builds to `backend/public/`
- Access app at http://localhost:8080

**Production mode:**
```bash
docker compose --profile prod up --build
```
- Backend serves the built frontend and API
- All services start automatically

### Backend (Rust)

**Local development (requires PostgreSQL + Redis running):**
```bash
cd backend
cargo run
```

**Run tests:**
```bash
cd backend
cargo test
```

**Database migrations:**
```bash
cd backend
diesel migration run
```

**Regenerate Diesel schema (Docker dev environment):**
```bash
docker compose exec backend-dev sh -c 'RUST_LOG=off diesel print-schema 2>/dev/null' > backend/src/schema.rs
```
- Run this after creating or modifying migrations
- Suppresses logging output to avoid corrupting the schema file
- Must be run from the project root directory
- The schema file (`src/schema.rs`) should NEVER be manually edited

**Build for production:**
```bash
cd backend
cargo build --release
```

### Frontend (Vue.js)

**Development server:**
```bash
cd frontend
npm install
npm run dev
```

**Build for production:**
```bash
cd frontend
npm run build
```
Output goes to `frontend/dist/` (symlinked to `backend/public/`)

**Type checking:**
```bash
cd frontend
npm run type-check
```

**Build with watch mode (for Docker dev):**
```bash
cd frontend
npm run dev:unified
```

## Architecture Overview

### Backend Structure

The Rust backend follows a layered architecture:

- **`src/main.rs`**: Application entry point, configures Actix-web server, middleware (CORS, JWT auth, rate limiting), and route registration. Serves the SPA frontend for non-API routes.

- **`src/handlers/`**: HTTP request handlers organized by domain:
  - `tickets.rs`, `projects.rs`, `users.rs`, `devices.rs` - CRUD operations
  - `auth.rs`, `auth_providers.rs` - Authentication and OAuth (Microsoft Entra ID)
  - `collaboration.rs` - WebSocket handlers for real-time collaborative editing using Yjs CRDTs
  - `sse.rs` - Server-sent events for live updates across clients
  - `microsoft_graph.rs`, `msgraph_integration.rs` - Microsoft Intune/Graph API integration
  - `files.rs` - File upload/download with thumbnail generation
  - `documentation.rs` - Internal documentation/knowledge base

- **`src/repository/`**: Data access layer using Diesel ORM. Each file corresponds to a database table and provides CRUD operations. All database queries are isolated here.

- **`src/models.rs`**: Diesel models and API DTOs. Contains both database schema structs and serializable data transfer objects.

- **`src/schema.rs`**: Auto-generated Diesel schema definitions from database migrations. **NEVER edit manually** - regenerate using the `diesel print-schema` command documented above.

- **`src/utils/`**: Shared utilities:
  - `jwt.rs` - JWT token generation/validation
  - `mfa.rs` - TOTP-based multi-factor authentication
  - `storage.rs` - File storage abstraction (local filesystem or AWS S3)
  - `auth.rs` - OAuth helpers
  - `sse.rs` - Server-sent events broadcasting utilities
  - `validation.rs` - Input validation helpers

- **`src/db.rs`**: Database connection pool configuration using r2d2.

- **`migrations/`**: Diesel database migrations. The main schema is in `2025-06-03-015044_initial_schema/up.sql`.

### Frontend Structure

Vue.js 3 SPA with TypeScript, Pinia for state management, and Tailwind CSS:

- **`src/views/`**: Page-level components (one per route). Handle layout and orchestrate multiple components.

- **`src/components/`**: Reusable UI components. Organized by feature/domain.

- **`src/services/`**: API client modules:
  - `apiConfig.ts` - Axios configuration with auth interceptors
  - `*Service.ts` - API calls for each domain (tickets, users, devices, etc.)
  - `sseService.ts` - Server-sent events client for real-time updates

- **`src/stores/`**: Pinia stores for global state management (auth, tickets, users, notifications, etc.)

- **`src/composables/`**: Vue 3 composables for shared reactive logic.

- **`src/router/index.ts`**: Vue Router configuration. Handles authentication guards, admin role checks, and onboarding flow.

- **`src/editor/`**: ProseMirror + Yjs integration for collaborative markdown editing.

- **`src/types/`**: TypeScript type definitions.

- **`src/utils/`**: Frontend utility functions.

### Real-time Collaboration Architecture

The system uses Yjs CRDTs for conflict-free collaborative editing:

1. **WebSocket Connection**: Clients connect via WebSocket to `collaboration.rs` handlers
2. **Document Sync**: Yjs state vectors and updates are exchanged using binary protocol
3. **Awareness**: User presence (cursor positions, selections) shared via awareness protocol
4. **Persistence**: Document state periodically saved to `article_content` table in PostgreSQL
5. **SSE Broadcasting**: Changes broadcast to all connected clients via server-sent events

Backend maintains in-memory Yjs documents with automatic persistence and cleanup when rooms are empty.

### Authentication Flow

1. **Local Auth**: Username/password with bcrypt hashing, JWT tokens, optional TOTP MFA
2. **Microsoft OAuth**: Entra ID integration with PKCE flow, stores user mappings in `user_auth_identities`
3. **JWT Middleware**: All protected routes validated via `validator()` function in `main.rs`
4. **Session Management**: Active sessions tracked in `active_sessions` table with device fingerprinting
5. **Security Events**: Login attempts, MFA changes, and security events logged in `security_events`

### File Storage

Abstracted storage layer supports multiple backends:

- **Local Filesystem**: Default for development (stored in `backend/uploads/`)
- **AWS S3**: Production storage configured via environment variables
- **Public Assets**: Served from `backend/public/` for frontend static assets

Images automatically generate thumbnails on upload (handled in `utils/image.rs`).

### Microsoft Integration

- **Entra ID (Azure AD)**: OAuth SSO for user authentication
- **Intune**: Device and user data import via Microsoft Graph API
- **Sync History**: Tracks import operations in `sync_history` table
- **Auto-mapping**: Users and devices automatically linked to tickets/profiles

## Key Patterns

- **Repository Pattern**: All database access goes through repository layer, making testing and refactoring easier
- **JWT Authentication**: Stateless auth with refresh token support, validated by middleware
- **SPA Routing**: Backend serves `index.html` for all non-API routes, frontend handles routing
- **Error Handling**: Consistent JSON error responses across all endpoints
- **Rate Limiting**: Redis-backed rate limiting on authentication endpoints
- **Binary Protocol**: Yjs updates use efficient binary encoding for real-time collaboration

## Environment Configuration

- **Docker**: Uses `docker.env` file for all services
- **Local Backend**: Auto-generates `.env` from `backend/env.example`
- **Local Frontend**: Uses Vite environment variables

Key environment variables:
- `DATABASE_URL` - PostgreSQL connection
- `REDIS_URL` - Redis connection  
- `JWT_SECRET` - JWT signing key
- `MFA_ENCRYPTION_KEY` - AES-256 key for MFA secret encryption
- `STORAGE_TYPE` - "local" or "s3"
- Microsoft OAuth: `MS_CLIENT_ID`, `MS_CLIENT_SECRET`, `MS_TENANT_ID`

## Testing

- Backend tests use `cargo test` with mockall for mocking
- Database tests require a test database (configured in `diesel.toml`)
- Integration tests in `backend/src/handlers/` test modules

## Binary Tools

**Ticket Import Utility:**
```bash
cd backend
cargo run --bin import_tickets
```
Imports tickets from external data sources (see `src/bin/import_tickets.rs`).
