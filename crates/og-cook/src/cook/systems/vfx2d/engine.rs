//! The Pixel VFX Forge renderer, ported from its `engine.js`
//! (third_party/scenario-pixel-vfx): a recipe in, RGBA frames out. Every
//! layer writes a heat field (stored as f32, as the Forge's `Float32Array`
//! stores it), heat is cut into hard colour bands from the layer's ramp,
//! then the frame gets clean-up, outline and palette passes. The arithmetic
//! is the JavaScript's, in its order, in f64, so the frames are the Forge's.

use super::color::{self, Nearest};
use super::jsmath::{self as m, hypot, max, min, round, to_i32, to_u32, truthy};
use super::json::Val;
use std::f64::consts::PI;

/// The renderer's version: part of what the cook hashes, so a change to
/// what it draws cooks every effect again.
pub const ENGINE_VERSION: u32 = 1;

pub(super) const TAU: f64 = PI * 2.0;
pub(super) const DEG: f64 = PI / 180.0;
/// Heat that forces the darkest ramp colour (void cores).
pub(super) const DARK: f64 = 1000.0;

/// The 22 layer types, in the Forge's order.
pub const LAYER_TYPES: [&str; 22] = [
    "burst",
    "ring",
    "flame",
    "beam",
    "slash",
    "orb",
    "comet",
    "vortex",
    "pillar",
    "rays",
    "glyph",
    "puffs",
    "particles",
    "bolt",
    "cracks",
    "spikes",
    "star",
    "streaks",
    "bubble",
    "flash",
    "text",
    "tornado",
];
/// Layers whose isolated pixels the clean-up keeps.
const STAMPS: [&str; 6] = ["particles", "bolt", "glyph", "cracks", "streaks", "text"];

pub(super) fn clamp(x: f64, a: f64, b: f64) -> f64 {
    if x < a {
        a
    } else if x > b {
        b
    } else {
        x
    }
}
pub(super) fn c01(x: f64) -> f64 {
    clamp(x, 0.0, 1.0)
}
pub(super) fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
pub(super) fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = c01((x - a) / (b - a));
    t * t * (3.0 - 2.0 * t)
}

/// `ease(name, t)`: the named curve of `t` clamped to 0..1 (an unknown name is outCubic).
pub(super) fn ease(name: &str, t: f64) -> f64 {
    let t = c01(t);
    match name {
        "linear" => t,
        "inQuad" => t * t,
        "inCubic" => t * t * t,
        "outQuad" => 1.0 - (1.0 - t) * (1.0 - t),
        "outQuart" => 1.0 - m::pow(1.0 - t, 4.0),
        "outExpo" => {
            if t >= 1.0 {
                1.0
            } else {
                1.0 - m::pow(2.0, -10.0 * t)
            }
        }
        "inOutSine" => 0.5 - 0.5 * m::cos(PI * t),
        "outBack" => 1.0 + 2.70158 * m::pow(t - 1.0, 3.0) + 1.70158 * m::pow(t - 1.0, 2.0),
        _ => 1.0 - m::pow(1.0 - t, 3.0),
    }
}

/// `L.ease || d`, as the name `ease` looks up.
pub(super) fn ease_name<'a>(l: &'a Val, d: &'a str) -> String {
    match l.get("ease") {
        Some(v) if v.truthy() => v.js_string(),
        _ => d.to_string(),
    }
}

/// `pair(v, d)`: `[a, b]` from a pair, a number for both, or the default.
pub(super) fn pair(l: &Val, key: &str, d: [f64; 2]) -> [f64; 2] {
    match l.get(key) {
        Some(Val::Arr(a)) => {
            let first = a.first().map_or(f64::NAN, Val::num);
            match a.get(1) {
                Some(v) if !matches!(v, Val::Null) => [first, v.num()],
                _ => [first, first],
            }
        }
        None | Some(Val::Null) => d,
        Some(v) => [v.num(), v.num()],
    }
}

// ---------- hashing, random, noise ----------

