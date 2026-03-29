# CKM — Codebase Knowledge Manifest

**Machine-readable operational knowledge for CLI tools. Any language. Any framework. One contract.**

---

## What is CKM?

CKM bridges the gap between API documentation and actionable help. While `llms.txt` tells you what functions exist, CKM tells you what the tool **does**, what **concepts** it has, what **config** controls what **behavior**, and what **constraints** are enforced.

A `ckm.json` file is the universal contract. Any generator produces it. Any SDK consumes it. Any adapter wires it into any CLI framework.

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

## Why CKM Exists

### The Problem

Every CLI tool reinvents help output. Humans get `--help` text. LLM agents get nothing structured. Topic-based help requires manual maintenance. Config documentation drifts from implementation.

### The Solution

CKM provides a structured manifest (`ckm.json`) that captures:

| Section | What it answers | Source |
|---------|----------------|--------|
| **concepts** | "What domain objects does this tool have?" | Exported interfaces/types |
| **operations** | "What can I do with this tool?" | Exported functions |
| **constraints** | "What rules are enforced?" | Validation logic |
| **workflows** | "How do I accomplish X?" | Annotated composite functions |
| **configSchema** | "What config controls what behavior?" | Config type properties |

The SDK then auto-derives **topics** from this manifest — zero manual mapping. A human types `mytool ckm calver` and gets formatted help. An LLM agent adds `--json` and gets structured data. Progressive disclosure keeps token budgets tight.

## Design Principles

### 1. Universal Contract

`ckm.json` is language-agnostic. A TypeScript project, a Rust crate, a Python package, and a Go module all produce the same schema. The SDK reads it identically everywhere.

### 2. Batteries Included, Framework Agnostic

Install `ckm`, wire one adapter, get topic-based help with human and machine output. Works with Commander.js, Citty, oclif, Clipanion (TS), Click, Typer (Python), Clap (Rust), Cobra (Go).

### 3. Spec-Based Backbone

The backbone is NOT a compiled binary. It is:
- **`ckm.schema.json`** — what goes IN (the JSON Schema contract)
- **`INTERFACE.md`** — what comes OUT (the API every SDK exposes)
- **`SPEC.md`** — the deterministic algorithm (how input becomes output)
- **`conformance/`** — the proof (test fixtures every implementation must pass)

Each language implements natively. No WASM. No FFI. No shared runtime.

### 4. Progressive Disclosure

CKM mandates four disclosure levels as a protocol requirement:

| Level | Command | Audience | Token Budget |
|-------|---------|----------|-------------|
| 0 | `ckm` | Human / Agent discovery | 300 |
| 1 | `ckm <topic>` | Human / Agent drill-down | 800 |
| 1J | `ckm <topic> --json` | Agent structured | 1200 |
| 2 | `ckm --json` | Agent full index | 3000 |

### 5. Zero Manual Maintenance

Topics are auto-derived from the manifest structure. No topic mapping files. No manually curated help text. The generator (forge-ts or any tool) extracts knowledge from source code. The SDK displays it.

## What CKM is NOT

- **Not a documentation generator.** CKM does not parse source code. Generators like forge-ts do that. CKM consumes the output.
- **Not a replacement for `--help`.** CKM is a structured complement. It provides topic-based, progressive knowledge that `--help` flags cannot.
- **Not tied to any organization.** CKM is an open, unscoped package (`ckm` on npm/PyPI/crates.io). Any tool can adopt it.

## Origin

CKM originated as a module inside [VersionGuard](https://github.com/kryptobaseddev/versionguard) (v0.4.0, 2026). After proving the concept — auto-derived topics from forge-ts manifests, progressive disclosure for LLM agents, zero-config integration — it was extracted into a standalone SDK to serve any CLI tool in any language.

## Package Names

| Ecosystem | Package | Registry |
|-----------|---------|----------|
| TypeScript/JS | `ckm` | npm |
| CLI binary | `ckm-cli` | npm |
| Python | `ckm` | PyPI |
| Rust | `ckm` | crates.io |
| Go | `github.com/kryptobaseddev/ckm/go` | Go modules |

No scopes. No org prefixes. Universal.

## Relationship to forge-ts

- **forge-ts** = generation (produces `ckm.json` from TypeScript source code)
- **CKM SDK** = consumption/display (reads `ckm.json`, provides help/topics/adapters)
- Any tool can generate a valid `ckm.json` — forge-ts is one generator, not the only one
- Phase 6 of the rollout adds native v2 support to forge-ts

## Relationship to VersionGuard

- VersionGuard was CKM's incubator and first consumer
- After extraction, VersionGuard depends on `ckm` as a library (replaces `src/ckm/`)
- VersionGuard's `vg ckm` command becomes: `import { createCkmEngine } from 'ckm'`
