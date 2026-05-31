use swc_ecma_ast::{Lit, Program, PropName};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_number_literals(program: &mut Program) {
    program.visit_mut_with(&mut NumberLiteralTransform);
}

struct NumberLiteralTransform;

impl VisitMut for NumberLiteralTransform {
    fn visit_mut_lit(&mut self, literal: &mut Lit) {
        match literal {
            Lit::Num(number) => transform_number_raw(number),
            Lit::BigInt(bigint) => {
                bigint.raw = Some(format!("0x{}n", bigint.value.to_str_radix(16)).into());
            }
            _ => {}
        }
    }

    fn visit_mut_prop_name(&mut self, property_name: &mut PropName) {
        if let PropName::Num(number) = property_name {
            transform_number_raw(number);
        }
    }
}

fn should_emit_hex_number(value: f64) -> bool {
    value.is_finite() && value.fract() == 0.0
}

fn transform_number_raw(number: &mut swc_ecma_ast::Number) {
    if !should_emit_hex_number(number.value) {
        return;
    }

    let integer = number.value.abs() as u128;
    number.raw = Some(format!("0x{integer:x}").into());
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_number_literals(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_integer_number_literal_raw_value() {
        let code = transform("const value = 10;");

        assert!(code.contains("const value=0xa"), "{code}");
    }

    #[test]
    fn keeps_float_number_literal_decimal() {
        let code = transform("const value = 10.5;");

        assert!(code.contains("const value=10.5"), "{code}");
        assert!(!code.contains("0xa.5"), "{code}");
    }

    #[test]
    fn transforms_bigint_literal_raw_value() {
        let code = transform("const value = 10n;");

        assert!(code.contains("const value=0xan"), "{code}");
    }

    #[test]
    fn transforms_numeric_property_key_raw_value() {
        let code = transform("const value = {1: 'bar'};");

        assert!(code.contains("const value={0x1:'bar'}"), "{code}");
    }
}
