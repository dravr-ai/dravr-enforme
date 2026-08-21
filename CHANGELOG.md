# Changelog

## [0.1.47] — 2026-08-21

### Other

- chore(sciotte): integrate dravr-sciotte v0.9.0



## [0.1.46] — 2026-08-17

### Fixed

- fix: repair the SessionStart bootstrap guard for an empty .build

### Other

- chore(sciotte): integrate dravr-sciotte v0.8.6
- ci(sciotte): bump and release when sciotte does
- chore(deps): bump dravr-tronc to 0.6.2 and .build to a02d456
- chore(register): ledger + weekly phase review
- chore(register): point at dravr-carnet, the dravr-family register



## [0.1.45] — 2026-07-16



## [0.1.44] — 2026-07-13



## [0.1.43] — 2026-07-13



## [0.1.42] — 2026-07-11



## [0.1.41] — 2026-07-10

### Fixed

- **Sync writes no longer violate store foreign keys.** The orchestrator (and
  `backfill_user`) now upsert a per-user `DataSource` via `DataSourceStore`
  and re-stamp every fetched sleep/recovery/health record with the returned
  id before storage. Providers' hardcoded `{provider}-default` placeholder
  ids violated `data_source_id` foreign keys on the platform store, so no
  WHOOP/Garmin/Strava health record ever persisted. Continuous-metric batches
  now also carry the real data-source id instead of the `"default"` literal.
- **WHOOP fetches check HTTP status before deserializing.** A 401/403 error
  body used to fail serde decode and surface as `serialization error`,
  masking token expiry. Non-success responses now map to structured errors
  via `EnformeError::from_http_status` (401/403 → `CredentialsExpired`,
  429 → `RateLimited` honoring `Retry-After`, else `ProviderError` with
  status + body snippet).
- **Expired tokens are refreshed during sync.** `sync_user` proactively calls
  `CredentialStore::refresh_credentials` when the stored token is expired and
  refreshable, and retries a data type once when a fetch reports
  `CredentialsExpired` — previously `refresh_credentials` had zero call
  sites, so a stale token failed every scheduled sync forever.
- **WHOOP `total_sleep_seconds` now measures sleep, not time in bed.** The
  value was mapped from `total_in_bed_time_milli`; it is now the sum of
  light + slow-wave + REM stage durations (new
  `asleep_seconds_from_stage_millis` helper).

## [0.1.39] — 2026-06-30

### Changed

- deps: bump `dravr-sciotte` v0.7.11 → v0.7.12 (credential-login observability —
  page-navigation breadcrumbs, no-navigation stall warnings, and the last page
  named in the timeout error). Keeps the platform sciotte lock on one version.

## [0.1.37] — 2026-06-26

### Changed

- deps: bump `dravr-sciotte` v0.7.9 → v0.7.10 (Strava interval-feed now reads the
  canonical numeric `a.distance` in meters instead of the formatted stat value,
  fixing the 2024 "0.0X km" distance bug). Keeps the platform sciotte lock on a
  single version.

## [0.1.36] — 2026-06-26

### Changed

- deps: bump `dravr-sciotte` v0.7.8 → v0.7.9 (restores the Garmin activity
  scrape — the dravr-browser capture map moved responses under `.byUrl` with the
  body in `chunks[]`, which Garmin's passive-capture `js_extract` no longer read,
  so it scraped 0 activities; v0.7.9 reads both shapes and adds
  `SportType::from_garmin` so Garmin typeKeys stop bucketing as `Other`).

## [0.1.35] — 2026-06-26

### Changed

- deps: bump `dravr-sciotte` v0.7.7 → v0.7.8 (distance values from the Strava
  training-log interval feed now strip the `<abbr>` unit wrapper before parsing,
  fixing km/meters mis-scaling). Keeps the platform lock on a single sciotte
  version. No enforme API change.

## [0.1.34] — 2026-06-22

### Changed

