//! Plant growth and explicit branch paths. Branch topology, equal path/radius
//! counts, leaf ranges and named-material/season override conflicts are domain
//! checks; this describes every parser input without inferring any composition.
use super::*;

input!(Plant {
    kind: Option<PlantKind>,
    seed: Option<u32>,
    growth: Option<Growth>,
    #[schemars(range(max = 3))]
    levels: Option<u32>,
    trunk: Option<Level>,
    branch1: Option<Level>,
    branch2: Option<Level>,
    branch3: Option<Level>,
    /// Direction xyz and bending strength.
    force: Option<[f64; 4]>,
    #[schemars(inner(range(min = 0)))]
    bark_color: Option<[f64; 3]>,
    material: Option<String>,
    leaves: Option<Leaves>,
    flowers: Option<Flowers>,
    #[serde(default)]
    branches: Vec<Branch>,
    /// Model-space minimum/maximum, metres; ordering is a domain check.
    #[schemars(extend("x-og-unit" = "m"))]
    bounds: Option<[[f64; 3]; 2]>,
    #[serde(default)]
    #[schemars(extend("x-og-unit" = "m"))]
    path: Vec<[f64; 3]>,
    leaf_geometry: Option<Geometry>,
    flower_geometry: Option<Geometry>,
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum PlantKind {
    Oak,
    Birch,
    Willow,
    Pine,
    Palm,
    Bamboo,
    Cactus,
    Shrub,
    Fern,
    Reed,
    Daisy,
    Tulip,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Growth {
    Deciduous,
    Evergreen,
}

input!(Level {
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    length: Option<f64>,
    /// Trunk radius in metres; branch radius as a share of the parent's.
    #[schemars(range(min = 0))]
    radius: Option<f64>,
    #[schemars(range(min = 1, max = 64))]
    sections: Option<u32>,
    #[schemars(range(min = 3, max = 32))]
    segments: Option<u32>,
    #[schemars(range(min = 0, max = 1))]
    taper: Option<f64>,
    #[schemars(extend("x-og-unit" = "rad"))]
    gnarliness: Option<f64>,
    #[schemars(extend("x-og-unit" = "rad"))]
    twist: Option<f64>,
    #[schemars(range(max = 400))]
    children: Option<u32>,
    #[schemars(extend("x-og-unit" = "deg"))]
    angle: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    start: Option<f64>,
});

input!(Leaves {
    shape: Option<LeafShape>,
    #[schemars(range(max = 2000))]
    count: Option<u32>,
    #[schemars(range(min = 0, max = 1))]
    start: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    end: Option<f64>,
    #[schemars(extend("x-og-unit" = "deg"))]
    angle: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    facing: Option<f64>,
    #[schemars(range(max = 8))]
    rows: Option<u32>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    size: Option<f64>,
    /// Width as a share of leaf length.
    #[schemars(range(min = 0))]
    width: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    size_variance: Option<f64>,
    #[schemars(inner(range(min = 0)))]
    color: Option<[f32; 3]>,
    #[schemars(inner(range(min = 0)))]
    color2: Option<[f32; 3]>,
    #[schemars(range(min = 0, max = 1))]
    color_variation: Option<f32>,
    material: Option<String>,
    #[schemars(range(min = 0, max = 1))]
    translucency: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    dome_normals: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    season: Option<f32>,
    #[schemars(inner(range(min = 0)))]
    autumn_color: Option<[f32; 3]>,
    #[schemars(inner(range(min = 0)))]
    autumn_color2: Option<[f32; 3]>,
    #[schemars(range(min = -og_render_data::foliage::LeafSeason::MAX_SHIFT, max = og_render_data::foliage::LeafSeason::MAX_SHIFT), extend("x-og-unit" = "days"))]
    season_shift: Option<f32>,
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum LeafShape {
    Blade,
    Card,
    Cross,
}

input!(Flowers {
    #[schemars(range(min = 0, max = 1))]
    share: Option<f64>,
    #[schemars(range(max = 200))]
    count: Option<u32>,
    #[schemars(range(min = 1, max = 32))]
    petals: Option<u32>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    size: Option<f64>,
    #[schemars(range(min = 0))]
    width: Option<f64>,
    #[schemars(extend("x-og-unit" = "deg"))]
    cup: Option<f64>,
    #[schemars(inner(range(min = 0)))]
    color: Option<[f32; 3]>,
    #[schemars(inner(range(min = 0)))]
    color2: Option<[f32; 3]>,
    #[schemars(inner(range(min = 0)))]
    center_color: Option<[f32; 3]>,
});

input!(Branch {
    /// -1 is the trunk; otherwise an earlier branch index.
    #[schemars(range(min = -1, max = 19999))]
    parent: Option<i32>,
    #[schemars(extend("x-og-unit" = "m"))]
    path: Vec<[f64; 3]>,
    #[schemars(extend("x-og-unit" = "m"))]
    radii: Vec<f64>,
    #[schemars(range(min = 3, max = 32))]
    segments: Option<u32>,
});
