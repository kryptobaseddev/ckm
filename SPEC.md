# CKM SDK Algorithm Specification

**Status**: REFERENCE DOCUMENTATION
**Version**: 2.0.0
**Date**: 2026-03-29

> This document describes the deterministic algorithms implemented in `packages/rust-core/`.
> The **canonical source of truth** is the Rust implementation. This prose specification
> exists for human understanding and serves as the design reference.
> All algorithms are tested in rust-core via `conformance/` fixtures.

---

## 1. Version Detection

### Algorithm: `detectVersion(data) -> 1 | 2`

```
FUNCTION detectVersion(data):
  IF data has property "meta" AND typeof data.meta == "object":
    RETURN 2
  IF data has property "$schema" AND typeof data.$schema == "string":
    IF data.$schema contains "v2":
      RETURN 2
  RETURN 1
```

**Rules:**
- The presence of a `meta` object is the primary v2 indicator
- A `$schema` URL containing "v2" is the secondary indicator
- Everything else (including malformed data) is treated as v1

---

## 2. v1 to v2 Migration

### Algorithm: `migrateV1toV2(manifest) -> CkmManifest`

Input: A parsed v1 manifest object.
Output: A valid v2 CkmManifest.

```
FUNCTION migrateV1toV2(v1):
  result = {}

  # Step 1: Add schema and version
  result.$schema = "https://ckm.dev/schemas/v2.json"
  result.version = "2.0.0"

  # Step 2: Create meta block
  result.meta = {
    project:   v1.project OR "unknown",
    language:  "typescript",
    generator: "unknown",
    generated: v1.generated OR new Date().toISOString(),
    sourceUrl: null
  }

  # Step 3: Migrate concepts
  result.concepts = []
  FOR EACH concept IN (v1.concepts OR []):
    migratedConcept = {
      id:         concept.id,
      name:       concept.name,
      slug:       deriveSlug(concept.name),
      what:       concept.what,
      tags:       inferTags(concept.name),
      properties: migrateProperties(concept.properties)
    }
    APPEND migratedConcept TO result.concepts

  # Step 4: Migrate operations
  result.operations = []
  FOR EACH op IN (v1.operations OR []):
    migratedOp = {
      id:      op.id,
      name:    op.name,
      what:    op.what,
      tags:    inferOperationTags(op, result.concepts),
      inputs:  migrateInputs(op.inputs),
      outputs: migrateOutput(op.outputs)
    }
    APPEND migratedOp TO result.operations

  # Step 5: Migrate constraints
  result.constraints = []
  FOR EACH c IN (v1.constraints OR []):
    migratedConstraint = {
      id:         c.id,
      rule:       c.rule,
      enforcedBy: c.enforcedBy,
      severity:   "error"
    }
    APPEND migratedConstraint TO result.constraints

  # Step 6: Migrate workflows
  result.workflows = []
  FOR EACH wf IN (v1.workflows OR []):
    migratedWorkflow = {
      id:   wf.id,
      goal: wf.goal,
      tags: [],
      steps: migrateWorkflowSteps(wf.steps)
    }
    APPEND migratedWorkflow TO result.workflows

  # Step 7: Migrate config schema
  result.configSchema = []
  FOR EACH entry IN (v1.configSchema OR []):
    migratedEntry = {
      key:         migrateConfigKey(entry.key, result.concepts),
      type:        migrateTypeString(entry.type),
      description: entry.description OR "",
      default:     entry.default OR null,
      required:    true
    }
    APPEND migratedEntry TO result.configSchema

  RETURN result
```

### Helper: `deriveSlug(conceptName) -> string`

```
FUNCTION deriveSlug(name):
  slug = name
  # Strip known suffixes in order
  IF slug ends with "Config":  slug = slug without "Config" suffix
  ELSE IF slug ends with "Result":  slug = slug without "Result" suffix
  ELSE IF slug ends with "Options": slug = slug without "Options" suffix
  # Convert to lowercase
  RETURN lowercase(slug)
```

**Examples:**
- `"CalVerConfig"` → `"calver"`
- `"SemVerConfig"` → `"semver"`
- `"ValidationResult"` → `"validation"`
- `"GitHooksConfig"` → `"githooks"`
- `"VersionGuardConfig"` → `"versionguard"`

