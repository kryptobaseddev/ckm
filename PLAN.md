# CKM SDK — Master Plan

**Status**: Active (Rust-core SSoT pivot)
**Date**: 2026-03-29

---

## Objective

Extract CKM from VersionGuard into a standalone, multi-language SDK powered by a **single Rust core**. Ship `ckm` on npm (via napi-rs), PyPI (via PyO3), crates.io (native), and Go modules (via CGo/WASM). One implementation. Thin wrappers. Zero drift.

## Architecture Decision

**Previous approach (rejected):** Independent per-language implementations guided by INTERFACE.md and SPEC.md prose specs.

**Current approach:** Single Rust core library (`packages/rust-core/`) containing ALL algorithms — types, engine, migration, validation, formatting. Each language gets a thin FFI wrapper (~50-100 LOC) that exposes idiomatic APIs backed by the same Rust code.

**Why:** Four independent implementations of ~500 LOC algorithms *will* drift on edge cases, even with conformance tests. One implementation + mechanical wrappers eliminates drift by construction.

## Upstream Context

- **Origin**: CKM currently lives at `/mnt/projects/versionguard/src/ckm/` (~500 LOC, 4 files)
- **Reference implementation**: Copied to `docs/specs/reference-*.ts` for porting reference
- **Reference v1 manifest**: Copied to `docs/specs/reference-ckm-v1.json` (1518 lines, real-world)
- **Architecture spec**: `docs/specs/CKM-SDK-ARCHITECTURE.md` (complete)

---

## Epic 0: Project Foundation

Bootstrap the monorepo and backbone documents.

### E0-T01: Monorepo tooling
- Root `package.json` (workspaces), Biome, tsconfig, .gitignore, .editorconfig, LICENSE
- **Status**: DONE

### E0-T02: ckm.json v2 JSON Schema (`ckm.schema.json`)
- JSON Schema draft 2020-12 defining the v2 manifest format
- **Status**: DONE

### E0-T03: INTERFACE.md
- API surface documentation derived from Rust core
- **Status**: DONE (updated for Rust-core SSoT)

### E0-T04: SPEC.md
- Algorithm documentation derived from Rust core
- **Status**: DONE (updated for Rust-core SSoT)

### E0-T05: Conformance test fixtures
- 5 fixtures: minimal, multi-topic, polyglot, v1-legacy, edge-cases
- Expected outputs for each
- **Status**: DONE

### E0-T06: CI pipeline
- GitHub Actions for lint, format, build, test
- **Status**: DONE

---

## Epic 1: Rust Core Library (`packages/rust-core/`)

**THE SSoT.** All CKM algorithms in pure Rust. Zero FFI concerns.

### E1-T01: Scaffold rust-core crate
- `Cargo.toml`: name `ckm-core`, serde + serde_json
- **Depends on**: E0-T01
- **AC**: `cargo build` and `cargo test` pass

### E1-T02: Types (`src/types.rs`)
- All INTERFACE.md types as `#[derive(Serialize, Deserialize)]` structs
- **AC**: Can roundtrip-deserialize all conformance fixtures

### E1-T03: Engine (`src/engine.rs`)
- `CkmEngine::new(data)`, topic derivation per SPEC.md
- All query methods: topics, topic_index, topic_content, topic_json, manifest, inspect
- **AC**: All conformance fixtures pass

### E1-T04: Migration (`src/migrate.rs`)
- `detect_version()`, `migrate_v1_to_v2()`
- **AC**: v1-legacy fixture migrates correctly

### E1-T05: Validation (`src/validate.rs`)
- `validate_manifest()` with JSON pointer error paths
- **AC**: Valid v2 passes, v1 fails, invalid data returns correct errors

### E1-T06: Formatter (`src/format.rs`)
- Plain text terminal output, token budget compliance
- **AC**: Output matches expected text fixtures

### E1-T07: Full conformance suite
- Load all `conformance/fixtures/`, compare to `conformance/expected/`
- **AC**: All fixtures pass, exact-match on expected outputs

### E1-T08: Publish to crates.io
- Crate name: `ckm`
- **AC**: `cargo add ckm` works

---

## Epic 2: Node.js Wrapper (`packages/node/`)

napi-rs 3.8+ wrapper around rust-core.

### E2-T01: Scaffold napi-rs project
- `Cargo.toml` with napi-rs deps, `package.json` for npm
- **Depends on**: E1-T07
- **AC**: `napi build` produces `.node` file

