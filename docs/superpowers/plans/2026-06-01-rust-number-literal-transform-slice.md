# Rust Number Literal Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of `NumberLiteralTransformer` so integer numeric literals and bigint literals emit hexadecimal raw values.

**Architecture:** Reuse the Rust transform runner added in the boolean slice. Add a second SWC mutable visitor that updates `Lit::Num.raw` and `Lit::BigInt.raw`, then call it from the runner after boolean literal conversion.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports only raw literal formatting:

- integer `10` emits as `0xa`
- bigint `10n` emits as `0xan`
- non-integer numbers such as `10.5` keep decimal output

It does not port `numbersToExpressions`, numeric expression analysis, object key transforms, string splitting, or source map mapping parity. The TypeScript package facade remains on the existing TypeScript engine.

## File Structure

- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the number literal transform.
- Create `crates/javascript-obfuscator/src/transforms/number_literals.rs`: SWC visitor that sets number and bigint raw literals.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for number and bigint literal output.

## Task 1: Add Number Literal Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/number_literals.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export number transform with runner still no-op for it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod number_literals;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add number literal tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/number_literals.rs`:

```rust
use swc_ecma_ast::Program;

pub fn transform_number_literals(_program: &mut Program) {}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_number_literals(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_integer_number_literal_raw_value() {
        let code = transform("const value = 10;");

        assert!(code.contains("const value=0xa"));
    }

    #[test]
    fn keeps_float_number_literal_decimal() {
        let code = transform("const value = 10.5;");

        assert!(code.contains("const value=10.5"));
        assert!(!code.contains("0xa.5"));
    }

    #[test]
    fn transforms_bigint_literal_raw_value() {
        let code = transform("const value = 10n;");

        assert!(code.contains("const value=0xan"));
    }
}
```

- [ ] **Step 3: Add public API tests**

Append these tests inside the existing `#[cfg(test)] mod tests` in `crates/javascript-obfuscator/src/api.rs`:

```rust
#[test]
fn obfuscate_transforms_integer_number_literal_raw_value() {
    let result = obfuscate(
        "const value = 10;",
        Options {
            compact: Some(true),
            string_array: Some(false),
            rename_globals: Some(false),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("const value=0xa"));
}

#[test]
fn obfuscate_transforms_bigint_literal_raw_value() {
    let result = obfuscate(
        "const value = 10n;",
        Options {
            compact: Some(true),
            string_array: Some(false),
            rename_globals: Some(false),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("const value=0xan"));
}
```

- [ ] **Step 4: Run targeted tests and confirm failure**

Run:

```bash
cargo test -p javascript-obfuscator number_literals
cargo test -p javascript-obfuscator obfuscate_transforms_integer_number_literal_raw_value
```

Expected: FAIL because the transform stub does not update literal raw values and the runner does not call it.

## Task 2: Implement Number Literal Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/number_literals.rs`

- [ ] **Step 1: Replace number literal transform stub**

Replace `crates/javascript-obfuscator/src/transforms/number_literals.rs` with:

```rust
use swc_ecma_ast::{Lit, Program};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_number_literals(program: &mut Program) {
    program.visit_mut_with(&mut NumberLiteralTransform);
}

struct NumberLiteralTransform;

impl VisitMut for NumberLiteralTransform {
    fn visit_mut_lit(&mut self, literal: &mut Lit) {
        match literal {
            Lit::Num(number) if should_emit_hex_number(number.value) => {
                let integer = number.value.abs() as u128;
                number.raw = Some(format!("0x{integer:x}").into());
            }
            Lit::BigInt(bigint) => {
                bigint.raw = Some(format!("0x{}n", bigint.value.to_str_radix(16)).into());
            }
            _ => {}
        }
    }
}

fn should_emit_hex_number(value: f64) -> bool {
    value.is_finite() && value.fract() == 0.0
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_number_literals(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_integer_number_literal_raw_value() {
        let code = transform("const value = 10;");

        assert!(code.contains("const value=0xa"));
    }

    #[test]
    fn keeps_float_number_literal_decimal() {
        let code = transform("const value = 10.5;");

        assert!(code.contains("const value=10.5"));
        assert!(!code.contains("0xa.5"));
    }

    #[test]
    fn transforms_bigint_literal_raw_value() {
        let code = transform("const value = 10n;");

        assert!(code.contains("const value=0xan"));
    }
}
```

- [ ] **Step 2: Run number transform tests**

Run:

```bash
cargo test -p javascript-obfuscator number_literals
```

Expected: PASS for transform unit tests while public API tests still fail because the runner is not wired.

## Task 3: Wire Number Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call number transform after boolean transform**

Update `apply_transforms` in `crates/javascript-obfuscator/src/transforms/mod.rs`:

```rust
pub fn apply_transforms(program: &mut Program, _options: &Options) {
    boolean_literals::transform_boolean_literals(program);
    number_literals::transform_number_literals(program);
}
```

- [ ] **Step 2: Run public API number tests**

Run:

```bash
cargo test -p javascript-obfuscator obfuscate_transforms_integer_number_literal_raw_value
cargo test -p javascript-obfuscator obfuscate_transforms_bigint_literal_raw_value
```

Expected: PASS.

- [ ] **Step 3: Run full Rust engine tests**

Run:

```bash
cargo test -p javascript-obfuscator
```

Expected: PASS.

- [ ] **Step 4: Commit number literal transform slice**

Run:

```bash
git add crates/javascript-obfuscator
git commit -m "feat: add rust number literal transform"
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
- It keeps the heavier `numbersToExpressions` feature out of this slice because that requires analyzer and expression-converter parity.
- It keeps VMP/Pro removal intact and includes the removal scan in verification.

Placeholder scan:

- The plan has concrete code, commands, and expected results.

Type consistency:

- `transform_number_literals` mutates `Program`, matching the boolean transform and runner shape.
- Public API tests use existing `Options` fields only.
