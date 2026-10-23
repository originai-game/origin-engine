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
}

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

impl Timeline {
    /// Parses timeline text. `read` loads a caption file a track names
    /// (`captions "narration.srt"`); the cooked text has none.
    pub fn parse(text: &str, read: &mut Read<'_>) -> Result<Timeline, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != TIMELINE_TYPE {
            return Err(format!("a timeline is `type timeline`, not `{}`", doc.ty));
        }
        if doc.version != 1 {
            return Err(format!(
                "timeline version {} (this engine reads 1)",
                doc.version
            ));
        }
        let mut t = Timeline {
            name: String::new(),
            duration: 0.0,
            paths: Vec::new(),
            tracks: Vec::new(),
        };
        let mut duration = None;
        for e in &doc.body.entries {
            match e.key.as_str() {
                "name" => t.name = string(e)?,
                "duration" => {
                    let d = one_number(e)?;
                    if d <= 0.0 {
                        return Err(at(e, "duration is seconds, more than 0"));
                    }
                    duration = Some(d);
                }
                "path" => {
                    let (name, desc) = path_entry(e)?;
                    if t.paths.iter().any(|(n, _)| *n == name) {
                        return Err(at(e, format!("path '{name}' is declared twice")));
                    }
                    t.paths.push((name, desc));
                }
                "track" => t.tracks.push(track(e, read)?),
                _ => {
                    return Err(unknown(
                        e,
                        "timeline entry",
                        "name, duration, path or track",
                    ));
                }
            }
        }
        let last = t.tracks.iter().map(track_end).fold(0.0f32, f32::max);
        t.duration = match duration {
            Some(d) => d,
            None if last > 0.0 => last,
            None => return Err("a timeline needs `duration` (seconds) or a key after 0".into()),
        };
        // Keys past the end never play.
        for tr in &t.tracks {
            let first_late = match tr {
                Track::Camera(c) => c.keys.iter().map(|k| k.time).find(|&k| k > t.duration),
                Track::Transform(c) => c.keys.iter().map(|k| k.time).find(|&k| k > t.duration),
                Track::Animation(c) => c.keys.iter().map(|k| k.time).find(|&k| k > t.duration),
                Track::Audio(c) => c.iter().map(|k| k.time).find(|&k| k > t.duration),
                Track::Music(c) => c.iter().map(|k| k.time).find(|&k| k > t.duration),
                Track::Caption(c) => c.keys.iter().map(|k| k.time).find(|&k| k > t.duration),
                Track::Event(c) => c.iter().map(|k| k.time).find(|&k| k > t.duration),
                Track::Transition(c) => c.iter().map(|k| k.time).find(|&k| k > t.duration),
                Track::Marker(c) => c.iter().map(|k| k.time).find(|&k| k > t.duration),
            };
            if let Some(k) = first_late {
                return Err(format!(
                    "the {} track has a key at {k} s, after the timeline's duration {} s",
                    tr.kind(),
                    t.duration
                ));
            }
        }
        // Rails name a path of the file or one the world will have; the
        // runtime refuses an unknown one when it plays.
        Ok(t)
    }

    /// A path the file declares.
    pub fn path(&self, name: &str) -> Option<&PathDesc> {
        self.paths.iter().find(|(n, _)| n == name).map(|(_, d)| d)
    }

    /// The canonical text: every caption file imported as keys, every
    /// rotation as a quaternion, every key with its resolved values.
    pub fn to_text(&self) -> String {
        let mut doc = Document::new(TIMELINE_TYPE, 1);
        let b = &mut doc.body;
        if !self.name.is_empty() {
            b.push(Entry::new("name").with(Value::String(self.name.clone())));
        }
        b.push(Entry::new("duration").with(num(self.duration)));
        for (name, p) in &self.paths {
            b.push(write_path(name, p));
        }
        for tr in &self.tracks {
            b.push(write_track(tr));
        }
        doc.to_text()
            .expect("timeline text is made of words, numbers and strings")
    }
}

