# Rust Identifier Generation Cache Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rust identifier-name generator primitives and typed identifier-name cache handling that later rename, property, string-array, and helper transforms can consume.

**Architecture:** The Rust core gains `generators` and `storages` module roots. Identifier generation is deterministic and option-driven, while identifier-name cache storage normalizes the public `globalIdentifiers` and `propertyIdentifiers` JSON shape without changing JavaScript output yet.

**Tech Stack:** Rust 1.96, serde, serde_json, SWC-backed parser/codegen from the previous slice, napi-rs boundary unchanged.

---

## Scope Boundary

This slice does not rename AST identifiers. It ports the identifier-name generator and cache substrate only. The TypeScript package facade remains on the existing engine, and Rust output remains a no-transform parse/codegen path.

## File Structure

- Modify `crates/javascript-obfuscator/src/lib.rs`: export new `generators` and `storages` modules.
- Modify `crates/javascript-obfuscator/src/options.rs`: add identifier generator, prefix, and dictionary option fields.
- Create `crates/javascript-obfuscator/src/generators/mod.rs`: module root and re-exports.
- Create `crates/javascript-obfuscator/src/generators/identifier_names.rs`: deterministic hexadecimal, mangled, mangled-shuffled, and dictionary name generation.
- Create `crates/javascript-obfuscator/src/storages/mod.rs`: module root and re-exports.
- Create `crates/javascript-obfuscator/src/storages/identifier_names_cache.rs`: normalized `globalIdentifiers` and `propertyIdentifiers` cache storage.
- Modify `crates/javascript-obfuscator/src/pipeline.rs`: normalize identifier-name cache before assembling `ObfuscationResult`.
- Modify `crates/javascript-obfuscator/src/api.rs`: add cache contract tests and thread updated cache through `obfuscate_multiple`.

## Task 1: Add Rust Identifier Contracts

**Files:**
- Create: `crates/javascript-obfuscator/src/generators/mod.rs`
- Create: `crates/javascript-obfuscator/src/generators/identifier_names.rs`
- Create: `crates/javascript-obfuscator/src/storages/mod.rs`
- Create: `crates/javascript-obfuscator/src/storages/identifier_names_cache.rs`
- Modify: `crates/javascript-obfuscator/src/lib.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Export module roots**

In `crates/javascript-obfuscator/src/lib.rs`, add:

```rust
pub mod generators;
pub mod storages;
```

Keep existing public re-exports unchanged in this step.

- [ ] **Step 2: Add generator module root**

Create `crates/javascript-obfuscator/src/generators/mod.rs`:

```rust
pub mod identifier_names;

pub use identifier_names::{IdentifierNamesGenerator, IdentifierNamesGeneratorKind};
```

- [ ] **Step 3: Add failing identifier generator tests with minimal stubs**

Create `crates/javascript-obfuscator/src/generators/identifier_names.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdentifierNamesGeneratorKind {
    #[default]
    Hexadecimal,
    Mangled,
    MangledShuffled,
    Dictionary,
}

pub struct IdentifierNamesGenerator;

impl IdentifierNamesGenerator {
    pub fn new(
        _kind: IdentifierNamesGeneratorKind,
        _prefix: impl Into<String>,
        _dictionary: Vec<String>,
    ) -> Self {
        Self
    }

    pub fn generate_next(&mut self) -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexadecimal_generator_uses_prefix_and_counter() {
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::Hexadecimal,
            "file_",
            Vec::new(),
        );

