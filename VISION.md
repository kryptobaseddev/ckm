# CKM — Codebase Knowledge Manifest

**Machine-readable operational knowledge for CLI tools. Any language. Any framework. One contract. One implementation.**

---

## What is CKM?

CKM bridges the gap between API documentation and actionable help. While `llms.txt` tells you what functions exist, CKM tells you what the tool **does**, what **concepts** it has, what **config** controls what **behavior**, and what **constraints** are enforced.

A `ckm.json` file is the universal contract. Any generator produces it. A single Rust core consumes it. Thin FFI wrappers expose it to every language. Any adapter wires it into any CLI framework.

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
                    +-------------+
                          |
              +-----------+-----------+
              |           |           |
              v           v           v
         napi-rs      PyO3        CGo/WASM
         (Node)     (Python)       (Go)
              |           |           |
              v           v           v
         YOUR CLI    YOUR CLI    YOUR CLI
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

### 2. Single Implementation, Multiple Surfaces

**The CKM engine is implemented exactly once in Rust.** This is the Single Source of Truth. Every other language consumes it through thin FFI wrappers:

- **Node.js/TypeScript**: napi-rs 3.8+ generates native `.node` bindings with auto-generated `.d.ts` types, plus WASM fallback
- **Python**: PyO3 + Maturin generates native wheels for PyPI
- **Go**: CGo FFI or WASM via wazero
- **Rust**: Direct dependency — no wrapper needed

This eliminates drift. When the algorithm changes, it changes once in Rust. All language surfaces automatically follow.

### 3. Why Rust Core, Not Spec-Based

The original architecture proposed independent implementations in each language, guided by spec documents (INTERFACE.md, SPEC.md). This was rejected because:

- **Drift is inevitable.** Four independent implementations of the same algorithm *will* diverge on edge cases, even with conformance tests.
- **Spec docs aren't executable.** A prose specification can be ambiguous. Code cannot.
- **Maintenance scales linearly.** One bugfix in Rust vs. four bugfixes in four languages.
- **The algorithm is small.** ~500 LOC of pure data transformation. The overhead of FFI wrappers is negligible compared to the cost of maintaining four codebases.

The spec docs (INTERFACE.md, SPEC.md) remain as documentation of what the Rust code does — valuable for understanding, but not the source of truth.

### 4. Batteries Included, Framework Agnostic

Install `ckm`, wire one adapter, get topic-based help with human and machine output. Works with Commander.js, Citty, oclif, Clipanion (TS), Click, Typer (Python), Clap (Rust), Cobra (Go).

### 5. Progressive Disclosure

CKM mandates four disclosure levels as a protocol requirement:

| Level | Command | Audience | Token Budget |
|-------|---------|----------|-------------|
| 0 | `ckm` | Human / Agent discovery | 300 |
| 1 | `ckm <topic>` | Human / Agent drill-down | 800 |
| 1J | `ckm <topic> --json` | Agent structured | 1200 |
| 2 | `ckm --json` | Agent full index | 3000 |

### 6. Zero Manual Maintenance

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
| TypeScript/JS | `ckm` | npm (via napi-rs native + WASM) |
| CLI binary | `ckm-cli` | npm + crates.io |
| Python | `ckm` | PyPI (via PyO3 native wheel) |
| Rust | `ckm` | crates.io (direct dependency) |
| Go | `github.com/kryptobaseddev/ckm/go` | Go modules (via CGo/WASM) |

No scopes. No org prefixes. Universal.

## Relationship to forge-ts

- **forge-ts** = generation (produces `ckm.json` from TypeScript source code)
- **CKM SDK** = consumption/display (reads `ckm.json`, provides help/topics/adapters)
- Any tool can generate a valid `ckm.json` — forge-ts is one generator, not the only one

## Relationship to VersionGuard

- VersionGuard was CKM's incubator and first consumer
- After extraction, VersionGuard depends on `ckm` as a library (replaces `src/ckm/`)
- VersionGuard's `vg ckm` command becomes: `import { createCkmEngine } from 'ckm'`
