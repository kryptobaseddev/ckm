# CKM SDK Interface Definition

**Status**: REFERENCE DOCUMENTATION
**Version**: 2.0.0
**Date**: 2026-03-29

> This document describes the public API surface of the CKM SDK.
> The **canonical source of truth** is the Rust implementation in `packages/rust-core/`.
> This document is derived from that code and exists for human understanding and
> cross-language wrapper authors. When this document and the code disagree, the code wins.

---

## 1. Guiding Principle

Implement once in Rust. Expose via FFI wrappers. This document describes what the code does.

---

## 2. Input Types (from ckm.json v2)

These types describe the structure of a valid `ckm.json` v2 manifest.

### CkmManifest

The top-level manifest object.

```
TYPE CkmManifest {
  $schema:      string                 # Schema URL (e.g., "https://ckm.dev/schemas/v2.json")
  version:      string                 # Schema version (e.g., "2.0.0")
  meta:         CkmMeta                # Project metadata and provenance
  concepts:     CkmConcept[]           # Domain concepts (interfaces, types)
  operations:   CkmOperation[]         # User-facing operations (functions)
  constraints:  CkmConstraint[]        # Enforced rules
  workflows:    CkmWorkflow[]          # Multi-step workflows
  configSchema: CkmConfigEntry[]       # Configuration schema entries
}
```

### CkmMeta

Provenance metadata about the manifest source.

```
TYPE CkmMeta {
  project:    string                   # Project name (e.g., "my-tool")
  language:   string                   # Source language (e.g., "typescript", "python", "rust")
  generator:  string                   # Tool that generated the manifest (e.g., "forge-ts@0.21.1")
  generated:  string                   # ISO 8601 timestamp of generation
  sourceUrl:  string?                  # Optional URL to source repository
}
```

### CkmConcept

A domain concept extracted from source code (e.g., an interface, struct, or class).

```
TYPE CkmConcept {
  id:         string                   # Unique identifier (e.g., "concept-calver-config")
  name:       string                   # Type name (e.g., "CalVerConfig")
  slug:       string                   # Topic slug (e.g., "calver") — used for topic derivation
  what:       string                   # One-line description
  tags:       string[]                 # Semantic tags (e.g., ["config"])
  properties: CkmProperty[]?          # Properties of the type, if applicable
}
```

### CkmProperty

A property within a concept.

```
TYPE CkmProperty {
  name:        string                  # Property name
  type:        CkmTypeRef              # Type reference (canonical + original)
  description: string                  # Description from source documentation
  required:    boolean                 # Whether the property is required
  default:     string?                 # Default value (nullable — null means no default)
}
```

### CkmTypeRef

A portable type reference with canonical mapping.

```
TYPE CkmTypeRef {
  canonical:  CanonicalType            # Language-agnostic type
  original:   string?                  # Source language type annotation (e.g., "CalVerFormat")
  enum:       string[]?               # Known values for string enums (e.g., ["YYYY.MM.DD", "YYYY.MM"])
}
```

### CanonicalType

The set of portable primitive types, mapped to JSON Schema primitives.

```
ENUM CanonicalType = "string" | "boolean" | "number" | "integer" | "array" | "object" | "null" | "any"
```

### CkmOperation

A user-facing operation extracted from source code (e.g., an exported function).

```
TYPE CkmOperation {
  id:      string                      # Unique identifier (e.g., "op-validate")
  name:    string                      # Function name (e.g., "validate")
  what:    string                      # One-line description
  tags:    string[]                    # Semantic tags for topic linkage (e.g., ["calver", "validation"])
  inputs:  CkmInput[]?                # Function parameters (optional)
  outputs: CkmOutput?                  # Return value (optional)
}
```

### CkmInput

A function parameter within an operation.

```
TYPE CkmInput {
  name:        string                  # Parameter name
  type:        CkmTypeRef              # Type reference
  required:    boolean                 # Whether the parameter is required
  description: string                  # Description from source documentation
}
```

### CkmOutput

A return value from an operation.

