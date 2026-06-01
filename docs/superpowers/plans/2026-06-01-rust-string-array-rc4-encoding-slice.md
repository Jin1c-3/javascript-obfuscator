# Rust String Array RC4 Encoding Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust `stringArrayEncoding: ['rc4']` support for the current minimal string-array transform.

**Architecture:** Build directly on the Base64 encoding slice. Store RC4-encrypted values as TypeScript-compatible swapped Base64 without padding, emit `_0x1(index, key)` references, and add a local wrapper decoder that first decodes swapped Base64 and then RC4-decrypts with the provided key. Keep this slice deterministic with one fixed 4-character key. Random key selection, collision retry parity, wrapper randomization, and self-defending behavior stay out of scope.

**Tech Stack:** Rust 1.96, Serde option deserialization, SWC AST visitors, SWC AST mutation/code generation, compact JavaScript helper generation.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Replace RC4 compatibility fallback test with RC4 encoding test**

Update the existing RC4 fallback test so `stringArrayEncoding: [StringArrayEncoding::Rc4]` now asserts:

```rust
"function _0x1(index,key)"
"'rc4K'"
"key.charCodeAt"
"const value=_0x1(0x0,'rc4K');"
```

Assert the storage no longer contains:

```rust
"['test']"
```

- [x] **Step 2: Add runtime RC4 decode test**

Add a runtime test that obfuscates:

```javascript
console.log(['foo', 'bar', 'baz'].join('|'));
```

Use options:

```rust
Options {
    compact: Some(true),
    string_array: Some(true),
    string_array_threshold: Some(1.0),
    string_array_encoding: Some(vec![crate::options::StringArrayEncoding::Rc4]),
    string_array_index_shift: Some(true),
    rename_globals: Some(false),
    property_bracketing: Some(false),
    unicode_escape_sequence: Some(false),
    ..Options::default()
}
```

Run the generated code with `node -e` and assert stdout is:

```text
foo|bar|baz
```

- [x] **Step 3: Add transform unit test for RC4 plus shuffle**

Add `remaps_rc4_wrapper_indexes_when_string_array_shuffle_is_enabled` using:

```rust
StringArrayTransformOptions {
    enabled: true,
    threshold: 1.0,
    indexes_type: &[],
    encoding: StringArrayEncoding::Rc4,
    index_shift: false,
    shuffle: true,
    rotate: false,
    reserved_strings: &[],
    ignore_imports: false,
}
```

Assert references are remapped to:

```rust
"const first=_0x1(0x1,'rc4K');const second=_0x1(0x0,'rc4K');"
```

- [x] **Step 4: Run red test**

Run:

```bash
cargo test -p javascript-obfuscator rc4
```

Expected: assertion failures because RC4 currently falls back to unencoded storage and direct member references.

### Task 2: RC4 Encoding And Wrapper

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Select RC4 as a supported encoding**

Update `select_supported_string_array_encoding` so `StringArrayEncoding::Rc4` is selected when configured.

- [x] **Step 2: Store per-entry decode keys**

Replace raw string storage values with a small internal entry containing:

```rust
encoded_value: String,
decode_key: Option<&'static str>,
```

Use deterministic key:

```rust
const DEFAULT_RC4_KEY: &str = "rc4K";
```

- [x] **Step 3: Implement RC4 encoding**

Add a local RC4 encrypt helper and encode its output with the same swapped Base64/no-padding helper used by Base64 storage.

- [x] **Step 4: Emit keyed references**

When the active encoding is RC4, emit:

```javascript
_0x1(index, 'rc4K')
```

Preserve index-shift behavior by shifting only the first argument.

- [x] **Step 5: Emit RC4 wrapper**

Generate a compact helper that reads `_0x0[index]`, decodes swapped Base64 locally, then RC4-decrypts with `key`. Keep the Base64-only wrapper behavior unchanged.

- [x] **Step 6: Preserve shuffle and rotate remapping**

The existing call-expression remapper should continue to remap the first wrapper argument while leaving the key argument unchanged.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator rc4
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
git add -- crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/string_array.rs docs/superpowers/plans/2026-06-01-rust-string-array-rc4-encoding-slice.md
```

Commit and push:

```bash
git commit -m "feat: add rust string array rc4 encoding option"
git push origin codex/rust-rewrite-slice-1
```
