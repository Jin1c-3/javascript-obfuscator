# Rust Directive Placement Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `DirectivePlacementTransformer` needed to preserve directive prologues after final string escaping.

**Architecture:** Add a `directive_placement` SWC mutable visitor under the Rust transform runner. The visitor restores raw text for leading directive string statements in program and function-like scopes after `escape_sequences` rewrites general string literal raw output.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, existing serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice preserves directive prologues in the current Rust pipeline:

- Program-scope `'use strict';` remains exactly `'use strict';` after escape sequence finalization.
- Function-scope `'use strict';` remains exactly `'use strict';` after escape sequence finalization.
- Directive-like strings in the middle of a scope are not moved or restored.
- Non-directive string literals continue to be escaped by `escape_sequences`.

This slice does not port string-array movement, full preparing/finalizing two-pass directive storage, custom-code-helper injection, or directive recovery after future control-flow/dead-code transforms. It covers the current Rust pipeline state where the main directive risk is the final escape pass.

## File Structure

- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the directive placement transform after escape sequences.
- Create `crates/javascript-obfuscator/src/transforms/directive_placement.rs`: SWC visitor for leading directive raw restoration.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests proving directive preservation with escaped neighboring strings.

## Task 1: Add Directive Placement Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/directive_placement.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod directive_placement;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/directive_placement.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_directive_placement(_program: &mut Program) {}
```

Add tests that first run `escape_sequences::transform_escape_sequences(..., false)` and then `transform_directive_placement`:

- `"'use strict'; const value = 'hello world';"` should contain `"'use strict';const value='hello\x20world';"`.
- `"const value = 'hello world'; 'use strict';"` should contain `"const value='hello\x20world';'use\x20strict';"`.
- `"function run(){'use strict'; const value = 'hello world';}"` should contain `"function run(){'use strict';const value='hello\x20world';}"`.

Run:

```bash
cargo test -p javascript-obfuscator directive_placement
```

Expected: FAIL for the top-of-scope directive preservation cases until implementation is added.

- [ ] **Step 3: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_preserves_program_directive_after_escape_sequences`
- `obfuscate_preserves_function_directive_after_escape_sequences`

Expected: these tests fail until the runner is wired.

## Task 2: Implement Directive Placement Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/directive_placement.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- restores leading script/module statement directive raw strings.
- restores leading `Function` body directive raw strings.
- restores leading block-body directives for arrow functions with block bodies.
- visits nested function-like scopes after restoring the current scope.
- only handles leading expression statements where the expression is `Expr::Lit(Lit::Str(_))`.
- stops at the first non-string expression statement.
- rewrites the string raw to `Some(format!("'{escaped_value}'").into())` where only backslash and single quote are escaped.

Add helper functions:

```rust
fn restore_statement_directives(statements: &mut [Stmt])
fn restore_module_directives(module_items: &mut [ModuleItem])
fn restore_directive_statement(statement: &mut Stmt) -> bool
fn directive_raw(value: &str) -> String
```

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator directive_placement
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call directive placement after escape sequences**

Call it after `escape_sequences::transform_escape_sequences(...)`:

```rust
    directive_placement::transform_directive_placement(program);
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator directive_after_escape_sequences
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
git add docs/superpowers/plans/2026-06-01-rust-directive-placement-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust directive placement transform"
git push origin codex/rust-rewrite-slice-1
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan ports another TypeScript finalizing transformer behavior to Rust.
- It explicitly preserves directives after the new Rust escape-sequence transform.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_directive_placement` mutates `Program`, matching the existing transform runner.
- The helper functions use SWC `Stmt` and `ModuleItem`, matching script and module bodies.
- Directive raw restoration is intentionally limited to leading string expression statements.
