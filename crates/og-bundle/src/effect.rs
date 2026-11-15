//! Effect materials as text (og-render's [`EffectMaterial`]): a file of
//! `type effect`, or an `effect { ... }` block inside an emitter. The
//! cook checks both with this parser and the runtime loads them with it.
//!
//! Its lines are the rows of [`EFFECT_LINES`] (a texture's block
//! [`UV_LINES`]), which the parser holds every line to and `og api gen`
//! writes into the `effect-files` guide.
//!
//! A value is a number, `random <min> <max>` (fixed per particle, strip or
//! mesh) or `curve <t> <v> ...` (over the life: a particle's age over its
//! lifetime, a strip's length from its start, an effect mesh's life; at
//! most 8 keys).

use crate::grammar::{Line, find, keys};
use og_core::ogt::{Block, Document, Entry, Value};
use og_render_data::{
    ColorCurve, Curve, EFFECT_PARAMS, EffectChannel, EffectMaterial, EffectTexture, EffectUv,
    EffectValue, Erosion, Flow, Fresnel, MAX_CURVE_KEYS, ShaderHandle, ShaderParam,
    TextureHandle,
};

/// The `.ogt` type of an effect material file.
pub const EFFECT_TYPE: &str = "effect";

/// Every line an effect material takes: the parser refuses any other
/// (`crate::grammar`).
pub const EFFECT_LINES: &[Line] = &[
    Line::new(
        "main",
        "[\"<id>\"] [rgba | red | green | blue | alpha] [{ uv }]",
        "",
        "The main texture and the channel read; left out, the surface's own (an emitter's texture).",
        "main \"fx/noise.png\" red {\n  tile 3 1\n  scroll 0.15 -0.6\n}",
    ),
    Line::new(
        "ramp",
        "{ key <t> <r> <g> <b> <a> ... }",
        "",
        "A gradient map: the texture's value picks the linear colour; at most 8 keys.",
        "ramp {\n  key 0 0 0 0 0\n  key 1 6 3 1 1\n}",
    ),
    Line::new(
        "mask",
        "[\"<id>\"] [<channel>] [{ uv }]",
        "",
        "A texture (red by default) multiplied into the alpha.",
        "mask \"fx/bar.png\" red {\n  polar true\n}",
    ),
    Line::new(
        "noise",
        "[\"<id>\"] [<channel>] [{ uv }]",
        "",
        "The erosion's noise (red by default); left out, value noise. Needs an `erosion` line.",
        "noise \"fx/cells.png\" red {\n  tile 2 2\n}\nerosion curve 0 0 1 1",
    ),
    Line::new(
        "erosion",
        "<value>",
        "",
        "The cut: where the noise is below it the surface is gone (a dissolve over the life).",
        "erosion curve 0 0 1 1",
    ),
    Line::new(
        "erosionSoftness",
        "<width>",
        "0.05",
        "How soft the cut's edge is.",
        "erosionSoftness 0.1",
    ),
    Line::new(
        "erosionEdge",
        "<width> <r> <g> <b> [<a>]",
        "",
        "A glowing border of this width and linear colour along the cut.",
        "erosionEdge 0.05 8 4 1",
    ),
    Line::new(
        "flowMap",
        "[\"<id>\"] [<channel>] [{ uv }]",
        "",
        "Red and green push the main uv (0.5 still).",
        "flowMap \"fx/swirl.png\"",
    ),
    Line::new(
        "flow",
        "<strength> [<cycles per second>]",
        "0.05 0.5",
        "How far and how fast the flow map pushes.",
        "flow 0.08 0.6",
    ),
    Line::new(
        "fresnel",
        "<power> [<strength>] [invert]",
        "",
        "A rim brighter at grazing angles (strength 1), or with `invert` soft silhouettes.",
        "fresnel 2.5 1 invert",
    ),
    Line::new(
        "softDepth",
        "<metres>",
        "0",
        "Fade against the scene behind (particles, meshes).",
        "softDepth 0.3",
    ),
    Line::new(
        "color",
        "<r> <g> <b> [<a>]",
        "1 1 1 1",
        "A linear tint; above 1 glows.",
        "color 4 2 1",
    ),
    Line::new(
        "intensity",
        "<value>",
        "1",
        "Multiplies the colour.",
        "intensity curve 0 2 1 0",
    ),
    Line::new(
        "alpha",
        "<value>",
        "1",
        "Multiplies the alpha.",
        "alpha random 0.6 1",
    ),
    Line::new(
        "offsetU",
        "<value>",
        "0",
        "Added to the main u.",
        "offsetU curve 0 0 1 1",
    ),
    Line::new(
        "offsetV",
        "<value>",
        "0",
        "Added to the main v.",
        "offsetV 0.5",
    ),
    Line::new(
        "custom",
        "<slot 1 to 8> <value>",
        "",
        "A number a material shader reads: slots 1 to 4 are effect_custom1().x to .w, 5 to 8 effect_custom2().",
        "custom 1 random 2 6",
    ),
    Line::new(
        "blend",
        crate::grammar::BLENDS,
        "",
        "Over the surface's own blend. Subtractive takes its colour away from what is behind (dark smoke, a void), multiply tints what is behind by it (scorch, shadow); a trail, ribbon or beam blends either as alpha.",
        "blend additive",
    ),
    Line::new(
        "doubleSided",
        "true | false",
        "false",
        "Meshes: draw both faces.",
        "doubleSided true",
    ),
    Line::new(
        "shader",
        "\"<id>\"",
        "",
        "A material shader (`type shader`) whose surface_color hook runs last; it reads effect_life(), effect_random(), effect_custom1() and effect_custom2().",
        "shader \"fx/bands.ogt\"",
    ),
    Line::new(
        "param",
        "<name> <1 to 4 numbers>",
        "",
        "One of the shader's parameters by name.",
        "shader \"fx/bands.ogt\"\nparam tint 4 0.6 2.5 1",
    ),
];

