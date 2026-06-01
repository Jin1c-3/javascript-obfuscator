use swc_common::DUMMY_SP;
use swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, BlockStmt, ComputedPropName, Decl, Expr,
    ExprStmt, Ident, Lit, MemberExpr, MemberProp, ModuleItem, ObjectLit, Pat, Program, Prop,
    PropName, PropOrSpread, ReturnStmt, SimpleAssignTarget, Stmt, Str, SwitchCase, VarDecl,
    VarDeclKind, VarDeclarator,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::generators::{IdentifierNamesGenerator, IdentifierNamesGeneratorKind};

pub fn transform_object_expression_keys(
    program: &mut Program,
    enabled: bool,
    generator_kind: IdentifierNamesGeneratorKind,
    identifiers_prefix: &str,
    identifiers_dictionary: &[String],
) {
    if !enabled {
        return;
    }

    program.visit_mut_with(&mut ObjectExpressionKeysTransform {
        generator: IdentifierNamesGenerator::new(
            generator_kind,
            identifiers_prefix,
            identifiers_dictionary.to_vec(),
        ),
    });
}

struct ObjectExpressionKeysTransform {
    generator: IdentifierNamesGenerator,
}

impl VisitMut for ObjectExpressionKeysTransform {
    fn visit_mut_program(&mut self, program: &mut Program) {
        program.visit_mut_children_with(self);

        match program {
            Program::Module(module) => expand_module_items(&mut module.body, &mut self.generator),
            Program::Script(script) => expand_statements(&mut script.body, &mut self.generator),
        }
    }

    fn visit_mut_block_stmt(&mut self, block_statement: &mut BlockStmt) {
        block_statement.visit_mut_children_with(self);
        expand_statements(&mut block_statement.stmts, &mut self.generator);
    }

    fn visit_mut_switch_case(&mut self, switch_case: &mut SwitchCase) {
        switch_case.visit_mut_children_with(self);
        expand_statements(&mut switch_case.cons, &mut self.generator);
    }
}

fn expand_module_items(
    module_items: &mut Vec<ModuleItem>,
    generator: &mut IdentifierNamesGenerator,
) {
    let mut output = Vec::with_capacity(module_items.len());

    for module_item in std::mem::take(module_items) {
        match module_item {
            ModuleItem::Stmt(statement) => {
                output.extend(
                    expand_statement(statement, generator)
                        .into_iter()
                        .map(ModuleItem::Stmt),
                );
            }
            module_item => output.push(module_item),
        }
    }

    *module_items = output;
}

fn expand_statements(statements: &mut Vec<Stmt>, generator: &mut IdentifierNamesGenerator) {
    let mut output = Vec::with_capacity(statements.len());

    for statement in std::mem::take(statements) {
        output.extend(expand_statement(statement, generator));
    }

    *statements = output;
}

fn expand_statement(statement: Stmt, generator: &mut IdentifierNamesGenerator) -> Vec<Stmt> {
    match statement {
        Stmt::Decl(Decl::Var(variable_declaration)) => {
            expand_variable_declaration(*variable_declaration, generator)
        }
        Stmt::Return(return_statement) => expand_return_statement(return_statement, generator),
        statement => vec![statement],
    }
}

fn expand_variable_declaration(
    mut variable_declaration: VarDecl,
    generator: &mut IdentifierNamesGenerator,
) -> Vec<Stmt> {
    let Some(assignments) = extract_property_assignments(&variable_declaration) else {
        return vec![create_var_statement(variable_declaration)];
    };

    let temporary_name = generator.generate_next();
    let mut statements = Vec::with_capacity(assignments.len() + 2);
    statements.push(create_empty_object_statement(&temporary_name));

    for (key, value) in assignments {
        statements.push(create_property_assignment_statement(
            &temporary_name,
            &key,
            value,
        ));
    }

    variable_declaration.decls[0].init =
        Some(Box::new(Expr::Ident(create_identifier(&temporary_name))));
    statements.push(create_var_statement(variable_declaration));

    statements
}

fn expand_return_statement(
    mut return_statement: ReturnStmt,
    generator: &mut IdentifierNamesGenerator,
) -> Vec<Stmt> {
    let Some(assignments) = extract_return_property_assignments(&return_statement) else {
        return vec![Stmt::Return(return_statement)];
    };

    let temporary_name = generator.generate_next();
    let mut statements = Vec::with_capacity(assignments.len() + 2);
    statements.push(create_empty_object_statement(&temporary_name));

    for (key, value) in assignments {
        statements.push(create_property_assignment_statement(
            &temporary_name,
            &key,
            value,
        ));
    }

    return_statement.arg = Some(Box::new(Expr::Ident(create_identifier(&temporary_name))));
    statements.push(Stmt::Return(return_statement));

    statements
}

fn extract_property_assignments(variable_declaration: &VarDecl) -> Option<Vec<(String, Expr)>> {
    if variable_declaration.decls.len() != 1 {
        return None;
    }

    let declarator = variable_declaration.decls.first()?;
    if !matches!(declarator.name, Pat::Ident(_)) {
        return None;
    }

    let Expr::Object(object_lit) = declarator.init.as_deref()? else {
        return None;
    };

    collect_key_value_properties(object_lit)
}

