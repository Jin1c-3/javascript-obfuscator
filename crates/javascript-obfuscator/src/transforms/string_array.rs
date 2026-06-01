use std::collections::BTreeMap;

use swc_common::DUMMY_SP;
use swc_ecma_ast::{
    ArrayLit, BinExpr, BinaryOp, BindingIdent, BlockStmt, CallExpr, Callee, ComputedPropName, Decl,
    Expr, ExprOrSpread, FnDecl, Function, Ident, Lit, MemberExpr, MemberProp, ModuleDecl,
    ModuleItem, Number, Param, Pat, Program, ReturnStmt, Stmt, Str, VarDecl, VarDeclKind,
    VarDeclarator,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::options::StringArrayIndexesType;

const INDEX_SHIFT_AMOUNT: usize = 100;
const ROTATION_AMOUNT: usize = 1;
const SHIFTED_WRAPPER_NAME: &str = "_0x1";

pub struct StringArrayTransformOptions<'a> {
    pub enabled: bool,
    pub threshold: f64,
    pub indexes_type: &'a [StringArrayIndexesType],
    pub index_shift: bool,
    pub shuffle: bool,
    pub rotate: bool,
    pub reserved_strings: &'a [String],
    pub ignore_imports: bool,
}

pub fn transform_string_array(program: &mut Program, options: StringArrayTransformOptions<'_>) {
    if !options.enabled || options.threshold <= 0.0 {
        return;
    }

    let mut transform = StringArrayTransform {
        storage_name: "_0x0",
        indexes_by_value: BTreeMap::new(),
        values: Vec::new(),
        index_type: first_index_type(options.indexes_type),
        index_shift_enabled: options.index_shift,
        reserved_strings: options.reserved_strings,
        ignore_imports: options.ignore_imports,
    };

    program.visit_mut_with(&mut transform);

    if transform.values.is_empty() {
        return;
    }

    let storage_name = transform.storage_name;
    let index_type = transform.index_type;
    let mut values = transform.values;

    if options.shuffle {
        let index_remap = reverse_string_array_values(&mut values);
        remap_string_array_indexes(
            program,
            storage_name,
            index_type,
            options.index_shift,
            &index_remap,
        );
    }

    if options.rotate {
        let index_remap = rotate_string_array_values(&mut values, ROTATION_AMOUNT);
        remap_string_array_indexes(
            program,
            storage_name,
            index_type,
            options.index_shift,
            &index_remap,
        );
    }

    insert_string_array_declarations(program, storage_name, values, options.index_shift);
}

struct StringArrayTransform<'a> {
    storage_name: &'static str,
    indexes_by_value: BTreeMap<String, usize>,
    values: Vec<String>,
    index_type: StringArrayIndexesType,
    index_shift_enabled: bool,
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
        *expression = create_string_array_reference_expression(
            self.storage_name,
            index,
            self.index_type,
            self.index_shift_enabled,
        );
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

struct StringArrayIndexRemapTransform<'a> {
    storage_name: &'static str,
    index_type: StringArrayIndexesType,
    index_shift_enabled: bool,
    index_remap: &'a [usize],
}

impl VisitMut for StringArrayIndexRemapTransform<'_> {
    fn visit_mut_expr(&mut self, expression: &mut Expr) {
        expression.visit_mut_children_with(self);

        match expression {
            Expr::Member(member_expression) => self.remap_member_expression(member_expression),
            Expr::Call(call_expression) => self.remap_call_expression(call_expression),
            _ => {}
        }
    }
}

impl StringArrayIndexRemapTransform<'_> {
    fn remap_member_expression(&self, member_expression: &mut MemberExpr) {
        if !is_identifier_expression(member_expression.obj.as_ref(), self.storage_name) {
            return;
        }

        let MemberProp::Computed(property) = &mut member_expression.prop else {
            return;
        };

        let Some(old_index) = index_from_literal(property.expr.as_ref()) else {
            return;
        };
        let Some(new_index) = self.remapped_index(old_index) else {
            return;
        };

        *property.expr = create_index_literal(new_index, self.index_type);
    }

    fn remap_call_expression(&self, call_expression: &mut CallExpr) {
        if !self.index_shift_enabled || !is_identifier_callee(call_expression, SHIFTED_WRAPPER_NAME)
        {
            return;
        }

        let Some(first_argument) = call_expression.args.first_mut() else {
            return;
        };
        let Some(shifted_index) = index_from_literal(first_argument.expr.as_ref()) else {
            return;
        };
        let Some(old_index) = shifted_index.checked_sub(INDEX_SHIFT_AMOUNT) else {
            return;
        };
        let Some(new_index) = self.remapped_index(old_index) else {
            return;
        };

        *first_argument.expr =
            create_index_literal(new_index + INDEX_SHIFT_AMOUNT, self.index_type);
    }

    fn remapped_index(&self, old_index: usize) -> Option<usize> {
        self.index_remap.get(old_index).copied()
    }
}

