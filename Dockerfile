# syntax=docker/dockerfile:1

FROM rust:1.72.0 as builder
WORKDIR /usr/src/chronosbot
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/chronosbot /usr/local/bin/chronosbot
CMD ["chronosbot"]