### Helper: `inferTags(conceptName) -> string[]`

```
FUNCTION inferTags(name):
  tags = []
  IF name ends with "Config":
    APPEND "config" TO tags
  IF name ends with "Result":
    APPEND "result" TO tags
  IF name ends with "Options":
    APPEND "options" TO tags
  RETURN tags
```

### Helper: `inferOperationTags(operation, concepts) -> string[]`

```
FUNCTION inferOperationTags(op, concepts):
  tags = []
  haystack = lowercase(op.name + " " + op.what)
  FOR EACH concept IN concepts:
    IF concept.slug != "" AND haystack contains concept.slug:
      APPEND concept.slug TO tags
  # Deduplicate tags
  RETURN unique(tags)
```

### Helper: `migrateTypeString(typeStr) -> CkmTypeRef`

```
FUNCTION migrateTypeString(typeStr):
  IF typeStr is null OR typeStr is undefined:
    RETURN { canonical: "any", original: null }

  RETURN {
    canonical: inferCanonical(typeStr),
    original:  typeStr
  }

FUNCTION inferCanonical(typeStr):
  lower = lowercase(typeStr)
  IF lower == "string":    RETURN "string"
  IF lower == "boolean":   RETURN "boolean"
  IF lower == "number":    RETURN "number"
  IF lower == "integer":   RETURN "integer"
  IF lower == "null":      RETURN "null"
  IF lower == "undefined": RETURN "null"
  IF lower == "void":      RETURN "null"
  IF lower contains "[]" OR lower starts with "array":  RETURN "array"
  IF lower == "object" OR lower == "record":  RETURN "object"
  IF lower == "unknown" OR lower == "any":    RETURN "any"
  # For complex types (union types, custom type names), default to "object"
  IF lower contains "|":   RETURN "string"  # union types are typically string enums
  RETURN "object"  # Custom type names (e.g., "CalVerFormat") → object
```

### Helper: `migrateProperties(properties) -> CkmProperty[]`

```
FUNCTION migrateProperties(props):
  IF props is null OR props is undefined:
    RETURN null
  result = []
  FOR EACH p IN props:
    APPEND {
      name:        p.name,
      type:        migrateTypeString(p.type),
      description: p.description OR "",
      required:    true,
      default:     null
    } TO result
  RETURN result
```

### Helper: `migrateInputs(inputs) -> CkmInput[]`

```
FUNCTION migrateInputs(inputs):
  IF inputs is null OR inputs is undefined:
    RETURN null
  result = []
  FOR EACH i IN inputs:
    APPEND {
      name:        i.name,
      type:        migrateTypeString(i.type),
      required:    i.required IF defined ELSE true,
      description: i.description OR ""
    } TO result
  RETURN result
```

### Helper: `migrateOutput(output) -> CkmOutput | null`

```
FUNCTION migrateOutput(output):
  IF output is null OR output is undefined:
    RETURN null
  RETURN {
    type:        migrateTypeString(output.text OR "unknown"),
    description: output.text OR ""
  }
```

### Helper: `migrateWorkflowSteps(steps) -> CkmWorkflowStep[]`

```
FUNCTION migrateWorkflowSteps(steps):
  result = []
  FOR EACH step IN steps:
    IF step.command is defined AND step.command is not null:
      APPEND { action: "command", value: step.command, note: step.note OR null } TO result
    ELSE IF step.manual is defined AND step.manual is not null:
      APPEND { action: "manual", value: step.manual, note: step.note OR null } TO result
    ELSE:
      # Fallback: treat as manual step with whatever value is available
      APPEND { action: "manual", value: "", note: step.note OR null } TO result
  RETURN result
```

### Helper: `migrateConfigKey(key, concepts) -> string`

```
FUNCTION migrateConfigKey(key, concepts):
  # v1 keys are "ConceptName.propName" (e.g., "CalVerConfig.format")
  # v2 keys are "slug.propName" (e.g., "calver.format")
  parts = split(key, ".")
  IF length(parts) >= 2:
    conceptPart = parts[0]
    restParts = parts[1:]
    # Find matching concept to get slug
    FOR EACH concept IN concepts:
      IF concept.name == conceptPart:
        RETURN join([concept.slug] + restParts, ".")
  # If no concept match, lowercase the first segment
  IF length(parts) >= 2:
    RETURN join([lowercase(parts[0])] + parts[1:], ".")
  RETURN lowercase(key)
```

