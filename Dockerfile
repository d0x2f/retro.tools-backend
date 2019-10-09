FROM clux/muslrust:nightly AS build

WORKDIR /build

COPY . .

RUN cargo build --release

FROM scratch AS run

ENV PORT 8000

COPY --from=build /build/target/x86_64-unknown-linux-musl/release/retrograde /retrograde

EXPOSE $PORT

ENTRYPOINT ["/retrograde"]