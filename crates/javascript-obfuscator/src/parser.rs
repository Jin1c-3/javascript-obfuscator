use swc_common::{sync::Lrc, FileName, SourceFile, SourceMap};
use swc_ecma_ast::Program;
use swc_ecma_parser::{lexer::Lexer, EsSyntax, Parser, StringInput, Syntax};

use crate::diagnostics::{ObfuscatorError, ObfuscatorResult};

pub struct ParsedProgram {
    pub program: Program,
    pub source_map: Lrc<SourceMap>,
}

pub fn parse_program(source_code: &str) -> ObfuscatorResult<ParsedProgram> {
    let source_map: Lrc<SourceMap> = Default::default();
    let source_file = source_map.new_source_file(
        FileName::Custom("source.js".to_string()).into(),
        source_code.to_string(),
    );

    parse_with_module_mode(source_map.clone(), &source_file, true)
        .or_else(|_| parse_with_module_mode(source_map.clone(), &source_file, false))
}

fn parse_with_module_mode(
    source_map: Lrc<SourceMap>,
    source_file: &SourceFile,
    module: bool,
) -> ObfuscatorResult<ParsedProgram> {
    let lexer = Lexer::new(
        Syntax::Es(EsSyntax {
            jsx: true,
            export_default_from: true,
            import_attributes: true,
            allow_super_outside_method: true,
            allow_return_outside_function: true,
            ..Default::default()
        }),
        swc_ecma_ast::EsVersion::Es2022,
        StringInput::from(source_file),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    let program = if module {
        parser.parse_module().map(Program::Module)
    } else {
        parser.parse_script().map(Program::Script)
    }
    .map_err(|error| ObfuscatorError::Parse(error.kind().msg().into_owned()))?;

    Ok(ParsedProgram {
        program,
        source_map,
    })
}
