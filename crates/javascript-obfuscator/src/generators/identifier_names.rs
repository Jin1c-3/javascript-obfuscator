use serde::{Deserialize, Serialize};

const MANGLED_FIRST_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_";
const MANGLED_NEXT_CHARS: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_0123456789";
const SHUFFLED_FIRST_CHARS: &[u8] = b"_$ZYXWVUTSRQPONMLKJIHGFEDCBAzyxwvutsrqponmlkjihgfedcba";
const RESERVED_WORDS: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "let",
    "static",
    "enum",
    "await",
    "implements",
    "package",
    "protected",
    "interface",
    "private",
    "public",
];

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdentifierNamesGeneratorKind {
    #[default]
    Hexadecimal,
    Mangled,
    MangledShuffled,
    Dictionary,
}

#[derive(Clone, Debug)]
pub struct IdentifierNamesGenerator {
    kind: IdentifierNamesGeneratorKind,
    prefix: String,
    dictionary: Vec<String>,
    index: usize,
}

impl IdentifierNamesGenerator {
    pub fn new(
        kind: IdentifierNamesGeneratorKind,
        prefix: impl Into<String>,
        dictionary: Vec<String>,
    ) -> Self {
        Self {
            kind,
            prefix: prefix.into(),
            dictionary,
            index: 0,
        }
    }

    pub fn generate_next(&mut self) -> String {
        loop {
            let raw_name = match self.kind {
                IdentifierNamesGeneratorKind::Hexadecimal => format!("_0x{:x}", self.index),
                IdentifierNamesGeneratorKind::Mangled => {
                    encode_name(self.index, MANGLED_FIRST_CHARS, MANGLED_NEXT_CHARS)
                }
                IdentifierNamesGeneratorKind::MangledShuffled => {
                    encode_name(self.index, SHUFFLED_FIRST_CHARS, MANGLED_NEXT_CHARS)
                }
                IdentifierNamesGeneratorKind::Dictionary => self.generate_dictionary_name(),
            };
            self.index += 1;

            let candidate = format!("{}{}", self.prefix, raw_name);

            if is_valid_identifier(&candidate) && !is_reserved_word(&candidate) {
                return candidate;
            }
        }
    }

    fn generate_dictionary_name(&self) -> String {
        if self.dictionary.is_empty() {
            return encode_name(self.index, MANGLED_FIRST_CHARS, MANGLED_NEXT_CHARS);
        }

        let dictionary_index = self.index % self.dictionary.len();
        let cycle = self.index / self.dictionary.len();
        let mut name = sanitize_identifier_fragment(&self.dictionary[dictionary_index]);

        if cycle > 0 {
            name.push_str(&cycle.to_string());
        }

        name
    }
}

fn encode_name(index: usize, first_chars: &[u8], next_chars: &[u8]) -> String {
    let mut value = index;
    let mut name = String::new();
    name.push(first_chars[value % first_chars.len()] as char);
    value /= first_chars.len();

    while value > 0 {
        value -= 1;
        name.push(next_chars[value % next_chars.len()] as char);
        value /= next_chars.len();
    }

    name
}

fn sanitize_identifier_fragment(value: &str) -> String {
    let mut output = String::new();

    for character in value.chars() {
        if is_identifier_part(character) {
            output.push(character);
        } else {
            output.push('_');
        }
    }

    if output.is_empty() {
        return "_".to_string();
    }

    if output
        .chars()
        .next()
        .map(is_identifier_start)
        .unwrap_or(false)
    {
        output
    } else {
        format!("_{output}")
    }
}

fn is_valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    is_identifier_start(first) && chars.all(is_identifier_part)
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character == '$' || character.is_ascii_alphabetic()
}

fn is_identifier_part(character: char) -> bool {
    is_identifier_start(character) || character.is_ascii_digit()
}

fn is_reserved_word(value: &str) -> bool {
    RESERVED_WORDS.contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexadecimal_generator_uses_prefix_and_counter() {
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::Hexadecimal,
            "file_",
            Vec::new(),
        );

        assert_eq!(generator.generate_next(), "file__0x0");
        assert_eq!(generator.generate_next(), "file__0x1");
        assert_eq!(generator.generate_next(), "file__0x2");
    }

    #[test]
    fn mangled_generator_uses_short_identifier_sequence() {
        let mut generator =
            IdentifierNamesGenerator::new(IdentifierNamesGeneratorKind::Mangled, "", Vec::new());

        assert_eq!(generator.generate_next(), "a");
        assert_eq!(generator.generate_next(), "b");
        assert_eq!(generator.generate_next(), "c");
    }

    #[test]
    fn mangled_shuffled_generator_is_deterministic() {
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::MangledShuffled,
            "",
            Vec::new(),
        );

        assert_eq!(generator.generate_next(), "_");
        assert_eq!(generator.generate_next(), "$");
        assert_eq!(generator.generate_next(), "Z");
    }

    #[test]
    fn dictionary_generator_sanitizes_invalid_names() {
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::Dictionary,
            "p_",
            vec!["first-name".to_string(), "2cool".to_string()],
        );

        assert_eq!(generator.generate_next(), "p_first_name");
        assert_eq!(generator.generate_next(), "p__2cool");
    }
}