### E2-T02: Wrap engine functions
- `#[napi]` annotations on: `createCkmEngine`, `validateManifest`, `migrateV1toV2`, `detectVersion`
- Auto-generated `.d.ts` TypeScript types
- **AC**: `import { createCkmEngine } from 'ckm'` works in Node.js

### E2-T03: WASM fallback build
- `wasm32-wasip1-threads` target for unsupported platforms
- **AC**: WASM fallback loads when native binary unavailable

### E2-T04: Cross-platform builds
- GitHub Actions matrix for linux-x64, linux-arm64, darwin-x64, darwin-arm64, win32-x64
- **AC**: All platform binaries build in CI

### E2-T05: Commander.js adapter
- Adapter written in TypeScript, calls engine via napi-rs binding
- **AC**: Integration test passes

### E2-T06: Additional TS adapters (Citty, oclif, Clipanion)
- Each adapter ~50 LOC calling engine via binding
- **AC**: Integration tests pass

### E2-T07: Publish `ckm` to npm
- Platform-specific packages + root package
- **AC**: `npm install ckm` works, types resolve

---

## Epic 3: Python Wrapper (`packages/python/`)

PyO3 + Maturin wrapper around rust-core.

### E3-T01: Scaffold PyO3 project
- `Cargo.toml` with PyO3 deps, `pyproject.toml` with Maturin
- **Depends on**: E1-T07
- **AC**: `maturin develop` builds and installs

### E3-T02: Wrap engine functions
- `#[pyclass]`/`#[pymethods]` on: `CkmEngine`, `validate_manifest`, `migrate_v1_to_v2`, `detect_version`
- **AC**: `from ckm import create_engine` works

### E3-T03: Click + Typer adapters
- Pure Python adapters calling engine via PyO3 binding
- **AC**: Integration tests pass

### E3-T04: Publish `ckm` to PyPI
- Maturin-built wheels for common platforms
- **AC**: `pip install ckm` works

---

## Epic 4: Go Wrapper (`packages/go/`)

CGo FFI or WASM (via wazero) wrapper around rust-core.

### E4-T01: Build strategy decision
- Evaluate CGo FFI vs WASM via wazero
- **Depends on**: E1-T07
- **AC**: Chosen approach builds and passes basic tests

### E4-T02: Wrap engine functions
- Go-idiomatic API: `NewEngine`, `ValidateManifest`, `MigrateV1ToV2`, `DetectVersion`
- **AC**: `engine.TopicIndex("tool")` returns correct output

### E4-T03: Cobra + urfave/cli adapters
- Go adapters calling engine via chosen FFI
- **AC**: Integration tests pass

### E4-T04: Publish Go module
- **AC**: `go get github.com/kryptobaseddev/ckm/go` works

---

## Epic 5: Standalone CLI (`packages/cli/`)

Pure Rust binary depending on rust-core directly.

### E5-T01: Scaffold CLI binary
- Clap-based CLI with subcommands
- **Depends on**: E1-T07
- **AC**: `cargo build` produces `ckm` binary

### E5-T02: Commands
- `ckm browse [topic] [--json]`, `ckm validate <file>`, `ckm migrate <file>`, `ckm inspect <file>`
- **AC**: All commands work with conformance fixtures

### E5-T03: Publish
- crates.io as `ckm-cli`, npm as `ckm-cli` (optional)
- **AC**: `cargo install ckm-cli` works

---

## Epic 6: forge-ts v2 Integration

### E6-T01: v2 schema generation
- forge-ts produces v2 `ckm.json` by default
- **AC**: `forge-ts build` on VG produces valid v2 manifest

### E6-T02: Enum resolution + operation input types
- **AC**: Manifest has meaningful types, not "unknown"

---

## Epic 7: VersionGuard Migration

### E7-T01: Replace `src/ckm/` with `ckm` dependency
- **AC**: `vg ckm` works identically, all tests pass

---

## Dependency Graph

```
E0 (foundation) ──> E1 (rust-core)
                      |
          +-----------+-----------+-----------+
          |           |           |           |
          v           v           v           v
        E2 (node)   E3 (python) E4 (go)    E5 (cli)
          |           |
          v           v
        E6 (forge-ts) ──> E7 (VG migration)
```

**Critical path**: E0 → E1 → E2 → E7