```
TYPE CkmOutput {
  type:        CkmTypeRef              # Type reference
  description: string                  # Description of the return value
}
```

### CkmConstraint

A rule enforced by the tool.

```
TYPE CkmConstraint {
  id:         string                   # Unique identifier (e.g., "constraint-future-date")
  rule:       string                   # Human-readable rule description
  enforcedBy: string                   # Function or module that enforces the constraint
  severity:   "error" | "warning" | "info"   # Severity level
}
```

### CkmWorkflow

A multi-step workflow for achieving a common goal.

```
TYPE CkmWorkflow {
  id:    string                        # Unique identifier
  goal:  string                        # What the workflow achieves
  tags:  string[]                      # Semantic tags
  steps: CkmWorkflowStep[]            # Ordered steps (minimum 1)
}
```

### CkmWorkflowStep

A single step within a workflow.

```
TYPE CkmWorkflowStep {
  action: "command" | "manual"         # Discriminant: CLI command or manual action
  value:  string                       # The command or instruction
  note:   string?                      # Optional explanatory note
}
```

### CkmConfigEntry

A configuration schema entry.

```
TYPE CkmConfigEntry {
  key:         string                  # Dotted key path (e.g., "calver.format")
  type:        CkmTypeRef              # Type reference
  description: string                  # Description
  default:     string?                 # Default value (nullable)
  required:    boolean                 # Whether the config entry is required
}
```

---

## 3. Derived Types (computed by the engine)

These types are produced by the engine from manifest data.

### CkmTopic

An auto-derived topic grouping related concepts, operations, config, and constraints.

```
TYPE CkmTopic {
  name:        string                  # Slug used as CLI argument (e.g., "calver")
  summary:     string                  # One-line description (from the primary concept)
  concepts:    CkmConcept[]            # Related concepts
  operations:  CkmOperation[]          # Related operations
  configSchema: CkmConfigEntry[]       # Related config entries
  constraints: CkmConstraint[]         # Related constraints
}
```

### CkmTopicIndexEntry

A summary entry for the topic index.

```
TYPE CkmTopicIndexEntry {
  name:         string                 # Topic slug
  summary:      string                 # One-line description
  concepts:     integer                # Count of related concepts
  operations:   integer                # Count of related operations
  configFields: integer                # Count of related config entries
  constraints:  integer                # Count of related constraints
}
```

### CkmTopicIndex

The full topic index returned by `getTopicJson()` with no argument.

```
TYPE CkmTopicIndex {
  topics:  CkmTopicIndexEntry[]        # All topic summaries
  ckm: {
    concepts:    integer               # Total concepts in manifest
    operations:  integer               # Total operations
    constraints: integer               # Total constraints
    workflows:   integer               # Total workflows
    configSchema: integer              # Total config entries
  }
}
```

### CkmInspectResult

Manifest statistics returned by `inspect()`.

```
TYPE CkmInspectResult {
  meta:         CkmMeta                # Manifest metadata
  counts: {
    concepts:    integer
    operations:  integer
    constraints: integer
    workflows:   integer
    configKeys:  integer
    topics:      integer
  }
  topicNames:   string[]               # List of derived topic slugs
}
```

### CkmValidationResult

Result of manifest validation.

```
TYPE CkmValidationResult {
  valid:   boolean                     # Whether the manifest is valid
  errors:  CkmValidationError[]        # Validation errors (empty if valid)
}
```

### CkmValidationError

A single validation error.

```
TYPE CkmValidationError {
  path:    string                      # JSON pointer (e.g., "/concepts/0/slug")
  message: string                      # Human-readable error message
}
```

### CkmErrorResult

Error returned when a topic is not found.

```
TYPE CkmErrorResult {
  error:   string                      # Error message (e.g., "Unknown topic: foo")
  topics:  string[]                    # Available topic names for suggestion
}
```

---

## 4. Engine Interface

The core API every SDK must expose.

