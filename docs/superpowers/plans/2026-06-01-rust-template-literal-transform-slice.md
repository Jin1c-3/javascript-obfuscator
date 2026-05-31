# Rust Template Literal Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `TemplateLiteralTransformer` for untagged template literals.

**Architecture:** Add a `template_literals` SWC mutable visitor under the existing Rust transform runner. The visitor converts untagged `Tpl` expressions into string literals and left-associative `+` binary expressions while leaving tagged templates untouched.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, existing serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports only untagged template literal conversion:

- `` `abc ${foo}` `` becomes `'abc '+foo`
- `` `${foo}` `` becomes `''+foo`
- `` `abc` `` becomes `'abc'`
- tagged templates such as `` tag`abc ${foo}` `` remain tagged templates

It does not port unicode escape finalization, string splitting, string array extraction, object-key side effects, or source map mapping parity. The TypeScript package facade remains on the existing TypeScript engine.

## File Structure

- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the template literal transform.
- Create `crates/javascript-obfuscator/src/transforms/template_literals.rs`: SWC visitor for untagged template literal conversion.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for template literal output.

## Task 1: Add Template Literal Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/template_literals.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export template transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod template_literals;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add template transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/template_literals.rs`:

```rust
use swc_ecma_ast::Program;

pub fn transform_template_literals(_program: &mut Program) {}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_template_literals(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_simple_template_literal_with_expression() {
        let code = transform("const value = `abc ${foo}`;");

        assert!(code.contains("const value='abc '+foo"), "{code}");
        assert!(!code.contains('`'), "{code}");
    }

    #[test]
    fn transforms_expression_only_template_literal() {
        let code = transform("const value = `${foo}`;");

        assert!(code.contains("const value=''+foo"), "{code}");
    }

    #[test]
    fn transforms_literal_only_template_literal() {
        let code = transform("const value = `abc`;");

        assert!(code.contains("const value='abc'"), "{code}");
    }

    #[test]
    fn keeps_tagged_template_literal() {
        let code = transform("tag`abc ${foo}`;");

        assert!(code.contains("tag`abc ${foo}`"), "{code}");
    }
}
```

- [ ] **Step 3: Add public API tests**

Append these tests inside the existing `#[cfg(test)] mod tests` in `crates/javascript-obfuscator/src/api.rs`:

```rust
#[test]
fn obfuscate_transforms_template_literal_with_expression() {
    let result = obfuscate(
        "const value = `abc ${foo}`;",
        Options {
            compact: Some(true),
            string_array: Some(false),
            rename_globals: Some(false),
            property_bracketing: Some(false),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("const value='abc '+foo"));
    assert!(!result.code.contains('`'));
}

