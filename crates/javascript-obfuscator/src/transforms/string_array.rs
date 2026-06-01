use std::collections::BTreeMap;

use regex::Regex;
use swc_common::DUMMY_SP;
use swc_ecma_ast::{
    ArrayLit, ArrowExpr, BinExpr, BinaryOp, BindingIdent, BlockStmt, BlockStmtOrExpr, CallExpr,
    Callee, ComputedPropName, Decl, Expr, ExprOrSpread, FnDecl, Function, Ident, IdentName,
    KeyValueProp, Lit, MemberExpr, MemberProp, ModuleDecl, ModuleItem, Number, ObjectLit, Param,
    Pat, Program, Prop, PropName, PropOrSpread, ReturnStmt, Stmt, Str, VarDecl, VarDeclKind,
    VarDeclarator,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::options::{StringArrayEncoding, StringArrayIndexesType, StringArrayWrappersType};
use crate::parser::parse_program;

const INDEX_SHIFT_AMOUNT: usize = 100;
const MINIMUM_LENGTH_FOR_STRING_ARRAY: usize = 3;
const ROTATION_AMOUNT: usize = 1;
const SHIFTED_WRAPPER_NAME: &str = "_0x1";
const BASE64_ALPHABET_SWAPPED: &[u8; 64] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";
const DEFAULT_RC4_KEY: &str = "rc4K";

pub struct StringArrayTransformOptions<'a> {
    pub enabled: bool,
    pub threshold: f64,
    pub calls_transform: bool,
    pub calls_transform_threshold: f64,
    pub indexes_type: &'a [StringArrayIndexesType],
    pub encoding: StringArrayEncoding,
    pub index_shift: bool,
    pub shuffle: bool,
    pub rotate: bool,
    pub reserved_strings: &'a [String],
    pub force_transform_strings: &'a [String],
    pub ignore_imports: bool,
    pub wrappers_count: usize,
    pub wrappers_parameters_max_count: usize,
    pub wrappers_type: StringArrayWrappersType,
    pub wrappers_chained_calls: bool,
    pub self_defending: bool,
}

pub fn transform_string_array(program: &mut Program, options: StringArrayTransformOptions<'_>) {
    if !options.enabled {
        return;
    }

    let wrappers = root_call_wrappers(
        options.wrappers_count,
        options.wrappers_type,
        options.wrappers_parameters_max_count,
    );
    let force_transform_patterns =
        compile_force_transform_patterns(options.force_transform_strings);
    let mut transform = StringArrayTransform {
        storage_name: "_0x0",
        indexes_by_value: BTreeMap::new(),
        values: Vec::new(),
        threshold: options.threshold,
        index_type: first_index_type(options.indexes_type),
        encoding: options.encoding,
        index_shift_enabled: options.index_shift,
        reserved_string_patterns: compile_patterns(options.reserved_strings),
        force_transform_patterns,
        ignore_imports: options.ignore_imports,
        wrappers,
        next_wrapper_index: 0,
        used_wrapper_count: 0,
    };

    program.visit_mut_with(&mut transform);

    if transform.values.is_empty() {
        return;
    }

    let storage_name = transform.storage_name;
    let index_type = transform.index_type;
    let encoding = options.encoding;
    let wrapper_enabled = should_emit_string_array_wrapper(options.index_shift, encoding);
    let wrappers = transform.used_wrappers();
    let active_call_wrappers = active_string_array_call_wrappers(&wrappers);
    let mut values = transform.values;

    if options.shuffle {
        let index_remap = reverse_string_array_values(&mut values);
        remap_string_array_indexes(
            program,
            storage_name,
            index_type,
            options.index_shift,
            wrapper_enabled,
            &active_call_wrappers,
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
            wrapper_enabled,
            &active_call_wrappers,
            &index_remap,
        );
    }

    if options.calls_transform && options.calls_transform_threshold > 0.0 && wrapper_enabled {
        transform_string_array_calls(program, &active_call_wrappers, wrappers.len());
    }

    if options.wrappers_chained_calls
        && options.wrappers_type == StringArrayWrappersType::Variable
        && !wrappers.is_empty()
    {
        transform_string_array_chained_variable_wrappers(program, &active_call_wrappers);
    }

    insert_string_array_declarations(
        program,
        storage_name,
        values,
        options.index_shift,
        encoding,
        &wrappers,
        options.self_defending,
    );
}

#[derive(Clone)]
struct StringArrayValue {
    encoded_value: String,
    decode_key: Option<&'static str>,
}

#[derive(Clone)]
struct StringArrayCallWrapper {
    name: String,
    kind: StringArrayCallWrapperKind,
    index_shift: usize,
    parameters_count: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum StringArrayCallWrapperKind {
    Root,
    Variable,
    Function,
}

struct StringArrayTransform {
    storage_name: &'static str,
    indexes_by_value: BTreeMap<String, usize>,
    values: Vec<StringArrayValue>,
    threshold: f64,
    index_type: StringArrayIndexesType,
    encoding: StringArrayEncoding,
    index_shift_enabled: bool,
    reserved_string_patterns: Vec<Regex>,
    force_transform_patterns: Vec<Regex>,
    ignore_imports: bool,
    wrappers: Vec<StringArrayCallWrapper>,
    next_wrapper_index: usize,
    used_wrapper_count: usize,
}

impl VisitMut for StringArrayTransform {
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
        let is_force_transform_string =
            is_force_transform_string(&value, &self.force_transform_patterns);

