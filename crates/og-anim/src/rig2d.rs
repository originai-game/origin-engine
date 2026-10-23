//! 2D skeletal animation: the `type rig2d` asset (bones, slots in draw
//! order, region and weighted-mesh attachments, IK constraints, clips with
//! bone, slot and deform keys and events), its pose at a clip time and the
//! textured triangles that pose draws ([`Rig2dAsset::triangles`]).
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
//! bone hip { parent root; at 0 40; rotation 90; scale 1 1; length 20 }
//! slot torso { bone hip; attachment torso; color 1 1 1 1 }
//! region torso { slot torso; rect 0 0 24 40; at 10 0; rotation -90; size 24 40 }
//! mesh cape {
//!   slot cape
//!   vertex u v x y                      # unweighted: x y in the slot's bone
//!   vertex u v bone x y w [bone x y w]  # weighted: x y in each bone, weights summing to 1
//!   triangles 0 1 2 2 3 0
//! }
//! ik leg { bones thigh shin; target foot-target; bend clockwise; weight 1 }
//! clip walk {
//!   duration 0.8
//!   loop true
//!   bone thigh { rotate 0 20; rotate 0.4 -20; translate 0 0 0; scale 0.2 1 1.1 step }
//!   slot eye { attachment 0.5 eye-shut; attachment 0.6 eye-open; color 0 1 1 1 1 }
//!   deform cape { key 0 0 0 0 0; key 0.4 0 -3 0 -6 }
//!   event 0.2 "step"
//! }
//! ```
//!
//! (Entries are one per line in a file; `;` above only saves space.) A
//! bone key is relative to the setup pose: `rotate` adds degrees,
//! `translate` adds pixels, `scale` multiplies; keys interpolate linearly
//! (rotation the shorter way round) unless marked `step`, which holds the
//! value until the next key. A `deform` key offsets every influence of the
//! mesh's vertices in order (one x y pair per influence; an unweighted
//! vertex has one), in that influence's bone space. An `ik` constraint
//! turns one bone to point at the target bone's origin, or two (a parent
//! and its child) so the child's tip reaches it, bending clockwise or
//! counter-clockwise; a game moves the target with og.IkTarget2D.
//!
//! **Converting another format.** DragonBones JSON is imported by the cook
//! (og-cook's `rig2d.rs`). Spine's format is not imported (its licence);
//! a converter maps it so: Spine `bones` (`x`, `y`, `rotation`, `scaleX`,
//! `scaleY`, `length`, y up, degrees counter-clockwise) to `bone` blocks
//! as they are; `slots` (`bone`, `attachment`, `color` hex to linear 0..1
//! after sRGB) to `slot` blocks in the same order; a `region` attachment's
//! `x`, `y`, `rotation`, `width`, `height` to `at`, `rotation`, `size`
//! with the atlas region's rectangle (pixels from the top left) as `rect`;
//! a `mesh` attachment's `uvs` (times the region's size, plus its corner)
//! and `vertices` (unweighted pairs, or Spine's `count, bone, x, y, weight`
//! groups) to `vertex` lines and `triangles` as they are; `ik` with one or
//! two `bones`, `target` and `bendPositive` true as `counterclockwise`;
//! an animation's `bones` timelines (`rotate` `value`, `translate` and
//! `scale` `x` `y`, `time` in seconds; `curve: "stepped"` as `step`,
//! Bézier curves as linear), `slots` `attachment` and `rgba`, `deform`
//! `offset` and `vertices` per influence, and `events`. Skew, path and
//! transform constraints, clipping and draw-order keys have no line.
//!
//! **Determinism.** Every angle goes through [`sincos`] and [`atan2`],
//! written with `+ - * /` and `sqrt` in f64, so a pose is the same bits on
//! every host.

use og_core::ogt::{Block, Document, Entry, Number};
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
    /// Degrees, counter-clockwise.
    pub rotation: f32,
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
    /// `rotation` degrees.
    Region {
        rect: [f32; 4],
        at: [f32; 2],
        rotation: f32,
        size: [f32; 2],
    },
    /// Triangles over weighted vertices.
    Mesh {
        vertices: Vec<MeshVertex2d>,
        triangles: Vec<u32>,
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
            AttachmentKind::Region { .. } => 0,
        }
    }
}

