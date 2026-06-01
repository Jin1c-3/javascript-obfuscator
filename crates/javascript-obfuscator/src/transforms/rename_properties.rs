use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use regex::Regex;
use swc_common::DUMMY_SP;
use swc_ecma_ast::{
    AssignPat, AssignPatProp, Expr, IdentName, KeyValuePatProp, Lit, MemberExpr, MemberProp,
    ObjectPat, ObjectPatProp, Pat, Program, PropName, Str,
};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith};

use crate::generators::{IdentifierNamesGenerator, IdentifierNamesGeneratorKind};

const UNSAFE_MODE: &str = "unsafe";
const SAFE_MODE: &str = "safe";
const RESERVED_DOM_PROPERTY_NAMES_JSON: &str =
    include_str!("../../assets/ReservedDomProperties.json");
static RESERVED_DOM_PROPERTY_NAMES: OnceLock<HashSet<String>> = OnceLock::new();

pub fn transform_rename_properties(
    program: &mut Program,
    enabled: bool,
    mode: Option<&str>,
    generator_kind: IdentifierNamesGeneratorKind,
    identifiers_prefix: &str,
    identifiers_dictionary: &[String],
    reserved_names: &[String],
) {
    if !enabled {
        return;
    }

    let mode = mode.unwrap_or(SAFE_MODE);
    if mode != SAFE_MODE && mode != UNSAFE_MODE {
        return;
    }

    let excluded_property_names = if mode == SAFE_MODE {
        collect_auto_excluded_property_names(program)
    } else {
        HashSet::new()
    };

    program.visit_mut_with(&mut RenamePropertiesTransform {
        generator: IdentifierNamesGenerator::new(
            generator_kind,
            identifiers_prefix,
            identifiers_dictionary.to_vec(),
        ),
        excluded_property_names,
        generator_kind,
        property_names: HashMap::new(),
        reserved_name_patterns: compile_patterns(reserved_names),
    });
}

struct AutoExcludedPropertyNamesCollector {
    excluded_property_names: HashSet<String>,
}

struct RenamePropertiesTransform {
    generator: IdentifierNamesGenerator,
    excluded_property_names: HashSet<String>,
    generator_kind: IdentifierNamesGeneratorKind,
    property_names: HashMap<String, String>,
    reserved_name_patterns: Vec<Regex>,
}

impl Visit for AutoExcludedPropertyNamesCollector {
    fn visit_member_prop(&mut self, member_prop: &MemberProp) {
        match member_prop {
            MemberProp::Computed(computed_property_name) => {
                if !matches!(computed_property_name.expr.as_ref(), Expr::Lit(Lit::Str(_))) {
                    computed_property_name.expr.visit_with(self);
                }
            }
            MemberProp::Ident(_) | MemberProp::PrivateName(_) => {}
        }
    }

    fn visit_prop_name(&mut self, property_name: &PropName) {
        match property_name {
            PropName::Computed(computed_property_name) => {
                if !matches!(computed_property_name.expr.as_ref(), Expr::Lit(Lit::Str(_))) {
                    computed_property_name.expr.visit_with(self);
                }
            }
            PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) | PropName::BigInt(_) => {}
        }
    }

    fn visit_str(&mut self, string_literal: &Str) {
        self.excluded_property_names
            .insert(string_literal.value.to_string_lossy().into_owned());
    }
}

impl VisitMut for RenamePropertiesTransform {
    fn visit_mut_prop_name(&mut self, property_name: &mut PropName) {
        property_name.visit_mut_children_with(self);

        match property_name {
            PropName::Ident(identifier) => {
                let renamed = self.rename_property_name(identifier.sym.as_ref());
                *property_name = create_string_property_name(&renamed);
            }
            PropName::Str(string_literal) => {
                let renamed = self.rename_property_name(&string_literal.value.to_string_lossy());
                *string_literal = create_string_literal(&renamed);
            }
            PropName::Computed(computed_property_name) => {
                let Expr::Lit(Lit::Str(string_literal)) = computed_property_name.expr.as_ref()
                else {
                    return;
                };
                let renamed = self.rename_property_name(&string_literal.value.to_string_lossy());
                *computed_property_name.expr = Expr::Lit(Lit::Str(create_string_literal(&renamed)));
            }
            _ => {}
        }
    }

