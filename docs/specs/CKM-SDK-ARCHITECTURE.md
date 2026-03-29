# CKM SDK Architecture Specification

**Status**: DRAFT
**Date**: 2026-03-29
**Author**: Architecture design session

---

## 1. Overview

CKM (Codebase Knowledge Manifest) is being extracted from VersionGuard into a standalone, multi-language SDK. Any CLI tool that generates or adopts a `ckm.json` gets batteries-included help, topic browsing, and LLM-consumable structured output — with zero manual topic mapping.

**Core principle**: `ckm.json` is the universal contract. Any generator produces it. Any SDK consumes it. Any adapter wires it into any CLI framework.

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
              +-----------+-----------+
              |           |           |
              v           v           v
         ckm (npm)   ckm (PyPI)  ckm (crates.io)
         core lib    core lib    core lib
              |           |           |
     +--------+--+   +---+---+   +---+---+
     |  |  |  |  |   |       |   |       |
     v  v  v  v  v   v       v   v       v
    Cmdr Citty ...  Click  Typer Clap   ...
    adapter         adapter      adapter
              |           |           |
              v           v           v
         YOUR CLI    YOUR CLI    YOUR CLI
         (embeds)    (embeds)    (embeds)
```

---

## 2. Package Naming (All Unscoped, Universal)

| Ecosystem | Package | Registry | Binary |
|-----------|---------|----------|--------|
| TypeScript/JS | `ckm` | npm | — |
| CLI binary | `ckm-cli` | npm | `ckm` |
| Python | `ckm` | PyPI | `ckm` |
| Rust | `ckm` | crates.io | `ckm` |
| Go | `github.com/kryptobaseddev/ckm/go` | Go modules | `ckm` |

**No scopes, no org prefixes.** CKM is a universal standard, not a branded utility. Using unscoped `ckm` on every registry communicates that any project — regardless of language, framework, or toolchain — can generate and consume a `ckm.json`.

**Repository**: `github.com/kryptobaseddev/ckm` (monorepo)

---

## 3. Monorepo Layout

```
ckm/
  INTERFACE.md                   # SDK Interface Definition (types + methods — the API contract)
  SPEC.md                        # Deterministic algorithm specification (the behavior)
  ckm.schema.json                # ckm.json v2 JSON Schema (the input contract)

  conformance/                   # Cross-language conformance test suite
    fixtures/
      minimal.ckm.json          # Single concept, single operation
      multi-topic.ckm.json      # Multiple config concepts
      no-config.ckm.json        # Concepts without Config suffix
      v1-legacy.ckm.json        # v1 schema (tests migration)
      polyglot.ckm.json         # Language-agnostic types
    expected/
      minimal/
        topics.json             # Expected topic derivation output
        index.json              # Expected getTopicJson() output
        topic-calver.json       # Expected getTopicJson('calver') output
      multi-topic/
        ...

  packages/
    core/                        # TypeScript core library (npm: ckm)
      package.json
      src/
        types.ts                 # CkmManifest, CkmConcept, CkmTypeRef, etc. (from INTERFACE.md)
        engine.ts                # CkmEngine impl + createCkmEngine() factory
        validate.ts              # validateManifest() — JSON Schema validation
        migrate.ts               # migrateV1toV2(), detectVersion()
        format.ts                # Terminal formatter (plain text, no chalk dep)
        adapters/
          types.ts               # CkmCliAdapter, CkmAdapterOptions, CkmFormatter interfaces
          registry.ts            # ADAPTER_TABLE (lazy-loaded adapter map)
          commander.ts           # Commander.js adapter
          citty.ts               # Citty adapter
          oclif.ts               # oclif adapter
          clipanion.ts           # Clipanion adapter
        index.ts                 # Public API barrel

    cli/                         # Standalone CLI binary (npm: ckm-cli)
      package.json
      src/
        main.ts

    python/                      # Python SDK (PyPI: ckm)
      pyproject.toml
      ckm/
        __init__.py
        types.py                 # @dataclass types (from INTERFACE.md)
        engine.py                # CkmEngine class + create_engine()
        validate.py              # validate_manifest()
        migrate.py               # migrate_v1_to_v2(), detect_version()
        format.py                # Terminal + JSON formatters
        adapters/
          __init__.py            # CkmCliAdapter ABC
          click_adapter.py
          typer_adapter.py
        cli.py                   # Standalone CLI entry

    rust/                        # Rust SDK (crates.io: ckm)
      Cargo.toml
      src/
        lib.rs
        types.rs                 # #[derive(Deserialize)] types (from INTERFACE.md)
        engine.rs                # impl CkmEngine + CkmEngine::new()
        validate.rs              # fn validate_manifest()
        migrate.rs               # fn migrate_v1_to_v2(), fn detect_version()
        format.rs                # Terminal + JSON formatters
        adapters/
          mod.rs                 # trait CkmCliAdapter
          clap_adapter.rs
        main.rs                  # Optional CLI binary

    go/                          # Go SDK (go module)
      go.mod
      types.go                   # struct types (from INTERFACE.md)
      engine.go                  # func NewEngine(), methods
      validate.go                # func ValidateManifest()
      migrate.go                 # func MigrateV1ToV2(), func DetectVersion()
      format.go                  # Terminal + JSON formatters
      adapters/
        adapter.go               # Adapter interface
        cobra.go
        urfave.go
      cmd/ckm/main.go