pub(super) fn hash(a: f64, b: f64, c: f64, s: f64) -> u32 {
    let mut h = (to_i32(s) ^ 0x9e37_79b9_u32 as i32) as u32;
    h = (h ^ to_i32(a) as u32).wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = (h ^ to_i32(b) as u32).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    h = (h ^ to_i32(c) as u32).wrapping_mul(0x27d4_eb2f);
    h ^= h >> 15;
    h = h.wrapping_mul(0x1656_67b1);
    h ^= h >> 13;
    h
}
pub(super) fn hash01(a: f64, b: f64, c: f64, s: f64) -> f64 {
    f64::from(hash(a, b, c, s)) / 4294967296.0
}

/// The Forge's `rng`: mulberry32.
pub(super) struct Rng(u32);
impl Rng {
    pub(super) fn new(seed: f64) -> Rng {
        Rng(to_u32(seed))
    }
    pub(super) fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        f64::from(t ^ (t >> 14)) / 4294967296.0
    }
    pub(super) fn q7(&mut self) -> [f64; 7] {
        std::array::from_fn(|_| self.next())
    }
}

const GX: [f64; 12] = [1., -1., 1., -1., 1., -1., 1., -1., 0., 0., 0., 0.];
const GY: [f64; 12] = [1., 1., -1., -1., 0., 0., 0., 0., 1., -1., 1., -1.];
const GZ: [f64; 12] = [0., 0., 0., 0., 1., 1., -1., -1., 1., 1., -1., -1.];
fn wrap(i: f64, p: f64) -> f64 {
    if p > 0.0 { ((i % p) + p) % p } else { i }
}
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}
#[allow(clippy::too_many_arguments)]
fn grad(ix: f64, iy: f64, iz: f64, s: f64, x: f64, y: f64, z: f64) -> f64 {
    let g = (hash(ix, iy, iz, s) % 12) as usize;
    GX[g] * x + GY[g] * y + GZ[g] * z
}

/// Gradient noise with optional integer periods per axis, so loops tile.
#[allow(clippy::too_many_arguments)]
pub(super) fn noise3(x: f64, y: f64, z: f64, s: f64, px: f64, py: f64, pz: f64) -> f64 {
    let (xi, yi, zi) = (x.floor(), y.floor(), z.floor());
    let (xf, yf, zf) = (x - xi, y - yi, z - zi);
    let (x0, x1) = (wrap(xi, px), wrap(xi + 1.0, px));
    let (y0, y1) = (wrap(yi, py), wrap(yi + 1.0, py));
    let (z0, z1) = (wrap(zi, pz), wrap(zi + 1.0, pz));
    let (u, v, w) = (fade(xf), fade(yf), fade(zf));
    let a = lerp(
        grad(x0, y0, z0, s, xf, yf, zf),
        grad(x1, y0, z0, s, xf - 1.0, yf, zf),
        u,
    );
    let b = lerp(
        grad(x0, y1, z0, s, xf, yf - 1.0, zf),
        grad(x1, y1, z0, s, xf - 1.0, yf - 1.0, zf),
        u,
    );
    let c = lerp(
        grad(x0, y0, z1, s, xf, yf, zf - 1.0),
        grad(x1, y0, z1, s, xf - 1.0, yf, zf - 1.0),
        u,
    );
    let d = lerp(
        grad(x0, y1, z1, s, xf, yf - 1.0, zf - 1.0),
        grad(x1, y1, z1, s, xf - 1.0, yf - 1.0, zf - 1.0),
        u,
    );
    lerp(lerp(a, b, v), lerp(c, d, v), w)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn fbm_p(
    mut x: f64,
    mut y: f64,
    mut z: f64,
    s: f64,
    oct: u32,
    mut px: f64,
    mut py: f64,
    mut pz: f64,
) -> f64 {
    let (mut sum, mut amp, mut norm) = (0.0, 1.0, 0.0);
    for o in 0..oct {
        sum += amp * noise3(x, y, z, s + f64::from(o) * 1013.0, px, py, pz);
        norm += amp;
        amp *= 0.5;
        x *= 2.0;
        y *= 2.0;
        z *= 2.0;
        px *= 2.0;
        py *= 2.0;
        pz *= 2.0;
    }
    c01(0.5 + (0.78 * sum) / norm)
}
pub(super) fn fbm(x: f64, y: f64, z: f64, s: f64, oct: u32) -> f64 {
    fbm_p(x, y, z, s, oct, 0.0, 0.0, 0.0)
}

/// Per-angle values (silhouette wobble, ring breakup), as a `Float32Array`.
pub(super) fn angle_table(n: usize, f: impl Fn(f64) -> f64) -> Vec<f32> {
    (0..=n)
        .map(|i| f((i as f64 / n as f64) * TAU) as f32)
        .collect()
}
pub(super) fn lookup(t: &[f32], a: f64) -> f64 {
    let n = (t.len() - 1) as f64;
    let mut f = ((((a / TAU) % 1.0) + 1.0) % 1.0) * n;
    if f.is_nan() {
        return f64::NAN;
    }
    let i = f.floor();
    f -= i;
    let i = i as usize;
    f64::from(t[i]) * (1.0 - f) + f64::from(t[i + 1]) * f
}

// ---------- the canvas a layer draws its heat into ----------

pub(super) struct C<'a> {
    pub(super) w: f64,
    pub(super) h: f64,
    pub(super) wi: usize,
    pub(super) hi: usize,
    pub(super) s: f64,
    pub(super) f: f64,
    pub(super) lt: f64,
    pub(super) ph: f64,
    pub(super) looping: bool,
    pub(super) seed: f64,
    pub(super) heat: &'a mut [f32],
}

