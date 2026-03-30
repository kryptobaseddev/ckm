# CKM Producer Guide

How to generate `ckm.json` manifests for your project.

---

## Who is this for?

You're building a **documentation generator** — a tool that reads source code and produces a `ckm.json` file. forge-ts is one example. You could build a generator for Python (from docstrings), Rust (from rustdoc), Go (from godoc), or any language.

## What the SDK gives you

```
npm install ckm-sdk
```

```typescript
import type { CkmManifest, CkmConcept, CkmOperation } from 'ckm-sdk';
import { createManifestBuilder, validateManifest } from 'ckm-sdk';
```

Two things:

1. **TypeScript types** — `CkmManifest`, `CkmConcept`, `CkmOperation`, etc. Import these so your generator output is type-checked at compile time. No more `unknown[]` arrays.

2. **ManifestBuilder** — a fluent API that constructs valid manifests. Guarantees your output matches the schema without hand-rolling JSON.

## Option A: Use the Builder (recommended)

The builder handles schema compliance, ID generation, and structure for you.

```typescript
import { createManifestBuilder, validateManifest } from 'ckm-sdk';

function generateCKM(symbols, config) {
  const builder = createManifestBuilder(config.projectName, 'typescript')
    .generator(`my-generator@${version}`)
    .sourceUrl(config.repoUrl);

  // Walk your symbol graph and add what you find
  for (const symbol of symbols) {
    if (isConfigType(symbol)) {
      builder.addConcept(
        symbol.name,                    // "CalVerConfig"
        deriveSlug(symbol.name),        // "calver"
        symbol.description,             // "Configures CalVer validation."
        inferTags(symbol.name)          // ["config"]
      );

      for (const prop of symbol.properties) {
        builder.addConceptProperty(
          deriveSlug(symbol.name),       // which concept this belongs to
          prop.name,                     // "format"
          mapToCanonical(prop.type),     // "string"
          prop.description,              // "Calendar format."
          !prop.optional,                // required?
          prop.defaultValue              // "YYYY.MM.DD" or null
        );
      }
    }

    if (isOperation(symbol)) {
      builder.addOperation(
        symbol.name,                     // "validate"
        symbol.description,              // "Validates a CalVer string."
        inferOperationTags(symbol)       // ["calver", "validation"]
      );

      for (const param of symbol.params) {
        builder.addOperationInput(
          symbol.name,                   // which operation
          param.name,                    // "version"
          mapToCanonical(param.type),    // "string"
          !param.optional,               // required?
          param.description              // "Version string to validate."
        );
      }
    }

    if (isConstraint(symbol)) {
      builder.addConstraint(
        symbol.rule,                     // "Rejects future dates."
        symbol.enforcedBy,               // "validate"
        symbol.severity || 'error'       // "error" | "warning" | "info"
      );
    }

    if (isConfigEntry(symbol)) {
      builder.addConfig(
        symbol.key,                      // "calver.format"
        mapToCanonical(symbol.type),     // "string"
        symbol.description,              // "Calendar format."
        !symbol.optional,                // required?
        symbol.defaultValue              // "YYYY.MM.DD" or null
      );
    }
  }

  // Build and validate
  const manifest = builder.buildJson();
  const result = validateManifest(manifest);
  if (!result.valid) {
    console.error('Generated manifest is invalid:', result.errors);
    process.exit(1);
  }

  return manifest;
}
```

### Canonical Type Mapping

The builder expects canonical types. Map your language's types:

| Your type | Canonical |
|-----------|-----------|
| `string`, `str`, `String` | `"string"` |
| `boolean`, `bool` | `"boolean"` |
| `number`, `float`, `f64` | `"number"` |
| `int`, `i32`, `i64` | `"integer"` |
| `Array<T>`, `T[]`, `Vec<T>`, `list` | `"array"` |
| `Record`, `Map`, `HashMap`, `dict` | `"object"` |
| `null`, `None`, `nil`, `void` | `"null"` |
| `any`, `unknown`, `interface{}` | `"any"` |
| Custom types (CalVerFormat, etc.) | `"object"` (use `addConceptPropertyTyped` to preserve original) |

