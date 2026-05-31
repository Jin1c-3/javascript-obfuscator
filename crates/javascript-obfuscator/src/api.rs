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
        let result =
            obfuscate("const value = 1;", Options::default()).expect("obfuscation should succeed");

        assert_eq!(result.code, "const value = 1;");
        assert_eq!(result.source_map, "");
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
    }
}
