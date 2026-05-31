# Rust Member Expression Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `MemberExpressionTransformer` so dot-notation property access can be emitted as bracket notation.

**Architecture:** Add a `member_expressions` SWC mutable visitor under the existing Rust transform runner. The visitor converts `MemberProp::Ident` into `MemberProp::Computed` containing a string literal when `propertyBracketing` is enabled.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, existing serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports only member-expression property bracketing:

- `console.log` becomes `console['log']` when `propertyBracketing` is enabled or omitted.
- `console[identifier]` remains unchanged.
- `console['log']` remains unchanged.
- `propertyBracketing: false` disables the transform.

It does not port string-array handling for member-expression string literals, ignored-node metadata, property rename, optional chaining transforms, or computed property analysis. The TypeScript package facade remains on the existing TypeScript engine.

## File Structure

- Modify `crates/javascript-obfuscator/src/options.rs`: add `property_bracketing`.
- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the member expression transform.
- Create `crates/javascript-obfuscator/src/transforms/member_expressions.rs`: SWC visitor for dot-to-bracket member expression conversion.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests and make older parser/codegen tests explicitly opt out where they expect dot notation.

## Task 1: Add Member Expression Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/member_expressions.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add the option field**

In `crates/javascript-obfuscator/src/options.rs`, add this field to `Options`:

```rust
#[serde(default)]
pub property_bracketing: Option<bool>,
```

- [ ] **Step 2: Export member transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod member_expressions;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 3: Add member transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/member_expressions.rs`:

```rust
use swc_ecma_ast::Program;

pub fn transform_member_expressions(_program: &mut Program, _property_bracketing: bool) {}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, property_bracketing: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_member_expressions(&mut parsed_program.program, property_bracketing);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_dot_notation_to_bracket_notation() {
        let code = transform("const value = console.log;", true);

        assert!(code.contains("const value=console['log']"), "{code}");
    }

    #[test]
    fn keeps_computed_identifier_member_expression() {
        let code = transform("const value = console[identifier];", true);

        assert!(code.contains("const value=console[identifier]"), "{code}");
    }

    #[test]
    fn keeps_existing_computed_string_member_expression() {
        let code = transform("const value = console['log'];", true);

        assert!(code.contains("const value=console['log']"), "{code}");
    }

    #[test]
    fn skips_transform_when_property_bracketing_is_disabled() {
        let code = transform("const value = console.log;", false);

        assert!(code.contains("const value=console.log"), "{code}");
    }
}
```

- [ ] **Step 4: Add public API tests**

Append these tests inside the existing `#[cfg(test)] mod tests` in `crates/javascript-obfuscator/src/api.rs`:

```rust
#[test]
fn obfuscate_transforms_member_expression_dot_notation() {
    let result = obfuscate(
        "const value = console.log;",
        Options {
            compact: Some(true),
            string_array: Some(false),
            rename_globals: Some(false),
            property_bracketing: Some(true),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("const value=console['log']"));
}

#[test]
fn obfuscate_keeps_member_expression_dot_notation_when_disabled() {
    let result = obfuscate(
        "const value = console.log;",
        Options {
            compact: Some(true),
            string_array: Some(false),
            rename_globals: Some(false),
            property_bracketing: Some(false),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("const value=console.log"));
}
```

- [ ] **Step 5: Run targeted tests and confirm failure**

Run:

```bash
cargo test -p javascript-obfuscator member_expressions
cargo test -p javascript-obfuscator obfuscate_transforms_member_expression_dot_notation
```

Expected: FAIL because the transform stub does not mutate member expressions and the runner does not call it.

## Task 2: Implement Member Expression Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/member_expressions.rs`

- [ ] **Step 1: Replace member transform stub**

Replace `crates/javascript-obfuscator/src/transforms/member_expressions.rs` with:

```rust
use swc_common::DUMMY_SP;
use swc_ecma_ast::{ComputedPropName, Expr, Lit, MemberExpr, MemberProp, Program, Str};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_member_expressions(program: &mut Program, property_bracketing: bool) {
    if !property_bracketing {
        return;
    }

    program.visit_mut_with(&mut MemberExpressionTransform);
}

struct MemberExpressionTransform;

impl VisitMut for MemberExpressionTransform {
    fn visit_mut_member_expr(&mut self, member_expr: &mut MemberExpr) {
        member_expr.visit_mut_children_with(self);

        let MemberProp::Ident(identifier) = &member_expr.prop else {
            return;
        };

        let property_name = identifier.sym.to_string();
        member_expr.prop = MemberProp::Computed(ComputedPropName {
            span: DUMMY_SP,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: property_name.into(),
                raw: None,
            }))),
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, property_bracketing: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_member_expressions(&mut parsed_program.program, property_bracketing);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_dot_notation_to_bracket_notation() {
        let code = transform("const value = console.log;", true);

        assert!(code.contains("const value=console['log']"), "{code}");
    }

    #[test]
    fn keeps_computed_identifier_member_expression() {
        let code = transform("const value = console[identifier];", true);

        assert!(code.contains("const value=console[identifier]"), "{code}");
    }

    #[test]
    fn keeps_existing_computed_string_member_expression() {
        let code = transform("const value = console['log'];", true);

        assert!(code.contains("const value=console['log']"), "{code}");
    }

    #[test]
    fn skips_transform_when_property_bracketing_is_disabled() {
        let code = transform("const value = console.log;", false);

        assert!(code.contains("const value=console.log"), "{code}");
    }
}
```

- [ ] **Step 2: Run member transform tests**

Run:

```bash
cargo test -p javascript-obfuscator member_expressions
```

Expected: PASS for transform unit tests while public API transform test still fails because the runner is not wired.

## Task 3: Wire Member Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Call member transform from runner**

Update `apply_transforms` in `crates/javascript-obfuscator/src/transforms/mod.rs`:

```rust
pub fn apply_transforms(program: &mut Program, options: &Options) {
    boolean_literals::transform_boolean_literals(program);
    number_literals::transform_number_literals(program);
    member_expressions::transform_member_expressions(program, options.property_bracketing.unwrap_or(true));
}
```

- [ ] **Step 2: Make parser/codegen tests opt out of member bracketing**

In `crates/javascript-obfuscator/src/api.rs`, add `property_bracketing: Some(false),` to the `Options` literals in tests that assert `console.log(value)`:

```rust
property_bracketing: Some(false),
```

The test `obfuscate_generates_code_from_parsed_ast` is the one that currently checks dot notation.

- [ ] **Step 3: Run public API member tests**

Run:

```bash
cargo test -p javascript-obfuscator obfuscate_transforms_member_expression_dot_notation
cargo test -p javascript-obfuscator obfuscate_keeps_member_expression_dot_notation_when_disabled
```

Expected: PASS.

- [ ] **Step 4: Run full Rust engine tests**

Run:

```bash
cargo test -p javascript-obfuscator
```

Expected: PASS.

- [ ] **Step 5: Commit member transform slice**

Run:

```bash
git add crates/javascript-obfuscator
git commit -m "feat: add rust member expression transform"
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

- This plan continues migration slice 7 by adding another simple converting transform.
- It keeps string-array member literal processing out of scope because that belongs to the later string-array slice.
- It keeps VMP/Pro removal intact and includes the removal scan in verification.

Placeholder scan:

- The plan has concrete code, commands, and expected results.

Type consistency:

- `property_bracketing` uses serde camelCase mapping from `propertyBracketing`.
- `transform_member_expressions` mutates `Program`, matching the boolean and number transform runner shape.
- Public API tests use existing Rust `Options` fields plus the new `property_bracketing` option.
