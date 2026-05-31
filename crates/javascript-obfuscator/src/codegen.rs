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
            cfg: Config::default().with_minify(compact),
            comments: None,
            cm: source_map,
            wr: writer,
        };

        emitter
            .emit_program(program)
            .map_err(|error| ObfuscatorError::Codegen(error.to_string()))?;
    }

    String::from_utf8(output).map_err(|error| ObfuscatorError::Codegen(error.to_string()))
}