```
INTERFACE CkmEngine {

  # ── Properties ──────────────────────────────────────
  topics: CkmTopic[]                   # All auto-derived topics (read-only, computed at construction)

  # ── Topic Queries ───────────────────────────────────
  getTopicIndex(toolName: string = "tool") -> string
    # Returns formatted topic index for terminal display.
    # Includes tool name, usage line, topic list with summaries, and flag descriptions.
    # Output MUST stay within 300 tokens (Level 0 disclosure).

  getTopicContent(topicName: string) -> string | null
    # Returns human-readable content for a specific topic.
    # Includes: concepts with properties, operations with params, config fields, constraints.
    # Returns null if topic not found.
    # Output MUST stay within 800 tokens (Level 1 disclosure).

  getTopicJson(topicName?: string) -> CkmTopicIndex | CkmTopic | CkmErrorResult
    # If topicName is undefined/null: returns CkmTopicIndex (full index with counts).
    #   Token budget: 3000 (Level 2 disclosure).
    # If topicName matches a topic: returns the full CkmTopic object.
    #   Token budget: 1200 (Level 1J disclosure).
    # If topicName does not match: returns CkmErrorResult with available topics.

  # ── Manifest Access ─────────────────────────────────
  getManifest() -> CkmManifest
    # Returns the raw manifest (v2, possibly migrated from v1).

  # ── Inspection ──────────────────────────────────────
  inspect() -> CkmInspectResult
    # Returns manifest statistics: metadata, counts, topic names.
}
```

---

## 5. Factory Function

```
FUNCTION createCkmEngine(manifest: object) -> CkmEngine
  # Main entry point. Accepts a parsed JSON object.
  # If v1 is detected (via detectVersion), auto-migrates to v2 internally.
  # Derives topics at construction time using the algorithm defined in SPEC.md.
  # Returns a configured, immutable engine instance.
  # MUST NOT throw on valid v1 or v2 input.
```

---

## 6. Schema Utilities

```
FUNCTION validateManifest(data: object) -> CkmValidationResult
  # Validates a parsed JSON object against the ckm.json v2 schema.
  # Returns { valid: true, errors: [] } for valid manifests.
  # Returns { valid: false, errors: [...] } with JSON pointer paths for invalid manifests.
  # v1 manifests MUST fail validation (they are not v2).

FUNCTION migrateV1toV2(manifest: object) -> CkmManifest
  # Deterministic migration from v1 format to v2 format.
  # Algorithm is defined in SPEC.md and conformance-tested.
  # Steps:
  #   1. Wrap project/generated into meta block (language: "typescript", generator: "unknown")
  #   2. For each type string, wrap as { canonical: inferCanonical(type), original: type }
  #   3. Derive slug from concept name (strip Config/Result/Options suffix, lowercase)
  #   4. Infer tags: ["config"] for concepts ending in "Config"
  #   5. Rewrite config schema keys from "ConceptName.prop" to "slug.prop"
  #   6. Convert workflow steps to discriminated union format
  #   7. Add severity "error" to all constraints (default)
  #   8. Add version "2.0.0" and $schema URL

FUNCTION detectVersion(data: object) -> 1 | 2
  # Returns the schema version of a parsed manifest.
  # Returns 2 if: data has "meta" block OR $schema contains "v2"
  # Returns 1 otherwise (including malformed data)
```

---

## 7. Adapter Interface

```
INTERFACE CkmCliAdapter<TProgram> {
  name:      string                    # Adapter identifier (e.g., "commander", "click", "clap")
  framework: string                    # Framework display name (e.g., "Commander.js", "Click", "Clap")

  register(
    program:  TProgram,                # Host CLI program object
    engine:   CkmEngine,               # Configured engine instance
    options?: CkmAdapterOptions,       # Optional configuration
  ) -> void
    # Registers a `ckm [topic]` subcommand on the host program.
    # MUST support all four progressive disclosure levels.
    # MUST handle: no topic (Level 0), topic (Level 1), --json (Level 1J/2).
}

TYPE CkmAdapterOptions {
  commandName: string = "ckm"          # Subcommand name to register (default: "ckm")
  toolName:    string?                 # Tool name in help output (default: inferred from program)
  formatter:   CkmFormatter?           # Custom output formatter (default: built-in plain text)
}

INTERFACE CkmFormatter {
  formatIndex(topics: CkmTopic[], toolName: string) -> string
    # Formats the topic index for terminal display.

  formatTopic(topic: CkmTopic) -> string
    # Formats a single topic's content for terminal display.

  formatJson(data: object) -> string
    # Formats JSON output (default: JSON.stringify with 2-space indent).
}
```