/// A `path "<name>" { kind, closed, tension, point... }` entry, as a
/// timeline or a scene declares one; the points are checked against the kind.
pub fn path_entry(e: &Entry) -> Result<(String, PathDesc), String> {
    let name = match e.values.as_slice() {
        [Value::String(s)] if !s.is_empty() => s.clone(),
        _ => return Err(at(e, "a path is `path \"<name>\" {`")),
    };
    let block = e
        .block
        .as_ref()
        .ok_or_else(|| at(e, "a path opens a block: `path \"name\" {`"))?;
    let desc = path_desc(block)?;
    crate::Path::new(desc.clone()).map_err(|m| at(e, format!("path '{name}': {m}")))?;
    Ok((name, desc))
}

fn write_path(name: &str, p: &PathDesc) -> Entry {
    let mut pb = Block::new();
    pb.push(Entry::new("kind").with(Value::Word(p.kind.name().into())));
    pb.push(Entry::new("closed").with(boolean(p.closed)));
    pb.push(Entry::new("tension").with(num(p.tension)));
    for q in &p.points {
        pb.push(v3(Entry::new("point"), *q));
    }
    Entry::new("path")
        .with(Value::String(name.to_string()))
        .with_block(pb)
}

/// The `.ogt` type of a scene's paths section.
pub const PATHS_TYPE: &str = "paths";

/// Named paths as canonical text (`type paths`: a scene blob's `paths`).
pub fn paths_to_text(paths: &[(String, PathDesc)]) -> String {
    let mut doc = Document::new(PATHS_TYPE, 1);
    for (name, p) in paths {
        doc.body.push(write_path(name, p));
    }
    doc.to_text()
        .expect("paths text is made of words and numbers")
}

/// Reads [`paths_to_text`]'s text back.
pub fn parse_paths(text: &str) -> Result<Vec<(String, PathDesc)>, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != PATHS_TYPE {
        return Err(format!("expected `type paths`, not `{}`", doc.ty));
    }
    doc.body
        .entries
        .iter()
        .map(|e| {
            if e.key != "path" {
                return Err(unknown(e, "paths entry", "path"));
            }
            path_entry(e)
        })
        .collect()
}

fn num(v: f32) -> Value {
    Value::Number(Number::from_f32(v).unwrap_or_else(|_| Number::from(0)))
}

fn boolean(v: bool) -> Value {
    Value::Word(if v { "true" } else { "false" }.into())
}

fn v3(e: Entry, v: Vec3) -> Entry {
    e.with(num(v.x)).with(num(v.y)).with(num(v.z))
}

fn track_end(t: &Track) -> f32 {
    let max = |it: &mut dyn Iterator<Item = f32>| it.fold(0.0f32, f32::max);
    match t {
        Track::Camera(c) => max(&mut c.keys.iter().map(|k| k.time)),
        Track::Transform(c) => max(&mut c.keys.iter().map(|k| k.time)),
        Track::Animation(c) => max(&mut c.keys.iter().map(|k| k.time)),
        Track::Audio(c) => max(&mut c.iter().map(|k| k.time)),
        Track::Music(c) => max(&mut c.iter().map(|k| k.end.unwrap_or(k.time))),
        Track::Caption(c) => max(&mut c.keys.iter().map(|k| k.end)),
        Track::Event(c) => max(&mut c.iter().map(|k| k.time)),
        Track::Transition(c) => max(&mut c.iter().map(|k| k.time + k.duration)),
        Track::Marker(c) => max(&mut c.iter().map(|k| k.time)),
    }
}

fn path_desc(block: &Block) -> Result<PathDesc, String> {
    let mut d = PathDesc::default();
    for e in &block.entries {
        no_block(e)?;
        match e.key.as_str() {
            "kind" => {
                let w = word(e)?;
                d.kind = PathKind::from_name(w).ok_or_else(|| {
                    at(
                        e,
                        format!("path kind '{w}' is not catmullRom, bezier or linear"),
                    )
                })?;
            }
            "closed" => d.closed = flag(e)?,
            "tension" => d.tension = one_number(e)?,
            "point" => d.points.push(vec3(e)?),
            _ => return Err(unknown(e, "path entry", "kind, closed, tension or point")),
        }
    }
    Ok(d)
}

