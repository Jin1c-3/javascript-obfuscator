# Rust String Array Shuffle Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust `stringArrayShuffle` option support for the current minimal string-array transform.

**Architecture:** Extend the Rust `Options` model with `string_array_shuffle` and pass it into the string-array transform. When enabled, deterministically reverse the extracted storage values and remap generated string-array references to the new indexes. This preserves runtime behavior and establishes the option surface without depending on random-seed parity that has not been ported yet.

**Tech Stack:** Rust 1.96, Serde option deserialization, SWC AST visitors, SWC AST literal/member/call mutation.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add API test for direct shuffled indexes**

Add `obfuscate_uses_string_array_shuffle_when_enabled` with source:

```javascript
const first = 'foo'; const second = 'bar';
```

Use options:

```rust
Options {
    compact: Some(true),
    string_array: Some(true),
    string_array_threshold: Some(1.0),
    string_array_shuffle: Some(true),
    rename_globals: Some(false),
    property_bracketing: Some(false),
    unicode_escape_sequence: Some(false),
    ..Options::default()
}
```

Assert the output contains:

```rust
"const _0x0=['bar','foo'];"
"const first=_0x0[0x1];"
"const second=_0x0[0x0];"
```

- [x] **Step 2: Add transform unit test for shifted shuffled indexes**

Add `remaps_shifted_indexes_when_string_array_shuffle_is_enabled` that calls:

```rust
transform_string_array(
    &mut parsed_program.program,
    StringArrayTransformOptions {
        enabled: true,
        threshold: 1.0,
        indexes_type: &[],
        index_shift: true,
        shuffle: true,
        reserved_strings: &[],
        ignore_imports: false,
    },
);
```

Assert the compact code contains:

```rust
"const _0x0=['bar','foo'];"
"function _0x1(index){return _0x0[index-0x64];}"
"const first=_0x1(0x65);const second=_0x1(0x64);"
```

- [x] **Step 3: Run red test**

Run:

```bash
cargo test -p javascript-obfuscator string_array_shuffle
```

Expected: compile failure because Rust `Options` does not expose `string_array_shuffle` and `transform_string_array` does not accept the shuffle flag, or assertion failure because the option is ignored.

### Task 2: Option And Transform

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add option field**

Add this field after `string_array_index_shift`:

```rust
#[serde(default)]
pub string_array_shuffle: Option<bool>,
```

- [x] **Step 2: Thread option into transform**

Change the transform call in `crates/javascript-obfuscator/src/transforms/mod.rs` to pass:

```rust
options.string_array_shuffle.unwrap_or(false),
```

after `options.string_array_index_shift.unwrap_or(false)`.

- [x] **Step 3: Add transform options struct**

Change `transform_string_array` to accept a `StringArrayTransformOptions` struct containing:

```rust
pub struct StringArrayTransformOptions<'a> {
    pub enabled: bool,
    pub threshold: f64,
    pub indexes_type: &'a [StringArrayIndexesType],
    pub index_shift: bool,
    pub shuffle: bool,
    pub reserved_strings: &'a [String],
    pub ignore_imports: bool,
}
```

and update all local test helper calls.

- [x] **Step 4: Reverse values and build remap**

After the collection visitor runs and before declarations are inserted, if shuffle is enabled:

```rust
let index_remap = reverse_string_array_values(&mut values);
```

Use a helper that returns a vector where `index_remap[old_index] == new_index`.

- [x] **Step 5: Remap direct and shifted references**

Add a second SWC visitor that rewrites generated references:

```javascript
_0x0[oldIndex] -> _0x0[newIndex]
_0x1(oldIndex + 0x64) -> _0x1(newIndex + 0x64)
```

It must preserve the selected `StringArrayIndexesType`, including hexadecimal numeric string indexes.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator string_array_shuffle
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
git add -- crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/string_array.rs docs/superpowers/plans/2026-06-01-rust-string-array-shuffle-slice.md
```

Commit and push:

```bash
git commit -m "feat: add rust string array shuffle option"
git push origin codex/rust-rewrite-slice-1
```
