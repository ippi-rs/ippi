#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print header
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}       KvmDust Code Formatter           ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if rustfmt is installed
check_rustfmt() {
    if ! command -v rustfmt &> /dev/null; then
        echo -e "${YELLOW}⚠️  rustfmt not found. Installing...${NC}"
        rustup component add rustfmt
        if [ $? -ne 0 ]; then
            echo -e "${RED}❌ Failed to install rustfmt${NC}"
            exit 1
        fi
        echo -e "${GREEN}✅ rustfmt installed successfully${NC}"
    fi
}

# Format Rust code
format_rust() {
    echo -e "${BLUE}🦀 Formatting Rust code...${NC}"
    
    check_rustfmt
    
    # Format all Rust files
    cargo fmt
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Rust code formatted successfully${NC}"
        return 0
    else
        echo -e "${RED}❌ Failed to format Rust code${NC}"
        return 1
    fi
}

# Format frontend code
format_frontend() {
    echo -e "${BLUE}🎨 Formatting frontend code...${NC}"
    
    if [ ! -d "frontend" ]; then
        echo -e "${YELLOW}⚠️  Frontend directory not found, skipping${NC}"
        return 0
    fi
    
    cd frontend
    
    # Check if prettier is available
    if ! npx prettier --version &> /dev/null; then
        echo -e "${YELLOW}⚠️  prettier not found, installing...${NC}"
        npm install --save-dev prettier
    fi
    
    # Format JavaScript/TypeScript files
    npx prettier --write "src/**/*.{js,ts,svelte}" "*.{js,ts,json}" 2>/dev/null || true
    
    # Format package.json
    if [ -f "package.json" ]; then
        npx prettier --write package.json 2>/dev/null || true
    fi
    
    cd ..
    
    echo -e "${GREEN}✅ Frontend code formatted${NC}"
    return 0
}

# Format configuration files
format_config() {
    echo -e "${BLUE}⚙️  Formatting configuration files...${NC}"
    
    # Format YAML files
    if command -v yq &> /dev/null; then
        for file in config/*.yaml; do
            if [ -f "$file" ]; then
                yq eval -P "$file" > "$file.tmp" && mv "$file.tmp" "$file"
            fi
        done
    fi
    
    # Format TOML files (Cargo.toml is formatted by cargo fmt)
    echo -e "${GREEN}✅ Configuration files formatted${NC}"
    return 0
}

# Check formatting without applying
check_format() {
    echo -e "${BLUE}🔍 Checking code formatting...${NC}"
    
    check_rustfmt
    
    # Check Rust formatting
    cargo fmt --check
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Rust code is properly formatted${NC}"
    else
        echo -e "${RED}❌ Rust code formatting issues found${NC}"
        echo -e "${YELLOW}Run './scripts/fmt.sh' to fix${NC}"
        return 1
    fi
    
    # Check frontend formatting if directory exists
    if [ -d "frontend" ]; then
        cd frontend
        
        if npx prettier --check "src/**/*.{js,ts,svelte}" "*.{js,ts,json}" 2>/dev/null; then
            echo -e "${GREEN}✅ Frontend code is properly formatted${NC}"
        else
            echo -e "${RED}❌ Frontend code formatting issues found${NC}"
            cd ..
            return 1
        fi
        
        cd ..
    fi
    
    return 0
}

# Show formatting statistics
show_stats() {
    echo -e "${BLUE}📊 Formatting statistics:${NC}"
    
    # Count Rust files
    rust_files=$(find . -name "*.rs" -not -path "./target/*" | wc -l)
    echo "  Rust files: $rust_files"
    
    # Count frontend files
    if [ -d "frontend" ]; then
        frontend_files=$(find frontend -name "*.js" -o -name "*.ts" -o -name "*.svelte" | wc -l)
        echo "  Frontend files: $frontend_files"
    fi
    
    # Count configuration files
    config_files=$(find config -name "*.yaml" -o -name "*.toml" -o -name "*.json" 2>/dev/null | wc -l)
    echo "  Config files: $config_files"
}

# Main execution
main() {
    case "${1:-}" in
        "check")
            check_format
            ;;
        "rust")
            format_rust
            ;;
        "frontend")
            format_frontend
            ;;
        "config")
            format_config
            ;;
        "stats")
            show_stats
            ;;
        "help"|"-h"|"--help")
            echo -e "${BLUE}Usage: $0 [OPTION]${NC}"
            echo ""
            echo -e "${BLUE}Options:${NC}"
            echo "  check     Check formatting without applying"
            echo "  rust      Format only Rust code"
            echo "  frontend  Format only frontend code"
            echo "  config    Format only configuration files"
            echo "  stats     Show formatting statistics"
            echo "  help      Show this help message"
            echo ""
            echo -e "${BLUE}Default: Format all code${NC}"
            ;;
        "")
            # Format everything
            format_rust
            echo ""
            format_frontend
            echo ""
            format_config
            echo ""
            show_stats
            echo ""
            echo -e "${GREEN}✅ All code formatted successfully!${NC}"
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"