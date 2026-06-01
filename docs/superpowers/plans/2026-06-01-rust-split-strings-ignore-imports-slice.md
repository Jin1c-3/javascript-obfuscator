# Rust Split Strings Ignore Imports Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust `splitStrings` skip `require()` and dynamic `import()` string arguments when `ignoreImports` is enabled.

**Architecture:** Thread `Options.ignore_imports` into the split-string transform and copy the existing call-expression skip pattern used by Rust string-array and escape-sequence transforms. Only import/require call subtrees should be skipped; other string literals should still split normally. Do not add VMP behavior.

**Tech Stack:** Rust 1.96, SWC AST transforms, existing Rust API tests.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/split_strings.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add require-call transform test**

Add a split-string transform test with `ignore_imports: true` and:

```javascript
const foo = require('./abcdef'); const bar = './ghijkl';
```

Assert `require('./abcdef')` remains inline and `bar` still splits into chunks.

- [x] **Step 2: Add dynamic import transform test**

Add a split-string transform test with `ignore_imports: true` and:

```javascript
const mod = import('./abcdef'); const bar = './ghijkl';
```

Assert `import('./abcdef')` remains inline and `bar` still splits into chunks.

- [x] **Step 3: Add API test**

Add an API test with `splitStrings: true`, `splitStringsChunkLength: 3`, `ignoreImports: true`, and `stringArray: false`. Assert the `require()` literal stays inline while a non-import literal still splits.

- [x] **Step 4: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator split_strings_ignore_imports
```

Expected: compile/test failure because `transform_split_strings` does not yet accept or apply `ignore_imports`.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/split_strings.rs`

- [x] **Step 1: Thread ignoreImports**

Add an `ignore_imports: bool` parameter to `transform_split_strings` and pass `options.ignore_imports.unwrap_or(false)` from `apply_transforms`.

- [x] **Step 2: Skip ignored import call subtrees**

Add `visit_mut_call_expr` to `SplitStringTransform` and return early when `ignore_imports` is true and the call is `require(...)` or dynamic `import(...)`.

- [x] **Step 3: Update existing tests**

Update existing split-string transform test helper calls to pass `ignore_imports: false`.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator split_strings_ignore_imports
cargo test -p javascript-obfuscator split_strings
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
git add -- crates/javascript-obfuscator/src/transforms/mod.rs crates/javascript-obfuscator/src/transforms/split_strings.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-split-strings-ignore-imports-slice.md
```

Commit and push:

```bash
git commit -m "feat: respect ignore imports in rust split strings"
git push origin codex/rust-rewrite-slice-1
```
