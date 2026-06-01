use swc_ecma_ast::{Expr, Lit, ModuleDecl, ModuleItem, Program, Stmt};

use crate::parser::parse_program;

pub fn transform_console_output(program: &mut Program, enabled: bool) {
    if !enabled {
        return;
    }

    insert_console_output_disable_helper(program, create_console_output_disable_statements());
}

fn insert_console_output_disable_helper(program: &mut Program, statements: Vec<Stmt>) {
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
                .take_while(|item| matches!(item, ModuleItem::Stmt(statement) if is_directive_statement(statement)))
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

fn create_console_output_disable_statements() -> Vec<Stmt> {
    let helper_source = r#"
        (function () {
            const _0xconsoleMethods = ['log', 'warn', 'info', 'error', 'exception', 'table', 'trace'];
            const _0xglobal = typeof globalThis !== 'undefined'
                ? globalThis
                : typeof self !== 'undefined'
                    ? self
                    : typeof window !== 'undefined'
                        ? window
                        : typeof global !== 'undefined'
                            ? global
                            : this;
            const _0xconsoleObject = (_0xglobal.console = _0xglobal.console || {});

            for (let _0xindex = 0; _0xindex < _0xconsoleMethods.length; _0xindex++) {
                const _0xmethodName = _0xconsoleMethods[_0xindex];
                const _0xoriginalFunction = _0xconsoleObject[_0xmethodName];
                const _0xoriginalToString = _0xoriginalFunction && _0xoriginalFunction.toString;
                const _0xnoop = function () {};

                _0xnoop.toString = _0xoriginalToString
                    ? _0xoriginalToString.bind(_0xoriginalFunction)
                    : _0xnoop.toString;
                _0xconsoleObject[_0xmethodName] = _0xnoop;
            }
        })();
    "#;
    let parsed_program = parse_program(helper_source).expect("console output helper should parse");

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

        transform_console_output(&mut parsed_program.program, enabled);

        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("transformed code should generate")
    }

    #[test]
    fn prepends_console_output_disable_helper_when_enabled() {
        let code = transform("console.log('visible');", true);

        assert!(
            code.find("(function(){").expect("helper should be present")
                < code
                    .find("console.log")
                    .expect("user code should be present"),
            "{code}"
        );
        assert!(code.contains("exception"), "{code}");
        assert!(code.contains("console.log('visible')"), "{code}");
    }

    #[test]
    fn skips_console_output_disable_helper_when_disabled() {
        let code = transform("console.log('visible');", false);

        assert_eq!(code, "console.log('visible');");
    }

    #[test]
    fn inserts_console_output_disable_helper_after_imports() {
        let code = transform("import value from 'pkg'; console.log(value);", true);

        assert!(
            code.find("import value from")
                .expect("import should be present")
                < code.find("(function(){").expect("helper should be present"),
            "{code}"
        );
    }

    #[test]
    fn inserts_console_output_disable_helper_after_directives() {
        let code = transform("'use strict'; console.log(this);", true);

        assert!(code.starts_with("'use strict';"), "{code}");
        assert!(
            code.find("'use strict'")
                .expect("directive should be present")
                < code.find("(function(){").expect("helper should be present"),
            "{code}"
        );
    }
}
