pub mod boolean_literals;
pub mod class_fields;
pub mod member_expressions;
pub mod number_literals;
pub mod object_expressions;
pub mod split_strings;
pub mod template_literals;

use swc_ecma_ast::Program;

use crate::options::Options;

pub fn apply_transforms(program: &mut Program, options: &Options) {
    template_literals::transform_template_literals(program);
    boolean_literals::transform_boolean_literals(program);
    number_literals::transform_number_literals(program);
    class_fields::transform_class_fields(program, options.reserved_names.as_deref().unwrap_or(&[]));
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
}
