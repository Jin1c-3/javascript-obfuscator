use swc_ecma_ast::{BlockStmt, Expr, Lit, ModuleItem, Program, Stmt, SwitchCase};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::parser::parse_program;

pub fn transform_control_flow_flattening(program: &mut Program, enabled: bool, threshold: f64) {
    if !enabled || threshold <= 0.0 {
        return;
    }

    program.visit_mut_with(&mut ControlFlowFlatteningTransform);
}

struct ControlFlowFlatteningTransform;

impl VisitMut for ControlFlowFlatteningTransform {
    fn visit_mut_block_stmt(&mut self, block_statement: &mut BlockStmt) {
        block_statement.visit_mut_children_with(self);

        if let Some(flattened_statements) = flatten_block_statements(&block_statement.stmts) {
            block_statement.stmts = flattened_statements;
        }
    }
}

fn flatten_block_statements(statements: &[Stmt]) -> Option<Vec<Stmt>> {
    let directive_count = statements
        .iter()
        .take_while(|statement| is_directive_statement(statement))
        .count();
    let candidate_statements = &statements[directive_count..];

    if candidate_statements.len() < 5 || !candidate_statements.iter().all(is_flattenable_statement)
    {
        return None;
    }

    let mut flattened_statements = statements[..directive_count].to_vec();
    flattened_statements.extend(create_flattened_statements(candidate_statements.to_vec()));

    Some(flattened_statements)
}

fn is_directive_statement(statement: &Stmt) -> bool {
    matches!(
        statement,
        Stmt::Expr(expression_statement)
            if matches!(expression_statement.expr.as_ref(), Expr::Lit(Lit::Str(_)))
    )
}

fn is_flattenable_statement(statement: &Stmt) -> bool {
    matches!(
        statement,
        Stmt::Expr(expression_statement)
            if !matches!(expression_statement.expr.as_ref(), Expr::Lit(Lit::Str(_)))
    )
}

fn create_flattened_statements(statements: Vec<Stmt>) -> Vec<Stmt> {
    let cases_source = (0..statements.len())
        .map(|index| format!("case {index}:continue;"))
        .collect::<Vec<_>>()
        .join("");
    let helper_source = format!(
        "var _0xcontrolFlowIndex=0;while(true){{switch(_0xcontrolFlowIndex++){{{cases_source}default:break;}}break;}}"
    );
    let parsed_program =
        parse_program(&helper_source).expect("control flow flattening helper should parse");
    let mut body = program_statements(parsed_program.program);

    let switch_cases = get_flattening_switch_cases(&mut body)
        .expect("control flow helper should contain switch cases");

    for (switch_case, statement) in switch_cases
        .iter_mut()
        .take(statements.len())
        .zip(statements)
    {
        switch_case.cons.insert(0, statement);
    }

    body
}

fn program_statements(program: Program) -> Vec<Stmt> {
    match program {
        Program::Script(script) => script.body,
        Program::Module(module) => module
            .body
            .into_iter()
            .filter_map(|item| match item {
                ModuleItem::Stmt(statement) => Some(statement),
                ModuleItem::ModuleDecl(_) => None,
            })
            .collect(),
    }
}

fn get_flattening_switch_cases(statements: &mut [Stmt]) -> Option<&mut Vec<SwitchCase>> {
    let Stmt::While(while_statement) = statements.get_mut(1)? else {
        return None;
    };
    let Stmt::Block(while_body) = while_statement.body.as_mut() else {
        return None;
    };
    let Stmt::Switch(switch_statement) = while_body.stmts.get_mut(0)? else {
        return None;
    };

    Some(&mut switch_statement.cases)
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, enabled: bool, threshold: f64) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");

        transform_control_flow_flattening(&mut parsed_program.program, enabled, threshold);

        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("transformed code should generate")
    }

    #[test]
    fn flattens_simple_expression_block_when_enabled() {
        let code = transform("function run(){a();b();c();d();e();}", true, 1.0);

        assert!(code.contains("_0xcontrolFlowIndex"), "{code}");
        assert!(code.contains("switch(_0xcontrolFlowIndex++)"), "{code}");
        assert!(code.contains("case 0:a();continue;"), "{code}");
        assert!(code.contains("case 4:e();continue;"), "{code}");
    }

    #[test]
    fn skips_flattening_when_disabled() {
        let code = transform("function run(){a();b();c();d();e();}", false, 1.0);

        assert_eq!(code, "function run(){a();b();c();d();e();}");
    }

    #[test]
    fn skips_flattening_when_threshold_is_zero() {
        let code = transform("function run(){a();b();c();d();e();}", true, 0.0);

        assert_eq!(code, "function run(){a();b();c();d();e();}");
    }

    #[test]
    fn preserves_directives_before_flattened_body() {
        let code = transform(
            "function run(){'use strict';a();b();c();d();e();}",
            true,
            1.0,
        );

        assert!(code.contains("function run(){'use strict';"), "{code}");
        assert!(
            code.find("'use strict'")
                .expect("directive should be present")
                < code
                    .find("_0xcontrolFlowIndex")
                    .expect("flattened block should be present"),
            "{code}"
        );
    }

    #[test]
    fn skips_blocks_with_non_expression_statements_for_first_slice() {
        let code = transform("function run(){const value=1;a();b();}", true, 1.0);

        assert_eq!(code, "function run(){const value=1;a();b();}");
    }

    #[test]
    fn skips_blocks_with_fewer_than_five_candidate_statements() {
        let code = transform("function run(){a();b();c();d();}", true, 1.0);

        assert_eq!(code, "function run(){a();b();c();d();}");
    }
}
