# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/JoelEarps/observe-rs/releases/tag/v0.1.0) - 2026-01-30

### Added

- Initial release of observe-rs: multi-backend observability library for Rust
- Prometheus backend with counters, gauges, histograms, and labeled metrics
- Standalone HTTP server for `/metrics`, `/health`, `/ready`
- Mock backend for testing
- JSON and YAML config support (optional features)

### Other

- Conventional commit and SemVer-based releases via release-plz
