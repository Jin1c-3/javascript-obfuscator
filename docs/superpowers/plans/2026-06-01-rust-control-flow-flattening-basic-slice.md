# Rust Control Flow Flattening Basic Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first Rust parity slice for the `controlFlowFlattening` and `controlFlowFlatteningThreshold` options by flattening safe straight-line blocks without VMP behavior.

**Architecture:** Add the TypeScript-compatible option fields to Rust, then transform eligible block statements into a deterministic `while`/`switch` dispatcher when control-flow flattening is enabled and the threshold is positive. This first slice only flattens blocks with at least five non-directive expression statements and preserves leading directives.

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
  "controlFlowFlattening": true,
  "controlFlowFlatteningThreshold": 0.75
}
```

Expected before implementation: the test fails because Rust does not expose these fields.

- [x] **Step 2: Add runtime API tests**

Add tests that verify:
- Enabled control-flow flattening with threshold `1` inserts a `switch` dispatcher and preserves runtime order.
- Threshold `0` does not flatten.
- Disabled control-flow flattening ignores a positive threshold.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p javascript-obfuscator control_flow_flattening
```

Expected: tests fail because Rust currently lacks option fields and transform behavior.

### Task 2: Implement Rust Control Flow Flattening

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/control_flow_flattening.rs`

- [x] **Step 1: Add option fields**

Add:

```rust
#[serde(default)]
pub control_flow_flattening: Option<bool>,
#[serde(default)]
pub control_flow_flattening_threshold: Option<f64>,
```

- [x] **Step 2: Add transform module**

Implement `transform_control_flow_flattening(program, enabled, threshold)` with an early return when disabled or when `threshold <= 0`.

- [x] **Step 3: Flatten safe block statements**

Preserve directive prologues and replace eligible expression-only block tails with a deterministic dispatcher:

```javascript
var _0xcontrolFlowIndex = 0;
while (true) {
  switch (_0xcontrolFlowIndex++) {
    case 0:
      statement0;
      continue;
  }
  break;
}
```

- [x] **Step 4: Match first TypeScript eligibility gate**

Require at least five non-directive candidate statements, matching TypeScript's minimum direct statement count for block flattening.

### Task 3: Verify and Ship

**Files:**
- All changed files from Tasks 1-2

- [x] **Step 1: Run focused Rust tests**

```bash
cargo test -p javascript-obfuscator control_flow_flattening
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

Stage only the control-flow-flattening slice files, commit with:

```bash
git commit -m "feat: add rust control flow flattening"
```

Then fetch/rebase as needed and push `codex/rust-rewrite-slice-1`.
