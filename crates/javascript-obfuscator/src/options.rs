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

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Preset {
    #[default]
    Default,
    LowObfuscation,
    MediumObfuscation,
    HighObfuscation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObfuscationResult {
    pub code: String,
    pub source_map: String,
    pub identifier_names_cache: Option<IdentifierNamesCache>,
}

impl ObfuscationResult {
    pub fn new(
        code: String,
        source_map: String,
        identifier_names_cache: Option<IdentifierNamesCache>,
    ) -> Self {
        Self {
            code,
            source_map,
            identifier_names_cache,
        }
    }
}