#[test]
fn obfuscate_keeps_tagged_template_literal() {
    let result = obfuscate(
        "tag`abc ${foo}`;",
        Options {
            compact: Some(true),
            string_array: Some(false),
            rename_globals: Some(false),
            property_bracketing: Some(false),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("tag`abc ${foo}`"));
}
```

- [ ] **Step 4: Run targeted tests and confirm failure**

Run:

```bash
cargo test -p javascript-obfuscator template_literals
cargo test -p javascript-obfuscator obfuscate_transforms_template_literal_with_expression
```

Expected: FAIL because the transform stub does not mutate template literals and the runner does not call it.

## Task 2: Implement Template Literal Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/template_literals.rs`

- [ ] **Step 1: Replace template transform stub**

Replace `crates/javascript-obfuscator/src/transforms/template_literals.rs` with:

```rust
use swc_common::DUMMY_SP;
use swc_ecma_ast::{BinExpr, BinaryOp, Expr, Lit, Program, Str, TaggedTpl, Tpl};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_template_literals(program: &mut Program) {
    program.visit_mut_with(&mut TemplateLiteralTransform);
}

struct TemplateLiteralTransform;

impl VisitMut for TemplateLiteralTransform {
    fn visit_mut_tagged_tpl(&mut self, tagged_tpl: &mut TaggedTpl) {
        tagged_tpl.tag.visit_mut_with(self);
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);

        let Expr::Tpl(template_literal) = expr else {
            return;
        };

        if let Some(transformed_expression) = transform_template_literal(template_literal) {
            *expr = transformed_expression;
        }
    }
}

fn transform_template_literal(template_literal: &Tpl) -> Option<Expr> {
    let mut nodes = Vec::new();

    for (index, quasi) in template_literal.quasis.iter().enumerate() {
        let cooked = quasi.cooked.as_ref()?.to_string();
        nodes.push(create_string_literal(&cooked));

        if let Some(expression) = template_literal.exprs.get(index) {
            nodes.push((**expression).clone());
        }
    }

    nodes.retain(|node| !matches!(node, Expr::Lit(Lit::Str(value)) if value.value.is_empty()));

    if !is_string_literal(nodes.first()) && !is_string_literal(nodes.get(1)) {
        nodes.insert(0, create_string_literal(""));
    }

    let mut iterator = nodes.into_iter();
    let first = iterator.next()?;
    let Some(second) = iterator.next() else {
        return Some(first);
    };

    let mut root = create_add_expression(first, second);

    for node in iterator {
        root = create_add_expression(root, node);
    }

    Some(root)
}

fn create_add_expression(left: Expr, right: Expr) -> Expr {
    Expr::Bin(BinExpr {
        span: DUMMY_SP,
        op: BinaryOp::Add,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn create_string_literal(value: &str) -> Expr {
    Expr::Lit(Lit::Str(Str {
        span: DUMMY_SP,
        value: value.to_string().into(),
        raw: Some(single_quote_raw(value).into()),
    }))
}

fn single_quote_raw(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r");

    format!("'{escaped}'")
}

fn is_string_literal(node: Option<&Expr>) -> bool {
    matches!(node, Some(Expr::Lit(Lit::Str(_))))
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_template_literals(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_simple_template_literal_with_expression() {
        let code = transform("const value = `abc ${foo}`;");

        assert!(code.contains("const value='abc '+foo"), "{code}");
        assert!(!code.contains('`'), "{code}");
    }

    #[test]
    fn transforms_expression_only_template_literal() {
        let code = transform("const value = `${foo}`;");

        assert!(code.contains("const value=''+foo"), "{code}");
    }

    #[test]
    fn transforms_literal_only_template_literal() {
        let code = transform("const value = `abc`;");

        assert!(code.contains("const value='abc'"), "{code}");
    }

    #[test]
    fn keeps_tagged_template_literal() {
        let code = transform("tag`abc ${foo}`;");

        assert!(code.contains("tag`abc ${foo}`"), "{code}");
    }
}
```

- [ ] **Step 2: Run template transform tests**

Run:

```bash
cargo test -p javascript-obfuscator template_literals
```

Expected: PASS for transform unit tests while public API transform test still fails because the runner is not wired.

## Task 3: Wire Template Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call template transform before member bracketing**

Update `apply_transforms` in `crates/javascript-obfuscator/src/transforms/mod.rs`:

```rust
pub fn apply_transforms(program: &mut Program, options: &Options) {
    template_literals::transform_template_literals(program);
    boolean_literals::transform_boolean_literals(program);
    number_literals::transform_number_literals(program);
    member_expressions::transform_member_expressions(
        program,
        options.property_bracketing.unwrap_or(true),
    );
}
```

- [ ] **Step 2: Run public API template tests**

Run:

```bash
cargo test -p javascript-obfuscator obfuscate_transforms_template_literal_with_expression
cargo test -p javascript-obfuscator obfuscate_keeps_tagged_template_literal
```

Expected: PASS.

- [ ] **Step 3: Run full Rust engine tests**

Run:

```bash
cargo test -p javascript-obfuscator
```

Expected: PASS.

- [ ] **Step 4: Commit template literal transform slice**

Run:

```bash
git add crates/javascript-obfuscator
git commit -m "feat: add rust template literal transform"
```

## Task 4: Slice Verification and Push

**Files:**
- No extra source files unless verification changes formatting.

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

- This plan continues migration slice 7 by adding another converting transform.
- It keeps final unicode escaping and string-array extraction for later slices.
- It keeps VMP/Pro removal intact and includes the removal scan in verification.

Placeholder scan:

- The plan has concrete code, commands, and expected results.

Type consistency:

- `transform_template_literals` mutates `Program`, matching the existing transform runner shape.
- Tagged template handling is explicit through `visit_mut_tagged_tpl`.
- Public API tests use current Rust `Options` fields only.
