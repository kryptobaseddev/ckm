# CKM SDK Architecture Specification

**Status**: DRAFT
**Date**: 2026-03-29
**Author**: Architecture design session
**Supersedes**: Original spec-based backbone architecture (ADR-001 revised)

---

## 1. Overview

CKM (Codebase Knowledge Manifest) is being extracted from VersionGuard into a standalone, multi-language SDK. Any CLI tool that generates or adopts a `ckm.json` gets batteries-included help, topic browsing, and LLM-consumable structured output — with zero manual topic mapping.

**Core principle**: `ckm.json` is the universal contract. Any generator produces it. A single Rust core consumes it. Thin FFI wrappers expose it to every language. Any adapter wires it into any CLI framework.

```
                    ANY GENERATOR
                   (forge-ts, rustdoc, pydoc, custom)
                          |
                          | generates
                          v
                    +-------------+
                    | ckm.json v2 |  <-- universal contract
                    +-------------+
                          |
                    +-------------+
                    | rust-core   |  <-- THE implementation (SSoT)
                    |  (pure Rust)|     all algorithms, all logic
                    +-------------+
                          |
              +-----------+-----------+-----------+
              |           |           |           |
              v           v           v           v
         napi-rs      PyO3        CGo/WASM    direct dep
         (Node.js)   (Python)     (Go)        (Rust CLI)
              |           |           |           |
         +----+---+   +--+--+       |        ckm-cli
         |  |  |  |   |     |       |        binary
         v  v  v  v   v     v       v
        Cmdr Citty  Click  Typer  Cobra
        ...adapter  adapter      adapter
              |           |           |
              v           v           v
         YOUR CLI    YOUR CLI    YOUR CLI
         (embeds)    (embeds)    (embeds)
```

**Key architectural decision**: The CKM engine is implemented **exactly once** in Rust. Every other language consumes it through thin FFI wrappers (napi-rs for Node.js, PyO3 for Python, CGo/WASM for Go). This eliminates behavioral drift between language SDKs. When the algorithm changes, it changes once.

---

## 2. Package Naming (All Unscoped, Universal)

| Ecosystem | Package | Registry | Binary | Build Pipeline |
|-----------|---------|----------|--------|----------------|
| TypeScript/JS | `ckm-sdk` | npm | — | napi-rs 3 → native `.node` + WASM fallback |
| CLI binary | `ckm-cli` | npm + crates.io | `ckm` | Pure Rust binary (direct rust-core dep) |
| Python | `ckm` | PyPI | — | PyO3 + Maturin → native wheels |
| Rust | `ckm` | crates.io | — | Direct dependency on rust-core |
| Go | `github.com/kryptobaseddev/ckm/go` | Go modules | — | CGo FFI or WASM via wazero |

**No scopes, no org prefixes.** CKM is a universal standard, not a branded utility. The npm package is `ckm-sdk` (the `ckm` name was unavailable). On all other registries it's `ckm`.

**Repository**: `github.com/kryptobaseddev/ckm` (monorepo)

**Build pipeline note**: Unlike the previous spec-based architecture where each language re-implemented the engine, all packages now compile from the same Rust source. The napi-rs, PyO3, and CGo build steps are integrated into CI — a single `cargo build` plus wrapper-specific tooling produces all artifacts.

---

## 3. Monorepo Layout

```
ckm/
  ckm.schema.json                  # ckm.json v2 JSON Schema (INPUT contract)
  INTERFACE.md                     # SDK Interface Definition (documents what rust-core exposes)
  SPEC.md                          # Algorithm specification (documents how rust-core behaves)

  conformance/                     # Cross-language conformance test suite
    fixtures/
      minimal.ckm.json            # Single concept, single operation
      multi-topic.ckm.json        # Multiple config concepts
      no-config.ckm.json          # Concepts without Config suffix
      v1-legacy.ckm.json          # v1 schema (tests migration)
      polyglot.ckm.json           # Language-agnostic types
      edge-cases.ckm.json         # Empty arrays, null defaults, unknown topics
    expected/
      minimal/
        topics.json               # Expected topic derivation output
        topicIndex.json           # Expected getTopicJson() output (no arg)
        topicContent-calver.txt   # Expected getTopicContent("calver") string
        topicJson-calver.json     # Expected getTopicJson("calver") output
        topicJson-unknown.json    # Expected getTopicJson("nonexistent") error output
        inspect.json              # Expected inspect() output
        validate.json             # Expected validateManifest() output
      v1-legacy/
        detectVersion.json        # Expected: 1
        migrateResult.json        # Expected v2 manifest after migration
        topics.json               # Expected topics after auto-migration
      multi-topic/
        ...

  packages/
    rust-core/                     # THE implementation — pure Rust, zero FFI
      Cargo.toml                   # crate name: ckm, published as ckm on crates.io
      src/
        lib.rs                     # Public API surface
        types.rs                   # CkmManifest, CkmConcept, CkmTypeRef, etc.
        engine.rs                  # CkmEngine impl + topic derivation algorithm
        builder.rs                 # CkmManifestBuilder — producer API for generators
        validate.rs                # validate_manifest() — lightweight structural validation
        migrate.rs                 # migrate_v1_to_v2(), detect_version()
        format.rs                  # Terminal text formatters
      tests/
        conformance.rs             # Loads conformance/ fixtures, asserts outputs

    node/                          # Node.js/TypeScript wrapper (npm: ckm-sdk)
      Cargo.toml                   # crate: ckm-node, napi-rs 3 + rust-core dep
      package.json                 # name: ckm-sdk, version: 0.3.1
      src/
        lib.rs                     # #[napi] annotated bridge functions
      index.d.ts                   # Auto-generated by napi-rs (committed or gitignored)
      index.js                     # Auto-generated native binding loader
      js/
        adapters/
          types.ts                 # CkmCliAdapter, CkmAdapterOptions, CkmFormatter
          registry.ts              # ADAPTER_TABLE (lazy-loaded adapter map)
          commander.ts             # Commander.js adapter
          citty.ts                 # Citty adapter
          oclif.ts                 # oclif adapter
          clipanion.ts             # Clipanion adapter

    python/                        # Python wrapper (PyPI: ckm)
      Cargo.toml                   # PyO3 dependency + rust-core workspace dep
      pyproject.toml               # Maturin build backend
      src/
        lib.rs                     # #[pyfunction] / #[pyclass] bridge
      ckm/
        __init__.py                # Re-exports from native module
        _native.pyi                # Type stubs for the native module
        adapters/
          __init__.py              # CkmCliAdapter ABC
          click_adapter.py         # Click adapter
          typer_adapter.py         # Typer adapter (with Rich output)

    go/                            # Go wrapper (module: github.com/kryptobaseddev/ckm/go)
      go.mod
      ckm.go                       # CGo bindings or WASM-based calls to rust-core
      ckm_test.go                  # Conformance tests via Go wrapper
      adapters/
        adapter.go                 # Adapter interface
        cobra.go                   # Cobra adapter
        urfave.go                  # urfave/cli adapter
      internal/
        ffi/                       # CGo bridge or WASM loader (wazero)
          bridge.go
          bridge.h                 # C header for CGo (auto-generated)

    cli-rs/                        # Standalone CLI binary (crates.io: ckm-cli)
      Cargo.toml                   # Direct dependency on rust-core (no FFI)
      src/
        main.rs                    # CLI entry point (browse, validate, migrate, inspect)

    core/                          # OLD TypeScript core (LEGACY, being replaced by node/)
    cli/                           # OLD TypeScript CLI (LEGACY, being replaced by cli-rs/)

  docs/
    specs/
      CKM-SDK-ARCHITECTURE.md     # This document
```

