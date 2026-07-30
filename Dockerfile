# Build stage
FROM rust:1.96-slim-bookworm AS builder

WORKDIR /app

# Cache dependencies first
COPY Cargo.toml Cargo.lock ./
COPY config.toml.example ./config.toml
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Copy the compiled binary from the builder
COPY --from=builder /app/target/release/autocal /usr/local/bin/autocal

# Run the command
CMD ["autocal"]