- deps: bump `dravr-sciotte` v0.7.6→v0.7.7 (Strava training-log interval-feed
  date-jump pagination + locale-correct decimal parsing). Keeps enforme's
  `dravr-sciotte` on the same resolved version the platform's direct pins now
  use, so the fleet converges on one sciotte (no functional change here — the
  core enforme crate is unchanged; the scraper improvement ships in sciotte).

## [0.1.33] — 2026-06-19

### Changed

- deps: align internal satellite pins to the tronc-0.5.3 fleet release —
  `dravr-equilibre` v0.2.3→v0.2.4, `dravr-riviere` v0.2.3→v0.2.4,
  `dravr-sciotte` v0.7.5→v0.7.6. Keeps a single resolved version of each shared
  crate when the platform consumes enforme alongside its direct pins (no
  functional change; those cores are unchanged).

## [0.1.32] — 2026-06-19

### Changed

- deps: migrate `dravr-enforme-mcp` and `dravr-enforme-server` to dravr-tronc
  0.5.3 (dual-era MCP engine); state is `Arc<S>` directly (tronc no longer wraps
  it in a `RwLock`). The core `dravr-enforme` crate is unchanged.

## [0.1.23] — 2026-06-10

### Other

- chore(deps): bump dravr-sciotte v0.6.0 -> v0.7.0 sciotte v0.7.0 consolidates browser handling onto dravr-browser (crates.io); API unchanged for enforme. Keeps the platform able to converge on a single sciotte version.



## [0.1.22] — 2026-06-03



## [0.1.21] — 2026-06-03



## [0.1.20] — 2026-06-03



## [0.1.19] — 2026-05-29



## [0.1.18] — 2026-05-29



## [0.1.17] — 2026-05-29

### Other

- chore(deps): bump dravr-sciotte to v0.5.21 for real activity start_date Sync's Garmin/Strava scrape path now gets the true UTC start (was Utc::now/segment-effort time); also converges the platform on a single sciotte version.



## [0.1.16] — 2026-05-29



## [0.1.15] — 2026-05-29



## [0.1.14] — 2026-05-29



## [0.1.13] — 2026-05-29



## [0.1.12] — 2026-05-22

### Other

- deps: bump dravr-sciotte v0.5.14 -> v0.5.16 Matches the platform direct pin; v0.5.16 ships /challenge/dp digit scrape + Try-Another-Way debounce + stealth-JS-spoof removal that fix Strava-Google chooser surfacing and Garmin headless on Cloudflare.
- ci(release): cap upload-artifact retention at 7d



## [0.1.11] — 2026-05-07



## [0.1.10] — 2026-05-07



## [0.1.9] — 2026-05-05



## [0.1.8] — 2026-05-04



## [0.1.6] — 2026-04-23

### Fixed

- fix(tests): express jitter bounds as proportions of base Sidesteps new Rust 1.95.0 clippy::duration_suboptimal_units pedantic lint on from_secs(120) and matches 80%/120%-of-base comment.



## [0.1.5] — 2026-04-19

### Other

- ci(release): inject RELEASE_PAT for private dravr-sciotte git dep cargo generate-lockfile needs auth to clone the private sciotte repo; GITHUB_TOKEN doesn't have access
- chore(deps): remove dead rand dep, bump to v0.1.4 Closes transitive rand 0.8 Dependabot alert in dravr-platform



## [0.1.3] — 2026-04-10

### Other

- build: reduce tokio feature footprint to minimal set



## [0.1.2] — 2026-04-07

### Other

- bump dravr-sciotte v0.5.0 → v0.5.6 (dedup chromiumoxide)



## [0.1.0] — 2026-03-31



## [0.0.1] — 2026-03-31

### Added

- feat: integrate dravr-build-config for shared validation and lint rules
- feat: add Garmin and Strava providers via sciotte browser scraping Maps DailySummary to StoredRecoveryMetrics, StoredHealthMetrics, StoredSleepSession



All notable changes to this project will be documented in this file.
