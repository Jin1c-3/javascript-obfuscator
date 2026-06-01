use swc_ecma_ast::{Expr, Lit, ModuleDecl, ModuleItem, Program, Stmt};

use crate::parser::parse_program;

pub fn transform_self_defending(program: &mut Program, enabled: bool) {
    if !enabled {
        return;
    }

    insert_self_defending_helper(program, create_self_defending_statements());
}

fn insert_self_defending_helper(program: &mut Program, statements: Vec<Stmt>) {
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

fn create_self_defending_statements() -> Vec<Stmt> {
    let helper_source = r#"
        const _0xcallsController = (function () {
            let firstCall = true;

            return function (context, fn) {
                const rfn = firstCall
                    ? function () {
                        if (fn) {
                            const res = fn.apply(context, arguments);
                            fn = null;

                            return res;
                        }
                    }
                    : function () {};

                firstCall = false;

                return rfn;
            };
        })();
        const _0xselfDefending = _0xcallsController(this, function () {
            return _0xselfDefending
                .toString()
                .search('(((.+)+)+)+$')
                .toString()
                .constructor(_0xselfDefending)
                .search('(((.+)+)+)+$');
        });

        _0xselfDefending();
    "#;
    let parsed_program = parse_program(helper_source).expect("self defending helper should parse");

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

    fn transform(source_code: &str, enabled: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");

        transform_self_defending(&mut parsed_program.program, enabled);

        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("transformed code should generate")
    }

    #[test]
    fn prepends_self_defending_helper_when_enabled() {
        let code = transform("globalThis.result = 'ran';", true);

        assert!(
            code.find("_0xselfDefending")
                .expect("helper should be present")
                < code
                    .find("globalThis.result")
                    .expect("user code should be present"),
            "{code}"
        );
        assert!(code.contains("(((.+)+)+)+$"), "{code}");
        assert!(code.contains("_0xcallsController"), "{code}");
    }

    #[test]
    fn skips_self_defending_helper_when_disabled() {
        let code = transform("globalThis.result = 'ran';", false);

        assert_eq!(code, "globalThis.result='ran';");
    }

    #[test]
    fn inserts_self_defending_helper_after_imports() {
        let code = transform("import value from 'pkg'; console.log(value);", true);

        assert!(
            code.find("import value from")
                .expect("import should be present")
                < code
                    .find("_0xselfDefending")
                    .expect("helper should be present"),
            "{code}"
        );
    }

    #[test]
    fn inserts_self_defending_helper_after_directives() {
        let code = transform("'use strict'; globalThis.result = this;", true);

        assert!(code.starts_with("'use strict';"), "{code}");
        assert!(
            code.find("'use strict'")
                .expect("directive should be present")
                < code
                    .find("_0xselfDefending")
                    .expect("helper should be present"),
            "{code}"
        );
    }
}