/// Every line a texture's `{ uv }` block takes.
pub const UV_LINES: &[Line] = &[
    Line::new(
        "tile",
        "<u> [<v>]",
        "1 1",
        "Repeats across the surface.",
        "main {\n  tile 3 1\n}",
    ),
    Line::new(
        "offset",
        "<u> [<v>]",
        "0 0",
        "Added to the uv.",
        "main {\n  offset 0.5\n}",
    ),
    Line::new(
        "scroll",
        "<u> [<v>]",
        "0 0",
        "Uv per second.",
        "main {\n  scroll 0.15 -0.6\n}",
    ),
    Line::new(
        "rotate",
        "<degrees per second>",
        "0",
        "Turns about the centre.",
        "main {\n  rotate 45\n}",
    ),
    Line::new(
        "polar",
        "true | false",
        "false",
        "u the angle about the centre, v the radius: rings and portals.",
        "main {\n  polar true\n}",
    ),
];

/// A linked material shader and the parameters it declares.
pub type ShaderLink = (ShaderHandle, Vec<ShaderParam>);

/// How an effect's references become the scene's handles. Each answers
/// `None` when the caller only checks the text (the cook).
pub struct EffectLinks<'a> {
    pub texture: &'a mut dyn FnMut(&str) -> Result<Option<TextureHandle>, String>,
    /// A material shader and its parameters (to set `param` lines by name).
    pub shader: &'a mut dyn FnMut(&str) -> Result<Option<ShaderLink>, String>,
}

/// The texture and shader asset ids an effect's block names.
pub fn effect_block_references(block: &Block) -> Vec<String> {
    block
        .entries
        .iter()
        .filter(|e| {
            matches!(
                e.key.as_str(),
                "main" | "mask" | "noise" | "flowMap" | "shader"
            )
        })
        .filter_map(|e| e.values.first().and_then(Value::as_str).map(str::to_string))
        .collect()
}

/// The asset ids an effect file names (the cook checks they exist).
pub fn effect_references(text: &str) -> Vec<String> {
    Document::parse(text)
        .map(|d| effect_block_references(&d.body))
        .unwrap_or_default()
}

/// Parses an effect file.
pub fn parse_effect(text: &str, links: &mut EffectLinks<'_>) -> Result<EffectMaterial, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != EFFECT_TYPE {
        return Err(format!(
            "an effect material is `type {EFFECT_TYPE}`, not `type {}`",
            doc.ty
        ));
    }
    if doc.version != 1 {
        return Err(format!(
            "effect version {} (this engine reads 1)",
            doc.version
        ));
    }
    parse_effect_block(&doc.body, links)
}

pub(crate) fn nums(e: &Entry, from: usize) -> Result<Vec<f32>, String> {
    e.numbers(from).map_err(|x| x.to_string())
}

