# Rust String Array Root Wrappers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a narrow Rust parity slice for `stringArrayWrappersCount` and `stringArrayWrappersType` by supporting root-scope variable wrapper aliases.

**Architecture:** Keep the existing Rust root string array wrapper as the canonical decoder and add optional alias wrappers in front of transformed call sites. The first slice handles deterministic root wrappers only; nested lexical wrappers, chained calls, fake parameters, and randomized function wrappers remain separate slices.

**Tech Stack:** Rust, SWC AST, existing `cargo test` unit/API coverage, Node runtime smoke tests.

---

### Task 1: Option Surface And Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Write the failing tests**

Add tests that prove:
- `stringArrayWrappersCount`, `stringArrayWrappersType`, and `stringArrayWrappersParametersMaxCount` deserialize into Rust `Options`.
- `stringArrayWrappersCount: 2` with `stringArrayWrappersType: "variable"` emits two root aliases and uses them for string-array calls.
- The generated alias-wrapper code executes correctly in Node.

- [x] **Step 2: Run the focused tests to verify red**

Run: `cargo test -p javascript-obfuscator string_array_wrappers -- --nocapture`

Expected: compile failure or assertion failure because the Rust options and transform behavior do not exist yet.

- [x] **Step 3: Implement minimal support**

Add a `StringArrayWrappersType` enum and optional fields to `Options`. Extend `StringArrayTransformOptions` with `wrappers_count` and `wrappers_type`. When `wrappers_count > 0` and type is `variable`, insert `const _0x2=_0x1;`, `const _0x3=_0x1;`, and rotate call sites across those alias names.

- [x] **Step 4: Run focused tests to verify green**

Run: `cargo test -p javascript-obfuscator string_array_wrappers -- --nocapture`

Expected: all focused wrapper tests pass.

- [x] **Step 5: Run full verification and ship**

Run:
`cargo fmt --check`
`cargo test --workspace`
`cargo clippy --workspace --all-targets -- -D warnings`
`npx eslint "src/**/*.ts"`
`npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts`
`rg -n "VMP|vmp|virtual machine|virtual-machine|virtualMachine" crates src test package.json`

Expected: all commands exit 0 except the VMP scan, which exits 1 with no matches.
