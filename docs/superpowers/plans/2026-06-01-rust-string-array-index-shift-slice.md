# Rust String Array Index Shift Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust `stringArrayIndexShift` option support for the current minimal string-array transform.

**Architecture:** Extend the Rust `Options` model with `string_array_index_shift`. Keep direct array member expressions when the option is disabled. When enabled, emit a small string-array wrapper function that subtracts a deterministic shift amount from the call argument before reading the storage array, matching the TypeScript index-shift behavior while avoiding the larger wrapper/encoding system that is not ported yet.

**Tech Stack:** Rust 1.96, Serde option deserialization, SWC AST function declarations, SWC AST call/member/binary expressions.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add API test**

Add `obfuscate_uses_string_array_index_shift_when_enabled` with options:

```rust
Options {
    compact: Some(true),
    string_array: Some(true),
    string_array_threshold: Some(1.0),
    string_array_index_shift: Some(true),
    rename_globals: Some(false),
    property_bracketing: Some(false),
    unicode_escape_sequence: Some(false),
    ..Options::default()
}
```

Use source `const first = 'foo'; const second = 'bar';`. Assert the output contains:

```rust
"function _0x1(index){return _0x0[index-0x64];}"
"const first=_0x1(0x64);"
"const second=_0x1(0x65);"
```

- [x] **Step 2: Add transform unit test**

Add `uses_shifted_wrapper_when_index_shift_enabled` that calls:

```rust
transform_string_array(&mut parsed_program.program, true, 1.0, &[], true, &[], false);
```

Assert the generated compact code contains:

```rust
"const _0x0=['foo','bar'];"
"function _0x1(index){return _0x0[index-0x64];}"
"const first=_0x1(0x64);const second=_0x1(0x65);"
```

- [x] **Step 3: Run red test**

Run:

```bash
cargo test -p javascript-obfuscator string_array_index_shift
```

Expected: compile failure because Rust `Options` does not expose `string_array_index_shift`, or assertion failure because the option is ignored.

### Task 2: Option And Transform

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add option field**

Add this field after `string_array_indexes_type`:

```rust
#[serde(default)]
pub string_array_index_shift: Option<bool>,
```

- [x] **Step 2: Thread option into transform**

Change the transform call in `crates/javascript-obfuscator/src/transforms/mod.rs` to pass:

```rust
options.string_array_index_shift.unwrap_or(false),
```

after the index type slice.

- [x] **Step 3: Add deterministic shift mode**

In `crates/javascript-obfuscator/src/transforms/string_array.rs`, add:

```rust
const INDEX_SHIFT_AMOUNT: usize = 100;
const SHIFTED_WRAPPER_NAME: &str = "_0x1";
```

Store `index_shift_enabled: bool` on `StringArrayTransform`.

- [x] **Step 4: Emit shifted calls**

When `index_shift_enabled` is true, replace string literals with:

```javascript
_0x1(0x64 + index)
```

using the existing selected index literal type, so `hexadecimal-numeric-string` emits calls such as:

```javascript
_0x1('0x64')
```

When the option is false, keep the existing direct `_0x0[0x0]` behavior.

- [x] **Step 5: Insert wrapper after storage**

When values were extracted and index shift is enabled, insert this function immediately after the storage declaration:

```javascript
function _0x1(index){return _0x0[index-0x64];}
```

The wrapper must be inserted after imports in modules, preserving the existing import-safe storage insertion behavior.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator string_array_index_shift
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
git add -- crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/string_array.rs docs/superpowers/plans/2026-06-01-rust-string-array-index-shift-slice.md
```

Commit and push:

```bash
git commit -m "feat: add rust string array index shift option"
git push origin codex/rust-rewrite-slice-1
```
