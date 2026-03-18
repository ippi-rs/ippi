#!/bin/bash
set -e

echo "=== ippi Integration Test ==="
echo ""

# Build the project
echo "Building ippi..."
cargo build --no-default-features

echo ""
echo "=== Module Tests ==="
echo ""

# Test configuration module
echo "1. Testing configuration module..."
cargo test --lib config_tests -- --nocapture

# Test web module
echo ""
echo "2. Testing web module..."
cargo test --lib web_tests -- --nocapture

# Test utils module
echo ""
echo "3. Testing utils module..."
cargo test --lib utils_tests -- --nocapture

# Test integration
echo ""
echo "4. Testing module integration..."
cargo test --lib test_integration -- --nocapture

echo ""
echo "=== Quick Start Demo ==="
echo ""

# Create a simple demo
echo "Creating demo configuration..."
cat > /tmp/ippi-demo.toml << 'EOF'
[web]
host = "127.0.0.1"
port = 8081
cors_origins = ["*"]

[kvm]
enabled = false
device_path = "/dev/kvm"
memory_mb = 1024

[p2p]
enabled = false
bootstrap_nodes = []
listen_port = 4001
protocol_version = "/ippi/0.1.0"

[webrtc]
enabled = false
stun_servers = ["stun:stun.l.google.com:19302"]
video_bitrate = 2000
audio_bitrate = 128

[cloud_init]
enabled = true
data_source = "NoCloud"
metadata_url = "http://169.254.169.254/latest/meta-data"
userdata_url = "http://169.254.169.254/latest/user-data"
EOF

echo "Demo configuration created at /tmp/ippi-demo.toml"
echo ""
echo "To run ippi with this configuration:"
echo "  ./target/debug/ippi --config /tmp/ippi-demo.toml --verbose"
echo ""
echo "Then open http://127.0.0.1:8081 in your browser"

echo ""
echo "=== Build Information ==="
echo ""
cargo version
rustc --version
echo "Rust edition: 2024"
echo "Target: $(rustc -vV | grep host | cut -d' ' -f2)"

echo ""
echo "=== Docker Build Test ==="
echo ""
echo "To build Docker image:"
echo "  docker build -t ippi:test ."
echo ""
echo "To run with Docker:"
echo "  docker run -p 8080:8080 -v /tmp/ippi-demo.toml:/etc/ippi/ippi.toml ippi:test"

echo ""
echo "=== Raspberry Pi Zero Cross-Compilation ==="
echo ""
echo "For Raspberry Pi Zero (ARMv6):"
echo "  rustup target add arm-unknown-linux-gnueabihf"
echo "  cargo build --target arm-unknown-linux-gnueabihf --features pi-zero --no-default-features"
echo ""
echo "Binary will be at: target/arm-unknown-linux-gnueabihf/debug/ippi"

echo ""
echo "=== Frontend Build ==="
echo ""
echo "To build frontend:"
echo "  cd frontend && npm run build"
echo ""
echo "To run with embedded frontend:"
echo "  cargo build --features frontend-embedded"
echo ""
echo "Test completed successfully!"