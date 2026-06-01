# Rust Rename Properties Reserved DOM Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust `renameProperties` unsafe mode honor the same reserved DOM property-name list used by the TypeScript implementation.

**Architecture:** Keep a copy of `ReservedDomProperties.json` inside the Rust crate assets so packaged Rust builds do not depend on repo-root TypeScript source files. The rename-properties visitor will query that parsed list before allocating a generated property name, keeping existing `reservedNames` behavior intact.

**Tech Stack:** Rust, SWC AST visitors, `serde_json`, `OnceLock`, existing TypeScript reserved DOM JSON fixture.

---

### Task 1: Failing API Test

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add a runtime test for a known DOM-reserved property**

```rust
#[test]
fn obfuscate_keeps_reserved_dom_properties_when_renaming_properties() {
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
        "const object = {then: 1, custom: 2}; console.log(object.then, object.custom);",
        options,
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("'then':0x1"), "{}", result.code);
    assert!(result.code.contains("'_0x0':0x2"), "{}", result.code);
    assert!(result.code.contains("object.then"), "{}", result.code);
    assert!(result.code.contains("object._0x0"), "{}", result.code);

    let output = run_node_source(&result.code);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1 2\n");
}
```

- [ ] **Step 2: Verify red**

Run: `cargo test -p javascript-obfuscator obfuscate_keeps_reserved_dom_properties_when_renaming_properties`

Expected: FAIL because `then` is currently renamed.

### Task 2: Rust Reserved DOM Lookup

**Files:**
- Create: `crates/javascript-obfuscator/assets/ReservedDomProperties.json`
- Modify: `crates/javascript-obfuscator/src/transforms/rename_properties.rs`

- [ ] **Step 1: Include and parse the JSON list**

Copy `src/constants/ReservedDomProperties.json` to `crates/javascript-obfuscator/assets/ReservedDomProperties.json`, then use `include_str!("../../assets/ReservedDomProperties.json")` from `rename_properties.rs`.

- [ ] **Step 2: Add a cached lookup**

Use `OnceLock<HashSet<String>>` to parse the JSON list once:

```rust
static RESERVED_DOM_PROPERTY_NAMES: OnceLock<HashSet<String>> = OnceLock::new();
```

- [ ] **Step 3: Update `should_keep_name`**

Replace the current hard-coded list with `reserved_dom_property_names().contains(name)`.

### Task 3: Verification and Shipping

**Files:**
- Modify only `api.rs`, `rename_properties.rs`, `crates/javascript-obfuscator/assets/ReservedDomProperties.json`, and this plan unless the compiler requires another file.

- [ ] **Step 1: Run focused tests**

Run:
```bash
cargo test -p javascript-obfuscator obfuscate_keeps_reserved_dom_properties_when_renaming_properties
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
git add docs/superpowers/plans/2026-06-01-rust-rename-properties-dom-reserved-slice.md crates/javascript-obfuscator/assets/ReservedDomProperties.json crates/javascript-obfuscator/src/api.rs crates/javascript-obfuscator/src/transforms/rename_properties.rs
git commit -m "feat: reserve rust dom property names"
git pull --rebase origin codex/rust-rewrite-slice-1
git push origin codex/rust-rewrite-slice-1
```
