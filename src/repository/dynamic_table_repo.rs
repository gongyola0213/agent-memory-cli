use crate::domain::schema::{SchemaClass, SchemaDef};
use rusqlite::Connection;

fn sanitize_ident(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

pub fn table_name_for(def: &SchemaDef) -> String {
    let schema = sanitize_ident(&def.schema_id);
    let version = sanitize_ident(&def.version);
    format!("dyn_{}_v{}", schema, version)
}

fn map_sql_type(t: &str) -> &'static str {
    match t.to_ascii_lowercase().as_str() {
        "int" | "integer" | "long" => "INTEGER",
        "float" | "double" | "number" | "real" => "REAL",
        "bool" | "boolean" => "INTEGER",
        _ => "TEXT",
    }
}

pub fn create_table_for_schema(conn: &Connection, def: &SchemaDef) -> Result<String, String> {
    let table = table_name_for(def);

    let mut cols = vec![
        "record_id TEXT PRIMARY KEY".to_string(),
        "created_at TEXT NOT NULL".to_string(),
        "updated_at TEXT NOT NULL".to_string(),
    ];

    match def.class {
        SchemaClass::Domain => {
            cols.push("entity_key TEXT NOT NULL".to_string());
        }
        SchemaClass::UserContext => {
            cols.push("ref_user_id TEXT NOT NULL".to_string());
            cols.push("ref_scope_id TEXT".to_string());
            cols.push("entity_key TEXT".to_string());
        }
    }

    for f in &def.fields {
        let col = sanitize_ident(&f.name);
        let sql_ty = map_sql_type(&f.field_type);
        if cols.iter().any(|c| c.starts_with(&format!("{col} "))) {
            continue;
        }
        let nullable = if f.nullable { "" } else { " NOT NULL" };
        cols.push(format!("{col} {sql_ty}{nullable}"));
    }

    let ddl = format!("CREATE TABLE IF NOT EXISTS {table} ({})", cols.join(", "));
    conn.execute_batch(&ddl)
        .map_err(|e| format!("failed to create dynamic table {table}: {e}"))?;

    conn.execute_batch(&format!(
        "CREATE INDEX IF NOT EXISTS idx_{table}_updated_at ON {table}(updated_at);"
    ))
    .map_err(|e| format!("failed to create index for {table}: {e}"))?;

    if matches!(def.class, SchemaClass::UserContext) {
        conn.execute_batch(&format!(
            "CREATE INDEX IF NOT EXISTS idx_{table}_ref_user_id ON {table}(ref_user_id);"
        ))
        .map_err(|e| format!("failed to create user index for {table}: {e}"))?;
    }

    Ok(table)
}