        if !is_force_transform_string {
            if self.threshold <= 0.0 {
                return;
            }

            if !has_minimum_length_for_string_array(&value) {
                return;
            }

            if is_matching_pattern(&value, &self.reserved_string_patterns) {
                return;
            }
        }

        let (index, decode_key) = self.get_or_insert_value(value);
        let wrapper = self.next_string_array_calls_wrapper();
        *expression = create_string_array_reference_expression(
            self.storage_name,
            &wrapper,
            index,
            self.index_type,
            self.encoding,
            self.index_shift_enabled,
            decode_key,
        );
    }
}

impl StringArrayTransform {
    fn get_or_insert_value(&mut self, value: String) -> (usize, Option<&'static str>) {
        if let Some(index) = self.indexes_by_value.get(&value) {
            return (*index, self.values[*index].decode_key);
        }

        let index = self.values.len();
        let string_array_value = encode_string_array_value(&value, self.encoding);
        let decode_key = string_array_value.decode_key;
        self.values.push(string_array_value);
        self.indexes_by_value.insert(value, index);
        (index, decode_key)
    }

    fn next_string_array_calls_wrapper(&mut self) -> StringArrayCallWrapper {
        if self.wrappers.is_empty() {
            return root_string_array_call_wrapper();
        }

        let wrapper_index = self.next_wrapper_index % self.wrappers.len();
        self.next_wrapper_index += 1;
        self.used_wrapper_count = self.used_wrapper_count.max(wrapper_index + 1);

        self.wrappers[wrapper_index].clone()
    }

    fn used_wrappers(&self) -> Vec<StringArrayCallWrapper> {
        self.wrappers
            .iter()
            .take(self.used_wrapper_count)
            .cloned()
            .collect()
    }
}

struct StringArrayIndexRemapTransform<'a> {
    storage_name: &'static str,
    index_type: StringArrayIndexesType,
    index_shift_enabled: bool,
    wrapper_enabled: bool,
    call_wrappers: &'a [StringArrayCallWrapper],
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
        if !self.wrapper_enabled {
            return;
        }

        let Some(call_wrapper) =
            string_array_call_wrapper_for_call(call_expression, self.call_wrappers)
        else {
            return;
        };

        let Some(first_argument) = call_expression.args.first_mut() else {
            return;
        };
        let Some(encoded_index) = index_from_literal(first_argument.expr.as_ref()) else {
            return;
        };
        let Some(root_wrapper_index) = encoded_index.checked_sub(call_wrapper.index_shift) else {
            return;
        };
        let old_index = if self.index_shift_enabled {
            let Some(unshifted_index) = root_wrapper_index.checked_sub(INDEX_SHIFT_AMOUNT) else {
                return;
            };
            unshifted_index
        } else {
            root_wrapper_index
        };
        let Some(new_index) = self.remapped_index(old_index) else {
            return;
        };
        let remapped_index = if self.index_shift_enabled {
            new_index + INDEX_SHIFT_AMOUNT
        } else {
            new_index
        };

        *first_argument.expr =
            create_index_literal(remapped_index + call_wrapper.index_shift, self.index_type);
    }

    fn remapped_index(&self, old_index: usize) -> Option<usize> {
        self.index_remap.get(old_index).copied()
    }
}

fn reverse_string_array_values(values: &mut [StringArrayValue]) -> Vec<usize> {
    let length = values.len();
    let index_remap = (0..length).map(|index| length - index - 1).collect();
    values.reverse();

    index_remap
}

fn rotate_string_array_values(
    values: &mut [StringArrayValue],
    rotation_amount: usize,
) -> Vec<usize> {
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
    wrapper_enabled: bool,
    call_wrappers: &[StringArrayCallWrapper],
    index_remap: &[usize],
) {
    program.visit_mut_with(&mut StringArrayIndexRemapTransform {
        storage_name,
        index_type,
        index_shift_enabled,
        wrapper_enabled,
        call_wrappers,
        index_remap,
    });
}

fn transform_string_array_calls(
    program: &mut Program,
    call_wrappers: &[StringArrayCallWrapper],
    used_wrapper_count: usize,
) {
    program.visit_mut_with(&mut StringArrayCallsTransform {
        call_wrappers,
        next_storage_index: 2 + used_wrapper_count,
    });
}

fn transform_string_array_chained_variable_wrappers(
    program: &mut Program,
    call_wrappers: &[StringArrayCallWrapper],
) {
    let Some(root_wrapper_name) = call_wrappers.first().map(|wrapper| wrapper.name.clone()) else {
        return;
    };
    let active_wrapper_names = call_wrappers
        .iter()
        .map(|wrapper| wrapper.name.clone())
        .collect();

    program.visit_mut_with(&mut StringArrayChainedVariableWrappersTransform {
        active_wrapper_names,
        wrapper_stack: vec![root_wrapper_name],
        next_scope_wrapper_index: 0,
    });
}

struct StringArrayChainedVariableWrappersTransform {
    active_wrapper_names: Vec<String>,
    wrapper_stack: Vec<String>,
    next_scope_wrapper_index: usize,
}

impl VisitMut for StringArrayChainedVariableWrappersTransform {
    fn visit_mut_function(&mut self, function: &mut Function) {
        if let Some(body) = &mut function.body {
            self.transform_block_body(body);
        }
    }

    fn visit_mut_arrow_expr(&mut self, arrow_expression: &mut ArrowExpr) {
        if let BlockStmtOrExpr::BlockStmt(body) = arrow_expression.body.as_mut() {
            self.transform_block_body(body);
        } else {
            arrow_expression.visit_mut_children_with(self);
        }
    }
}

