use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::generators::IdentifierNamesGeneratorKind;

pub type IdentifierNamesCache = Map<String, Value>;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    #[serde(default)]
    pub compact: Option<bool>,
    #[serde(default)]
    pub numbers_to_expressions: Option<bool>,
    #[serde(default)]
    pub simplify: Option<bool>,
    #[serde(default)]
    pub string_array: Option<bool>,
    #[serde(default)]
    pub string_array_threshold: Option<f64>,
    #[serde(default)]
    pub string_array_encoding: Option<Vec<StringArrayEncoding>>,
    #[serde(default)]
    pub string_array_indexes_type: Option<Vec<StringArrayIndexesType>>,
    #[serde(default)]
    pub string_array_index_shift: Option<bool>,
    #[serde(default)]
    pub string_array_shuffle: Option<bool>,
    #[serde(default)]
    pub string_array_rotate: Option<bool>,
    #[serde(default)]
    pub ignore_imports: Option<bool>,
    #[serde(default)]
    pub rename_globals: Option<bool>,
    #[serde(default)]
    pub property_bracketing: Option<bool>,
    #[serde(default)]
    pub reserved_names: Option<Vec<String>>,
    #[serde(default)]
    pub reserved_strings: Option<Vec<String>>,
    #[serde(default)]
    pub split_strings: Option<bool>,
    #[serde(default)]
    pub split_strings_chunk_length: Option<usize>,
    #[serde(default)]
    pub unicode_escape_sequence: Option<bool>,
    #[serde(default)]
    pub disable_console_output: Option<bool>,
    #[serde(default)]
    pub source_map: Option<bool>,
    #[serde(default)]
    pub input_file_name: Option<String>,
    #[serde(default)]
    pub source_map_mode: Option<String>,
    #[serde(default)]
    pub source_map_sources_mode: Option<String>,
    #[serde(default)]
    pub identifier_names_generator: Option<IdentifierNamesGeneratorKind>,
    #[serde(default)]
    pub identifiers_prefix: Option<String>,
    #[serde(default)]
    pub identifiers_dictionary: Option<Vec<String>>,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StringArrayIndexesType {
    HexadecimalNumber,
    HexadecimalNumericString,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StringArrayEncoding {
    #[default]
    None,
    Base64,
    Rc4,
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn deserializes_rc4_string_array_encoding_for_option_compatibility() {
        let options: Options = serde_json::from_value(json!({
            "stringArrayEncoding": ["rc4"]
        }))
        .expect("rc4 string array encoding option should deserialize");

        assert_eq!(
            options.string_array_encoding,
            Some(vec![StringArrayEncoding::Rc4])
        );
    }

    #[test]
    fn deserializes_keep_original_identifier_names_generator_for_option_compatibility() {
        let options: Options = serde_json::from_value(json!({
            "identifierNamesGenerator": "keep-original"
        }))
        .expect("keep-original identifier generator option should deserialize");

        assert_eq!(
            options.identifier_names_generator,
            Some(IdentifierNamesGeneratorKind::KeepOriginal)
        );
    }
}
