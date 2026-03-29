# CKM SDK — Master Plan

**Status**: Ready for implementation
**Date**: 2026-03-29

---

## Objective

Extract CKM from VersionGuard into a standalone, multi-language SDK. Ship `ckm` on npm first, then PyPI, crates.io, and Go modules. Define the interface once, implement per language, prove with conformance tests.

## Upstream Context

- **Origin**: CKM currently lives at `/mnt/projects/versionguard/src/ckm/` (~500 LOC, 4 files)
- **Reference implementation**: Copied to `docs/specs/reference-*.ts` for porting
- **Reference v1 manifest**: Copied to `docs/specs/reference-ckm-v1.json` (1518 lines, real-world)
- **Architecture spec**: `docs/specs/CKM-SDK-ARCHITECTURE.md` (complete, 1000+ lines)
- **After Phase 1**: VersionGuard replaces `src/ckm/` with `import { createCkmEngine } from 'ckm'`

---

## Epic 0: Project Foundation

Bootstrap the monorepo, CI, and backbone documents that all language SDKs depend on.

### E0-T01: Initialize monorepo tooling
- Create root `package.json` with workspaces (`packages/*`)
- Add Biome config for formatting/linting
- Add root `tsconfig.json` base config
- Add `.gitignore`, `.editorconfig`, `LICENSE` (MIT)
- **AC**: `npm install` works at root, Biome format/lint passes on empty project