/// Sorts keys by time, keeping the file's order among equal times.
fn sorted<K>(mut keys: Vec<K>, time: impl Fn(&K) -> f32) -> Vec<K> {
    keys.sort_by(|a, b| time(a).total_cmp(&time(b)));
    keys
}

fn track(e: &Entry, read: &mut Read<'_>) -> Result<Track, String> {
    let kind = e
        .values
        .first()
        .and_then(Value::as_word)
        .ok_or_else(|| at(e, "a track names its kind: `track camera {`"))?;
    let target = match e.values.get(1) {
        None => None,
        Some(Value::String(s)) if e.values.len() == 2 => Some(s.clone()),
        _ => return Err(at(e, "a track is `track <kind> [\"entity\"] {`")),
    };
    let needs_target = matches!(kind, "transform" | "animation");
    match (&target, needs_target) {
        (None, true) => {
            return Err(at(
                e,
                format!("a {kind} track names its entity: `track {kind} \"crate\" {{`"),
            ));
        }
        (Some(_), false) => {
            return Err(at(e, format!("a {kind} track names no entity")));
        }
        _ => {}
    }
    let empty = Block::new();
    let block = e.block.as_ref().unwrap_or(&empty);
    let target = target.unwrap_or_default();
    Ok(match kind {
        "camera" => Track::Camera(camera(block)?),
        "transform" => Track::Transform(transform(block, target)?),
        "animation" => Track::Animation(animation(block, target)?),
        "audio" => Track::Audio(sounds(block)?),
        "music" => Track::Music(music(block)?),
        "caption" => Track::Caption(caption(block, read)?),
        "event" => Track::Event(events(block)?),
        "transition" => Track::Transition(transitions(block)?),
        "marker" => Track::Marker(markers(block)?),
        k => {
            return Err(at(
                e,
                format!(
                    "track kind '{k}' is not camera, transform, animation, audio, music, caption, event, transition or marker"
                ),
            ));
        }
    })
}

fn only_keys<'a>(block: &'a Block, what: &str) -> Result<Vec<&'a Entry>, String> {
    for e in &block.entries {
        if e.key != "key" {
            return Err(unknown(e, &format!("{what} track entry"), "key"));
        }
    }
    Ok(keys(block).collect())
}

fn camera(block: &Block) -> Result<CameraTrack, String> {
    let mut c = CameraTrack::default();
    let mut ks = Vec::new();
    for e in &block.entries {
        match e.key.as_str() {
            "rail" | "follow" => {
                no_block(e)?;
                if c.mode != CameraMode::Keys {
                    return Err(at(e, "a camera track has one `rail` or one `follow`"));
                }
                let s = string(e)?;
                c.mode = if e.key == "rail" {
                    CameraMode::Rail(s)
                } else {
                    CameraMode::Follow(s)
                };
            }
            "key" => {
                let mut k = CameraKey {
                    time: key_time(e)?,
                    ..CameraKey::default()
                };
                for f in key_block(e)? {
                    no_block(f)?;
                    match f.key.as_str() {
                        "position" => k.position = Some(vec3(f)?),
                        "target" => k.target = Some(vec3(f)?),
                        "at" => k.at = Some(one_number(f)?),
                        "fov" => {
                            let v = one_number(f)?;
                            if !(1.0..=179.0).contains(&v) {
                                return Err(at(f, "fov is degrees, 1 to 179"));
                            }
                            k.fov = Some(v);
                        }
                        "ease" => k.ease = ease(f)?,
                        _ => {
                            return Err(unknown(
                                f,
                                "camera key entry",
                                "position, target, at, fov or ease",
                            ));
                        }
                    }
                }
                ks.push((e, k));
            }
            _ => return Err(unknown(e, "camera track entry", "rail, follow or key")),
        }
    }
    for (e, k) in &ks {
        match c.mode {
            CameraMode::Rail(_) if k.position.is_some() => {
                return Err(at(
                    e,
                    "a camera on a rail is placed by `at` (0..1 along the path), not `position`",
                ));
            }
            CameraMode::Keys | CameraMode::Follow(_) if k.at.is_some() => {
                return Err(at(
                    e,
                    "`at` places a camera on a rail: add `rail \"<path>\"` to the track",
                ));
            }
            _ => {}
        }
    }
    c.keys = sorted(ks.into_iter().map(|(_, k)| k).collect(), |k| k.time);
    Ok(c)
}