```

**Rationale for monorepo**: A single repository ensures the JSON Schema, the algorithm specification, and all language implementations stay in lock-step. The spec changes in one commit, all implementations update in the same PR.

---

## 4. The Backbone: Spec-Based Protocol (ADR-001)

### Decision: JSON Schema + Deterministic Algorithm Specification

The backbone is NOT a compiled binary. It is:

1. **`ckm.schema.json`** — a versioned JSON Schema that defines the `ckm.json` contract (INPUT)
2. **`INTERFACE.md`** — the SDK Interface Definition: types, methods, return shapes every SDK must expose (API SURFACE)
3. **`SPEC.md`** — a prose specification that defines the deterministic topic derivation algorithm (BEHAVIOR)
4. **`conformance/`** — a test suite (JSON fixtures + expected output) that any implementation must pass (PROOF)

Each language implements the algorithm natively, in its idiomatic style.

### Why NOT a shared binary?

| Option | Verdict | Why |
|--------|---------|-----|
| Rust core + WASM | Rejected | WASM adds 2-10MB to every consumer; Python/Go FFI is fragile; build complexity explodes |
| TypeScript core + WASM | Rejected | Same WASM overhead; Go/Rust developers would never accept a JS runtime dependency |
| JSON Schema + Spec | **Selected** | Zero runtime dependency between languages; each SDK is idiomatic; smallest footprint |

**Key insight**: The CKM engine is ~500 LOC of pure data transformation — parse JSON, filter by naming convention, derive slugs, match by substring, format output. This is entirely deterministic string processing. Shipping a WASM binary for this is like shipping a Docker container to run `echo`.

### The Four Backbone Documents

| Document | Role | What it defines |
|----------|------|-----------------|
| `ckm.schema.json` | **INPUT** | What goes into the engine (the `ckm.json` structure) |
| `INTERFACE.md` | **API SURFACE** | What every SDK must expose: types, methods, return shapes |
| `SPEC.md` | **BEHAVIOR** | How the engine transforms input to output (deterministic algorithm) |
| `conformance/` | **PROOF** | Test fixtures that prove an implementation is correct |

An SDK is conformant when it implements every type and method from `INTERFACE.md`, follows the algorithm in `SPEC.md`, and passes every fixture in `conformance/`. No more, no less.

### Conformance Guarantee

The spec includes a conformance test suite: a set of `ckm.json` input files paired with expected output JSON for every operation. Each language SDK runs these tests in CI. Same pattern as the JSON Schema Test Suite and the TOML test suite.

---

## 4a. SDK Interface Definition (INTERFACE.md)

This is the contract every language SDK must implement. Defined once, language-agnostic, enforced by conformance tests. Every SDK ships these exact types and methods — the names adapt to language conventions (camelCase/snake_case/PascalCase) but the **shape is identical**.

### Guiding Principle

> Define the interface in the spec. Implement it per language. Prove it with conformance tests. No SDK may add, remove, or rename a method on the engine without updating INTERFACE.md first.

### SSoT Type Definitions (Language-Agnostic)

These are defined in `INTERFACE.md` as the canonical shapes. Each language maps them to its idiomatic equivalent.

#### Input Types (from ckm.json v2)

```
TYPE CkmManifest {
  meta:         CkmMeta
  concepts:     CkmConcept[]
  operations:   CkmOperation[]
  constraints:  CkmConstraint[]
  workflows:    CkmWorkflow[]
  configSchema: CkmConfigEntry[]
}