pub(crate) fn one(e: &Entry) -> Result<f32, String> {
    match nums(e, 0)?[..] {
        [v] => Ok(v),
        _ => Err(format!("line {}: `{}` takes one number", e.line, e.key)),
    }
}

pub(crate) fn flag(e: &Entry) -> Result<bool, String> {
    e.values
        .first()
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("line {}: {} true | false", e.line, e.key))
}

/// A value from `values` (a number, `random a b`, `curve t v ...`).
pub(crate) fn value(e: &Entry, from: usize) -> Result<EffectValue, String> {
    let usage = || {
        format!(
            "line {}: {} <number> | random <min> <max> | curve <t> <v> ...",
            e.line, e.key
        )
    };
    let rest = &e.values[from.min(e.values.len())..];
    match rest.first() {
        Some(v) if v.as_number().is_some() && rest.len() == 1 => {
            Ok(EffectValue::Constant(nums(e, from)?[0]))
        }
        Some(v) if v.as_word() == Some("random") => match nums(e, from + 1)?[..] {
            [a, b] => Ok(EffectValue::Random([a, b])),
            _ => Err(usage()),
        },
        Some(v) if v.as_word() == Some("curve") => {
            let n = nums(e, from + 1)?;
            if n.is_empty() || n.len() % 2 != 0 || n.len() / 2 > MAX_CURVE_KEYS {
                return Err(format!(
                    "line {}: {} curve is 1 to {MAX_CURVE_KEYS} <t> <value> pairs",
                    e.line, e.key
                ));
            }
            let keys: Vec<(f32, f32)> = n.chunks(2).map(|c| (c[0], c[1])).collect();
            Ok(EffectValue::Curve(Curve::new(&keys)))
        }
        _ => Err(usage()),
    }
}

fn channel(e: &Entry, at: usize) -> Result<EffectChannel, String> {
    Ok(match e.values.get(at).and_then(Value::as_word) {
        None | Some("rgba") => EffectChannel::Rgba,
        Some("red") => EffectChannel::Red,
        Some("green") => EffectChannel::Green,
        Some("blue") => EffectChannel::Blue,
        Some("alpha") => EffectChannel::Alpha,
        Some(other) => {
            return Err(format!(
                "line {}: {} channel `{other}` (rgba, red, green, blue, alpha)",
                e.line, e.key
            ));
        }
    })
}

fn pair(e: &Entry) -> Result<glam::Vec2, String> {
    match nums(e, 0)?[..] {
        [v] => Ok(glam::Vec2::splat(v)),
        [a, b] => Ok(glam::Vec2::new(a, b)),
        _ => Err(format!("line {}: {} <u> [<v>]", e.line, e.key)),
    }
}

/// A texture slot: `<slot> ["<id>"] [<channel>] [{ uv }]`.
fn slot(
    e: &Entry,
    texture: &mut dyn FnMut(&str) -> Result<Option<TextureHandle>, String>,
    default_channel: EffectChannel,
) -> Result<EffectTexture, String> {
    let line = e.line;
    let (tex, at) = match e.values.first().and_then(Value::as_str) {
        Some(id) => (
            texture(id).map_err(|r| format!("line {line}: {} {id}: {r}", e.key))?,
            1,
        ),
        None => (None, 0),
    };
    let channel = if e.values.len() > at {
        channel(e, at)?
    } else {
        default_channel
    };
    let mut uv = EffectUv::default();
    if let Some(b) = &e.block {
        for k in &b.entries {
            if find(UV_LINES, &k.key, &[]).is_none() {
                return Err(format!(
                    "line {}: unknown uv setting `{}` ({})",
                    k.line,
                    k.key,
                    keys(UV_LINES, &[])
                ));
            }
            match k.key.as_str() {
                "tile" => uv.tile = pair(k)?,
                "offset" => uv.offset = pair(k)?,
                "scroll" => uv.scroll = pair(k)?,
                "rotate" => uv.rotation = one(k)?,
                "polar" => uv.polar = flag(k)?,
                other => return Err(format!("line {}: `{other}` is not read yet", k.line)),
            }
        }
    }
    Ok(EffectTexture {
        texture: tex,
        uv,
        channel,
    })
}

