//! Scene/prefab shells, bound to the complete component/resource roles supplied
//! by the project. Packing, prefab deltas/orphans, graph cycles, duplicate entity
//! identities and source-dependent defaults remain their domain responsibilities.
use super::records;
use crate::cook::source_schemas::{ContentRef, media::timeline::Path};
use og_core::Schema;
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct SceneInput {
    pub name: Option<String>,
    #[serde(default)]
    pub entities: Vec<EntityInput>,
    /// Resource patches, never complete replacement values.
    #[serde(default)]
    #[schemars(schema_with = "records::unbound", extend("x-og-record-role" = "resource"))]
    pub resources: BTreeMap<String, Map<String, Value>>,
    #[serde(default)]
    #[schemars(extend("propertyNames" = {"pattern":"^[A-Za-z0-9_-]{1,64}$"}))]
    pub streams: BTreeMap<String, StreamInput>,
    pub hlod: Option<HlodInput>,
    /// Named local-space paths use the same typed path as timeline rails.
    #[serde(default)]
    pub paths: BTreeMap<String, Path>,
}

impl SceneInput {
    /// The caller supplies both full role sets (engine + game). Packed layouts
    /// lack readonly/asset/row/localization column metadata; this does not infer
    /// any of those semantics from numeric types or replace project graph checks.
    pub fn schema_with_records(
        components: &[Schema],
        resources: &[Schema],
    ) -> Result<InputSchema<Self>, String> {
        records::bind(&[
            ("component", records::patches(components)?),
            (
                "component-name",
                json!({"type":"string", "enum":components.iter().map(|s| &s.name).collect::<Vec<_>>()}),
            ),
            ("resource", records::patches(resources)?),
        ])
    }
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct PrefabInput {
    #[schemars(extend("x-og-ref-kinds" = ["prefab"]))]
    pub base: Option<ContentRef>,
    /// A prefab has exactly one root, including a variant of a base prefab.
    pub entity: EntityInput,
}

impl PrefabInput {
    /// Prefabs have component patches, not top-level resource declarations.
    pub fn schema_with_records(components: &[Schema]) -> Result<InputSchema<Self>, String> {
        records::bind(&[
            ("component", records::patches(components)?),
            (
                "component-name",
                json!({"type":"string", "enum":components.iter().map(|s| &s.name).collect::<Vec<_>>()}),
            ),
        ])
    }
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct EntityInput {
    pub name: String,
    #[schemars(extend("x-og-ref-kinds" = ["model"]))]
    pub model: Option<ContentRef>,
    #[schemars(extend("x-og-ref-kinds" = ["scene"]))]
    pub scene: Option<ContentRef>,
    #[schemars(extend("x-og-ref-kinds" = ["prefab"]))]
    pub prefab: Option<ContentRef>,
    #[serde(rename = "static")]
    pub static_model: Option<bool>,
    #[serde(default)]
    #[schemars(schema_with = "records::unbound", extend("x-og-record-role" = "component"))]
    pub components: BTreeMap<String, Map<String, Value>>,
    /// Applied in order to an instance's existing tree. An override with path
    /// "" can patch the root again after an earlier edit. Added children follow
    /// the inherited tree, as in the existing prefab flattener.
    #[serde(default)]
    pub edits: Vec<PrefabEdit>,
    #[serde(default)]
    pub children: Vec<EntityInput>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum PrefabEdit {
    Override {
        /// Relative descendant path, or "" for the instance root.
        path: String,
        #[schemars(extend("x-og-ref-kinds" = ["model"]))]
        model: Option<ContentRef>,
        #[serde(rename = "static")]
        static_model: Option<bool>,
        #[serde(default)]
        #[schemars(schema_with = "records::unbound", extend("x-og-record-role" = "component"))]
        components: BTreeMap<String, Map<String, Value>>,
    },
    Remove {
        path: String,
        /// Omitted removes the subtree; present removes that component only.
        #[schemars(schema_with = "records::unbound", extend("x-og-record-role" = "component-name"))]
        component: Option<String>,
    },
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct HlodInput {
    /// Leaf cell side in metres; omitted: 64.
    #[schemars(range(min = 4, max = 4096), extend("x-og-unit" = "m"))]
    pub size: Option<f32>,
    /// Maximum proxy displacement in metres. Omitted/zero: size / 256;
    /// a supplied positive value must be at most size / 4 (domain check).
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    pub error: Option<f32>,
    /// Longest proxy texture side in pixels; omitted: 1024.
    #[schemars(range(min = 64, max = 4096), extend("x-og-unit" = "px"))]
    pub texels: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct StreamInput {
    /// A cell group may hold only its own subscene; other groups need assets.
    #[serde(default)]
    pub assets: Vec<ContentRef>,
    /// Omitted means manual loading only.
    pub trigger: Option<StreamTriggerInput>,
    /// Higher loads first and unloads last; omitted: 0.
    pub priority: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum StreamTriggerInput {
    Manual,
    Cell,
    Volume {
        entity: String,
    },
    Distance {
        #[schemars(extend("x-og-unit" = "m"))]
        anchor: [f32; 3],
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
        radius: f32,
        /// Omitted: radius * 1.25; cannot be less than radius (domain check).
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        unload: Option<f32>,
    },
}
