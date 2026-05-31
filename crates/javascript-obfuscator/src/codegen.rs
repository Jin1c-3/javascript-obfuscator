use swc_common::{sync::Lrc, SourceMap};
use swc_ecma_ast::Program;
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};

use crate::diagnostics::{ObfuscatorError, ObfuscatorResult};

pub fn generate_code(
    program: &Program,
    source_map: Lrc<SourceMap>,
    compact: bool,
) -> ObfuscatorResult<String> {
    let mut output = Vec::new();
    {
        let writer = Box::new(JsWriter::new(source_map.clone(), "\n", &mut output, None));
        let mut emitter = Emitter {
            cfg: Config::default().with_minify(false),
            comments: None,
            cm: source_map,
            wr: writer,
        };

        emitter
            .emit_program(program)
            .map_err(|error| ObfuscatorError::Codegen(error.to_string()))?;
    }

    let code =
        String::from_utf8(output).map_err(|error| ObfuscatorError::Codegen(error.to_string()))?;

    if compact {
        Ok(compact_code(&code))
    } else {
        Ok(code)
    }
}

fn compact_code(code: &str) -> String {
    let chars: Vec<char> = code.chars().collect();
    let mut output = String::with_capacity(code.len());
    let mut index = 0;

    while index < chars.len() {
        let character = chars[index];

        match character {
            '\'' | '"' | '`' => {
                copy_quoted(&chars, &mut index, &mut output, character);
            }
            '/' if chars.get(index + 1) == Some(&'/') => {
                index = skip_line_comment(&chars, index + 2);
            }
            '/' if chars.get(index + 1) == Some(&'*') => {
                index = skip_block_comment(&chars, index + 2);
            }
            character if character.is_whitespace() => {
                if should_preserve_space(&output, next_non_whitespace(&chars, index + 1)) {
                    output.push(' ');
                }
                index += 1;
            }
            _ => {
                output.push(character);
                index += 1;
            }
        }
    }

    output.trim().to_string()
}

fn copy_quoted(chars: &[char], index: &mut usize, output: &mut String, quote: char) {
    let mut escaped = false;

    while *index < chars.len() {
        let character = chars[*index];
        output.push(character);
        *index += 1;

        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == quote {
            break;
        }
    }
}

fn skip_line_comment(chars: &[char], mut index: usize) -> usize {
    while index < chars.len() && chars[index] != '\n' {
        index += 1;
    }

    index
}

fn skip_block_comment(chars: &[char], mut index: usize) -> usize {
    while index + 1 < chars.len() {
        if chars[index] == '*' && chars[index + 1] == '/' {
            return index + 2;
        }

        index += 1;
    }

    chars.len()
}

fn should_preserve_space(output: &str, next: Option<char>) -> bool {
    let Some(previous) = output.chars().next_back() else {
        return false;
    };
    let Some(next) = next else {
        return false;
    };

    (is_identifier_part(previous) && is_identifier_part(next))
        || (previous == '+' && next == '+')
        || (previous == '-' && next == '-')
}

fn next_non_whitespace(chars: &[char], mut index: usize) -> Option<char> {
    while index < chars.len() {
        if !chars[index].is_whitespace() {
            return Some(chars[index]);
        }

        index += 1;
    }

    None
}

fn is_identifier_part(character: char) -> bool {
    character == '_' || character == '$' || character == '\\' || character.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_program;

    use super::*;

    fn generate(source_code: &str, compact: bool) -> String {
        let parsed_program = parse_program(source_code).expect("source should parse");

        generate_code(&parsed_program.program, parsed_program.source_map, compact)
            .expect("code should generate")
    }

    #[test]
    fn compacts_output_without_rewriting_string_literal_spaces() {
        let code = generate("const value = 'hello world'; console.log(value);", true);

        assert!(code.contains("const value='hello world';console.log(value);"));
    }

    #[test]
    fn preserves_readable_output_when_compact_is_disabled() {
        let code = generate("const value = 1;", false);

        assert!(code.contains("const value = 1;"));
    }
}
