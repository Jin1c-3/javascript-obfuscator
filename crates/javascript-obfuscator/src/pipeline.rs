use crate::codegen::generate_code;
use crate::diagnostics::ObfuscatorResult;
use crate::options::{validate_regex_options, ObfuscationResult, Options};
use crate::parser::parse_program;
use crate::storages::normalize_identifier_names_cache;
use crate::transforms::apply_transforms;

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn run_pipeline(source_code: &str, options: Options) -> ObfuscatorResult<ObfuscationResult> {
    let (hashbang, prepared_code) = extract_hashbang(source_code);
    let mut parsed_program = parse_program(&prepared_code)?;
    validate_regex_options(&options)?;
    apply_transforms(&mut parsed_program.program, &options);
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

    if !source_map.is_empty() {
        code = append_source_mapping_url(code, &source_map, &options);
    }

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

    let mut source_map = serde_json::Map::new();
    source_map.insert("version".to_string(), serde_json::json!(3));
    source_map.insert("file".to_string(), serde_json::json!(source_name));
    source_map.insert("sources".to_string(), serde_json::json!([source_name]));

    if options.source_map_sources_mode.as_deref() != Some("sources") {
        source_map.insert(
            "sourcesContent".to_string(),
            serde_json::json!([source_code]),
        );
    }

    source_map.insert("names".to_string(), serde_json::json!([]));
    source_map.insert("mappings".to_string(), serde_json::json!(""));

    serde_json::Value::Object(source_map).to_string()
}

fn append_source_mapping_url(mut code: String, source_map: &str, options: &Options) -> String {
    let Some(source_mapping_url) = source_mapping_url(source_map, options) else {
        return code;
    };

    code.push('\n');
    code.push_str("//# sourceMappingURL=");
    code.push_str(&source_mapping_url);
    code
}

fn source_mapping_url(source_map: &str, options: &Options) -> Option<String> {
    match options.source_map_mode.as_deref() {
        Some("inline") => Some(format!(
            "data:application/json;base64,{}",
            encode_base64(source_map.as_bytes())
        )),
        _ => {
            let url = format!(
                "{}{}",
                options.source_map_base_url.as_deref().unwrap_or(""),
                options.source_map_file_name.as_deref().unwrap_or("")
            );

            if url.is_empty() {
                None
            } else {
                Some(url)
            }
        }
    }
}

fn encode_base64(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);

        encoded.push(BASE64_ALPHABET[(first >> 2) as usize] as char);
        encoded
            .push(BASE64_ALPHABET[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);

        if chunk.len() > 1 {
            encoded.push(
                BASE64_ALPHABET[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char,
            );
        } else {
            encoded.push('=');
        }

        if chunk.len() > 2 {
            encoded.push(BASE64_ALPHABET[(third & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }

    encoded
}
