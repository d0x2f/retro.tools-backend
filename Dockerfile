FROM clux/muslrust:nightly AS build

WORKDIR /build

COPY . .

RUN apt-get update && apt-get install -y wget unzip ca-certificates
RUN ./fetch-protos.sh
RUN rustup component add rustfmt

RUN cargo build --release

ENV TINI_VERSION v0.18.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini-static-muslc-amd64 /tini
RUN chmod +x /tini

FROM scratch AS run

ENV PORT 8000

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=build /build/target/x86_64-unknown-linux-musl/release/retrograde /retrograde
COPY --from=build /tini /tini

EXPOSE $PORT

ENTRYPOINT ["/tini", "--"]
CMD ["/retrograde"]