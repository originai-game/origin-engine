//! Authored model inputs. Declaration order is retained for named materials and
//! parent-first pivots; geometry and Transform are the existing shared schemas.
use super::{
    super::{ContentRef, reflected::Reflected},
    geometry::{Geometry, Mirror, Transform},
    materials::Material,
};
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

// Source descriptors are closed and camelCase throughout this domain. The
// macro adds only those invariants; constraints/docs stay on their Rust fields.
macro_rules! input {
    ($(#[$doc:meta])* $name:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Deserialize, JsonSchema)]
        #[schemars(crate = "og_core::reflect::source::schemars")]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $name { $($(#[$attr])* pub $field: $ty,)* }
    };
}

pub mod plant;
pub mod process;

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(
    tag = "kind",
    content = "desc",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum ModelInput {
    Geometry(Box<GeometryModel>),
    Program(Box<ProgramModel>),
}

input!(GeometryModel {
    /// Ordered declarations: a named dependency must precede its consumer.
    items: Vec<Item>,
    conform: Option<Conform>,
    process: Option<process::Process>,
    fracture: Option<process::Fracture>,
});

input!(ProgramModel {
    #[schemars(extend("x-og-ref-kinds" = ["model-program"]))]
    program: ContentRef,
    #[serde(default)]
    params: BTreeMap<String, f64>,
    #[serde(default)]
    materials: Vec<ProgramMaterial>,
    #[schemars(range(min = 16, max = 8192), extend("x-og-unit" = "px"))]
    atlas: Option<u32>,
    conform: Option<Conform>,
    process: Option<process::Process>,
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Conform {
    Off,
    Character,
    Effect,
}

input!(Item {
    name: String,
    input: ItemInput,
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(
    tag = "kind",
    content = "desc",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum ItemInput {
    Material(Box<Material>),
    Pivot(Pivot),
    Part(Box<Part>),
    Plant(Box<plant::Plant>),
}

input!(Pivot {
    /// Previously declared pivot. Positive scale and nonsingular rotation are
    /// model-domain constraints; reflected Transform is not duplicated here.
    parent: Option<String>,
    transform: Option<Reflected<Transform>>,
});

input!(Part {
    material: Option<String>,
    pivot: Option<String>,
    geometry: Geometry,
    /// Omitted: identity placement. Explicit placements must be nonempty; a
    /// single placement also represents the old part's own TRS, with no XOR.
    #[schemars(length(min = 1))]
    placements: Option<Vec<Placement>>,
});

input!(Placement {
    transform: Option<Reflected<Transform>>,
    /// Reflections bake into geometry with corrected triangle winding.
    mirror: Option<Mirror>,
});

input!(
/// Program material overrides have their own source semantics (sheen and
/// roughness/emissive maps); they are not the procedural material's field set.
ProgramMaterial {
    name: String,
    color: Option<[f64; 3]>,
    #[schemars(range(min = 0, max = 1))]
    metalness: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    roughness: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    opacity: Option<f64>,
    emissive: Option<[f64; 3]>,
    #[schemars(range(min = 0))]
    emissive_intensity: Option<f64>,
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    map: Option<String>,
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    normal_map: Option<String>,
    #[schemars(range(min = 0))]
    normal_scale: Option<f64>,
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    roughness_map: Option<String>,
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    emissive_map: Option<String>,
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    ao_map: Option<String>,
    #[schemars(range(min = 0, max = 1))]
    ao_map_intensity: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    clearcoat: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    clearcoat_roughness: Option<f64>,
    sheen: Option<[f64; 3]>,
    #[schemars(range(min = 0, max = 1))]
    sheen_roughness: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    transmission: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    iridescence: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    alpha_test: Option<f64>,
    transparent: Option<bool>,
    double_sided: Option<bool>,
});

impl ModelInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<ModelInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("model input schema"))
    }
}
