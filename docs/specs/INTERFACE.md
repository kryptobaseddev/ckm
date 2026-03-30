# CKM SDK Interface Definition

**Status**: Documentation (derived from rust-core, the SSoT)
**Date**: 2026-03-29
**Source**: `packages/rust-core/src/` (authoritative)

> When this document and the code disagree, the code wins.

---

## 1. Type Alias

### Extensions

Freeform extension data. Producers can attach arbitrary key-value pairs to any CKM entity.

- **Rust**: `HashMap<String, serde_json::Value>`
- **TypeScript**: `Record<string, unknown>`
- **Python**: `dict[str, Any]`

---

## 2. Input Types (from ckm.json v2)

### CanonicalType (enum)

Portable primitive types, mapped to JSON Schema primitives.

| Variant | JSON value |
|---------|-----------|
| String | `"string"` |
| Boolean | `"boolean"` |
| Number | `"number"` |
| Integer | `"integer"` |
| Array | `"array"` |
| Object | `"object"` |
| Null | `"null"` |
| Any | `"any"` |

Has a `parse(s)` method that maps common aliases (e.g., `"bool"` -> Boolean, `"float"` -> Number, `"void"` -> Null, unknown types -> Object).

### CkmTypeRef

Portable type reference with canonical mapping.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| canonical | CanonicalType | yes | Language-agnostic canonical type |
| original | string | no | Source language type annotation (e.g., "CalVerFormat") |
| enum | string[] | no | Known enum values for string types |

### CkmProperty

A property within a concept.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | yes | Property name |
| type | CkmTypeRef | yes | Type reference |
| description | string | yes | Description from source documentation |
| required | boolean | yes | Whether the property is required |
| default | string | no | Default value (null means no default) |

### CkmConcept

A domain concept extracted from source code.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | yes | Unique identifier (e.g., "concept-calver-config") |
| name | string | yes | Type name (e.g., "CalVerConfig") |
| slug | string | yes | Topic slug (e.g., "calver") -- used for topic derivation |
| what | string | yes | One-line description |
| tags | string[] | yes | Semantic tags (e.g., ["config"]) |
| properties | CkmProperty[] | no | Properties of the type |
| rules | string[] | no | Validation rules from remarks or constraint tags |
| relatedTo | string[] | no | Related concept names from @see tags |
| extensions | Extensions | no | Producer-defined extension data |

### CkmInput

A function parameter within an operation.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | yes | Parameter name |
| type | CkmTypeRef | yes | Type reference |
| required | boolean | yes | Whether the parameter is required |
| description | string | yes | Description from source documentation |

### CkmOutput

A return value from an operation.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| type | CkmTypeRef | yes | Type reference |
| description | string | yes | Description of the return value |

### CkmOperation

A user-facing operation extracted from source code.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | yes | Unique identifier (e.g., "op-validate") |
| name | string | yes | Function name (e.g., "validate") |
| what | string | yes | One-line description |
| tags | string[] | yes | Semantic tags for topic linkage |
| preconditions | string[] | no | Preconditions that must be met |
| inputs | CkmInput[] | no | Function parameters |
| outputs | CkmOutput | no | Return value |
| exitCodes | Map<string, string> | no | Exit codes and their meanings |
| checksPerformed | string[] | no | Checks/validations performed |
| extensions | Extensions | no | Producer-defined extension data |

### CkmConstraint

A rule enforced by the tool.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | yes | Unique identifier (e.g., "constraint-future-date") |
| rule | string | yes | Human-readable rule description |
| enforcedBy | string | yes | Function or module that enforces the constraint |
| severity | Severity | yes | Severity level |
| configKey | string | no | Config key that controls this constraint |
| default | string | no | Default value for the config key |
| security | boolean | no | Whether this constraint has security implications |
| extensions | Extensions | no | Producer-defined extension data |

### Severity (enum)

| Variant | JSON value |
|---------|-----------|
| Error | `"error"` |
| Warning | `"warning"` |
| Info | `"info"` |

### CkmWorkflowStep

A single step within a workflow.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| action | StepAction | yes | Discriminant: command or manual |
| value | string | yes | The command or instruction |
| note | string | no | Optional explanatory note |
| expect | string | no | Expected outcome of this step |

### StepAction (enum)

| Variant | JSON value |
|---------|-----------|
| Command | `"command"` |
| Manual | `"manual"` |

### CkmWorkflow

A multi-step workflow for achieving a common goal.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | yes | Unique identifier |
| goal | string | yes | What the workflow achieves |
| tags | string[] | yes | Semantic tags |
| steps | CkmWorkflowStep[] | yes | Ordered steps (minimum 1) |
| extensions | Extensions | no | Producer-defined extension data |

### CkmConfigEntry

A configuration schema entry.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| key | string | yes | Dotted key path (e.g., "calver.format") |
| type | CkmTypeRef | yes | Type reference |
| description | string | yes | Description |
| default | string | no | Default value (null means no default) |
| required | boolean | yes | Whether the config entry is required |
| effect | string | no | Downstream effect or behavior this controls |
| extensions | Extensions | no | Producer-defined extension data |

