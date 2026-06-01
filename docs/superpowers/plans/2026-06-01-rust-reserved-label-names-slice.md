# Rust Reserved Label Names Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust labeled-statement renaming honor `reservedNames` for user labels.

**Architecture:** Thread the existing `Options.reserved_names` list into the labeled-statement transform. Match the current Rust class-field behavior with exact string matching; TypeScript regex-compatible reserved-name matching can be factored later as a shared enhancement. Do not add VMP behavior.

**Tech Stack:** Rust 1.96, SWC AST labeled statement transform, existing `reservedNames` option.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/labeled_statements.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add transform test**

Add a labeled-statement transform test where `reserved_names == ["label"]` and assert:

```rust
assert!(
    code.contains("label:for(;;){continue label;break label;}"),
    "{code}"
);
```

- [x] **Step 2: Add API test**

Add an API test using:

```json
{
  "compact": true,
  "propertyBracketing": false,
  "renameGlobals": false,
  "reservedNames": ["label"],
  "stringArray": false
}
```

Assert the label and matching `break` remain `label`.

- [x] **Step 3: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator reserved_label
```

Expected: compile/test failure because the labeled statement transform does not accept or apply reserved names yet.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/labeled_statements.rs`

- [x] **Step 1: Thread reserved names**

Add a `reserved_names: &[String]` parameter to `transform_labeled_statements` and pass `options.reserved_names.as_deref().unwrap_or(&[])` from `apply_transforms`.

- [x] **Step 2: Store reserved names in transform**

Add a `reserved_names` field to `LabeledStatementTransform`.

- [x] **Step 3: Skip reserved labels**

Before generating a replacement name, return without changing the label when `reserved_names.iter().any(|reserved_name| reserved_name == original_label_name)`.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator reserved_label
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
git add -- crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/labeled_statements.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-reserved-label-names-slice.md
```

Commit and push:

```bash
git commit -m "feat: respect reserved label names in rust"
git push origin codex/rust-rewrite-slice-1
```
