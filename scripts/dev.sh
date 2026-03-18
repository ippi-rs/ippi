#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
FEATURES="frontend-embedded dev"
CARGO_WATCH="cargo watch -x"
DEV_PORT=8080
API_PORT=8081

# Print header
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}    KvmDust Development Environment     ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if cargo-watch is installed
check_cargo_watch() {
    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${YELLOW}⚠️  cargo-watch not found. Installing...${NC}"
        cargo install cargo-watch
        if [ $? -ne 0 ]; then
            echo -e "${RED}❌ Failed to install cargo-watch${NC}"
            exit 1
        fi
        echo -e "${GREEN}✅ cargo-watch installed successfully${NC}"
    fi
}

# Start frontend development server
start_frontend_dev() {
    echo -e "${BLUE}🎨 Starting frontend development server...${NC}"
    
    if [ ! -d "frontend" ]; then
        echo -e "${YELLOW}⚠️  Frontend directory not found${NC}"
        return 1
    fi
    
    # Check if frontend dev server is already running
    if lsof -ti:5173 >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Frontend dev server already running on port 5173${NC}"
        return 0
    fi
    
    # Start frontend in background
    cd frontend
    npm run dev &
    FRONTEND_PID=$!
    cd ..
    
    echo -e "${GREEN}✅ Frontend dev server started (PID: $FRONTEND_PID)${NC}"
    echo -e "${BLUE}   Access: http://localhost:5173${NC}"
}

# Start backend development server
start_backend_dev() {
    echo -e "${BLUE}🦀 Starting backend development server...${NC}"
    
    # Kill any existing process on dev port
    if lsof -ti:$DEV_PORT >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Killing existing process on port $DEV_PORT${NC}"
        lsof -ti:$DEV_PORT | xargs kill -9 2>/dev/null || true
    fi
    
    # Build and run with cargo watch
    echo -e "${BLUE}🔨 Building with features: $FEATURES${NC}"
    echo -e "${BLUE}👀 Watching for changes...${NC}"
    
    $CARGO_WATCH "run --features $FEATURES -- --port $DEV_PORT --dev"
}

# Setup development environment
setup_dev_env() {
    echo -e "${BLUE}🔧 Setting up development environment...${NC}"
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        echo -e "${RED}❌ Rust not found. Please install Rust:${NC}"
        echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    echo -e "${GREEN}✅ Rust installed${NC}"
    
    # Check Node.js for frontend
    if ! command -v node &> /dev/null; then
        echo -e "${YELLOW}⚠️  Node.js not found. Frontend development will be limited.${NC}"
    else
        echo -e "${GREEN}✅ Node.js installed (v$(node --version))${NC}"
    fi
    
    # Check npm
    if ! command -v npm &> /dev/null; then
        echo -e "${YELLOW}⚠️  npm not found. Frontend development will be limited.${NC}"
    else
        echo -e "${GREEN}✅ npm installed (v$(npm --version))${NC}"
    fi
    
    check_cargo_watch
    
    # Install frontend dependencies if needed
    if [ -d "frontend" ] && [ ! -d "frontend/node_modules" ]; then
        echo -e "${YELLOW}⚠️  Frontend dependencies not found. Installing...${NC}"
        cd frontend
        npm install
        if [ $? -ne 0 ]; then
            echo -e "${RED}❌ Failed to install frontend dependencies${NC}"
            cd ..
            return 1
        fi
        cd ..
        echo -e "${GREEN}✅ Frontend dependencies installed${NC}"
    fi
}

# Show development URLs
show_urls() {
    echo ""
    echo -e "${BLUE}🌐 Development URLs:${NC}"
    echo -e "  Frontend (Vite):  ${GREEN}http://localhost:5173${NC}"
    echo -e "  Backend (Axum):   ${GREEN}http://localhost:$DEV_PORT${NC}"
    echo -e "  API:              ${GREEN}http://localhost:$DEV_PORT/api${NC}"
    echo -e "  Health:           ${GREEN}http://localhost:$DEV_PORT/api/health${NC}"
    echo ""
    echo -e "${BLUE}📁 Project Structure:${NC}"
    echo "  src/           - Rust source code"
    echo "  frontend/      - Svelte frontend"
    echo "  config/        - Configuration files"
    echo "  scripts/       - Development scripts"
    echo ""
    echo -e "${BLUE}🔧 Useful Commands:${NC}"
    echo "  cargo test     - Run tests"
    echo "  cargo fmt      - Format code"
    echo "  cargo clippy   - Run linter"
    echo "  ./scripts/test.sh - Run all tests"
}

# Cleanup function
cleanup() {
    echo -e "${YELLOW}🛑 Cleaning up...${NC}"
    
    # Kill frontend dev server
    if [ ! -z "$FRONTEND_PID" ]; then
        echo -e "${YELLOW}⚠️  Killing frontend dev server (PID: $FRONTEND_PID)${NC}"
        kill $FRONTEND_PID 2>/dev/null || true
    fi
    
    # Kill backend
    if lsof -ti:$DEV_PORT >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Killing backend on port $DEV_PORT${NC}"
        lsof -ti:$DEV_PORT | xargs kill -9 2>/dev/null || true
    fi
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Main execution
main() {
    # Trap Ctrl+C for cleanup
    trap cleanup INT TERM EXIT
    
    echo -e "${BLUE}🚀 Starting KvmDust development environment${NC}"
    echo ""
    
    # Setup environment
    setup_dev_env
    echo ""
    
    # Show URLs
    show_urls
    
    # Start frontend dev server
    start_frontend_dev
    echo ""
    
    # Wait a bit for frontend to start
    sleep 2
    
    # Start backend dev server
    echo -e "${BLUE}🔄 Starting backend with hot reload...${NC}"
    echo -e "${YELLOW}📝 Press Ctrl+C to stop${NC}"
    echo ""
    
    start_backend_dev
}

# Check if we should run in different mode
case "${1:-}" in
    "frontend-only")
        setup_dev_env
        start_frontend_dev
        wait
        ;;
    "backend-only")
        setup_dev_env
        start_backend_dev
        ;;
    "setup")
        setup_dev_env
        ;;
    *)
        main
        ;;
esac