pub mod boolean_literals;
pub mod number_literals;

use swc_ecma_ast::Program;

use crate::options::Options;

pub fn apply_transforms(program: &mut Program, _options: &Options) {
    boolean_literals::transform_boolean_literals(program);
    number_literals::transform_number_literals(program);
}