TYPE CkmMeta {
  project:    string
  language:   string
  generator:  string
  generated:  string (ISO 8601)
  sourceUrl:  string? (optional)
}

TYPE CkmConcept {
  id:         string
  name:       string
  slug:       string
  what:       string
  tags:       string[]
  properties: CkmProperty[]?
}

TYPE CkmProperty {
  name:        string
  type:        CkmTypeRef
  description: string
  required:    boolean
  default:     string? (nullable)
}

TYPE CkmTypeRef {
  canonical:  CanonicalType           # "string" | "boolean" | "number" | "integer" | "array" | "object" | "null" | "any"
  original:   string?                 # Source language annotation (e.g., "CalVerFormat")
  enum:       string[]?               # Known values (e.g., ["YYYY.MM.DD", "YYYY.MM"])
}

ENUM CanonicalType = "string" | "boolean" | "number" | "integer" | "array" | "object" | "null" | "any"

TYPE CkmOperation {
  id:      string
  name:    string
  what:    string
  tags:    string[]
  inputs:  CkmInput[]?
  outputs: CkmOutput?
}

TYPE CkmInput {
  name:        string
  type:        CkmTypeRef
  required:    boolean
  description: string
}

TYPE CkmOutput {
  type:        CkmTypeRef
  description: string
}

TYPE CkmConstraint {
  id:         string
  rule:       string
  enforcedBy: string
  severity:   "error" | "warning" | "info"
}

TYPE CkmWorkflow {
  id:    string
  goal:  string
  tags:  string[]
  steps: CkmWorkflowStep[]
}

TYPE CkmWorkflowStep {
  action: "command" | "manual"
  value:  string
  note:   string?
}

TYPE CkmConfigEntry {
  key:         string               # Dotted path (e.g., "calver.format")
  type:        CkmTypeRef
  description: string
  default:     string? (nullable)
  required:    boolean
}
```

#### Derived Types (computed by the engine)

```
TYPE CkmTopic {
  name:        string               # Slug used as CLI argument (e.g., "calver")
  summary:     string               # One-line description
  concepts:    CkmConcept[]         # Related concepts
  operations:  CkmOperation[]       # Related operations
  configSchema: CkmConfigEntry[]    # Related config entries
  constraints: CkmConstraint[]      # Related constraints
}

TYPE CkmTopicIndexEntry {
  name:        string
  summary:     string
  concepts:    integer              # Count
  operations:  integer              # Count
  configFields: integer             # Count
  constraints: integer              # Count
}

TYPE CkmTopicIndex {
  topics:  CkmTopicIndexEntry[]
  ckm: {
    concepts:    integer
    operations:  integer
    constraints: integer
    workflows:   integer
    configSchema: integer
  }
}

TYPE CkmInspectResult {
  meta:         CkmMeta
  counts: {
    concepts:    integer
    operations:  integer
    constraints: integer
    workflows:   integer
    configKeys:  integer
    topics:      integer
  }
  topicNames:   string[]
}

