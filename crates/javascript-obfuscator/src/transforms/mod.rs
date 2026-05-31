pub mod boolean_literals;

use swc_ecma_ast::Program;

use crate::options::Options;

pub fn apply_transforms(program: &mut Program, _options: &Options) {
    boolean_literals::transform_boolean_literals(program);
}
