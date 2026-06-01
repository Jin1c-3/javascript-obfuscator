# Rust String Array Self Defending Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend Rust `selfDefending` parity into Base64 and RC4 string-array wrappers without adding VMP behavior.

**Architecture:** The existing Rust string-array transform already builds Base64 and RC4 decode wrappers from compact JavaScript snippets. This slice threads `selfDefending` into `StringArrayTransformOptions` and adds the TypeScript-style atob guard expression to the generated Base64 decode path used by both wrappers.

**Tech Stack:** Rust, SWC AST parsing/codegen, serde options, Node runtime smoke tests.

---

### Task 1: RED API Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Write the failing tests**

Add API tests that enable `selfDefending` with `stringArrayEncoding: ["base64"]` and `["rc4"]`, assert generated code includes the atob guard marker `func.charCodeAt`, and execute generated code through Node to verify decoded string output still matches.

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p javascript-obfuscator string_array_self_defending`
Expected: FAIL because Rust string-array wrappers do not receive `selfDefending` and therefore do not emit the guard marker.

### Task 2: Transform Plumbing

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [ ] **Step 1: Add `self_defending` to string-array transform options**

Add `self_defending: bool` to `StringArrayTransformOptions` and pass `options.self_defending.unwrap_or(false)` from `transforms/mod.rs`.

- [ ] **Step 2: Update existing transform tests**

Update every existing `StringArrayTransformOptions` test literal with `self_defending: false` so unchanged behavior remains explicit.

### Task 3: Wrapper Guard Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [ ] **Step 1: Add guarded Base64 character expression**

When `self_defending` is true, have `create_base64_wrapper_statement` insert:

```js
let func = output + wrapperName;
let __ = ('' + function(){return 0;}).indexOf('\n') !== -1;
```

and wrap `String.fromCharCode(...)` with:

```js
((__ || func.charCodeAt(idx + 10) - 10 !== 0) ? String.fromCharCode(...) : bc)
```

- [ ] **Step 2: Apply same atob guard to RC4 wrapper**

Use the same guard in the RC4 wrapper Base64 decode phase, with `data` as the accumulating output string before RC4 decode.

- [ ] **Step 3: Run focused tests**

Run: `cargo test -p javascript-obfuscator string_array_self_defending`
Expected: PASS.

### Task 4: Verification And Ship

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
`git add crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/string_array.rs docs/superpowers/plans/2026-06-01-rust-string-array-self-defending-slice.md`
`git commit -m "feat: add rust string array self defending guard"`
`git push origin codex/rust-rewrite-slice-1`

Expected: commit and push succeed.
