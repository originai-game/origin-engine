//! Recursive one-key profiles and typed path pieces, matching the shared kernel.
use super::*;

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Profile {
    Rect(Rect),
    Circle(Circle),
    Ellipse(Ellipse),
    Slot(Slot),
    Polygon(Polygon),
    Poly(Poly),
    Gear(Gear),
    Union(#[schemars(length(min = 1))] Vec<Profile>),
    Subtract(#[schemars(length(min = 1))] Vec<Profile>),
    Intersect(#[schemars(length(min = 1))] Vec<Profile>),
    Offset(Box<Offset>),
    Translate(Box<Translate>),
    Rotate(Box<Rotate>),
    Scale(Box<Scale>),
    Superellipse(Superellipse),
    Spline(Spline),
    Path(Path2),
    Stroke(Stroke),
    Hull(#[schemars(length(min = 1))] Vec<Profile>),
    Mirror(Box<Mirror>),
    Text(Text),
}
input!(Rect { width: Option<Number>, height: Option<Number>, radius: Option<Number> });
input!(Circle { radius: Option<Number>, segments: Option<Count> });
input!(Ellipse { rx: Option<Number>, ry: Option<Number>, segments: Option<Count> });
input!(Superellipse { rx: Option<Number>, ry: Option<Number>, exponent: Option<Number>, segments: Option<Count> });
input!(Slot { length: Option<Number>, width: Option<Number> });
input!(Polygon { sides: Option<Count>, radius: Option<Number>, flats: Option<Number> });
input!(Gear { teeth: Option<Count>, module: Option<Number>, pressure_angle: Option<Number> });
input!(Poly { points: Vec<PolyPoint> });
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum PolyPoint {
    Pair(Pair),
    Corner(Corner),
}
input!(Corner {
    x: Number, y: Number, radius: Option<Number>,
    /// Circular bulge in half-chords.
    bulge: Option<Number>,
});
input!(Offset { profile: Profile, distance: Option<Number> });
input!(Translate { profile: Profile, x: Option<Number>, y: Option<Number> });
input!(Rotate { profile: Profile, angle: Option<Number> });
input!(Scale { profile: Profile, x: Option<Number>, y: Option<Number> });
input!(Spline { points: Vec<Pair> });
vocabulary!(Join { Round => "round", Miter => "miter", Bevel => "bevel" });
vocabulary!(MirrorAxis { X => "x", Y => "y" });
input!(Mirror { profile: Profile, axis: Option<MirrorAxis> });
input!(Stroke { path: Path2, width: Option<Number>, join: Option<Join>, round_caps: Option<bool>, closed: Option<bool> });
input!(Text {
    text: String,
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    font: String,
    size: Option<Number>, spacing: Option<Number>,
});

/// The path starts with a point, followed by typed line/curve pieces. Arc
/// continuity and tangent prerequisites remain geometric validation.
#[derive(Deserialize)]
#[serde(transparent)]
pub struct PathList<const D: usize>(pub Vec<Piece<D>>)
where
    [Number; D]: serde::de::DeserializeOwned + JsonSchema;

impl<const D: usize> JsonSchema for PathList<D>
where
    [Number; D]: serde::de::DeserializeOwned + JsonSchema,
{
    fn schema_name() -> std::borrow::Cow<'static, str> {
        format!("ModelPath{D}").into()
    }
    fn json_schema(
        generator: &mut og_core::reflect::source::schemars::SchemaGenerator,
    ) -> og_core::reflect::source::schemars::Schema {
        let mut schema = Vec::<Piece<D>>::json_schema(generator);
        schema.insert("minItems".into(), 1.into());
        schema.insert(
            "prefixItems".into(),
            serde_json::json!([generator.subschema_for::<[Number; D]>()]),
        );
        schema
    }
}
pub type Path2 = PathList<2>;
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Path3 {
    Pieces(PathList<3>),
    Helix(HelixPath),
}
input!(HelixPath { helix: Helix });
input!(Helix {
    radius: Number,
    pitch: Number,
    turns: Number
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Piece<const D: usize>
where
    [Number; D]: serde::de::DeserializeOwned + JsonSchema,
{
    Point([Number; D]),
    Curve(CurvePiece<D>),
}
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum CurvePiece<const D: usize>
where
    [Number; D]: serde::de::DeserializeOwned + JsonSchema,
{
    Arc(Arc<D>),
    TangentArc(TangentArc<D>),
    Spline(PathSpline<D>),
}
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Arc<const D: usize>
where
    [Number; D]: serde::de::DeserializeOwned + JsonSchema,
{
    pub through: [Number; D],
    pub to: [Number; D],
}
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct TangentArc<const D: usize>
where
    [Number; D]: serde::de::DeserializeOwned + JsonSchema,
{
    pub to: [Number; D],
}
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct PathSpline<const D: usize>
where
    [Number; D]: serde::de::DeserializeOwned + JsonSchema,
{
    #[schemars(length(min = 1))]
    pub points: Vec<[Number; D]>,
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum View {
    Constant(Number),
    Points(Vec<Pair>),
    Curve(ViewCurve),
}
input!(ViewCurve { points: Vec<Pair>, smooth: Option<bool> });
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Sections {
    Height(Vec<HeightSection>),
    Placed(Vec<PlacedSection>),
}
input!(HeightSection {
    profile: Profile,
    z: Number
});
input!(PlacedSection {
    profile: Profile,
    at: Frame
});
