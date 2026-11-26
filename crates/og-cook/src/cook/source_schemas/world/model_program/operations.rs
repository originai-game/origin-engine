//! Closed operation vocabulary. Cross-operation ids, names, geometry and
//! mutually dependent numerical results are validated by the model kernel.
use super::{profiles::*, *};

// Keep each operation's tag inside its closed branch. An internally tagged
// newtype containing an untagged enum would put the tag outside the inner
// branch's additionalProperties:false constraint.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Operation {
    Basic(Box<BasicOperation>),
    Extrude(Extrusion),
    Material(Material),
    Pose(Pose),
    Skin(Skin),
    Morph(Morph),
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum BasicOperation {
    Frame {
        origin: Point,
        normal: Point,
        up: Option<Point>,
    },
    Offset {
        frame: Frame,
        distance: Number,
    },
    Translate {
        frame: Frame,
        x: Option<Number>,
        y: Option<Number>,
        z: Option<Number>,
    },
    Turn {
        frame: Frame,
        rx: Option<Number>,
        ry: Option<Number>,
        rz: Option<Number>,
    },
    Compose {
        frame: Frame,
        local: Frame,
    },
    Bbox {
        ids: Ids,
        sides: String,
    },
    Face {
        id: Id,
        name: String,
    },
    Edge {
        id: Id,
        name: String,
    },
    Local {
        id: Id,
        frame: Frame,
    },
    Anchor {
        id: Id,
        name: String,
    },
    Placement {
        id: Id,
    },
    Grid {
        nx: Option<Count>,
        ny: Option<Count>,
        px: Option<Number>,
        py: Option<Number>,
        stagger: Option<bool>,
        on: Option<Frame>,
    },
    Ring {
        n: Option<Count>,
        radius: Number,
        start: Option<Number>,
        angle: Option<Number>,
        on: Option<Frame>,
    },
    Along {
        path: AlongPath,
        pitch: Option<Number>,
        count: Option<Count>,
    },
    Snap {
        ids: Ids,
        points: Vec<Point>,
        direction: Option<Point>,
    },
    Names {
        id: Id,
    },
    Measure {
        posed: Option<bool>,
    },
    Units {
        unit: Unit,
    },
    Uvs {},
    Box {
        name: String,
        width: Number,
        depth: Number,
        height: Number,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Cylinder {
        name: String,
        radius: Number,
        radius_top: Option<Number>,
        height: Number,
        segments: Option<Count>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Cone {
        name: String,
        radius: Number,
        height: Number,
        segments: Option<Count>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Sphere {
        name: String,
        radius: Number,
        axis: Option<Axis>,
        segments: Option<Count>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Dome {
        name: String,
        radius: Number,
        height: Option<Number>,
        axis: Option<Axis>,
        segments: Option<Count>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Tube {
        name: String,
        outer: Number,
        inner: Number,
        height: Number,
        segments: Option<Count>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Torus {
        name: String,
        radius: Number,
        tube: Number,
        angle: Option<Number>,
        axis: Option<Axis>,
        segments: Option<Count>,
        tube_segments: Option<Count>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Wedge {
        name: String,
        width: Number,
        depth: Number,
        height: Number,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Revolve {
        name: String,
        profile: Profile,
        angle: Option<Number>,
        axis: Option<Axis>,
        segments: Option<Count>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Sweep {
        name: String,
        profile: Profile,
        path: Path3,
        up: Option<Point>,
        twist: Option<Number>,
        scale_end: Option<Number>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    /// All sections use one placement form. Placed sections allow only
    /// straight/catmullRom; a closed loop needs placed sections.
    Loft {
        name: String,
        sections: Sections,
        samples: Option<Count>,
        steps: Option<Count>,
        curve: Option<Curve>,
        closed: Option<bool>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Thread {
        name: String,
        radius: Number,
        pitch: Number,
        length: Number,
        depth: Number,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Wire {
        name: String,
        path: Path3,
        diameter: Number,
        closed: Option<bool>,
        fillet: Option<Number>,
        fillets: Option<Vec<Number>>,
        segments: Option<Count>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Body {
        name: String,
        plan: View,
        top: View,
        bottom: View,
        n: Option<View>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Knurl {
        name: String,
        diameter: Number,
        height: Number,
        pitch: Option<Number>,
        straight: Option<bool>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    Gear {
        name: String,
        teeth: Option<Count>,
        module: Number,
        width: Number,
        pressure_angle: Option<Number>,
        bore: Option<Number>,
        hub: Option<Number>,
        hub_height: Option<Number>,
        uvs: Option<bool>,
        on: Option<Frame>,
    },
    At {
        id: Id,
        frame: Frame,
    },
    Transform {
        id: Id,
        frame: Frame,
    },
    Group {
        name: String,
        children: Ids,
    },
    Wrap {
        name: String,
        parts: Ids,
    },
    Assembly {
        name: String,
        children: Ids,
        anchors: Option<BTreeMap<String, Frame>>,
    },
    Attach {
        to: Id,
        name: String,
        children: Ids,
    },
    Copy {
        id: Id,
        name: String,
    },
    Mirror {
        id: Id,
        plane: Frame,
        name: String,
    },
    Cut {
        target: Id,
        tool: Id,
        check: Option<bool>,
    },
    Join {
        target: Id,
        tool: Id,
        check: Option<bool>,
    },
    Intersect {
        target: Id,
        tool: Id,
        check: Option<bool>,
    },
    Merge {
        target: Id,
        tool: Id,
    },
    Delete {
        id: Id,
    },
    Seat {
        host: Id,
        part: Id,
    },
    Scale {
        id: Id,
        x: Option<Number>,
        y: Option<Number>,
        z: Option<Number>,
    },
    Hole {
        id: Id,
        diameter: Number,
        at: Frames,
        depth: Option<Number>,
        counterbore: Option<Pair>,
        countersink: Option<Pair>,
    },
    Pocket {
        id: Id,
        profile: Profile,
        depth: Option<Number>,
        at: Option<Frames>,
    },
    Split {
        id: Id,
        frame: Frame,
        keep: Option<Keep>,
    },
    Shell {
        id: Id,
        thickness: Number,
        open: Option<Names>,
    },
    Dent {
        id: Id,
        at: Vec<Point>,
        radius: Number,
        depth: Number,
    },
    Subdivide {
        id: Id,
        levels: Option<Count>,
        sharp: Option<Names>,
    },
    Displace {
        id: Id,
        noise: Option<Noise>,
        scale: Number,
        amount: Number,
        octaves: Option<Count>,
        seed: Option<Count>,
        density: Option<Number>,
    },
    Fillet {
        id: Id,
        radius: Number,
        edges: Option<Names>,
        segments: Option<Count>,
        profile: Option<Number>,
    },
    Chamfer {
        id: Id,
        distance: Number,
        distance2: Option<Number>,
        angle: Option<Number>,
        edges: Option<Names>,
        segments: Option<Count>,
    },
    Soften {
        id: Id,
        radius: Number,
        angle: Option<Number>,
        edges: Option<Names>,
        segments: Option<Count>,
    },
    Joint {
        child: Id,
        parent: Id,
        name: String,
        kind: Option<JointKind>,
        on: Frame,
        axis: Option<Direction>,
        range: Option<Pair>,
        mimic: Option<Mimic>,
    },
    Clip {
        name: String,
        keys: Vec<PoseKey>,
    },
    Humanoid {
        id: Id,
        bind: Option<bool>,
    },
}

vocabulary!(ExtrudeTag { Extrude => "extrude" });
vocabulary!(MaterialTag { Material => "material" });
vocabulary!(PoseTag { Pose => "pose" });
vocabulary!(SkinTag { Skin => "skin" });
vocabulary!(MorphTag { Morph => "morph" });

// The three closed paint sources share the same physical material fields.
// Expanding actual serde structs avoids flatten + deny_unknown_fields, which
// would otherwise lose closed-branch decoding or require a custom decoder.
macro_rules! material {
    ($name:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        input!($name {
            op: MaterialTag,
            ids: Ids, name: Option<String>, color: Option<Color>, metalness: Option<Number>,
            roughness: Option<Number>, emissive: Option<Color>, opacity: Option<Number>,
            clearcoat: Option<Number>, clearcoat_roughness: Option<Number>,
            sheen_color: Option<Color>, sheen_roughness: Option<Number>, transmission: Option<Number>,
            iridescence: Option<Number>, $($(#[$attr])* $field: $ty,)*
        });
    };
}
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Material {
    Plain(PlainMaterial),
    Image(ImageMaterial),
    Painted(PaintedMaterial),
}
material!(PlainMaterial {});
material!(ImageMaterial {
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    map: String,
    /// Model units spanned by the image; only valid with map.
    map_size: Option<Number>,
});
material!(PaintedMaterial { surface: Surface });

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Extrusion {
    Height(ExtrudeHeight),
    Until(ExtrudeUntil),
}
input!(ExtrudeHeight { op: ExtrudeTag, name: String, profile: Profile, height: Number, twist: Option<Number>, taper: Option<Number>, both_sides: Option<bool>, uvs: Option<bool>, on: Option<Frame> });
input!(ExtrudeUntil { op: ExtrudeTag, name: String, profile: Profile, until: Until, uvs: Option<bool>, on: Option<Frame> });
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Pose {
    Joint(JointPose),
    Clip(ClipPose),
}
input!(JointPose {
    op: PoseTag,
    joint: String,
    value: Number
});
input!(ClipPose {
    op: PoseTag,
    clip: String,
    #[schemars(extend("x-og-unit" = "s"))]
    time: Number
});
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Skin {
    Regions(RegionSkin),
    Heat(HeatSkin),
}
input!(RegionSkin { op: SkinTag, id: Id, bones: Vec<Bone>, blend: Option<Number>, max_edge: Option<Number> });
input!(HeatSkin { op: SkinTag, id: Id, joints: Vec<SkinJoint> });
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Morph {
    Target(TargetMorph),
    Deltas(DeltaMorph),
    Regions(RegionMorph),
    Offset(OffsetMorph),
    Scale(ScaleMorph),
}
input!(TargetMorph { op: MorphTag, id: Id, name: String, target: Id, weight: Option<Number> });
input!(DeltaMorph { op: MorphTag, id: Id, name: String, deltas: Vec<Point>, weight: Option<Number> });
input!(RegionMorph { op: MorphTag, id: Id, name: String, regions: Vec<String>, offset: Option<Point>, scale: Option<Point>, weight: Option<Number> });
input!(OffsetMorph { op: MorphTag, id: Id, name: String, offset: Point, scale: Option<Point>, weight: Option<Number> });
input!(ScaleMorph { op: MorphTag, id: Id, name: String, scale: Point, weight: Option<Number> });

input!(AlongPath { points: Vec<Point>, closed: Option<bool> });
input!(Until {
    part: Id,
    face: String
});
input!(Mimic { joint: String, multiplier: Option<Number> });
input!(PoseKey {
    /// Seconds, strictly increasing across keys (domain validation).
    #[schemars(extend("x-og-unit" = "s"))]
    time: Number,
    pose: BTreeMap<String, Number>,
});
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Pivot {
    Point(Point),
    Frame(Box<Frame>),
}
input!(Bone { name: String, parent: Option<String>, pivot: Option<Pivot>, regions: Option<Vec<String>> });
input!(SkinJoint { name: String, parent: Option<String>, at: Point });
