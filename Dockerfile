# ---- Build stage ----
FROM rust:1.85-bookworm AS builder
WORKDIR /app

# Cache de dependências
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release 2>/dev/null || true
RUN rm -rf src

# Build real
COPY . .
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/cloaker /usr/local/bin/cloaker

ENV PORT=8080
EXPOSE 8080

CMD ["cloaker"]
