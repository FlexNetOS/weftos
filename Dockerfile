# Multi-stage Dockerfile for weft (clawft CLI)
#
# Uses cargo-chef for dependency caching and distroless/cc-debian12 as the
# runtime base (includes glibc + libgcc + ca-certificates, no shell).
#
# Build:
#   docker buildx build --platform linux/amd64,linux/arm64 -t weft .
#
# Target: <50MB compressed image.

# ---------------------------------------------------------------------------
# Stage 1: Chef -- prepare dependency recipe
# ---------------------------------------------------------------------------
FROM rust:1.93-bookworm AS chef

RUN cargo install cargo-chef --locked
WORKDIR /app

# ---------------------------------------------------------------------------
# Stage 2: Planner -- extract the dependency recipe
# ---------------------------------------------------------------------------
FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------------------------------------------------------------------------
# Stage 3: Builder -- build dependencies (cached), then build the application
# ---------------------------------------------------------------------------
FROM chef AS builder

# Install build dependencies (pkg-config for system lib detection)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies (cached layer -- only rebuilds when Cargo.toml/lock change)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build the application
COPY . .
RUN cargo build --release --bin weft \
    && strip target/release/weft

# ---------------------------------------------------------------------------
# Stage 4: Runtime -- minimal image with only the binary
# ---------------------------------------------------------------------------
FROM gcr.io/distroless/cc-debian12 AS runtime

COPY --from=builder /app/target/release/weft /usr/local/bin/weft

# distroless runs as nonroot (uid 65534) by default
USER nonroot
WORKDIR /home/nonroot

# Default config directory
VOLUME ["/home/nonroot/.clawft"]

# Health check for gateway mode
# HEALTHCHECK uses exec form (no shell in distroless)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/usr/local/bin/weft", "status"]

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/weft"]
CMD ["gateway"]
