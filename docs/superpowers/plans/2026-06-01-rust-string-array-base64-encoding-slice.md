# Rust String Array Base64 Encoding Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust `stringArrayEncoding: ['base64']` support for the current minimal string-array transform.

**Architecture:** Extend the Rust `Options` model with a `StringArrayEncoding` enum and pass the selected encoding into the string-array transform. For this slice, implement deterministic Base64 encoding only: store strings with the TypeScript-compatible swapped alphabet and omitted padding, emit a decoding wrapper when Base64 is selected, and keep RC4 out of scope. Existing direct string-array behavior must remain unchanged when encoding is `none`.

**Tech Stack:** Rust 1.96, Serde option deserialization, SWC AST visitors, SWC AST mutation/code generation, compact JavaScript helper generation.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add API test for Base64-encoded storage**

Add `obfuscate_uses_base64_string_array_encoding_when_enabled` with source:

```javascript
const value = 'test';
```

Use options:

```rust
Options {
    compact: Some(true),
    string_array: Some(true),
    string_array_threshold: Some(1.0),
    string_array_encoding: Some(vec![crate::options::StringArrayEncoding::Base64]),
    rename_globals: Some(false),
    property_bracketing: Some(false),
    unicode_escape_sequence: Some(false),
    ..Options::default()
}
```

Assert the output contains:

```rust
"const _0x0=['DgvZDa'];"
"function _0x1(index)"
"const chars='abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/=';"
"const value=_0x1(0x0);"
```

Assert it does not contain:

```rust
"['test']"
```

- [x] **Step 1a: Add compatibility and runtime tests**

Add tests that verify `stringArrayEncoding: ['rc4']` still deserializes and falls back to the current unencoded Rust behavior, `['rc4', 'base64']` uses the supported Base64 implementation, and Base64 output decodes correctly at runtime with index shifting, short strings, and UTF-8 input.

- [x] **Step 2: Add transform unit test for Base64 plus shuffle**

Add `remaps_base64_wrapper_indexes_when_string_array_shuffle_is_enabled` that calls:

```rust
transform_string_array(
    &mut parsed_program.program,
    StringArrayTransformOptions {
        enabled: true,
        threshold: 1.0,
        indexes_type: &[],
        encoding: StringArrayEncoding::Base64,
        index_shift: false,
        shuffle: true,
        rotate: false,
        reserved_strings: &[],
        ignore_imports: false,
    },
);
```

Use source:

```javascript
const first = 'foo'; const second = 'bar';
```

Assert the compact code contains:

```rust
"const _0x0=['yMfY','zM9V'];"
"function _0x1(index)"
"const first=_0x1(0x1);const second=_0x1(0x0);"
```

- [x] **Step 3: Run red test**

Run:

```bash
cargo test -p javascript-obfuscator string_array_base64
```

Expected: compile failure because Rust `Options` does not expose `string_array_encoding` and `StringArrayEncoding` yet, or assertion failure because the option is ignored.

### Task 2: Option And Transform

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add encoding option model**

Add:

```rust
#[serde(default)]
pub string_array_encoding: Option<Vec<StringArrayEncoding>>,
```

Add enum:

```rust
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StringArrayEncoding {
    None,
    Base64,
    Rc4,
}
```

- [x] **Step 2: Thread selected encoding into transform**

Pass the first supported configured encoding from `Options` to `StringArrayTransformOptions`, defaulting to `StringArrayEncoding::None`. Accept `Rc4` during deserialization for option compatibility, but filter it out until a later RC4 implementation slice.

- [x] **Step 3: Add transform option field**

Add:

```rust
pub encoding: StringArrayEncoding,
```

Update all local test helper construction sites with `encoding: StringArrayEncoding::None` unless the test explicitly enables Base64.

- [x] **Step 4: Encode stored values**

Keep deduplication keyed by the original string value, but push encoded values into storage when encoding is Base64. Implement swapped-alphabet, no-padding Base64 locally so `"test"` becomes `"DgvZDa"`.

- [x] **Step 5: Emit a wrapper for Base64**

When Base64 is enabled, emit `_0x1(index)` even if `stringArrayIndexShift` is false. The wrapper must read from `_0x0`, decode with a local swapped-alphabet Base64 decoder, and UTF-8 decode the result. When index shift is also enabled, subtract `0x64` before reading storage.

- [x] **Step 6: Remap wrapper indexes**

When shuffle or rotate is enabled, remap `_0x1(...)` call arguments whenever a wrapper is emitted, not only when index shift is enabled.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator string_array_base64
cargo test -p javascript-obfuscator string_array
```

- [x] **Step 2: Run full checks**

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npx eslint "src/**/*.ts"
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
rg -n "VMP|vmp|virtual machine|virtual-machine|virtualMachine" crates src test package.json
```

Expected: all commands exit 0 except the VMP scan, which exits 1 with no matches.

- [ ] **Step 3: Commit and push**

Stage:

```bash
git add -- crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/string_array.rs docs/superpowers/plans/2026-06-01-rust-string-array-base64-encoding-slice.md
```

Commit and push:

```bash
git commit -m "feat: add rust string array base64 encoding option"
git push origin codex/rust-rewrite-slice-1
```
