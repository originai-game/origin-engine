//! 2D skeletal animation: the `type rig2d` asset (bones, slots in draw
//! order, region, weighted-mesh, bounding-box and child-armature
//! attachments, IK constraints, child armatures, clips with bone, slot,
//! deform, draw-order and IK keys and events), its pose at a clip time, the
//! textured triangles that pose draws ([`Rig2dAsset::triangles`]) and the
//! bounding boxes it holds ([`Rig2dAsset::boxes`]).
//!
//! **The text** (`.ogt`, `type rig2d`, `version 1`; the cook stores the
//! canonical form [`Rig2dAsset::to_text`] writes). Rig coordinates are
//! pixels with +y up, angles degrees counter-clockwise; `unit` pixels make
//! one world unit. Bones come parent first; slots draw in file order.
//!
//! ```text
//! texture "hero.png"          # the image the attachments cut from
//! size 128 128                # its pixels (the cook fills it in)
//! filter linear               # or nearest
//! unit 32                     # pixels per world unit
//! bone hip { parent root; at 0 40; rotation 90; skew 0; scale 1 1; length 20 }
//! slot torso { bone hip; attachment torso; color 1 1 1 1; offset 0 0 0 0 }
//! region torso { slot torso; rect 0 0 24 40; at 10 0; rotation -90; skew 0; size 24 40 }
//! mesh cape {
//!   slot cape
//!   vertex u v x y                      # unweighted: x y in the slot's bone
//!   vertex u v bone x y w [bone x y w]  # weighted: x y in each bone, weights summing to 1
//!   triangles 0 1 2 2 3 0
//! }
//! box hurt { slot torso; polygon -8 0 8 0 8 30 -8 30 }
//! child flame { slot lamp; armature fire; clip burn; at 0 6; rotation 0; skew 0; scale 1 1 }
//! ik leg { bones thigh shin; target foot-target; bend clockwise; weight 1 }
//! path rail {                  # a cubic spline, drawn as nothing
//!   slot rail; closed false; constant true   # constant: placed by arc length
//!   lengths 20 45                            # constant false: each curve's end along it
//!   vertex x y                               # three per knot (handle in, knot, handle out),
//!   vertex bone x y w [bone x y w]           # in the slot's bone or weighted
//! }
//! transform follow { bones a b; target t; rotation 0; at 0 0; scale 0 0; shear 0; mix 1 1 1 1 }
//! along spine { path rail; bones s1 s2 s3; position 0 percent; spacing 0 length; rotate tangent 0; mix 1 1 }
//! armature fire {               # a child armature: bone, slot, region, mesh,
//!   bone base { }               # box, child, ik and clip lines as above,
//!   ...                         # sharing the rig's texture and unit
//! }
//! clip walk {
//!   duration 0.8
//!   loop true
//!   bone thigh { rotate 0 20 counterclockwise 1 curve 0.25 0 0.75 1; rotate 0.4 -20; skew 0 0; translate 0 0 0; scale 0.2 1 1.1 step }
//!   slot eye { attachment 0.5 eye-shut; attachment 0.6 eye-open; color 0 1 1 1 1; offset 0 0.2 0 0 0 }
//!   deform cape { key 0 0 0 0 0; key 0.4 0 -3 0 -6 }
//!   order 0.3 cape torso eye    # every slot, back to front; `order <time>` alone is the setup order
//!   ik leg { key 0 1 clockwise; key 0.4 0 counterclockwise }
//!   event 0.2 "step"
//! }
//! ```
//!
//! (Entries are one per line in a file; `;` above only saves space.)
//!
//! **Bones.** `skew` turns the bone's y axis that many degrees further
//! counter-clockwise than its x axis (`rotation`): a bone's x axis is
//! (cos r, sin r) times `scale` x and its y axis (-sin (r + k), cos (r + k))
//! times `scale` y. Regions and child attachments take `skew` the same way.
//!
//! **Slots.** A slot's colour is `color` (linear RGBA multiplied with the
//! texel) and `offset` (RGBA added after it, each -1..1): a drawn texel is
//! `texel * color + offset`, clamped to 0..1. A child armature's slots
//! compose with the slot showing it: the parent's multiplier and offset
//! apply to the child's result.
//!
//! **Keys.** A bone key is relative to the setup pose: `rotate` and `skew`
//! add degrees, `translate` adds pixels, `scale` multiplies. A `rotate`
//! key turns to the next key's angle the shorter way round, unless its
//! angle is followed by `clockwise <n>` or `counterclockwise <n>`: that
//! way round, through n more whole turns (`clockwise 0` from 0 to 30 turns
//! 330 degrees, `counterclockwise 2` from 0 to 30 turns 750). Every key
//! eases towards the next one: linearly by default, `step` holds the value
//! until the next key, and `curve x1 y1
//! x2 y2` plays a cubic Bézier from (0, 0) through those control points to
//! (1, 1), where x is the fraction of time between the keys and y the
//! fraction of the value; more segments follow as `x y x1 y1 x2 y2` (an
//! anchor then its two control points), every x in 0..1 and the anchors in
//! order. A `deform` key offsets every influence of the mesh's vertices in
//! order (one x y pair per influence; an unweighted vertex has one), in
//! that influence's bone space. An `order` key holds a draw order (every
//! slot once, back to front) until the next. An `ik` key sets the
//! constraint's weight (eased) and its bend (held until the next key).
//!
//! **IK.** An `ik` constraint's `bones` are a chain, root first, each the
//! previous one's child. One bone turns to point at the target bone's
//! origin; two (a parent and its child) are solved exactly so the child's
//! tip reaches it, the joint bending clockwise or counter-clockwise; three
//! or more are solved by FABRIK (at most 64 passes, to 1e-4 pixels) from
//! the animated pose, its joints first put on the `bend` side of the line
//! from the root to the target, so a tail or a tentacle curls the asked
//! way. Out of reach the chain points straight at the target. A game moves
//! the target with og.IkTarget2D, whose weight multiplies the constraint's
//! (its `weight` line, or its clip's key).
//!
//! **Child armatures.** A `child` attachment shows an `armature` in its
//! slot, placed by `at`, `rotation`, `skew` and `scale` in the slot's bone
//! and drawn at the slot's place in the draw order. It plays the clip of
//! the armature named as the parent's playing clip when there is one (as
//! DragonBones' inherited animation does), else its `clip` line, else its
//! first clip, at the parent's clip time (wrapped by its own duration when
//! it loops, held at its end otherwise); its IK reaches its own target
//! bones. An armature never shows itself, directly or through another.
//!
//! **Transform and path constraints**, applied after the IK, each in
//! file order, to the world transforms (a constrained bone's children
//! follow it). A `transform` moves each of its `bones` `mix` (rotate,
//! translate, scale, shear, each 0..1) of the way to the `target` bone's
//! world transform: its rotation plus `rotation` degrees, its origin at
//! the point `at` in the target, its axes' lengths plus `scale`, the angle
//! between its axes plus `shear` (Spine's absolute world mode; DragonBones
//! has none). An `along` places its `bones` on a `path` attachment while
//! its slot shows it (DragonBones' and Spine's path constraint): from
//! `position` (`percent` of the path's length or `fixed` pixels), `spacing`
//! apart (`length`: each bone's posed length plus that many pixels;
//! `fixed` pixels; `percent` of the length), turned along the path's
//! tangent (`tangent`) or towards the next bone's place (`chain`, the next
//! bone then starting at this one's tip; `chainscale`, the bone stretched
//! to reach it), plus `rotate`'s degrees; `mix` is rotate then translate.
//! A closed path wraps; an open one runs on past its ends along its end
//! handles. A `path` draws nothing and a `deform` key offsets its points.
//!
//! **Skins.** The rig's own attachments are its `default` skin. A `skin
//! armor { swap torso torso-armor; swap hat none }` block (at the top
//! level, after the attachments it names) shows, wherever the setup pose
//! or a clip shows the first attachment, the second (in the same slot) or
//! nothing; og.Skeleton2D.skin picks one (Rig2DInfo.skins, `default`
//! first). A child armature wears its own default skin.
//!
//! **Bounding boxes.** A `box` attachment draws nothing: its `polygon` (at
//! least three x y points in the slot's bone) is data a game queries
//! (og.anim.skeleton2dBoxes) while its slot shows it.
//!
//! **Converting another format.** DragonBones JSON is imported by the cook
//! (og-cook's `rig2d.rs`). Spine's format is not imported (its licence);
//! a converter maps it so: Spine `bones` (`x`, `y`, `rotation`, `scaleX`,
//! `scaleY`, `length`, y up, degrees counter-clockwise; `shearY` minus
//! `shearX` as `skew`, with `shearX` added to `rotation`) to `bone` blocks;
//! `slots` (`bone`, `attachment`, `color` hex to linear 0..1 after sRGB;
//! `dark` has no line) to `slot` blocks in the same order; a `region`
//! attachment's `x`, `y`, `rotation`, `width`, `height` to `at`,
//! `rotation`, `size` with the atlas region's rectangle (pixels from the
//! top left) as `rect`; a `mesh` attachment's `uvs` (times the region's
//! size, plus its corner) and `vertices` (unweighted pairs, or Spine's
//! `count, bone, x, y, weight` groups) to `vertex` lines and `triangles` as
//! they are; a `boundingbox` to `box` with its `vertices` as `polygon`;
//! `ik` with its `bones` (any number), `target` and `bendPositive` true as
//! `counterclockwise`; an animation's `bones` timelines (`rotate` `value`,
//! `translate` and `scale` `x` `y`, `shear` as `skew`, `time` in seconds;
//! `curve: "stepped"` as `step`, a Bézier `curve` c1 c2 c3 c4 as `curve`),
//! `slots` `attachment` and `rgba`, `deform` `offset` and `vertices` per
//! influence, `drawOrder` offsets resolved into `order` keys, `ik`
//! `mix` and `bendPositive` as `ik` keys, and `events`; a `path`
//! attachment to `path`, a path constraint to `along` and a transform
//! constraint (absolute, world) to `transform`. Clipping has no line.
//!
//! **Determinism.** Every angle goes through [`sincos`] and [`atan2`],
//! written with `+ - * /` and `sqrt` in f64, and a Bézier is solved by
//! bisection, so a pose is the same bits on every host.

use og_core::ogt::{Block, Document, Entry, Number, Value};
use og_core::{Error, ErrorKind, Result};

use crate::SheetFilter;

/// The value `type` names in a rig's text.
pub const RIG2D_TYPE: &str = "rig2d";

const PI: f64 = std::f64::consts::PI;
const DEG: f64 = PI / 180.0;

/// sin and cos of `x` with basic arithmetic only (the camera systems' series).
pub fn sincos(x: f64) -> (f64, f64) {
    let q = (x / (PI / 2.0)).round();
    let r = x - q * (PI / 2.0);
    let r2 = r * r;
    let mut s = 1.0 / 1_307_674_368_000.0;
    for d in [6_227_020_800.0, 39_916_800.0, 362_880.0, 5040.0, 120.0, 6.0] {
        s = s * -r2 + 1.0 / d;
    }
    let s = r - r * r2 * s;
    let mut c = 1.0 / 87_178_291_200.0;
    for d in [479_001_600.0, 3_628_800.0, 40_320.0, 720.0, 24.0, 2.0] {
        c = c * -r2 + 1.0 / d;
    }
    let c = 1.0 - r2 * c;
    match (q as i64).rem_euclid(4) {
        0 => (s, c),
        1 => (c, -s),
        2 => (-s, -c),
        _ => (-c, s),
    }
}

fn atan_small(x: f64) -> f64 {
    let x2 = x * x;
    let mut s = 0.0;
    for n in (0..=22).rev() {
        s = s * -x2 + 1.0 / (2 * n + 1) as f64;
    }
    x * s
}

fn atan_unit(x: f64) -> f64 {
    if x > 0.414_213_562_373_095_03 {
        PI / 4.0 + atan_small((x - 1.0) / (x + 1.0))
    } else {
        atan_small(x)
    }
}

/// atan2(y, x) with basic arithmetic only.
pub fn atan2(y: f64, x: f64) -> f64 {
    if x == 0.0 && y == 0.0 {
        return 0.0;
    }
    let (ax, ay) = (x.abs(), y.abs());
    let a = if ay <= ax {
        atan_unit(ay / ax)
    } else {
        PI / 2.0 - atan_unit(ax / ay)
    };
    let r = if x < 0.0 { PI - a } else { a };
    if y < 0.0 { -r } else { r }
}

/// An angle wrapped into -pi..pi.
fn wrap(a: f64) -> f64 {
    let k = ((a + PI) / (2.0 * PI)).floor();
    a - k * 2.0 * PI
}

/// A 2D affine transform: `(x, y)` maps to `(a x + c y + tx, b x + d y + ty)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine2 {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Affine2 {
    pub const IDENTITY: Affine2 = Affine2 {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    /// Translate, rotate (degrees, counter-clockwise), scale.
    pub fn trs(x: f32, y: f32, degrees: f32, sx: f32, sy: f32) -> Affine2 {
        let (s, c) = sincos(f64::from(degrees) * DEG);
        let (s, c) = (s as f32, c as f32);
        Affine2 {
            a: c * sx,
            b: s * sx,
            c: -s * sy,
            d: c * sy,
            tx: x,
            ty: y,
        }
    }

    /// [`Affine2::trs`] with the y axis turned `skew` degrees further than
    /// the x axis (counter-clockwise); skew 0 is `trs` to the bit.
    pub fn trsk(x: f32, y: f32, degrees: f32, skew: f32, sx: f32, sy: f32) -> Affine2 {
        let mut m = Affine2::trs(x, y, degrees, sx, sy);
        if skew != 0.0 {
            let (s, c) = sincos(f64::from(degrees + skew) * DEG);
            m.c = -(s as f32) * sy;
            m.d = c as f32 * sy;
        }
        m
    }

    pub fn mul(&self, l: &Affine2) -> Affine2 {
        Affine2 {
            a: self.a * l.a + self.c * l.b,
            b: self.b * l.a + self.d * l.b,
            c: self.a * l.c + self.c * l.d,
            d: self.b * l.c + self.d * l.d,
            tx: self.a * l.tx + self.c * l.ty + self.tx,
            ty: self.b * l.tx + self.d * l.ty + self.ty,
        }
    }

    pub fn apply(&self, x: f32, y: f32) -> [f32; 2] {
        [
            self.a * x + self.c * y + self.tx,
            self.b * x + self.d * y + self.ty,
        ]
    }

    /// [`Affine2::apply`] of a point.
    pub fn apply_to(&self, p: [f32; 2]) -> [f32; 2] {
        self.apply(p[0], p[1])
    }

    /// The inverse (a singular transform inverts to the identity).
    pub fn inverse(&self) -> Affine2 {
        let det = self.a * self.d - self.b * self.c;
        if det == 0.0 {
            return Affine2::IDENTITY;
        }
        let i = 1.0 / det;
        Affine2 {
            a: self.d * i,
            b: -self.b * i,
            c: -self.c * i,
            d: self.a * i,
            tx: (self.c * self.ty - self.d * self.tx) * i,
            ty: (self.b * self.tx - self.a * self.ty) * i,
        }
    }

    /// The direction of the x axis, radians.
    fn angle(&self) -> f64 {
        atan2(f64::from(self.b), f64::from(self.a))
    }
}

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
struct Body<'a> {
    bones: &'a [Bone2d],
    slots: &'a [Slot2d],
    attachments: &'a [Attachment2d],
    iks: &'a [Ik2d],
    transforms: &'a [TransformConstraint2d],
    paths: &'a [PathConstraint2d],
    clips: &'a [Clip2d],
}

impl Armature2d {
    fn body(&self) -> Body<'_> {
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

fn parse_err(line: usize, what: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Parse, format!("line {line}: {what}"))
}

fn name_of(e: &Entry) -> Result<String> {
    e.values
        .first()
        .and_then(|v| v.as_word().or(v.as_str()))
        .map(str::to_string)
        .ok_or_else(|| parse_err(e.line, format!("`{}` needs a name", e.key)))
}

fn num(e: &Entry, i: usize) -> Result<f32> {
    let v = e
        .f32(i)
        .map_err(|_| parse_err(e.line, format!("`{}` needs a number at {}", e.key, i + 1)))?;
    if !v.is_finite() {
        return Err(parse_err(e.line, format!("`{}`: {v} is not finite", e.key)));
    }
    Ok(v)
}

fn word(e: &Entry, i: usize) -> Result<String> {
    e.values
        .get(i)
        .and_then(|v| v.as_word().or(v.as_str()))
        .map(str::to_string)
        .ok_or_else(|| parse_err(e.line, format!("`{}` needs a name at {}", e.key, i + 1)))
}

fn block(e: &Entry) -> Result<&Block> {
    e.block.as_ref().ok_or_else(|| {
        parse_err(
            e.line,
            format!("`{} <name> {{ ... }}` needs a block", e.key),
        )
    })
}

/// The ease written from value `from` on: nothing, `step`, or `curve` and
/// its control points.
fn ease_at(e: &Entry, from: usize) -> Result<Ease> {
    match e.values.get(from).map(Value::as_word) {
        None => Ok(Ease::Linear),
        Some(Some("step")) if e.values.len() == from + 1 => Ok(Ease::Step),
        Some(Some("curve")) => {
            let c = (from + 1..e.values.len())
                .map(|i| num(e, i))
                .collect::<Result<Vec<f32>>>()?;
            check_curve(&c).map_err(|w| parse_err(e.line, w))?;
            Ok(Ease::Bezier(c))
        }
        _ => Err(parse_err(
            e.line,
            format!(
                "`{}`: after its values a key takes `step` or `curve x1 y1 x2 y2 [x y x1 y1 x2 y2]...`",
                e.key
            ),
        )),
    }
}

