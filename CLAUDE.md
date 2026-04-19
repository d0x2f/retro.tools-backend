# retrograde backend

Rust/Actix-web backend for retro.tools. Stores all data in Firestore. Deployed to Cloud Run.

## Architecture

- **Framework**: Actix-web 4
- **Database**: Firestore via the `firestore` crate (gRPC)
- **Auth**: Session cookies (`actix-session` + `actix-identity`). The `/auth` endpoint signs a Firebase custom token using a service account key pair, which the client exchanges for a Firebase ID token.
- **Participants**: A participant is identified by their session cookie. The `Participant` extractor reads the identity from the session.

## Module layout

```
src/
  boards/      routes, models, db
  columns/     routes, models, db
  cards/       routes, models, db
  participants/ routes, models, db
  config.rs    env-var config
  error.rs     AppError → HTTP status mapping
  cloudrun.rs  GCP project ID detection
  main.rs      server setup, all routes registered here
  integration_tests/  HTTP-level integration tests
```

## Running tests

Unit tests (no emulator needed):
```
cargo test
```

All tests including integration tests (requires Firestore emulator):
```
firebase emulators:start --only firestore &
FIRESTORE_EMULATOR_HOST=localhost:8080 cargo test -- --ignored
```

## Firestore emulator + security rules

The emulator enforces `firestore.rules`. Test helpers use `Authorization: Bearer owner` — the Firebase Admin SDK's magic bypass token that skips all security rule evaluation. This matches how production works (Admin SDK / service account bypasses rules entirely).

**Do not change the test token back to a user JWT** (e.g. `alg:none, sub:test`). A user JWT causes the emulator to evaluate security rules, which will deny all writes via the default-deny rule.

The three `emulator_db()` helpers that must use `"owner"`:
- `src/boards/db.rs`
- `src/cards/db.rs`
- `src/integration_tests/mod.rs`

## Security rules

`firestore.rules` controls **client-side** direct Firestore access only. The backend itself bypasses rules via the Admin SDK. Current rules allow authenticated board participants to read their boards and subcollections; all writes from clients are denied.

## Environment variables

| Var | Purpose |
|-----|---------|
| `PORT` | HTTP port (default 8000) |
| `ENVIRONMENT` | `development` or `production` |
| `SECRET_KEY` | Session signing key (required in production) |
| `FIRESTORE_PROJECT` | GCP project ID (falls back to Cloud Run metadata) |
| `FIREBASE_SERVICE_ACCOUNT_CREDENTIALS` | Path to service account JSON |
| `ALLOWED_ORIGINS` | Comma-separated CORS origins |
| `SECURE_COOKIE` | `true` to set Secure flag on session cookie |
| `SAME_SITE` | `strict`, `lax`, or `none` |

## Deployment

Dockerfile + Cloud Run. CD deploys Firestore security rules via `firebase deploy --only firestore:rules`.
