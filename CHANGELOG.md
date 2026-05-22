# Changelog

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
