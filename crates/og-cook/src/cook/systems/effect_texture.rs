//! Offline volume transport and Beer-Lambert flipbooks; provenance in
//! third_party/stable-fluids. All appearance parameters belong to the file.
use super::*;
use og_core::noise::{Fbm, NoiseKind};
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

mod volume;

pub(crate) static TEXTURE: KindDef = KindDef {
    name: "effect-texture",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: false,
    plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&TEXTURE];

/// Offline texture transport input; all appearance values are explicit.
/// Atlas dimensions, total work and nonzero light direction are domain checks.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars", extend("x-og-output-kind" = "texture"))]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Density/heat transport, or the shared noise/Worley/curl kernels; no preset.
    #[schemars(extend("enum" = ["smoke", "fire", "explosion", "noise", "worley", "curl"]))]
    pub mode: String,
    /// Square frame resolution, 8..1024. Sheet extents <=8192; <=16777216 frame pixels.
    #[schemars(range(min = 8, max = 1024), extend("x-og-unit" = "px"))]
    pub size: u32,
    /// 1..256 chronological frames, in row-major sheet order.
    #[schemars(range(min = 1, max = 256))]
    pub frames: u32,
    /// Sheet columns. Domain validation also checks columns <= frames.
    #[schemars(range(min = 1, max = 256))]
    pub columns: u32,
    /// 8..96 volume cells per axis of the unit cube; no per-frame rescaling.
    #[schemars(range(min = 8, max = 96))]
    pub grid: usize,
    /// 1..64 fixed simulation substeps between frames; frame zero follows one interval.
    #[schemars(range(min = 1, max = 64))]
    pub steps: u32,
    /// Seconds per substep, >0..0.1; noise time is (frame+1)*steps*dt.
    #[schemars(range(max = 0.1), extend("exclusiveMinimum" = 0, "x-og-unit" = "s"))]
    pub dt: f32,
    /// 1..128 Jacobi pressure iterations per volume step.
    #[schemars(range(min = 1, max = 128))]
    pub pressure: u32,
    /// Seed for the same fBm/curl/Worley kernels as og.procgen.
    pub seed: u32,
    /// Positive cycles per unit cube; three simplex octaves, lacunarity 2, gain 0.5.
    #[schemars(extend("exclusiveMinimum" = 0))]
    pub frequency: f32,
    /// Positive encoding range: signed noise/curl -range..range maps to 0..1;
    /// Worley 0..range maps to 0..1. Values clamp. Unused for volume transport.
    #[schemars(extend("exclusiveMinimum" = 0))]
    pub range: f32,
    /// Nonnegative curl acceleration; on noise textures this is Z travel speed.
    #[schemars(range(min = 0), extend("x-og-unit" = "cube widths/s²"))]
    pub turbulence: f32,
    /// Injection centre in the unit cube, Y up.
    #[schemars(extend("items" = {"type": "number", "minimum": 0, "maximum": 1}))]
    pub source: [f32; 3],
    /// Source boundary/density fBm modulation, 0..1; zero is a smooth sphere.
    #[schemars(range(min = 0, max = 1))]
    pub detail: f32,
    /// Nonnegative vorticity confinement in 1/s, scaled by cell width; zero disables it.
    #[schemars(range(min = 0), extend("x-og-unit" = "1/s"))]
    pub vorticity: f32,
    /// Injection radius in cube widths, >0..0.5.
    #[schemars(range(max = 0.5), extend("exclusiveMinimum" = 0, "x-og-unit" = "cube widths"))]
    pub radius: f32,
    /// Nonnegative source duration in seconds; explosion injects only initially.
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub inject: f32,
    /// Density/second; explosion uses initial density. Nonnegative.
    #[schemars(range(min = 0))]
    pub density: f32,
    /// Heat/second; explosion uses initial heat. Nonnegative.
    #[schemars(range(min = 0))]
    pub heat: f32,
    /// Signed Y acceleration per unit heat.
    #[schemars(extend("x-og-unit" = "cube widths/s²"))]
    pub buoyancy: f32,
    /// Nonnegative radial source speed in cube widths/s.
    #[schemars(range(min = 0))]
    pub expansion: f32,
    /// Nonnegative exponential density loss/second.
    #[schemars(range(min = 0))]
    pub decay: f32,
    /// Nonnegative exponential heat loss/second.
    #[schemars(range(min = 0))]
    pub cooling: f32,
    /// Nonnegative Beer-Lambert extinction/density/cube width.
    #[schemars(range(min = 0))]
    pub absorption: f32,
    /// Linear scattering RGB, each 0..1, illuminated by ambient/direct/light.
    #[schemars(extend("items" = {"type": "number", "minimum": 0, "maximum": 1}))]
    pub color: [f32; 3],
    /// Linear emission RGB/unit heat, each 0..1, written separately to #emission.
    #[schemars(extend("items" = {"type": "number", "minimum": 0, "maximum": 1}))]
    pub emission: [f32; 3],
    /// Nonzero direction toward the light, normalized at cook. View rays travel +Z.
    pub light: [f32; 3],
    /// Nonnegative isotropic scattering light; does not change alpha or emission.
    #[schemars(range(min = 0))]
    pub ambient: f32,
    /// Nonnegative directional light with volume self-shadow; RGB clamps to sRGB8.
    #[schemars(range(min = 0))]
    pub direct: f32,
}

impl Config {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<Config>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("effect-texture input schema"))
    }
}

/// Domain relationships beyond the reflected structural constraints. Keep the
/// same checks on every native entry, including a cached author IR.
fn validate(c: &Config) -> Result<(), String> {
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
    Ok(())
}

/// Actual outputs of this producer, shared by planning and the content graph.
/// A reference never invents array/layer suffixes or their runtime kinds.
fn outputs(c: &Config) -> impl Iterator<Item = (String, Option<u32>, bool, bool)> {
    [false, true].into_iter().flat_map(move |emission| {
        let suffix = if emission { "emission" } else { "" };
        let array = if emission { "emission#array" } else { "array" };
        std::iter::once((suffix.to_string(), None, false, emission))
            .chain(std::iter::once((array.to_string(), None, true, emission)))
            .chain((0..c.frames).map(move |f| {
                (
                    texture_array::layer_id(array, f as usize),
                    Some(f),
                    false,
                    emission,
                )
            }))
    })
}

pub(crate) fn derived(data: &generator::value::Data) -> Result<BTreeMap<String, String>, String> {
    let file: source_schemas::File =
        serde_json::from_value(data.root().into_json()).map_err(|e| e.to_string())?;
    let config: Config = serde_json::from_value(file.value).map_err(|e| e.to_string())?;
    validate(&config)?;
    Ok(outputs(&config)
        .filter(|(suffix, _, _, _)| !suffix.is_empty())
        .map(|(suffix, _, array, _)| {
            (
                suffix,
                if array { "texture-array" } else { "texture" }.into(),
            )
        })
        .collect())
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
    let c = match s.input::<Config>().and_then(|c| {
        validate(&c)?;
        Ok(c)
    }) {
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
    for (suffix, frame, array, emission) in outputs(&c) {
        let name = if suffix.is_empty() {
            s.rel.clone()
        } else {
            format!("{}#{suffix}", s.rel)
        };
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
                id: format!("{}#{}array", s.rel, if emission { "emission#" } else { "" }),
            }),
        ));
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
