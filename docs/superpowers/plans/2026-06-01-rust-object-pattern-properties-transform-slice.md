# Rust Object Pattern Properties Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `ObjectPatternPropertiesTransformer` for object destructuring shorthand expansion.

**Architecture:** Add an `object_pattern_properties` SWC mutable visitor under the existing Rust transform runner. The visitor converts shorthand object pattern properties into key-value properties, while preserving top-level shorthand when `renameGlobals` is disabled.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, existing serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports object pattern shorthand expansion:

- with `renameGlobals: true`, top-level `const {foo} = source;` becomes `const {foo:foo}=source;`.
- with `renameGlobals: false`, top-level `const {foo} = source;` remains `const {foo}=source;`.
- inside function scopes, `function run({foo}) {}` becomes `function run({foo:foo}) {}`.
- default object pattern values such as `const {foo = 1} = source;` become `const {foo:foo=0x1}=source;` when transformed.

This slice does not port full scope analysis, identifier renaming, rest-element renaming, or source-map parity. It uses a conservative Rust scope heuristic: top-level object patterns are treated as global unless they are inside a function-like node or static block.

## File Structure

- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the object pattern properties transform.
- Create `crates/javascript-obfuscator/src/transforms/object_pattern_properties.rs`: SWC visitor for object pattern shorthand conversion.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for object pattern output.

## Task 1: Add Object Pattern Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/object_pattern_properties.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod object_pattern_properties;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/object_pattern_properties.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_object_pattern_properties(_program: &mut Program, _rename_globals: bool) {}
```

Add tests for:

- `const {foo} = source;` with `rename_globals: true` should contain `const {foo:foo}=source;`.
- `const {foo} = source;` with `rename_globals: false` should contain `const {foo}=source;`.
- `function run({foo}) {}` with `rename_globals: false` should contain `function run({foo:foo}){}`.
- `const {foo = 1} = source;` with `rename_globals: true` should contain `const {foo:foo=1}=source;`.

Run:

```bash
cargo test -p javascript-obfuscator object_pattern_properties
```

Expected: FAIL for the cases that require expansion until implementation is added.

- [ ] **Step 3: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_transforms_object_pattern_when_rename_globals_enabled`
- `obfuscate_keeps_top_level_object_pattern_when_rename_globals_disabled`

Expected: enabled transform test fails until the runner is wired.

## Task 2: Implement Object Pattern Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/object_pattern_properties.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- tracks `function_depth` by incrementing around function bodies.
- tracks `static_block_depth` by incrementing around static block bodies.
- visits object pattern children first.
- transforms `ObjectPatProp::Assign` into `ObjectPatProp::KeyValue` only when `rename_globals` is true, or `function_depth > 0`, or `static_block_depth > 0`.
- for `{foo}`, creates `KeyValuePatProp { key: PropName::Ident(foo), value: Pat::Ident(foo) }`.
- for `{foo = expr}`, creates `KeyValuePatProp { key: PropName::Ident(foo), value: Pat::Assign(AssignPat { left: Pat::Ident(foo), right: expr }) }`.
- leaves `KeyValue` and `Rest` object pattern properties unchanged.

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator object_pattern_properties
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call object pattern transform in `apply_transforms`**

Call it before object expression/member conversions:

```rust
object_pattern_properties::transform_object_pattern_properties(
    program,
    options.rename_globals.unwrap_or(false),
);
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator obfuscate_transforms_object_pattern_when_rename_globals_enabled
cargo test -p javascript-obfuscator obfuscate_keeps_top_level_object_pattern_when_rename_globals_disabled
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
git add docs/superpowers/plans/2026-06-01-rust-object-pattern-properties-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust object pattern properties transform"
git push
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan ports another TypeScript converting transformer to Rust.
- It keeps full identifier renaming and full lexical scope analysis for later slices.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_object_pattern_properties` mutates `Program`, matching the existing transform runner.
- The transform accepts `rename_globals: bool`, matching the current Rust `Options` field.
- SWC `AssignPatProp` maps to ESTree object pattern shorthand properties.
