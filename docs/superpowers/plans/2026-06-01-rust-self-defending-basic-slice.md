# Rust Self Defending Basic Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust rewrite support for the `selfDefending` option without adding any VMP behavior.

**Architecture:** The Rust pipeline gets a compact-safe helper insertion transform that runs with the existing custom helper group transforms. The option model accepts and serializes `selfDefending`, and the pipeline mirrors the TypeScript normalizer by forcing compact generation when it is enabled.

**Tech Stack:** Rust, SWC AST, serde option compatibility, Node runtime smoke tests, Cargo test/lint gates.

---

### Task 1: Option Compatibility

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`

- [ ] **Step 1: Write the failing test**

Add a test that deserializes `{"selfDefending": true}`, asserts `options.self_defending == Some(true)`, serializes it back, and checks the `selfDefending` JSON key.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p javascript-obfuscator deserializes_self_defending_option_for_option_compatibility`
Expected: FAIL because `Options` has no `self_defending` field.

- [ ] **Step 3: Write minimal implementation**

Add `pub self_defending: Option<bool>` with `#[serde(default)]` to `Options`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p javascript-obfuscator deserializes_self_defending_option_for_option_compatibility`
Expected: PASS.

### Task 2: Helper Transform

**Files:**
- Create: `crates/javascript-obfuscator/src/transforms/self_defending.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Write failing tests**

Add transform tests for enabled insertion, disabled skip, import preservation, and directive preservation. Add API runtime tests that enabled code contains `_0xselfDefending`, still runs user code in a Node VM, and disabled code omits the helper.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p javascript-obfuscator self_defending`
Expected: FAIL because no transform module or pipeline hook exists.

- [ ] **Step 3: Write minimal implementation**

Create a helper insertion transform following `debug_protection.rs` and `domain_lock.rs`: insert statements after script directives or module imports/directives. The helper should call a deterministic self-check function using `toString().search('(((.+)+)+)+$')` and `constructor(...)` shape, preserving normal runtime.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p javascript-obfuscator self_defending`
Expected: PASS.

### Task 3: Compact Normalization

**Files:**
- Modify: `crates/javascript-obfuscator/src/pipeline.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Write failing test**

Add an API test with `compact: false` and `selfDefending: true` that expects generated code to be compact.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p javascript-obfuscator obfuscate_self_defending_forces_compact_output`
Expected: FAIL because generation currently honors `compact: false`.

- [ ] **Step 3: Write minimal implementation**

Use `options.compact.unwrap_or(true) || options.self_defending.unwrap_or(false)` for code generation compactness.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p javascript-obfuscator obfuscate_self_defending_forces_compact_output`
Expected: PASS.

### Task 4: Verification And Ship

**Files:**
- Verify all changed files only by normal test and lint commands.

- [ ] **Step 1: Run focused Rust tests**

Run: `cargo test -p javascript-obfuscator self_defending`
Expected: PASS.

- [ ] **Step 2: Run full gate**

Run:
`cargo fmt --check`
`cargo test --workspace`
`cargo clippy --workspace --all-targets -- -D warnings`
`npx eslint "src/**/*.ts"`
`npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts`
`rg -n "VMP|vmp|virtual machine|virtual-machine|virtualMachine" crates src test package.json`

Expected: all pass; the VMP scan exits with no matches.

- [ ] **Step 3: Sync and publish**

Run:
`git fetch upstream`
`git merge-base --is-ancestor upstream/master HEAD`
`git add crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/self_defending.rs crates/javascript-obfuscator/src/pipeline.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-self-defending-basic-slice.md`
`git commit -m "feat: add rust self defending helper"`
`git push origin codex/rust-rewrite-slice-1`

Expected: commit and push succeed.
