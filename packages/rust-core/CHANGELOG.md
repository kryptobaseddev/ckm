# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [CalVer](https://calver.org/) (YYYY.M.MICRO).

## [2026.3.1] - 2026-03-30

### Added

- CkmManifestBuilder: fluent producer API for generators
- Producer-declared topics: manifest `topics` field overrides auto-derivation
- Extensions field on all entities (freeform escape hatch)
- 13 new optional fields: rules, relatedTo, preconditions, exitCodes, checksPerformed, configKey, default, security, expect, effect

### Changed

- Topic derivation: all concepts with slugs become topics (not just *Config)
- Switched to CalVer (YYYY.M.MICRO) versioning

## [0.1.0] - 2026-03-29

### Added

- Initial CKM SDK implementation for Rust
- Types from INTERFACE.md as serde-compatible structs
- CkmEngine with topic derivation from SPEC.md
- v1 to v2 migration support
- Manifest validation
- Terminal formatting (plain text)
- Clap adapter (feature-gated)