        assert_eq!(generator.generate_next(), "file__0x0");
        assert_eq!(generator.generate_next(), "file__0x1");
        assert_eq!(generator.generate_next(), "file__0x2");
    }

    #[test]
    fn mangled_generator_uses_short_identifier_sequence() {
        let mut generator =
            IdentifierNamesGenerator::new(IdentifierNamesGeneratorKind::Mangled, "", Vec::new());

        assert_eq!(generator.generate_next(), "a");
        assert_eq!(generator.generate_next(), "b");
        assert_eq!(generator.generate_next(), "c");
    }

    #[test]
    fn mangled_shuffled_generator_is_deterministic() {
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::MangledShuffled,
            "",
            Vec::new(),
        );

        assert_eq!(generator.generate_next(), "_");
        assert_eq!(generator.generate_next(), "$");
        assert_eq!(generator.generate_next(), "Z");
    }

    #[test]
    fn dictionary_generator_sanitizes_invalid_names() {
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::Dictionary,
            "p_",
            vec!["first-name".to_string(), "2cool".to_string()],
        );

        assert_eq!(generator.generate_next(), "p_first_name");
        assert_eq!(generator.generate_next(), "p__2cool");
    }
}
```

- [ ] **Step 4: Add storage module root**

Create `crates/javascript-obfuscator/src/storages/mod.rs`:

```rust
pub mod identifier_names_cache;

pub use identifier_names_cache::{
    normalize_identifier_names_cache, IdentifierNamesCacheStorage,
};
```

- [ ] **Step 5: Add failing cache storage tests with minimal stubs**

Create `crates/javascript-obfuscator/src/storages/identifier_names_cache.rs`:

```rust
use serde_json::{Map, Value};

use crate::generators::IdentifierNamesGenerator;
use crate::options::IdentifierNamesCache;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IdentifierNamesCacheStorage {
    global_identifiers: Map<String, Value>,
    property_identifiers: Map<String, Value>,
}

impl IdentifierNamesCacheStorage {
    pub fn from_cache(_cache: IdentifierNamesCache) -> Self {
        Self::default()
    }

    pub fn into_cache(self) -> IdentifierNamesCache {
        Map::new()
    }

    pub fn resolve_or_insert_global(
        &mut self,
        _original_name: &str,
        _generator: &mut IdentifierNamesGenerator,
    ) -> String {
        String::new()
    }
}

pub fn normalize_identifier_names_cache(
    cache: Option<IdentifierNamesCache>,
) -> Option<IdentifierNamesCache> {
    cache
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::generators::IdentifierNamesGeneratorKind;

    #[test]
    fn normalizes_empty_cache_to_public_shape() {
        let normalized =
            normalize_identifier_names_cache(Some(Map::new())).expect("cache should remain enabled");

        assert_eq!(normalized.get("globalIdentifiers"), Some(&json!({})));
        assert_eq!(normalized.get("propertyIdentifiers"), Some(&json!({})));
    }

    #[test]
    fn preserves_existing_global_and_property_mappings() {
        let mut input = Map::new();
        input.insert("globalIdentifiers".to_string(), json!({ "alpha": "_0x1" }));
        input.insert("propertyIdentifiers".to_string(), json!({ "beta": "_0x2" }));

        let normalized = normalize_identifier_names_cache(Some(input)).expect("cache enabled");

        assert_eq!(normalized["globalIdentifiers"]["alpha"], "_0x1");
        assert_eq!(normalized["propertyIdentifiers"]["beta"], "_0x2");
    }

    #[test]
    fn resolves_existing_global_mapping_before_generating() {
        let mut input = Map::new();
        input.insert("globalIdentifiers".to_string(), json!({ "alpha": "_0x9" }));
        let mut storage = IdentifierNamesCacheStorage::from_cache(input);
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::Hexadecimal,
            "",
            Vec::new(),
        );

        assert_eq!(
            storage.resolve_or_insert_global("alpha", &mut generator),
            "_0x9"
        );
        assert_eq!(
            storage.resolve_or_insert_global("beta", &mut generator),
            "_0x0"
        );

        let output = storage.into_cache();
        assert_eq!(output["globalIdentifiers"]["alpha"], "_0x9");
        assert_eq!(output["globalIdentifiers"]["beta"], "_0x0");
    }
}
```

- [ ] **Step 6: Add failing public API cache tests**

Append these tests inside the existing `#[cfg(test)] mod tests` in `crates/javascript-obfuscator/src/api.rs`:

```rust
#[test]
fn obfuscate_normalizes_identifier_names_cache() {
    let result = obfuscate(
        "const value = 1;",
        Options {
            identifier_names_cache: Some(serde_json::Map::new()),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    let cache = result
        .identifier_names_cache
        .expect("cache should be returned when option is provided");

    assert_eq!(cache.get("globalIdentifiers"), Some(&json!({})));
    assert_eq!(cache.get("propertyIdentifiers"), Some(&json!({})));
}

#[test]
fn obfuscate_multiple_preserves_normalized_identifier_cache() {
    let mut input = BTreeMap::new();
    input.insert("first.js".to_string(), "const first = 1;".to_string());
    input.insert("second.js".to_string(), "const second = 2;".to_string());

    let result = obfuscate_multiple(
        input,
        Options {
            identifier_names_cache: Some(serde_json::Map::new()),
            ..Options::default()
        },
    )
    .expect("obfuscation should succeed");

    for obfuscation_result in result.values() {
        let cache = obfuscation_result
            .identifier_names_cache
            .as_ref()
            .expect("cache should be returned for every file");

        assert_eq!(cache.get("globalIdentifiers"), Some(&json!({})));
        assert_eq!(cache.get("propertyIdentifiers"), Some(&json!({})));
    }
}
```

- [ ] **Step 7: Run targeted tests and confirm failure**

Run:

```bash
cargo test -p javascript-obfuscator identifier_names
cargo test -p javascript-obfuscator obfuscate_normalizes_identifier_names_cache
```

Expected: FAIL because the generator returns empty strings and the cache normalizer returns the input unchanged.

## Task 2: Implement Identifier Generator Options and Logic

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/generators/identifier_names.rs`

- [ ] **Step 1: Add identifier option fields**

In `crates/javascript-obfuscator/src/options.rs`, add this import:

```rust
use crate::generators::IdentifierNamesGeneratorKind;
```

Add these fields to `Options`:

```rust
#[serde(default)]
pub identifier_names_generator: Option<IdentifierNamesGeneratorKind>,
#[serde(default)]
pub identifiers_prefix: Option<String>,
#[serde(default)]
pub identifiers_dictionary: Option<Vec<String>>,
```

- [ ] **Step 2: Replace the generator stub with deterministic implementation**

Replace `crates/javascript-obfuscator/src/generators/identifier_names.rs` with:

```rust
use serde::{Deserialize, Serialize};

const MANGLED_FIRST_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_";
const MANGLED_NEXT_CHARS: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_0123456789";
const SHUFFLED_FIRST_CHARS: &[u8] = b"_$ZYXWVUTSRQPONMLKJIHGFEDCBAzyxwvutsrqponmlkjihgfedcba";
const RESERVED_WORDS: &[&str] = &[
    "break", "case", "catch", "class", "const", "continue", "debugger", "default", "delete",
    "do", "else", "export", "extends", "finally", "for", "function", "if", "import", "in",
    "instanceof", "new", "return", "super", "switch", "this", "throw", "try", "typeof", "var",
    "void", "while", "with", "yield", "let", "static", "enum", "await", "implements",
    "package", "protected", "interface", "private", "public",
];

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdentifierNamesGeneratorKind {
    #[default]
    Hexadecimal,
    Mangled,
    MangledShuffled,
    Dictionary,
}

#[derive(Clone, Debug)]
pub struct IdentifierNamesGenerator {
    kind: IdentifierNamesGeneratorKind,
    prefix: String,
    dictionary: Vec<String>,
    index: usize,
}

impl IdentifierNamesGenerator {
    pub fn new(
        kind: IdentifierNamesGeneratorKind,
        prefix: impl Into<String>,
        dictionary: Vec<String>,
    ) -> Self {
        Self {
            kind,
            prefix: prefix.into(),
            dictionary,
            index: 0,
        }
    }

