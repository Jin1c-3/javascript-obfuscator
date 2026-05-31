use swc_ecma_ast::{ArrowExpr, BlockStmtOrExpr, Expr, Function, Lit, ModuleItem, Program, Stmt};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_directive_placement(program: &mut Program) {
    match program {
        Program::Module(module) => restore_module_directives(&mut module.body),
        Program::Script(script) => restore_statement_directives(&mut script.body),
    }

    program.visit_mut_with(&mut DirectivePlacementTransform);
}

struct DirectivePlacementTransform;

impl VisitMut for DirectivePlacementTransform {
    fn visit_mut_arrow_expr(&mut self, arrow_expression: &mut ArrowExpr) {
        if let BlockStmtOrExpr::BlockStmt(body) = arrow_expression.body.as_mut() {
            restore_statement_directives(&mut body.stmts);
        }

        arrow_expression.visit_mut_children_with(self);
    }

    fn visit_mut_function(&mut self, function: &mut Function) {
        if let Some(body) = &mut function.body {
            restore_statement_directives(&mut body.stmts);
        }

        function.visit_mut_children_with(self);
    }
}

fn restore_module_directives(module_items: &mut [ModuleItem]) {
    for module_item in module_items {
        let ModuleItem::Stmt(statement) = module_item else {
            break;
        };

        if !restore_directive_statement(statement) {
            break;
        }
    }
}

fn restore_statement_directives(statements: &mut [Stmt]) {
    for statement in statements {
        if !restore_directive_statement(statement) {
            break;
        }
    }
}

fn restore_directive_statement(statement: &mut Stmt) -> bool {
    let Stmt::Expr(expression_statement) = statement else {
        return false;
    };

    let Expr::Lit(Lit::Str(string_literal)) = expression_statement.expr.as_mut() else {
        return false;
    };

    let value = string_literal.value.to_string_lossy();
    string_literal.raw = Some(directive_raw(&value).into());

    true
}

fn directive_raw(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");

    format!("'{escaped}'")
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;
    use crate::transforms::escape_sequences::transform_escape_sequences;

    use super::*;

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_escape_sequences(&mut parsed_program.program, false, &[]);
        transform_directive_placement(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn preserves_program_scope_directive_after_escape_sequences() {
        let code = transform("'use strict'; const value = 'hello world';");

        assert!(
            code.contains("'use strict';const value='hello\\x20world';"),
            "{code}"
        );
    }

    #[test]
    fn keeps_middle_directive_like_string_escaped() {
        let code = transform("const value = 'hello world'; 'use strict';");

        assert!(
            code.contains("const value='hello\\x20world';'use\\x20strict';"),
            "{code}"
        );
    }

    #[test]
    fn preserves_function_scope_directive_after_escape_sequences() {
        let code = transform("function run(){'use strict'; const value = 'hello world';}");

        assert!(
            code.contains("function run(){'use strict';const value='hello\\x20world';}"),
            "{code}"
        );
    }
}
