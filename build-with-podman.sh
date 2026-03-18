#!/bin/sh
echo "=== Building IPPI with Podman ==="
echo ""

# Check if podman is available
if command -v podman >/dev/null 2>&1; then
    echo "✅ Podman found: $(podman --version)"
    CONTAINER_CMD="podman"
elif command -v docker >/dev/null 2>&1; then
    echo "⚠️  Podman not found, using Docker: $(docker --version)"
    CONTAINER_CMD="docker"
else
    echo "❌ Neither Podman nor Docker found"
    exit 1
fi

echo ""
echo "1. Building container image..."
$CONTAINER_CMD build -t ippi/ippi:latest .

echo ""
echo "2. Testing container..."
$CONTAINER_CMD run --rm -p 8080:8080 --name ippi-test ippi/ippi:latest --version

echo ""
echo "3. Container info:"
$CONTAINER_CMD images | grep ippi

echo ""
echo "=== Build complete ==="
echo "To run: $CONTAINER_CMD run -p 8080:8080 ippi/ippi:latest"
echo "Or with config: $CONTAINER_CMD run -p 8080:8080 -v ./config:/etc/ippi:ro ippi/ippi:latest"