# Rust Expression Statements Merge Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `ExpressionStatementsMergeTransformer` for the `simplify` option.

**Architecture:** Add a focused SWC statement-list transform that merges adjacent expression statements into a single sequence expression statement. The transform runs from the existing Rust transform runner when `simplify` is enabled and preserves directive prologues plus non-expression statement boundaries.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports expression-statement merging only:

- `simplify: true` merges adjacent expression statements such as `foo(); bar(); baz();` into `foo(),bar(),baz();`.
- Merging works inside program bodies, block statements, and switch-case consequent statement lists.
- Non-expression statements such as function declarations and variable declarations break merge groups.
- String-literal directive statements such as `'use strict';` are not merged with following expressions.
- `simplify: false` leaves the existing Rust output unchanged.

This slice does not port `BlockStatementSimplifyTransformer`, `IfStatementSimplifyTransformer`, or `VariableDeclarationsMergeTransformer`.

## File Structure

- Modify `crates/javascript-obfuscator/src/options.rs`: add `simplify: Option<bool>`.
- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the expression-statement merge transform.
- Create `crates/javascript-obfuscator/src/transforms/expression_statements_merge.rs`: SWC visitor for statement-list merging.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for `simplify`.

## Task 1: Add Expression Merge Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/expression_statements_merge.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add the Rust option field**

In `crates/javascript-obfuscator/src/options.rs`, add:

```rust
    #[serde(default)]
    pub simplify: Option<bool>,
```

This uses the existing `#[serde(rename_all = "camelCase")]`, so JSON `simplify` maps directly to Rust `simplify`.

- [ ] **Step 2: Export transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod expression_statements_merge;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 3: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/expression_statements_merge.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_expression_statements_merge(_program: &mut Program, _enabled: bool) {}
```

Add tests for:

- `function foo(){bar();baz();bark();}` with enabled `true` should contain `function foo(){bar(),baz(),bark()}`.
- `function foo(){bar();baz();}` with enabled `false` should contain `function foo(){bar();baz()}`.
- `function foo(){'use strict';bar();baz();}` with enabled `true` should keep `'use strict';` as a directive and merge only `bar(),baz()`.
- `function foo(){a();function bar(){}b();c();const value=1;d();}` with enabled `true` should merge only groups separated by function and variable declarations.

Run:

```bash
cargo test -p javascript-obfuscator expression_statements_merge
```

Expected: FAIL for enabled transform cases until implementation is added.

- [ ] **Step 4: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_merges_expression_statements_when_simplify_enabled`
- `obfuscate_keeps_expression_statements_when_simplify_disabled`

Expected: enabled test fails until the runner is wired.

## Task 2: Implement Statement-List Merge Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/expression_statements_merge.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- returns early when `enabled` is false.
- visits children first.
- runs merging for `Script.body`, `Module.body` statement items, `BlockStmt.stmts`, and `SwitchCase.cons`.
- starts a merge group only for expression statements that are not string-literal directives.
- appends later expression statements to the previous group as a `SeqExpr`.
- flattens existing `SeqExpr` values when extending a group.
- flushes the active group whenever a non-expression statement or directive statement is encountered.

Add helper functions:

```rust
fn merge_script_statements(statements: &mut Vec<Stmt>)
fn merge_module_items(module_items: &mut Vec<ModuleItem>)
fn merge_statements(statements: &mut Vec<Stmt>)
fn is_mergeable_expression_statement(statement: &Stmt) -> bool
fn statement_into_expression(statement: Stmt) -> Expr
fn create_expression_statement(expressions: Vec<Box<Expr>>) -> Stmt
fn push_expression(expressions: &mut Vec<Box<Expr>>, expression: Box<Expr>)
```

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator expression_statements_merge
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call expression-statement merge near the end of the Rust transform pipeline**

Call it after labeled-statement handling and before escape-sequence/directive placement:

```rust
    expression_statements_merge::transform_expression_statements_merge(
        program,
        options.simplify.unwrap_or(false),
    );
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator simplify
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
git add docs/superpowers/plans/2026-06-01-rust-expression-statements-merge-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust expression statement merge transform"
git push origin codex/rust-rewrite-slice-1
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan ports one TypeScript simplifying transformer to Rust.
- It adds the missing Rust option required to drive the transform.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_expression_statements_merge` mutates `Program`, matching the existing transform runner.
- The transform accepts `enabled: bool`, matching the new Rust `Options` field.
- SWC sequence expressions are represented by `Expr::Seq(SeqExpr { exprs, .. })`, matching the TypeScript ESTree `SequenceExpression` behavior.