fn reverse_string_array_values(values: &mut [String]) -> Vec<usize> {
    let length = values.len();
    let index_remap = (0..length).map(|index| length - index - 1).collect();
    values.reverse();

    index_remap
}

fn rotate_string_array_values(values: &mut [String], rotation_amount: usize) -> Vec<usize> {
    let length = values.len();
    if length == 0 {
        return Vec::new();
    }

    let normalized_rotation_amount = rotation_amount % length;
    if normalized_rotation_amount == 0 {
        return (0..length).collect();
    }

    values.rotate_right(normalized_rotation_amount);

    (0..length)
        .map(|index| (index + normalized_rotation_amount) % length)
        .collect()
}

fn remap_string_array_indexes(
    program: &mut Program,
    storage_name: &'static str,
    index_type: StringArrayIndexesType,
    index_shift_enabled: bool,
    index_remap: &[usize],
) {
    program.visit_mut_with(&mut StringArrayIndexRemapTransform {
        storage_name,
        index_type,
        index_shift_enabled,
        index_remap,
    });
}

fn insert_string_array_declarations(
    program: &mut Program,
    storage_name: &str,
    values: Vec<String>,
    index_shift_enabled: bool,
) {
    let mut statements = vec![create_storage_statement(storage_name, values)];

    if index_shift_enabled {
        statements.push(create_index_shift_wrapper_statement(
            storage_name,
            SHIFTED_WRAPPER_NAME,
            INDEX_SHIFT_AMOUNT,
        ));
    }

    match program {
        Program::Script(script) => {
            script.body.splice(0..0, statements);
        }
        Program::Module(module) => {
            let insert_index = module
                .body
                .iter()
                .position(|item| !matches!(item, ModuleItem::ModuleDecl(ModuleDecl::Import(_))))
                .unwrap_or(module.body.len());

            module.body.splice(
                insert_index..insert_index,
                statements.into_iter().map(ModuleItem::Stmt),
            );
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

fn create_string_array_reference_expression(
    storage_name: &str,
    index: usize,
    index_type: StringArrayIndexesType,
    index_shift_enabled: bool,
) -> Expr {
    if index_shift_enabled {
        return create_string_array_call_expression(
            SHIFTED_WRAPPER_NAME,
            index + INDEX_SHIFT_AMOUNT,
            index_type,
        );
    }

    create_string_array_member_expression(storage_name, index, index_type)
}

fn create_string_array_call_expression(
    wrapper_name: &str,
    index: usize,
    index_type: StringArrayIndexesType,
) -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        ctxt: Default::default(),
        callee: Callee::Expr(Box::new(Expr::Ident(create_identifier(wrapper_name)))),
        args: vec![create_expr_or_spread(create_index_literal(
            index, index_type,
        ))],
        type_args: None,
    })
}

fn create_string_array_member_expression(
    storage_name: &str,
    index: usize,
    index_type: StringArrayIndexesType,
) -> Expr {
    create_string_array_member_expression_with_property(
        storage_name,
        create_index_literal(index, index_type),
    )
}