impl StringArrayChainedVariableWrappersTransform {
    fn transform_block_body(&mut self, body: &mut BlockStmt) {
        let parent_wrapper_name = self
            .wrapper_stack
            .last()
            .expect("root wrapper should be present")
            .clone();
        let scope_wrapper_name = format!("_0xscopeWrapper{}", self.next_scope_wrapper_index);
        let mut calls_transform = FunctionStringArrayChainedVariableCallsTransform {
            active_wrapper_names: &self.active_wrapper_names,
            scope_wrapper_name: &scope_wrapper_name,
            has_rewritten_call: false,
        };

        body.visit_mut_with(&mut calls_transform);

        if !calls_transform.has_rewritten_call {
            body.visit_mut_children_with(self);
            return;
        }

        self.next_scope_wrapper_index += 1;
        insert_scope_wrapper_statement(body, &scope_wrapper_name, &parent_wrapper_name);
        self.wrapper_stack.push(scope_wrapper_name);
        body.visit_mut_children_with(self);
        self.wrapper_stack.pop();
    }
}

struct FunctionStringArrayChainedVariableCallsTransform<'a> {
    active_wrapper_names: &'a [String],
    scope_wrapper_name: &'a str,
    has_rewritten_call: bool,
}

impl VisitMut for FunctionStringArrayChainedVariableCallsTransform<'_> {
    fn visit_mut_function(&mut self, _function: &mut Function) {}

    fn visit_mut_arrow_expr(&mut self, _arrow_expression: &mut ArrowExpr) {}

    fn visit_mut_call_expr(&mut self, call_expression: &mut CallExpr) {
        call_expression.visit_mut_children_with(self);

        let Callee::Expr(callee_expression) = &mut call_expression.callee else {
            return;
        };
        let Expr::Ident(identifier) = callee_expression.as_mut() else {
            return;
        };

        if !self
            .active_wrapper_names
            .iter()
            .any(|wrapper_name| wrapper_name == identifier.sym.as_ref())
        {
            return;
        }

        *identifier = create_identifier(self.scope_wrapper_name);
        self.has_rewritten_call = true;
    }
}

fn insert_scope_wrapper_statement(
    body: &mut BlockStmt,
    scope_wrapper_name: &str,
    parent_wrapper_name: &str,
) {
    let insert_index = first_non_directive_statement_index(&body.stmts);

    body.stmts.insert(
        insert_index,
        create_variable_wrapper_statement(scope_wrapper_name, parent_wrapper_name),
    );
}

struct StringArrayCallsTransform<'a> {
    call_wrappers: &'a [StringArrayCallWrapper],
    next_storage_index: usize,
}

impl VisitMut for StringArrayCallsTransform<'_> {
    fn visit_mut_function(&mut self, function: &mut Function) {
        if let Some(body) = &mut function.body {
            self.transform_block_body(body);
        }

        function.visit_mut_children_with(self);
    }

    fn visit_mut_arrow_expr(&mut self, arrow_expression: &mut ArrowExpr) {
        if let BlockStmtOrExpr::BlockStmt(body) = arrow_expression.body.as_mut() {
            self.transform_block_body(body);
        }

        arrow_expression.visit_mut_children_with(self);
    }
}

impl StringArrayCallsTransform<'_> {
    fn transform_block_body(&mut self, body: &mut BlockStmt) {
        let storage_name = format!("_0x{}", self.next_storage_index);
        let mut body_transform = FunctionStringArrayCallsTransform {
            call_wrappers: self.call_wrappers,
            storage_name: storage_name.clone(),
            entries: Vec::new(),
        };

        body.visit_mut_with(&mut body_transform);

        if body_transform.entries.is_empty() {
            return;
        }

        self.next_storage_index += 1;
        body.stmts.insert(
            0,
            create_string_array_calls_storage_statement(&storage_name, body_transform.entries),
        );
    }
}

struct FunctionStringArrayCallsTransform<'a> {
    call_wrappers: &'a [StringArrayCallWrapper],
    storage_name: String,
    entries: Vec<(String, Expr)>,
}

impl VisitMut for FunctionStringArrayCallsTransform<'_> {
    fn visit_mut_function(&mut self, _function: &mut Function) {}

    fn visit_mut_arrow_expr(&mut self, _arrow_expression: &mut ArrowExpr) {}

    fn visit_mut_call_expr(&mut self, call_expression: &mut CallExpr) {
        call_expression.visit_mut_children_with(self);

        if string_array_call_wrapper_for_call(call_expression, self.call_wrappers).is_none() {
            return;
        }

        if !self.replace_generated_string_array_call_argument(call_expression, 0) {
            return;
        }

        self.replace_generated_rc4_decode_key_argument(call_expression);
    }
}

impl FunctionStringArrayCallsTransform<'_> {
    fn replace_generated_string_array_call_argument(
        &mut self,
        call_expression: &mut CallExpr,
        argument_index: usize,
    ) -> bool {
        let Some(argument) = call_expression.args.get_mut(argument_index) else {
            return false;
        };

        if !is_generated_string_array_call_index_literal(argument.expr.as_ref()) {
            return false;
        }

        self.replace_argument_with_storage_member(argument);

        true
    }

    fn replace_generated_rc4_decode_key_argument(&mut self, call_expression: &mut CallExpr) {
        let Some(argument) = call_expression.args.get_mut(1) else {
            return;
        };

        if !is_generated_rc4_decode_key_literal(argument.expr.as_ref()) {
            return;
        }

        self.replace_argument_with_storage_member(argument);
    }

    fn replace_argument_with_storage_member(&mut self, argument: &mut ExprOrSpread) {
        let storage_key = format!("_0x{}", self.entries.len());
        let original_argument = argument.expr.as_ref().clone();

        self.entries.push((storage_key.clone(), original_argument));
        *argument.expr =
            create_string_array_calls_storage_member_expression(&self.storage_name, &storage_key);
    }
}

