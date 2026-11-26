//! Binding the fixed world/data shells to caller-owned record layouts. Nothing
//! is cached globally: two projects may use the same name for different fields.
use super::super::reflected::schema_for_layout;
use og_core::Schema;
use og_core::reflect::source::{InputSchema, schemars, schemars::JsonSchema};
use schemars::transform::{RecursiveTransform, Transform};
use serde_json::{Map, Value, json};

/// A dynamic slot is impossible until a context explicitly binds its role.
/// This is not an arbitrary-object fallback for unregistered game records.
pub(super) fn unbound(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    false.into()
}

pub(super) fn patches(layouts: &[Schema]) -> Result<Value, String> {
    let mut properties = Map::new();
    for layout in layouts {
        layout.validate().map_err(|e| e.to_string())?;
        if properties
            .insert(layout.name.clone(), schema_for_layout(layout)?)
            .is_some()
        {
            return Err(format!("duplicate record {} in the same role", layout.name));
        }
    }
    Ok(json!({"type":"object", "properties":properties, "additionalProperties":false}))
}

pub(super) fn bind<T: JsonSchema>(roles: &[(&str, Value)]) -> Result<InputSchema<T>, String> {
    let mut document = schemars::generate::SchemaSettings::draft2020_12()
        .for_deserialize()
        .into_generator()
        .into_root_schema_for::<T>()
        .to_value()
        .try_into()
        .expect("derived world input schema");
    RecursiveTransform(|schema: &mut schemars::Schema| {
        if let Some(role) = schema.get("x-og-record-role").and_then(Value::as_str) {
            let value = &roles
                .iter()
                .find(|(name, _)| *name == role)
                .expect("every record role is bound by the domain entry point")
                .1;
            *schema = value.clone().try_into().expect("record object schema");
        }
    })
    .transform(&mut document);
    InputSchema::from_document(document.to_value())
}
