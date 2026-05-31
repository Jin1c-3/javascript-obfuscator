use swc_common::DUMMY_SP;
use swc_ecma_ast::{
    BinExpr, BinaryOp, BlockStmt, CondExpr, Decl, Expr, ExprStmt, IfStmt, Lit, ParenExpr, Program,
    ReturnStmt, SeqExpr, Stmt, VarDeclKind,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_if_statement_simplify(program: &mut Program, enabled: bool) {
    if !enabled {
        return;
    }

    program.visit_mut_with(&mut IfStatementSimplifyTransform);
}

struct IfStatementSimplifyTransform;

impl VisitMut for IfStatementSimplifyTransform {
    fn visit_mut_stmt(&mut self, statement: &mut Stmt) {
        statement.visit_mut_children_with(self);

        let Stmt::If(if_statement) = statement else {
            return;
        };

        if let Some(transformed_statement) = transform_if_statement(if_statement) {
            *statement = transformed_statement;
        }
    }
}

#[derive(Clone)]
struct TrailingStatement {
    statement: Stmt,
    expression: Expr,
}

struct StatementSimplifyData {
    leading_statements: Vec<Stmt>,
    trailing_statement: Option<TrailingStatement>,
    has_return_statement: bool,
}

fn transform_if_statement(if_statement: &IfStmt) -> Option<Stmt> {
    let consequent_data = statement_simplify_data(&if_statement.cons)?;

    if let Some(alternate_statement) = if_statement.alt.as_deref() {
        let alternate_data = statement_simplify_data(alternate_statement)?;

        return Some(transform_consequent_and_alternate(
            if_statement,
            &consequent_data,
            &alternate_data,
        ));
    }

    Some(transform_consequent(if_statement, &consequent_data))
}

fn transform_consequent(if_statement: &IfStmt, consequent_data: &StatementSimplifyData) -> Stmt {
    let Some(trailing_statement) = &consequent_data.trailing_statement else {
        return create_if_statement(
            if_statement.test.as_ref().clone(),
            partial_if_branch_statement(consequent_data),
            None,
        );
    };

    if !consequent_data.leading_statements.is_empty() {
        return create_if_statement(
            if_statement.test.as_ref().clone(),
            partial_if_branch_statement(consequent_data),
            None,
        );
    }

    if consequent_data.has_return_statement {
        return create_if_statement(
            if_statement.test.as_ref().clone(),
            trailing_statement.statement.clone(),
            None,
        );
    }

    create_expression_statement(create_logical_and(
        if_statement.test.as_ref().clone(),
        trailing_statement.expression.clone(),
    ))
}

fn transform_consequent_and_alternate(
    if_statement: &IfStmt,
    consequent_data: &StatementSimplifyData,
    alternate_data: &StatementSimplifyData,
) -> Stmt {
    let Some(consequent_trailing_statement) = &consequent_data.trailing_statement else {
        return create_if_statement(
            if_statement.test.as_ref().clone(),
            partial_if_branch_statement(consequent_data),
            Some(partial_if_branch_statement(alternate_data)),
        );
    };
    let Some(alternate_trailing_statement) = &alternate_data.trailing_statement else {
        return create_if_statement(
            if_statement.test.as_ref().clone(),
            partial_if_branch_statement(consequent_data),
            Some(partial_if_branch_statement(alternate_data)),
        );
    };

    if !consequent_data.leading_statements.is_empty()
        || !alternate_data.leading_statements.is_empty()
    {
        return create_if_statement(
            if_statement.test.as_ref().clone(),
            partial_if_branch_statement(consequent_data),
            Some(partial_if_branch_statement(alternate_data)),
        );
    }

    if consequent_data.has_return_statement && alternate_data.has_return_statement {
        return create_return_statement(create_conditional(
            if_statement.test.as_ref().clone(),
            consequent_trailing_statement.expression.clone(),
            alternate_trailing_statement.expression.clone(),
        ));
    }

    if consequent_data.has_return_statement || alternate_data.has_return_statement {
        return create_if_statement(
            if_statement.test.as_ref().clone(),
            consequent_trailing_statement.statement.clone(),
            Some(alternate_trailing_statement.statement.clone()),
        );
    }

    create_expression_statement(create_conditional(
        if_statement.test.as_ref().clone(),
        consequent_trailing_statement.expression.clone(),
        alternate_trailing_statement.expression.clone(),
    ))
}

fn statement_simplify_data(statement: &Stmt) -> Option<StatementSimplifyData> {
    let Stmt::Block(block_statement) = statement else {
        return Some(StatementSimplifyData {
            leading_statements: vec![statement.clone()],
            trailing_statement: None,
            has_return_statement: false,
        });
    };

    Some(collect_block_simplify_data(block_statement))
}

fn collect_block_simplify_data(block_statement: &BlockStmt) -> StatementSimplifyData {
    let mut unwrapped_expressions = Vec::new();
    let mut has_return_statement = false;
    let mut has_statements_after_return_statement = false;
    let mut start_index = None;
    let statement_count = block_statement.stmts.len();

    for index in (0..statement_count).rev() {
        let statement = &block_statement.stmts[index];

        if let Stmt::Expr(expression_statement) = statement {
            if matches!(expression_statement.expr.as_ref(), Expr::Lit(Lit::Str(_))) {
                break;
            }

            let mut expressions = collect_expression_statement(expression_statement);
            expressions.append(&mut unwrapped_expressions);
            unwrapped_expressions = expressions;
            start_index = Some(index);
            continue;
        }

        if let Stmt::Return(return_statement) = statement {
            if let Some(argument) = &return_statement.arg {
                unwrapped_expressions.insert(0, argument.as_ref().clone());
                has_return_statement = true;
                has_statements_after_return_statement = index != statement_count - 1;
                start_index = Some(index);
                continue;
            }
        }

        break;
    }

    if has_statements_after_return_statement {
        return StatementSimplifyData {
            leading_statements: block_statement.stmts.clone(),
            trailing_statement: None,
            has_return_statement: false,
        };
    }

    let leading_statements = match start_index {
        Some(0) => vec![],
        Some(index) => block_statement.stmts[..index].to_vec(),
        None => block_statement.stmts.clone(),
    };

    if unwrapped_expressions.is_empty() {
        return StatementSimplifyData {
            leading_statements,
            trailing_statement: None,
            has_return_statement,
        };
    }

    let expression = create_sequence_or_single(unwrapped_expressions);
    let statement = if has_return_statement {
        create_return_statement(expression.clone())
    } else {
        create_expression_statement(expression.clone())
    };

    StatementSimplifyData {
        leading_statements,
        trailing_statement: Some(TrailingStatement {
            statement,
            expression,
        }),
        has_return_statement,
    }
}

fn collect_expression_statement(expression_statement: &ExprStmt) -> Vec<Expr> {
    match expression_statement.expr.as_ref() {
        Expr::Seq(sequence_expression) => sequence_expression
            .exprs
            .iter()
            .map(|expression| expression.as_ref().clone())
            .collect(),
        expression => vec![expression.clone()],
    }
}

fn partial_statement(data: &StatementSimplifyData) -> Stmt {
    if data.leading_statements.is_empty() {
        if let Some(trailing_statement) = &data.trailing_statement {
            return trailing_statement.statement.clone();
        }
    }

    let mut statements = data.leading_statements.clone();

    if let Some(trailing_statement) = &data.trailing_statement {
        statements.push(trailing_statement.statement.clone());
    }

    Stmt::Block(BlockStmt {
        span: DUMMY_SP,
        ctxt: Default::default(),
        stmts: statements,
    })
}

fn partial_if_branch_statement(data: &StatementSimplifyData) -> Stmt {
    let statement = partial_statement(data);

    let Stmt::Block(block_statement) = statement else {
        return statement;
    };

    if block_statement.stmts.len() == 1 {
        let statement = block_statement
            .stmts
            .first()
            .expect("single statement should exist")
            .clone();

        if !is_prohibited_single_if_branch_statement(&statement) {
            return statement;
        }
    }

    Stmt::Block(block_statement)
}

fn is_prohibited_single_if_branch_statement(statement: &Stmt) -> bool {
    match statement {
        Stmt::Decl(Decl::Fn(_)) | Stmt::If(_) => true,
        Stmt::For(_)
        | Stmt::ForIn(_)
        | Stmt::ForOf(_)
        | Stmt::While(_)
        | Stmt::DoWhile(_)
        | Stmt::Labeled(_) => true,
        Stmt::Decl(Decl::Var(variable_declaration)) => {
            !matches!(variable_declaration.kind, VarDeclKind::Var)
        }
        _ => false,
    }
}

fn create_if_statement(test: Expr, consequent: Stmt, alternate: Option<Stmt>) -> Stmt {
    Stmt::If(IfStmt {
        span: DUMMY_SP,
        test: Box::new(test),
        cons: Box::new(consequent),
        alt: alternate.map(Box::new),
    })
}

fn create_expression_statement(expression: Expr) -> Stmt {
    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(expression),
    })
}