    fn visit_mut_member_expr(&mut self, member_expr: &mut MemberExpr) {
        member_expr.visit_mut_children_with(self);

        match &mut member_expr.prop {
            MemberProp::Ident(identifier) => {
                let renamed = self.rename_property_name(identifier.sym.as_ref());
                *identifier = IdentName::from(renamed);
            }
            MemberProp::Computed(computed_property_name) => {
                let Expr::Lit(Lit::Str(string_literal)) = computed_property_name.expr.as_ref()
                else {
                    return;
                };
                let renamed = self.rename_property_name(&string_literal.value.to_string_lossy());
                *computed_property_name.expr = Expr::Lit(Lit::Str(create_string_literal(&renamed)));
            }
            MemberProp::PrivateName(_) => {}
        }
    }

    fn visit_mut_object_pat(&mut self, object_pattern: &mut ObjectPat) {
        object_pattern.visit_mut_children_with(self);

        for property in &mut object_pattern.props {
            let ObjectPatProp::Assign(assign_property) = property else {
                continue;
            };

            *property = self.create_key_value_pattern_property(assign_property);
        }
    }
}

impl RenamePropertiesTransform {
    fn rename_property_name(&mut self, name: &str) -> String {
        if self.should_keep_name(name)
            || self.generator_kind == IdentifierNamesGeneratorKind::KeepOriginal
        {
            return name.to_string();
        }

        if let Some(renamed) = self.property_names.get(name) {
            return renamed.clone();
        }

        let renamed = self.generator.generate_next();
        self.property_names
            .insert(name.to_string(), renamed.clone());

        renamed
    }

    fn should_keep_name(&self, name: &str) -> bool {
        self.excluded_property_names.contains(name)
            || reserved_dom_property_names().contains(name)
            || self
                .reserved_name_patterns
                .iter()
                .any(|reserved_name_pattern| reserved_name_pattern.is_match(name))
    }

    fn create_key_value_pattern_property(
        &mut self,
        assign_property: &AssignPatProp,
    ) -> ObjectPatProp {
        let binding_identifier = assign_property.key.clone();
        let property_name = self.rename_property_name(binding_identifier.id.sym.as_ref());
        let key = create_string_property_name(&property_name);
        let value = match &assign_property.value {
            Some(default_value) => Pat::Assign(AssignPat {
                span: DUMMY_SP,
                left: Box::new(Pat::Ident(binding_identifier)),
                right: default_value.clone(),
            }),
            None => Pat::Ident(binding_identifier),
        };

        ObjectPatProp::KeyValue(KeyValuePatProp {
            key,
            value: Box::new(value),
        })
    }
}

fn collect_auto_excluded_property_names(program: &Program) -> HashSet<String> {
    let mut collector = AutoExcludedPropertyNamesCollector {
        excluded_property_names: HashSet::new(),
    };
    program.visit_with(&mut collector);

    collector.excluded_property_names
}

fn reserved_dom_property_names() -> &'static HashSet<String> {
    RESERVED_DOM_PROPERTY_NAMES.get_or_init(|| {
        serde_json::from_str::<Vec<String>>(RESERVED_DOM_PROPERTY_NAMES_JSON)
            .expect("reserved DOM property names JSON should parse")
            .into_iter()
            .collect()
    })
}

fn compile_patterns(patterns: &[String]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect()
}

fn create_string_property_name(value: &str) -> PropName {
    PropName::Str(create_string_literal(value))
}

