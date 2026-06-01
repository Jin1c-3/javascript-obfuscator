use std::error::Error;
use std::fmt::{Display, Formatter};

pub type ObfuscatorResult<T> = Result<T, ObfuscatorError>;

#[derive(Debug)]
pub enum ObfuscatorError {
    Parse(String),
    Options(String),
    Codegen(String),
}

impl Display for ObfuscatorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(message) => write!(formatter, "JavaScript parse error: {message}"),
            Self::Options(message) => {
                write!(formatter, "JavaScript obfuscator options error: {message}")
            }
            Self::Codegen(message) => {
                write!(formatter, "JavaScript code generation error: {message}")
            }
        }
    }
}

impl Error for ObfuscatorError {}
