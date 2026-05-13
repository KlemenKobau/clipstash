# Stage 1: Build
FROM rust:1.87-bookworm AS builder

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/server/Cargo.toml crates/server/Cargo.toml
COPY crates/frontend/Cargo.toml crates/frontend/Cargo.toml
COPY crates/shared/Cargo.toml crates/shared/Cargo.toml

# Create dummy source files to build dependencies
RUN mkdir -p crates/server/src crates/frontend/src crates/shared/src && \
    echo "fn main() {}" > crates/server/src/main.rs && \
    echo "" > crates/server/src/lib.rs && \
    echo "" > crates/frontend/src/lib.rs && \
    echo "" > crates/shared/src/lib.rs && \
    cargo build --release -p clipstash-server 2>/dev/null || true && \
    rm -rf crates/

# Copy actual source code
COPY crates/ crates/
COPY migrations/ migrations/

# Touch source files to invalidate the cache for our code (not deps)
RUN touch crates/server/src/main.rs crates/server/src/lib.rs \
          crates/frontend/src/lib.rs crates/shared/src/lib.rs && \
    cargo build --release -p clipstash-server

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash clipstash
USER clipstash
WORKDIR /home/clipstash

COPY --from=builder /app/target/release/clipstash-server .
COPY static/ static/
COPY migrations/ migrations/

ENV DATABASE_URL="sqlite:/home/clipstash/data/clipstash.db?mode=rwc"
ENV CLIPSTASH_HOST="0.0.0.0"
ENV CLIPSTASH_PORT="3000"

EXPOSE 3000

VOLUME ["/home/clipstash/data"]

CMD ["./clipstash-server"]
