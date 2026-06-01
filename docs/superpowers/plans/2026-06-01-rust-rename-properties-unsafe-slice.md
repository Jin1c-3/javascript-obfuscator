# Rust Rename Properties Unsafe Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first Rust bridge parity slice for `renameProperties: true` with `renamePropertiesMode: "unsafe"`.

**Architecture:** The Rust bridge will deserialize the TypeScript-compatible option names, then run a dedicated SWC visitor before property bracketing and after object literal normalization. The visitor keeps one property-name map so object literal keys and member accesses receive the same generated name, and it skips names matched by `reservedNames`.

**Tech Stack:** Rust, SWC AST visitors, `regex`, existing `IdentifierNamesGenerator`, existing API and transform unit tests.

---

### Task 1: Failing API Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add tests for unsafe property renaming**

```rust
#[test]
fn obfuscate_renames_properties_in_unsafe_mode() {
    let options: Options = serde_json::from_value(json!({
        "compact": true,
        "identifierNamesGenerator": "hexadecimal",
        "propertyBracketing": false,
        "renameGlobals": false,
        "renameProperties": true,
        "renamePropertiesMode": "unsafe",
        "stringArray": false
    }))
    .expect("rename properties options should deserialize");
    let result = obfuscate(
        "const object = {foo: 1}; console.log(object.foo, object['foo']);",
        options,
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("const object={'_0x0':0x1};"), "{}", result.code);
    assert!(result.code.contains("object._0x0"), "{}", result.code);
    assert!(result.code.contains("object['_0x0']"), "{}", result.code);

    let output = run_node_source(&result.code);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1 1\n");
}
```

- [ ] **Step 2: Add reserved-name coverage**

```rust
#[test]
fn obfuscate_keeps_reserved_properties_when_renaming_properties() {
    let options: Options = serde_json::from_value(json!({
        "compact": true,
        "propertyBracketing": false,
        "renameGlobals": false,
        "renameProperties": true,
        "renamePropertiesMode": "unsafe",
        "reservedNames": ["^keep$"],
        "stringArray": false
    }))
    .expect("rename properties options should deserialize");
    let result = obfuscate(
        "const object = {keep: 1, change: 2}; console.log(object.keep, object.change);",
        options,
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("'keep':0x1"), "{}", result.code);
    assert!(result.code.contains("'_0x0':0x2"), "{}", result.code);
    assert!(result.code.contains("object.keep"), "{}", result.code);
    assert!(result.code.contains("object._0x0"), "{}", result.code);
}
```

- [ ] **Step 3: Run red test**

Run: `cargo test -p javascript-obfuscator obfuscate_renames_properties_in_unsafe_mode obfuscate_keeps_reserved_properties_when_renaming_properties`

Expected: compile failure because `Options` has no `rename_properties` fields, or assertion failure because the option is ignored.

### Task 2: Options Surface

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`

- [ ] **Step 1: Add deserializable fields**

```rust
#[serde(default)]
pub rename_properties: Option<bool>,
#[serde(default)]
pub rename_properties_mode: Option<String>,
```

- [ ] **Step 2: Add option deserialization test**

```rust
#[test]
fn deserializes_rename_properties_options_for_option_compatibility() {
    let options: Options = serde_json::from_value(json!({
        "renameProperties": true,
        "renamePropertiesMode": "unsafe"
    }))
    .expect("rename properties options should deserialize");

    assert_eq!(options.rename_properties, Some(true));
    assert_eq!(options.rename_properties_mode, Some("unsafe".to_string()));
}
```

### Task 3: Rename Transform

**Files:**
- Create: `crates/javascript-obfuscator/src/transforms/rename_properties.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Implement transform module**

The visitor should:
- return early unless `renameProperties` is true and `renamePropertiesMode` is exactly `"unsafe"`;
- compile `reservedNames` regexes;
- rename `PropName::Ident`, `PropName::Str`, and computed string keys;
- rename `MemberProp::Ident` and computed string members;
- expand and rename shorthand object-pattern properties so destructuring reads the renamed key;
- skip non-string computed expressions, private names, constructors, and reserved names;
- reuse the same generated name for every original property name.

- [ ] **Step 2: Wire transform before `member_expressions`**

Call the new transform after `object_expression_keys` and before `split_strings` / `member_expressions`, so string property literals are already normalized and later property bracketing can still run.

### Task 4: Verification and Shipping

**Files:**
- Modify only the files above unless a compile error shows a required import or module change.

- [ ] **Step 1: Run focused tests**

Run:
```bash
cargo test -p javascript-obfuscator rename_properties
cargo test -p javascript-obfuscator obfuscate_renames_properties_in_unsafe_mode obfuscate_keeps_reserved_properties_when_renaming_properties
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
git add docs/superpowers/plans/2026-06-01-rust-rename-properties-unsafe-slice.md crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/rename_properties.rs
git commit -m "feat: rename rust properties in unsafe mode"
git pull --rebase origin codex/rust-rewrite-slice-1
git push origin codex/rust-rewrite-slice-1
```
