# Rust Boolean Literal Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first Rust AST transform runner and port the BooleanLiteralTransformer behavior into the Rust no-transform pipeline.

**Architecture:** The Rust core gains a `transforms` module root with a small runner. The first converting transform uses SWC mutable visitors to replace boolean literals with unary-expression equivalents before code generation.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen, `swc_ecma_visit`, serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports only boolean literal conversion:

- `true` becomes `!![]`
- `false` becomes `![]`

It does not port number expression conversion, object key transforms, template literals, split strings, finalizing unicode escape logic, or any rename/string-array/control-flow transforms. The TypeScript package facade remains on the existing TypeScript engine.

## File Structure

- Modify `Cargo.toml`: add `swc_ecma_visit` workspace dependency.
- Modify `crates/javascript-obfuscator/Cargo.toml`: consume `swc_ecma_visit` from the workspace.
- Modify `crates/javascript-obfuscator/src/lib.rs`: export `transforms`.
- Create `crates/javascript-obfuscator/src/transforms/mod.rs`: transform runner and re-export module.
- Create `crates/javascript-obfuscator/src/transforms/boolean_literals.rs`: SWC visitor for boolean literal conversion and focused unit tests.
- Modify `crates/javascript-obfuscator/src/pipeline.rs`: run transforms between parse and code generation.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for boolean literal output.

## Task 1: Add Boolean Transform Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/lib.rs`
- Create: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/boolean_literals.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export transform module root**

In `crates/javascript-obfuscator/src/lib.rs`, add:

```rust
pub mod transforms;
```

- [ ] **Step 2: Add transform runner stub**

Create `crates/javascript-obfuscator/src/transforms/mod.rs`:

```rust
pub mod boolean_literals;

use swc_ecma_ast::Program;

use crate::options::Options;

pub fn apply_transforms(_program: &mut Program, _options: &Options) {}
```

- [ ] **Step 3: Add boolean transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/boolean_literals.rs`:

```rust
use swc_ecma_ast::Program;

pub fn transform_boolean_literals(_program: &mut Program) {}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_boolean_literals(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_true_boolean_literal() {
        let code = transform("const value = true;");

        assert!(code.contains("const value=!![]"));
        assert!(!code.contains("true"));
    }

    #[test]
    fn transforms_false_boolean_literal() {
        let code = transform("const value = false;");

        assert!(code.contains("const value=![]"));
        assert!(!code.contains("false"));
    }

    #[test]
    fn transforms_nested_boolean_literals() {
        let code = transform("if (true) { console.log(false); }");

        assert!(code.contains("if(!![])"));
        assert!(code.contains("console.log(![])"));
    }
}
```

- [ ] **Step 4: Add public API boolean output tests**

Append these tests inside the existing `#[cfg(test)] mod tests` in `crates/javascript-obfuscator/src/api.rs`:

```rust
#[test]
fn obfuscate_transforms_true_boolean_literals() {
    let result = obfuscate(
        "const value = true;",
        Options {
            compact: Some(true),
            string_array: Some(false),
            rename_globals: Some(false),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("const value=!![]"));
    assert!(!result.code.contains("true"));
}

#[test]
fn obfuscate_transforms_false_boolean_literals() {
    let result = obfuscate(
        "const value = false;",
        Options {
            compact: Some(true),
            string_array: Some(false),
            rename_globals: Some(false),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("const value=![]"));
    assert!(!result.code.contains("false"));
}
```

- [ ] **Step 5: Run targeted tests and confirm failure**

Run:

```bash
cargo test -p javascript-obfuscator boolean_literals
cargo test -p javascript-obfuscator obfuscate_transforms_true_boolean_literals
```

Expected: FAIL because the transform stub does not mutate the SWC AST.

## Task 2: Implement SWC Visit Dependency and Boolean Transform

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/javascript-obfuscator/Cargo.toml`
- Modify: `crates/javascript-obfuscator/src/transforms/boolean_literals.rs`

- [ ] **Step 1: Add SWC visit dependency**

In root `Cargo.toml`, add:

```toml
swc_ecma_visit = "25"
```

In `crates/javascript-obfuscator/Cargo.toml`, add:

```toml
swc_ecma_visit.workspace = true
```

- [ ] **Step 2: Replace boolean transform stub**

Replace `crates/javascript-obfuscator/src/transforms/boolean_literals.rs` with:

```rust
use swc_common::DUMMY_SP;
use swc_ecma_ast::{ArrayLit, Expr, Lit, Program, UnaryExpr, UnaryOp};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_boolean_literals(program: &mut Program) {
    program.visit_mut_with(&mut BooleanLiteralTransform);
}

