use crate::codegen::generate_code;
use crate::diagnostics::ObfuscatorResult;
use crate::options::{ObfuscationResult, Options};
use crate::parser::parse_program;
use crate::storages::normalize_identifier_names_cache;

pub fn run_pipeline(source_code: &str, options: Options) -> ObfuscatorResult<ObfuscationResult> {
    let (hashbang, prepared_code) = extract_hashbang(source_code);
    let parsed_program = parse_program(&prepared_code)?;
    let mut code = generate_code(
        &parsed_program.program,
        parsed_program.source_map,
        options.compact.unwrap_or(true),
    )?;

    if let Some(hashbang) = hashbang {
        code = format!("{hashbang}{code}");
    }

    let source_map = if options.source_map.unwrap_or(false) {
        build_source_map_metadata(source_code, &options)
    } else {
        String::new()
    };

    let identifier_names_cache = normalize_identifier_names_cache(options.identifier_names_cache);

    Ok(ObfuscationResult::new(
        code,
        source_map,
        identifier_names_cache,
    ))
}

fn extract_hashbang(source_code: &str) -> (Option<String>, String) {
    if !source_code.starts_with("#!") {
        return (None, source_code.to_string());
    }

    let line_end = source_code
        .find('\n')
        .map(|index| index + 1)
        .unwrap_or(source_code.len());
    let hashbang = source_code[..line_end].to_string();
    let prepared_code = source_code[line_end..].trim().to_string();

    (Some(hashbang), prepared_code)
}

fn build_source_map_metadata(source_code: &str, options: &Options) -> String {
    let source_name = options
        .input_file_name
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or("sourceMap");

    serde_json::json!({
        "version": 3,
        "file": source_name,
        "sources": [source_name],
        "sourcesContent": [source_code],
        "names": [],
        "mappings": ""
    })
    .to_string()
}
