# Rust Export Specifier Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust export-specifier aliasing needed to preserve public export names when global identifiers are renamed.

**Architecture:** Add an `export_specifiers` SWC mutable visitor under the existing Rust transform runner. The visitor sets explicit `exported` aliases for shorthand named exports when `renameGlobals` is enabled, while preserving default output until the Rust identifier renamer is ported.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, existing serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports the aliasing substrate for `ExportSpecifierTransformer`:

- with `renameGlobals: true`, `export {foo};` becomes `export{foo as foo};`.
- with `renameGlobals: false`, `export {foo};` remains `export{foo};`.
- existing explicit aliases such as `export {foo as bar};` stay `export{foo as bar};`.
- namespace/default export specifiers are left unchanged.

This slice does not port Rust identifier renaming itself. Once identifier renaming is ported, the explicit alias created here gives the renamer a stable exported name to preserve.

## File Structure

- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the export specifier transform.
- Create `crates/javascript-obfuscator/src/transforms/export_specifiers.rs`: SWC visitor for named export aliasing.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for export specifier output.

## Task 1: Add Export Specifier Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/export_specifiers.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod export_specifiers;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/export_specifiers.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_export_specifiers(_program: &mut Program, _rename_globals: bool) {}
```

Add tests for:

- `export {foo};` with `rename_globals: true` should contain `export{foo as foo};`.
- `export {foo};` with `rename_globals: false` should contain `export{foo};`.
- `export {foo as bar};` with `rename_globals: true` should contain `export{foo as bar};`.
- `export * as ns from 'mod';` with `rename_globals: true` should contain `export*as ns from'mod';`.

Run:

```bash
cargo test -p javascript-obfuscator export_specifiers
```

Expected: FAIL only for the enabled shorthand alias case until implementation is added.

- [ ] **Step 3: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_aliases_export_specifier_when_rename_globals_enabled`
- `obfuscate_keeps_export_specifier_when_rename_globals_disabled`

Expected: alias-enabled test fails until the runner is wired.

## Task 2: Implement Export Specifier Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/export_specifiers.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- returns immediately when `rename_globals` is false.
- visits `ExportNamedSpecifier`.
- when `exported` is `None`, clones `orig` into `exported`.
- leaves existing explicit aliases unchanged.

This mirrors the preservation role of the TypeScript transformer without forcing visible `as` output when Rust global renaming is disabled.

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator export_specifiers
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call export specifier transform in `apply_transforms`**

Call it before future identifier-renaming work and before code generation:

```rust
export_specifiers::transform_export_specifiers(program, options.rename_globals.unwrap_or(false));
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator obfuscate_aliases_export_specifier_when_rename_globals_enabled
cargo test -p javascript-obfuscator obfuscate_keeps_export_specifier_when_rename_globals_disabled
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
git add docs/superpowers/plans/2026-06-01-rust-export-specifier-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust export specifier transform"
git push
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan ports the export-name preservation substrate used by the TypeScript converting transformer.
- It keeps full identifier renaming for a later slice.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_export_specifiers` mutates `Program`, matching the existing transform runner.
- The transform accepts `rename_globals: bool`, matching the current Rust `Options` field.
- It only mutates `ExportNamedSpecifier.exported`, which is the SWC equivalent of ESTree `exported`.
