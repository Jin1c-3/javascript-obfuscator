use swc_common::DUMMY_SP;
use swc_ecma_ast::{ClassMember, ComputedPropName, Expr, Key, Lit, Program, PropName, Str};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_class_fields(program: &mut Program, reserved_names: &[String]) {
    program.visit_mut_with(&mut ClassFieldTransform { reserved_names });
}

struct ClassFieldTransform<'a> {
    reserved_names: &'a [String],
}

impl VisitMut for ClassFieldTransform<'_> {
    fn visit_mut_class_member(&mut self, class_member: &mut ClassMember) {
        class_member.visit_mut_children_with(self);

        match class_member {
            ClassMember::Method(class_method) => {
                transform_property_name(&mut class_method.key, self.reserved_names);
            }
            ClassMember::ClassProp(class_property) => {
                transform_property_name(&mut class_property.key, self.reserved_names);
            }
            ClassMember::AutoAccessor(auto_accessor) => {
                if let Key::Public(property_name) = &mut auto_accessor.key {
                    transform_property_name(property_name, self.reserved_names);
                }
            }
            _ => {}
        }
    }
}

fn transform_property_name(property_name: &mut PropName, reserved_names: &[String]) {
    let Some(name) = get_public_property_name(property_name) else {
        return;
    };

    if should_ignore_name(&name, reserved_names) {
        return;
    }

    *property_name = create_computed_string_property_name(&name);
}

fn get_public_property_name(property_name: &PropName) -> Option<String> {
    match property_name {
        PropName::Ident(identifier) => Some(identifier.sym.to_string()),
        PropName::Str(string_literal) => Some(string_literal.value.to_string_lossy().into_owned()),
        _ => None,
    }
}

fn should_ignore_name(name: &str, reserved_names: &[String]) -> bool {
    name == "constructor"
        || reserved_names
            .iter()
            .any(|reserved_name| reserved_name == name)
}

fn create_computed_string_property_name(value: &str) -> PropName {
    PropName::Computed(ComputedPropName {
        span: DUMMY_SP,
        expr: Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: value.to_string().into(),
            raw: Some(single_quote_raw(value).into()),
        }))),
    })
}

fn single_quote_raw(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");

    format!("'{escaped}'")
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, reserved_names: &[String]) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_class_fields(&mut parsed_program.program, reserved_names);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_method_identifier_key() {
        let code = transform("class Foo { bar() {} }", &[]);

        assert!(code.contains("class Foo{['bar'](){}}"), "{code}");
    }

    #[test]
    fn transforms_property_identifier_key() {
        let code = transform("class Foo { property = value; }", &[]);

        assert!(code.contains("class Foo{['property']=value;}"), "{code}");
    }

    #[test]
    fn keeps_constructor_key() {
        let code = transform("class Foo { constructor() {} }", &[]);

        assert!(code.contains("class Foo{constructor(){}}"), "{code}");
    }

    #[test]
    fn keeps_reserved_method_key() {
        let reserved_names = vec!["bar".to_string()];
        let code = transform("class Foo { bar() {} }", &reserved_names);

        assert!(code.contains("class Foo{bar(){}}"), "{code}");
    }
}
