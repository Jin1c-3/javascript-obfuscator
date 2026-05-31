# Rust Escape Sequence Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust equivalent of the finalizing `EscapeSequenceTransformer` for string literal raw escape output.

**Architecture:** Add an `escape_sequences` SWC mutable visitor under the existing Rust transform runner. The visitor keeps string literal runtime values intact and rewrites `Str.raw` so generated JavaScript uses `\xNN`/`\uNNNN` escape sequences according to the `unicodeEscapeSequence` option.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visit, serde options, existing napi-rs boundary unchanged.

---

## Scope Boundary

This slice ports the escape-sequence finalizing behavior for Rust string literals:

- `unicodeEscapeSequence: true` encodes every character in a string literal, for example `'test'` becomes `'\x74\x65\x73\x74'`.
- `unicodeEscapeSequence: false` encodes only forced escape characters: control characters, quotes, backslashes, and whitespace.
- String literal runtime values stay unchanged; only `Str.raw` changes.
- Existing object-expression string keys and computed class string keys are covered because they are SWC `Str` literals.

This slice does not port string-array storage, reserved-string guards, force-transform-string guards, custom-code-helper escaping, or source-map parity.

## File Structure

- Modify `crates/javascript-obfuscator/src/options.rs`: add `unicode_escape_sequence: Option<bool>`.
- Modify `crates/javascript-obfuscator/src/transforms/mod.rs`: add and run the escape sequence transform at the end of `apply_transforms`.
- Create `crates/javascript-obfuscator/src/transforms/escape_sequences.rs`: SWC visitor and escape encoder.
- Modify `crates/javascript-obfuscator/src/api.rs`: add public Rust API tests for escaped string literal output.

## Task 1: Add Escape Sequence Contracts

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/escape_sequences.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add the Rust option field**

In `crates/javascript-obfuscator/src/options.rs`, add:

```rust
    #[serde(default)]
    pub unicode_escape_sequence: Option<bool>,
```

This uses the existing `#[serde(rename_all = "camelCase")]`, so JSON `unicodeEscapeSequence` maps to Rust `unicode_escape_sequence`.

- [ ] **Step 2: Export transform with runner still not calling it**

In `crates/javascript-obfuscator/src/transforms/mod.rs`, add:

```rust
pub mod escape_sequences;
```

Do not call it from `apply_transforms` in this step.

- [ ] **Step 3: Add transform tests with a no-op stub**

Create `crates/javascript-obfuscator/src/transforms/escape_sequences.rs` with:

```rust
use swc_ecma_ast::Program;

pub fn transform_escape_sequences(_program: &mut Program, _unicode_escape_sequence: bool) {}
```

Add tests for:

- `const value = 'test';` with `unicode_escape_sequence: true` should contain `const value='\x74\x65\x73\x74'`.
- `const value = 'hello world';` with `unicode_escape_sequence: false` should contain `const value='hello\x20world'`.
- `const value = 'тест';` with `unicode_escape_sequence: true` should contain `const value='\u0442\u0435\u0441\u0442'`.
- `const value = 'test';` with `unicode_escape_sequence: false` should contain `const value='test'`.

Run:

```bash
cargo test -p javascript-obfuscator escape_sequences
```

Expected: FAIL for the cases that require raw escape rewriting until implementation is added.

- [ ] **Step 4: Add public API tests**

Append tests in `crates/javascript-obfuscator/src/api.rs` for:

- `obfuscate_encodes_all_string_literal_characters_when_unicode_escape_sequence_enabled`
- `obfuscate_encodes_forced_string_literal_characters_when_unicode_escape_sequence_disabled`

Expected: these tests fail until the runner is wired.

## Task 2: Implement Escape Sequence Visitor

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/escape_sequences.rs`

- [ ] **Step 1: Replace the no-op stub**

Implement a `VisitMut` visitor that:

- visits children first.
- transforms every `Str` literal.
- computes `encoded_value = encode_escape_sequence(value, unicode_escape_sequence)`.
- sets `string_literal.raw = Some(format!("'{encoded_value}'").into())`.
- keeps `string_literal.value` unchanged.

Add helper functions:

```rust
fn encode_escape_sequence(value: &str, encode_all_symbols: bool) -> String
fn should_encode_character(character: char, encode_all_symbols: bool) -> bool
fn encode_character(character: char) -> String
```

Rules:

- `encode_all_symbols == true` encodes every character.
- Forced characters when `encode_all_symbols == false`: `'\u{0000}'..='\u{001f}'`, `'\u{007f}'..='\u{009f}'`, single quote, double quote, backslash, and `character.is_whitespace()`.
- ASCII encoded form is `\x` plus two lowercase hex digits.
- Non-ASCII encoded form is `\u` plus four lowercase hex digits.

- [ ] **Step 2: Run transform tests**

Run:

```bash
cargo test -p javascript-obfuscator escape_sequences
```

Expected: PASS.

## Task 3: Wire Transform Into Runner

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Call escape sequence transform at the end of `apply_transforms`**

Call it after `member_expressions::transform_member_expressions(...)`:

```rust
    escape_sequences::transform_escape_sequences(
        program,
        options.unicode_escape_sequence.unwrap_or(false),
    );
```

- [ ] **Step 2: Run public API tests**

Run:

```bash
cargo test -p javascript-obfuscator unicode_escape_sequence
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
git add docs/superpowers/plans/2026-06-01-rust-escape-sequence-transform-slice.md crates/javascript-obfuscator
git commit -m "feat: add rust escape sequence transform"
git push origin codex/rust-rewrite-slice-1
```

Expected: `origin/codex/rust-rewrite-slice-1` updates.

## Self-Review

Spec coverage:

- This plan ports another TypeScript finalizing transformer to Rust.
- It adds the missing Rust option required to drive the transformer.
- It preserves the already-removed VMP/Pro surface.

Placeholder scan:

- The plan has concrete files, commands, expected outputs, and no placeholder tokens.

Type consistency:

- `transform_escape_sequences` mutates `Program`, matching the existing transform runner.
- The transform accepts `unicode_escape_sequence: bool`, matching the new Rust `Options` field.
- SWC string literal raw output is controlled through `Str.raw`, while the runtime value remains in `Str.value`.
