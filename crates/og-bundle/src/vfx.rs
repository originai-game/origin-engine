//! Composed effects as text: a file of `type vfx`, one effect as a timeline
//! of elements the runtime plays from an og.Vfx's start tick. The cook
//! checks it with this parser (and that what it names is in the bundle);
//! the runtime loads it with it.
//!
//! Its lines are the rows of [`VFX_LINES`] (the header lines and the
//! elements) and [`ELEMENT_LINES`] (an element's block, with the elements
//! that take each), which the parser holds every line to and `og api gen`
//! writes into the `effect-files` guide.
//!
//! Every element takes `at`, `offset`, `rotate`, `move` and `accelerate`
//! (its offset moves from its start: an emitter spawns along the path, a
//! mesh, trail or light goes with it); those that last take
//! `life` (seconds) or `until` (an end time). A value is a number,
//! `random <min> <max>` (fixed per effect instance) or `curve <t> <v> ...`
//! (over the element's life, at most 8 keys). Times are seconds from the
//! effect's start; the tick they land on is `start + round(time × rate)`.
//! The shared lines (`duration`, `loop`, `event`, `impact`) mean the same in
//! a `type vfx2d` pixel effect. Every og.FlipbookEvent a `flipbook`
//! element's clip fires (a vfx2d's `impact`) is passed on as an
//! og.VfxEvent of the effect with the same name, so a file whose impact a
//! pixel effect carries needs no `impact` line of its own.

mod play;

pub use play::{VfxCue, VfxDraw, VfxDrawKind, VfxInstance, element_id, hue_rotated, random};

use crate::effect::{flag, nums, one, value};
use crate::grammar::{Line, find, keys};
use og_core::ogt::{Document, Entry, Value};
use og_render_data::{EffectValue, ParticleBlend};

/// Every line of a composed effect's body: the header lines, then the
/// elements, each a block (`crate::grammar`).
pub const VFX_LINES: &[Line] = &[
    Line::new(
        "duration",
        "<seconds>",
        "",
        "How long the effect lasts; its elements end within it. Required.",
        "duration 1.2",
    ),
    Line::new(
        "loop",
        "true | false",
        "false",
        "Start over at the end.",
        "loop true",
    ),
    Line::new(
        "impact",
        "<seconds>",
        "",
        "The same as `event <seconds> \"impact\"`; at most one.",
        "impact 0.25",
    ),
    Line::new(
        "event",
        "<seconds> \"<name>\"",
        "",
        "An og.VfxEvent with this name at this time.",
        "event 0.8 \"linger\"",
    ),
    Line::new(
        "param",
        "<name> <default>",
        "",
        "A number the game sets per instance (og.Vfx.params, in the order declared; at most 4): elements read it with `when` and `amount`.",
        "param power 1",
    ),
    Line::new(
        "emitter",
        "\"<id>\" { ... }",
        "",
        "An emitter asset (§1) playing its own cycle; `until` stops its emission and its particles live out.",
        "emitter \"fx/sparks.ogt\" {\n  at 0.25\n  until 0.5\n}",
    ),
    Line::new(
        "mesh",
        "sphere | quad | ring | box | cylinder | \"<model id>\" { ... }",
        "",
        "A mesh drawn with an effect material (§2); a ring builds its own shape at load.",
        "mesh ring {\n  at 0.25\n  life 0.4\n  effect \"fx/shock.ogt\"\n}",
    ),
    Line::new(
        "trail",
        "{ ... }",
        "",
        "A trail behind the element's point.",
        "trail {\n  at 0\n  until 0.6\n}",
    ),
    Line::new(
        "beam",
        "{ ... }",
        "",
        "A beam from the element's point to `to`.",
        "beam {\n  at 0.1\n  until 0.6\n  to 0 4 0\n}",
    ),
    Line::new(
        "light",
        "{ ... }",
        "",
        "A point light.",
        "light {\n  at 0.25\n  life 0.4\n}",
    ),
    Line::new(
        "decal",
        "{ ... }",
        "",
        "A decal projected down the element's -y onto every lit surface in its box: its `texture` and `normal` map, or a square of its colour without them.",
        "decal {\n  at 0.25\n  life 4\n  texture \"fx/scorch.png\"\n}",
    ),
    Line::new(
        "glow",
        "{ ... }",
        "",
        "A soft bright glow facing the eye (version 1 named it `flash`).",
        "glow {\n  at 0.25\n  life 0.15\n}",
    ),
    Line::new(
        "shake",
        "{ ... }",
        "",
        "og.camera.shake, at once.",
        "shake {\n  at 0.25\n  trauma 0.3\n}",
    ),
    Line::new(
        "sound",
        "\"<id>\" { ... }",
        "",
        "A sound asset, played at once by a child entity with og.AudioSource, cut when the effect despawns.",
        "sound \"sfx/boom.wav\" {\n  at 0.25\n}",
    ),
    Line::new(
        "flipbook",
        "\"<id>\" { ... }",
        "",
        "A camera-facing og.Sprite with og.Flipbook playing a sheet's first clip once (a `type vfx2d` pixel effect or any og.load.flipbook sheet); its og.FlipbookEvent events, the impact among them, reach the game as this effect's og.VfxEvent.",
        "flipbook \"fx/slash.ogt\" {\n  at 0.1\n}",
    ),
];

