# Rust Parser Codegen Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Rust engine echo scaffold with a real parse-and-generate pipeline for no-transform JavaScript while keeping the TypeScript engine as the package compatibility fallback.

**Architecture:** The Rust library adds diagnostics, parser, codegen, and pipeline modules. SWC parses JavaScript into Rust AST, codegen emits JavaScript, and the public Rust API assembles `ObfuscationResult` with hashbang preservation and minimal source map metadata. Node binding shape remains unchanged.

**Tech Stack:** Rust 1.96, Cargo workspace, `swc_common`, `swc_ecma_ast`, `swc_ecma_parser`, `swc_ecma_codegen`, `serde`, `serde_json`, napi-rs, Mocha/Chai for package boundary smoke tests.

---

## Scope Boundary

This slice does not port obfuscation transformers. It moves the Rust core from source echoing to a real parser/codegen result path, which is required before transformer groups can be ported. The TypeScript facade must not switch to Rust in this slice because Rust output parity is still limited to no-transform scenarios.

## File Structure

- Modify `Cargo.toml`: add SWC crates to workspace dependencies.
- Modify `crates/javascript-obfuscator/Cargo.toml`: add SWC dependencies from the workspace.
- Modify `crates/javascript-obfuscator/src/lib.rs`: export new modules.
- Create `crates/javascript-obfuscator/src/diagnostics.rs`: structured Rust error type and result alias.
- Create `crates/javascript-obfuscator/src/parser.rs`: SWC parser adapter with script/module fallback.
- Create `crates/javascript-obfuscator/src/codegen.rs`: SWC code generator adapter.
- Create `crates/javascript-obfuscator/src/pipeline.rs`: hashbang preprocessing and no-transform result assembly.
- Modify `crates/javascript-obfuscator/src/api.rs`: call the pipeline from `obfuscate`.
- Modify `crates/javascript-obfuscator/src/options.rs`: expand options needed by parser/codegen tests.
- Modify `crates/javascript-obfuscator-node/src/lib.rs`: map structured diagnostics to napi `Error`.
- Modify `test/functional-tests/rust-rewrite/RustBridge.spec.ts`: add package-side boundary tests for the still-disabled bridge and no accidental facade switch.

## Task 1: Add Rust Parser/Codegen Contract Tests

**Files:**
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Create: `crates/javascript-obfuscator/src/pipeline.rs`
- Create: `crates/javascript-obfuscator/src/parser.rs`
- Create: `crates/javascript-obfuscator/src/codegen.rs`
- Create: `crates/javascript-obfuscator/src/diagnostics.rs`

- [ ] **Step 1: Add failing API tests for generated Rust output**

