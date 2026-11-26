//! Structured kernel programs. No JSON source-text or arbitrary-value escape:
//! numeric expressions are the kernel's parameter algebra, not JavaScript.
//! Topology, rigid frames, named dependencies and expression results remain
//! checked by og-shape after parameters have been evaluated.
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

macro_rules! input {
    ($(#[$doc:meta])* $name:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Deserialize, JsonSchema)]
        #[schemars(crate = "og_core::reflect::source::schemars")]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $name { $($(#[$attr])* pub $field: $ty,)* }
    };
}
macro_rules! vocabulary {
    ($name:ident { $($variant:ident => $value:literal),* $(,)? }) => {
        #[derive(Deserialize, JsonSchema)]
        #[schemars(crate = "og_core::reflect::source::schemars")]
        pub enum $name { $(#[serde(rename = $value)] $variant,)* }
    };
}

pub mod operations;
pub mod profiles;

input!(
/// Operations use Z up and the declared model unit (mm by default); cooking
/// converts the result to metres/Y up. Angles are radians, times seconds.
ModelProgramInput {
    /// Omitted: the shared kernel's model name.
    name: Option<String>,
    #[serde(default)]
    params: BTreeMap<String, f64>,
    ops: Vec<operations::Operation>,
});

/// A finite scalar or a parameter expression containing `$`. The kernel checks
/// grammar, declared names and finite results before executing any operation.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Number {
    Literal(f64),
    Expression(#[schemars(regex(pattern = r"\$"))] String),
}

/// Whole u32 counts, or expressions whose evaluated result must be whole u32.
/// Zero selects a kernel automatic choice only where that mechanism supports it.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Count {
    Literal(u32),
    Expression(#[schemars(regex(pattern = r"\$"))] String),
}

pub type Point = [Number; 3];
pub type Pair = [Number; 2];

input!(
    /// Orthonormal right-handed basis. The kernel rejects singular/non-rigid frames.
    Frame {
        origin: Point,
        x: Point,
        y: Point,
        z: Point
    }
);

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Id {
    Index(u32),
    Handle(IdHandle),
}
input!(IdHandle { id: u32 });

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Ids {
    One(Id),
    Many(Vec<Id>),
}
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Frames {
    One(Box<Frame>),
    Many(Vec<Frame>),
}
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Names {
    One(String),
    Many(Vec<String>),
}

/// sRGB colour. Numeric triples may contain parameter expressions.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Color {
    Hex(#[schemars(regex(pattern = "^#[0-9a-fA-F]{6}$"))] String),
    Rgb(Point),
}

vocabulary!(Axis { Y => "+Y", Z => "+Z" });
vocabulary!(SignedAxis { X => "+X", NegX => "-X", Y => "+Y", NegY => "-Y", Z => "+Z", NegZ => "-Z" });
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Direction {
    Axis(SignedAxis),
    Vector(Point),
}
vocabulary!(Unit { Millimetres => "mm", Centimetres => "cm", Metres => "m" });
vocabulary!(Curve { Auto => "auto", Straight => "straight", CatmullRom => "catmullRom", Spline => "spline" });
vocabulary!(Keep { Above => "above", Below => "below" });
vocabulary!(Noise { Fbm => "fbm", Ridged => "ridged", Crater => "crater" });
vocabulary!(JointKind { Revolute => "revolute", Continuous => "continuous", Prismatic => "prismatic", Fixed => "fixed", Floating => "floating" });

input!(Surface {
    /// Texels per model unit.
    density: Option<Number>,
    seed: Option<Count>,
    normal_scale: Option<Number>,
    #[serde(default)]
    layers: Vec<SurfaceLayer>,
});
vocabulary!(Pattern {
    Noise => "noise", Ridged => "ridged", Craters => "craters", Strata => "strata",
    Stripes => "stripes", Panels => "panels", Rivets => "rivets", Veins => "veins",
    Edges => "edges", Cavities => "cavities", Facing => "facing", Gradient => "gradient",
});
input!(SurfaceLayer {
    pattern: Pattern,
    scale: Option<Number>, width: Option<Number>, cell: Option<Number>, inset: Option<Number>,
    axis: Option<Direction>, octaves: Option<Count>, threshold: Option<Number>,
    softness: Option<Number>, seed: Option<Count>, amount: Option<Number>, breakup: Option<Number>,
    color: Option<Color>, jitter: Option<Number>, roughness: Option<Number>, metalness: Option<Number>,
    emissive: Option<Color>, glow: Option<Number>, relief: Option<Number>,
});

impl ModelProgramInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<ModelProgramInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("model-program input schema"))
    }
}