TYPE CkmValidationResult {
  valid:   boolean
  errors:  CkmValidationError[]
}

TYPE CkmValidationError {
  path:    string                   # JSON pointer (e.g., "/concepts/0/slug")
  message: string
}
```

### Engine Interface (the API every SDK exposes)

```
INTERFACE CkmEngine {

  # ── Properties ──────────────────────────────────────
  topics: CkmTopic[]                 # All auto-derived topics (read-only)

  # ── Topic Queries ───────────────────────────────────
  getTopicIndex(toolName: string = "tool") -> string
    # Returns formatted topic index for terminal display.
    # Output MUST stay within 300 tokens.

  getTopicContent(topicName: string) -> string | null
    # Returns human-readable content for a topic.
    # Returns null if topic not found.
    # Output MUST stay within 800 tokens.

  getTopicJson(topicName?: string) -> CkmTopicIndex | CkmTopic | CkmErrorResult
    # If topicName is undefined: returns CkmTopicIndex (full index with counts)
    # If topicName matches: returns the full CkmTopic object
    # If topicName not found: returns { error: string, topics: string[] }

  # ── Manifest Access ─────────────────────────────────
  getManifest() -> CkmManifest
    # Returns the raw (possibly migrated) manifest.

  # ── Inspection ──────────────────────────────────────
  inspect() -> CkmInspectResult
    # Returns manifest statistics and topic summary.
}
```

### Factory Function

```
FUNCTION createCkmEngine(manifest: CkmManifest) -> CkmEngine
  # Main entry point. Accepts a parsed CkmManifest (v1 or v2).
  # If v1 is detected, auto-migrates to v2 internally.
  # Derives topics at construction time.
  # Returns a configured, immutable engine.
```

### Schema Utilities

```
FUNCTION validateManifest(data: any) -> CkmValidationResult
  # Validates a JSON value against the ckm.json v2 schema.
  # Returns { valid: true, errors: [] } or { valid: false, errors: [...] }.

FUNCTION migrateV1toV2(manifest: CkmManifestV1) -> CkmManifest
  # Deterministic migration from v1 to v2.
  # Spec-defined algorithm — conformance tested.

FUNCTION detectVersion(data: any) -> 1 | 2
  # Returns the schema version of a parsed manifest.
  # v2: has "meta" block or $schema contains "v2"
  # v1: everything else
```

### Adapter Interface

```
INTERFACE CkmCliAdapter<TProgram> {
  name:      string                  # e.g., "commander", "click", "clap"
  framework: string                  # e.g., "Commander.js", "Click", "Clap"

  register(
    program:  TProgram,              # Host CLI program object
    engine:   CkmEngine,             # Configured engine instance
    options?: CkmAdapterOptions,
  ) -> void
}

TYPE CkmAdapterOptions {
  commandName: string = "ckm"        # Subcommand name to register
  toolName:    string?               # Tool name in help output (default: inferred)
  formatter:   CkmFormatter?         # Custom output formatter
}

