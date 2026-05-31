# Rust Rewrite Slice 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the VMP/Pro API surface and add the first Rust-backed package scaffold without changing preserved non-VMP user workflows.

**Architecture:** This slice keeps the existing TypeScript obfuscation engine temporarily as a compatibility oracle while adding Rust crates and a Rust API boundary. The Node package will gain a Rust binding scaffold, and Pro API/VM/parseHtml entrypoints will be deleted so future slices can port non-VMP behavior into Rust without carrying the VMP surface forward.

**Tech Stack:** TypeScript 4.9, Mocha/Chai, Yarn, Rust 1.96, Cargo workspace, `serde`, `serde_json`, and `napi-rs` for the Node binding.

---

## Scope Boundary

This is the first implementation slice for the approved full Rust rewrite. It must leave the repository in a buildable, testable state while moving the final state forward. It does not port every transformer. It removes the VMP surface and creates the Rust package boundary that later slices will fill with parser, option, transform, source map, and codegen logic.

## File Structure

- Create `Cargo.toml`: root Cargo workspace.
- Create `crates/javascript-obfuscator/Cargo.toml`: Rust engine crate manifest.
- Create `crates/javascript-obfuscator/src/lib.rs`: engine API scaffold.
- Create `crates/javascript-obfuscator/src/api.rs`: Rust `obfuscate`, `obfuscate_multiple`, and preset API.
- Create `crates/javascript-obfuscator/src/options.rs`: Rust option/result structs for this slice.
- Create `crates/javascript-obfuscator-node/Cargo.toml`: Node binding crate manifest.
- Create `crates/javascript-obfuscator-node/src/lib.rs`: napi-rs exports.
- Create `src/rust/RustObfuscatorBridge.ts`: TypeScript bridge for loading the native Rust binding.
- Create `test/functional-tests/rust-rewrite/RustBridge.spec.ts`: bridge smoke tests.
- Modify `index.ts`: remove Pro API exports.
- Modify `src/JavaScriptObfuscatorFacade.ts`: remove `obfuscatePro` and Pro exports.
- Modify `src/cli/JavaScriptObfuscatorCLI.ts`: remove Pro API imports, flags, and routing.
- Modify `src/enums/logger/LoggingMessage.ts`: remove Pro advertisement imports/messages.
- Modify `src/JavaScriptObfuscator.ts`: remove Pro advertisement logging.
- Modify `src/interfaces/options/ICLIOptions.ts`: remove Pro CLI fields.
- Modify `test/index.spec.ts`: remove Pro API test imports and add Rust bridge test import.
- Delete `src/pro-api/**`, `src/interfaces/pro-api/**`, `test/unit-tests/pro-api/**`, and `test/functional-tests/pro-api/**`.
- Modify `typings/**`: remove generated Pro API declarations after running the typings build.
- Modify `README.md`: remove Pro API, VM, VMP, `parseHtml`, and Pro CLI documentation.
- Modify `package.json`: add Rust build scripts and remove `@vercel/blob` after Pro API deletion.
- Modify `yarn.lock`: update after dependency removal.

## Task 1: Add Non-VMP Public API Regression Tests

**Files:**
- Create: `test/functional-tests/rust-rewrite/RustBridge.spec.ts`
- Modify: `test/index.spec.ts`

- [ ] **Step 1: Write the bridge/public API smoke tests**

Create `test/functional-tests/rust-rewrite/RustBridge.spec.ts` with:

```typescript
import { assert } from 'chai';

import { JavaScriptObfuscator } from '../../../src/JavaScriptObfuscatorFacade';

describe('Rust rewrite compatibility boundary', () => {
    describe('Variant #1: public facade remains available', () => {
        it('should obfuscate code through the existing facade', () => {
            const result = JavaScriptObfuscator.obfuscate('const value = 1; console.log(value);', {
                compact: true,
                stringArray: false,
                renameGlobals: false
            });

            assert.isString(result.getObfuscatedCode());
            assert.include(result.getObfuscatedCode(), 'console');
        });
    });

    describe('Variant #2: multiple source facade remains available', () => {
        it('should obfuscate multiple source files', () => {
            const results = JavaScriptObfuscator.obfuscateMultiple(
                {
                    'first.js': 'const first = 1;',
                    'second.js': 'const second = 2;'
                },
                {
                    compact: true,
                    stringArray: false,
                    renameGlobals: false
                }
            );

            assert.hasAllKeys(results, ['first.js', 'second.js']);
            assert.isString(results['first.js'].getObfuscatedCode());
            assert.isString(results['second.js'].getObfuscatedCode());
        });
    });

    describe('Variant #3: Pro API is not exposed', () => {
        it('should not expose obfuscatePro on the public facade', () => {
            assert.isUndefined((JavaScriptObfuscator as unknown as { obfuscatePro?: unknown }).obfuscatePro);
        });
    });
});
```

- [ ] **Step 2: Import the new test file**

Append this import to `test/index.spec.ts` near the other functional imports:

```typescript
import './functional-tests/rust-rewrite/RustBridge.spec';
```

- [ ] **Step 3: Run the new test and confirm it fails before Pro removal**

Run:

```bash
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts --grep "Pro API is not exposed"
```

Expected: FAIL because `JavaScriptObfuscator.obfuscatePro` is still defined.

- [ ] **Step 4: Commit the failing regression test**

Run:

```bash
git add test/functional-tests/rust-rewrite/RustBridge.spec.ts test/index.spec.ts
git commit -m "test: lock non-vmp public api boundary"
```

## Task 2: Remove Pro API Exports and Runtime Advertisement

**Files:**
- Modify: `index.ts`
- Modify: `src/JavaScriptObfuscatorFacade.ts`
- Modify: `src/JavaScriptObfuscator.ts`
- Modify: `src/enums/logger/LoggingMessage.ts`
- Modify: `src/utils/AdvertisementUtils.ts`

- [ ] **Step 1: Remove Pro exports from `index.ts`**

Edit `index.ts` so the imports and exports are:

```typescript
"use strict";

import { TDictionary } from './src/types/TDictionary';
import { TInputOptions } from './src/types/options/TInputOptions';
import { TObfuscationResultsObject } from './src/types/TObfuscationResultsObject';
import { TOptionsPreset } from './src/types/options/TOptionsPreset';

import { IObfuscationResult } from './src/interfaces/source-code/IObfuscationResult';
import { JavaScriptObfuscator } from './src/JavaScriptObfuscatorFacade';

export type ObfuscatorOptions = TInputOptions;

export interface ObfuscationResult extends IObfuscationResult {}

export declare function obfuscate(sourceCode: string, inputOptions?: ObfuscatorOptions): ObfuscationResult;

export declare function obfuscateMultiple<TSourceCodesObject extends TDictionary<string>>(
    sourceCodesObject: TSourceCodesObject,
    inputOptions?: TInputOptions
): TObfuscationResultsObject<TSourceCodesObject>;

export declare function getOptionsByPreset(optionsPreset: TOptionsPreset): TInputOptions;

export declare const version: string;

module.exports = JavaScriptObfuscator;
```

- [ ] **Step 2: Remove `obfuscatePro` from `src/JavaScriptObfuscatorFacade.ts`**

Remove these imports:

```typescript
import { IProApiConfig, IProObfuscationResult, TProApiProgressCallback } from './interfaces/pro-api/IProApiClient';
```

Remove the full `public static async obfuscatePro(...)` method.

Remove these exports at the bottom:

```typescript
export { ApiError } from './pro-api/ApiError';
export type { IProApiConfig, IProObfuscationResult, TProApiProgressCallback } from './interfaces/pro-api/IProApiClient';
```

Keep only:

```typescript
export { JavaScriptObfuscatorFacade as JavaScriptObfuscator };
```

- [ ] **Step 3: Remove Pro advertisement from `src/JavaScriptObfuscator.ts`**

Remove:

```typescript
import { AdvertisementUtils } from './utils/AdvertisementUtils';
```

Remove this block from `obfuscate`:

```typescript
if (AdvertisementUtils.shouldShowAdvertisement()) {
    this.logger.advertise(LoggingMessage.JavaScriptObfuscatorProAdFirstPart);
    this.logger.advertise(LoggingMessage.JavaScriptObfuscatorProAdSecondPart);
}
```

- [ ] **Step 4: Remove Pro logging enum members**

In `src/enums/logger/LoggingMessage.ts`, remove the `proAdvertiseMessageFirstPart` and `proAdvertiseMessageSecondPart` import and remove the enum members that reference them. Keep all non-Pro logging messages unchanged.

- [ ] **Step 5: Remove the advertisement utility file references**

Run:

```bash
rg -n "AdvertisementUtils|JavaScriptObfuscatorProAd|proAdvertise" src test index.ts
```

Expected: no output.

- [ ] **Step 6: Run targeted tests**

Run:

```bash
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
```

Expected: PASS.

- [ ] **Step 7: Commit Pro facade removal**

Run:

```bash
git add index.ts src/JavaScriptObfuscatorFacade.ts src/JavaScriptObfuscator.ts src/enums/logger/LoggingMessage.ts
git commit -m "refactor: remove pro api facade surface"
```

## Task 3: Remove Pro API and VM CLI Surface

**Files:**
- Modify: `src/cli/JavaScriptObfuscatorCLI.ts`
- Modify: `src/interfaces/options/ICLIOptions.ts`
- Modify: `test/functional-tests/cli/JavaScriptObfuscatorCLI.spec.ts`

- [ ] **Step 1: Remove Pro imports from CLI**

Remove these imports from `src/cli/JavaScriptObfuscatorCLI.ts`:

```typescript
import { ProApiClient } from '../pro-api/ProApiClient';
import { IProObfuscationResult } from '../interfaces/pro-api/IProApiClient';
import { VMTargetFunctionsMode } from '../pro-api/enums/VMTargetFunctionsMode';
import { VMBytecodeFormat } from '../pro-api/enums/VMBytecodeFormat';
```

- [ ] **Step 2: Remove Pro and VM option declarations**

Delete the `.option(...)` chain entries for:

```text
--pro-api-token
--pro-api-version
--vm-obfuscation
--vm-obfuscation-threshold
--vm-preprocess-identifiers
--vm-dynamic-opcodes
--vm-target-functions
--vm-exclude-functions
--vm-target-functions-mode
--vm-wrap-top-level-initializers
--vm-opcode-shuffle
--vm-bytecode-encoding
--vm-bytecode-array-encoding
--vm-bytecode-array-encoding-key
--vm-bytecode-array-encoding-key-getter
--vm-instruction-shuffle
--vm-jumps-encoding
--vm-decoy-opcodes
--vm-dead-code-injection
--vm-split-dispatcher
--vm-macro-ops
--vm-debug-protection
--vm-runtime-opcode-derivation
--vm-stateful-opcodes
--vm-stack-encoding
--vm-randomize-keys
--vm-indirect-dispatch
--vm-compact-dispatcher
--vm-bytecode-format
--parse-html
```

Keep `--strict-mode` and all non-Pro options.

- [ ] **Step 3: Remove Pro routing from `processSourceCode`**

Delete:

```typescript
const proApiToken = this.inputCLIOptions.proApiToken;

if (proApiToken) {
    await this.processSourceCodeWithProApi(sourceCode, outputCodePath, options, proApiToken);

    return;
}
```

- [ ] **Step 4: Delete `processSourceCodeWithProApi`**

Remove the complete private method named `processSourceCodeWithProApi`.

- [ ] **Step 5: Remove Pro fields from CLI options interface**

Edit `src/interfaces/options/ICLIOptions.ts` and remove:

```typescript
readonly proApiToken: string;
readonly proApiVersion: string;
```

- [ ] **Step 6: Remove Pro CLI tests**

In `test/functional-tests/cli/JavaScriptObfuscatorCLI.spec.ts`, remove the import of `ProApiClient` and remove the describe block for ``--pro-api-token`` plus the `ProApiClient.hasProFeatures` tests. Keep non-Pro CLI tests unchanged.

- [ ] **Step 7: Verify removed CLI flags are no longer accepted**

Run:

```bash
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/cli/JavaScriptObfuscatorCLI.spec.ts --grep "pro-api"
```

Expected: no matching tests are run.

Run:

```bash
npx eslint src/cli/JavaScriptObfuscatorCLI.ts src/interfaces/options/ICLIOptions.ts test/functional-tests/cli/JavaScriptObfuscatorCLI.spec.ts
```

Expected: PASS.

- [ ] **Step 8: Commit CLI removal**

Run:

```bash
git add src/cli/JavaScriptObfuscatorCLI.ts src/interfaces/options/ICLIOptions.ts test/functional-tests/cli/JavaScriptObfuscatorCLI.spec.ts
git commit -m "refactor: remove pro api cli options"
```

## Task 4: Delete Pro API Source and Tests

**Files:**
- Delete: `src/pro-api/**`
- Delete: `src/interfaces/pro-api/**`
- Delete: `test/unit-tests/pro-api/**`
- Delete: `test/functional-tests/pro-api/**`
- Modify: `test/index.spec.ts`

- [ ] **Step 1: Remove Pro test imports**

Delete these lines from `test/index.spec.ts`:

```typescript
import './unit-tests/pro-api/ProApiClient.spec';
import './functional-tests/pro-api/ProApiClient.spec';
```

- [ ] **Step 2: Delete Pro API directories**

Run:

```bash
git rm -r src/pro-api src/interfaces/pro-api test/unit-tests/pro-api test/functional-tests/pro-api
```

- [ ] **Step 3: Search for remaining Pro API references**

Run:

```bash
rg -n "pro-api|ProApi|IProApi|obfuscatePro|ApiError|vmObfuscation|parseHtml|VMBytecode|VMTarget" src test index.ts typings package.json
```

Expected: no output outside generated `typings` before the typings rebuild. If `typings` still contains matches, leave them for Task 8.

- [ ] **Step 4: Run non-Pro test index import check**

Run:

```bash
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
```

Expected: PASS.

- [ ] **Step 5: Commit deleted Pro code**

Run:

```bash
git add test/index.spec.ts
git rm -r src/pro-api src/interfaces/pro-api test/unit-tests/pro-api test/functional-tests/pro-api
git commit -m "refactor: delete pro api implementation"
```

## Task 5: Add Rust Workspace and Engine API Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `crates/javascript-obfuscator/Cargo.toml`
- Create: `crates/javascript-obfuscator/src/lib.rs`
- Create: `crates/javascript-obfuscator/src/api.rs`
- Create: `crates/javascript-obfuscator/src/options.rs`

- [ ] **Step 1: Add root Cargo workspace**

Create `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/javascript-obfuscator"
]

[workspace.package]
edition = "2021"
license = "BSD-2-Clause"
repository = "https://github.com/Jin1c-3/javascript-obfuscator"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Add engine crate manifest**

Create `crates/javascript-obfuscator/Cargo.toml`:

```toml
[package]
name = "javascript-obfuscator"
version = "5.4.3"
edition.workspace = true
license.workspace = true
repository.workspace = true

[lib]
name = "javascript_obfuscator"
path = "src/lib.rs"

[dependencies]
serde.workspace = true
serde_json.workspace = true
```

- [ ] **Step 3: Add engine module root**

Create `crates/javascript-obfuscator/src/lib.rs`:

```rust
pub mod api;
pub mod options;

pub use api::{get_options_by_preset, obfuscate, obfuscate_multiple};
pub use options::{IdentifierNamesCache, ObfuscationResult, Options, Preset};
```

- [ ] **Step 4: Add options and result structs**

Create `crates/javascript-obfuscator/src/options.rs`:

```rust
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub type IdentifierNamesCache = Map<String, Value>;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    #[serde(default)]
    pub compact: Option<bool>,
    #[serde(default)]
    pub string_array: Option<bool>,
    #[serde(default)]
    pub rename_globals: Option<bool>,
    #[serde(default)]
    pub source_map: Option<bool>,
    #[serde(default)]
    pub identifier_names_cache: Option<IdentifierNamesCache>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Preset {
    Default,
    LowObfuscation,
    MediumObfuscation,
    HighObfuscation,
}

