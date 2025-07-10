# Multi-stage build for a smaller final image

# Build stage
FROM rust:1.85-slim-bullseye as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy Cargo files first to leverage Docker cache for dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {println!(\"Placeholder\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY src ./src

# Build the real application (only app source code rebuild needed)
RUN touch src/main.rs && cargo build --release

# Final stage
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies - using bullseye which has libssl1.1
RUN apt-get update && \
    apt-get install -y ca-certificates libssl1.1 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/consume-button-templates /app/consume-button-templates

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["./consume-button-templates"]