INTERFACE CkmFormatter {
  formatIndex(topics: CkmTopic[], toolName: string) -> string
  formatTopic(topic: CkmTopic) -> string
  formatJson(data: any) -> string
}
```

### Language Mapping Table

Every type/method in the interface maps to each language like this:

| Interface Concept | TypeScript | Python | Rust | Go |
|-------------------|-----------|--------|------|-----|
| `CkmEngine` | `interface CkmEngine` | `class CkmEngine` | `struct CkmEngine` + `impl` | `type Engine struct` |
| `CkmCliAdapter` | `interface CkmCliAdapter<T>` | `class CkmCliAdapter(ABC)` | `trait CkmCliAdapter` | `type Adapter interface` |
| `CkmManifest` | `interface CkmManifest` | `@dataclass CkmManifest` | `#[derive(Deserialize)] struct CkmManifest` | `type Manifest struct` |
| `CkmTypeRef` | `interface CkmTypeRef` | `@dataclass CkmTypeRef` | `struct CkmTypeRef` | `type TypeRef struct` |
| `CanonicalType` | `type CanonicalType = "string" \| ...` | `class CanonicalType(StrEnum)` | `enum CanonicalType` | `type CanonicalType string` + consts |
| `createCkmEngine()` | `function createCkmEngine(m)` | `def create_engine(m)` | `fn CkmEngine::new(m)` | `func NewEngine(m)` |
| `getTopicIndex()` | `engine.getTopicIndex(name)` | `engine.get_topic_index(name)` | `engine.topic_index(name)` | `engine.TopicIndex(name)` |
| `getTopicContent()` | `engine.getTopicContent(name)` | `engine.get_topic_content(name)` | `engine.topic_content(name)` | `engine.TopicContent(name)` |
| `getTopicJson()` | `engine.getTopicJson(name?)` | `engine.get_topic_json(name=None)` | `engine.topic_json(name)` | `engine.TopicJSON(name)` |
| `validateManifest()` | `function validateManifest(d)` | `def validate_manifest(d)` | `fn validate_manifest(d)` | `func ValidateManifest(d)` |
| `migrateV1toV2()` | `function migrateV1toV2(m)` | `def migrate_v1_to_v2(m)` | `fn migrate_v1_to_v2(m)` | `func MigrateV1ToV2(m)` |
| null return | `null` | `None` | `Option::None` | `nil` |
| optional field | `field?: T` | `field: T \| None = None` | `field: Option<T>` | `Field *T` (pointer) |
| error result | union type | union type | `Result<T, E>` | `(T, error)` |

### Conformance Test Matrix

Every method on the engine interface has conformance test cases. The test fixtures define:

```
conformance/
  fixtures/
    minimal.ckm.json
    multi-topic.ckm.json
    polyglot.ckm.json
    v1-legacy.ckm.json
  expected/
    minimal/
      topics.json               # Expected engine.topics array
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
```

A language SDK is conformant when ALL fixtures pass. CI runs: `npm test`, `pytest`, `cargo test`, `go test` — all against the same fixtures.

### How the Layers Connect

```
┌─────────────────────────────────────────────────────────────┐
│                     INTERFACE.md                            │
│  (defines types + methods — the API contract)               │
│                                                             │
│  CkmManifest, CkmConcept, CkmTypeRef, CkmTopic, ...       │
│  CkmEngine { topics, getTopicIndex, getTopicContent, ... } │
│  CkmCliAdapter { register }                                │
│  createCkmEngine(), validateManifest(), migrateV1toV2()    │
└──────────────────────────┬──────────────────────────────────┘
                           │ "what to implement"
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        v                  v                  v
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  TypeScript  │  │    Python    │  │     Rust     │  ...
│   SDK impl   │  │   SDK impl   │  │   SDK impl   │
│              │  │              │  │              │
│ interface    │  │ @dataclass   │  │ struct +     │
│ CkmEngine    │  │ CkmEngine    │  │ impl Engine  │
│ { ... }      │  │ class:       │  │ { ... }      │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       │  conformance/   │  conformance/   │  conformance/
       │  (same fixtures)│  (same fixtures)│  (same fixtures)
       v                 v                 v
┌─────────────────────────────────────────────────────────────┐
│                    conformance/                              │
│  fixtures/ → expected/ → every SDK must produce identical   │
│  JSON output for identical JSON input                       │
└─────────────────────────────────────────────────────────────┘
```

### File Placement in Monorepo