**Rationale for monorepo**: A single repository ensures the Rust core, all FFI wrappers, the JSON Schema, and conformance fixtures stay in lock-step. A change to the Rust core automatically triggers wrapper rebuilds and conformance tests for every language in CI.

**Key difference from previous layout**: The old `packages/core/` TypeScript engine and `packages/cli/` TypeScript CLI are legacy and being replaced. The `packages/node/` wrapper compiles rust-core via napi-rs and exposes it to Node.js/TypeScript consumers. The `packages/cli-rs/` is a pure Rust CLI binary.

---

## 4. The SSoT: Rust Core (ADR-001 Revised)

### Decision: Single Rust Implementation over Spec-Based Backbone

The previous architecture proposed a "spec-based backbone" — three documents (INTERFACE.md, SPEC.md, ckm.schema.json) serving as the source of truth, with each language independently implementing the engine from the spec. This has been **superseded** by a single Rust core that IS the source of truth.

### Why the Change

| Concern | Spec-Based (Old) | Rust Core (New) |
|---------|-------------------|-----------------|
| **Drift risk** | Four implementations diverge on edge cases despite conformance tests | One implementation — drift is impossible |
| **Spec ambiguity** | Prose specs can be interpreted differently | Code is unambiguous |
| **Bug fix cost** | Fix in 4 languages (TS, Python, Rust, Go) | Fix once in Rust |
| **New feature cost** | Implement in 4 languages, update spec, update fixtures | Implement in Rust, wrappers inherit |
| **Conformance burden** | Must maintain extensive fixture suites PER method PER language | Rust core is tested directly; wrappers are thin pass-through |
| **Time to ship all languages** | Sequential: TS first, then Python, Rust, Go (each from scratch) | Parallel: Rust core first, then all wrappers simultaneously (each ~200 LOC) |

### The Original Argument Against Shared Binary

The original ADR-001 rejected "Rust core + WASM" because:

> *"WASM adds 2-10MB to every consumer; Python/Go FFI is fragile; build complexity explodes."*

This was valid for a WASM-only approach. The revised architecture avoids these pitfalls:

1. **napi-rs produces native `.node` files**, not WASM. Binary size is ~1-3MB. WASM is available as a fallback for environments without native compilation, not as the primary distribution.
2. **PyO3 + Maturin produces native wheels**, pre-compiled for major platforms. Users `pip install ckm` and get a native binary — no compilation step.
3. **Go uses CGo (primary) or WASM via wazero (fallback)**. CGo is the standard Go FFI mechanism. wazero provides a pure-Go WASM runtime for environments where CGo is unavailable.
4. **The engine is ~500 LOC of pure data transformation.** The compiled artifact is small. The FFI boundary is simple (JSON in, JSON/string out).

### The Three Backbone Documents: New Role

The backbone documents still exist but their role has changed:

| Document | Old Role | New Role |
|----------|----------|----------|
| `ckm.schema.json` | INPUT (SSoT for manifest structure) | **INPUT (unchanged)** — still the SSoT for what a valid `ckm.json` looks like |
| `INTERFACE.md` | API SURFACE (SSoT — SDKs implement from this) | **DOCUMENTATION** — describes what rust-core exposes. Updated when the Rust API changes, but the Rust code is authoritative. |
| `SPEC.md` | BEHAVIOR (SSoT — SDKs follow this algorithm) | **DOCUMENTATION** — describes how rust-core behaves. Useful for understanding, not for re-implementation. |
| `conformance/` | PROOF (every language runs these) | **PROOF (narrower scope)** — verifies rust-core directly. Wrappers inherit correctness. Wrapper tests verify FFI bridge fidelity, not algorithmic correctness. |

The schema remains a true SSoT because it defines an external contract (the manifest format) that generators must produce. The algorithm spec and interface doc are now documentation of the Rust code, not independent sources of truth.

### Conformance Test Scope

| Layer | What is tested | How |
|-------|---------------|-----|
| **rust-core** | Full algorithmic correctness | `cargo test` loads all conformance fixtures, asserts exact output |
| **node wrapper** | FFI bridge fidelity | Vitest calls wrapper functions, compares to expected output (subset of fixtures) |
| **python wrapper** | FFI bridge fidelity | pytest calls wrapper functions, compares to expected output (subset of fixtures) |
| **go wrapper** | FFI bridge fidelity | `go test` calls wrapper functions, compares to expected output (subset of fixtures) |
| **cli binary** | End-to-end integration | Shell tests invoke `ckm` binary, compare stdout to expected strings |

Wrapper tests are lighter than full conformance suites because the wrappers are pass-through. If rust-core is correct and the bridge marshals data faithfully, the wrapper is correct.

---

## 5. ckm.json v2 Schema Design

### Problems with v1

1. **TypeScript-centric types**: `"CalVerFormat"` is meaningless to Python/Rust consumers
2. **No metadata**: No way to know which tool produced the manifest or the source language
3. **Concept naming tied to TS conventions**: Topic derivation strips `Config` suffix — breaks for `calver_config` (Python) or `CalverConfig` (Rust)
4. **Operation types are all `"unknown"`**: forge-ts cannot resolve cross-module types
5. **Implicit topic derivation**: Engine guesses slugs from naming conventions instead of explicit declaration

