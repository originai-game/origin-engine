//! Structured inputs of model_text's geometry mechanisms. Defaults remain
//! absent until domain normalization; no runtime handle stands in for geometry.
use super::super::ContentRef;
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::sync::OnceLock;

/// Exactly one geometry, named as the corresponding render mechanism.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(
    tag = "kind",
    content = "desc",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum Geometry {
    ModelGeometry(ModelGeometry),
    BufferGeometry(BufferGeometry),
    BoxGeometry(BoxGeometry),
    PlaneGeometry(PlaneGeometry),
    SphereGeometry(SphereGeometry),
    CylinderGeometry(CylinderGeometry),
    ConeGeometry(ConeGeometry),
    CapsuleGeometry(CapsuleGeometry),
    TorusGeometry(TorusGeometry),
    RingGeometry(RingGeometry),
    CircleGeometry(CircleGeometry),
    TubeGeometry(TubeGeometry),
    ShapeGeometry(ShapeGeometry),
    ExtrudeGeometry(ExtrudeGeometry),
    LatheGeometry(LatheGeometry),
    PolyhedronGeometry(PolyhedronGeometry),
    IcosahedronGeometry(SolidGeometry),
    DodecahedronGeometry(SolidGeometry),
    OctahedronGeometry(SolidGeometry),
    TetrahedronGeometry(SolidGeometry),
    SweepGeometry(SweepGeometry),
    CsgGeometry(CsgGeometry),
    MergeGeometries(MergeGeometries),
}

impl Geometry {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<Geometry>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("geometry input schema"))
    }
}

