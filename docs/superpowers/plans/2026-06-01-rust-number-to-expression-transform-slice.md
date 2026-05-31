# Rust Number To Expression Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `NumberToNumericalExpressionTransformer` for the `numbersToExpressions` option.

**Architecture:** Add a `number_to_expressions` SWC mutable visitor under the existing Rust transform runner. The visitor replaces safe numeric expression literals with deterministic arithmetic expressions that evaluate to the original value while skipping unsafe integers and non-expression property keys.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports basic number-to-expression conversion:

- `numbersToExpressions: true` transforms safe integer literals such as `10` to arithmetic expressions such as `0xb-0x1`.
- Float literals are transformed to arithmetic expressions such as `50+0.5`.
- Unsafe integers outside JavaScript safe integer bounds are not transformed and continue through the existing hexadecimal raw number literal pass.
- Non-computed object property numeric keys are not transformed because they are SWC property names, not expression literals.
- `numbersToExpressions: false` leaves the existing Rust number literal behavior unchanged.

This slice does not port randomized expression analysis, the full TypeScript `NumberNumericalExpressionAnalyzer`, or every randomized expression shape. It adds deterministic parity for preserving runtime value while increasing AST obfuscation coverage.

## File Structure

- Modify `crates/javascript-obfuscator/src/options.rs`: add `numbers_to_expressions: Option<bool>`.
- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the number-to-expressions transform.
- Create `crates/javascript-obfuscator/src/transforms/number_to_expressions.rs`: SWC visitor and arithmetic expression helpers.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for `numbersToExpressions`.

## Task 1: Add Number To Expression Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/number_to_expressions.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add the Rust option field**

In `crates/javascript-obfuscator/src/options.rs`, add:

```rust
    #[serde(default)]
    pub numbers_to_expressions: Option<bool>,
```

This uses the existing `#[serde(rename_all = "camelCase")]`, so JSON `numbersToExpressions` maps to Rust `numbers_to_expressions`.

- [ ] **Step 2: Export transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod number_to_expressions;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 3: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/number_to_expressions.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_number_to_expressions(_program: &mut Program, _enabled: bool) {}
```

Add tests for:

- `const value = 10;` with enabled `true` should contain `const value=0xb-0x1`.
- `const value = 10;` with enabled `false` should contain `const value=0xa` after the existing number literal pass.
- `const value = 50.5;` with enabled `true` should contain `const value=0x32+0.5`.
- `const value = {1: 'bar'};` with enabled `true` should contain `const value={0x1:'bar'}` and should not transform the key to a binary expression.
- `const value = 9007199254740992;` with enabled `true` should contain `const value=0x20000000000000`.

Run:

```bash
cargo test -p javascript-obfuscator number_to_expressions
```

Expected: FAIL for enabled transform cases until implementation is added.

- [ ] **Step 4: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_transforms_number_to_expression_when_enabled`
- `obfuscate_keeps_number_literal_when_numbers_to_expressions_disabled`

Expected: enabled test fails until the runner is wired.

## Task 2: Implement Number To Expression Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/number_to_expressions.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- returns early when `enabled` is false.
- visits expression children first.
- transforms only `Expr::Lit(Lit::Num(_))`.
- skips non-finite values.
- skips integer values where `abs(value) > 9007199254740991.0`.
- for integer values, replaces `n` with `(n + 1) - 1`.
- for fractional values, replaces `n` with `floor(n) + (n - floor(n))`.
- creates integer number literals with hexadecimal `raw` values.
- creates fractional number literals with decimal `raw` values.

Add helper functions:

```rust
fn transform_number_expression(number: &Number) -> Option<Expr>
fn create_integer_expression(value: f64) -> Expr
fn create_fractional_expression(value: f64) -> Expr
fn create_number_literal(value: f64) -> Expr
fn create_sub_expression(left: Expr, right: Expr) -> Expr
fn create_add_expression(left: Expr, right: Expr) -> Expr
```

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator number_to_expressions
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call number-to-expressions after number literal raw conversion**

Call it immediately after `number_literals::transform_number_literals(program);`:

```rust
    number_to_expressions::transform_number_to_expressions(
        program,
        options.numbers_to_expressions.unwrap_or(false),
    );
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator numbers_to_expressions
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
git add docs/superpowers/plans/2026-06-01-rust-number-to-expression-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust number to expression transform"
git push origin codex/rust-rewrite-slice-1
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan ports another TypeScript converting transformer to Rust.
- It adds the missing Rust option required to drive the transform.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_number_to_expressions` mutates `Program`, matching the existing transform runner.
- The transform accepts `enabled: bool`, matching the new Rust `Options` field.
- SWC numeric expression literals are represented by `Expr::Lit(Lit::Num(_))`; object property keys are not expression literals and remain handled by the existing number-literal raw pass.
