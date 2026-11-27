//! 2D rig inputs retain names; the domain resolves parent-first bones,
//! slots, attachments, constraints and animation channels together.
use super::clips::Filter;
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Rig2d {
    Authored(Authored),
    Dragonbones(Dragonbones),
    Parts(Parts),
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Dragonbones {
    pub conform: Option<super::tiles::TilesetConform>,
    /// Explicit DragonBones 5.5 exchange text from input.text.
    pub dragonbones: String,
    /// Explicit atlas exchange text from input.text.
    pub atlas: String,
    #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
    pub texture: ContentRef,
    pub filter: Option<Filter>,
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "px/m"))]
    pub unit: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Parts {
    pub conform: Option<super::tiles::TilesetConform>,
    /// Canonical module-relative PSD/PSB or PNG-parts directory under assets/
    /// or sources/. Directory paths have no trailing slash; no sibling lookup.
    #[schemars(extend("x-og-input" = "bytes"))]
    pub parts: String,
    pub template: String,
    #[serde(default)]
    pub pivots: BTreeMap<String, [f32; 2]>,
    pub filter: Option<Filter>,
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "px/m"))]
    pub unit: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Authored {
    pub conform: Option<super::tiles::TilesetConform>,
    #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
    pub texture: ContentRef,
    pub filter: Option<Filter>,
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "px/m"))]
    pub unit: Option<f32>,
    pub rig: Armature,
    #[serde(default)]
    pub armatures: BTreeMap<String, Armature>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Armature {
    #[schemars(length(min = 1))]
    pub bones: Vec<Bone>,
    #[serde(default)]
    pub slots: Vec<Slot>,
    #[serde(default)]
    pub attachments: BTreeMap<String, Attachment>,
    #[serde(default)]
    pub iks: Vec<Ik>,
    #[serde(default)]
    pub transforms: Vec<TransformConstraint>,
    #[serde(default)]
    pub paths: Vec<PathConstraint>,
    #[serde(default)]
    pub clips: Vec<Clip>,
    /// Slot attachment swaps, names resolved in this armature.
    #[serde(default)]
    pub skins: BTreeMap<String, BTreeMap<String, Option<String>>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Bone {
    pub name: String,
    pub parent: Option<String>,
    #[schemars(extend("x-og-unit" = "px"))]
    pub at: Option<[f32; 2]>,
    #[schemars(extend("x-og-unit" = "deg"))]
    pub rotation: Option<f32>,
    #[schemars(extend("x-og-unit" = "deg"))]
    pub skew: Option<f32>,
    pub scale: Option<[f32; 2]>,
    #[schemars(extend("x-og-unit" = "px"))]
    pub length: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Slot {
    pub name: String,
    pub bone: String,
    pub attachment: Option<String>,
    pub color: Option<[f32; 4]>,
    pub offset: Option<[f32; 4]>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Attachment {
    Region {
        slot: String,
        rect: [f32; 4],
        at: Option<[f32; 2]>,
        rotation: Option<f32>,
        skew: Option<f32>,
        size: Option<[f32; 2]>,
    },
    Mesh {
        slot: String,
        vertices: Vec<Vertex>,
        triangles: Vec<u32>,
    },
    Box {
        slot: String,
        polygon: Vec<[f32; 2]>,
    },
    Path {
        slot: String,
        points: Vec<Vec<Influence>>,
        closed: bool,
        constant: bool,
        lengths: Vec<f32>,
    },
    Child {
        slot: String,
        armature: String,
        clip: Option<String>,
        at: Option<[f32; 2]>,
        rotation: Option<f32>,
        skew: Option<f32>,
        scale: Option<[f32; 2]>,
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Vertex {
    pub uv: [f32; 2],
    pub influences: Vec<Influence>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Influence {
    pub bone: String,
    /// Bone-local pixel position, y up.
    pub at: [f32; 2],
    #[schemars(range(min = 0, max = 1))]
    pub weight: f32,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ik {
    pub name: String,
    pub bones: Vec<String>,
    pub target: String,
    pub counterclockwise: Option<bool>,
    #[schemars(range(min = 0, max = 1))]
    pub weight: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct TransformConstraint {
    pub name: String,
    pub bones: Vec<String>,
    pub target: String,
    pub rotation: Option<f32>,
    pub at: Option<[f32; 2]>,
    pub scale: Option<[f32; 2]>,
    pub shear: Option<f32>,
    #[schemars(inner(range(min = 0, max = 1)))]
    pub mix: Option<[f32; 4]>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PathConstraint {
    pub name: String,
    pub path: String,
    pub bones: Vec<String>,
    pub position: Option<f32>,
    pub position_unit: Option<PathUnit>,
    pub spacing: Option<f32>,
    pub spacing_mode: Option<PathSpacing>,
    pub rotate: Option<PathRotate>,
    pub rotation: Option<f32>,
    #[schemars(inner(range(min = 0, max = 1)))]
    pub mix: Option<[f32; 2]>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum PathUnit {
    Fixed,
    Percent,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum PathSpacing {
    Fixed,
    Percent,
    Length,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase")]
pub enum PathRotate {
    Tangent,
    Chain,
    ChainScale,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Ease {
    Linear,
    Step,
    Bezier { points: Vec<f32> },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Key<T> {
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub time: f32,
    pub value: T,
    pub ease: Option<Ease>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Turn {
    Shorter,
    Clockwise { turns: u32 },
    Counterclockwise { turns: u32 },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Rotation {
    pub degrees: f32,
    pub turn: Option<Turn>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct BoneKeys {
    pub bone: String,
    #[serde(default)]
    pub rotate: Vec<Key<Rotation>>,
    #[serde(default)]
    pub skew: Vec<Key<f32>>,
    #[serde(default)]
    pub translate: Vec<Key<[f32; 2]>>,
    #[serde(default)]
    pub scale: Vec<Key<[f32; 2]>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct SlotKeys {
    pub slot: String,
    #[serde(default)]
    pub attachment: Vec<Key<Option<String>>>,
    #[serde(default)]
    pub color: Vec<Key<[f32; 4]>>,
    #[serde(default)]
    pub offset: Vec<Key<[f32; 4]>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub name: String,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub duration: f32,
    pub looping: Option<bool>,
    #[serde(default)]
    pub bones: Vec<BoneKeys>,
    #[serde(default)]
    pub slots: Vec<SlotKeys>,
    #[serde(default)]
    pub deforms: BTreeMap<String, Vec<Key<Vec<f32>>>>,
    #[serde(default)]
    pub order: Vec<Key<Vec<String>>>,
    #[serde(default)]
    pub iks: BTreeMap<String, Vec<Key<IkValue>>>,
    #[serde(default)]
    pub events: Vec<Key<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct IkValue {
    #[schemars(range(min = 0, max = 1))]
    pub weight: f32,
    pub counterclockwise: bool,
}

schema!(Rig2d);
