# Build the bot (stage)

FROM rust:1.97-slim AS builder
WORKDIR /usr/local/app

# Needs openssl-sys
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release -p server

# Run the bot (stage)

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/app
COPY --from=builder /usr/local/app/target/release/server /app/server

EXPOSE 8080

CMD ["/app/server"]
