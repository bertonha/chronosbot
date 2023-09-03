# syntax=docker/dockerfile:1

FROM rust:1.72.0-slim-bookworm as builder
WORKDIR /usr/src/chronosbot
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/chronosbot* target/release/chronosbot*
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/chronosbot/target/release/chronosbot /usr/local/bin/chronosbot
CMD ["chronosbot"]
