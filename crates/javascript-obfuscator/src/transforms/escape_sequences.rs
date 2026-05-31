use swc_ecma_ast::{CallExpr, Callee, ExportAll, Expr, ImportDecl, NamedExport, Program, Str};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn transform_escape_sequences(
    program: &mut Program,
    unicode_escape_sequence: bool,
    reserved_strings: &[String],
    ignore_imports: bool,
) {
    program.visit_mut_with(&mut EscapeSequenceTransform {
        unicode_escape_sequence,
        reserved_strings,
        ignore_imports,
    });
}

struct EscapeSequenceTransform<'a> {
    unicode_escape_sequence: bool,
    reserved_strings: &'a [String],
    ignore_imports: bool,
}

impl VisitMut for EscapeSequenceTransform<'_> {
    fn visit_mut_import_decl(&mut self, _import_declaration: &mut ImportDecl) {}

    fn visit_mut_named_export(&mut self, _named_export: &mut NamedExport) {}

    fn visit_mut_export_all(&mut self, _export_all: &mut ExportAll) {}

    fn visit_mut_call_expr(&mut self, call_expression: &mut CallExpr) {
        if self.ignore_imports && is_ignored_import_call(call_expression) {
            return;
        }

        call_expression.visit_mut_children_with(self);
    }

    fn visit_mut_str(&mut self, string_literal: &mut Str) {
        string_literal.visit_mut_children_with(self);

        let value = string_literal.value.to_string_lossy();
        if is_reserved_string(&value, self.reserved_strings) {
            return;
        }

        let escaped_value = encode_escape_sequence(&value, self.unicode_escape_sequence);
        string_literal.raw = Some(format!("'{escaped_value}'").into());
    }
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

fn encode_escape_sequence(value: &str, encode_all_symbols: bool) -> String {
    let mut escaped_value = String::new();

    for character in value.chars() {
        if should_encode_character(character, encode_all_symbols) {
            escaped_value.push_str(&encode_character(character));
        } else {
            escaped_value.push(character);
        }
    }

    escaped_value
}

fn should_encode_character(character: char, encode_all_symbols: bool) -> bool {
    encode_all_symbols
        || matches!(character, '\u{0000}'..='\u{001f}' | '\u{007f}'..='\u{009f}' | '\'' | '"' | '\\')
        || character.is_whitespace()
}

fn encode_character(character: char) -> String {
    let mut encoded_character = String::new();
    let mut code_units = [0; 2];

    for code_unit in character.encode_utf16(&mut code_units) {
        if *code_unit <= 0x7f {
            encoded_character.push_str(&format!("\\x{code_unit:02x}"));
        } else {
            encoded_character.push_str(&format!("\\u{code_unit:04x}"));
        }
    }

    encoded_character
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(
        source_code: &str,
        unicode_escape_sequence: bool,
        reserved_strings: &[String],
    ) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");
        transform_escape_sequences(
            &mut parsed_program.program,
            unicode_escape_sequence,
            reserved_strings,
            false,
        );
        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("code should generate")
    }

    #[test]
    fn encodes_all_ascii_characters_when_unicode_escape_sequence_is_enabled() {
        let code = transform("const value = 'test';", true, &[]);

        assert!(
            code.contains("const value='\\x74\\x65\\x73\\x74'"),
            "{code}"
        );
    }

    #[test]
    fn encodes_forced_characters_when_unicode_escape_sequence_is_disabled() {
        let code = transform("const value = 'hello world';", false, &[]);

        assert!(code.contains("const value='hello\\x20world'"), "{code}");
    }

    #[test]
    fn encodes_non_ascii_characters_when_unicode_escape_sequence_is_enabled() {
        let code = transform(
            "const value = '\u{0442}\u{0435}\u{0441}\u{0442}';",
            true,
            &[],
        );

        assert!(
            code.contains("const value='\\u0442\\u0435\\u0441\\u0442'"),
            "{code}"
        );
    }

    #[test]
    fn keeps_plain_string_when_unicode_escape_sequence_is_disabled() {
        let code = transform("const value = 'test';", false, &[]);

        assert!(code.contains("const value='test'"), "{code}");
    }

    #[test]
    fn keeps_reserved_string_when_unicode_escape_sequence_is_enabled() {
        let code = transform(
            "const foo = 'foo'; const bar = 'bar';",
            true,
            &["foo".to_string()],
        );

        assert!(code.contains("const foo='foo';"), "{code}");
        assert!(code.contains("const bar='\\x62\\x61\\x72';"), "{code}");
    }
}
