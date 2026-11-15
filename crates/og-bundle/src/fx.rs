//! Effect assets as text: particle emitters (`.ogt` of `type particles`) and
//! colour lookup tables (`.cube`). The cook validates both and stores them in
//! blobs; the runtime turns them into og-render's [`ParticleEmitter`] and
//! [`ColorLut`]. One parser serves both sides, so what cooks loads.
//!
//! An emitter's lines are the rows of [`EMITTER_LINES`]: every line is
//! optional (the defaults are og-render's: a small fountain of soft white
//! points), the parser refuses a line the table does not list, and `og api
//! gen` writes the table into the `effect-files` guide.
//!
//! `array` names a texture array's asset id and picks each particle's layer
//! as `sheet` picks a frame: `life <cycles>` steps through the layers over
//! the particle's life (by age), `fps <n>` at a fixed rate, `random` one
//! layer per particle for its whole life, `off` (the default) the first;
//! `<layers>` uses the first that many (default every layer). Every
//! particle of the emitter is one draw whichever layers they show. A
//! billboard emitter with an array is refused by the renderer.
//!
//! `render mesh` draws a mesh per particle: a built-in shape (a unit `box`,
//! a `sphere` of diameter 1 or a `tetrahedron` shard) or the first mesh of
//! a model asset (a glTF). `subEmitter` fires another emitter asset at
//! this one's particle events (where each particle is born, dies or hits a
//! collider), `count` particles each, moving at their own start velocity
//! plus `inherit` times the parent's; `subEmitter life` has each particle
//! run the other emitter's own emission (rate, bursts, duration, looping)
//! from its birth while it lives, born where it is at each tick.

use crate::grammar::Line;
use og_core::ogt::{Document, Entry, Value};
use og_render_data::{Attractor, MAX_ATTRACTORS, MAX_SPLINE_POINTS, VectorField};
use og_render_data::{
    Burst, CollisionTarget, ColorCurve, ColorLut, Curve, EmitterHandle, EmitterShape,
    MAX_CURVE_KEYS, MAX_SUB_EMITTERS, MeshHandle, ParticleAlign, ParticleBlend, ParticleCollision,
    ParticleEmitter, ParticleEventKind, ParticleRender, SheetMode, SimulationSpace, SubEmitter,
    SubTrigger, TextureArrayHandle, TextureHandle,
};

/// The built-in meshes `render mesh <name>` names.
pub const PARTICLE_MESHES: &[&str] = &["box", "sphere", "tetrahedron"];

/// What a `render mesh` line names: a built-in shape or a model asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleMesh<'a> {
    /// One of [`PARTICLE_MESHES`].
    Builtin(&'a str),
    /// A model's asset id (its first mesh).
    Asset(&'a str),
}

/// How an emitter's references become the scene's handles. Each answers
/// `None` when the caller only checks the text (the cook): a mesh then
/// draws as handle 0 and a sub-emitter is left out.
pub struct EmitterLinks<'a> {
    pub texture: &'a mut dyn FnMut(&str) -> Result<Option<TextureHandle>, String>,
    pub array: &'a mut dyn FnMut(&str) -> Result<Option<TextureArrayHandle>, String>,
    pub mesh: &'a mut dyn FnMut(ParticleMesh<'_>) -> Result<Option<MeshHandle>, String>,
    pub emitter: &'a mut dyn FnMut(&str) -> Result<Option<EmitterHandle>, String>,
    /// An `effect` line's effect material (`type effect`).
    pub effect:
        &'a mut dyn FnMut(&str) -> Result<Option<og_render_data::EffectMaterialHandle>, String>,
}

/// The line `og cook` writes in place of `field "<texture>" ...`: the
/// baked grid (`fieldGrid <cells> <size> <strength>` and the vectors, x
/// fastest), which the runtime reads back.
pub const FIELD_GRID: &str = "fieldGrid";