fn transform(block: &Block, entity: String) -> Result<TransformTrack, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "transform")? {
        let mut k = TransformKey {
            time: key_time(e)?,
            ..TransformKey::default()
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "position" => k.position = Some(vec3(f)?),
                "scale" => k.scale = Some(vec3(f)?),
                "rotation" => k.quaternion = Some(euler_degrees(vec3(f)?)),
                "quaternion" => {
                    if f.values.len() != 4 {
                        return Err(at(f, "`quaternion` takes four numbers: x y z w"));
                    }
                    let q =
                        Quat::from_xyzw(number(f, 0)?, number(f, 1)?, number(f, 2)?, number(f, 3)?);
                    if q.length_squared() < 1e-12 {
                        return Err(at(f, "a quaternion of length 0 is no rotation"));
                    }
                    k.quaternion = Some(q.normalize());
                }
                "ease" => k.ease = ease(f)?,
                _ => {
                    return Err(unknown(
                        f,
                        "transform key entry",
                        "position, rotation (degrees, XYZ), quaternion, scale or ease",
                    ));
                }
            }
        }
        ks.push(k);
    }
    Ok(TransformTrack {
        entity,
        keys: sorted(ks, |k| k.time),
    })
}

fn animation(block: &Block, entity: String) -> Result<AnimationTrack, String> {
    let mut a = AnimationTrack {
        entity,
        ..AnimationTrack::default()
    };
    let mut ks = Vec::new();
    for e in &block.entries {
        match e.key.as_str() {
            "model" => {
                no_block(e)?;
                a.model = string(e)?;
            }
            "key" => {
                let mut k = AnimationKey {
                    time: key_time(e)?,
                    clip: String::new(),
                    speed: 1.0,
                    blend_in: 0.2,
                    looping: true,
                };
                for f in key_block(e)? {
                    no_block(f)?;
                    match f.key.as_str() {
                        "clip" => k.clip = string(f)?,
                        "speed" => k.speed = one_number(f)?,
                        "blendIn" => {
                            k.blend_in = one_number(f)?;
                            if k.blend_in < 0.0 {
                                return Err(at(f, "blendIn is seconds, ≥ 0"));
                            }
                        }
                        "loop" => k.looping = flag(f)?,
                        _ => {
                            return Err(unknown(
                                f,
                                "animation key entry",
                                "clip, speed, blendIn or loop",
                            ));
                        }
                    }
                }
                if k.clip.is_empty() {
                    return Err(at(e, "an animation key names its clip: `clip \"walk\"`"));
                }
                ks.push(k);
            }
            _ => return Err(unknown(e, "animation track entry", "model or key")),
        }
    }
    a.keys = sorted(ks, |k| k.time);
    Ok(a)
}

fn bus(f: &Entry) -> Result<u32, String> {
    let w = word(f)?;
    bus_code(w).ok_or_else(|| at(f, format!("bus '{w}' is not master, music, sfx or ui")))
}

fn volume(f: &Entry) -> Result<f32, String> {
    let v = one_number(f)?;
    if v < 0.0 {
        return Err(at(f, "volume is a linear gain, ≥ 0"));
    }
    Ok(v)
}

fn sounds(block: &Block) -> Result<Vec<SoundKey>, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "audio")? {
        let mut k = SoundKey {
            time: key_time(e)?,
            sound: String::new(),
            volume: 1.0,
            pitch: 1.0,
            bus: 2,
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "sound" => k.sound = string(f)?,
                "volume" => k.volume = volume(f)?,
                "pitch" => {
                    k.pitch = one_number(f)?;
                    if k.pitch <= 0.0 {
                        return Err(at(f, "pitch is a playback rate, more than 0"));
                    }
                }
                "bus" => k.bus = bus(f)?,
                _ => return Err(unknown(f, "audio key entry", "sound, volume, pitch or bus")),
            }
        }
        if k.sound.is_empty() {
            return Err(at(
                e,
                "an audio key names its sound: `sound \"audio/hit.wav\"`",
            ));
        }
        ks.push(k);
    }
    Ok(sorted(ks, |k| k.time))
}

