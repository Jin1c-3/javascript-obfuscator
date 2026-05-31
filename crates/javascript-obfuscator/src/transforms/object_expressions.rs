use swc_common::DUMMY_SP;
use swc_ecma_ast::{Expr, KeyValueProp, ObjectLit, Program, Prop, PropName, PropOrSpread, Str};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_object_expressions(program: &mut Program) {
    program.visit_mut_with(&mut ObjectExpressionTransform);
}

struct ObjectExpressionTransform;

impl VisitMut for ObjectExpressionTransform {
    fn visit_mut_object_lit(&mut self, object_lit: &mut ObjectLit) {
        object_lit.visit_mut_children_with(self);

        for prop_or_spread in &mut object_lit.props {
            let PropOrSpread::Prop(property) = prop_or_spread else {
                continue;
            };

            transform_property(property);
        }
    }
}

fn transform_property(property: &mut Box<Prop>) {
    match property.as_mut() {
        Prop::Shorthand(identifier) => {
            let identifier = identifier.clone();
            let property_name = identifier.sym.to_string();

            **property = Prop::KeyValue(KeyValueProp {
                key: create_string_property_name(&property_name),
                value: Box::new(Expr::Ident(identifier)),
            });
        }
        Prop::KeyValue(key_value_property) => {
            transform_property_name(&mut key_value_property.key);
        }
        Prop::Getter(getter_property) => {
            transform_property_name(&mut getter_property.key);
        }
        Prop::Setter(setter_property) => {
            transform_property_name(&mut setter_property.key);
        }
        Prop::Method(method_property) => {
            transform_property_name(&mut method_property.key);
        }
        Prop::Assign(_) => {}
    }
}

fn transform_property_name(property_name: &mut PropName) {
    let PropName::Ident(identifier) = property_name else {
        return;
    };

    let name = identifier.sym.to_string();
    *property_name = create_string_property_name(&name);
}

fn create_string_property_name(value: &str) -> PropName {
    PropName::Str(Str {
        span: DUMMY_SP,
        value: value.to_string().into(),
        raw: Some(single_quote_raw(value).into()),
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

    fn transform(source_code: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_object_expressions(&mut parsed_program.program);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn transforms_identifier_property_key() {
        let code = transform("const value = {foo: bar};");

        assert!(code.contains("const value={'foo':bar}"), "{code}");
    }

    #[test]
    fn expands_shorthand_property() {
        let code = transform("const value = {foo};");

        assert!(code.contains("const value={'foo':foo}"), "{code}");
    }

    #[test]
    fn transforms_method_property_key() {
        let code = transform("const value = {foo() { return bar; }};");

        assert!(
            code.contains("const value={'foo'(){return bar;}}"),
            "{code}"
        );
    }

    #[test]
    fn preserves_spread_property() {
        let code = transform("const value = {...source};");

        assert!(code.contains("const value={...source}"), "{code}");
    }
}
