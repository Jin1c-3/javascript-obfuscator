use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value;

#[napi]
pub fn obfuscate(source_code: String, options: Option<Value>) -> Result<Value> {
    let options = options
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| Error::from_reason(error.to_string()))?
        .unwrap_or_default();

    let result = javascript_obfuscator::obfuscate(&source_code, options)
        .map_err(|error| Error::from_reason(error.to_string()))?;

    serde_json::to_value(result).map_err(|error| Error::from_reason(error.to_string()))
}

#[napi]
pub fn get_options_by_preset(preset: String) -> Result<Value> {
    let preset = match preset.as_str() {
        "default" => javascript_obfuscator::Preset::Default,
        "low-obfuscation" => javascript_obfuscator::Preset::LowObfuscation,
        "medium-obfuscation" => javascript_obfuscator::Preset::MediumObfuscation,
        "high-obfuscation" => javascript_obfuscator::Preset::HighObfuscation,
        value => {
            return Err(Error::from_reason(format!(
                "Unknown options preset: {value}"
            )))
        }
    };

    Ok(javascript_obfuscator::get_options_by_preset(preset))
}
