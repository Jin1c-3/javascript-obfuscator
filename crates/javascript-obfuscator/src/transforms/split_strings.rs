use swc_common::DUMMY_SP;
use swc_ecma_ast::{BinExpr, BinaryOp, Expr, ExprStmt, Lit, Program, Str};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_split_strings(
    program: &mut Program,
    enabled: bool,
    chunk_length: usize,
    reserved_strings: &[String],
) {
    if !enabled || chunk_length == 0 {
        return;
    }

    program.visit_mut_with(&mut SplitStringTransform {
        chunk_length,
        reserved_strings,
    });
}

struct SplitStringTransform<'a> {
    chunk_length: usize,
    reserved_strings: &'a [String],
}

impl VisitMut for SplitStringTransform<'_> {
    fn visit_mut_expr_stmt(&mut self, expression_statement: &mut ExprStmt) {
        if matches!(expression_statement.expr.as_ref(), Expr::Lit(Lit::Str(_))) {
            return;
        }

        expression_statement.visit_mut_children_with(self);
    }

    fn visit_mut_expr(&mut self, expression: &mut Expr) {
        expression.visit_mut_children_with(self);

        let Expr::Lit(Lit::Str(string_literal)) = expression else {
            return;
        };

        if is_reserved_string(
            &string_literal.value.to_string_lossy(),
            self.reserved_strings,
        ) {
            return;
        }

        if let Some(transformed_expression) =
            transform_string_literal(string_literal, self.chunk_length)
        {
            *expression = transformed_expression;
        }
    }
}

fn is_reserved_string(value: &str, reserved_strings: &[String]) -> bool {
    reserved_strings
        .iter()
        .any(|reserved_string| value.contains(reserved_string))
}

fn transform_string_literal(string_literal: &Str, chunk_length: usize) -> Option<Expr> {
    let value = string_literal.value.to_string_lossy();
    let chunks = chunk_string(&value, chunk_length);

    if chunks.len() < 2 {
        return None;
    }

    let mut iterator = chunks
        .into_iter()
        .map(|chunk| create_string_literal(&chunk));
    let first = iterator.next()?;
    let second = iterator.next()?;
    let mut root = create_add_expression(first, second);

    for chunk in iterator {
        root = create_add_expression(root, chunk);
    }

    Some(root)
}

fn chunk_string(value: &str, chunk_length: usize) -> Vec<String> {
    let characters = value.chars().collect::<Vec<_>>();

    characters
        .chunks(chunk_length)
        .map(|chunk| chunk.iter().collect())
        .collect()
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
    let escaped = value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r");

    format!("'{escaped}'")
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(
        source_code: &str,
        enabled: bool,
        chunk_length: usize,
        reserved_strings: &[String],
    ) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_split_strings(
            &mut parsed_program.program,
            enabled,
            chunk_length,
            reserved_strings,
        );
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn splits_string_literal_when_enabled() {
        let code = transform("const value = 'abcdef';", true, 3, &[]);

        assert!(code.contains("const value='abc'+'def'"), "{code}");
    }

    #[test]
    fn keeps_string_literal_when_disabled() {
        let code = transform("const value = 'abcdef';", false, 3, &[]);

        assert!(code.contains("const value='abcdef'"), "{code}");
    }

    #[test]
    fn keeps_string_literal_when_chunk_is_oversized() {
        let code = transform("const value = 'abcdef';", true, 10, &[]);

        assert!(code.contains("const value='abcdef'"), "{code}");
    }

    #[test]
    fn preserves_directive_string_statement() {
        let code = transform("'use strict'; const value = 'abcdef';", true, 3, &[]);

        assert!(code.contains("'use strict';"), "{code}");
        assert!(code.contains("const value='abc'+'def'"), "{code}");
    }

    #[test]
    fn keeps_reserved_split_string_literals_inline() {
        let reserved_strings = vec!["keep".to_string()];
        let code = transform(
            "const keep = 'please-keep-me'; const split = 'abcdef';",
            true,
            3,
            &reserved_strings,
        );

        assert!(code.contains("const keep='please-keep-me';"), "{code}");
        assert!(code.contains("const split='abc'+'def';"), "{code}");
    }
}
