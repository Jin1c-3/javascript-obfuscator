# Rust Labeled Statement Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `LabeledStatementTransformer` for label, `break`, and `continue` identifier renaming.

**Architecture:** Add a `labeled_statements` SWC mutable visitor under the existing Rust transform runner. The visitor creates deterministic replacement label names from the Rust identifier-name generator and rewrites matching `break`/`continue` label references inside each labeled statement body.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, existing Rust identifier name generator, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports basic labeled statement renaming:

- `label: for (...) { continue label; break label; }` becomes a generated label such as `_0x0:for(...) { continue _0x0; break _0x0; }`.
- The label declaration and matching `break`/`continue` references use the same generated name.
- The generator honors `identifierNamesGenerator`, `identifiersPrefix`, and `identifiersDictionary`.
- Unlabeled `break`/`continue` statements are unchanged.

This slice does not port full lexical scope analysis, `ScopeIdentifiersTransformer`, `IdentifierReplacer`, reserved-name collision checks, or cross-file identifier cache behavior for labels.

## File Structure

- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the labeled statement transform.
- Create `crates/javascript-obfuscator/src/transforms/labeled_statements.rs`: SWC visitor and label-reference replacement helper.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for generated label consistency.

## Task 1: Add Labeled Statement Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/labeled_statements.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod labeled_statements;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/labeled_statements.rs` with:

```rust
use swc_ecma_ast::Program;

use crate::generators::IdentifierNamesGeneratorKind;

pub fn transform_labeled_statements(
    _program: &mut Program,
    _identifier_names_generator: IdentifierNamesGeneratorKind,
    _identifiers_prefix: &str,
    _identifiers_dictionary: &[String],
) {
}
```

Add tests for:

- `label: for (;;) { continue label; break label; }` with default hexadecimal generator should contain `_0x0:for(;;){continue _0x0;break _0x0;}`.
- `label: for (;;) { break; }` should rename the label but keep `break;` unlabeled.
- `label: for (;;) { break label; }` with mangled generator should contain `a:for(;;){break a;}`.
- `label: for (;;) { break label; }` with dictionary generator and dictionary `["nice-label"]` should contain `nice_label:for(;;){break nice_label;}`.

Run:

```bash
cargo test -p javascript-obfuscator labeled_statements
```

Expected: FAIL for cases that require label renaming until implementation is added.

- [ ] **Step 3: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_renames_labeled_statement_and_matching_references`
- `obfuscate_renames_labeled_statement_with_mangled_generator`

Expected: these tests fail until the runner is wired.

## Task 2: Implement Labeled Statement Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/labeled_statements.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- owns an `IdentifierNamesGenerator`.
- visits labeled statement bodies after replacing the current label references.
- for each `LabeledStmt`, stores the original label symbol and generated name.
- rewrites `labeled_statement.label.sym` to the generated name.
- walks `labeled_statement.body` with a helper visitor that replaces `BreakStmt.label` and `ContinueStmt.label` when their symbol matches the original label.
- keeps the original span/context on rewritten labels by updating only `sym`.

Add helper:

```rust
struct LabelReferenceTransform {
    original_label_name: String,
    next_label_name: String,
}
```

The helper implements `VisitMut` for `BreakStmt` and `ContinueStmt`.

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator labeled_statements
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call labeled statement transform in `apply_transforms`**

Call it before final string escaping:

```rust
    labeled_statements::transform_labeled_statements(
        program,
        options.identifier_names_generator.unwrap_or_default(),
        options.identifiers_prefix.as_deref().unwrap_or(""),
        options.identifiers_dictionary.as_deref().unwrap_or(&[]),
    );
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator labeled_statement
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
git add docs/superpowers/plans/2026-06-01-rust-labeled-statement-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust labeled statement transform"
git push origin codex/rust-rewrite-slice-1
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan ports one remaining TypeScript rename-identifiers transformer to Rust.
- It uses the existing Rust identifier-name generator and option fields.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_labeled_statements` mutates `Program`, matching the existing transform runner.
- The transform accepts `IdentifierNamesGeneratorKind`, prefix, and dictionary inputs already present in Rust `Options`.
- SWC label declarations and references use `Ident`, so rewriting `sym` is sufficient for this slice.