struct BooleanLiteralTransform;

impl VisitMut for BooleanLiteralTransform {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);

        let Expr::Lit(Lit::Bool(boolean_literal)) = expr else {
            return;
        };

        *expr = create_boolean_expression(boolean_literal.value);
    }
}

fn create_boolean_expression(value: bool) -> Expr {
    if value {
        Expr::Unary(UnaryExpr {
            span: DUMMY_SP,
            op: UnaryOp::Bang,
            arg: Box::new(create_boolean_expression(false)),
        })
    } else {
        Expr::Unary(UnaryExpr {
            span: DUMMY_SP,
            op: UnaryOp::Bang,
            arg: Box::new(Expr::Array(ArrayLit {
                span: DUMMY_SP,
                elems: Vec::new(),
            })),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_boolean_literals(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_true_boolean_literal() {
        let code = transform("const value = true;");

        assert!(code.contains("const value=!![]"));
        assert!(!code.contains("true"));
    }

    #[test]
    fn transforms_false_boolean_literal() {
        let code = transform("const value = false;");

        assert!(code.contains("const value=![]"));
        assert!(!code.contains("false"));
    }

    #[test]
    fn transforms_nested_boolean_literals() {
        let code = transform("if (true) { console.log(false); }");

        assert!(code.contains("if(!![])"));
        assert!(code.contains("console.log(![])"));
    }
}
```

- [ ] **Step 3: Run boolean transform tests**

Run:

```bash
cargo test -p javascript-obfuscator boolean_literals
```

Expected: PASS.

## Task 3: Wire Transform Runner Into Pipeline

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/pipeline.rs`

- [ ] **Step 1: Call boolean transform from runner**

Replace `crates/javascript-obfuscator/src/transforms/mod.rs` with:

```rust
pub mod boolean_literals;

use swc_ecma_ast::Program;

use crate::options::Options;

pub fn apply_transforms(program: &mut Program, _options: &Options) {
    boolean_literals::transform_boolean_literals(program);
}
```

- [ ] **Step 2: Run transforms before code generation**

In `crates/javascript-obfuscator/src/pipeline.rs`, add:

```rust
use crate::transforms::apply_transforms;
```

Change parsed program handling to:

```rust
let mut parsed_program = parse_program(&prepared_code)?;
apply_transforms(&mut parsed_program.program, &options);
```

Keep code generation using `&parsed_program.program` and `parsed_program.source_map`.

- [ ] **Step 3: Run public API boolean tests**

Run:

```bash
cargo test -p javascript-obfuscator obfuscate_transforms_true_boolean_literals
cargo test -p javascript-obfuscator obfuscate_transforms_false_boolean_literals
```

Expected: PASS.

- [ ] **Step 4: Run full Rust engine tests**

Run:

```bash
cargo test -p javascript-obfuscator
```

Expected: PASS.

- [ ] **Step 5: Commit boolean transform slice**

Run:

```bash
git add Cargo.toml Cargo.lock crates/javascript-obfuscator
git commit -m "feat: add rust boolean literal transform"
```

## Task 4: Slice Verification and Push

**Files:**
- No extra source files unless verification updates `Cargo.lock`.

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

- [ ] **Step 4: Confirm branch status**

Run:

```bash
git status --short --branch
```

Expected: only pre-existing untracked `AGENTS.md` and `scripts/` remain.

- [ ] **Step 5: Push the branch**

Run:

```bash
git push
```

Expected: branch updates `origin/codex/rust-rewrite-slice-1`.

## Self-Review

Spec coverage:

- This plan starts migration slice 7 from the approved design with one simple converting transform.
- It creates the transform runner needed by subsequent converting and finalizing slices.
- It keeps package compatibility safe by leaving the TypeScript facade on the existing engine.

Placeholder scan:

- The plan has concrete code, commands, and expected results for each step.

Type consistency:

- `apply_transforms` accepts `&mut Program` and `&Options` in both runner and pipeline.
- `transform_boolean_literals` mutates `Program` directly and is called by the runner.
- The public API tests use existing `Options` fields only.
