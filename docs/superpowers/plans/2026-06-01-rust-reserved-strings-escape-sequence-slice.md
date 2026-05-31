# Rust Reserved Strings Escape Sequence Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust pipeline parity for the `reservedStrings` option in escape-sequence finalization.

**Architecture:** Extend the Rust `Options` model with `reserved_strings`, thread that option into `transform_escape_sequences`, and skip escape-sequence raw rewriting for string literals whose value matches one of the reserved string patterns. This slice uses substring matching as the Rust bridge currently lacks a JavaScript-compatible regex engine.

**Tech Stack:** Rust 1.96, Serde camelCase option deserialization, SWC AST visitor.

---

### Task 1: Failing API Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add tests for reserved strings**

Add one API test for `unicodeEscapeSequence: true` that keeps `foo` unescaped while still encoding `bar`, and one API test for a reserved string containing spaces and quotes.

- [ ] **Step 2: Run red test**

Run: `cargo test -p javascript-obfuscator reserved_string`

Expected: compile or test failure because `Options` does not yet expose `reserved_strings` and the Rust escape transformer does not honor it.

### Task 2: Option And Transform

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/escape_sequences.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Add option field**

Add `reserved_strings: Option<Vec<String>>` to `Options`; serde camelCase maps this to `reservedStrings` for bridge consumers.

- [ ] **Step 2: Skip reserved strings in escape transform**

Pass reserved strings from `apply_transforms` into `transform_escape_sequences`. In `visit_mut_str`, if the string literal value contains any reserved pattern, return before setting escaped raw text.

- [ ] **Step 3: Update unit tests**

Adjust existing escape-sequence unit test helpers for the new transform signature and add focused reserved-string tests.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [ ] **Step 1: Run focused tests**

Run: `cargo test -p javascript-obfuscator reserved_string`.

- [ ] **Step 2: Run full checks**

Run: `cargo fmt --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `npx eslint src/**/*.ts`, and `npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts`.

- [ ] **Step 3: Commit and push**

Stage the Rust files and plan, commit with `feat: add rust reserved strings escape handling`, and push `codex/rust-rewrite-slice-1`.