impl C<'_> {
    pub(super) fn plot(&mut self, x: f64, y: f64, v: f64) {
        if !(x >= 0.0 && y >= 0.0 && x < self.w && y < self.h)
            || x.fract() != 0.0
            || y.fract() != 0.0
        {
            return;
        }
        let i = y as usize * self.wi + x as usize;
        if v > f64::from(self.heat[i]) {
            self.heat[i] = v as f32;
        }
    }
    pub(super) fn px(&self, u: f64) -> f64 {
        round(self.w / 2.0 + u * self.s - 0.5)
    }
    pub(super) fn py(&self, v: f64) -> f64 {
        round(self.h / 2.0 + v * self.s - 0.5)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn line(
        &mut self,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        v0: f64,
        v1: f64,
        thick: f64,
    ) {
        if ![x0, y0, x1, y1].iter().all(|v| v.is_finite()) {
            return;
        }
        let (dx, dy) = ((x1 - x0).abs(), (y1 - y0).abs());
        let sx = if x0 < x1 { 1.0 } else { -1.0 };
        let sy = if y0 < y1 { 1.0 } else { -1.0 };
        let n = max(dx, dy);
        let n = if truthy(n) { n } else { 1.0 };
        let (mut err, mut x, mut y, mut k) = (dx - dy, x0, y0, 0.0);
        loop {
            let v = lerp(v0, v1, k / n);
            self.plot(x, y, v);
            if thick > 1.0 {
                if dx > dy {
                    self.plot(x, y + 1.0, v * 0.92);
                } else {
                    self.plot(x + 1.0, y, v * 0.92);
                }
                if thick > 2.0 {
                    if dx > dy {
                        self.plot(x, y - 1.0, v * 0.92);
                    } else {
                        self.plot(x - 1.0, y, v * 0.92);
                    }
                }
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2.0 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
            k += 1.0;
        }
    }
    pub(super) fn line1(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, v0: f64) {
        self.line(x0, y0, x1, y1, v0, v0, 1.0);
    }

    pub(super) fn disc(&mut self, cx: f64, cy: f64, r: f64, v: f64) {
        let ri = r.ceil();
        let mut y = -ri;
        while y <= ri {
            let mut x = -ri;
            while x <= ri {
                if x * x + y * y <= r * r + 0.3 {
                    self.plot(cx + x, cy + y, v * (1.0 - 0.25 * hypot(x, y) / (r + 0.5)));
                }
                x += 1.0;
            }
            y += 1.0;
        }
    }

    /// Grows bright pixels outward in decaying steps: a pixel-art glow.
    fn dilate(&mut self, passes: f64, falloff: f64) {
        let (w, h) = (self.wi, self.hi);
        let mut src = self.heat.to_vec();
        let mut p = 0.0;
        while p < passes {
            let mut dst = src.clone();
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    let mut mm = f64::from(src[i]);
                    let mut take = |j: usize| {
                        let c = f64::from(src[j]) * falloff;
                        if c > mm {
                            mm = c;
                        }
                    };
                    if x > 0 {
                        take(i - 1);
                    }
                    if x + 1 < w {
                        take(i + 1);
                    }
                    if y > 0 {
                        take(i - w);
                    }
                    if y + 1 < h {
                        take(i + w);
                    }
                    dst[i] = if mm >= DARK { src[i] } else { mm as f32 };
                }
            }
            src = dst;
            p += 1.0;
        }
        self.heat.copy_from_slice(&src);
    }

    /// Evaluates `f` over the pixels near an anchor, in rotated and
    /// squashed local units.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn field(
        &mut self,
        sq: f64,
        reach: f64,
        ax: f64,
        ay: f64,
        angle: f64,
        mut f: impl FnMut(f64, f64) -> f64,
    ) {
        let (w, h, s) = (self.w, self.h, self.s);
        let (ca, sa) = (m::cos(-angle * DEG), m::sin(-angle * DEG));
        let rr = reach * max(1.0, sq);
        let x0 = max(0.0, (w / 2.0 + (ax - rr) * s).floor());
        let x1 = min(w - 1.0, (w / 2.0 + (ax + rr) * s).ceil());
        let y0 = max(0.0, (h / 2.0 + (ay - rr) * s).floor());
        let y1 = min(h - 1.0, (h / 2.0 + (ay + rr) * s).ceil());
        let mut y = y0;
        while y <= y1 {
            let v = (y + 0.5 - h / 2.0) / s - ay;
            let mut x = x0;
            while x <= x1 {
                let u = (x + 0.5 - w / 2.0) / s - ax;
                let (lx, ly) = (u * ca - v * sa, (u * sa + v * ca) / sq);
                let val = f(lx, ly);
                let i = y as usize * self.wi + x as usize;
                if val > f64::from(self.heat[i]) {
                    self.heat[i] = val as f32;
                }
                x += 1.0;
            }
            y += 1.0;
        }
    }

    /// `field` at the layer's own anchor, angle and squash.
    pub(super) fn field_l(&mut self, l: &Val, reach: f64, f: impl FnMut(f64, f64) -> f64) {
        self.field(
            l.n("squash", 1.0),
            reach,
            l.n("x", 0.0),
            l.n("y", 0.0),
            l.n("angle", 0.0),
            f,
        );
    }
}