fn insert_string_array_declarations(
    program: &mut Program,
    storage_name: &str,
    values: Vec<StringArrayValue>,
    index_shift_enabled: bool,
    encoding: StringArrayEncoding,
    wrappers: &[StringArrayCallWrapper],
    self_defending: bool,
) {
    let mut statements = vec![create_storage_statement(storage_name, values)];

    if should_emit_string_array_wrapper(index_shift_enabled, encoding) {
        statements.push(create_string_array_wrapper_statement(
            storage_name,
            SHIFTED_WRAPPER_NAME,
            INDEX_SHIFT_AMOUNT,
            index_shift_enabled,
            encoding,
            self_defending,
        ));
        statements.extend(
            wrappers
                .iter()
                .map(create_string_array_call_wrapper_statement),
        );
    }

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

fn create_storage_statement(
    storage_name: &str,
    values: Vec<StringArrayValue>,
) -> swc_ecma_ast::Stmt {
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
                    .map(|value| {
                        Some(create_expr_or_spread(create_string_literal(
                            &value.encoded_value,
                        )))
                    })
                    .collect(),
            }))),
            definite: false,
        }],
    })))
}

fn create_string_array_calls_storage_statement(
    storage_name: &str,
    entries: Vec<(String, Expr)>,
) -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(VarDecl {
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
            init: Some(Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: entries
                    .into_iter()
                    .map(|(key, value)| {
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(IdentName::from(key)),
                            value: Box::new(value),
                        })))
                    })
                    .collect(),
            }))),
            definite: false,
        }],
    })))
}

fn create_variable_wrapper_statement(wrapper_name: &str, root_wrapper_name: &str) -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        ctxt: Default::default(),
        kind: VarDeclKind::Const,
        declare: false,
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(BindingIdent {
                id: create_identifier(wrapper_name),
                type_ann: None,
            }),
            init: Some(Box::new(Expr::Ident(create_identifier(root_wrapper_name)))),
            definite: false,
        }],
    })))
}

fn create_string_array_call_wrapper_statement(wrapper: &StringArrayCallWrapper) -> Stmt {
    match wrapper.kind {
        StringArrayCallWrapperKind::Variable => {
            create_variable_wrapper_statement(&wrapper.name, SHIFTED_WRAPPER_NAME)
        }
        StringArrayCallWrapperKind::Function => create_function_wrapper_statement(wrapper),
        StringArrayCallWrapperKind::Root => {
            unreachable!("root wrapper is emitted separately from scope call wrappers")
        }
    }
}

fn create_function_wrapper_statement(wrapper: &StringArrayCallWrapper) -> Stmt {
    let parameters: Vec<Param> = function_wrapper_parameter_names(wrapper.parameters_count)
        .into_iter()
        .map(|name| create_parameter(&name))
        .collect();
    let index_argument = create_sub_expression(
        Expr::Ident(create_identifier("index")),
        create_number_literal(wrapper.index_shift),
    );

    Stmt::Decl(Decl::Fn(FnDecl {
        ident: create_identifier(&wrapper.name),
        declare: false,
        function: Box::new(Function {
            params: parameters,
            decorators: Vec::new(),
            span: DUMMY_SP,
            ctxt: Default::default(),
            body: Some(BlockStmt {
                span: DUMMY_SP,
                ctxt: Default::default(),
                stmts: vec![create_return_statement(Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    ctxt: Default::default(),
                    callee: Callee::Expr(Box::new(Expr::Ident(create_identifier(
                        SHIFTED_WRAPPER_NAME,
                    )))),
                    args: vec![
                        create_expr_or_spread(index_argument),
                        create_expr_or_spread(Expr::Ident(create_identifier("key"))),
                    ],
                    type_args: None,
                }))],
            }),
            is_generator: false,
            is_async: false,
            type_params: None,
            return_type: None,
        }),
    }))
}

fn function_wrapper_parameter_names(parameters_count: usize) -> Vec<String> {
    let mut parameter_names = vec!["index".to_string(), "key".to_string()];

    parameter_names.extend((0..parameters_count.saturating_sub(2)).map(|index| {
        if index == 0 {
            "unused0".to_string()
        } else {
            format!("unused{index}")
        }
    }));

    parameter_names.truncate(parameters_count.max(2));
    parameter_names
}

fn create_parameter(name: &str) -> Param {
    Param {
        span: DUMMY_SP,
        decorators: Vec::new(),
        pat: Pat::Ident(BindingIdent {
            id: create_identifier(name),
            type_ann: None,
        }),
    }
}

fn root_call_wrappers(
    wrappers_count: usize,
    wrappers_type: StringArrayWrappersType,
    wrappers_parameters_max_count: usize,
) -> Vec<StringArrayCallWrapper> {
    (0..wrappers_count)
        .map(|index| StringArrayCallWrapper {
            name: format!("_0x{}", index + 2),
            kind: string_array_call_wrapper_kind(wrappers_type),
            index_shift: if wrappers_type == StringArrayWrappersType::Function {
                index + 1
            } else {
                0
            },
            parameters_count: wrappers_parameters_max_count.max(2),
        })
        .collect()
}

