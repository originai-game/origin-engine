//! Rig bones, attachments, timelines and pose data.

use super::*;

/// A bone's setup pose, relative to its parent.
#[derive(Clone, Debug, PartialEq)]
pub struct Bone2d {
    pub name: String,
    pub parent: Option<usize>,
    pub x: f32,
    pub y: f32,
    /// Degrees, counter-clockwise: the x axis's direction.
    pub rotation: f32,
    /// Degrees the y axis turns further than the x axis (counter-clockwise).
    pub skew: f32,
    pub scale: [f32; 2],
    /// Along the bone's x axis, to its tip (IK reaches with it).
    pub length: f32,
}

/// A slot: what one bone shows, in draw order.
#[derive(Clone, Debug, PartialEq)]
pub struct Slot2d {
    pub name: String,
    pub bone: usize,
    /// The attachment shown in the setup pose (an index into attachments).
    pub attachment: Option<usize>,
    /// Linear RGBA multiplied with the texels.
    pub color: [f32; 4],
    /// RGBA added after the multiplier (each -1..1).
    pub offset: [f32; 4],
}

/// One bone's pull on a mesh vertex: the point in that bone's space and its weight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Influence {
    pub bone: usize,
    pub x: f32,
    pub y: f32,
    pub weight: f32,
}

/// A mesh vertex: its texel (pixels from the image's top left) and influences.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshVertex2d {
    pub uv: [f32; 2],
    pub influences: Vec<Influence>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AttachmentKind {
    /// A rectangle of the texture (`rect`: x y w h, pixels from the top
    /// left) drawn `size` big, centred at `at` in the bone, turned by
    /// `rotation` degrees and skewed by `skew`.
    Region {
        rect: [f32; 4],
        at: [f32; 2],
        rotation: f32,
        skew: f32,
        size: [f32; 2],
    },
    /// Triangles over weighted vertices.
    Mesh {
        vertices: Vec<MeshVertex2d>,
        triangles: Vec<u32>,
    },
    /// A polygon in the slot's bone (pixels), drawn as nothing: data a game queries.
    Box { polygon: Vec<[f32; 2]> },
    /// A cubic Bézier spline, drawn as nothing: what a path constraint
    /// ([`PathConstraint2d`]) places its bones along. Its points are three
    /// per knot (the handle in, the knot, the handle out), each weighted to
    /// bones as a mesh vertex is; `closed` joins the last knot to the
    /// first; `constant` spaces positions by arc length, otherwise a
    /// position within a curve is the curve's parameter and `lengths`
    /// (each curve's end along the path, pixels) find the curve.
    Path {
        points: Vec<Vec<Influence>>,
        closed: bool,
        constant: bool,
        lengths: Vec<f32>,
    },
    /// A child armature ([`Rig2dAsset::armatures`]) placed in the slot's
    /// bone, playing `clip` (by name) unless it has the parent's clip.
    Child {
        armature: usize,
        clip: Option<String>,
        at: [f32; 2],
        rotation: f32,
        skew: f32,
        scale: [f32; 2],
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attachment2d {
    pub name: String,
    pub slot: usize,
    pub kind: AttachmentKind,
}

impl Attachment2d {
    /// Influences across the vertices (a deform key's length is twice this).
    pub fn influences(&self) -> usize {
        match &self.kind {
            AttachmentKind::Mesh { vertices, .. } => {
                vertices.iter().map(|v| v.influences.len()).sum()
            }
            AttachmentKind::Path { points, .. } => points.iter().map(Vec::len).sum(),
            _ => 0,
        }
    }
}

/// A transform constraint: each of `bones` takes `mix` of the way to the
/// `target` bone's transform in the armature's space (Spine's absolute
/// world mode): its rotation (plus `rotation` degrees), its origin (the
/// point `at` in the target's space), its axes' lengths (plus `scale`) and
/// the angle between its axes (plus `shear` degrees).
#[derive(Clone, Debug, PartialEq)]
pub struct TransformConstraint2d {
    pub name: String,
    pub bones: Vec<usize>,
    pub target: usize,
    pub rotation: f32,
    pub at: [f32; 2],
    pub scale: [f32; 2],
    pub shear: f32,
    /// Rotate, translate, scale and shear, each 0 (none) to 1 (the target's).
    pub mix: [f32; 4],
}

/// How a path constraint's `position` (and a `percent` spacing) is read.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PathUnit {
    /// A fraction of the path's length.
    #[default]
    Percent,
    /// Pixels along the path.
    Fixed,
}

/// How a path constraint spaces its bones.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PathSpacing {
    /// Each bone's length (as posed) plus `spacing` pixels.
    #[default]
    Length,
    /// `spacing` pixels.
    Fixed,
    /// `spacing` of the path's length.
    Percent,
}