### v2 Schema

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

  "concepts": [
    {
      "id": "concept-calver-config",
      "name": "CalVerConfig",
      "slug": "calver",
      "what": "Configures CalVer validation rules.",
      "tags": ["config"],
      "properties": [
        {
          "name": "format",
          "type": {
            "canonical": "string",
            "enum": ["YYYY.MM.DD", "YYYY.MM", "YYYY.0M.0D"],
            "original": "CalVerFormat"
          },
          "description": "Calendar format used when parsing and validating versions.",
          "required": true,
          "default": null
        }
      ]
    }
  ],

  "operations": [
    {
      "id": "op-validate",
      "name": "validate",
      "what": "Validates a CalVer string against formatting and date rules.",
      "tags": ["calver", "validation"],
      "inputs": [
        {
          "name": "version",
          "type": { "canonical": "string" },
          "required": true,
          "description": "Version string to validate."
        }
      ],
      "outputs": {
        "type": { "canonical": "object", "original": "ValidationResult" },
        "description": "A validation result containing any discovered errors."
      }
    }
  ],

  "constraints": [
    {
      "id": "constraint-future-date",
      "rule": "Rejects versions with dates in the future.",
      "enforcedBy": "validate",
      "severity": "error"
    }
  ],

  "workflows": [
    {
      "id": "workflow-release",
      "goal": "Release a new version",
      "tags": ["release"],
      "steps": [
        { "action": "command", "value": "vg validate", "note": "Ensure version is valid" },
        { "action": "manual", "value": "Push tag to remote", "note": "Triggers CI/CD" }
      ]
    }
  ],

  "configSchema": [
    {
      "key": "calver.format",
      "type": { "canonical": "string", "enum": ["YYYY.MM.DD", "YYYY.MM"] },
      "description": "Calendar format for version strings.",
      "default": null,
      "required": true
    }
  ]
}
```

### Key v2 Changes

| Change | v1 | v2 | Rationale |
|--------|----|----|-----------|
| Schema URL | `forge-ts.dev/schemas/ckm/v1.json` | `ckm.dev/schemas/v2.json` | CKM is independent of forge-ts |
| Meta block | `project`, `generated` at top level | `meta.project`, `meta.language`, `meta.generator` | Explicit provenance; language field enables language-aware type display |
| Type representation | Raw string (`"boolean"`) | Object: `{ canonical, original?, enum? }` | Portable canonical set + source fidelity |
| Canonical type set | None | `string`, `boolean`, `number`, `integer`, `array`, `object`, `null`, `any` | Maps to JSON Schema primitives every language can interpret |
| Concept slugs | Derived at runtime (strip `Config`) | Explicit `slug` field in manifest | Generator decides; engine doesn't guess; works for any naming convention |
| Concept tags | None (suffix heuristic) | `tags: string[]` | `["config"]` tag replaces suffix-based heuristic; extensible |
| Operation tags | None (keyword matching) | `tags: string[]` | Explicit topic linkage replaces fragile substring matching |
| Constraint severity | Absent | `"error" \| "warning" \| "info"` | Graduated enforcement display |
| Workflow steps | `{ command?, manual?, note? }` | `{ action: "command"\|"manual", value, note? }` | Discriminated union, cleaner |
| Config keys | `ConceptName.propName` | Dotted path from config root (`calver.format`) | Language-agnostic; mirrors actual config file paths |

### Canonical Type Mapping

| Canonical | TypeScript | Python | Rust | Go |
|-----------|-----------|--------|------|-----|
| `string` | `string` | `str` | `String` | `string` |
| `boolean` | `boolean` | `bool` | `bool` | `bool` |
| `number` | `number` | `float` | `f64` | `float64` |
| `integer` | `number` | `int` | `i64` | `int64` |
| `array` | `T[]` | `list[T]` | `Vec<T>` | `[]T` |
| `object` | `Record/interface` | `dict/dataclass` | `HashMap/struct` | `map/struct` |
| `null` | `null/undefined` | `None` | `Option<T>` | `nil` |
| `any` | `unknown` | `Any` | `serde_json::Value` | `interface{}` |

### v1 -> v2 Migration

The engine accepts both v1 and v2 manifests. When it encounters v1 (detected by `$schema` URL or absence of `meta` block), it runs deterministic migration:

1. Wraps `project`/`generated` into `meta` block, sets `language: "typescript"`, `generator: "unknown"`
2. For each type string, wraps as `{ canonical: infer_canonical(type), original: type }`
3. Derives `slug` from concept name using v1 algorithm (strip `Config`/`Result`/`Options`, lowercase)
4. Infers `tags: ["config"]` for concepts ending in `Config`
5. Rewrites config schema keys from `ConceptName.prop` to `slug.prop`

This migration runs inside the Rust core. FFI wrappers expose it as a single function call.

---

## 6. Rust Core Design

The Rust core (`packages/rust-core/`) is the single implementation of the CKM engine. It contains all business logic: parsing, validation, migration, topic derivation, and formatting. It has **zero FFI concerns** — it is a pure Rust library that other Rust code (including the CLI and the FFI wrapper crates) depends on via Cargo workspace.

### Crate Structure

```
packages/rust-core/
  Cargo.toml
  src/
    lib.rs          # Public API re-exports
    types.rs        # All CKM types (serde-enabled)
    engine.rs       # CkmEngine struct + topic derivation algorithm
    builder.rs      # CkmManifestBuilder (producer API)
    validate.rs     # Lightweight structural validation
    migrate.rs      # v1 -> v2 migration + version detection
    format.rs       # Terminal text formatters
  tests/
    conformance.rs  # Loads conformance fixtures, asserts against expected/
```

### Core Types (types.rs)

All types derive `Serialize`, `Deserialize`, `Clone`, and `Debug`. All use `#[serde(rename_all = "camelCase")]`. This allows seamless JSON round-tripping and inspection.

**Key types** (see `packages/rust-core/src/types.rs` for the authoritative definitions):

- `Extensions` = `HashMap<String, serde_json::Value>` — freeform extension data on any entity
- `CanonicalType` — enum: String, Boolean, Number, Integer, Array, Object, Null, Any
- `CkmTypeRef` — `{ canonical, original?, enum? }`
- `CkmProperty` — `{ name, type, description, required, default? }`
- `CkmConcept` — `{ id, name, slug, what, tags, properties?, rules?, relatedTo?, extensions? }`
- `CkmInput` — `{ name, type, required, description }`
- `CkmOutput` — `{ type, description }`
- `CkmOperation` — `{ id, name, what, tags, preconditions?, inputs?, outputs?, exitCodes?, checksPerformed?, extensions? }`
- `CkmConstraint` — `{ id, rule, enforcedBy, severity, configKey?, default?, security?, extensions? }`
- `Severity` — enum: Error, Warning, Info
- `CkmWorkflowStep` — `{ action, value, note?, expect? }`
- `StepAction` — enum: Command, Manual
- `CkmWorkflow` — `{ id, goal, tags, steps, extensions? }`
- `CkmConfigEntry` — `{ key, type, description, default?, required, effect?, extensions? }`
- `CkmDeclaredTopic` — `{ name, summary, conceptIds, operationIds, constraintIds, configKeys }` — producer-declared topics
- `CkmMeta` — `{ project, language, generator, generated, sourceUrl? }`
- `CkmManifest` — `{ $schema, version, meta, concepts, operations, constraints, workflows, configSchema, topics?, extensions? }`

**Derived types** (computed by the engine):

- `CkmTopic` — `{ name, summary, concepts, operations, configSchema, constraints }`
- `CkmTopicIndexEntry` — `{ name, summary, concepts, operations, configFields, constraints }` (counts)
- `CkmManifestCounts` — `{ concepts, operations, constraints, workflows, configSchema }` (aggregate)
- `CkmTopicIndex` — `{ topics: CkmTopicIndexEntry[], ckm: CkmManifestCounts }`
- `CkmInspectResult` — `{ meta, counts: CkmInspectCounts, topicNames }`
- `CkmInspectCounts` — `{ concepts, operations, constraints, workflows, configKeys, topics }`
- `CkmValidationError` — `{ path, message }`
- `CkmValidationResult` — `{ valid, errors: CkmValidationError[] }`
- `CkmErrorResult` — `{ error, topics }` — returned when topic not found
- `TopicJsonResult` — untagged enum: Index(CkmTopicIndex) | Topic(CkmTopic) | Error(CkmErrorResult)`

### Engine Implementation (engine.rs)

```rust
/// The CKM engine. Immutable after construction.
/// All algorithms run at construction time — queries are lookups.
pub struct CkmEngine {
    manifest: CkmManifest,
    derived_topics: Vec<CkmTopic>,
}

impl CkmEngine {
    /// Create a new engine from a parsed manifest (v1 or v2).
    /// If v1 is detected, auto-migrates to v2 internally.
    /// If manifest has declared `topics`, resolves IDs; otherwise auto-derives.
    /// Topics are derived at construction time.
    pub fn new(data: serde_json::Value) -> Self { /* ... */ }

    /// All derived topics.
    pub fn topics(&self) -> &[CkmTopic] { &self.derived_topics }

    /// Formatted topic index for terminal display (< 300 tokens).
    pub fn topic_index(&self, tool_name: &str) -> String { /* ... */ }

    /// Human-readable content for a specific topic (< 800 tokens).
    /// Returns None if topic not found.
    pub fn topic_content(&self, topic_name: &str) -> Option<String> { /* ... */ }

