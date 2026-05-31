use swc_ecma_ast::{BlockStmt, Decl, ModuleItem, Program, Stmt, SwitchCase, VarDecl};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_variable_declarations_merge(program: &mut Program, enabled: bool) {
    if !enabled {
        return;
    }

    program.visit_mut_with(&mut VariableDeclarationsMergeTransform);
}

struct VariableDeclarationsMergeTransform;

impl VisitMut for VariableDeclarationsMergeTransform {
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
    let mut active_declaration = None;

    for module_item in std::mem::take(module_items) {
        match module_item {
            ModuleItem::Stmt(statement) => match try_take_var_declaration(statement) {
                Ok(variable_declaration) => {
                    if let Some(previous_declaration) =
                        push_var_declaration(&mut active_declaration, variable_declaration)
                    {
                        merged_module_items
                            .push(ModuleItem::Stmt(create_var_statement(previous_declaration)));
                    }
                }
                Err(statement) => {
                    flush_var_declaration(
                        &mut merged_module_items,
                        &mut active_declaration,
                        ModuleItem::Stmt,
                    );
                    merged_module_items.push(ModuleItem::Stmt(statement));
                }
            },
            module_item => {
                flush_var_declaration(
                    &mut merged_module_items,
                    &mut active_declaration,
                    ModuleItem::Stmt,
                );
                merged_module_items.push(module_item);
            }
        }
    }

    flush_var_declaration(
        &mut merged_module_items,
        &mut active_declaration,
        ModuleItem::Stmt,
    );
    *module_items = merged_module_items;
}

fn merge_statements(statements: &mut Vec<Stmt>) {
    let mut merged_statements = Vec::with_capacity(statements.len());
    let mut active_declaration = None;

    for statement in std::mem::take(statements) {
        match try_take_var_declaration(statement) {
            Ok(variable_declaration) => {
                if let Some(previous_declaration) =
                    push_var_declaration(&mut active_declaration, variable_declaration)
                {
                    merged_statements.push(create_var_statement(previous_declaration));
                }
            }
            Err(statement) => {
                flush_var_declaration(
                    &mut merged_statements,
                    &mut active_declaration,
                    |statement| statement,
                );
                merged_statements.push(statement);
            }
        }
    }

    flush_var_declaration(
        &mut merged_statements,
        &mut active_declaration,
        |statement| statement,
    );
    *statements = merged_statements;
}

fn push_var_declaration(
    active_declaration: &mut Option<VarDecl>,
    mut variable_declaration: VarDecl,
) -> Option<VarDecl> {
    let Some(active_declaration) = active_declaration else {
        *active_declaration = Some(variable_declaration);
        return None;
    };

    if active_declaration.kind == variable_declaration.kind {
        active_declaration
            .decls
            .append(&mut variable_declaration.decls);
        return None;
    }

    Some(std::mem::replace(active_declaration, variable_declaration))
}

fn try_take_var_declaration(statement: Stmt) -> Result<VarDecl, Stmt> {
    match statement {
        Stmt::Decl(Decl::Var(variable_declaration)) => Ok(*variable_declaration),
        statement => Err(statement),
    }
}

fn flush_var_declaration<T>(
    output: &mut Vec<T>,
    active_declaration: &mut Option<VarDecl>,
    wrap_statement: impl FnOnce(Stmt) -> T,
) {
    let Some(variable_declaration) = active_declaration.take() else {
        return;
    };

    output.push(wrap_statement(create_var_statement(variable_declaration)));
}

fn create_var_statement(variable_declaration: VarDecl) -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(variable_declaration)))
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, enabled: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_variable_declarations_merge(&mut parsed_program.program, enabled);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn merges_adjacent_var_declarations_when_enabled() {
        let code = transform("var foo=1;var bar=2;var baz=3;", true);

        assert!(code.contains("var foo=1,bar=2,baz=3;"), "{code}");
    }

    #[test]
    fn keeps_adjacent_var_declarations_when_disabled() {
        let code = transform("var foo=1;var bar=2;", false);

        assert!(code.contains("var foo=1;var bar=2;"), "{code}");
    }

    #[test]
    fn keeps_non_variable_statement_as_merge_boundary() {
        let code = transform("var foo=1;console.log(foo);var bar=2;var baz=3;", true);

        assert!(
            code.contains("var foo=1;console.log(foo);var bar=2,baz=3;"),
            "{code}"
        );
    }

    #[test]
    fn merges_only_same_kind_declaration_groups() {
        let code = transform(
            "var foo=1;var bar=2;let baz=3;let bark=4;const hawk=5;const pork=6;",
            true,
        );

        assert!(
            code.contains("var foo=1,bar=2;let baz=3,bark=4;const hawk=5,pork=6;"),
            "{code}"
        );
    }
}
