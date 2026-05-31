use swc_common::DUMMY_SP;
use swc_ecma_ast::{ArrayLit, Expr, Lit, Program, UnaryExpr, UnaryOp};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_boolean_literals(program: &mut Program) {
    program.visit_mut_with(&mut BooleanLiteralTransform);
}

struct BooleanLiteralTransform;

impl VisitMut for BooleanLiteralTransform {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);

        let Expr::Lit(Lit::Bool(boolean_literal)) = expr else {
            return;
        };

        *expr = create_boolean_expression(boolean_literal.value);
    }
}

fn create_boolean_expression(value: bool) -> Expr {
    if value {
        Expr::Unary(UnaryExpr {
            span: DUMMY_SP,
            op: UnaryOp::Bang,
            arg: Box::new(create_boolean_expression(false)),
        })
    } else {
        Expr::Unary(UnaryExpr {
            span: DUMMY_SP,
            op: UnaryOp::Bang,
            arg: Box::new(Expr::Array(ArrayLit {
                span: DUMMY_SP,
                elems: Vec::new(),
            })),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_boolean_literals(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_true_boolean_literal() {
        let code = transform("const value = true;");

        assert!(code.contains("const value=!![]"));
        assert!(!code.contains("true"));
    }

    #[test]
    fn transforms_false_boolean_literal() {
        let code = transform("const value = false;");

        assert!(code.contains("const value=![]"));
        assert!(!code.contains("false"));
    }

    #[test]
    fn transforms_nested_boolean_literals() {
        let code = transform("if (true) { console.log(false); }");

        assert!(code.contains("if(!![])"));
        assert!(code.contains("console.log(![])"));
    }
}