    /// Structured JSON output. Returns TopicJsonResult enum.
    /// None => TopicJsonResult::Index; Some(name) => Topic or Error.
    pub fn topic_json(&self, topic_name: Option<&str>) -> TopicJsonResult { /* ... */ }

    /// The raw (possibly migrated) manifest.
    pub fn manifest(&self) -> &CkmManifest { &self.manifest }

    /// Manifest statistics and topic summary.
    pub fn inspect(&self) -> CkmInspectResult { /* ... */ }
}
```

Note: `CkmEngine::new()` does **not** return a `Result` — it handles errors internally (defaults to empty manifest on parse failure, auto-migrates v1). This simplifies the FFI boundary.

### Topic Derivation Algorithm (in engine.rs)

The topic derivation algorithm is the heart of CKM. It runs in two modes:

**Mode 1: Producer-declared topics** — When the manifest has a `topics` field, the engine resolves concept/operation/constraint IDs into full topic objects. The generator has full control over grouping.

**Mode 2: Auto-derived topics** — When no `topics` field is present, the engine derives topics from the manifest:

1. **Phase 1 — Every concept with a non-empty slug becomes a topic.** (Not just `*Config` concepts — ALL concepts.) Related concepts with overlapping slugs are merged.
2. **Match operations**: Operations whose `tags` match the topic slug, or whose name/description contains the slug keyword, are grouped under that topic.
3. **Group config entries**: Config schema entries whose `key` starts with `{slug}.` belong to that topic.
4. **Link constraints**: Constraints whose `enforcedBy` references a concept name or matched operation name are included.
5. **Generate summary**: The topic summary is the primary concept's `what` field.
6. **Phase 2 — Unclaimed operations**: Operations not matched to any concept-based topic get their own topics (so nothing is invisible).
7. **Phase 3 — Unclaimed constraints**: Constraints not yet assigned get added to a topic whose operations match `enforcedBy`, or to the first topic as a fallback.

For v1 compatibility (after migration), the same algorithm applies — the migration step populates `slug` and `tags` so the derivation code has a single path.

### Validation (validate.rs)

```rust
/// Validates a JSON value against the ckm.json v2 schema.
pub fn validate_manifest(data: &serde_json::Value) -> CkmValidationResult {
    // Lightweight structural validation — no jsonschema crate dependency.
    // Checks required fields, types, enum values, and returns JSON pointer paths.
    // Returns { valid: true, errors: [] } or { valid: false, errors: [...] }
}
```

Validation uses hand-written structural checks (no external JSON Schema library) to keep dependencies minimal. It validates required fields, correct types, enum values (e.g., severity, canonical types), and returns errors with JSON pointer paths.

### Migration (migrate.rs)

```rust
/// Returns the schema version of a parsed manifest.
/// v2: has "meta" block or $schema contains "v2"
/// v1: everything else (including malformed data)
pub fn detect_version(data: &serde_json::Value) -> u8 { /* ... */ }

/// Deterministic migration from v1 to v2. Best-effort, no errors.
pub fn migrate_v1_to_v2(data: &serde_json::Value) -> CkmManifest { /* ... */ }
```

### Formatting (format.rs)

Terminal output formatters produce plain text (no ANSI color dependencies in the core). The formatter respects token budgets:

- **Topic index**: Max 300 tokens. Aligned columns, truncated if necessary.
- **Topic content**: Max 800 tokens. Concepts, operations, config fields, constraints.
- **JSON output**: Structured `serde_json::Value`, pretty-printed. 1200 tokens for single topic, 3000 for full index.

### Error Handling

The Rust core does **not** have a separate `error.rs` module or `CkmError` enum. Instead:

- `CkmEngine::new()` handles errors internally (defaults to empty manifest on parse failure)
- `validate_manifest()` returns `CkmValidationResult { valid, errors }` — no panics, no Result
- `detect_version()` returns `u8` directly (1 or 2)
- `migrate_v1_to_v2()` returns `CkmManifest` directly (best-effort migration)
- `topic_json()` returns `TopicJsonResult` which includes an `Error` variant with `CkmErrorResult`

This "no-fail" design simplifies the FFI boundary — wrappers don't need to translate Rust error types.

### Dependencies

The Rust core keeps dependencies minimal:

| Crate | Purpose |
|-------|---------|
| `serde` + `serde_json` | JSON serialization/deserialization |

No other runtime dependencies. No `jsonschema` (validation is hand-written). No `thiserror` (no error enum). No async. No networking. Pure data transformation.

---

## 7. FFI Wrapper Architecture

Each language wrapper is a thin bridge between the Rust core and the target language's calling conventions. The wrappers contain **no business logic** — they serialize inputs to JSON, call rust-core functions, and deserialize outputs back to native types.

### 7.1 Node.js Wrapper (napi-rs 3)

**Location**: `packages/node/`

napi-rs generates native `.node` bindings with auto-generated TypeScript declaration files (`.d.ts`). This gives Node.js consumers zero-overhead native calls with full type safety.

#### Bridge Code (src/lib.rs)

The actual bridge exports (see `packages/node/src/lib.rs` for full code):

- **Consumer**: `CkmEngineWrapper` struct with `#[napi]` methods: `get_topic_index()`, `get_topic_content()`, `get_topic_json()`, `get_manifest()`, `inspect()`, `topics_count()`
- **Factory**: `create_ckm_engine(manifest_json: String)` — creates engine from JSON string
- **Standalone**: `validate_manifest(json)`, `migrate_v1_to_v2(json)`, `detect_version(json)`
- **Producer**: `CkmManifestBuilderWrapper` — wraps `CkmManifestBuilder` with `#[napi]` methods: `generator()`, `source_url()`, `add_concept()`, `add_concept_property()`, `add_operation()`, `add_operation_input()`, `add_constraint()`, `add_config()`, `build()`

All functions accept/return JSON strings across the FFI boundary. The `index.js` wrapper parses/stringifies for ergonomic JS usage.

#### JavaScript Ergonomic Layer

The `index.js` file provides a clean API that wraps the native binding, handling JSON serialization:

```javascript
// packages/node/index.js (CommonJS)
const binding = require('./ckm.node');

function createCkmEngine(manifest) {
  const json = typeof manifest === 'string' ? manifest : JSON.stringify(manifest);
  const engine = binding.createCkmEngine(json);
  return {
    get topicsCount() { return engine.topicsCount(); },
    getTopicIndex(toolName) { return engine.getTopicIndex(toolName); },
    getTopicContent(topicName) { return engine.getTopicContent(topicName); },
    getTopicJson(topicName) { return JSON.parse(engine.getTopicJson(topicName)); },
    getManifest() { return JSON.parse(engine.getManifest()); },
    inspect() { return JSON.parse(engine.inspect()); },
  };
}

// Also exports: validateManifest, migrateV1toV2, detectVersion, createManifestBuilder
```

Types are provided via a hand-maintained `index.d.ts` that exports all CKM types (CkmManifest, CkmConcept, etc.) plus the consumer and producer API interfaces.

#### Package Configuration

```json
{
  "name": "ckm-sdk",
  "version": "0.3.1",
  "main": "index.js",
  "types": "index.d.ts",
  "files": ["index.js", "index.d.ts", "ckm.node", "ckm.linux-x64-gnu.node", "README.md"],
  "engines": { "node": ">=18" }
}
```

Currently published with a linux-x64-gnu native binary. Cross-platform builds and WASM fallback are pending.

#### WASM Fallback (Planned)

