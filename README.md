# dravr-enforme — Health Data Sync Orchestrator

[![CI](https://github.com/dravr-ai/dravr-enforme/actions/workflows/ci.yml/badge.svg)](https://github.com/dravr-ai/dravr-enforme/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Background sync service that pulls health data from wearable providers and persists it locally, so AI coaches can query historical trends instantly without live API calls.

## Why

Without enforme, every coach question like "How was my sleep last night?" triggers a live API call to WHOOP/Garmin. No historical data is stored — trend analysis requires N API calls, rate limits become a problem, and data is discarded after each response.

With enforme running, health data is synced continuously in the background. Coaches read from the local database: instant response, 30+ days of history, cross-provider comparisons, no API calls at query time.

```
enforme syncs providers → DB (sleep, recovery, health, time-series)
                              ↓
              coach asks "how's my recovery trending?"
                              ↓
              reads from DB → passes to dravr-cageux → analysis → response
```

## Supported Providers

| Provider | Mechanism | Data | Feature flag |
|----------|-----------|------|-------------|
| **WHOOP** | REST API v1 + webhooks | Sleep, recovery, body measurements | `provider-whoop` |
| **Garmin** | Browser scraping via [dravr-sciotte](https://github.com/dravr-ai/dravr-sciotte) | HR, body battery, stress, VO2 max, steps, sleep stages, training load | `provider-garmin` (planned) |
| **Strava** | Browser scraping via dravr-sciotte | Fitness, fatigue, form (TSB) | `provider-strava` (planned) |

Garmin and Strava use sciotte's authenticated browser session to scrape health data from provider web apps — direct API access is blocked by Cloudflare WAF.

## Architecture

- **8 granular store traits** (Interface Segregation) — `SleepStore`, `RecoveryStore`, `HealthStore`, `TimeSeriesPointStore`, `DataSourceStore`, `SyncCursorStore`, `CredentialStore`, `UserConnectionStore`
- **`SyncProvider` trait** — per-provider, feature-gated. Providers return [dravr-equilibre](https://github.com/dravr-ai/dravr-equilibre) stored types directly
- **`SyncOrchestrator`** — scheduler (priority queue with jitter), backfill on OAuth connect, webhook handler
- **CDC cursors** — incremental sync per user/provider/data_type, cursor advances only on successful persist
- **HMAC-SHA256 webhooks** — WHOOP pushes events, enforme validates and processes async
- **Soft delete** — configurable via `ENFORME_DELETION_STRATEGY` (soft/hard)

## Quick Start

```toml
[dependencies]
dravr-enforme = { version = "0.1", features = ["provider-whoop"] }
```

```rust
use dravr_enforme::{SyncOrchestrator, SyncConfig, SyncDeps};

let deps = Arc::new(SyncDeps { /* your store implementations */ });
let config = SyncConfig::from_env();
let providers = dravr_enforme::providers::build_provider_registry();

let orchestrator = Arc::new(SyncOrchestrator::new(deps, providers, config));

// Start background scheduler (polls every 15 minutes)
let handle = orchestrator.clone().start_scheduler();

// Or sync a specific user on demand
orchestrator.sync_user("user-123", "whoop").await?;

// Or handle an incoming webhook
orchestrator.handle_webhook("whoop", &headers, &body).await?;
```

## Configuration

| Environment variable | Default | Description |
|---------------------|---------|-------------|
| `ENFORME_BACKFILL_DAYS` | `30` | Days of history to fetch on first connect |
| `ENFORME_POLL_INTERVAL_SECS` | `900` | Scheduler polling interval (15 min) |
| `ENFORME_MAX_CONCURRENT_SYNCS` | `3` | Max simultaneous provider syncs |
| `ENFORME_WEBHOOK_BASE_URL` | — | Public URL for webhook registration |
| `ENFORME_DELETION_STRATEGY` | `soft` | `soft` (tombstone) or `hard` (delete) |
| `ENFORME_TOMBSTONE_RETENTION_DAYS` | `90` | Days to keep soft-deleted records |
| `WHOOP_WEBHOOK_SECRET` | — | HMAC-SHA256 secret for WHOOP webhooks |

## Workspace Crates

| Crate | Description |
|-------|-------------|
| `dravr-enforme` | Core library — sync traits, models, orchestrator, providers |
| `dravr-enforme-mcp` | MCP server exposing sync tools via [dravr-tronc](https://github.com/dravr-ai/dravr-tronc) |
| `dravr-enforme-server` | Unified REST API + MCP server binary |

## Sibling Crates

| Crate | Role in the sync pipeline |
|-------|--------------------------|
| [dravr-equilibre](https://github.com/dravr-ai/dravr-equilibre) | Stored health types (`StoredSleepSession`, `StoredRecoveryMetrics`, etc.) |
| [dravr-riviere](https://github.com/dravr-ai/dravr-riviere) | Time-series storage trait + 122 `SeriesType` metric variants |
| [dravr-sciotte](https://github.com/dravr-ai/dravr-sciotte) | Browser-based scraping for Garmin/Strava health data |
| [dravr-cageux](https://github.com/dravr-ai/dravr-cageux) | Analysis algorithms (NSF/AASM sleep, VDOT, TSS) that run on synced data |
| [dravr-tronc](https://github.com/dravr-ai/dravr-tronc) | Shared MCP server infrastructure |

## License

Apache-2.0