const MESH: &[&str] = &["mesh"];
const TRAIL: &[&str] = &["trail"];
const BEAM: &[&str] = &["beam"];
const LIGHT: &[&str] = &["light"];
const DECAL: &[&str] = &["decal"];
const RING: &[&str] = &["ring"];

/// Every line of an element's block, with the elements that take it.
pub const ELEMENT_LINES: &[Line] = &[
    Line::of(
        &[],
        "at",
        "<seconds>",
        "0",
        "When it starts, from the effect's start.",
        "at 0.25",
    ),
    Line::of(
        &[],
        "life",
        "<seconds>",
        "",
        "How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either).",
        "life 0.4",
    ),
    Line::of(
        &[],
        "until",
        "<seconds>",
        "",
        "When it ends, instead of `life`.",
        "until 0.6",
    ),
    Line::of(
        &[],
        "offset",
        "<x> <y> <z>",
        "0 0 0",
        "Where it starts, in the effect's space.",
        "offset 0 0.5 0",
    ),
    Line::of(
        &[],
        "rotate",
        "<x> <y> <z>",
        "0 0 0",
        "Degrees, in the effect's space.",
        "rotate -90 0 0",
    ),
    Line::of(
        &[],
        "move",
        "<x> <y> <z>",
        "0 0 0",
        "Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it.",
        "move 0 2 0",
    ),
    Line::of(
        &[],
        "accelerate",
        "<x> <y> <z>",
        "0 0 0",
        "Metres per second squared.",
        "accelerate 0 -1 0",
    ),
    Line::of(
        &[],
        "when",
        "<param>",
        "",
        "Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground).",
        "when power",
    ),
    Line::of(
        &[],
        "amount",
        "<param>",
        "",
        "Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume.",
        "amount power",
    ),
    Line::of(
        &["mesh", "glow"],
        "stretch",
        "<param>",
        "",
        "Multiplies its length alone by the param (at least 0), along its own y after `rotate`: a jet plume that lengthens with thrust without growing wider. Applied with `amount`, which scales it whole.",
        "stretch power",
    ),
    Line::of(
        &["emitter"],
        "seed",
        "<u32>",
        "0",
        "Added to the effect's seed.",
        "seed 3",
    ),
    Line::of(
        &["mesh", "trail", "beam"],
        "effect",
        "\"<id>\"",
        "",
        "Its effect material (§2); a mesh needs one.",
        "effect \"fx/shock.ogt\"",
    ),
    Line::of(
        MESH,
        "scale",
        "<value>",
        "1",
        "Uniform scale over its life.",
        "scale curve 0 0.5 1 3",
    ),
    Line::of(
        MESH,
        "spin",
        "<x> <y> <z>",
        "0 0 0",
        "Degrees per second.",
        "spin 0 180 0",
    ),
    Line::of(
        MESH,
        "face",
        "camera",
        "",
        "Turned to face the eye as a billboard is (a glow ring seen from any side): its spin and scale kept, the effect's and its own rotation left out. Without it a mesh lies as it is placed.",
        "face camera",
    ),
    Line::of(
        &["mesh", "flipbook"],
        "blend",
        crate::grammar::BLENDS,
        "",
        "A mesh adds by default, a flipbook blends alpha; a flipbook takes additive or alpha only.",
        "blend alpha",
    ),
    Line::of(
        &[
            "mesh", "trail", "beam", "decal", "glow", "flipbook", "light",
        ],
        "color",
        "<r> <g> <b> [<a>]",
        "1 1 1 1",
        "A linear colour; above 1 blooms (a light's alpha is ignored).",
        "color 4 3 2",
    ),
    Line::of(
        TRAIL,
        "width",
        "<head> [<tail>]",
        "0.2 0",
        "Metres across at its head and its tail.",
        "width 0.3 0.05",
    ),
    Line::of(
        BEAM,
        "width",
        "<metres>",
        "0.2",
        "Metres across.",
        "width 0.4",
    ),
    Line::of(
        TRAIL,
        "lifetime",
        "<seconds>",
        "0.4",
        "How long a point stays in the trail.",
        "lifetime 0.6",
    ),
    Line::of(
        TRAIL,
        "orbit",
        "<radius> <turns per second>",
        "0 0",
        "Circles the element's y.",
        "orbit 0.6 1.5",
    ),
    Line::of(
        TRAIL,
        "rise",
        "<metres per second>",
        "0",
        "Climbs the element's y.",
        "rise 0.8",
    ),
    Line::of(
        BEAM,
        "to",
        "<x> <y> <z>",
        "0 1 0",
        "Its far end, from the element's point.",
        "to 0 4 0",
    ),
    Line::of(
        BEAM,
        "segments",
        "<n>",
        "1",
        "1 to 256: more let `jitter` bend it.",
        "segments 8",
    ),
    Line::of(
        BEAM,
        "jitter",
        "<metres>",
        "0",
        "Sideways jitter, new every tick (lightning).",
        "jitter 0.1",
    ),
    Line::of(
        LIGHT,
        "intensity",
        "<value>",
        "1",
        "Candela over its life.",
        "intensity curve 0 12 1 0",
    ),
    Line::of(
        LIGHT,
        "distance",
        "<metres>",
        "5",
        "Where it reaches zero.",
        "distance 6",
    ),
    Line::of(
        DECAL,
        "texture",
        "\"<id>\"",
        "",
        "Its image (a texture's id): the colour times `color`, the alpha its coverage. Without one it is a square of `color`.",
        "texture \"fx/scorch.png\"",
    ),
    Line::of(
        DECAL,
        "normal",
        "\"<id>\"",
        "",
        "A tangent-space normal map (a texture's id; +x along u, +y up the image, as glTF) that bends the surface under it: a crater's rim, a crack, a bullet hole.",
        "normal \"fx/scorch-normal.png\"",
    ),
    Line::of(
        DECAL,
        "normalScale",
        "<strength>",
        "1",
        "How strongly `normal` bends the surface, 0 to 4.",
        "normalScale 1.5",
    ),
    Line::of(
        DECAL,
        "uv",
        "<u0> <v0> <u1> <v1>",
        "0 0 1 1",
        "Its region of `texture` and `normal`, v down: one mark of a sheet of several.",
        "uv 0 0 0.5 0.5",
    ),
    Line::of(
        DECAL,
        "emissive",
        "<r> <g> <b>",
        "0 0 0",
        "Linear light it adds where it covers, times the image (embers in a scorch, a glowing rune); above 1 blooms. It fades with the decal.",
        "emissive 4 1.2 0.2",
    ),
    Line::of(
        DECAL,
        "roughness",
        "<0..1>",
        "",
        "The surface's roughness under it (soot is rough, a puddle smooth); without the line the surface keeps its own.",
        "roughness 0.95",
    ),
    Line::of(
        DECAL,
        "metallic",
        "<0..1>",
        "",
        "The surface's metalness under it; without the line the surface keeps its own.",
        "metallic 0",
    ),
    Line::of(
        DECAL,
        "wet",
        "true | false",
        "false",
        "Darkens the surface by its colour and flattens it toward the decal's normal instead of covering it: a puddle, a splash mark, blood soaking in. Set `roughness` low for standing water.",
        "wet true",
    ),
    Line::of(
        DECAL,
        "angleFade",
        "<degrees>",
        "75",
        "Surfaces turned further than this from the projection are not covered (the last 15 degrees fade), 0 to 90.",
        "angleFade 80",
    ),
    Line::of(
        DECAL,
        "turn",
        "<degrees> | <min> <max>",
        "0",
        "Turns it about the projection; a range picks one angle per instance from its seed, so repeated marks do not look stamped.",
        "turn 0 360",
    ),
    Line::of(
        DECAL,
        "size",
        "<s> | <x> <y> <z>",
        "1 1 1",
        "x, the depth it reaches down its -y, and z.",
        "size 2 1 2",
    ),
    Line::of(
        DECAL,
        "fade",
        "<seconds>",
        "0",
        "Fades out over its last this many seconds.",
        "fade 1",
    ),
    Line::of(
        &["glow"],
        "size",
        "<value>",
        "1",
        "Metres across over its life.",
        "size curve 0 1 1 3",
    ),
    Line::of(
        &["flipbook"],
        "size",
        "<metres>",
        "1",
        "Its height; the width follows the frame's shape.",
        "size 2",
    ),
    Line::of(
        &["shake"],
        "trauma",
        "<0..1>",
        "0.2",
        "og.camera.shake's trauma.",
        "trauma 0.4",
    ),
    Line::of(
        &["sound"],
        "volume",
        "<gain>",
        "1",
        "Its volume.",
        "volume 0.8",
    ),
    Line::of(
        &["sound"],
        "playbackRate",
        "<rate> | <min> <max>",
        "1",
        "A rate picked from the range by the effect's seed.",
        "playbackRate 0.9 1.1",
    ),
    Line::of(
        RING,
        "inner",
        "<radius>",
        "",
        "The inner radius (the outer is 1).",
        "inner 0.6",
    ),
    Line::of(
        RING,
        "band",
        "<0..1>",
        "",
        "Where its middle row lies between the edges.",
        "band 0.5",
    ),
    Line::of(
        RING,
        "height",
        "<outer> <inner>",
        "",
        "Its edges raised along its axis: a cone, a tube.",
        "height 0 0.5",
    ),
    Line::of(
        RING,
        "arc",
        "<start> <length>",
        "",
        "Degrees from +x.",
        "arc 90 180",
    ),
    Line::of(
        RING,
        "segments",
        "<n>",
        "",
        "3 to 256 around it.",
        "segments 64",
    ),
    Line::of(
        RING,
        "colors",
        "<outer rgba> <band rgba> <inner rgba>",
        "",
        "Its vertex colours; u runs along the arc and v from the outer edge (0) to the inner (1).",
        "colors 1 1 1 0  1 1 1 1  1 1 1 0",
    ),
];

