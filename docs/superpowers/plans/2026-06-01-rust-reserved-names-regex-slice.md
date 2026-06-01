# Rust Reserved Names Regex Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make current Rust `reservedNames` consumers match regular expressions like the TypeScript identifier replacer.

**Architecture:** Compile `reservedNames` patterns inside the class-field and labeled-statement transforms and use regex matching instead of exact string equality. Keep constructor preservation unchanged, and do not broaden into full identifier/property rename support or VMP behavior.

**Tech Stack:** Rust 1.96, `regex` crate, SWC AST transforms.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/class_fields.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/labeled_statements.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add class-field regex reserved test**

Add a class-field transform test with `reserved_names == ["^bar$"]` and:

```javascript
class Foo { bar() {} baz() {} }
```

Assert `bar` remains an identifier method key and `baz` becomes a computed string key.

- [x] **Step 2: Add labeled-statement regex reserved test**

Add a labeled-statement transform test with `reserved_names == ["^keep"]` and:

```javascript
keepLabel: for (;;) { continue keepLabel; } other: for (;;) { break other; }
```

Assert `keepLabel` is preserved and `other` is renamed with matching break reference.

- [x] **Step 3: Add API regex reserved label test**

Add an API test with `reservedNames: ["^keep"]` and the same labeled source. Assert preserved and renamed label behavior through `obfuscate`.

- [x] **Step 4: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator reserved_names_regex
```

Expected: tests fail because Rust currently compares `reservedNames` exactly.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/class_fields.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/labeled_statements.rs`

- [x] **Step 1: Compile class-field reserved names as regex**

Compile `reserved_names` into `Regex` values in `transform_class_fields`, and use regex matching in `should_ignore_name`.

- [x] **Step 2: Compile labeled-statement reserved names as regex**

Compile `reserved_names` into `Regex` values in `transform_labeled_statements`, and use regex matching in `is_reserved_name`.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator reserved_names_regex
cargo test -p javascript-obfuscator class_fields
cargo test -p javascript-obfuscator labeled_statement
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
git add -- crates/javascript-obfuscator/src/transforms/class_fields.rs crates/javascript-obfuscator/src/transforms/labeled_statements.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-reserved-names-regex-slice.md
```

Commit and push:

```bash
git commit -m "feat: match rust reserved names as regex"
git push origin codex/rust-rewrite-slice-1
```