// The three common descriptor fields are actual Rust fields in every input,
// not a parallel string field catalogue. No IDL desc records are registered
// in og_records::RECORDS; components such as Transform use the reflected bridge.
macro_rules! desc {
    ($(#[$doc:meta])* $name:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Deserialize, JsonSchema)]
        #[schemars(crate = "og_core::reflect::source::schemars")]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $name {
            $($(#[$attr])* pub $field: $ty,)*
            /// Scale the imported UVs. Unset means [1, 1].
            pub uv_repeat: Option<[f32; 2]>,
            /// Runtime descriptor flag; model import settings own cooked collision.
            pub physics: Option<bool>,
            /// Runtime descriptor flag; a cooked model is static source data.
            pub dynamic: Option<bool>,
        }
    };
}

desc!(ModelGeometry {
    /// A model cooked before this geometry; dependency cycles are domain errors.
    #[schemars(extend("x-og-ref-kinds" = ["model"]))]
    source: ContentRef,
    /// Select a mesh node, or merge the visible default scene at rest.
    node: Option<String>,
});
desc!(BufferGeometry {
    /// Finite x/y/z vertex positions in metres. Triangle count, attribute
    /// counts and index bounds are checked together by the domain validator.
    #[schemars(length(min = 3), inner(range(min = -f32::MAX, max = f32::MAX)), extend("x-og-unit" = "m"))]
    positions: Vec<f32>,
    /// One xyz normal per vertex; absent computes normals.
    #[schemars(inner(range(min = -f32::MAX, max = f32::MAX)))]
    normals: Option<Vec<f32>>,
    /// One uv pair per vertex; v down the image.
    #[schemars(inner(range(min = -f32::MAX, max = f32::MAX)))]
    uvs: Option<Vec<f32>>,
    /// One linear RGB or RGBA colour per vertex.
    #[schemars(inner(range(min = -f32::MAX, max = f32::MAX)))]
    colors: Option<Vec<f32>>,
    /// One xyzw tangent frame per vertex; w is handedness.
    #[schemars(inner(range(min = -f32::MAX, max = f32::MAX)))]
    tangents: Option<Vec<f32>>,
    /// Counter-clockwise triangle indices; absent uses consecutive vertices.
    #[schemars(length(min = 3))]
    indices: Option<Vec<u32>>,
});
desc!(BoxGeometry {
    /// X extent, metres; unset 1.
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    width: Option<f64>,
    /// Y extent, metres; unset 1.
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    height: Option<f64>,
    /// Z extent, metres; unset 1.
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    depth: Option<f64>,
});
desc!(PlaneGeometry {
    /// X extent, metres; unset 1. The plane faces +Z.
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    width: Option<f64>,
    /// Y extent, metres; unset 1.
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    height: Option<f64>,
});
desc!(SphereGeometry {
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    radius: Option<f64>,
    #[schemars(range(min = 3, max = 512))]
    width_segments: Option<u32>,
    #[schemars(range(min = 2, max = 512))]
    height_segments: Option<u32>,
});
desc!(CylinderGeometry {
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    radius_top: Option<f64>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    radius_bottom: Option<f64>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    height: Option<f64>,
    #[schemars(range(min = 3, max = 65536))]
    radial_segments: Option<u32>,
    #[schemars(range(min = 1, max = 65536))]
    height_segments: Option<u32>,
    open_ended: Option<bool>,
    #[schemars(extend("x-og-unit" = "rad"))]
    theta_start: Option<f64>,
    #[schemars(extend("x-og-unit" = "rad"))]
    theta_length: Option<f64>,
});
desc!(ConeGeometry {
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    radius: Option<f64>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    height: Option<f64>,
    #[schemars(range(min = 3, max = 65536))]
    radial_segments: Option<u32>,
    #[schemars(range(min = 1, max = 65536))]
    height_segments: Option<u32>,
    open_ended: Option<bool>,
});
desc!(CapsuleGeometry {
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    radius: Option<f64>,
    /// Straight middle length, not total capsule length.
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    height: Option<f64>,
    #[schemars(range(min = 1, max = 65536))]
    cap_segments: Option<u32>,
    #[schemars(range(min = 3, max = 65536))]
    radial_segments: Option<u32>,
    #[schemars(range(min = 1, max = 65536))]
    height_segments: Option<u32>,
});
desc!(TorusGeometry {
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    radius: Option<f64>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    tube: Option<f64>,
    #[schemars(range(min = 2, max = 65536))]
    radial_segments: Option<u32>,
    #[schemars(range(min = 3, max = 65536))]
    tubular_segments: Option<u32>,
    #[schemars(extend("x-og-unit" = "rad"))]
    arc: Option<f64>,
});
desc!(RingGeometry {
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    inner_radius: Option<f64>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    outer_radius: Option<f64>,
    #[schemars(range(min = 3, max = 65536))]
    theta_segments: Option<u32>,
    #[schemars(range(min = 1, max = 65536))]
    phi_segments: Option<u32>,
    #[schemars(extend("x-og-unit" = "rad"))]
    theta_start: Option<f64>,
    #[schemars(extend("x-og-unit" = "rad"))]
    theta_length: Option<f64>,
});
desc!(CircleGeometry {
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    radius: Option<f64>,
    #[schemars(range(min = 3, max = 65536))]
    segments: Option<u32>,
    #[schemars(extend("x-og-unit" = "rad"))]
    theta_start: Option<f64>,
    #[schemars(extend("x-og-unit" = "rad"))]
    theta_length: Option<f64>,
});
desc!(TubeGeometry {
    /// A sampled path or a Catmull-Rom point list, never a runtime Path handle.
    path: TubePath,
    #[schemars(range(min = 1, max = 65536))]
    tubular_segments: Option<u32>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    radius: Option<f64>,
    #[schemars(range(min = 3, max = 65536))]
    radial_segments: Option<u32>,
});
desc!(ShapeGeometry {
    /// Unset uses the generator's default triangle.
    shapes: Option<Vec<Shape>>,
    #[schemars(range(min = 1, max = 4096))]
    curve_segments: Option<u32>,
});
desc!(ExtrudeGeometry {
    /// Unset uses the generator's default square.
    shapes: Option<Vec<Shape>>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    depth: Option<f64>,
    #[schemars(range(min = 1, max = 65536))]
    steps: Option<u32>,
    bevel_enabled: Option<bool>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    bevel_thickness: Option<f64>,
    /// Unset is bevelThickness - 0.1, not a second fixed default.
    #[schemars(extend("x-og-unit" = "m"))]
    bevel_size: Option<f64>,
    #[schemars(extend("x-og-unit" = "m"))]
    bevel_offset: Option<f64>,
    #[schemars(range(min = 1, max = 4096))]
    bevel_segments: Option<u32>,
    #[schemars(range(min = 1, max = 4096))]
    curve_segments: Option<u32>,
    extrude_path: Option<Curve>,
});
desc!(LatheGeometry {
    /// Radial x and height y, metres; unset uses the generator's profile.
    #[schemars(length(min = 2), extend("x-og-unit" = "m"))]
    points: Option<Vec<[f64; 2]>>,
    #[schemars(range(min = 1, max = 65536))]
    segments: Option<u32>,
    #[schemars(extend("x-og-unit" = "rad"))]
    phi_start: Option<f64>,
    #[schemars(extend("x-og-unit" = "rad"))]
    phi_length: Option<f64>,
});
desc!(PolyhedronGeometry {
    /// Flat xyz corner coordinates, metres.
    #[schemars(length(min = 3), extend("x-og-unit" = "m"))]
    vertices: Vec<f64>,
    /// At least one triangle, indexing the corner triples.
    #[schemars(length(min = 3))]
    indices: Vec<u32>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    radius: Option<f64>,
    #[schemars(range(min = 0, max = 64))]
    detail: Option<u32>,
});
desc!(SolidGeometry {
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    radius: Option<f64>,
    #[schemars(range(min = 0, max = 64))]
    detail: Option<u32>,
});
desc!(SweepGeometry {
    /// Cross-section without holes. Curve flattening uses curveSegments.
    section: Vec<Path2Command>,
    path: Curve,
    #[schemars(range(min = 1, max = 65536))]
    steps: Option<u32>,
    /// Ordered interpolation keys along the path (t, scale).
    scale: Option<Vec<[f64; 2]>>,
    /// Ordered interpolation keys (t, radians).
    twist: Option<Vec<[f64; 2]>>,
    caps: Option<bool>,
    #[schemars(range(min = 1, max = 4096))]
    curve_segments: Option<u32>,
});
desc!(CsgGeometry {
    op: Option<CsgOp>,
    a: PlacedGeometry,
    b: PlacedGeometry,
});
desc!(MergeGeometries {
    #[schemars(length(min = 1))]
    parts: Vec<PlacedGeometry>,
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum CsgOp {
    Union,
    Subtract,
    Intersect,
}

/// One CSG operand or merged part. The transform is the actual reflected
/// engine component, not a copied table of position/quaternion/scale fields.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct PlacedGeometry {
    pub geometry: Box<Geometry>,
    pub transform: Option<super::super::reflected::Reflected<Transform>>,
    pub mirror: Option<Mirror>,
}

pub struct Transform;
impl super::super::reflected::RecordMarker for Transform {
    const NAME: &'static str = "Transform";
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Mirror {
    None,
    X,
    Y,
    Z,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum TubePath {
    Points {
        #[schemars(length(min = 2), inner(inner(range(min = -f32::MAX, max = f32::MAX))), extend("x-og-unit" = "m"))]
        points: Vec<[f32; 3]>,
        closed: Option<bool>,
    },
    Curve {
        curve: Curve,
    },
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Curve {
    #[schemars(length(min = 2), inner(inner(range(min = -f32::MAX, max = f32::MAX))), extend("x-og-unit" = "m"))]
    pub points: Vec<[f32; 3]>,
    pub kind: Option<CurveKind>,
    pub closed: Option<bool>,
    /// Catmull-Rom tangent scale; unset 0.5.
    #[schemars(range(min = -f32::MAX, max = f32::MAX))]
    pub tension: Option<f32>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase")]
pub enum CurveKind {
    CatmullRom,
    Bezier,
    Linear,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Shape {
    pub commands: Vec<Path2Command>,
    /// Hole contours cannot contain more holes.
    #[serde(default)]
    pub holes: Vec<Vec<Path2Command>>,
}

/// Ordered Path2 operations. These are structured calls, not embedded OGT.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(
    tag = "op",
    content = "args",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum Path2Command {
    MoveTo([f64; 2]),
    LineTo([f64; 2]),
    QuadraticCurveTo([f64; 4]),
    BezierCurveTo([f64; 6]),
    SplineThru(#[schemars(length(min = 1))] Vec<[f64; 2]>),
    Points(#[schemars(length(min = 1))] Vec<[f64; 2]>),
    Arc(Arc),
    #[serde(rename = "absarc")]
    AbsArc(Arc),
    Ellipse(Ellipse),
    #[serde(rename = "absellipse")]
    AbsEllipse(Ellipse),
    ClosePath,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Arc {
    #[schemars(extend("x-og-unit" = "m"))]
    pub center: [f64; 2],
    #[schemars(extend("x-og-unit" = "m"))]
    pub radius: f64,
    #[schemars(extend("x-og-unit" = "rad"))]
    pub start: f64,
    #[schemars(extend("x-og-unit" = "rad"))]
    pub end: f64,
    #[serde(default)]
    pub clockwise: bool,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Ellipse {
    #[schemars(extend("x-og-unit" = "m"))]
    pub center: [f64; 2],
    #[schemars(extend("x-og-unit" = "m"))]
    pub radii: [f64; 2],
    #[schemars(extend("x-og-unit" = "rad"))]
    pub start: f64,
    #[schemars(extend("x-og-unit" = "rad"))]
    pub end: f64,
    #[serde(default)]
    pub clockwise: bool,
    #[serde(default)]
    #[schemars(extend("x-og-unit" = "rad"))]
    pub rotation: f64,
}