    pub fn generate_next(&mut self) -> String {
        loop {
            let raw_name = match self.kind {
                IdentifierNamesGeneratorKind::Hexadecimal => format!("_0x{:x}", self.index),
                IdentifierNamesGeneratorKind::Mangled => encode_name(
                    self.index,
                    MANGLED_FIRST_CHARS,
                    MANGLED_NEXT_CHARS,
                ),
                IdentifierNamesGeneratorKind::MangledShuffled => encode_name(
                    self.index,
                    SHUFFLED_FIRST_CHARS,
                    MANGLED_NEXT_CHARS,
                ),
                IdentifierNamesGeneratorKind::Dictionary => self.generate_dictionary_name(),
            };
            self.index += 1;

            let candidate = format!("{}{}", self.prefix, raw_name);

            if is_valid_identifier(&candidate) && !is_reserved_word(&candidate) {
                return candidate;
            }
        }
    }

    fn generate_dictionary_name(&self) -> String {
        if self.dictionary.is_empty() {
            return encode_name(self.index, MANGLED_FIRST_CHARS, MANGLED_NEXT_CHARS);
        }

        let dictionary_index = self.index % self.dictionary.len();
        let cycle = self.index / self.dictionary.len();
        let mut name = sanitize_identifier_fragment(&self.dictionary[dictionary_index]);

        if cycle > 0 {
            name.push_str(&cycle.to_string());
        }

        name
    }
}

fn encode_name(index: usize, first_chars: &[u8], next_chars: &[u8]) -> String {
    let mut value = index;
    let mut name = String::new();
    name.push(first_chars[value % first_chars.len()] as char);
    value /= first_chars.len();

    while value > 0 {
        value -= 1;
        name.push(next_chars[value % next_chars.len()] as char);
        value /= next_chars.len();
    }

    name
}

fn sanitize_identifier_fragment(value: &str) -> String {
    let mut output = String::new();

    for (index, character) in value.chars().enumerate() {
        let valid = if index == 0 {
            is_identifier_start(character)
        } else {
            is_identifier_part(character)
        };

        if valid {
            output.push(character);
        } else {
            output.push('_');
        }
    }

    if output.is_empty() {
        return "_".to_string();
    }

    if output
        .chars()
        .next()
        .map(is_identifier_start)
        .unwrap_or(false)
    {
        output
    } else {
        format!("_{output}")
    }
}

fn is_valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    is_identifier_start(first) && chars.all(is_identifier_part)
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character == '$' || character.is_ascii_alphabetic()
}

fn is_identifier_part(character: char) -> bool {
    is_identifier_start(character) || character.is_ascii_digit()
}

fn is_reserved_word(value: &str) -> bool {
    RESERVED_WORDS.contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexadecimal_generator_uses_prefix_and_counter() {
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::Hexadecimal,
            "file_",
            Vec::new(),
        );

        assert_eq!(generator.generate_next(), "file__0x0");
        assert_eq!(generator.generate_next(), "file__0x1");
        assert_eq!(generator.generate_next(), "file__0x2");
    }

    #[test]
    fn mangled_generator_uses_short_identifier_sequence() {
        let mut generator =
            IdentifierNamesGenerator::new(IdentifierNamesGeneratorKind::Mangled, "", Vec::new());

        assert_eq!(generator.generate_next(), "a");
        assert_eq!(generator.generate_next(), "b");
        assert_eq!(generator.generate_next(), "c");
    }

    #[test]
    fn mangled_shuffled_generator_is_deterministic() {
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::MangledShuffled,
            "",
            Vec::new(),
        );

        assert_eq!(generator.generate_next(), "_");
        assert_eq!(generator.generate_next(), "$");
        assert_eq!(generator.generate_next(), "Z");
    }

    #[test]
    fn dictionary_generator_sanitizes_invalid_names() {
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::Dictionary,
            "p_",
            vec!["first-name".to_string(), "2cool".to_string()],
        );

        assert_eq!(generator.generate_next(), "p_first_name");
        assert_eq!(generator.generate_next(), "p__2cool");
    }
}
```

- [ ] **Step 3: Run generator tests**

Run:

```bash
cargo test -p javascript-obfuscator identifier_names
```

Expected: PASS.

## Task 3: Implement Identifier Cache Storage and Pipeline Wiring

**Files:**
- Modify: `crates/javascript-obfuscator/src/storages/identifier_names_cache.rs`
- Modify: `crates/javascript-obfuscator/src/pipeline.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`

- [ ] **Step 1: Replace cache storage stub**

Replace `crates/javascript-obfuscator/src/storages/identifier_names_cache.rs` with:

```rust
use serde_json::{Map, Value};

