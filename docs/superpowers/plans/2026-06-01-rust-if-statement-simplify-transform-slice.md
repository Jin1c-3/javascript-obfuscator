# Rust If Statement Simplify Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of the core `IfStatementSimplifyTransformer` behavior for the existing `simplify` option.

**Architecture:** Add a SWC statement visitor that rewrites simplifiable `if` statements into logical expressions, conditional expressions, or slimmer `if` statements. The transform reuses the same statement-simplification rules as the TypeScript transformer and runs after expression, variable, and block simplification in the Rust pipeline.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports the core `IfStatementSimplifyTransformer` paths:

- `if (test) { exprs; }` becomes `test && exprs;`.
- `if (test) { exprs; } else { exprs; }` becomes `test ? consequentExprs : alternateExprs;`.
- `if (test) { return exprs; } else { return exprs; }` becomes `return test ? consequentExprs : alternateExprs;`.
- `if` branches with leading statements remain as `if` statements with partially simplified branch blocks.
- Single `function`, nested `if`, single-statement-body loops/labels, and `let`/`const` declaration branches keep braces to avoid invalid or ambiguous JavaScript.
- `simplify: false` leaves the existing Rust output unchanged.

This slice does not port the randomized control-flow flattening transformers.

## File Structure

- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the if-statement simplify transform.
- Create `crates/javascript-obfuscator/src/transforms/if_statement_simplify.rs`: SWC visitor and branch simplification helpers.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for `simplify` if-statement behavior.

## Task 1: Add If Statement Simplify Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/if_statement_simplify.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod if_statement_simplify;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/if_statement_simplify.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_if_statement_simplify(_program: &mut Program, _enabled: bool) {}
```

Add tests for:

- `if(true){bar();baz();}` with enabled `true` should contain `true&&(bar(),baz());`.
- `if(true){return bar();}else{return baz();}` with enabled `true` should contain `return true?bar():baz();`.
- `if(true){const value=1;bar();return baz();}` with enabled `true` should contain `if(true){const value=1;return bar(),baz();}`.
- `if(true){const value=1;}` with enabled `true` should keep braces as `if(true){const value=1;}`.
- `if(true){bar();}else{baz();}` with enabled `false` should remain `if(true){bar();}else{baz();}`.

Run:

```bash
cargo test -p javascript-obfuscator if_statement_simplify
```

Expected: FAIL for enabled transform cases until implementation is added.

- [ ] **Step 3: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_simplifies_if_expression_branch_when_simplify_enabled`
- `obfuscate_keeps_if_expression_branch_when_simplify_disabled`

Expected: enabled test fails until the runner is wired.

## Task 2: Implement If Statement Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/if_statement_simplify.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- returns early when `enabled` is false.
- visits child statements first.
- rewrites `Stmt::If` in `visit_mut_stmt`.
- computes `StatementSimplifyData` for each branch:
  - non-block statements become leading-only data.
  - block statements collect trailing expression statements and a final `return` argument.
  - blocks with statements after a return remain leading-only data.
  - string-literal directive statements are not collected as expressions.
- creates partial branch statements from leading statements plus a trailing expression or return statement.
- unwraps single-statement branch blocks only when the statement is not prohibited for bare `if` branches.
- creates `Expr::Bin(BinaryOp::LogicalAnd)` for consequent-only expression branches.
- creates `Expr::Cond(CondExpr)` for expression/return branches with both consequent and alternate.

Add helper functions:

```rust
fn transform_if_statement(if_statement: &IfStmt) -> Option<Stmt>
fn statement_simplify_data(statement: &Stmt) -> Option<StatementSimplifyData>
fn collect_block_simplify_data(block_statement: &BlockStmt) -> StatementSimplifyData
fn partial_statement(data: &StatementSimplifyData) -> Stmt
fn partial_if_branch_statement(data: &StatementSimplifyData) -> Stmt
fn is_prohibited_single_if_branch_statement(statement: &Stmt) -> bool
fn create_expression_statement(expression: Expr) -> Stmt
fn create_return_statement(expression: Expr) -> Stmt
fn create_logical_and(left: Expr, right: Expr) -> Expr
fn create_conditional(test: Expr, consequent: Expr, alternate: Expr) -> Expr
fn create_sequence_or_single(expressions: Vec<Expr>) -> Expr
fn parenthesize_sequence_expression(expression: Expr) -> Expr
```

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator if_statement_simplify
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call if-statement simplify with the existing simplify option**

Call it after block-statement simplification and before escape-sequence/directive placement:

```rust
    if_statement_simplify::transform_if_statement_simplify(
        program,
        options.simplify.unwrap_or(false),
    );
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator if_expression_branch
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
git add docs/superpowers/plans/2026-06-01-rust-if-statement-simplify-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust if statement simplify transform"
git push origin codex/rust-rewrite-slice-1
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan ports the remaining simplifying transformer’s core behavior to Rust.
- It reuses the existing Rust `simplify` option.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_if_statement_simplify` mutates `Program`, matching the existing transform runner.
- The transform accepts `enabled: bool`, matching `Options.simplify`.
- SWC if statements use `Stmt::If(IfStmt)`, branch statements use boxed `Stmt`, logical expressions use `BinaryOp::LogicalAnd`, and conditional expressions use `CondExpr`.