---

## 3. Topic Derivation

### Algorithm: `deriveTopics(manifest) -> CkmTopic[]`

This is the core algorithm that transforms a v2 manifest into a list of topics.

```
FUNCTION deriveTopics(manifest):
  topics = []

  FOR EACH concept IN manifest.concepts:
    # Step 1: Only concepts tagged "config" become topics
    IF "config" NOT IN concept.tags:
      CONTINUE

    slug = concept.slug
    IF slug is empty:
      CONTINUE

    # Step 2: Collect all related concepts
    relatedConcepts = [concept]
    FOR EACH other IN manifest.concepts:
      IF other.id == concept.id:
        CONTINUE
      IF lowercase(other.name) contains slug OR slug contains deriveSlug(other.name):
        APPEND other TO relatedConcepts
    conceptNames = [c.name FOR c IN relatedConcepts]

    # Step 3: Match operations by tags
    matchedOperations = []
    FOR EACH op IN manifest.operations:
      IF hasTagOverlap(op.tags, [slug]):
        APPEND op TO matchedOperations
      ELSE IF operationMatchesByKeyword(op, slug, conceptNames):
        APPEND op TO matchedOperations

    # Step 4: Match config entries by key prefix
    matchedConfig = []
    FOR EACH entry IN manifest.configSchema:
      keyPrefix = firstSegment(entry.key)  # e.g., "calver" from "calver.format"
      IF keyPrefix == slug:
        APPEND entry TO matchedConfig

    # Step 5: Match constraints by enforcedBy
    matchedConstraints = []
    FOR EACH constraint IN manifest.constraints:
      # Match if enforcedBy references any concept name
      IF any(name IN constraint.enforcedBy FOR name IN conceptNames):
        APPEND constraint TO matchedConstraints
        CONTINUE
      # Match if enforcedBy references any matched operation
      IF any(constraint.enforcedBy contains op.name FOR op IN matchedOperations):
        APPEND constraint TO matchedConstraints

    # Step 6: Build topic
    topic = {
      name:        slug,
      summary:     concept.what,
      concepts:    relatedConcepts,
      operations:  matchedOperations,
      configSchema: matchedConfig,
      constraints: matchedConstraints
    }
    APPEND topic TO topics

  RETURN topics
```

### Helper: `hasTagOverlap(tags1, tags2) -> boolean`

```
FUNCTION hasTagOverlap(tags1, tags2):
  FOR EACH t1 IN tags1:
    FOR EACH t2 IN tags2:
      IF lowercase(t1) == lowercase(t2):
        RETURN true
  RETURN false
```

### Helper: `operationMatchesByKeyword(op, slug, conceptNames) -> boolean`

```
FUNCTION operationMatchesByKeyword(op, slug, conceptNames):
  haystack = lowercase(op.name + " " + op.what)
  IF haystack contains slug:
    RETURN true
  FOR EACH name IN conceptNames:
    IF haystack contains lowercase(name):
      RETURN true
  RETURN false
```

### Helper: `firstSegment(dottedKey) -> string`

```
FUNCTION firstSegment(key):
  parts = split(key, ".")
  RETURN parts[0]
```

---

## 4. Output Formatting

### 4.1 Topic Index (Level 0)

```
FUNCTION formatTopicIndex(topics, toolName):
  lines = []
  APPEND "{toolName} CKM — Codebase Knowledge Manifest" TO lines
  APPEND "" TO lines
  APPEND "Usage: {toolName} ckm [topic] [--json] [--llm]" TO lines
  APPEND "" TO lines
  APPEND "Topics:" TO lines

  # Calculate alignment
  maxNameLen = max(length(t.name) FOR t IN topics)

  FOR EACH topic IN topics:
    paddedName = rightPad(topic.name, maxNameLen + 2)
    APPEND "  {paddedName}{topic.summary}" TO lines

  APPEND "" TO lines
  APPEND "Flags:" TO lines
  APPEND "  --json    Machine-readable CKM output (concepts, operations, config schema)" TO lines
  APPEND "  --llm     Full API context for LLM agents (forge-ts llms.txt)" TO lines

  RETURN join(lines, "\n")
```

