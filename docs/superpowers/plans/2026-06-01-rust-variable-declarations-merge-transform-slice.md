# Rust Variable Declarations Merge Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `VariableDeclarationsMergeTransformer` for the existing `simplify` option.

**Architecture:** Add a focused SWC statement-list transform that merges adjacent variable declaration statements of the same declaration kind. The transform runs from the existing Rust transform runner when `simplify` is enabled, after expression-statement merging, and treats non-variable statements plus different declaration kinds as merge boundaries.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports variable-declaration merging only:

- `simplify: true` merges adjacent `var` statements such as `var foo=1; var bar=2;` into `var foo=1,bar=2;`.
- Adjacent `let` statements merge only with `let`, and adjacent `const` statements merge only with `const`.
- Existing multi-declarator statements are flattened into the previous same-kind declaration.
- Non-variable statements break merge groups.
- `simplify: false` leaves the existing Rust output unchanged.

This slice does not port `BlockStatementSimplifyTransformer` or `IfStatementSimplifyTransformer`.

## File Structure

- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the variable-declarations merge transform.
- Create `crates/javascript-obfuscator/src/transforms/variable_declarations_merge.rs`: SWC visitor for statement-list variable declaration merging.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for `simplify` variable declaration behavior.

## Task 1: Add Variable Declaration Merge Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/variable_declarations_merge.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod variable_declarations_merge;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/variable_declarations_merge.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_variable_declarations_merge(_program: &mut Program, _enabled: bool) {}
```

Add tests for:

- `var foo=1;var bar=2;var baz=3;` with enabled `true` should contain `var foo=1,bar=2,baz=3;`.
- `var foo=1;var bar=2;` with enabled `false` should contain `var foo=1;var bar=2;`.
- `var foo=1;console.log(foo);var bar=2;var baz=3;` with enabled `true` should keep the console statement as a boundary and only merge the second group.
- `var foo=1;var bar=2;let baz=3;let bark=4;const hawk=5;const pork=6;` with enabled `true` should merge each same-kind group separately.

Run:

```bash
cargo test -p javascript-obfuscator variable_declarations_merge
```

Expected: FAIL for enabled transform cases until implementation is added.

- [ ] **Step 3: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_merges_variable_declarations_when_simplify_enabled`
- `obfuscate_keeps_variable_declarations_when_simplify_disabled`

Expected: enabled test fails until the runner is wired.

## Task 2: Implement Statement-List Variable Merge Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/variable_declarations_merge.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- returns early when `enabled` is false.
- visits children first.
- runs merging for `Script.body`, `Module.body` statement items, `BlockStmt.stmts`, and `SwitchCase.cons`.
- recognizes variable declaration statements as `Stmt::Decl(Decl::Var(_))`.
- appends declarators to the active previous declaration only when `VarDecl.kind` matches.
- flushes the active declaration whenever a non-variable statement or different variable kind is encountered.

Add helper functions:

```rust
fn merge_module_items(module_items: &mut Vec<ModuleItem>)
fn merge_statements(statements: &mut Vec<Stmt>)
fn push_var_declaration(active_declaration: &mut Option<VarDecl>, variable_declaration: VarDecl) -> Option<VarDecl>
fn try_take_var_declaration(statement: Stmt) -> Result<VarDecl, Stmt>
fn flush_var_declaration<T>(output: &mut Vec<T>, active: &mut Option<VarDecl>, wrap_statement: impl FnOnce(Stmt) -> T)
fn create_var_statement(variable_declaration: VarDecl) -> Stmt
```

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator variable_declarations_merge
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call variable-declaration merge with the existing simplify option**

Call it after expression-statement merging and before escape-sequence/directive placement:

```rust
    variable_declarations_merge::transform_variable_declarations_merge(
        program,
        options.simplify.unwrap_or(false),
    );
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator variable_declarations
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
git add docs/superpowers/plans/2026-06-01-rust-variable-declarations-merge-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust variable declaration merge transform"
git push origin codex/rust-rewrite-slice-1
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan ports one TypeScript simplifying transformer to Rust.
- It reuses the existing Rust `simplify` option added by the previous slice.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_variable_declarations_merge` mutates `Program`, matching the existing transform runner.
- The transform accepts `enabled: bool`, matching the existing Rust `Options.simplify` field.
- SWC variable declaration statements are represented by `Stmt::Decl(Decl::Var(_))`, and same-kind checks use `VarDecl.kind`.
