use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

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
        Preset::Default => Value::Object(default_preset_options()),
        Preset::LowObfuscation => preset_with_overrides(
            default_preset_options(),
            [
                ("disableConsoleOutput", json!(true)),
                ("optionsPreset", json!("low-obfuscation")),
                ("stringArrayRotate", json!(true)),
                ("selfDefending", json!(true)),
                ("simplify", json!(true)),
                ("stringArrayCallsTransform", json!(false)),
                ("stringArrayCallsTransformThreshold", json!(0)),
                ("stringArrayShuffle", json!(true)),
            ],
        ),
        Preset::MediumObfuscation => preset_with_overrides(
            low_obfuscation_preset_options(),
            [
                ("controlFlowFlattening", json!(true)),
                ("deadCodeInjection", json!(true)),
                ("numbersToExpressions", json!(true)),
                ("optionsPreset", json!("medium-obfuscation")),
                ("splitStrings", json!(true)),
                ("splitStringsChunkLength", json!(10)),
                ("stringArrayCallsTransformThreshold", json!(0.75)),
                ("stringArrayEncoding", json!(["base64"])),
                ("stringArrayWrappersCount", json!(2)),
                ("stringArrayWrappersParametersMaxCount", json!(4)),
                ("stringArrayWrappersType", json!("function")),
                ("transformObjectKeys", json!(true)),
            ],
        ),
        Preset::HighObfuscation => preset_with_overrides(
            medium_obfuscation_preset_options(),
            [
                ("controlFlowFlatteningThreshold", json!(1)),
                ("deadCodeInjectionThreshold", json!(1)),
                ("debugProtection", json!(true)),
                ("debugProtectionInterval", json!(4000)),
                ("optionsPreset", json!("high-obfuscation")),
                ("splitStringsChunkLength", json!(5)),
                ("stringArrayCallsTransformThreshold", json!(1)),
                ("stringArrayEncoding", json!(["rc4"])),
                ("stringArrayWrappersCount", json!(5)),
                ("stringArrayWrappersParametersMaxCount", json!(5)),
                ("stringArrayThreshold", json!(1)),
            ],
        ),
    }
}

fn low_obfuscation_preset_options() -> Map<String, Value> {
    preset_object(preset_with_overrides(
        default_preset_options(),
        [
            ("disableConsoleOutput", json!(true)),
            ("optionsPreset", json!("low-obfuscation")),
            ("stringArrayRotate", json!(true)),
            ("selfDefending", json!(true)),
            ("simplify", json!(true)),
            ("stringArrayCallsTransform", json!(false)),
            ("stringArrayCallsTransformThreshold", json!(0)),
            ("stringArrayShuffle", json!(true)),
        ],
    ))
}

fn medium_obfuscation_preset_options() -> Map<String, Value> {
    preset_object(preset_with_overrides(
        low_obfuscation_preset_options(),
        [
            ("controlFlowFlattening", json!(true)),
            ("deadCodeInjection", json!(true)),
            ("numbersToExpressions", json!(true)),
            ("optionsPreset", json!("medium-obfuscation")),
            ("splitStrings", json!(true)),
            ("splitStringsChunkLength", json!(10)),
            ("stringArrayCallsTransformThreshold", json!(0.75)),
            ("stringArrayEncoding", json!(["base64"])),
            ("stringArrayWrappersCount", json!(2)),
            ("stringArrayWrappersParametersMaxCount", json!(4)),
            ("stringArrayWrappersType", json!("function")),
            ("transformObjectKeys", json!(true)),
        ],
    ))
}

fn preset_object(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(object) => object,
        _ => unreachable!("preset builders should always return an object"),
    }
}

fn preset_with_overrides<const N: usize>(
    mut preset: Map<String, Value>,
    overrides: [(&str, Value); N],
) -> Value {
    for (key, value) in overrides {
        preset.insert(key.to_string(), value);
    }

    Value::Object(preset)
}

