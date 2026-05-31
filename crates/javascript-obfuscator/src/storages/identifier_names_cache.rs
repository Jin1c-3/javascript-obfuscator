use serde_json::{Map, Value};

use crate::generators::IdentifierNamesGenerator;
use crate::options::IdentifierNamesCache;

const GLOBAL_IDENTIFIERS: &str = "globalIdentifiers";
const PROPERTY_IDENTIFIERS: &str = "propertyIdentifiers";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IdentifierNamesCacheStorage {
    global_identifiers: Map<String, Value>,
    property_identifiers: Map<String, Value>,
}

impl IdentifierNamesCacheStorage {
    pub fn from_cache(cache: IdentifierNamesCache) -> Self {
        Self {
            global_identifiers: read_section(&cache, GLOBAL_IDENTIFIERS),
            property_identifiers: read_section(&cache, PROPERTY_IDENTIFIERS),
        }
    }

    pub fn into_cache(self) -> IdentifierNamesCache {
        let mut cache = Map::new();
        cache.insert(
            GLOBAL_IDENTIFIERS.to_string(),
            Value::Object(self.global_identifiers),
        );
        cache.insert(
            PROPERTY_IDENTIFIERS.to_string(),
            Value::Object(self.property_identifiers),
        );
        cache
    }

    pub fn resolve_or_insert_global(
        &mut self,
        original_name: &str,
        generator: &mut IdentifierNamesGenerator,
    ) -> String {
        if let Some(existing_name) = self
            .global_identifiers
            .get(original_name)
            .and_then(Value::as_str)
        {
            return existing_name.to_string();
        }

        let generated_name = generator.generate_next();
        self.global_identifiers.insert(
            original_name.to_string(),
            Value::String(generated_name.clone()),
        );
        generated_name
    }
}

pub fn normalize_identifier_names_cache(
    cache: Option<IdentifierNamesCache>,
) -> Option<IdentifierNamesCache> {
    cache.map(|cache| IdentifierNamesCacheStorage::from_cache(cache).into_cache())
}

fn read_section(cache: &IdentifierNamesCache, section_name: &str) -> Map<String, Value> {
    cache
        .get(section_name)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::generators::IdentifierNamesGeneratorKind;

    #[test]
    fn normalizes_empty_cache_to_public_shape() {
        let normalized = normalize_identifier_names_cache(Some(Map::new()))
            .expect("cache should remain enabled");

        assert_eq!(normalized.get("globalIdentifiers"), Some(&json!({})));
        assert_eq!(normalized.get("propertyIdentifiers"), Some(&json!({})));
    }

    #[test]
    fn preserves_existing_global_and_property_mappings() {
        let mut input = Map::new();
        input.insert("globalIdentifiers".to_string(), json!({ "alpha": "_0x1" }));
        input.insert("propertyIdentifiers".to_string(), json!({ "beta": "_0x2" }));

        let normalized = normalize_identifier_names_cache(Some(input)).expect("cache enabled");

        assert_eq!(normalized["globalIdentifiers"]["alpha"], "_0x1");
        assert_eq!(normalized["propertyIdentifiers"]["beta"], "_0x2");
    }

    #[test]
    fn resolves_existing_global_mapping_before_generating() {
        let mut input = Map::new();
        input.insert("globalIdentifiers".to_string(), json!({ "alpha": "_0x9" }));
        let mut storage = IdentifierNamesCacheStorage::from_cache(input);
        let mut generator = IdentifierNamesGenerator::new(
            IdentifierNamesGeneratorKind::Hexadecimal,
            "",
            Vec::new(),
        );

        assert_eq!(
            storage.resolve_or_insert_global("alpha", &mut generator),
            "_0x9"
        );
        assert_eq!(
            storage.resolve_or_insert_global("beta", &mut generator),
            "_0x0"
        );

        let output = storage.into_cache();
        assert_eq!(output["globalIdentifiers"]["alpha"], "_0x9");
        assert_eq!(output["globalIdentifiers"]["beta"], "_0x0");
    }
}
