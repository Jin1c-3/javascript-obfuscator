pub mod block_statement_simplify;
pub mod boolean_literals;
pub mod class_fields;
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
pub mod object_expressions;
pub mod object_pattern_properties;
pub mod split_strings;
pub mod string_array;
pub mod template_literals;
pub mod variable_declarations_merge;

use swc_ecma_ast::Program;

use crate::options::Options;

pub fn apply_transforms(program: &mut Program, options: &Options) {
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
    split_strings::transform_split_strings(
        program,
        options.split_strings.unwrap_or(false),
        options.split_strings_chunk_length.unwrap_or(10),
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
        options.string_array.unwrap_or(false),
        options.string_array_threshold.unwrap_or(1.0),
        options.string_array_indexes_type.as_deref().unwrap_or(&[]),
        options.string_array_index_shift.unwrap_or(false),
        options.reserved_strings.as_deref().unwrap_or(&[]),
        options.ignore_imports.unwrap_or(false),
    );
    escape_sequences::transform_escape_sequences(
        program,
        options.unicode_escape_sequence.unwrap_or(false),
        options.reserved_strings.as_deref().unwrap_or(&[]),
        options.ignore_imports.unwrap_or(false),
    );
    directive_placement::transform_directive_placement(program);
}