fn music(block: &Block) -> Result<Vec<MusicKey>, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "music")? {
        let mut k = MusicKey {
            time: key_time(e)?,
            sound: String::new(),
            volume: 1.0,
            fade_in: 0.0,
            fade_out: 0.0,
            end: None,
            looping: true,
            bus: 1,
        };
        for f in key_block(e)? {
            no_block(f)?;
            let secs = |f: &Entry| -> Result<f32, String> {
                let v = one_number(f)?;
                if v < 0.0 {
                    return Err(at(f, format!("{} is seconds, ≥ 0", f.key)));
                }
                Ok(v)
            };
            match f.key.as_str() {
                "sound" => k.sound = string(f)?,
                "volume" => k.volume = volume(f)?,
                "fadeIn" => k.fade_in = secs(f)?,
                "fadeOut" => k.fade_out = secs(f)?,
                "end" => k.end = Some(secs(f)?),
                "loop" => k.looping = flag(f)?,
                "bus" => k.bus = bus(f)?,
                _ => {
                    return Err(unknown(
                        f,
                        "music key entry",
                        "sound, volume, fadeIn, fadeOut, end, loop or bus",
                    ));
                }
            }
        }
        if k.sound.is_empty() {
            return Err(at(
                e,
                "a music key names its sound: `sound \"audio/theme.wav\"`",
            ));
        }
        if k.end.is_some_and(|end| end <= k.time) {
            return Err(at(e, "a music key's end comes after its time"));
        }
        ks.push(k);
    }
    Ok(sorted(ks, |k| k.time))
}

/// A caption as written: time, end if given, text, style, anchor.
type RawCaption = (f32, Option<f32>, String, Option<String>, Option<String>);

fn caption(block: &Block, read: &mut Read<'_>) -> Result<CaptionTrack, String> {
    let mut c = CaptionTrack::default();
    let mut raw: Vec<RawCaption> = Vec::new();
    for e in &block.entries {
        match e.key.as_str() {
            "style" => {
                no_block(e)?;
                c.style = word(e)?.to_string();
            }
            "anchor" => {
                no_block(e)?;
                c.anchor = word(e)?.to_string();
            }
            "captions" => {
                no_block(e)?;
                let (file, offset) = match e.values.as_slice() {
                    [Value::String(f)] => (f.clone(), 0.0),
                    [Value::String(f), Value::Number(n)] => (f.clone(), n.as_f32()),
                    _ => {
                        return Err(at(
                            e,
                            "`captions \"<file>.srt\" [offset seconds]` (or a word-timestamp .json)",
                        ));
                    }
                };
                let text = read(&file).map_err(|m| at(e, format!("captions '{file}': {m}")))?;
                let cues =
                    import(&file, &text).map_err(|m| at(e, format!("captions '{file}': {m}")))?;
                for q in cues {
                    let start = q.start + offset;
                    if start < 0.0 {
                        return Err(at(
                            e,
                            format!(
                                "captions '{file}': a cue starts before 0 with offset {offset}"
                            ),
                        ));
                    }
                    raw.push((start, Some(q.end + offset), q.text, None, None));
                }
            }
            "key" => {
                let time = key_time(e)?;
                let (mut end, mut text, mut style, mut anchor) = (None, None, None, None);
                for f in key_block(e)? {
                    no_block(f)?;
                    match f.key.as_str() {
                        "text" => text = Some(string(f)?),
                        "end" => end = Some(one_number(f)?),
                        "style" => style = Some(word(f)?.to_string()),
                        "anchor" => anchor = Some(word(f)?.to_string()),
                        _ => {
                            return Err(unknown(
                                f,
                                "caption key entry",
                                "text, end, style or anchor",
                            ));
                        }
                    }
                }
                let text = text.ok_or_else(|| at(e, "a caption key has `text \"...\"`"))?;
                if end.is_some_and(|x| x <= time) {
                    return Err(at(e, "a caption's end comes after its time"));
                }
                raw.push((time, end, text, style, anchor));
            }
            _ => {
                return Err(unknown(
                    e,
                    "caption track entry",
                    "style, anchor, captions or key",
                ));
            }
        }
    }
    let raw = sorted(raw, |r| r.0);
    for (i, (time, end, text, style, anchor)) in raw.iter().enumerate() {
        let next = raw.get(i + 1).map(|r| r.0);
        let end = end.unwrap_or_else(|| {
            next.unwrap_or(time + CAPTION_SECONDS)
                .min(time + CAPTION_SECONDS)
        });
        c.keys.push(CaptionKey {
            time: *time,
            end,
            text: text.clone(),
            style: style.clone(),
            anchor: anchor.clone(),
        });
    }
    Ok(c)
}

