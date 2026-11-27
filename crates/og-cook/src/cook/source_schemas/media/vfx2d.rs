//! Forge recipe values as typed data. The renderer's normalization and
//! coverage/event checks remain in vfx2d; recipes are not JSON text fields.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars", extend("x-og-output-kind" = "flipbook"))]
#[serde(deny_unknown_fields)]
pub struct Vfx2d {
    pub recipe: Recipe,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub duration: Option<f32>,
    #[serde(rename = "loop")]
    pub looping: Option<bool>,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub impact: Option<f32>,
    #[serde(default)]
    #[schemars(length(max = 16))]
    pub events: Vec<Event>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Event {
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub time: f32,
    #[schemars(length(min = 1, max = 63))]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Recipe {
    pub name: Option<String>,
    #[schemars(range(min = 8, max = 512), extend("x-og-unit" = "px"))]
    pub w: Option<u32>,
    #[schemars(range(min = 8, max = 512), extend("x-og-unit" = "px"))]
    pub h: Option<u32>,
    #[schemars(range(min = 1, max = 120))]
    pub frames: Option<u32>,
    #[schemars(range(min = 1, max = 60), extend("x-og-unit" = "Hz"))]
    pub fps: Option<u32>,
    #[schemars(range(max = 2_147_483_647))]
    pub seed: Option<u32>,
    #[serde(rename = "loop")]
    pub looping: Option<bool>,
    #[serde(default)]
    pub ramps: BTreeMap<String, Ramp>,
    #[schemars(extend("x-og-unit" = "deg"))]
    pub hue: Option<f64>,
    pub sat: Option<f64>,
    pub outline: Option<Outline>,
    pub outline_color: Option<String>,
    #[schemars(range(min = 0, max = 1))]
    pub dither: Option<f64>,
    #[schemars(range(min = 1, max = 8))]
    pub supersample: Option<u32>,
    #[schemars(range(min = 0, max = 8))]
    pub line: Option<f64>,
    #[schemars(range(min = 0.25, max = 4))]
    pub scale: Option<f64>,
    #[schemars(range(min = 0, max = 8))]
    pub inner_line: Option<f64>,
    pub palette: Option<Ramp>,
    pub cleanup: Option<bool>,
    pub pivot: Option<[f64; 2]>,
    #[schemars(range(min = 0, max = 1))]
    pub impact: Option<f64>,
    #[schemars(length(min = 1))]
    pub layers: Vec<Layer>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Ramp {
    /// A named ramp, $reference, hue:degrees, or a named palette.
    Named(String),
    Colors(Vec<String>),
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Outline {
    None,
    Outside,
    Outside8,
    Inside,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Pair {
    Constant(f64),
    Range([f64; 2]),
}

// The common fields are one Rust definition, expanded into closed variants.
// Flattening an open parameter map would let typos bypass both serde and TS.
macro_rules! layers {
    ($($kind:ident { $($field:ident: $ty:ty),* $(,)? }),+ $(,)?) => {
        #[derive(Debug, Deserialize, JsonSchema)]
        #[schemars(crate = "og_core::reflect::source::schemars")]
        #[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
        pub enum Layer { $($kind(Box<$kind>)),+ }

        $(
            #[derive(Debug, Deserialize, JsonSchema)]
            #[schemars(crate = "og_core::reflect::source::schemars")]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            pub struct $kind {
                pub ramp: Option<Ramp>,
                /// Normalized effect time; a layer may extend past the clip.
                pub t0: Option<f64>,
                pub t1: Option<f64>,
                /// Half the shorter canvas dimension, y down.
                pub x: Option<f64>, pub y: Option<f64>,
                #[schemars(extend("x-og-unit" = "deg"))]
                pub angle: Option<f64>,
                pub squash: Option<f64>, pub gain: Option<f64>, pub cut: Option<f64>,
                pub gamma: Option<f64>,
                #[schemars(range(min = 0, max = 1))]
                pub dither: Option<f64>,
                pub glow: Option<f64>, pub glow_falloff: Option<f64>,
                pub blend: Option<Blend>, pub hidden: Option<bool>, pub seed: Option<u32>,
                pub hue: Option<f64>, pub ease: Option<String>,
                $(pub $field: Option<$ty>),*
            }
        )+
    };
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Blend {
    Over,
    Under,
}

layers! {
    Burst { cool: f64, drift: f64, erode: f64, erode_start: f64, evolve: f64, freq: f64, grow: f64, hollow: f64, r0: f64, r1: f64, rough: f64, wobble: f64 },
    Ring { breakup: f64, fade: f64, freq: f64, inner: f64, r0: f64, r1: f64, th: f64, th_end: f64 },
    Flame { flick: f64, freq: f64, hgt: f64, hollow: f64, rough: f64, speed: f64, taper: f64, w: f64 },
    Beam { cap: f64, freq: f64, grow: f64, len: f64, speed: f64, th: f64, wob: f64 },
    Slash { a0: f64, a1: f64, erode: f64, r: f64, sweep: f64, th: f64, trail: f64 },
    Orb { freq: f64, grow: f64, pulse: f64, pulse_rate: f64, r: f64, rim: f64, spin: f64 },
    Comet { freq: f64, from: [f64; 2], len: f64, r: f64, rough: f64, speed: f64, until: f64 },
    Vortex { arms: f64, core: Core, dir: f64, freq: f64, r: f64, r_in: f64, spin: f64, twist: f64 },
    Pillar { freq: f64, grow: f64, hgt: f64, rough: f64, speed: f64, w: f64 },
    Rays { count: f64, inner: f64, jitter: f64, r: f64, spin: f64, twinkle: f64, width: f64 },
    Glyph { draw: f64, fill: f64, r: f64, rings: Vec<f64>, runes: f64, spin: f64, star: f64, step: f64, ticks: f64 },
    Puffs { cone: f64, count: f64, decay: f64, dir: f64, dissolve: f64, drift: f64, emit: f64, flat: f64, freq: f64, life: Pair, r0: f64, r1: f64, rise: f64, speed: Pair, spread: f64 },
    Particles { area: f64, area_x: f64, area_y: f64, both: bool, count: f64, dir: f64, drag: f64, drift_x: f64, emit: f64, fade_in: f64, fall_scale: f64, gravity: f64, ground: f64, heat: Pair, implode_r: f64, jitter: f64, life: Pair, mode: ParticleMode, orbit_r: f64, shape: ParticleShape, size: Pair, span_x: f64, span_y: f64, speed: Pair, spread: f64, swirl: f64, turns: f64, twinkle: f64, wiggle: f64, wiggle_rate: f64 },
    Bolt { branch: f64, count: f64, depth: f64, fade_start: f64, flicker: f64, hold: f64, jag: f64, len: f64, mode: BoltMode, spread: f64, thick: f64, x0: f64, x1: f64, y0: f64, y1: f64 },
    Cracks { count: f64, fade_start: f64, grow: f64, len: f64 },
    Spikes { count: f64, fan: f64, grow: f64, hgt: Pair, hold: f64, lean: f64, spread: f64, width: Pair },
    Star { hollow: f64, inner: f64, jitter: f64, points: f64, pop: f64, r: f64, sharp: f64 },
    Streaks { count: f64, curve: f64, dir: f64, gap: f64, len: f64, mode: StreakMode, r: f64, span_y: f64, speed: f64, stagger: f64, th: f64, turns: f64 },
    Bubble { cell: f64, fill: f64, hex: f64, r: f64, shimmer: f64, th: f64 },
    Flash { dur: f64, r: f64 },
    Text { text: String, fade_start: f64, pop: bool, rise: f64, scale: f64 },
    Tornado { freq: f64, hgt: f64, rough: f64, spin: f64, sway: f64, twist: f64, w0: f64, w1: f64 },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Core {
    Dark,
    Light,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum ParticleMode {
    Burst,
    Spray,
    Rise,
    Fall,
    Ambient,
    Orbit,
    Implode,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum ParticleShape {
    Dot,
    Square,
    Plus,
    Cross,
    Star,
    Diamond,
    Ring,
    Bubble,
    Streak,
    Drop,
    Shard,
    Leaf,
    Petal,
    Chevron,
    Coin,
    Chunk,
    Circle,
    Heart,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum BoltMode {
    Radial,
    Strike,
    Line,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum StreakMode {
    Claw,
    Speed,
    Flow,
}

schema!(Vfx2d);