fn string_array_call_wrapper_kind(
    wrappers_type: StringArrayWrappersType,
) -> StringArrayCallWrapperKind {
    match wrappers_type {
        StringArrayWrappersType::Function => StringArrayCallWrapperKind::Function,
        StringArrayWrappersType::Variable => StringArrayCallWrapperKind::Variable,
    }
}

fn active_string_array_call_wrappers(
    wrappers: &[StringArrayCallWrapper],
) -> Vec<StringArrayCallWrapper> {
    if wrappers.is_empty() {
        return vec![root_string_array_call_wrapper()];
    }

    wrappers.to_vec()
}

fn root_string_array_call_wrapper() -> StringArrayCallWrapper {
    StringArrayCallWrapper {
        name: SHIFTED_WRAPPER_NAME.to_string(),
        kind: StringArrayCallWrapperKind::Root,
        index_shift: 0,
        parameters_count: 2,
    }
}

fn first_index_type(index_types: &[StringArrayIndexesType]) -> StringArrayIndexesType {
    index_types
        .first()
        .copied()
        .unwrap_or(StringArrayIndexesType::HexadecimalNumber)
}

fn should_emit_string_array_wrapper(
    _index_shift_enabled: bool,
    _encoding: StringArrayEncoding,
) -> bool {
    true
}

fn encode_string_array_value(value: &str, encoding: StringArrayEncoding) -> StringArrayValue {
    match encoding {
        StringArrayEncoding::None => StringArrayValue {
            encoded_value: value.to_string(),
            decode_key: None,
        },
        StringArrayEncoding::Base64 => StringArrayValue {
            encoded_value: encode_base64_swapped(value),
            decode_key: None,
        },
        StringArrayEncoding::Rc4 => StringArrayValue {
            encoded_value: encode_base64_swapped_bytes(&rc4_bytes(
                value.as_bytes(),
                DEFAULT_RC4_KEY,
            )),
            decode_key: Some(DEFAULT_RC4_KEY),
        },
    }
}

fn encode_base64_swapped(value: &str) -> String {
    encode_base64_swapped_bytes(value.as_bytes())
}

fn encode_base64_swapped_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);

        encoded.push(BASE64_ALPHABET_SWAPPED[(first >> 2) as usize] as char);
        encoded.push(
            BASE64_ALPHABET_SWAPPED[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize]
                as char,
        );

        if chunk.len() > 1 {
            encoded.push(
                BASE64_ALPHABET_SWAPPED[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize]
                    as char,
            );
        }

        if chunk.len() > 2 {
            encoded.push(BASE64_ALPHABET_SWAPPED[(third & 0b0011_1111) as usize] as char);
        }
    }

    encoded
}

fn rc4_bytes(input: &[u8], key: &str) -> Vec<u8> {
    let key_bytes = key.as_bytes();
    let mut state = [0_u8; 256];

    for (index, value) in state.iter_mut().enumerate() {
        *value = index as u8;
    }

    let mut j = 0_usize;
    for i in 0..256 {
        j = (j + state[i] as usize + key_bytes[i % key_bytes.len()] as usize) % 256;
        state.swap(i, j);
    }

    let mut i = 0_usize;
    j = 0;

    input
        .iter()
        .map(|byte| {
            i = (i + 1) % 256;
            j = (j + state[i] as usize) % 256;
            state.swap(i, j);
            let key_stream_index = (state[i] as usize + state[j] as usize) % 256;

            byte ^ state[key_stream_index]
        })
        .collect()
}

fn create_string_array_reference_expression(
    storage_name: &str,
    wrapper: &StringArrayCallWrapper,
    index: usize,
    index_type: StringArrayIndexesType,
    encoding: StringArrayEncoding,
    index_shift_enabled: bool,
    decode_key: Option<&str>,
) -> Expr {
    if should_emit_string_array_wrapper(index_shift_enabled, encoding) {
        let wrapper_index = if index_shift_enabled {
            index + INDEX_SHIFT_AMOUNT
        } else {
            index
        };
        let wrapper_index = wrapper_index + wrapper.index_shift;

        return create_string_array_call_expression(wrapper, wrapper_index, index_type, decode_key);
    }

    create_string_array_member_expression(storage_name, index, index_type)
}

fn create_string_array_call_expression(
    wrapper: &StringArrayCallWrapper,
    index: usize,
    index_type: StringArrayIndexesType,
    decode_key: Option<&str>,
) -> Expr {
    let args = match wrapper.kind {
        StringArrayCallWrapperKind::Function => {
            create_function_wrapper_call_arguments(index, index_type, decode_key, wrapper)
        }
        StringArrayCallWrapperKind::Root | StringArrayCallWrapperKind::Variable => {
            create_root_wrapper_call_arguments(index, index_type, decode_key)
        }
    };

    Expr::Call(CallExpr {
        span: DUMMY_SP,
        ctxt: Default::default(),
        callee: Callee::Expr(Box::new(Expr::Ident(create_identifier(&wrapper.name)))),
        args,
        type_args: None,
    })
}

fn create_root_wrapper_call_arguments(
    index: usize,
    index_type: StringArrayIndexesType,
    decode_key: Option<&str>,
) -> Vec<ExprOrSpread> {
    let mut args = vec![create_expr_or_spread(create_index_literal(
        index, index_type,
    ))];

    if let Some(decode_key) = decode_key {
        args.push(create_expr_or_spread(create_string_literal(decode_key)));
    }

    args
}

