#!/bin/sh
echo "=== Running IPPI P2P Example ==="
echo ""

echo "This example requires the 'p2p-full' feature"
echo ""

echo "1. Checking if cargo is available..."
if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ Cargo not found. Install Rust first."
    echo "   Visit: https://rustup.rs/"
    exit 1
fi

echo "✅ Cargo found: $(cargo --version)"
echo ""

echo "2. Available features:"
grep "^\[features\]" -A 20 Cargo.toml | grep -E "^\s*\w+" | sed 's/^[ \t]*//'

echo ""
echo "3. To run the P2P example:"
echo "   cargo run --example p2p_network --features \"p2p-full\""
echo ""
echo "4. Alternative: Build first, then run"
echo "   cargo build --example p2p_network --features \"p2p-full\""
echo "   ./target/debug/examples/p2p_network"
echo ""
echo "5. For minimal test (no P2P):"
echo "   cargo run --example p2p_network --no-default-features"
echo ""
echo "Note: The P2P feature requires libp2p, snow, and tun dependencies"
echo "      Make sure they're installed in your system if needed"