A WASM fallback for environments where native compilation is unavailable (Stackblitz, CodeSandbox) is planned but not yet implemented. Currently, only the native `.node` binary is published.

### 7.2 Python Wrapper (PyO3 + Maturin)

**Location**: `packages/python/`

PyO3 exposes Rust functions as Python-callable objects. Maturin handles building native wheels for PyPI distribution.

#### Bridge Code (src/lib.rs)

```rust
use pyo3::prelude::*;
use ckm_core::{CkmEngine, validate_manifest, migrate_v1_to_v2, detect_version};

#[pyclass]
struct PyCkmEngine {
    inner: CkmEngine,
}

#[pymethods]
impl PyCkmEngine {
    #[new]
    fn new(manifest_json: &str) -> PyResult<Self> {
        let data: serde_json::Value = serde_json::from_str(manifest_json)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let engine = CkmEngine::new(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(Self { inner: engine })
    }

    fn topic_index(&self, tool_name: &str) -> String {
        self.inner.topic_index(tool_name)
    }

    fn topic_content(&self, topic_name: &str) -> Option<String> {
        self.inner.topic_content(topic_name)
    }

    fn topic_json(&self, topic_name: Option<&str>) -> String {
        let result = self.inner.topic_json(topic_name);
        serde_json::to_string(&result).unwrap()
    }

    fn manifest_json(&self) -> String {
        serde_json::to_string(self.inner.manifest()).unwrap()
    }

    fn inspect_json(&self) -> String {
        serde_json::to_string(&self.inner.inspect()).unwrap()
    }
}

#[pyfunction]
fn validate_manifest_json(data: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(data).unwrap_or_default();
    let result = validate_manifest(&value);
    serde_json::to_string(&result).unwrap()
}

#[pyfunction]
fn migrate_v1_to_v2_json(data: &str) -> PyResult<String> {
    let value: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    let manifest = migrate_v1_to_v2(value)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    Ok(serde_json::to_string(&manifest).unwrap())
}

#[pyfunction]
fn detect_manifest_version(data: &str) -> PyResult<u8> {
    let value: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    detect_version(&value)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[pymodule]
fn ckm_native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCkmEngine>()?;
    m.add_function(wrap_pyfunction!(validate_manifest_json, m)?)?;
    m.add_function(wrap_pyfunction!(migrate_v1_to_v2_json, m)?)?;
    m.add_function(wrap_pyfunction!(detect_manifest_version, m)?)?;
    Ok(())
}
```

#### Python Ergonomic Layer

```python
# packages/python/ckm/__init__.py
"""CKM — Codebase Knowledge Manifest SDK."""

import json
from typing import Any, Optional

from .ckm_native import (
    PyCkmEngine,
    validate_manifest_json,
    migrate_v1_to_v2_json,
    detect_manifest_version,
)


class CkmEngine:
    """The CKM engine. Wraps the native Rust implementation."""

    def __init__(self, manifest: dict | str) -> None:
        data = manifest if isinstance(manifest, str) else json.dumps(manifest)
        self._inner = PyCkmEngine(data)

    def topic_index(self, tool_name: str = "tool") -> str:
        return self._inner.topic_index(tool_name)

    def topic_content(self, topic_name: str) -> str | None:
        return self._inner.topic_content(topic_name)

    def topic_json(self, topic_name: str | None = None) -> Any:
        return json.loads(self._inner.topic_json(topic_name))

    def manifest(self) -> dict:
        return json.loads(self._inner.manifest_json())

    def inspect(self) -> dict:
        return json.loads(self._inner.inspect_json())


def create_engine(manifest: dict | str) -> CkmEngine:
    """Create a CKM engine from a parsed or raw manifest."""
    return CkmEngine(manifest)


def validate_manifest(data: Any) -> dict:
    """Validate a manifest against the v2 schema."""
    raw = data if isinstance(data, str) else json.dumps(data)
    return json.loads(validate_manifest_json(raw))


def migrate_v1_to_v2(manifest: Any) -> dict:
    """Migrate a v1 manifest to v2."""
    raw = manifest if isinstance(manifest, str) else json.dumps(manifest)
    return json.loads(migrate_v1_to_v2_json(raw))


def detect_version(data: Any) -> int:
    """Detect manifest schema version (1 or 2)."""
    raw = data if isinstance(data, str) else json.dumps(data)
    return detect_manifest_version(raw)
```

#### Maturin Configuration

```toml
# packages/python/pyproject.toml
[build-system]
requires = ["maturin>=1.5,<2.0"]
build-backend = "maturin"

[project]
name = "ckm"
requires-python = ">=3.10"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
]

[project.optional-dependencies]
click = ["click>=8.0"]
typer = ["typer>=0.9", "rich>=13.0"]

[tool.maturin]
features = ["pyo3/extension-module"]
python-source = "."
module-name = "ckm.ckm_native"
```

### 7.3 Go Wrapper (CGo / WASM via wazero)

**Location**: `packages/go/`

The Go wrapper has two strategies, selected at build time:

1. **CGo (primary)**: Links the Rust core as a C-compatible shared library. Requires `cgo` and a C compiler but provides native performance.
2. **WASM via wazero (fallback)**: Embeds a WASM binary and uses the pure-Go wazero runtime. No CGo dependency. Slightly slower but fully portable.

#### CGo Bridge

The Rust core exposes a C-compatible API via a thin `cdylib` crate:

```rust
// packages/go/internal/ffi/bridge.rs (separate cdylib crate)
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use ckm_core::CkmEngine;

#[no_mangle]
pub extern "C" fn ckm_engine_new(manifest_json: *const c_char) -> *mut CkmEngine {
    let json = unsafe { CStr::from_ptr(manifest_json) }.to_str().unwrap();
    let data: serde_json::Value = serde_json::from_str(json).unwrap();
    match CkmEngine::new(data) {
        Ok(engine) => Box::into_raw(Box::new(engine)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn ckm_engine_topic_index(
    engine: *const CkmEngine,
    tool_name: *const c_char,
) -> *mut c_char {
    let engine = unsafe { &*engine };
    let name = unsafe { CStr::from_ptr(tool_name) }.to_str().unwrap();
    CString::new(engine.topic_index(name)).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn ckm_engine_free(engine: *mut CkmEngine) {
    if !engine.is_null() {
        unsafe { drop(Box::from_raw(engine)); }
    }
}

#[no_mangle]
pub extern "C" fn ckm_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}
```

#### Go Consumer

```go
// packages/go/ckm.go
package ckm

/*
#cgo LDFLAGS: -L${SRCDIR}/lib -lckm_ffi
#include "internal/ffi/bridge.h"
*/
import "C"
import (
    "encoding/json"
    "unsafe"
)

type Engine struct {
    ptr unsafe.Pointer
}

func NewEngine(manifest interface{}) (*Engine, error) {
    data, err := json.Marshal(manifest)
    if err != nil {
        return nil, err
    }
    cStr := C.CString(string(data))
    defer C.free(unsafe.Pointer(cStr))

    ptr := C.ckm_engine_new(cStr)
    if ptr == nil {
        return nil, fmt.Errorf("failed to create CKM engine")
    }
    return &Engine{ptr: unsafe.Pointer(ptr)}, nil
}

func (e *Engine) TopicIndex(toolName string) string {
    cName := C.CString(toolName)
    defer C.free(unsafe.Pointer(cName))
    result := C.ckm_engine_topic_index((*C.CkmEngine)(e.ptr), cName)
    defer C.ckm_string_free(result)
    return C.GoString(result)
}

func (e *Engine) Close() {
    if e.ptr != nil {
        C.ckm_engine_free((*C.CkmEngine)(e.ptr))
        e.ptr = nil
    }
}
```

