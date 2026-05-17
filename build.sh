#!/usr/bin/env bash
set -euo pipefail

IMAGE="retrograde:latest"
BUILDER_IMAGE=""
BUILD_ONLY=0

usage() {
  echo "Usage: $0 [options]"
  echo ""
  echo "  --image <tag>    Runtime image tag (default: retrograde:latest)"
  echo "  --builder <tag>  Also commit the build stage under this tag"
  echo "  --build-only     Skip assembling the runtime image"
  echo "  --help           Show this message"
}

while [[ $# -gt 0 ]]; do
  case $1 in
    --image)      IMAGE="$2";         shift 2 ;;
    --builder)    BUILDER_IMAGE="$2"; shift 2 ;;
    --build-only) BUILD_ONLY=1;       shift   ;;
    --help)       usage; exit 0       ;;
    *)            echo "Unknown option: $1"; usage; exit 1 ;;
  esac
done

if [ "$BUILD_ONLY" -eq 1 ] && [ -z "$BUILDER_IMAGE" ]; then
  echo "error: --build-only requires --builder"; exit 1
fi

build=$(buildah from docker.io/rustlang/rust:nightly-alpine)
run=""

cleanup() {
  buildah unmount "$build" 2>/dev/null || true
  [ -n "$build" ] && buildah rm "$build" 2>/dev/null || true
  [ -n "$run" ]   && buildah rm "$run"   2>/dev/null || true
}
trap cleanup EXIT

echo "==> Copying source"
buildah copy "$build" . /build
buildah config --workingdir /build "$build"
buildah run "$build" -- apk add --no-cache ca-certificates

echo "==> Building"
buildah run --env RUSTFLAGS="-C target-cpu=x86-64-v3" "$build" -- cargo build --release

if [ -n "$BUILDER_IMAGE" ]; then
  echo "==> Installing clippy"
  buildah run "$build" -- rustup component add clippy
  echo "==> Compiling tests"
  buildah run "$build" -- cargo test --no-run
  echo "==> Committing builder: $BUILDER_IMAGE"
  buildah commit "$build" "$BUILDER_IMAGE"
fi

if [ "$BUILD_ONLY" -eq 0 ]; then
  echo "==> Assembling runtime image"
  mount=$(buildah mount "$build")

  run=$(buildah from scratch)
  buildah copy "$run" "$mount/etc/ssl/certs/ca-certificates.crt" /etc/ssl/certs/ca-certificates.crt
  buildah copy "$run" "$mount/build/target/release/retrograde" /retrograde
  buildah config --env PORT=8000 --port 8000 --entrypoint '["/retrograde"]' "$run"

  buildah unmount "$build"
  buildah commit "$run" "$IMAGE"
  echo "==> Done: $IMAGE"
fi
