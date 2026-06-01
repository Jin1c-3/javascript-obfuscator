use swc_ecma_ast::{Expr, Lit, ModuleDecl, ModuleItem, Program, Stmt};

use crate::parser::parse_program;

pub fn transform_debug_protection(program: &mut Program, enabled: bool, interval: usize) {
    if !enabled {
        return;
    }

    insert_debug_protection_helper(program, create_debug_protection_statements(interval));
}

fn insert_debug_protection_helper(program: &mut Program, statements: Vec<Stmt>) {
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

fn create_debug_protection_statements(interval: usize) -> Vec<Stmt> {
    let interval_source = if interval > 0 {
        format!(
            r#"
                if (typeof _0xdebugProtectionGlobal.setInterval === 'function') {{
                    _0xdebugProtectionGlobal.setInterval(_0xdebugProtection, {interval});
                }}
            "#
        )
    } else {
        String::new()
    };
    let helper_source = format!(
        r#"
            (function () {{
                const _0xdebugProtectionGlobal = typeof globalThis !== 'undefined'
                    ? globalThis
                    : typeof self !== 'undefined'
                        ? self
                        : typeof window !== 'undefined'
                            ? window
                            : typeof global !== 'undefined'
                                ? global
                                : this;
                const _0xdebugProtection = function () {{
                    debugger;
                }};

                _0xdebugProtection();
                {interval_source}
            }})();
        "#
    );
    let parsed_program =
        parse_program(&helper_source).expect("debug protection helper should parse");

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

    fn transform(source_code: &str, enabled: bool, interval: usize) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");

        transform_debug_protection(&mut parsed_program.program, enabled, interval);

        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("transformed code should generate")
    }

    #[test]
    fn prepends_debug_protection_helper_when_enabled() {
        let code = transform("globalThis.result = 'ran';", true, 0);

        assert!(
            code.find("debugger").expect("helper should be present")
                < code
                    .find("globalThis.result")
                    .expect("user code should be present"),
            "{code}"
        );
        assert!(!code.contains("setInterval"), "{code}");
    }

    #[test]
    fn adds_interval_call_when_interval_is_configured() {
        let code = transform("globalThis.result = 'ran';", true, 250);

        assert!(code.contains("setInterval"), "{code}");
        assert!(code.contains(",250"), "{code}");
    }

    #[test]
    fn skips_debug_protection_helper_when_disabled() {
        let code = transform("globalThis.result = 'ran';", false, 250);

        assert_eq!(code, "globalThis.result='ran';");
    }

    #[test]
    fn inserts_debug_protection_helper_after_imports() {
        let code = transform("import value from 'pkg'; console.log(value);", true, 0);

        assert!(
            code.find("import value from")
                .expect("import should be present")
                < code.find("(function(){").expect("helper should be present"),
            "{code}"
        );
    }

    #[test]
    fn inserts_debug_protection_helper_after_directives() {
        let code = transform("'use strict'; globalThis.result = this;", true, 0);

        assert!(code.starts_with("'use strict';"), "{code}");
        assert!(
            code.find("'use strict'")
                .expect("directive should be present")
                < code.find("(function(){").expect("helper should be present"),
            "{code}"
        );
    }
}
