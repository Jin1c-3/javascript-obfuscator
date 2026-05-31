# Rust Eval Call Expression Transform Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the literal/template `eval(...)` string transformation behavior into the Rust obfuscation pipeline.

**Architecture:** Add a focused SWC visitor that finds direct `eval` calls with a first argument that is a string literal or cooked no-expression template literal. Parse that string as JavaScript, run the existing Rust transform pipeline recursively over the nested program, generate compact nested code, and replace the eval argument with the transformed code string. Invalid eval source and nonliteral eval arguments remain unchanged.

**Tech Stack:** Rust 1.96, SWC AST/parser/codegen/visitor crates, existing `javascript-obfuscator` Rust crate test harness.

---

### Task 1: Failing API Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Add eval call tests**

Add API tests that prove direct eval string contents are transformed, template literals are accepted after template conversion, and invalid eval strings remain unchanged.

- [ ] **Step 2: Run red test**

Run: `cargo test -p javascript-obfuscator obfuscate_transforms_literal_eval_string_contents obfuscate_transforms_template_eval_string_contents obfuscate_keeps_unparseable_eval_string`

Expected: the transform tests fail because eval string contents are currently treated as opaque string literals.

### Task 2: Eval Transform Module

**Files:**
- Create: `crates/javascript-obfuscator/src/transforms/eval_call_expressions.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`

- [ ] **Step 1: Implement visitor**

Create a visitor over `CallExpr` that recognizes `eval(...)`, extracts the first literal/template argument, parses it with `parse_program`, recursively calls `apply_transforms`, generates compact code with `generate_code`, and replaces the first argument with a string literal containing that generated code.

- [ ] **Step 2: Wire transform**

Add the module to `transforms/mod.rs` and invoke it after simplify transforms and before escape sequences so nested eval output gets the same final string escaping as other string literals.

- [ ] **Step 3: Run green test**

Run: `cargo test -p javascript-obfuscator eval_call_expression`

Expected: module tests pass.

Run: `cargo test -p javascript-obfuscator obfuscate_transforms_literal_eval_string_contents obfuscate_transforms_template_eval_string_contents obfuscate_keeps_unparseable_eval_string`

Expected: API tests pass.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [ ] **Step 1: Format and verify Rust**

Run: `cargo fmt --check`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings`.

- [ ] **Step 2: Verify TypeScript bridge still passes**

Run: `npx eslint src/**/*.ts` and `npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts`.

- [ ] **Step 3: Commit and push**

Stage the Rust files and plan, commit with `feat: add rust eval call expression transform`, and push `codex/rust-rewrite-slice-1`.