fn create_return_statement(expression: Expr) -> Stmt {
    Stmt::Return(ReturnStmt {
        span: DUMMY_SP,
        arg: Some(Box::new(expression)),
    })
}

fn create_logical_and(left: Expr, right: Expr) -> Expr {
    Expr::Bin(BinExpr {
        span: DUMMY_SP,
        op: BinaryOp::LogicalAnd,
        left: Box::new(left),
        right: Box::new(parenthesize_sequence_expression(right)),
    })
}

fn create_conditional(test: Expr, consequent: Expr, alternate: Expr) -> Expr {
    Expr::Cond(CondExpr {
        span: DUMMY_SP,
        test: Box::new(test),
        cons: Box::new(consequent),
        alt: Box::new(alternate),
    })
}

fn create_sequence_or_single(mut expressions: Vec<Expr>) -> Expr {
    if expressions.len() == 1 {
        return expressions
            .pop()
            .expect("single-expression sequence should contain expression");
    }

    Expr::Seq(SeqExpr {
        span: DUMMY_SP,
        exprs: expressions.into_iter().map(Box::new).collect(),
    })
}

fn parenthesize_sequence_expression(expression: Expr) -> Expr {
    if !matches!(expression, Expr::Seq(_)) {
        return expression;
    }

    Expr::Paren(ParenExpr {
        span: DUMMY_SP,
        expr: Box::new(expression),
    })
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, enabled: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_if_statement_simplify(&mut parsed_program.program, enabled);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn simplifies_consequent_only_expression_branch() {
        let code = transform("if(true){bar();baz();}", true);

        assert!(code.contains("true&&(bar(),baz());"), "{code}");
    }

    #[test]
    fn simplifies_return_branches_to_conditional_return() {
        let code = transform("if(true){return bar();}else{return baz();}", true);

        assert!(code.contains("return true?bar():baz();"), "{code}");
    }

    #[test]
    fn preserves_leading_statements_in_branch_blocks() {
        let code = transform("if(true){const value=1;bar();return baz();}", true);

        assert!(
            code.contains("if(true){const value=1;return bar(),baz();}"),
            "{code}"
        );
    }

    #[test]
    fn keeps_prohibited_const_declaration_branch_block() {
        let code = transform("if(true){const value=1;}", true);

        assert!(code.contains("if(true){const value=1;}"), "{code}");
    }

    #[test]
    fn keeps_if_statement_when_disabled() {
        let code = transform("if(true){bar();}else{baz();}", false);

        assert!(code.contains("if(true){bar();}else{baz();}"), "{code}");
    }
}
