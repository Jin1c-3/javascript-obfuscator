# Rust Split Strings Graphemes Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust `splitStrings` chunk user-visible Unicode graphemes like the TypeScript `stringz` implementation.

**Architecture:** Replace scalar-value chunking in `split_strings.rs` with Unicode grapheme clustering. Keep the transform shape and existing options unchanged; this slice only changes how string chunks are measured.

**Tech Stack:** Rust 1.96, `unicode-segmentation`, SWC AST transforms.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/split_strings.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add transform emoji grapheme test**

Add a split string transform test for:

```javascript
const value = 'ab👋🏼cd';
```

with `splitStringsChunkLength` equivalent to `1`. Assert the output contains:

```javascript
const value='a'+'b'+'👋🏼'+'c'+'d'
```

- [x] **Step 2: Add API emoji grapheme test**

Add an API-level test for the same input and options. Assert the obfuscated code preserves `👋🏼` as one generated chunk.

- [x] **Step 3: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator split_strings_emoji
```

Expected: tests fail because Rust currently splits `👋🏼` into separate scalar-value chunks.

### Task 2: Implementation

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/javascript-obfuscator/Cargo.toml`
- Modify: `crates/javascript-obfuscator/src/transforms/split_strings.rs`

- [x] **Step 1: Add unicode segmentation dependency**

Add `unicode-segmentation = "1"` to workspace dependencies and depend on it from `crates/javascript-obfuscator`.

- [x] **Step 2: Chunk by grapheme clusters**

Use `unicode_segmentation::UnicodeSegmentation` in `chunk_string`, collect `value.graphemes(true)`, and chunk those grapheme strings by the configured chunk length.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo fmt
cargo test -p javascript-obfuscator split_strings_emoji
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
git add -- Cargo.toml Cargo.lock crates/javascript-obfuscator/Cargo.toml crates/javascript-obfuscator/src/transforms/split_strings.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-split-strings-graphemes-slice.md
```

Commit and push:

```bash
git commit -m "feat: split rust strings by graphemes"
git push origin codex/rust-rewrite-slice-1
```
