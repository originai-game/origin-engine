//! Navmesh and baked-lightmap structural inputs. Bake geometry, stale hashes,
//! topology, lattice sizes and duplicate ids still need domain validation.
use super::{
    super::{ContentRef, reflected::Reflected},
    geometry::Transform,
};
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavmeshInput {
    #[schemars(extend("x-og-ref-kinds" = ["scene"]))]
    pub scene: Option<ContentRef>,
    /// Ordered profiles; unnamed profile is the default, otherwise the first.
    #[serde(default)]
    pub agents: Vec<Agent>,
    /// Voxel dimensions in metres; omitted: 0.2 horizontally, 0.1 vertically.
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
    pub cell_size: Option<f32>,
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
    pub cell_height: Option<f32>,
    /// Clamped to 8..256 and truncated by normalization; omitted: 32.
    pub tile_size: Option<f32>,
    /// Contour simplification error; omitted: 1.3.
    pub max_error: Option<f32>,
    #[serde(default)]
    pub boxes: Vec<NavBox>,
    #[serde(default)]
    pub models: Vec<NavModel>,
    #[serde(default)]
    pub terrains: Vec<NavTerrain>,
    pub bounds: Option<Bounds>,
    /// Later shapes override earlier overlaps.
    #[serde(default)]
    pub areas: Vec<Area>,
    #[serde(default)]
    pub links: Vec<Link>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Agent {
    #[schemars(regex(pattern = "^[A-Za-z0-9_-]+$"))]
    pub name: Option<String>,
    /// Metres; omitted radius/height/climb: 0.4/1.8/0.4.
    #[schemars(extend("x-og-unit" = "m"))]
    pub radius: Option<f32>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub height: Option<f32>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub climb: Option<f32>,
    /// Maximum walkable slope; omitted: 45 degrees.
    #[schemars(extend("x-og-unit" = "deg"))]
    pub slope: Option<f32>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct NavBox {
    #[schemars(extend("x-og-unit" = "m"))]
    pub center: Option<[f32; 3]>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub size: Option<[f32; 3]>,
    /// Normalized by the domain; zero-length rotations are invalid.
    pub quaternion: Option<[f32; 4]>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct NavModel {
    #[schemars(extend("x-og-ref-kinds" = ["model"]))]
    pub model: ContentRef,
    pub transform: Option<Reflected<Transform>>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct NavTerrain {
    #[schemars(extend("x-og-ref-kinds" = ["terrain"]))]
    pub terrain: ContentRef,
    #[schemars(extend("x-og-unit" = "m"))]
    pub position: Option<[f32; 3]>,
    /// Sampling stride; normalization clamps to >=1 and truncates; omitted: 1.
    pub step: Option<f32>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    #[schemars(extend("x-og-unit" = "m"))]
    pub min: Option<[f32; 3]>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub max: Option<[f32; 3]>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Area {
    #[schemars(range(min = 1, max = 63))]
    pub id: u8,
    pub shape: AreaShape,
}

/// Explicit branch selection replaces the parser's radius/point precedence.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum AreaShape {
    Box {
        center: Option<[f32; 3]>,
        size: Option<[f32; 3]>,
        #[schemars(extend("x-og-unit" = "rad"))]
        yaw: Option<f32>,
    },
    Cylinder {
        center: Option<[f32; 3]>,
        /// Metres; negative values clamp to zero during normalization.
        radius: f32,
        height: Option<f32>,
    },
    Convex {
        /// x/z coordinates in metres; vertical bounds are bottom/top.
        #[schemars(length(min = 3), extend("x-og-unit" = "m"))]
        points: Vec<[f32; 2]>,
        bottom: Option<f32>,
        top: Option<f32>,
    },
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Link {
    #[schemars(range(min = 1))]
    pub id: u32,
    #[schemars(extend("x-og-unit" = "m"))]
    pub start: [f32; 3],
    #[schemars(extend("x-og-unit" = "m"))]
    pub end: [f32; 3],
    pub kind: Option<LinkKind>,
    pub bidirectional: Option<bool>,
    /// Negative cost/duration clamp to zero; defaults are zero.
    pub cost: Option<f32>,
    #[schemars(extend("x-og-unit" = "s"))]
    pub duration: Option<f32>,
    pub open: Option<bool>,
    pub traverse: Option<Traverse>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    Jump,
    Ladder,
    Door,
    Teleport,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Traverse {
    Engine,
    Game,
}

/// Full-width u64 source hashes are hex text, not lossy JS numbers.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(transparent)]
pub struct SourceHash(#[schemars(regex(pattern = "^[0-9a-fA-F]{1,16}$"))] pub String);

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LightmapInput {
    #[schemars(extend("x-og-ref-kinds" = ["scene"]))]
    pub scene: ContentRef,
    pub scene_hash: SourceHash,
    /// External RGBA16F KTX2 array, consumed by this lightmap only.
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    pub pages: String,
    pub samples: Option<u32>,
    #[schemars(extend("x-og-unit" = "s"))]
    pub seconds: Option<f32>,
    #[serde(default)]
    pub models: Vec<BakedModel>,
    #[serde(default)]
    pub volumes: Vec<BakedVolume>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct BakedModel {
    pub entity: String,
    pub asset: BakedAsset,
    pub hash: SourceHash,
    #[serde(default)]
    pub parts: Vec<BakedPart>,
}

/// HLOD nodes are generated by the baked scene, not independently authored
/// model declarations. Their node number binds through that scene's producer.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum BakedAsset {
    Model(#[schemars(extend("x-og-ref-kinds" = ["model"]))] ContentRef),
    Proxy(HlodProxy),
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HlodProxy {
    pub hlod_proxy: u32,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct BakedPart {
    pub vertices: Option<u32>,
    #[serde(default)]
    pub xref: Vec<u32>,
    #[serde(default)]
    pub indices: Vec<u32>,
    /// Flat u, v, page triples per remapped vertex; accepts Float32Array.
    #[serde(default)]
    pub uv: Vec<f32>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct BakedVolume {
    pub entity: String,
    #[schemars(extend("x-og-unit" = "m"))]
    pub min: [f32; 3],
    #[schemars(extend("x-og-unit" = "m"))]
    pub max: [f32; 3],
    #[schemars(extend("x-og-unit" = "m"))]
    pub fade: Option<f32>,
    pub counts: [u32; 3],
    /// Flat RGB coefficients [c1.x,c1.y,c1.z,c0] per probe (12 floats),
    /// x fastest; accepts Float32Array.
    #[serde(default)]
    pub probes: Vec<f32>,
}

impl NavmeshInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<NavmeshInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("navmesh input schema"))
    }
}

impl LightmapInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<LightmapInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("lightmap input schema"))
    }
}
