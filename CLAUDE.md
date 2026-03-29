# CKM — Codebase Knowledge Manifest

## Project Identity

- **Package**: `ckm` (unscoped, universal — npm, PyPI, crates.io)
- **Repo**: `kryptobaseddev/ckm` (target GitHub remote — not yet created)
- **Local path**: `/mnt/projects/codebase-knowledge-manifest`
- **Purpose**: Multi-language SDK for consuming `ckm.json` manifests — auto-derived topics, progressive disclosure, CLI framework adapters
- **Status**: Pre-implementation (architecture spec complete, PLAN.md defines epics/tasks)

## Key Documents

| Document | Purpose |
|----------|---------|
| `VISION.md` | Product intent, design principles, relationships |
| `PLAN.md` | Full epic breakdown with tasks, dependencies, acceptance criteria |
| `docs/specs/CKM-SDK-ARCHITECTURE.md` | Complete architecture specification (1000+ lines) |
| `docs/specs/INTERFACE.md` | SDK Interface Definition — SSoT types and methods (to be written) |
| `docs/specs/SPEC.md` | Deterministic algorithm specification (to be written) |
| `ckm.schema.json` | ckm.json v2 JSON Schema (to be written) |

## Architecture Overview

The backbone is spec-based (NOT a compiled binary):

1. **`ckm.schema.json`** — INPUT: what goes into the engine
2. **`INTERFACE.md`** — API SURFACE: types + methods every SDK exposes
3. **`SPEC.md`** — BEHAVIOR: deterministic topic derivation algorithm
4. **`conformance/`** — PROOF: test fixtures every implementation must pass

Each language implements natively. Conformance tests prove correctness.

## Monorepo Structure

```
packages/
  core/     — TypeScript core library (npm: ckm)
  cli/      — Standalone CLI binary (npm: ckm-cli)
  python/   — Python SDK (PyPI: ckm)
  rust/     — Rust SDK (crates.io: ckm)
  go/       — Go SDK (go module)
conformance/ — Cross-language test fixtures
docs/specs/  — Architecture, interface, and algorithm specs
```

## Rollout Phases

1. **Phase 1**: TypeScript Foundation (core + Commander adapter + CLI + conformance)
2. **Phase 2**: TypeScript Adapter Expansion (Citty, oclif, Clipanion)
3. **Phase 3**: Python SDK (Click, Typer adapters)
4. **Phase 4**: Rust SDK (Clap adapter)
5. **Phase 5**: Go SDK (Cobra, urfave/cli adapters)
6. **Phase 6**: forge-ts v2 integration

## Upstream Dependencies

- **VersionGuard** (`/mnt/projects/versionguard`): CKM's origin — `src/ckm/` module being extracted. After Phase 1, VG depends on `ckm` as a library.
- **forge-ts** (`/mnt/projects/forge-ts`): Generates `ckm.json` from TypeScript source. Phase 6 adds v2 schema support.

## Conventions

- ESM-only (TypeScript package)
- Vitest for TypeScript tests
- Biome for formatting/linting
- Zero runtime dependencies in core (adapters use peerDependencies)
- All types defined in INTERFACE.md first, then implemented per language

@AGENTS.md
