#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TARGET="arm-unknown-linux-gnueabihf"  # Raspberry Pi Zero W (ARMv6)
BINARY_NAME="ippi"
FEATURES="frontend-embedded"
PROFILE="release"
OUTPUT_DIR="dist-pi"

# Print header
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}    ippi Pi Zero Build Script       ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if cross is installed
check_cross() {
    if ! command -v cross &> /dev/null; then
        echo -e "${YELLOW}⚠️  cross not found. Installing...${NC}"
        cargo install cross
        if [ $? -ne 0 ]; then
            echo -e "${RED}❌ Failed to install cross${NC}"
            exit 1
        fi
        echo -e "${GREEN}✅ cross installed successfully${NC}"
    else
        echo -e "${GREEN}✅ cross is already installed${NC}"
    fi
}

# Check dependencies
check_dependencies() {
    echo -e "${BLUE}🔍 Checking dependencies...${NC}"
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        echo -e "${RED}❌ Rust not found. Please install Rust:${NC}"
        echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    echo -e "${GREEN}✅ Rust installed${NC}"
    
    # Check Cargo
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo not found${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Cargo installed${NC}"
    
    # Check target
    if ! rustup target list | grep -q "$TARGET (installed)"; then
        echo -e "${YELLOW}⚠️  Target $TARGET not installed. Installing...${NC}"
        rustup target add $TARGET
        if [ $? -ne 0 ]; then
            echo -e "${RED}❌ Failed to add target $TARGET${NC}"
            exit 1
        fi
        echo -e "${GREEN}✅ Target $TARGET installed${NC}"
    else
        echo -e "${GREEN}✅ Target $TARGET is installed${NC}"
    fi
    
    check_cross
}

# Build frontend
build_frontend() {
    echo -e "${BLUE}🏗️  Building frontend...${NC}"
    
    if [ ! -d "frontend" ]; then
        echo -e "${YELLOW}⚠️  Frontend directory not found, using placeholder${NC}"
        return 0
    fi
    
    cd frontend
    
    # Check if node_modules exists
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}⚠️  node_modules not found. Installing dependencies...${NC}"
        npm install
        if [ $? -ne 0 ]; then
            echo -e "${RED}❌ Failed to install frontend dependencies${NC}"
            cd ..
            return 1
        fi
    fi
    
    # Build frontend
    echo -e "${BLUE}📦 Building frontend with Vite...${NC}"
    npm run build
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Frontend build failed${NC}"
        cd ..
        return 1
    fi
    
    cd ..
    echo -e "${GREEN}✅ Frontend built successfully${NC}"
}

# Build Rust binary
build_rust() {
    echo -e "${BLUE}🦀 Building Rust binary for $TARGET...${NC}"
    
    # Clean previous build
    echo -e "${BLUE}🧹 Cleaning previous build...${NC}"
    cross clean --target $TARGET
    
    # Build with cross
    echo -e "${BLUE}🔨 Building with features: $FEATURES...${NC}"
    cross build \
        --target $TARGET \
        --$PROFILE \
        --features $FEATURES
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Rust binary built successfully${NC}"
}

