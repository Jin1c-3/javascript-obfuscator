# Rust String Array Calls Transform RC4 Key Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend Rust `stringArrayCallsTransform` parity so generated RC4 decode-key literals are stored in function-local call-control storage alongside generated string-array indexes.

**Architecture:** Keep the existing function-body-only transform from the previous slice. For each generated string-array wrapper call, continue transforming the first generated index argument, then also transform the second argument when it is the generated RC4 decode-key literal. Do not transform fake numeric wrapper arguments or user-authored literal calls.

**Tech Stack:** Rust, SWC AST (`swc_ecma_ast`, `swc_ecma_visit`), existing Rust API tests, Node runtime smoke checks.

---

### Task 1: Add RED Coverage

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add RC4 call-transform API/runtime test**

Add a test using:

```rust
{
    "compact": true,
    "propertyBracketing": false,
    "renameGlobals": false,
    "stringArray": true,
    "stringArrayCallsTransform": true,
    "stringArrayCallsTransformThreshold": 1,
    "stringArrayEncoding": ["rc4"],
    "stringArrayIndexShift": false,
    "stringArrayRotate": false,
    "stringArrayShuffle": false,
    "stringArrayThreshold": 1,
    "unicodeEscapeSequence": false
}
```

Expected output shape:

```javascript
function test(){const _0x2={_0x0:0x0,_0x1:'rc4K',_0x2:0x1,_0x3:'rc4K'};const first=_0x1(_0x2._0x0,_0x2._0x1);return first+_0x1(_0x2._0x2,_0x2._0x3);}
```

Expected runtime output: `foobar\n`.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p javascript-obfuscator rc4_string_array_calls_transform -- --nocapture
```

Expected: the new test fails because the index argument is in storage but `'rc4K'` remains inline.

### Task 2: Implement Decode-Key Argument Storage

**Files:**
- Modify: `crates/javascript-obfuscator/src/transforms/string_array.rs`

- [x] **Step 1: Extract argument replacement helper**

Replace the duplicated storage-entry code in `FunctionStringArrayCallsTransform::visit_mut_call_expr` with a helper that accepts an argument index.

- [x] **Step 2: Transform the RC4 decode-key argument**

After replacing argument `0`, inspect argument `1`. If it is a generated string literal whose value is `DEFAULT_RC4_KEY`, move it into the same storage object and replace it with the next storage member expression.

- [x] **Step 3: Avoid false positives**

Keep the existing generated-literal guard for indexes and only transform the second argument when it is exactly the generated RC4 key. Do not transform fake numeric wrapper arguments.

### Task 3: Verify and Ship

**Files:**
- All changed files from Tasks 1-2

- [x] **Step 1: Run focused Rust tests**

```bash
cargo test -p javascript-obfuscator rc4_string_array_calls_transform -- --nocapture
cargo test -p javascript-obfuscator calls_transform -- --nocapture
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

Stage only the RC4 decode-key slice files, commit with:

```bash
git commit -m "feat: transform rust rc4 string array call keys"
```

Then fetch/rebase as needed and push `codex/rust-rewrite-slice-1`.