fn create_function_wrapper_call_arguments(
    index: usize,
    index_type: StringArrayIndexesType,
    decode_key: Option<&str>,
    wrapper: &StringArrayCallWrapper,
) -> Vec<ExprOrSpread> {
    let parameters_count = wrapper.parameters_count.max(2);

    (0..parameters_count)
        .map(|argument_index| {
            if argument_index == 0 {
                return create_expr_or_spread(create_index_literal(index, index_type));
            }

            if argument_index == 1 {
                if let Some(decode_key) = decode_key {
                    return create_expr_or_spread(create_string_literal(decode_key));
                }
            }

            create_expr_or_spread(create_index_literal(index + argument_index, index_type))
        })
        .collect()
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

fn create_string_array_calls_storage_member_expression(
    storage_name: &str,
    storage_key: &str,
) -> Expr {
    Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(create_identifier(storage_name))),
        prop: MemberProp::Ident(IdentName::from(storage_key.to_string())),
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

fn create_string_array_wrapper_statement(
    storage_name: &str,
    wrapper_name: &str,
    shift_amount: usize,
    index_shift_enabled: bool,
    encoding: StringArrayEncoding,
    self_defending: bool,
) -> Stmt {
    match encoding {
        StringArrayEncoding::None => create_index_shift_wrapper_statement(
            storage_name,
            wrapper_name,
            shift_amount,
            index_shift_enabled,
        ),
        StringArrayEncoding::Base64 => create_base64_wrapper_statement(
            storage_name,
            wrapper_name,
            shift_amount,
            index_shift_enabled,
            self_defending,
        ),
        StringArrayEncoding::Rc4 => create_rc4_wrapper_statement(
            storage_name,
            wrapper_name,
            shift_amount,
            index_shift_enabled,
            self_defending,
        ),
    }
}

fn create_base64_wrapper_statement(
    storage_name: &str,
    wrapper_name: &str,
    shift_amount: usize,
    index_shift_enabled: bool,
    self_defending: bool,
) -> Stmt {
    let index_expression = if index_shift_enabled {
        format!("index-0x{shift_amount:x}")
    } else {
        "index".to_string()
    };
    let self_defending_prelude = if self_defending {
        format!(
            "let func=output+{wrapper_name};let __=(''+function(){{return 0;}}).indexOf('\\n')!==-0x1;"
        )
    } else {
        String::new()
    };
    let decoded_character_expression = guarded_base64_character_expression(self_defending);
    let wrapper_source = format!(
        "function {wrapper_name}(index){{let value={storage_name}[{index_expression}];const chars='abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/=';let output='';let tempEncodedString='';{self_defending_prelude}for(let bc=0,bs,buffer,idx=0;buffer=value.charAt(idx++);~buffer&&(bs=bc%4?bs*64+buffer:buffer,bc++%4)?output+={decoded_character_expression}:0){{buffer=chars.indexOf(buffer);}}for(let k=0,length=output.length;k<length;k++){{tempEncodedString+='%'+('00'+output.charCodeAt(k).toString(0x10)).slice(-0x2);}}return decodeURIComponent(tempEncodedString);}}"
    );
    let parsed_program = parse_program(&wrapper_source).expect("base64 wrapper should parse");

    match parsed_program.program {
        Program::Script(script) => script
            .body
            .into_iter()
            .next()
            .expect("base64 wrapper should contain a statement"),
        Program::Module(module) => module
            .body
            .into_iter()
            .find_map(|item| match item {
                ModuleItem::Stmt(statement) => Some(statement),
                ModuleItem::ModuleDecl(_) => None,
            })
            .expect("base64 wrapper should contain a statement"),
    }
}

fn create_rc4_wrapper_statement(
    storage_name: &str,
    wrapper_name: &str,
    shift_amount: usize,
    index_shift_enabled: bool,
    self_defending: bool,
) -> Stmt {
    let index_expression = if index_shift_enabled {
        format!("index-0x{shift_amount:x}")
    } else {
        "index".to_string()
    };
    let self_defending_prelude = if self_defending {
        format!(
            "let func=data+{wrapper_name};let __=(''+function(){{return 0;}}).indexOf('\\n')!==-0x1;"
        )
    } else {
        String::new()
    };
    let decoded_character_expression = guarded_base64_character_expression(self_defending);
    let wrapper_source = format!(
        "function {wrapper_name}(index,key){{let value={storage_name}[{index_expression}];const chars='abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/=';let data='';{self_defending_prelude}for(let bc=0,bs,buffer,idx=0;buffer=value.charAt(idx++);~buffer&&(bs=bc%4?bs*64+buffer:buffer,bc++%4)?data+={decoded_character_expression}:0){{buffer=chars.indexOf(buffer);}}let s=[],j=0,x,output='';let i;for(i=0;i<0x100;i++){{s[i]=i;}}for(i=0;i<0x100;i++){{j=(j+s[i]+key.charCodeAt(i%key.length))%0x100;x=s[i];s[i]=s[j];s[j]=x;}}i=0;j=0;for(let y=0;y<data.length;y++){{i=(i+0x1)%0x100;j=(j+s[i])%0x100;x=s[i];s[i]=s[j];s[j]=x;output+=String.fromCharCode(data.charCodeAt(y)^s[(s[i]+s[j])%0x100]);}}let tempEncodedString='';for(let k=0,length=output.length;k<length;k++){{tempEncodedString+='%'+('00'+output.charCodeAt(k).toString(0x10)).slice(-0x2);}}return decodeURIComponent(tempEncodedString);}}"
    );
    let parsed_program = parse_program(&wrapper_source).expect("rc4 wrapper should parse");

    match parsed_program.program {
        Program::Script(script) => script
            .body
            .into_iter()
            .next()
            .expect("rc4 wrapper should contain a statement"),
        Program::Module(module) => module
            .body
            .into_iter()
            .find_map(|item| match item {
                ModuleItem::Stmt(statement) => Some(statement),
                ModuleItem::ModuleDecl(_) => None,
            })
            .expect("rc4 wrapper should contain a statement"),
    }
}

fn guarded_base64_character_expression(self_defending: bool) -> &'static str {
    let base_expression = "String.fromCharCode(0xff&bs>>(-0x2*bc&0x6))";

    if self_defending {
        "((__||func.charCodeAt(idx+0xa)-0xa!==0)?String.fromCharCode(0xff&bs>>(-0x2*bc&0x6)):bc)"
    } else {
        base_expression
    }
}

