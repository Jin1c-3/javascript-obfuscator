use swc_common::DUMMY_SP;
use swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, Lit, Program, Str, Tpl};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::codegen::generate_code;
use crate::options::Options;
use crate::parser::parse_program;

use super::apply_transforms;

pub fn transform_eval_call_expressions(program: &mut Program, options: &Options) {
    program.visit_mut_with(&mut EvalCallExpressionTransform { options });
}

struct EvalCallExpressionTransform<'a> {
    options: &'a Options,
}

impl VisitMut for EvalCallExpressionTransform<'_> {
    fn visit_mut_call_expr(&mut self, call_expression: &mut CallExpr) {
        call_expression.visit_mut_children_with(self);

        if !is_direct_eval_call(call_expression) {
            return;
        }

        let Some(first_argument) = call_expression.args.first_mut() else {
            return;
        };
        let Some(eval_source) = extract_eval_source(first_argument) else {
            return;
        };
        let Some(transformed_source) = transform_eval_source(&eval_source, self.options) else {
            return;
        };

        *first_argument.expr = create_string_literal(&transformed_source);
    }
}

fn is_direct_eval_call(call_expression: &CallExpr) -> bool {
    let Callee::Expr(callee_expression) = &call_expression.callee else {
        return false;
    };
    let Expr::Ident(identifier) = callee_expression.as_ref() else {
        return false;
    };

    identifier.sym.as_ref() == "eval"
}

fn extract_eval_source(argument: &ExprOrSpread) -> Option<String> {
    if argument.spread.is_some() {
        return None;
    }

    match argument.expr.as_ref() {
        Expr::Lit(Lit::Str(string_literal)) => {
            Some(string_literal.value.to_string_lossy().into_owned())
        }
        Expr::Tpl(template_literal) => extract_template_literal_source(template_literal),
        _ => None,
    }
}

fn extract_template_literal_source(template_literal: &Tpl) -> Option<String> {
    if !template_literal.exprs.is_empty() || template_literal.quasis.len() != 1 {
        return None;
    }

    template_literal.quasis[0]
        .cooked
        .as_ref()
        .map(|value| value.to_string_lossy().into_owned())
}

fn transform_eval_source(eval_source: &str, options: &Options) -> Option<String> {
    let mut parsed_program = parse_program(eval_source).ok()?;

    apply_transforms(&mut parsed_program.program, options, None);

    let generated_code =
        generate_code(&parsed_program.program, parsed_program.source_map, true).ok()?;

    if generated_code.contains("<invalid>") {
        return None;
    }

    Some(generated_code)
}

fn create_string_literal(value: &str) -> Expr {
    Expr::Lit(Lit::Str(Str {
        span: DUMMY_SP,
        value: value.to_string().into(),
        raw: Some(single_quote_raw(value).into()),
    }))
}

fn single_quote_raw(value: &str) -> String {
    let mut escaped = String::new();

    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            character => escaped.push(character),
        }
    }

    format!("'{escaped}'")
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_eval_call_expressions(&mut parsed_program.program, &Options::default());
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_literal_eval_string_contents() {
        let code = transform("eval('true;');");

        assert!(code.contains("eval('!![];');"), "{code}");
    }

    #[test]
    fn keeps_non_literal_eval_argument() {
        let code = transform("eval(source);");

        assert!(code.contains("eval(source);"), "{code}");
    }

    #[test]
    fn keeps_unparseable_eval_string() {
        let code = transform("eval('~');");

        assert!(code.contains("eval('~');"), "{code}");
    }
}
