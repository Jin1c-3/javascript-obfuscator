use swc_ecma_ast::{BreakStmt, ContinueStmt, LabeledStmt, Program};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::generators::{IdentifierNamesGenerator, IdentifierNamesGeneratorKind};

pub fn transform_labeled_statements(
    program: &mut Program,
    identifier_names_generator: IdentifierNamesGeneratorKind,
    identifiers_prefix: &str,
    identifiers_dictionary: &[String],
) {
    program.visit_mut_with(&mut LabeledStatementTransform {
        generator: IdentifierNamesGenerator::new(
            identifier_names_generator,
            identifiers_prefix,
            identifiers_dictionary.to_vec(),
        ),
    });
}

struct LabeledStatementTransform {
    generator: IdentifierNamesGenerator,
}

impl VisitMut for LabeledStatementTransform {
    fn visit_mut_labeled_stmt(&mut self, labeled_statement: &mut LabeledStmt) {
        let original_label_name = labeled_statement.label.sym.to_string();
        let next_label_name = self.generator.generate_next();

        labeled_statement.label.sym = next_label_name.clone().into();
        labeled_statement
            .body
            .visit_mut_with(&mut LabelReferenceTransform {
                original_label_name,
                next_label_name,
            });
        labeled_statement.body.visit_mut_with(self);
    }
}

struct LabelReferenceTransform {
    original_label_name: String,
    next_label_name: String,
}

impl VisitMut for LabelReferenceTransform {
    fn visit_mut_break_stmt(&mut self, break_statement: &mut BreakStmt) {
        replace_label_symbol(
            break_statement.label.as_mut(),
            &self.original_label_name,
            &self.next_label_name,
        );
    }

    fn visit_mut_continue_stmt(&mut self, continue_statement: &mut ContinueStmt) {
        replace_label_symbol(
            continue_statement.label.as_mut(),
            &self.original_label_name,
            &self.next_label_name,
        );
    }
}

fn replace_label_symbol(
    label: Option<&mut swc_ecma_ast::Ident>,
    original_label_name: &str,
    next_label_name: &str,
) {
    let Some(label) = label else {
        return;
    };

    if label.sym == original_label_name {
        label.sym = next_label_name.into();
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::generators::IdentifierNamesGeneratorKind;
    use crate::parser::parse_program;

    use super::*;

    fn transform(
        source_code: &str,
        identifier_names_generator: IdentifierNamesGeneratorKind,
        identifiers_dictionary: &[String],
    ) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_labeled_statements(
            &mut parsed_program.program,
            identifier_names_generator,
            "",
            identifiers_dictionary,
        );
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn renames_labeled_statement_and_matching_references() {
        let code = transform(
            "label: for (;;) { continue label; break label; }",
            IdentifierNamesGeneratorKind::Hexadecimal,
            &[],
        );

        assert!(
            code.contains("_0x0:for(;;){continue _0x0;break _0x0;}"),
            "{code}"
        );
    }

    #[test]
    fn keeps_unlabeled_break_unlabeled() {
        let code = transform(
            "label: for (;;) { break; }",
            IdentifierNamesGeneratorKind::Hexadecimal,
            &[],
        );

        assert!(code.contains("_0x0:for(;;){break;}"), "{code}");
    }

    #[test]
    fn uses_mangled_generator_for_label_names() {
        let code = transform(
            "label: for (;;) { break label; }",
            IdentifierNamesGeneratorKind::Mangled,
            &[],
        );

        assert!(code.contains("a:for(;;){break a;}"), "{code}");
    }

    #[test]
    fn uses_dictionary_generator_for_label_names() {
        let dictionary = vec!["nice-label".to_string()];
        let code = transform(
            "label: for (;;) { break label; }",
            IdentifierNamesGeneratorKind::Dictionary,
            &dictionary,
        );

        assert!(
            code.contains("nice_label:for(;;){break nice_label;}"),
            "{code}"
        );
    }
}
