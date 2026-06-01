# Rust Root String Array Wrapper Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Route Rust string-array references through the root string-array wrapper instead of direct storage member reads.

**Architecture:** Keep the existing Rust storage array and existing root wrapper helper topology for this slice. Make the wrapper emit for every transformed string-array literal, including `encoding: none` with `stringArrayIndexShift: false`, and make the no-shift wrapper read `storage[index]` instead of subtracting `0x64`. Do not add scoped wrappers, wrapper counts, chained wrappers, fake indexes, wrapper randomization, or VMP behavior.

**Tech Stack:** Rust 1.96, SWC AST transforms, existing Rust API and transform tests.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Update transform expectations for root wrapper calls**

Update existing string-array transform tests so `encoding: none` with no index shift expects `_0x1(...)` calls:

```rust
assert!(code.contains("function _0x1(index){return _0x0[index];}"), "{code}");
assert!(code.contains("const value=_0x1(0x0);console.log(_0x1(0x0));"), "{code}");
assert!(!code.contains("const value=_0x0[0x0];"), "{code}");
```

Apply the same call expectation to reserved-string, ignore-imports, force-transform, hexadecimal numeric string, shuffle, and rotate assertions.

- [x] **Step 2: Update API expectations for root wrapper calls**

Update existing API tests so root string-array references expect wrapper calls:

```rust
assert!(result.code.contains("function _0x1(index){return _0x0[index];}"), "{}", result.code);
assert!(result.code.contains("const value=_0x1(0x0);console.log(_0x1(0x0));"), "{}", result.code);
assert!(!result.code.contains("const value=_0x0[0x0];"), "{}", result.code);
```

Update no-shift shuffle and rotate expectations to `_0x1(0xN)` calls, and update hexadecimal numeric string index expectations to `_0x1('0x0')`.

- [x] **Step 3: Add no-shift runtime API test**

Add a runtime API test that disables index shift and verifies the generated root wrapper reads the correct string:

```rust
#[test]
fn obfuscate_none_string_array_root_wrapper_decodes_at_runtime() {
    let result = obfuscate(
        "console.log(['foo', 'bar'].join('|'));",
        Options {
            compact: Some(true),
            string_array: Some(true),
            string_array_threshold: Some(1.0),
            string_array_index_shift: Some(false),
            rename_globals: Some(false),
            property_bracketing: Some(false),
            unicode_escape_sequence: Some(false),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    let output = run_node_source(&result.code);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "foo|bar\n");
}
```

- [x] **Step 4: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator string_array_root_wrapper
```

Expected: the new runtime test fails to run or targeted updated assertions fail because Rust still emits direct `_0x0[...]` reads when no wrapper is required by encoding/index shift.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Always emit root wrapper for string array references**

Change `should_emit_string_array_wrapper` so transformed string-array references and declarations use the wrapper for all encodings and index-shift states.

- [x] **Step 2: Make the none wrapper respect index shift**

Thread `index_shift_enabled` into `create_index_shift_wrapper_statement` and make the wrapper return `_0x0[index]` when shift is disabled, or `_0x0[index-0x64]` when shift is enabled.

- [x] **Step 3: Preserve existing encoded wrapper behavior**

Leave base64 and rc4 wrapper generation unchanged, and keep remap behavior on wrapper call arguments.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator string_array_root_wrapper
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
git add -- crates/javascript-obfuscator/src/transforms/string_array.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-root-string-array-wrapper-slice.md
```

Commit and push:

```bash
git commit -m "feat: route rust string array through root wrapper"
git push origin codex/rust-rewrite-slice-1
```
