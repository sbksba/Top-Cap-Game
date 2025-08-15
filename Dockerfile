# --- Stage 1: Build the Rust binary ---
# We use a Rust image with the necessary tools to compile the application.
FROM rust:1.88-slim as builder

# Set the working directory inside the container.
WORKDIR /app

# Copy the Cargo files to take advantage of Docker's caching.
COPY Cargo.toml ./

# Copy the source code.
COPY src ./src

# Build the release binary. We use --release for an optimized, production-ready build.
RUN cargo build --release

# --- Stage 2: Final image ---
# Use a much smaller base image for the final container.
FROM debian:bookworm-slim

# Set the working directory.
WORKDIR /app

# Copy the pre-compiled binary from the builder stage.
COPY --from=builder /app/target/release/top-cap .

# Copy the front-end file (index.html) into the same directory as the server.
# The server will now be able to find and serve this file.
COPY assets ./assets

# Expose port 3000 so the container can receive web traffic.
EXPOSE 3000

# The command to run the server when the container starts.
# Replace 'top-cap' with the actual name of your executable if it's different.
CMD ["./top-cap"]
