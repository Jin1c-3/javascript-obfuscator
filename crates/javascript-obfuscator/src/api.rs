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
}
