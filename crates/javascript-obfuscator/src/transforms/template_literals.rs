use swc_common::DUMMY_SP;
use swc_ecma_ast::{BinExpr, BinaryOp, Expr, Lit, Program, Str, TaggedTpl, Tpl};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_template_literals(program: &mut Program) {
    program.visit_mut_with(&mut TemplateLiteralTransform);
}

struct TemplateLiteralTransform;

impl VisitMut for TemplateLiteralTransform {
    fn visit_mut_tagged_tpl(&mut self, tagged_tpl: &mut TaggedTpl) {
        tagged_tpl.tag.visit_mut_with(self);
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);

        let Expr::Tpl(template_literal) = expr else {
            return;
        };

        if let Some(transformed_expression) = transform_template_literal(template_literal) {
            *expr = transformed_expression;
        }
    }
}

fn transform_template_literal(template_literal: &Tpl) -> Option<Expr> {
    let mut nodes = Vec::new();

    for (index, quasi) in template_literal.quasis.iter().enumerate() {
        let Some(cooked) = quasi
            .cooked
            .as_ref()
            .map(|value| value.to_string_lossy().into_owned())
        else {
            continue;
        };

        nodes.push(create_string_literal(&cooked));

        if let Some(expression) = template_literal.exprs.get(index) {
            nodes.push((**expression).clone());
        }
    }

    nodes.retain(|node| !matches!(node, Expr::Lit(Lit::Str(value)) if value.value.is_empty()));

    if !is_string_literal(nodes.first()) && !is_string_literal(nodes.get(1)) {
        nodes.insert(0, create_string_literal(""));
    }

    let mut iterator = nodes.into_iter();
    let first = iterator.next()?;
    let Some(second) = iterator.next() else {
        return Some(first);
    };

    let mut root = create_add_expression(first, second);

    for node in iterator {
        root = create_add_expression(root, node);
    }

    Some(root)
}

fn create_add_expression(left: Expr, right: Expr) -> Expr {
    Expr::Bin(BinExpr {
        span: DUMMY_SP,
        op: BinaryOp::Add,
        left: Box::new(left),
        right: Box::new(right),
    })
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
            '\n' => escaped.push_str("\\x0a"),
            '\r' => escaped.push_str("\\x0d"),
            '\t' => escaped.push_str("\\x09"),
            ' ' => escaped.push_str("\\x20"),
            '\u{2028}' => escaped.push_str("\\u2028"),
            '\u{2029}' => escaped.push_str("\\u2029"),
            character if character.is_ascii_control() => {
                escaped.push_str(&format!("\\x{:02x}", character as u32));
            }
            character => escaped.push(character),
        }
    }

    format!("'{escaped}'")
}

fn is_string_literal(node: Option<&Expr>) -> bool {
    matches!(node, Some(Expr::Lit(Lit::Str(_))))
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_template_literals(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_simple_template_literal_with_expression() {
        let code = transform("const value = `abc ${foo}`;");

        assert!(code.contains("const value='abc\\x20'+foo"), "{code}");
        assert!(!code.contains('`'), "{code}");
    }

    #[test]
    fn transforms_expression_only_template_literal() {
        let code = transform("const value = `${foo}`;");

        assert!(code.contains("const value=''+foo"), "{code}");
    }

    #[test]
    fn transforms_literal_only_template_literal() {
        let code = transform("const value = `abc`;");

        assert!(code.contains("const value='abc'"), "{code}");
    }

    #[test]
    fn transforms_multiline_template_literal() {
        let code = transform("const value = `abc\nfoo`;");

        assert!(code.contains("const value='abc\\x0afoo'"), "{code}");
    }

    #[test]
    fn keeps_tagged_template_literal() {
        let code = transform("tag`abc ${foo}`;");

        assert!(code.contains("tag`abc ${foo}`"), "{code}");
    }
}
