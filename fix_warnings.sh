#!/bin/sh
echo "=== Fixing warnings in IPPI ==="
echo ""

# 1. Remove unused vergen_gitcl::Emitter
echo "1. Removing unused vergen_gitcl::Emitter..."
if grep -q "use vergen_gitcl::Emitter;" build.rs; then
    sed -i '/use vergen_gitcl::Emitter;/d' build.rs
    echo "   ✅ Removed from build.rs"
else
    echo "   ⏭️  Already fixed"
fi

# 2. Remove unused extract::State
echo ""
echo "2. Removing unused extract::State..."
if grep -q "extract::State," src/web/mod.rs; then
    sed -i '/extract::State,/d' src/web/mod.rs
    echo "   ✅ Removed from src/web/mod.rs"
else
    echo "   ⏭️  Already fixed"
fi

# 3. Fix header import in assets.rs
echo ""
echo "3. Fixing header import in assets.rs..."
if grep -q "http::{header, StatusCode}" src/web/assets.rs; then
    sed -i 's/http::{header, StatusCode}/http::StatusCode/' src/web/assets.rs
    echo "   ✅ Fixed import"
else
    echo "   ⏭️  Already fixed"
fi

# 4. Rename unused uri parameter
echo ""
echo "4. Renaming unused uri parameter..."
if grep -q "pub async fn serve_static(uri:" src/web/assets.rs; then
    sed -i 's/pub async fn serve_static(uri:/pub async fn serve_static(_uri:/' src/web/assets.rs
    echo "   ✅ Renamed parameter"
else
    echo "   ⏭️  Already fixed"
fi

# 5. Remove unused Device import
echo ""
echo "5. Removing unused Device import..."
if grep -q "use tun::{Configuration, Device};" src/tunnel/mod.rs; then
    sed -i 's/use tun::{Configuration, Device};/use tun::Configuration;/' src/tunnel/mod.rs
    echo "   ✅ Removed Device import"
else
    echo "   ⏭️  Already fixed"
fi

# 6. Fix deprecated tun::Configuration::name
echo ""
echo "6. Fixing deprecated tun::Configuration::name..."
if grep -q 'config\.name("ippi-tun")' src/tunnel/mod.rs; then
    sed -i 's/config\.name("ippi-tun")/config.tun_name("ippi-tun")/' src/tunnel/mod.rs
    echo "   ✅ Updated to tun_name"
else
    echo "   ⏭️  Already fixed"
fi

# 7. Rename unused device variable
echo ""
echo "7. Renaming unused device variable..."
if grep -q "let device = tun::create" src/tunnel/mod.rs; then
    sed -i 's/let device = tun::create/let _device = tun::create/' src/tunnel/mod.rs
    echo "   ✅ Renamed device variable"
else
    echo "   ⏭️  Already fixed"
fi

# 8. Note about tunnel parameter (requires manual fix)
echo ""
echo "8. Tunnel parameter warning (requires manual check)..."
echo "   ℹ️  Check line ~295 in src/tunnel/mod.rs:"
echo "   Function: receive_tun_packet(&self, tunnel: &mut TunnelInfo)"
echo "   If 'tunnel' is unused, change to '_tunnel'"

echo ""
echo "=== Running cargo check to verify ==="
echo ""
cargo check --no-default-features 2>&1 | grep -A2 -B2 "warning\|error" | head -20

echo ""
echo "=== Summary ==="
echo "✅ Most warnings fixed automatically"
echo "🔧 Check src/tunnel/mod.rs line ~295 for manual fix"
echo ""
echo "To commit changes:"
echo "  git add ."
echo "  git commit -m 'fix: Clean up compiler warnings'"
echo ""
echo "Then test: cargo build --features p2p-full"