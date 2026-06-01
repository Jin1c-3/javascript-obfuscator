# Rust String Array Threshold Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust `stringArrayThreshold` option support for deterministic endpoint behavior.

**Architecture:** Extend the Rust `Options` model with `string_array_threshold` and pass it into the minimal string-array transform. The transform should skip extraction when the threshold is `0` or lower and continue extracting eligible literals when the threshold is positive; random probabilistic selection remains a later parity slice.

**Tech Stack:** Rust 1.96, Serde camelCase option deserialization, existing SWC string-array transform.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add threshold API test**

Add an API test proving `stringArray: true` plus `stringArrayThreshold: 0` keeps strings inline and does not emit the Rust storage array.

- [ ] **Step 2: Run red test**

Run: `cargo test -p javascript-obfuscator string_array_threshold`

Expected: compile failure because `Options` does not expose `string_array_threshold`, or assertion failure if the option exists but is ignored.

### Task 2: Option And Transform

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [ ] **Step 1: Add option field**

Add `string_array_threshold: Option<f64>` to `Options`; serde camelCase maps this to `stringArrayThreshold`.

- [ ] **Step 2: Thread threshold into transform**

Pass `options.string_array_threshold.unwrap_or(1.0)` into `transform_string_array`.

- [ ] **Step 3: Honor threshold endpoint**

If threshold is `<= 0.0`, return before traversing. Keep current extraction for threshold values above zero.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [ ] **Step 1: Run focused tests**

Run: `cargo test -p javascript-obfuscator string_array_threshold`.

- [ ] **Step 2: Run full checks**

Run: `cargo fmt --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `npx eslint src/**/*.ts`, and `npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts`.

- [ ] **Step 3: Commit and push**

Stage the Rust files and plan, commit with `feat: add rust string array threshold option`, and push `codex/rust-rewrite-slice-1`.
