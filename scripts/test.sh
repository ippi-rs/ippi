#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_FEATURES="frontend-embedded"
TEST_PROFILE="debug"
CARGO_TEST="cargo test"
CARGO_NEXTEST="cargo nextest run"

# Print header
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}       IPPI Test Suite Runner        ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if nextest is installed
check_nextest() {
    if ! command -v cargo-nextest &> /dev/null; then
        echo -e "${YELLOW}⚠️  cargo-nextest not found. Install for better test output:${NC}"
        echo "cargo install cargo-nextest"
        return 1
    fi
    return 0
}

# Run unit tests
run_unit_tests() {
    echo -e "${BLUE}🧪 Running unit tests...${NC}"
    
    if check_nextest; then
        $CARGO_NEXTEST --features "$TEST_FEATURES" --lib
    else
        $CARGO_TEST --features "$TEST_FEATURES" --lib
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Unit tests passed${NC}"
        return 0
    else
        echo -e "${RED}❌ Unit tests failed${NC}"
        return 1
    fi
}

# Run integration tests
run_integration_tests() {
    echo -e "${BLUE}🔗 Running integration tests...${NC}"
    
    if check_nextest; then
        $CARGO_NEXTEST --features "$TEST_FEATURES" --test "*"
    else
        $CARGO_TEST --features "$TEST_FEATURES" --test "*"
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Integration tests passed${NC}"
        return 0
    else
        echo -e "${RED}❌ Integration tests failed${NC}"
        return 1
    fi
}

# Run doc tests
run_doc_tests() {
    echo -e "${BLUE}📚 Running documentation tests...${NC}"
    
    $CARGO_TEST --features "$TEST_FEATURES" --doc
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Documentation tests passed${NC}"
        return 0
    else
        echo -e "${RED}❌ Documentation tests failed${NC}"
        return 1
    fi
}

# Run frontend tests
run_frontend_tests() {
    echo -e "${BLUE}🎨 Running frontend tests...${NC}"
    
    if [ ! -d "frontend" ]; then
        echo -e "${YELLOW}⚠️  Frontend directory not found, skipping${NC}"
        return 0
    fi
    
    cd frontend
    
    # Check if test script exists
    if ! grep -q '"test"' package.json; then
        echo -e "${YELLOW}⚠️  No test script in package.json, skipping${NC}"
        cd ..
        return 0
    fi
    
    npm test
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Frontend tests passed${NC}"
        cd ..
        return 0
    else
        echo -e "${RED}❌ Frontend tests failed${NC}"
        cd ..
        return 1
    fi
}

# Run formatting check
run_format_check() {
    echo -e "${BLUE}🎨 Checking code formatting...${NC}"
    
    cargo fmt --check
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Code formatting OK${NC}"
        return 0
    else
        echo -e "${RED}❌ Code formatting issues found${NC}"
        echo -e "${YELLOW}Run 'cargo fmt' to fix${NC}"
        return 1
    fi
}

# Run linter
run_linter() {
    echo -e "${BLUE}🔍 Running linter...${NC}"
    
    cargo clippy --features "$TEST_FEATURES" -- -D warnings
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Linter passed${NC}"
        return 0
    else
        echo -e "${RED}❌ Linter found issues${NC}"
        return 1
    fi
}

# Run security audit
run_security_audit() {
    echo -e "${BLUE}🔒 Running security audit...${NC}"
    
    if ! command -v cargo-audit &> /dev/null; then
        echo -e "${YELLOW}⚠️  cargo-audit not installed. Install with:${NC}"
        echo "cargo install cargo-audit"
        return 0
    fi
    
    cargo audit
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ No security vulnerabilities found${NC}"
        return 0
    else
        echo -e "${RED}❌ Security vulnerabilities found${NC}"
        return 1
    fi
}