```
ckm/
  INTERFACE.md                  # SDK Interface Definition (this section)
  SPEC.md                       # Algorithm specification
  ckm.schema.json               # ckm.json v2 JSON Schema
  conformance/                  # Cross-language test suite

  packages/
    core/src/                   # TypeScript: implements INTERFACE.md
      types.ts                  # CkmManifest, CkmConcept, CkmTypeRef, ...
      engine.ts                 # CkmEngine impl
      validate.ts               # validateManifest()
      migrate.ts                # migrateV1toV2()
      adapters/                 # CkmCliAdapter impls

    python/ckm/                 # Python: implements INTERFACE.md
      types.py                  # @dataclass CkmManifest, CkmConcept, ...
      engine.py                 # CkmEngine class
      validate.py               # validate_manifest()
      migrate.py                # migrate_v1_to_v2()
      adapters/                 # CkmCliAdapter impls

    rust/src/                   # Rust: implements INTERFACE.md
      types.rs                  # struct CkmManifest, CkmConcept, ...
      engine.rs                 # impl CkmEngine
      validate.rs               # fn validate_manifest()
      migrate.rs                # fn migrate_v1_to_v2()
      adapters/                 # trait CkmCliAdapter impls

    go/                         # Go: implements INTERFACE.md
      types.go                  # type Manifest struct, Concept struct, ...
      engine.go                 # func NewEngine(), methods
      validate.go               # func ValidateManifest()
      migrate.go                # func MigrateV1ToV2()
      adapters/                 # Adapter interface impls
```

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
| Canonical type set | None | `string`, `boolean`, `number`, `integer`, `array`, `object`, `null`, `any` | Maps to JSON Schema primitives; every language can interpret |
| Concept slugs | Derived at runtime (strip `Config`) | Explicit `slug` field in manifest | Generator decides; engine doesn't guess; works for any naming convention |
| Concept tags | None (suffix heuristic) | `tags: string[]` | `["config"]` tag replaces suffix-based heuristic; extensible |
| Operation tags | None (keyword matching) | `tags: string[]` | Explicit topic linkage replaces fragile substring matching |
| Constraint severity | Absent | `"error" | "warning" | "info"` | Graduated enforcement display |
| Workflow steps | `{ command?, manual?, note? }` | `{ action: "command"|"manual", value, note? }` | Discriminated union, cleaner |
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

The v2 engine accepts both v1 and v2 manifests. When it encounters v1 (detected by `$schema` URL or absence of `meta` block), it runs deterministic migration:

1. Wraps `project`/`generated` into `meta` block, sets `language: "typescript"`, `generator: "unknown"`
2. For each type string, wraps as `{ canonical: inferCanonical(type), original: type }`
3. Derives `slug` from concept name using v1 algorithm (strip `Config`/`Result`/`Options`, lowercase)
4. Infers `tags: ["config"]` for concepts ending in `Config`
5. Rewrites config schema keys from `ConceptName.prop` to `slug.prop`

---

## 6. Adapter Interface Design

### Design Philosophy

Mirrors VG's `VersionSourceProvider` + `DETECTION_TABLE` pattern, but for CLI framework registration:

- **VG Sources**: `VersionSourceProvider` interface + `DETECTION_TABLE` maps files to factories + `resolveVersionSource()` walks the table
- **CKM Adapters**: `CkmCliAdapter` interface + `ADAPTER_TABLE` maps framework IDs to adapter factories + user selects their adapter explicitly

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
```

### Adapter Table (Lazy-Loaded)

```typescript
const ADAPTER_TABLE: Record<string, () => Promise<CkmCliAdapter>> = {
  commander:  () => import('./adapters/commander').then(m => m.default),
  citty:      () => import('./adapters/citty').then(m => m.default),
  oclif:      () => import('./adapters/oclif').then(m => m.default),
  clipanion:  () => import('./adapters/clipanion').then(m => m.default),
};
```

Adapters are lazy-loaded so `npm install ckm` does NOT force a dependency on any CLI framework. Only the adapter you import pulls in that framework.

### Commander.js Adapter (Reference Implementation)

```typescript
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
          console.log(JSON.stringify(engine.getTopicJson(topic), null, 2));
        } else if (topic) {
          const content = engine.getTopicContent(topic);
          if (!content) {
            console.error(`Unknown topic: ${topic}`);
            console.log(engine.getTopicIndex(toolName));
            process.exit(1);
          }
          console.log(content);
        } else {
          console.log(engine.getTopicIndex(toolName));
        }
      });
  },
};
```

### Pattern Across Languages

| Concern | TypeScript | Python | Rust | Go |
|---------|-----------|--------|------|-----|
| Interface | `interface CkmCliAdapter` | `class CkmCliAdapter(ABC)` | `trait CkmCliAdapter` | `type CkmCliAdapter interface` |
| Registration | `adapter.register(program, engine)` | `adapter.register(group, engine)` | `adapter.register(&engine)` | `adapter.Register(engine, opts)` |
| Lazy loading | Dynamic `import()` | Standard Python imports | Feature flags in Cargo.toml | Build tags |
| Framework dep | peerDependency | optional extra (`ckm[click]`) | optional feature | go module |

### Python: Click Adapter Example

```python
class ClickAdapter(CkmCliAdapter):
    name = "click"
    framework = "Click"

    def register(self, group, engine, *, command_name="ckm", tool_name=None):
        tool = tool_name or group.name or "tool"

        @group.command(name=command_name)
        @click.argument("topic", required=False)
        @click.option("--json", "as_json", is_flag=True)
        def ckm_command(topic, as_json):
            if as_json:
                click.echo(json.dumps(engine.get_topic_json(topic), indent=2))
            elif topic:
                content = engine.get_topic_content(topic)
                if content is None:
                    click.echo(f"Unknown topic: {topic}", err=True)
                    raise SystemExit(1)
                click.echo(content)
            else:
                click.echo(engine.get_topic_index(tool))
