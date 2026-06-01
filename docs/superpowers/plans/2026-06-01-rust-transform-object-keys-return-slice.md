# Rust Transform Object Keys Return Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend Rust `transformObjectKeys` support to simple return-statement object literals.

**Architecture:** Reuse the existing object-expression-key extraction helpers. When a `return { ... }` statement has a simple key-value object literal argument, emit a temporary object declaration and property assignments before returning the temporary. Leave complex object literals and other host contexts unchanged.

**Tech Stack:** Rust 1.96, SWC AST transforms, existing Rust identifier generator.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/object_expression_keys.rs`

- [x] **Step 1: Add API return object test**

Add an API test with `transformObjectKeys: true`, `simplify: false`, and source:

```javascript
function getObject() { return {foo: 'bar', baz: 'bark'}; }
```

Assert the function body contains a generated temp object, two bracket assignments, and `return _0x0`.

- [x] **Step 2: Add transform return object test**

Add a transform-module unit test with the same source and assertions.

- [x] **Step 3: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator transform_object_keys_return
```

Expected: tests fail because return-statement object literals are not extracted yet.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/object_expression_keys.rs`

- [x] **Step 1: Extract return statement object literal**

Update `expand_statement` to recognize `Stmt::Return` with an object literal argument. If the object has only key-value properties, emit temp object setup statements plus a return statement for the temp identifier.

- [x] **Step 2: Keep complex returns untouched**

Add a module test that a return object with a spread property remains inline.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator transform_object_keys_return
cargo test -p javascript-obfuscator transform_object_keys
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
git add -- crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/transforms/object_expression_keys.rs docs/superpowers/plans/2026-06-01-rust-transform-object-keys-return-slice.md
```

Commit and push:

```bash
git commit -m "feat: transform rust return object keys"
git push origin codex/rust-rewrite-slice-1
```
