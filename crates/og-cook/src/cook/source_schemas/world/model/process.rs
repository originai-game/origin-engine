//! Explicit mesh preparation and prefracture. Kernel topology, source/UV/map
//! compatibility, named dependencies and nonzero plane normals stay domain checks.
use super::*;

input!(Process {
    repair: Option<Repair>,
    decimate: Option<Decimate>,
    unwrap: Option<Unwrap>,
    bake: Option<Bake>,
    project: Option<Project>,
    transfer: Option<Transfer>,
    rig: Option<Rig>,
});

input!(Repair {
    #[schemars(range(min = 0, max = 1e9), extend("x-og-unit" = "m"))]
    weld: Option<f64>,
    weld_all: Option<bool>,
    #[schemars(range(min = 0, max = 1e9), extend("x-og-unit" = "m"))]
    floaters: Option<f64>,
    #[schemars(range(min = 0, max = 1))]
    floater_fraction: Option<f64>,
    non_manifold: Option<bool>,
    hole_edges: Option<u32>,
    holes_through: Option<bool>,
    #[schemars(range(min = 0, max = 1e9), extend("x-og-unit" = "m"))]
    thicken: Option<f64>,
    fuse: Option<bool>,
    #[schemars(range(min = 0, max = 1e9), extend("x-og-unit" = "m"))]
    min_edge: Option<f64>,
});

input!(Decimate {
    /// Zero leaves the triangle-count budget unset.
    target: Option<u32>,
    #[schemars(range(min = 0, max = 1))]
    ratio: Option<f64>,
    #[schemars(range(min = 0, max = 1e9), extend("x-og-unit" = "m"))]
    max_error: Option<f64>,
    lock_border: Option<bool>,
    #[schemars(range(min = 0, max = 1e9))]
    merge_charts: Option<f64>,
    cross_color_borders: Option<bool>,
});

input!(Unwrap {
    #[schemars(range(min = 0, max = 89.999999), extend("x-og-unit" = "deg"))]
    angle: Option<f64>,
    #[schemars(range(max = 8192), extend("x-og-unit" = "px"))]
    resolution: Option<u32>,
    #[schemars(range(min = 0, max = 8192), extend("x-og-unit" = "px"))]
    padding: Option<f64>,
    shared: Option<bool>,
    conformal: Option<bool>,
    seams: Option<bool>,
});

input!(Bake {
    #[schemars(range(min = 1, max = 8192), extend("x-og-unit" = "px"))]
    size: Option<u32>,
    #[schemars(range(min = 0, max = 1e9), extend("x-og-unit" = "m"))]
    distance: Option<f64>,
    #[schemars(range(max = 8192), extend("x-og-unit" = "px"))]
    padding: Option<u32>,
    /// Zero fills all uncovered texels.
    dilate: Option<u32>,
    normals: Option<bool>,
});

input!(Project {
    #[schemars(range(min = 1, max = 8192), extend("x-og-unit" = "px"))]
    size: Option<u32>,
    #[schemars(range(max = 8192), extend("x-og-unit" = "px"))]
    padding: Option<u32>,
    #[schemars(length(min = 1))]
    views: Vec<View>,
});

input!(View {
    /// Square RGBA source; actual dimensions/decoding are input-domain checks.
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    image: String,
    #[schemars(extend("x-og-unit" = "m"))]
    position: [f32; 3],
    #[schemars(extend("x-og-unit" = "m"))]
    target: [f32; 3],
    projection: Projection,
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Projection {
    Perspective {
        #[schemars(range(max = 179), extend("exclusiveMinimum" = 0, "x-og-unit" = "deg"))]
        fov: f32,
    },
    Orthographic {
        #[schemars(range(max = 1e9), extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
        ortho: f32,
    },
}

input!(Transfer {
    #[schemars(extend("x-og-ref-kinds" = ["model"]))]
    source: ContentRef,
    #[schemars(range(min = 1, max = 8192), extend("x-og-unit" = "px"))]
    size: Option<u32>,
    #[schemars(range(max = 8192), extend("x-og-unit" = "px"))]
    padding: Option<u32>,
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Rig {
    Auto,
    Joints {
        #[schemars(length(min = 1))]
        joints: Vec<Joint>,
    },
}

input!(Joint {
    name: String,
    /// Empty string denotes root; all other parents must precede the child.
    parent: String,
    #[schemars(extend("x-og-unit" = "m"))]
    position: [f32; 3],
});

input!(Fracture {
    style: Option<Style>,
    #[schemars(range(min = 2, max = 256))]
    chunks: Option<u32>,
    seed: Option<u32>,
    /// Explicit centres replace sampling; count 2..128, distinct, volume
    /// Voronoi only, exclusive with chunks/impact (domain validation).
    #[schemars(length(min = 2, max = 128), extend("x-og-unit" = "m"))]
    sites: Option<Vec<[f64; 3]>>,
    #[schemars(range(max = 32))]
    children: Option<u32>,
    pattern: Option<Pattern>,
    #[schemars(inner(range(min = 0.01)), extend("x-og-unit" = "m"))]
    brick: Option<[f64; 2]>,
    grain: Option<Axis>,
    #[schemars(range(min = 1, max = 8))]
    rings: Option<u32>,
    impact: Option<Impact>,
    anchor: Option<Anchor>,
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "1/m"))]
    interior_scale: Option<f64>,
    interior: Option<Material>,
    mortar: Option<Material>,
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Style {
    Voronoi,
    Concrete,
    Brick,
    Wood,
    Glass,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Pattern {
    Auto,
    Volume,
    Slab,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
    Z,
}

input!(Impact {
    #[schemars(extend("x-og-unit" = "m"))]
    position: [f64; 3],
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
    radius: f64,
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Anchor {
    None,
    Supports {
        bottom: bool,
        ends: bool,
    },
    Plane {
        normal: [f32; 3],
        #[schemars(extend("x-og-unit" = "m"))]
        distance: f32,
    },
}