// ---------- recipe ----------

/// A layer of a normalised recipe: its JSON, type and resolved ramp.
struct Layer<'a> {
    l: &'a Val,
    ty: &'a str,
    ramp: Vec<i32>,
}

/// A recipe read as `normalize` reads it.
struct Recipe<'a> {
    w: usize,
    h: usize,
    frames: usize,
    fps: u32,
    looping: bool,
    seed: f64,
    dither: f64,
    cleanup: bool,
    outline: Option<String>,
    outline_color: Option<i32>,
    palette: Option<Vec<i32>>,
    layers: Vec<Layer<'a>>,
}

const DEFAULT_RAMPS: [(&str, &str); 4] = [
    ("main", "fire"),
    ("alt", "fire"),
    ("smoke", "smoke"),
    ("spark", "white"),
];

fn clamp_int(v: Option<&Val>, a: f64, b: f64, d: f64) -> f64 {
    let n = round(v.map_or(f64::NAN, Val::num));
    if n.is_finite() { min(b, max(a, n)) } else { d }
}

fn normalize(recipe: &Val) -> Recipe<'_> {
    // `{ ...DEFAULTS, ...recipe }`: a key the recipe has wins, null included.
    let field = |k: &str| recipe.get(k);
    let w = clamp_int(field("w").or(Some(&Val::Num(64.0))), 8.0, 512.0, 64.0) as usize;
    let h = clamp_int(field("h").or(Some(&Val::Num(64.0))), 8.0, 512.0, 64.0) as usize;
    let frames = clamp_int(field("frames").or(Some(&Val::Num(16.0))), 1.0, 120.0, 16.0) as usize;
    let fps = clamp_int(field("fps").or(Some(&Val::Num(15.0))), 1.0, 60.0, 15.0) as u32;
    let seed = clamp_int(
        field("seed").or(Some(&Val::Num(1.0))),
        0.0,
        2147483647.0,
        1.0,
    );
    let dither = field("dither").map_or(0.0, Val::num);
    let dither = c01(if truthy(dither) { dither } else { 0.0 });
    let looping = field("loop").is_some_and(Val::truthy);
    let cleanup = field("cleanup").is_none_or(Val::truthy);
    let outline = match field("outline") {
        None => Some("none".to_string()),
        Some(v) if v.truthy() => Some(v.js_string()),
        Some(_) => None,
    }
    .filter(|o| o != "none");
    let outline_color = field("outlineColor")
        .filter(|v| v.truthy())
        .map(|v| color::pack(color::hex_to_rgb(&v.js_string())));
    let palette = field("palette").filter(|v| v.truthy()).and_then(|v| {
        let hexes: Vec<String> = match v {
            Val::Arr(a) => a.iter().map(Val::js_string).collect(),
            v => color::named(color::PALETTES, &v.js_string())?
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        (!hexes.is_empty()).then(|| {
            hexes
                .iter()
                .map(|h| color::pack(color::hex_to_rgb(h)))
                .collect()
        })
    });
    let hue = field("hue").map_or(0.0, Val::num);
    let hue = if truthy(hue) { hue } else { 0.0 };
    let sat = match field("sat") {
        None | Some(Val::Null) => 1.0,
        Some(v) => v.num(),
    };
    let ramp_of = |name: &str| -> Val {
        if let Some(r) = recipe.get("ramps").filter(|r| r.truthy())
            && let Some(v) = r.get(name)
        {
            return v.clone();
        }
        match DEFAULT_RAMPS.iter().find(|(k, _)| *k == name) {
            Some((_, d)) => Val::Str(d.to_string()),
            None => Val::Null,
        }
    };
    let layers = match recipe.get("layers") {
        Some(Val::Arr(a)) => a
            .iter()
            .filter_map(|l| {
                let ty = LAYER_TYPES.iter().find(|t| l.s("type") == Some(**t))?;
                let mut spec = l.opt("ramp").cloned().unwrap_or(Val::Str("$main".into()));
                if let Val::Str(s) = &spec
                    && let Some(name) = s.strip_prefix('$')
                {
                    spec = match ramp_of(name) {
                        Val::Null => Val::Str("fire".into()),
                        v => v,
                    };
                }
                let lh = l.get("hue").map_or(0.0, Val::num);
                let lh = if truthy(lh) { lh } else { 0.0 };
                Some(Layer {
                    l,
                    ty,
                    ramp: color::resolve_ramp(&spec, hue + lh, sat),
                })
            })
            .collect(),
        _ => Vec::new(),
    };
    Recipe {
        w,
        h,
        frames,
        fps,
        looping,
        seed,
        dither,
        cleanup,
        outline,
        outline_color,
        palette,
        layers,
    }
}