/// The `.ogt` type of a composed effect file.
pub const VFX_TYPE: &str = "vfx";

/// The `type vfx` version this engine writes; it reads every version from
/// 1 (whose `flash` element is version 2's `glow`).
pub const VFX_VERSION: u32 = 2;
/// The event an `impact` line sends.
pub const IMPACT: &str = "impact";
/// Elements in one effect at most.
pub const MAX_ELEMENTS: usize = 64;
/// Named events in one effect at most.
pub const MAX_EVENTS: usize = 16;
/// An event name's bytes at most (og.VfxEvent.name).
pub const MAX_EVENT_NAME: usize = 63;
/// Named parameters in one effect at most (og.Vfx.params).
pub const MAX_PARAMS: usize = 4;

/// A composed effect: its length, events and elements.
#[derive(Clone, Debug, PartialEq)]
pub struct Vfx {
    pub duration: f32,
    pub looping: bool,
    /// By time, then as written.
    pub events: Vec<VfxEventKey>,
    pub elements: Vec<VfxElement>,
    /// Its `param` lines, in file order: og.Vfx.params' order.
    pub params: Vec<VfxParam>,
}

/// A number the game sets per instance (`param <name> <default>`).
#[derive(Clone, Debug, PartialEq)]
pub struct VfxParam {
    pub name: String,
    pub default: f32,
}