fn extract_return_property_assignments(
    return_statement: &ReturnStmt,
) -> Option<Vec<(String, Expr)>> {
    let Expr::Object(object_lit) = return_statement.arg.as_deref()? else {
        return None;
    };

    collect_key_value_properties(object_lit)
}

fn collect_key_value_properties(object_lit: &ObjectLit) -> Option<Vec<(String, Expr)>> {
    if object_lit.props.is_empty() {
        return None;
    }

    let mut assignments = Vec::with_capacity(object_lit.props.len());

    for prop_or_spread in &object_lit.props {
        let PropOrSpread::Prop(property) = prop_or_spread else {
            return None;
        };
        let Prop::KeyValue(key_value_property) = property.as_ref() else {
            return None;
        };
        let key = property_key_name(&key_value_property.key)?;

        assignments.push((key, key_value_property.value.as_ref().clone()));
    }

    Some(assignments)
}

fn property_key_name(property_name: &PropName) -> Option<String> {
    match property_name {
        PropName::Ident(identifier) => Some(identifier.sym.to_string()),
        PropName::Str(string_literal) => Some(string_literal.value.to_string_lossy().into_owned()),
        PropName::Num(number_literal) => Some(
            number_literal
                .raw
                .as_deref()
                .map(str::to_string)
                .unwrap_or_else(|| number_literal.value.to_string()),
        ),
        PropName::Computed(computed_property_name) => {
            let Expr::Lit(Lit::Str(string_literal)) = computed_property_name.expr.as_ref() else {
                return None;
            };

            Some(string_literal.value.to_string_lossy().into_owned())
        }
        _ => None,
    }
}

fn create_empty_object_statement(name: &str) -> Stmt {
    create_var_statement(VarDecl {
        span: DUMMY_SP,
        ctxt: Default::default(),
        kind: VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(BindingIdent {
                id: create_identifier(name),
                type_ann: None,
            }),
            init: Some(Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: Vec::new(),
            }))),
            definite: false,
        }],
    })
}

fn create_property_assignment_statement(object_name: &str, key: &str, value: Expr) -> Stmt {
    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Assign(AssignExpr {
            span: DUMMY_SP,
            op: AssignOp::Assign,
            left: AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(create_identifier(object_name))),
                prop: MemberProp::Computed(ComputedPropName {
                    span: DUMMY_SP,
                    expr: Box::new(create_string_literal(key)),
                }),
            })),
            right: Box::new(value),
        })),
    })
}

fn create_var_statement(variable_declaration: VarDecl) -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(variable_declaration)))
}

fn create_identifier(name: &str) -> Ident {
    Ident::from(name.to_string())
}

fn create_string_literal(value: &str) -> Expr {
    Expr::Lit(Lit::Str(Str {
        span: DUMMY_SP,
        value: value.to_string().into(),
        raw: Some(single_quote_raw(value).into()),
    }))
}

fn single_quote_raw(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");

    format!("'{escaped}'")
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::generators::IdentifierNamesGeneratorKind;
    use crate::parser::parse_program;
    use crate::transforms::object_expressions::transform_object_expressions;

    use super::*;

    fn transform(source_code: &str, enabled: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_object_expressions(&mut parsed_program.program);
        transform_object_expression_keys(
            &mut parsed_program.program,
            enabled,
            IdentifierNamesGeneratorKind::Hexadecimal,
            "",
            &[],
        );
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transform_object_keys_simple_variable_declarator_when_enabled() {
        let code = transform("var object = {foo: 'bar', baz: 'bark'};", true);

        assert!(code.contains("var _0x0={};"), "{code}");
        assert!(code.contains("_0x0['foo']='bar';"), "{code}");
        assert!(code.contains("_0x0['baz']='bark';"), "{code}");
        assert!(code.contains("var object=_0x0;"), "{code}");
    }

    #[test]
    fn keeps_object_literal_when_disabled() {
        let code = transform("var object = {foo: 'bar'};", false);

        assert!(code.contains("var object={'foo':'bar'};"), "{code}");
    }

    #[test]
    fn skips_multiple_declarators() {
        let code = transform("var object = {foo: 'bar'}, other = {baz: 'bark'};", true);

        assert!(
            code.contains("var object={'foo':'bar'},other={'baz':'bark'};"),
            "{code}"
        );
    }

    #[test]
    fn transform_object_keys_return_object_when_enabled() {
        let code = transform(
            "function getObject() { return {foo: 'bar', baz: 'bark'}; }",
            true,
        );

        assert!(code.contains("var _0x0={};"), "{code}");
        assert!(code.contains("_0x0['foo']='bar';"), "{code}");
        assert!(code.contains("_0x0['baz']='bark';"), "{code}");
        assert!(code.contains("return _0x0;"), "{code}");
    }

    #[test]
    fn keeps_return_object_with_spread_property() {
        let code = transform(
            "function getObject() { return {...source, foo: 'bar'}; }",
            true,
        );

        assert!(
            code.contains("function getObject(){return{...source,'foo':'bar'};}"),
            "{code}"
        );
    }
}
