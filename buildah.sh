#!/usr/bin/env bash

set -o errexit
set -x

# Stage 1 - build
BUILD=$(buildah from docker.io/rustlang/rust:nightly-slim)
BUILD_MOUNT=$(buildah mount "$BUILD")

buildah config --label maintainer="Dylan McGannon <dylan.mcgannon@gmail.com>" "$BUILD"
buildah config --workingdir /build "$BUILD"

buildah copy "$BUILD" . /build

buildah run "$BUILD" apt-get update
buildah run "$BUILD" apt-get install -y musl-tools
buildah run "$BUILD" rustup target install x86_64-unknown-linux-musl
buildah run "$BUILD" cargo build --release --target=x86_64-unknown-linux-musl

# Stage 2 - Run
RUN=$(buildah from alpine)
PORT=8000

buildah copy "$RUN" "$BUILD_MOUNT/build/target/x86_64-unknown-linux-musl/release/retrograde" /retrograde

buildah config --env PORT=$PORT "$RUN"
buildah config --port PORT=$PORT "$RUN"
buildah config --entrypoint "env ROCKET_PORT=\$PORT /retrograde" "$RUN"

# Finally saves the running container to an image
buildah commit "$RUN" retrograde:latest