/// An IK constraint: `bones` (one, or a parent and its child) turn so the
/// last one's tip (one bone: its direction) reaches `target`'s origin.
#[derive(Clone, Debug, PartialEq)]
pub struct Ik2d {
    pub name: String,
    pub bones: Vec<usize>,
    pub target: usize,
    /// True bends the joint counter-clockwise (as seen with +y up).
    pub counterclockwise: bool,
    pub weight: f32,
}

/// A key: a time, a value and whether it holds until the next key.
#[derive(Clone, Debug, PartialEq)]
pub struct Key<T> {
    pub time: f32,
    pub value: T,
    pub step: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoneTimeline {
    pub bone: usize,
    pub rotate: Vec<Key<f32>>,
    pub translate: Vec<Key<[f32; 2]>>,
    pub scale: Vec<Key<[f32; 2]>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlotTimeline {
    pub slot: usize,
    pub attachment: Vec<(f32, Option<usize>)>,
    pub color: Vec<Key<[f32; 4]>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeformTimeline {
    pub attachment: usize,
    pub keys: Vec<Key<Vec<f32>>>,
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
    /// (time, name), in time order.
    pub events: Vec<(f32, String)>,
}

/// A 2D rig: the texture, bones, slots, attachments, constraints and clips.
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
    pub clips: Vec<Clip2d>,
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

/// Whether the entry ends with the word `step`.
fn stepped(e: &Entry) -> bool {
    e.values.last().and_then(|v| v.as_word()) == Some("step")
}

fn index(names: &[String], name: &str, what: &str, line: usize) -> Result<usize> {
    names
        .iter()
        .position(|n| n == name)
        .ok_or_else(|| parse_err(line, format!("no {what} `{name}`")))
}

impl Rig2dAsset {
    /// Reads a `type rig2d` text (the grammar is the module's docs).
    pub fn parse(text: &str) -> Result<Rig2dAsset> {
        let doc = Document::parse(text)?;
        if doc.ty != RIG2D_TYPE {
            return Err(Error::new(
                ErrorKind::Parse,
                format!("a 2D rig is `type rig2d`, not `{}`", doc.ty),
            ));
        }
        let mut rig = Rig2dAsset {
            texture: String::new(),
            size: [0, 0],
            filter: SheetFilter::Linear,
            unit: 1.0,
            bones: Vec::new(),
            slots: Vec::new(),
            attachments: Vec::new(),
            iks: Vec::new(),
            clips: Vec::new(),
        };
        let body = &doc.body.entries;
        for e in body {
            match e.key.as_str() {
                "texture" => {
                    rig.texture = e
                        .string(0)
                        .map_err(|_| parse_err(e.line, "texture \"path\""))?
                        .to_string()
                }
                "size" => rig.size = [num(e, 0)? as u32, num(e, 1)? as u32],
                "filter" => {
                    rig.filter = match e.word(0).unwrap_or("") {
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
                    rig.unit = num(e, 0)?;
                    if rig.unit <= 0.0 {
                        return Err(parse_err(e.line, "unit is above 0 (pixels per world unit)"));
                    }
                }
                "bone" | "slot" | "region" | "mesh" | "ik" | "clip" | "dragonbones" | "atlas" => {}
                k => return Err(parse_err(e.line, format!("unknown key `{k}` in a 2D rig"))),
            }
        }
        if rig.texture.is_empty() {
            return Err(Error::new(
                ErrorKind::Parse,
                "a 2D rig needs `texture \"path\"`",
            ));
        }
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
                                        format!(
                                            "bone {name}: parent `{p}` is not declared before it"
                                        ),
                                    )
                                })?);
                        }
                        "at" => (b.x, b.y) = (num(c, 0)?, num(c, 1)?),
                        "rotation" => b.rotation = num(c, 0)?,
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
                "a 2D rig needs at least one bone",
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
            for c in &blk.entries {
                match c.key.as_str() {
                    "bone" => bone = Some(index(&bone_names, &word(c, 0)?, "bone", c.line)?),
                    "attachment" => attachment = Some((word(c, 0)?, c.line)),
                    "color" => color = [num(c, 0)?, num(c, 1)?, num(c, 2)?, num(c, 3)?],
                    k => {
                        return Err(parse_err(
                            c.line,
                            format!("unknown key `{k}` in slot {name}"),
                        ));
                    }
                }
            }
            let bone =
                bone.ok_or_else(|| parse_err(e.line, format!("slot {name} needs `bone`")))?;
            rig.slots.push(Slot2d {
                name: name.clone(),
                bone,
                attachment: None,
                color,
            });
            slot_attachment.push(attachment);
            slot_names.push(name);
        }
        let mut attachment_names: Vec<String> = Vec::new();
        for e in body.iter().filter(|e| e.key == "region" || e.key == "mesh") {
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
            let kind = if e.key == "region" {
                let mut rect = None;
                let mut at = [0.0; 2];
                let mut rotation = 0.0;
                let mut size = None;
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "rect" => rect = Some([num(c, 0)?, num(c, 1)?, num(c, 2)?, num(c, 3)?]),
                        "at" => at = [num(c, 0)?, num(c, 1)?],
                        "rotation" => rotation = num(c, 0)?,
                        "size" => size = Some([num(c, 0)?, num(c, 1)?]),
                        k => {
                            return Err(parse_err(
                                c.line,
                                format!("unknown key `{k}` in region {name}"),
                            ));
                        }
                    }
                }
                let rect = rect.ok_or_else(|| {
                    parse_err(e.line, format!("region {name} needs `rect x y w h`"))
                })?;
                AttachmentKind::Region {
                    rect,
                    at,
                    rotation,
                    size: size.unwrap_or([rect[2], rect[3]]),
                }
            } else {
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
                                if c.values.len() < 6 || (c.values.len() - 2) % 4 != 0 {
                                    return Err(parse_err(
                                        c.line,
                                        format!(
                                            "mesh {name}: vertex u v x y, or vertex u v then bone x y weight per influence"
                                        ),
                                    ));
                                }
                                let mut out = Vec::new();
                                for k in (2..c.values.len()).step_by(4) {
                                    out.push(Influence {
                                        bone: index(&bone_names, &word(c, k)?, "bone", c.line)?,
                                        x: num(c, k + 1)?,
                                        y: num(c, k + 2)?,
                                        weight: num(c, k + 3)?,
                                    });
                                }
                                let total: f32 = out.iter().map(|i| i.weight).sum();
                                if (total - 1.0).abs() > 1e-3 {
                                    return Err(parse_err(
                                        c.line,
                                        format!("mesh {name}: the weights sum to {total}, not 1"),
                                    ));
                                }
                                out
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
                        k => {
                            return Err(parse_err(
                                c.line,
                                format!("unknown key `{k}` in mesh {name}"),
                            ));
                        }
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
                    "bend" => {
                        ik.counterclockwise = match word(c, 0)?.as_str() {
                            "clockwise" => false,
                            "counterclockwise" => true,
                            w => {
                                return Err(parse_err(
                                    c.line,
                                    format!("bend is clockwise or counterclockwise, not `{w}`"),
                                ));
                            }
                        }
                    }
                    "weight" => ik.weight = num(c, 0)?.clamp(0.0, 1.0),
                    k => return Err(parse_err(c.line, format!("unknown key `{k}` in ik {name}"))),
                }
            }
            match ik.bones.as_slice() {
                [_] => {}
                [p, c] if rig.bones[*c].parent == Some(*p) => {}
                [_, _] => {
                    return Err(parse_err(
                        e.line,
                        format!("ik {name}: the second bone is the first's child"),
                    ));
                }
                _ => {
                    return Err(parse_err(
                        e.line,
                        format!("ik {name}: `bones` names one or two bones"),
                    ));
                }
            }
            if ik.target == usize::MAX {
                return Err(parse_err(e.line, format!("ik {name} needs `target`")));
            }
            rig.iks.push(ik);
        }
        for e in body.iter().filter(|e| e.key == "clip") {
            rig.clips.push(parse_clip(
                e,
                &rig,
                &bone_names,
                &slot_names,
                &attachment_names,
            )?);
        }
        Ok(rig)
    }

    /// The canonical text: every value spelled out. Parses back equal.
    pub fn to_text(&self) -> String {
        let n = num_text;
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
        s += &format!("unit {}\n", n(self.unit));
        for b in &self.bones {
            s += &format!("bone {} {{\n", quote(&b.name));
            if let Some(p) = b.parent {
                s += &format!("  parent {}\n", quote(&self.bones[p].name));
            }
            s += &format!("  at {} {}\n  rotation {}\n", n(b.x), n(b.y), n(b.rotation));
            s += &format!(
                "  scale {} {}\n  length {}\n}}\n",
                n(b.scale[0]),
                n(b.scale[1]),
                n(b.length)
            );
        }
        for sl in &self.slots {
            s += &format!(
                "slot {} {{\n  bone {}\n",
                quote(&sl.name),
                quote(&self.bones[sl.bone].name)
            );
            if let Some(a) = sl.attachment {
                s += &format!("  attachment {}\n", quote(&self.attachments[a].name));
            }
            let [r, g, b, a] = sl.color;
            s += &format!("  color {} {} {} {}\n}}\n", n(r), n(g), n(b), n(a));
        }
        for a in &self.attachments {
            let slot = quote(&self.slots[a.slot].name);
            match &a.kind {
                AttachmentKind::Region {
                    rect,
                    at,
                    rotation,
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
                        "  at {} {}\n  rotation {}\n",
                        n(at[0]),
                        n(at[1]),
                        n(*rotation)
                    );
                    s += &format!("  size {} {}\n}}\n", n(size[0]), n(size[1]));
                }
                AttachmentKind::Mesh {
                    vertices,
                    triangles,
                } => {
                    s += &format!("mesh {} {{\n  slot {slot}\n", quote(&a.name));
                    let bone = self.slots[a.slot].bone;
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
                                        quote(&self.bones[i.bone].name),
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
            }
        }
        for ik in &self.iks {
            s += &format!("ik {} {{\n  bones", quote(&ik.name));
            for b in &ik.bones {
                s += &format!(" {}", quote(&self.bones[*b].name));
            }
            s += &format!(
                "\n  target {}\n  bend {}\n  weight {}\n}}\n",
                quote(&self.bones[ik.target].name),
                if ik.counterclockwise {
                    "counterclockwise"
                } else {
                    "clockwise"
                },
                n(ik.weight)
            );
        }
        let step = |k: bool| if k { " step" } else { "" };
        for c in &self.clips {
            s += &format!(
                "clip {} {{\n  duration {}\n  loop {}\n",
                quote(&c.name),
                n(c.duration),
                c.looping
            );
            for t in &c.bones {
                s += &format!("  bone {} {{\n", quote(&self.bones[t.bone].name));
                for k in &t.rotate {
                    s += &format!("    rotate {} {}{}\n", n(k.time), n(k.value), step(k.step));
                }
                for k in &t.translate {
                    s += &format!(
                        "    translate {} {} {}{}\n",
                        n(k.time),
                        n(k.value[0]),
                        n(k.value[1]),
                        step(k.step)
                    );
                }
                for k in &t.scale {
                    s += &format!(
                        "    scale {} {} {}{}\n",
                        n(k.time),
                        n(k.value[0]),
                        n(k.value[1]),
                        step(k.step)
                    );
                }
                s += "  }\n";
            }
            for t in &c.slots {
                s += &format!("  slot {} {{\n", quote(&self.slots[t.slot].name));
                for (time, a) in &t.attachment {
                    let a = a.map_or("none".to_string(), |a| quote(&self.attachments[a].name));
                    s += &format!("    attachment {} {a}\n", n(*time));
                }
                for k in &t.color {
                    let [r, g, b, a] = k.value;
                    s += &format!(
                        "    color {} {} {} {} {}{}\n",
                        n(k.time),
                        n(r),
                        n(g),
                        n(b),
                        n(a),
                        step(k.step)
                    );
                }
                s += "  }\n";
            }
            for t in &c.deforms {
                s += &format!(
                    "  deform {} {{\n",
                    quote(&self.attachments[t.attachment].name)
                );
                for k in &t.keys {
                    s += &format!("    key {}", n(k.time));
                    for v in &k.value {
                        s += &format!(" {}", n(*v));
                    }
                    s += &format!("{}\n", step(k.step));
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
    /// replace the target bone's origin and the rig's weight).
    pub fn pose(
        &self,
        clip: Option<&Clip2d>,
        time: f32,
        targets: &[(usize, [f32; 2], f32)],
    ) -> Pose2d {
        let mut local: Vec<[f32; 5]> = self
            .bones
            .iter()
            .map(|b| [b.x, b.y, b.rotation, b.scale[0], b.scale[1]])
            .collect();
        let mut slots: Vec<Option<usize>> = self.slots.iter().map(|s| s.attachment).collect();
        let mut colors: Vec<[f32; 4]> = self.slots.iter().map(|s| s.color).collect();
        let mut deforms: Vec<(usize, Vec<f32>)> = Vec::new();
        if let Some(c) = clip {
            for t in &c.bones {
                let l = &mut local[t.bone];
                if let Some(r) = sample(&t.rotate, time, |a, b, f| {
                    let mut d = b - a;
                    d -= 360.0 * ((d + 180.0) / 360.0).floor();
                    a + d * f
                }) {
                    l[2] += r;
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
                if let Some(v) = sample(&t.color, time, |a, b, f| {
                    [0, 1, 2, 3].map(|i| a[i] + (b[i] - a[i]) * f)
                }) {
                    colors[t.slot] = v;
                }
            }
            for t in &c.deforms {
                if let Some(v) = sample(&t.keys, time, |a: &Vec<f32>, b: &Vec<f32>, f| {
                    a.iter().zip(b).map(|(x, y)| x + (y - x) * f).collect()
                }) {
                    deforms.push((t.attachment, v));
                }
            }
        }
        let mut world = self.world(&local);
        let mut ik = Vec::with_capacity(self.iks.len());
        for (i, c) in self.iks.iter().enumerate() {
            let (target, weight) = match targets.iter().find(|t| t.0 == i) {
                Some(&(_, t, w)) => (t, w.clamp(0.0, 1.0)),
                None => {
                    let w = world[c.target];
                    ([w.tx, w.ty], c.weight)
                }
            };
            let t = [f64::from(target[0]), f64::from(target[1])];
            let w = f64::from(weight);
            let tip_of = |world: &[Affine2], b: usize| world[b].apply(self.bones[b].length, 0.0);
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
                    let side = if c.counterclockwise { 1.0 } else { -1.0 };
                    let want = atan2(dy, dx) + side * a;
                    let now = atan2(o2[1] - o1[1], o2[0] - o1[0]);
                    local[*p][2] += (wrap(want - now) * w / DEG) as f32;
                    world = self.world(&local);
                    let o2 = [f64::from(world[*ch].tx), f64::from(world[*ch].ty)];
                    let tip = tip_of(&world, *ch).map(f64::from);
                    let want = atan2(t[1] - o2[1], t[0] - o2[0]);
                    let now = atan2(tip[1] - o2[1], tip[0] - o2[0]);
                    local[*ch][2] += (wrap(want - now) * w / DEG) as f32;
                }
                _ => {}
            }
            world = self.world(&local);
            let last = *c.bones.last().expect("one or two bones");
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
        Pose2d {
            world,
            slots,
            colors,
            deforms,
            ik_error: ik,
        }
    }

    fn world(&self, local: &[[f32; 5]]) -> Vec<Affine2> {
        let mut world: Vec<Affine2> = Vec::with_capacity(self.bones.len());
        for (b, l) in self.bones.iter().zip(local) {
            let m = Affine2::trs(l[0], l[1], l[2], l[3], l[4]);
            world.push(match b.parent {
                Some(p) => world[p].mul(&m),
                None => m,
            });
        }
        world
    }

    /// The triangles a pose draws, in slot order: corners in world units
    /// of the rig's space (pixels over `unit`), uvs (0..1, v down) and
    /// colours, with triangle indices.
    pub fn triangles(&self, pose: &Pose2d) -> (Vec<RigVertex>, Vec<u32>) {
        let (w, h) = (self.size[0].max(1) as f32, self.size[1].max(1) as f32);
        let inv = 1.0 / self.unit;
        let mut out = Vec::new();
        let mut indices = Vec::new();
        for (s, slot) in self.slots.iter().enumerate() {
            let Some(a) = pose.slots[s] else { continue };
            let color = pose.colors[s];
            let base = out.len() as u32;
            match &self.attachments[a].kind {
                AttachmentKind::Region {
                    rect,
                    at,
                    rotation,
                    size,
                } => {
                    let m =
                        pose.world[slot.bone].mul(&Affine2::trs(at[0], at[1], *rotation, 1.0, 1.0));
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
                            let q = pose.world[i.bone].apply(i.x + dx, i.y + dy);
                            p[0] += q[0] * i.weight;
                            p[1] += q[1] * i.weight;
                        }
                        out.push(RigVertex {
                            position: [p[0] * inv, p[1] * inv],
                            uv: [v.uv[0] / w, v.uv[1] / h],
                            color,
                        });
                    }
                    indices.extend(triangles.iter().map(|t| base + t));
                }
            }
        }
        (out, indices)
    }
}

/// The length of (x, y).
fn sq(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

fn lerp2(a: &[f32; 2], b: &[f32; 2], f: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
}

/// The value of keys at `t` (`None` without keys).
fn sample<T: Clone>(keys: &[Key<T>], t: f32, lerp: impl Fn(&T, &T, f32) -> T) -> Option<T> {
    let first = keys.first()?;
    if t <= first.time {
        return Some(first.value.clone());
    }
    let i = keys.iter().rposition(|k| k.time <= t).unwrap_or(0);
    let k = &keys[i];
    match keys.get(i + 1) {
        Some(n) if !k.step && n.time > k.time => {
            Some(lerp(&k.value, &n.value, (t - k.time) / (n.time - k.time)))
        }
        _ => Some(k.value.clone()),
    }
}

/// A rig's pose: each bone's transform in the rig's space (pixels), what
/// each slot shows and its colour, the mesh deforms, and each IK
/// constraint's remaining distance (pixels) after solving.
#[derive(Clone, Debug, PartialEq)]
pub struct Pose2d {
    pub world: Vec<Affine2>,
    pub slots: Vec<Option<usize>>,
    pub colors: Vec<[f32; 4]>,
    pub deforms: Vec<(usize, Vec<f32>)>,
    pub ik_error: Vec<f32>,
}

/// A corner of a drawn rig.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

fn parse_clip(
    e: &Entry,
    rig: &Rig2dAsset,
    bones: &[String],
    slots: &[String],
    attachments: &[String],
) -> Result<Clip2d> {
    let name = name_of(e)?;
    let blk = block(e)?;
    let mut clip = Clip2d {
        name: name.clone(),
        duration: 0.0,
        looping: true,
        bones: Vec::new(),
        slots: Vec::new(),
        deforms: Vec::new(),
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
                    bone: index(bones, &name_of(c)?, "bone", c.line)?,
                    ..Default::default()
                };
                for k in &block(c)?.entries {
                    match k.key.as_str() {
                        "rotate" => t.rotate.push(Key {
                            time: time(k)?,
                            value: num(k, 1)?,
                            step: stepped(k),
                        }),
                        "translate" => t.translate.push(Key {
                            time: time(k)?,
                            value: [num(k, 1)?, num(k, 2)?],
                            step: stepped(k),
                        }),
                        "scale" => t.scale.push(Key {
                            time: time(k)?,
                            value: [num(k, 1)?, num(k, 2)?],
                            step: stepped(k),
                        }),
                        x => {
                            return Err(parse_err(
                                k.line,
                                format!("unknown key `{x}` in a bone timeline"),
                            ));
                        }
                    }
                }
                sorted(&t.rotate.iter().map(|k| k.time).collect::<Vec<_>>(), c.line)?;
                sorted(
                    &t.translate.iter().map(|k| k.time).collect::<Vec<_>>(),
                    c.line,
                )?;
                sorted(&t.scale.iter().map(|k| k.time).collect::<Vec<_>>(), c.line)?;
                clip.bones.push(t);
            }
            "slot" => {
                let slot = index(slots, &name_of(c)?, "slot", c.line)?;
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
                                let i = index(attachments, &a, "attachment", k.line)?;
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
                        "color" => t.color.push(Key {
                            time: time(k)?,
                            value: [num(k, 1)?, num(k, 2)?, num(k, 3)?, num(k, 4)?],
                            step: stepped(k),
                        }),
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
                sorted(&t.color.iter().map(|k| k.time).collect::<Vec<_>>(), c.line)?;
                clip.slots.push(t);
            }
            "deform" => {
                let a = index(attachments, &name_of(c)?, "attachment", c.line)?;
                let len = 2 * rig.attachments[a].influences();
                if len == 0 {
                    return Err(parse_err(
                        c.line,
                        format!(
                            "clip {name}: deform names region {}; only a mesh deforms",
                            attachments[a]
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
                    let count = k.values.len() - 1 - usize::from(stepped(k));
                    if count != len {
                        return Err(parse_err(
                            k.line,
                            format!(
                                "deform {}: {count} offsets for {} influences (x y each)",
                                attachments[a],
                                len / 2
                            ),
                        ));
                    }
                    keys.push(Key {
                        time: time(k)?,
                        value: (1..=count).map(|i| num(k, i)).collect::<Result<_>>()?,
                        step: stepped(k),
                    });
                }
                sorted(&keys.iter().map(|k| k.time).collect::<Vec<_>>(), c.line)?;
                clip.deforms.push(DeformTimeline {
                    attachment: a,
                    keys,
                });
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