fn create_string_array_member_expression_with_property(storage_name: &str, property: Expr) -> Expr {
    Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(create_identifier(storage_name))),
        prop: MemberProp::Computed(ComputedPropName {
            span: DUMMY_SP,
            expr: Box::new(property),
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

fn create_index_shift_wrapper_statement(
    storage_name: &str,
    wrapper_name: &str,
    shift_amount: usize,
) -> Stmt {
    Stmt::Decl(Decl::Fn(FnDecl {
        ident: create_identifier(wrapper_name),
        declare: false,
        function: Box::new(Function {
            params: vec![Param {
                span: DUMMY_SP,
                decorators: Vec::new(),
                pat: Pat::Ident(BindingIdent {
                    id: create_identifier("index"),
                    type_ann: None,
                }),
            }],
            decorators: Vec::new(),
            span: DUMMY_SP,
            ctxt: Default::default(),
            body: Some(BlockStmt {
                span: DUMMY_SP,
                ctxt: Default::default(),
                stmts: vec![create_return_statement(
                    create_string_array_member_expression_with_property(
                        storage_name,
                        create_sub_expression(
                            Expr::Ident(create_identifier("index")),
                            create_number_literal(shift_amount),
                        ),
                    ),
                )],
            }),
            is_generator: false,
            is_async: false,
            type_params: None,
            return_type: None,
        }),
    }))
}

fn create_sub_expression(left: Expr, right: Expr) -> Expr {
    Expr::Bin(BinExpr {
        span: DUMMY_SP,
        op: BinaryOp::Sub,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn create_return_statement(argument: Expr) -> Stmt {
    Stmt::Return(ReturnStmt {
        span: DUMMY_SP,
        arg: Some(Box::new(argument)),
    })
}

fn create_index_literal(value: usize, index_type: StringArrayIndexesType) -> Expr {
    match index_type {
        StringArrayIndexesType::HexadecimalNumber => create_number_literal(value),
        StringArrayIndexesType::HexadecimalNumericString => {
            create_string_literal(&format!("0x{value:x}"))
        }
    }
}

fn index_from_literal(expression: &Expr) -> Option<usize> {
    match expression {
        Expr::Lit(Lit::Num(number)) if number.value.is_finite() && number.value >= 0.0 => {
            Some(number.value as usize)
        }
        Expr::Lit(Lit::Str(string)) => parse_index_string(&string.value.to_string_lossy()),
        _ => None,
    }
}

fn parse_index_string(value: &str) -> Option<usize> {
    if let Some(hexadecimal) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return usize::from_str_radix(hexadecimal, 16).ok();
    }

    value.parse().ok()
}

fn is_identifier_expression(expression: &Expr, name: &str) -> bool {
    let Expr::Ident(identifier) = expression else {
        return false;
    };

    identifier.sym.as_ref() == name
}

fn is_identifier_callee(call_expression: &CallExpr, name: &str) -> bool {
    let Callee::Expr(callee_expression) = &call_expression.callee else {
        return false;
    };

    is_identifier_expression(callee_expression.as_ref(), name)
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
            StringArrayTransformOptions {
                enabled,
                threshold: 1.0,
                indexes_type: &[],
                index_shift: false,
                shuffle: false,
                rotate: false,
                reserved_strings,
                ignore_imports,
            },
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
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 0.0,
                indexes_type: &[],
                index_shift: false,
                shuffle: false,
                rotate: false,
                reserved_strings: &[],
                ignore_imports: false,
            },
        );
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
            StringArrayTransformOptions {
                enabled: true,
                threshold: 1.0,
                indexes_type: &[StringArrayIndexesType::HexadecimalNumericString],
                index_shift: false,
                shuffle: false,
                rotate: false,
                reserved_strings: &[],
                ignore_imports: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const value=_0x0['0x0'];"), "{code}");
    }

    #[test]
    fn uses_shifted_wrapper_when_index_shift_enabled() {
        let mut parsed_program = parse_program("const first = 'foo'; const second = 'bar';")
            .expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 1.0,
                indexes_type: &[],
                index_shift: true,
                shuffle: false,
                rotate: false,
                reserved_strings: &[],
                ignore_imports: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const _0x0=['foo','bar'];"), "{code}");
        assert!(
            code.contains("function _0x1(index){return _0x0[index-0x64];}"),
            "{code}"
        );
        assert!(
            code.contains("const first=_0x1(0x64);const second=_0x1(0x65);"),
            "{code}"
        );
    }

    #[test]
    fn remaps_shifted_indexes_when_string_array_shuffle_is_enabled() {
        let mut parsed_program = parse_program("const first = 'foo'; const second = 'bar';")
            .expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 1.0,
                indexes_type: &[],
                index_shift: true,
                shuffle: true,
                rotate: false,
                reserved_strings: &[],
                ignore_imports: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const _0x0=['bar','foo'];"), "{code}");
        assert!(
            code.contains("function _0x1(index){return _0x0[index-0x64];}"),
            "{code}"
        );
        assert!(
            code.contains("const first=_0x1(0x65);const second=_0x1(0x64);"),
            "{code}"
        );
    }

    #[test]
    fn remaps_shifted_indexes_when_string_array_rotate_is_enabled() {
        let mut parsed_program =
            parse_program("const first = 'foo'; const second = 'bar'; const third = 'baz';")
                .expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 1.0,
                indexes_type: &[],
                index_shift: true,
                shuffle: false,
                rotate: true,
                reserved_strings: &[],
                ignore_imports: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const _0x0=['baz','foo','bar'];"), "{code}");
        assert!(
            code.contains("function _0x1(index){return _0x0[index-0x64];}"),
            "{code}"
        );
        assert!(
            code.contains("const first=_0x1(0x65);const second=_0x1(0x66);const third=_0x1(0x64);"),
            "{code}"
        );
    }
}