/// How a path constraint turns its bones.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PathRotate {
    /// Along the path's tangent at the bone's origin.
    #[default]
    Tangent,
    /// Towards the next bone's place on the path.
    Chain,
    /// As `Chain`, the bone's x axis scaled to reach that place.
    ChainScale,
}

/// A path constraint: `bones` (in order, each usually the previous one's
/// child) placed along the `path` attachment while its slot shows it, from
/// `position` on, `spacing` apart, turned by `rotate` (plus `rotation`
/// degrees), DragonBones' and Spine's semantics; `mix` is how much of the
/// rotation and the translation applies.
#[derive(Clone, Debug, PartialEq)]
pub struct PathConstraint2d {
    pub name: String,
    pub path: usize,
    pub bones: Vec<usize>,
    pub position: f32,
    pub position_unit: PathUnit,
    pub spacing: f32,
    pub spacing_mode: PathSpacing,
    pub rotate: PathRotate,
    pub rotation: f32,
    /// Rotate and translate, each 0 to 1.
    pub mix: [f32; 2],
}

/// An IK constraint: `bones` (a chain, root first, each the previous one's
/// child) turn so the last one's tip (one bone: its direction) reaches
/// `target`'s origin.
#[derive(Clone, Debug, PartialEq)]
pub struct Ik2d {
    pub name: String,
    pub bones: Vec<usize>,
    pub target: usize,
    /// True bends the joint counter-clockwise (as seen with +y up).
    pub counterclockwise: bool,
    pub weight: f32,
}

/// How a key's value moves to the next key's.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Ease {
    /// In proportion to time.
    #[default]
    Linear,
    /// Held until the next key.
    Step,
    /// A cubic Bézier from (0, 0) to (1, 1): `x1 y1 x2 y2`, then `x y x1 y1
    /// x2 y2` per further segment (x time, y value, each a fraction).
    Bezier(Vec<f32>),
}

/// A key: a time, a value and how it eases to the next key.
#[derive(Clone, Debug, PartialEq)]
pub struct Key<T> {
    pub time: f32,
    pub value: T,
    pub ease: Ease,
}

/// Which way a `rotate` key turns to the next key's angle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Turn {
    /// The shorter way round (half a turn: clockwise).
    #[default]
    Shorter,
    /// Clockwise (the angle decreasing), through this many more whole turns.
    Clockwise(u32),
    /// Counter-clockwise (the angle increasing), through this many more whole turns.
    Counterclockwise(u32),
}

/// A `rotate` key's value: degrees and the way to the next key.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rotation {
    pub degrees: f32,
    pub turn: Turn,
}