#### WASM Alternative

For environments where CGo is undesirable, the Go wrapper can use wazero to execute a WASM build of rust-core. The WASM binary is embedded via `go:embed` and loaded at init time. The API surface is identical — only the internal transport differs.

---

## 8. Adapter Interface Design

### Design Philosophy

Adapters translate the CKM engine API into framework-specific CLI registration. They live in the **wrapper layer** (TypeScript adapters in `packages/node/js/adapters/`, Python adapters in `packages/python/ckm/adapters/`, etc.). Adapters call through the FFI wrapper to rust-core — they contain no engine logic, only framework-specific glue.

This mirrors VG's `VersionSourceProvider` + `DETECTION_TABLE` pattern, applied to CLI framework registration:

- **VG Sources**: `VersionSourceProvider` interface + `DETECTION_TABLE` maps files to factories
- **CKM Adapters**: `CkmCliAdapter` interface + `ADAPTER_TABLE` maps framework IDs to adapter factories

### TypeScript Adapter Interface

```typescript
interface CkmCliAdapter<TProgram = unknown> {
  readonly name: string;        // e.g., 'commander', 'citty', 'oclif'
  readonly framework: string;   // e.g., 'Commander.js'

  register(
    program: TProgram,
    engine: CkmEngine,
    options?: CkmAdapterOptions,
  ): void;
}

interface CkmAdapterOptions {
  commandName?: string;         // default: 'ckm'
  toolName?: string;            // default: inferred from program
  extraFlags?: CkmExtraFlag[];
  formatter?: CkmFormatter;
}

interface CkmFormatter {
  formatIndex(topics: unknown[], toolName: string): string;
  formatTopic(topic: unknown): string;
  formatJson(data: unknown): string;
}
```

### Adapter Table (Lazy-Loaded)

```typescript
const ADAPTER_TABLE: Record<string, () => Promise<CkmCliAdapter>> = {
  commander:  () => import('./adapters/commander.js').then(m => m.default),
  citty:      () => import('./adapters/citty.js').then(m => m.default),
  oclif:      () => import('./adapters/oclif.js').then(m => m.default),
  clipanion:  () => import('./adapters/clipanion.js').then(m => m.default),
};
```

Adapters are lazy-loaded so `npm install ckm-sdk` does NOT force a dependency on any CLI framework. Only the adapter you import pulls in that framework.

### Commander.js Adapter (Reference Implementation)

```typescript
// packages/node/js/adapters/commander.ts
import type { Command } from 'commander';
import type { CkmEngine, CkmCliAdapter, CkmAdapterOptions } from '../index.js';

const adapter: CkmCliAdapter<Command> = {
  name: 'commander',
  framework: 'Commander.js',

  register(program, engine, options) {
    const cmdName = options?.commandName ?? 'ckm';
    const toolName = options?.toolName ?? program.name();

    program
      .command(`${cmdName} [topic]`)
      .description('Codebase Knowledge Manifest -- auto-generated docs and help')
      .option('--json', 'Machine-readable CKM output for LLM agents')
      .option('--llm', 'Full API context')
      .action((topic, flags) => {
        if (flags.json) {
          console.log(JSON.stringify(engine.topicJson(topic), null, 2));
        } else if (topic) {
          const content = engine.topicContent(topic);
          if (!content) {
            console.error(`Unknown topic: ${topic}`);
            console.log(engine.topicIndex(toolName));
            process.exit(1);
          }
          console.log(content);
        } else {
          console.log(engine.topicIndex(toolName));
        }
      });
  },
};

export default adapter;
```

### Python: Click Adapter

```python
# packages/python/ckm/adapters/click_adapter.py
import json
import click
from ..ckm import CkmEngine

class ClickAdapter:
    name = "click"
    framework = "Click"

    def register(self, group, engine: CkmEngine, *, command_name="ckm", tool_name=None):
        tool = tool_name or group.name or "tool"

        @group.command(name=command_name)
        @click.argument("topic", required=False)
        @click.option("--json", "as_json", is_flag=True)
        def ckm_command(topic, as_json):
            if as_json:
                click.echo(json.dumps(engine.topic_json(topic), indent=2))
            elif topic:
                content = engine.topic_content(topic)
                if content is None:
                    click.echo(f"Unknown topic: {topic}", err=True)
                    click.echo(engine.topic_index(tool))
                    raise SystemExit(1)
                click.echo(content)
            else:
                click.echo(engine.topic_index(tool))
```

### Rust: Clap Adapter (in rust-core, feature-gated)

The Rust adapter lives in rust-core itself (not a wrapper) since Rust consumers use the crate directly:

```rust
// packages/rust-core/src/adapters/clap_adapter.rs
// Feature-gated: ckm = { features = ["clap"] }

use clap::{Arg, ArgAction, Command};
use crate::CkmEngine;

pub fn register_ckm_command(engine: &CkmEngine, command_name: &str) -> Command {
    Command::new(command_name)
        .about("Codebase Knowledge Manifest -- auto-generated docs and help")
        .arg(Arg::new("topic").help("Topic to display"))
        .arg(
            Arg::new("json")
                .long("json")
                .action(ArgAction::SetTrue)
                .help("Machine-readable CKM output for LLM agents"),
        )
}

pub fn handle_ckm_matches(
    engine: &CkmEngine,
    matches: &clap::ArgMatches,
    tool_name: &str,
) {
    let topic = matches.get_one::<String>("topic").map(|s| s.as_str());
    let as_json = matches.get_flag("json");

    if as_json {
        let result = engine.topic_json(topic);
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else if let Some(name) = topic {
        match engine.topic_content(name) {
            Some(content) => println!("{}", content),
            None => {
                eprintln!("Unknown topic: {}", name);
                println!("{}", engine.topic_index(tool_name));
                std::process::exit(1);
            }
        }
    } else {
        println!("{}", engine.topic_index(tool_name));
    }
}
```

### Go: Cobra Adapter

```go
// packages/go/adapters/cobra.go
package adapters

import (
    "encoding/json"
    "fmt"
    "os"

    "github.com/kryptobaseddev/ckm/go"
    "github.com/spf13/cobra"
)

type CobraAdapter struct{}

func (a *CobraAdapter) Name() string      { return "cobra" }
func (a *CobraAdapter) Framework() string { return "Cobra" }

func (a *CobraAdapter) Register(engine *ckm.Engine, opts *RegisterOptions) *cobra.Command {
    cmdName := "ckm"
    toolName := "tool"
    if opts != nil {
        if opts.CommandName != "" { cmdName = opts.CommandName }
        if opts.ToolName != "" { toolName = opts.ToolName }
    }

    var asJSON bool
    cmd := &cobra.Command{
        Use:   fmt.Sprintf("%s [topic]", cmdName),
        Short: "Codebase Knowledge Manifest -- auto-generated docs and help",
        Args:  cobra.MaximumNArgs(1),
        Run: func(cmd *cobra.Command, args []string) {
            if asJSON {
                topic := ""
                if len(args) > 0 { topic = args[0] }
                result := engine.TopicJSON(topic)
                out, _ := json.MarshalIndent(result, "", "  ")
                fmt.Println(string(out))
            } else if len(args) > 0 {
                content := engine.TopicContent(args[0])
                if content == "" {
                    fmt.Fprintf(os.Stderr, "Unknown topic: %s\n", args[0])
                    fmt.Println(engine.TopicIndex(toolName))
                    os.Exit(1)
                }
                fmt.Println(content)
            } else {
                fmt.Println(engine.TopicIndex(toolName))
            }
        },
    }
    cmd.Flags().BoolVar(&asJSON, "json", false, "Machine-readable CKM output")
    return cmd
}
```

