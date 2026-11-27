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
            ("row", row_schema(schema_for_layout(row)?)),
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

/// Table-only author forms. Preserve the underlying record schema everywhere
/// else: a scene's str is not a localized key, and TableRow is not an asset.
pub(crate) fn row_schema(mut schema: Value) -> Value {
    fn column(schema: &mut Value) {
        if schema.get("type").and_then(Value::as_str) == Some("array") {
            column(&mut schema["items"]);
        } else if schema["x-og-handle-column"] == "TableRow" {
            let mut table =
                og_core::reflect::source::schemars::schema_for!(super::super::ContentRef)
                    .to_value();
            table["x-og-ref-kinds"] = json!(["table"]);
            *schema = json!({"type":"object","properties":{
                "row":{"type":"string","minLength":1},"table":table},
                "required":["row"],"additionalProperties":false});
        } else if schema["type"] == "string" && schema.get("x-og-capacity-bytes").is_some() {
            let reference =
                og_core::reflect::source::schemars::schema_for!(super::super::ContentRef)
                    .to_value();
            *schema = json!({"anyOf":[schema.clone(),
                {"type":"object","properties":{"asset":reference},"required":["asset"],"additionalProperties":false},
                {"type":"object","properties":{"text":{"type":"object","properties":{
                    "key":{"type":"string","minLength":1},"base":{"type":"string"}},
                    "required":["key","base"],"additionalProperties":false}},"required":["text"],"additionalProperties":false}
            ]});
        }
    }
    for field in schema["properties"]
        .as_object_mut()
        .expect("record properties")
        .values_mut()
    {
        column(field);
    }
    schema
}
