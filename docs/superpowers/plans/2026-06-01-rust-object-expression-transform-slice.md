# Rust Object Expression Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `ObjectExpressionTransformer` for object literal key normalization and shorthand expansion.

**Architecture:** Add an `object_expressions` SWC mutable visitor under the existing Rust transform runner. The visitor rewrites SWC object literal properties in place and keeps the TypeScript package facade unchanged.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, existing serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports only object literal conversion behavior:

- `{foo: value}` becomes `{'foo': value}`.
- `{foo}` becomes `{'foo': foo}`.
- `{foo() {}}` becomes `{'foo'() {}}`.
- Spread properties such as `{...source}` remain spread properties.

This slice does not port `ObjectExpressionKeysTransformer`, object key extraction to assignment statements, split strings, string arrays, property renaming, or source-map parity. The VMP/Pro removal remains unchanged.

## File Structure

- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the object expression transform.
- Create `crates/javascript-obfuscator/src/transforms/object_expressions.rs`: SWC visitor for object literal key normalization.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for object literal output.

## Task 1: Add Object Expression Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/object_expressions.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export object expression transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod object_expressions;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/object_expressions.rs` with a no-op `transform_object_expressions(_program: &mut Program)` and tests for:

- identifier key normalization: `const value = {foo: 1};` should contain `{'foo':0x1}` after the full local transform helper also runs number literals.
- shorthand expansion: `const value = {foo};` should contain `{'foo':foo}`.
- method key normalization: `const value = {foo() { return 1; }};` should contain `{'foo'(){return 0x1;}}`.
- spread preservation: `const value = {...source};` should still contain `{...source}`.

Run:

```bash
cargo test -p javascript-obfuscator object_expressions
```

Expected: FAIL until implementation is added.

- [ ] **Step 3: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_transforms_object_expression_identifier_key`
- `obfuscate_transforms_object_expression_shorthand_property`

Expected: they fail until the runner is wired.

## Task 2: Implement Object Expression Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/object_expressions.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- calls `object_lit.visit_mut_children_with(self)` first.
- iterates `ObjectLit.props`.
- skips `PropOrSpread::Spread`.
- converts `Prop::Shorthand(ident)` into `Prop::KeyValue(KeyValueProp { key: string_prop_name(ident.sym), value: Expr::Ident(ident) })`.
- converts `Prop::KeyValue` keys from `PropName::Ident` to `PropName::Str`.
- converts `Prop::Method`, `Prop::Getter`, and `Prop::Setter` keys from `PropName::Ident` to `PropName::Str`.
- leaves computed, numeric, bigint, and existing string keys unchanged.

Use single-quoted raw string keys for generated `Str` nodes.

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator object_expressions
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call object expression transform in `apply_transforms`**

Run order:

```rust
template_literals::transform_template_literals(program);
boolean_literals::transform_boolean_literals(program);
number_literals::transform_number_literals(program);
object_expressions::transform_object_expressions(program);
member_expressions::transform_member_expressions(
    program,
    options.property_bracketing.unwrap_or(true),
);
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator obfuscate_transforms_object_expression_identifier_key
cargo test -p javascript-obfuscator obfuscate_transforms_object_expression_shorthand_property
```

Expected: PASS.

- [ ] **Step 3: Run full Rust engine tests**

Run:

```bash
cargo test -p javascript-obfuscator
```

Expected: PASS.

## Task 4: Verification, Commit, Push

- [ ] **Step 1: Run Rust verification**

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: PASS. The napi test binary may print Node-API host-runtime load warnings while still exiting 0.

- [ ] **Step 2: Run TypeScript boundary verification**

Run:

```bash
npx eslint src/**/*.ts
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
```

Expected: PASS.

- [ ] **Step 3: Run Pro/VMP removal scan**

Run:

```bash
rg -n "Pro API|pro-api|ProApi|obfuscatePro|vmObfuscation|parseHtml|--vm-|--pro-api|VMP|VM Obfuscation|@vercel/blob" README.md src test typings index.ts package.json
```

Expected: no output.

- [ ] **Step 4: Commit and push**

Run:

```bash
git add docs/superpowers/plans/2026-06-01-rust-object-expression-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust object expression transform"
git push
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan adds another converting transformer from the TypeScript engine.
- It keeps object key extraction and string-array behavior for later slices.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_object_expressions` mutates `Program`, matching the existing transform runner.
- Generated object keys use `PropName::Str`, matching SWC's object literal shape.
- Shorthand expansion preserves the original identifier as the property value.
