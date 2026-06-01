# Rust Reserved Strings Regex Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust `reservedStrings` matching use regular expressions like the TypeScript obfuscating guard.

**Architecture:** Keep each transform’s current reserved-string gate, but replace substring checks with regex matching. Compile reserved string patterns once per transform invocation where practical, pass compiled patterns into visitors, and preserve `forceTransformStrings` priority over `reservedStrings` in the string-array transform. Do not add VMP behavior.

**Tech Stack:** Rust 1.96, `regex` crate already used by the Rust string-array transform, SWC AST transforms.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/split_strings.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/escape_sequences.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add string-array regex reserved test**

Add a transform test where `reserved_strings == ["ar$"]`, source is:

```javascript
const foo = 'foo'; const bar = 'bar';
```

Assert `foo` is moved to the string array and `bar` remains inline.

- [x] **Step 2: Add split-string regex reserved test**

Add a split-string transform test where `reserved_strings == ["ar$"]`, source is:

```javascript
const foo = 'foofoo'; const bar = 'barbar';
```

Assert `foofoo` splits and `barbar` remains inline.

- [x] **Step 3: Add escape-sequence regex reserved test**

Add an escape-sequence transform test where `reserved_strings == ["ar$"]`, source is:

```javascript
const foo = 'foo'; const bar = 'bar';
```

Assert `foo` is escaped when `unicodeEscapeSequence` is enabled and `bar` remains inline.

- [x] **Step 4: Add API regex reserved test**

Add an API test with `stringArray: true`, `stringArrayThreshold: 1`, `reservedStrings: ["ar$"]`, and `unicodeEscapeSequence: false`. Assert `foo` is transformed through the root string-array wrapper and `bar` remains inline.

- [x] **Step 5: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator regex_reserved
```

Expected: tests fail because Rust substring matching does not treat `ar$` as a regex that matches `bar`.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/split_strings.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/escape_sequences.rs`

- [x] **Step 1: Compile reserved regex patterns in string array**

Replace `StringArrayTransform.reserved_strings` with compiled regex patterns and use `Regex::is_match`. Keep `forceTransformStrings` priority unchanged.

- [x] **Step 2: Compile reserved regex patterns in split strings**

Replace the split-string visitor’s reserved string slice with compiled regex patterns and use regex matching before splitting.

- [x] **Step 3: Compile reserved regex patterns in escape sequences**

Replace the escape-sequence visitor’s reserved string slice with compiled regex patterns and use regex matching before escaping.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator regex_reserved
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
git add -- crates/javascript-obfuscator/src/transforms/string_array.rs crates/javascript-obfuscator/src/transforms/split_strings.rs crates/javascript-obfuscator/src/transforms/escape_sequences.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-reserved-strings-regex-slice.md
```

Commit and push:

```bash
git commit -m "feat: match rust reserved strings as regex"
git push origin codex/rust-rewrite-slice-1
```