/// Every line an emitter takes, in the order the guide lists them: the
/// parser refuses any other (`crate::grammar`).
pub const EMITTER_LINES: &[Line] = &[
    Line::new(
        "maxParticles",
        "<n>",
        "256",
        "The ring of particles, 1 to 1048576: the newest overwrite the oldest.",
        "maxParticles 512",
    ),
    Line::new(
        "shape",
        "point | sphere <radius> [surface] | cone <angle°> <radius> | box <x> <y> <z> | spline <x> <y> <z> ... [closed]",
        "cone 15 0.05",
        "Where particles are born and which way they start: a box by its half extents, a spline through 2 to 8 Catmull-Rom points, emitted evenly along it and along its tangent.",
        "shape sphere 0.5 surface",
    ),
    Line::new(
        "spread",
        "<degrees>",
        "0",
        "Random deviation from the shape's direction; 180 is every direction.",
        "spread 30",
    ),
    Line::new(
        "rate",
        "<per second>",
        "40",
        "Particles born per second.",
        "rate 120",
    ),
    Line::new(
        "burst",
        "<time> <count> [<cycles> <interval>]",
        "",
        "Particles born at once at a time of each cycle, repeated `cycles` times `interval` apart; any number of lines.",
        "burst 0 30 3 0.1",
    ),
    Line::new(
        "duration",
        "<seconds>",
        "1",
        "Seconds per emission cycle.",
        "duration 0.5",
    ),
    Line::new(
        "looping",
        "true | false",
        "true",
        "Start a new cycle at the end of each.",
        "looping false",
    ),
    Line::new(
        "lifetime",
        "<min> [<max>]",
        "1 1.5",
        "Seconds a particle lives.",
        "lifetime 0.4 0.8",
    ),
    Line::new(
        "speed",
        "<min> [<max>]",
        "1.5 2.5",
        "Metres per second at birth.",
        "speed 4 6",
    ),
    Line::new(
        "size",
        "<min> [<max>]",
        "0.08 0.12",
        "Metres across at birth.",
        "size 0.2",
    ),
    Line::new(
        "sizeOverLife",
        "<t> <value> ...",
        "",
        "A curve that scales the size over the life (1 when left out): the width and, without `heightOverLife`, the height.",
        "sizeOverLife 0 0.2 0.3 1 1 0",
    ),
    Line::new(
        "heightOverLife",
        "<t> <value> ...",
        "",
        "A curve that scales the height alone over the life (along the sprite's up, or the velocity when stretched; a mesh's y): a tall flame, a flat shockwave. Left out, the height is the width.",
        "heightOverLife 0 2 1 3",
    ),
    Line::new(
        "colorOverLife",
        "{ key <t> <r> <g> <b> <a> ... }",
        "",
        "Linear colour keys over the life, at most 8; left out, white fading over the last quarter.",
        "colorOverLife {\n  key 0 4 2 0.6 1\n  key 1 1 0.2 0 0\n}",
    ),
    Line::new(
        "randomColorOverLife",
        "{ key <t> <r> <g> <b> <a> ... }",
        "",
        "A second colour over the life: each particle's colour lies between colorOverLife and this at a share of its own (embers from orange to deep red). Left out, every particle takes colorOverLife.",
        "randomColorOverLife {\n  key 0 4 0.6 0.2 1\n  key 1 0.8 0.05 0 0\n}",
    ),
    Line::new(
        "color",
        "<r> <g> <b> [<a>]",
        "1 1 1 1",
        "A linear tint over the colour keys; above 1 blooms.",
        "color 2 1 0.5",
    ),
    Line::new(
        "rotation",
        "<min> [<max>]",
        "0",
        "The start rotation in degrees.",
        "rotation 0 360",
    ),
    Line::new(
        "angularVelocity",
        "<min> [<max>]",
        "0",
        "Degrees per second.",
        "angularVelocity -90 90",
    ),
    Line::new(
        "gravity",
        "<x> <y> <z>",
        "0 -2 0",
        "Metres per second squared.",
        "gravity 0 -9.8 0",
    ),
    Line::new(
        "drag",
        "<per second>",
        "0",
        "How fast motion relative to the air dies.",
        "drag 1.5",
    ),
    Line::new(
        "wind",
        "<0..1>",
        "0",
        "The share of the air's motion (og.Wind and every og.WindMotor where the system is) that `drag` closes on: smoke and dust drift downwind; 0 slows to rest.",
        "wind 1",
    ),
    Line::new(
        "noise",
        "<strength> [<frequency> <scroll>]",
        "0 0.5 0.3",
        "Turbulence: value noise pushing the particles.",
        "noise 0.6 0.8 0.2",
    ),
    Line::new(
        "texture",
        "\"<id>\"",
        "",
        "A texture asset; left out, a soft dot.",
        "texture \"textures/spark.png\"",
    ),
    Line::new(
        "sheet",
        "<columns> <rows> [<frames> [off | life <cycles> | fps <n> | random]]",
        "1 1",
        "The texture as a sprite sheet: a frame by age, at a rate, or one at random.",
        "sheet 4 4 16 life 1",
    ),
    Line::new(
        "blend",
        "additive | alpha",
        "",
        "Left out: alpha for a lit mesh (a solid chip; added it would glow white), additive for everything else.",
        "blend alpha",
    ),
    Line::new(
        "space",
        "world | local",
        "world",
        "Particles left behind in the world, or carried with the emitter.",
        "space local",
    ),
    Line::new(
        "align",
        "camera | velocity [<stretch>] | upright [<x> <y> <z>] | facing <x> <y> <z>",
        "camera",
        "Billboards face the camera, stretch along their velocity (stretch 0.05 when left out), stand on an axis of the emitter turning to face the eye (0 1 0 when left out: flames), or lie across one whatever the camera (facing 0 1 0: flat on the ground).",
        "align facing 0 1 0",
    ),
    Line::new(
        "softDistance",
        "<metres>",
        "0",
        "Fade where a billboard meets the scene behind it.",
        "softDistance 0.3",
    ),
    Line::new(
        "collision",
        "[<bounce> [<friction>]] [depth | colliders]",
        "",
        "Bounce (0.4) with friction (0.2) off the drawn depth (what the camera sees), or off the scene's og.Collider shapes on or off screen; sprite mode always meets the colliders.",
        "collision 0.3 0.1 colliders",
    ),
    Line::new(
        "render",
        "billboard | sprite [<layer>] | mesh box | sphere | tetrahedron | \"<model id>\"",
        "billboard",
        "3D billboards, 2D sprites in a sprite layer, or a small mesh per particle that tumbles with its rotation (a model's first mesh).",
        "render mesh tetrahedron",
    ),
    Line::new(
        "lit",
        "true | false",
        "false",
        "Billboards and meshes shaded by the sun, the sky and local lights.",
        "lit true",
    ),
    Line::new(
        "roughness",
        "<0..1>",
        "1",
        "A lit mesh's surface: 1 matte; smoother reflects over an alpha body (glass shards).",
        "roughness 0.2",
    ),
    Line::new(
        "distortion",
        "<0..1>",
        "0",
        "Billboards bend the scene behind them by this fraction of the screen at most: heat haze, shock rings.",
        "distortion 0.05",
    ),
    Line::new(
        "events",
        "birth | death | collision ...",
        "",
        "Send og.ParticleEvent for these; the emitter's systems then simulate in the tick.",
        "events death collision",
    ),
    Line::new(
        "subEmitter",
        "birth | death | collision \"<id>\" <count> [<inherit>] | life \"<id>\" [<inherit>]",
        "",
        "Another emitter's particles born where this one's are born, die or bounce (count 1 to 4096, moving at their start velocity plus `inherit` times the parent's), or each particle running the other's own emission while it lives; up to 4, one level deep.",
        "subEmitter death \"fx/sparks.ogt\" 8 0.3",
    ),
    Line::new(
        "glow",
        "<strength>",
        "0",
        "Sprite mode: strength in the 2D glow buffer (PostEffects.glow).",
        "glow 1.5",
    ),
    Line::new(
        "array",
        "\"<id>\" [<layers>] [off | life <cycles> | fps <n> | random]",
        "",
        "Sprite mode: a layer of a texture array (`type texture-array`) per particle instead of `texture`, picked as `sheet` picks a frame from the first `layers`; one draw per emitter.",
        "array \"fx/smoke.ogt\" random",
    ),
    Line::new(
        "round",
        "true | false",
        "false",
        "Fade each particle to the soft dot's round edge over its texture or layer.",
        "round true",
    ),
    Line::new(
        "effect",
        "\"<id>\"",
        "",
        "Draw billboards or mesh particles with an effect material (`type effect`, §2).",
        "effect \"fx/fire.ogt\"",
    ),
    Line::new(
        "speedOverLife",
        "<t> <value> ...",
        "",
        "A curve that scales the velocity over the life (a burst that slows).",
        "speedOverLife 0 1 1 0.2",
    ),
    Line::new(
        "spinOverLife",
        "<t> <value> ...",
        "",
        "A curve that scales the angular velocity over the life.",
        "spinOverLife 0 1 1 0",
    ),
    Line::new(
        "rateOverTime",
        "<t> <value> ...",
        "",
        "A curve that scales `rate` over each cycle (t a fraction of `duration`).",
        "rateOverTime 0 1 0.8 1 1 0",
    ),
    Line::new(
        "inheritVelocity",
        "<fraction>",
        "0",
        "The share of the emitter's own velocity, in world space, particles start with.",
        "inheritVelocity 0.5",
    ),
    Line::new(
        "attractor",
        "<x> <y> <z> <strength> [<radius>]",
        "",
        "A point in the emitter's space that pulls, or pushes when negative; up to 4.",
        "attractor 0 1 0 6 2",
    ),
    Line::new(
        "field",
        "curl <strength> [<frequency> <scroll>] | \"<texture>\" <strength> <size>",
        "",
        "Divergence-free curl noise (frequency 0.5, scroll 0.2), or a vector grid `og cook` bakes from an image N² pixels wide and N high (N to 16): pixel x + z·N holds cell (x, y, z), y from the bottom, rgb 0..1 as -1..1.",
        "field curl 3 0.8 0.2",
    ),
    Line::new(
        FIELD_GRID,
        "<cells> <size> <strength> <vectors>",
        "",
        "What `og cook` writes in place of a texture field: the baked grid, x fastest. Not written by hand.",
        "fieldGrid 1 2 1 0 1 0",
    ),
];