/// Parses the settings of an effect (a file's body or an emitter's
/// `effect { ... }` block).
pub fn parse_effect_block(
    block: &Block,
    links: &mut EffectLinks<'_>,
) -> Result<EffectMaterial, String> {
    let EffectLinks { texture, shader } = links;
    let mut m = EffectMaterial::default();
    let mut erosion: Option<EffectValue> = None;
    let mut noise: Option<EffectTexture> = None;
    let mut softness = Erosion::default().softness;
    let mut edge = (0.0, [1.0; 4]);
    let mut flow_map: Option<EffectTexture> = None;
    let mut flow = (0.05, 0.5);
    let mut params: Vec<ShaderParam> = Vec::new();
    let mut named: Vec<(usize, String, Vec<f32>)> = Vec::new();
    for e in &block.entries {
        let line = e.line;
        if find(EFFECT_LINES, &e.key, &[]).is_none() {
            return Err(format!(
                "line {line}: unknown effect setting `{}` (an effect takes {})",
                e.key,
                keys(EFFECT_LINES, &[])
            ));
        }
        match e.key.as_str() {
            "main" => m.main = slot(e, *texture, EffectChannel::Rgba)?,
            "mask" => m.mask = Some(slot(e, *texture, EffectChannel::Red)?),
            "noise" => noise = Some(slot(e, *texture, EffectChannel::Red)?),
            "flowMap" => flow_map = Some(slot(e, *texture, EffectChannel::Rgba)?),
            "ramp" => {
                let b = e.block.as_ref().ok_or_else(|| {
                    format!("line {line}: ramp {{ key <t> <r> <g> <b> <a> ... }}")
                })?;
                let mut keys = Vec::new();
                for k in &b.entries {
                    let [t, r, g, bb, a] = nums(k, 0)?[..] else {
                        return Err(format!("line {}: key <t> <r> <g> <b> <a>", k.line));
                    };
                    keys.push((t, [r, g, bb, a]));
                }
                if keys.is_empty() || keys.len() > MAX_CURVE_KEYS {
                    return Err(format!("line {line}: ramp has 1 to {MAX_CURVE_KEYS} keys"));
                }
                m.ramp = Some(ColorCurve::new(&keys));
            }
            "erosion" => erosion = Some(value(e, 0)?),
            "erosionSoftness" => softness = one(e)?.max(0.0),
            "erosionEdge" => {
                edge = match nums(e, 0)?[..] {
                    [w, r, g, b] => (w, [r, g, b, 1.0]),
                    [w, r, g, b, a] => (w, [r, g, b, a]),
                    _ => {
                        return Err(format!(
                            "line {line}: erosionEdge <width> <r> <g> <b> [<a>]"
                        ));
                    }
                }
            }
            "flow" => {
                flow = match nums(e, 0)?[..] {
                    [s] => (s, flow.1),
                    [s, c] => (s, c),
                    _ => return Err(format!("line {line}: flow <strength> [<speed>]")),
                }
            }
            "fresnel" => {
                let n: Vec<f32> = e
                    .values
                    .iter()
                    .filter_map(|v| v.as_number().map(|n| n.as_f32()))
                    .collect();
                let invert = e.values.iter().any(|v| v.as_word() == Some("invert"));
                let power = *n
                    .first()
                    .ok_or_else(|| format!("line {line}: fresnel <power> [<strength>] [invert]"))?;
                m.fresnel = Some(Fresnel {
                    power,
                    strength: n.get(1).copied().unwrap_or(1.0),
                    invert,
                });
            }
            "softDepth" => m.soft_depth = one(e)?.max(0.0),
            "color" => {
                m.color = match nums(e, 0)?[..] {
                    [r, g, b] => [r, g, b, 1.0],
                    [r, g, b, a] => [r, g, b, a],
                    _ => return Err(format!("line {line}: color <r> <g> <b> [<a>]")),
                }
            }
            "intensity" => m.intensity = value(e, 0)?,
            "alpha" => m.alpha = value(e, 0)?,
            "offsetU" => m.offset[0] = value(e, 0)?,
            "offsetV" => m.offset[1] = value(e, 0)?,
            "custom" => {
                let k = e
                    .values
                    .first()
                    .and_then(Value::as_number)
                    .map(|n| n.as_f32())
                    .filter(|k| (1.0..=8.0).contains(k) && k.fract() == 0.0)
                    .ok_or_else(|| format!("line {line}: custom <slot 1 to 8> <value>"))?
                    as usize
                    - 1;
                m.custom[k / 4][k % 4] = value(e, 1)?;
            }
            "blend" => {
                m.blend = Some(
                    crate::grammar::blend_word(e.values.first().and_then(Value::as_word))
                        .ok_or_else(|| format!("line {line}: blend {}", crate::grammar::BLENDS))?,
                )
            }
            "doubleSided" => m.double_sided = flag(e)?,
            "shader" => {
                let id = e
                    .values
                    .first()
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("line {line}: shader \"<asset id>\""))?;
                if let Some((h, p)) =
                    shader(id).map_err(|r| format!("line {line}: shader {id}: {r}"))?
                {
                    m.shader = Some(h);
                    params = p;
                }
            }
            "param" => {
                let name = e
                    .values
                    .first()
                    .and_then(Value::as_word)
                    .ok_or_else(|| format!("line {line}: param <name> <numbers>"))?;
                let v = nums(e, 1)?;
                if v.is_empty() || v.len() > 4 {
                    return Err(format!("line {line}: param {name} takes 1 to 4 numbers"));
                }
                named.push((line, name.to_string(), v));
            }
            // A row of EFFECT_LINES with no arm; the table's tests parse every row.
            other => return Err(format!("line {line}: `{other}` is not read yet")),
        }
    }
    if let Some(amount) = erosion {
        m.erosion = Some(Erosion {
            noise: noise.unwrap_or(Erosion::default().noise),
            amount,
            softness,
            edge: edge.0,
            edge_color: edge.1,
        });
    } else if noise.is_some() {
        return Err("a noise texture without an erosion line".into());
    }
    if let Some(map) = flow_map {
        m.flow = Some(Flow {
            map,
            strength: flow.0,
            speed: flow.1,
        });
    }
    // Parameters by name: the shader's defaults, then the lines. A cook
    // that resolves no shader only checks their form.
    if m.shader.is_some() {
        for (slot, p) in m.params.iter_mut().zip(&params).take(EFFECT_PARAMS) {
            *slot = p.default;
        }
        for (line, name, v) in named {
            let i = params
                .iter()
                .position(|p| p.name == name)
                .ok_or_else(|| format!("line {line}: the shader declares no parameter `{name}`"))?;
            let mut x = [0.0; 4];
            x[..v.len()].copy_from_slice(&v);
            m.params[i] = x;
        }
    }
    m.validate()?;
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(text: &str) -> Result<EffectMaterial, String> {
        parse_effect(
            text,
            &mut EffectLinks {
                texture: &mut |_| Ok(Some(TextureHandle::from_bits(3))),
                shader: &mut |_| Ok(None),
            },
        )
    }

    #[test]
    fn a_file_sets_every_part() {
        let m = check(
            "type effect\nversion 1\n---\nmain \"n.png\" red {\n  tile 3 1\n  scroll 0.1 -0.5\n  polar true\n}\nramp {\n  key 0 0 0 0 0\n  key 1 6 3 1 1\n}\nerosion curve 0 0 1 1\nerosionEdge 0.05 8 4 1\nfresnel 2 0.5 invert\ncustom 6 random 2 6\nintensity curve 0 2 1 0\nblend alpha\n",
        )
        .unwrap();
        assert_eq!(m.main.channel, EffectChannel::Red);
        assert!(m.main.uv.polar);
        assert_eq!(m.main.uv.tile, glam::Vec2::new(3.0, 1.0));
        let e = m.erosion.as_ref().unwrap();
        assert_eq!(e.amount, EffectValue::Curve(Curve::linear(0.0, 1.0)));
        // No noise texture: the erosion reads value noise.
        assert_eq!(e.noise.texture, None);
        assert_eq!(e.edge_color, [8.0, 4.0, 1.0, 1.0]);
        let f = m.fresnel.unwrap();
        assert!(f.invert && f.power == 2.0 && f.strength == 0.5);
        // Slot 6 is custom 2's y.
        assert_eq!(m.custom[1][1], EffectValue::Random([2.0, 6.0]));
        assert_eq!(m.blend, Some(og_render_data::ParticleBlend::Alpha));
    }

    #[test]
    fn mistakes_are_named() {
        let err =
            |body: &str| check(&format!("type effect\nversion 1\n---\n{body}\n")).unwrap_err();
        assert!(err("erosion curve 0").contains("pairs"));
        assert!(err("custom 9 1").contains("slot 1 to 8"));
        assert!(err("main \"a.png\" purple").contains("channel"));
        assert!(err("glow 1").contains("unknown effect setting `glow`"));
        assert!(err("noise \"a.png\"").contains("without an erosion"));
    }
}
