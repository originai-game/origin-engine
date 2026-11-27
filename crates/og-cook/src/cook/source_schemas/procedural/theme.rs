//! Theme recipe shape. Dimension files, inheritance and colour parsing remain
//! with the existing theme resolver/compiler; pack structure is shared with E2.
use super::{ContentRef, InputSchema, JsonSchema, OnceLock};
use crate::cook::source_schemas::world::pack::PackInput;
use serde::Deserialize;

/// A recipe selects dimensions and overlays a complete structured pack patch.
/// Dimension names are data read by the domain, not a schema-maintained list.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars", extend("x-og-output-kind" = "pack"))]
#[serde(deny_unknown_fields)]
pub struct ThemeInput {
    pub about: Option<String>,
    pub extends: Option<ThemeBase>,
    /// Built-in mix-in names, applied in order after the parent recipe.
    pub with: Option<Vec<String>>,
    pub color: Option<ThemeColor>,
    pub material: Option<String>,
    pub shape: Option<String>,
    #[serde(rename = "type")]
    pub typeface: Option<String>,
    pub motion: Option<String>,
    pub ornament: Option<OrnamentInput>,
    pub effects: Option<String>,
    pub sound: Option<String>,
    pub icons: Option<String>,
    pub density: Option<String>,
    pub pack: Option<PackInput>,
}

impl ThemeInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<ThemeInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("theme input schema"))
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum ThemeBase {
    Builtin(#[schemars(regex(pattern = "^@themes/[a-z0-9]+(-[a-z0-9]+)*$"))] String),
    Asset(#[schemars(extend("x-og-ref-factories" = ["theme"]))] ContentRef),
}
input! {
    ThemeColor {
        /// Any independent literal CSS colour; domain parsing rejects variables.
        source: Option<String>,
        /// Any independent literal CSS colour.
        accent: Option<String>,
        /// Any independent literal CSS colour.
        neutral: Option<String>,
        /// Any independent literal CSS colour.
        danger: Option<String>,
        variant: Option<String>,
        scheme: Option<ThemeScheme>,
    }
}
words!(ThemeScheme { Light, Dark });

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum OrnamentInput {
    All(String),
    Slots(OrnamentSlots),
}
input! {
    /// Per-piece names from the ornament dimension, not arbitrary object keys.
    OrnamentSlots {
        button: Option<String>,
        panel: Option<String>,
        plate: Option<String>,
        slot: Option<String>,
        chip: Option<String>,
    }
}