## Option B: Use Types Only (hand-build JSON)

If the builder doesn't fit your workflow, import just the types and build the manifest yourself:

```typescript
import type { CkmManifest, CkmConcept, CkmOperation, CkmTypeRef } from 'ckm-sdk';
import { validateManifest } from 'ckm-sdk';

const manifest: CkmManifest = {
  $schema: 'https://ckm.dev/schemas/v2.json',
  version: '2.0.0',
  meta: {
    project: 'my-tool',
    language: 'typescript',
    generator: 'my-generator@1.0.0',
    generated: new Date().toISOString(),
  },
  concepts: [/* ... */],
  operations: [/* ... */],
  constraints: [/* ... */],
  workflows: [/* ... */],
  configSchema: [/* ... */],
};

// Always validate before writing
const result = validateManifest(manifest);
if (!result.valid) {
  throw new Error(`Invalid manifest: ${JSON.stringify(result.errors)}`);
}

fs.writeFileSync('docs/ckm.json', JSON.stringify(manifest, null, 2));
```

The types give you compile-time safety. `validateManifest()` gives you runtime safety.

## Topic Control

Topics are how end-users browse the manifest (`mytool ckm calver`). You have two options:

### Auto-derived (default)

Every concept with a non-empty `slug` becomes a topic. The engine groups operations, constraints, and config entries by tag overlap and keyword matching. You don't need to do anything — just set `slug` and `tags` on your concepts.

### Producer-declared (full control)

Add a `topics` array to the manifest to define exact groupings:

```typescript
const manifest = builder.buildJson();

// Override auto-derivation with explicit topics
manifest.topics = [
  {
    name: 'getting-started',
    summary: 'First-time setup guide.',
    conceptIds: ['concept-config'],
    operationIds: ['op-init', 'op-validate'],
    constraintIds: [],
    configKeys: ['config.'],
  },
  {
    name: 'calver',
    summary: 'Calendar versioning configuration.',
    conceptIds: ['concept-calver-config'],
    operationIds: ['op-validate-calver'],
    constraintIds: ['constraint-no-future-dates'],
    configKeys: ['calver.'],
  },
];
```

When `topics` is present, the engine uses exactly those groupings. No heuristics.

## Extensions

Need to attach custom metadata that CKM doesn't have a field for?

```typescript
builder.addConcept('MyThing', 'mything', 'A thing.', ['config']);

// After building, add extensions
const manifest = builder.buildJson();
manifest.concepts[0].extensions = {
  'forge-ts.sourceFile': 'src/config.ts',
  'forge-ts.lineNumber': 42,
  'forge-ts.deprecated': true,
};
```

Extensions are freeform `Record<string, unknown>`. CKM passes them through untouched. Your tooling reads them; the SDK ignores them.

## Inspecting Your Output

After generating `ckm.json`, use the built-in CLI to verify it looks right:

```bash
# Validate the manifest
npx ckm-sdk validate docs/ckm.json

# See what topics were derived
npx ckm-sdk browse --file docs/ckm.json

# Drill into a specific topic
npx ckm-sdk browse calver --file docs/ckm.json

# Check manifest stats (concept/operation/topic counts)
npx ckm-sdk inspect docs/ckm.json

# See the full JSON output an LLM agent would get
npx ckm-sdk browse --json --file docs/ckm.json
```

## Validation Checklist

Before shipping your generator:

- [ ] Output passes `npx ckm-sdk validate docs/ckm.json` with zero errors
- [ ] Every concept has a non-empty `slug`
- [ ] Every concept has at least one tag
- [ ] Operations have `tags` that link them to concepts
- [ ] Config keys use dotted paths matching concept slugs (e.g., `calver.format`)
- [ ] The `meta` block is fully populated (project, language, generator, generated)
- [ ] `npx ckm-sdk browse --file docs/ckm.json` shows the topics you expect
- [ ] `npx ckm-sdk inspect docs/ckm.json` shows correct concept/operation counts
