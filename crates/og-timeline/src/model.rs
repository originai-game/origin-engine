//! The `type timeline` asset: its text, parsed and written back
//! canonically (the cook writes the canonical text into the blob, with its
//! caption files imported as keys, and the runtime parses that again).
//!
//! ```text
//! type timeline
//! version 1
//! ---
//! name "Intro"
//! duration 8
//! path "flyby" {
//!   kind catmullRom
//!   point -6 3 9
//!   point 0 2.5 7
//!   point 5 2 5
//! }
//! track camera {
//!   rail "flyby"
//!   key 0 {
//!     at 0
//!     target 0 1 0
//!   }
//!   key 6 {
//!     at 1
//!     ease cubicInOut
//!   }
//! }
//! track caption {
//!   captions "narration.srt"
//!   style lg
//!   anchor bottom
//! }
//! ```
//!
//! Every track holds `key <seconds> { ... }` blocks; a value between two
//! keys is eased by the later key's `ease` (linear when it names none),
//! held before the first key and after the last.

mod text;
mod tracks;

pub use text::{PATHS_TYPE, parse_paths, path_entry, paths_to_text};
use tracks::track;

use crate::captions::{self, Cue};
use crate::path::{PathDesc, PathKind};
use og_core::glam::{Quat, Vec3};
use og_core::ogt::{Block, Document, Entry, Number, Value};
use og_tween::Ease;

/// The `.ogt` type of a timeline.
pub const TIMELINE_TYPE: &str = "timeline";
/// How long a caption shows when it gives no `end` and no caption follows.
pub const CAPTION_SECONDS: f32 = 3.0;
/// Word timestamps are grouped into lines of at most this many characters.
pub const CAPTION_CHARS: usize = 42;
/// A pause of this many seconds between words starts a new caption.
pub const CAPTION_GAP: f32 = 0.6;

/// A timeline: a duration, the paths it names and its tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Timeline {
    pub name: String,
    /// Seconds.
    pub duration: f32,
    /// Paths declared in the file, by name (a camera track's `rail`).
    pub paths: Vec<(String, PathDesc)>,
    pub tracks: Vec<Track>,
}

