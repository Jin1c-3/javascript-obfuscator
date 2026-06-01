# Rust Rename Properties Cache Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust parity for `identifierNamesCache.propertyIdentifiers` when `renameProperties` is enabled.

**Architecture:** Build one mutable `IdentifierNamesCacheStorage` at the pipeline boundary when `identifierNamesCache` is provided. Pass it into the transform runner so `rename_properties` can resolve existing property mappings before generating new names and write new mappings back for the returned obfuscation result.

**Tech Stack:** Rust, serde_json maps, existing `IdentifierNamesCacheStorage`, SWC rename-properties visitor, Rust API tests.

---

### Task 1: Failing Cache Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator/src/storages/identifier_names_cache.rs`

- [ ] **Step 1: Add API coverage for cached property names**

```rust
#[test]
fn obfuscate_uses_property_identifier_names_cache_for_renamed_properties() {
    let options: Options = serde_json::from_value(json!({
        "compact": true,
        "identifierNamesCache": {
            "globalIdentifiers": {},
            "propertyIdentifiers": {
                "foo": "foo_from_cache"
            }
        },
        "propertyBracketing": false,
        "renameGlobals": false,
        "renameProperties": true,
        "renamePropertiesMode": "unsafe",
        "stringArray": false
    }))
    .expect("identifier names cache options should deserialize");
    let result = obfuscate("const object = {foo: 1, bar: 2}; object.foo; object.bar;", options)
        .expect("obfuscation should succeed");

    assert!(result.code.contains("'foo_from_cache':0x1"), "{}", result.code);
    assert!(result.code.contains("'_0x0':0x2"), "{}", result.code);
    assert!(result.code.contains("object.foo_from_cache"), "{}", result.code);
    assert!(result.code.contains("object._0x0"), "{}", result.code);

    let cache = result.identifier_names_cache.expect("cache should be returned");
    assert_eq!(cache["propertyIdentifiers"]["foo"], "foo_from_cache");
    assert_eq!(cache["propertyIdentifiers"]["bar"], "_0x0");
}
```

- [ ] **Step 2: Add reserved-name cache behavior**

```rust
#[test]
fn obfuscate_keeps_reserved_property_even_when_cache_contains_mapping() {
    let options: Options = serde_json::from_value(json!({
        "compact": true,
        "identifierNamesCache": {
            "globalIdentifiers": {},
            "propertyIdentifiers": {
                "keep": "keep_from_cache"
            }
        },
        "propertyBracketing": false,
        "renameGlobals": false,
        "renameProperties": true,
        "renamePropertiesMode": "unsafe",
        "reservedNames": ["^keep$"],
        "stringArray": false
    }))
    .expect("identifier names cache options should deserialize");
    let result = obfuscate("const object = {keep: 1}; object.keep;", options)
        .expect("obfuscation should succeed");

    assert!(result.code.contains("'keep':0x1"), "{}", result.code);
    assert!(result.code.contains("object.keep"), "{}", result.code);

    let cache = result.identifier_names_cache.expect("cache should be returned");
    assert_eq!(cache["propertyIdentifiers"]["keep"], "keep_from_cache");
}
```

- [ ] **Step 3: Add storage unit coverage**

Add `resolve_or_insert_property` tests mirroring the existing global cache test.

- [ ] **Step 4: Verify red**

Run:
```bash
cargo test -p javascript-obfuscator property_identifier
cargo test -p javascript-obfuscator obfuscate_uses_property_identifier_names_cache_for_renamed_properties
```

Expected: FAIL because the Rust rename-properties visitor currently ignores `identifierNamesCache.propertyIdentifiers`.

### Task 2: Cache Plumbing

**Files:**
- Modify: `crates/javascript-obfuscator/src/pipeline.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/eval_call_expressions.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/rename_properties.rs`
- Modify: `crates/javascript-obfuscator/src/storages/identifier_names_cache.rs`

- [ ] **Step 1: Add property resolver to storage**

Add `resolve_or_insert_property` using the existing `property_identifiers` map.

- [ ] **Step 2: Thread storage through pipeline**

In `run_pipeline`, create `identifier_names_cache_storage` from `options.identifier_names_cache.clone()`, pass `as_mut()` into `apply_transforms`, then return `storage.into_cache()` when cache was enabled.

- [ ] **Step 3: Thread storage through transforms**

Accept `Option<&mut IdentifierNamesCacheStorage>` in `apply_transforms` and pass it to `rename_properties::transform_rename_properties`.

- [ ] **Step 4: Use cache in rename-properties**

When a property needs renaming and cache storage is present, call `resolve_or_insert_property`; otherwise use the existing local map plus generator path.

### Task 3: Verification and Shipping

**Files:**
- Modify only the files above and this plan unless the compiler requires another file.

- [ ] **Step 1: Run focused tests**

Run:
```bash
cargo test -p javascript-obfuscator property_identifier
cargo test -p javascript-obfuscator rename_properties
```

- [ ] **Step 2: Run full verification suite**

Run:
```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npx eslint "src/**/*.ts"
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
rg -n "VMP|vmp|virtual machine|virtual-machine|virtualMachine" crates src test package.json
```

The final `rg` command should exit with code 1 and no output.

- [ ] **Step 3: Commit and push**

Run:
```bash
git add docs/superpowers/plans/2026-06-01-rust-rename-properties-cache-slice.md crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/pipeline.rs crates/javascript-obfuscator/src/storages/identifier_names_cache.rs crates/javascript-obfuscator/src/transforms/eval_call_expressions.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/rename_properties.rs
git commit -m "feat: cache rust renamed property names"
git pull --rebase origin codex/rust-rewrite-slice-1
git push origin codex/rust-rewrite-slice-1
```
