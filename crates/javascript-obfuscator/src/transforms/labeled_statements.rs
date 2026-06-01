use regex::Regex;
use swc_ecma_ast::{BreakStmt, ContinueStmt, LabeledStmt, Program};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::generators::{IdentifierNamesGenerator, IdentifierNamesGeneratorKind};

pub fn transform_labeled_statements(
    program: &mut Program,
    identifier_names_generator: IdentifierNamesGeneratorKind,
    identifiers_prefix: &str,
    identifiers_dictionary: &[String],
    reserved_names: &[String],
) {
    if identifier_names_generator == IdentifierNamesGeneratorKind::KeepOriginal {
        return;
    }

    program.visit_mut_with(&mut LabeledStatementTransform {
        generator: IdentifierNamesGenerator::new(
            identifier_names_generator,
            identifiers_prefix,
            identifiers_dictionary.to_vec(),
        ),
        reserved_name_patterns: compile_patterns(reserved_names),
    });
}

struct LabeledStatementTransform {
    generator: IdentifierNamesGenerator,
    reserved_name_patterns: Vec<Regex>,
}

impl VisitMut for LabeledStatementTransform {
    fn visit_mut_labeled_stmt(&mut self, labeled_statement: &mut LabeledStmt) {
        let original_label_name = labeled_statement.label.sym.to_string();
        if is_reserved_name(&original_label_name, &self.reserved_name_patterns) {
            return;
        }

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

fn is_reserved_name(name: &str, reserved_name_patterns: &[Regex]) -> bool {
    reserved_name_patterns
        .iter()
        .any(|reserved_name_pattern| reserved_name_pattern.is_match(name))
}

fn compile_patterns(patterns: &[String]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect()
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
        reserved_names: &[String],
    ) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_labeled_statements(
            &mut parsed_program.program,
            identifier_names_generator,
            "",
            identifiers_dictionary,
            reserved_names,
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
            &[],
        );

        assert!(
            code.contains("nice_label:for(;;){break nice_label;}"),
            "{code}"
        );
    }

    #[test]
    fn keeps_original_label_names_with_keep_original_generator() {
        let code = transform(
            "label: for (;;) { continue label; break label; }",
            IdentifierNamesGeneratorKind::KeepOriginal,
            &[],
            &[],
        );

        assert!(
            code.contains("label:for(;;){continue label;break label;}"),
            "{code}"
        );
    }

    #[test]
    fn keeps_reserved_label_names() {
        let reserved_names = vec!["label".to_string()];
        let code = transform(
            "label: for (;;) { continue label; break label; }",
            IdentifierNamesGeneratorKind::Hexadecimal,
            &[],
            &reserved_names,
        );

        assert!(
            code.contains("label:for(;;){continue label;break label;}"),
            "{code}"
        );
    }

    #[test]
    fn keeps_reserved_names_regex_label_names() {
        let reserved_names = vec!["^keep".to_string()];
        let code = transform(
            "keepLabel: for (;;) { continue keepLabel; } other: for (;;) { break other; }",
            IdentifierNamesGeneratorKind::Hexadecimal,
            &[],
            &reserved_names,
        );

        assert!(
            code.contains("keepLabel:for(;;){continue keepLabel;}"),
            "{code}"
        );
        assert!(code.contains("_0x0:for(;;){break _0x0;}"), "{code}");
    }
}
