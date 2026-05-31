use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::diagnostics::ObfuscatorResult;
use crate::options::{ObfuscationResult, Options, Preset};
use crate::pipeline::run_pipeline;

pub fn obfuscate(source_code: &str, options: Options) -> ObfuscatorResult<ObfuscationResult> {
    run_pipeline(source_code, options)
}

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
    fn obfuscate_generates_code_from_parsed_ast() {
        let result = obfuscate(
            "const value = 1; console.log(value);",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value=0x1"));
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
        assert!(result.code.contains("const value=0x1"));
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

    #[test]
    fn obfuscate_transforms_true_boolean_literals() {
        let result = obfuscate(
            "const value = true;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value=!![]"));
        assert!(!result.code.contains("true"));
    }

    #[test]
    fn obfuscate_transforms_false_boolean_literals() {
        let result = obfuscate(
            "const value = false;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value=![]"));
        assert!(!result.code.contains("false"));
    }

    #[test]
    fn obfuscate_transforms_integer_number_literal_raw_value() {
        let result = obfuscate(
            "const value = 10;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value=0xa"));
    }

    #[test]
    fn obfuscate_transforms_bigint_literal_raw_value() {
        let result = obfuscate(
            "const value = 10n;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value=0xan"));
    }

    #[test]
    fn obfuscate_transforms_member_expression_dot_notation() {
        let result = obfuscate(
            "const value = console.log;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(true),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value=console['log']"));
    }

    #[test]
    fn obfuscate_keeps_member_expression_dot_notation_when_disabled() {
        let result = obfuscate(
            "const value = console.log;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value=console.log"));
    }

    #[test]
    fn obfuscate_transforms_template_literal_with_expression() {
        let result = obfuscate(
            "const value = `abc ${foo}`;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value='abc\\x20'+foo"));
        assert!(!result.code.contains('`'));
    }

    #[test]
    fn obfuscate_keeps_tagged_template_literal() {
        let result = obfuscate(
            "tag`abc ${foo}`;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("tag`abc ${foo}`"));
    }

    #[test]
    fn obfuscate_transforms_object_expression_identifier_key() {
        let result = obfuscate(
            "const value = {foo: 1};",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value={'foo':0x1}"));
    }

    #[test]
    fn obfuscate_transforms_object_expression_shorthand_property() {
        let result = obfuscate(
            "const value = {foo};",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value={'foo':foo}"));
    }

    #[test]
    fn obfuscate_transforms_class_method_identifier_key() {
        let result = obfuscate(
            "class Foo { bar() {} }",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("class Foo{['bar'](){}}"));
    }

    #[test]
    fn obfuscate_transforms_class_property_identifier_key() {
        let result = obfuscate(
            "class Foo { property = 1; }",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("class Foo{['property']=0x1;}"));
    }

    #[test]
    fn obfuscate_splits_string_literals_when_enabled() {
        let result = obfuscate(
            "const value = 'abcdef';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                split_strings: Some(true),
                split_strings_chunk_length: Some(3),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value='abc'+'def'"));
    }

    #[test]
    fn obfuscate_keeps_string_literals_when_split_strings_disabled() {
        let result = obfuscate(
            "const value = 'abcdef';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                split_strings: Some(false),
                split_strings_chunk_length: Some(3),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value='abcdef'"));
    }
}
