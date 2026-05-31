# Rust Class Field Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `ClassFieldTransformer` for public class method and property keys.

**Architecture:** Add a `class_fields` SWC mutable visitor under the existing Rust transform runner. The visitor converts eligible public class member keys into computed string keys and keeps private members, constructors, and reserved names untouched.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, existing serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports only class member key conversion:

- `class Foo { bar() {} }` becomes `class Foo { ['bar']() {} }`.
- `class Foo { property = 1; }` becomes `class Foo { ['property'] = 1; }`.
- `constructor() {}` remains `constructor() {}`.
- public string literal keys such as `'bar'() {}` become computed string keys unless reserved.
- private class members, computed keys, numeric keys, bigint keys, decorators, and static blocks are left to SWC/default behavior.

This slice does not port string array extraction, property renaming, class name renaming, control-flow behavior, or source-map parity. The TypeScript package facade remains on the existing TypeScript engine.

## File Structure

- Modify `crates/javascript-obfuscator/src/options.rs`: add `reserved_names: Option<Vec<String>>`.
- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the class field transform.
- Create `crates/javascript-obfuscator/src/transforms/class_fields.rs`: SWC visitor for class member key conversion.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for class method/property output.

## Task 1: Add Class Field Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/class_fields.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add option and module export with runner still not calling it**

Add `reserved_names: Option<Vec<String>>` to `Options`.

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod class_fields;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/class_fields.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_class_fields(_program: &mut Program, _reserved_names: &[String]) {}
```

Add tests for:

- method identifier key: `class Foo { bar() {} }` should contain `['bar'](){}`.
- property identifier key: `class Foo { property = value; }` should contain `['property']=value`.
- constructor key remains `constructor(){}`.
- reserved method key remains `bar(){}` when `reservedNames` contains `bar`.

Run:

```bash
cargo test -p javascript-obfuscator class_fields
```

Expected: FAIL until implementation is added.

- [ ] **Step 3: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_transforms_class_method_identifier_key`
- `obfuscate_transforms_class_property_identifier_key`

Expected: they fail until the runner is wired.

## Task 2: Implement Class Field Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/class_fields.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- traverses class member children first.
- handles `ClassMember::Method` and `ClassMember::ClassProp`.
- for `PropName::Ident` and `PropName::Str`, gets the public key name.
- skips names equal to `constructor`.
- skips names present in `reserved_names`.
- replaces the key with `PropName::Computed(ComputedPropName { expr: string literal })`.
- leaves computed, numeric, bigint, private members, constructors, and non-public keys unchanged.

Use single-quoted raw string literals inside computed keys.

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator class_fields
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call class field transform in `apply_transforms`**

Run order:

```rust
template_literals::transform_template_literals(program);
boolean_literals::transform_boolean_literals(program);
number_literals::transform_number_literals(program);
class_fields::transform_class_fields(program, options.reserved_names.as_deref().unwrap_or(&[]));
object_expressions::transform_object_expressions(program);
member_expressions::transform_member_expressions(
    program,
    options.property_bracketing.unwrap_or(true),
);
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator obfuscate_transforms_class_method_identifier_key
cargo test -p javascript-obfuscator obfuscate_transforms_class_property_identifier_key
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
git add docs/superpowers/plans/2026-06-01-rust-class-field-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust class field transform"
git push
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan adds another TypeScript converting transformer to the Rust path.
- It keeps string-array and rename interactions for later slices.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_class_fields` mutates `Program`, matching the existing transform runner.
- Generated keys use `PropName::Computed` to force bracket notation in class members.
- `reservedNames` is optional and defaults to an empty slice.