/// Whether a Bézier's numbers make a curve over time: 4 + 6n of them,
/// every x in 0..1, the anchors in time order.
pub fn check_curve(c: &[f32]) -> std::result::Result<(), String> {
    if c.len() < 4 || !(c.len() - 4).is_multiple_of(6) {
        return Err(format!(
            "a curve is x1 y1 x2 y2, then x y x1 y1 x2 y2 per further segment ({} numbers)",
            c.len()
        ));
    }
    if c.iter().step_by(2).any(|x| !(0.0..=1.0).contains(x)) {
        return Err("a curve's x values (time) are within 0..1".into());
    }
    let anchors: Vec<f32> = (1..=(c.len() - 4) / 6).map(|k| c[6 * k - 2]).collect();
    if anchors.windows(2).any(|w| w[1] < w[0]) {
        return Err("a curve's anchors are in time order".into());
    }
    Ok(())
}

/// The value fraction of a Bézier ease at time fraction `x`: the segment
/// whose anchors span `x`, solved for its parameter by bisection.
pub fn bezier(c: &[f32], x: f32) -> f32 {
    let g = |i: usize| f64::from(c[i]);
    let segments = (c.len() - 4) / 6 + 1;
    let anchor = |k: usize| -> (f64, f64) {
        if k == 0 {
            (0.0, 0.0)
        } else if k == segments {
            (1.0, 1.0)
        } else {
            (g(6 * k - 2), g(6 * k - 1))
        }
    };
    let x = f64::from(x).clamp(0.0, 1.0);
    let mut k = 0;
    while k + 1 < segments && anchor(k + 1).0 < x {
        k += 1;
    }
    let (p0, p3) = (anchor(k), anchor(k + 1));
    let (p1, p2) = ((g(6 * k), g(6 * k + 1)), (g(6 * k + 2), g(6 * k + 3)));
    let at = |s: f64| -> (f64, f64) {
        let l = 1.0 - s;
        let (a, b, c, d) = (l * l * l, 3.0 * l * l * s, 3.0 * l * s * s, s * s * s);
        (
            a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
            a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
        )
    };
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..40 {
        let m = (lo + hi) * 0.5;
        if at(m).0 < x {
            lo = m;
        } else {
            hi = m;
        }
    }
    at((lo + hi) * 0.5).1 as f32
}

fn ease_text(e: &Ease) -> String {
    match e {
        Ease::Linear => String::new(),
        Ease::Step => " step".into(),
        Ease::Bezier(c) => {
            let mut s = String::from(" curve");
            for v in c {
                s += &format!(" {}", num_text(*v));
            }
            s
        }
    }
}

fn index(names: &[String], name: &str, what: &str, line: usize) -> Result<usize> {
    names
        .iter()
        .position(|n| n == name)
        .ok_or_else(|| parse_err(line, format!("no {what} `{name}`")))
}

/// The keys of an armature's body (the rig's top level, or an `armature` block).
const BODY_KEYS: [&str; 11] = [
    "bone",
    "slot",
    "region",
    "mesh",
    "box",
    "path",
    "child",
    "ik",
    "transform",
    "along",
    "clip",
];

/// Reads an armature's bones, slots, attachments, constraints and clips
/// from `body` (`armatures`: the rig's child armatures, by name).
fn parse_body(body: &[Entry], armatures: &[String]) -> Result<Armature2d> {
    let mut rig = Armature2d::default();
    // Bones, parent first.
    let mut bone_names: Vec<String> = Vec::new();
    for e in body.iter().filter(|e| e.key == "bone") {
        let name = name_of(e)?;
        if bone_names.contains(&name) {
            return Err(parse_err(
                e.line,
                format!("bone `{name}` is declared twice"),
            ));
        }
        let mut b = Bone2d {
            name: name.clone(),
            parent: None,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            skew: 0.0,
            scale: [1.0, 1.0],
            length: 0.0,
        };
        if let Some(blk) = &e.block {
            for c in &blk.entries {
                match c.key.as_str() {
                    "parent" => {
                        let p = word(c, 0)?;
                        b.parent =
                            Some(bone_names.iter().position(|n| *n == p).ok_or_else(|| {
                                parse_err(
                                    c.line,
                                    format!("bone {name}: parent `{p}` is not declared before it"),
                                )
                            })?);
                    }
                    "at" => (b.x, b.y) = (num(c, 0)?, num(c, 1)?),
                    "rotation" => b.rotation = num(c, 0)?,
                    "skew" => b.skew = num(c, 0)?,
                    "scale" => b.scale = [num(c, 0)?, num(c, 1)?],
                    "length" => b.length = num(c, 0)?,
                    k => {
                        return Err(parse_err(
                            c.line,
                            format!("unknown key `{k}` in bone {name}"),
                        ));
                    }
                }
            }
        }
        bone_names.push(name);
        rig.bones.push(b);
    }
    if rig.bones.is_empty() {
        return Err(Error::new(
            ErrorKind::Parse,
            "a 2D rig (and each armature) needs at least one bone",
        ));
    }
    // Slots, then attachments (which name their slot), then the slots' setup attachments.
    let mut slot_names: Vec<String> = Vec::new();
    let mut slot_attachment: Vec<Option<(String, usize)>> = Vec::new();
    for e in body.iter().filter(|e| e.key == "slot") {
        let name = name_of(e)?;
        if slot_names.contains(&name) {
            return Err(parse_err(
                e.line,
                format!("slot `{name}` is declared twice"),
            ));
        }
        let blk = block(e)?;
        let mut bone = None;
        let mut attachment = None;
        let mut color = [1.0; 4];
        let mut offset = [0.0; 4];
        for c in &blk.entries {
            match c.key.as_str() {
                "bone" => bone = Some(index(&bone_names, &word(c, 0)?, "bone", c.line)?),
                "attachment" => attachment = Some((word(c, 0)?, c.line)),
                "color" => color = [num(c, 0)?, num(c, 1)?, num(c, 2)?, num(c, 3)?],
                "offset" => offset = [num(c, 0)?, num(c, 1)?, num(c, 2)?, num(c, 3)?],
                k => {
                    return Err(parse_err(
                        c.line,
                        format!("unknown key `{k}` in slot {name}"),
                    ));
                }
            }
        }
        let bone = bone.ok_or_else(|| parse_err(e.line, format!("slot {name} needs `bone`")))?;
        rig.slots.push(Slot2d {
            name: name.clone(),
            bone,
            attachment: None,
            color,
            offset,
        });
        slot_attachment.push(attachment);
        slot_names.push(name);
    }
    let mut attachment_names: Vec<String> = Vec::new();
    for e in body
        .iter()
        .filter(|e| matches!(e.key.as_str(), "region" | "mesh" | "box" | "path" | "child"))
    {
        let name = name_of(e)?;
        if attachment_names.contains(&name) {
            return Err(parse_err(
                e.line,
                format!("attachment `{name}` is declared twice"),
            ));
        }
        let blk = block(e)?;
        let slot = blk
            .get("slot")
            .ok_or_else(|| parse_err(e.line, format!("{} {name} needs `slot`", e.key)))?;
        let slot_index = index(&slot_names, &word(slot, 0)?, "slot", slot.line)?;
        let unknown = |c: &Entry| {
            parse_err(
                c.line,
                format!("unknown key `{}` in {} {name}", c.key, e.key),
            )
        };
        let kind = match e.key.as_str() {
            "region" => {
                let mut rect = None;
                let mut at = [0.0; 2];
                let mut rotation = 0.0;
                let mut skew = 0.0;
                let mut size = None;
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "rect" => rect = Some([num(c, 0)?, num(c, 1)?, num(c, 2)?, num(c, 3)?]),
                        "at" => at = [num(c, 0)?, num(c, 1)?],
                        "rotation" => rotation = num(c, 0)?,
                        "skew" => skew = num(c, 0)?,
                        "size" => size = Some([num(c, 0)?, num(c, 1)?]),
                        _ => return Err(unknown(c)),
                    }
                }
                let rect = rect.ok_or_else(|| {
                    parse_err(e.line, format!("region {name} needs `rect x y w h`"))
                })?;
                AttachmentKind::Region {
                    rect,
                    at,
                    rotation,
                    skew,
                    size: size.unwrap_or([rect[2], rect[3]]),
                }
            }
            "box" => {
                let mut polygon = Vec::new();
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "polygon" => {
                            if c.values.len() < 6 || c.values.len() % 2 != 0 {
                                return Err(parse_err(
                                    c.line,
                                    format!("box {name}: polygon is at least three x y points"),
                                ));
                            }
                            for i in (0..c.values.len()).step_by(2) {
                                polygon.push([num(c, i)?, num(c, i + 1)?]);
                            }
                        }
                        _ => return Err(unknown(c)),
                    }
                }
                if polygon.is_empty() {
                    return Err(parse_err(
                        e.line,
                        format!("box {name} needs `polygon x y x y x y ...`"),
                    ));
                }
                AttachmentKind::Box { polygon }
            }
            "path" => {
                let slot_bone = rig.slots[slot_index].bone;
                let mut points = Vec::new();
                let mut closed = false;
                let mut constant = true;
                let mut lengths = Vec::new();
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "closed" => closed = flag(c)?,
                        "constant" => constant = flag(c)?,
                        "lengths" => {
                            lengths = (0..c.values.len())
                                .map(|i| num(c, i))
                                .collect::<Result<_>>()?
                        }
                        "vertex" => points.push(if c.values.len() == 2 {
                            vec![Influence {
                                bone: slot_bone,
                                x: num(c, 0)?,
                                y: num(c, 1)?,
                                weight: 1.0,
                            }]
                        } else {
                            weighted(c, 0, &bone_names, &format!("path {name}"))?
                        }),
                        _ => return Err(unknown(c)),
                    }
                }
                if points.len() < 6 || !points.len().is_multiple_of(3) {
                    return Err(parse_err(
                        e.line,
                        format!(
                            "path {name}: its vertices are three per knot (handle in, knot, handle out), at least two knots ({} vertices)",
                            points.len()
                        ),
                    ));
                }
                let curves = points.len() / 3 - usize::from(!closed);
                if !constant && lengths.len() != curves {
                    return Err(parse_err(
                        e.line,
                        format!(
                            "path {name}: `constant false` needs `lengths`, each of its {curves} curves' end along the path"
                        ),
                    ));
                }
                AttachmentKind::Path {
                    points,
                    closed,
                    constant,
                    lengths,
                }
            }
            "child" => {
                let mut armature = None;
                let mut clip = None;
                let mut at = [0.0; 2];
                let mut rotation = 0.0;
                let mut skew = 0.0;
                let mut scale = [1.0; 2];
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "armature" => {
                            armature = Some(index(armatures, &word(c, 0)?, "armature", c.line)?)
                        }
                        "clip" => clip = Some(word(c, 0)?),
                        "at" => at = [num(c, 0)?, num(c, 1)?],
                        "rotation" => rotation = num(c, 0)?,
                        "skew" => skew = num(c, 0)?,
                        "scale" => scale = [num(c, 0)?, num(c, 1)?],
                        _ => return Err(unknown(c)),
                    }
                }
                AttachmentKind::Child {
                    armature: armature.ok_or_else(|| {
                        parse_err(e.line, format!("child {name} needs `armature <name>`"))
                    })?,
                    clip,
                    at,
                    rotation,
                    skew,
                    scale,
                }
            }
            _ => {
                let slot_bone = rig.slots[slot_index].bone;
                let mut vertices = Vec::new();
                let mut triangles = Vec::new();
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "vertex" => {
                            let uv = [num(c, 0)?, num(c, 1)?];
                            let influences = if c.values.len() == 4 {
                                vec![Influence {
                                    bone: slot_bone,
                                    x: num(c, 2)?,
                                    y: num(c, 3)?,
                                    weight: 1.0,
                                }]
                            } else {
                                weighted(c, 2, &bone_names, &format!("mesh {name}"))?
                            };
                            vertices.push(MeshVertex2d { uv, influences });
                        }
                        "triangles" => {
                            for i in 0..c.values.len() {
                                let v = num(c, i)?;
                                if v < 0.0 || v.fract() != 0.0 {
                                    return Err(parse_err(
                                        c.line,
                                        format!("mesh {name}: triangles are vertex indices"),
                                    ));
                                }
                                triangles.push(v as u32);
                            }
                        }
                        _ => return Err(unknown(c)),
                    }
                }
                if triangles.is_empty() || triangles.len() % 3 != 0 {
                    return Err(parse_err(
                        e.line,
                        format!("mesh {name}: triangles are three indices each"),
                    ));
                }
                if let Some(t) = triangles.iter().find(|t| **t as usize >= vertices.len()) {
                    return Err(parse_err(
                        e.line,
                        format!("mesh {name}: index {t} of {} vertices", vertices.len()),
                    ));
                }
                AttachmentKind::Mesh {
                    vertices,
                    triangles,
                }
            }
        };
        rig.attachments.push(Attachment2d {
            name: name.clone(),
            slot: slot_index,
            kind,
        });
        attachment_names.push(name);
    }
    for (s, a) in slot_attachment.into_iter().enumerate() {
        if let Some((a, line)) = a
            && a != "none"
        {
            let i = index(&attachment_names, &a, "attachment", line)?;
            if rig.attachments[i].slot != s {
                return Err(parse_err(
                    line,
                    format!("attachment {a} belongs to another slot"),
                ));
            }
            rig.slots[s].attachment = Some(i);
        }
    }
    for e in body.iter().filter(|e| e.key == "ik") {
        let name = name_of(e)?;
        let blk = block(e)?;
        let mut ik = Ik2d {
            name: name.clone(),
            bones: Vec::new(),
            target: usize::MAX,
            counterclockwise: false,
            weight: 1.0,
        };
        for c in &blk.entries {
            match c.key.as_str() {
                "bones" => {
                    for i in 0..c.values.len() {
                        ik.bones
                            .push(index(&bone_names, &word(c, i)?, "bone", c.line)?);
                    }
                }
                "target" => ik.target = index(&bone_names, &word(c, 0)?, "bone", c.line)?,
                "bend" => ik.counterclockwise = bend(c, 0)?,
                "weight" => ik.weight = num(c, 0)?.clamp(0.0, 1.0),
                k => return Err(parse_err(c.line, format!("unknown key `{k}` in ik {name}"))),
            }
        }
        if ik.bones.is_empty() {
            return Err(parse_err(
                e.line,
                format!("ik {name}: `bones` names the chain, root first"),
            ));
        }
        if ik
            .bones
            .windows(2)
            .any(|w| rig.bones[w[1]].parent != Some(w[0]))
        {
            return Err(parse_err(
                e.line,
                format!("ik {name}: each bone of `bones` is the previous one's child"),
            ));
        }
        if ik.target == usize::MAX {
            return Err(parse_err(e.line, format!("ik {name} needs `target`")));
        }
        rig.iks.push(ik);
    }
    let bones_of = |c: &Entry, what: &str| -> Result<Vec<usize>> {
        let list = (0..c.values.len())
            .map(|i| index(&bone_names, &word(c, i)?, "bone", c.line))
            .collect::<Result<Vec<_>>>()?;
        if list.is_empty() {
            return Err(parse_err(
                c.line,
                format!("{what}: `bones` names at least one"),
            ));
        }
        Ok(list)
    };
    for e in body.iter().filter(|e| e.key == "transform") {
        let name = name_of(e)?;
        let mut t = TransformConstraint2d {
            name: name.clone(),
            bones: Vec::new(),
            target: usize::MAX,
            rotation: 0.0,
            at: [0.0; 2],
            scale: [0.0; 2],
            shear: 0.0,
            mix: [1.0, 1.0, 1.0, 1.0],
        };
        for c in &block(e)?.entries {
            match c.key.as_str() {
                "bones" => t.bones = bones_of(c, &format!("transform {name}"))?,
                "target" => t.target = index(&bone_names, &word(c, 0)?, "bone", c.line)?,
                "rotation" => t.rotation = num(c, 0)?,
                "at" => t.at = [num(c, 0)?, num(c, 1)?],
                "scale" => t.scale = [num(c, 0)?, num(c, 1)?],
                "shear" => t.shear = num(c, 0)?,
                "mix" => {
                    t.mix = [0, 1, 2, 3]
                        .map(|i| num(c, i).map(|v| v.clamp(0.0, 1.0)))
                        .into_iter()
                        .collect::<Result<Vec<_>>>()?
                        .try_into()
                        .expect("four")
                }
                k => {
                    return Err(parse_err(
                        c.line,
                        format!("unknown key `{k}` in transform {name}"),
                    ));
                }
            }
        }
        if t.bones.is_empty() || t.target == usize::MAX {
            return Err(parse_err(
                e.line,
                format!("transform {name} needs `bones` and `target`"),
            ));
        }
        if t.bones.contains(&t.target) {
            return Err(parse_err(
                e.line,
                format!("transform {name}: its target is not one of its bones"),
            ));
        }
        rig.transforms.push(t);
    }
    for e in body.iter().filter(|e| e.key == "along") {
        let name = name_of(e)?;
        let mut p = PathConstraint2d {
            name: name.clone(),
            path: usize::MAX,
            bones: Vec::new(),
            position: 0.0,
            position_unit: PathUnit::Percent,
            spacing: 0.0,
            spacing_mode: PathSpacing::Length,
            rotate: PathRotate::Tangent,
            rotation: 0.0,
            mix: [1.0, 1.0],
        };
        let mode = |c: &Entry, i: usize, words: &[&str]| -> Result<usize> {
            let w = word(c, i)?;
            words.iter().position(|x| *x == w).ok_or_else(|| {
                parse_err(
                    c.line,
                    format!(
                        "along {name}: `{}` takes {}, not `{w}`",
                        c.key,
                        words.join(" or ")
                    ),
                )
            })
        };
        for c in &block(e)?.entries {
            match c.key.as_str() {
                "path" => {
                    let a = index(&attachment_names, &word(c, 0)?, "attachment", c.line)?;
                    if !matches!(rig.attachments[a].kind, AttachmentKind::Path { .. }) {
                        return Err(parse_err(
                            c.line,
                            format!("along {name}: {} is not a path", attachment_names[a]),
                        ));
                    }
                    p.path = a;
                }
                "bones" => p.bones = bones_of(c, &format!("along {name}"))?,
                "position" => {
                    p.position = num(c, 0)?;
                    if c.values.len() > 1 {
                        p.position_unit = [PathUnit::Percent, PathUnit::Fixed]
                            [mode(c, 1, &["percent", "fixed"])?];
                    }
                }
                "spacing" => {
                    p.spacing = num(c, 0)?;
                    if c.values.len() > 1 {
                        p.spacing_mode = [
                            PathSpacing::Length,
                            PathSpacing::Fixed,
                            PathSpacing::Percent,
                        ][mode(c, 1, &["length", "fixed", "percent"])?];
                    }
                }
                "rotate" => {
                    p.rotate = [
                        PathRotate::Tangent,
                        PathRotate::Chain,
                        PathRotate::ChainScale,
                    ][mode(c, 0, &["tangent", "chain", "chainscale"])?];
                    if c.values.len() > 1 {
                        p.rotation = num(c, 1)?;
                    }
                }
                "mix" => p.mix = [num(c, 0)?.clamp(0.0, 1.0), num(c, 1)?.clamp(0.0, 1.0)],
                k => {
                    return Err(parse_err(
                        c.line,
                        format!("unknown key `{k}` in along {name}"),
                    ));
                }
            }
        }
        if p.bones.is_empty() || p.path == usize::MAX {
            return Err(parse_err(
                e.line,
                format!("along {name} needs `path` and `bones`"),
            ));
        }
        rig.paths.push(p);
    }
    let names = Names {
        bones: &bone_names,
        slots: &slot_names,
        attachments: &attachment_names,
    };
    for e in body.iter().filter(|e| e.key == "clip") {
        rig.clips.push(parse_clip(e, &rig, &names)?);
    }
    Ok(rig)
}