**Token budget**: Output MUST stay within 300 tokens.

### 4.2 Topic Content (Level 1)

```
FUNCTION formatTopicContent(topics, topicName):
  topic = find(t IN topics WHERE t.name == topicName)
  IF topic is null:
    RETURN null

  lines = []
  APPEND "# {topic.summary}" TO lines
  APPEND "" TO lines

  # Concepts section
  IF length(topic.concepts) > 0:
    APPEND "## Concepts" TO lines
    APPEND "" TO lines
    FOR EACH c IN topic.concepts:
      APPEND "  {c.name} — {c.what}" TO lines
      IF c.properties is not null:
        FOR EACH p IN c.properties:
          defaultStr = findDefault(topic.configSchema, c, p)
          typeStr = formatTypeRef(p.type)
          APPEND "    {p.name}: {typeStr}{defaultStr}" TO lines
          IF p.description is not empty:
            APPEND "      {p.description}" TO lines
      APPEND "" TO lines

  # Operations section
  IF length(topic.operations) > 0:
    APPEND "## Operations" TO lines
    APPEND "" TO lines
    FOR EACH o IN topic.operations:
      APPEND "  {o.name}() — {o.what}" TO lines
      IF o.inputs is not null:
        FOR EACH i IN o.inputs:
          APPEND "    @param {i.name}: {i.description}" TO lines
      APPEND "" TO lines

  # Config section
  IF length(topic.configSchema) > 0:
    APPEND "## Config Fields" TO lines
    APPEND "" TO lines
    FOR EACH c IN topic.configSchema:
      typeStr = formatTypeRef(c.type)
      defaultStr = IF c.default THEN " = {c.default}" ELSE ""
      APPEND "  {c.key}: {typeStr}{defaultStr}" TO lines
      IF c.description is not empty:
        APPEND "    {c.description}" TO lines
    APPEND "" TO lines

  # Constraints section
  IF length(topic.constraints) > 0:
    APPEND "## Constraints" TO lines
    APPEND "" TO lines
    FOR EACH c IN topic.constraints:
      APPEND "  [{c.id}] {c.rule}" TO lines
      APPEND "    Enforced by: {c.enforcedBy}" TO lines
    APPEND "" TO lines

  RETURN join(lines, "\n")
```

**Token budget**: Output MUST stay within 800 tokens.

### Helper: `formatTypeRef(typeRef) -> string`

```
FUNCTION formatTypeRef(typeRef):
  # For v2 CkmTypeRef objects
  IF typeRef is object AND typeRef has "canonical":
    IF typeRef.original is defined:
      RETURN typeRef.original
    RETURN typeRef.canonical
  # For raw strings (v1 compat in display)
  IF typeRef is string:
    RETURN typeRef
  RETURN "unknown"
```

### Helper: `findDefault(configSchema, concept, property) -> string`

```
FUNCTION findDefault(configSchema, concept, property):
  # Look for matching config entry
  FOR EACH entry IN configSchema:
    IF entry.key ends with ".{property.name}":
      IF entry.default is not null:
        RETURN " = {entry.default}"
  RETURN ""
```

### 4.3 Topic JSON (Level 1J — single topic)

```
FUNCTION buildTopicJson(topics, topicName):
  topic = find(t IN topics WHERE t.name == topicName)
  IF topic is null:
    RETURN {
      error: "Unknown topic: {topicName}",
      topics: [t.name FOR t IN topics]
    }
  RETURN {
    topic:       topic.name,
    summary:     topic.summary,
    concepts:    topic.concepts,
    operations:  topic.operations,
    configSchema: topic.configSchema,
    constraints: topic.constraints
  }
```

**Token budget**: Output MUST stay within 1200 tokens.

### 4.4 Full Index JSON (Level 2 — no topic argument)

```
FUNCTION buildTopicIndexJson(topics, manifest):
  RETURN {
    topics: [
      {
        name:         t.name,
        summary:      t.summary,
        concepts:     length(t.concepts),
        operations:   length(t.operations),
        configFields: length(t.configSchema),
        constraints:  length(t.constraints)
      }
      FOR t IN topics
    ],
    ckm: {
      concepts:    length(manifest.concepts),
      operations:  length(manifest.operations),
      constraints: length(manifest.constraints),
      workflows:   length(manifest.workflows),
      configSchema: length(manifest.configSchema)
    }
  }
```

