# Build stage
FROM rust:1.70-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev gcc nodejs npm

WORKDIR /app

# Copy source
COPY . .

# Build frontend
WORKDIR /app/frontend
RUN npm ci && npm run build

# Build Rust binary
WORKDIR /app
RUN cargo build --release --features frontend-embedded

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache libgcc

# Create non-root user
RUN addgroup -g 1000 ippi && \
    adduser -D -u 1000 -G ippi ippi

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/ippi /usr/local/bin/ippi

# Copy configuration
COPY config /etc/ippi

# Set permissions
RUN chown -R ippi:ippi /etc/ippi

USER ippi

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

ENTRYPOINT ["ippi"]
CMD ["--config", "/etc/ippi/ippi.toml"]