/// The top level's `skin` blocks: `swap <attachment> <attachment or none>`
/// lines, each pair in one slot.
fn parse_skins(body: &[Entry], rig: &Rig2dAsset) -> Result<Vec<Skin2d>> {
    let find = |n: &str, line: usize| -> Result<usize> {
        rig.attachments
            .iter()
            .position(|a| a.name == n)
            .ok_or_else(|| parse_err(line, format!("no attachment `{n}`")))
    };
    let mut out: Vec<Skin2d> = Vec::new();
    for e in body.iter().filter(|e| e.key == "skin") {
        let name = name_of(e)?;
        if name == "default" || out.iter().any(|k| k.name == name) {
            return Err(parse_err(
                e.line,
                format!(
                    "skin `{name}` is declared twice (the rig's own attachments are `default`)"
                ),
            ));
        }
        let mut skin = Skin2d {
            name,
            swaps: Vec::new(),
        };
        for c in &block(e)?.entries {
            if c.key != "swap" {
                return Err(parse_err(
                    c.line,
                    format!(
                        "unknown key `{}` in skin {} (swap <attachment> <attachment or none>)",
                        c.key, skin.name
                    ),
                ));
            }
            let from = find(&word(c, 0)?, c.line)?;
            let to = match word(c, 1)?.as_str() {
                "none" => None,
                t => Some(find(t, c.line)?),
            };
            if to.is_some_and(|t| rig.attachments[t].slot != rig.attachments[from].slot) {
                return Err(parse_err(
                    c.line,
                    format!("skin {}: a swap stays in its slot", skin.name),
                ));
            }
            skin.swaps.push((from, to));
        }
        out.push(skin);
    }
    Ok(out)
}

/// A vertex's influences from value `from` on: `bone x y weight` each,
/// the weights summing to 1.
fn weighted(c: &Entry, from: usize, bones: &[String], what: &str) -> Result<Vec<Influence>> {
    if c.values.len() < from + 4 || !(c.values.len() - from).is_multiple_of(4) {
        return Err(parse_err(
            c.line,
            format!(
                "{what}: a vertex is {}x y, or {}bone x y weight per influence",
                if from == 2 { "u v " } else { "" },
                if from == 2 { "u v then " } else { "" }
            ),
        ));
    }
    let mut out = Vec::new();
    for k in (from..c.values.len()).step_by(4) {
        out.push(Influence {
            bone: index(bones, &word(c, k)?, "bone", c.line)?,
            x: num(c, k + 1)?,
            y: num(c, k + 2)?,
            weight: num(c, k + 3)?,
        });
    }
    let total: f32 = out.iter().map(|i| i.weight).sum();
    if (total - 1.0).abs() > 1e-3 {
        return Err(parse_err(
            c.line,
            format!("{what}: the weights sum to {total}, not 1"),
        ));
    }
    Ok(out)
}

/// `true` or `false` at value 0.
fn flag(c: &Entry) -> Result<bool> {
    c.values
        .first()
        .and_then(Value::as_bool)
        .ok_or_else(|| parse_err(c.line, format!("`{}` is true or false", c.key)))
}

/// `clockwise` (false) or `counterclockwise` (true) at value `i`.
fn bend(c: &Entry, i: usize) -> Result<bool> {
    match word(c, i)?.as_str() {
        "clockwise" => Ok(false),
        "counterclockwise" => Ok(true),
        w => Err(parse_err(
            c.line,
            format!("bend is clockwise or counterclockwise, not `{w}`"),
        )),
    }
}

/// Writes an armature's bones, slots, attachments, constraints and clips.
fn body_text(rig: Body<'_>, armatures: &[Armature2d]) -> String {
    let n = num_text;
    let mut s = String::new();
    for b in rig.bones {
        s += &format!("bone {} {{\n", quote(&b.name));
        if let Some(p) = b.parent {
            s += &format!("  parent {}\n", quote(&rig.bones[p].name));
        }
        s += &format!(
            "  at {} {}\n  rotation {}\n  skew {}\n",
            n(b.x),
            n(b.y),
            n(b.rotation),
            n(b.skew)
        );
        s += &format!(
            "  scale {} {}\n  length {}\n}}\n",
            n(b.scale[0]),
            n(b.scale[1]),
            n(b.length)
        );
    }
    for sl in rig.slots {
        s += &format!(
            "slot {} {{\n  bone {}\n",
            quote(&sl.name),
            quote(&rig.bones[sl.bone].name)
        );
        if let Some(a) = sl.attachment {
            s += &format!("  attachment {}\n", quote(&rig.attachments[a].name));
        }
        let [r, g, b, a] = sl.color;
        s += &format!("  color {} {} {} {}\n", n(r), n(g), n(b), n(a));
        let [r, g, b, a] = sl.offset;
        s += &format!("  offset {} {} {} {}\n}}\n", n(r), n(g), n(b), n(a));
    }
    for a in rig.attachments {
        let slot = quote(&rig.slots[a.slot].name);
        match &a.kind {
            AttachmentKind::Region {
                rect,
                at,
                rotation,
                skew,
                size,
            } => {
                s += &format!("region {} {{\n  slot {slot}\n", quote(&a.name));
                s += &format!(
                    "  rect {} {} {} {}\n",
                    n(rect[0]),
                    n(rect[1]),
                    n(rect[2]),
                    n(rect[3])
                );
                s += &format!(
                    "  at {} {}\n  rotation {}\n  skew {}\n",
                    n(at[0]),
                    n(at[1]),
                    n(*rotation),
                    n(*skew)
                );
                s += &format!("  size {} {}\n}}\n", n(size[0]), n(size[1]));
            }
            AttachmentKind::Mesh {
                vertices,
                triangles,
            } => {
                s += &format!("mesh {} {{\n  slot {slot}\n", quote(&a.name));
                let bone = rig.slots[a.slot].bone;
                for v in vertices {
                    s += &format!("  vertex {} {}", n(v.uv[0]), n(v.uv[1]));
                    match v.influences.as_slice() {
                        [i] if i.bone == bone && i.weight == 1.0 => {
                            s += &format!(" {} {}", n(i.x), n(i.y))
                        }
                        all => {
                            for i in all {
                                s += &format!(
                                    " {} {} {} {}",
                                    quote(&rig.bones[i.bone].name),
                                    n(i.x),
                                    n(i.y),
                                    n(i.weight)
                                );
                            }
                        }
                    }
                    s += "\n";
                }
                s += "  triangles";
                for t in triangles {
                    s += &format!(" {t}");
                }
                s += "\n}\n";
            }
            AttachmentKind::Path {
                points,
                closed,
                constant,
                lengths,
            } => {
                s += &format!(
                    "path {} {{\n  slot {slot}\n  closed {closed}\n  constant {constant}\n",
                    quote(&a.name)
                );
                if !lengths.is_empty() {
                    s += "  lengths";
                    for l in lengths {
                        s += &format!(" {}", n(*l));
                    }
                    s += "\n";
                }
                let bone = rig.slots[a.slot].bone;
                for p in points {
                    s += "  vertex";
                    match p.as_slice() {
                        [i] if i.bone == bone && i.weight == 1.0 => {
                            s += &format!(" {} {}", n(i.x), n(i.y))
                        }
                        all => {
                            for i in all {
                                s += &format!(
                                    " {} {} {} {}",
                                    quote(&rig.bones[i.bone].name),
                                    n(i.x),
                                    n(i.y),
                                    n(i.weight)
                                );
                            }
                        }
                    }
                    s += "\n";
                }
                s += "}\n";
            }
            AttachmentKind::Box { polygon } => {
                s += &format!("box {} {{\n  slot {slot}\n  polygon", quote(&a.name));
                for p in polygon {
                    s += &format!(" {} {}", n(p[0]), n(p[1]));
                }
                s += "\n}\n";
            }
            AttachmentKind::Child {
                armature,
                clip,
                at,
                rotation,
                skew,
                scale,
            } => {
                s += &format!(
                    "child {} {{\n  slot {slot}\n  armature {}\n",
                    quote(&a.name),
                    quote(&armatures[*armature].name)
                );
                if let Some(c) = clip {
                    s += &format!("  clip {}\n", quote(c));
                }
                s += &format!(
                    "  at {} {}\n  rotation {}\n  skew {}\n  scale {} {}\n}}\n",
                    n(at[0]),
                    n(at[1]),
                    n(*rotation),
                    n(*skew),
                    n(scale[0]),
                    n(scale[1])
                );
            }
        }
    }
    let bend_text = |ccw: bool| if ccw { "counterclockwise" } else { "clockwise" };
    for ik in rig.iks {
        s += &format!("ik {} {{\n  bones", quote(&ik.name));
        for b in &ik.bones {
            s += &format!(" {}", quote(&rig.bones[*b].name));
        }
        s += &format!(
            "\n  target {}\n  bend {}\n  weight {}\n}}\n",
            quote(&rig.bones[ik.target].name),
            bend_text(ik.counterclockwise),
            n(ik.weight)
        );
    }
    let bones_text = |list: &[usize]| -> String {
        list.iter()
            .map(|b| format!(" {}", quote(&rig.bones[*b].name)))
            .collect()
    };
    for t in rig.transforms {
        s += &format!(
            "transform {} {{\n  bones{}\n  target {}\n",
            quote(&t.name),
            bones_text(&t.bones),
            quote(&rig.bones[t.target].name)
        );
        s += &format!(
            "  rotation {}\n  at {} {}\n  scale {} {}\n  shear {}\n",
            n(t.rotation),
            n(t.at[0]),
            n(t.at[1]),
            n(t.scale[0]),
            n(t.scale[1]),
            n(t.shear)
        );
        let [r, m, k, h] = t.mix;
        s += &format!("  mix {} {} {} {}\n}}\n", n(r), n(m), n(k), n(h));
    }
    for p in rig.paths {
        s += &format!(
            "along {} {{\n  path {}\n  bones{}\n",
            quote(&p.name),
            quote(&rig.attachments[p.path].name),
            bones_text(&p.bones)
        );
        let unit = match p.position_unit {
            PathUnit::Percent => "percent",
            PathUnit::Fixed => "fixed",
        };
        let spacing = match p.spacing_mode {
            PathSpacing::Length => "length",
            PathSpacing::Fixed => "fixed",
            PathSpacing::Percent => "percent",
        };
        let rotate = match p.rotate {
            PathRotate::Tangent => "tangent",
            PathRotate::Chain => "chain",
            PathRotate::ChainScale => "chainscale",
        };
        s += &format!(
            "  position {} {unit}\n  spacing {} {spacing}\n  rotate {rotate} {}\n  mix {} {}\n}}\n",
            n(p.position),
            n(p.spacing),
            n(p.rotation),
            n(p.mix[0]),
            n(p.mix[1])
        );
    }
    for c in rig.clips {
        s += &format!(
            "clip {} {{\n  duration {}\n  loop {}\n",
            quote(&c.name),
            n(c.duration),
            c.looping
        );
        for t in &c.bones {
            s += &format!("  bone {} {{\n", quote(&rig.bones[t.bone].name));
            for k in &t.rotate {
                let turn = match k.value.turn {
                    Turn::Shorter => String::new(),
                    Turn::Clockwise(t) => format!(" clockwise {t}"),
                    Turn::Counterclockwise(t) => format!(" counterclockwise {t}"),
                };
                s += &format!(
                    "    rotate {} {}{turn}{}\n",
                    n(k.time),
                    n(k.value.degrees),
                    ease_text(&k.ease)
                );
            }
            for k in &t.skew {
                s += &format!(
                    "    skew {} {}{}\n",
                    n(k.time),
                    n(k.value),
                    ease_text(&k.ease)
                );
            }
            for k in &t.translate {
                s += &format!(
                    "    translate {} {} {}{}\n",
                    n(k.time),
                    n(k.value[0]),
                    n(k.value[1]),
                    ease_text(&k.ease)
                );
            }
            for k in &t.scale {
                s += &format!(
                    "    scale {} {} {}{}\n",
                    n(k.time),
                    n(k.value[0]),
                    n(k.value[1]),
                    ease_text(&k.ease)
                );
            }
            s += "  }\n";
        }
        for t in &c.slots {
            s += &format!("  slot {} {{\n", quote(&rig.slots[t.slot].name));
            for (time, a) in &t.attachment {
                let a = a.map_or("none".to_string(), |a| quote(&rig.attachments[a].name));
                s += &format!("    attachment {} {a}\n", n(*time));
            }
            for (key, keys) in [("color", &t.color), ("offset", &t.offset)] {
                for k in keys {
                    let [r, g, b, a] = k.value;
                    s += &format!(
                        "    {key} {} {} {} {} {}{}\n",
                        n(k.time),
                        n(r),
                        n(g),
                        n(b),
                        n(a),
                        ease_text(&k.ease)
                    );
                }
            }
            s += "  }\n";
        }
        for t in &c.deforms {
            s += &format!(
                "  deform {} {{\n",
                quote(&rig.attachments[t.attachment].name)
            );
            for k in &t.keys {
                s += &format!("    key {}", n(k.time));
                for v in &k.value {
                    s += &format!(" {}", n(*v));
                }
                s += &format!("{}\n", ease_text(&k.ease));
            }
            s += "  }\n";
        }
        for (time, order) in &c.order {
            s += &format!("  order {}", n(*time));
            for o in order {
                s += &format!(" {}", quote(&rig.slots[*o].name));
            }
            s += "\n";
        }
        for t in &c.iks {
            s += &format!("  ik {} {{\n", quote(&rig.iks[t.ik].name));
            for k in &t.keys {
                s += &format!(
                    "    key {} {} {}{}\n",
                    n(k.time),
                    n(k.value.weight),
                    bend_text(k.value.counterclockwise),
                    ease_text(&k.ease)
                );
            }
            s += "  }\n";
        }
        for (t, name) in &c.events {
            s += &format!("  event {} {}\n", n(*t), quote(name));
        }
        s += "}\n";
    }
    s
}

impl Rig2dAsset {
    fn body(&self) -> Body<'_> {
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

    /// Reads a `type rig2d` text (the grammar is the module's docs).
    pub fn parse(text: &str) -> Result<Rig2dAsset> {
        let doc = Document::parse(text)?;
        if doc.ty != RIG2D_TYPE {
            return Err(Error::new(
                ErrorKind::Parse,
                format!("a 2D rig is `type rig2d`, not `{}`", doc.ty),
            ));
        }
        let mut texture = String::new();
        let mut size = [0, 0];
        let mut filter = SheetFilter::Linear;
        let mut unit = 1.0;
        let body = &doc.body.entries;
        for e in body {
            match e.key.as_str() {
                "texture" => {
                    texture = e
                        .string(0)
                        .map_err(|_| parse_err(e.line, "texture \"path\""))?
                        .to_string()
                }
                "size" => size = [num(e, 0)? as u32, num(e, 1)? as u32],
                "filter" => {
                    filter = match e.word(0).unwrap_or("") {
                        "nearest" => SheetFilter::Nearest,
                        "linear" => SheetFilter::Linear,
                        w => {
                            return Err(parse_err(
                                e.line,
                                format!("filter is nearest or linear, not `{w}`"),
                            ));
                        }
                    }
                }
                "unit" => {
                    unit = num(e, 0)?;
                    if unit <= 0.0 {
                        return Err(parse_err(e.line, "unit is above 0 (pixels per world unit)"));
                    }
                }
                "armature" | "dragonbones" | "atlas" | "skin" => {}
                k if BODY_KEYS.contains(&k) => {}
                k => return Err(parse_err(e.line, format!("unknown key `{k}` in a 2D rig"))),
            }
        }
        if texture.is_empty() {
            return Err(Error::new(
                ErrorKind::Parse,
                "a 2D rig needs `texture \"path\"`",
            ));
        }
        // Child armatures: names first, so any body can show any of them.
        let blocks: Vec<&Entry> = body.iter().filter(|e| e.key == "armature").collect();
        let mut names: Vec<String> = Vec::new();
        for e in &blocks {
            let name = name_of(e)?;
            if names.contains(&name) {
                return Err(parse_err(
                    e.line,
                    format!("armature `{name}` is declared twice"),
                ));
            }
            names.push(name);
        }
        let mut armatures = Vec::new();
        for (e, name) in blocks.iter().zip(&names) {
            let entries = &block(e)?.entries;
            if let Some(c) = entries
                .iter()
                .find(|c| !BODY_KEYS.contains(&c.key.as_str()))
            {
                return Err(parse_err(
                    c.line,
                    format!("unknown key `{}` in armature {name}", c.key),
                ));
            }
            let mut a = parse_body(entries, &names).map_err(|err| {
                Error::new(
                    ErrorKind::Parse,
                    format!("armature {name}: {}", err.reason()),
                )
            })?;
            a.name = name.clone();
            armatures.push(a);
        }
        let top = parse_body(body, &names)?;
        let rig = Rig2dAsset {
            texture,
            size,
            filter,
            unit,
            bones: top.bones,
            slots: top.slots,
            attachments: top.attachments,
            iks: top.iks,
            transforms: top.transforms,
            paths: top.paths,
            clips: top.clips,
            skins: Vec::new(),
            armatures,
        };
        let mut rig = rig;
        rig.skins = parse_skins(body, &rig)?;
        rig.check_children()?;
        Ok(rig)
    }