fn create_string_literal(value: &str) -> Str {
    Str {
        span: DUMMY_SP,
        value: value.to_string().into(),
        raw: Some(single_quote_raw(value).into()),
    }
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

    fn transform(source_code: &str, mode: Option<&str>, reserved_names: &[String]) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_object_expressions(&mut parsed_program.program);
        transform_rename_properties(
            &mut parsed_program.program,
            true,
            mode,
            IdentifierNamesGeneratorKind::Hexadecimal,
            "",
            &[],
            reserved_names,
        );
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn renames_object_keys_and_member_expressions_consistently() {
        let code = transform(
            "const value = {'foo': 1}; value.foo; value['foo'];",
            Some(UNSAFE_MODE),
            &[],
        );

        assert!(code.contains("const value={'_0x0':1};"), "{code}");
        assert!(code.contains("value._0x0;"), "{code}");
        assert!(code.contains("value['_0x0'];"), "{code}");
    }

    #[test]
    fn keeps_reserved_properties() {
        let reserved_names = vec!["^keep$".to_string()];
        let code = transform(
            "const value = {'keep': 1, 'change': 2}; value.keep; value.change;",
            Some(UNSAFE_MODE),
            &reserved_names,
        );

        assert!(code.contains("'keep':1"), "{code}");
        assert!(code.contains("'_0x0':2"), "{code}");
        assert!(code.contains("value.keep;"), "{code}");
        assert!(code.contains("value._0x0;"), "{code}");
    }

    #[test]
    fn keeps_reserved_dom_properties() {
        let code = transform(
            "const value = {'then': 1, 'custom': 2}; value.then; value.custom;",
            Some(UNSAFE_MODE),
            &[],
        );

        assert!(code.contains("'then':1"), "{code}");
        assert!(code.contains("'_0x0':2"), "{code}");
        assert!(code.contains("value.then;"), "{code}");
        assert!(code.contains("value._0x0;"), "{code}");
    }

    #[test]
    fn skips_computed_non_string_members() {
        let code = transform(
            "const value = {'foo': 1}; value[foo];",
            Some(UNSAFE_MODE),
            &[],
        );

        assert!(code.contains("const value={'_0x0':1};"), "{code}");
        assert!(code.contains("value[foo];"), "{code}");
    }

    #[test]
    fn renames_shorthand_object_pattern_properties() {
        let code = transform(
            "const value = {'foo': 1}; const {foo} = value;",
            Some(UNSAFE_MODE),
            &[],
        );

        assert!(code.contains("const value={'_0x0':1};"), "{code}");
        assert!(code.contains("const{'_0x0':foo}=value;"), "{code}");
    }

    #[test]
    fn safe_mode_excludes_ordinary_string_literals() {
        let code = transform(
            "const value = {'foo': 1, 'bar': 2}; const excluded = 'foo'; value.foo; value.bar;",
            Some(SAFE_MODE),
            &[],
        );

        assert!(code.contains("'foo':1"), "{code}");
        assert!(code.contains("'_0x0':2"), "{code}");
        assert!(code.contains("value.foo;"), "{code}");
        assert!(code.contains("value._0x0;"), "{code}");
    }

    #[test]
    fn safe_mode_renames_property_position_string_literals() {
        let code = transform(
            "const value = {'bar': 1}; value['bar'];",
            Some(SAFE_MODE),
            &[],
        );

        assert!(code.contains("const value={'_0x0':1};"), "{code}");
        assert!(code.contains("value['_0x0'];"), "{code}");
    }

    #[test]
    fn missing_mode_defaults_to_safe_mode() {
        let code = transform(
            "const value = {'foo': 1, 'bar': 2}; const excluded = 'foo'; value.foo; value.bar;",
            None,
            &[],
        );

        assert!(code.contains("'foo':1"), "{code}");
        assert!(code.contains("'_0x0':2"), "{code}");
        assert!(code.contains("value.foo;"), "{code}");
        assert!(code.contains("value._0x0;"), "{code}");
    }
}
