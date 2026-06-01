pub mod block_statement_simplify;
pub mod boolean_literals;
pub mod class_fields;
pub mod console_output;
pub mod directive_placement;
pub mod escape_sequences;
pub mod eval_call_expressions;
pub mod export_specifiers;
pub mod expression_statements_merge;
pub mod if_statement_simplify;
pub mod labeled_statements;
pub mod member_expressions;
pub mod number_literals;
pub mod number_to_expressions;
pub mod object_expression_keys;
pub mod object_expressions;
pub mod object_pattern_properties;
pub mod rename_properties;
pub mod split_strings;
pub mod string_array;
pub mod template_literals;
pub mod variable_declarations_merge;

use swc_ecma_ast::Program;

use crate::options::{Options, StringArrayEncoding};
use crate::storages::IdentifierNamesCacheStorage;

pub fn apply_transforms(
    program: &mut Program,
    options: &Options,
    identifier_names_cache_storage: Option<&mut IdentifierNamesCacheStorage>,
) {
    console_output::transform_console_output(
        program,
        options.disable_console_output.unwrap_or(false),
    );
    export_specifiers::transform_export_specifiers(
        program,
        options.rename_globals.unwrap_or(false),
    );
    template_literals::transform_template_literals(program);
    boolean_literals::transform_boolean_literals(program);
    number_literals::transform_number_literals(program);
    number_to_expressions::transform_number_to_expressions(
        program,
        options.numbers_to_expressions.unwrap_or(false),
    );
    class_fields::transform_class_fields(program, options.reserved_names.as_deref().unwrap_or(&[]));
    object_pattern_properties::transform_object_pattern_properties(
        program,
        options.rename_globals.unwrap_or(false),
    );
    object_expressions::transform_object_expressions(program);
    object_expression_keys::transform_object_expression_keys(
        program,
        options.transform_object_keys.unwrap_or(false),
        options.identifier_names_generator.unwrap_or_default(),
        options.identifiers_prefix.as_deref().unwrap_or(""),
        options.identifiers_dictionary.as_deref().unwrap_or(&[]),
    );
    rename_properties::transform_rename_properties(
        program,
        rename_properties::RenamePropertiesTransformOptions {
            enabled: options.rename_properties.unwrap_or(false),
            mode: options.rename_properties_mode.as_deref(),
            generator_kind: options.identifier_names_generator.unwrap_or_default(),
            identifiers_prefix: options.identifiers_prefix.as_deref().unwrap_or(""),
            identifiers_dictionary: options.identifiers_dictionary.as_deref().unwrap_or(&[]),
            reserved_names: options.reserved_names.as_deref().unwrap_or(&[]),
            identifier_names_cache_storage,
        },
    );
    split_strings::transform_split_strings(
        program,
        options.split_strings.unwrap_or(false),
        options.split_strings_chunk_length.unwrap_or(10),
        options.reserved_strings.as_deref().unwrap_or(&[]),
        options.ignore_imports.unwrap_or(false),
    );
    member_expressions::transform_member_expressions(
        program,
        options.property_bracketing.unwrap_or(true),
    );
    labeled_statements::transform_labeled_statements(
        program,
        options.identifier_names_generator.unwrap_or_default(),
        options.identifiers_prefix.as_deref().unwrap_or(""),
        options.identifiers_dictionary.as_deref().unwrap_or(&[]),
        options.reserved_names.as_deref().unwrap_or(&[]),
    );
    expression_statements_merge::transform_expression_statements_merge(
        program,
        options.simplify.unwrap_or(false),
    );
    variable_declarations_merge::transform_variable_declarations_merge(
        program,
        options.simplify.unwrap_or(false),
    );
    block_statement_simplify::transform_block_statement_simplify(
        program,
        options.simplify.unwrap_or(false),
    );
    if_statement_simplify::transform_if_statement_simplify(
        program,
        options.simplify.unwrap_or(false),
    );
    eval_call_expressions::transform_eval_call_expressions(program, options);
    string_array::transform_string_array(
        program,
        string_array::StringArrayTransformOptions {
            enabled: options.string_array.unwrap_or(false),
            threshold: options.string_array_threshold.unwrap_or(1.0),
            indexes_type: options.string_array_indexes_type.as_deref().unwrap_or(&[]),
            encoding: select_supported_string_array_encoding(options),
            index_shift: options.string_array_index_shift.unwrap_or(false),
            shuffle: options.string_array_shuffle.unwrap_or(false),
            rotate: options.string_array_rotate.unwrap_or(false),
            reserved_strings: options.reserved_strings.as_deref().unwrap_or(&[]),
            force_transform_strings: options.force_transform_strings.as_deref().unwrap_or(&[]),
            ignore_imports: options.ignore_imports.unwrap_or(false),
            wrappers_count: options.string_array_wrappers_count.unwrap_or(0),
            wrappers_type: options.string_array_wrappers_type.unwrap_or_default(),
        },
    );
    escape_sequences::transform_escape_sequences(
        program,
        options.unicode_escape_sequence.unwrap_or(false),
        options.reserved_strings.as_deref().unwrap_or(&[]),
        options.ignore_imports.unwrap_or(false),
    );
    directive_placement::transform_directive_placement(program);
}

fn select_supported_string_array_encoding(options: &Options) -> StringArrayEncoding {
    options
        .string_array_encoding
        .as_deref()
        .and_then(|encodings| {
            encodings.iter().copied().find(|encoding| {
                matches!(
                    encoding,
                    StringArrayEncoding::None
                        | StringArrayEncoding::Base64
                        | StringArrayEncoding::Rc4
                )
            })
        })
        .unwrap_or_default()
}
