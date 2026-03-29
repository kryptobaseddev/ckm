# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-29

### Added

- Initial PyO3 wrapper around ckm-core Rust library
- CkmEngine class with topic index, content, and JSON methods
- create_engine function for constructing engines from JSON strings
- validate_manifest function for v2 schema validation
- detect_version function for v1/v2 detection
- migrate_v1_to_v2 function for legacy manifest migration
- Maturin-based build system (pyproject.toml)
- Test suite with conformance fixture integration
