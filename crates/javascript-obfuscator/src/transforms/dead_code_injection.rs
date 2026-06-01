use swc_ecma_ast::{Expr, Lit, ModuleDecl, ModuleItem, Program, Stmt};

use crate::parser::parse_program;

pub fn transform_dead_code_injection(program: &mut Program, enabled: bool, threshold: f64) {
    if !enabled || threshold <= 0.0 {
        return;
    }

    insert_dead_code(program, create_dead_code_statements());
}

fn insert_dead_code(program: &mut Program, statements: Vec<Stmt>) {
    match program {
        Program::Script(script) => {
            let insert_index = first_non_directive_statement_index(&script.body);

            script.body.splice(insert_index..insert_index, statements);
        }
        Program::Module(module) => {
            let first_non_import_index = module
                .body
                .iter()
                .position(|item| !matches!(item, ModuleItem::ModuleDecl(ModuleDecl::Import(_))))
                .unwrap_or(module.body.len());
            let directive_count = module.body[first_non_import_index..]
                .iter()
                .take_while(|item| {
                    matches!(item, ModuleItem::Stmt(statement) if is_directive_statement(statement))
                })
                .count();
            let insert_index = first_non_import_index + directive_count;

            module.body.splice(
                insert_index..insert_index,
                statements.into_iter().map(ModuleItem::Stmt),
            );
        }
    }
}

fn first_non_directive_statement_index(statements: &[Stmt]) -> usize {
    statements
        .iter()
        .position(|statement| !is_directive_statement(statement))
        .unwrap_or(statements.len())
}

fn is_directive_statement(statement: &Stmt) -> bool {
    matches!(
        statement,
        Stmt::Expr(expression_statement)
            if matches!(expression_statement.expr.as_ref(), Expr::Lit(Lit::Str(_)))
    )
}

fn create_dead_code_statements() -> Vec<Stmt> {
    let helper_source = r#"
        if ('_0xdeadCode' === '_0xliveCode') {
            const _0xdeadCodeInjection = function (_0xvalue) {
                const _0xnoise = _0xvalue ? _0xvalue.length : 0;

                return _0xnoise + 1;
            };

            _0xdeadCodeInjection('dead-code');
        }
    "#;
    let parsed_program = parse_program(helper_source).expect("dead code helper should parse");

    match parsed_program.program {
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

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, enabled: bool, threshold: f64) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");

        transform_dead_code_injection(&mut parsed_program.program, enabled, threshold);

        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("transformed code should generate")
    }

    #[test]
    fn prepends_dead_code_when_enabled_and_threshold_is_positive() {
        let code = transform("globalThis.result = 'ran';", true, 1.0);

        assert!(
            code.find("_0xdeadCodeInjection")
                .expect("dead code should be present")
                < code
                    .find("globalThis.result")
                    .expect("user code should be present"),
            "{code}"
        );
    }

    #[test]
    fn skips_dead_code_when_disabled() {
        let code = transform("globalThis.result = 'ran';", false, 1.0);

        assert_eq!(code, "globalThis.result='ran';");
    }

    #[test]
    fn skips_dead_code_when_threshold_is_zero() {
        let code = transform("globalThis.result = 'ran';", true, 0.0);

        assert_eq!(code, "globalThis.result='ran';");
    }

    #[test]
    fn inserts_dead_code_after_imports() {
        let code = transform("import value from 'pkg'; console.log(value);", true, 1.0);

        assert!(
            code.find("import value from")
                .expect("import should be present")
                < code.find("if(").expect("dead code should be present"),
            "{code}"
        );
    }

    #[test]
    fn inserts_dead_code_after_directives() {
        let code = transform("'use strict'; globalThis.result = this;", true, 1.0);

        assert!(code.starts_with("'use strict';"), "{code}");
        assert!(
            code.find("'use strict'")
                .expect("directive should be present")
                < code.find("if(").expect("dead code should be present"),
            "{code}"
        );
    }
}
