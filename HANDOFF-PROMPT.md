# CKM SDK — New Agent Handoff Prompt

Copy everything below the `---` line into a new conversation at `/mnt/projects/codebase-knowledge-manifest`. After reading and confirming understanding, you may delete this file — all durable context lives in CLAUDE.md, VISION.md, PLAN.md, and docs/specs/.

---

## Who You Are and What This Project Is

You are starting work on **CKM (Codebase Knowledge Manifest)** — a standalone, multi-language SDK. This project was just bootstrapped (2 commits, 11 files) and has zero implementation code yet. Everything is spec and planning documents.

### The Problem CKM Solves

Every CLI tool reinvents help output. Humans get `--help` text. LLM agents get nothing structured. Topic-based help requires manual maintenance. Config documentation drifts from implementation.

### How CKM Solves It

A `ckm.json` file is a structured manifest that captures a CLI tool's **concepts** (domain objects), **operations** (what the tool does), **constraints** (rules enforced), **workflows** (multi-step goals), and **configSchema** (what config controls what). The CKM SDK reads this manifest and auto-derives **topics** — zero manual mapping. A human types `mytool ckm calver` and gets formatted help. An agent adds `--json` and gets structured data.

### Package Identity

| Ecosystem | Package | Registry | Status |
|-----------|---------|----------|--------|
| TypeScript/JS | `ckm` | npm | Available (confirmed 2026-03-29) |
| CLI binary | `ckm-cli` | npm | Available |
| Python | `ckm` | PyPI | Available |
| Rust | `ckm` | crates.io | Available |
| Go | `github.com/kryptobaseddev/ckm/go` | Go modules | — |

**All packages are unscoped root names. No `@codluv` scope. No org prefix. This was an explicit owner decision — CKM is a universal standard, not a branded utility.**

### Repository

- **Local path**: `/mnt/projects/codebase-knowledge-manifest`
- **Target GitHub**: `kryptobaseddev/ckm` (not yet created)
- **Branch**: `main`, 2 commits
- **npm user**: `kryptobaseddev`

---

## Where CKM Came From (Origin Story)

CKM was born inside **VersionGuard** (`/mnt/projects/versionguard`), a strict versioning enforcement tool for SemVer/CalVer. VersionGuard uses **forge-ts** (a TypeScript documentation compiler at `/mnt/projects/forge-ts`) to generate a `ckm.json` from TSDoc annotations. The CKM module inside VG reads that manifest and provides topic-based help via `vg ckm [topic]`.

The CKM module was always self-contained — zero dependencies on VG internals, ~500 LOC across 4 files. When we saw it could serve ANY CLI tool in ANY language, we decided to extract it into a standalone SDK.

### What forge-ts Is (You'll See References)

forge-ts is a TypeScript documentation compiler that:
- Enforces TSDoc coverage as a build gate
- Generates `ckm.json` (the manifest CKM consumes)
- Generates `llms.txt` (API context for LLM agents)

**Relationship**: forge-ts = generation (produces `ckm.json`). CKM SDK = consumption (reads it, provides help/topics). Any tool can generate a valid `ckm.json` — forge-ts is one generator, not the only one. Phase 6 of the plan adds v2 schema support to forge-ts.

### What VersionGuard Is (You'll See References)

VersionGuard is CKM's first consumer. After Phase 1 ships `ckm` on npm, VG replaces `src/ckm/` with `import { createCkmEngine } from 'ckm'`. The reference implementation copied into this project (`docs/specs/reference-*.ts`) is VG's CKM module — the code you're porting and evolving.

---

## Key Decisions Already Made (and WHY)

### 1. Spec-Based Backbone (NOT a compiled binary)

**Decision**: The backbone is a JSON Schema + algorithm specification + conformance tests. Each language implements natively.

**Why we rejected alternatives**:
- **Rust core + WASM**: Adds 2-10MB to every consumer. Python/Go FFI is fragile. Build complexity explodes. Disproportionate to the algorithm complexity.
- **TypeScript core + WASM**: Same overhead. Go/Rust developers would never accept a JS runtime dependency.

**Why spec-based wins**: The CKM engine is ~500 LOC of pure data transformation — parse JSON, filter by naming convention, derive slugs, match by substring, format output. Entirely deterministic string processing. Each language gets a zero-dependency, idiomatic implementation verified by shared conformance tests. The algorithm is simple enough that reimplementing per language is cheaper than the integration cost of a shared binary.

### 2. ckm.json v2 Schema (Breaking from v1)

