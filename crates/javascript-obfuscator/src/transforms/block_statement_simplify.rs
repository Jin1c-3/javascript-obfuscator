use swc_common::DUMMY_SP;
use swc_ecma_ast::{BlockStmt, Expr, ExprStmt, Lit, Program, ReturnStmt, SeqExpr, Stmt};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_block_statement_simplify(program: &mut Program, enabled: bool) {
    if !enabled {
        return;
    }

    program.visit_mut_with(&mut BlockStatementSimplifyTransform);
}

struct BlockStatementSimplifyTransform;

impl VisitMut for BlockStatementSimplifyTransform {
    fn visit_mut_block_stmt(&mut self, block_statement: &mut BlockStmt) {
        block_statement.visit_mut_children_with(self);
        simplify_block_statement(block_statement);
    }
}

fn simplify_block_statement(block_statement: &mut BlockStmt) {
    if block_statement.stmts.len() < 2 {
        return;
    }

    let Some(return_argument) = final_return_argument(block_statement) else {
        return;
    };

    let mut expressions = Vec::new();
    let mut start_index = block_statement.stmts.len() - 1;

    for index in (0..block_statement.stmts.len() - 1).rev() {
        let Stmt::Expr(expression_statement) = &block_statement.stmts[index] else {
            break;
        };

        let mut current_expressions = Vec::new();

        if !collect_expression_statement(expression_statement, &mut current_expressions) {
            break;
        }

        current_expressions.append(&mut expressions);
        expressions = current_expressions;
        start_index = index;
    }

    if expressions.is_empty() {
        return;
    }

    expressions.push(return_argument);
    let return_statement = create_return_statement(create_sequence_expression(expressions));
    block_statement
        .stmts
        .splice(start_index.., [return_statement]);
}

fn final_return_argument(block_statement: &BlockStmt) -> Option<Expr> {
    let Stmt::Return(return_statement) = block_statement.stmts.last()? else {
        return None;
    };

    return_statement
        .arg
        .as_ref()
        .map(|argument| argument.as_ref().clone())
}

fn collect_expression_statement(
    expression_statement: &ExprStmt,
    expressions: &mut Vec<Expr>,
) -> bool {
    match expression_statement.expr.as_ref() {
        Expr::Lit(Lit::Str(_)) => false,
        Expr::Seq(sequence_expression) => {
            expressions.extend(
                sequence_expression
                    .exprs
                    .iter()
                    .map(|expression| expression.as_ref().clone()),
            );

            true
        }
        expression => {
            expressions.push(expression.clone());
            true
        }
    }
}

fn create_return_statement(argument: Box<Expr>) -> Stmt {
    Stmt::Return(ReturnStmt {
        span: DUMMY_SP,
        arg: Some(argument),
    })
}

fn create_sequence_expression(expressions: Vec<Expr>) -> Box<Expr> {
    Box::new(Expr::Seq(SeqExpr {
        span: DUMMY_SP,
        exprs: expressions.into_iter().map(Box::new).collect(),
    }))
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, enabled: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_block_statement_simplify(&mut parsed_program.program, enabled);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn merges_trailing_expressions_into_return_when_enabled() {
        let code = transform("function foo(){bar();baz();return bark();}", true);

        assert!(
            code.contains("function foo(){return bar(),baz(),bark();}"),
            "{code}"
        );
    }

    #[test]
    fn keeps_trailing_expressions_before_return_when_disabled() {
        let code = transform("function foo(){bar();baz();return bark();}", false);

        assert!(
            code.contains("function foo(){bar();baz();return bark();}"),
            "{code}"
        );
    }

    #[test]
    fn preserves_leading_non_expression_statements() {
        let code = transform("function foo(){const value=1;bar();return baz();}", true);

        assert!(
            code.contains("function foo(){const value=1;return bar(),baz();}"),
            "{code}"
        );
    }

    #[test]
    fn keeps_block_when_return_is_not_last_statement() {
        let code = transform("function foo(){return bar();baz();}", true);

        assert!(
            code.contains("function foo(){return bar();baz();}"),
            "{code}"
        );
    }

    #[test]
    fn preserves_directive_before_simplified_return() {
        let code = transform("function foo(){'use strict';bar();return baz();}", true);

        assert!(
            code.contains("function foo(){'use strict';return bar(),baz();}"),
            "{code}"
        );
    }
}
