//! District layout inputs. Polygon validity, least/greatest ordering, nonzero
//! density's grammar requirement and named rule/prop dependencies remain domain
//! checks. No inferred buildings or hidden gameplay composition are added.
use super::{super::ContentRef, scene::HlodInput};
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct DistrictInput {
    pub seed: Option<u32>,
    pub region: Region,
    pub streets: Option<Streets>,
    pub lots: Lots,
    pub buildings: Option<Buildings>,
    #[serde(default)]
    pub props: Vec<Prop>,
    #[schemars(extend("x-og-ref-kinds" = ["materials"]))]
    pub materials: ContentRef,
    pub cells: Option<Cells>,
    pub hlod: Option<HlodInput>,
    /// Omitted: merge generated static geometry.
    pub merge: Option<bool>,
    pub terrain: Option<Terrain>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Region {
    Area {
        #[schemars(extend("x-og-unit" = "m"))]
        points: Vec<[f64; 2]>,
    },
    Along {
        street: Street,
        #[schemars(extend("x-og-unit" = "m"))]
        depth: Option<f64>,
    },
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Street {
    pub name: Option<String>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub points: Vec<[f64; 2]>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub width: Option<f64>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub sidewalk: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Streets {
    /// Grid street settings are explicit, never inferred from authored streets.
    pub grid: Option<Grid>,
    #[serde(default)]
    pub streets: Vec<Street>,
    #[schemars(range(min = 0, max = 2), extend("x-og-unit" = "m"))]
    pub kerb: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Grid {
    #[schemars(extend("x-og-unit" = "m"))]
    pub size: [f64; 2],
    #[schemars(extend("x-og-unit" = "deg"))]
    pub turn: Option<f64>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub jitter: Option<f64>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub width: Option<f64>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub sidewalk: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Lots {
    /// Least/greatest dimensions in metres; ordering checked by layout domain.
    #[schemars(extend("x-og-unit" = "m"))]
    pub width: [f64; 2],
    #[schemars(extend("x-og-unit" = "m"))]
    pub depth: [f64; 2],
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Buildings {
    #[schemars(range(min = 0, max = 1))]
    pub density: Option<f64>,
    #[schemars(inner(range(min = 0)), extend("x-og-unit" = "m"))]
    pub height: Option<[f64; 2]>,
    pub height_noise: Option<HeightNoise>,
    #[serde(default)]
    pub rules: Vec<RuleUse>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct HeightNoise {
    #[schemars(range(min = 0), extend("x-og-unit" = "1/m"))]
    pub frequency: f64,
    #[schemars(range(min = 0, max = 1))]
    pub share: f64,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct RuleUse {
    #[schemars(extend("x-og-ref-kinds" = ["grammar"]))]
    pub grammar: ContentRef,
    #[schemars(extend("exclusiveMinimum" = 0))]
    pub weight: Option<f64>,
    pub street: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Prop {
    #[schemars(length(min = 1))]
    pub name: String,
    #[schemars(extend("x-og-ref-kinds" = ["model"]))]
    pub model: ContentRef,
    #[schemars(extend("x-og-unit" = "m"))]
    pub every: Option<f64>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub jitter: Option<f64>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub kerb: Option<f64>,
    pub corners: Option<bool>,
    pub street: Option<String>,
    pub collider: Option<Collider>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Collider {
    Box {
        #[schemars(extend("items" = {"type": "number", "exclusiveMinimum": 0}, "x-og-unit" = "m"))]
        size: [f64; 3],
    },
    Capsule {
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
        radius: f64,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        height: f64,
    },
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Cells {
    #[schemars(range(min = 8, max = 4096), extend("x-og-unit" = "m"))]
    pub size: f64,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    pub load: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Terrain {
    #[schemars(extend("x-og-ref-kinds" = ["terrain"]))]
    pub terrain: ContentRef,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    pub depth: Option<f32>,
}

impl DistrictInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<DistrictInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("district input schema"))
    }
}
