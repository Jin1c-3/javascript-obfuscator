pub mod api;
pub mod options;

pub use api::{get_options_by_preset, obfuscate, obfuscate_multiple};
pub use options::{IdentifierNamesCache, ObfuscationResult, Options, Preset};
