# Rust String Array Rotate Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust `stringArrayRotate` option support for the current minimal string-array transform.

**Architecture:** Extend the Rust `Options` model with `string_array_rotate` and pass it into the string-array transform options. When enabled, deterministically right-rotate extracted storage values by one slot and remap generated string-array references to the new indexes. This preserves runtime behavior and moves the Rust engine toward the TypeScript option surface while leaving the larger randomized rotate-helper parity for a later slice.

**Tech Stack:** Rust 1.96, Serde option deserialization, SWC AST visitors, SWC AST literal/member/call mutation.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add API test for direct rotated indexes**

Add `obfuscate_uses_string_array_rotate_when_enabled` with source:

```javascript
const first = 'foo'; const second = 'bar'; const third = 'baz';
```

Use options:

```rust
Options {
    compact: Some(true),
    string_array: Some(true),
    string_array_threshold: Some(1.0),
    string_array_rotate: Some(true),
    rename_globals: Some(false),
    property_bracketing: Some(false),
    unicode_escape_sequence: Some(false),
    ..Options::default()
}
```

Assert the output contains:

```rust
"const _0x0=['baz','foo','bar'];"
"const first=_0x0[0x1];"
"const second=_0x0[0x2];"
"const third=_0x0[0x0];"
```

- [x] **Step 2: Add transform unit test for shifted rotated indexes**

Add `remaps_shifted_indexes_when_string_array_rotate_is_enabled` that calls:

```rust
transform_string_array(
    &mut parsed_program.program,
    StringArrayTransformOptions {
        enabled: true,
        threshold: 1.0,
        indexes_type: &[],
        index_shift: true,
        shuffle: false,
        rotate: true,
        reserved_strings: &[],
        ignore_imports: false,
    },
);
```

Assert the compact code contains:

```rust
"const _0x0=['baz','foo','bar'];"
"function _0x1(index){return _0x0[index-0x64];}"
"const first=_0x1(0x65);const second=_0x1(0x66);const third=_0x1(0x64);"
```

- [x] **Step 3: Run red test**

Run:

```bash
cargo test -p javascript-obfuscator string_array_rotate
```

Expected: compile failure because Rust `Options` does not expose `string_array_rotate` and `StringArrayTransformOptions` does not expose `rotate`, or assertion failure because the option is ignored.

### Task 2: Option And Transform

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add option field**

Add this field after `string_array_shuffle`:

```rust
#[serde(default)]
pub string_array_rotate: Option<bool>,
```

- [x] **Step 2: Thread option into transform**

Add this field to `StringArrayTransformOptions` in `crates/javascript-obfuscator/src/transforms/mod.rs`:

```rust
rotate: options.string_array_rotate.unwrap_or(false),
```

- [x] **Step 3: Add transform option field**

Add this field to `StringArrayTransformOptions`:

```rust
pub rotate: bool,
```

Update all local test helper construction sites with `rotate: false` unless the test explicitly enables rotation.

- [x] **Step 4: Rotate values and build remap**

After the shuffle remap block and before declarations are inserted, if rotate is enabled:

```rust
let index_remap = rotate_string_array_values(&mut values, 1);
remap_string_array_indexes(program, storage_name, index_type, options.index_shift, &index_remap);
```

Use a helper that right-rotates storage by `rotation_amount % values.len()` and returns a vector where `index_remap[old_index] == new_index`.

- [x] **Step 5: Compose with shuffle**

When both shuffle and rotate are enabled, remap after each storage operation in sequence. The second remap visitor must operate on the indexes already emitted by the first remap, so old indexes for rotate are the post-shuffle indexes.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator string_array_rotate
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
git add -- crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/string_array.rs docs/superpowers/plans/2026-06-01-rust-string-array-rotate-slice.md
```

Commit and push:

```bash
git commit -m "feat: add rust string array rotate option"
git push origin codex/rust-rewrite-slice-1
```
