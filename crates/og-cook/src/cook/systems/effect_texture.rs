//! Offline volume transport and Beer-Lambert flipbooks; provenance in
//! third_party/stable-fluids. All appearance parameters belong to the file.
use super::*;
use og_core::noise::{Fbm, NoiseKind};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

mod volume;

pub(crate) static TEXTURE: KindDef = KindDef {
    name: "effect-texture",
    extensions: &[],
    types: &["effect-texture"],
    by_content: true,
    streams: false,
    plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&TEXTURE];

pub const LINES: &[crate::grammar::Line] = &[
    crate::grammar::Line::new(
        "mode",
        "smoke|fire|explosion|noise|worley|curl",
        "required",
        "Transported density/heat volume, or shared procgen kernels. No appearance preset.",
        "",
    ),
    crate::grammar::Line::new(
        "size",
        "<pixels>",
        "required",
        "Square frame resolution, 8..1024. Sheet extents at most 8192, total frame pixels at most 16777216.",
        "",
    ),
    crate::grammar::Line::new(
        "frames",
        "<count>",
        "required",
        "1..256 frames, in row-major sheet order and chronological array order.",
        "",
    ),
    crate::grammar::Line::new(
        "columns",
        "<count>",
        "required",
        "1..frames sheet columns; rows are ceil(frames/columns). Unused cells are transparent.",
        "",
    ),
    crate::grammar::Line::new(
        "grid",
        "<cells>",
        "required",
        "Volume cells per axis, 8..96, over the unit cube. No per-frame rescaling.",
        "",
    ),
    crate::grammar::Line::new(
        "steps",
        "<count>",
        "required",
        "Fixed simulation substeps between frames, 1..64; frame zero follows one interval.",
        "",
    ),
    crate::grammar::Line::new(
        "dt",
        "<seconds>",
        "required",
        "Seconds per substep, >0..0.1. Noise time is (frame+1)*steps*dt.",
        "",
    ),
    crate::grammar::Line::new(
        "pressure",
        "<iterations>",
        "required",
        "1..128 Jacobi pressure iterations per volume step.",
        "",
    ),
    crate::grammar::Line::new(
        "seed",
        "<u32>",
        "required",
        "Seed for the same fBm/curl/Worley kernels as og.procgen.",
        "",
    ),
    crate::grammar::Line::new(
        "frequency",
        "<cycles>",
        "required",
        "Positive noise cycles per unit cube; three simplex octaves, lacunarity 2 and gain 0.5.",
        "",
    ),
    crate::grammar::Line::new(
        "range",
        "<magnitude>",
        "required",
        "Positive data encoding range: signed noise/curl map -range..range to 0..1; Worley maps 0..range. Values beyond it clamp. Unused for volumes.",
        "",
    ),
    crate::grammar::Line::new(
        "turbulence",
        "<acceleration>",
        "required",
        "Nonnegative curl forcing, cube widths/s²; on noise textures it is z travel speed.",
        "",
    ),
    crate::grammar::Line::new(
        "source",
        "<x> <y> <z>",
        "required",
        "Injection centre, each 0..1 in cube coordinates, Y up.",
        "",
    ),
    crate::grammar::Line::new(
        "detail",
        "<strength>",
        "required",
        "Source boundary/density modulation by seeded fBm, 0..1; 0 is a smooth sphere, 1 exposes the noise's full signed variation.",
        "",
    ),
    crate::grammar::Line::new(
        "vorticity",
        "<strength>",
        "required",
        "Nonnegative vorticity-confinement rate in 1/s; multiplied by cell width to restore rotational motion lost to advection. 0 disables it.",
        "",
    ),
    crate::grammar::Line::new(
        "radius",
        "<fraction>",
        "required",
        "Source radius in cube widths, >0..0.5, smooth spherical injection.",
        "",
    ),
    crate::grammar::Line::new(
        "inject",
        "<seconds>",
        "required",
        "Nonnegative source duration; explosion injects only initially.",
        "",
    ),
    crate::grammar::Line::new(
        "density",
        "<rate>",
        "required",
        "Nonnegative density/second; explosion uses initial density instead.",
        "",
    ),
    crate::grammar::Line::new(
        "heat",
        "<rate>",
        "required",
        "Nonnegative heat/second; explosion uses initial heat instead.",
        "",
    ),
    crate::grammar::Line::new(
        "buoyancy",
        "<acceleration>",
        "required",
        "Signed Y acceleration/unit heat, in cube widths/s².",
        "",
    ),
    crate::grammar::Line::new(
        "expansion",
        "<speed>",
        "required",
        "Nonnegative radial source speed in cube widths/s.",
        "",
    ),
    crate::grammar::Line::new(
        "decay",
        "<rate>",
        "required",
        "Nonnegative exponential density loss/second.",
        "",
    ),
    crate::grammar::Line::new(
        "cooling",
        "<rate>",
        "required",
        "Nonnegative exponential heat loss/second.",
        "",
    ),
    crate::grammar::Line::new(
        "absorption",
        "<coefficient>",
        "required",
        "Nonnegative Beer-Lambert extinction/density/cube width.",
        "",
    ),
    crate::grammar::Line::new(
        "color",
        "<r> <g> <b>",
        "required",
        "Linear scattering RGB, each 0..1; ambient/direct/light supply baked volume self-shadowing.",
        "",
    ),
    crate::grammar::Line::new(
        "emission",
        "<r> <g> <b>",
        "required",
        "Linear emission RGB/unit heat, each 0..1, written separately to #emission. HDR gain belongs to the effect material.",
        "",
    ),
    crate::grammar::Line::new(
        "light",
        "<x> <y> <z>",
        "required",
        "Nonzero direction toward the baked scattering light in cube coordinates, normalized at cook; view rays travel +Z.",
        "",
    ),
    crate::grammar::Line::new(
        "ambient",
        "<strength>",
        "required",
        "Nonnegative isotropic ambient scattering illumination. Does not change alpha or emission.",
        "",
    ),
    crate::grammar::Line::new(
        "direct",
        "<strength>",
        "required",
        "Nonnegative directional scattering illumination, attenuated through density toward light (volume self-shadow). Output RGB clamps to sRGB8; use material intensity for HDR gain.",
        "",
    ),
];

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    mode: String,
    size: u32,
    frames: u32,
    columns: u32,
    grid: usize,
    steps: u32,
    dt: f32,
    pressure: u32,
    seed: u32,
    frequency: f32,
    range: f32,
    turbulence: f32,
    source: [f32; 3],
    detail: f32,
    vorticity: f32,
    radius: f32,
    inject: f32,
    density: f32,
    heat: f32,
    buoyancy: f32,
    expansion: f32,
    decay: f32,
    cooling: f32,
    absorption: f32,
    color: [f32; 3],
    emission: [f32; 3],
    light: [f32; 3],
    ambient: f32,
    direct: f32,
}