/// A caption file by its extension: SubRip, or word timestamps in JSON.
fn import(file: &str, text: &str) -> Result<Vec<Cue>, String> {
    let lower = file.to_ascii_lowercase();
    if lower.ends_with(".srt") {
        return captions::parse_srt(text);
    }
    if lower.ends_with(".json") {
        #[cfg(feature = "import")]
        return captions::parse_word_json(text, CAPTION_CHARS, CAPTION_GAP);
        #[cfg(not(feature = "import"))]
        return Err("word-timestamp JSON is imported by the cook (og cook)".into());
    }
    Err("a caption file is .srt (SubRip) or .json (word timestamps)".into())
}

fn events(block: &Block) -> Result<Vec<EventKey>, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "event")? {
        let mut k = EventKey {
            time: key_time(e)?,
            ..EventKey::default()
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "do" => k.action = string(f)?,
                "retroactive" => k.retroactive = flag(f)?,
                _ => return Err(unknown(f, "event key entry", "do or retroactive")),
            }
        }
        let name = k.action.split(':').next().unwrap_or("");
        let mut chars = name.chars();
        let valid = matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c == '_')
            && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid {
            return Err(at(
                e,
                "an event key runs a command-channel action: `do \"teleport:crate:0,4,0\"` (the action, then its `:`-separated arguments)",
            ));
        }
        ks.push(k);
    }
    Ok(sorted(ks, |k| k.time))
}

fn transitions(block: &Block) -> Result<Vec<TransitionKey>, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "transition")? {
        let mut k = TransitionKey {
            time: key_time(e)?,
            kind: TransitionKind::Fade,
            mode: TransitionMode::Through,
            duration: 1.0,
            color: Vec3::ZERO,
            from: 0,
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "kind" => {
                    let w = word(f)?;
                    k.kind = TransitionKind::from_name(w).ok_or_else(|| {
                        at(
                            f,
                            format!(
                                "transition kind '{w}' is not {}",
                                TransitionKind::NAMES.join(", ")
                            ),
                        )
                    })?;
                }
                "mode" => {
                    let w = word(f)?;
                    k.mode = TransitionMode::from_name(w).ok_or_else(|| {
                        at(
                            f,
                            format!("transition mode '{w}' is not through, in or out"),
                        )
                    })?;
                }
                "duration" => {
                    k.duration = one_number(f)?;
                    if k.duration < 0.0 {
                        return Err(at(f, "duration is seconds, ≥ 0"));
                    }
                }
                "color" => k.color = vec3(f)?,
                "from" => {
                    let w = word(f)?;
                    k.from = SIDES.iter().position(|s| *s == w).ok_or_else(|| {
                        at(f, format!("from '{w}' is not left, right, top or bottom"))
                    })? as u32;
                }
                _ => {
                    return Err(unknown(
                        f,
                        "transition key entry",
                        "kind, mode, duration, color or from",
                    ));
                }
            }
        }
        ks.push(k);
    }
    Ok(sorted(ks, |k| k.time))
}

fn markers(block: &Block) -> Result<Vec<MarkerKey>, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "marker")? {
        let mut k = MarkerKey {
            time: key_time(e)?,
            name: String::new(),
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "name" => k.name = string(f)?,
                _ => return Err(unknown(f, "marker key entry", "name")),
            }
        }
        if k.name.is_empty() || k.name.len() > 63 {
            return Err(at(e, "a marker key has a `name \"...\"` of 1 to 63 bytes"));
        }
        ks.push(k);
    }
    Ok(sorted(ks, |k| k.time))
}

// ── writing ────────────────────────────────────────────────────────────────