### CkmDeclaredTopic

Producer-declared topic grouping. When present in the manifest `topics` field, these override engine-derived topics.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | yes | Topic slug (e.g., "getting-started") |
| summary | string | yes | One-line summary |
| conceptIds | string[] | no | IDs of concepts belonging to this topic |
| operationIds | string[] | no | IDs of operations belonging to this topic |
| constraintIds | string[] | no | IDs of constraints belonging to this topic |
| configKeys | string[] | no | Config key prefixes belonging to this topic |

### CkmMeta

Provenance metadata about the manifest source.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| project | string | yes | Project name (e.g., "my-tool") |
| language | string | yes | Source language (e.g., "typescript") |
| generator | string | yes | Tool that generated the manifest |
| generated | string | yes | ISO 8601 timestamp |
| sourceUrl | string | no | URL to source repository |

### CkmManifest

The top-level CKM manifest object (v2).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| $schema | string | yes | Schema URL (e.g., "https://ckm.dev/schemas/v2.json") |
| version | string | yes | Schema version (e.g., "2.0.0") |
| meta | CkmMeta | yes | Project metadata and provenance |
| concepts | CkmConcept[] | yes | Domain concepts |
| operations | CkmOperation[] | yes | User-facing operations |
| constraints | CkmConstraint[] | yes | Enforced rules |
| workflows | CkmWorkflow[] | yes | Multi-step workflows |
| configSchema | CkmConfigEntry[] | yes | Configuration schema entries |
| topics | CkmDeclaredTopic[] | no | Producer-declared topics (override auto-derivation) |
| extensions | Extensions | no | Manifest-level extension data |

---

## 3. Derived Types (computed by the engine)

### CkmTopic

An auto-derived (or producer-declared) topic grouping.

| Field | Type | Description |
|-------|------|-------------|
| name | string | Slug used as CLI argument |
| summary | string | One-line description |
| concepts | CkmConcept[] | Related concepts |
| operations | CkmOperation[] | Related operations |
| configSchema | CkmConfigEntry[] | Related config entries |
| constraints | CkmConstraint[] | Related constraints |

### CkmTopicIndexEntry

A summary entry for the topic index.

| Field | Type | Description |
|-------|------|-------------|
| name | string | Topic slug |
| summary | string | One-line description |
| concepts | number | Count of related concepts |
| operations | number | Count of related operations |
| configFields | number | Count of related config entries |
| constraints | number | Count of related constraints |

### CkmManifestCounts

Aggregate manifest counts.

| Field | Type | Description |
|-------|------|-------------|
| concepts | number | Total concepts |
| operations | number | Total operations |
| constraints | number | Total constraints |
| workflows | number | Total workflows |
| configSchema | number | Total config entries |

### CkmTopicIndex

The full topic index returned by `topic_json()` with no argument.

| Field | Type | Description |
|-------|------|-------------|
| topics | CkmTopicIndexEntry[] | All topic summaries |
| ckm | CkmManifestCounts | Aggregate manifest counts |

### CkmInspectCounts

| Field | Type | Description |
|-------|------|-------------|
| concepts | number | Number of concepts |
| operations | number | Number of operations |
| constraints | number | Number of constraints |
| workflows | number | Number of workflows |
| configKeys | number | Number of config keys |
| topics | number | Number of derived topics |

### CkmInspectResult

Manifest statistics returned by `inspect()`.

| Field | Type | Description |
|-------|------|-------------|
| meta | CkmMeta | Manifest metadata |
| counts | CkmInspectCounts | Counts of each section |
| topicNames | string[] | List of derived topic slugs |

### CkmValidationError

| Field | Type | Description |
|-------|------|-------------|
| path | string | JSON pointer path (e.g., "/concepts/0/slug") |
| message | string | Human-readable error message |

### CkmValidationResult

| Field | Type | Description |
|-------|------|-------------|
| valid | boolean | Whether the manifest is valid |
| errors | CkmValidationError[] | Validation errors (empty if valid) |

### CkmErrorResult

Error returned when a topic is not found.

| Field | Type | Description |
|-------|------|-------------|
| error | string | Error message (e.g., "Unknown topic: foo") |
| topics | string[] | Available topic names for suggestion |

### TopicJsonResult (untagged enum)

The result of `topic_json()` -- one of:

- **Index(CkmTopicIndex)** -- when no topic name is given
- **Topic(CkmTopic)** -- when topic name matches
- **Error(CkmErrorResult)** -- when topic name does not match

---

## 4. Engine Interface

### CkmEngine

The core CKM engine. Immutable after construction.

#### `CkmEngine::new(data: serde_json::Value) -> Self`

Creates a new engine from a parsed JSON value. If v1 is detected, auto-migrates to v2. If manifest has `topics` field, resolves producer-declared topics; otherwise auto-derives topics from all concepts with non-empty slugs. Does not return a Result -- handles errors internally.

