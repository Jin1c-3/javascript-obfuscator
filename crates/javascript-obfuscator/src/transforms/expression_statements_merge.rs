use swc_common::DUMMY_SP;
use swc_ecma_ast::{
    BlockStmt, Expr, ExprStmt, Lit, ModuleItem, Program, SeqExpr, Stmt, SwitchCase,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_expression_statements_merge(program: &mut Program, enabled: bool) {
    if !enabled {
        return;
    }

    program.visit_mut_with(&mut ExpressionStatementsMergeTransform);
}

struct ExpressionStatementsMergeTransform;

impl VisitMut for ExpressionStatementsMergeTransform {
    fn visit_mut_program(&mut self, program: &mut Program) {
        program.visit_mut_children_with(self);

        match program {
            Program::Module(module) => merge_module_items(&mut module.body),
            Program::Script(script) => merge_statements(&mut script.body),
        }
    }

    fn visit_mut_block_stmt(&mut self, block_statement: &mut BlockStmt) {
        block_statement.visit_mut_children_with(self);
        merge_statements(&mut block_statement.stmts);
    }

    fn visit_mut_switch_case(&mut self, switch_case: &mut SwitchCase) {
        switch_case.visit_mut_children_with(self);
        merge_statements(&mut switch_case.cons);
    }
}

fn merge_module_items(module_items: &mut Vec<ModuleItem>) {
    let mut merged_module_items = Vec::with_capacity(module_items.len());
    let mut expressions = Vec::new();

    for module_item in std::mem::take(module_items) {
        match module_item {
            ModuleItem::Stmt(statement) if is_mergeable_expression_statement(&statement) => {
                push_expression(&mut expressions, statement_into_expression(statement));
            }
            ModuleItem::Stmt(statement) => {
                flush_expressions(&mut merged_module_items, &mut expressions, ModuleItem::Stmt);
                merged_module_items.push(ModuleItem::Stmt(statement));
            }
            module_item => {
                flush_expressions(&mut merged_module_items, &mut expressions, ModuleItem::Stmt);
                merged_module_items.push(module_item);
            }
        }
    }

    flush_expressions(&mut merged_module_items, &mut expressions, ModuleItem::Stmt);
    *module_items = merged_module_items;
}

fn merge_statements(statements: &mut Vec<Stmt>) {
    let mut merged_statements = Vec::with_capacity(statements.len());
    let mut expressions = Vec::new();

    for statement in std::mem::take(statements) {
        if is_mergeable_expression_statement(&statement) {
            push_expression(&mut expressions, statement_into_expression(statement));
            continue;
        }

        flush_expressions(&mut merged_statements, &mut expressions, |statement| {
            statement
        });
        merged_statements.push(statement);
    }

    flush_expressions(&mut merged_statements, &mut expressions, |statement| {
        statement
    });
    *statements = merged_statements;
}

fn flush_expressions<T>(
    output: &mut Vec<T>,
    expressions: &mut Vec<Expr>,
    wrap_statement: impl FnOnce(Stmt) -> T,
) {
    if expressions.is_empty() {
        return;
    }

    output.push(wrap_statement(create_expression_statement(std::mem::take(
        expressions,
    ))));
}

fn is_mergeable_expression_statement(statement: &Stmt) -> bool {
    let Stmt::Expr(expression_statement) = statement else {
        return false;
    };

    !matches!(expression_statement.expr.as_ref(), Expr::Lit(Lit::Str(_)))
}

fn statement_into_expression(statement: Stmt) -> Expr {
    let Stmt::Expr(expression_statement) = statement else {
        unreachable!("caller should only pass expression statements");
    };

    *expression_statement.expr
}

fn create_expression_statement(mut expressions: Vec<Expr>) -> Stmt {
    let expression = if expressions.len() == 1 {
        Box::new(
            expressions
                .pop()
                .expect("single-expression group should contain expression"),
        )
    } else {
        Box::new(Expr::Seq(SeqExpr {
            span: DUMMY_SP,
            exprs: expressions.into_iter().map(Box::new).collect(),
        }))
    };

    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: expression,
    })
}

fn push_expression(expressions: &mut Vec<Expr>, expression: Expr) {
    match expression {
        Expr::Seq(sequence_expression) => {
            expressions.extend(
                sequence_expression
                    .exprs
                    .into_iter()
                    .map(|expression| *expression),
            );
        }
        expression => expressions.push(expression),
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, enabled: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_expression_statements_merge(&mut parsed_program.program, enabled);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn merges_adjacent_expression_statements_when_enabled() {
        let code = transform("function foo(){bar();baz();bark();}", true);

        assert!(
            code.contains("function foo(){bar(),baz(),bark();}"),
            "{code}"
        );
    }

    #[test]
    fn keeps_adjacent_expression_statements_when_disabled() {
        let code = transform("function foo(){bar();baz();}", false);

        assert!(code.contains("function foo(){bar();baz();}"), "{code}");
    }

    #[test]
    fn preserves_directive_before_expression_group() {
        let code = transform("function foo(){'use strict';bar();baz();}", true);

        assert!(
            code.contains("function foo(){'use strict';bar(),baz();}"),
            "{code}"
        );
    }

    #[test]
    fn merges_only_groups_separated_by_non_expression_statements() {
        let code = transform(
            "function foo(){a();function bar(){}b();c();const value=1;d();}",
            true,
        );

        assert!(
            code.contains("function foo(){a();function bar(){}b(),c();const value=1;d();}"),
            "{code}"
        );
    }
}
