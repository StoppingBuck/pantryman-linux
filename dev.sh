#!/bin/bash
set -e
GREEN='\033[0;32m'; CYAN='\033[0;36m'; RED='\033[0;31m'; NC='\033[0m'
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JANUS_ENGINE_PATH="$(cd "$PROJECT_ROOT/../janus-engine" 2>/dev/null && pwd || echo "")"

show_help() {
    echo "Usage: $0 <command>"
    echo "  run     - Build and run Pantryman (GTK4 desktop)"
    echo "  compile - Compile without running"
    echo "  test    - Run all tests (requires display)"
    echo "  test-headless - Run tests via xvfb-run"
    echo "  check   - cargo check"
    echo "  clean   - Clean build artifacts"
    echo "  help    - Show this help"
}

check_engine() {
    if [ -z "$JANUS_ENGINE_PATH" ]; then
        echo -e "${RED}❌ janus-engine not found at ../janus-engine${NC}"
        echo "   Clone it: git clone <url> $PROJECT_ROOT/../janus-engine"
        exit 1
    fi
}

case "${1:-help}" in
    "run")
        check_engine
        echo -e "${CYAN}🍳 Building and running Pantryman (GTK4)...${NC}"
        cd "$PROJECT_ROOT"
        cargo run
        ;;
    "compile")
        check_engine
        echo -e "${CYAN}🔨 Compiling Pantryman...${NC}"
        cd "$PROJECT_ROOT"
        cargo build
        echo -e "${GREEN}✅ Compile complete${NC}"
        ;;
    "test")
        check_engine
        echo -e "${CYAN}🧪 Running tests...${NC}"
        cd "$PROJECT_ROOT"
        cargo test
        echo -e "${GREEN}✅ Tests complete${NC}"
        ;;
    "test-headless")
        check_engine
        echo -e "${CYAN}🧪 Running tests (headless)...${NC}"
        cd "$PROJECT_ROOT"
        xvfb-run --auto-servernum cargo test
        echo -e "${GREEN}✅ Headless tests complete${NC}"
        ;;
    "check")
        check_engine
        echo -e "${CYAN}🔍 Running cargo check...${NC}"
        cd "$PROJECT_ROOT"
        cargo check
        echo -e "${GREEN}✅ Check complete${NC}"
        ;;
    "clean")
        echo -e "${CYAN}🧹 Cleaning build artifacts...${NC}"
        cd "$PROJECT_ROOT"
        cargo clean
        echo -e "${GREEN}✅ Clean complete${NC}"
        ;;
    "help"|"--help"|"-h") show_help ;;
    *) echo -e "${RED}❌ Unknown command: $1${NC}"; show_help; exit 1 ;;
esac
