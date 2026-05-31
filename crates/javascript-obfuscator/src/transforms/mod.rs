pub mod boolean_literals;
pub mod member_expressions;
pub mod number_literals;

use swc_ecma_ast::Program;

use crate::options::Options;

pub fn apply_transforms(program: &mut Program, options: &Options) {
    boolean_literals::transform_boolean_literals(program);
    number_literals::transform_number_literals(program);
    member_expressions::transform_member_expressions(
        program,
        options.property_bracketing.unwrap_or(true),
    );
}
