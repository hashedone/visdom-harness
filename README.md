# visdom-harness

A local AI agent harness for running structured evaluation pipelines. Ingests context, drives LLM inference loops, persists results, and exposes a live debug view.

## Requirements

- Rust (stable, 1.80+)
- SQLite (no separate server needed — the binary creates the database file on first run)

## Running

```sh
cargo run
```

The server listens on `127.0.0.1:3000` by default.

```sh
# Verify it's up
curl http://127.0.0.1:3000/health
# → {"status":"ok"}
```

## Configuration

All configuration is via environment variables. Copy `.env.example` and adjust:

```sh
cp .env.example .env
# then edit .env
```

| Variable | Default | Description |
|---|---|---|
| `BIND_ADDR` | `127.0.0.1:3000` | TCP address the HTTP server binds to |
| `DATABASE_URL` | `sqlite://visdom.db?mode=rwc` | SQLite database path (`?mode=rwc` creates the file) |
| `RUST_LOG` | `info,visdom_harness=debug` | Log filter (passed to `tracing-subscriber`) |

## Testing

```sh
cargo test
```

Tests spin up in-process servers and temporary SQLite databases — no external setup needed.

## Project layout

```
src/
  main.rs        binary entry point — config, telemetry, DB, server startup
  lib.rs         public surface: AppState, build_app
  config.rs      env-var config parsing and validation
  db.rs          SQLite pool + sqlx migration runner
  telemetry.rs   OpenTelemetry + tracing-subscriber wiring
  error.rs       AppError (thiserror) + Axum IntoResponse
  http/
    mod.rs       Axum router
    health.rs    GET /health
migrations/      versioned SQL migration files (applied by sqlx at startup)
tests/           integration tests
```
