# Retro.Tools Backend

The main frontend project lives at: [d0x2f/retro.tools](https://github.com/d0x2f/retro.tools)

## v0.3

Refactor to use abdolence/firestore-rs instead of raw grpc with tonic

## v0.2

Total rewrite

- Switched from postgres to google firestore (more scalable, cheaper, cooler).
- Switched from rocket to async actix (fun).

## Testing

### Unit tests

No setup required:

```bash
cargo test
```

### Integration tests

Integration tests run against a local Firestore emulator and are marked `#[ignore]` by default.

**Start the emulator** (requires Java 21+ and the
[gcloud CLI](https://cloud.google.com/sdk/docs/install)):

```bash
gcloud beta emulators firestore start --host-port=localhost:8080
```

**Run the integration tests** in a separate terminal:

```bash
FIRESTORE_EMULATOR_HOST=localhost:8080 cargo test -- --ignored
```

To run everything at once:

```bash
FIRESTORE_EMULATOR_HOST=localhost:8080 cargo test -- --include-ignored
```

## TODO

- Past boards pagination
