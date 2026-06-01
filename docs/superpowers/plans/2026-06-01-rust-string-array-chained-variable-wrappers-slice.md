# Rust String Array Chained Variable Wrappers Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-slice Rust support for `stringArrayWrappersChainedCalls` through variable string-array scope wrappers.

**Architecture:** The existing Rust transform already emits root variable wrappers and rewrites string literals to wrapper calls. This slice adds option compatibility and a post-string-array scope pass that, for variable wrappers only, inserts deterministic local aliases inside function bodies and rewrites direct wrapper calls in that lexical scope to call the local alias; nested scopes chain their alias to the parent alias.

**Tech Stack:** Rust, SWC AST visitors, serde options, Node runtime smoke tests.

---

### Task 1: Option Compatibility

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`

- [ ] **Step 1: Write the failing test**

Add a test that deserializes `{"stringArrayWrappersChainedCalls": true}`, asserts `options.string_array_wrappers_chained_calls == Some(true)`, serializes it back, and checks the JSON key.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p javascript-obfuscator deserializes_string_array_wrappers_chained_calls_option_for_option_compatibility`
Expected: FAIL because the Rust options struct does not expose this field.

- [ ] **Step 3: Write minimal implementation**

Add `pub string_array_wrappers_chained_calls: Option<bool>` with `#[serde(default)]` beside the other string-array wrapper options.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p javascript-obfuscator deserializes_string_array_wrappers_chained_calls_option_for_option_compatibility`
Expected: PASS.

### Task 2: Chained Variable Wrapper Behavior

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [ ] **Step 1: Write failing API tests**

Add a runtime test where `stringArrayWrappersChainedCalls: true`, `stringArrayWrappersCount: 1`, and `stringArrayWrappersType: "variable"` obfuscates nested functions. Assert generated code contains `_0xscopeWrapper0=_0x2` and `_0xscopeWrapper1=_0xscopeWrapper0`, and Node prints `foobar`. Add a disabled test that `stringArrayWrappersChainedCalls: false` omits `_0xscopeWrapper`.

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p javascript-obfuscator string_array_chained_variable`
Expected: FAIL because Rust currently ignores the option and always calls root wrappers directly.

- [ ] **Step 3: Implement variable scope chaining**

Thread `string_array_wrappers_chained_calls` through `StringArrayTransformOptions`. When enabled, wrappers exist, and wrapper type is `Variable`, traverse function and block-arrow bodies. For each lexical body with direct string-array wrapper calls, insert `const _0xscopeWrapperN = <parent-wrapper>;` after directives, rewrite direct calls in that body to `_0xscopeWrapperN`, and let nested bodies chain to the nearest parent `_0xscopeWrapperN`.

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p javascript-obfuscator string_array_chained_variable`
Expected: PASS.

### Task 3: Verification And Ship

**Files:**
- Verify all changed files through the standard gate.

- [ ] **Step 1: Run full gate**

Run:
`cargo fmt --check`
`cargo test --workspace`
`cargo clippy --workspace --all-targets -- -D warnings`
`npx eslint "src/**/*.ts"`
`npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts`
`rg -n "VMP|vmp|virtual machine|virtual-machine|virtualMachine" crates src test package.json`

Expected: all pass; the VMP scan exits with no matches.

- [ ] **Step 2: Sync and publish**

Run:
`git fetch upstream`
`git merge-base --is-ancestor upstream/master HEAD`
`git add crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/string_array.rs docs/superpowers/plans/2026-06-01-rust-string-array-chained-variable-wrappers-slice.md`
`git commit -m "feat: add rust string array chained variable wrappers"`
`git push origin codex/rust-rewrite-slice-1`

Expected: commit and push succeed.