const BAYER: [f64; 16] = {
    let order = [
        0., 8., 2., 10., 12., 4., 14., 6., 3., 11., 1., 9., 15., 7., 13., 5.,
    ];
    let mut out = [0.0; 16];
    let mut i = 0;
    while i < 16 {
        out[i] = (order[i] + 0.5) / 16.0;
        i += 1;
    }
    out
};

/// Rendered frames: straight RGBA8 rows, `w`×`h` each.
#[derive(Clone, Debug, PartialEq)]
pub struct Rendered {
    pub w: u32,
    pub h: u32,
    pub fps: u32,
    pub looping: bool,
    pub frames: Vec<Vec<u8>>,
    /// The impact frame: the recipe's `impact` (0..1 of the effect), else
    /// the frame with the largest jump in covered pixels in the first 60%.
    pub impact: usize,
    /// Covered pixels per frame.
    pub covered: Vec<usize>,
}

/// Renders every frame of a recipe.
pub fn render(recipe: &Val) -> Rendered {
    let r = normalize(recipe);
    let (w, h) = (r.w, r.h);
    let n = w * h;
    let mut heat = vec![0f32; n];
    let mut color = vec![-1i32; n];
    let mut owner = vec![-1i16; n];
    let mut kind = vec![0u8; n];
    let mut nearest = r.palette.clone().map(Nearest::new);
    let mut out = Rendered {
        w: w as u32,
        h: h as u32,
        fps: r.fps,
        looping: r.looping,
        frames: Vec::new(),
        impact: 0,
        covered: Vec::new(),
    };
    let (mut prev, mut best) = (0i64, -1i64);
    let frames = r.frames as f64;
    for fi in 0..r.frames {
        let f = fi as f64;
        color.fill(-1);
        owner.fill(-1);
        kind.fill(0);
        let t = if r.looping {
            f / frames
        } else if r.frames > 1 {
            f / (frames - 1.0)
        } else {
            0.0
        };
        let s = (w.min(h) as f64) / 2.0;
        for (li, layer) in r.layers.iter().enumerate() {
            let l = layer.l;
            if l.t("hidden") {
                continue;
            }
            let (t0, t1) = (l.n("t0", 0.0), l.n("t1", 1.0));
            let lt = (t - t0) / max(t1 - t0, 1e-4);
            if lt < 0.0 || lt > 1.0 {
                continue;
            }
            heat.fill(0.0);
            let lseed = l.get("seed").map_or(0.0, Val::num);
            let lseed = if truthy(lseed) { lseed } else { 0.0 };
            let mut c = C {
                w: w as f64,
                h: h as f64,
                wi: w,
                hi: h,
                s,
                f,
                lt,
                ph: if r.looping { t } else { lt },
                looping: r.looping,
                seed: f64::from(to_u32(
                    r.seed * 7919.0 + li as f64 * 104729.0 + lseed * 15485863.0,
                )),
                heat: &mut heat,
            };
            super::layers::draw(layer.ty, &mut c, l);
            if l.t("glow") {
                c.dilate(min(4.0, round(l.n("glow", 0.0))), l.n("glowFalloff", 0.5));
            }
            paint(&heat, w, layer, li, &r, &mut color, &mut owner, &mut kind);
        }
        post(&r, &mut color, &mut owner, &kind, nearest.as_mut());
        let mut rgba = vec![0u8; n * 4];
        let mut covered = 0usize;
        for (i, &col) in color.iter().enumerate() {
            if col < 0 {
                continue;
            }
            covered += 1;
            rgba[i * 4..i * 4 + 4].copy_from_slice(&[
                (col >> 16) as u8,
                (col >> 8) as u8,
                col as u8,
                255,
            ]);
        }
        let jump = covered as i64 - prev;
        if (fi as f64) < (frames * 0.6).ceil() && jump > best {
            best = jump;
            out.impact = fi;
        }
        prev = covered as i64;
        out.frames.push(rgba);
        out.covered.push(covered);
    }
    if let Some(i) = recipe.opt("impact") {
        let v = round(i.num() * (frames - 1.0));
        out.impact = if v.is_nan() {
            0
        } else {
            v.clamp(0.0, frames - 1.0) as usize
        };
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn paint(
    heat: &[f32],
    w: usize,
    layer: &Layer,
    li: usize,
    r: &Recipe,
    color: &mut [i32],
    owner: &mut [i16],
    kind: &mut [u8],
) {
    let l = layer.l;
    let ramp = &layer.ramp;
    let n = ramp.len();
    let cut = l.n("cut", 0.1);
    let dith = l.opt("dither").map_or(r.dither, Val::num);
    let gamma = l.n("gamma", 1.0);
    let under = l.s("blend") == Some("under");
    let bs = n as f64 / (1.0 - cut);
    let stamp = u8::from(STAMPS.contains(&layer.ty));
    for (i, &hv) in heat.iter().enumerate() {
        let mut v = f64::from(hv);
        if v <= 0.0 || (under && color[i] >= 0) {
            continue;
        }
        let idx = if v >= DARK {
            0
        } else {
            if gamma != 1.0 {
                v = m::pow(v, gamma);
            }
            if truthy(dith) {
                let (x, y) = (i % w, i / w);
                v += ((BAYER[(x & 3) + ((y & 3) << 2)] - 0.5) * dith) / bs;
            }
            if v < cut || v.is_nan() {
                continue;
            }
            let k = ((v - cut) * bs).floor();
            if k >= n as f64 { n - 1 } else { k as usize }
        };
        color[i] = ramp[idx];
        owner[i] = li as i16;
        kind[i] = stamp;
    }
}

fn post(
    r: &Recipe,
    color: &mut [i32],
    owner: &mut [i16],
    kind: &[u8],
    nearest: Option<&mut Nearest>,
) {
    let (w, h) = (r.w, r.h);
    if r.cleanup {
        let mut rm = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if color[i] < 0 || kind[i] != 0 {
                    continue;
                }
                let mut nb = 0;
                for dy in -1i64..=1 {
                    for dx in -1i64..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let (xx, yy) = (x as i64 + dx, y as i64 + dy);
                        if xx >= 0
                            && yy >= 0
                            && xx < w as i64
                            && yy < h as i64
                            && color[(yy as usize) * w + xx as usize] >= 0
                        {
                            nb += 1;
                        }
                    }
                }
                if nb == 0 {
                    rm.push(i);
                }
            }
        }
        for i in rm {
            color[i] = -1;
        }
        for y in 1..h.saturating_sub(1) {
            for x in 1..w - 1 {
                let i = y * w + x;
                if color[i] >= 0 {
                    continue;
                }
                let (a, b, u, d) = (color[i - 1], color[i + 1], color[i - w], color[i + w]);
                if a >= 0
                    && b >= 0
                    && u >= 0
                    && d >= 0
                    && owner[i - 1] == owner[i + 1]
                    && owner[i - w] == owner[i + w]
                {
                    color[i] = a;
                    owner[i] = owner[i - 1];
                }
            }
        }
    }
    if let Some(mode) = &r.outline {
        let fixed = r.outline_color;
        let ramp0 = |o: i16| {
            usize::try_from(o)
                .ok()
                .and_then(|o| r.layers.get(o))
                .map(|l| l.ramp[0])
        };
        if mode == "inside" {
            let mut edge = Vec::new();
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    if color[i] < 0 {
                        continue;
                    }
                    if x == 0
                        || y == 0
                        || x == w - 1
                        || y == h - 1
                        || color[i - 1] < 0
                        || color[i + 1] < 0
                        || color[i - w] < 0
                        || color[i + w] < 0
                    {
                        edge.push(i);
                    }
                }
            }
            for i in edge {
                color[i] = fixed.or_else(|| ramp0(owner[i])).unwrap_or(color[i]);
            }
        } else {
            let diag = mode == "outside8";
            let mut add = Vec::new();
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    if color[i] >= 0 {
                        continue;
                    }
                    let mut src = None;
                    if x > 0 && color[i - 1] >= 0 {
                        src = Some(i - 1);
                    } else if x < w - 1 && color[i + 1] >= 0 {
                        src = Some(i + 1);
                    } else if y > 0 && color[i - w] >= 0 {
                        src = Some(i - w);
                    } else if y < h - 1 && color[i + w] >= 0 {
                        src = Some(i + w);
                    } else if diag {
                        for (dx, dy) in [(-1i64, -1i64), (1, -1), (-1, 1), (1, 1)] {
                            let (xx, yy) = (x as i64 + dx, y as i64 + dy);
                            if xx >= 0
                                && yy >= 0
                                && xx < w as i64
                                && yy < h as i64
                                && color[yy as usize * w + xx as usize] >= 0
                            {
                                src = Some(yy as usize * w + xx as usize);
                                break;
                            }
                        }
                    }
                    if let Some(s) = src {
                        add.push((i, s));
                    }
                }
            }
            for (i, s) in add {
                color[i] =
                    fixed.unwrap_or_else(|| color::darken(ramp0(owner[s]).unwrap_or(0), 0.55));
                owner[i] = owner[s];
            }
        }
    }
    if let Some(nearest) = nearest {
        for c in color.iter_mut() {
            if *c >= 0 {
                *c = nearest.get(*c);
            }
        }
    }
}