fn key_entry(time: f32, block: Block) -> Entry {
    Entry::new("key").with(num(time)).with_block(block)
}

fn ease_entry(b: &mut Block, e: Ease) {
    if e != Ease::Linear {
        b.push(Entry::new("ease").with(Value::Word(e.name().into())));
    }
}

fn write_track(tr: &Track) -> Entry {
    let mut b = Block::new();
    let mut head = Entry::new("track").with(Value::Word(tr.kind().into()));
    match tr {
        Track::Camera(c) => {
            match &c.mode {
                CameraMode::Keys => {}
                CameraMode::Rail(p) => b.push(Entry::new("rail").with(Value::String(p.clone()))),
                CameraMode::Follow(p) => {
                    b.push(Entry::new("follow").with(Value::String(p.clone())))
                }
            }
            for k in &c.keys {
                let mut kb = Block::new();
                if let Some(p) = k.position {
                    kb.push(v3(Entry::new("position"), p));
                }
                if let Some(a) = k.at {
                    kb.push(Entry::new("at").with(num(a)));
                }
                if let Some(t) = k.target {
                    kb.push(v3(Entry::new("target"), t));
                }
                if let Some(f) = k.fov {
                    kb.push(Entry::new("fov").with(num(f)));
                }
                ease_entry(&mut kb, k.ease);
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Transform(t) => {
            head = head.with(Value::String(t.entity.clone()));
            for k in &t.keys {
                let mut kb = Block::new();
                if let Some(p) = k.position {
                    kb.push(v3(Entry::new("position"), p));
                }
                if let Some(q) = k.quaternion {
                    kb.push(
                        Entry::new("quaternion")
                            .with(num(q.x))
                            .with(num(q.y))
                            .with(num(q.z))
                            .with(num(q.w)),
                    );
                }
                if let Some(s) = k.scale {
                    kb.push(v3(Entry::new("scale"), s));
                }
                ease_entry(&mut kb, k.ease);
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Animation(a) => {
            head = head.with(Value::String(a.entity.clone()));
            if !a.model.is_empty() {
                b.push(Entry::new("model").with(Value::String(a.model.clone())));
            }
            for k in &a.keys {
                let mut kb = Block::new();
                kb.push(Entry::new("clip").with(Value::String(k.clip.clone())));
                kb.push(Entry::new("speed").with(num(k.speed)));
                kb.push(Entry::new("blendIn").with(num(k.blend_in)));
                kb.push(Entry::new("loop").with(boolean(k.looping)));
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Audio(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("sound").with(Value::String(k.sound.clone())));
                kb.push(Entry::new("volume").with(num(k.volume)));
                kb.push(Entry::new("pitch").with(num(k.pitch)));
                kb.push(Entry::new("bus").with(Value::Word(bus_name(k.bus).into())));
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Music(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("sound").with(Value::String(k.sound.clone())));
                kb.push(Entry::new("volume").with(num(k.volume)));
                kb.push(Entry::new("fadeIn").with(num(k.fade_in)));
                kb.push(Entry::new("fadeOut").with(num(k.fade_out)));
                if let Some(end) = k.end {
                    kb.push(Entry::new("end").with(num(end)));
                }
                kb.push(Entry::new("loop").with(boolean(k.looping)));
                kb.push(Entry::new("bus").with(Value::Word(bus_name(k.bus).into())));
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Caption(c) => {
            b.push(Entry::new("style").with(Value::Word(c.style.clone())));
            b.push(Entry::new("anchor").with(Value::Word(c.anchor.clone())));
            for k in &c.keys {
                let mut kb = Block::new();
                kb.push(Entry::new("text").with(Value::String(k.text.clone())));
                kb.push(Entry::new("end").with(num(k.end)));
                if let Some(s) = &k.style {
                    kb.push(Entry::new("style").with(Value::Word(s.clone())));
                }
                if let Some(a) = &k.anchor {
                    kb.push(Entry::new("anchor").with(Value::Word(a.clone())));
                }
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Event(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("do").with(Value::String(k.action.clone())));
                if k.retroactive {
                    kb.push(Entry::new("retroactive").with(boolean(true)));
                }
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Transition(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("kind").with(Value::Word(k.kind.name().into())));
                kb.push(Entry::new("mode").with(Value::Word(k.mode.name().into())));
                kb.push(Entry::new("duration").with(num(k.duration)));
                kb.push(v3(Entry::new("color"), k.color));
                kb.push(Entry::new("from").with(Value::Word(SIDES[k.from as usize].into())));
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Marker(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("name").with(Value::String(k.name.clone())));
                b.push(key_entry(k.time, kb));
            }
        }
    }
    head.with_block(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = r#"type timeline
version 1
---
name "Intro"
duration 8
path "flyby" {
  point -6 3 9
  point 0 2.5 7
  point 5 2 5
}
track camera {
  rail "flyby"
  key 0 {
    at 0
    target 0 1 0
    fov 50
  }
  key 6 {
    at 1
    ease cubicInOut
  }
}
track transform "crate" {
  key 3 {
    rotation 0 90 0
    position 1 2 3
  }
  key 1 {
    position 0 0 0
  }
}
track caption {
  captions "narration.srt" 0.5
  key 7 {
    text "Press \"Start\""
  }
}
track event {
  key 2 {
    do "teleport:crate:0,4,0"
    retroactive true
  }
}
track transition {
  key 7.5 {
    mode out
    duration 0.5
  }
}
track marker {
  key 3 {
    name "drop"
  }
}
"#;

    fn read(file: &str) -> Result<String, String> {
        assert_eq!(file, "narration.srt");
        Ok("1\n00:00:01,000 --> 00:00:02,500\nHello.\n\n2\n00:00:03,000 --> 00:00:04,000\nAgain.\n".into())
    }

    #[test]
    fn parses_sorts_imports_and_round_trips_canonically() {
        let t = Timeline::parse(TEXT, &mut read).unwrap();
        assert_eq!(t.name, "Intro");
        assert_eq!(t.duration, 8.0);
        let Track::Transform(tr) = &t.tracks[1] else {
            panic!()
        };
        assert_eq!(tr.keys[0].time, 1.0, "keys are sorted by time");
        let q = tr.keys[1].quaternion.unwrap();
        assert!(
            (q.y - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6,
            "{q:?}"
        );
        let Track::Caption(c) = &t.tracks[2] else {
            panic!()
        };
        let cues: Vec<_> = c
            .keys
            .iter()
            .map(|k| (k.time, k.end, k.text.as_str()))
            .collect();
        assert_eq!(
            cues,
            vec![
                (1.5, 3.0, "Hello."),
                (3.5, 4.5, "Again."),
                (7.0, 10.0, "Press \"Start\"")
            ]
        );
        let text = t.to_text();
        let back = Timeline::parse(&text, &mut |_| Err("no files".into())).unwrap();
        assert_eq!(
            back, t,
            "the canonical text reads back to the same timeline:\n{text}"
        );
        assert_eq!(back.to_text(), text);
    }

    #[test]
    fn refusals_name_the_line_and_the_fix() {
        let cases = [
            (
                "track camera {\n  key 0 {\n    position 0 0\n  }\n}\n",
                "line 6: `position` takes three numbers",
            ),
            ("track transform {\n}\n", "names its entity"),
            (
                "track camera {\n  key 1 {\n    at 0.5\n  }\n}\n",
                "add `rail",
            ),
            (
                "duration 1\ntrack marker {\n  key 2 {\n    name \"x\"\n  }\n}\n",
                "after the timeline's duration",
            ),
            (
                "track event {\n  key 1 {\n    do \"Teleport\"\n  }\n}\n",
                "command-channel action",
            ),
            (
                "track audio {\n  key 1 {\n    volume 1\n  }\n}\n",
                "names its sound",
            ),
            (
                "path \"p\" {\n  kind bezier\n  point 0 0 0\n  point 1 0 0\n}\n",
                "path 'p': an open bezier",
            ),
            ("track wobble {\n}\n", "track kind 'wobble'"),
        ];
        for (body, want) in cases {
            let text = format!("type timeline\nversion 1\n---\n{body}");
            let e = Timeline::parse(&text, &mut |_| Err("none".into())).unwrap_err();
            assert!(e.contains(want), "{body}: {e}");
        }
    }
}