In `crates/javascript-obfuscator/src/api.rs`, replace the existing scaffold tests with tests that require parser/codegen behavior:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obfuscate_generates_code_from_parsed_ast() {
        let result = obfuscate(
            "const value = 1; console.log(value);",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value=1"));
        assert!(result.code.contains("console.log(value)"));
        assert_eq!(result.source_map, "");
    }

    #[test]
    fn obfuscate_preserves_hashbang() {
        let result = obfuscate(
            "#!/usr/bin/env node\nconst value = 1;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.starts_with("#!/usr/bin/env node\n"));
        assert!(result.code.contains("const value=1"));
    }

    #[test]
    fn obfuscate_reports_parse_errors() {
        let error = obfuscate("const =", Options::default()).expect_err("parse should fail");

        assert!(error.to_string().contains("JavaScript parse error"));
    }

    #[test]
    fn obfuscate_multiple_preserves_keys() {
        let mut input = BTreeMap::new();
        input.insert("first.js".to_string(), "const first = 1;".to_string());
        input.insert("second.js".to_string(), "const second = 2;".to_string());

        let result =
            obfuscate_multiple(input, Options::default()).expect("obfuscation should succeed");

        assert!(result.contains_key("first.js"));
        assert!(result.contains_key("second.js"));
        assert!(result["first.js"].code.contains("first"));
        assert!(result["second.js"].code.contains("second"));
    }
}
```

- [ ] **Step 2: Add module-level temporary stubs**

Create these files with temporary module stubs so imports can be added in later steps:

```rust
// crates/javascript-obfuscator/src/diagnostics.rs
pub type ObfuscatorResult<T> = Result<T, ObfuscatorError>;

#[derive(Debug)]
pub enum ObfuscatorError {
    Parse(String),
    Codegen(String),
}
```

```rust
// crates/javascript-obfuscator/src/parser.rs
pub struct ParsedProgram;
```

```rust
// crates/javascript-obfuscator/src/codegen.rs
pub fn generate_code() -> String {
    String::new()
}
```

```rust
// crates/javascript-obfuscator/src/pipeline.rs
pub fn run_pipeline(source_code: &str) -> String {
    source_code.to_string()
}
```

- [ ] **Step 3: Export the modules**

In `crates/javascript-obfuscator/src/lib.rs`, make the module list:

```rust
pub mod api;
pub mod codegen;
pub mod diagnostics;
pub mod options;
pub mod parser;
pub mod pipeline;

pub use api::{get_options_by_preset, obfuscate, obfuscate_multiple};
pub use diagnostics::{ObfuscatorError, ObfuscatorResult};
pub use options::{IdentifierNamesCache, ObfuscationResult, Options, Preset};
```

- [ ] **Step 4: Run the targeted Rust test and confirm failure**

Run:

```bash
cargo test -p javascript-obfuscator obfuscate_generates_code_from_parsed_ast
```

Expected: FAIL because `obfuscate` still echoes source code and does not call parser/codegen.

## Task 2: Implement SWC Parser and Codegen Adapters

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/javascript-obfuscator/Cargo.toml`
- Modify: `crates/javascript-obfuscator/src/diagnostics.rs`
- Modify: `crates/javascript-obfuscator/src/parser.rs`
- Modify: `crates/javascript-obfuscator/src/codegen.rs`

- [ ] **Step 1: Add SWC workspace dependencies**

In root `Cargo.toml`, add:

```toml
swc_common = "23"
swc_ecma_ast = "25"
swc_ecma_codegen = "28"
swc_ecma_parser = "41"
```

In `crates/javascript-obfuscator/Cargo.toml`, add:

```toml
swc_common.workspace = true
swc_ecma_ast.workspace = true
swc_ecma_codegen.workspace = true
swc_ecma_parser.workspace = true
```

- [ ] **Step 2: Implement structured diagnostics**

Replace `crates/javascript-obfuscator/src/diagnostics.rs` with:

```rust
use std::error::Error;
use std::fmt::{Display, Formatter};

pub type ObfuscatorResult<T> = Result<T, ObfuscatorError>;

#[derive(Debug)]
pub enum ObfuscatorError {
    Parse(String),
    Codegen(String),
}

impl Display for ObfuscatorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(message) => write!(formatter, "JavaScript parse error: {message}"),
            Self::Codegen(message) => write!(formatter, "JavaScript code generation error: {message}"),
        }
    }
}

impl Error for ObfuscatorError {}
```

- [ ] **Step 3: Implement parser adapter**

Replace `crates/javascript-obfuscator/src/parser.rs` with:

```rust
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::Program;
use swc_ecma_parser::{lexer::Lexer, EsSyntax, Parser, StringInput, Syntax};

use crate::diagnostics::{ObfuscatorError, ObfuscatorResult};

pub struct ParsedProgram {
    pub program: Program,
    pub source_map: Lrc<SourceMap>,
}

pub fn parse_program(source_code: &str) -> ObfuscatorResult<ParsedProgram> {
    let source_map: Lrc<SourceMap> = Default::default();
    let source_file = source_map.new_source_file(
        FileName::Custom("source.js".to_string()).into(),
        source_code.to_string(),
    );

    parse_with_module_mode(source_map.clone(), &source_file, true)
        .or_else(|_| parse_with_module_mode(source_map.clone(), &source_file, false))
}

fn parse_with_module_mode(
    source_map: Lrc<SourceMap>,
    source_file: &swc_common::SourceFile,
    module: bool,
) -> ObfuscatorResult<ParsedProgram> {
    let lexer = Lexer::new(
        Syntax::Es(EsSyntax {
            jsx: true,
            export_default_from: true,
            import_attributes: true,
            allow_super_outside_method: true,
            allow_return_outside_function: true,
            ..Default::default()
        }),
        swc_ecma_ast::EsVersion::Es2022,
        StringInput::from(source_file),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    let program = if module {
        parser.parse_module().map(Program::Module)
    } else {
        parser.parse_script().map(Program::Script)
    }
    .map_err(|error| ObfuscatorError::Parse(error.kind().msg().into_owned()))?;

    Ok(ParsedProgram { program, source_map })
}
```

- [ ] **Step 4: Implement codegen adapter**

Replace `crates/javascript-obfuscator/src/codegen.rs` with:

```rust
use swc_common::SourceMap;
use swc_ecma_ast::Program;
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};

use crate::diagnostics::{ObfuscatorError, ObfuscatorResult};

pub fn generate_code(program: &Program, source_map: &SourceMap, compact: bool) -> ObfuscatorResult<String> {
    let mut output = Vec::new();
    {
        let writer = Box::new(JsWriter::new(source_map, "\n", &mut output, None));
        let mut emitter = Emitter {
            cfg: Config::default().with_minify(compact),
            comments: None,
            cm: source_map.into(),
            wr: writer,
        };

        emitter
            .emit_program(program)
            .map_err(|error| ObfuscatorError::Codegen(error.to_string()))?;
    }

    String::from_utf8(output).map_err(|error| ObfuscatorError::Codegen(error.to_string()))
}
```

- [ ] **Step 5: Compile the engine crate**

Run:

```bash
cargo test -p javascript-obfuscator --no-run
```

Expected: PASS after adjusting for exact SWC API names if the current crate versions differ.

## Task 3: Route Rust API Through Pipeline

**Files:**
- Modify: `crates/javascript-obfuscator/src/options.rs`
- Modify: `crates/javascript-obfuscator/src/pipeline.rs`
- Modify: `crates/javascript-obfuscator/src/api.rs`
- Modify: `crates/javascript-obfuscator-node/src/lib.rs`

- [ ] **Step 1: Expand Rust options for this slice**

In `crates/javascript-obfuscator/src/options.rs`, add these fields to `Options`:

```rust
#[serde(default)]
pub input_file_name: Option<String>,
#[serde(default)]
pub source_map_mode: Option<String>,
#[serde(default)]
pub source_map_sources_mode: Option<String>,
```

- [ ] **Step 2: Implement pipeline assembly**

Replace `crates/javascript-obfuscator/src/pipeline.rs` with:

```rust
use crate::codegen::generate_code;
use crate::diagnostics::ObfuscatorResult;
use crate::options::{ObfuscationResult, Options};
use crate::parser::parse_program;

pub fn run_pipeline(source_code: &str, options: Options) -> ObfuscatorResult<ObfuscationResult> {
    let (hashbang, prepared_code) = extract_hashbang(source_code);
    let parsed_program = parse_program(&prepared_code)?;
    let mut code = generate_code(
        &parsed_program.program,
        &parsed_program.source_map,
        options.compact.unwrap_or(true),
    )?;

    if let Some(hashbang) = hashbang {
        code = format!("{hashbang}{code}");
    }

    let source_map = if options.source_map.unwrap_or(false) {
        build_source_map_metadata(source_code, &options)
    } else {
        String::new()
    };

    Ok(ObfuscationResult::new(
        code,
        source_map,
        options.identifier_names_cache,
    ))
}

fn extract_hashbang(source_code: &str) -> (Option<String>, String) {
    if !source_code.starts_with("#!") {
        return (None, source_code.to_string());
    }

    let line_end = source_code.find('\n').map(|index| index + 1).unwrap_or(source_code.len());
    let hashbang = source_code[..line_end].to_string();
    let prepared_code = source_code[line_end..].trim().to_string();

    (Some(hashbang), prepared_code)
}

fn build_source_map_metadata(source_code: &str, options: &Options) -> String {
    let source_name = options
        .input_file_name
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or("sourceMap");

    serde_json::json!({
        "version": 3,
        "file": source_name,
        "sources": [source_name],
        "sourcesContent": [source_code],
        "names": [],
        "mappings": ""
    })
    .to_string()
}
```

- [ ] **Step 3: Update API to return structured errors**

In `crates/javascript-obfuscator/src/api.rs`, change signatures and imports:

```rust
use crate::diagnostics::{ObfuscatorError, ObfuscatorResult};
use crate::pipeline::run_pipeline;
```

Use:

```rust
pub fn obfuscate(source_code: &str, options: Options) -> ObfuscatorResult<ObfuscationResult> {
    run_pipeline(source_code, options)
}

pub fn obfuscate_multiple(
    source_codes: BTreeMap<String, String>,
    options: Options,
) -> ObfuscatorResult<BTreeMap<String, ObfuscationResult>> {
    source_codes
        .into_iter()
        .map(|(file_name, source_code)| {
            let result = obfuscate(&source_code, options.clone())?;

            Ok((file_name, result))
        })
        .collect::<Result<_, ObfuscatorError>>()
}
```

- [ ] **Step 4: Update Node binding error mapping**

In `crates/javascript-obfuscator-node/src/lib.rs`, keep the public napi shape but map errors via `to_string()`:

```rust
let result = javascript_obfuscator::obfuscate(&source_code, options)
    .map_err(|error| Error::from_reason(error.to_string()))?;
```

- [ ] **Step 5: Run targeted Rust tests**

Run:

```bash
cargo test -p javascript-obfuscator
```

Expected: PASS.

- [ ] **Step 6: Commit parser/codegen pipeline**

Run:

```bash
git add Cargo.toml Cargo.lock crates/javascript-obfuscator crates/javascript-obfuscator-node
git commit -m "feat: add rust parser codegen pipeline"
```

## Task 4: Preserve Package Boundary and Add Bridge Tests

**Files:**
- Modify: `test/functional-tests/rust-rewrite/RustBridge.spec.ts`

- [ ] **Step 1: Add a boundary test that TypeScript facade still owns package output**

Append this describe block:

```typescript
describe('Variant #5: package facade stays on TypeScript engine for this slice', () => {
    it('should keep TypeScript transformer behavior while Rust parser parity is incomplete', () => {
        const result = JavaScriptObfuscator.obfuscate('const value = true;', {
            compact: true,
            booleanLiterals: false,
            stringArray: false,
            renameGlobals: false
        } as any);

        assert.include(result.getObfuscatedCode(), 'const');
    });
});
```

If `booleanLiterals` is not a valid option in this codebase, remove that property and keep the assertion focused on `stringArray: false` and public facade output.

- [ ] **Step 2: Run bridge tests**

Run:

```bash
npx mocha --require ts-node/register --require source-map-support/register test/functional-tests/rust-rewrite/RustBridge.spec.ts
```

Expected: PASS.

- [ ] **Step 3: Commit bridge boundary test**

Run:

```bash
git add test/functional-tests/rust-rewrite/RustBridge.spec.ts
git commit -m "test: keep rust bridge behind compatibility boundary"
```

## Task 5: Slice Verification and Push

**Files:**
- No new source files unless verification reveals generated lockfile changes.

- [ ] **Step 1: Run Rust verification**

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: PASS. The napi test binary may print Node-API host-runtime load warnings while still exiting 0.

- [ ] **Step 2: Run TypeScript targeted verification**

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

- This plan implements migration slice 5 from the approved design: parser/codegen and basic result assembly.
- It keeps VMP removed and does not restore Pro API surface.
- It preserves package compatibility by keeping the TypeScript facade on the existing engine until Rust transformer parity improves.

Placeholder scan:

- The plan has no unfinished markers or incomplete implementation stubs in the final task instructions.

Type consistency:

- `ObfuscatorResult<T>` is used consistently by parser, codegen, pipeline, and API.
- `ObfuscationResult` remains the serialized result type expected by the existing Node binding.
