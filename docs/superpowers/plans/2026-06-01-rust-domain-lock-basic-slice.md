# Rust Domain Lock Basic Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first Rust parity slice for the `domainLock` protection option by emitting a runtime helper that redirects disallowed browser-like domains.

**Architecture:** Add `domainLock` and `domainLockRedirectUrl` to the Rust option surface, normalize configured domains using the same simple extraction rule as TypeScript, and insert a small helper after imports/directives when at least one domain is configured. The helper reads `document.domain` or `document.location.hostname`, applies the TypeScript suffix/exact-match rule, and assigns `document.location` to the redirect URL when the current domain is not allowed.

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
  "domainLock": ["https://Example.com:9000/path"],
  "domainLockRedirectUrl": "https://blocked.example/path"
}
```

Expected before implementation: serialized options do not preserve these fields.

- [x] **Step 2: Add runtime API tests**

Add tests that verify:
- An allowed domain keeps `document.location.hostname` unchanged and generated code still executes.
- A blocked domain assigns `document.location` to the configured redirect URL.
- Empty `domainLock` does not inject a domain-lock helper.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p javascript-obfuscator domain_lock -- --nocapture
```

Expected: tests compile and fail because Rust currently ignores `domainLock`.

### Task 2: Implement Rust Domain Lock

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/transforms/mod.rs`
- Create: `crates/javascript-obfuscator/src/transforms/domain_lock.rs`

- [x] **Step 1: Add option fields**

Add:

```rust
#[serde(default)]
pub domain_lock: Option<Vec<String>>,
#[serde(default)]
pub domain_lock_redirect_url: Option<String>,
```

- [x] **Step 2: Add transform module**

Implement `transform_domain_lock(program, domains, redirect_url)` with early return when `domains` is empty.

- [x] **Step 3: Normalize domains**

Use the TypeScript-compatible rule:
- If the value contains `://` or starts with `//`, use the third slash-delimited segment.
- Otherwise use the first slash-delimited segment.
- Drop any port after `:`.
- Lowercase the result.

- [x] **Step 4: Insert helper after directives/imports**

Parse helper source and splice statements after imports and directive prologues, matching the existing helper insertion pattern.

### Task 3: Verify and Ship

**Files:**
- All changed files from Tasks 1-2

- [x] **Step 1: Run focused Rust tests**

```bash
cargo test -p javascript-obfuscator domain_lock -- --nocapture
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

Stage only the domain-lock slice files, commit with:

```bash
git commit -m "feat: add rust domain lock helper"
```

Then fetch/rebase as needed and push `codex/rust-rewrite-slice-1`.
