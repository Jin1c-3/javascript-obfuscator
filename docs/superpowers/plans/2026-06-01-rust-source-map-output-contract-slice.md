# Rust Source Map Output Contract Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align the Rust result contract for source map options with the TypeScript `ObfuscationResult` behavior for inline/separate source map comments and `sourceMapSourcesMode`.

**Architecture:** Keep the current placeholder source map metadata with empty mappings. Add option fields for source map URL pieces, generate `sourcesContent` only for `sourceMapSourcesMode: "sources-content"`, append an inline `data:application/json;base64,...` comment for inline mode, and append the configured URL for separate mode when a URL is available. Do not add VMP behavior.

**Tech Stack:** Rust 1.96, serde option fields, `serde_json::Value`, small local base64 encoder for source map comments.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add sources mode test**

Add an API test that obfuscates with:

```json
{
  "sourceMap": true,
  "sourceMapSourcesMode": "sources",
  "inputFileName": "input.js"
}
```

Parse `result.source_map` as JSON and assert `sources == ["input.js"]` and `sourcesContent` is absent.

- [x] **Step 2: Add inline sourceMappingURL test**

Add an API test that obfuscates with:

```json
{
  "compact": true,
  "sourceMap": true,
  "sourceMapMode": "inline",
  "stringArray": false
}
```

Assert `result.code` contains:

```text
//# sourceMappingURL=data:application/json;base64,
```

- [x] **Step 3: Add separate sourceMappingURL test**

Add an API test that obfuscates with:

```json
{
  "compact": true,
  "sourceMap": true,
  "sourceMapMode": "separate",
  "sourceMapBaseUrl": "https://cdn.example/maps/",
  "sourceMapFileName": "bundle.js.map",
  "stringArray": false
}
```

Assert `result.code` ends with:

```text
//# sourceMappingURL=https://cdn.example/maps/bundle.js.map
```

- [x] **Step 4: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator source_map
```

Expected: failures because Rust currently always includes `sourcesContent` and never appends sourceMappingURL comments.

### Task 2: Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/pipeline.rs`

- [x] **Step 1: Add option fields**

Add camelCase serde fields:

```rust
pub source_map_base_url: Option<String>,
pub source_map_file_name: Option<String>,
```

- [x] **Step 2: Respect `sourceMapSourcesMode`**

In `build_source_map_metadata`, include `sourcesContent` only unless `source_map_sources_mode == Some("sources")`.

- [x] **Step 3: Append sourceMappingURL comments**

After building source map metadata, append to `code`:

```rust
//# sourceMappingURL=data:application/json;base64,<base64(source_map)>
```

for inline mode, or:

```rust
//# sourceMappingURL=<sourceMapBaseUrl><sourceMapFileName>
```

for separate mode when the concatenated URL is non-empty.

- [x] **Step 4: Add local standard base64 encoder**

Add a small helper in `pipeline.rs` using the standard Base64 alphabet with `=` padding.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator source_map
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
git add -- crates/javascript-obfuscator/src/options.rs crates/javascript-obfuscator/src/pipeline.rs crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-source-map-output-contract-slice.md
```

Commit and push:

```bash
git commit -m "feat: align rust source map output contract"
git push origin codex/rust-rewrite-slice-1
```