# Create distribution package
create_package() {
    echo -e "${BLUE}📦 Creating distribution package...${NC}"
    
    # Create output directory
    mkdir -p $OUTPUT_DIR
    
    # Copy binary
    BINARY_PATH="target/$TARGET/$PROFILE/$BINARY_NAME"
    if [ ! -f "$BINARY_PATH" ]; then
        echo -e "${RED}❌ Binary not found at $BINARY_PATH${NC}"
        exit 1
    fi
    
    cp "$BINARY_PATH" "$OUTPUT_DIR/"
    
    # Strip binary (optional, makes it smaller)
    echo -e "${BLUE}✂️  Stripping binary...${NC}"
    arm-linux-gnueabihf-strip "$OUTPUT_DIR/$BINARY_NAME" 2>/dev/null || true
    
    # Copy configuration
    echo -e "${BLUE}📄 Copying configuration files...${NC}"
    cp -r config/*.yaml "$OUTPUT_DIR/" 2>/dev/null || true
    
    # Create README
    cat > "$OUTPUT_DIR/README.md" << EOF
# ippi for Raspberry Pi Zero

## Installation

1. Copy all files to your Raspberry Pi Zero
2. Make the binary executable:
   \`\`\`bash
   chmod +x ippi
   \`\`\`
3. Run ippi:
   \`\`\`bash
   ./ippi --config config.yaml
   \`\`\`

## Configuration

Edit \`config.yaml\` to match your setup.

## Web Interface

Access the web interface at: http://<pi-ip>:8080

## System Requirements

- Raspberry Pi Zero W or Zero 2 W
- Raspberry Pi OS Lite (32-bit)
- At least 512MB RAM
- 8GB+ SD card

## Support

For issues and questions, visit:
https://github.com/ippi/ippi
EOF
    
    # Create install script
    cat > "$OUTPUT_DIR/install.sh" << 'EOF'
#!/bin/bash
set -e

echo "Installing ippi..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root (use sudo)"
    exit 1
fi

# Create directories
mkdir -p /opt/ippi
mkdir -p /var/lib/ippi
mkdir -p /var/log/ippi

# Copy files
cp ippi /opt/ippi/
cp *.yaml /opt/ippi/ 2>/dev/null || true

# Create systemd service
cat > /etc/systemd/system/ippi.service << 'SERVICE'
[Unit]
Description=ippi - Lightweight P2P KVM-over-IP
After=network.target
Wants=network.target

[Service]
Type=simple
User=ippi
Group=ippi
WorkingDirectory=/opt/ippi
ExecStart=/opt/ippi/ippi --config /opt/ippi/config.yaml
Restart=always
RestartSec=3
StandardOutput=journal
StandardError=journal

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/ippi /var/log/ippi

[Install]
WantedBy=multi-user.target
SERVICE

# Create user
if ! id "ippi" &>/dev/null; then
    useradd -r -s /bin/false -d /opt/ippi ippi
fi

# Set permissions
chown -R ippi:ippi /opt/ippi /var/lib/ippi /var/log/ippi
chmod 755 /opt/ippi/ippi

# Enable and start service
systemctl daemon-reload
systemctl enable ippi
systemctl start ippi

echo "ippi installed successfully!"
echo "Check status: systemctl status ippi"
echo "View logs: journalctl -u ippi -f"
EOF
    
    chmod +x "$OUTPUT_DIR/install.sh"
    
    # Create tarball
    echo -e "${BLUE}📦 Creating tarball...${NC}"
    tar -czf "ippi-pi-zero.tar.gz" -C "$OUTPUT_DIR" .
    
    # Show package info
    echo -e "${GREEN}✅ Package created: ippi-pi-zero.tar.gz${NC}"
    echo ""
    echo -e "${BLUE}📊 Package Information:${NC}"
    echo "  Binary size: $(du -h "$OUTPUT_DIR/$BINARY_NAME" | cut -f1)"
    echo "  Package size: $(du -h "ippi-pi-zero.tar.gz" | cut -f1)"
    echo "  Target: $TARGET"
    echo "  Features: $FEATURES"
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting ippi build for Raspberry Pi Zero${NC}"
    echo ""
    
    # Check dependencies
    check_dependencies
    echo ""
    
    # Build frontend
    build_frontend
    echo ""
    
    # Build Rust
    build_rust
    echo ""
    
    # Create package
    create_package
    echo ""
    
    # Success message
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}    Build completed successfully!      ${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo -e "${BLUE}📦 Output files:${NC}"
    echo "  - dist-pi/ - Directory with all files"
    echo "  - ippi-pi-zero.tar.gz - Complete package"
    echo ""
    echo -e "${BLUE}🚀 To deploy to Raspberry Pi:${NC}"
    echo "  1. Copy ippi-pi-zero.tar.gz to your Pi"
    echo "  2. Extract: tar -xzf ippi-pi-zero.tar.gz"
    echo "  3. Run: ./install.sh (as root)"
    echo ""
    echo -e "${BLUE}🔧 Or run manually:${NC}"
    echo "  ./ippi --config config.yaml"
    echo ""
    echo -e "${YELLOW}⚠️  Note: First run may take longer as it generates certificates${NC}"
}

# Run main function
main "$@"