/// How far a rotation turns from `a` to `b` degrees the `turn` way: the
/// shorter way in -180..180, else `b - a` brought into 0..360 the asked
/// way (0 when they are equal) plus 360 per further whole turn.
pub fn turn_delta(a: f32, b: f32, turn: Turn) -> f32 {
    let around = |d: f32| d - 360.0 * (d / 360.0).floor();
    match turn {
        Turn::Shorter => {
            let d = b - a;
            d - 360.0 * ((d + 180.0) / 360.0).floor()
        }
        Turn::Counterclockwise(n) => around(b - a) + 360.0 * n as f32,
        Turn::Clockwise(n) => -(around(a - b) + 360.0 * n as f32),
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoneTimeline {
    pub bone: usize,
    pub rotate: Vec<Key<Rotation>>,
    pub skew: Vec<Key<f32>>,
    pub translate: Vec<Key<[f32; 2]>>,
    pub scale: Vec<Key<[f32; 2]>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlotTimeline {
    pub slot: usize,
    pub attachment: Vec<(f32, Option<usize>)>,
    pub color: Vec<Key<[f32; 4]>>,
    pub offset: Vec<Key<[f32; 4]>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeformTimeline {
    pub attachment: usize,
    pub keys: Vec<Key<Vec<f32>>>,
}

/// An IK key's value: the constraint's weight (eased) and bend (held).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IkKey {
    pub weight: f32,
    pub counterclockwise: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IkTimeline {
    /// An index into the rig's (or armature's) constraints.
    pub ik: usize,
    pub keys: Vec<Key<IkKey>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Clip2d {
    pub name: String,
    pub duration: f32,
    /// Whether og.Skeleton2D's `loop` starts true for it (the IDL's default is true).
    pub looping: bool,
    pub bones: Vec<BoneTimeline>,
    pub slots: Vec<SlotTimeline>,
    pub deforms: Vec<DeformTimeline>,
    /// Draw orders by time: every slot once, back to front; empty is the setup order.
    pub order: Vec<(f32, Vec<usize>)>,
    pub iks: Vec<IkTimeline>,
    /// (time, name), in time order.
    pub events: Vec<(f32, String)>,
}

/// A child armature: bones, slots, attachments, constraints and clips of
/// its own, shown by a [`AttachmentKind::Child`] and drawn with the rig's
/// texture and unit.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Armature2d {
    pub name: String,
    pub bones: Vec<Bone2d>,
    pub slots: Vec<Slot2d>,
    pub attachments: Vec<Attachment2d>,
    pub iks: Vec<Ik2d>,
    pub transforms: Vec<TransformConstraint2d>,
    pub paths: Vec<PathConstraint2d>,
    pub clips: Vec<Clip2d>,
    /// Its skins beyond its default (its own attachments): it wears the
    /// one named as the skin the rig wears, else its default.
    pub skins: Vec<Skin2d>,
}

/// A skin: what each slot shows instead of the rig's own (the default
/// skin's) attachments, wherever the setup pose or a clip shows them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Skin2d {
    pub name: String,
    /// (an attachment of the default skin, what this skin shows for it: another or none).
    pub swaps: Vec<(usize, Option<usize>)>,
}

/// A 2D rig: the texture, bones, slots, attachments, constraints, clips,
/// skins and the child armatures its `child` attachments show.
#[derive(Clone, Debug, PartialEq)]
pub struct Rig2dAsset {
    pub texture: String,
    pub size: [u32; 2],
    pub filter: SheetFilter,
    /// Pixels per world unit.
    pub unit: f32,
    pub bones: Vec<Bone2d>,
    pub slots: Vec<Slot2d>,
    pub attachments: Vec<Attachment2d>,
    pub iks: Vec<Ik2d>,
    /// Applied after the IK, in order.
    pub transforms: Vec<TransformConstraint2d>,
    /// Applied after the transform constraints, in order.
    pub paths: Vec<PathConstraint2d>,
    pub clips: Vec<Clip2d>,
    /// Skins beyond the default (the rig's own attachments): skin `i + 1`
    /// of og.Skeleton2D.skin.
    pub skins: Vec<Skin2d>,
    pub armatures: Vec<Armature2d>,
}

/// A bounding box a pose shows: its slot (`parent/child` inside a child
/// armature), its attachment's name and its polygon in the rig's pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct BoundingBox2d {
    pub slot: String,
    pub name: String,
    pub points: Vec<[f32; 2]>,
}

/// One armature's parts: the rig's own or a child's.
#[derive(Clone, Copy)]
pub(super) struct Body<'a> {
    pub(super) bones: &'a [Bone2d],
    pub(super) slots: &'a [Slot2d],
    pub(super) attachments: &'a [Attachment2d],
    pub(super) iks: &'a [Ik2d],
    pub(super) transforms: &'a [TransformConstraint2d],
    pub(super) paths: &'a [PathConstraint2d],
    pub(super) clips: &'a [Clip2d],
}

impl Armature2d {
    pub(super) fn body(&self) -> Body<'_> {
        Body {
            bones: &self.bones,
            slots: &self.slots,
            attachments: &self.attachments,
            iks: &self.iks,
            transforms: &self.transforms,
            paths: &self.paths,
            clips: &self.clips,
        }
    }
}

/// A rig's pose: each bone's transform in the rig's space (pixels), what
/// each slot shows, its colour multiplier and offset, the draw order, the
/// mesh deforms, each IK constraint's remaining distance (pixels) after
/// solving, and the pose of each child armature a slot shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Pose2d {
    pub world: Vec<Affine2>,
    pub slots: Vec<Option<usize>>,
    pub colors: Vec<[f32; 4]>,
    pub offsets: Vec<[f32; 4]>,
    /// Slot indices, back to front.
    pub order: Vec<usize>,
    pub deforms: Vec<(usize, Vec<f32>)>,
    pub ik_error: Vec<f32>,
    /// (slot, the child armature's pose in that slot's placement).
    pub children: Vec<(usize, Pose2d)>,
}

/// A corner of a drawn rig.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    /// Multiplied with the texel.
    pub color: [f32; 4],
    /// Added after the multiplier.
    pub offset: [f32; 4],
}