fn default_preset_options() -> Map<String, Value> {
    [
        ("compact", json!(true)),
        ("config", json!("")),
        ("controlFlowFlattening", json!(false)),
        ("controlFlowFlatteningThreshold", json!(0.75)),
        ("deadCodeInjection", json!(false)),
        ("deadCodeInjectionThreshold", json!(0.4)),
        ("debugProtection", json!(false)),
        ("debugProtectionInterval", json!(0)),
        ("disableConsoleOutput", json!(false)),
        ("domainLock", json!([])),
        ("domainLockRedirectUrl", json!("about:blank")),
        ("exclude", json!([])),
        ("forceTransformStrings", json!([])),
        ("identifierNamesCache", Value::Null),
        ("identifierNamesGenerator", json!("hexadecimal")),
        ("identifiersPrefix", json!("")),
        ("identifiersDictionary", json!([])),
        ("ignoreImports", json!(false)),
        ("inputFileName", json!("")),
        ("log", json!(false)),
        ("numbersToExpressions", json!(false)),
        ("optionsPreset", json!("default")),
        ("renameGlobals", json!(false)),
        ("renameProperties", json!(false)),
        ("renamePropertiesMode", json!("safe")),
        ("reservedNames", json!([])),
        ("reservedStrings", json!([])),
        ("stringArrayRotate", json!(true)),
        ("seed", json!(0)),
        ("selfDefending", json!(false)),
        ("stringArrayShuffle", json!(true)),
        ("simplify", json!(true)),
        ("sourceMap", json!(false)),
        ("sourceMapBaseUrl", json!("")),
        ("sourceMapFileName", json!("")),
        ("sourceMapMode", json!("separate")),
        ("sourceMapSourcesMode", json!("sources-content")),
        ("splitStrings", json!(false)),
        ("splitStringsChunkLength", json!(10)),
        ("stringArray", json!(true)),
        ("stringArrayCallsTransform", json!(false)),
        ("stringArrayCallsTransformThreshold", json!(0.5)),
        ("stringArrayEncoding", json!(["none"])),
        ("stringArrayIndexesType", json!(["hexadecimal-number"])),
        ("stringArrayIndexShift", json!(true)),
        ("stringArrayWrappersChainedCalls", json!(true)),
        ("stringArrayWrappersCount", json!(1)),
        ("stringArrayWrappersParametersMaxCount", json!(2)),
        ("stringArrayWrappersType", json!("variable")),
        ("stringArrayThreshold", json!(0.75)),
        ("target", json!("browser")),
        ("transformObjectKeys", json!(false)),
        ("propertyBracketing", json!(true)),
        ("unicodeEscapeSequence", json!(false)),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect()
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Output};

    use super::*;

    fn run_node_source(source_code: &str) -> Output {
        Command::new("node")
            .arg("-e")
            .arg(source_code)
            .output()
            .expect("node should execute generated code")
    }

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
    fn obfuscate_source_map_sources_mode_sources_omits_sources_content() {
        let options: Options = serde_json::from_value(json!({
            "inputFileName": "input.js",
            "sourceMap": true,
            "sourceMapSourcesMode": "sources"
        }))
        .expect("source map options should deserialize");
        let result = obfuscate("const value = 1;", options).expect("obfuscation should succeed");
        let source_map: Value =
            serde_json::from_str(&result.source_map).expect("source map should be JSON");

        assert_eq!(source_map["sources"], json!(["input.js"]));
        assert!(source_map.get("sourcesContent").is_none());
    }

    #[test]
    fn obfuscate_inline_source_map_appends_data_url_comment() {
        let options: Options = serde_json::from_value(json!({
            "compact": true,
            "sourceMap": true,
            "sourceMapMode": "inline",
            "stringArray": false
        }))
        .expect("source map options should deserialize");
        let result = obfuscate("const value = 1;", options).expect("obfuscation should succeed");

        assert!(result
            .code
            .contains("\n//# sourceMappingURL=data:application/json;base64,"));
    }

    #[test]
    fn obfuscate_separate_source_map_appends_configured_url_comment() {
        let options: Options = serde_json::from_value(json!({
            "compact": true,
            "sourceMap": true,
            "sourceMapMode": "separate",
            "sourceMapBaseUrl": "https://cdn.example/maps/",
            "sourceMapFileName": "bundle.js.map",
            "stringArray": false
        }))
        .expect("source map options should deserialize");
        let result = obfuscate("const value = 1;", options).expect("obfuscation should succeed");

        assert!(result
            .code
            .ends_with("\n//# sourceMappingURL=https://cdn.example/maps/bundle.js.map"));
    }

    #[test]
    fn obfuscate_reports_parse_errors() {
        let error = obfuscate("const =", Options::default()).expect_err("parse should fail");

        assert!(error.to_string().contains("JavaScript parse error"));
    }

    fn assert_invalid_regex_option_error(option_name: &str, options: Options) {
        let error = obfuscate("const value = 'abcdef';", options)
            .expect_err("invalid regex option should fail");
        let message = error.to_string();

        assert!(message.contains(option_name), "{message}");
        assert!(message.contains('['), "{message}");
    }

    #[test]
    fn obfuscate_reports_invalid_regex_option_reserved_strings() {
        assert_invalid_regex_option_error(
            "reservedStrings",
            Options {
                reserved_strings: Some(vec!["[".to_string()]),
                string_array: Some(false),
                ..Options::default()
            },
        );
    }

    #[test]
    fn obfuscate_reports_invalid_regex_option_force_transform_strings() {
        assert_invalid_regex_option_error(
            "forceTransformStrings",
            Options {
                force_transform_strings: Some(vec!["[".to_string()]),
                string_array: Some(false),
                ..Options::default()
            },
        );
    }

    #[test]
    fn obfuscate_reports_invalid_regex_option_reserved_names() {
        assert_invalid_regex_option_error(
            "reservedNames",
            Options {
                reserved_names: Some(vec!["[".to_string()]),
                string_array: Some(false),
                ..Options::default()
            },
        );
    }

    #[test]
    fn obfuscate_reports_parse_error_before_invalid_regex_option() {
        let error = obfuscate(
            "const =",
            Options {
                reserved_strings: Some(vec!["[".to_string()]),
                ..Options::default()
            },
        )
        .expect_err("parse should fail before option regex validation");

        assert!(error.to_string().contains("JavaScript parse error"));
    }

    #[test]
    fn obfuscate_disable_console_output_suppresses_console_methods_at_runtime() {
        let options: Options = serde_json::from_value(json!({
            "compact": true,
            "disableConsoleOutput": true,
            "propertyBracketing": true,
            "renameGlobals": false,
            "simplify": true,
            "stringArray": true,
            "stringArrayIndexShift": true,
            "stringArrayRotate": true,
            "stringArrayShuffle": true,
            "unicodeEscapeSequence": true
        }))
        .expect("disable console output options should deserialize");
        let result = obfuscate(
            "'use strict'; function strictThis(){ return this; } if (strictThis() !== undefined) { throw new Error('strict mode changed'); } console.log('log'); console.warn('warn'); console.info('info'); console.error('error'); console.exception('exception'); console.table(['table']); console.trace('trace'); console.log.toString(); console.log.bind(console);",
            options,
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.starts_with("'use strict';"),
            "{code}",
            code = result.code
        );

        let output = run_node_source(&result.code);

        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");

        let missing_console_output = run_node_source(&format!(
            "require('node:vm').runInNewContext({}, {{}});",
            serde_json::to_string(&result.code).expect("generated code should serialize")
        ));

        assert!(
            missing_console_output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&missing_console_output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&missing_console_output.stdout), "");
        assert_eq!(String::from_utf8_lossy(&missing_console_output.stderr), "");
    }

    #[test]
    fn obfuscate_disable_console_output_false_preserves_console_methods_at_runtime() {
        let options: Options = serde_json::from_value(json!({
            "compact": true,
            "disableConsoleOutput": false,
            "propertyBracketing": false,
            "renameGlobals": false,
            "stringArray": false,
            "unicodeEscapeSequence": false
        }))
        .expect("disable console output options should deserialize");
        let result =
            obfuscate("console.log('visible');", options).expect("obfuscation should succeed");

        let output = run_node_source(&result.code);

        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), "visible\n");
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");
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
    fn options_preset_default_matches_typescript_option_surface() {
        let preset = get_options_by_preset(Preset::Default);
        let preset_object = preset.as_object().expect("preset should be an object");

        assert!(
            preset_object.len() >= 50,
            "preset should expose the broad TypeScript option surface: {preset}"
        );
        assert_eq!(preset["optionsPreset"], json!("default"));
        assert_eq!(preset["identifierNamesGenerator"], json!("hexadecimal"));
        assert_eq!(preset["stringArray"], json!(true));
        assert_eq!(preset["stringArrayRotate"], json!(true));
        assert_eq!(preset["stringArrayShuffle"], json!(true));
        assert_eq!(preset["stringArrayEncoding"], json!(["none"]));
        assert_eq!(preset["stringArrayIndexShift"], json!(true));
        assert_eq!(preset["stringArrayThreshold"], json!(0.75));
        assert_eq!(preset["target"], json!("browser"));
        assert_eq!(preset["propertyBracketing"], json!(true));
    }

    #[test]
    fn options_preset_low_medium_and_high_apply_typescript_overrides() {
        let low = get_options_by_preset(Preset::LowObfuscation);
        assert_eq!(low["optionsPreset"], json!("low-obfuscation"));
        assert_eq!(low["disableConsoleOutput"], json!(true));
        assert_eq!(low["selfDefending"], json!(true));
        assert_eq!(low["stringArrayCallsTransformThreshold"], json!(0));

        let medium = get_options_by_preset(Preset::MediumObfuscation);
        assert_eq!(medium["optionsPreset"], json!("medium-obfuscation"));
        assert_eq!(medium["controlFlowFlattening"], json!(true));
        assert_eq!(medium["deadCodeInjection"], json!(true));
        assert_eq!(medium["stringArrayEncoding"], json!(["base64"]));
        assert_eq!(medium["stringArrayWrappersType"], json!("function"));
        assert_eq!(medium["transformObjectKeys"], json!(true));

        let high = get_options_by_preset(Preset::HighObfuscation);
        assert_eq!(high["optionsPreset"], json!("high-obfuscation"));
        assert_eq!(high["debugProtection"], json!(true));
        assert_eq!(high["debugProtectionInterval"], json!(4000));
        assert_eq!(high["stringArrayEncoding"], json!(["rc4"]));
        assert_eq!(high["stringArrayWrappersCount"], json!(5));
        assert_eq!(high["stringArrayThreshold"], json!(1));
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
    fn obfuscate_transforms_computed_string_object_expression_key() {
        let result = obfuscate(
            "const value = {['foo']: bar};",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value={'foo':bar}"));
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
    fn obfuscate_split_strings_emoji_modifier_as_single_chunk() {
        let result = obfuscate(
            "const value = 'ab👋🏼cd';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                split_strings: Some(true),
                split_strings_chunk_length: Some(1),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const value='a'+'b'+'👋🏼'+'c'+'d';"),
            "{}",
            result.code
        );
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

    #[test]
    fn obfuscate_keeps_reserved_split_string_literals_inline() {
        let result = obfuscate(
            "const keep = 'please-keep-me'; const split = 'abcdef';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                reserved_strings: Some(vec!["keep".to_string()]),
                split_strings: Some(true),
                split_strings_chunk_length: Some(3),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const keep='please-keep-me';"));
        assert!(result.code.contains("const split='abc'+'def';"));
    }

    #[test]
    fn obfuscate_keeps_require_string_when_split_strings_ignore_imports_enabled() {
        let result = obfuscate(
            "const foo = require('./abcdef'); const bar = './ghijkl';",
            Options {
                compact: Some(true),
                ignore_imports: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                split_strings: Some(true),
                split_strings_chunk_length: Some(3),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("require('./abcdef')"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const bar='./g'+'hij'+'kl';"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_aliases_export_specifier_when_rename_globals_enabled() {
        let result = obfuscate(
            "const foo = 1; export {foo};",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(true),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("export{foo as foo};"));
    }

    #[test]
    fn obfuscate_keeps_export_specifier_when_rename_globals_disabled() {
        let result = obfuscate(
            "const foo = 1; export {foo};",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("export{foo};"));
    }

    #[test]
    fn obfuscate_transforms_object_pattern_when_rename_globals_enabled() {
        let result = obfuscate(
            "const {foo} = source;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(true),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const{foo:foo}=source"));
    }

    #[test]
    fn obfuscate_keeps_top_level_object_pattern_when_rename_globals_disabled() {
        let result = obfuscate(
            "const {foo} = source;",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const{foo}=source"));
    }

    #[test]
    fn obfuscate_encodes_all_string_literal_characters_when_unicode_escape_sequence_enabled() {
        let result = obfuscate(
            "const value = 'test';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(true),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value='\\x74\\x65\\x73\\x74'"));
    }

    #[test]
    fn obfuscate_encodes_forced_string_literal_characters_when_unicode_escape_sequence_disabled() {
        let result = obfuscate(
            "const value = 'hello world';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value='hello\\x20world'"));
    }

    #[test]
    fn obfuscate_preserves_program_directive_after_escape_sequences() {
        let result = obfuscate(
            "'use strict'; const value = 'hello world';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result
            .code
            .contains("'use strict';const value='hello\\x20world';"));
    }

    #[test]
    fn obfuscate_preserves_function_directive_after_escape_sequences() {
        let result = obfuscate(
            "function run(){'use strict'; const value = 'hello world';}",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result
            .code
            .contains("function run(){'use strict';const value='hello\\x20world';}"));
    }

    #[test]
    fn obfuscate_renames_labeled_statement_and_matching_references() {
        let result = obfuscate(
            "label: for (;;) { continue label; break label; }",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result
            .code
            .contains("_0x0:for(;;){continue _0x0;break _0x0;}"));
    }

    #[test]
    fn obfuscate_renames_labeled_statement_with_mangled_generator() {
        let result = obfuscate(
            "label: for (;;) { break label; }",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                identifier_names_generator: Some(
                    crate::generators::IdentifierNamesGeneratorKind::Mangled,
                ),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("a:for(;;){break a;}"));
    }

    #[test]
    fn obfuscate_keeps_labeled_statement_with_keep_original_generator() {
        let options: Options = serde_json::from_value(json!({
            "compact": true,
            "identifierNamesGenerator": "keep-original",
            "propertyBracketing": false,
            "renameGlobals": false,
            "stringArray": false
        }))
        .expect("keep-original options should deserialize");
        let result = obfuscate("label: for (;;) { break label; }", options)
            .expect("obfuscation should succeed");

        assert!(result.code.contains("label:for(;;){break label;}"));
    }

    #[test]
    fn obfuscate_keeps_reserved_labeled_statement_name() {
        let options: Options = serde_json::from_value(json!({
            "compact": true,
            "propertyBracketing": false,
            "renameGlobals": false,
            "reservedNames": ["label"],
            "stringArray": false
        }))
        .expect("reserved names options should deserialize");
        let result = obfuscate("label: for (;;) { break label; }", options)
            .expect("obfuscation should succeed");

        assert!(result.code.contains("label:for(;;){break label;}"));
    }

    #[test]
    fn obfuscate_keeps_reserved_names_regex_labeled_statement_name() {
        let options: Options = serde_json::from_value(json!({
            "compact": true,
            "propertyBracketing": false,
            "renameGlobals": false,
            "reservedNames": ["^keep"],
            "stringArray": false
        }))
        .expect("reserved names options should deserialize");
        let result = obfuscate(
            "keepLabel: for (;;) { continue keepLabel; } other: for (;;) { break other; }",
            options,
        )
        .expect("obfuscation should succeed");

        assert!(result
            .code
            .contains("keepLabel:for(;;){continue keepLabel;}"));
        assert!(result.code.contains("_0x0:for(;;){break _0x0;}"));
    }

    #[test]
    fn obfuscate_transforms_number_to_expression_when_enabled() {
        let result = obfuscate(
            "const value = 10;",
            Options {
                compact: Some(true),
                numbers_to_expressions: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value=0xb-0x1"));
    }

    #[test]
    fn obfuscate_keeps_number_literal_when_numbers_to_expressions_disabled() {
        let result = obfuscate(
            "const value = 10;",
            Options {
                compact: Some(true),
                numbers_to_expressions: Some(false),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const value=0xa"));
    }

    #[test]
    fn obfuscate_merges_expression_statements_when_simplify_enabled() {
        let result = obfuscate(
            "function foo(){bar();baz();bark();}",
            Options {
                compact: Some(true),
                simplify: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("function foo(){bar(),baz(),bark();}"));
    }

    #[test]
    fn obfuscate_keeps_expression_statements_when_simplify_disabled() {
        let result = obfuscate(
            "function foo(){bar();baz();}",
            Options {
                compact: Some(true),
                simplify: Some(false),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("function foo(){bar();baz();}"));
    }

    #[test]
    fn obfuscate_merges_variable_declarations_when_simplify_enabled() {
        let result = obfuscate(
            "var foo=1;var bar=2;var baz=3;",
            Options {
                compact: Some(true),
                simplify: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("var foo=0x1,bar=0x2,baz=0x3;"));
    }

    #[test]
    fn obfuscate_keeps_variable_declarations_when_simplify_disabled() {
        let result = obfuscate(
            "var foo=1;var bar=2;",
            Options {
                compact: Some(true),
                simplify: Some(false),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("var foo=0x1;var bar=0x2;"));
    }

    #[test]
    fn obfuscate_simplifies_trailing_return_block_when_simplify_enabled() {
        let result = obfuscate(
            "function foo(){bar();baz();return bark();}",
            Options {
                compact: Some(true),
                simplify: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result
            .code
            .contains("function foo(){return bar(),baz(),bark();}"));
    }

    #[test]
    fn obfuscate_keeps_trailing_return_block_when_simplify_disabled() {
        let result = obfuscate(
            "function foo(){bar();baz();return bark();}",
            Options {
                compact: Some(true),
                simplify: Some(false),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result
            .code
            .contains("function foo(){bar();baz();return bark();}"));
    }

    #[test]
    fn obfuscate_simplifies_if_expression_branch_when_simplify_enabled() {
        let result = obfuscate(
            "if(true){bar();baz();}",
            Options {
                compact: Some(true),
                simplify: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("!![]&&(bar(),baz());"));
    }

    #[test]
    fn obfuscate_keeps_if_expression_branch_when_simplify_disabled() {
        let result = obfuscate(
            "if(true){bar();}",
            Options {
                compact: Some(true),
                simplify: Some(false),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("if(!![]){bar();}"));
    }

    #[test]
    fn obfuscate_transforms_literal_eval_string_contents() {
        let result = obfuscate(
            "eval('console.log(true);');",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(true),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result
                .code
                .contains("eval('console[\\x27log\\x27](!![]);');"),
            "{}",
            result.code
        );
        assert!(
            !result.code.contains("console.log(true)"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_transforms_template_eval_string_contents() {
        let result = obfuscate(
            "eval(`true;`);",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("eval('!![];');"), "{}", result.code);
    }

    #[test]
    fn obfuscate_keeps_unparseable_eval_string() {
        let result = obfuscate(
            "eval('~');",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("eval('~');"), "{}", result.code);
    }

    #[test]
    fn obfuscate_keeps_reserved_string_literals_unescaped() {
        let result = obfuscate(
            "const foo = 'foo'; const bar = 'bar';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                reserved_strings: Some(vec!["foo".to_string()]),
                unicode_escape_sequence: Some(true),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("const foo='foo';"), "{}", result.code);
        assert!(
            result.code.contains("const bar='\\x62\\x61\\x72';"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_keeps_reserved_string_with_special_characters_unescaped() {
        let result = obfuscate(
            "var foo = 'bar'; var baz = 'Cannot find module \\'foo\\'';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                reserved_strings: Some(vec!["Cannot find module".to_string()]),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result
                .code
                .contains("var baz='Cannot find module \\'foo\\'';"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_keeps_static_import_string_unescaped() {
        let result = obfuscate(
            "import foo from './foo'; const bar = './bar';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(true),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("from'./foo';"), "{}", result.code);
        assert!(
            result
                .code
                .contains("const bar='\\x2e\\x2f\\x62\\x61\\x72';"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_keeps_import_call_strings_when_ignore_imports_enabled() {
        let result = obfuscate(
            "const foo = require('./foo'); const bar = './bar'; const baz = import('./baz');",
            Options {
                compact: Some(true),
                ignore_imports: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(true),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(result.code.contains("require('./foo')"), "{}", result.code);
        assert!(result.code.contains("import('./baz')"), "{}", result.code);
        assert!(
            result
                .code
                .contains("const bar='\\x2e\\x2f\\x62\\x61\\x72';"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_encodes_require_string_when_ignore_imports_disabled() {
        let result = obfuscate(
            "const foo = require('./foo');",
            Options {
                compact: Some(true),
                ignore_imports: Some(false),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(true),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("require('\\x2e\\x2f\\x66\\x6f\\x6f')"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_extracts_string_literals_when_string_array_enabled() {
        let result = obfuscate(
            "const value = 'test'; console.log('test');",
            Options {
                compact: Some(true),
                ignore_imports: Some(false),
                string_array: Some(true),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const _0x0=['test'];"),
            "{}",
            result.code
        );
        assert!(
            result
                .code
                .contains("function _0x1(index){return _0x0[index];}"),
            "{}",
            result.code
        );
        assert!(
            result
                .code
                .contains("const value=_0x1(0x0);console.log(_0x1(0x0));"),
            "{}",
            result.code
        );
        assert!(
            !result.code.contains("const value=_0x0[0x0];"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_keeps_regex_reserved_string_literals_inline() {
        let result = obfuscate(
            "const foo = 'foo'; const bar = 'bar';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                reserved_strings: Some(vec!["ar$".to_string()]),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const _0x0=['foo'];"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const foo=_0x1(0x0);"),
            "{}",
            result.code
        );
        assert!(result.code.contains("const bar='bar';"), "{}", result.code);
    }

    #[test]
    fn obfuscate_respects_string_array_minimum_length_for_normal_strings() {
        let result = obfuscate(
            "const a = 'f'; const b = 'fo'; const c = 'foo';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const _0x0=['foo'];"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const a='f';const b='fo';"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const c=_0x1(0x0);"),
            "{}",
            result.code
        );
        assert!(!result.code.contains("'f','fo','foo'"), "{}", result.code);
    }

    #[test]
    fn obfuscate_keeps_string_literals_when_string_array_disabled() {
        let result = obfuscate(
            "const value = 'test';",
            Options {
                compact: Some(true),
                string_array: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const value='test';"),
            "{}",
            result.code
        );
        assert!(!result.code.contains("const _0x0=["), "{}", result.code);
    }

    #[test]
    fn obfuscate_keeps_string_literals_when_string_array_threshold_zero() {
        let result = obfuscate(
            "const value = 'test';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(0.0),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const value='test';"),
            "{}",
            result.code
        );
        assert!(!result.code.contains("const _0x0=["), "{}", result.code);
    }

    #[test]
    fn obfuscate_force_transforms_matching_string_when_threshold_is_zero() {
        let result = obfuscate(
            "const foo = 'foo'; const bar = 'bar';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(0.0),
                force_transform_strings: Some(vec!["ar$".to_string()]),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const _0x0=['bar'];"),
            "{}",
            result.code
        );
        assert!(result.code.contains("const foo='foo';"), "{}", result.code);
        assert!(
            result.code.contains("const bar=_0x1(0x0);"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_uses_hexadecimal_number_string_array_index_type() {
        let result = obfuscate(
            "const value = 'test';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_indexes_type: Some(vec![
                    crate::options::StringArrayIndexesType::HexadecimalNumber,
                ]),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const value=_0x1(0x0);"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_uses_hexadecimal_numeric_string_array_index_type() {
        let result = obfuscate(
            "const value = 'test';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_indexes_type: Some(vec![
                    crate::options::StringArrayIndexesType::HexadecimalNumericString,
                ]),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const value=_0x1('0x0');"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_uses_string_array_index_shift_when_enabled() {
        let result = obfuscate(
            "const first = 'foo'; const second = 'bar';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_index_shift: Some(true),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result
                .code
                .contains("function _0x1(index){return _0x0[index-0x64];}"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const first=_0x1(0x64);"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const second=_0x1(0x65);"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_uses_string_array_shuffle_when_enabled() {
        let result = obfuscate(
            "const first = 'foo'; const second = 'bar';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_shuffle: Some(true),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const _0x0=['bar','foo'];"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const first=_0x1(0x1);"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const second=_0x1(0x0);"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_uses_string_array_rotate_when_enabled() {
        let result = obfuscate(
            "const first = 'foo'; const second = 'bar'; const third = 'baz';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_rotate: Some(true),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const _0x0=['baz','foo','bar'];"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const first=_0x1(0x1);"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const second=_0x1(0x2);"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const third=_0x1(0x0);"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_uses_string_array_base64_encoding_when_enabled() {
        let result = obfuscate(
            "const value = 'test';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_encoding: Some(vec![crate::options::StringArrayEncoding::Base64]),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const _0x0=['DgvZDa'];"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("function _0x1(index)"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains(
                "const chars='abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/=';"
            ),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const value=_0x1(0x0);"),
            "{}",
            result.code
        );
        assert!(!result.code.contains("['test']"), "{}", result.code);
    }

    #[test]
    fn obfuscate_uses_string_array_rc4_encoding_when_enabled() {
        let result = obfuscate(
            "const value = 'test';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_encoding: Some(vec![crate::options::StringArrayEncoding::Rc4]),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("function _0x1(index,key)"),
            "{}",
            result.code
        );
        assert!(result.code.contains("'rc4K'"), "{}", result.code);
        assert!(result.code.contains("key.charCodeAt"), "{}", result.code);
        assert!(
            result.code.contains("const value=_0x1(0x0,'rc4K');"),
            "{}",
            result.code
        );
        assert!(!result.code.contains("['test']"), "{}", result.code);
    }

    #[test]
    fn obfuscate_uses_first_supported_string_array_encoding() {
        let result = obfuscate(
            "const value = 'test';",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_encoding: Some(vec![
                    crate::options::StringArrayEncoding::Base64,
                    crate::options::StringArrayEncoding::Rc4,
                ]),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        assert!(
            result.code.contains("const _0x0=['DgvZDa'];"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("const value=_0x1(0x0);"),
            "{}",
            result.code
        );
    }

    #[test]
    fn obfuscate_none_string_array_root_wrapper_decodes_at_runtime() {
        let result = obfuscate(
            "console.log(['foo', 'bar'].join('|'));",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_index_shift: Some(false),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        let output = run_node_source(&result.code);

        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), "foo|bar\n");
    }

    #[test]
    fn obfuscate_rc4_string_array_encoding_decodes_at_runtime() {
        let result = obfuscate(
            "console.log(['foo', 'bar', 'baz'].join('|'));",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_encoding: Some(vec![crate::options::StringArrayEncoding::Rc4]),
                string_array_index_shift: Some(true),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        let output = Command::new("node")
            .arg("-e")
            .arg(&result.code)
            .output()
            .expect("node should execute generated code");

        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), "foo|bar|baz\n");
    }

    #[test]
    fn obfuscate_base64_string_array_encoding_decodes_at_runtime() {
        let result = obfuscate(
            "console.log(['f', 'fo', 'foo', 'test', '✓'].join('|'));",
            Options {
                compact: Some(true),
                string_array: Some(true),
                string_array_threshold: Some(1.0),
                string_array_encoding: Some(vec![crate::options::StringArrayEncoding::Base64]),
                string_array_index_shift: Some(true),
                rename_globals: Some(false),
                property_bracketing: Some(false),
                unicode_escape_sequence: Some(false),
                ..Options::default()
            },
        )
        .expect("obfuscation should succeed");

        let output = Command::new("node")
            .arg("-e")
            .arg(&result.code)
            .output()
            .expect("node should execute generated code");

        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), "f|fo|foo|test|✓\n");
    }
}
