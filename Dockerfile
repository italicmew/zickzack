# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.72.1
ARG APP_NAME=zickzack
FROM rust:${RUST_VERSION}-bullseye AS build
ARG APP_NAME
WORKDIR /app
RUN apt update && apt-get install sqlite3

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --release
cp ./target/release/$APP_NAME /bin/server
EOF


FROM rust:${RUST_VERSION}-bullseye AS build_wasm
ARG APP_NAME
WORKDIR /app
RUN rustup target add wasm32-unknown-unknown
RUN --mount=type=bind,source=web/src,target=web/src \
    --mount=type=bind,source=web/static,target=web/static \
    --mount=type=bind,source=web/Cargo.toml,target=web/Cargo.toml \
    --mount=type=bind,source=web/Cargo.lock,target=web/Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
     <<EOF
set -e
cd /app/web
apt update && apt install -y pkg-config libssl-dev
cargo install wasm-pack
wasm-pack build --target web
EOF


FROM debian:bookworm-slim as final


# Copy the executable from the "build" stage.
COPY --from=build /bin/server /bin/
COPY --from=build_wasm /app/web/pkg /bin/web/static
COPY ./web/static/* /bin/web/static


# Expose the port that the application listens on.
EXPOSE 8080

# What the container should run when it is started.
CMD ["/bin/server"]