use crate::generators::IdentifierNamesGenerator;
use crate::options::IdentifierNamesCache;

const GLOBAL_IDENTIFIERS: &str = "globalIdentifiers";
const PROPERTY_IDENTIFIERS: &str = "propertyIdentifiers";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IdentifierNamesCacheStorage {
    global_identifiers: Map<String, Value>,
    property_identifiers: Map<String, Value>,
}

impl IdentifierNamesCacheStorage {
    pub fn from_cache(cache: IdentifierNamesCache) -> Self {
        Self {
            global_identifiers: read_section(&cache, GLOBAL_IDENTIFIERS),
            property_identifiers: read_section(&cache, PROPERTY_IDENTIFIERS),
        }
    }

    pub fn into_cache(self) -> IdentifierNamesCache {
        let mut cache = Map::new();
        cache.insert(
            GLOBAL_IDENTIFIERS.to_string(),
            Value::Object(self.global_identifiers),
        );
        cache.insert(
            PROPERTY_IDENTIFIERS.to_string(),
            Value::Object(self.property_identifiers),
        );
        cache
    }

    pub fn resolve_or_insert_global(
        &mut self,
        original_name: &str,
        generator: &mut IdentifierNamesGenerator,
    ) -> String {
        if let Some(existing_name) = self
            .global_identifiers
            .get(original_name)
            .and_then(Value::as_str)
        {
            return existing_name.to_string();
        }

        let generated_name = generator.generate_next();
        self.global_identifiers.insert(
            original_name.to_string(),
            Value::String(generated_name.clone()),
        );
        generated_name
    }
}

pub fn normalize_identifier_names_cache(
    cache: Option<IdentifierNamesCache>,
) -> Option<IdentifierNamesCache> {
    cache.map(|cache| IdentifierNamesCacheStorage::from_cache(cache).into_cache())
}

