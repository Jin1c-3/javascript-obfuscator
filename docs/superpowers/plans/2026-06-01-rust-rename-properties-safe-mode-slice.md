# Rust Rename Properties Safe Mode Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust parity for `renamePropertiesMode: "safe"` and the TypeScript-compatible default safe behavior when `renameProperties` is enabled without an explicit mode.

**Architecture:** Before the rename-properties mutation pass, run a read-only SWC visitor that collects string literals that are not property keys. The mutating visitor will preserve collected names while still renaming object keys, member access, class keys, and destructuring keys that are not excluded, reserved, or DOM-reserved.

**Tech Stack:** Rust, SWC immutable and mutable visitors, existing `IdentifierNamesGenerator`, existing API and transform tests.

---

### Task 1: Failing Safe-Mode Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add explicit safe-mode coverage**

```rust
#[test]
fn obfuscate_excludes_string_literals_in_safe_property_mode() {
    let options: Options = serde_json::from_value(json!({
        "compact": true,
        "identifierNamesGenerator": "hexadecimal",
        "propertyBracketing": false,
        "renameGlobals": false,
        "renameProperties": true,
        "renamePropertiesMode": "safe",
        "stringArray": false
    }))
    .expect("rename properties options should deserialize");
    let result = obfuscate(
        "const object = {foo: 1, bar: 2}; const excluded = 'foo'; console.log(object.foo, object['bar']);",
        options,
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("'foo':0x1"), "{}", result.code);
    assert!(result.code.contains("'_0x0':0x2"), "{}", result.code);
    assert!(result.code.contains("object.foo"), "{}", result.code);
    assert!(result.code.contains("object['_0x0']"), "{}", result.code);
}
```

- [ ] **Step 2: Add default safe-mode coverage**

```rust
#[test]
fn obfuscate_defaults_rename_properties_mode_to_safe() {
    let options: Options = serde_json::from_value(json!({
        "compact": true,
        "identifierNamesGenerator": "hexadecimal",
        "propertyBracketing": false,
        "renameGlobals": false,
        "renameProperties": true,
        "stringArray": false
    }))
    .expect("rename properties options should deserialize");
    let result = obfuscate(
        "const object = {foo: 1, bar: 2}; const excluded = 'foo'; console.log(object.foo, object.bar);",
        options,
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("'foo':0x1"), "{}", result.code);
    assert!(result.code.contains("'_0x0':0x2"), "{}", result.code);
    assert!(result.code.contains("object.foo"), "{}", result.code);
    assert!(result.code.contains("object._0x0"), "{}", result.code);
}
```

- [ ] **Step 3: Verify red**

Run:
```bash
cargo test -p javascript-obfuscator obfuscate_excludes_string_literals_in_safe_property_mode
cargo test -p javascript-obfuscator obfuscate_defaults_rename_properties_mode_to_safe
```

Expected: FAIL because safe mode currently no-ops.

### Task 2: Safe-Mode Collector

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/rename_properties.rs`

- [ ] **Step 1: Add mode selection**

Treat `None` as `"safe"`, `"safe"` as safe, and `"unsafe"` as unsafe when `renameProperties` is true.

- [ ] **Step 2: Add auto-exclusion collector**

Collect `Lit::Str` values, but skip direct property keys:
- `PropName::Ident`, `PropName::Str`, and computed string `PropName` keys;
- `MemberProp::Ident` and computed string member props.

For computed non-string expressions, still visit the expression so nested string literals remain auto-excluded.

- [ ] **Step 3: Preserve excluded names during rename**

Add `excluded_property_names: HashSet<String>` to the mutating visitor and check it in `should_keep_name`.

### Task 3: Verification and Shipping

**Files:**
- Modify only `api.rs`, `rename_properties.rs`, and this plan unless the compiler requires another file.

- [ ] **Step 1: Run focused tests**

Run:
```bash
cargo test -p javascript-obfuscator obfuscate_excludes_string_literals_in_safe_property_mode
cargo test -p javascript-obfuscator obfuscate_defaults_rename_properties_mode_to_safe
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
git add docs/superpowers/plans/2026-06-01-rust-rename-properties-safe-mode-slice.md crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/transforms/rename_properties.rs
git commit -m "feat: support rust safe property renaming"
git pull --rebase origin codex/rust-rewrite-slice-1
git push origin codex/rust-rewrite-slice-1
```
