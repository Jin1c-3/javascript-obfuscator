use regex::Regex;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::diagnostics::{ObfuscatorError, ObfuscatorResult};
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
    pub transform_object_keys: Option<bool>,
    #[serde(default)]
    pub string_array_index_shift: Option<bool>,
    #[serde(default)]
    pub string_array_shuffle: Option<bool>,
    #[serde(default)]
    pub string_array_rotate: Option<bool>,
    #[serde(default)]
    pub force_transform_strings: Option<Vec<String>>,
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
    #[serde(default, deserialize_with = "deserialize_optional_usize_floor")]
    pub split_strings_chunk_length: Option<usize>,
    #[serde(default)]
    pub unicode_escape_sequence: Option<bool>,
    #[serde(default)]
    pub disable_console_output: Option<bool>,
    #[serde(default)]
    pub source_map: Option<bool>,
    #[serde(default)]
    pub source_map_base_url: Option<String>,
    #[serde(default)]
    pub source_map_file_name: Option<String>,
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

pub fn validate_regex_options(options: &Options) -> ObfuscatorResult<()> {
    validate_regex_option("reservedNames", options.reserved_names.as_deref())?;
    validate_regex_option("reservedStrings", options.reserved_strings.as_deref())?;
    validate_regex_option(
        "forceTransformStrings",
        options.force_transform_strings.as_deref(),
    )
}

fn validate_regex_option(option_name: &str, patterns: Option<&[String]>) -> ObfuscatorResult<()> {
    let Some(patterns) = patterns else {
        return Ok(());
    };

    for pattern in patterns {
        Regex::new(pattern).map_err(|error| {
            ObfuscatorError::Options(format!(
                "Invalid regular expression in `{option_name}` option: `{pattern}` ({error})"
            ))
        })?;
    }

    Ok(())
}

fn deserialize_optional_usize_floor<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(value) = Option::<Value>::deserialize(deserializer)? else {
        return Ok(None);
    };

    match value {
        Value::Null => Ok(None),
        Value::Number(number) => deserialize_usize_number(number).map(Some),
        _ => Err(D::Error::custom("expected a number or null")),
    }
}

fn deserialize_usize_number<D>(number: serde_json::Number) -> Result<usize, D>
where
    D: DeError,
{
    if let Some(unsigned) = number.as_u64() {
        return usize::try_from(unsigned).map_err(D::custom);
    }

    let Some(float) = number.as_f64() else {
        return Err(D::custom("expected a finite non-negative number"));
    };

    if !float.is_finite() || float < 0.0 {
        return Err(D::custom("expected a finite non-negative number"));
    }

    let floored = float.floor();
    if floored > usize::MAX as f64 {
        return Err(D::custom("number is too large"));
    }

    Ok(floored as usize)
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

    #[test]
    fn deserializes_force_transform_strings_for_option_compatibility() {
        let options: Options = serde_json::from_value(json!({
            "forceTransformStrings": ["ar$"]
        }))
        .expect("force transform strings option should deserialize");

        assert_eq!(
            options.force_transform_strings,
            Some(vec!["ar$".to_string()])
        );
    }

    #[test]
    fn deserializes_split_strings_chunk_length_float_for_option_compatibility() {
        let options: Options = serde_json::from_value(json!({
            "splitStringsChunkLength": 5.6
        }))
        .expect("split strings chunk length should deserialize");

        assert_eq!(options.split_strings_chunk_length, Some(5));
    }
}