impl Default for Preset {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObfuscationResult {
    pub code: String,
    pub source_map: String,
    pub identifier_names_cache: Option<IdentifierNamesCache>,
}

impl ObfuscationResult {
    pub fn new(code: String, source_map: String, identifier_names_cache: Option<IdentifierNamesCache>) -> Self {
        Self {
            code,
            source_map,
            identifier_names_cache,
        }
    }
}
```

- [ ] **Step 5: Add API scaffold**

Create `crates/javascript-obfuscator/src/api.rs`:

```rust
use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::options::{ObfuscationResult, Options, Preset};

pub fn obfuscate(source_code: &str, options: Options) -> Result<ObfuscationResult, String> {
    let source_map = if options.source_map.unwrap_or(false) {
        "{}".to_string()
    } else {
        String::new()
    };

    Ok(ObfuscationResult::new(
        source_code.to_string(),
        source_map,
        options.identifier_names_cache,
    ))
}

pub fn obfuscate_multiple(
    source_codes: BTreeMap<String, String>,
    options: Options,
) -> Result<BTreeMap<String, ObfuscationResult>, String> {
    source_codes
        .into_iter()
        .map(|(file_name, source_code)| {
            let result = obfuscate(&source_code, options.clone())?;
            Ok((file_name, result))
        })
        .collect()
}

pub fn get_options_by_preset(preset: Preset) -> Value {
    match preset {
        Preset::Default => json!({
            "compact": true,
            "stringArray": true,
            "renameGlobals": false
        }),
        Preset::LowObfuscation => json!({
            "compact": true,
            "stringArray": true,
            "renameGlobals": false
        }),
        Preset::MediumObfuscation => json!({
            "compact": true,
            "stringArray": true,
            "renameGlobals": false,
            "controlFlowFlattening": true
        }),
        Preset::HighObfuscation => json!({
            "compact": true,
            "stringArray": true,
            "renameGlobals": false,
            "controlFlowFlattening": true,
            "deadCodeInjection": true
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obfuscate_returns_source_code_for_scaffold() {
        let result = obfuscate("const value = 1;", Options::default()).expect("obfuscation should succeed");

        assert_eq!(result.code, "const value = 1;");
        assert_eq!(result.source_map, "");
    }

    #[test]
    fn obfuscate_multiple_preserves_keys() {
        let mut input = BTreeMap::new();
        input.insert("first.js".to_string(), "const first = 1;".to_string());
        input.insert("second.js".to_string(), "const second = 2;".to_string());

        let result = obfuscate_multiple(input, Options::default()).expect("obfuscation should succeed");

        assert!(result.contains_key("first.js"));
        assert!(result.contains_key("second.js"));
    }
}
```

- [ ] **Step 6: Format and test Rust engine crate**

Run:

```bash
cargo fmt --check
cargo test -p javascript-obfuscator
```

Expected: PASS.

- [ ] **Step 7: Commit Rust engine scaffold**

Run:

```bash
git add Cargo.toml crates/javascript-obfuscator
git commit -m "feat: add rust engine scaffold"
```

## Task 6: Add Node Binding Scaffold

**Files:**
- Create: `crates/javascript-obfuscator-node/Cargo.toml`
- Create: `crates/javascript-obfuscator-node/src/lib.rs`
- Modify: `Cargo.toml`
- Modify: `package.json`

- [ ] **Step 1: Add Node binding manifest**

Update root `Cargo.toml` so the workspace members are:

```toml
members = [
    "crates/javascript-obfuscator",
    "crates/javascript-obfuscator-node"
]
```

Create `crates/javascript-obfuscator-node/Cargo.toml`:

```toml
[package]
name = "javascript-obfuscator-node"
version = "5.4.3"
edition.workspace = true
license.workspace = true
repository.workspace = true

[lib]
crate-type = ["cdylib"]
path = "src/lib.rs"

[dependencies]
javascript-obfuscator = { path = "../javascript-obfuscator" }
napi = { version = "2", default-features = false, features = ["napi4", "serde-json"] }
napi-derive = "2"
serde_json.workspace = true
```

- [ ] **Step 2: Add binding exports**

Create `crates/javascript-obfuscator-node/src/lib.rs`:

```rust
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value;

#[napi]
pub fn obfuscate(source_code: String, options: Option<Value>) -> Result<Value> {
    let options = options
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| Error::from_reason(error.to_string()))?
        .unwrap_or_default();

    let result = javascript_obfuscator::obfuscate(&source_code, options).map_err(Error::from_reason)?;

    serde_json::to_value(result).map_err(|error| Error::from_reason(error.to_string()))
}

#[napi]
pub fn get_options_by_preset(preset: String) -> Result<Value> {
    let preset = match preset.as_str() {
        "default" => javascript_obfuscator::Preset::Default,
        "low-obfuscation" => javascript_obfuscator::Preset::LowObfuscation,
        "medium-obfuscation" => javascript_obfuscator::Preset::MediumObfuscation,
        "high-obfuscation" => javascript_obfuscator::Preset::HighObfuscation,
        value => return Err(Error::from_reason(format!("Unknown options preset: {value}"))),
    };

    Ok(javascript_obfuscator::get_options_by_preset(preset))
}
```

- [ ] **Step 3: Add npm scripts for Rust build checks**

Add these scripts to `package.json`:

```json
"rust:fmt": "cargo fmt --check",
"rust:test": "cargo test --workspace",
"rust:clippy": "cargo clippy --workspace --all-targets -- -D warnings"
```

- [ ] **Step 4: Install binding dependencies through Cargo**

Run:

```bash
cargo test -p javascript-obfuscator-node
```

Expected: Cargo downloads `napi` dependencies and the binding crate compiles.

- [ ] **Step 5: Commit Node binding scaffold**

Run:

```bash
git add Cargo.toml crates/javascript-obfuscator-node package.json Cargo.lock
git commit -m "feat: add rust node binding scaffold"
```

## Task 7: Add TypeScript Rust Bridge

**Files:**
- Create: `src/rust/RustObfuscatorBridge.ts`
- Modify: `src/JavaScriptObfuscatorFacade.ts`

- [ ] **Step 1: Add TypeScript bridge file**

Create `src/rust/RustObfuscatorBridge.ts`:

```typescript
import { TInputOptions } from '../types/options/TInputOptions';

interface IRustObfuscationPayload {
    readonly code: string;
    readonly sourceMap: string;
    readonly identifierNamesCache?: unknown;
}

export class RustObfuscatorBridge {
    public static isAvailable(): boolean {
        return false;
    }

    public static obfuscate(_sourceCode: string, _inputOptions: TInputOptions): IRustObfuscationPayload | null {
        return null;
    }

    public static getOptionsByPreset(_optionsPreset: string): TInputOptions | null {
        return null;
    }
}
```

- [ ] **Step 2: Keep facade on TypeScript engine for this slice**

Do not change `JavaScriptObfuscatorFacade.obfuscate` to call the bridge in this slice. The bridge exists so later slices can switch behavior behind tests after Rust output parity improves.

- [ ] **Step 3: Add bridge unit check to RustBridge test**

Append this describe block to `test/functional-tests/rust-rewrite/RustBridge.spec.ts`:

```typescript
describe('Variant #4: Rust bridge scaffold', () => {
    it('should expose a disabled bridge until Rust parity is ready', () => {
        const { RustObfuscatorBridge } = require('../../../src/rust/RustObfuscatorBridge');

        assert.isFalse(RustObfuscatorBridge.isAvailable());
        assert.isNull(RustObfuscatorBridge.obfuscate('const value = 1;', {}));
    });
});
```

- [ ] **Step 4: Run bridge tests**

Run:

```bash
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
```

Expected: PASS.

- [ ] **Step 5: Commit TypeScript bridge**

Run:

```bash
git add src/rust/RustObfuscatorBridge.ts test/functional-tests/rust-rewrite/RustBridge.spec.ts
git commit -m "feat: add rust bridge scaffold"
```

## Task 8: Update Typings, README, and Dependencies

**Files:**
- Modify: `README.md`
- Modify: `typings/**`
- Modify: `package.json`
- Modify: `yarn.lock`

- [ ] **Step 1: Remove Pro dependency**

Remove this dependency from `package.json`:

```json
"@vercel/blob": ">=0.23.0"
```

Run:

```bash
yarn install --frozen-lockfile
```

Expected: FAIL because `yarn.lock` still contains the removed dependency.

Run:

```bash
yarn install
```

Expected: PASS and `yarn.lock` updates.

- [ ] **Step 2: Regenerate typings**

Run:

```bash
yarn run build:typings
```

Expected: PASS and Pro API declarations are removed from `typings`.

- [ ] **Step 3: Remove Pro documentation**

Edit `README.md` to remove sections and references for:

```text
Obfuscator.io with VM Obfuscation
Pro API Methods
obfuscatePro
--pro-api-token
--pro-api-version
--vm-*
parseHtml
VM obfuscation option descriptions
```

Keep non-Pro install, API, CLI, and option documentation.

- [ ] **Step 4: Verify no VMP/Pro references remain**

Run:

```bash
rg -n "Pro API|pro-api|ProApi|obfuscatePro|vmObfuscation|parseHtml|--vm-|--pro-api|VMP|VM Obfuscation" README.md src test typings index.ts package.json
```

Expected: no output except unrelated fixture comments that refer to JavaScript runtime VM turns.

- [ ] **Step 5: Run docs and typings checks**

Run:

```bash
npx eslint src/**/*.ts
yarn run prettier:check
```

Expected: PASS.

- [ ] **Step 6: Commit docs and dependency update**

Run:

```bash
git add README.md typings package.json yarn.lock
git commit -m "docs: remove pro api and vm documentation"
```

## Task 9: Slice Verification

**Files:**
- No new files.

- [ ] **Step 1: Run Rust verification**

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: PASS.

- [ ] **Step 2: Run TypeScript lint**

Run:

```bash
yarn run eslint
```

Expected: PASS.

- [ ] **Step 3: Run non-Pro Mocha tests**

Run:

```bash
yarn run test:mocha
```

Expected: PASS with Pro API tests absent from `test/index.spec.ts`.

- [ ] **Step 4: Run CLI smoke test**

Create `C:\tmp\javascript-obfuscator-smoke-input.js` with:

```javascript
const message = 'hello';
console.log(message);
```

Run:

```bash
node bin/javascript-obfuscator C:\tmp\javascript-obfuscator-smoke-input.js --output C:\tmp\javascript-obfuscator-smoke-output.js --compact true --string-array false
```

Expected: `C:\tmp\javascript-obfuscator-smoke-output.js` exists and contains JavaScript output.

- [ ] **Step 5: Confirm branch status**

Run:

```bash
git status --short --branch
```

Expected: only pre-existing untracked `AGENTS.md` and `scripts/` remain.

- [ ] **Step 6: Commit verification notes only if files changed**

If verification changes generated typings after Task 8, stage the typings directory and commit:

```bash
git add typings
git commit -m "chore: refresh generated artifacts after pro removal"
```

If no files changed, do not create a commit.

## Next Plans After This Slice

After this plan passes, write the next implementation plan for the Rust parser, option normalization, codegen, and result assembly slice. Subsequent plans should port transformer families in this order: identifier generation and cache, literal/converting transforms, string array transforms, rename identifiers and scope analysis, rename properties, control flow flattening, dead code injection, and runtime helper protections.

## Self-Review

Spec coverage for this slice:

- VMP/Pro removal is covered by Tasks 2, 3, 4, and 8.
- Rust workspace and Node binding scaffold are covered by Tasks 5, 6, and 7.
- Compatibility tests are covered by Tasks 1 and 9.
- Full transformer porting is intentionally assigned to later slice plans because the approved design is a multi-subsystem rewrite.

Placeholder scan:

- This plan contains no placeholder tokens and no conflict markers.

Type consistency:

- Rust `Options`, `Preset`, and `ObfuscationResult` are defined before the Rust API and binding use them.
- TypeScript `RustObfuscatorBridge` return shapes match the scaffold test.