fn create_index_shift_wrapper_statement(
    storage_name: &str,
    wrapper_name: &str,
    shift_amount: usize,
    index_shift_enabled: bool,
) -> Stmt {
    let index_expression = if index_shift_enabled {
        create_sub_expression(
            Expr::Ident(create_identifier("index")),
            create_number_literal(shift_amount),
        )
    } else {
        Expr::Ident(create_identifier("index"))
    };

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
                        index_expression,
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

fn is_generated_string_array_call_index_literal(expression: &Expr) -> bool {
    match expression {
        Expr::Lit(Lit::Num(number)) if number.span == DUMMY_SP => {
            index_from_literal(expression).is_some()
        }
        Expr::Lit(Lit::Str(string)) if string.span == DUMMY_SP => {
            index_from_literal(expression).is_some()
        }
        _ => false,
    }
}

fn is_generated_rc4_decode_key_literal(expression: &Expr) -> bool {
    let Expr::Lit(Lit::Str(string)) = expression else {
        return false;
    };

    string.span == DUMMY_SP && string.value == DEFAULT_RC4_KEY
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

fn string_array_call_wrapper_for_call<'a>(
    call_expression: &CallExpr,
    wrappers: &'a [StringArrayCallWrapper],
) -> Option<&'a StringArrayCallWrapper> {
    wrappers
        .iter()
        .find(|wrapper| is_identifier_callee(call_expression, &wrapper.name))
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

fn compile_force_transform_patterns(force_transform_strings: &[String]) -> Vec<Regex> {
    compile_patterns(force_transform_strings)
}

fn is_force_transform_string(value: &str, force_transform_patterns: &[Regex]) -> bool {
    is_matching_pattern(value, force_transform_patterns)
}

fn compile_patterns(patterns: &[String]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect()
}

fn is_matching_pattern(value: &str, patterns: &[Regex]) -> bool {
    patterns.iter().any(|pattern| pattern.is_match(value))
}

fn has_minimum_length_for_string_array(value: &str) -> bool {
    value.encode_utf16().count() >= MINIMUM_LENGTH_FOR_STRING_ARRAY
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
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: false,
                shuffle: false,
                rotate: false,
                reserved_strings,
                force_transform_strings: &[],
                ignore_imports,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
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
            code.contains("function _0x1(index){return _0x0[index];}"),
            "{code}"
        );
        assert!(
            code.contains("const value=_0x1(0x0);console.log(_0x1(0x0));"),
            "{code}"
        );
        assert!(!code.contains("const value=_0x0[0x0];"), "{code}");
    }

    #[test]
    fn inserts_string_array_declarations_after_directives() {
        let code = transform("'use strict'; const value = 'test';", true, &[], false);

        assert!(code.starts_with("'use strict';const _0x0="), "{code}");
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
        assert!(code.contains("const take=_0x1(0x0);"), "{code}");
    }

    #[test]
    fn keeps_regex_reserved_string_literals_inline() {
        let code = transform(
            "const foo = 'foo'; const bar = 'bar';",
            true,
            &["ar$".to_string()],
            false,
        );

        assert!(code.contains("const _0x0=['foo'];"), "{code}");
        assert!(code.contains("const foo=_0x1(0x0);"), "{code}");
        assert!(code.contains("const bar='bar';"), "{code}");
    }

    #[test]
    fn respects_string_array_minimum_length_for_normal_strings() {
        let code = transform(
            "const a = 'f'; const b = 'fo'; const c = 'foo';",
            true,
            &[],
            false,
        );

        assert!(code.contains("const _0x0=['foo'];"), "{code}");
        assert!(code.contains("const a='f';const b='fo';"), "{code}");
        assert!(code.contains("const c=_0x1(0x0);"), "{code}");
        assert!(!code.contains("'f','fo','foo'"), "{code}");
    }

    #[test]
    fn force_transform_strings_override_string_array_minimum_length() {
        let force_transform_strings = vec!["^f$".to_string()];
        let mut parsed_program = parse_program("const value = 'f';").expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 0.0,
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: false,
                shuffle: false,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &force_transform_strings,
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const _0x0=['f'];"), "{code}");
        assert!(code.contains("const value=_0x1(0x0);"), "{code}");
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
        assert!(code.contains("const bar=_0x1(0x0);"), "{code}");
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
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: false,
                shuffle: false,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &[],
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const value='test';"), "{code}");
        assert!(!code.contains("const _0x0=["), "{code}");
    }

    #[test]
    fn force_transforms_matching_string_when_threshold_is_zero() {
        let force_transform_strings = vec!["ar$".to_string()];
        let mut parsed_program =
            parse_program("const foo = 'foo'; const bar = 'bar';").expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 0.0,
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: false,
                shuffle: false,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &force_transform_strings,
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const _0x0=['bar'];"), "{code}");
        assert!(code.contains("const foo='foo';"), "{code}");
        assert!(code.contains("const bar=_0x1(0x0);"), "{code}");
    }

    #[test]
    fn force_transform_strings_take_priority_over_reserved_strings() {
        let reserved_strings = vec!["bar".to_string()];
        let force_transform_strings = vec!["bar".to_string()];
        let mut parsed_program =
            parse_program("const foo = 'foo'; const bar = 'bar';").expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 0.0,
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: false,
                shuffle: false,
                rotate: false,
                reserved_strings: &reserved_strings,
                force_transform_strings: &force_transform_strings,
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const _0x0=['bar'];"), "{code}");
        assert!(code.contains("const foo='foo';"), "{code}");
        assert!(code.contains("const bar=_0x1(0x0);"), "{code}");
    }

    #[test]
    fn uses_string_array_root_wrapper_without_index_shift() {
        let mut parsed_program =
            parse_program("const value = 'test';").expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 1.0,
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: false,
                shuffle: false,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &[],
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(
            code.contains("function _0x1(index){return _0x0[index];}"),
            "{code}"
        );
        assert!(code.contains("const value=_0x1(0x0);"), "{code}");
        assert!(!code.contains("const value=_0x0[0x0];"), "{code}");
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
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[StringArrayIndexesType::HexadecimalNumericString],
                encoding: StringArrayEncoding::None,
                index_shift: false,
                shuffle: false,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &[],
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const value=_0x1('0x0');"), "{code}");
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
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: true,
                shuffle: false,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &[],
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
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
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: true,
                shuffle: true,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &[],
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
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
    fn remaps_root_variable_string_array_wrappers_when_shuffle_is_enabled() {
        let mut parsed_program =
            parse_program("const first = 'foo'; const second = 'bar'; const third = 'baz';")
                .expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 1.0,
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: true,
                shuffle: true,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &[],
                ignore_imports: false,
                wrappers_count: 2,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const _0x0=['baz','bar','foo'];"), "{code}");
        assert!(
            code.contains(
                "function _0x1(index){return _0x0[index-0x64];}const _0x2=_0x1;const _0x3=_0x1;"
            ),
            "{code}"
        );
        assert!(
            code.contains("const first=_0x2(0x66);const second=_0x3(0x65);const third=_0x2(0x64);"),
            "{code}"
        );
    }

    #[test]
    fn remaps_root_function_string_array_wrappers_when_shuffle_is_enabled() {
        let mut parsed_program =
            parse_program("const first = 'foo'; const second = 'bar'; const third = 'baz';")
                .expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 1.0,
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: true,
                shuffle: true,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &[],
                ignore_imports: false,
                wrappers_count: 2,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Function,
                wrappers_chained_calls: false,
                self_defending: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const _0x0=['baz','bar','foo'];"), "{code}");
        assert!(
            code.contains(
                "function _0x2(index,key){return _0x1(index-0x1,key);}function _0x3(index,key){return _0x1(index-0x2,key);}"
            ),
            "{code}"
        );
        assert!(
            code.contains("const first=_0x2(0x67,0x66);const second=_0x3(0x67,0x68);const third=_0x2(0x65,0x68);"),
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
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::None,
                index_shift: true,
                shuffle: false,
                rotate: true,
                reserved_strings: &[],
                force_transform_strings: &[],
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
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

    #[test]
    fn remaps_base64_wrapper_indexes_when_string_array_shuffle_is_enabled() {
        let mut parsed_program = parse_program("const first = 'foo'; const second = 'bar';")
            .expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 1.0,
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::Base64,
                index_shift: false,
                shuffle: true,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &[],
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("const _0x0=['yMfY','zM9V'];"), "{code}");
        assert!(code.contains("function _0x1(index)"), "{code}");
        assert!(
            code.contains("const first=_0x1(0x1);const second=_0x1(0x0);"),
            "{code}"
        );
    }

    #[test]
    fn remaps_rc4_wrapper_indexes_when_string_array_shuffle_is_enabled() {
        let mut parsed_program = parse_program("const first = 'foo'; const second = 'bar';")
            .expect("source should parse");
        transform_string_array(
            &mut parsed_program.program,
            StringArrayTransformOptions {
                enabled: true,
                threshold: 1.0,
                calls_transform: false,
                calls_transform_threshold: 0.0,
                indexes_type: &[],
                encoding: StringArrayEncoding::Rc4,
                index_shift: false,
                shuffle: true,
                rotate: false,
                reserved_strings: &[],
                force_transform_strings: &[],
                ignore_imports: false,
                wrappers_count: 0,
                wrappers_parameters_max_count: 2,
                wrappers_type: StringArrayWrappersType::Variable,
                wrappers_chained_calls: false,
                self_defending: false,
            },
        );
        let code = generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate");

        assert!(code.contains("function _0x1(index,key)"), "{code}");
        assert!(
            code.contains("const first=_0x1(0x1,'rc4K');const second=_0x1(0x0,'rc4K');"),
            "{code}"
        );
    }
}
