# Rust Split String Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port an option-gated Rust equivalent of `SplitStringTransformer` for string literal chunking.

**Architecture:** Add a `split_strings` SWC mutable visitor under the existing Rust transform runner. The visitor rewrites eligible string literal expressions into left-associative `+` binary expressions when `splitStrings` is enabled.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, existing serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports only basic string literal splitting:

- with `splitStrings: true` and `splitStringsChunkLength: 3`, `'abcdef'` becomes `'abc' + 'def'`.
- with `splitStrings: false`, strings remain unchanged.
- top-level string expression statements such as `'use strict';` are preserved as string statements.
- generated binary expressions are left-associative.

This slice does not port grapheme-cluster exact splitting, two-pass 1000-character recursion avoidance, `forceTransformStrings`, string array extraction, reserved string guards, import/export source handling beyond SWC expression-string safety, or source-map parity.

## File Structure

- Modify `crates/javascript-obfuscator/src/options.rs`: add `split_strings` and `split_strings_chunk_length`.
- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the split string transform.
- Create `crates/javascript-obfuscator/src/transforms/split_strings.rs`: SWC visitor for option-gated splitting.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for split string output.

## Task 1: Add Split String Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/split_strings.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add options and module export with runner still not calling it**

Add these fields to `Options`:

```rust
pub split_strings: Option<bool>,
pub split_strings_chunk_length: Option<usize>,
```

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod split_strings;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 2: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/split_strings.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_split_strings(_program: &mut Program, _enabled: bool, _chunk_length: usize) {}
```

Add tests for:

- enabled splitting: `const value = 'abcdef';` with chunk length `3` should contain `const value='abc'+'def'`.
- disabled splitting: same source with `enabled: false` should contain `const value='abcdef'`.
- oversized chunk: chunk length `10` should keep `const value='abcdef'`.
- directive string preservation: `'use strict'; const value = 'abcdef';` should keep `'use strict';` and split the assignment string.

Run:

```bash
cargo test -p javascript-obfuscator split_strings
```

Expected: FAIL until implementation is added.

- [ ] **Step 3: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_splits_string_literals_when_enabled`
- `obfuscate_keeps_string_literals_when_split_strings_disabled`

Expected: split-enabled test fails until the runner is wired.

## Task 2: Implement Split String Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/split_strings.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- returns immediately when disabled or chunk length is `0`.
- skips direct string expression statements to preserve directives and standalone string statements.
- visits expression children first.
- replaces `Expr::Lit(Lit::Str)` with a left-associative `BinaryOp::Add` expression when the string length is greater than `chunk_length`.
- creates single-quoted raw string literals for each chunk.

Use scalar-value `char` chunks for this slice.

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator split_strings
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call split string transform in `apply_transforms`**

Call it after class/object/template conversion so generated string expressions can be split when the option is enabled:

```rust
split_strings::transform_split_strings(
    program,
    options.split_strings.unwrap_or(false),
    options.split_strings_chunk_length.unwrap_or(10),
);
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator obfuscate_splits_string_literals_when_enabled
cargo test -p javascript-obfuscator obfuscate_keeps_string_literals_when_split_strings_disabled
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
git add docs/superpowers/plans/2026-06-01-rust-split-string-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust split string transform"
git push
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan adds an option-gated converting transformer without changing default Rust output.
- It keeps string-array and guard parity for later slices.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_split_strings` mutates `Program`, matching the existing transform runner.
- `splitStrings` defaults to `false`; `splitStringsChunkLength` defaults to `10`.
- Generated expressions use `BinaryOp::Add`, matching the template literal conversion style.
