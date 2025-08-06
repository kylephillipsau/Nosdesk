#!/bin/bash

# Native development setup script for Nosdesk
# This script sets up the development environment for running cargo run + npm run dev

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Nosdesk Native Development Setup${NC}"
echo ""

# Function to display usage
usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Available commands:"
    echo "  setup       - Start only database services (PostgreSQL + Redis)"
    echo "  env         - Show environment variables for native development"
    echo "  check       - Check if all dependencies are available"
    echo "  clean       - Stop database services"
    echo ""
    echo "After running 'setup', you can:"
    echo "  1. cd backend && cargo run"
    echo "  2. (new terminal) cd frontend && npm run dev"
}

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        echo -e "${RED}❌ Docker is not running. Please start Docker first.${NC}"
        exit 1
    fi
}

# Function to check native dependencies
check_dependencies() {
    echo -e "${BLUE}🔍 Checking native development dependencies...${NC}"
    
    # Check Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo not found. Please install Rust: https://rustup.rs/${NC}"
        exit 1
    else
        echo -e "${GREEN}✅ Rust/Cargo found: $(cargo --version)${NC}"
    fi
    
    # Check Node.js/npm
    if ! command -v npm &> /dev/null; then
        echo -e "${RED}❌ npm not found. Please install Node.js: https://nodejs.org/${NC}"
        exit 1
    else
        echo -e "${GREEN}✅ Node.js/npm found: $(node --version) / $(npm --version)${NC}"
    fi
    
    # Check if diesel CLI is installed
    if ! command -v diesel &> /dev/null; then
        echo -e "${YELLOW}⚠️  Diesel CLI not found. Installing...${NC}"
        cargo install diesel_cli --no-default-features --features postgres
    else
        echo -e "${GREEN}✅ Diesel CLI found${NC}"
    fi
    
    echo ""
}

# Function to setup database services only
setup_databases() {
    echo -e "${BLUE}🗃️  Starting database services...${NC}"
    
    # Start only postgres and redis
    docker compose up -d postgres redis
    
    # Wait for services to be healthy
    echo "Waiting for services to be ready..."
    docker compose exec postgres pg_isready -U nosdesk -d helpdesk
    docker compose exec redis redis-cli ping
    
    echo ""
    echo -e "${GREEN}✅ Database services ready!${NC}"
    echo -e "${YELLOW}📊 PostgreSQL: localhost:5432${NC}"
    echo -e "${YELLOW}🚀 Redis: localhost:6379${NC}"
    echo ""
}

# Function to show environment variables
show_env() {
    echo -e "${BLUE}📋 Environment variables for native development:${NC}"
    echo ""
    echo "Copy these to your shell or create a .env file in the backend directory:"
    echo ""
    cat << 'EOF'
export DATABASE_URL="postgres://nosdesk:nosdesk_password@localhost:5432/helpdesk"
export JWT_SECRET="CPfynq2V6hnpJxiGhMY1KMV1ZFRH5khVbqVuXS4f2mA="
export MFA_ENCRYPTION_KEY="c44b4a1d89f937f9c62a348f8edd2b5a1ed14af2e5ce476324e3a35c01bb93e5"
export REDIS_URL="redis://:nosdesk_redis_password@localhost:6379"
export FRONTEND_URL="http://localhost:5173"
export HOST="127.0.0.1"
export PORT="8080"
export ENVIRONMENT="development"
export RUST_LOG="debug"
EOF
    echo ""
    echo -e "${YELLOW}💡 Tip: Add these to your ~/.bashrc or ~/.zshrc for persistence${NC}"
    echo ""
}

# Function to create .env file for backend
create_backend_env() {
    echo -e "${BLUE}📝 Creating .env file for backend...${NC}"
    
    cat > backend/.env << 'EOF'
DATABASE_URL=postgres://nosdesk:nosdesk_password@localhost:5432/helpdesk
JWT_SECRET=CPfynq2V6hnpJxiGhMY1KMV1ZFRH5khVbqVuXS4f2mA=
MFA_ENCRYPTION_KEY=c44b4a1d89f937f9c62a348f8edd2b5a1ed14af2e5ce476324e3a35c01bb93e5
REDIS_URL=redis://:nosdesk_redis_password@localhost:6379
FRONTEND_URL=http://localhost:5173
HOST=127.0.0.1
PORT=8080
ENVIRONMENT=development
RUST_LOG=debug
EOF
    
    echo -e "${GREEN}✅ Created backend/.env${NC}"
}

# Function to clean up
clean() {
    echo -e "${BLUE}🧹 Stopping database services...${NC}"
    docker compose down postgres redis
    echo -e "${GREEN}✅ Database services stopped${NC}"
}

# Function to run full setup
full_setup() {
    check_dependencies
    setup_databases
    create_backend_env
    
    echo ""
    echo -e "${GREEN}🚀 Native development environment ready!${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Terminal 1: cd backend && cargo run"
    echo "2. Terminal 2: cd frontend && npm run dev"
    echo ""
    echo -e "${YELLOW}URLs:${NC}"
    echo "🌐 Frontend: http://localhost:5173"
    echo "🔧 Backend API: http://localhost:8080"
    echo "📊 Database: localhost:5432"
    echo "🚀 Redis: localhost:6379"
    echo ""
    echo "To stop databases: ./scripts/dev-native.sh clean"
}

# Main script logic
COMMAND=${1:-setup}

check_docker

case $COMMAND in
    setup)
        full_setup
        ;;
    env)
        show_env
        ;;
    check)
        check_dependencies
        ;;
    clean)
        clean
        ;;
    databases|db)
        setup_databases
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $COMMAND${NC}"
        echo ""
        usage
        exit 1
        ;;
esac 