**Token budget**: Output MUST stay within 3000 tokens.

---

## 5. Inspection

### Algorithm: `inspect(manifest, topics) -> CkmInspectResult`

```
FUNCTION inspect(manifest, topics):
  RETURN {
    meta: manifest.meta,
    counts: {
      concepts:    length(manifest.concepts),
      operations:  length(manifest.operations),
      constraints: length(manifest.constraints),
      workflows:   length(manifest.workflows),
      configKeys:  length(manifest.configSchema),
      topics:      length(topics)
    },
    topicNames: [t.name FOR t IN topics]
  }
```

---

## 6. Validation

### Algorithm: `validateManifest(data) -> CkmValidationResult`

```
FUNCTION validateManifest(data):
  errors = []

  # Validate against ckm.schema.json (v2 JSON Schema)
  schemaErrors = jsonSchemaValidate(data, "ckm.schema.json")
  FOR EACH err IN schemaErrors:
    APPEND { path: err.instancePath, message: err.message } TO errors

  RETURN {
    valid:  length(errors) == 0,
    errors: errors
  }
```

**Rules:**
- Uses standard JSON Schema validation against `ckm.schema.json`
- Returns JSON pointer paths (e.g., `/concepts/0/slug`)
- v1 manifests MUST fail validation (they lack required v2 fields like `meta`, `slug`, `tags`)
- Empty arrays are valid (a manifest with zero concepts is valid)

---

## 7. Engine Construction

### Algorithm: `createCkmEngine(data) -> CkmEngine`

```
FUNCTION createCkmEngine(data):
  # Step 1: Detect version
  version = detectVersion(data)

  # Step 2: Migrate if v1
  manifest = data
  IF version == 1:
    manifest = migrateV1toV2(data)

  # Step 3: Derive topics
  topics = deriveTopics(manifest)

  # Step 4: Return engine interface
  RETURN {
    topics: topics,
    getTopicIndex: (toolName = "tool") => formatTopicIndex(topics, toolName),
    getTopicContent: (name) => formatTopicContent(topics, name),
    getTopicJson: (name) => {
      IF name is undefined OR name is null:
        RETURN buildTopicIndexJson(topics, manifest)
      RETURN buildTopicJson(topics, name)
    },
    getManifest: () => manifest,
    inspect: () => inspect(manifest, topics)
  }
```

**Invariants:**
- Engine is immutable after construction
- Topics are derived once at construction time
- v1 input is transparently migrated — callers see v2 output
- `getManifest()` returns the (possibly migrated) v2 manifest
- All methods are pure functions of the constructed state

---

## 8. Token Budget Enforcement

Token budgets are approximate guidelines enforced by conformance tests. Estimation uses the heuristic:

```
tokens ≈ characters / 4
```

| Level | Method | Max Tokens | Max Characters |
|-------|--------|-----------|---------------|
| 0 | `getTopicIndex()` | 300 | ~1200 |
| 1 | `getTopicContent()` | 800 | ~3200 |
| 1J | `getTopicJson(topic)` | 1200 | ~4800 |
| 2 | `getTopicJson()` | 3000 | ~12000 |

Conformance tests measure the output character count for each fixture and verify it stays within the character budget. Implementations SHOULD truncate output that would exceed the budget, but conformance tests are written so that well-formed fixtures naturally stay within budget.

---

## 9. Determinism Guarantee

For any given input manifest, the following properties hold:

1. **Topic order**: Topics appear in the same order as their primary concept appears in `manifest.concepts`
2. **Related concept order**: Related concepts appear in the same order as in `manifest.concepts`
3. **Operation order**: Matched operations appear in the same order as in `manifest.operations`
4. **Config order**: Matched config entries appear in the same order as in `manifest.configSchema`
5. **Constraint order**: Matched constraints appear in the same order as in `manifest.constraints`
6. **Migration determinism**: The same v1 input always produces the same v2 output
7. **String comparison**: All string matching is case-insensitive using ASCII lowercase

These guarantees ensure that two independent implementations produce byte-identical output for the same input.
