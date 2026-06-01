# Rust Options Preset Parity Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust `get_options_by_preset` return the same preset option surface as the TypeScript presets.

**Architecture:** Keep this as API/data parity only. Return full JSON option objects for default, low, medium, and high presets using the same string enum values as TypeScript. Do not implement any new transforms in this slice.

**Tech Stack:** Rust 1.96, `serde_json::Value`, public API tests.

---

### Task 1: Failing Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add default preset shape test**

Assert `get_options_by_preset(Preset::Default)` includes TypeScript default keys and values, including:

```rust
"optionsPreset": "default"
"identifierNamesGenerator": "hexadecimal"
"stringArray": true
"stringArrayRotate": true
"stringArrayShuffle": true
"stringArrayEncoding": ["none"]
"stringArrayIndexShift": true
"stringArrayThreshold": 0.75
"target": "browser"
"propertyBracketing": true
```

Also assert the object has at least the broad TypeScript option surface, not only the old minimal keys.

- [x] **Step 2: Add low/medium/high override tests**

Assert:

```rust
low.disableConsoleOutput == true
low.selfDefending == true
low.stringArrayCallsTransformThreshold == 0

medium.controlFlowFlattening == true
medium.deadCodeInjection == true
medium.stringArrayEncoding == ["base64"]
medium.stringArrayWrappersType == "function"
medium.transformObjectKeys == true

high.debugProtection == true
high.debugProtectionInterval == 4000
high.stringArrayEncoding == ["rc4"]
high.stringArrayWrappersCount == 5
high.stringArrayThreshold == 1
```

- [x] **Step 3: Run red tests**

Run:

```bash
cargo test -p javascript-obfuscator options_preset
```

Expected: failures because Rust currently returns only a minimal subset of preset fields.

### Task 2: Full Preset JSON

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [x] **Step 1: Add default preset helper**

Build a `serde_json::Map<String, Value>` containing all TypeScript default preset values from `src/options/presets/Default.ts`.

- [x] **Step 2: Add override helper**

Add a small helper that applies `(&str, Value)` overrides to a preset map and returns `Value::Object`.

- [x] **Step 3: Build low/medium/high from defaults**

Apply TypeScript-equivalent overrides from `LowObfuscation.ts`, `MediumObfuscation.ts`, and `HighObfuscation.ts`.

### Task 3: Verification And Ship

**Files:**
- Modify as needed based on formatter/linter output.

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p javascript-obfuscator options_preset
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
git add -- crates/javascript-obfuscator/src/api.rs docs/superpowers/plans/2026-06-01-rust-options-preset-parity-slice.md
```

Commit and push:

```bash
git commit -m "feat: align rust options presets"
git push origin codex/rust-rewrite-slice-1
```