/// `<t> <value>` pairs, 1 to [`MAX_CURVE_KEYS`] of them.
fn curve(e: &Entry) -> Result<Curve, String> {
    let v = nums(e)?;
    if v.is_empty() || v.len() % 2 != 0 || v.len() / 2 > MAX_CURVE_KEYS {
        return Err(format!(
            "line {}: {} is 1 to {MAX_CURVE_KEYS} <t> <value> pairs",
            e.line, e.key
        ));
    }
    Ok(Curve::new(
        &v.chunks(2).map(|c| (c[0], c[1])).collect::<Vec<_>>(),
    ))
}

/// The `.ogt` type of an emitter file.
pub const EMITTER_TYPE: &str = "particles";

fn nums(e: &Entry) -> Result<Vec<f32>, String> {
    e.numbers(0).map_err(|x| x.to_string())
}

fn range(e: &Entry) -> Result<[f32; 2], String> {
    match nums(e)?[..] {
        [v] => Ok([v, v]),
        [a, b] => Ok([a, b]),
        _ => Err(format!(
            "line {}: `{}` is one number or a min and a max",
            e.line, e.key
        )),
    }
}

fn one(e: &Entry) -> Result<f32, String> {
    match nums(e)?[..] {
        [v] => Ok(v),
        _ => Err(format!("line {}: `{}` takes one number", e.line, e.key)),
    }
}

