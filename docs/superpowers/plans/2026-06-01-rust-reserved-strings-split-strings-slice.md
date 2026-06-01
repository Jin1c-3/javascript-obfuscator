# Rust Reserved Strings Split Strings Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust `splitStrings` respect `reservedStrings`.

**Architecture:** Thread the existing `Options.reserved_strings` list into the split-string transform. Match current Rust string-array behavior by treating a reserved entry as a substring match (`value.contains(reserved_string)`), while preserving directive string statement behavior. Do not add VMP behavior.

**Tech Stack:** Rust 1.96, SWC AST split-string transform.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/split_strings.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add transform test**

Add a split-string transform test with `reserved_strings == ["keep"]` for:

```javascript
const keep = 'please-keep-me'; const split = 'abcdef';
```

Assert the keep literal remains inline and the second literal is split.

- [x] **Step 2: Add API test**

Add an API test using:

```json
{
  "compact": true,
  "propertyBracketing": false,
  "renameGlobals": false,
  "reservedStrings": ["keep"],
  "splitStrings": true,
  "splitStringsChunkLength": 3,
  "stringArray": false
}
```

Assert `please-keep-me` remains inline and `abcdef` splits to `'abc'+'def'`.

- [x] **Step 3: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator reserved_split_string
```

Expected: compile/test failure because `split_strings` does not accept or apply reserved strings yet.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/split_strings.rs`

- [x] **Step 1: Thread reserved strings**

Add `reserved_strings: &[String]` to `transform_split_strings` and pass `options.reserved_strings.as_deref().unwrap_or(&[])` from `apply_transforms`.

- [x] **Step 2: Store reserved strings in transform**

Add a `reserved_strings` field to `SplitStringTransform`.

- [x] **Step 3: Skip reserved literals**

Before splitting a string literal, return without changing it when any reserved string is contained in the literal value.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator reserved_split_string
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
git add -- crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/split_strings.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-reserved-strings-split-strings-slice.md
```

Commit and push:

```bash
git commit -m "feat: respect reserved strings in rust split strings"
git push origin codex/rust-rewrite-slice-1
```
