# Rust String Array Calls Transform Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first Rust parity slice for `stringArrayCallsTransform`, moving generated string-array call index literals into per-function control-flow storage objects.

**Architecture:** Keep the slice deterministic and local to the existing Rust string-array transform. After string-array references are generated and shuffle/rotate remaps are applied, walk function bodies only, replace eligible generated string-array call index literals with storage member reads, and prepend one local storage object per function body that had replacements. Root-scope string-array calls remain inline, and threshold `0` disables the call-index transform.

**Tech Stack:** Rust, SWC AST (`swc_ecma_ast`, `swc_ecma_visit`), existing Rust API tests, Node runtime smoke checks.

---

### Task 1: Add RED Coverage

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add option-surface test**

Add an `Options` JSON round-trip assertion for:

```json
{
  "stringArrayCallsTransform": true,
  "stringArrayCallsTransformThreshold": 0.25
}
```

Expected before implementation: the serialized `Options` value does not preserve these fields.

- [x] **Step 2: Add API behavior tests**

Add tests that call `obfuscate` with JSON-deserialized options:

```rust
{
    "compact": true,
    "propertyBracketing": false,
    "renameGlobals": false,
    "stringArray": true,
    "stringArrayCallsTransform": true,
    "stringArrayCallsTransformThreshold": 1,
    "stringArrayIndexShift": false,
    "stringArrayRotate": false,
    "stringArrayShuffle": false,
    "stringArrayThreshold": 1,
    "unicodeEscapeSequence": false
}
```

Cover:
- A function-body positive case inserts `const _0x2={_0x0:0x0,_0x1:0x1};` and replaces `_0x1(0x0)`/`_0x1(0x1)` with `_0x1(_0x2._0x0)`/`_0x1(_0x2._0x1)`.
- Root-scope calls are not moved into storage.
- Threshold `0` leaves function-body call indexes inline.
- The transformed positive case still runs under Node and prints the original result.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p javascript-obfuscator string_array_calls_transform -- --nocapture
```

Expected: tests compile and fail because Rust currently ignores the option and leaves call indexes inline.

### Task 2: Implement Option Plumbing

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add option fields**

Add to `Options`:

```rust
#[serde(default)]
pub string_array_calls_transform: Option<bool>,
#[serde(default)]
pub string_array_calls_transform_threshold: Option<f64>,
```

- [x] **Step 2: Pass fields into `StringArrayTransformOptions`**

Use:

```rust
calls_transform: options.string_array_calls_transform.unwrap_or(false),
calls_transform_threshold: options
    .string_array_calls_transform_threshold
    .unwrap_or(0.5),
```

### Task 3: Implement Function-Body Storage Transform

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Extend `StringArrayTransformOptions`**

Add:

```rust
pub calls_transform: bool,
pub calls_transform_threshold: f64,
```

- [x] **Step 2: Run after shuffle/rotate remapping**

In `transform_string_array`, after index remapping and before `insert_string_array_declarations`, call a new helper only when `calls_transform` is true and threshold is above zero.

- [x] **Step 3: Replace generated call index literals**

For each function body, replace the first argument of calls whose callee is an active string-array wrapper (`_0x1`, or used wrapper aliases/functions) and whose first argument is a numeric or string literal. Store each literal in a local object property (`_0x0`, `_0x1`, ...), and replace the argument with `<storage>.<key>`.

- [x] **Step 4: Insert local storage**

Prepend a `const` object statement such as:

```javascript
const _0x2={_0x0:0x0,_0x1:0x1};
```

Use `_0x2` when no root wrapper aliases are emitted, otherwise start after the used wrapper names to avoid collisions.

### Task 4: Verify and Ship

**Files:**
- All changed files from Tasks 1-3

- [x] **Step 1: Run focused Rust tests**

```bash
cargo test -p javascript-obfuscator string_array_calls_transform -- --nocapture
```

Expected: all focused tests pass.

- [x] **Step 2: Run full verification gate**

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npx eslint "src/**/*.ts"
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
rg -n "VMP|vmp|virtual machine|virtual-machine|virtualMachine" crates src test package.json
```

Expected: all commands exit 0 except the VMP scan exits 1 with no matches.

- [ ] **Step 3: Commit and push**

Stage only the Rust slice files and plan doc, commit with:

```bash
git commit -m "feat: add rust string array calls transform"
```

Then fetch/rebase as needed and push `codex/rust-rewrite-slice-1`.