fn word(e: &Entry, i: usize) -> Result<&str, String> {
    e.values.get(i).and_then(Value::as_word).ok_or_else(|| {
        format!(
            "line {}: `{}` needs a word at position {}",
            e.line,
            e.key,
            i + 1
        )
    })
}

/// A `sheet` or `array` line's mode: off | life <cycles> | fps <n> | random.
fn sheet_mode(line: usize, key: &str, rest: &[Value]) -> Result<SheetMode, String> {
    Ok(match rest.first().and_then(Value::as_word) {
        None | Some("off") => SheetMode::Off,
        Some("random") => SheetMode::Random,
        Some(m @ ("life" | "fps")) => {
            let n = rest
                .get(1)
                .and_then(Value::as_number)
                .map(|n| n.as_f32())
                .ok_or_else(|| format!("line {line}: {key} ... {m} <number>"))?;
            if m == "life" {
                SheetMode::OverLife { cycles: n }
            } else {
                SheetMode::Fps(n)
            }
        }
        Some(other) => {
            return Err(format!(
                "line {line}: {key} mode `{other}` (off, life, fps, random)"
            ));
        }
    })
}

/// The texture and texture array asset ids an emitter names (the cook
/// checks they exist).
pub fn emitter_references(text: &str) -> Vec<String> {
    Document::parse(text)
        .map(|d| {
            d.body
                .entries
                .iter()
                .filter_map(|e| match e.key.as_str() {
                    "texture" | "array" => e.values.first(),
                    "render" => e.values.get(1),
                    "subEmitter" => e.values.get(1),
                    "effect" | "field" => e.values.first(),
                    _ => None,
                })
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Parses an emitter file. `texture` resolves a texture asset id to the
/// scene's handle (the cook passes one that accepts anything); an `array`
/// line's texture array resolves to none (see [`parse_emitter_with`]).
pub fn parse_emitter(
    text: &str,
    texture: &mut dyn FnMut(&str) -> Result<Option<TextureHandle>, String>,
) -> Result<ParticleEmitter, String> {
    parse_emitter_with(text, texture, &mut |_| Ok(None))
}

/// [`parse_emitter`] with `array` resolving a texture array's asset id
/// (an `array` line) to the scene's handle; meshes and sub-emitters are
/// not resolved (see [`parse_emitter_linked`]).
pub fn parse_emitter_with(
    text: &str,
    texture: &mut dyn FnMut(&str) -> Result<Option<TextureHandle>, String>,
    array: &mut dyn FnMut(&str) -> Result<Option<TextureArrayHandle>, String>,
) -> Result<ParticleEmitter, String> {
    parse_emitter_linked(
        text,
        &mut EmitterLinks {
            texture,
            array,
            mesh: &mut |_| Ok(None),
            emitter: &mut |_| Ok(None),
            effect: &mut |_| Ok(None),
        },
    )
}

/// Parses an emitter file, resolving every reference through `links`.
pub fn parse_emitter_linked(
    text: &str,
    links: &mut EmitterLinks<'_>,
) -> Result<ParticleEmitter, String> {
    let EmitterLinks {
        texture,
        array,
        mesh,
        emitter,
        effect,
    } = links;
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != EMITTER_TYPE {
        return Err(format!(
            "an emitter is `type {EMITTER_TYPE}`, not `type {}`",
            doc.ty
        ));
    }
    if doc.version != 1 {
        return Err(format!(
            "emitter version {} (this engine reads 1)",
            doc.version
        ));
    }
    let mut em = ParticleEmitter::default();
    let mut bursts = Vec::new();
    let mut subs = 0;
    let mut blend = None;
    for e in &doc.body.entries {
        let line = e.line;
        if crate::grammar::find(EMITTER_LINES, &e.key, &[]).is_none() {
            return Err(format!(
                "line {line}: unknown emitter setting `{}` (an emitter takes {})",
                e.key,
                crate::grammar::keys(EMITTER_LINES, &[])
            ));
        }
        match e.key.as_str() {
            "maxParticles" => {
                let v = one(e)?;
                if !(1.0..=1_048_576.0).contains(&v) || v.fract() != 0.0 {
                    return Err(format!(
                        "line {line}: maxParticles is a whole number from 1 to 1048576"
                    ));
                }
                em.max_particles = v as u32;
            }
            "shape" => {
                let n = |i: usize| {
                    e.values
                        .get(i)
                        .and_then(Value::as_number)
                        .map(|n| n.as_f32())
                        .ok_or_else(|| {
                            format!(
                                "line {line}: shape {} needs number {i}",
                                word(e, 0).unwrap_or("?")
                            )
                        })
                };
                em.shape = match word(e, 0)? {
                    "point" => EmitterShape::Point,
                    "sphere" => EmitterShape::Sphere {
                        radius: n(1)?,
                        surface: e.values.get(2).and_then(Value::as_word) == Some("surface"),
                    },
                    "cone" => EmitterShape::Cone {
                        angle_degrees: n(1)?,
                        radius: n(2)?,
                    },
                    "box" => EmitterShape::Box {
                        half_extents: glam::Vec3::new(n(1)?, n(2)?, n(3)?),
                    },
                    "spline" => {
                        let closed = e.values.last().and_then(Value::as_word) == Some("closed");
                        let v: Vec<f32> = e.values[1..e.values.len() - closed as usize]
                            .iter()
                            .map(|v| v.as_number().map(|n| n.as_f32()))
                            .collect::<Option<_>>()
                            .ok_or_else(|| {
                                format!("line {line}: shape spline <x> <y> <z> ... [closed]")
                            })?;
                        let count = v.len() / 3;
                        if v.len() % 3 != 0 || !(2..=MAX_SPLINE_POINTS).contains(&count) {
                            return Err(format!(
                                "line {line}: shape spline takes 2 to {MAX_SPLINE_POINTS} points of <x> <y> <z>"
                            ));
                        }
                        let mut points = [glam::Vec3::ZERO; MAX_SPLINE_POINTS];
                        for (i, c) in v.chunks(3).enumerate() {
                            points[i] = glam::Vec3::new(c[0], c[1], c[2]);
                        }
                        EmitterShape::Spline {
                            points,
                            count: count as u32,
                            closed,
                        }
                    }
                    other => {
                        return Err(format!(
                            "line {line}: shape `{other}` (point, sphere, cone, box, spline)"
                        ));
                    }
                };
            }
            "spread" => em.spread_degrees = one(e)?,
            "rate" => em.rate = one(e)?.max(0.0),
            "burst" => {
                let v = nums(e)?;
                let burst = match v[..] {
                    [time, count] => Burst::once(time, count as u32),
                    [time, count, cycles, interval] => Burst {
                        time,
                        count: count as u32,
                        cycles: cycles as u32,
                        interval,
                    },
                    _ => {
                        return Err(format!(
                            "line {line}: burst <time> <count> [<cycles> <interval>]"
                        ));
                    }
                };
                bursts.push(burst);
            }
            "duration" => em.duration = one(e)?.max(1e-3),
            "looping" => {
                em.looping = e
                    .values
                    .first()
                    .and_then(Value::as_bool)
                    .ok_or_else(|| format!("line {line}: looping true | false"))?
            }
            "lifetime" => em.lifetime = range(e)?,
            "speed" => em.speed = range(e)?,
            "size" => em.size = range(e)?,
            "sizeOverLife" => {
                let v = nums(e)?;
                if v.is_empty() || v.len() % 2 != 0 || v.len() / 2 > MAX_CURVE_KEYS {
                    return Err(format!(
                        "line {line}: sizeOverLife is 1 to {MAX_CURVE_KEYS} <t> <value> pairs"
                    ));
                }
                let keys: Vec<(f32, f32)> = v.chunks(2).map(|c| (c[0], c[1])).collect();
                em.size_over_life = Curve::new(&keys);
            }
            key @ ("colorOverLife" | "randomColorOverLife") => {
                let block = e.block.as_ref().ok_or_else(|| {
                    format!("line {line}: {key} {{ key <t> <r> <g> <b> <a> ... }}")
                })?;
                let mut keys = Vec::new();
                for k in &block.entries {
                    let v = nums(k)?;
                    let [t, r, g, b, a] = v[..] else {
                        return Err(format!("line {}: key <t> <r> <g> <b> <a>", k.line));
                    };
                    keys.push((t, [r, g, b, a]));
                }
                if keys.is_empty() || keys.len() > MAX_CURVE_KEYS {
                    return Err(format!("line {line}: {key} has 1 to {MAX_CURVE_KEYS} keys"));
                }
                if key == "colorOverLife" {
                    em.color_over_life = ColorCurve::new(&keys);
                } else {
                    em.random_color_over_life = Some(ColorCurve::new(&keys));
                }
            }
            "color" => {
                em.color = match nums(e)?[..] {
                    [r, g, b] => [r, g, b, 1.0],
                    [r, g, b, a] => [r, g, b, a],
                    _ => return Err(format!("line {line}: color <r> <g> <b> [<a>]")),
                }
            }
            "rotation" => em.rotation_degrees = range(e)?,
            "angularVelocity" => em.angular_velocity_degrees = range(e)?,
            "gravity" => {
                let [x, y, z] = nums(e)?[..] else {
                    return Err(format!("line {line}: gravity <x> <y> <z>"));
                };
                em.gravity = glam::Vec3::new(x, y, z);
            }
            "drag" => em.drag = one(e)?.max(0.0),
            "wind" => em.wind = one(e)?.clamp(0.0, 1.0),
            "noise" => {
                let v = nums(e)?;
                em.noise.strength = *v.first().ok_or_else(|| {
                    format!("line {line}: noise <strength> [<frequency> <scroll>]")
                })?;
                if let Some(f) = v.get(1) {
                    em.noise.frequency = *f;
                }
                if let Some(s) = v.get(2) {
                    em.noise.scroll = *s;
                }
            }
            "texture" => {
                let id = e
                    .values
                    .first()
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("line {line}: texture \"<asset id>\""))?;
                em.texture = texture(id).map_err(|r| format!("line {line}: texture {id}: {r}"))?;
            }
            "sheet" => {
                let v: Vec<f32> = e
                    .values
                    .iter()
                    .take_while(|v| v.as_number().is_some())
                    .filter_map(|v| v.as_number().map(|n| n.as_f32()))
                    .collect();
                if v.len() < 2 || v[0] < 1.0 || v[1] < 1.0 {
                    return Err(format!(
                        "line {line}: sheet <columns> <rows> [<frames> [mode]]"
                    ));
                }
                em.sheet.columns = v[0] as u32;
                em.sheet.rows = v[1] as u32;
                em.sheet.frames = v.get(2).map_or(0, |f| *f as u32);
                let rest = &e.values[v.len()..];
                em.sheet.mode = sheet_mode(line, "sheet", rest)?;
            }
            "blend" => {
                blend = Some(match word(e, 0)? {
                    "additive" => ParticleBlend::Additive,
                    "alpha" => ParticleBlend::Alpha,
                    other => return Err(format!("line {line}: blend `{other}` (additive, alpha)")),
                });
            }
            "space" => {
                em.space = match word(e, 0)? {
                    "world" => SimulationSpace::World,
                    "local" => SimulationSpace::Local,
                    other => return Err(format!("line {line}: space `{other}` (world, local)")),
                }
            }
            "align" => {
                em.align = match word(e, 0)? {
                    "camera" => ParticleAlign::Camera,
                    "velocity" => ParticleAlign::Velocity {
                        stretch: e
                            .values
                            .get(1)
                            .and_then(Value::as_number)
                            .map_or(0.05, |n| n.as_f32()),
                    },
                    m @ ("upright" | "facing") => {
                        let axis = match e.values[1..]
                            .iter()
                            .map(|v| v.as_number().map(|n| n.as_f32()))
                            .collect::<Option<Vec<_>>>()
                            .as_deref()
                        {
                            Some([x, y, z]) => glam::Vec3::new(*x, *y, *z),
                            Some([]) if m == "upright" => glam::Vec3::Y,
                            _ => glam::Vec3::ZERO,
                        };
                        if axis.length_squared() < 1e-12 {
                            return Err(format!(
                                "line {line}: align {m} takes an axis <x> <y> <z>"
                            ));
                        }
                        if m == "upright" {
                            ParticleAlign::Upright { axis }
                        } else {
                            ParticleAlign::Facing { normal: axis }
                        }
                    }
                    other => {
                        return Err(format!(
                            "line {line}: align `{other}` (camera, velocity <stretch>, upright <axis>, facing <axis>)"
                        ));
                    }
                }
            }
            "softDistance" => em.soft_distance = one(e)?.max(0.0),
            "collision" => {
                // <bounce> <friction> numbers, then what it meets.
                let d = ParticleCollision::default();
                let mut v = Vec::new();
                let mut against = d.against;
                for (i, value) in e.values.iter().enumerate() {
                    match (value.as_number(), value.as_word()) {
                        (Some(n), _) if i < 2 && v.len() == i => v.push(n.as_f32()),
                        (_, Some("depth")) if i == e.values.len() - 1 => {
                            against = CollisionTarget::Depth
                        }
                        (_, Some("colliders")) if i == e.values.len() - 1 => {
                            against = CollisionTarget::Colliders
                        }
                        _ => {
                            return Err(format!(
                                "line {line}: collision [<bounce> [<friction>]] [depth | colliders]"
                            ));
                        }
                    }
                }
                em.collision = Some(ParticleCollision {
                    bounce: v.first().copied().unwrap_or(d.bounce),
                    friction: v.get(1).copied().unwrap_or(d.friction),
                    against,
                });
            }
            "render" => {
                em.render = match word(e, 0)? {
                    "billboard" => ParticleRender::Billboard,
                    "sprite" => ParticleRender::Sprite {
                        layer: e
                            .values
                            .get(1)
                            .and_then(Value::as_number)
                            .map_or(0, |n| n.as_f32() as i32),
                    },
                    "mesh" => {
                        let what = match e.values.get(1) {
                            Some(v) if v.as_str().is_some() => {
                                ParticleMesh::Asset(v.as_str().unwrap_or_default())
                            }
                            Some(v)
                                if v.as_word().is_some_and(|w| PARTICLE_MESHES.contains(&w)) =>
                            {
                                ParticleMesh::Builtin(v.as_word().unwrap_or_default())
                            }
                            _ => {
                                return Err(format!(
                                    "line {line}: render mesh <{}> or a model's \"<asset id>\"",
                                    PARTICLE_MESHES.join(" | ")
                                ));
                            }
                        };
                        let handle = mesh(what).map_err(|r| format!("line {line}: mesh: {r}"))?;
                        ParticleRender::Mesh {
                            mesh: handle.unwrap_or(MeshHandle::from_bits(0)),
                        }
                    }
                    other => {
                        return Err(format!(
                            "line {line}: render `{other}` (billboard, sprite <layer>, mesh <shape>)"
                        ));
                    }
                }
            }
            "lit" => {
                em.lit = e
                    .values
                    .first()
                    .and_then(Value::as_bool)
                    .ok_or_else(|| format!("line {line}: lit true | false"))?
            }
            "roughness" => {
                let v = one(e)?;
                if !(0.0..=1.0).contains(&v) {
                    return Err(format!("line {line}: roughness is from 0 to 1"));
                }
                em.roughness = v;
            }
            "distortion" => {
                let v = one(e)?;
                if !(0.0..=1.0).contains(&v) {
                    return Err(format!(
                        "line {line}: distortion is a fraction of the screen from 0 to 1"
                    ));
                }
                em.distortion = v;
            }
            "events" => {
                em.events.clear();
                for i in 0..e.values.len().max(1) {
                    let kind = match word(e, i)? {
                        "birth" => ParticleEventKind::Birth,
                        "death" => ParticleEventKind::Death,
                        "collision" => ParticleEventKind::Collision,
                        other => {
                            return Err(format!(
                                "line {line}: events `{other}` (birth, death, collision)"
                            ));
                        }
                    };
                    if !em.events.contains(&kind) {
                        em.events.push(kind);
                    }
                }
            }
            "subEmitter" => {
                let trigger = match word(e, 0)? {
                    "birth" => SubTrigger::Event(ParticleEventKind::Birth),
                    "death" => SubTrigger::Event(ParticleEventKind::Death),
                    "collision" => SubTrigger::Event(ParticleEventKind::Collision),
                    "life" => SubTrigger::Life,
                    other => {
                        return Err(format!(
                            "line {line}: subEmitter `{other}` (birth, death, collision, life)"
                        ));
                    }
                };
                let usage = || {
                    format!(
                        "line {line}: subEmitter <birth | death | collision> \"<emitter asset id>\" <count> [<inherit velocity>], or subEmitter life \"<emitter asset id>\" [<inherit velocity>]"
                    )
                };
                let id = e.values.get(1).and_then(Value::as_str).ok_or_else(usage)?;
                let num = |i: usize| {
                    e.values
                        .get(i)
                        .and_then(Value::as_number)
                        .map(|n| n.as_f32())
                };
                // `life` runs the child's own emission: no count.
                let (count, inherit) = if trigger == SubTrigger::Life {
                    if e.values.len() > 3 {
                        return Err(usage());
                    }
                    (0.0, num(2))
                } else {
                    let count = num(2).ok_or_else(usage)?;
                    if !(1.0..=4096.0).contains(&count) || count.fract() != 0.0 {
                        return Err(format!(
                            "line {line}: subEmitter's count is a whole number from 1 to 4096"
                        ));
                    }
                    (count, num(3))
                };
                subs += 1;
                if subs > MAX_SUB_EMITTERS {
                    return Err(format!(
                        "line {line}: at most {MAX_SUB_EMITTERS} subEmitter lines"
                    ));
                }
                if let Some(handle) =
                    emitter(id).map_err(|r| format!("line {line}: subEmitter {id}: {r}"))?
                {
                    em.sub_emitters.push(SubEmitter {
                        trigger,
                        emitter: handle,
                        count: count as u32,
                        inherit_velocity: inherit.unwrap_or(0.0),
                    });
                }
            }
            "glow" => em.glow = one(e)?.max(0.0),
            "array" => {
                let id = e.values.first().and_then(Value::as_str).ok_or_else(|| {
                    format!("line {line}: array \"<asset id>\" [<layers>] [mode]")
                })?;
                em.array = array(id).map_err(|r| format!("line {line}: array {id}: {r}"))?;
                let mut rest = &e.values[1..];
                em.sheet.columns = 1;
                em.sheet.rows = 1;
                em.sheet.frames = 0;
                if let Some(n) = rest.first().and_then(Value::as_number) {
                    let n = n.as_f32();
                    if !(n >= 1.0 && n.fract() == 0.0) {
                        return Err(format!(
                            "line {line}: array ... <layers> is a whole number from 1"
                        ));
                    }
                    em.sheet.frames = n as u32;
                    rest = &rest[1..];
                }
                em.sheet.mode = sheet_mode(line, "array", rest)?;
            }
            "round" => {
                em.round = e
                    .values
                    .first()
                    .and_then(Value::as_bool)
                    .ok_or_else(|| format!("line {line}: round true | false"))?
            }
            "effect" => {
                let id = e
                    .values
                    .first()
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("line {line}: effect \"<effect asset id>\""))?;
                em.effect = effect(id).map_err(|r| format!("line {line}: effect {id}: {r}"))?;
            }
            "heightOverLife" => em.height_over_life = Some(curve(e)?),
            "speedOverLife" => em.speed_over_life = Some(curve(e)?),
            "spinOverLife" => em.spin_over_life = Some(curve(e)?),
            "rateOverTime" => em.rate_over_time = Some(curve(e)?),
            "inheritVelocity" => em.inherit_velocity = one(e)?,
            "attractor" => {
                let a = match nums(e)?[..] {
                    [x, y, z, strength] => Attractor {
                        position: glam::Vec3::new(x, y, z),
                        strength,
                        radius: 0.0,
                    },
                    [x, y, z, strength, radius] => Attractor {
                        position: glam::Vec3::new(x, y, z),
                        strength,
                        radius: radius.max(0.0),
                    },
                    _ => {
                        return Err(format!(
                            "line {line}: attractor <x> <y> <z> <strength> [<radius>]"
                        ));
                    }
                };
                if em.attractors.len() == MAX_ATTRACTORS {
                    return Err(format!(
                        "line {line}: at most {MAX_ATTRACTORS} attractor lines"
                    ));
                }
                em.attractors.push(a);
            }
            "field" => {
                if e.values.first().and_then(Value::as_str).is_some() {
                    return Err(format!(
                        "line {line}: a field from a texture is baked by `og cook` (load the cooked emitter)"
                    ));
                }
                if word(e, 0)? != "curl" {
                    return Err(format!(
                        "line {line}: field curl <strength> [<frequency> <scroll>] | \"<texture>\" <strength> <size>"
                    ));
                }
                let v: Vec<f32> = e.values[1..]
                    .iter()
                    .map(|v| v.as_number().map(|n| n.as_f32()))
                    .collect::<Option<_>>()
                    .unwrap_or_default();
                let (strength, frequency, scroll) = match v[..] {
                    [s] => (s, 0.5, 0.2),
                    [s, f] => (s, f, 0.2),
                    [s, f, c] => (s, f, c),
                    _ => {
                        return Err(format!(
                            "line {line}: field curl <strength> [<frequency> <scroll>]"
                        ));
                    }
                };
                em.field = Some(VectorField::Curl {
                    strength,
                    frequency,
                    scroll,
                });
            }
            FIELD_GRID => {
                let v = nums(e)?;
                let [cells, size, strength, ..] = v[..] else {
                    return Err(format!(
                        "line {line}: {FIELD_GRID} <cells> <size> <strength> <vectors>"
                    ));
                };
                let vectors: Vec<glam::Vec3> = v[3..]
                    .chunks(3)
                    .map(|c| {
                        glam::Vec3::new(
                            c[0],
                            c.get(1).copied().unwrap_or(0.0),
                            c.get(2).copied().unwrap_or(0.0),
                        )
                    })
                    .collect();
                em.field = Some(VectorField::Grid {
                    strength,
                    size,
                    cells: cells as u32,
                    vectors,
                });
            }
            // A row of EMITTER_LINES with no arm; the table's tests parse every row.
            other => return Err(format!("line {line}: `{other}` is not read yet")),
        }
    }
    em.bursts = bursts;
    // Unsaid, a lit mesh is a solid thing (a chip, a shard, a splinter)
    // and blends alpha: added onto what is behind it, a lit grey chip
    // glows white. Light (sparks, glows, unlit or billboard) adds.
    em.blend = blend.unwrap_or(match em.render {
        ParticleRender::Mesh { .. } if em.lit => ParticleBlend::Alpha,
        _ => ParticleBlend::Additive,
    });
    em.validate()?;
    Ok(em)
}

/// A `.cube` colour table as og-render's 32³ [`ColorLut`] (other sizes are
/// resampled).
pub fn parse_cube(text: &str) -> Result<ColorLut, String> {
    ColorLut::from_cube(text)
}
