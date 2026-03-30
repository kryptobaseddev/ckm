# CKM — Codebase Knowledge Manifest

## Project Identity

- **Package**: `ckm` (unscoped, universal — npm, PyPI, crates.io)
- **Repo**: `kryptobaseddev/ckm` (target GitHub remote — not yet created)
- **Local path**: `/mnt/projects/codebase-knowledge-manifest`
- **Purpose**: Multi-language SDK for consuming `ckm.json` manifests — auto-derived topics, progressive disclosure, CLI framework adapters
- **Status**: Rust-core SSoT architecture (pivot from spec-based to single implementation)
- **Published**: `ckm@0.1.0` (crates.io), `ckm-sdk@0.3.1` (npm)

## Architecture: Rust Core SSoT

**One implementation. Thin wrappers. Zero drift.**

```
packages/rust-core/   ← THE SSoT. Pure Rust. Crate: ckm (crates.io). Lib: ckm.
packages/node/        ← napi-rs 3 wrapper → npm: ckm-sdk@0.3.1
packages/python/      ← PyO3 + Maturin wrapper → PyPI: ckm (not published yet)
packages/go/          ← CGo FFI wrapper → Go modules (not published yet)
packages/cli-rs/      ← Pure Rust CLI binary → crates.io: ckm-cli (not published yet)
packages/core/        ← OLD TypeScript core (LEGACY, being replaced by node/)
packages/cli/         ← OLD TypeScript CLI (LEGACY, being replaced by cli-rs/)
conformance/          ← Test fixtures (verify rust-core)
```

All CKM logic (types, engine, builder, migration, validation, formatting) lives in `rust-core`. Every other language package is a thin FFI wrapper that calls into the Rust code. When behavior changes, it changes once in Rust.

## Key Documents

| Document | Purpose |
|----------|---------|
| `VISION.md` | Product intent, design principles, why Rust-core |
| `PLAN.md` | Epic breakdown with tasks, dependencies, critical path |
| `docs/specs/CKM-SDK-ARCHITECTURE.md` | Complete architecture specification |
| `docs/specs/INTERFACE.md` | API surface documentation (derived from rust-core) |
| `docs/specs/SPEC.md` | Algorithm documentation (derived from rust-core) |
| `ckm.schema.json` | ckm.json v2 JSON Schema (the input contract) |

## SSoT Flow

```
ckm.schema.json         → Defines what goes IN (the input format)
rust-core               → THE implementation (types, engine, builder, algorithms)
docs/specs/INTERFACE.md → Documents what comes OUT (derived from code)
docs/specs/SPEC.md      → Documents HOW (derived from code)
conformance/            → PROVES correctness (tests the Rust core)
```

**When this document and the code disagree, the code wins.**

## Build Tooling

| Tool | Purpose |
|------|---------|
| `cargo` | Rust builds and tests |
| `napi-rs` 3 | Node.js native bindings from Rust |
| `PyO3` + `maturin` | Python native wheels from Rust (not yet active) |

## Upstream Dependencies

- **VersionGuard** (`/mnt/projects/versionguard`): CKM's origin — `src/ckm/` module being replaced by `ckm` dependency
- **forge-ts** (`/mnt/projects/forge-ts`): Generates `ckm.json` from TypeScript source

## Conventions

- Rust-core depends only on serde + serde_json (no jsonschema, no thiserror)
- FFI wrappers are as thin as possible — NO logic, only marshaling
- Adapters (Commander.js, Click, Clap, Cobra) are written in the target language, calling engine via FFI
- Conformance tests run against rust-core; wrappers inherit correctness
- ESM-only for Node.js packages
- All packages are unscoped root names (no `@codluv` scope)

@AGENTS.md
