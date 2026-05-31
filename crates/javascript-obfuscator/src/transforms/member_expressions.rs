use swc_common::DUMMY_SP;
use swc_ecma_ast::{ComputedPropName, Expr, Lit, MemberExpr, MemberProp, Program, Str};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_member_expressions(program: &mut Program, property_bracketing: bool) {
    if !property_bracketing {
        return;
    }

    program.visit_mut_with(&mut MemberExpressionTransform);
}

struct MemberExpressionTransform;

impl VisitMut for MemberExpressionTransform {
    fn visit_mut_member_expr(&mut self, member_expr: &mut MemberExpr) {
        member_expr.visit_mut_children_with(self);

        let MemberProp::Ident(identifier) = &member_expr.prop else {
            return;
        };

        let property_name = identifier.sym.to_string();
        member_expr.prop = MemberProp::Computed(ComputedPropName {
            span: DUMMY_SP,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: property_name.clone().into(),
                raw: Some(format!("'{property_name}'").into()),
            }))),
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, property_bracketing: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_member_expressions(&mut parsed_program.program, property_bracketing);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_dot_notation_to_bracket_notation() {
        let code = transform("const value = console.log;", true);

        assert!(code.contains("const value=console['log']"), "{code}");
    }

    #[test]
    fn keeps_computed_identifier_member_expression() {
        let code = transform("const value = console[identifier];", true);

        assert!(code.contains("const value=console[identifier]"), "{code}");
    }

    #[test]
    fn keeps_existing_computed_string_member_expression() {
        let code = transform("const value = console['log'];", true);

        assert!(code.contains("const value=console['log']"), "{code}");
    }

    #[test]
    fn skips_transform_when_property_bracketing_is_disabled() {
        let code = transform("const value = console.log;", false);

        assert!(code.contains("const value=console.log"), "{code}");
    }
}
