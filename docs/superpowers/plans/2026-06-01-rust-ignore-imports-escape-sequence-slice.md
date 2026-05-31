# Rust Ignore Imports Escape Sequence Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve import module specifiers and add Rust `ignoreImports` parity for escape-sequence rewriting of `require(...)` and dynamic `import(...)` sources.

**Architecture:** Extend `Options` with `ignore_imports`, teach the escape-sequence visitor to skip static import/export source fields unconditionally, and skip direct `require(...)` plus dynamic `import(...)` call arguments when `ignore_imports` is enabled. Ordinary string literals continue through existing escape-sequence encoding.

**Tech Stack:** Rust 1.96, Serde camelCase option deserialization, SWC AST visitor.

---

### Task 1: Failing API Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add import escape tests**

Add API tests proving static import specifiers stay unescaped under `unicodeEscapeSequence`, dynamic/require import strings stay unescaped when `ignoreImports` is enabled, and require strings still encode when `ignoreImports` is disabled.

- [ ] **Step 2: Run red test**

Run: `cargo test -p javascript-obfuscator import_string`

Expected: compile or assertion failure because `Options` does not yet expose `ignore_imports` and the escape visitor currently treats import call strings like ordinary strings.

### Task 2: Option And Visitor Skip Logic

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/escape_sequences.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/directive_placement.rs`

- [ ] **Step 1: Add option field**

Add `ignore_imports: Option<bool>` to `Options`; serde camelCase maps it to `ignoreImports`.

- [ ] **Step 2: Pass option into escape transform**

Extend `transform_escape_sequences` to accept `ignore_imports` and pass the option from `apply_transforms`.

- [ ] **Step 3: Skip import sources**

Override the relevant SWC visitor methods so static import/export source strings are not visited, and direct `require(...)`/dynamic `import(...)` call arguments are skipped when `ignore_imports` is true.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [ ] **Step 1: Run focused tests**

Run: `cargo test -p javascript-obfuscator import_string`.

- [ ] **Step 2: Run full checks**

Run: `cargo fmt --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `npx eslint src/**/*.ts`, and `npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts`.

- [ ] **Step 3: Commit and push**

Stage the Rust files and plan, commit with `feat: add rust ignore imports escape handling`, and push `codex/rust-rewrite-slice-1`.
