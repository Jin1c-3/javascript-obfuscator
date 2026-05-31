# Rust Rewrite Design

## Goal

Rewrite the local JavaScript obfuscator implementation in Rust while preserving the existing non-VMP npm package API, CLI workflows, options, output contracts, and tests. The Pro API, VM obfuscation, and VMP-only surface will be removed from the rewritten project.

## Current State

The repository is on `master` with official `javascript-obfuscator/javascript-obfuscator` upstream merged. The merge commit is `e603bc53`.

The current implementation is TypeScript. The public package entrypoint is `index.ts`, the CLI entrypoint is `bin/javascript-obfuscator`, and the core orchestration lives in `src/JavaScriptObfuscator.ts` and `src/JavaScriptObfuscatorFacade.ts`.

The VMP surface is not part of the local AST transformation pipeline. It is a cloud Pro API feature exposed through `src/pro-api`, `obfuscatePro()`, CLI `--pro-api-*` and `--vm-*` options, `parseHtml`, Pro API tests, Pro API typings, and README sections. This design removes that surface.

## Non-VMP Compatibility Requirements

The rewritten project must preserve these user-facing contracts:

- `JavaScriptObfuscator.obfuscate(sourceCode, options)`
- `JavaScriptObfuscator.obfuscateMultiple(sourceCodesObject, options)`
- `JavaScriptObfuscator.getOptionsByPreset(optionsPreset)`
- npm package loading from `require('javascript-obfuscator')`
- CLI invocation through `javascript-obfuscator <inputPath> [options]`
- single-file and directory obfuscation
- output path handling
- source map modes and source map filenames
- identifier names cache input and output
- existing non-Pro options, presets, validators, and normalizer behavior
- existing non-Pro obfuscation transformations and runtime helpers
- browser and node targets, including browser-no-eval behavior

The rewritten project must remove these contracts:

- `obfuscatePro()`
- exported Pro API types
- `src/pro-api`
- CLI `--pro-api-token` and `--pro-api-version`
- CLI `--vm-*` options
- `parseHtml`
- Pro API and VM documentation
- Pro API and VM tests

## Architecture

Use a Rust core engine with a thin Node/npm compatibility wrapper.

The Rust core owns parsing, option normalization, AST transformation, code generation, source map metadata, identifier cache updates, and result assembly. The JavaScript/TypeScript layer remains only where it is needed for npm package compatibility, CLI process integration, and loading the native Rust artifact.

This keeps the package usable by current Node consumers while making the obfuscation engine Rust-based.

## Rust Workspace

Add a root Rust workspace with these crates:

- `crates/javascript-obfuscator`: Rust library containing the obfuscation engine.
- `crates/javascript-obfuscator-cli`: Rust CLI binary for direct native execution and CLI smoke tests.
- `crates/javascript-obfuscator-node`: Node binding crate that exposes the Rust engine to the npm package.

The Rust library is the source of truth. The Node binding and Rust CLI call the same library APIs.

## Rust Library Modules

The Rust library will mirror the current TypeScript boundaries:

- `api`: public Rust functions for `obfuscate`, `obfuscate_multiple`, and `get_options_by_preset`.
- `options`: option structs, defaults, presets, validation, normalization, and CLI option mapping.
- `parser`: JavaScript parsing adapter.
- `ast`: AST node helpers, guards, traversal, parent metadata, and node cloning utilities.
- `transforms`: staged transformer runner and transformer implementations.
- `analyzers`: scope analysis, call graph analysis, string array analysis, numerical expression analysis, and prevailing variable kind analysis.
- `generators`: identifier names and string array index generation.
- `storages`: string array, control flow, helper, and identifier cache storage.
- `helpers`: generated runtime helper snippets for string arrays, debug protection, domain lock, self-defending code, console disabling, and call controllers.
- `codegen`: JavaScript output generation and source map handling.
- `result`: obfuscation result and identifier cache result objects.
- `diagnostics`: structured errors shared by CLI, Node bindings, and tests.

## Parser and Codegen Strategy

The Rust implementation should use mature Rust JavaScript tooling where it provides a stable AST and code generation path. The preferred first implementation target is SWC crates because they provide parsing, AST traversal, code generation, and source maps in Rust.

If SWC cannot represent a current TypeScript ESTree-dependent behavior exactly, the Rust AST layer will isolate those differences behind project-owned helpers. The public API will remain package-compatible even if the internal AST representation differs from ESTree.