---

## 8. Progressive Disclosure Levels

Every adapter MUST support all four levels. Token budgets are conformance-tested.

| Level | Trigger | Returns | Max Tokens | Audience |
|-------|---------|---------|-----------|----------|
| 0 | `ckm` (no args) | `getTopicIndex()` | 300 | Human / Agent discovery |
| 1 | `ckm <topic>` | `getTopicContent(topic)` | 800 | Human / Agent drill-down |
| 1J | `ckm <topic> --json` | `getTopicJson(topic)` | 1200 | Agent structured |
| 2 | `ckm --json` | `getTopicJson()` | 3000 | Agent full index |

---

## 9. Language Surface Table

All languages call the same Rust core via FFI. The wrapper exposes idiomatic names:

| Rust Core (SSoT) | Node.js (napi-rs) | Python (PyO3) | Go (CGo/WASM) |
|-------------------|-----------|--------|------|
| `CkmEngine::new()` | `createCkmEngine()` | `create_engine()` | `NewEngine()` |
| `engine.topic_index()` | `engine.getTopicIndex()` | `engine.get_topic_index()` | `engine.TopicIndex()` |
| `engine.topic_content()` | `engine.getTopicContent()` | `engine.get_topic_content()` | `engine.TopicContent()` |
| `engine.topic_json()` | `engine.getTopicJson()` | `engine.get_topic_json()` | `engine.TopicJSON()` |
| `engine.manifest()` | `engine.getManifest()` | `engine.get_manifest()` | `engine.Manifest()` |
| `engine.inspect()` | `engine.inspect()` | `engine.inspect()` | `engine.Inspect()` |
| `validate_manifest()` | `validateManifest()` | `validate_manifest()` | `ValidateManifest()` |
| `migrate_v1_to_v2()` | `migrateV1toV2()` | `migrate_v1_to_v2()` | `MigrateV1ToV2()` |
| `detect_version()` | `detectVersion()` | `detect_version()` | `DetectVersion()` |
| `Option<T>` | `T \| null` | `T \| None` | `*T` (nil) |

**Key difference from the original spec-based approach:** These are NOT independent implementations. Each language surface is a thin FFI wrapper (~50-100 LOC) calling the exact same Rust code. The names are idiomatic but the behavior is mechanically identical.

---

## 10. Conformance and Correctness

### How correctness is guaranteed

Since all language surfaces call the same Rust core, conformance is structural:

1. **Rust core tests** verify algorithm correctness against `conformance/` fixtures
2. **FFI wrapper tests** verify data crosses the boundary correctly (serialization round-trips)
3. **Integration tests** verify end-to-end behavior in each language

There is no drift between languages because there is only one implementation. The conformance fixtures (`conformance/fixtures/` + `conformance/expected/`) test the Rust core directly. Wrappers inherit correctness by construction.

### What the conformance suite tests

| Test | What it proves |
|------|---------------|
| `topics` derivation | Rust core correctly implements SPEC.md Section 3 |
| `getTopicIndex()` output | Formatter produces correct terminal text |
| `getTopicContent()` output | Per-topic formatting matches expected |
| `getTopicJson()` output | JSON serialization is correct |
| `inspect()` output | Counts and metadata are accurate |
| `validateManifest()` | Schema validation catches invalid manifests |
| `migrateV1toV2()` | v1 migration produces valid v2 |
| `detectVersion()` | Version detection is correct |
| Token budgets | Output stays within Level 0/1/1J/2 limits |
