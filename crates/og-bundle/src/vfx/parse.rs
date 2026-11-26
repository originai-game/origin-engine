//! The `type vfx` parser: a file's text to a [`Vfx`], every line held to
//! the tables in `lines`.

use super::lines::{ELEMENT_LINES, VFX_LINES};
use super::*;
use crate::effect::{flag, nums, one, value};
use crate::grammar::{find, keys};
use og_core::ogt::{Document, Entry, Value};

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
            | (
                key @ "stretch",
                VfxKind::Mesh { .. } | VfxKind::Glow { .. } | VfxKind::Beam { .. },
            ) => {
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
