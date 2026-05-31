# Rust Minimal String Array Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first Rust `stringArray` behavior slice by extracting plain string expression literals into deterministic storage.

**Architecture:** Add a new SWC transform that runs when `stringArray` is enabled. It traverses string literal expressions, skips directive statements plus reserved/import call strings already guarded by options, stores unique values in a top-level `_0x0` array, and replaces each literal with `_0x0[0xN]`. This intentionally does not implement wrappers, encodings, shuffling, rotation, or probabilistic thresholds yet.

**Tech Stack:** Rust 1.96, SWC AST visitor, existing Rust codegen/parser test harness.

---

### Task 1: Failing API Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add stringArray API tests**

Add tests proving `stringArray: true` extracts repeated string literals into a shared storage array and `stringArray: false` keeps literals inline.

- [ ] **Step 2: Run red test**

Run: `cargo test -p javascript-obfuscator string_array`

Expected: assertion failure because the Rust pipeline currently ignores the `string_array` option.

### Task 2: Transform Module

**Files:**
- Create: `crates/javascript-obfuscator/src/transforms/string_array.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Implement storage collection and replacement**

Create `transform_string_array` that collects unique string values, replaces each string expression with `_0x0[0xN]`, and appends one storage declaration when at least one value was collected.

- [ ] **Step 2: Preserve existing guards**

Skip directive string expression statements, reserved string values, and `require(...)`/dynamic `import(...)` arguments when `ignoreImports` is enabled.

- [ ] **Step 3: Wire into pipeline**

Run the transform after eval-call processing and before escape-sequence finalization so storage values still receive the existing final string escaping.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [ ] **Step 1: Run focused tests**

Run: `cargo test -p javascript-obfuscator string_array`.

- [ ] **Step 2: Run full checks**

Run: `cargo fmt --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `npx eslint src/**/*.ts`, and `npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts`.

- [ ] **Step 3: Commit and push**

Stage the Rust files and plan, commit with `feat: add rust minimal string array transform`, and push `codex/rust-rewrite-slice-1`.
