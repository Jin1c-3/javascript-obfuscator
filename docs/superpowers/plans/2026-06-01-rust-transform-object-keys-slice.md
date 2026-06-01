# Rust Transform Object Keys Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-slice Rust support for the opt-in `transformObjectKeys` option on simple variable-declarator object literals.

**Architecture:** Add the option to Rust `Options`, then add a transform that expands `var object = {'foo': 'bar'}` into a temporary object plus bracket assignment statements. Keep the slice narrow: only single-declarator variable declarations with simple key-value object properties are transformed; other object-expression host contexts are left unchanged for later parity slices.

**Tech Stack:** Rust 1.96, SWC AST transforms, existing Rust identifier generator.

---

### Task 1: Failing Test

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add API transformObjectKeys test**

Add an API test with `transformObjectKeys: true`, `simplify: false`, and source:

```javascript
var object = {foo: 'bar', baz: 'bark'};
```

Assert the output contains:

```javascript
var _0x0={};
_0x0['foo']='bar';
_0x0['baz']='bark';
var object=_0x0;
```

- [x] **Step 2: Run red test**

Run:

```bash
cargo test -p javascript-obfuscator transform_object_keys
```

Expected: compile or test failure because Rust does not yet expose or implement `transformObjectKeys`.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/object_expression_keys.rs`

- [x] **Step 1: Add transformObjectKeys option**

Add `transform_object_keys: Option<bool>` to `Options`.

- [x] **Step 2: Add object expression keys transform module**

Create `object_expression_keys.rs` with a transform that expands only single-declarator variable declarations whose initializer is an object literal with only key-value properties.

- [x] **Step 3: Wire transform into pipeline**

Export the new module and call it after `object_expressions::transform_object_expressions`, passing identifier generator options.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator transform_object_keys
cargo test -p javascript-obfuscator object_expression
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
git add -- crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/object_expression_keys.rs docs/superpowers/plans/2026-06-01-rust-transform-object-keys-slice.md
```

Commit and push:

```bash
git commit -m "feat: add rust transform object keys option"
git push origin codex/rust-rewrite-slice-1
```
