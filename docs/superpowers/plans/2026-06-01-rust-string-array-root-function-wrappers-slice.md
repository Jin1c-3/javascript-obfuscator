# Rust String Array Root Function Wrappers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Rust parity slice for root-scope `stringArrayWrappersType: "function"` wrappers.

**Architecture:** Reuse the existing Rust root decoder `_0x1` as the upper wrapper and generate deterministic function wrappers (`_0x2`, `_0x3`, ...). Calls routed through function wrappers carry a wrapper-local index offset and optional padded fake arguments so `stringArrayWrappersParametersMaxCount` affects output without changing runtime semantics.

**Tech Stack:** Rust, SWC AST, existing `cargo test` coverage, Node runtime smoke tests.

---

### Task 1: Function Wrapper Tests And Implementation

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Write failing tests**

Add tests that prove:
- API output emits root function wrappers for `stringArrayWrappersType: "function"` and `stringArrayWrappersCount > 0`.
- `stringArrayWrappersParametersMaxCount` pads function-wrapper calls with fake arguments.
- Generated function-wrapper output executes correctly under Node.
- Shuffle remaps call indexes correctly when function wrappers apply local index offsets.

- [x] **Step 2: Verify red**

Run: `cargo test -p javascript-obfuscator root_function_string_array_wrappers -- --nocapture`

Expected: FAIL because function wrappers currently fall back to the root `_0x1` calls.

- [x] **Step 3: Implement minimal function-wrapper support**

Extend `StringArrayTransformOptions` with `wrappers_parameters_max_count`. Replace name-only wrapper state with wrapper data containing name, kind, local index shift, and parameter count. Generate function declarations such as `function _0x2(index,key,unused0){return _0x1(index-0x1,key);}` and route calls through padded argument lists.

- [x] **Step 4: Verify green**

Run: `cargo test -p javascript-obfuscator root_function_string_array_wrappers -- --nocapture`

Expected: all focused function-wrapper tests pass.

- [x] **Step 5: Run full verification and ship**

Run:
`cargo fmt --check`
`cargo test --workspace`
`cargo clippy --workspace --all-targets -- -D warnings`
`npx eslint "src/**/*.ts"`
`npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts`
`rg -n "VMP|vmp|virtual machine|virtual-machine|virtualMachine" crates src test package.json`

Expected: all commands exit 0 except the VMP scan, which exits 1 with no matches.