fn parse(bytes: &[u8]) -> Result<Config, String> {
    let doc = Document::parse(&String::from_utf8_lossy(bytes)).map_err(|e| e.to_string())?;
    if doc.ty != "effect-texture" || doc.version != 1 {
        return Err("effect-texture requires version 1".into());
    }
    let mut object = serde_json::Map::new();
    for e in &doc.body.entries {
        if !LINES.iter().any(|l| l.key == e.key) || e.block.is_some() {
            return Err(format!("unknown effect-texture field '{}'", e.key));
        }
        let value = if e.key == "mode" {
            if e.values.len() != 1 {
                return Err("mode takes one word".into());
            }
            serde_json::json!(e.word(0).map_err(|e| e.to_string())?)
        } else {
            let values = e
                .values
                .iter()
                .map(|v| {
                    let n = v
                        .as_number()
                        .ok_or("effect-texture fields take finite numbers")?
                        .as_f64();
                    if !n.is_finite() {
                        return Err("effect-texture fields take finite numbers");
                    }
                    Ok(if n.fract() == 0.0 && n >= 0.0 {
                        serde_json::json!(n as u64)
                    } else {
                        serde_json::json!(n)
                    })
                })
                .collect::<Result<Vec<_>, &str>>()?;
            if matches!(e.key.as_str(), "source" | "color" | "emission" | "light") {
                serde_json::json!(values)
            } else if values.len() == 1 {
                values[0].clone()
            } else {
                return Err(format!("{} takes one number", e.key));
            }
        };
        if object.insert(e.key.clone(), value).is_some() {
            return Err(format!("duplicate field '{}'", e.key));
        }
    }
    let c: Config = serde_json::from_value(object.into()).map_err(|e| e.to_string())?;
    if !matches!(
        c.mode.as_str(),
        "smoke" | "fire" | "explosion" | "noise" | "worley" | "curl"
    ) || !(8..=1024).contains(&c.size)
        || !(1..=256).contains(&c.frames)
        || !(1..=c.frames).contains(&c.columns)
        || !(8..=96).contains(&c.grid)
        || !(1..=64).contains(&c.steps)
        || !(1..=128).contains(&c.pressure)
        || !(0.0 < c.dt && c.dt <= 0.1)
        || c.frequency <= 0.0
        || c.range <= 0.0
        || [
            c.dt,
            c.frequency,
            c.range,
            c.turbulence,
            c.detail,
            c.vorticity,
            c.ambient,
            c.direct,
            c.radius,
            c.inject,
            c.density,
            c.heat,
            c.buoyancy,
            c.expansion,
            c.decay,
            c.cooling,
            c.absorption,
        ]
        .iter()
        .any(|v| !v.is_finite())
        || !(0.0..=1.0).contains(&c.detail)
        || c.light.iter().any(|v| !v.is_finite())
        || !c.light.iter().map(|v| v * v).sum::<f32>().is_normal()
        || !(0.0 < c.radius && c.radius <= 0.5)
        || c.source
            .iter()
            .chain(c.color.iter())
            .chain(c.emission.iter())
            .any(|v| !(0.0..=1.0).contains(v))
        || [
            c.turbulence,
            c.vorticity,
            c.ambient,
            c.direct,
            c.inject,
            c.density,
            c.heat,
            c.expansion,
            c.decay,
            c.cooling,
            c.absorption,
        ]
        .iter()
        .any(|v| *v < 0.0)
        || c.size * c.columns > 8192
        || c.size * c.frames.div_ceil(c.columns) > 8192
        || u64::from(c.size).pow(2) * u64::from(c.frames) > 16_777_216
    {
        return Err("effect-texture parameters outside documented ranges".into());
    }
    Ok(c)
}

