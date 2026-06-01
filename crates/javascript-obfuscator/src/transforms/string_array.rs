use std::collections::BTreeMap;

use swc_common::DUMMY_SP;
use swc_ecma_ast::{
    ArrayLit, BindingIdent, CallExpr, Callee, ComputedPropName, Decl, Expr, ExprOrSpread, Ident,
    Lit, MemberExpr, MemberProp, ModuleDecl, ModuleItem, Number, Pat, Program, Str, VarDecl,
    VarDeclKind, VarDeclarator,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::options::StringArrayIndexesType;

pub fn transform_string_array(
    program: &mut Program,
    enabled: bool,
    threshold: f64,
    string_array_indexes_type: &[StringArrayIndexesType],
    reserved_strings: &[String],
    ignore_imports: bool,
) {
    if !enabled || threshold <= 0.0 {
        return;
    }

    let mut transform = StringArrayTransform {
        storage_name: "_0x0",
        indexes_by_value: BTreeMap::new(),
        values: Vec::new(),
        index_type: first_index_type(string_array_indexes_type),
        reserved_strings,
        ignore_imports,
    };

    program.visit_mut_with(&mut transform);

    if transform.values.is_empty() {
        return;
    }

    insert_storage_declaration(program, transform.storage_name, transform.values);
}

struct StringArrayTransform<'a> {
    storage_name: &'static str,
    indexes_by_value: BTreeMap<String, usize>,
    values: Vec<String>,
    index_type: StringArrayIndexesType,
    reserved_strings: &'a [String],
    ignore_imports: bool,
}

impl VisitMut for StringArrayTransform<'_> {
    fn visit_mut_expr_stmt(&mut self, expression_statement: &mut swc_ecma_ast::ExprStmt) {
        if matches!(expression_statement.expr.as_ref(), Expr::Lit(Lit::Str(_))) {
            return;
        }

        expression_statement.visit_mut_children_with(self);
    }

    fn visit_mut_call_expr(&mut self, call_expression: &mut CallExpr) {
        if self.ignore_imports && is_ignored_import_call(call_expression) {
            return;
        }

        call_expression.visit_mut_children_with(self);
    }

    fn visit_mut_expr(&mut self, expression: &mut Expr) {
        expression.visit_mut_children_with(self);

        let Expr::Lit(Lit::Str(string_literal)) = expression else {
            return;
        };

        let value = string_literal.value.to_string_lossy().into_owned();
        if is_reserved_string(&value, self.reserved_strings) {
            return;
        }

        let index = self.get_or_insert_value(value);
        *expression =
            create_string_array_member_expression(self.storage_name, index, self.index_type);
    }
}

impl StringArrayTransform<'_> {
    fn get_or_insert_value(&mut self, value: String) -> usize {
        if let Some(index) = self.indexes_by_value.get(&value) {
            return *index;
        }

        let index = self.values.len();
        self.values.push(value.clone());
        self.indexes_by_value.insert(value, index);
        index
    }
}

fn insert_storage_declaration(program: &mut Program, storage_name: &str, values: Vec<String>) {
    let storage_statement = create_storage_statement(storage_name, values);

    match program {
        Program::Script(script) => script.body.insert(0, storage_statement),
        Program::Module(module) => {
            let insert_index = module
                .body
                .iter()
                .position(|item| !matches!(item, ModuleItem::ModuleDecl(ModuleDecl::Import(_))))
                .unwrap_or(module.body.len());

            module
                .body
                .insert(insert_index, ModuleItem::Stmt(storage_statement));
        }
    }
}

fn create_storage_statement(storage_name: &str, values: Vec<String>) -> swc_ecma_ast::Stmt {
    swc_ecma_ast::Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        ctxt: Default::default(),
        kind: VarDeclKind::Const,
        declare: false,
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(BindingIdent {
                id: create_identifier(storage_name),
                type_ann: None,
            }),
            init: Some(Box::new(Expr::Array(ArrayLit {
                span: DUMMY_SP,
                elems: values
                    .into_iter()
                    .map(|value| Some(create_expr_or_spread(create_string_literal(&value))))
                    .collect(),
            }))),
            definite: false,
        }],
    })))
}

fn first_index_type(index_types: &[StringArrayIndexesType]) -> StringArrayIndexesType {
    index_types
        .first()
        .copied()
        .unwrap_or(StringArrayIndexesType::HexadecimalNumber)
}