**What's wrong with v1** (the current format from forge-ts):
- Types are raw TypeScript strings (`"CalVerFormat"`, `"string | undefined"`) — meaningless to Python/Rust/Go
- No metadata about the generating tool or source language
- Topic derivation depends on TypeScript naming conventions (strip `Config` suffix) — breaks for `calver_config` (Python) or `CalverConfig` (Rust)
- Operation input types are all `"unknown"` (forge-ts limitation)
- No explicit topic linkage — engine guesses via fragile substring matching

**What v2 fixes**:
- **Canonical type system**: `{ canonical: "string", original?: "CalVerFormat", enum?: ["YYYY.MM.DD", ...] }` — portable across all languages
- **Explicit slugs**: Generator populates `concept.slug` (e.g., `"calver"`) instead of engine guessing from names
- **Explicit tags**: `concept.tags: ["config"]` replaces suffix heuristic; `operation.tags: ["calver"]` replaces keyword matching
- **Meta block**: `{ project, language, generator, generated, sourceUrl }` — full provenance
- **Constraint severity**: `"error" | "warning" | "info"` instead of flat rules
- **Workflow step discriminant**: `{ action: "command" | "manual", value, note? }` instead of nullable-field-soup

**v2 example** (the shape you're building toward):
```json
{
  "$schema": "https://ckm.dev/schemas/v2.json",
  "version": "2.0.0",
  "meta": {
    "project": "my-tool",
    "language": "typescript",
    "generator": "forge-ts@0.21.1",
    "generated": "2026-03-29T19:42:49.373Z",
    "sourceUrl": "https://github.com/org/repo"
  },
  "concepts": [{
    "id": "concept-calver-config",
    "name": "CalVerConfig",
    "slug": "calver",
    "what": "Configures CalVer validation rules.",
    "tags": ["config"],
    "properties": [{
      "name": "format",
      "type": { "canonical": "string", "enum": ["YYYY.MM.DD", "YYYY.MM"], "original": "CalVerFormat" },
      "description": "Calendar format.",
      "required": true,
      "default": null
    }]
  }],
  "operations": [{
    "id": "op-validate",
    "name": "validate",
    "what": "Validates a CalVer string.",
    "tags": ["calver", "validation"],
    "inputs": [{ "name": "version", "type": { "canonical": "string" }, "required": true, "description": "Version string." }],
    "outputs": { "type": { "canonical": "object", "original": "ValidationResult" }, "description": "Validation result." }
  }],
  "constraints": [{ "id": "c-1", "rule": "Rejects future dates.", "enforcedBy": "validate", "severity": "error" }],
  "workflows": [{ "id": "wf-1", "goal": "Release", "tags": ["release"], "steps": [{ "action": "command", "value": "vg validate" }] }],
  "configSchema": [{ "key": "calver.format", "type": { "canonical": "string" }, "description": "Format.", "default": null, "required": true }]
}
```

**v1 backward compatibility**: The engine auto-migrates v1 manifests to v2 internally. The migration algorithm is deterministic and conformance-tested.

### 3. Adapter Pattern (Inspired by VG's DETECTION_TABLE)

VersionGuard has a provider pattern for version sources: a `DETECTION_TABLE` maps manifest files to provider factories, and `resolveVersionSource()` walks the table. CKM mirrors this:

- `ADAPTER_TABLE` maps framework identifiers to lazy-loaded adapter factories
- `CkmCliAdapter` interface defines `register(program, engine, options)`
- Each adapter is ~50-100 LOC of framework-specific registration
- Adapters are **peer dependencies** — `npm install ckm` does NOT pull in Commander, Click, Clap, etc.

### 4. Progressive Disclosure (Mandatory, Conformance-Tested)

4 disclosure levels. Every adapter MUST support all four. Token budgets enforced by conformance tests.

| Level | Command | Audience | Budget |
|-------|---------|----------|--------|
| 0 | `ckm` | Human/Agent discovery | 300 tokens |
| 1 | `ckm <topic>` | Drill-down | 800 tokens |
| 1J | `ckm <topic> --json` | Agent structured | 1200 tokens |
| 2 | `ckm --json` | Agent full index | 3000 tokens |

### 5. SSoT Interface Definition

All types and methods are defined ONCE in `INTERFACE.md` (language-agnostic pseudocode), then implemented per language. No SDK may add, remove, or rename a method without updating INTERFACE.md first. The architecture spec section 4a contains the full interface definition — it needs to be extracted into a standalone `INTERFACE.md` document (task E0-T03).

### 6. Monorepo

Single repo, all languages. Schema + spec + conformance + implementations stay in lock-step.

### 7. Package Rename for VersionGuard Too

Separately, VersionGuard is also renaming from `@codluv/versionguard` to `versionguard` (unscoped) at v1.0.0. Both packages become clean root names. **This is VG's concern, not CKM's — do NOT implement this here.**

---

## What's In This Project Right Now

### Documents (READ ALL OF THESE):

| File | What | Lines |
|------|------|-------|
| `CLAUDE.md` | Project instructions, architecture overview, conventions | 55 |
| `VISION.md` | Product intent, design principles, relationships | 120 |
| `PLAN.md` | **8 epics, 40+ tasks**, dependencies, acceptance criteria, critical path | 300 |
| `docs/specs/CKM-SDK-ARCHITECTURE.md` | **Complete architecture spec** — the bible of this project | 1028 |

### Reference Implementation (for porting, NOT to use directly):

| File | What | Lines |
|------|------|-------|
| `docs/specs/reference-engine.ts` | VG's CKM engine — the v1 implementation you're porting | 340 |
| `docs/specs/reference-types.ts` | VG's CKM types — v1 type definitions | 171 |
| `docs/specs/reference-index.ts` | VG's barrel export | 43 |
| `docs/specs/reference-README-CKM.md` | VG's CKM developer guide | 140 |
| `docs/specs/reference-ckm-v1.json` | Real v1 manifest from VG (use as test fixture) | 1518 |

### Directory Scaffolding:

```
packages/core/src/adapters/    — TypeScript core (npm: ckm)
packages/cli/src/              — Standalone CLI (npm: ckm-cli)
packages/python/ckm/adapters/  — Python SDK (PyPI: ckm)
packages/rust/src/adapters/    — Rust SDK (crates.io: ckm)
packages/go/adapters/          — Go SDK
packages/go/cmd/ckm/           — Go CLI binary
conformance/fixtures/           — Test input manifests
conformance/expected/           — Expected output for each fixture
```

---

## What to Build (Priority Order)

### Immediate: Epic 0 (Foundation) + Epic 1 (TypeScript Core)

Critical path to first npm publish:

```
E0-T01 (monorepo tooling)
  → E0-T02 (ckm.schema.json — v2 JSON Schema)
    → E0-T03 (INTERFACE.md — extract from arch spec section 4a)
      → E0-T04 (SPEC.md — deterministic algorithm)
        → E0-T05 (conformance fixtures + expected outputs)
          → E1-T01 (scaffold packages/core)
            → E1-T02 (TypeScript types from INTERFACE.md)
              → E1-T03 (v1→v2 migration)
              → E1-T05 (CKM engine — port from reference, adapt to v2)
                → E1-T06 (terminal formatter)
                → E1-T08 (Commander.js adapter)
                  → E1-T09 (barrel export + subpath exports)
                    → E1-T10 (full conformance suite)
                      → E1-T11 (publish ckm to npm)
```

See `PLAN.md` for every task's full description, acceptance criteria, and dependencies.

### Later Epics (in order):

- **Epic 2**: Standalone CLI (`ckm-cli` — validate, migrate, inspect commands)
- **Epic 3**: TypeScript adapter expansion (Citty, oclif, Clipanion)
- **Epic 4**: Python SDK (Click + Typer adapters)
- **Epic 5**: Rust SDK (Clap adapter)
- **Epic 6**: Go SDK (Cobra + urfave/cli adapters)
- **Epic 7**: forge-ts v2 integration
- **Epic 8**: VersionGuard migration (replace src/ckm/ with ckm dependency)

---

## Conventions

- ESM-only for TypeScript packages
- Vitest for TypeScript testing
- Biome for formatting/linting
- Zero runtime dependencies in core library
- All types defined in INTERFACE.md FIRST, then implemented per language
- Conformance tests are the source of truth for correctness
- Adapters are peer/optional dependencies (never bundled)
- No color/chalk dependencies in core — plain text formatting only

## What NOT to Do

- Do NOT modify `/mnt/projects/versionguard` — separate project, separate concerns
- Do NOT add `@codluv` scope to any package — everything is unscoped root
- Do NOT build a WASM/FFI binary — the backbone is spec-based
- Do NOT manually maintain topics — they are auto-derived from the manifest
- Do NOT skip writing INTERFACE.md and SPEC.md to jump to code — the backbone documents ARE the product

## After Reading This

1. Read `CLAUDE.md`, `VISION.md`, `PLAN.md`, and `docs/specs/CKM-SDK-ARCHITECTURE.md`
2. Read `docs/specs/reference-engine.ts` and `docs/specs/reference-types.ts` to understand the v1 implementation
3. Delete this file (`HANDOFF-PROMPT.md`) — all durable context is in the other documents
4. Start with Epic 0, Task 01 (E0-T01: Initialize monorepo tooling)
