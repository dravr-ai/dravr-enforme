# ABOUTME: Multi-stage Docker build for dravr-enforme-server and dravr-enforme-mcp binaries
# ABOUTME: Minimal runtime image for health data sync orchestrator

FROM rust:1-bookworm AS builder
WORKDIR /build
COPY . .
RUN cargo build --release -p dravr-enforme-server -p dravr-enforme-mcp

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash enforme

COPY --from=builder /build/target/release/dravr-enforme-server /usr/local/bin/
COPY --from=builder /build/target/release/dravr-enforme-mcp /usr/local/bin/

USER enforme
WORKDIR /home/enforme

EXPOSE 3300
ENTRYPOINT ["dravr-enforme-server"]
CMD ["--host", "0.0.0.0"]