```

### Rust: Clap Adapter Example

```rust
pub trait CkmCliAdapter {
    fn name(&self) -> &str;
    fn framework(&self) -> &str;
    fn register(
        &self,
        engine: &CkmEngine,
        command_name: Option<&str>,
        tool_name: Option<&str>,
    ) -> clap::Command;
}
```

### Go: Cobra Adapter Example

```go
type CkmCliAdapter interface {
    Name() string
    Framework() string
    Register(engine *ckm.Engine, opts *RegisterOptions) error
}
```

---

## 7. Standalone CLI Design

The `ckm-cli` package provides a standalone binary.

```bash
ckm [topic] [flags]        # Browse/query CKM data
ckm validate <file>        # Validate a ckm.json against v2 schema
ckm migrate <file>         # Migrate v1 -> v2 (--dry-run supported)
ckm inspect <file>         # Show manifest stats
```

### Progressive Disclosure (Mandatory Protocol Requirement)

Every adapter MUST support all four disclosure levels:

| Level | Command | Audience | Max Token Budget |
|-------|---------|----------|-----------------|
| 0 | `ckm` | Human / Agent discovery | 300 |
| 1 | `ckm <topic>` | Human / Agent drill-down | 800 |
| 1J | `ckm <topic> --json` | Agent structured | 1200 |
| 2 | `ckm --json` | Agent full index | 3000 |

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

## 8. Rollout Phases

### Phase 1: TypeScript Foundation

**Scope**: Core library, Commander adapter, standalone CLI, conformance tests, v2 schema.

**Deliverables**:
- `ckm` on npm (core library)
- `ckm-cli` on npm (standalone CLI binary)
- `ckm.schema.json` (v2 JSON Schema)
- `SPEC.md` (algorithm specification)
- Conformance test suite
- Commander.js adapter (production-ready)
- v1 -> v2 migration function
- VersionGuard migrated to consume `ckm` as a dependency

**Exit criteria**: `npm install ckm` works. VG's `src/ckm/` replaced by `import { createCkmEngine } from 'ckm'`. Conformance tests pass.

### Phase 2: TypeScript Adapter Expansion

**Scope**: Citty, oclif, Clipanion adapters.

Each adapter is ~50-100 LOC of framework-specific registration. Small scope.

**Exit criteria**: Each adapter has integration tests. ADAPTER_TABLE complete for TypeScript.

### Phase 3: Python SDK

**Scope**: Full Python implementation of SPEC.md.

**Deliverables**:
- `ckm` on PyPI
- Engine (port of SPEC.md)
- Click adapter + Typer adapter (with Rich output)
- Conformance tests pass

**Exit criteria**: `pip install ckm` works. Python CLI projects can embed CKM.

### Phase 4: Rust SDK

**Scope**: Full Rust implementation.

**Deliverables**:
- `ckm` on crates.io
- Engine (serde-based port of SPEC.md)
- Clap adapter (derive macro compatible)
- Conformance tests pass

**Exit criteria**: `cargo add ckm` works.

### Phase 5: Go SDK

**Scope**: Full Go implementation.

**Deliverables**:
- Go module
- Engine + Cobra adapter + urfave/cli adapter
- Conformance tests pass

### Phase 6: forge-ts Integration

**Scope**: forge-ts generates v2 `ckm.json` natively.

**Deliverables**:
- forge-ts generates v2 by default (`--ckm-version 1` for legacy)
- `forge-ts build` populates `concept.slug`, `concept.tags`, canonical types, operation tags
- forge-ts resolves enum values into `type.enum`

**Exit criteria**: `forge-ts build` produces valid v2 `ckm.json` that passes `ckm validate`.

---

## 9. Architectural Decision Records

### ADR-001: Spec-Based Backbone over Shared Binary

**Decision**: The backbone is a JSON Schema + algorithm spec, not a compiled binary.

**Rationale**: ~500 LOC of pure data transformation does not justify WASM/FFI overhead. Each language gets an idiomatic, zero-dependency implementation verified by conformance tests.

### ADR-002: Explicit Slugs and Tags (Generator Responsibility)

**Decision**: `ckm.json` generator populates `concept.slug` and `concept.tags` explicitly. Engine uses them directly.

**Rationale**: v1's suffix-stripping heuristic only works for TypeScript naming conventions. Explicit slugs work for any language.

### ADR-003: Canonical Type System

**Decision**: Types use `{ canonical, original?, enum? }` structure.

**Rationale**: `"CalVerFormat"` is meaningless to Python. Canonical types map to JSON Schema primitives every language understands. `original` preserves source fidelity.

### ADR-004: Monorepo with Per-Language Publishing

**Decision**: Single monorepo, each language publishes independently.

**Rationale**: Schema + spec + conformance tests + all implementations stay in lock-step. Single CI validates cross-language conformance.

### ADR-005: Adapters as Peer Dependencies

**Decision**: Framework adapters are peer/optional dependencies. `npm install ckm` does NOT install Commander/Citty/oclif/Clipanion.

**Rationale**: Zero unnecessary transitive dependencies. Users already have their CLI framework installed.

### ADR-006: Progressive Disclosure as Conformance Requirement

**Decision**: Four disclosure levels are mandatory. Adapters MUST support all four. Conformance tests verify token budgets.

**Rationale**: CKM's primary consumers are LLM agents under token budgets. Dumping full manifests at level 0 defeats the purpose.

---

## 10. Related Decisions

### VersionGuard Package Rename

`@codluv/versionguard` will be renamed to `versionguard` (unscoped) as part of the v1.0.0 release. Both `versionguard` and `ckm` are available on npm.

| Package | Registry | Current | New |
|---------|----------|---------|-----|
| VersionGuard | npm | `@codluv/versionguard` | `versionguard` |
| CKM SDK | npm | (does not exist) | `ckm` |

### Relationship to forge-ts

- **forge-ts** = generation (produces `ckm.json` from source code)
- **CKM SDK** = consumption/display (reads `ckm.json`, provides help/topics)
- forge-ts is one possible generator — any tool can produce a valid `ckm.json`
- Phase 6 adds forge-ts v2 support, but CKM works independently
