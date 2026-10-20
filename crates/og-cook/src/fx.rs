//! Effect assets as text: particle emitters (`.ogt` of `type particles`) and
//! colour lookup tables (`.cube`). The cook validates both and stores them in
//! blobs; the runtime turns them into og-render's [`ParticleEmitter`] and
//! [`ColorLut`]. One parser serves both sides, so what cooks loads.
//!
//! An emitter, every line optional (the defaults are og-render's: a small
//! fountain of soft white points); `[min, max]` pairs are uniform random
//! ranges per particle, a single number is both:
//!
//! ```text
//! type particles
//! version 1
//! ---
//! maxParticles 256                  # the ring: the newest overwrite the oldest
//! shape cone 15 0.05                # point | sphere <radius> [surface] | cone <angle°> <radius> | box <x> <y> <z> (half extents)
//! spread 0                          # degrees of random deviation (180: every direction)
//! rate 40                           # particles per second
//! burst 0 30                        # <time> <count> [<cycles> <interval>]; repeatable
//! duration 1                        # seconds per emission cycle
//! looping true
//! lifetime 1 1.5
//! speed 1.5 2.5
//! size 0.08 0.12
//! sizeOverLife 0 1 1 0              # <t> <value> pairs, t 0..1 over the life (at most 8 keys)
//! colorOverLife {                   # keys <t> <r> <g> <b> <a>, linear (at most 8)
//!   key 0 1 1 1 1
//!   key 1 1 1 1 0
//! }
//! color 1 1 1 1                     # a tint; above 1 blooms
//! rotation 0 360                    # start rotation, degrees
//! angularVelocity -90 90            # degrees per second
//! gravity 0 -2 0
//! drag 0
//! noise 0 0.5 0.3                   # <strength> [<frequency> <scroll>]
//! texture "textures/spark.png"      # an asset id (og.load.texture's); omitted: a soft dot
//! sheet 4 4 16 life 1               # <columns> <rows> [<frames> [off | life <cycles> | fps <n> | random]]
//! blend additive                    # additive | alpha
//! space world                       # world | local
//! align camera                      # camera | velocity <stretch>
//! softDistance 0
//! collision 0.4 0.2                 # <bounce> <friction> (depth-buffer collision, full tier)
//! render billboard                  # billboard | sprite <layer> (2D: drawn as sprites)
//! glow 0                            # sprite mode: strength in the 2D glow buffer
//! array "fx/smoke.ogt" random       # sprite mode: draw from a texture array (`type texture-array`)
//!                                   #   instead of `texture`, a layer per particle:
//!                                   #   [<layers>] [off | life <cycles> | fps <n> | random]
//! round false                       # sprite mode: the soft dot's round edge over the texture or layer
//! ```
//!
//! `array` names a texture array's asset id and picks each particle's layer
//! as `sheet` picks a frame: `life <cycles>` steps through the layers over
//! the particle's life (by age), `fps <n>` at a fixed rate, `random` one
//! layer per particle for its whole life, `off` (the default) the first;
//! `<layers>` uses the first that many (default every layer). Every
//! particle of the emitter is one draw whichever layers they show. A
//! billboard emitter with an array is refused by the renderer.

use og_core::ogt::{Document, Entry, Value};
use og_render::{
    Burst, ColorCurve, ColorLut, Curve, EmitterShape, MAX_CURVE_KEYS, ParticleAlign, ParticleBlend,
    ParticleCollision, ParticleEmitter, ParticleRender, SheetMode, SimulationSpace,
    TextureArrayHandle, TextureHandle,
};

/// The `.ogt` type of an emitter file.
pub const EMITTER_TYPE: &str = "particles";

fn nums(e: &Entry) -> Result<Vec<f32>, String> {
    e.values
        .iter()
        .map(|v| {
            v.as_number()
                .map(|n| n.as_f32())
                .filter(|f| f.is_finite())
                .ok_or_else(|| format!("line {}: `{}` takes numbers", e.line, e.key))
        })
        .collect()
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
                .filter(|e| e.key == "texture" || e.key == "array")
                .filter_map(|e| e.values.first().and_then(Value::as_str).map(str::to_string))
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
/// (an `array` line) to the scene's handle.
pub fn parse_emitter_with(
    text: &str,
    texture: &mut dyn FnMut(&str) -> Result<Option<TextureHandle>, String>,
    array: &mut dyn FnMut(&str) -> Result<Option<TextureArrayHandle>, String>,
) -> Result<ParticleEmitter, String> {
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
    for e in &doc.body.entries {
        let line = e.line;
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
                    other => {
                        return Err(format!(
                            "line {line}: shape `{other}` (point, sphere, cone, box)"
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
            "colorOverLife" => {
                let block = e.block.as_ref().ok_or_else(|| {
                    format!("line {line}: colorOverLife {{ key <t> <r> <g> <b> <a> ... }}")
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
                    return Err(format!(
                        "line {line}: colorOverLife has 1 to {MAX_CURVE_KEYS} keys"
                    ));
                }
                em.color_over_life = ColorCurve::new(&keys);
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
                em.blend = match word(e, 0)? {
                    "additive" => ParticleBlend::Additive,
                    "alpha" => ParticleBlend::Alpha,
                    other => return Err(format!("line {line}: blend `{other}` (additive, alpha)")),
                }
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
                    other => {
                        return Err(format!(
                            "line {line}: align `{other}` (camera, velocity <stretch>)"
                        ));
                    }
                }
            }
            "softDistance" => em.soft_distance = one(e)?.max(0.0),
            "collision" => {
                let v = nums(e)?;
                let d = ParticleCollision::default();
                em.collision = Some(ParticleCollision {
                    bounce: v.first().copied().unwrap_or(d.bounce),
                    friction: v.get(1).copied().unwrap_or(d.friction),
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
                    other => {
                        return Err(format!(
                            "line {line}: render `{other}` (billboard, sprite <layer>)"
                        ));
                    }
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
            other => return Err(format!("line {line}: unknown emitter setting `{other}`")),
        }
    }
    em.bursts = bursts;
    Ok(em)
}

/// A `.cube` colour table as og-render's 32³ [`ColorLut`] (other sizes are
/// resampled).
pub fn parse_cube(text: &str) -> Result<ColorLut, String> {
    ColorLut::from_cube(text)
}
