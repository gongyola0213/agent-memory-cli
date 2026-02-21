use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaClass {
    Domain,
    UserContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDef {
    pub schema_id: String,
    pub version: String,
    pub class: SchemaClass,
    pub fields: Vec<FieldDef>,
}

pub fn validate_schema_def(def: &SchemaDef) -> Result<(), String> {
    if def.schema_id.trim().is_empty() {
        return Err("schema validation failed: schema_id is required".to_string());
    }
    if def.version.trim().is_empty() {
        return Err("schema validation failed: version is required".to_string());
    }
    if def.fields.is_empty() {
        return Err("schema validation failed: fields[] is required".to_string());
    }

    let mut seen = HashSet::new();
    for f in &def.fields {
        if f.name.trim().is_empty() {
            return Err("schema validation failed: each field requires non-empty name".to_string());
        }
        if !seen.insert(f.name.clone()) {
            return Err(format!(
                "schema validation failed: duplicate field name='{}' in schema_id={}",
                f.name, def.schema_id
            ));
        }
    }

    if def.class == SchemaClass::UserContext && !seen.contains("refUserId") {
        return Err(format!(
            "schema validation failed: user_context schema_id={} must include field name=refUserId",
            def.schema_id
        ));
    }

    Ok(())
}