#### `engine.topics() -> &[CkmTopic]`

Returns all derived topics.

#### `engine.topic_index(tool_name: &str) -> String`

Returns formatted topic index for terminal display (Level 0, < 300 tokens).

#### `engine.topic_content(topic_name: &str) -> Option<String>`

Returns human-readable content for a specific topic (Level 1, < 800 tokens). Returns None if topic not found.

#### `engine.topic_json(topic_name: Option<&str>) -> TopicJsonResult`

Returns structured JSON. None => full CkmTopicIndex (Level 2); Some(name) => CkmTopic (Level 1J) or CkmErrorResult.

#### `engine.manifest() -> &CkmManifest`

Returns the raw (possibly migrated) manifest.

#### `engine.inspect() -> CkmInspectResult`

Returns manifest statistics: metadata, counts, and topic names.

---

## 5. Standalone Functions

### `validate_manifest(data: &serde_json::Value) -> CkmValidationResult`

Validates a JSON value against the v2 schema using lightweight structural checks.

### `detect_version(data: &serde_json::Value) -> u8`

Returns 2 if the data has a `meta` object or `$schema` containing "v2". Returns 1 otherwise.

### `migrate_v1_to_v2(data: &serde_json::Value) -> CkmManifest`

Deterministic migration from v1 to v2 format.

---

## 6. Builder Interface

### CkmManifestBuilder

Fluent builder for constructing valid CKM v2 manifests (producer API).

#### `CkmManifestBuilder::new(project: &str, language: &str) -> Self`

Creates a new builder.

#### Chainable methods

| Method | Description |
|--------|-------------|
| `.generator(name)` | Sets generator name (e.g., "forge-ts@1.0.0") |
| `.source_url(url)` | Sets source repository URL |
| `.add_concept(name, slug, what, tags)` | Adds a concept |
| `.add_concept_property(slug, name, type, desc, required, default)` | Adds a property to a concept |
| `.add_concept_property_typed(slug, name, type, original, desc, required, default)` | Adds a property with original type annotation |
| `.add_operation(name, what, tags)` | Adds an operation |
| `.add_operation_input(op_name, param, type, required, desc)` | Adds an input to an operation |
| `.set_operation_output(op_name, type, desc)` | Sets operation output |
| `.add_constraint(rule, enforced_by, severity)` | Adds a constraint |
| `.add_workflow(goal, tags)` | Adds a workflow |
| `.add_workflow_command(command, note)` | Adds a command step to last workflow |
| `.add_workflow_manual(instruction, note)` | Adds a manual step to last workflow |
| `.add_config(key, type, desc, required, default)` | Adds a config entry |

#### Terminal methods

| Method | Description |
|--------|-------------|
| `.build()` | Returns `CkmManifest` |
| `.build_json()` | Returns JSON string |

---

## 7. Format Functions

### `format_topic_index(topics, tool_name) -> String`

Formats the topic index for terminal display (Level 0). Includes tool name, usage line, topic list, and flag descriptions.

### `format_topic_content(topics, topic_name) -> Option<String>`

Formats a topic's content for human-readable display (Level 1). Sections: Concepts, Operations, Config Fields, Constraints.

---

## 8. Language Mapping

| Rust Core | Node.js (`ckm-sdk`) | Python (`ckm`) | Go |
|-----------|----------------------|-----------------|-----|
| `CkmEngine::new(data)` | `createCkmEngine(manifest)` | `create_engine(manifest)` | `ckm.NewEngine(manifest)` |
| `engine.topics()` | `engine.topicsCount` | -- | -- |
| `engine.topic_index(name)` | `engine.getTopicIndex(name)` | `engine.topic_index(name)` | `engine.TopicIndex(name)` |
| `engine.topic_content(name)` | `engine.getTopicContent(name)` | `engine.topic_content(name)` | `engine.TopicContent(name)` |
| `engine.topic_json(name)` | `engine.getTopicJson(name)` | `engine.topic_json(name)` | `engine.TopicJSON(name)` |
| `engine.manifest()` | `engine.getManifest()` | `engine.manifest()` | `engine.Manifest()` |
| `engine.inspect()` | `engine.inspect()` | `engine.inspect()` | `engine.Inspect()` |
| `validate_manifest(data)` | `validateManifest(data)` | `validate_manifest(data)` | `ckm.ValidateManifest(data)` |
| `migrate_v1_to_v2(data)` | `migrateV1toV2(data)` | `migrate_v1_to_v2(data)` | `ckm.MigrateV1ToV2(data)` |
| `detect_version(data)` | `detectVersion(data)` | `detect_version(data)` | `ckm.DetectVersion(data)` |
| `CkmManifestBuilder::new()` | `createManifestBuilder(p, l)` | -- (not yet) | -- (not yet) |
| `Option<T>` | `T \| null` | `T \| None` | `*T` |
