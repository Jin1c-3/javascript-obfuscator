use regex::Regex;
use swc_common::DUMMY_SP;
use swc_ecma_ast::{BinExpr, BinaryOp, CallExpr, Callee, Expr, ExprStmt, Lit, Program, Str};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_split_strings(
    program: &mut Program,
    enabled: bool,
    chunk_length: usize,
    reserved_strings: &[String],
    ignore_imports: bool,
) {
    if !enabled || chunk_length == 0 {
        return;
    }

    program.visit_mut_with(&mut SplitStringTransform {
        chunk_length,
        reserved_string_patterns: compile_patterns(reserved_strings),
        ignore_imports,
    });
}

struct SplitStringTransform {
    chunk_length: usize,
    reserved_string_patterns: Vec<Regex>,
    ignore_imports: bool,
}

impl VisitMut for SplitStringTransform {
    fn visit_mut_expr_stmt(&mut self, expression_statement: &mut ExprStmt) {
        if matches!(expression_statement.expr.as_ref(), Expr::Lit(Lit::Str(_))) {
            return;
        }

        expression_statement.visit_mut_children_with(self);
    }

    fn visit_mut_call_expr(&mut self, call_expression: &mut CallExpr) {
        if self.ignore_imports && is_ignored_import_call(call_expression) {
            return;
        }

        call_expression.visit_mut_children_with(self);
    }

    fn visit_mut_expr(&mut self, expression: &mut Expr) {
        expression.visit_mut_children_with(self);

        let Expr::Lit(Lit::Str(string_literal)) = expression else {
            return;
        };

        if is_reserved_string(
            &string_literal.value.to_string_lossy(),
            &self.reserved_string_patterns,
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

fn is_reserved_string(value: &str, reserved_string_patterns: &[Regex]) -> bool {
    reserved_string_patterns
        .iter()
        .any(|reserved_string_pattern| reserved_string_pattern.is_match(value))
}

fn compile_patterns(patterns: &[String]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect()
}

fn is_ignored_import_call(call_expression: &CallExpr) -> bool {
    is_dynamic_import_call(call_expression) || is_require_call(call_expression)
}

fn is_dynamic_import_call(call_expression: &CallExpr) -> bool {
    matches!(call_expression.callee, Callee::Import(_))
}

fn is_require_call(call_expression: &CallExpr) -> bool {
    let Callee::Expr(callee_expression) = &call_expression.callee else {
        return false;
    };
    let Expr::Ident(identifier) = callee_expression.as_ref() else {
        return false;
    };

    identifier.sym.as_ref() == "require"
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
        ignore_imports: bool,
    ) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_split_strings(
            &mut parsed_program.program,
            enabled,
            chunk_length,
            reserved_strings,
            ignore_imports,
        );
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn splits_string_literal_when_enabled() {
        let code = transform("const value = 'abcdef';", true, 3, &[], false);

        assert!(code.contains("const value='abc'+'def'"), "{code}");
    }

    #[test]
    fn keeps_string_literal_when_disabled() {
        let code = transform("const value = 'abcdef';", false, 3, &[], false);

        assert!(code.contains("const value='abcdef'"), "{code}");
    }

    #[test]
    fn keeps_string_literal_when_chunk_is_oversized() {
        let code = transform("const value = 'abcdef';", true, 10, &[], false);

        assert!(code.contains("const value='abcdef'"), "{code}");
    }

    #[test]
    fn preserves_directive_string_statement() {
        let code = transform("'use strict'; const value = 'abcdef';", true, 3, &[], false);

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
            false,
        );

        assert!(code.contains("const keep='please-keep-me';"), "{code}");
        assert!(code.contains("const split='abc'+'def';"), "{code}");
    }

    #[test]
    fn keeps_regex_reserved_split_string_literals_inline() {
        let reserved_strings = vec!["ar$".to_string()];
        let code = transform(
            "const foo = 'foofoo'; const bar = 'barbar';",
            true,
            3,
            &reserved_strings,
            false,
        );

        assert!(code.contains("const foo='foo'+'foo';"), "{code}");
        assert!(code.contains("const bar='barbar';"), "{code}");
    }

    #[test]
    fn keeps_require_string_when_split_strings_ignore_imports_enabled() {
        let code = transform(
            "const foo = require('./abcdef'); const bar = './ghijkl';",
            true,
            3,
            &[],
            true,
        );

        assert!(code.contains("require('./abcdef')"), "{code}");
        assert!(code.contains("const bar='./g'+'hij'+'kl';"), "{code}");
    }

    #[test]
    fn keeps_dynamic_import_string_when_split_strings_ignore_imports_enabled() {
        let code = transform(
            "const mod = import('./abcdef'); const bar = './ghijkl';",
            true,
            3,
            &[],
            true,
        );

        assert!(code.contains("import('./abcdef')"), "{code}");
        assert!(code.contains("const bar='./g'+'hij'+'kl';"), "{code}");
    }
}
