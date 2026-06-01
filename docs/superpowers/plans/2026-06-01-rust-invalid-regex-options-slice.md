# Rust Invalid Regex Options Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust reject malformed regex option patterns instead of silently dropping them.

**Architecture:** Validate `reservedNames`, `reservedStrings`, and `forceTransformStrings` after parsing and before transforms. Reuse Rust `regex` compilation for the currently implemented Rust regex semantics, and report an options error that includes the option name and pattern.

**Tech Stack:** Rust 1.96, `regex`, existing Rust obfuscator diagnostics.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add reservedStrings invalid regex test**

Add an API test with `reservedStrings: ["["]` and valid source. Assert `obfuscate` returns an error whose message contains `reservedStrings` and `[`.

- [x] **Step 2: Add forceTransformStrings invalid regex test**

Add an API test with `forceTransformStrings: ["["]` and valid source. Assert the error message contains `forceTransformStrings` and `[`.

- [x] **Step 3: Add reservedNames invalid regex test**

Add an API test with `reservedNames: ["["]` and valid source. Assert the error message contains `reservedNames` and `[`.

- [x] **Step 4: Add parse precedence test**

Add an API test with invalid source and invalid regex options. Assert the returned error is still a JavaScript parse error.

- [x] **Step 5: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator invalid_regex_option
```

Expected: invalid-regex option tests fail because Rust currently ignores bad patterns.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/diagnostics.rs`
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/pipeline.rs`

- [x] **Step 1: Add options error variant**

Add `ObfuscatorError::Options(String)` and display it as an options error.

- [x] **Step 2: Add regex option validator**

Add `validate_regex_options(&Options) -> ObfuscatorResult<()>` in `options.rs`. It should compile every pattern in `reservedNames`, `reservedStrings`, and `forceTransformStrings`, returning an options error on the first invalid pattern.

- [x] **Step 3: Call validator after parse**

Call `validate_regex_options(&options)?` in `run_pipeline` after `parse_program` and before `apply_transforms`.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator invalid_regex_option
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
git add -- crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/diagnostics.rs crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/pipeline.rs docs/superpowers/plans/2026-06-01-rust-invalid-regex-options-slice.md
```

Commit and push:

```bash
git commit -m "feat: validate rust regex options"
git push origin codex/rust-rewrite-slice-1
```