    /// Each child attachment names a clip its armature has, and no
    /// armature shows itself (directly or through another).
    fn check_children(&self) -> Result<()> {
        let shows = |b: Body<'_>| -> Vec<(usize, Option<String>, String)> {
            b.attachments
                .iter()
                .filter_map(|a| match &a.kind {
                    AttachmentKind::Child { armature, clip, .. } => {
                        Some((*armature, clip.clone(), a.name.clone()))
                    }
                    _ => None,
                })
                .collect()
        };
        let edges: Vec<Vec<usize>> = self
            .armatures
            .iter()
            .map(|a| shows(a.body()).into_iter().map(|s| s.0).collect())
            .collect();
        let all = std::iter::once(shows(self.body()))
            .chain(self.armatures.iter().map(|a| shows(a.body())));
        for list in all {
            for (a, clip, name) in list {
                if let Some(c) = clip
                    && !self.armatures[a].clips.iter().any(|x| x.name == c)
                {
                    return Err(Error::new(
                        ErrorKind::Parse,
                        format!(
                            "child {name}: armature {} has no clip `{c}`",
                            self.armatures[a].name
                        ),
                    ));
                }
            }
        }
        // Depth-first from each armature: reaching it again is a cycle.
        for start in 0..self.armatures.len() {
            let mut stack = edges[start].clone();
            let mut seen = vec![false; self.armatures.len()];
            while let Some(a) = stack.pop() {
                if a == start {
                    return Err(Error::new(
                        ErrorKind::Parse,
                        format!(
                            "armature {} shows itself through its child attachments",
                            self.armatures[start].name
                        ),
                    ));
                }
                if !std::mem::replace(&mut seen[a], true) {
                    stack.extend(&edges[a]);
                }
            }
        }
        Ok(())
    }

    /// The canonical text: every value spelled out. Parses back equal.
    pub fn to_text(&self) -> String {
        let mut s = String::from("type rig2d\nversion 1\n---\n");
        s += &format!("texture {}\n", quote(&self.texture));
        s += &format!("size {} {}\n", self.size[0], self.size[1]);
        s += &format!(
            "filter {}\n",
            if self.filter == SheetFilter::Nearest {
                "nearest"
            } else {
                "linear"
            }
        );
        s += &format!("unit {}\n", num_text(self.unit));
        s += &body_text(self.body(), &self.armatures);
        for k in &self.skins {
            s += &format!("skin {} {{\n", quote(&k.name));
            for (from, to) in &k.swaps {
                let to = to.map_or("none".to_string(), |t| quote(&self.attachments[t].name));
                s += &format!("  swap {} {to}\n", quote(&self.attachments[*from].name));
            }
            s += "}\n";
        }
        for a in &self.armatures {
            s += &format!("armature {} {{\n", quote(&a.name));
            for line in body_text(a.body(), &self.armatures).lines() {
                s += &format!("  {line}\n");
            }
            s += "}\n";
        }
        s
    }

    /// The index of a clip by name.
    pub fn clip(&self, name: &str) -> Option<u32> {
        self.clips
            .iter()
            .position(|c| c.name == name)
            .map(|i| i as u32)
    }

    pub fn bone(&self, name: &str) -> Option<usize> {
        self.bones.iter().position(|b| b.name == name)
    }

    /// The pose at `time` of `clip` (`None`: the setup pose), with IK
    /// `targets` (per constraint: a target in rig pixels and a weight that
    /// replace the target bone's origin and multiply the constraint's weight).
    pub fn pose(
        &self,
        clip: Option<&Clip2d>,
        time: f32,
        targets: &[(usize, [f32; 2], f32)],
    ) -> Pose2d {
        self.pose_of(self.body(), clip, time, targets, &[])
    }

    /// [`Rig2dAsset::pose`] wearing skin `skin` (0 the default, `i + 1`
    /// [`Rig2dAsset::skins`]`[i]`; one past them is the default).
    pub fn pose_in_skin(
        &self,
        clip: Option<&Clip2d>,
        time: f32,
        targets: &[(usize, [f32; 2], f32)],
        skin: u32,
    ) -> Pose2d {
        let swaps = (skin as usize)
            .checked_sub(1)
            .and_then(|i| self.skins.get(i))
            .map_or(&[][..], |k| k.swaps.as_slice());
        self.pose_of(self.body(), clip, time, targets, swaps)
    }

    fn pose_of(
        &self,
        body: Body<'_>,
        clip: Option<&Clip2d>,
        time: f32,
        targets: &[(usize, [f32; 2], f32)],
        swaps: &[(usize, Option<usize>)],
    ) -> Pose2d {
        // x, y, rotation, scale x, scale y, skew.
        let mut local: Vec<[f32; 6]> = body
            .bones
            .iter()
            .map(|b| [b.x, b.y, b.rotation, b.scale[0], b.scale[1], b.skew])
            .collect();
        let mut slots: Vec<Option<usize>> = body.slots.iter().map(|s| s.attachment).collect();
        let mut colors: Vec<[f32; 4]> = body.slots.iter().map(|s| s.color).collect();
        let mut offsets: Vec<[f32; 4]> = body.slots.iter().map(|s| s.offset).collect();
        let mut order: Vec<usize> = (0..body.slots.len()).collect();
        let mut deforms: Vec<(usize, Vec<f32>)> = Vec::new();
        let mut ik_keys: Vec<(f32, bool)> = body
            .iks
            .iter()
            .map(|c| (c.weight, c.counterclockwise))
            .collect();
        let lerp4 =
            |a: &[f32; 4], b: &[f32; 4], f: f32| [0, 1, 2, 3].map(|i| a[i] + (b[i] - a[i]) * f);
        if let Some(c) = clip {
            for t in &c.bones {
                let l = &mut local[t.bone];
                if let Some(r) = sample(&t.rotate, time, |a, b, f| Rotation {
                    degrees: a.degrees + turn_delta(a.degrees, b.degrees, a.turn) * f,
                    turn: a.turn,
                }) {
                    l[2] += r.degrees;
                }
                if let Some(k) = sample(&t.skew, time, |a, b, f| a + (b - a) * f) {
                    l[5] += k;
                }
                if let Some([x, y]) = sample(&t.translate, time, lerp2) {
                    l[0] += x;
                    l[1] += y;
                }
                if let Some([x, y]) = sample(&t.scale, time, lerp2) {
                    l[3] *= x;
                    l[4] *= y;
                }
            }
            for t in &c.slots {
                if let Some((_, a)) = t.attachment.iter().rev().find(|(k, _)| *k <= time) {
                    slots[t.slot] = *a;
                }
                if let Some(v) = sample(&t.color, time, lerp4) {
                    colors[t.slot] = v;
                }
                if let Some(v) = sample(&t.offset, time, lerp4) {
                    offsets[t.slot] = v;
                }
            }
            for t in &c.deforms {
                if let Some(v) = sample(&t.keys, time, |a: &Vec<f32>, b: &Vec<f32>, f| {
                    a.iter().zip(b).map(|(x, y)| x + (y - x) * f).collect()
                }) {
                    deforms.push((t.attachment, v));
                }
            }
            if let Some((_, o)) = c.order.iter().rev().find(|(k, _)| *k <= time)
                && !o.is_empty()
            {
                order = o.clone();
            }
            for t in &c.iks {
                if let Some(k) = sample(&t.keys, time, |a, b, f| IkKey {
                    weight: a.weight + (b.weight - a.weight) * f,
                    counterclockwise: a.counterclockwise,
                }) {
                    ik_keys[t.ik] = (k.weight.clamp(0.0, 1.0), k.counterclockwise);
                }
            }
        }
        // The skin: what each slot shows in place of the default's.
        for a in slots.iter_mut() {
            if let Some(&(_, to)) = a.and_then(|x| swaps.iter().find(|(f, _)| *f == x)) {
                *a = to;
            }
        }
        let mut world = world_of(body.bones, &local);
        let mut ik = Vec::with_capacity(body.iks.len());
        for (i, c) in body.iks.iter().enumerate() {
            let (rig_weight, counterclockwise) = ik_keys[i];
            let (target, weight) = match targets.iter().find(|t| t.0 == i) {
                Some(&(_, t, w)) => (t, rig_weight * w.clamp(0.0, 1.0)),
                None => {
                    let w = world[c.target];
                    ([w.tx, w.ty], rig_weight)
                }
            };
            let t = [f64::from(target[0]), f64::from(target[1])];
            let w = f64::from(weight);
            let tip_of = |world: &[Affine2], b: usize| world[b].apply(body.bones[b].length, 0.0);
            match c.bones.as_slice() {
                [b] => {
                    let m = world[*b];
                    let want = atan2(t[1] - f64::from(m.ty), t[0] - f64::from(m.tx));
                    local[*b][2] += (wrap(want - m.angle()) * w / DEG) as f32;
                }
                [p, ch] => {
                    let o1 = [f64::from(world[*p].tx), f64::from(world[*p].ty)];
                    let o2 = [f64::from(world[*ch].tx), f64::from(world[*ch].ty)];
                    let tip = tip_of(&world, *ch).map(f64::from);
                    let l1 = sq(o2[0] - o1[0], o2[1] - o1[1]);
                    let l2 = sq(tip[0] - o2[0], tip[1] - o2[1]);
                    let (dx, dy) = (t[0] - o1[0], t[1] - o1[1]);
                    let d = (dx * dx + dy * dy)
                        .sqrt()
                        .clamp((l1 - l2).abs() + 1e-6, (l1 + l2 - 1e-6).max(1e-6));
                    let cos_a =
                        ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d).max(1e-12)).clamp(-1.0, 1.0);
                    let a = atan2((1.0 - cos_a * cos_a).max(0.0).sqrt(), cos_a);
                    let side = if counterclockwise { 1.0 } else { -1.0 };
                    let want = atan2(dy, dx) + side * a;
                    let now = atan2(o2[1] - o1[1], o2[0] - o1[0]);
                    local[*p][2] += (wrap(want - now) * w / DEG) as f32;
                    world = world_of(body.bones, &local);
                    let o2 = [f64::from(world[*ch].tx), f64::from(world[*ch].ty)];
                    let tip = tip_of(&world, *ch).map(f64::from);
                    let want = atan2(t[1] - o2[1], t[0] - o2[0]);
                    let now = atan2(tip[1] - o2[1], tip[0] - o2[0]);
                    local[*ch][2] += (wrap(want - now) * w / DEG) as f32;
                }
                chain => fabrik(body.bones, &mut local, chain, t, w, counterclockwise),
            }
            world = world_of(body.bones, &local);
            let last = *c.bones.last().expect("a chain has a bone");
            let reach = if c.bones.len() == 1 {
                // One bone points: its error is how far off its direction the target lies.
                let m = world[last];
                let dir = m.angle();
                let (s, co) = sincos(dir);
                let (vx, vy) = (t[0] - f64::from(m.tx), t[1] - f64::from(m.ty));
                (vx * s - vy * co).abs() as f32
            } else {
                let tip = tip_of(&world, last);
                sq(f64::from(tip[0] - target[0]), f64::from(tip[1] - target[1])) as f32
            };
            ik.push(reach);
        }
        // Transform, then path constraints: world transforms that hold, the
        // bones under them following.
        if !body.transforms.is_empty() || !body.paths.is_empty() {
            let mut fixed: Vec<Option<Affine2>> = vec![None; body.bones.len()];
            for c in body.transforms {
                transform_constraint(body.bones, &local, &mut fixed, &mut world, c);
            }
            for c in body.paths {
                let slot = body.attachments[c.path].slot;
                if slots[slot] != Some(c.path) {
                    continue;
                }
                let deform = deforms
                    .iter()
                    .find(|d| d.0 == c.path)
                    .map(|d| d.1.as_slice());
                path_constraint(body, &local, &mut fixed, &mut world, c, deform);
            }
        }
        // Each slot showing a child armature: its pose at this time.
        let mut children = Vec::new();
        for (s, a) in slots.iter().enumerate() {
            let Some(a) = a else { continue };
            let AttachmentKind::Child {
                armature,
                clip: own,
                ..
            } = &body.attachments[*a].kind
            else {
                continue;
            };
            let arm = &self.armatures[*armature];
            let by_name = |n: &str| arm.clips.iter().find(|c| c.name == n);
            let played = clip
                .and_then(|c| by_name(&c.name))
                .or_else(|| own.as_deref().and_then(by_name))
                .or(arm.clips.first());
            let t = match played {
                Some(c) if c.looping => time - c.duration * (time / c.duration).floor(),
                Some(c) => time.clamp(0.0, c.duration),
                None => 0.0,
            };
            children.push((s, self.pose_of(arm.body(), played, t, &[], &[])));
        }
        Pose2d {
            world,
            slots,
            colors,
            offsets,
            order,
            deforms,
            ik_error: ik,
            children,
        }
    }

    /// The triangles a pose draws, in its draw order (a child armature at
    /// its slot's place): corners in world units of the rig's space
    /// (pixels over `unit`), uvs (0..1, v down), colour multipliers and
    /// offsets, with triangle indices.
    pub fn triangles(&self, pose: &Pose2d) -> (Vec<RigVertex>, Vec<u32>) {
        let mut out = Vec::new();
        let mut indices = Vec::new();
        self.draw(
            self.body(),
            pose,
            &Affine2::IDENTITY,
            ([1.0; 4], [0.0; 4]),
            &mut out,
            &mut indices,
        );
        (out, indices)
    }

    fn draw(
        &self,
        body: Body<'_>,
        pose: &Pose2d,
        place: &Affine2,
        tint: ([f32; 4], [f32; 4]),
        out: &mut Vec<RigVertex>,
        indices: &mut Vec<u32>,
    ) {
        let (w, h) = (self.size[0].max(1) as f32, self.size[1].max(1) as f32);
        let inv = 1.0 / self.unit;
        for &s in &pose.order {
            let slot = &body.slots[s];
            let Some(a) = pose.slots[s] else { continue };
            // The parent's colour map applied to this slot's: m (c t + o) + p.
            let (m, p) = tint;
            let color = [0, 1, 2, 3].map(|i| m[i] * pose.colors[s][i]);
            let offset = [0, 1, 2, 3].map(|i| m[i] * pose.offsets[s][i] + p[i]);
            let bone = place.mul(&pose.world[slot.bone]);
            let base = out.len() as u32;
            match &body.attachments[a].kind {
                AttachmentKind::Region {
                    rect,
                    at,
                    rotation,
                    skew,
                    size,
                } => {
                    let m = bone.mul(&Affine2::trsk(at[0], at[1], *rotation, *skew, 1.0, 1.0));
                    let (hx, hy) = (size[0] / 2.0, size[1] / 2.0);
                    let (u0, u1) = (rect[0] / w, (rect[0] + rect[2]) / w);
                    let (v_top, v_bottom) = (rect[1] / h, (rect[1] + rect[3]) / h);
                    for (x, y, u, v) in [
                        (-hx, -hy, u0, v_bottom),
                        (hx, -hy, u1, v_bottom),
                        (hx, hy, u1, v_top),
                        (-hx, hy, u0, v_top),
                    ] {
                        let p = m.apply(x, y);
                        out.push(RigVertex {
                            position: [p[0] * inv, p[1] * inv],
                            uv: [u, v],
                            color,
                            offset,
                        });
                    }
                    indices.extend([0, 1, 2, 2, 3, 0].map(|i| base + i));
                }
                AttachmentKind::Mesh {
                    vertices,
                    triangles,
                } => {
                    let deform = pose
                        .deforms
                        .iter()
                        .find(|d| d.0 == a)
                        .map(|d| d.1.as_slice());
                    let mut k = 0usize;
                    for v in vertices {
                        let mut p = [0.0f32; 2];
                        for i in &v.influences {
                            let (dx, dy) = deform.map_or((0.0, 0.0), |d| {
                                (
                                    d.get(2 * k).copied().unwrap_or(0.0),
                                    d.get(2 * k + 1).copied().unwrap_or(0.0),
                                )
                            });
                            k += 1;
                            let q = place.apply_to(pose.world[i.bone].apply(i.x + dx, i.y + dy));
                            p[0] += q[0] * i.weight;
                            p[1] += q[1] * i.weight;
                        }
                        out.push(RigVertex {
                            position: [p[0] * inv, p[1] * inv],
                            uv: [v.uv[0] / w, v.uv[1] / h],
                            color,
                            offset,
                        });
                    }
                    indices.extend(triangles.iter().map(|t| base + t));
                }
                AttachmentKind::Box { .. } | AttachmentKind::Path { .. } => {}
                AttachmentKind::Child {
                    armature,
                    at,
                    rotation,
                    skew,
                    scale,
                    ..
                } => {
                    if let Some((_, child)) = pose.children.iter().find(|c| c.0 == s) {
                        let m = bone.mul(&Affine2::trsk(
                            at[0], at[1], *rotation, *skew, scale[0], scale[1],
                        ));
                        self.draw(
                            self.armatures[*armature].body(),
                            child,
                            &m,
                            (color, offset),
                            out,
                            indices,
                        );
                    }
                }
            }
        }
    }

    /// Whether any attachment (of the rig or a child armature) is a box.
    pub fn has_boxes(&self) -> bool {
        let any = |a: &[Attachment2d]| {
            a.iter()
                .any(|a| matches!(a.kind, AttachmentKind::Box { .. }))
        };
        any(&self.attachments) || self.armatures.iter().any(|x| any(&x.attachments))
    }

    /// The bounding boxes a pose shows, in its draw order (a child
    /// armature's inside its slot, named `slot/child-slot`): each polygon
    /// in the rig's pixels.
    pub fn boxes(&self, pose: &Pose2d) -> Vec<BoundingBox2d> {
        let mut out = Vec::new();
        self.collect_boxes(self.body(), pose, &Affine2::IDENTITY, "", &mut out);
        out
    }

    fn collect_boxes(
        &self,
        body: Body<'_>,
        pose: &Pose2d,
        place: &Affine2,
        prefix: &str,
        out: &mut Vec<BoundingBox2d>,
    ) {
        for &s in &pose.order {
            let Some(a) = pose.slots[s] else { continue };
            let slot = &body.slots[s];
            let bone = place.mul(&pose.world[slot.bone]);
            let attachment = &body.attachments[a];
            match &attachment.kind {
                AttachmentKind::Box { polygon } => out.push(BoundingBox2d {
                    slot: format!("{prefix}{}", slot.name),
                    name: attachment.name.clone(),
                    points: polygon.iter().map(|p| bone.apply_to(*p)).collect(),
                }),
                AttachmentKind::Child {
                    armature,
                    at,
                    rotation,
                    skew,
                    scale,
                    ..
                } => {
                    if let Some((_, child)) = pose.children.iter().find(|c| c.0 == s) {
                        let m = bone.mul(&Affine2::trsk(
                            at[0], at[1], *rotation, *skew, scale[0], scale[1],
                        ));
                        self.collect_boxes(
                            self.armatures[*armature].body(),
                            child,
                            &m,
                            &format!("{prefix}{}/", slot.name),
                            out,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

/// FABRIK's passes over a chain of three or more bones: at most this many.
const FABRIK_PASSES: usize = 64;
/// ... stopping when the tip is this close to the target (pixels).
const FABRIK_TOLERANCE: f64 = 1e-4;

/// Turns `chain` (three or more bones, each the previous one's child) so
/// the last one's tip reaches `t`, by FABRIK over the joints (each bone's
/// origin, then the tip): from the animated pose, its interior joints
/// first moved to the asked side of the line from the root to the target
/// (mirrored across it when they lie on the other side, pushed off it when
/// the chain is straight), then backward and forward passes that keep each
/// segment's length; each bone then turns, root first, towards its solved
/// joint, by `w` of the way. Out of reach the chain points straight at it.
fn fabrik(
    bones: &[Bone2d],
    local: &mut [[f32; 6]],
    chain: &[usize],
    t: [f64; 2],
    w: f64,
    counterclockwise: bool,
) {
    let joint = |world: &[Affine2], i: usize| -> [f64; 2] {
        match chain.get(i) {
            Some(&b) => [f64::from(world[b].tx), f64::from(world[b].ty)],
            None => {
                let last = chain[chain.len() - 1];
                world[last].apply(bones[last].length, 0.0).map(f64::from)
            }
        }
    };
    let world = world_of(bones, local);
    let mut p: Vec<[f64; 2]> = (0..=chain.len()).map(|i| joint(&world, i)).collect();
    let n = chain.len();
    let lengths: Vec<f64> = (0..n)
        .map(|i| sq(p[i + 1][0] - p[i][0], p[i + 1][1] - p[i][1]))
        .collect();
    let total: f64 = lengths.iter().sum();
    let root = p[0];
    let (dx, dy) = (t[0] - root[0], t[1] - root[1]);
    let dist = sq(dx, dy);
    if dist >= total {
        let (ux, uy) = if dist > 0.0 {
            (dx / dist, dy / dist)
        } else {
            (1.0, 0.0)
        };
        for i in 1..=n {
            p[i] = [
                p[i - 1][0] + ux * lengths[i - 1],
                p[i - 1][1] + uy * lengths[i - 1],
            ];
        }
    } else {
        if dist > 0.0 {
            let (ux, uy) = (dx / dist, dy / dist);
            let side = if counterclockwise { 1.0 } else { -1.0 };
            // Left of the line (counter-clockwise) is a positive cross product.
            let bent: f64 = p[1..n]
                .iter()
                .map(|q| ux * (q[1] - root[1]) - uy * (q[0] - root[0]))
                .sum();
            if bent * side < 0.0 {
                for q in &mut p[1..] {
                    let (rx, ry) = (q[0] - root[0], q[1] - root[1]);
                    let along = rx * ux + ry * uy;
                    *q = [
                        root[0] + 2.0 * along * ux - rx,
                        root[1] + 2.0 * along * uy - ry,
                    ];
                }
            } else if bent.abs() <= 1e-9 * total * total {
                for q in &mut p[1..n] {
                    *q = [
                        q[0] - side * uy * total * 0.05,
                        q[1] + side * ux * total * 0.05,
                    ];
                }
            }
        }
        for _ in 0..FABRIK_PASSES {
            if sq(p[n][0] - t[0], p[n][1] - t[1]) <= FABRIK_TOLERANCE {
                break;
            }
            p[n] = t;
            for i in (0..n).rev() {
                let (ex, ey) = (p[i][0] - p[i + 1][0], p[i][1] - p[i + 1][1]);
                let l = sq(ex, ey);
                if l > 1e-12 {
                    let k = lengths[i] / l;
                    p[i] = [p[i + 1][0] + ex * k, p[i + 1][1] + ey * k];
                }
            }
            p[0] = root;
            for i in 1..=n {
                let (ex, ey) = (p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]);
                let l = sq(ex, ey);
                if l > 1e-12 {
                    let k = lengths[i - 1] / l;
                    p[i] = [p[i - 1][0] + ex * k, p[i - 1][1] + ey * k];
                }
            }
        }
    }
    // Root first: each bone turns its next joint towards the solved one.
    for (i, &b) in chain.iter().enumerate() {
        let world = world_of(bones, local);
        let o = joint(&world, i);
        let next = joint(&world, i + 1);
        let now = atan2(next[1] - o[1], next[0] - o[0]);
        let want = atan2(p[i + 1][1] - o[1], p[i + 1][0] - o[0]);
        local[b][2] += (wrap(want - now) * w / DEG) as f32;
    }
}

/// [`world_of`] with the bones a constraint placed held where it put them
/// (their children following).
fn world_fixed(bones: &[Bone2d], local: &[[f32; 6]], fixed: &[Option<Affine2>]) -> Vec<Affine2> {
    let mut world: Vec<Affine2> = Vec::with_capacity(bones.len());
    for (i, (b, l)) in bones.iter().zip(local).enumerate() {
        let m = match fixed[i] {
            Some(m) => m,
            None => {
                let m = Affine2::trsk(l[0], l[1], l[2], l[5], l[3], l[4]);
                match b.parent {
                    Some(p) => world[p].mul(&m),
                    None => m,
                }
            }
        };
        world.push(m);
    }
    world
}

/// `m`'s axes turned by `r` radians.
fn turned(m: &Affine2, r: f64) -> Affine2 {
    let (s, c) = sincos(r);
    let (a, b, cc, d) = (
        f64::from(m.a),
        f64::from(m.b),
        f64::from(m.c),
        f64::from(m.d),
    );
    Affine2 {
        a: (c * a - s * b) as f32,
        b: (s * a + c * b) as f32,
        c: (c * cc - s * d) as f32,
        d: (s * cc + c * d) as f32,
        tx: m.tx,
        ty: m.ty,
    }
}

/// A transform constraint in the armature's space (Spine's
/// `applyAbsoluteWorld`): each bone's rotation, origin, axis lengths and
/// shear moved `mix` of the way to the target's (plus the offsets).
fn transform_constraint(
    bones: &[Bone2d],
    local: &[[f32; 6]],
    fixed: &mut [Option<Affine2>],
    world: &mut Vec<Affine2>,
    c: &TransformConstraint2d,
) {
    let t = world[c.target];
    let f = |v: f32| f64::from(v);
    let reflect = if f(t.a) * f(t.d) - f(t.b) * f(t.c) > 0.0 {
        DEG
    } else {
        -DEG
    };
    let (offset_rotation, offset_shear) = (f(c.rotation) * reflect, f(c.shear) * reflect);
    let [mr, mt, ms, mh] = c.mix.map(f64::from);
    // Each bone from its transform before the constraint (a child's is not
    // moved by its constrained parent until the constraint is done).
    let before = world.clone();
    for &b in &c.bones {
        let mut m = before[b];
        if mr != 0.0 {
            let r = wrap(t.angle() - m.angle() + offset_rotation) * mr;
            m = turned(&m, r);
        }
        if mt != 0.0 {
            let p = t.apply(c.at[0], c.at[1]);
            m.tx = (f(m.tx) + (f(p[0]) - f(m.tx)) * mt) as f32;
            m.ty = (f(m.ty) + (f(p[1]) - f(m.ty)) * mt) as f32;
        }
        if ms > 0.0 {
            for (axis, offset) in [(0, c.scale[0]), (1, c.scale[1])] {
                let (x, y, tx, ty) = if axis == 0 {
                    (m.a, m.b, t.a, t.b)
                } else {
                    (m.c, m.d, t.c, t.d)
                };
                let s = sq(f(x), f(y));
                let ts = sq(f(tx), f(ty));
                if s > 1e-5 {
                    let k = ((s + (ts - s + f(offset)) * ms) / s) as f32;
                    if axis == 0 {
                        (m.a, m.b) = (m.a * k, m.b * k);
                    } else {
                        (m.c, m.d) = (m.c * k, m.d * k);
                    }
                }
            }
        }
        if mh > 0.0 {
            // The angle between the axes moves towards the target's.
            let by = atan2(f(m.d), f(m.c));
            let r = atan2(f(t.d), f(t.c)) - t.angle() - (by - m.angle());
            let r = by + (wrap(r) + offset_shear) * mh;
            let s = sq(f(m.c), f(m.d));
            let (sn, cs) = sincos(r);
            (m.c, m.d) = ((cs * s) as f32, (sn * s) as f32);
        }
        fixed[b] = Some(m);
    }
    *world = world_fixed(bones, local, fixed);
}

/// A path attachment's points in the armature's space (`deform` offsets
/// each influence, as a mesh's).
fn path_points(
    points: &[Vec<Influence>],
    world: &[Affine2],
    deform: Option<&[f32]>,
) -> Vec<[f64; 2]> {
    let mut k = 0usize;
    points
        .iter()
        .map(|v| {
            let mut p = [0.0f64; 2];
            for i in v {
                let (dx, dy) = deform.map_or((0.0, 0.0), |d| {
                    (
                        d.get(2 * k).copied().unwrap_or(0.0),
                        d.get(2 * k + 1).copied().unwrap_or(0.0),
                    )
                });
                k += 1;
                let q = world[i.bone].apply(i.x + dx, i.y + dy);
                p[0] += f64::from(q[0]) * f64::from(i.weight);
                p[1] += f64::from(q[1]) * f64::from(i.weight);
            }
            p
        })
        .collect()
}

/// The point (and the tangent's angle) at `t` of the cubic from `p[0]`
/// through the handles `p[1]`, `p[2]` to `p[3]`.
fn cubic_at(t: f64, p: [[f64; 2]; 4]) -> [f64; 3] {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    let at = |i: usize| a * p[0][i] + b * p[1][i] + c * p[2][i] + d * p[3][i];
    // The derivative's direction: (1-t)² (p1-p0) + 2(1-t)t (p2-p1) + t² (p3-p2).
    let dv = |i: usize| {
        u * u * (p[1][i] - p[0][i])
            + 2.0 * u * t * (p[2][i] - p[1][i])
            + t * t * (p[3][i] - p[2][i])
    };
    let (mut dx, mut dy) = (dv(0), dv(1));
    if dx * dx + dy * dy < 1e-18 {
        (dx, dy) = (p[3][0] - p[0][0], p[3][1] - p[0][1]);
    }
    [at(0), at(1), atan2(dy, dx)]
}

/// The point `d` from `from` along the direction from `a` to `b`, and that direction's angle.
fn along_line(a: [f64; 2], b: [f64; 2], from: [f64; 2], d: f64) -> [f64; 3] {
    let r = atan2(b[1] - a[1], b[0] - a[0]);
    let (s, c) = sincos(r);
    [from[0] + d * c, from[1] + d * s, r]
}

/// Where each of `spaces` (added up from `position`) falls on the path
/// (`points` in the armature's space), with the tangent's angle there:
/// DragonBones' `_computeBezierCurve`. By the `lengths` (a position within
/// a curve is its parameter), or by arc length when `constant` (each
/// curve's length by four forward-difference steps, then ten within the
/// curve). A closed path wraps; an open one extends past its ends along
/// its end handles (Spine; DragonBones leaves the last tick's value).
#[allow(clippy::too_many_arguments)]
fn path_positions(
    points: &[[f64; 2]],
    closed: bool,
    constant: bool,
    lengths: &[f32],
    spaces: &mut [f64],
    percent_position: bool,
    percent_spacing: bool,
    mut position: f64,
) -> Vec<[f64; 3]> {
    let knots = points.len() / 3;
    let mut out = Vec::with_capacity(spaces.len());
    // The curves' four points each, in order.
    let curves: Vec<[[f64; 2]; 4]> = (0..knots - usize::from(!closed))
        .map(|c| {
            let i = |k: usize| points[k % points.len()];
            [i(3 * c + 1), i(3 * c + 2), i(3 * c + 3), i(3 * c + 4)]
        })
        .collect();
    let first = curves[0];
    let last = curves[curves.len() - 1];
    let place = |p: f64, total: f64, find: &dyn Fn(f64) -> [f64; 3], out: &mut Vec<[f64; 3]>| {
        let p = if closed {
            let w = p % total;
            if w < 0.0 { w + total } else { w }
        } else {
            p
        };
        out.push(if !closed && p < 0.0 {
            along_line(first[0], first[1], first[0], p)
        } else if !closed && p > total {
            along_line(last[2], last[3], last[3], p - total)
        } else {
            find(p)
        });
    };
    if !constant {
        let ends: Vec<f64> = lengths.iter().map(|l| f64::from(*l)).collect();
        let total = ends.last().copied().unwrap_or(0.0);
        if percent_position {
            position *= total;
        }
        if percent_spacing {
            spaces.iter_mut().for_each(|s| *s *= total);
        }
        let find = |p: f64| -> [f64; 3] {
            let c = ends.iter().position(|e| p <= *e).unwrap_or(ends.len() - 1);
            let (lo, hi) = (if c == 0 { 0.0 } else { ends[c - 1] }, ends[c]);
            let t = if hi > lo { (p - lo) / (hi - lo) } else { 0.0 };
            cubic_at(t.clamp(0.0, 1.0), curves[c])
        };
        for s in spaces.iter() {
            position += s;
            place(position, total, &find, &mut out);
        }
        return out;
    }
    // Each curve's length by four forward-difference steps (DragonBones' and Spine's).
    let mut ends = Vec::with_capacity(curves.len());
    let mut total = 0.0f64;
    for q in &curves {
        let [p0, p1, p2, p3] = *q;
        let tmp = [0, 1].map(|i| (p0[i] - p1[i] * 2.0 + p2[i]) * 0.1875);
        let ddd = [0, 1].map(|i| ((p1[i] - p2[i]) * 3.0 - p0[i] + p3[i]) * 0.09375);
        let mut dd = [0, 1].map(|i| tmp[i] * 2.0 + ddd[i]);
        let mut d = [0, 1].map(|i| (p1[i] - p0[i]) * 0.75 + tmp[i] + ddd[i] * 0.166_666_67);
        total += sq(d[0], d[1]);
        for _ in 0..2 {
            d = [0, 1].map(|i| d[i] + dd[i]);
            dd = [0, 1].map(|i| dd[i] + ddd[i]);
            total += sq(d[0], d[1]);
        }
        d = [0, 1].map(|i| d[i] + dd[i] + ddd[i]);
        total += sq(d[0], d[1]);
        ends.push(total);
    }
    if percent_position {
        position *= total;
    }
    if percent_spacing {
        spaces.iter_mut().for_each(|s| *s *= total);
    }
    let find = |p: f64| -> [f64; 3] {
        let c = ends.iter().position(|e| p <= *e).unwrap_or(ends.len() - 1);
        let (lo, hi) = (if c == 0 { 0.0 } else { ends[c - 1] }, ends[c]);
        let mut p = if hi > lo { (p - lo) / (hi - lo) } else { 0.0 };
        // Ten forward-difference steps within the curve, to find the parameter.
        let [p0, p1, p2, p3] = curves[c];
        let tmp = [0, 1].map(|i| (p0[i] - p1[i] * 2.0 + p2[i]) * 0.03);
        let ddd = [0, 1].map(|i| ((p1[i] - p2[i]) * 3.0 - p0[i] + p3[i]) * 0.006);
        let mut dd = [0, 1].map(|i| tmp[i] * 2.0 + ddd[i]);
        let mut d = [0, 1].map(|i| (p1[i] - p0[i]) * 0.3 + tmp[i] + ddd[i] * 0.166_666_67);
        let mut segments = [0.0f64; 10];
        let mut length = sq(d[0], d[1]);
        segments[0] = length;
        for s in segments.iter_mut().take(8).skip(1) {
            d = [0, 1].map(|i| d[i] + dd[i]);
            dd = [0, 1].map(|i| dd[i] + ddd[i]);
            length += sq(d[0], d[1]);
            *s = length;
        }
        d = [0, 1].map(|i| d[i] + dd[i]);
        length += sq(d[0], d[1]);
        segments[8] = length;
        d = [0, 1].map(|i| d[i] + dd[i] + ddd[i]);
        length += sq(d[0], d[1]);
        segments[9] = length;
        p *= length;
        let k = segments.iter().position(|e| p <= *e).unwrap_or(9);
        let t = if k == 0 {
            if segments[0] > 0.0 {
                p / segments[0]
            } else {
                0.0
            }
        } else {
            let (lo, hi) = (segments[k - 1], segments[k]);
            k as f64 + if hi > lo { (p - lo) / (hi - lo) } else { 0.0 }
        };
        cubic_at((t * 0.1).clamp(0.0, 1.0), curves[c])
    };
    for s in spaces.iter() {
        position += s;
        place(position, total, &find, &mut out);
    }
    out
}

/// A path constraint (DragonBones' `PathConstraint.update`): the bones'
/// places along the path from their spacing, each bone moved `mix[1]` of
/// the way to its place and turned `mix[0]` of the way along the tangent
/// there or towards the next place (scaled to reach it in `chainscale`).
fn path_constraint(
    body: Body<'_>,
    local: &[[f32; 6]],
    fixed: &mut [Option<Affine2>],
    world: &mut Vec<Affine2>,
    c: &PathConstraint2d,
    deform: Option<&[f32]>,
) {
    let AttachmentKind::Path {
        points,
        closed,
        constant,
        lengths,
    } = &body.attachments[c.path].kind
    else {
        return;
    };
    let points = path_points(points, world, deform);
    let tangent = c.rotate == PathRotate::Tangent;
    let chain_scale = c.rotate == PathRotate::ChainScale;
    let count = c.bones.len();
    let spacing = f64::from(c.spacing);
    let mut spaces = vec![spacing; if tangent { count } else { count + 1 }];
    let mut reach = vec![0.0f64; count];
    if chain_scale || c.spacing_mode == PathSpacing::Length {
        spaces[0] = 0.0;
        for i in 0..spaces.len() - 1 {
            let b = c.bones[i];
            let l = f64::from(body.bones[b].length);
            let m = world[b];
            let len = sq(l * f64::from(m.a), l * f64::from(m.b));
            reach[i] = len;
            spaces[i + 1] = if l > 1e-9 {
                (l + spacing) * len / l
            } else {
                0.0
            };
        }
    }
    let at = path_positions(
        &points,
        *closed,
        *constant,
        lengths,
        &mut spaces,
        c.position_unit == PathUnit::Percent,
        c.spacing_mode == PathSpacing::Percent,
        f64::from(c.position),
    );
    let (mut bx, mut by) = (at[0][0], at[0][1]);
    let mut offset = f64::from(c.rotation);
    let tip = offset == 0.0 && c.rotate == PathRotate::Chain;
    if offset != 0.0 {
        let m = world[body.slots[body.attachments[c.path].slot].bone];
        let det = f64::from(m.a) * f64::from(m.d) - f64::from(m.b) * f64::from(m.c);
        offset *= if det > 0.0 { DEG } else { -DEG };
    }
    let [mr, mt] = c.mix.map(f64::from);
    // As DragonBones and Spine: each bone from its transform before the constraint.
    let before = world.clone();
    for (i, &b) in c.bones.iter().enumerate() {
        let mut m = before[b];
        m.tx = (f64::from(m.tx) + (bx - f64::from(m.tx)) * mt) as f32;
        m.ty = (f64::from(m.ty) + (by - f64::from(m.ty)) * mt) as f32;
        let next = at.get(i + 1).copied().unwrap_or(at[i]);
        let (dx, dy) = (next[0] - bx, next[1] - by);
        if chain_scale && reach[i] > 1e-9 {
            let s = ((sq(dx, dy) / reach[i] - 1.0) * mr + 1.0) as f32;
            (m.a, m.b) = (m.a * s, m.b * s);
        }
        (bx, by) = (next[0], next[1]);
        if mr > 0.0 {
            let mut r = if tangent { at[i][2] } else { atan2(dy, dx) } - m.angle();
            if tip {
                let (s, co) = sincos(r);
                let l = f64::from(body.bones[b].length);
                let (a, bb) = (f64::from(m.a), f64::from(m.b));
                bx += (l * (co * a - s * bb) - dx) * mr;
                by += (l * (s * a + co * bb) - dy) * mr;
            } else {
                r += offset;
            }
            m = turned(&m, wrap(r) * mr);
        }
        fixed[b] = Some(m);
    }
    *world = world_fixed(body.bones, local, fixed);
}

/// Each bone's transform in the armature's space, parents first.
fn world_of(bones: &[Bone2d], local: &[[f32; 6]]) -> Vec<Affine2> {
    let mut world: Vec<Affine2> = Vec::with_capacity(bones.len());
    for (b, l) in bones.iter().zip(local) {
        let m = Affine2::trsk(l[0], l[1], l[2], l[5], l[3], l[4]);
        world.push(match b.parent {
            Some(p) => world[p].mul(&m),
            None => m,
        });
    }
    world
}

/// The length of (x, y).
fn sq(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

fn lerp2(a: &[f32; 2], b: &[f32; 2], f: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
}

/// The value of keys at `t` (`None` without keys), eased by the key in effect.
fn sample<T: Clone>(keys: &[Key<T>], t: f32, lerp: impl Fn(&T, &T, f32) -> T) -> Option<T> {
    let first = keys.first()?;
    if t <= first.time {
        return Some(first.value.clone());
    }
    let i = keys.iter().rposition(|k| k.time <= t).unwrap_or(0);
    let k = &keys[i];
    match keys.get(i + 1) {
        Some(n) if n.time > k.time => {
            let f = (t - k.time) / (n.time - k.time);
            match &k.ease {
                Ease::Linear => Some(lerp(&k.value, &n.value, f)),
                Ease::Step => Some(k.value.clone()),
                Ease::Bezier(c) => Some(lerp(&k.value, &n.value, bezier(c, f))),
            }
        }
        _ => Some(k.value.clone()),
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

/// An armature's names, for its clips' lookups.
struct Names<'a> {
    bones: &'a [String],
    slots: &'a [String],
    attachments: &'a [String],
}

fn parse_clip(e: &Entry, rig: &Armature2d, names: &Names<'_>) -> Result<Clip2d> {
    let name = name_of(e)?;
    let blk = block(e)?;
    let mut clip = Clip2d {
        name: name.clone(),
        duration: 0.0,
        looping: true,
        bones: Vec::new(),
        slots: Vec::new(),
        deforms: Vec::new(),
        order: Vec::new(),
        iks: Vec::new(),
        events: Vec::new(),
    };
    let time = |c: &Entry| -> Result<f32> {
        let t = num(c, 0)?;
        if t < 0.0 {
            return Err(parse_err(
                c.line,
                format!("clip {name}: a key's time is at least 0"),
            ));
        }
        Ok(t)
    };
    let sorted = |times: &[f32], line: usize| -> Result<()> {
        if times.windows(2).any(|w| w[1] < w[0]) {
            return Err(parse_err(
                line,
                format!("clip {name}: keys are in time order"),
            ));
        }
        Ok(())
    };
    fn times<T>(keys: &[Key<T>]) -> Vec<f32> {
        keys.iter().map(|k| k.time).collect()
    }
    let key1 = |k: &Entry| -> Result<Key<f32>> {
        Ok(Key {
            time: time(k)?,
            value: num(k, 1)?,
            ease: ease_at(k, 2)?,
        })
    };
    let key2 = |k: &Entry| -> Result<Key<[f32; 2]>> {
        Ok(Key {
            time: time(k)?,
            value: [num(k, 1)?, num(k, 2)?],
            ease: ease_at(k, 3)?,
        })
    };
    let key4 = |k: &Entry| -> Result<Key<[f32; 4]>> {
        Ok(Key {
            time: time(k)?,
            value: [num(k, 1)?, num(k, 2)?, num(k, 3)?, num(k, 4)?],
            ease: ease_at(k, 5)?,
        })
    };
    for c in &blk.entries {
        match c.key.as_str() {
            "duration" => clip.duration = num(c, 0)?,
            "loop" => clip.looping = c.values.first().and_then(|v| v.as_bool()).unwrap_or(true),
            "event" => {
                let n = c
                    .string(1)
                    .map_err(|_| parse_err(c.line, "event <time> \"name\""))?
                    .to_string();
                if n.len() > 63 {
                    return Err(parse_err(
                        c.line,
                        format!("event `{n}` is longer than 63 bytes"),
                    ));
                }
                clip.events.push((time(c)?, n));
            }
            "bone" => {
                let mut t = BoneTimeline {
                    bone: index(names.bones, &name_of(c)?, "bone", c.line)?,
                    ..Default::default()
                };
                for k in &block(c)?.entries {
                    match k.key.as_str() {
                        "rotate" => {
                            // `clockwise <turns>` or `counterclockwise <turns>` may follow the angle.
                            let (turn, ease_from) = match k.values.get(2).and_then(Value::as_word) {
                                Some(w @ ("clockwise" | "counterclockwise")) => {
                                    let n = num(k, 3)?;
                                    if n < 0.0 || n.fract() != 0.0 || n > 1e6 {
                                        return Err(parse_err(
                                            k.line,
                                            format!(
                                                "rotate: {w} takes a whole number of further turns (0 the nearest {w} way)"
                                            ),
                                        ));
                                    }
                                    let n = n as u32;
                                    let turn = if w == "clockwise" {
                                        Turn::Clockwise(n)
                                    } else {
                                        Turn::Counterclockwise(n)
                                    };
                                    (turn, 4)
                                }
                                _ => (Turn::Shorter, 2),
                            };
                            t.rotate.push(Key {
                                time: time(k)?,
                                value: Rotation {
                                    degrees: num(k, 1)?,
                                    turn,
                                },
                                ease: ease_at(k, ease_from)?,
                            });
                        }
                        "skew" => t.skew.push(key1(k)?),
                        "translate" => t.translate.push(key2(k)?),
                        "scale" => t.scale.push(key2(k)?),
                        x => {
                            return Err(parse_err(
                                k.line,
                                format!("unknown key `{x}` in a bone timeline"),
                            ));
                        }
                    }
                }
                sorted(&times(&t.rotate), c.line)?;
                sorted(&times(&t.skew), c.line)?;
                sorted(&times(&t.translate), c.line)?;
                sorted(&times(&t.scale), c.line)?;
                clip.bones.push(t);
            }
            "slot" => {
                let slot = index(names.slots, &name_of(c)?, "slot", c.line)?;
                let mut t = SlotTimeline {
                    slot,
                    ..Default::default()
                };
                for k in &block(c)?.entries {
                    match k.key.as_str() {
                        "attachment" => {
                            let a = word(k, 1)?;
                            let a = if a == "none" {
                                None
                            } else {
                                let i = index(names.attachments, &a, "attachment", k.line)?;
                                if rig.attachments[i].slot != slot {
                                    return Err(parse_err(
                                        k.line,
                                        format!("attachment {a} belongs to another slot"),
                                    ));
                                }
                                Some(i)
                            };
                            t.attachment.push((time(k)?, a));
                        }
                        "color" => t.color.push(key4(k)?),
                        "offset" => t.offset.push(key4(k)?),
                        x => {
                            return Err(parse_err(
                                k.line,
                                format!("unknown key `{x}` in a slot timeline"),
                            ));
                        }
                    }
                }
                sorted(
                    &t.attachment.iter().map(|k| k.0).collect::<Vec<_>>(),
                    c.line,
                )?;
                sorted(&times(&t.color), c.line)?;
                sorted(&times(&t.offset), c.line)?;
                clip.slots.push(t);
            }
            "deform" => {
                let a = index(names.attachments, &name_of(c)?, "attachment", c.line)?;
                let len = 2 * rig.attachments[a].influences();
                if len == 0 {
                    return Err(parse_err(
                        c.line,
                        format!(
                            "clip {name}: deform names {}, not a mesh or a path; only those deform",
                            names.attachments[a]
                        ),
                    ));
                }
                let mut keys = Vec::new();
                for k in &block(c)?.entries {
                    if k.key != "key" {
                        return Err(parse_err(
                            k.line,
                            format!("unknown key `{}` in a deform timeline", k.key),
                        ));
                    }
                    // The offsets run to the first word (`step` or `curve`).
                    let end = k
                        .values
                        .iter()
                        .position(|v| v.as_word().is_some())
                        .unwrap_or(k.values.len());
                    let count = end.saturating_sub(1);
                    if count != len {
                        return Err(parse_err(
                            k.line,
                            format!(
                                "deform {}: {count} offsets for {} influences (x y each)",
                                names.attachments[a],
                                len / 2
                            ),
                        ));
                    }
                    keys.push(Key {
                        time: time(k)?,
                        value: (1..=count).map(|i| num(k, i)).collect::<Result<_>>()?,
                        ease: ease_at(k, end)?,
                    });
                }
                sorted(&times(&keys), c.line)?;
                clip.deforms.push(DeformTimeline {
                    attachment: a,
                    keys,
                });
            }
            "order" => {
                let mut order = Vec::new();
                for i in 1..c.values.len() {
                    order.push(index(names.slots, &word(c, i)?, "slot", c.line)?);
                }
                let mut seen = order.clone();
                seen.sort_unstable();
                seen.dedup();
                if !order.is_empty()
                    && (seen.len() != order.len() || order.len() != rig.slots.len())
                {
                    return Err(parse_err(
                        c.line,
                        format!(
                            "clip {name}: an order names each of the {} slots once (or none, the setup order)",
                            rig.slots.len()
                        ),
                    ));
                }
                clip.order.push((time(c)?, order));
            }
            "ik" => {
                let ik_name = name_of(c)?;
                let ik = rig
                    .iks
                    .iter()
                    .position(|k| k.name == ik_name)
                    .ok_or_else(|| parse_err(c.line, format!("no ik `{ik_name}`")))?;
                let mut keys = Vec::new();
                for k in &block(c)?.entries {
                    if k.key != "key" {
                        return Err(parse_err(
                            k.line,
                            format!(
                                "unknown key `{}` in an ik timeline (key <time> <weight> <bend>)",
                                k.key
                            ),
                        ));
                    }
                    keys.push(Key {
                        time: time(k)?,
                        value: IkKey {
                            weight: num(k, 1)?.clamp(0.0, 1.0),
                            counterclockwise: bend(k, 2)?,
                        },
                        ease: ease_at(k, 3)?,
                    });
                }
                sorted(&times(&keys), c.line)?;
                clip.iks.push(IkTimeline { ik, keys });
            }
            x => {
                return Err(parse_err(
                    c.line,
                    format!("unknown key `{x}` in clip {name}"),
                ));
            }
        }
    }
    if clip.duration.is_nan() || clip.duration <= 0.0 {
        return Err(parse_err(
            e.line,
            format!("clip {name} needs `duration` above 0"),
        ));
    }
    sorted(&clip.order.iter().map(|o| o.0).collect::<Vec<_>>(), e.line)?;
    clip.events.sort_by(|a, b| a.0.total_cmp(&b.0));
    Ok(clip)
}

fn quote(s: &str) -> String {
    let mut q = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => q += "\\\"",
            '\\' => q += "\\\\",
            '\n' => q += "\\n",
            c => q.push(c),
        }
    }
    q + "\""
}

fn num_text(v: f32) -> String {
    Number::from_f32(v)
        .map(|n| n.text().to_string())
        .unwrap_or_else(|_| "0".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARM: &str = r#"type rig2d
version 1
---
texture "arm.png"
size 64 32
unit 10
bone root {
}
bone upper {
  parent root
  rotation 0
  length 20
}
bone lower {
  parent upper
  at 20 0
  length 20
}
bone hand {
  parent root
  at 30 10
}
slot skin {
  bone upper
  attachment skin
}
slot plate {
  bone lower
  attachment plate
  color 1 0.5 0.5 1
}
mesh skin {
  slot skin
  vertex 0 0 0 -2
  vertex 64 0 upper 20 -2 0.5 lower 0 -2 0.5
  vertex 64 32 lower 20 2 1
  triangles 0 1 2
}
region plate {
  slot plate
  rect 0 0 10 4
  at 10 0
  rotation 90
}
ik reach {
  bones upper lower
  target hand
  bend counterclockwise
}
clip wave {
  duration 1
  loop false
  bone upper {
    rotate 0 0
    rotate 1 90
  }
  slot plate {
    attachment 0.5 none
    color 0 1 1 1 1
    color 1 0 0 0 1
  }
  deform skin {
    key 0 0 0 0 0 0 0 0 0
    key 1 0 2 0 0 0 0 0 4
  }
  event 0.25 "lift"
}
"#;

    fn rig() -> Rig2dAsset {
        Rig2dAsset::parse(ARM).expect("the arm parses")
    }

    fn close(a: [f32; 2], b: [f32; 2], eps: f32) -> bool {
        (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps
    }

    #[test]
    fn trig_from_arithmetic_matches_the_library() {
        for i in -40..40 {
            let x = f64::from(i) * 0.37;
            let (s, c) = sincos(x);
            assert!(
                (s - x.sin()).abs() < 1e-14 && (c - x.cos()).abs() < 1e-14,
                "{x}"
            );
            assert!(
                (atan2(s * 3.0, c * 3.0) - x.sin().atan2(x.cos())).abs() < 1e-14,
                "{x}"
            );
        }
    }

    #[test]
    fn the_text_parses_back_equal() {
        let r = rig();
        assert_eq!(r.slots[1].attachment, Some(1));
        assert_eq!(r.attachments[0].influences(), 4);
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
    }

    #[test]
    fn clips_turn_bones_relative_to_setup_and_switch_slots() {
        let r = rig();
        let c = &r.clips[0];
        // Half way: the upper arm turned 45 degrees; its tip is at 20 (cos, sin) 45.
        let p = r.pose(Some(c), 0.5, &[(0, [100.0, 100.0], 0.0)]);
        let tip = p.world[1].apply(20.0, 0.0);
        let h = 20.0 * std::f32::consts::FRAC_1_SQRT_2;
        assert!(close(tip, [h, h], 1e-4), "{tip:?}");
        // The lower bone's origin is the upper's tip.
        assert!(close([p.world[2].tx, p.world[2].ty], [h, h], 1e-4));
        // At 0.5 the plate's attachment key hides it; the colour runs from white to black.
        assert_eq!(p.slots[1], None);
        assert_eq!(r.pose(Some(c), 0.25, &[]).slots[1], Some(1));
        assert_eq!(p.colors[1], [0.5, 0.5, 0.5, 1.0]);
    }

    #[test]
    fn two_bone_ik_reaches_and_bends_the_asked_way() {
        let r = rig();
        // The hand bone at (30, 10) is the target: within reach (40).
        let p = r.pose(None, 0.0, &[]);
        assert!(p.ik_error[0] < 1e-3, "{:?}", p.ik_error);
        let tip = p.world[2].apply(20.0, 0.0);
        assert!(close(tip, [30.0, 10.0], 1e-3), "{tip:?}");
        // Counter-clockwise: the elbow lies left of the line from shoulder to target.
        let elbow = [p.world[2].tx, p.world[2].ty];
        assert!(30.0 * elbow[1] - 10.0 * elbow[0] > 0.0, "{elbow:?}");
        // A target out of reach: the arm points at it, straight.
        let p = r.pose(None, 0.0, &[(0, [0.0, -100.0], 1.0)]);
        let tip = p.world[2].apply(20.0, 0.0);
        assert!(close(tip, [0.0, -40.0], 1e-2), "{tip:?}");
        assert!((p.ik_error[0] - 60.0).abs() < 1e-2);
        // Weight 0 keeps the animated pose.
        let p = r.pose(None, 0.0, &[(0, [0.0, -100.0], 0.0)]);
        assert!(close([p.world[2].tx, p.world[2].ty], [20.0, 0.0], 1e-4));
    }

    #[test]
    fn weighted_and_deformed_vertices_follow_their_bones() {
        let r = rig();
        let c = &r.clips[0];
        // At the end the upper arm is turned 90 degrees and the deform at full.
        let p = r.pose(Some(c), 1.0, &[(0, [0.0, 40.0], 0.0)]);
        let (v, i) = r.triangles(&p);
        // The mesh first (slot order); the plate is hidden at 1.
        assert_eq!(i, [0, 1, 2]);
        // Vertex 0: (0, -2) + (0, 2) in the upper arm turned 90: the origin.
        assert!(close(v[0].position, [0.0, 0.0], 1e-5), "{:?}", v[0]);
        // Vertex 1: half of upper (20, -2) and half of lower (0, -2), both
        // turned: (2, 20) in pixels, a tenth in world units.
        assert!(close(v[1].position, [0.2, 2.0], 1e-5), "{:?}", v[1]);
        // Vertex 2: lower (20, 2 + 4) turned: (-6, 40).
        assert!(close(v[2].position, [-0.6, 4.0], 1e-5), "{:?}", v[2]);
        assert_eq!(v[2].uv, [1.0, 1.0]);
    }

    #[test]
    fn a_region_is_a_quad_of_its_rect() {
        let r = rig();
        let p = r.pose(None, 0.0, &[(0, [40.0, 0.0], 1.0)]);
        let (v, i) = r.triangles(&p);
        assert_eq!(i.len(), 9);
        // The plate: 10 x 4 at (10, 0) of the lower bone (from 20), turned 90.
        let quad = &v[3..7];
        assert!(close(quad[0].position, [3.2, -0.5], 1e-3), "{quad:?}");
        assert_eq!(quad[0].uv, [0.0, 4.0 / 32.0]);
        assert_eq!(quad[2].uv, [10.0 / 64.0, 0.0]);
        assert_eq!(quad[0].color, [1.0, 0.5, 0.5, 1.0]);
    }

    /// A rig of every DragonBones feature the text carries beyond the
    /// arm: a child armature in a slot (with a box of its own), a box,
    /// colour offsets, a draw-order key.
    const NEST: &str = r#"type rig2d
version 1
---
texture "parts.png"
size 4 4
unit 1
bone root {
}
bone hand {
  parent root
  at 10 0
  rotation 90
  length 4
}
slot body {
  bone root
  attachment body
}
slot hand {
  bone hand
  attachment lamp
  color 0.5 1 1 1
  offset 0 0.25 0 0
}
slot top {
  bone root
  attachment top
}
region body {
  slot body
  rect 0 0 2 2
}
child lamp {
  slot hand
  armature flame
  clip burn
  at 4 0
  scale 2 2
}
box reach {
  slot top
  polygon 0 0 4 0 4 2
}
region top {
  slot top
  rect 2 2 2 2
}
clip wave {
  duration 1
}
clip idle {
  duration 4
  slot top {
    attachment 0 top
    attachment 3.5 reach
  }
  order 1 "top" "hand" "body"
  order 3
}
armature flame {
  bone base {
  }
  slot fire {
    bone base
    attachment fire
    color 0.5 0.5 0.5 1
    offset 0.1 0 0 0
  }
  slot heat {
    bone base
    attachment heat
  }
  region fire {
    slot fire
    rect 0 0 2 2
    at 1 0
  }
  box heat {
    slot heat
    polygon 0 0 1 0 1 1
  }
  clip burn {
    duration 1
    bone base {
      rotate 0 0
      rotate 1 90
    }
  }
  clip wave {
    duration 2
    bone base {
      rotate 0 0
      rotate 2 -90
    }
  }
}
"#;

    /// The centre of quad `q` (four corners from `first`).
    fn centre(v: &[RigVertex], first: usize) -> [f32; 2] {
        let q = &v[first..first + 4];
        [
            q.iter().map(|c| c.position[0]).sum::<f32>() / 4.0,
            q.iter().map(|c| c.position[1]).sum::<f32>() / 4.0,
        ]
    }

    #[test]
    fn skew_turns_a_bones_y_axis_apart_from_its_x_axis() {
        let text = ARM
            .replace("bone hand {\n  parent root\n  at 30 10\n}", "bone hand {\n  parent root\n  at 30 10\n}\nbone tilt {\n  parent root\n  at 10 0\n  rotation 10\n  skew 30\n  scale 2 3\n}")
            .replace("  rect 0 0 10 4\n  at 10 0\n  rotation 90\n", "  rect 0 0 10 4\n  at 0 0\n  rotation 0\n  skew 30\n  size 2 2\n")
            .replace("bone lower\n  attachment plate", "bone tilt\n  attachment plate")
            .replace("  bone upper {\n    rotate 0 0", "  bone tilt {\n    skew 0 0\n    skew 1 20\n  }\n  bone upper {\n    rotate 0 0");
        let r = Rig2dAsset::parse(&text).expect("skewed");
        let tilt = r.bone("tilt").unwrap();
        assert_eq!(r.bones[tilt].skew, 30.0);
        let d = |deg: f64| deg.to_radians();
        let near = |a: f32, b: f64| (f64::from(a) - b).abs() < 1e-5;
        // Setup: x axis at 10 degrees times 2, y axis at 40 degrees times 3.
        let m = r.pose(None, 0.0, &[(0, [40.0, 0.0], 0.0)]).world[tilt];
        assert!(
            near(m.a, 2.0 * d(10.0).cos()) && near(m.b, 2.0 * d(10.0).sin()),
            "{m:?}"
        );
        assert!(
            near(m.c, -3.0 * d(40.0).sin()) && near(m.d, 3.0 * d(40.0).cos()),
            "{m:?}"
        );
        // Half way through the clip the skew key adds 10: the y axis at 50.
        let p = r.pose(Some(&r.clips[0]), 0.5, &[(0, [40.0, 0.0], 0.0)]);
        let m = p.world[tilt];
        assert!(near(m.a, 2.0 * d(10.0).cos()), "{m:?}");
        assert!(
            near(m.c, -3.0 * d(50.0).sin()) && near(m.d, 3.0 * d(50.0).cos()),
            "{m:?}"
        );
        // The plate region's own skew 30: its corner (1, 1) is (1 - sin 30, cos 30) in the bone.
        let p = r.pose(None, 0.0, &[(0, [40.0, 0.0], 0.0)]);
        let (v, _) = r.triangles(&p);
        let want = p.world[tilt].apply(1.0 - 0.5, 3f32.sqrt() / 2.0);
        assert!(
            close(v[5].position, [want[0] / 10.0, want[1] / 10.0], 1e-5),
            "{:?} {want:?}",
            v[5]
        );
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
    }

    #[test]
    fn bezier_keys_follow_their_curve_not_the_line() {
        let text = ARM
            .replace("    rotate 0 0\n    rotate 1 90", "    rotate 0 0 curve 0.42 0 1 1\n    rotate 1 90")
            .replace(
                "  slot plate {",
                "  bone hand {\n    translate 0 0 0 curve 0.1 0.6 0.3 0.8 0.5 0.8 0.7 0.8 0.9 1\n    translate 1 100 0\n  }\n  slot plate {",
            );
        let r = Rig2dAsset::parse(&text).expect("curves");
        let c = &r.clips[0];
        let p = r.pose(Some(c), 0.5, &[(0, [0.0, 0.0], 0.0)]);
        // Ease-in (0.42 0 1 1) at half time is 0.3153568 of the way
        // (solved independently): 28.38211 degrees, not the line's 45.
        let angle = f64::from(p.world[1].b)
            .atan2(f64::from(p.world[1].a))
            .to_degrees();
        assert!((angle - 90.0 * 0.315_356_81).abs() < 1e-3, "{angle}");
        // Two segments through the anchor (0.5, 0.8).
        let hand = r.bone("hand").unwrap();
        for (t, f) in [(0.25, 0.673_215_3), (0.75, 0.879_110_6), (0.5, 0.8)] {
            let x = r.pose(Some(c), t, &[(0, [0.0, 0.0], 0.0)]).world[hand].tx;
            assert!((x - (30.0 + 100.0 * f)).abs() < 1e-3, "{t}: {x}");
        }
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        let bad = |curve: &str| {
            Rig2dAsset::parse(
                &ARM.replace("    rotate 0 0\n", &format!("    rotate 0 0 {curve}\n")),
            )
            .expect_err(curve)
            .reason()
            .to_string()
        };
        assert!(bad("curve 0.5 0 1").contains("x1 y1 x2 y2"));
        assert!(bad("curve 1.5 0 1 1").contains("0..1"));
        assert!(bad("smooth").contains("step"));
    }

    #[test]
    fn rotate_keys_turn_the_asked_way_through_whole_turns() {
        // The bone's angle at `t` of a 1 s turn from 0 to 30 degrees; each
        // expected angle is the turn's total (worked out by hand) times t.
        let angle_at = |turn: &str, t: f32| -> f64 {
            let text = ARM.replace(
                "    rotate 0 0\n    rotate 1 90",
                &format!("    rotate 0 0{turn}\n    rotate 1 30"),
            );
            let r = Rig2dAsset::parse(&text).expect(turn);
            assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
            let p = r.pose(Some(&r.clips[0]), t, &[(0, [0.0, 0.0], 0.0)]);
            f64::from(p.world[1].b)
                .atan2(f64::from(p.world[1].a))
                .to_degrees()
        };
        let same = |a: f64, b: f64| (a - b).rem_euclid(360.0).min((b - a).rem_euclid(360.0)) < 1e-3;
        for (turn, total) in [
            ("", 30.0),
            (" counterclockwise 0", 30.0),
            (" counterclockwise 2", 750.0),
            (" clockwise 0", -330.0),
            (" clockwise 1", -690.0),
        ] {
            for t in [0.1f32, 0.25, 0.6] {
                let a = angle_at(turn, t);
                assert!(
                    same(a, total * f64::from(t)),
                    "{turn} at {t}: {a}, not {}",
                    total * f64::from(t)
                );
            }
        }
        // The turn's own arithmetic: equal angles turn 0 the nearest way, a whole turn per count.
        assert_eq!(turn_delta(10.0, 10.0, Turn::Clockwise(0)), 0.0);
        assert_eq!(turn_delta(10.0, 10.0, Turn::Counterclockwise(1)), 360.0);
        assert_eq!(turn_delta(170.0, -170.0, Turn::Shorter), 20.0);
        let bad =
            Rig2dAsset::parse(&ARM.replace("    rotate 0 0\n", "    rotate 0 0 clockwise 1.5\n"))
                .expect_err("a fraction of a turn");
        assert!(bad.reason().contains("whole number"), "{}", bad.reason());
    }

    #[test]
    fn colour_offsets_add_after_the_multiplier_and_animate() {
        let text = ARM
            .replace(
                "  color 1 0.5 0.5 1\n}",
                "  color 1 0.5 0.5 1\n  offset 0.5 0 0 0\n}",
            )
            .replace(
                "    color 1 0 0 0 1\n",
                "    color 1 0 0 0 1\n    offset 0 0 0.2 0 0\n    offset 1 0 0.6 0 0\n",
            );
        let r = Rig2dAsset::parse(&text).expect("offsets");
        assert_eq!(r.slots[1].offset, [0.5, 0.0, 0.0, 0.0]);
        let p = r.pose(None, 0.0, &[(0, [40.0, 0.0], 1.0)]);
        let (v, _) = r.triangles(&p);
        assert_eq!(
            (v[3].color, v[3].offset),
            ([1.0, 0.5, 0.5, 1.0], [0.5, 0.0, 0.0, 0.0])
        );
        let p = r.pose(Some(&r.clips[0]), 0.25, &[(0, [40.0, 0.0], 1.0)]);
        assert!(
            close([p.offsets[1][0], p.offsets[1][1]], [0.0, 0.3], 1e-6),
            "{:?}",
            p.offsets
        );
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
    }

    #[test]
    fn ik_keys_animate_the_weight_and_bend_and_the_target_weight_multiplies() {
        let text = ARM.replace(
            "  event 0.25 \"lift\"",
            "  ik reach {\n    key 0 1 counterclockwise\n    key 0.5 0 clockwise\n    key 1 1 clockwise\n  }\n  event 0.25 \"lift\"",
        );
        let r = Rig2dAsset::parse(&text).expect("ik keys");
        let c = &r.clips[0];
        let elbow_side = |p: &Pose2d| 30.0 * p.world[2].ty - 10.0 * p.world[2].tx;
        // At 0 the key's weight 1, counter-clockwise: the hand reached, the elbow left.
        let p = r.pose(Some(c), 0.0, &[]);
        assert!(close(p.world[2].apply(20.0, 0.0), [30.0, 10.0], 1e-3) && elbow_side(&p) > 0.0);
        // At 0.5 the weight is 0: the clip's arm (upper turned 45, straight),
        // even with a target of weight 1 (it multiplies the clip's).
        let h = 40.0 * std::f32::consts::FRAC_1_SQRT_2;
        for targets in [&[][..], &[(0, [30.0, 10.0], 1.0)][..]] {
            let p = r.pose(Some(c), 0.5, targets);
            assert!(
                close(p.world[2].apply(20.0, 0.0), [h, h], 1e-3),
                "{:?}",
                p.world[2]
            );
        }
        // At 1 weight 1, clockwise: reached with the elbow right of the line.
        let p = r.pose(Some(c), 1.0, &[]);
        assert!(close(p.world[2].apply(20.0, 0.0), [30.0, 10.0], 1e-3) && elbow_side(&p) < 0.0);
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
    }

    #[test]
    fn a_child_armature_is_posed_and_drawn_in_its_slot() {
        let r = Rig2dAsset::parse(NEST).expect("nested");
        assert_eq!(r.armatures.len(), 1);
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        // The hand bone at (10, 0) points up; the child sits 4 along it at
        // scale 2: its origin (10, 4), its x axis (0, 2), its y axis (-2, 0).
        let place = |cx: f32, cy: f32| [10.0 - 2.0 * cy, 4.0 + 2.0 * cx];
        let s = std::f32::consts::FRAC_1_SQRT_2;
        let idle = &r.clips[1];
        // idle has no child clip of its name: the child plays burn (its
        // `clip` line) at the parent's time, 0.5 (base turned 45), and wraps it at 2.5.
        for t in [0.5, 2.5] {
            let p = r.pose(Some(idle), t, &[]);
            let (v, i) = r.triangles(&p);
            assert_eq!((v.len(), i.len()), (12, 18), "body, the flame, top");
            assert!(
                close(centre(&v, 4), place(s, s), 1e-4),
                "{t}: {:?}",
                centre(&v, 4)
            );
        }
        // wave: the child has a wave of its own (2 s, to -90) and plays it.
        let p = r.pose(Some(&r.clips[0]), 0.5, &[]);
        let (v, _) = r.triangles(&p);
        let (sn, cs) = (22.5f32.to_radians().sin(), 22.5f32.to_radians().cos());
        assert!(
            close(centre(&v, 4), place(cs, -sn), 1e-4),
            "{:?}",
            centre(&v, 4)
        );
        // Colours compose: the hand slot's map over the fire's.
        assert_eq!(v[4].color, [0.25, 0.5, 0.5, 1.0]);
        assert!(
            close([v[4].offset[0], v[4].offset[1]], [0.05, 0.25], 1e-6),
            "{:?}",
            v[4].offset
        );
        // A child that shows its parent is refused.
        let cycle = NEST.replace(
            "  clip burn {",
            "  slot again {\n    bone base\n  }\n  child again {\n    slot again\n    armature flame\n  }\n  clip burn {",
        );
        let e = Rig2dAsset::parse(&cycle)
            .expect_err("a cycle")
            .reason()
            .to_string();
        assert!(e.contains("shows itself"), "{e}");
        let e = Rig2dAsset::parse(&NEST.replace("  clip burn\n", "  clip smoulder\n"))
            .expect_err("no such clip")
            .reason()
            .to_string();
        assert!(e.contains("smoulder"), "{e}");
    }

    #[test]
    fn order_keys_redraw_the_slots_back_to_front() {
        let r = Rig2dAsset::parse(NEST).expect("nested");
        let idle = &r.clips[1];
        let first_u = |t: f32| {
            let (v, _) = r.triangles(&r.pose(Some(idle), t, &[]));
            v[0].uv[0]
        };
        // Setup order: body (u from 0) first; from 1 s top (u from 0.5);
        // `order 3` alone is the setup order again.
        assert_eq!((first_u(0.5), first_u(1.5), first_u(3.2)), (0.0, 0.5, 0.0));
        let p = r.pose(Some(idle), 1.5, &[]);
        assert_eq!(p.order, [2, 1, 0]);
        let e = Rig2dAsset::parse(
            &NEST.replace("order 1 \"top\" \"hand\" \"body\"", "order 1 top hand"),
        )
        .expect_err("two of three")
        .reason()
        .to_string();
        assert!(e.contains("each of the 3 slots"), "{e}");
    }

    #[test]
    fn bounding_boxes_are_the_polygons_the_pose_shows() {
        let r = Rig2dAsset::parse(NEST).expect("nested");
        let idle = &r.clips[1];
        let s = std::f32::consts::FRAC_1_SQRT_2;
        // At 0.5 the top slot shows its region, the child's heat box shows
        // in the hand: (1, 0) in the base turned 45, then placed.
        let b = r.boxes(&r.pose(Some(idle), 0.5, &[]));
        assert_eq!(b.len(), 1, "{b:?}");
        assert_eq!(
            (b[0].slot.as_str(), b[0].name.as_str()),
            ("hand/heat", "heat")
        );
        assert!(
            close(b[0].points[1], [10.0 - 2.0 * s, 4.0 + 2.0 * s], 1e-4),
            "{b:?}"
        );
        // From 3.5 the top slot shows the reach box, in the root bone (back
        // in the setup order, after the hand).
        let b = r.boxes(&r.pose(Some(idle), 3.6, &[]));
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].slot, "hand/heat");
        assert_eq!(
            (b[1].slot.as_str(), b[1].points.clone()),
            ("top", vec![[0.0, 0.0], [4.0, 0.0], [4.0, 2.0]])
        );
        let (v, i) = r.triangles(&r.pose(Some(idle), 3.6, &[]));
        assert_eq!((v.len(), i.len()), (8, 12), "a box draws nothing");
    }

    /// The one- and two-bone solutions as they were before chains of any
    /// length (the numbers the depth stream's solver gave, to the bit).
    #[test]
    fn one_and_two_bone_ik_give_the_poses_they_always_gave() {
        let a = |a: f32, b: f32, c: f32, d: f32, tx: f32, ty: f32| Affine2 { a, b, c, d, tx, ty };
        let r = rig();
        let cases = [
            (
                [30.0, 10.0],
                1.0,
                a(0.5563508, 0.8309475, -0.8309475, 0.5563508, 0.0, 0.0),
                a(
                    0.9436492,
                    -0.33094752,
                    0.33094752,
                    0.9436492,
                    11.127016,
                    16.61895,
                ),
                0.0,
            ),
            (
                [-12.5, 21.25],
                1.0,
                a(-0.9912518, 0.13198434, -0.13198434, -0.9912518, 0.0, 0.0),
                a(
                    0.36625195,
                    0.93051577,
                    -0.93051577,
                    0.36625195,
                    -19.825037,
                    2.6396868,
                ),
                2.6973983e-6,
            ),
            (
                [0.0, -100.0],
                1.0,
                a(0.00022357237, -1.0, 1.0, 0.00022357237, 0.0, 0.0),
                a(-5.5893106e-5, -1.0, 1.0, -5.5893106e-5, 0.0044714473, -20.0),
                60.0,
            ),
            (
                [25.0, -5.0],
                0.4,
                a(0.96298677, 0.26954868, -0.26954868, 0.96298677, 0.0, 0.0),
                a(
                    0.9656971,
                    -0.25967127,
                    0.25967127,
                    0.9656971,
                    19.259735,
                    5.3909736,
                ),
                14.534759,
            ),
        ];
        for (t, w, upper, lower, err) in cases {
            let p = r.pose(None, 0.0, &[(0, t, w)]);
            assert_eq!(
                (p.world[1], p.world[2], p.ik_error[0]),
                (upper, lower, err),
                "{t:?}"
            );
        }
        let r = Rig2dAsset::parse(&ARM.replace("bones upper lower", "bones upper")).unwrap();
        for (t, w, upper, err) in [
            (
                [30.0, 10.0],
                1.0,
                a(0.9486833, 0.31622776, -0.31622776, 0.9486833, 0.0, 0.0),
                2.9802322e-7,
            ),
            (
                [-12.5, -21.25],
                0.5,
                a(0.4964775, -0.86804956, 0.86804956, 0.4964775, 0.0, 0.0),
                21.400766,
            ),
        ] {
            let p = r.pose(None, 0.0, &[(0, t, w)]);
            assert_eq!((p.world[1], p.ik_error[0]), (upper, err), "{t:?}");
        }
    }

    /// A tail of four 10-pixel bones, straight along x in its setup pose.
    const TAIL: &str = r#"type rig2d
version 1
---
texture "tail.png"
size 8 8
bone root {
}
bone t1 {
  parent root
  length 10
}
bone t2 {
  parent t1
  at 10 0
  length 10
}
bone t3 {
  parent t2
  at 10 0
  length 10
}
bone t4 {
  parent t3
  at 10 0
  length 10
}
bone goal {
  parent root
  at 25 15
}
ik curl {
  bones t1 t2 t3 t4
  target goal
  bend counterclockwise
}
"#;

    #[test]
    fn a_chain_of_any_length_reaches_and_curls_the_asked_way() {
        let r = Rig2dAsset::parse(TAIL).expect("a four-bone chain");
        assert_eq!(r.iks[0].bones, [1, 2, 3, 4]);
        let joints = |p: &Pose2d| -> Vec<[f32; 2]> {
            (1..=4)
                .map(|b| [p.world[b].tx, p.world[b].ty])
                .chain(std::iter::once(p.world[4].apply(10.0, 0.0)))
                .collect()
        };
        // Left of the line from the root to the target is counter-clockwise.
        let sides = |p: &Pose2d, t: [f32; 2]| -> Vec<f32> {
            joints(p)[1..4]
                .iter()
                .map(|q| t[0] * q[1] - t[1] * q[0])
                .collect()
        };
        for (bend, sign) in [("counterclockwise", 1.0), ("clockwise", -1.0)] {
            let r = Rig2dAsset::parse(&TAIL.replace("counterclockwise", bend)).unwrap();
            // From the straight setup pose, to the goal bone (25, 15): 29.15 of 40 away.
            let p = r.pose(None, 0.0, &[]);
            let j = joints(&p);
            assert!(close(j[4], [25.0, 15.0], 1e-3), "{bend}: {j:?}");
            assert!(p.ik_error[0] < 1e-3, "{:?}", p.ik_error);
            for w in j.windows(2) {
                let l = ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
                assert!((l - 10.0).abs() < 1e-3, "{bend}: a bone stretched to {l}");
            }
            assert!(
                sides(&p, [25.0, 15.0]).iter().all(|s| s * sign > 0.0),
                "{bend}: {j:?}"
            );
        }
        // A target behind the root, closer than the chain is long: the tail folds back to it.
        let p = r.pose(None, 0.0, &[(0, [-12.0, -6.0], 1.0)]);
        assert!(
            close(joints(&p)[4], [-12.0, -6.0], 1e-3) && p.ik_error[0] < 1e-3,
            "{:?}",
            joints(&p)
        );
        assert!(sides(&p, [-12.0, -6.0]).iter().all(|s| *s > 0.0));
        // Out of reach: straight at the target, 60 short.
        let p = r.pose(None, 0.0, &[(0, [0.0, -100.0], 1.0)]);
        assert!(close(joints(&p)[4], [0.0, -40.0], 1e-3), "{:?}", joints(&p));
        assert!((p.ik_error[0] - 60.0).abs() < 1e-3);
        // Weight 0 keeps the setup pose.
        let p = r.pose(None, 0.0, &[(0, [25.0, 15.0], 0.0)]);
        assert!(close(joints(&p)[4], [40.0, 0.0], 1e-4));
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        // A chain with a gap is refused.
        let e = Rig2dAsset::parse(&TAIL.replace("bones t1 t2 t3 t4", "bones t1 t3 t4"))
            .expect_err("t3 is not t1's child")
            .reason()
            .to_string();
        assert!(e.contains("previous one's child"), "{e}");
    }

    fn near(m: &Affine2, want: [f32; 6], eps: f32) -> bool {
        [m.a, m.b, m.c, m.d, m.tx, m.ty]
            .iter()
            .zip(want)
            .all(|(g, w)| (g - w).abs() < eps)
    }

    #[test]
    fn a_transform_constraint_moves_its_bones_by_the_mix_towards_the_target() {
        let text = r#"type rig2d
version 1
---
texture "t.png"
bone root {
}
bone target {
  parent root
  at 10 5
  rotation 90
  scale 2 1
}
bone arm {
  parent root
}
bone hand {
  parent arm
  at 10 0
}
transform follow {
  bones arm
  target target
  at 2 0
  mix 0.5 0.5 1 0
}
"#;
        let r = Rig2dAsset::parse(text).expect("a transform constraint");
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        let p = r.pose(None, 0.0, &[]);
        // Worked by hand: half of the target's 90 degrees (45); half way to
        // the target's point (2, 0), which is (10, 5) + 2 * (0, 2) = (10, 9),
        // from (0, 0): (5, 4.5); the x axis's length all the way to the
        // target's 2, the y axis's kept at the target's 1; no shear mix.
        let h = std::f32::consts::FRAC_1_SQRT_2;
        let arm = p.world[2];
        assert!(
            near(&arm, [2.0 * h, 2.0 * h, -h, h, 5.0, 4.5], 1e-5),
            "{arm:?}"
        );
        // The child follows: (10, 0) in the arm, 20 along the 45-degree axis.
        let hand = p.world[3];
        assert!(
            (hand.tx - (5.0 + 20.0 * h)).abs() < 1e-4 && (hand.ty - (4.5 + 20.0 * h)).abs() < 1e-4,
            "{hand:?}"
        );
        let bad = Rig2dAsset::parse(&text.replace("bones arm", "bones target"))
            .expect_err("its own target");
        assert!(
            bad.reason().contains("not one of its bones"),
            "{}",
            bad.reason()
        );
    }

    /// A path of one curve from (0, 0) through the handles (0, 10) and
    /// (10, 10) to (10, 0) on bone `rail`, two bones placed on it.
    fn path_rig(rail: &str, along: &str) -> Rig2dAsset {
        Rig2dAsset::parse(&format!(
            r#"type rig2d
version 1
---
texture "t.png"
bone root {{
}}
bone rail {{
  parent root
  {rail}
}}
bone a {{
  parent root
  at -40 3
  rotation 120
  length 8
}}
bone b {{
  parent a
  at 8 0
  length 8
}}
slot rail {{
  bone rail
  attachment rail
}}
path rail {{
  slot rail
  {along}
}}
along follow {{
  path rail
  bones a b
}}
"#
        ))
        .expect("a path constraint")
    }

    #[test]
    fn a_path_constraint_places_its_bones_along_the_curve() {
        // By the curve's parameter (`constant false`): spacing 25 percent of
        // its length 20 puts a at 5 (t 0.25) and b at 10 (t 0.5). Worked by
        // hand from the Bernstein form: B(0.25) = (1.5625, 5.625) with the
        // tangent (3.75, 5), 53.130102 degrees; B(0.5) = (5, 7.5) with the
        // tangent (5, 0), 0 degrees.
        let curve = "constant false\n  lengths 20\n  vertex 0 -10\n  vertex 0 0\n  vertex 0 10\n  vertex 10 10\n  vertex 10 0\n  vertex 10 -10";
        let mut r = path_rig("", curve);
        r.paths[0].spacing = 0.25;
        r.paths[0].spacing_mode = PathSpacing::Percent;
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        let p = r.pose(None, 0.0, &[]);
        // (3.75, 5) over its length 6.25.
        let (s, c) = (0.8f32, 0.6f32);
        assert!(
            near(&p.world[2], [c, s, -s, c, 1.5625, 5.625], 1e-4),
            "{:?}",
            p.world[2]
        );
        assert!(
            near(&p.world[3], [1.0, 0.0, 0.0, 1.0, 5.0, 7.5], 1e-4),
            "{:?}",
            p.world[3]
        );

        // By arc length on a straight path turned 30 degrees with its bone:
        // `length` spacing puts a at position 3 and b one bone length plus
        // spacing 2 further, at 13; both along the line.
        let line = "vertex -10 0\n  vertex 0 0\n  vertex 10 0\n  vertex 20 0\n  vertex 30 0\n  vertex 40 0";
        let mut r = path_rig("rotation 30", line);
        r.paths[0].position = 0.1;
        r.paths[0].spacing = 2.0;
        let p = r.pose(None, 0.0, &[]);
        let (s, c) = (0.5f32, 0.75f32.sqrt());
        assert!(
            near(&p.world[2], [c, s, -s, c, 3.0 * c, 3.0 * s], 1e-4),
            "{:?}",
            p.world[2]
        );
        assert!(
            near(&p.world[3], [c, s, -s, c, 13.0 * c, 13.0 * s], 1e-4),
            "{:?}",
            p.world[3]
        );

        // Chain: each bone turns towards the next place and the next bone
        // starts at its tip (3 + 8 = 11); chainscale stretches a to reach
        // 13 from 3 (10 of its 8 pixels, 1.25) and b to reach 23 from 13.
        r.paths[0].rotate = PathRotate::Chain;
        let p = r.pose(None, 0.0, &[]);
        assert!(
            near(&p.world[3], [c, s, -s, c, 11.0 * c, 11.0 * s], 1e-4),
            "{:?}",
            p.world[3]
        );
        r.paths[0].rotate = PathRotate::ChainScale;
        let p = r.pose(None, 0.0, &[]);
        let k = 1.25;
        assert!(
            near(&p.world[2], [k * c, k * s, -s, c, 3.0 * c, 3.0 * s], 1e-4),
            "{:?}",
            p.world[2]
        );
        assert!(
            near(&p.world[3], [k * c, k * s, -s, c, 13.0 * c, 13.0 * s], 1e-4),
            "{:?}",
            p.world[3]
        );

        // Translate mix 0.5: half way from a's own origin (-40, 3).
        r.paths[0].rotate = PathRotate::Tangent;
        r.paths[0].mix = [1.0, 0.5];
        let a = r.pose(None, 0.0, &[]).world[2];
        assert!(
            (a.tx - (-40.0 + 3.0 * c) / 2.0).abs() < 1e-4
                && (a.ty - (3.0 + 3.0 * s) / 2.0).abs() < 1e-4,
            "{a:?}"
        );
        // A path its slot does not show places nothing.
        r.slots[0].attachment = None;
        let a = r.pose(None, 0.0, &[]).world[2];
        assert!(
            (a.tx + 40.0).abs() < 1e-6 && (a.ty - 3.0).abs() < 1e-6,
            "{a:?}"
        );
    }

    #[test]
    fn a_skin_swaps_the_default_attachments_wherever_they_show() {
        let text = ARM.replace(
            "ik reach {",
            "region plate-gold {\n  slot plate\n  rect 10 0 10 4\n  at 10 0\n  rotation 90\n}\nskin gold {\n  swap plate plate-gold\n  swap skin none\n}\nik reach {",
        );
        let r = Rig2dAsset::parse(&text).expect("a skin");
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        let gold = r
            .attachments
            .iter()
            .position(|a| a.name == "plate-gold")
            .unwrap();
        let plate = r
            .attachments
            .iter()
            .position(|a| a.name == "plate")
            .unwrap();
        // Default: the mesh and the plate; gold: no mesh and the gold plate.
        let pose = |skin: u32, t: f32| {
            r.pose_in_skin(Some(&r.clips[0]), t, &[(0, [0.0, 0.0], 0.0)], skin)
                .slots
        };
        assert_eq!(pose(0, 0.2), [Some(0), Some(plate)]);
        assert_eq!(pose(1, 0.2), [None, Some(gold)]);
        // A skin index past the rig's is the default.
        assert_eq!(pose(7, 0.2), [Some(0), Some(plate)]);
        // The clip hides the plate at 0.5: in every skin.
        assert_eq!(pose(1, 0.6), [None, None]);
        // The gold plate's texels: u from 10 of 64.
        let p = r.pose_in_skin(None, 0.0, &[(0, [0.0, 0.0], 0.0)], 1);
        let (v, _) = r.triangles(&p);
        assert_eq!(v.len(), 4);
        assert!((v[0].uv[0] - 10.0 / 64.0).abs() < 1e-6, "{:?}", v[0]);
        let bad = |swap: &str| {
            Rig2dAsset::parse(&text.replace("  swap skin none", swap))
                .expect_err(swap)
                .reason()
                .to_string()
        };
        assert!(bad("  swap skin plate").contains("stays in its slot"));
        assert!(bad("  swap skin cape").contains("no attachment"));
    }

    #[test]
    fn mistakes_are_refused_with_their_line() {
        let bad = |from: &str, to: &str| {
            Rig2dAsset::parse(&ARM.replace(from, to))
                .expect_err(to)
                .reason()
                .to_string()
        };
        assert!(bad("  parent upper\n  at 20 0", "  parent nobody\n  at 20 0").contains("nobody"));
        assert!(bad("upper 20 -2 0.5 lower", "upper 20 -2 0.6 lower").contains("sum"));
        assert!(bad("key 1 0 2 0 0 0 0 0 4", "key 1 0 2 0 0").contains("offsets"));
        assert!(bad("bones upper lower", "bones lower upper").contains("child"));
        assert!(bad("duration 1", "duration 0").contains("duration"));
    }
}