fn read_section(cache: &IdentifierNamesCache, section_name: &str) -> Map<String, Value> {
    cache
        .get(section_name)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::generators::IdentifierNamesGeneratorKind;

    #[test]
    fn normalizes_empty_cache_to_public_shape() {
        let normalized =
            normalize_identifier_names_cache(Some(Map::new())).expect("cache should remain enabled");

        assert_eq!(normalized.get("globalIdentifiers"), Some(&json!({})));
        assert_eq!(normalized.get("propertyIdentifiers"), Some(&json!({})));
    }

    #[test]
    fn preserves_existing_global_and_property_mappings() {
        let mut input = Map::new();
        input.insert("globalIdentifiers".to_string(), json!({ "alpha": "_0x1" }));
        input.insert("propertyIdentifiers".to_string(), json!({ "beta": "_0x2" }));

        let normalized = normalize_identifier_names_cache(Some(input)).expect("cache enabled");

        assert_eq!(normalized["globalIdentifiers"]["alpha"], "_0x1");
        assert_eq!(normalized["propertyIdentifiers"]["beta"], "_0x2");
    }

    #[test]
    fn resolves_existing_global_mapping_before_generating() {
        let mut input = Map::new();
        input.insert("globalIdentifiers".to_string(), json!({ "alpha": "_0x9" }));
        let mut storage = IdentifierNamesCacheStorage::from_cache(input);
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::Hexadecimal,
            "",
            Vec::new(),
        );

        assert_eq!(
            storage.resolve_or_insert_global("alpha", &mut generator),
            "_0x9"
        );
        assert_eq!(
            storage.resolve_or_insert_global("beta", &mut generator),
            "_0x0"
        );

        let output = storage.into_cache();
        assert_eq!(output["globalIdentifiers"]["alpha"], "_0x9");
        assert_eq!(output["globalIdentifiers"]["beta"], "_0x0");
    }
}
```

- [ ] **Step 2: Normalize cache in the pipeline**

In `crates/javascript-obfuscator/src/pipeline.rs`, add this import:

```rust
use crate::storages::normalize_identifier_names_cache;
```

Inside `run_pipeline`, before `Ok(ObfuscationResult::new(...))`, compute:

```rust
let identifier_names_cache = normalize_identifier_names_cache(options.identifier_names_cache);
```

Pass `identifier_names_cache` to `ObfuscationResult::new`.

- [ ] **Step 3: Thread updated cache through `obfuscate_multiple`**

Replace `obfuscate_multiple` in `crates/javascript-obfuscator/src/api.rs` with:

```rust
pub fn obfuscate_multiple(
    source_codes: BTreeMap<String, String>,
    options: Options,
) -> ObfuscatorResult<BTreeMap<String, ObfuscationResult>> {
    let mut output = BTreeMap::new();
    let mut next_options = options;

    for (file_name, source_code) in source_codes {
        let result = obfuscate(&source_code, next_options.clone())?;

        if result.identifier_names_cache.is_some() {
            next_options.identifier_names_cache = result.identifier_names_cache.clone();
        }

        output.insert(file_name, result);
    }

    Ok(output)
}
```

- [ ] **Step 4: Run cache and API tests**

Run:

```bash
cargo test -p javascript-obfuscator identifier_names_cache
cargo test -p javascript-obfuscator obfuscate_normalizes_identifier_names_cache
cargo test -p javascript-obfuscator obfuscate_multiple_preserves_normalized_identifier_cache
```

Expected: PASS.

- [ ] **Step 5: Commit identifier substrate**

Run:

```bash
git add crates/javascript-obfuscator
git commit -m "feat: add rust identifier cache substrate"
```

## Task 4: Slice Verification and Push

**Files:**
- No extra source files unless verification surfaces compile or formatting changes.

- [ ] **Step 1: Run Rust verification**

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: PASS. The napi test binary may print Node-API host-runtime load warnings while still exiting 0.

- [ ] **Step 2: Run TypeScript boundary verification**

Run:

```bash
npx eslint src/**/*.ts
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
```

Expected: PASS.

- [ ] **Step 3: Run Pro/VMP removal scan**

Run:

```bash
rg -n "Pro API|pro-api|ProApi|obfuscatePro|vmObfuscation|parseHtml|--vm-|--pro-api|VMP|VM Obfuscation|@vercel/blob" README.md src test typings index.ts package.json
```

Expected: no output.

- [ ] **Step 4: Confirm branch status**

Run:

```bash
git status --short --branch
```

Expected: only pre-existing untracked `AGENTS.md` and `scripts/` remain.

- [ ] **Step 5: Push the branch**

Run:

```bash
git push
```

Expected: branch updates `origin/codex/rust-rewrite-slice-1`.

## Self-Review

Spec coverage:

- This plan implements migration slice 6 from the approved design: identifier generation and identifier-name cache handling.
- It does not port rename transforms, property rename transforms, string-array helpers, or facade switching; those remain separate transformer slices in the design order.
- It keeps VMP/Pro removal intact and includes the removal scan in verification.

Placeholder scan:

- The plan has no unfinished markers or vague implementation instructions.

Type consistency:

- `IdentifierNamesGeneratorKind` is defined before `Options` imports it.
- `IdentifierNamesCacheStorage` accepts and returns the existing `IdentifierNamesCache` alias.
- `normalize_identifier_names_cache` is the only pipeline-facing storage function in this slice.
