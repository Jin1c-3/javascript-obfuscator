# Rust Object Expression Computed String Key Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align Rust object-expression conversion with TypeScript for computed string literal keys.

**Architecture:** The TypeScript always-on `ObjectExpressionTransformer` converts object properties like `{['foo']: value}` into non-computed string keys. Extend the existing Rust `object_expressions` transform to recognize `PropName::Computed` whose expression is a string literal and replace it with `PropName::Str`. Do not add VMP behavior.

**Tech Stack:** Rust 1.96, SWC AST object expression transform.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/object_expressions.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add transform test**

Add a unit test that transforms:

```javascript
const value = {['foo']: bar};
```

and asserts:

```rust
assert!(code.contains("const value={'foo':bar}"), "{code}");
```

- [x] **Step 2: Add API test**

Add an API test with `stringArray: false`, `propertyBracketing: false`, and assert the same output shape.

- [x] **Step 3: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator computed_string_object
```

Expected: failures because Rust currently keeps computed string keys as `['foo']`.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/object_expressions.rs`

- [x] **Step 1: Handle computed string property names**

In `transform_property_name`, add support for:

```rust
PropName::Computed(computed_property_name)
```

when `computed_property_name.expr` is `Expr::Lit(Lit::Str(_))`.

- [x] **Step 2: Reuse existing raw-string creation**

Build the replacement with existing `create_string_property_name(&name)` so escaping stays consistent with identifier key conversion.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator computed_string_object
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
git add -- crates/javascript-obfuscator/src/transforms/object_expressions.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-object-expression-computed-string-key-slice.md
```

Commit and push:

```bash
git commit -m "feat: transform rust computed object string keys"
git push origin codex/rust-rewrite-slice-1
```
