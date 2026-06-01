# Rust Force Transform Strings Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust support for the `forceTransformStrings` option in the string-array transform.

**Architecture:** Deserialize `forceTransformStrings` into Rust `Options`, thread it into `StringArrayTransformOptions`, and treat matching string literals as forced string-array candidates. Forced literals should transform even when `stringArrayThreshold` is `0` and should take priority over `reservedStrings`, while `stringArray: false` still disables string-array behavior. Use regular-expression matching to mirror the TypeScript guard behavior for this option.

**Tech Stack:** Rust 1.96, SWC AST transforms, `regex` crate, existing Rust API and transform tests.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add option deserialization test**

Add a test in `crates/javascript-obfuscator/src/options.rs`:

```rust
#[test]
fn deserializes_force_transform_strings_for_option_compatibility() {
    let options: Options = serde_json::from_value(json!({
        "forceTransformStrings": ["ar$"]
    }))
    .expect("force transform strings option should deserialize");

    assert_eq!(options.force_transform_strings, Some(vec!["ar$".to_string()]));
}
```

- [x] **Step 2: Add transform tests**

Add two tests in `crates/javascript-obfuscator/src/transforms/string_array.rs`:

```rust
#[test]
fn force_transforms_matching_string_when_threshold_is_zero() {
    let force_transform_strings = vec!["ar$".to_string()];
    let mut parsed_program = parse_program("const foo = 'foo'; const bar = 'bar';")
        .expect("source should parse");
    transform_string_array(
        &mut parsed_program.program,
        StringArrayTransformOptions {
            enabled: true,
            threshold: 0.0,
            indexes_type: &[],
            encoding: StringArrayEncoding::None,
            index_shift: false,
            shuffle: false,
            rotate: false,
            reserved_strings: &[],
            force_transform_strings: &force_transform_strings,
            ignore_imports: false,
        },
    );
    let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
        .expect("code should generate");

    assert!(code.contains("const _0x0=['bar'];"), "{code}");
    assert!(code.contains("const foo='foo';"), "{code}");
    assert!(code.contains("const bar=_0x0[0x0];"), "{code}");
}

#[test]
fn force_transform_strings_take_priority_over_reserved_strings() {
    let reserved_strings = vec!["bar".to_string()];
    let force_transform_strings = vec!["bar".to_string()];
    let mut parsed_program = parse_program("const foo = 'foo'; const bar = 'bar';")
        .expect("source should parse");
    transform_string_array(
        &mut parsed_program.program,
        StringArrayTransformOptions {
            enabled: true,
            threshold: 0.0,
            indexes_type: &[],
            encoding: StringArrayEncoding::None,
            index_shift: false,
            shuffle: false,
            rotate: false,
            reserved_strings: &reserved_strings,
            force_transform_strings: &force_transform_strings,
            ignore_imports: false,
        },
    );
    let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
        .expect("code should generate");

    assert!(code.contains("const _0x0=['bar'];"), "{code}");
    assert!(code.contains("const foo='foo';"), "{code}");
    assert!(code.contains("const bar=_0x0[0x0];"), "{code}");
}
```

- [x] **Step 3: Add API test**

Add a test in `crates/javascript-obfuscator/src/api.rs`:

```rust
#[test]
fn obfuscate_force_transforms_matching_string_when_threshold_is_zero() {
    let result = obfuscate(
        "const foo = 'foo'; const bar = 'bar';",
        Options {
            compact: Some(true),
            string_array: Some(true),
            string_array_threshold: Some(0.0),
            force_transform_strings: Some(vec!["ar$".to_string()]),
            rename_globals: Some(false),
            property_bracketing: Some(false),
            unicode_escape_sequence: Some(false),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    assert!(result.code.contains("const _0x0=['bar'];"), "{}", result.code);
    assert!(result.code.contains("const foo='foo';"), "{}", result.code);
    assert!(result.code.contains("const bar=_0x0[0x0];"), "{}", result.code);
}
```

- [x] **Step 4: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator force_transform_strings
```

Expected: compile/test failure because Rust does not yet expose or thread `force_transform_strings`.

### Task 2: Implementation

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/javascript-obfuscator/Cargo.toml`
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Add regex dependency**

Add `regex = "1"` to `[workspace.dependencies]` in `Cargo.toml` and `regex.workspace = true` to `crates/javascript-obfuscator/Cargo.toml`.

- [x] **Step 2: Add option field**

Add to `Options`:

```rust
#[serde(default)]
pub force_transform_strings: Option<Vec<String>>,
```

- [x] **Step 3: Thread option into string array transform**

Add `force_transform_strings: &'a [String]` to `StringArrayTransformOptions<'a>`, pass `options.force_transform_strings.as_deref().unwrap_or(&[])` from `apply_transforms`, and update existing transform tests to pass `&[]`.

- [x] **Step 4: Apply force matching**

Compile the force patterns with `regex::Regex`, keep transforming only when `stringArray` is enabled, and allow matching strings through the transform even when threshold is `0` or the value also matches `reservedStrings`.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator force_transform_strings
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
git add -- Cargo.toml Cargo.lock crates/javascript-obfuscator/Cargo.toml crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/string_array.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-force-transform-strings-slice.md
```

Commit and push:

```bash
git commit -m "feat: add rust force transform strings option"
git push origin codex/rust-rewrite-slice-1
```