/// How a camera track places the camera.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CameraMode {
    /// Keys give `position` and `target` in world space.
    #[default]
    Keys,
    /// The camera rides a path: keys give `at` (its arc-length parameter,
    /// 0..1) and `target` in world space; without a target it looks along
    /// the path.
    Rail(String),
    /// The camera follows an entity: keys give `position` and `target` as
    /// offsets from the entity's position, in world axes.
    Follow(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraKey {
    pub time: f32,
    pub ease: Ease,
    pub position: Option<Vec3>,
    pub target: Option<Vec3>,
    pub at: Option<f32>,
    pub fov: Option<f32>,
    /// Degrees about the view direction (a dutch angle), counter-clockwise.
    pub roll: Option<f32>,
    /// The distance from the camera that is sharp (og.PostEffects' depth of
    /// field).
    pub focus: Option<f32>,
    /// Keep the entity of this name sharp: the focus is its distance from
    /// the camera (held from this key to the next that names one or a
    /// `focus`).
    pub focus_on: Option<String>,
    /// The half-width of the sharp band around the focus.
    pub focus_range: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraTrack {
    pub mode: CameraMode,
    pub keys: Vec<CameraKey>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransformKey {
    pub time: f32,
    pub ease: Ease,
    pub position: Option<Vec3>,
    pub quaternion: Option<Quat>,
    pub scale: Option<Vec3>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransformTrack {
    pub entity: String,
    pub keys: Vec<TransformKey>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationKey {
    pub time: f32,
    pub clip: String,
    pub speed: f32,
    /// Seconds the clip cross-fades in over (og.AnimationPlayer.fade).
    pub blend_in: f32,
    pub looping: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationTrack {
    pub entity: String,
    /// The glTF whose clips the keys name; empty: the entity's og.Model.
    pub model: String,
    pub keys: Vec<AnimationKey>,
}

/// An og.AudioBus by name (`master`, `music`, `sfx`, `ui`), as its code.
pub fn bus_code(name: &str) -> Option<u32> {
    ["master", "music", "sfx", "ui"]
        .iter()
        .position(|b| *b == name)
        .map(|i| i as u32)
}

pub fn bus_name(code: u32) -> &'static str {
    ["master", "music", "sfx", "ui"]
        .get(code as usize)
        .copied()
        .unwrap_or("sfx")
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundKey {
    pub time: f32,
    pub sound: String,
    pub volume: f32,
    pub pitch: f32,
    pub bus: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MusicKey {
    pub time: f32,
    pub sound: String,
    pub volume: f32,
    pub fade_in: f32,
    pub fade_out: f32,
    /// When it stops (faded out by then); `None`: the timeline's end.
    pub end: Option<f32>,
    pub looping: bool,
    pub bus: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaptionKey {
    pub time: f32,
    /// When it goes (resolved at parse: the next caption's time, or
    /// [`CAPTION_SECONDS`] later).
    pub end: f32,
    pub text: String,
    /// A UI type step (`lg`, `xl2`, ...); `None`: the track's.
    pub style: Option<String>,
    /// A UI anchor (`bottom`, `top`, `center`, ...); `None`: the track's.
    pub anchor: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaptionTrack {
    pub style: String,
    pub anchor: String,
    pub keys: Vec<CaptionKey>,
}

impl Default for CaptionTrack {
    fn default() -> Self {
        CaptionTrack {
            style: "lg".into(),
            anchor: "bottom".into(),
            keys: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventKey {
    pub time: f32,
    /// A command-channel action and its arguments, as a script step writes
    /// them without the time: `teleport:crate:0,4,0`.
    pub action: String,
    /// A seek forward over it runs it too.
    pub retroactive: bool,
}

/// How a transition covers the screen (og.ui's screen transition).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransitionKind {
    /// A hard cut: covered for its duration (a beat of black), nothing at 0.
    Cut,
    #[default]
    Fade,
    Wipe,
    Dissolve,
    Iris,
    Slide,
}

impl TransitionKind {
    pub const NAMES: [&'static str; 6] = ["cut", "fade", "wipe", "dissolve", "iris", "slide"];

    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    pub fn from_name(name: &str) -> Option<TransitionKind> {
        use TransitionKind::*;
        Self::NAMES
            .iter()
            .position(|n| *n == name)
            .map(|i| [Cut, Fade, Wipe, Dissolve, Iris, Slide][i])
    }
}

/// Which way a transition goes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransitionMode {
    /// Covers over the first half of its duration, reveals over the second.
    #[default]
    Through,
    /// Reveals: covered at its time, clear after its duration.
    In,
    /// Covers: clear at its time, covered after its duration until the next.
    Out,
}

impl TransitionMode {
    pub fn name(self) -> &'static str {
        match self {
            TransitionMode::Through => "through",
            TransitionMode::In => "in",
            TransitionMode::Out => "out",
        }
    }

    pub fn from_name(name: &str) -> Option<TransitionMode> {
        Some(match name {
            "through" => TransitionMode::Through,
            "in" => TransitionMode::In,
            "out" => TransitionMode::Out,
            _ => return None,
        })
    }
}

/// The side a wipe or slide comes from.
pub const SIDES: [&str; 4] = ["left", "right", "top", "bottom"];

#[derive(Clone, Debug, PartialEq)]
pub struct TransitionKey {
    pub time: f32,
    pub kind: TransitionKind,
    pub mode: TransitionMode,
    pub duration: f32,
    /// The cover's colour, sRGB 0..1.
    pub color: Vec3,
    /// An index into [`SIDES`].
    pub from: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MarkerKey {
    pub time: f32,
    pub name: String,
}

/// What a key sets on a field: numbers (one for a scalar, two to four for
/// a vector, a colour or a quaternion; 0 or 1 for a bool) or a word (an
/// enum variant, `true` or `false`).
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    Numbers(Vec<f32>),
    Word(String),
}

/// One key of a field track: the fields it sets at its time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FieldKey {
    pub time: f32,
    pub ease: Ease,
    /// `(field, value)`, in file order. A property track's field is its
    /// whole path (`og.PointLight.intensity`), the others' the field of the
    /// record the track implies (`intensity`).
    pub values: Vec<(String, FieldValue)>,
}

/// Which records a field track writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldsKind {
    /// Any reflected field: `og.PointLight.intensity` on the track's entity,
    /// a resource's field without one, a game component's by its name.
    Property,
    /// The entity's light: og.DirectionalLight, og.PointLight,
    /// og.SpotLight, og.RectLight or og.Light2D, whichever it has.
    Light,
    /// `visible` of whatever the entity draws (og.Mesh, og.Model,
    /// og.Sprite, og.ParticleSystem, ...), stepped.
    Visibility,
    /// og.PostEffects (no entity).
    Post,
    /// The entity's og.Blendshapes weights (`source` set to component).
    Blendshape,
    /// Presentation: the material the entity draws (its og.Mesh's, or its
    /// og.Model's material `index`), reapplied from the time every frame.
    Material,
}

impl FieldsKind {
    pub const ALL: [FieldsKind; 6] = [
        FieldsKind::Property,
        FieldsKind::Light,
        FieldsKind::Visibility,
        FieldsKind::Post,
        FieldsKind::Blendshape,
        FieldsKind::Material,
    ];

    pub fn name(self) -> &'static str {
        match self {
            FieldsKind::Property => "property",
            FieldsKind::Light => "light",
            FieldsKind::Visibility => "visibility",
            FieldsKind::Post => "post",
            FieldsKind::Blendshape => "blendshape",
            FieldsKind::Material => "material",
        }
    }

    pub fn from_name(name: &str) -> Option<FieldsKind> {
        Self::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// A track that eases the fields of records (or a material) between keys.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldsTrack {
    pub kind: FieldsKind,
    /// The entity it drives (empty: a resource, for `property` and `post`).
    pub entity: String,
    /// A model's material index (`material` on an og.Model).
    pub index: u32,
    pub keys: Vec<FieldKey>,
}

impl FieldsTrack {
    /// The fields its keys set, in first-seen order.
    pub fn fields(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for k in &self.keys {
            for (n, _) in &k.values {
                if !out.contains(&n.as_str()) {
                    out.push(n);
                }
            }
        }
        out
    }
}

/// A key of a `timeline` track: from its time (until the next key), the
/// track plays another timeline nested, at `speed`, looping or holding its
/// last frame.
#[derive(Clone, Debug, PartialEq)]
pub struct NestedKey {
    pub time: f32,
    /// The timeline's asset id (`timelines/door.ogt`).
    pub play: String,
    pub speed: f32,
    pub looping: bool,
}

/// A key of a `shot` track: from its time (until the next key), the
/// virtual camera named `camera` (an entity's name; empty: none) is
/// og.CameraBrain's solo shot, blended to over `blend` seconds by `ease`
/// (linear unless given; `blend` below 0: the camera's own blendTime and
/// blendEase).
#[derive(Clone, Debug, PartialEq)]
pub struct ShotKey {
    pub time: f32,
    pub camera: String,
    pub blend: f32,
    pub ease: Ease,
}

/// How deep timelines nest (a timeline's own tracks are depth 0).
pub const NEST_DEPTH: usize = 4;

/// One track.
#[derive(Clone, Debug, PartialEq)]
pub enum Track {
    Camera(CameraTrack),
    Transform(TransformTrack),
    Animation(AnimationTrack),
    Audio(Vec<SoundKey>),
    Music(Vec<MusicKey>),
    Caption(CaptionTrack),
    Event(Vec<EventKey>),
    Transition(Vec<TransitionKey>),
    Marker(Vec<MarkerKey>),
    /// property, light, visibility, post, blendshape and material.
    Fields(FieldsTrack),
    /// Nested timelines.
    Timeline(Vec<NestedKey>),
    /// Virtual camera shots.
    Shot(Vec<ShotKey>),
}

/// Every track kind the file may name, for the refusal of another.
pub const TRACK_KINDS: &str = "camera, transform, animation, audio, music, caption, event, transition, marker, property, light, visibility, post, blendshape, material, timeline or shot";

impl Track {
    /// The track's kind as the file names it.
    pub fn kind(&self) -> &'static str {
        match self {
            Track::Camera(_) => "camera",
            Track::Transform(_) => "transform",
            Track::Animation(_) => "animation",
            Track::Audio(_) => "audio",
            Track::Music(_) => "music",
            Track::Caption(_) => "caption",
            Track::Event(_) => "event",
            Track::Transition(_) => "transition",
            Track::Marker(_) => "marker",
            Track::Fields(f) => f.kind.name(),
            Track::Timeline(_) => "timeline",
            Track::Shot(_) => "shot",
        }
    }

    /// Every key's time, in order.
    pub fn times(&self) -> Vec<f32> {
        match self {
            Track::Camera(c) => c.keys.iter().map(|k| k.time).collect(),
            Track::Transform(c) => c.keys.iter().map(|k| k.time).collect(),
            Track::Animation(c) => c.keys.iter().map(|k| k.time).collect(),
            Track::Audio(c) => c.iter().map(|k| k.time).collect(),
            Track::Music(c) => c.iter().map(|k| k.time).collect(),
            Track::Caption(c) => c.keys.iter().map(|k| k.time).collect(),
            Track::Event(c) => c.iter().map(|k| k.time).collect(),
            Track::Transition(c) => c.iter().map(|k| k.time).collect(),
            Track::Marker(c) => c.iter().map(|k| k.time).collect(),
            Track::Fields(f) => f.keys.iter().map(|k| k.time).collect(),
            Track::Timeline(c) => c.iter().map(|k| k.time).collect(),
            Track::Shot(c) => c.iter().map(|k| k.time).collect(),
        }
    }

    /// The entity it drives, for `transform` and `animation` (and a
    /// following camera); the path for a camera on a rail.
    pub fn target(&self) -> &str {
        match self {
            Track::Camera(c) => match &c.mode {
                CameraMode::Keys => "",
                CameraMode::Rail(p) | CameraMode::Follow(p) => p,
            },
            Track::Transform(t) => &t.entity,
            Track::Animation(a) => &a.entity,
            Track::Fields(f) => &f.entity,
            _ => "",
        }
    }

    pub fn key_count(&self) -> usize {
        match self {
            Track::Camera(c) => c.keys.len(),
            Track::Transform(t) => t.keys.len(),
            Track::Animation(a) => a.keys.len(),
            Track::Audio(k) => k.len(),
            Track::Music(k) => k.len(),
            Track::Caption(c) => c.keys.len(),
            Track::Event(k) => k.len(),
            Track::Transition(k) => k.len(),
            Track::Marker(k) => k.len(),
            Track::Fields(f) => f.keys.len(),
            Track::Timeline(k) => k.len(),
            Track::Shot(k) => k.len(),
        }
    }
}

// ── reading ────────────────────────────────────────────────────────────────

fn at(e: &Entry, msg: impl std::fmt::Display) -> String {
    format!("line {}: {msg}", e.line)
}

fn number(e: &Entry, i: usize) -> Result<f32, String> {
    let v = e
        .values
        .get(i)
        .and_then(Value::as_number)
        .map(|n| n.as_f32())
        .ok_or_else(|| at(e, format!("`{}` takes a number", e.key)))?;
    if !v.is_finite() {
        return Err(at(e, format!("`{}` must be finite", e.key)));
    }
    Ok(v)
}

fn one_number(e: &Entry) -> Result<f32, String> {
    if e.values.len() != 1 {
        return Err(at(e, format!("`{}` takes one number", e.key)));
    }
    number(e, 0)
}

fn vec3(e: &Entry) -> Result<Vec3, String> {
    if e.values.len() != 3 {
        return Err(at(e, format!("`{}` takes three numbers: x y z", e.key)));
    }
    Ok(Vec3::new(number(e, 0)?, number(e, 1)?, number(e, 2)?))
}

fn string(e: &Entry) -> Result<String, String> {
    match e.values.as_slice() {
        [Value::String(s)] => Ok(s.clone()),
        _ => Err(at(e, format!("`{}` takes one quoted string", e.key))),
    }
}

fn word(e: &Entry) -> Result<&str, String> {
    match e.values.as_slice() {
        [Value::Word(w)] => Ok(w),
        _ => Err(at(e, format!("`{}` takes one word", e.key))),
    }
}

fn flag(e: &Entry) -> Result<bool, String> {
    match e.values.as_slice() {
        [v] => v
            .as_bool()
            .ok_or_else(|| at(e, format!("`{}` takes true or false", e.key))),
        _ => Err(at(e, format!("`{}` takes true or false", e.key))),
    }
}

fn ease(e: &Entry) -> Result<Ease, String> {
    let w = word(e)?;
    Ease::from_name(w).ok_or_else(|| {
        at(
            e,
            format!(
                "ease '{w}' is not one of {}",
                og_tween::EASE_NAMES.join(", ")
            ),
        )
    })
}

fn no_block(e: &Entry) -> Result<(), String> {
    if e.block.is_some() {
        return Err(at(e, format!("`{}` takes no block", e.key)));
    }
    Ok(())
}

fn unknown(e: &Entry, what: &str, allowed: &str) -> String {
    at(
        e,
        format!("unknown {what} '{}' (expected {allowed})", e.key),
    )
}

/// Euler angles in degrees, XYZ order (three.js's `Euler` order "XYZ"), as
/// a quaternion: og-core's deterministic sine and cosine.
pub fn euler_degrees(v: Vec3) -> Quat {
    let half = v * (std::f32::consts::PI / 360.0);
    let (s1, c1) = og_core::math::sin_cos(half.x);
    let (s2, c2) = og_core::math::sin_cos(half.y);
    let (s3, c3) = og_core::math::sin_cos(half.z);
    Quat::from_xyzw(
        s1 * c2 * c3 + c1 * s2 * s3,
        c1 * s2 * c3 - s1 * c2 * s3,
        c1 * c2 * s3 + s1 * s2 * c3,
        c1 * c2 * c3 - s1 * s2 * s3,
    )
    .normalize()
}

/// The key blocks of a track, with their times.
fn keys(block: &Block) -> impl Iterator<Item = &Entry> {
    block.entries.iter().filter(|e| e.key == "key")
}

fn key_time(e: &Entry) -> Result<f32, String> {
    let t = e
        .values
        .first()
        .and_then(Value::as_number)
        .map(|n| n.as_f32())
        .filter(|t| t.is_finite() && *t >= 0.0)
        .ok_or_else(|| at(e, "a key starts with its time in seconds, ≥ 0: `key 1.5 {`"))?;
    if e.values.len() != 1 {
        return Err(at(e, "`key` takes one time: `key 1.5 {`"));
    }
    Ok(t)
}

fn key_block(e: &Entry) -> Result<&[Entry], String> {
    e.block
        .as_ref()
        .map(|b| b.entries.as_slice())
        .ok_or_else(|| at(e, "a key opens a block: `key 1.5 {`"))
}

/// Reads a caption file a track names. `read` is given the path as written.
pub type Read<'a> = dyn FnMut(&str) -> Result<String, String> + 'a;
