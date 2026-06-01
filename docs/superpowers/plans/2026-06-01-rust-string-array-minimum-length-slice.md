# Rust String Array Minimum Length Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust string-array extraction skip normal string literals shorter than 3 characters while preserving `forceTransformStrings` overrides.

**Architecture:** Add the TypeScript string-array minimum length rule to the Rust string-array transform’s candidate gate. Only normal candidates should be skipped by the minimum length check; forced candidates should continue to transform even when short or when threshold is zero. Do not add VMP behavior.

**Tech Stack:** Rust 1.96, SWC AST transform tests, existing Rust API tests.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add transform minimum-length test**

Add a transform test with:

```javascript
const a = 'f'; const b = 'fo'; const c = 'foo';
```

Assert only `foo` is stored and the 1-2 character literals remain inline.

- [x] **Step 2: Add force override transform test**

Add a transform test with `forceTransformStrings: ["^f$"]`, `stringArrayThreshold: 0`, and:

```javascript
const value = 'f';
```

Assert `f` is stored and referenced through `_0x1(0x0)`.

- [x] **Step 3: Add API minimum-length test**

Add an API test with `stringArray: true`, `stringArrayThreshold: 1`, and:

```javascript
const a = 'f'; const b = 'fo'; const c = 'foo';
```

Assert only `foo` is stored and the short literals remain inline.

- [x] **Step 4: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator string_array_minimum_length
```

Expected: at least the normal minimum-length tests fail because Rust currently extracts 1-2 character strings.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add minimum length constant**

Add a `MINIMUM_LENGTH_FOR_STRING_ARRAY` constant with value `3`.

- [x] **Step 2: Apply minimum length to normal candidates**

In `visit_mut_expr`, after the `threshold <= 0` gate and before reserved-string matching, return when the string has fewer than 3 UTF-16 code units. Keep the check inside the `!is_force_transform_string` branch.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator string_array_minimum_length
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

- [x] **Step 3: Commit and push**

Stage:

```bash
git add -- crates/javascript-obfuscator/src/transforms/string_array.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-string-array-minimum-length-slice.md
```

Commit and push:

```bash
git commit -m "feat: enforce rust string array minimum length"
git push origin codex/rust-rewrite-slice-1
```