### Adapter Pattern Across Languages

| Concern | TypeScript (Node) | Python | Rust | Go |
|---------|-------------------|--------|------|-----|
| Interface | `interface CkmCliAdapter` | `class ClickAdapter` | feature-gated functions | `type Adapter interface` |
| Registration | `adapter.register(program, engine)` | `adapter.register(group, engine)` | `register_ckm_command(&engine)` | `adapter.Register(engine, opts)` |
| Lazy loading | Dynamic `import()` | Standard Python imports | Feature flags in Cargo.toml | Build tags |
| Framework dep | peerDependency | optional extra (`ckm[click]`) | optional feature | go module |
| Engine access | Via FFI wrapper | Via FFI wrapper | Direct Rust call | Via CGo/WASM wrapper |

---

## 9. Standalone CLI Design

The `ckm-cli` package provides a standalone binary. It is a **pure Rust binary** that depends directly on rust-core — no FFI overhead. Published to both crates.io and npm (via platform-specific binaries).

```bash
ckm browse [topic] [--json] [--file <path>]   # Browse/query CKM data
ckm validate <file>                            # Validate a ckm.json against v2 schema
ckm migrate <file> [--dry-run] [--output <path>]  # Migrate v1 -> v2
ckm inspect <file>                             # Show manifest stats
```

### CLI Implementation (packages/cli-rs/)

The CLI uses Clap with subcommands:

```
ckm browse [topic] [--json] [--file <path>]   # Browse/query CKM data
ckm validate <file>                             # Validate against v2 schema
ckm migrate <file> [--dry-run] [--output <path>] # Migrate v1 -> v2
ckm inspect <file>                              # Show manifest stats
```

See `packages/cli-rs/src/main.rs` for the full implementation. Depends directly on `ckm` crate (no FFI). Published as `ckm-cli` on crates.io (not yet published).

### Progressive Disclosure (Mandatory Protocol Requirement)

Every adapter MUST support all four disclosure levels:

| Level | Command | Audience | Max Token Budget |
|-------|---------|----------|-----------------|
| 0 | `ckm` | Human / Agent discovery | 300 |
| 1 | `ckm <topic>` | Human / Agent drill-down | 800 |
| 1J | `ckm <topic> --json` | Agent structured | 1200 |
| 2 | `ckm --json` | Agent full index | 3000 |

Progressive disclosure is enforced by the **formatting functions in rust-core** (`format.rs`). Since all adapters call through to the same Rust formatters, token budget compliance is guaranteed by the core — adapters cannot accidentally exceed budgets.

### File Resolution Order

1. Explicit `--file` flag
2. `./ckm.json`
3. `./docs/ckm.json`
4. `./.ckm/ckm.json`

### Example Output

```bash
$ ckm inspect docs/ckm.json
Project:    my-tool
Language:   typescript
Generator:  forge-ts@0.21.1

Concepts:     14 (8 config, 6 result)
Operations:   12
Constraints:   0
Workflows:     0
Config keys:  32
Topics:        8 (auto-derived)
```

---

## 10. Rollout Phases

### Phase 1: Rust Core + Node.js Wrapper

**Scope**: Pure Rust core library, napi-rs Node.js wrapper, Commander adapter, conformance tests, v2 schema.

**Deliverables**:
- `packages/rust-core/` — Full engine implementation
- `ckm.schema.json` — v2 JSON Schema
- `conformance/` — Test fixtures with expected outputs
- `packages/node/` — napi-rs wrapper with TypeScript types
- Commander.js adapter (production-ready)
- `ckm-sdk` on npm (native `.node` — WASM fallback pending)
- `ckm` on crates.io (Rust core library, v0.1.0)

**Exit criteria**: `npm install ckm-sdk` works. `const { createCkmEngine } = require('ckm-sdk')` returns a working engine backed by native Rust. `cargo add ckm` works for Rust consumers. All conformance fixtures pass via `cargo test`.

### Phase 2: Python Wrapper

**Scope**: PyO3 + Maturin wrapper, Click and Typer adapters.

**Deliverables**:
- `packages/python/` — PyO3 wrapper with Python type stubs
- Click adapter + Typer adapter (with Rich output)
- `ckm` on PyPI (native wheels for major platforms)

**Exit criteria**: `pip install ckm` works. `from ckm import create_engine` returns a working engine backed by native Rust. Click/Typer adapters register correctly.

### Phase 3: Go Wrapper

**Scope**: CGo or WASM wrapper, Cobra and urfave/cli adapters.

**Deliverables**:
- `packages/go/` — CGo bridge or WASM-based wrapper
- Cobra adapter + urfave/cli adapter
- Go module published

**Exit criteria**: `go get github.com/kryptobaseddev/ckm/go` works. Engine functions callable from Go. Adapters register correctly.

### Phase 4: Standalone CLI Binary

**Scope**: Pure Rust CLI binary.

**Deliverables**:
- `packages/cli-rs/` — Clap-based CLI
- `ckm-cli` on crates.io (Rust binary)
- `ckm-cli` on npm (optional, platform-specific binaries)
- Commands: `ckm browse [topic]`, `ckm validate`, `ckm migrate`, `ckm inspect`

**Exit criteria**: `cargo install ckm-cli` works. `npx ckm-cli` works. All CLI commands produce correct output.

### Phase 5: forge-ts v2 Integration

**Scope**: forge-ts generates v2 `ckm.json` natively.

**Deliverables**:
- forge-ts generates v2 by default (`--ckm-version 1` for legacy)
- `forge-ts build` populates `concept.slug`, `concept.tags`, canonical types, operation tags
- forge-ts resolves enum values into `type.enum`

**Exit criteria**: `forge-ts build` produces valid v2 `ckm.json` that passes `ckm validate`.

### Phase 6: VersionGuard Migration

**Scope**: Replace VersionGuard's `src/ckm/` with the published `ckm` npm package.

**Deliverables**:
- VersionGuard depends on `ckm` as a library
- `src/ckm/` directory removed
- Commander adapter used for CLI registration

**Exit criteria**: `vg ckm` command works identically. All VersionGuard tests pass. Build succeeds.

### Phase Comparison: Old vs. New

| Phase | Old (Spec-Based) | New (Rust Core) |
|-------|-------------------|-----------------|
| 1 | TypeScript core (re-implement from spec) | Rust core + napi-rs Node.js wrapper |
| 2 | TypeScript adapter expansion | PyO3 Python wrapper |
| 3 | Python SDK (re-implement from spec) | Go wrapper |
| 4 | Rust SDK (re-implement from spec) | Standalone CLI (pure Rust) |
| 5 | Go SDK (re-implement from spec) | forge-ts v2 integration |
| 6 | forge-ts integration | VersionGuard migration |

The new approach eliminates the "re-implement from spec" burden for phases 2-5. Each wrapper is ~200-400 LOC of FFI bridge code, not a full engine re-implementation.

---

## 11. Architectural Decision Records

### ADR-001: Rust Core SSoT over Spec-Based Backbone (Revised)

**Decision**: The CKM engine is implemented once in Rust. All other languages consume it through FFI wrappers.