# Run benchmarks
run_benchmarks() {
    echo -e "${BLUE}📊 Running benchmarks...${NC}"
    
    # Check if there are any benchmarks
    if [ ! -d "benches" ] && [ ! -f "benches/"*.rs 2>/dev/null ]; then
        echo -e "${YELLOW}⚠️  No benchmarks found, skipping${NC}"
        return 0
    fi
    
    cargo bench --features "$TEST_FEATURES"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Benchmarks completed${NC}"
        return 0
    else
        echo -e "${RED}❌ Benchmarks failed${NC}"
        return 1
    fi
}

# Generate test coverage report
run_coverage() {
    echo -e "${BLUE}📈 Generating test coverage...${NC}"
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}⚠️  cargo-tarpaulin not installed. Install with:${NC}"
        echo "cargo install cargo-tarpaulin"
        return 0
    fi
    
    cargo tarpaulin --features "$TEST_FEATURES" --out Html
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Coverage report generated: tarpaulin-report.html${NC}"
        return 0
    else
        echo -e "${RED}❌ Coverage generation failed${NC}"
        return 1
    fi
}

# Run all tests
run_all_tests() {
    local failed=0
    
    echo -e "${BLUE}🚀 Running complete test suite...${NC}"
    echo ""
    
    # Format check
    run_format_check
    if [ $? -ne 0 ]; then failed=1; fi
    echo ""
    
    # Linter
    run_linter
    if [ $? -ne 0 ]; then failed=1; fi
    echo ""
    
    # Security audit
    run_security_audit
    if [ $? -ne 0 ]; then failed=1; fi
    echo ""
    
    # Unit tests
    run_unit_tests
    if [ $? -ne 0 ]; then failed=1; fi
    echo ""
    
    # Integration tests
    run_integration_tests
    if [ $? -ne 0 ]; then failed=1; fi
    echo ""
    
    # Doc tests
    run_doc_tests
    if [ $? -ne 0 ]; then failed=1; fi
    echo ""
    
    # Frontend tests
    run_frontend_tests
    if [ $? -ne 0 ]; then failed=1; fi
    echo ""
    
    # Benchmarks (optional)
    if [ "${1:-}" = "--with-benches" ]; then
        run_benchmarks
        if [ $? -ne 0 ]; then failed=1; fi
        echo ""
    fi
    
    # Coverage (optional)
    if [ "${2:-}" = "--with-coverage" ]; then
        run_coverage
        if [ $? -ne 0 ]; then failed=1; fi
        echo ""
    fi
    
    # Summary
    echo -e "${BLUE}========================================${NC}"
    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}✅ All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}❌ Some tests failed${NC}"
        return 1
    fi
}

# Show help
show_help() {
    echo -e "${BLUE}Usage: $0 [OPTION]${NC}"
    echo ""
    echo -e "${BLUE}Options:${NC}"
    echo "  unit              Run only unit tests"
    echo "  integration       Run only integration tests"
    echo "  doc               Run only documentation tests"
    echo "  frontend          Run only frontend tests"
    echo "  format            Check code formatting"
    echo "  lint              Run linter"
    echo "  security          Run security audit"
    echo "  bench             Run benchmarks"
    echo "  coverage          Generate test coverage"
    echo "  all               Run all tests (default)"
    echo "  --with-benches    Include benchmarks in 'all'"
    echo "  --with-coverage   Include coverage in 'all'"
    echo "  help              Show this help message"
    echo ""
    echo -e "${BLUE}Examples:${NC}"
    echo "  $0 unit           # Run unit tests only"
    echo "  $0 all            # Run all tests"
    echo "  $0 all --with-benches --with-coverage"
}

# Main execution
main() {
    case "${1:-}" in
        "unit")
            run_unit_tests
            ;;
        "integration")
            run_integration_tests
            ;;
        "doc")
            run_doc_tests
            ;;
        "frontend")
            run_frontend_tests
            ;;
        "format")
            run_format_check
            ;;
        "lint")
            run_linter
            ;;
        "security")
            run_security_audit
            ;;
        "bench")
            run_benchmarks
            ;;
        "coverage")
            run_coverage
            ;;
        "all")
            run_all_tests "$2" "$3"
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        "")
            run_all_tests
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"