/// A named event at a time (`event`, `impact`).
#[derive(Clone, Debug, PartialEq)]
pub struct VfxEventKey {
    pub time: f32,
    pub name: String,
}

/// One element: when it runs, where it sits, what it is.
#[derive(Clone, Debug, PartialEq)]
pub struct VfxElement {
    /// Seconds from the effect's start.
    pub at: f32,
    /// Seconds it lasts (`life`, or `until` − `at`); 0 for an instant
    /// (a shake, a sound) and for an emitter left to its own cycle.
    pub life: f32,
    /// From the effect's origin, in its space (metres before its scale).
    pub offset: glam::Vec3,
    /// Degrees about x, y, z (applied z, y, x: as three.js's Euler XYZ).
    pub rotate: glam::Vec3,
    /// Metres per second it moves from its offset (`move`), in the
    /// effect's space, from its start.
    pub velocity: glam::Vec3,
    /// Metres per second squared (`accelerate`).
    pub acceleration: glam::Vec3,
    /// The param it plays only above 0 (`when`), by index.
    pub when: Option<usize>,
    /// The param that multiplies how much of it there is (`amount`).
    pub amount: Option<usize>,
    /// The param that multiplies its length along its y (`stretch`): a
    /// mesh's or a glow's.
    pub stretch: Option<usize>,
    pub kind: VfxKind,
}

impl VfxElement {
    /// Its local transform in the effect's space where it starts.
    pub fn local(&self) -> glam::Mat4 {
        self.local_at(0.0)
    }

    /// Its local transform `t` seconds after its start: its offset moved
    /// by its velocity and acceleration (an Effekseer ancestor's motion,
    /// which its descendants follow).
    pub fn local_at(&self, t: f32) -> glam::Mat4 {
        let t = t.max(0.0);
        glam::Mat4::from_rotation_translation(
            play::euler(self.rotate),
            self.offset + self.velocity * t + self.acceleration * (0.5 * t * t),
        )
    }
}

/// The shape an effect mesh draws.
#[derive(Clone, Debug, PartialEq)]
pub enum VfxShape {
    Sphere,
    Quad,
    /// The built-in ring (inner radius 0.5, its uv laid flat over it), or,
    /// with any of `inner`, `band`, `height`, `arc`, `segments` and `colors`, a ring
    /// of that shape (og_render_data::EffectRing: u along its arc, v across).
    Ring(Option<og_render_data::EffectRing>),
    Box,
    Cylinder,
    /// A model asset id (its first mesh).
    Model(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum VfxKind {
    Emitter {
        id: String,
        seed: u32,
    },
    Mesh {
        shape: VfxShape,
        effect: String,
        scale: EffectValue,
        /// Degrees per second about x, y, z.
        spin: glam::Vec3,
        color: [f32; 4],
        blend: ParticleBlend,
        /// `face camera`: turned to face the eye, its spin and scale kept.
        face_camera: bool,
    },
    Trail {
        effect: Option<String>,
        width: [f32; 2],
        lifetime: f32,
        color: [f32; 4],
        /// Radius and turns per second about the element's y.
        orbit: [f32; 2],
        /// Metres per second up the element's y.
        rise: f32,
    },
    Beam {
        effect: Option<String>,
        to: glam::Vec3,
        width: f32,
        color: [f32; 4],
        segments: u32,
        jitter: f32,
    },
    Light {
        color: [f32; 3],
        intensity: EffectValue,
        distance: f32,
    },
    Decal {
        /// Its image and its normal map (texture ids).
        texture: Option<String>,
        normal: Option<String>,
        normal_scale: f32,
        /// `[u0, v0, u1, v1]` of both.
        uv: [f32; 4],
        size: glam::Vec3,
        /// Seconds it fades out over at its end.
        fade: f32,
        color: [f32; 4],
        emissive: [f32; 3],
        /// The surface's under it; negative keeps the surface's own.
        roughness: f32,
        metallic: f32,
        wet: bool,
        /// Degrees from the projection past which a surface is not covered.
        angle_fade: f32,
        /// Degrees about the projection, an instance's picked between them.
        turn: [f32; 2],
    },
    Glow {
        size: EffectValue,
        color: [f32; 4],
    },
    Shake {
        trauma: f32,
    },
    Sound {
        id: String,
        volume: f32,
        pitch: [f32; 2],
    },
    /// A flipbook sheet's first clip on a camera-facing sprite.
    Flipbook {
        id: String,
        /// Metres tall; the width follows the frame.
        size: f32,
        color: [f32; 4],
        blend: ParticleBlend,
    },
}

impl VfxKind {
    pub fn name(&self) -> &'static str {
        match self {
            VfxKind::Emitter { .. } => "emitter",
            VfxKind::Mesh { .. } => "mesh",
            VfxKind::Trail { .. } => "trail",
            VfxKind::Beam { .. } => "beam",
            VfxKind::Light { .. } => "light",
            VfxKind::Decal { .. } => "decal",
            VfxKind::Glow { .. } => "glow",
            VfxKind::Shake { .. } => "shake",
            VfxKind::Sound { .. } => "sound",
            VfxKind::Flipbook { .. } => "flipbook",
        }
    }
}

/// What an effect names, by kind of asset, for the cook to check and the
/// runtime to load.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VfxReferences {
    pub emitters: Vec<String>,
    pub effects: Vec<String>,
    pub models: Vec<String>,
    pub sounds: Vec<String>,
    pub flipbooks: Vec<String>,
    /// Decals' images and normal maps.
    pub textures: Vec<String>,
}

