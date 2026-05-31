pub mod api;
pub mod codegen;
pub mod diagnostics;
pub mod options;
pub mod parser;
pub mod pipeline;

pub use api::{get_options_by_preset, obfuscate, obfuscate_multiple};
pub use diagnostics::{ObfuscatorError, ObfuscatorResult};
pub use options::{IdentifierNamesCache, ObfuscationResult, Options, Preset};
