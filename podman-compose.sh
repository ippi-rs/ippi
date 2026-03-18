#!/bin/sh
echo "=== IPPI with Podman Compose ==="
echo ""

# Check available tools
if command -v podman-compose >/dev/null 2>&1; then
    echo "✅ Using podman-compose"
    COMPOSE_CMD="podman-compose"
elif command -v docker-compose >/dev/null 2>&1; then
    echo "⚠️  Using docker-compose (podman-compose not found)"
    COMPOSE_CMD="docker-compose"
else
    echo "❌ No compose tool found"
    echo "Install podman-compose or docker-compose"
    exit 1
fi

echo ""
echo "Available commands:"
echo "  $COMPOSE_CMD up -d      # Start in background"
echo "  $COMPOSE_CMD down       # Stop and remove"
echo "  $COMPOSE_CMD logs       # View logs"
echo "  $COMPOSE_CMD ps         # List containers"
echo ""
echo "Full example:"
echo "  $COMPOSE_CMD up -d"
echo "  $COMPOSE_CMD logs -f ippi"
echo ""
echo "To build and run:"
echo "  $COMPOSE_CMD up --build -d"