struct Frames {
    config: Config,
    data: OnceLock<Vec<u8>>,
}
impl Frames {
    fn data(&self) -> &Vec<u8> {
        self.data.get_or_init(|| {
            let config = serde_json::to_vec(&self.config).expect("finite configuration");
            let key = cache::key(
                &[Area::Texture, Area::Effect],
                "effect-texture-1",
                &[
                    &config,
                    include_bytes!("effect_texture.rs"),
                    include_bytes!("effect_texture/volume.rs"),
                    include_bytes!("../../../../og-core/src/noise.rs"),
                ],
            );
            let store = cache::default_path().map(cache::Store::new);
            if let Some((data, _)) = store.as_ref().and_then(|s| s.action(&key)) {
                return data;
            }
            let data = volume::render(&self.config);
            if let Some(store) = store {
                let _ = store.put_action(&key, &data, &[]);
            }
            data
        })
    }
    fn image(&self, frame: Option<u32>, emission: bool) -> texture::Image {
        let c = &self.config;
        let (width, height) = if frame.is_some() {
            (c.size, c.size)
        } else {
            (c.size * c.columns, c.size * c.frames.div_ceil(c.columns))
        };
        let mut rgba = vec![0; width as usize * height as usize * 4];
        let data = self.data();
        for f in 0..c.frames {
            if frame.is_some_and(|v| v != f) {
                continue;
            }
            for y in 0..c.size {
                for x in 0..c.size {
                    let src = ((f * c.size * c.size + y * c.size + x) * 8) as usize
                        + usize::from(emission) * 4;
                    let (dx, dy) = if frame.is_some() {
                        (x, y)
                    } else {
                        (f % c.columns * c.size + x, f / c.columns * c.size + y)
                    };
                    let dst = ((dy * width + dx) * 4) as usize;
                    rgba[dst..dst + 4].copy_from_slice(&data[src..src + 4]);
                }
            }
        }
        texture::Image {
            width,
            height,
            rgba,
        }
    }
}

fn plan<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let c = match parse(&s.bytes) {
        Ok(c) => c,
        Err(e) => {
            state
                .early_errors
                .push((s.rel.clone(), "effect-texture", e));
            return;
        }
    };
    let frames = Arc::new(Frames {
        config: c.clone(),
        data: OnceLock::new(),
    });
    for emission in [false, true] {
        let id = if emission {
            format!("{}#emission", s.rel)
        } else {
            s.rel.clone()
        };
        for (name, frame, array) in std::iter::once((id.clone(), None, false))
            .chain(std::iter::once((format!("{id}#array"), None, true)))
            .chain((0..c.frames).map(|f| {
                (
                    texture_array::layer_id(&format!("{id}#array"), f as usize),
                    Some(f),
                    false,
                )
            }))
        {
            let st = stamp(
                &[Area::Texture, Area::Effect],
                &[
                    &hash,
                    name.as_bytes(),
                    cx.tex_desc.as_bytes(),
                    include_bytes!("effect_texture.rs"),
                    include_bytes!("effect_texture/volume.rs"),
                    include_bytes!("../../../../og-core/src/noise.rs"),
                ],
            );
            outs.push((
                name,
                if array {
                    format::TextureArray::NAME
                } else {
                    format::Texture::NAME
                },
                st,
                Box::new(TextureJob {
                    source: s,
                    frames: frames.clone(),
                    frame,
                    emission,
                    array,
                    id: format!("{id}#array"),
                }),
            ));
        }
    }
}
struct TextureJob<'a> {
    source: &'a Source,
    frames: Arc<Frames>,
    frame: Option<u32>,
    emission: bool,
    array: bool,
    id: String,
}
impl Job for TextureJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let c = &self.frames.config;
        let kind = if matches!(c.mode.as_str(), "noise" | "worley" | "curl") {
            format::kind::LINEAR
        } else {
            format::kind::COLOR
        };
        let bytes = if self.array {
            texture_array::array_blob(
                &self.id,
                (c.size, c.size),
                c.frames as usize,
                kind,
                &ex.options.texture,
                p.stamp,
            )
        } else {
            texture::cook(
                &self.frames.image(self.frame, self.emission),
                kind,
                &ex.options.texture,
                p.stamp,
            )?
        };
        Ok((
            bytes,
            vec![format!(
                "{}: {} frames, {} px, {} columns",
                c.mode, c.frames, c.size, c.columns
            )],
        ))
    }
    fn action(&self) -> Action<'_> {
        Action::Input(&self.source.bytes, true)
    }
}
