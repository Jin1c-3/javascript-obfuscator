# Rust Debug Protection Basic Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first Rust parity slice for the `debugProtection` and `debugProtectionInterval` options by emitting a deterministic runtime helper without VMP behavior.

**Architecture:** Add the TypeScript-compatible option fields to Rust, then insert a small helper after imports/directives when `debugProtection` is enabled. The helper runs a `debugger` statement once and optionally schedules the same function with `setInterval` when `debugProtectionInterval` is greater than zero.

**Tech Stack:** Rust, SWC parser/codegen, existing Rust transform pipeline, Node `vm` runtime checks.

---

### Task 1: Add RED Coverage

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add option compatibility test**

Add a JSON deserialize/serialize test for:

```json
{
  "debugProtection": true,
  "debugProtectionInterval": 4000.9
}
```

Expected before implementation: the test fails because Rust does not expose these fields.

- [x] **Step 2: Add runtime API tests**

Add tests that verify:
- Enabled debug protection inserts a `debugger` helper, omits `setInterval` when the interval is zero, and still executes user code.
- A positive interval inserts `setInterval` without executing the live interval in tests.
- Disabled debug protection ignores a positive interval and does not insert helper code.

- [x] **Step 3: Verify RED**

Run focused Cargo filters for the new option and API tests.

Expected: tests fail because Rust currently lacks option fields and helper behavior.

### Task 2: Implement Rust Debug Protection

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/debug_protection.rs`

- [x] **Step 1: Add option fields**

Add:

```rust
#[serde(default)]
pub debug_protection: Option<bool>,
#[serde(default, deserialize_with = "deserialize_optional_usize_floor")]
pub debug_protection_interval: Option<usize>,
```

- [x] **Step 2: Add transform module**

Implement `transform_debug_protection(program, enabled, interval)` with an early return when the option is disabled.

- [x] **Step 3: Insert helper after directives/imports**

Parse helper source and splice statements after imports and directive prologues, matching the existing helper insertion pattern.

- [x] **Step 4: Gate interval helper**

Only emit `setInterval` when `debugProtectionInterval` is greater than zero.

### Task 3: Verify and Ship

**Files:**
- All changed files from Tasks 1-2

- [x] **Step 1: Run focused Rust tests**

```bash
cargo test -p javascript-obfuscator debug_protection
```

Expected: all focused tests pass.

- [x] **Step 2: Run full verification gate**

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npx eslint "src/**/*.ts"
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
rg -n "VMP|vmp|virtual machine|virtual-machine|virtualMachine" crates src test package.json
```

Expected: all commands exit 0 except the VMP scan exits 1 with no matches.

- [ ] **Step 3: Commit and push**

Stage only the debug-protection slice files, commit with:

```bash
git commit -m "feat: add rust debug protection helper"
```

Then fetch/rebase as needed and push `codex/rust-rewrite-slice-1`.
