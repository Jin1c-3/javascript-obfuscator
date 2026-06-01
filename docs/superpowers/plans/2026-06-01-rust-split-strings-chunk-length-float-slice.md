# Rust Split Strings Chunk Length Float Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Match TypeScript option normalization for fractional `splitStringsChunkLength` values.

**Architecture:** Keep the public Rust field as `Option<usize>`, but add a custom serde deserializer that accepts JSON numbers and floors fractional values. Preserve missing/null behavior as `None`.

**Tech Stack:** Rust 1.96, serde, serde_json.

---

### Task 1: Failing Test

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`

- [x] **Step 1: Add float deserialization test**

Add an options test that deserializes:

```json
{"splitStringsChunkLength": 5.6}
```

and asserts `split_strings_chunk_length == Some(5)`.

- [x] **Step 2: Run red test**

Run:

```bash
cargo test -p javascript-obfuscator split_strings_chunk_length_float
```

Expected: test fails because Rust currently expects a `usize`.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`

- [x] **Step 1: Add custom deserializer**

Add a helper that accepts missing/null, integer, and finite non-negative float values. Floor floats before converting to `usize`.

- [x] **Step 2: Wire deserializer to splitStringsChunkLength**

Annotate `split_strings_chunk_length` with the custom deserializer while preserving `#[serde(default)]`.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator split_strings_chunk_length_float
cargo test -p javascript-obfuscator options
```

- [x] **Step 2: Run full checks**

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npx eslint "src/**/*.ts"
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
rg -n "VMP|vmp|virtual machine|virtual-machine|virtualMachine" crates src test package.json
```

Expected: all commands exit 0 except the VMP scan, which exits 1 with no matches.

- [x] **Step 3: Commit and push**

Stage:

```bash
git add -- crates/javascript-obfuscator/src/options.rs docs/superpowers/plans/2026-06-01-rust-split-strings-chunk-length-float-slice.md
```

Commit and push:

```bash
git commit -m "feat: floor rust split strings chunk length"
git push origin codex/rust-rewrite-slice-1
```