**Supersedes**: Original ADR-001 ("Spec-Based Backbone over Shared Binary"), which proposed independent per-language implementations guided by spec documents.

**Rationale**: The original decision rejected a shared binary because "WASM adds 2-10MB to every consumer; Python/Go FFI is fragile; build complexity explodes." This was valid for a WASM-only approach. Modern FFI tooling (napi-rs, PyO3, CGo) provides native bindings with minimal overhead:

- **napi-rs**: Generates native `.node` files (~1-3MB), auto-generates `.d.ts`, zero-cost function calls
- **PyO3 + Maturin**: Produces pre-compiled wheels for major platforms, standard `pip install`
- **CGo**: Standard Go FFI mechanism; wazero provides pure-Go WASM fallback

The engine is ~500 LOC of pure data transformation. Four independent implementations would inevitably diverge on edge cases. One Rust implementation with thin wrappers eliminates drift entirely.

**Trade-offs accepted**:
- Build pipeline is more complex (Rust toolchain required for all wrapper builds)
- CI must cross-compile for multiple targets
- Contributors need Rust knowledge to modify engine logic

**Trade-offs gained**:
- Zero behavioral drift between languages
- One bugfix fixes all languages
- Wrapper code is trivial (~200 LOC each)
- Conformance testing is narrower (core only; wrappers test bridge fidelity)

### ADR-002: Explicit Slugs and Tags (Generator Responsibility)

**Decision**: `ckm.json` generator populates `concept.slug` and `concept.tags` explicitly. Engine uses them directly.

**Rationale**: v1's suffix-stripping heuristic only works for TypeScript naming conventions. Explicit slugs work for any language. This decision is unchanged by the Rust core migration — the schema contract is independent of the implementation language.

### ADR-003: Canonical Type System

**Decision**: Types use `{ canonical, original?, enum? }` structure.

**Rationale**: `"CalVerFormat"` is meaningless to Python. Canonical types map to JSON Schema primitives every language understands. `original` preserves source fidelity. Unchanged.

### ADR-004: Monorepo with Per-Language Publishing

**Decision**: Single monorepo, each language publishes independently.

**Rationale**: Now even more critical with the Rust core architecture — rust-core, all wrappers, schema, and conformance tests share a Cargo workspace. A single CI pipeline builds and tests everything.

### ADR-005: Adapters as Peer/Optional Dependencies

**Decision**: Framework adapters are peer/optional dependencies. `npm install ckm-sdk` does NOT install Commander/Citty/oclif/Clipanion. `pip install ckm` does NOT install Click/Typer.

**Rationale**: Zero unnecessary transitive dependencies. Users already have their CLI framework installed. Unchanged.

### ADR-006: Progressive Disclosure as Conformance Requirement

**Decision**: Four disclosure levels are mandatory. Token budgets are enforced by the formatting functions in rust-core.

**Rationale**: CKM's primary consumers are LLM agents under token budgets. Dumping full manifests at level 0 defeats the purpose. With the Rust core architecture, budget enforcement is guaranteed — all output flows through the same Rust formatters regardless of which wrapper or adapter is used.

### ADR-007: napi-rs for Node.js over wasm-bindgen

**Decision**: The Node.js wrapper uses napi-rs 3 (published as `ckm-sdk` on npm) for native bindings, not wasm-bindgen.

**Rationale**:

| Approach | Binary Size | Call Overhead | Type Safety | DX |
|----------|-------------|---------------|-------------|-----|
| wasm-bindgen | 2-10MB WASM blob | WASM VM overhead per call | Manual TS types | Requires wasm-pack toolchain |
| napi-rs | 1-3MB native `.node` | Zero-cost native calls | Auto-generated `.d.ts` | Standard npm publish |

napi-rs produces native Node.js addons that load as shared libraries — no WASM VM, no startup cost, no memory overhead. Type declarations are auto-generated from the Rust source, ensuring TypeScript consumers get accurate types without manual maintenance.

WASM is available as a **fallback** for browser/sandboxed environments (Stackblitz, CodeSandbox) but is not the primary distribution channel.

---

## 12. Related Decisions

### VersionGuard Package Rename

`@codluv/versionguard` will be renamed to `versionguard` (unscoped) as part of the v1.0.0 release. Both `versionguard` and `ckm` are available on npm.

| Package | Registry | Current | New |
|---------|----------|---------|-----|
| VersionGuard | npm | `@codluv/versionguard` | `versionguard` |
| CKM SDK | npm | (does not exist) | `ckm-sdk` (napi-rs native) |

### Relationship to forge-ts

- **forge-ts** = generation (produces `ckm.json` from source code)
- **CKM SDK** = consumption/display (reads `ckm.json`, provides help/topics)
- forge-ts is one possible generator — any tool can produce a valid `ckm.json`
- Phase 5 adds forge-ts v2 support, but CKM works independently

### Crate Dependencies

Each wrapper crate depends on `ckm` (the rust-core crate) via path:

```toml
# packages/node/Cargo.toml (crate: ckm-node)
[dependencies]
ckm = { path = "../rust-core" }
napi = { version = "3", features = ["full"] }
napi-derive = "3"
serde_json = "1"

# packages/cli-rs/Cargo.toml (crate: ckm-cli)
[dependencies]
ckm = { path = "../rust-core" }
clap = { version = "4", features = ["derive"] }
serde_json = "1"
```

Note: There is no root `Cargo.toml` workspace yet — each crate is built independently. Adding a workspace is a future improvement.

### Language Mapping Table

Every type/method in the Rust core maps to each language wrapper like this:

| Rust Core | Node.js (via napi-rs, `ckm-sdk`) | Python (via PyO3) | Go (via CGo/WASM) |
|-----------|-----------------------------------|--------------------|--------------------|
| `CkmEngine::new(data)` | `createCkmEngine(manifest)` | `create_engine(manifest)` | `ckm.NewEngine(manifest)` |
| `engine.topics()` | `engine.topicsCount` (count only) | — | — |
| `engine.topic_index(name)` | `engine.getTopicIndex(name)` | `engine.topic_index(name)` | `engine.TopicIndex(name)` |
| `engine.topic_content(name)` | `engine.getTopicContent(name)` | `engine.topic_content(name)` | `engine.TopicContent(name)` |
| `engine.topic_json(name)` | `engine.getTopicJson(name)` | `engine.topic_json(name)` | `engine.TopicJSON(name)` |
| `engine.manifest()` | `engine.getManifest()` | `engine.manifest()` | `engine.Manifest()` |
| `engine.inspect()` | `engine.inspect()` | `engine.inspect()` | `engine.Inspect()` |
| `validate_manifest(data)` | `validateManifest(data)` | `validate_manifest(data)` | `ckm.ValidateManifest(data)` |
| `migrate_v1_to_v2(data)` | `migrateV1toV2(manifest)` | `migrate_v1_to_v2(manifest)` | `ckm.MigrateV1ToV2(manifest)` |
| `detect_version(data)` | `detectVersion(data)` | `detect_version(data)` | `ckm.DetectVersion(data)` |
| `CkmManifestBuilder::new()` | `createManifestBuilder(project, lang)` | — (not yet) | — (not yet) |
| `Option<T>` | `T \| null` | `T \| None` | `*T` (pointer) |

The mapping follows each language's conventions. The FFI bridge handles all serialization — callers never interact with raw pointers or C strings. Note: Python and Go wrappers are scaffolded but not yet fully implemented.
