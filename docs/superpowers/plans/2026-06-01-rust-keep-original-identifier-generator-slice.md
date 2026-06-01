# Rust Keep Original Identifier Generator Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust option compatibility for `identifierNamesGenerator: "keep-original"` and preserve user label names when that generator is selected.

**Architecture:** The TypeScript engine keeps user identifiers unchanged in `keep-original` mode while still allowing internal helper names to be generated. The Rust engine currently uses the identifier generator only for labeled statements, so this slice adds enum support and skips label renaming for user labels under `KeepOriginal`; internal `generate_next` can keep using deterministic hexadecimal names for future helper use. Do not add VMP behavior.

**Tech Stack:** Rust 1.96, serde kebab-case enums, SWC AST labeled statement transform, Rust API tests.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/generators/identifier_names.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/labeled_statements.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add option deserialization test**

Add a test that deserializes:

```rust
let options: Options = serde_json::from_value(json!({
    "identifierNamesGenerator": "keep-original"
}))
.expect("keep-original identifier generator option should deserialize");

assert_eq!(
    options.identifier_names_generator,
    Some(IdentifierNamesGeneratorKind::KeepOriginal)
);
```

- [x] **Step 2: Add generator test**

Add a test proving the `KeepOriginal` generator kind still emits deterministic internal names:

```rust
let mut generator =
    IdentifierNamesGenerator::new(IdentifierNamesGeneratorKind::KeepOriginal, "", Vec::new());

assert_eq!(generator.generate_next(), "_0x0");
assert_eq!(generator.generate_next(), "_0x1");
```

- [x] **Step 3: Add labeled statement transform test**

Add a test that transforms:

```javascript
label: for (;;) { continue label; break label; }
```

with `IdentifierNamesGeneratorKind::KeepOriginal`, and asserts:

```rust
assert!(
    code.contains("label:for(;;){continue label;break label;}"),
    "{code}"
);
```

- [x] **Step 4: Add API test**

Add an API-level test with options deserialized from JSON:

```json
{
  "compact": true,
  "identifierNamesGenerator": "keep-original",
  "renameGlobals": false,
  "stringArray": false
}
```

Assert the obfuscated output keeps the original label and matching `break`.

- [x] **Step 5: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator keep_original
```

Expected: compile/test failure because `KeepOriginal` is not yet a Rust enum variant.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/generators/identifier_names.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/labeled_statements.rs`

- [x] **Step 1: Add enum variant**

Add `KeepOriginal` to `IdentifierNamesGeneratorKind`. The existing `#[serde(rename_all = "kebab-case")]` should make it deserialize from `keep-original`.

- [x] **Step 2: Keep internal generator behavior deterministic**

Handle `IdentifierNamesGeneratorKind::KeepOriginal` in `generate_next` by producing hexadecimal-style internal names:

```rust
IdentifierNamesGeneratorKind::Hexadecimal | IdentifierNamesGeneratorKind::KeepOriginal => {
    format!("_0x{:x}", self.index)
}
```

- [x] **Step 3: Preserve user labels**

In `transform_labeled_statements`, return early when `identifier_names_generator == IdentifierNamesGeneratorKind::KeepOriginal`.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator keep_original
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
git add -- crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/generators/identifier_names.rs crates/javascript-obfuscator/src/transforms/labeled_statements.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-keep-original-identifier-generator-slice.md
```

Commit and push:

```bash
git commit -m "feat: add rust keep original identifier generator option"
git push origin codex/rust-rewrite-slice-1
```
