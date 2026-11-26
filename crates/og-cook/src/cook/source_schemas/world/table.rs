//! Context-bound table rows. Decoding collects authored fields, not normalized
//! packed rows: canonical record defaults must be applied by the later cooker.
use super::{super::reflected::schema_for_layout, records};
use og_core::Schema;
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use serde_json::{Map, Value, json};

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct TableInput {
    /// The canonical record name, fixed by schema_with_record.
    #[schemars(schema_with = "records::unbound", extend("x-og-record-role" = "record-name"))]
    pub record: String,
    /// The base language of localized fields; omitted means en.
    #[schemars(length(min = 1))]
    pub locale: Option<String>,
    /// Authored order is retained; ids are stable identities, never indices.
    pub rows: Vec<RowInput>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct RowInput {
    #[schemars(length(min = 1))]
    pub id: String,
    /// All writable columns may be omitted, including str/list and columns
    /// without default lanes. Do not insert zero values here: packed layouts
    /// may not carry every canonical author default. Asset/row/localization
    /// semantics also need author metadata, not guesses from numeric columns.
    #[schemars(schema_with = "records::unbound", extend("x-og-record-role" = "row"))]
    pub values: Map<String, Value>,
}

impl TableInput {
    pub fn schema_with_record(row: &Schema) -> Result<InputSchema<Self>, String> {
        row.validate().map_err(|e| e.to_string())?;
        records::bind(&[
            ("record-name", json!({"type":"string", "const":row.name})),
            ("row", schema_for_layout(row)?),
        ])
    }

    /// Document identity validation, separate from per-field structural checks.
    /// Cross-shard duplicates/references still require the project graph.
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = std::collections::BTreeSet::new();
        for row in &self.rows {
            if !ids.insert(&row.id) {
                return Err(format!("duplicate row id {}", row.id));
            }
        }
        Ok(())
    }
}
