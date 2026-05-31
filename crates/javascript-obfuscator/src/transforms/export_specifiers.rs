use swc_ecma_ast::{ExportNamedSpecifier, Program};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_export_specifiers(program: &mut Program, rename_globals: bool) {
    if !rename_globals {
        return;
    }

    program.visit_mut_with(&mut ExportSpecifierTransform);
}

struct ExportSpecifierTransform;

impl VisitMut for ExportSpecifierTransform {
    fn visit_mut_export_named_specifier(&mut self, export_specifier: &mut ExportNamedSpecifier) {
        if export_specifier.exported.is_none() {
            export_specifier.exported = Some(export_specifier.orig.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, rename_globals: bool) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_export_specifiers(&mut parsed_program.program, rename_globals);
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn aliases_shorthand_export_when_rename_globals_is_enabled() {
        let code = transform("const foo = 1; export {foo};", true);

        assert!(code.contains("export{foo as foo};"), "{code}");
    }

    #[test]
    fn keeps_shorthand_export_when_rename_globals_is_disabled() {
        let code = transform("const foo = 1; export {foo};", false);

        assert!(code.contains("export{foo};"), "{code}");
    }

    #[test]
    fn keeps_existing_export_alias() {
        let code = transform("const foo = 1; export {foo as bar};", true);

        assert!(code.contains("export{foo as bar};"), "{code}");
    }

    #[test]
    fn keeps_namespace_export_specifier() {
        let code = transform("export * as ns from 'mod';", true);

        assert!(code.contains("export*as ns from'mod';"), "{code}");
    }
}
