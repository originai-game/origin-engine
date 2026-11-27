//! glTF-source materials, shared by authored models, fracture interiors and
//! material libraries. These are not runtime StandardMaterial handles/records:
//! source images, doubleSided and extension fields have different semantics.
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::sync::OnceLock;

/// Omitted fields remain absent until the material domain normalizes them.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Material {
    /// Linear base RGB (glTF factor); not a runtime colour handle.
    pub color: Option<[f64; 3]>,
    #[schemars(range(min = 0, max = 1))]
    pub opacity: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    pub metalness: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    pub roughness: Option<f64>,
    /// Linear emitted RGB, multiplied by emissiveIntensity.
    pub emissive: Option<[f64; 3]>,
    #[schemars(range(min = 0))]
    pub emissive_intensity: Option<f64>,
    pub double_sided: Option<bool>,
    /// Alpha-mask cutoff takes precedence over opacity blending.
    #[schemars(range(min = 0, max = 1))]
    pub alpha_test: Option<f64>,
    /// Source image inputs; not cooked texture references. Resolution and path
    /// containment are checked at the input boundary.
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    pub map: Option<String>,
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    pub normal_map: Option<String>,
    pub normal_scale: Option<f64>,
    /// Roughness in green, metalness in blue.
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    pub metalness_map: Option<String>,
    /// Ambient occlusion in red.
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    pub ao_map: Option<String>,
    #[schemars(range(min = 0, max = 1))]
    pub transmission: Option<f64>,
    #[schemars(range(min = 1, max = 3))]
    pub ior: Option<f64>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    pub thickness: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    pub clearcoat: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    pub clearcoat_roughness: Option<f64>,
    /// Diffuse transmission through thin leaves; enables both faces.
    #[schemars(range(min = 0, max = 1))]
    pub translucency: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    pub dome_normals: Option<f64>,
    /// Zero evergreen; one deciduous.
    #[schemars(range(min = 0, max = 1))]
    pub season: Option<f64>,
    #[schemars(inner(range(min = 0)))]
    pub autumn_color: Option<[f32; 3]>,
    #[schemars(inner(range(min = 0)))]
    pub autumn_color2: Option<[f32; 3]>,
    #[schemars(range(min = -og_render_data::foliage::LeafSeason::MAX_SHIFT, max = og_render_data::foliage::LeafSeason::MAX_SHIFT), extend("x-og-unit" = "days"))]
    pub season_shift: Option<f32>,
    #[schemars(range(min = 0, max = 1))]
    pub grime: Option<f64>,
    #[schemars(inner(range(min = 0)))]
    pub grime_color: Option<[f64; 3]>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    pub grime_height: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    pub stains: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    pub stochastic: Option<f64>,
}

impl Material {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<Material>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("material input schema"))
    }
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct MaterialsInput {
    /// Order is stable; duplicate names and generation requirements are checked
    /// together by the material-library validator.
    pub materials: Vec<LibraryMaterial>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryMaterial {
    #[schemars(length(min = 1))]
    pub name: String,
    /// Material labels produced by grammar surfaces.
    #[schemars(length(min = 1), inner(length(min = 1)))]
    pub labels: Vec<String>,
    #[schemars(extend("exclusiveMinimum" = 0))]
    pub weight: Option<f64>,
    /// Physical tile dimensions. Both dimensions are strictly positive.
    #[schemars(extend("items" = {"type":"number", "exclusiveMinimum":0}, "x-og-unit" = "m"))]
    pub tile: Option<[f64; 2]>,
    pub vary: Option<Variation>,
    #[schemars(length(min = 1))]
    pub brief: Option<String>,
    /// Explicit image-map preparation input, not a TS executor or runtime handle.
    pub generate: Option<Generate>,
    pub facade: Option<bool>,
    #[schemars(range(min = -1, max = 1), extend("x-og-unit" = "m"))]
    pub relief: Option<f64>,
    #[schemars(inner(range(min = 0, max = 1)))]
    pub roughness_range: Option<[f64; 2]>,
    pub occlusion: Option<Occlusion>,
    /// glTF material fields shared with authored models and fracture surfaces.
    pub surface: Material,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Generate {
    /// Authored image used to prepare the explicitly named map destinations.
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    pub source: String,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Variation {
    #[schemars(range(min = 0, max = 0.5))]
    pub value: Option<f64>,
    #[schemars(range(min = 0, max = 0.5))]
    pub warmth: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Occlusion {
    #[schemars(range(min = 0, max = 1))]
    pub strength: f64,
    #[schemars(range(min = 1, max = 64), extend("x-og-unit" = "px"))]
    pub radius: u32,
}

impl MaterialsInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<MaterialsInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("materials input schema"))
    }
}
