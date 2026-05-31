use swc_common::DUMMY_SP;
use swc_ecma_ast::{BinExpr, BinaryOp, Expr, Lit, Number, Program};
use swc_ecma_visit::{VisitMut, VisitMutWith};

const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

pub fn transform_number_to_expressions(program: &mut Program, enabled: bool) {
    if !enabled {
        return;
    }

    program.visit_mut_with(&mut NumberToExpressionTransform);
}

struct NumberToExpressionTransform;

impl VisitMut for NumberToExpressionTransform {
    fn visit_mut_expr(&mut self, expression: &mut Expr) {
        expression.visit_mut_children_with(self);

        let Expr::Lit(Lit::Num(number)) = expression else {
            return;
        };

        if let Some(transformed_expression) = transform_number_expression(number) {
            *expression = transformed_expression;
        }
    }
}

fn transform_number_expression(number: &Number) -> Option<Expr> {
    let value = number.value;

    if !value.is_finite() {
        return None;
    }

    if value.fract() == 0.0 {
        if value.abs() > MAX_SAFE_INTEGER {
            return None;
        }

        return Some(create_integer_expression(value));
    }

    Some(create_fractional_expression(value))
}

fn create_integer_expression(value: f64) -> Expr {
    create_sub_expression(
        create_number_literal(value + 1.0),
        create_number_literal(1.0),
    )
}

fn create_fractional_expression(value: f64) -> Expr {
    let integer_part = value.trunc();
    let decimal_part = value - integer_part;

    create_add_expression(
        create_number_literal(integer_part),
        create_number_literal(decimal_part),
    )
}

fn create_number_literal(value: f64) -> Expr {
    Expr::Lit(Lit::Num(Number {
        span: DUMMY_SP,
        value,
        raw: Some(number_raw(value).into()),
    }))
}

fn create_sub_expression(left: Expr, right: Expr) -> Expr {
    Expr::Bin(BinExpr {
        span: DUMMY_SP,
        op: BinaryOp::Sub,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn create_add_expression(left: Expr, right: Expr) -> Expr {
    Expr::Bin(BinExpr {
        span: DUMMY_SP,
        op: BinaryOp::Add,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn number_raw(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        return format!("0x{:x}", value.abs() as u128);
    }

    value.to_string()
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;
    use crate::transforms::number_literals::transform_number_literals;

    use super::*;

    fn transform(source_code: &str, enabled: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_number_literals(&mut parsed_program.program);
        transform_number_to_expressions(&mut parsed_program.program, enabled);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_integer_number_when_enabled() {
        let code = transform("const value = 10;", true);

        assert!(code.contains("const value=0xb-0x1"), "{code}");
    }

    #[test]
    fn keeps_integer_number_when_disabled() {
        let code = transform("const value = 10;", false);

        assert!(code.contains("const value=0xa"), "{code}");
    }

    #[test]
    fn transforms_float_number_when_enabled() {
        let code = transform("const value = 50.5;", true);

        assert!(code.contains("const value=0x32+0.5"), "{code}");
    }

    #[test]
    fn keeps_non_computed_object_property_key_as_number_literal() {
        let code = transform("const value = {1: 'bar'};", true);

        assert!(code.contains("const value={0x1:'bar'}"), "{code}");
    }

    #[test]
    fn keeps_unsafe_integer_as_number_literal() {
        let code = transform("const value = 9007199254740992;", true);

        assert!(code.contains("const value=0x20000000000000"), "{code}");
    }
}
