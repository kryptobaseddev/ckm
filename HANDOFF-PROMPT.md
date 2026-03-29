# CKM SDK — New Agent Handoff Prompt

Copy everything below the line into a new conversation at `/mnt/projects/codebase-knowledge-manifest`.

---

## Context

You are starting work on **CKM (Codebase Knowledge Manifest)** — a standalone, multi-language SDK for consuming `ckm.json` manifests. This project was just bootstrapped from an extraction out of VersionGuard.

### What CKM Does

CKM takes a `ckm.json` file (a structured manifest describing a CLI tool's concepts, operations, constraints, workflows, and config schema) and provides:
- **Auto-derived topics** from the manifest structure (zero manual mapping)
- **Progressive disclosure** for humans and LLM agents (4 levels with token budgets)
- **CLI framework adapters** that wire CKM into any CLI tool (Commander.js, Citty, oclif, Click, Clap, Cobra, etc.)
- **Multi-language SDKs** (TypeScript first, then Python, Rust, Go)

### Package Identity

- **npm**: `ckm` (unscoped, root package — confirmed available)
- **PyPI**: `ckm`
- **crates.io**: `ckm`
- **Go**: `github.com/kryptobaseddev/ckm/go`
- **CLI binary**: `ckm-cli` on npm

### Repository

- **Local path**: `/mnt/projects/codebase-knowledge-manifest`
- **Target GitHub**: `kryptobaseddev/ckm` (not yet created)
- **Single commit on main**: `06fd704` — project initialization

### Origin

CKM was extracted from **VersionGuard** (`/mnt/projects/versionguard`), where it lived as `src/ckm/` (~500 LOC, 4 files). The reference implementation has been copied to `docs/specs/reference-*.ts` for porting. A real-world v1 manifest (1518 lines) is at `docs/specs/reference-ckm-v1.json`.

After Phase 1 ships, VersionGuard will replace `src/ckm/` with `import { createCkmEngine } from 'ckm'`.

## What's Already Done

### Documents in this project (READ THESE FIRST):

1. **`VISION.md`** — Product intent, design principles, relationships to forge-ts and VersionGuard
2. **`PLAN.md`** — Full epic breakdown: 8 epics, 40+ tasks with dependencies and acceptance criteria
3. **`CLAUDE.md`** — Project instructions, architecture overview, conventions
4. **`docs/specs/CKM-SDK-ARCHITECTURE.md`** — Complete architecture specification (1000+ lines):
   - Package naming and monorepo layout
   - Spec-based backbone design (JSON Schema + INTERFACE.md + SPEC.md + conformance tests)
   - **Section 4a: SDK Interface Definition** — ALL SSoT types and methods defined in language-agnostic pseudocode, with per-language mapping table
   - ckm.json v2 schema design (canonical types, explicit slugs/tags, meta block)
   - Adapter interface design with concrete examples (Commander, Click, Clap, Cobra)
   - CLI design (`ckm`, `ckm validate`, `ckm migrate`, `ckm inspect`)
   - 6 rollout phases
   - 6 ADRs (architectural decision records)

### Reference implementation (for porting, NOT to be used directly):

- `docs/specs/reference-engine.ts` — Current CKM engine from VersionGuard (340 lines)
- `docs/specs/reference-types.ts` — Current v1 type definitions (171 lines)
- `docs/specs/reference-index.ts` — Current barrel export (43 lines)
- `docs/specs/reference-README-CKM.md` — Current developer guide
- `docs/specs/reference-ckm-v1.json` — Real v1 manifest from VersionGuard (1518 lines)

### Empty directories scaffolded:

```
packages/
  core/src/adapters/
  cli/src/
  python/ckm/adapters/
  rust/src/adapters/
  go/adapters/
  go/cmd/ckm/
conformance/fixtures/
conformance/expected/
```

## What Needs to Be Built

### Immediate Priority: Epic 0 + Epic 1 (TypeScript Foundation)

The critical path is:

1. **E0-T01**: Initialize monorepo tooling (root package.json, Biome, tsconfig, .gitignore, LICENSE)
2. **E0-T02**: Write `ckm.schema.json` (v2 JSON Schema)
3. **E0-T03**: Write `INTERFACE.md` (SDK Interface Definition — extract from architecture spec section 4a into standalone doc)
4. **E0-T04**: Write `SPEC.md` (deterministic algorithm specification)
5. **E0-T05**: Create conformance test fixtures with expected outputs
6. **E1-T01**: Scaffold `packages/core` (package.json, tsconfig, build config)
7. **E1-T02**: Implement TypeScript types from INTERFACE.md
8. **E1-T03**: Implement v1->v2 migration
9. **E1-T04**: Implement schema validation
10. **E1-T05**: Implement CKM engine (port from reference, adapt to v2)
11. **E1-T06**: Implement terminal formatter
12. **E1-T07**: Implement adapter types and registry
13. **E1-T08**: Implement Commander.js adapter
14. **E1-T09**: Write barrel export with subpath exports
15. **E1-T10**: Run full conformance suite
16. **E1-T11**: Publish `ckm` to npm

See `PLAN.md` for full details, dependencies, and acceptance criteria for every task.

## Key Architecture Decisions (Already Made)

1. **Spec-based backbone** — NOT a compiled binary. JSON Schema + INTERFACE.md + SPEC.md + conformance tests. Each language implements natively.
2. **Explicit slugs and tags in v2** — Generator populates them, engine uses directly. No suffix-stripping heuristics.
3. **Canonical type system** — `{ canonical: "string", original?: "CalVerFormat", enum?: [...] }`. Portable across all languages.
4. **Monorepo** — Single repo, all languages, atomic schema/spec/implementation updates.
5. **Adapters as peer dependencies** — `npm install ckm` does NOT install Commander/Citty/etc.
6. **Progressive disclosure as conformance requirement** — 4 levels, token budgets, tested.

## Conventions

- ESM-only for TypeScript
- Vitest for testing
- Biome for formatting/linting
- Zero runtime dependencies in core library
- All types defined in INTERFACE.md FIRST, then implemented per language
- Conformance tests are the source of truth for correctness

## What NOT to Do

- Do NOT modify `/mnt/projects/versionguard` — that's a separate project
- Do NOT add `@codluv` scope to any package — everything is unscoped root
- Do NOT build a WASM/FFI binary — the backbone is spec-based
- Do NOT add color/chalk dependencies to core — plain text formatting only
- Do NOT manually maintain topics — they are auto-derived from the manifest
