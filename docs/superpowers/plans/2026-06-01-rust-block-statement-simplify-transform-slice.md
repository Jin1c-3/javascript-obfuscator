# Rust Block Statement Simplify Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust return-oriented portion of `BlockStatementSimplifyTransformer` for the existing `simplify` option.

**Architecture:** Add a focused SWC block-statement visitor that rewrites trailing expression statements plus a final return statement into one return statement with a sequence expression. The transform runs after the existing Rust expression-statement and variable-declaration merge transforms, while preserving directive prologues and avoiding unreachable-code rewrites.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports the high-value block-statement behavior not already covered by expression-statement merging:

- `simplify: true` rewrites `foo(); bar(); return baz();` to `return foo(),bar(),baz();`.
- It works when leading non-expression statements remain before the simplified return.
- It leaves blocks unchanged when the return statement is not the final statement.
- It leaves directive strings such as `'use strict';` as directives instead of folding them into a sequence expression.
- `simplify: false` leaves the existing Rust output unchanged.

This slice does not port `IfStatementSimplifyTransformer`, and it does not attempt to remove function body braces.

## File Structure

- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the block-statement simplify transform.
- Create `crates/javascript-obfuscator/src/transforms/block_statement_simplify.rs`: SWC visitor for return-oriented block simplification.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for `simplify` block statement behavior.

## Task 1: Add Block Statement Simplify Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/block_statement_simplify.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod block_statement_simplify;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/block_statement_simplify.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_block_statement_simplify(_program: &mut Program, _enabled: bool) {}
```

Add tests for:

- `function foo(){bar();baz();return bark();}` with enabled `true` should contain `function foo(){return bar(),baz(),bark();}`.
- `function foo(){bar();baz();return bark();}` with enabled `false` should contain `function foo(){bar();baz();return bark();}`.
- `function foo(){const value=1;bar();return baz();}` with enabled `true` should contain `function foo(){const value=1;return bar(),baz();}`.
- `function foo(){return bar();baz();}` with enabled `true` should remain `function foo(){return bar();baz();}`.
- `function foo(){'use strict';bar();return baz();}` with enabled `true` should contain `function foo(){'use strict';return bar(),baz();}`.

Run:

```bash
cargo test -p javascript-obfuscator block_statement_simplify
```

Expected: FAIL for enabled transform cases until implementation is added.

- [ ] **Step 3: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_simplifies_trailing_return_block_when_simplify_enabled`
- `obfuscate_keeps_trailing_return_block_when_simplify_disabled`

Expected: enabled test fails until the runner is wired.

## Task 2: Implement Block Statement Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/block_statement_simplify.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- returns early when `enabled` is false.
- visits children first.
- inspects each `BlockStmt.stmts` list.
- only rewrites blocks whose final statement is `Stmt::Return` with an argument.
- scans backward over preceding expression statements.
- stops before string-literal directive statements.
- flattens `Expr::Seq` expression statements into the new return sequence.
- leaves blocks unchanged when no preceding expression statement is collected.

Add helper functions:

```rust
fn simplify_block_statement(block_statement: &mut BlockStmt)
fn final_return_argument(block_statement: &BlockStmt) -> Option<Expr>
fn collect_expression_statement(expression_statement: &ExprStmt, expressions: &mut Vec<Expr>) -> bool
fn create_return_statement(argument: Box<Expr>) -> Stmt
fn create_sequence_expression(expressions: Vec<Expr>) -> Box<Expr>
```

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator block_statement_simplify
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call block-statement simplify with the existing simplify option**

Call it after expression-statement and variable-declaration merging, before escape-sequence/directive placement:

```rust
    block_statement_simplify::transform_block_statement_simplify(
        program,
        options.simplify.unwrap_or(false),
    );
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator trailing_return_block
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
git add docs/superpowers/plans/2026-06-01-rust-block-statement-simplify-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust block statement simplify transform"
git push origin codex/rust-rewrite-slice-1
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan ports one more TypeScript simplifying transformer behavior to Rust.
- It reuses the existing Rust `simplify` option.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_block_statement_simplify` mutates `Program`, matching the existing transform runner.
- The transform accepts `enabled: bool`, matching `Options.simplify`.
- SWC block statements use `BlockStmt.stmts`, expression statements use `Stmt::Expr(ExprStmt)`, return statements use `Stmt::Return(ReturnStmt)`, and sequence expressions use `Expr::Seq(SeqExpr)`.