impl Vfx {
    pub fn references(&self) -> VfxReferences {
        let mut r = VfxReferences::default();
        let push = |v: &mut Vec<String>, id: &str| {
            if !v.iter().any(|x| x == id) {
                v.push(id.to_string());
            }
        };
        for e in &self.elements {
            match &e.kind {
                VfxKind::Emitter { id, .. } => push(&mut r.emitters, id),
                VfxKind::Mesh { shape, effect, .. } => {
                    push(&mut r.effects, effect);
                    if let VfxShape::Model(m) = shape {
                        push(&mut r.models, m);
                    }
                }
                VfxKind::Trail { effect, .. } | VfxKind::Beam { effect, .. } => {
                    if let Some(f) = effect {
                        push(&mut r.effects, f);
                    }
                }
                VfxKind::Sound { id, .. } => push(&mut r.sounds, id),
                VfxKind::Flipbook { id, .. } => push(&mut r.flipbooks, id),
                VfxKind::Decal {
                    texture, normal, ..
                } => {
                    for t in [texture, normal].into_iter().flatten() {
                        push(&mut r.textures, t);
                    }
                }
                VfxKind::Light { .. } | VfxKind::Glow { .. } | VfxKind::Shake { .. } => {}
            }
        }
        r
    }
}

fn color(e: &Entry) -> Result<[f32; 4], String> {
    e.color(0).map_err(|x| x.to_string())
}

fn vec3(e: &Entry) -> Result<glam::Vec3, String> {
    match e.values.len() {
        3 => e.vec3(0).map_err(|x| x.to_string()),
        _ => Err(format!("line {}: {} <x> <y> <z>", e.line, e.key)),
    }
}

fn id(e: &Entry, what: &str) -> Result<String, String> {
    e.values
        .first()
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("line {}: {} \"<{what} id>\" {{ ... }}", e.line, e.key))
}

fn time(e: &Entry) -> Result<f32, String> {
    let t = one(e)?;
    if t < 0.0 {
        return Err(format!("line {}: {} is seconds, 0 or more", e.line, e.key));
    }
    Ok(t)
}

/// Parses a `type vfx` file.
pub fn parse_vfx(text: &str) -> Result<Vfx, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != VFX_TYPE {
        return Err(format!(
            "a composed effect is `type {VFX_TYPE}`, not `type {}`",
            doc.ty
        ));
    }
    if !(1..=VFX_VERSION).contains(&doc.version) {
        return Err(format!(
            "vfx version {} (this engine reads 1 to {VFX_VERSION})",
            doc.version
        ));
    }
    let mut vfx = Vfx {
        duration: -1.0,
        looping: false,
        events: Vec::new(),
        elements: Vec::new(),
        params: Vec::new(),
    };
    // The params first, so an element may name one declared below it.
    for e in doc.body.entries.iter().filter(|e| e.key == "param") {
        let line = e.line;
        let (Some(Value::Word(name)), Some(default), 2) = (
            e.values.first(),
            e.values.get(1).and_then(Value::as_number),
            e.values.len(),
        ) else {
            return Err(format!("line {line}: param <name> <default>"));
        };
        let default = default.as_f32();
        if !default.is_finite() {
            return Err(format!("line {line}: param {name} {default}"));
        }
        if vfx.params.iter().any(|p| &p.name == name) {
            return Err(format!("line {line}: param `{name}` is declared twice"));
        }
        vfx.params.push(VfxParam {
            name: name.clone(),
            default,
        });
    }
    if vfx.params.len() > MAX_PARAMS {
        return Err(format!(
            "{} params (at most {MAX_PARAMS})",
            vfx.params.len()
        ));
    }
    let mut impact = false;
    for e in &doc.body.entries {
        let line = e.line;
        match e.key.as_str() {
            "param" => {}
            "duration" => vfx.duration = time(e)?,
            "loop" => vfx.looping = flag(e)?,
            "impact" => {
                if impact {
                    return Err(format!("line {line}: one impact line at most"));
                }
                impact = true;
                vfx.events.push(VfxEventKey {
                    time: time(e)?,
                    name: IMPACT.into(),
                });
            }
            "event" => {
                let (Some(t), Some(name)) = (
                    e.values.first().and_then(Value::as_number),
                    e.values.get(1).and_then(Value::as_str),
                ) else {
                    return Err(format!("line {line}: event <seconds> \"<name>\""));
                };
                let t = t.as_f32();
                if t < 0.0 || !t.is_finite() || e.values.len() != 2 {
                    return Err(format!("line {line}: event <seconds> \"<name>\""));
                }
                if name.is_empty() || name.len() > MAX_EVENT_NAME {
                    return Err(format!(
                        "line {line}: an event's name is 1 to {MAX_EVENT_NAME} bytes"
                    ));
                }
                vfx.events.push(VfxEventKey {
                    time: t,
                    name: name.into(),
                });
            }
            _ => vfx.elements.push(element(e, doc.version, &vfx.params)?),
        }
    }
    if vfx.duration < 0.0 {
        return Err("a vfx needs `duration <seconds>`".into());
    }
    if vfx.elements.len() > MAX_ELEMENTS {
        return Err(format!(
            "{} elements (at most {MAX_ELEMENTS})",
            vfx.elements.len()
        ));
    }
    if vfx.events.len() > MAX_EVENTS {
        return Err(format!(
            "{} events (at most {MAX_EVENTS})",
            vfx.events.len()
        ));
    }
    for ev in &vfx.events {
        if ev.time > vfx.duration {
            return Err(format!(
                "event \"{}\" at {} s is after the duration, {} s",
                ev.name, ev.time, vfx.duration
            ));
        }
    }
    og_core::sort::sort_by_key(&mut vfx.events, |ev| ev.time.to_bits());
    Ok(vfx)
}

