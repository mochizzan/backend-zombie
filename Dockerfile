# Stage 1: Build
FROM rust:1.85 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo 'fn main() { println!("dummy"); }' > src/main.rs
RUN cargo build --release
RUN rm -rf src
# Copy actual source and rebuild
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN groupadd -r appuser && useradd -r -g appuser appuser
WORKDIR /app
COPY --from=builder /app/target/release/roblox-backend /usr/local/bin/
COPY migrations ./migrations
RUN chown -R appuser:appuser /app
USER appuser
EXPOSE 3000
CMD ["roblox-backend"]
