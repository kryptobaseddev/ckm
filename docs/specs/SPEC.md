# CKM Algorithm Specification

**Status**: Documentation (derived from rust-core, the SSoT)
**Date**: 2026-03-29
**Source**: `packages/rust-core/src/engine.rs`, `packages/rust-core/src/migrate.rs` (authoritative)

> When this document and the code disagree, the code wins.

---

## 1. Version Detection

**Source**: `packages/rust-core/src/migrate.rs` -- `detect_version()`

Given a parsed JSON value `data`:

1. If `data` is an object with a `"meta"` key whose value is an object, return **2**.
2. If `data` is an object with a `"$schema"` key whose string value contains `"v2"`, return **2**.
3. Otherwise, return **1** (including malformed/non-object data).

---

## 2. v1 to v2 Migration

**Source**: `packages/rust-core/src/migrate.rs` -- `migrate_v1_to_v2()`

Given a parsed v1 manifest, produce a valid v2 `CkmManifest`:

### Step 1: Create meta block

```
meta.project   = v1.project   || "unknown"
meta.language  = "typescript"  (assumed for v1)
meta.generator = "unknown"
meta.generated = v1.generated  || ""
meta.sourceUrl = null
```

### Step 2: Migrate concepts

For each concept in `v1.concepts`:

1. **Derive slug** from `name` by stripping known suffixes:
   - `Config` -> strip (e.g., "CalVerConfig" -> "calver")
   - `Result` -> strip (e.g., "ValidationResult" -> "validation")
   - `Options` -> strip (e.g., "RenderOptions" -> "render")
   - No suffix match -> lowercase the full name
2. **Infer tags** from name:
   - Ends with `Config` -> `["config"]`
   - Ends with `Result` -> `["result"]`
   - Ends with `Options` -> `["options"]`
   - Otherwise -> `[]`
3. **Migrate properties**: For each property:
   - Wrap type string as `CkmTypeRef { canonical: infer_canonical(type), original: type, enum: null }`
   - Set `required: true`, `default: null`
4. Set `rules: null`, `relatedTo: null`, `extensions: null`

### Step 3: Migrate operations

For each operation in `v1.operations`:

1. **Infer tags** by matching operation name/description against concept slugs (case-insensitive substring match).
2. **Migrate inputs**: Wrap each input's type string as `CkmTypeRef`.
3. **Migrate outputs**: If present and non-null, wrap the `text` field.
4. Set `preconditions: null`, `exitCodes: null`, `checksPerformed: null`, `extensions: null`

### Step 4: Migrate constraints

For each constraint in `v1.constraints`:

1. Set `severity: "error"` (default).
2. Set `configKey: null`, `default: null`, `security: null`, `extensions: null`.

### Step 5: Migrate workflows

For each workflow in `v1.workflows`:

1. Migrate steps: `command` field -> `StepAction::Command`, `manual` field -> `StepAction::Manual`.
2. Set `tags: []`, `extensions: null`.
3. Set `expect: null` on each step.

### Step 6: Migrate config schema

For each entry in `v1.configSchema`:

1. **Migrate config key**: If key is `"ConceptName.prop"`, find matching concept by name and rewrite to `"slug.prop"`. If no match, lowercase the first segment.
2. Wrap type string as `CkmTypeRef`.
3. Set `required: true`, `effect: null`, `extensions: null`.

### Step 7: Assemble manifest

```
$schema    = "https://ckm.dev/schemas/v2.json"
version    = "2.0.0"
topics     = null
extensions = null
```

### Canonical Type Inference

The `infer_canonical()` function maps type strings to canonical types:

| Input | Canonical |
|-------|-----------|
| `"string"` | String |
| `"boolean"` | Boolean |
| `"number"` | Number |
| `"integer"` | Integer |
| `"null"`, `"undefined"`, `"void"` | Null |
| `"object"`, `"Record"` | Object |
| `"unknown"`, `"any"` | Any |
| Contains `[]` or starts with `"Array"` | Array |
| Contains `\|` (union) | String |
| Everything else (e.g., `"CalVerFormat"`) | Object |

---

## 3. Topic Derivation

**Source**: `packages/rust-core/src/engine.rs` -- `derive_topics()` and `resolve_declared_topics()`

### Mode 1: Producer-Declared Topics

When the manifest has a `topics` field (non-null), the engine resolves declared topics:

For each declared topic:
1. Resolve `conceptIds` -> find concepts in `manifest.concepts` by ID.
2. Resolve `operationIds` -> find operations in `manifest.operations` by ID.
3. Resolve `constraintIds` -> find constraints in `manifest.constraints` by ID.
4. Resolve `configKeys` -> find config entries whose `key` starts with any declared config key prefix.
5. Create `CkmTopic { name, summary, concepts, operations, configSchema, constraints }`.

### Mode 2: Auto-Derived Topics

When no `topics` field is present, derive topics algorithmically:

#### Phase 1: Concept-based topics

For each concept in `manifest.concepts` with a **non-empty slug**:

1. If a topic with this slug already exists, **merge** (add concept to existing topic).
2. Otherwise create a new topic:
   - **Related concepts**: Other concepts with the same slug, or whose name contains the slug (case-insensitive).
   - **Matched operations**: Operations whose tags contain the slug (case-insensitive), OR whose name/description matches by keyword.
   - **Matched config**: Config entries whose key prefix (before first `.`) equals the slug.
   - **Matched constraints**: Constraints whose `enforcedBy` references a concept name or matched operation name.
   - **Summary**: The primary concept's `what` field.
3. Track claimed operation IDs and constraint IDs.

**Keyword matching** (`matches_by_keyword`): Concatenate operation name and `what`, lowercase; check if it contains the slug or any concept name.

#### Phase 2: Unclaimed operations

For each operation NOT claimed in Phase 1:
1. Derive a slug from the operation name (using `derive_slug()`).
2. If a topic with this slug exists, add the operation to it.
3. Otherwise, create a new topic with only this operation.

#### Phase 3: Unclaimed constraints

For each constraint NOT claimed in Phase 1:
1. Try to find a topic whose operations match `enforcedBy` (by operation name substring).
2. If no match, add to the first topic (fallback).

**Key design decision**: ALL concepts with non-empty slugs become topics, not just concepts tagged with `"config"`. This ensures every concept is browsable.

---

## 4. Output Formatting

**Source**: `packages/rust-core/src/format.rs`

### Level 0: Topic Index (`format_topic_index`)

```
{tool_name} CKM -- Codebase Knowledge Manifest

Usage: {tool_name} ckm [topic] [--json] [--llm]

Topics:
  {topic_name}  {topic_summary}
  ...

Flags:
  --json    Machine-readable CKM output (concepts, operations, config schema)
  --llm     Full API context for LLM agents (forge-ts llms.txt)
```

Budget: < 300 tokens. Topic names are left-aligned with padding.

### Level 1: Topic Content (`format_topic_content`)

```
# {topic_summary}

## Concepts

  {concept_name} -- {concept_what}
    {prop_name}: {type_display}{default_display}
      {prop_description}

## Operations

  {op_name}() -- {op_what}
    @param {input_name}: {input_description}

## Config Fields

  {config_key}: {type_display}{default_display}
    {config_description}

## Constraints

  [{constraint_id}] {constraint_rule}
    Enforced by: {constraint_enforced_by}
```

Budget: < 800 tokens. Sections are omitted if empty.

**Type display**: Uses `original` if present, otherwise `canonical.to_string()`.

**Default display**: Looks up matching config entry by key suffix (`.{property_name}`). Shows ` = {default}` if found.

### Level 1J: Topic JSON (`topic_json(Some(name))`)

Returns `TopicJsonResult::Topic(CkmTopic)` serialized as JSON.

### Level 2: Full Index JSON (`topic_json(None)`)

Returns `TopicJsonResult::Index(CkmTopicIndex)` serialized as JSON.

---

## 5. Validation

**Source**: `packages/rust-core/src/validate.rs`

Lightweight structural validation without external JSON Schema library.

### Required top-level fields

- `version` (string)
- `meta` (object with: `project`, `language`, `generator`, `generated`)
- `concepts` (array)
- `operations` (array)
- `constraints` (array)
- `workflows` (array)
- `configSchema` (array)

### Per-entity validation

- **Concepts**: `id`, `name`, `slug`, `what` (strings), `tags` (array), properties validated if present.
- **Operations**: `id`, `name`, `what` (strings), `tags` (array).
- **Constraints**: `id`, `rule`, `enforcedBy` (strings), `severity` must be "error"|"warning"|"info".
- **Workflows**: `id`, `goal` (strings), `tags` (array), `steps` must be non-empty array.
- **Config entries**: `key`, `description` (strings), `required` (boolean), `type` (valid CkmTypeRef).

### Type reference validation

`type.canonical` must be one of: `string`, `boolean`, `number`, `integer`, `array`, `object`, `null`, `any`.

### Error format

Errors include JSON pointer paths (e.g., `/concepts/0/slug`) and human-readable messages.

---

## 6. Builder

**Source**: `packages/rust-core/src/builder.rs`

The `CkmManifestBuilder` provides a fluent API for constructing valid v2 manifests.

### ID generation

- Concepts: `"concept-{slug}"`
- Operations: `"op-{name}"`
- Constraints: `"constraint-{counter}"`
- Workflows: `"wf-{counter}"`

### Property targeting

`add_concept_property(slug, ...)` finds the concept by slug and appends to its `properties` array.

### Operation targeting

`add_operation_input(op_name, ...)` finds the operation by name and appends to its `inputs` array.

### Build output

`build()` produces a `CkmManifest` with:
- `$schema`: `"https://ckm.dev/schemas/v2.json"`
- `version`: `"2.0.0"`
- `generated`: Static placeholder timestamp (generators typically override)
- `topics`: `null`
- `extensions`: `null`

`build_json()` serializes via `serde_json::to_string_pretty`.