fn element(e: &Entry, version: u32, params: &[VfxParam]) -> Result<VfxElement, String> {
    let line = e.line;
    let kinds = keys(VFX_LINES, &[]);
    if e.key != "flash" && find(VFX_LINES, &e.key, &[]).is_none() {
        return Err(format!(
            "line {line}: unknown `{}` (a vfx takes {kinds})",
            e.key
        ));
    }
    let mut el = VfxElement {
        at: 0.0,
        life: 0.0,
        offset: glam::Vec3::ZERO,
        rotate: glam::Vec3::ZERO,
        velocity: glam::Vec3::ZERO,
        acceleration: glam::Vec3::ZERO,
        when: None,
        amount: None,
        stretch: None,
        kind: match e.key.as_str() {
            "emitter" => VfxKind::Emitter {
                id: id(e, "emitter")?,
                seed: 0,
            },
            "mesh" => VfxKind::Mesh {
                shape: match e.values.first() {
                    Some(Value::String(m)) => VfxShape::Model(m.clone()),
                    Some(Value::Word(w)) => match w.as_str() {
                        "sphere" => VfxShape::Sphere,
                        "quad" => VfxShape::Quad,
                        "ring" => VfxShape::Ring(None),
                        "box" => VfxShape::Box,
                        "cylinder" => VfxShape::Cylinder,
                        other => {
                            return Err(format!(
                                "line {line}: mesh `{other}` (sphere, quad, ring, box, cylinder or \"<model id>\")"
                            ));
                        }
                    },
                    _ => {
                        return Err(format!(
                            "line {line}: mesh sphere | quad | ring | box | cylinder | \"<model id>\" {{ ... }}"
                        ));
                    }
                },
                effect: String::new(),
                scale: EffectValue::Constant(1.0),
                spin: glam::Vec3::ZERO,
                color: [1.0; 4],
                blend: ParticleBlend::Additive,
                face_camera: false,
            },
            "trail" => VfxKind::Trail {
                effect: None,
                width: [0.2, 0.0],
                lifetime: 0.4,
                color: [1.0; 4],
                orbit: [0.0, 0.0],
                rise: 0.0,
            },
            "beam" => VfxKind::Beam {
                effect: None,
                to: glam::Vec3::Y,
                width: 0.2,
                color: [1.0; 4],
                segments: 1,
                jitter: 0.0,
            },
            "light" => VfxKind::Light {
                color: [1.0; 3],
                intensity: EffectValue::Constant(1.0),
                distance: 5.0,
            },
            "decal" => VfxKind::Decal {
                texture: None,
                normal: None,
                normal_scale: 1.0,
                uv: [0.0, 0.0, 1.0, 1.0],
                size: glam::Vec3::ONE,
                fade: 0.0,
                color: [1.0; 4],
                emissive: [0.0; 3],
                roughness: -1.0,
                metallic: -1.0,
                wet: false,
                angle_fade: 75.0,
                turn: [0.0; 2],
            },
            // Version 1 named the glow `flash`, as og.tween.flash and
            // PostEffects.flash name other things.
            "glow" => VfxKind::Glow {
                size: EffectValue::Constant(1.0),
                color: [1.0; 4],
            },
            "flash" if version == 1 => VfxKind::Glow {
                size: EffectValue::Constant(1.0),
                color: [1.0; 4],
            },
            "flash" => {
                return Err(format!(
                    "line {line}: `flash` is `glow` from vfx version 2 (`og upgrade` rewrites it)"
                ));
            }
            "shake" => VfxKind::Shake { trauma: 0.2 },
            "sound" => VfxKind::Sound {
                id: id(e, "sound")?,
                volume: 1.0,
                pitch: [1.0, 1.0],
            },
            "flipbook" => VfxKind::Flipbook {
                id: id(e, "flipbook")?,
                size: 1.0,
                color: [1.0; 4],
                blend: ParticleBlend::Alpha,
            },
            other => {
                return Err(format!("line {line}: `{other}` is not read yet ({kinds})"));
            }
        },
    };
    let block = e
        .block
        .as_ref()
        .ok_or_else(|| format!("line {line}: {} {{ at <seconds> ... }}", e.key))?;
    let mut until: Option<f32> = None;
    let mut life: Option<f32> = None;
    let ring = matches!(
        el.kind,
        VfxKind::Mesh {
            shape: VfxShape::Ring(_),
            ..
        }
    );
    let labels: &[&str] = if ring {
        &["mesh", "ring"]
    } else {
        &[el.kind.name()]
    };
    for k in &block.entries {
        let kl = k.line;
        if find(ELEMENT_LINES, &k.key, labels).is_none() {
            return Err(format!(
                "line {kl}: a {} takes no `{}` (it takes {})",
                el.kind.name(),
                k.key,
                keys(ELEMENT_LINES, labels)
            ));
        }
        match (k.key.as_str(), &mut el.kind) {
            ("at", _) => el.at = time(k)?,
            ("life", _) => life = Some(time(k)?),
            ("until", _) => until = Some(time(k)?),
            ("offset", _) => el.offset = vec3(k)?,
            ("rotate", _) => el.rotate = vec3(k)?,
            ("move", _) => el.velocity = vec3(k)?,
            ("accelerate", _) => el.acceleration = vec3(k)?,
            (key @ ("when" | "amount"), _)
            | (key @ "stretch", VfxKind::Mesh { .. } | VfxKind::Glow { .. }) => {
                let name = match &k.values[..] {
                    [Value::Word(w)] => w,
                    _ => return Err(format!("line {kl}: {key} <param>")),
                };
                let i = params.iter().position(|p| &p.name == name).ok_or_else(|| {
                    let names: Vec<_> = params.iter().map(|p| p.name.as_str()).collect();
                    format!(
                        "line {kl}: {key} `{name}`, which no param line declares (declared: {})",
                        if names.is_empty() {
                            "none".to_string()
                        } else {
                            names.join(", ")
                        }
                    )
                })?;
                *match key {
                    "when" => &mut el.when,
                    "stretch" => &mut el.stretch,
                    _ => &mut el.amount,
                } = Some(i);
            }
            ("seed", VfxKind::Emitter { seed, .. }) => {
                *seed = k
                    .values
                    .first()
                    .and_then(Value::as_number)
                    .and_then(|n| n.as_u64().ok())
                    .and_then(|n| u32::try_from(n).ok())
                    .ok_or_else(|| format!("line {kl}: seed <u32>"))?
            }
            ("effect", VfxKind::Mesh { effect, .. }) => *effect = id(k, "effect")?,
            ("effect", VfxKind::Trail { effect, .. } | VfxKind::Beam { effect, .. }) => {
                *effect = Some(id(k, "effect")?)
            }
            ("scale", VfxKind::Mesh { scale, .. }) => *scale = value(k, 0)?,
            ("spin", VfxKind::Mesh { spin, .. }) => *spin = vec3(k)?,
            ("face", VfxKind::Mesh { face_camera, .. }) => {
                *face_camera = match k.values.first().and_then(Value::as_word) {
                    Some("camera") => true,
                    _ => return Err(format!("line {kl}: face camera")),
                }
            }
            ("blend", VfxKind::Mesh { blend, .. }) => {
                *blend = crate::grammar::blend_word(k.values.first().and_then(Value::as_word))
                    .ok_or_else(|| format!("line {kl}: blend {}", crate::grammar::BLENDS))?
            }
            ("blend", VfxKind::Flipbook { blend, .. }) => {
                *blend = match k.values.first().and_then(Value::as_word) {
                    Some("additive") => ParticleBlend::Additive,
                    Some("alpha") => ParticleBlend::Alpha,
                    _ => return Err(format!("line {kl}: a flipbook's blend is additive | alpha")),
                }
            }
            (
                "color",
                VfxKind::Mesh { color, .. }
                | VfxKind::Trail { color, .. }
                | VfxKind::Beam { color, .. }
                | VfxKind::Decal { color, .. }
                | VfxKind::Glow { color, .. }
                | VfxKind::Flipbook { color, .. },
            ) => *color = self::color(k)?,
            ("color", VfxKind::Light { color, .. }) => {
                let [r, g, b, _] = self::color(k)?;
                *color = [r, g, b];
            }
            ("width", VfxKind::Trail { width, .. }) => {
                *width = match nums(k, 0)?[..] {
                    [w] => [w, 0.0],
                    [a, b] => [a, b],
                    _ => return Err(format!("line {kl}: width <head> [<tail>]")),
                }
            }
            ("width", VfxKind::Beam { width, .. }) => *width = one(k)?.max(0.0),
            ("lifetime", VfxKind::Trail { lifetime, .. }) => *lifetime = time(k)?,
            ("orbit", VfxKind::Trail { orbit, .. }) => {
                *orbit = match nums(k, 0)?[..] {
                    [r, t] => [r, t],
                    _ => return Err(format!("line {kl}: orbit <radius> <turns per second>")),
                }
            }
            ("rise", VfxKind::Trail { rise, .. }) => *rise = one(k)?,
            ("to", VfxKind::Beam { to, .. }) => *to = vec3(k)?,
            ("segments", VfxKind::Beam { segments, .. }) => {
                *segments = (one(k)?.round() as u32).clamp(1, 256)
            }
            ("jitter", VfxKind::Beam { jitter, .. }) => *jitter = one(k)?.max(0.0),
            ("intensity", VfxKind::Light { intensity, .. }) => *intensity = value(k, 0)?,
            ("distance", VfxKind::Light { distance, .. }) => *distance = one(k)?.max(0.0),
            ("size", VfxKind::Decal { size, .. }) => {
                *size = match nums(k, 0)?[..] {
                    [s] => glam::Vec3::splat(s),
                    [x, y, z] => glam::Vec3::new(x, y, z),
                    _ => return Err(format!("line {kl}: size <s> | <x> <y> <z>")),
                }
            }
            ("fade", VfxKind::Decal { fade, .. }) => *fade = time(k)?,
            ("uv", VfxKind::Decal { uv, .. }) => {
                *uv = match nums(k, 0)?[..] {
                    [a, b, c, d] => [a, b, c, d],
                    _ => return Err(format!("line {kl}: uv <u0> <v0> <u1> <v1>")),
                }
            }
            ("texture", VfxKind::Decal { texture, .. }) => *texture = Some(id(k, "texture")?),
            ("normal", VfxKind::Decal { normal, .. }) => *normal = Some(id(k, "texture")?),
            ("normalScale", VfxKind::Decal { normal_scale, .. }) => {
                *normal_scale = one(k)?.clamp(0.0, 4.0)
            }
            ("emissive", VfxKind::Decal { emissive, .. }) => {
                *emissive = vec3(k)?.max(glam::Vec3::ZERO).to_array()
            }
            ("roughness", VfxKind::Decal { roughness, .. }) => *roughness = one(k)?.clamp(0.0, 1.0),
            ("metallic", VfxKind::Decal { metallic, .. }) => *metallic = one(k)?.clamp(0.0, 1.0),
            ("wet", VfxKind::Decal { wet, .. }) => *wet = flag(k)?,
            ("angleFade", VfxKind::Decal { angle_fade, .. }) => {
                *angle_fade = one(k)?.clamp(0.0, 90.0)
            }
            ("turn", VfxKind::Decal { turn, .. }) => {
                *turn = match nums(k, 0)?[..] {
                    [a] => [a, a],
                    [a, b] => [a, b],
                    _ => return Err(format!("line {kl}: turn <degrees> | <min> <max>")),
                }
            }
            ("size", VfxKind::Glow { size, .. }) => *size = value(k, 0)?,
            ("size", VfxKind::Flipbook { size, .. }) => *size = one(k)?.max(0.0),
            ("trauma", VfxKind::Shake { trauma }) => *trauma = one(k)?.clamp(0.0, 1.0),
            ("volume", VfxKind::Sound { volume, .. }) => *volume = one(k)?.max(0.0),
            ("playbackRate", VfxKind::Sound { pitch, .. }) => {
                *pitch = match nums(k, 0)?[..] {
                    [p] => [p, p],
                    [a, b] => [a, b],
                    _ => return Err(format!("line {kl}: playbackRate <rate> | <min> <max>")),
                }
            }
            (
                key @ ("inner" | "band" | "height" | "arc" | "segments" | "colors"),
                VfxKind::Mesh {
                    shape: VfxShape::Ring(ring),
                    ..
                },
            ) => {
                let r = ring.get_or_insert_with(og_render_data::EffectRing::default);
                let v = nums(k, 0)?;
                match (key, &v[..]) {
                    ("inner", [x]) => r.inner = x.clamp(0.0, 16.0),
                    ("height", [o, i]) => r.height = [*o, *i],
                    ("band", [x]) => r.band = x.clamp(0.0, 1.0),
                    ("arc", [a, l]) => (r.start, r.length) = (*a, l.clamp(0.0, 360.0)),
                    ("segments", [n]) => r.segments = (n.round() as u32).clamp(3, 256),
                    ("colors", c) if c.len() == 12 => {
                        for (i, row) in r.colors.iter_mut().enumerate() {
                            row.copy_from_slice(&c[i * 4..i * 4 + 4]);
                        }
                    }
                    _ => {
                        return Err(format!(
                            "line {kl}: a ring takes inner <radius>, band <0..1>, height <outer> <inner>, arc <start> <length>, segments <n>, colors <outer rgba> <band rgba> <inner rgba>"
                        ));
                    }
                }
            }
            (other, kind) => {
                return Err(format!("line {kl}: a {} takes no `{other}`", kind.name()));
            }
        }
    }
    let instant = matches!(el.kind, VfxKind::Shake { .. } | VfxKind::Sound { .. });
    el.life = match (life, until) {
        (Some(_), Some(_)) => {
            return Err(format!("line {line}: life or until, not both"));
        }
        (_, _) if instant => 0.0,
        (Some(l), None) => l,
        (None, Some(u)) if u >= el.at => u - el.at,
        (None, Some(u)) => {
            return Err(format!("line {line}: until {u} is before at {}", el.at));
        }
        // An emitter runs its own cycle; a flipbook its clip.
        (None, None) if matches!(el.kind, VfxKind::Emitter { .. } | VfxKind::Flipbook { .. }) => {
            0.0
        }
        (None, None) => {
            return Err(format!(
                "line {line}: a {} needs `life <seconds>` or `until <seconds>`",
                el.kind.name()
            ));
        }
    };
    if let VfxKind::Mesh { effect, .. } = &el.kind
        && effect.is_empty()
    {
        return Err(format!("line {line}: a mesh needs `effect \"<id>\"`"));
    }
    Ok(el)
}