### E0-T02: Write ckm.json v2 JSON Schema (`ckm.schema.json`)
- Define the v2 schema as a JSON Schema draft 2020-12 document
- Include all types: `CkmMeta`, `CkmConcept` (with `slug`, `tags`), `CkmTypeRef` (with `canonical`/`original`/`enum`), `CkmOperation`, `CkmConstraint`, `CkmWorkflow`, `CkmConfigEntry`
- Include `$schema` URL: `https://ckm.dev/schemas/v2.json`
- Validate the reference-ckm-v1.json does NOT pass (it's v1)
- **AC**: Schema file exists at root. Can be used with `ajv` or any JSON Schema validator.
- **Depends on**: None

### E0-T03: Write INTERFACE.md (SDK Interface Definition)
- Define all SSoT types in language-agnostic pseudocode (from architecture spec section 4a)
- Define `CkmEngine` interface: `topics`, `getTopicIndex()`, `getTopicContent()`, `getTopicJson()`, `getManifest()`, `inspect()`
- Define factory: `createCkmEngine(manifest) -> CkmEngine`
- Define utilities: `validateManifest()`, `migrateV1toV2()`, `detectVersion()`
- Define `CkmCliAdapter` interface and `CkmAdapterOptions`
- Define `CkmFormatter` interface
- Include language mapping table (TS/Python/Rust/Go equivalents)
- **AC**: Document is complete, covers every public type and method. A developer in any language can implement from this alone.
- **Depends on**: E0-T02 (schema defines input types)

### E0-T04: Write SPEC.md (Algorithm Specification)
- Define topic derivation algorithm step by step:
  1. Filter concepts by `tags` containing `"config"` (v2) or suffix heuristic (v1 compat)
  2. Use `slug` field directly (v2) or derive from name (v1 compat)
  3. Match operations to topics via `tags` intersection (v2) or keyword matching (v1 compat)
  4. Group config entries by key prefix matching slug
  5. Link constraints by `enforcedBy` matching operation names
- Define v1 -> v2 migration algorithm
- Define version detection algorithm
- Define output formatting rules (terminal text format, JSON structure)
- Define progressive disclosure token budgets
- **AC**: Unambiguous, deterministic — two independent implementations from this spec produce identical output for identical input.
- **Depends on**: E0-T02, E0-T03

### E0-T05: Create conformance test fixtures
- Create `conformance/fixtures/minimal.ckm.json` — single concept, single operation (v2)
- Create `conformance/fixtures/multi-topic.ckm.json` — 3+ config concepts, operations, constraints
- Create `conformance/fixtures/polyglot.ckm.json` — language-agnostic, exercises all type refs
- Create `conformance/fixtures/v1-legacy.ckm.json` — real v1 manifest (subset of VG's)
- Create `conformance/fixtures/edge-cases.ckm.json` — empty arrays, null defaults, unknown topics
- For each fixture, create expected output files:
  - `expected/{name}/topics.json`
  - `expected/{name}/topicIndex.json`
  - `expected/{name}/topicContent-{slug}.txt`
  - `expected/{name}/topicJson-{slug}.json`
  - `expected/{name}/inspect.json`
  - `expected/{name}/validate.json`
- For v1-legacy: add `detectVersion.json` and `migrateResult.json`
- **AC**: Fixtures are valid JSON. Expected outputs are deterministic. At least 5 fixtures with full expected output suites.
- **Depends on**: E0-T02, E0-T03, E0-T04

### E0-T06: Set up CI pipeline
- GitHub Actions workflow: lint, format check, typecheck
- Conformance workflow placeholder (activated per-language as SDKs land)
- Path-filtered triggers: `ckm.schema.json`, `SPEC.md`, `INTERFACE.md`, `conformance/` trigger all language tests
- Per-language path filters: `packages/core/**` triggers TS tests only, etc.
- **AC**: CI runs on push/PR. Lint and format pass on initial commit.
- **Depends on**: E0-T01

---

## Epic 1: TypeScript Core Library (`ckm` on npm)

The reference implementation. First SDK to ship.

### E1-T01: Scaffold `packages/core` with package.json, tsconfig, Vite build
- `package.json`: name `ckm`, ESM-only, exports `.` and `./adapters/*`
- `tsconfig.json` extends root
- Vite or tsup build config (library mode, declaration files)
- Vitest config
- **AC**: `npm run build` produces `dist/` with `.js` + `.d.ts`. `npm run test` runs (empty suite passes).
- **Depends on**: E0-T01

### E1-T02: Implement SSoT types (`src/types.ts`)
- Translate every type from INTERFACE.md into TypeScript interfaces
- `CkmManifest`, `CkmMeta`, `CkmConcept`, `CkmProperty`, `CkmTypeRef`, `CanonicalType`
- `CkmOperation`, `CkmInput`, `CkmOutput`, `CkmConstraint`, `CkmWorkflow`, `CkmWorkflowStep`
- `CkmConfigEntry`, `CkmTopic`, `CkmTopicIndexEntry`, `CkmTopicIndex`
- `CkmInspectResult`, `CkmValidationResult`, `CkmValidationError`
- **AC**: All types from INTERFACE.md present. TSDoc on every public type. Compiles clean.
- **Depends on**: E0-T03, E1-T01

### E1-T03: Implement v1 types and migration (`src/migrate.ts`)
- Define v1 manifest types (from reference-types.ts)
- Implement `detectVersion(data) -> 1 | 2`
- Implement `migrateV1toV2(manifest) -> CkmManifest` following SPEC.md algorithm:
  - Wrap project/generated into meta block
  - Convert raw type strings to `CkmTypeRef` objects
  - Derive slugs from concept names (strip Config/Result/Options)
  - Infer tags from naming conventions
  - Rewrite config schema keys
- **AC**: Conformance tests for v1-legacy fixture pass. Round-trip: migrate then use engine produces correct topics.
- **Depends on**: E0-T04, E0-T05, E1-T02

### E1-T04: Implement schema validation (`src/validate.ts`)
- Implement `validateManifest(data) -> CkmValidationResult`
- Use `ckm.schema.json` for validation (bundle at build time or inline)
- Return structured errors with JSON pointer paths
- **AC**: Valid v2 manifests pass. Invalid manifests return correct error paths. v1 manifests fail validation (not v2).
- **Depends on**: E0-T02, E1-T02

### E1-T05: Implement CKM engine (`src/engine.ts`)
- Port `createCkmEngine()` from reference implementation
- Adapt to v2 types: use `slug` and `tags` instead of suffix heuristics
- Keep v1 compatibility: auto-migrate on construction if v1 detected
- Implement `CkmEngine` interface: `topics`, `getTopicIndex()`, `getTopicContent()`, `getTopicJson()`, `getManifest()`, `inspect()`
- **AC**: All conformance test fixtures pass. Engine handles both v1 and v2 input.
- **Depends on**: E1-T02, E1-T03, E0-T04

### E1-T06: Implement terminal formatter (`src/format.ts`)
- Plain text formatter (no chalk/color dependency)
- `formatTopicIndex()` — topic list with aligned columns
- `formatTopicContent()` — concepts, operations, config fields, constraints
- Token budget compliance: index < 300 tokens, topic < 800 tokens
- **AC**: Output matches conformance `topicContent-*.txt` fixtures exactly.
- **Depends on**: E1-T02

### E1-T07: Implement adapter types and registry (`src/adapters/`)
- `src/adapters/types.ts`: `CkmCliAdapter<T>`, `CkmAdapterOptions`, `CkmFormatter`
- `src/adapters/registry.ts`: `ADAPTER_TABLE` with lazy dynamic imports
- **AC**: Adapter interface matches INTERFACE.md. Registry supports lazy loading.
- **Depends on**: E1-T02

### E1-T08: Implement Commander.js adapter
- `src/adapters/commander.ts`
- Registers `ckm [topic]` command with `--json` and `--llm` flags
- Uses engine methods for all output
- Commander is a peerDependency (optional)
- **AC**: Integration test: create a Commander program, register adapter, parse `ckm calver --json`, verify output matches conformance.
- **Depends on**: E1-T05, E1-T06, E1-T07

### E1-T09: Write barrel export (`src/index.ts`)
- Export all public types
- Export `createCkmEngine`, `validateManifest`, `migrateV1toV2`, `detectVersion`
- Export adapter types from `./adapters`
- Subpath exports: `ckm` (core), `ckm/adapters/commander`, etc.
- **AC**: `import { createCkmEngine } from 'ckm'` works. `import { CommanderAdapter } from 'ckm/adapters/commander'` works.
- **Depends on**: E1-T05, E1-T07, E1-T08

### E1-T10: Run full conformance suite
- Wire up Vitest to load `conformance/fixtures/*.json`, parse, run through engine, compare to `conformance/expected/`
- Test every method: `topics`, `getTopicIndex`, `getTopicContent`, `getTopicJson`, `inspect`
- Test migration: v1-legacy fixture auto-migrates and produces correct topics
- Test validation: valid/invalid manifests
- **AC**: All conformance fixtures pass. Zero manual fixture skips.
- **Depends on**: E0-T05, E1-T05, E1-T04

### E1-T11: Publish `ckm` to npm
- Verify package.json metadata (description, keywords, repository, license, exports)
- Ensure `dist/` is clean, types are correct
- `npm publish` (or changesets if adopted)
- **AC**: `npm install ckm` works globally. `import { createCkmEngine } from 'ckm'` resolves.
- **Depends on**: E1-T09, E1-T10

---

## Epic 2: Standalone CLI (`ckm-cli` on npm)

### E2-T01: Scaffold `packages/cli`
- `package.json`: name `ckm-cli`, bin: `{ "ckm": "dist/main.js" }`
- Depends on `ckm` (workspace dependency)
- Uses Commander.js adapter internally
- **AC**: `npm run build` produces executable. `./dist/main.js --help` runs.
- **Depends on**: E1-T09

### E2-T02: Implement `ckm [topic]` command
- File resolution: `--file` flag, then `./ckm.json`, `./docs/ckm.json`, `./.ckm/ckm.json`
- Progressive disclosure: index, topic, `--json`, `--llm`
- **AC**: `ckm --file docs/ckm.json calver` displays topic. `--json` returns structured output.
- **Depends on**: E2-T01

### E2-T03: Implement `ckm validate <file>` command
- Loads file, runs `validateManifest()`, prints results
- Exit code 0 for valid, 1 for invalid
- **AC**: Valid v2 file passes. Invalid file prints errors with JSON pointer paths. v1 file reports schema version mismatch.
- **Depends on**: E2-T01

### E2-T04: Implement `ckm migrate <file>` command
- Loads v1 file, runs `migrateV1toV2()`, writes v2 output
- `--dry-run` flag shows diff without writing
- `--output` flag for custom output path
- **AC**: Migrating `reference-ckm-v1.json` produces valid v2 output. `--dry-run` does not modify filesystem.
- **Depends on**: E2-T01

### E2-T05: Implement `ckm inspect <file>` command
- Loads file, runs `engine.inspect()`, formats output
- Shows: project, language, generator, concept/operation/topic counts
- **AC**: Output matches `CkmInspectResult` shape. Works on both v1 and v2 files.
- **Depends on**: E2-T01

### E2-T06: Publish `ckm-cli` to npm
- **AC**: `npx ckm-cli` and `npx ckm-cli validate docs/ckm.json` work globally.
- **Depends on**: E2-T02, E2-T03, E2-T04, E2-T05

---

## Epic 3: TypeScript Adapter Expansion

### E3-T01: Implement Citty adapter (`src/adapters/citty.ts`)
- Registers CKM as a Citty subcommand
- Citty is peerDependency (optional)
- **AC**: Integration test with Citty. Conformance output matches.
- **Depends on**: E1-T07

### E3-T02: Implement oclif adapter (`src/adapters/oclif.ts`)
- Registers CKM as an oclif command class
- oclif is peerDependency (optional)
- **AC**: Integration test with oclif plugin pattern.
- **Depends on**: E1-T07

### E3-T03: Implement Clipanion adapter (`src/adapters/clipanion.ts`)
- Registers CKM as a Clipanion command
- Clipanion is peerDependency (optional)
- **AC**: Integration test with Clipanion.
- **Depends on**: E1-T07

---

## Epic 4: Python SDK (`ckm` on PyPI)

### E4-T01: Scaffold `packages/python` with pyproject.toml
- Package name: `ckm`
- Python 3.10+ minimum
- Optional extras: `ckm[click]`, `ckm[typer]`
- pytest for tests
- **AC**: `pip install -e .` works. `pytest` runs (empty suite passes).

### E4-T02: Implement SSoT types (`ckm/types.py`)
- Translate INTERFACE.md types to Python dataclasses (or Pydantic if preferred)
- All types present, docstrings match interface docs
- **AC**: Types importable. `from ckm.types import CkmManifest` works.
- **Depends on**: E0-T03

### E4-T03: Implement engine (`ckm/engine.py`)
- Port SPEC.md algorithm to Python
- `create_engine(manifest) -> CkmEngine`
- All engine methods: `topics`, `get_topic_index()`, `get_topic_content()`, `get_topic_json()`, `get_manifest()`, `inspect()`
- Auto-migrate v1 manifests
- **AC**: Conformance test fixtures pass.
- **Depends on**: E0-T04, E4-T02

### E4-T04: Implement validation and migration
- `validate_manifest(data) -> CkmValidationResult`
- `migrate_v1_to_v2(manifest) -> CkmManifest`
- `detect_version(data) -> int`
- **AC**: Conformance fixtures for validation and migration pass.
- **Depends on**: E4-T02

### E4-T05: Implement Click adapter
- `ckm/adapters/click_adapter.py`
- Registers CKM as a Click group command
- **AC**: Integration test with Click. Output matches conformance.
- **Depends on**: E4-T03

### E4-T06: Implement Typer adapter (with Rich output)
- `ckm/adapters/typer_adapter.py`
- Rich-formatted terminal output (optional — falls back to plain text)
- **AC**: Integration test with Typer. Rich formatting renders correctly.
- **Depends on**: E4-T03

### E4-T07: Run conformance suite and publish
- Wire pytest to load conformance fixtures
- All fixtures pass
- Publish to PyPI
- **AC**: `pip install ckm` works. `from ckm import create_engine` resolves.
- **Depends on**: E4-T03, E4-T04, E4-T05, E4-T06

---

## Epic 5: Rust SDK (`ckm` on crates.io)

### E5-T01: Scaffold `packages/rust` with Cargo.toml
- Crate name: `ckm`
- serde + serde_json for JSON handling
- Optional features: `clap` for adapter
- **AC**: `cargo build` succeeds. `cargo test` runs.

### E5-T02: Implement types (`src/types.rs`)
- `#[derive(Deserialize, Serialize)]` structs from INTERFACE.md
- **AC**: Can deserialize v2 `ckm.json` into `CkmManifest` struct.
- **Depends on**: E0-T03

### E5-T03: Implement engine (`src/engine.rs`)
- Port SPEC.md algorithm
- `CkmEngine::new(manifest)`, all interface methods
- **AC**: Conformance fixtures pass.
- **Depends on**: E0-T04, E5-T02

### E5-T04: Implement Clap adapter
- Feature-gated: `ckm = { features = ["clap"] }`
- **AC**: Integration test with Clap derive.
- **Depends on**: E5-T03

### E5-T05: Run conformance suite and publish
- **AC**: `cargo add ckm` works. All fixtures pass.
- **Depends on**: E5-T03, E5-T04

---

## Epic 6: Go SDK

### E6-T01: Scaffold `packages/go` with go.mod
- Module: `github.com/kryptobaseddev/ckm/go`
- **AC**: `go build ./...` succeeds. `go test ./...` runs.

### E6-T02: Implement types (`types.go`)
- Go structs with `json` struct tags from INTERFACE.md
- **AC**: Can unmarshal v2 `ckm.json` into `Manifest` struct.
- **Depends on**: E0-T03

### E6-T03: Implement engine (`engine.go`)
- Port SPEC.md algorithm
- `NewEngine(manifest)`, all interface methods
- **AC**: Conformance fixtures pass.
- **Depends on**: E0-T04, E6-T02

### E6-T04: Implement Cobra adapter
- **AC**: Integration test with Cobra.
- **Depends on**: E6-T03

### E6-T05: Implement urfave/cli adapter
- **AC**: Integration test.
- **Depends on**: E6-T03

### E6-T06: Run conformance suite and publish
- **AC**: `go get github.com/kryptobaseddev/ckm/go` works.
- **Depends on**: E6-T03, E6-T04, E6-T05

---

## Epic 7: forge-ts v2 Integration

### E7-T01: Add v2 schema generation to forge-ts
- `forge-ts build` produces v2 `ckm.json` by default
- Populates `meta` block, `concept.slug`, `concept.tags`, canonical types
- `--ckm-version 1` flag for legacy output
- **AC**: `forge-ts build` on VersionGuard produces valid v2 manifest that passes `ckm validate`.

### E7-T02: Resolve enum values in forge-ts
- Populate `type.enum` for string literal union types
- Example: `CalVerFormat` → `["YYYY.MM.DD", "YYYY.MM", ...]`
- **AC**: Enum values appear in v2 manifest.

### E7-T03: Resolve operation input types
- Replace `"unknown"` with actual canonical types where possible
- **AC**: Operation inputs have meaningful types instead of `"unknown"`.

---

## Epic 8: VersionGuard Migration

### E8-T01: Replace `src/ckm/` with `ckm` dependency
- Add `ckm` to VersionGuard's dependencies
- Update `src/cli.ts` imports: `import { createCkmEngine } from 'ckm'`
- Update `src/index.ts` re-export
- Remove `src/ckm/` directory
- **AC**: `vg ckm` command works identically. All 276 tests pass. Build succeeds.
- **Depends on**: E1-T11

### E8-T02: Update VersionGuard to use Commander adapter
- Replace manual Commander wiring with `import { CommanderAdapter } from 'ckm/adapters/commander'`
- `adapter.register(program, engine)`
- **AC**: `vg ckm`, `vg ckm calver`, `vg ckm --json` all work identically.
- **Depends on**: E1-T08, E8-T01

---

## Dependency Graph (Critical Path)

```
E0-T01 (monorepo) ─┬─> E0-T06 (CI)
                    └─> E1-T01 (scaffold core)

E0-T02 (schema) ──┬─> E0-T03 (INTERFACE.md)
                   └─> E1-T04 (validation)

E0-T03 (interface) ─┬─> E0-T04 (SPEC.md)
                     ├─> E1-T02 (TS types)
                     ├─> E4-T02 (Python types)
                     ├─> E5-T02 (Rust types)
                     └─> E6-T02 (Go types)

E0-T04 (spec) ──┬─> E0-T05 (fixtures)
                 ├─> E1-T05 (TS engine)
                 ├─> E4-T03 (Python engine)
                 ├─> E5-T03 (Rust engine)
                 └─> E6-T03 (Go engine)

E1-T05 (engine) ──> E1-T08 (Commander) ──> E1-T09 (barrel) ──> E1-T10 (conformance) ──> E1-T11 (publish)

E1-T11 (npm publish) ──> E8-T01 (VG migration)
```

**Critical path to first npm publish**: E0-T01 → E0-T02 → E0-T03 → E0-T04 → E0-T05 → E1-T01 → E1-T02 → E1-T03 → E1-T05 → E1-T06 → E1-T08 → E1-T09 → E1-T10 → E1-T11
