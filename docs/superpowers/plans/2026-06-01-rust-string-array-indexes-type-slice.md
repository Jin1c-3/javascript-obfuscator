# Rust String Array Indexes Type Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust `stringArrayIndexesType` option support for the current minimal string-array transform.

**Architecture:** Extend the Rust `Options` model with a kebab-case `StringArrayIndexesType` enum and pass the selected list into the string-array transform. The transform will keep hexadecimal numeric indexes by default and emit hexadecimal numeric string indexes when the selected type is `hexadecimal-numeric-string`.

**Tech Stack:** Rust 1.96, Serde option deserialization, SWC AST literals/member expressions.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add API tests**

Add tests for `stringArrayIndexesType: ['hexadecimal-number']` and `stringArrayIndexesType: ['hexadecimal-numeric-string']` using the Rust API enum.

- [x] **Step 2: Run red test**

Run: `cargo test -p javascript-obfuscator string_array_index`

Expected: compile failure because Rust `Options` does not expose `string_array_indexes_type`, or assertion failure if the option exists but is ignored.

### Task 2: Option And Transform

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add enum and option field**

Add `StringArrayIndexesType` with `HexadecimalNumber` and `HexadecimalNumericString`, plus `string_array_indexes_type: Option<Vec<StringArrayIndexesType>>` to `Options`.

- [x] **Step 2: Thread option into transform**

Pass `options.string_array_indexes_type.as_deref().unwrap_or(&[])` into `transform_string_array`.

- [x] **Step 3: Emit selected index literal**

Use a number literal for default/hexadecimal-number, and a string literal containing `0xN` for hexadecimal-numeric-string.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run: `cargo test -p javascript-obfuscator string_array_index`.

- [x] **Step 2: Run full checks**

Run: `cargo fmt --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `npx eslint src/**/*.ts`, and `npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts`.

- [ ] **Step 3: Commit and push**

Stage the Rust files and plan, commit with `feat: add rust string array indexes type option`, and push `codex/rust-rewrite-slice-1`.
