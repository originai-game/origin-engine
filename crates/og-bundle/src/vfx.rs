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

mod lines;
mod parse;
mod play;

pub use lines::{ELEMENT_LINES, VFX_LINES};
pub use parse::parse_vfx;
pub use play::{VfxCue, VfxDraw, VfxDrawKind, VfxInstance, element_id, hue_rotated, random};

use og_render_data::{EffectValue, ParticleBlend};

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
    /// The param that multiplies its length (`stretch`): a mesh's or a
    /// glow's along its y, a beam's `to`.
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
