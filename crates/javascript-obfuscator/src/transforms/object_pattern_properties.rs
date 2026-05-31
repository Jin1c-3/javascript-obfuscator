use swc_common::DUMMY_SP;
use swc_ecma_ast::{
    ArrowExpr, AssignPat, AssignPatProp, Function, KeyValuePatProp, ObjectPat, ObjectPatProp, Pat,
    Program, PropName, StaticBlock,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_object_pattern_properties(program: &mut Program, rename_globals: bool) {
    program.visit_mut_with(&mut ObjectPatternPropertiesTransform {
        rename_globals,
        function_depth: 0,
        static_block_depth: 0,
    });
}

struct ObjectPatternPropertiesTransform {
    rename_globals: bool,
    function_depth: usize,
    static_block_depth: usize,
}

impl VisitMut for ObjectPatternPropertiesTransform {
    fn visit_mut_arrow_expr(&mut self, arrow_expression: &mut ArrowExpr) {
        self.function_depth += 1;
        arrow_expression.visit_mut_children_with(self);
        self.function_depth -= 1;
    }

    fn visit_mut_function(&mut self, function: &mut Function) {
        self.function_depth += 1;
        function.visit_mut_children_with(self);
        self.function_depth -= 1;
    }

    fn visit_mut_object_pat(&mut self, object_pattern: &mut ObjectPat) {
        object_pattern.visit_mut_children_with(self);

        if !self.should_transform_current_scope() {
            return;
        }

        for property in &mut object_pattern.props {
            let ObjectPatProp::Assign(assign_property) = property else {
                continue;
            };

            *property = create_key_value_property(assign_property);
        }
    }

    fn visit_mut_static_block(&mut self, static_block: &mut StaticBlock) {
        self.static_block_depth += 1;
        static_block.visit_mut_children_with(self);
        self.static_block_depth -= 1;
    }
}

impl ObjectPatternPropertiesTransform {
    fn should_transform_current_scope(&self) -> bool {
        self.rename_globals || self.function_depth > 0 || self.static_block_depth > 0
    }
}

fn create_key_value_property(assign_property: &AssignPatProp) -> ObjectPatProp {
    let binding_identifier = assign_property.key.clone();
    let key = PropName::from(binding_identifier.id.clone());
    let value = match &assign_property.value {
        Some(default_value) => Pat::Assign(AssignPat {
            span: DUMMY_SP,
            left: Box::new(Pat::Ident(binding_identifier)),
            right: default_value.clone(),
        }),
        None => Pat::Ident(binding_identifier),
    };

    ObjectPatProp::KeyValue(KeyValuePatProp {
        key,
        value: Box::new(value),
    })
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, rename_globals: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_object_pattern_properties(&mut parsed_program.program, rename_globals);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_top_level_object_pattern_when_rename_globals_is_enabled() {
        let code = transform("const {foo} = source;", true);

        assert!(code.contains("const{foo:foo}=source"), "{code}");
    }

    #[test]
    fn keeps_top_level_object_pattern_when_rename_globals_is_disabled() {
        let code = transform("const {foo} = source;", false);

        assert!(code.contains("const{foo}=source"), "{code}");
    }

    #[test]
    fn transforms_function_scope_object_pattern_when_rename_globals_is_disabled() {
        let code = transform("function run({foo}) {}", false);

        assert!(code.contains("function run({foo:foo}){}"), "{code}");
    }

    #[test]
    fn transforms_default_value_object_pattern() {
        let code = transform("const {foo = 1} = source;", true);

        assert!(code.contains("const{foo:foo=1}=source"), "{code}");
    }
}
