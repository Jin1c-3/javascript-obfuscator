# Rust Dead Code Injection Basic Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first Rust parity slice for the `deadCodeInjection` and `deadCodeInjectionThreshold` options by inserting deterministic unreachable code without VMP behavior.

**Architecture:** Add the TypeScript-compatible option fields to Rust, then insert a small inert `if` statement after imports/directives when `deadCodeInjection` is enabled and `deadCodeInjectionThreshold` is positive. Later transforms still run over the injected block.

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
  "deadCodeInjection": true,
  "deadCodeInjectionThreshold": 0.4
}
```

Expected before implementation: the test fails because Rust does not expose these fields.

- [x] **Step 2: Add runtime API tests**

Add tests that verify:
- Enabled dead-code injection with threshold `1` inserts an unreachable block before user code and preserves runtime behavior.
- Threshold `0` does not insert dead code.
- Disabled dead-code injection ignores a positive threshold.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p javascript-obfuscator dead_code_injection
```

Expected: tests fail because Rust currently lacks option fields and transform behavior.

### Task 2: Implement Rust Dead Code Injection

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/dead_code_injection.rs`

- [x] **Step 1: Add option fields**

Add:

```rust
#[serde(default)]
pub dead_code_injection: Option<bool>,
#[serde(default)]
pub dead_code_injection_threshold: Option<f64>,
```

- [x] **Step 2: Add transform module**

Implement `transform_dead_code_injection(program, enabled, threshold)` with an early return when disabled or when `threshold <= 0`.

- [x] **Step 3: Insert unreachable block after directives/imports**

Parse helper source and splice statements after imports and directive prologues, matching existing helper insertion patterns.

- [x] **Step 4: Wire pipeline order**

Run the transform after helper/protection insertion and before converting/string transforms so later transformations can still process injected nodes.

### Task 3: Verify and Ship

**Files:**
- All changed files from Tasks 1-2

- [x] **Step 1: Run focused Rust tests**

```bash
cargo test -p javascript-obfuscator dead_code_injection
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

Stage only the dead-code-injection slice files, commit with:

```bash
git commit -m "feat: add rust dead code injection helper"
```

Then fetch/rebase as needed and push `codex/rust-rewrite-slice-1`.
