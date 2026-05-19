# visdom-harness

A local AI agent harness for running structured evaluation pipelines. Ingests context, drives LLM inference loops, persists results, and exposes a live debug view.

## Requirements

- Rust (stable, 1.80+)
- SQLite (no separate server needed — the binary creates the database file on first run)

## Running

Two processes are needed: the harness API server and the Dioxus WASM dev server.

**Terminal 1 — harness (API + persistence):**
```sh
cd harness
cp .env.example .env   # first time; add ANTHROPIC_API_KEY
cargo run
# → listening on http://127.0.0.1:3001
```

**Terminal 2 — debug UI (hot-reload WASM):**
```sh
# Install the Dioxus CLI once:
cargo install dioxus-cli

cd web
dx serve --port 8081
# → http://127.0.0.1:8081
```

Open `http://127.0.0.1:8081` in a browser.

```sh
# Verify the harness is up
curl http://127.0.0.1:3001/health
# → {"status":"ok"}
```

## Configuration

Harness config lives in `visdom.toml` (committed defaults) and is overridden by env vars or a local `.env` file:

```sh
cd harness
cp .env.example .env
# edit .env — add ANTHROPIC_API_KEY at minimum
```

| Variable | Default | Description |
|---|---|---|
| `BIND_ADDR` | `127.0.0.1:3001` | TCP address the harness binds to |
| `DATABASE_URL` | `sqlite://visdom.db?mode=rwc` | SQLite database path (`?mode=rwc` creates the file) |
| `RUST_LOG` | `info,visdom_harness=debug` | Log filter (passed to `tracing-subscriber`) |

**Changing ports:** If you need different ports (e.g. to avoid conflicts with another project):

```sh
# harness on 3002, UI pointing at it
BIND_ADDR=127.0.0.1:3002 cargo run -p visdom-harness

VISDOM_API_URL=http://127.0.0.1:3002 dx serve --port 8082
```

Default ports: harness `:3001`, UI `:8081` (pass `--port` to change).
```

`VISDOM_API_URL` is baked into the WASM binary at compile time — it must be set before `dx serve` (or `dx build`).

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
