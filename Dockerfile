# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.74.1
ARG DEBIAN_VERSION=bookworm
ARG APP_NAME=chronosbot
FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS build
ARG APP_NAME
WORKDIR /app

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server


FROM debian:${DEBIAN_VERSION}-slim AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser
COPY --from=build /bin/server /bin/
EXPOSE 3000
CMD ["/bin/server"]