fn create_string_array_member_expression(
    storage_name: &str,
    index: usize,
    index_type: StringArrayIndexesType,
) -> Expr {
    Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(create_identifier(storage_name))),
        prop: MemberProp::Computed(ComputedPropName {
            span: DUMMY_SP,
            expr: Box::new(create_index_literal(index, index_type)),
        }),
    })
}

fn create_identifier(name: &str) -> Ident {
    Ident::from(name.to_string())
}

fn create_expr_or_spread(expression: Expr) -> ExprOrSpread {
    ExprOrSpread {
        spread: None,
        expr: Box::new(expression),
    }
}

fn create_string_literal(value: &str) -> Expr {
    Expr::Lit(Lit::Str(Str {
        span: DUMMY_SP,
        value: value.to_string().into(),
        raw: Some(single_quote_raw(value).into()),
    }))
}

fn create_number_literal(value: usize) -> Expr {
    Expr::Lit(Lit::Num(Number {
        span: DUMMY_SP,
        value: value as f64,
        raw: Some(format!("0x{value:x}").into()),
    }))
}

fn create_index_literal(value: usize, index_type: StringArrayIndexesType) -> Expr {
    match index_type {
        StringArrayIndexesType::HexadecimalNumber => create_number_literal(value),
        StringArrayIndexesType::HexadecimalNumericString => {
            create_string_literal(&format!("0x{value:x}"))
        }
    }
}

fn single_quote_raw(value: &str) -> String {
    let mut escaped = String::new();

    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            character => escaped.push(character),
        }
    }

    format!("'{escaped}'")
}

fn is_ignored_import_call(call_expression: &CallExpr) -> bool {
    is_dynamic_import_call(call_expression) || is_require_call(call_expression)
}

fn is_dynamic_import_call(call_expression: &CallExpr) -> bool {
    matches!(call_expression.callee, Callee::Import(_))
}

fn is_require_call(call_expression: &CallExpr) -> bool {
    let Callee::Expr(callee_expression) = &call_expression.callee else {
        return false;
    };
    let Expr::Ident(identifier) = callee_expression.as_ref() else {
        return false;
    };

    identifier.sym.as_ref() == "require"
}

fn is_reserved_string(value: &str, reserved_strings: &[String]) -> bool {
    reserved_strings
        .iter()
        .any(|reserved_string| value.contains(reserved_string))
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(
        source_code: &str,
        enabled: bool,
        reserved_strings: &[String],
        ignore_imports: bool,
    ) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            enabled,
            1.0,
            &[],
            reserved_strings,
            ignore_imports,
        );
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn extracts_repeated_string_literals() {
        let code = transform(
            "const value = 'test'; console.log('test');",
            true,
            &[],
            false,
        );

        assert!(code.contains("const _0x0=['test'];"), "{code}");
        assert!(
            code.contains("const value=_0x0[0x0];console.log(_0x0[0x0]);"),
            "{code}"
        );
    }

    #[test]
    fn keeps_string_literals_when_disabled() {
        let code = transform("const value = 'test';", false, &[], false);

        assert!(code.contains("const value='test';"), "{code}");
        assert!(!code.contains("const _0x0=["), "{code}");
    }

    #[test]
    fn keeps_reserved_string_literals_inline() {
        let code = transform(
            "const keep = 'keep'; const take = 'take';",
            true,
            &["keep".to_string()],
            false,
        );

        assert!(code.contains("const _0x0=['take'];"), "{code}");
        assert!(code.contains("const keep='keep';"), "{code}");
        assert!(code.contains("const take=_0x0[0x0];"), "{code}");
    }

    #[test]
    fn keeps_import_call_literals_inline_when_ignore_imports_is_enabled() {
        let code = transform(
            "const foo = require('./foo'); const bar = './bar';",
            true,
            &[],
            true,
        );

        assert!(code.contains("const _0x0=['./bar'];"), "{code}");
        assert!(code.contains("require('./foo')"), "{code}");
        assert!(code.contains("const bar=_0x0[0x0];"), "{code}");
    }

    #[test]
    fn keeps_string_literals_when_threshold_is_zero() {
        let mut parsed_program =
            parse_program("const value = 'test';").expect("source should parse");
        transform_string_array(&mut parsed_program.program, true, 0.0, &[], &[], false);
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const value='test';"), "{code}");
        assert!(!code.contains("const _0x0=["), "{code}");
    }

    #[test]
    fn uses_hexadecimal_numeric_string_index_type() {
        let mut parsed_program =
            parse_program("const value = 'test';").expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            true,
            1.0,
            &[StringArrayIndexesType::HexadecimalNumericString],
            &[],
            false,
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const value=_0x0['0x0'];"), "{code}");
    }
}