## Transformation Pipeline

The Rust engine keeps the current stage order:

1. Preparing code transformations
2. AST parse
3. Initializing node transformations
4. Preparing node transformations
5. Dead code injection
6. Control flow flattening
7. Rename properties
8. Converting transformations
9. Rename identifiers
10. String array transformations
11. Simplifying transformations
12. Finalizing transformations
13. Code generation
14. Finalizing code transformations
15. Source map and result assembly

Each transformer group should be ported as a focused Rust module with tests copied from or mapped to the existing non-Pro test coverage.

## Node and CLI Compatibility Layer

The Node package will expose the same non-Pro functions from the existing package entrypoint. It will load the native Rust binding and adapt JavaScript values to Rust structs.

The CLI will keep the current command name and non-Pro flags. CLI parsing can stay in JavaScript initially if it only delegates to Rust, or move to Rust if npm wrapper behavior remains equivalent. The final state should avoid TypeScript implementation logic outside the wrapper boundary.

Generated `dist` artifacts should continue to support package consumers. Build scripts will be updated so npm builds compile the Rust binding and package wrapper artifacts.

## VMP Removal

Remove Pro API and VM code deliberately rather than leaving unused stubs.

Required removals:

- Delete `src/pro-api`.
- Delete `src/interfaces/pro-api`.
- Remove `obfuscatePro` and `ApiError` Pro exports from package entrypoints.
- Remove `ProApiClient` use from CLI.
- Remove all `--pro-api-*`, `--vm-*`, and `--parse-html` CLI options.
- Remove Pro API tests from `test/index.spec.ts` and delete Pro API test files.
- Remove README Pro API and VM sections.
- Remove Pro API typings.
- Remove Pro advertisement messages and imports.

Unsupported VMP options should fail as unknown CLI options or unknown package options consistently with normal option validation.

## Error Handling

Rust errors should be structured and converted to JavaScript `Error` objects in the Node binding. Error messages should preserve existing user-facing validation text where tests depend on it.

Parser errors, option validation errors, unsupported option errors, file IO errors, and source map errors should remain distinguishable in tests and CLI output.

## Testing Strategy

Use existing non-Pro tests as the compatibility oracle. Remove only tests that cover the deleted Pro API and VM surface.

Required verification before final push:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `yarn run eslint`
- `yarn run test:mocha` with Pro API tests removed
- targeted CLI smoke tests for single-file, directory, source map, and identifier cache behavior
- package API smoke tests for `obfuscate`, `obfuscateMultiple`, and presets

During porting, transformer groups may be verified with targeted test files before full-suite runs.

## Migration Plan

Implementation should proceed in vertical slices:

1. Remove VMP/Pro surface from TypeScript and tests.
2. Add Rust workspace and minimal Rust engine API.
3. Add Node binding and package wrapper that calls Rust.
4. Port options and presets.
5. Port parser/codegen and basic result assembly.
6. Port identifier generation and cache handling.
7. Port simple converting and finalizing transforms.
8. Port string array transforms and helpers.
9. Port rename identifiers and scope analysis.
10. Port rename properties.
11. Port control flow flattening.
12. Port dead code injection.
13. Port debug/self-defending/domain-lock/console helpers.
14. Retire TypeScript engine implementation once Rust coverage replaces it.
15. Run full verification, commit, and push to `origin/master`.

Each slice should preserve or improve the passing non-Pro test subset before continuing.

## Risks

The largest technical risks are AST semantic differences between Acorn/ESTree and Rust parser tooling, source map parity, scope analysis parity, and exact output assumptions in tests. The mitigation is to keep compatibility tests as the oracle and isolate parser/codegen differences behind Rust helper modules.

The largest project risk is attempting to port every transformer in one unverified edit. The mitigation is vertical migration with targeted tests and frequent commits.

## Acceptance Criteria

The goal is complete only when:

- The latest official upstream code is merged into the local branch.
- The Rust toolchain is usable from the repo.
- The obfuscation engine implementation is Rust-based.
- The Node/npm package still exposes the preserved non-VMP API.
- The CLI still supports preserved non-VMP workflows.
- Pro API, VM, and VMP features are removed.
- Existing non-Pro tests pass or are replaced by equivalent Rust-backed tests.
- Rust formatting, linting, and tests pass.
- Node linting and non-Pro tests pass.
- The completed changes are committed.
- The completed branch is pushed to the configured remote.
