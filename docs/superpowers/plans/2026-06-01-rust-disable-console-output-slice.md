# Rust Disable Console Output Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust support for the `disableConsoleOutput` option by injecting a deterministic console-disabling helper that preserves runtime behavior while suppressing console output.

**Architecture:** Keep this as a deterministic parity slice. The TypeScript engine chooses randomized helper names and host scopes; Rust will use stable names and prepend the helper to the program body after imports and directive prologues. Do not add VMP behavior.

**Tech Stack:** Rust 1.96, SWC AST, Node runtime checks in Rust API tests.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add runtime suppression test**

Create an API test that deserializes options from JSON with `disableConsoleOutput: true`, obfuscates code that calls common console methods, runs the output with Node, and asserts stdout/stderr are empty.

- [x] **Step 2: Add disabled behavior test**

Create an API test that deserializes options from JSON with `disableConsoleOutput: false`, runs a generated `console.log`, and asserts output still appears.

- [x] **Step 3: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator disable_console_output
```

Expected: suppression test fails because Rust currently ignores `disableConsoleOutput`.

### Task 2: Console Output Transform

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Add: `crates/javascript-obfuscator/src/transforms/console_output.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add option field**

Add `disable_console_output: Option<bool>` with serde camelCase support.

- [x] **Step 2: Add helper injection transform**

Parse and insert a deterministic helper that disables:

```text
log, warn, info, error, exception, table, trace
```

The helper should create/normalize the global `console` object and replace each method with a bound no-op that keeps `toString` compatible with the original function where possible. Helper insertion and string-array declaration insertion must preserve existing directive prologues such as `"use strict"`.

- [x] **Step 3: Wire transform**

Call the transform from `apply_transforms` when `options.disable_console_output.unwrap_or(false)` is true. Insert helper statements after imports in modules and at the start of scripts.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator disable_console_output
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

- [ ] **Step 3: Commit and push**

Stage:

```bash
git add -- crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/console_output.rs crates/javascript-obfuscator/src/transforms/string_array.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-disable-console-output-slice.md
```

Commit and push:

```bash
git commit -m "feat: add rust disable console output option"
git push origin codex/rust-rewrite-slice-1
```
