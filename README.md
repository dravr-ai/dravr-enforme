# dravr-enforme

Health data sync orchestrator — webhook-driven provider sync with cursor-based CDC for the Dravr platform.

## Overview

`dravr-enforme` orchestrates health data synchronization between wearable provider APIs (WHOOP, Garmin, Fitbit, Oura) and a local database. It is part of the [dravr](https://github.com/dravr-ai) ecosystem for the Pierre fitness AI coaching platform.

## Architecture

- **8 granular store traits** — Interface Segregation for write-path operations
- **`SyncProvider` trait** — per-provider, feature-gated implementations
- **`SyncOrchestrator`** — central coordinator for sync, backfill, and webhooks
- **CDC cursors** — incremental sync per user, provider, and data type
- **HMAC-SHA256 webhooks** — Standard Webhooks spec for real-time updates

## Quick Start

Add to your `Cargo.toml`:

```toml
dravr-enforme = "0.0"
```

Enable providers via feature flags:

```toml
dravr-enforme = { version = "0.0", features = ["provider-whoop"] }
```

## Crate Structure

| Crate | Description |
|-------|-------------|
| `dravr-enforme` | Core library with sync traits, models, and orchestrator |
| `dravr-enforme-mcp` | MCP server exposing sync tools via Model Context Protocol |
| `dravr-enforme-server` | Unified REST API + MCP server binary |

## Features

- `webhooks` — Webhook handling with axum types
- `provider-whoop` — WHOOP API v1 provider
- `provider-garmin` — Garmin API provider (planned)
- `all-providers` — Enable all providers

## Sibling Crates

- [dravr-equilibre](https://github.com/dravr-ai/dravr-equilibre) — Health domain models
- [dravr-riviere](https://github.com/dravr-ai/dravr-riviere) — Time-series storage
- [dravr-tronc](https://github.com/dravr-ai/dravr-tronc) — Shared MCP server infrastructure

## License

Apache-2.0
