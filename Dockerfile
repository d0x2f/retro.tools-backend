FROM docker.io/rustlang/rust:nightly-slim AS build

WORKDIR /build

COPY . .

RUN apt-get update
RUN apt-get install -y musl-tools
RUN rustup target install x86_64-unknown-linux-musl
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine AS run

ENV PORT 8000

COPY --from=build /build/target/x86_64-unknown-linux-musl/release/retrograde /retrograde

EXPOSE $PORT

ENTRYPOINT env ROCKET_PORT=$PORT /retrograde