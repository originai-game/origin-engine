//! The theme art generator: a theme's nine-slice frame art drawn from its
//! art spec (a material, a shape, ornaments and the resolved palette).
//!
//! The spec is `templates/tools/theme-art.py`; this is its port and draws
//! the same bytes (the parity test checks every piece of every spec in
//! `templates/tools/theme-art-specs.json` against the digests the Python
//! writes to `crates/og-cook/tests/ui_art_parity.json`). Every operation
//! keeps the Python's order and uses only + - * / sqrt, floor and
//! comparisons in f64, so the two agree exactly; read the Python for what
//! each step draws and why.
//!
//! Each piece is drawn on its collapsed canvas (its insets, no middle) and
//! its stretch column and row are repeated over the middle band, so every
//! stretched row and column of the output is identical. [`draw_cached`]
//! keeps the output under the spec's hash.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// One finished piece: its image name, its RGBA bytes and its pack entry
/// (`images.<name>` as a pack writes it).
pub struct Piece {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub entry: Value,
}

const MATERIAL_KEYS: [&str; 16] = [
    "drop", "glow", "outline", "lip", "rim", "face", "grain", "edge", "bevel", "gloss", "toplight",
    "hairline", "shade", "sunk", "hover", "press",
];

type Rgba = [f64; 4];

fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

fn hexc(h: &str) -> Result<Rgba, String> {
    let h = h.trim_start_matches('#');
    let byte = |i: usize| {
        h.get(i..i + 2)
            .and_then(|s| u8::from_str_radix(s, 16).ok())
            .map(|b| b as f64 / 255.0)
            .ok_or_else(|| format!("colour #{h}: not #rrggbb"))
    };
    let a = if h.len() == 8 { byte(6)? } else { 1.0 };
    Ok([byte(0)?, byte(2)?, byte(4)?, a])
}

fn mix(a: Rgba, b: Rgba, t: f64) -> Rgba {
    [0, 1, 2, 3].map(|i| a[i] * (1.0 - t) + b[i] * t)
}

const WHITE: Rgba = [1.0, 1.0, 1.0, 1.0];
const BLACK: Rgba = [0.0, 0.0, 0.0, 1.0];

/// Resolves colour refs against the palette and the current piece's face.
struct Colors<'a> {
    palette: &'a Value,
    top: Rgba,
    bottom: Rgba,
}

impl Colors<'_> {
    fn get(&self, r: &Value) -> Result<Rgba, String> {
        if let Some(m) = r.get("mix") {
            let t = num(&m[2])?;
            return Ok(mix(self.get(&m[0])?, self.get(&m[1])?, t));
        }
        let s = r
            .as_str()
            .ok_or_else(|| format!("colour {r}: not a string"))?;
        match s {
            "top" => Ok(self.top),
            "bottom" => Ok(self.bottom),
            _ if s.starts_with('#') => hexc(s),
            _ => {
                let v = self
                    .palette
                    .get(s)
                    .ok_or_else(|| format!("palette: no role {s}"))?;
                hexc(
                    v.as_str()
                        .or_else(|| v[0].as_str())
                        .ok_or_else(|| format!("palette.{s}"))?,
                )
            }
        }
    }

    fn named(&self, s: &str) -> Result<Rgba, String> {
        self.get(&Value::from(s))
    }

    fn pair(&self, role: &str) -> Result<(Rgba, Rgba), String> {
        let v = &self.palette[role];
        let pick = |i: usize| {
            hexc(
                v[i].as_str()
                    .ok_or_else(|| format!("palette.{role}: [top, bottom]"))?,
            )
        };
        Ok((pick(0)?, pick(1)?))
    }
}

fn num(v: &Value) -> Result<f64, String> {
    v.as_f64().ok_or_else(|| format!("{v}: not a number"))
}

fn opt(v: &Value, k: &str, d: f64) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(d)
}

fn key(v: &Value, k: &str) -> Result<f64, String> {
    num(v.get(k).ok_or_else(|| format!("missing `{k}`"))?)
}

/// A per-pixel field.
type F = Vec<f64>;

/// A colour, one or per pixel.
enum Col {
    One(Rgba),
    Each(Vec<Rgba>),
}

struct Canvas {
    s: f64,
    si: usize,
    insets: [usize; 4],
    band: (usize, usize),
    w: usize,
    h: usize,
    nx: usize,
    ny: usize,
    x: F,
    y: F,
    /// The finished image's columns and rows, each the sample it shows:
    /// the stretch column and row repeated over the band.
    cols: Vec<usize>,
    rows: Vec<usize>,
    /// The finished image. Layers are drawn on the samples and laid over
    /// it, so a layer drawn in its own coordinates can vary along the band.
    rgba: Vec<Rgba>,
    /// Whether a layer varies along the band, so the piece tiles it.
    tiled: bool,
}

fn blend(base: &mut Rgba, c: Rgba, alpha: f64) {
    let a = clamp(alpha) * c[3];
    let ba = base[3];
    let out_a = a + ba * (1.0 - a);
    let d = out_a.max(1e-6);
    for k in 0..3 {
        base[k] = (c[k] * a + base[k] * ba * (1.0 - a)) / d;
    }
    base[3] = out_a;
}

impl Canvas {
    fn new(scale: usize, insets: [f64; 4], band: (usize, usize)) -> Canvas {
        let insets = insets.map(|v| v.ceil() as usize);
        let [t, r, b, l] = insets;
        let (w, h) = (l + r, t + b);
        let (nx, ny) = (w * scale + 1, h * scale + 1);
        let mut x = Vec::with_capacity(nx * ny);
        let mut y = Vec::with_capacity(nx * ny);
        for j in 0..ny {
            for i in 0..nx {
                x.push(i as f64 / scale as f64);
                y.push(j as f64 / scale as f64);
            }
        }
        let s = scale;
        let idx = |n: usize, at: usize, band: usize| -> Vec<usize> {
            (0..at * s)
                .chain(std::iter::repeat_n(at * s, band * s))
                .chain(at * s + 1..n * s + 1)
                .collect()
        };
        let cols = idx(w, l, band.0);
        let rows = idx(h, t, band.1);
        let rgba = vec![[0.0; 4]; cols.len() * rows.len()];
        Canvas {
            s: scale as f64,
            si: scale,
            insets,
            band,
            w,
            h,
            nx,
            ny,
            x,
            y,
            cols,
            rows,
            rgba,
            tiled: false,
        }
    }

    fn wf(&self) -> f64 {
        self.w as f64
    }

    fn hf(&self) -> f64 {
        self.h as f64
    }

    fn map(&self, f: impl Fn(f64, f64) -> f64) -> F {
        self.x.iter().zip(&self.y).map(|(&x, &y)| f(x, y)).collect()
    }

    /// Lays `color` over the image at coverage `alpha`, both on the samples.
    fn over(&mut self, color: &Col, alpha: &[f64]) {
        let w = self.cols.len();
        for (k, base) in self.rgba.iter_mut().enumerate() {
            let i = self.rows[k / w] * self.nx + self.cols[k % w];
            let c = match color {
                Col::One(c) => *c,
                Col::Each(v) => v[i],
            };
            blend(base, c, alpha[i]);
        }
    }

    /// Lays `c` over the image at coverage `alpha`, on the image's pixels.
    fn over_whole(&mut self, c: Rgba, alpha: &[f64]) {
        for (base, &a) in self.rgba.iter_mut().zip(alpha) {
            blend(base, c, a);
        }
    }

    /// A sample field as the finished image lays it out.
    fn whole(&self, v: &[f64]) -> F {
        let w = self.cols.len();
        (0..self.rgba.len())
            .map(|k| v[self.rows[k / w] * self.nx + self.cols[k % w]])
            .collect()
    }

    /// Each finished pixel's own position in points: its sample's, moved on
    /// by its run through the band, a whole band past it.
    fn place(&self) -> (F, F) {
        let [t, _, _, l] = self.insets;
        let axis = |j: usize, at: usize, band: usize| {
            let j = if j < (at + band) * self.si { j } else { j + 1 };
            j as f64 / self.s
        };
        let w = self.cols.len();
        let x = (0..self.rgba.len())
            .map(|k| axis(k % w, l, self.band.0))
            .collect();
        let y = (0..self.rgba.len())
            .map(|k| axis(k / w, t, self.band.1))
            .collect();
        (x, y)
    }

    fn over1(&mut self, c: Rgba, alpha: &[f64]) {
        self.over(&Col::One(c), alpha);
    }

    fn cover(&self, d: &[f64]) -> F {
        d.iter().map(|&d| clamp(0.5 - d * self.s)).collect()
    }

    fn vgrad(&self, top: Rgba, bottom: Rgba, y0: f64, y1: f64) -> Col {
        let span = (y1 - y0).max(1e-6);
        Col::Each(
            self.y
                .iter()
                .map(|&y| {
                    let t = clamp((y - y0) / span);
                    [0, 1, 2, 3].map(|k| top[k] * (1.0 - t) + bottom[k] * t)
                })
                .collect(),
        )
    }

    fn vgrad1(&self, top: f64, bottom: f64, y0: f64, y1: f64) -> F {
        let span = (y1 - y0).max(1e-6);
        self.y
            .iter()
            .map(|&y| {
                let t = clamp((y - y0) / span);
                top * (1.0 - t) + bottom * t
            })
            .collect()
    }

    /// The finished image as RGBA bytes.
    fn bytes(&self) -> (u32, u32, Vec<u8>) {
        let out = self
            .rgba
            .iter()
            .flat_map(|px| px.map(|v| (clamp(v) * 255.0 + 0.5).floor() as u8))
            .collect();
        (self.cols.len() as u32, self.rows.len() as u32, out)
    }
}

fn zip(a: &[f64], b: &[f64], f: impl Fn(f64, f64) -> f64) -> F {
    a.iter().zip(b).map(|(&a, &b)| f(a, b)).collect()
}

fn add(d: &[f64], k: f64) -> F {
    d.iter().map(|&v| v + k).collect()
}

// ------------------------------------------------------------------ shapes

fn rounded(x: f64, y: f64, b: [f64; 4], radius: f64) -> f64 {
    let [x0, y0, x1, y1] = b;
    let (cx, cy, hw, hh) = (
        (x0 + x1) / 2.0,
        (y0 + y1) / 2.0,
        (x1 - x0) / 2.0,
        (y1 - y0) / 2.0,
    );
    let r = radius.min(hw).min(hh);
    let (qx, qy) = ((x - cx).abs() - hw + r, (y - cy).abs() - hh + r);
    let (mx, my) = (qx.max(0.0), qy.max(0.0));
    (mx * mx + my * my).sqrt() + qx.max(qy).min(0.0) - r
}

fn chamfer(mut d: f64, x: f64, y: f64, b: [f64; 4], cuts: &[&str], cut: f64) -> f64 {
    let [x0, y0, x1, y1] = b;
    let (cx, cy, hw, hh) = (
        (x0 + x1) / 2.0,
        (y0 + y1) / 2.0,
        (x1 - x0) / 2.0,
        (y1 - y0) / 2.0,
    );
    for c in cuts {
        let sx = if c.contains('l') { -1.0 } else { 1.0 };
        let sy = if c.contains('t') { -1.0 } else { 1.0 };
        d = d.max(
            (sx * (x - cx) + sy * (y - cy) - (hw + hh - cut)) * std::f64::consts::FRAC_1_SQRT_2,
        );
    }
    d
}

fn hash01(ix: i64, iy: i64, seed: i64) -> f64 {
    const M: i64 = 0xFFFF_FFFF;
    let mut h = (ix
        .wrapping_mul(374761393)
        .wrapping_add(iy.wrapping_mul(668265263))
        .wrapping_add(seed.wrapping_mul(2246822519)))
        & M;
    h = ((h ^ (h >> 13)).wrapping_mul(1274126177)) & M;
    h ^= h >> 16;
    h as f64 / 4294967296.0
}

fn noise(x: f64, y: f64, cell: f64, seed: i64) -> f64 {
    let (u, v) = (x / cell, y / cell);
    let (fx0, fy0) = (u.floor(), v.floor());
    let (ix, iy) = (fx0 as i64, fy0 as i64);
    let (mut fx, mut fy) = (u - fx0, v - fy0);
    fx = fx * fx * (3.0 - 2.0 * fx);
    fy = fy * fy * (3.0 - 2.0 * fy);
    let (a, b) = (hash01(ix, iy, seed), hash01(ix + 1, iy, seed));
    let (c, d) = (hash01(ix, iy + 1, seed), hash01(ix + 1, iy + 1, seed));
    let top = a + (b - a) * fx;
    let bot = c + (d - c) * fx;
    top + (bot - top) * fy
}

struct Shape<'a> {
    family: &'a str,
    radii: &'a Value,
    cuts: &'a Value,
    step: f64,
    wobble: f64,
    seed: i64,
}

impl Shape<'_> {
    fn radius(&self, role: &str) -> f64 {
        if let Some(r) = self.radii.get(role).and_then(Value::as_f64) {
            return r;
        }
        if role == "badge" {
            return 12.5;
        }
        opt(self.radii, "control", 6.0)
    }

    /// A chamfer's size, one for every piece or by role.
    fn cut(&self, role: &str) -> f64 {
        match self.cuts {
            Value::Object(_) => self
                .cuts
                .get(role)
                .and_then(Value::as_f64)
                .unwrap_or_else(|| opt(self.cuts, "control", 0.0)),
            v => v.as_f64().unwrap_or(0.0),
        }
    }

    fn corner(&self, role: &str) -> f64 {
        let cut = if matches!(self.family, "cut" | "cut4") {
            self.cut(role)
        } else {
            0.0
        };
        self.radius(role).max(cut)
    }

    fn dist(&self, cv: &Canvas, b: [f64; 4], role: &str, scale_cut: f64) -> F {
        let r = self.radius(role);
        let s = self.step;
        cv.map(|x, y| {
            let (x, y) = if self.family == "pixel" {
                (((x / s).floor() + 0.5) * s, ((y / s).floor() + 0.5) * s)
            } else {
                (x, y)
            };
            match self.family {
                "cut" => chamfer(
                    rounded(x, y, b, r.min(2.0)),
                    x,
                    y,
                    b,
                    &["tl", "br"],
                    self.cut(role) * scale_cut,
                ),
                "cut4" => chamfer(
                    rounded(x, y, b, 0.0),
                    x,
                    y,
                    b,
                    &["tl", "tr", "br", "bl"],
                    self.cut(role) * scale_cut,
                ),
                "brush" => {
                    rounded(x, y, b, r) + (noise(x, y, 5.0, self.seed) - 0.5) * 2.0 * self.wobble
                }
                // each block wholly in or out: its distance a half step
                // from any whole-step offset, so no block is partly covered
                "pixel" => ((rounded(x, y, b, r) / s).floor() + 0.5) * s,
                _ => rounded(x, y, b, r),
            }
        })
    }

    /// The samples a block of the stepped field spans (1 for a smooth one).
    fn span(&self, cv: &Canvas) -> usize {
        if self.family == "pixel" {
            (self.step * cv.s + 0.5).floor() as usize
        } else {
            1
        }
    }

    /// A width the stepped field draws as whole blocks.
    fn whole(&self, w: f64) -> f64 {
        if self.family == "pixel" {
            (w / self.step).ceil() * self.step
        } else {
            w
        }
    }
}

// --------------------------------------------------------------- blur, light

fn box_blur(a: &[f64], nx: usize, ny: usize, r: usize, along_x: bool) -> F {
    if r == 0 {
        return a.to_vec();
    }
    let mut out = vec![0.0; a.len()];
    for j in 0..ny {
        for i in 0..nx {
            let mut acc = 0.0;
            for k in 0..=2 * r {
                let (ii, jj) = if along_x {
                    (i as isize + k as isize - r as isize, j as isize)
                } else {
                    (i as isize, j as isize + k as isize - r as isize)
                };
                if ii >= 0 && jj >= 0 && (ii as usize) < nx && (jj as usize) < ny {
                    acc += a[jj as usize * nx + ii as usize];
                } else {
                    acc += 0.0;
                }
            }
            out[j * nx + i] = acc / (2 * r + 1) as f64;
        }
    }
    out
}

fn blur(cv: &Canvas, mut a: F, sigma: f64) -> F {
    if sigma <= 0.0 {
        return a;
    }
    let w = (4.0 * sigma * sigma + 1.0).sqrt();
    let r = ((w - 1.0) / 2.0 + 0.5).floor() as usize;
    for along_x in [true, false] {
        for _ in 0..3 {
            a = box_blur(&a, cv.nx, cv.ny, r, along_x);
        }
    }
    a
}

/// How much an edge faces the light (above, a little left): 0 to 1, from
/// central differences `k` samples apart (nearer at the edges). A stepped
/// field is flat inside a block, so its slope is read a block apart.
fn lit(cv: &Canvas, d: &[f64], k: usize) -> F {
    let (nx, ny) = (cv.nx, cv.ny);
    let span = |i: usize, n: usize| ((i + k).min(n - 1), i.saturating_sub(k));
    let mut out = vec![0.0; d.len()];
    for j in 0..ny {
        for i in 0..nx {
            let at = |i: usize, j: usize| d[j * nx + i];
            let (hx, lx) = span(i, nx);
            let (hy, ly) = span(j, ny);
            let gx = (at(hx, j) - at(lx, j)) / (hx - lx) as f64;
            let gy = (at(i, hy) - at(i, ly)) / (hy - ly) as f64;
            let n = (gx * gx + gy * gy).sqrt() + 1e-9;
            out[j * nx + i] = clamp(0.5 + 0.5 * ((0.0 - gy / n) * 0.92 - (gx / n) * 0.38));
        }
    }
    out
}

/// A bevel's two tones from `lit`: hard along straight edges, blended round
/// a curve so a rounded corner shades along its arc, not across a diagonal.
fn lit_step(l: f64) -> f64 {
    let t = clamp((l - 0.35) / 0.3);
    t * t * (3.0 - 2.0 * t)
}

fn shadow(cv: &mut Canvas, d: &[f64], dy: f64, sigma: f64, alpha: f64, color: Rgba) {
    let a = cv.cover(d);
    let k = (dy * cv.s + 0.5).floor() as usize;
    let mut sh = vec![0.0; a.len()];
    if k == 0 {
        sh = a;
    } else if k < cv.ny {
        sh[k * cv.nx..].copy_from_slice(&a[..(cv.ny - k) * cv.nx]);
    }
    let b = blur(cv, sh, sigma * cv.s);
    let al: F = b.iter().map(|&v| v * alpha).collect();
    cv.over1(color, &al);
}

fn glow(cv: &mut Canvas, d: &[f64], sigma: f64, alpha: f64, color: Rgba) {
    let c = cv.cover(d);
    let b = blur(cv, c.clone(), sigma * cv.s);
    let al = zip(&b, &c, |b, c| b * alpha * (1.0 - c));
    cv.over1(color, &al);
}

// ----------------------------------------------------------- the surface

fn grain_field(cv: &Canvas, g: &Value, b: [f64; 4]) -> Result<F, String> {
    let kind = g["kind"].as_str().unwrap_or("");
    let cell = opt(g, "cell", 16.0);
    let seed = g.get("seed").and_then(Value::as_i64).unwrap_or(1);
    Ok(match kind {
        "noise" => cv.map(|x, y| noise(x, y, cell, seed) * 2.0 - 1.0),
        "brushed" => cv.map(|_, y| noise(0.5, y, cell, seed) * 2.0 - 1.0),
        "wood" => cv.map(|_, y| {
            let t = y / cell + noise(0.5, y, cell * 3.0, seed) * 1.5;
            let f = t - t.floor();
            let tri = (f - 0.5).abs() * 2.0;
            tri * tri * (3.0 - 2.0 * tri) * 2.0 - 1.0
        }),
        "weave" => cv.map(|x, y| {
            let cx = (x / cell - (x / cell).floor()) - 0.5;
            let cy = (y / cell - (y / cell).floor()) - 0.5;
            (cx.abs() - cy.abs()) * 2.0
        }),
        "stone" => cv.map(|x, y| {
            let n = noise(x, y, cell, seed) * 0.65 + noise(x, y, cell * 0.37, seed + 1) * 0.35;
            n * 2.0 - 1.0
        }),
        "wash" => {
            let v = cv.vgrad1(-0.3, 0.3, b[1], b[3]);
            let n = cv.map(|x, y| noise(x, y, cell, seed));
            zip(&n, &v, |n, v| n * 2.0 - 1.0 + v)
        }
        _ => return Err(format!("unknown grain {kind}")),
    })
}

fn ring(cv: &Canvas, d: &[f64], w: f64) -> F {
    let a = cv.cover(d);
    let b = cv.cover(&add(d, w));
    zip(&a, &b, |a, b| a - b)
}

fn surface(
    cv: &mut Canvas,
    c: &Colors,
    m: &Value,
    d: &[f64],
    b: [f64; 4],
    sunk: bool,
    k: usize,
) -> Result<F, String> {
    let [x0, y0, x1, y1] = b;
    let mut face_d = d.to_vec();
    if let Some(rim) = m.get("rim") {
        let l = lit(cv, d, k);
        let (lo, hi) = (c.get(&rim["lo"])?, c.get(&rim["hi"])?);
        let col = Col::Each(
            l.iter()
                .map(|&l| [0, 1, 2, 3].map(|k| lo[k] * (1.0 - l) + hi[k] * l))
                .collect(),
        );
        let cov = cv.cover(d);
        cv.over(&col, &cov);
        face_d = add(d, key(rim, "width")?);
    }
    let f = m.get("face").cloned().unwrap_or(json!({}));
    let alpha = opt(&f, "alpha", 1.0);
    let (mut top, mut bottom) = (c.top, c.bottom);
    top[3] *= alpha;
    bottom[3] *= alpha;
    let col = match f.get("split").and_then(Value::as_f64) {
        None => cv.vgrad(top, bottom, y0, y1),
        Some(split) => {
            let sy = y0 + (y1 - y0) * split;
            let Col::Each(below) = cv.vgrad(mix(top, bottom, split), bottom, sy, y1) else {
                unreachable!()
            };
            Col::Each(
                cv.y.iter()
                    .zip(below)
                    .map(|(&y, b)| if y < sy { top } else { b })
                    .collect(),
            )
        }
    };
    let fc = cv.cover(&face_d);
    cv.over(&col, &fc);
    if let Some(g) = m.get("grain") {
        let n = grain_field(cv, g, b)?;
        let st = key(g, "strength")?;
        let light = c.get(g.get("light").unwrap_or(&json!("#ffffff")))?;
        let dark = c.get(g.get("color").unwrap_or(&json!("#000000")))?;
        cv.over1(light, &zip(&fc, &n, |f, n| f * clamp(n) * st));
        cv.over1(dark, &zip(&fc, &n, |f, n| f * clamp(0.0 - n) * st));
    }
    if let Some(e) = m.get("edge") {
        let (w, a) = (key(e, "width")?, key(e, "alpha")?);
        let al = zip(&fc, &face_d, |f, d| f * clamp(1.0 + (d + 0.5) / w) * a);
        cv.over1(c.get(&e["color"])?, &al);
    }
    if let Some(bv) = m.get("bevel") {
        let w = key(bv, "width")?;
        let rg = ring(cv, &face_d, w);
        let l: F = lit(cv, &face_d, k).into_iter().map(lit_step).collect();
        let (hi, lo) = (key(bv, "hi")?, key(bv, "lo")?);
        cv.over1(WHITE, &zip(&rg, &l, |r, l| r * l * hi));
        cv.over1(BLACK, &zip(&rg, &l, |r, l| r * (1.0 - l) * lo));
        let inner = c.get(bv.get("inner").unwrap_or(&json!("top")))?;
        let ia = opt(bv, "inner_alpha", 0.0);
        let al: F = cv.cover(&add(&face_d, w)).iter().map(|&v| v * ia).collect();
        cv.over1(inner, &al);
    }
    if let Some(gl) = m.get("gloss")
        && !sunk
    {
        let ins = opt(gl, "inset", 3.5);
        let h = (y1 - y0) * key(gl, "band")?;
        let gb_box = [
            x0 + ins,
            y0 + ins * 0.7,
            x1 - ins,
            y0 + ins * 0.7 + h * 0.62,
        ];
        let gr = (opt(gl, "radius", 12.0) - ins).max(2.0);
        let gb = cv.map(|x, y| rounded(x, y, gb_box, gr));
        let a = cv.vgrad1(
            num(&gl["alpha"][0])?,
            num(&gl["alpha"][1])?,
            y0 + ins * 0.7,
            y0 + h,
        );
        let gc = cv.cover(&gb);
        let al: F = (0..gc.len()).map(|i| gc[i] * a[i] * fc[i]).collect();
        cv.over1(WHITE, &al);
    }
    if let Some(tl) = m.get("toplight") {
        let rg = ring(cv, &face_d, opt(tl, "width", 1.0));
        let (dp, a) = (key(tl, "depth")?, key(tl, "alpha")?);
        let al = zip(&rg, &cv.y, |r, y| r * clamp((y0 + dp - y) / dp) * a);
        cv.over1(WHITE, &al);
    }
    if let Some(hl) = m.get("hairline") {
        let rg = ring(cv, &face_d, opt(hl, "width", 1.0));
        let a = key(hl, "alpha")?;
        let col = c.get(hl.get("color").unwrap_or(&json!("#ffffff")))?;
        cv.over1(col, &rg.iter().map(|&r| r * a).collect::<F>());
    }
    if let Some(sh) = m.get("shade")
        && !sunk
    {
        let fr = (y1 - y0) * key(sh, "frac")?;
        let a = key(sh, "alpha")?;
        let al = zip(&fc, &cv.y, |f, y| f * clamp((y - (y1 - fr)) / fr) * a);
        cv.over1(BLACK, &al);
    }
    if let Some(sk) = m.get("sunk")
        && sunk
    {
        let (dp, a) = (key(sk, "depth")?, key(sk, "alpha")?);
        let al = zip(&fc, &cv.y, |f, y| f * clamp((y0 + dp - y) / dp) * a);
        cv.over1(BLACK, &al);
        let lp = opt(sk, "lip", 0.0);
        let rg = ring(cv, d, 1.0);
        let al = zip(&rg, &cv.y, |r, y| r * clamp((y - (y1 - 10.0)) / 10.0) * lp);
        cv.over1(WHITE, &al);
    }
    Ok(face_d)
}

#[allow(clippy::too_many_arguments)]
fn piece(
    cv: &mut Canvas,
    c: &mut Colors,
    m: &Value,
    shape: &Shape,
    role: &str,
    b: [f64; 4],
    face: (Rgba, Rgba),
    lip_h: f64,
    sunk: bool,
    drop: bool,
    orn: Option<&Value>,
    scale_cut: f64,
) -> Result<F, String> {
    let [x0, y0, x1, y1] = b;
    (c.top, c.bottom) = face;
    let mut body = shape.dist(cv, b, role, scale_cut);
    if let Some(dr) = m.get("drop")
        && drop
    {
        let col = c.get(dr.get("color").unwrap_or(&json!("shadow")))?;
        shadow(
            cv,
            &body,
            key(dr, "dy")?,
            key(dr, "blur")?,
            key(dr, "alpha")?,
            col,
        );
    }
    if let Some(gw) = m.get("glow")
        && drop
    {
        glow(
            cv,
            &body,
            key(gw, "blur")?,
            key(gw, "alpha")?,
            c.get(&gw["color"])?,
        );
    }
    if let Some(ol) = m.get("outline") {
        let col = c.get(ol.get("color").unwrap_or(&json!("outline")))?;
        let cov = cv.cover(&add(&body, -key(ol, "width")?));
        cv.over1(col, &cov);
    }
    let mut fb = b;
    if lip_h > 0.0 {
        let lp = &m["lip"];
        let g = cv.vgrad(
            c.get(&lp["top"])?,
            c.get(&lp["bottom"])?,
            y1 - lip_h - 2.0,
            y1,
        );
        let cov = cv.cover(&body);
        cv.over(&g, &cov);
        fb = [x0, y0, x1, y1 - lip_h];
        body = shape.dist(cv, fb, role, scale_cut);
    }
    let fd = surface(cv, c, m, &body, fb, sunk, shape.span(cv))?;
    if let Some(o) = orn {
        ornament(cv, c, &fd, o, fb)?;
    }
    Ok(fd)
}

// --------------------------------------------------------------- ornaments

fn stroke(px: f64, py: f64, a: (f64, f64), b: (f64, f64), w: f64) -> f64 {
    let (px, py) = (px - a.0, py - a.1);
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let t = clamp((px * vx + py * vy) / (vx * vx + vy * vy));
    let (dx, dy) = (px - vx * t, py - vy * t);
    clamp(0.5 + (w / 2.0 - (dx * dx + dy * dy).sqrt()) * 4.0)
}

fn b2f(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

fn ornament(cv: &mut Canvas, c: &Colors, d: &[f64], o: &Value, b: [f64; 4]) -> Result<(), String> {
    let kind = o["kind"].as_str().unwrap_or("none");
    let col = c.get(o.get("color").unwrap_or(&json!("orn")))?;
    let a = opt(o, "alpha", 1.0);
    let size = opt(o, "size", 10.0);
    let [x0, y0, x1, y1] = b;
    let inside = cv.cover(d);
    let inner = add(d, opt(o, "inset", 4.0));
    let corners = [
        (x0, y0, 1.0, 1.0),
        (x1, y0, -1.0, 1.0),
        (x1, y1, -1.0, -1.0),
        (x0, y1, 1.0, -1.0),
    ];
    let n = d.len();
    match kind {
        "hairline" => {
            let rg = ring(cv, &inner, 0.75);
            cv.over1(col, &rg.iter().map(|&r| r * a).collect::<F>());
        }
        "stitch" => {
            // Dashes drawn on the finished image, so they run on along the
            // band, and the piece tiles its band instead of stretching it:
            // the period fits a whole number of dashes per band. x - y round
            // the top-left and bottom-right corners, x + y round the others:
            // each runs across its corners' tangent, so no corner turns solid.
            let rg = cv.whole(&ring(cv, &inner, 0.9));
            let band = cv.band.0.max(1) as f64;
            let period = band / (band / opt(o, "period", 3.0) + 0.5).floor().max(1.0);
            let (sx, sy) = (cv.whole(&cv.x), cv.whole(&cv.y));
            let (px, py) = cv.place();
            let al: F = (0..rg.len())
                .map(|i| {
                    let back = (sx[i] - (x0 + x1) / 2.0) * (sy[i] - (y0 + y1) / 2.0) > 0.0;
                    let (x, y) = (px[i], py[i]);
                    let t = (if back { x - y } else { x + y }) / period;
                    rg[i] * b2f((t - t.floor()) < 0.55) * a
                })
                .collect();
            cv.over_whole(col, &al);
            cv.tiled = true;
        }
        "brackets" => {
            for (cx, cy, sx, sy) in corners {
                let al: F = (0..n)
                    .map(|i| {
                        let (px, py) = ((cv.x[i] - cx) * sx, (cv.y[i] - cy) * sy);
                        let arm = ((3.0..3.0 + size).contains(&px) && (3.0..4.5).contains(&py))
                            || ((3.0..3.0 + size).contains(&py) && (3.0..4.5).contains(&px));
                        b2f(arm) * inside[i] * a
                    })
                    .collect();
                cv.over1(col, &al);
            }
        }
        "studs" => {
            for (cx, cy, sx, sy) in corners {
                let uv = |i: usize| (cv.x[i] - (cx + sx * 6.5), cv.y[i] - (cy + sy * 6.5));
                let r: F = (0..n)
                    .map(|i| {
                        let (ux, uy) = uv(i);
                        (ux * ux + uy * uy).sqrt()
                    })
                    .collect();
                let v: F = (0..n)
                    .map(|i| {
                        let (ux, uy) = uv(i);
                        let (vx, vy) = (ux + 0.6, uy + 0.6);
                        (vx * vx + vy * vy).sqrt()
                    })
                    .collect();
                cv.over1(
                    BLACK,
                    &(0..n)
                        .map(|i| clamp(2.6 - r[i]) * 0.5 * inside[i])
                        .collect::<F>(),
                );
                cv.over1(
                    col,
                    &(0..n)
                        .map(|i| clamp(2.0 - r[i]) * a * inside[i])
                        .collect::<F>(),
                );
                cv.over1(
                    WHITE,
                    &(0..n)
                        .map(|i| clamp(0.9 - v[i]) * 0.7 * inside[i])
                        .collect::<F>(),
                );
            }
        }
        "filigree" => {
            let rg = ring(cv, &inner, 0.75);
            cv.over1(col, &rg.iter().map(|&r| r * 0.5 * a).collect::<F>());
            for (cx, cy, sx, sy) in corners {
                let (xs, ys) = (cv.x.clone(), cv.y.clone());
                let pp = |i: usize| ((xs[i] - cx) * sx, (ys[i] - cy) * sy);
                let al: F = (0..n)
                    .map(|i| {
                        let (px, py) = pp(i);
                        clamp(3.2 - ((px - 9.0).abs() + (py - 9.0).abs())) * a
                    })
                    .collect();
                cv.over1(col, &al);
                let al: F = (0..n)
                    .map(|i| {
                        let (px, py) = pp(i);
                        let (ux, uy) = (px - 9.0, py - 9.0);
                        let curl = ((ux * ux + uy * uy).sqrt() - 6.5).abs();
                        clamp(0.6 - curl) * b2f(px < 9.0 || py < 9.0) * 0.9 * a
                    })
                    .collect();
                cv.over1(col, &al);
            }
        }
        "runes" => {
            let rg = ring(cv, &inner, 0.6);
            cv.over1(col, &rg.iter().map(|&r| r * 0.35 * a).collect::<F>());
            for (cx, cy, sx, sy) in corners {
                let al: F = (0..n)
                    .map(|i| {
                        let (px, py) = ((cv.x[i] - cx) * sx, (cv.y[i] - cy) * sy);
                        let m = stroke(px, py, (7.0, 6.0), (7.0, 13.0), 1.1)
                            + stroke(px, py, (7.0, 9.0), (10.5, 6.5), 1.1)
                            + stroke(px, py, (7.0, 9.0), (10.5, 12.0), 1.1);
                        clamp(m) * a * inside[i]
                    })
                    .collect();
                cv.over1(col, &al);
            }
        }
        "brush-corner" => {
            for (cx, cy, sx, sy) in [corners[0], corners[2]] {
                let al: F = (0..n)
                    .map(|i| {
                        let (px, py) = ((cv.x[i] - cx) * sx, (cv.y[i] - cy) * sy);
                        let along = clamp(1.0 - (px + py - 6.0) / (2.0 * size));
                        let m = stroke(
                            px,
                            py,
                            (4.0, 4.0 + size),
                            (4.0 + size, 4.0),
                            0.5 + 3.0 * along,
                        );
                        let m = m * clamp((px + py - 4.0) * 2.0);
                        m * a * inside[i]
                    })
                    .collect();
                cv.over1(col, &al);
            }
        }
        "notch" => {
            let c1 = cv.cover(&add(d, 1.0));
            let al: F = (0..n)
                .map(|i| {
                    let (x, y) = (cv.x[i] - (x0 + 3.0), cv.y[i] - (y0 + 3.0));
                    b2f(x + y < size * 0.7 && x >= 0.0 && y >= 0.0) * c1[i] * a
                })
                .collect();
            cv.over1(col, &al);
        }
        "bar" => {
            let inset = size.min((y1 - y0) * 0.3);
            let (m0, m1) = (y0 + inset, y1 - inset);
            let al: F = (0..n)
                .map(|i| {
                    let (x, y) = (cv.x[i], cv.y[i]);
                    let cov =
                        clamp((x - (x0 + 2.5)) * 4.0 + 0.5) * clamp((x0 + 4.0 - x) * 4.0 + 0.5);
                    let cov = cov * clamp((y - m0) * 4.0 + 0.5) * clamp((m1 - y) * 4.0 + 0.5);
                    cov * a
                })
                .collect();
            cv.over1(col, &al);
        }
        "tab" => {
            let al: F = (0..n)
                .map(|i| {
                    let (x, y) = (cv.x[i], cv.y[i]);
                    let cov = clamp((x - (x0 + 10.0)) * 4.0 + 0.5)
                        * clamp((x0 + 10.0 + size * 2.4 - x) * 4.0 + 0.5);
                    let cov = cov * clamp((y - y0) * 4.0 + 0.5) * clamp((y0 + 1.5 - y) * 4.0 + 0.5);
                    cov * a
                })
                .collect();
            cv.over1(col, &al);
        }
        _ => return Err(format!("unknown ornament {kind}")),
    }
    Ok(())
}

// ----------------------------------------------------------- the pieces

/// Every piece of `spec`, in the Python's order.
pub fn draw(spec: &Value) -> Result<Vec<Piece>, String> {
    let si = spec.get("scale").and_then(Value::as_u64).unwrap_or(4) as usize;
    let s = si as f64;
    let mut c = Colors {
        palette: &spec["palette"],
        top: [0.0; 4],
        bottom: [0.0; 4],
    };
    let m = &spec["material"];
    if let Some(o) = m.as_object() {
        for k in o.keys() {
            if !MATERIAL_KEYS.contains(&k.as_str()) && k != "about" {
                return Err(format!("material: unknown key {k}"));
            }
        }
    }
    let sh = &spec["shape"];
    let empty = json!({});
    let shape = Shape {
        family: sh
            .get("family")
            .and_then(Value::as_str)
            .unwrap_or("rounded"),
        radii: sh.get("radius").unwrap_or(&empty),
        cuts: sh.get("cut").unwrap_or(&Value::Null),
        step: opt(sh, "step", 1.0),
        wobble: opt(sh, "wobble", 0.0),
        seed: sh.get("seed").and_then(Value::as_i64).unwrap_or(7),
    };
    let orns = spec.get("ornament").unwrap_or(&empty);
    let orn = |role: &str| {
        orns.get(role)
            .filter(|o| o.get("kind").and_then(Value::as_str).unwrap_or("none") != "none")
    };
    let lip = m.get("lip").map(|l| opt(l, "height", 0.0)).unwrap_or(0.0);
    let press = m.get("press").map(|p| opt(p, "dy", 2.0)).unwrap_or(2.0);
    let hover = m
        .get("hover")
        .cloned()
        .unwrap_or(json!({"toward": "#ffffff", "t": 0.12}));
    let mut out = Vec::new();
    let put = |out: &mut Vec<Piece>, name: &str, cv: &Canvas, fill: bool, tint: bool| {
        let [t, r, b, l] = cv.insets;
        let mut e = json!({ "src": format!("art/{name}.png"), "slice": [t * si, r * si, b * si, l * si], "width": [t, r, b, l] });
        if !fill {
            e["fill"] = json!(false);
        }
        if tint {
            e["tint"] = json!(true);
        }
        if cv.tiled {
            e["repeat"] = json!("tile");
        }
        let (width, height, rgba) = cv.bytes();
        out.push(Piece {
            name: name.to_owned(),
            width,
            height,
            rgba,
            entry: e,
        });
    };
    let mg = 2.5;
    let cr = shape.corner("control");
    let bi = [
        12f64.max(cr + 4.0),
        14f64.max(cr + 6.0),
        12f64.max(cr + 4.0) + lip,
        14f64.max(cr + 6.0),
    ];
    for (name, role) in [
        ("button", "button"),
        ("button-primary", "primary"),
        ("button-danger", "danger"),
    ] {
        for st in ["", "-hover", "-pressed"] {
            let mut cv = Canvas::new(si, bi, (4, 4));
            let (mut top, mut bottom) = c.pair(role)?;
            if st == "-hover" {
                let tw = c.get(&hover["toward"])?;
                let t = key(&hover, "t")?;
                (top, bottom) = (mix(top, tw, t), mix(bottom, tw, t * 0.7));
            }
            let pressed = st == "-pressed";
            let (l2, y_top) = if lip > 0.0 {
                let l2 = if pressed { (lip - press).max(1.0) } else { lip };
                (l2, mg + (lip - l2))
            } else {
                (0.0, mg + if pressed { press * 0.4 } else { 0.0 })
            };
            let b = [mg, y_top, cv.wf() - mg, cv.hf() - mg];
            piece(
                &mut cv,
                &mut c,
                m,
                &shape,
                "control",
                b,
                (top, bottom),
                l2,
                false,
                !pressed,
                orn("button"),
                1.0,
            )?;
            put(&mut out, &format!("{name}{st}"), &cv, true, false);
        }
    }
    let mut cv = Canvas::new(si, bi, (4, 4));
    let b = [mg, mg, cv.wf() - mg, cv.hf() - mg];
    let f = c.pair("disabled")?;
    piece(
        &mut cv, &mut c, m, &shape, "control", b, f, lip, false, true, None, 1.0,
    )?;
    put(&mut out, "button-disabled", &cv, true, false);

    let pc = 24f64.max(shape.corner("panel") + 12.0);
    let head = spec.get("panel").and_then(|p| p.get("header"));
    let hh = head.map(|h| opt(h, "height", 0.0)).unwrap_or(0.0);
    let mut cv = Canvas::new(si, [pc + hh, pc, pc + lip * 0.5, pc], (4, 4));
    let b = [3.0, 3.0, cv.wf() - 3.0, cv.hf() - 3.0];
    let f = c.pair("panel")?;
    let fd = piece(
        &mut cv,
        &mut c,
        m,
        &shape,
        "panel",
        b,
        f,
        lip * 0.5,
        false,
        true,
        orn("panel"),
        1.6,
    )?;
    if let Some(h) = head {
        let fc = cv.cover(&fd);
        let band = zip(&fc, &cv.y, |f, y| f * clamp((3.0 + hh - y) * s));
        let hc = c.get(&h["color"])?;
        let g = cv.vgrad(mix(hc, WHITE, opt(h, "lift", 0.15)), hc, 3.0, 3.0 + hh);
        cv.over(&g, &band);
        if let Some(rule) = h.get("rule").filter(|r| !r.is_null()) {
            let al = zip(&fc, &cv.y, |f, y| {
                f * clamp(1.0 - (y - (3.0 + hh - 0.5)).abs() / 0.9)
            });
            cv.over1(c.get(rule)?, &al);
        }
    }
    put(&mut out, "panel", &cv, true, false);

    let pr = shape.corner("control").min(14.0);
    let pi = 12f64.max(pr + 4.0);
    let mut cv = Canvas::new(si, [pi, pi + 2.0, pi, pi + 2.0], (4, 4));
    let b = [2.0, 2.0, cv.wf() - 2.0, cv.hf() - 2.0 - lip * 0.5];
    let f = c.pair("plate")?;
    piece(
        &mut cv,
        &mut c,
        m,
        &shape,
        "control",
        b,
        f,
        lip * 0.5,
        false,
        true,
        orn("plate"),
        1.0,
    )?;
    put(&mut out, "plate", &cv, true, false);

    let ci = 9f64.max(shape.corner("chip") + 3.0);
    let mut cv = Canvas::new(si, [ci, ci + 1.0, ci, ci + 1.0], (4, 4));
    let b = [2.0, 2.0, cv.wf() - 2.0, cv.hf() - 2.0 - lip * 0.4];
    let f = c.pair("chip")?;
    piece(
        &mut cv,
        &mut c,
        m,
        &shape,
        "chip",
        b,
        f,
        lip * 0.4,
        false,
        false,
        orn("chip"),
        0.6,
    )?;
    put(&mut out, "chip", &cv, true, false);

    let sl = 16f64.max(shape.corner("slot") + 6.0);
    let mut cv = Canvas::new(si, [sl; 4], (4, 4));
    let b = [2.0, 2.0, cv.wf() - 2.0, cv.hf() - 2.0];
    let f = c.pair("slot")?;
    piece(
        &mut cv,
        &mut c,
        m,
        &shape,
        "slot",
        b,
        f,
        0.0,
        true,
        false,
        orn("slot"),
        0.8,
    )?;
    put(&mut out, "slot", &cv, true, false);
    let fi = 12f64.max(shape.corner("control").min(10.0) + 4.0);
    let mut cv = Canvas::new(si, [fi; 4], (4, 4));
    let b = [2.0, 2.0, cv.wf() - 2.0, cv.hf() - 2.0];
    let f = c.pair("field")?;
    piece(
        &mut cv, &mut c, m, &shape, "control", b, f, 0.0, true, false, None, 0.8,
    )?;
    put(&mut out, "field", &cv, true, false);

    let mut cv = Canvas::new(si, [4.0, 64.0, 4.0, 64.0], (4, 0));
    let w = cv.wf();
    let ends = cv.map(|x, _| {
        let e = clamp(x.min(w - x) / 56.0);
        e * e * (3.0 - 2.0 * e)
    });
    let line = cv.map(|_, y| clamp(1.0 - (y - 4.0).abs() / 0.6));
    let under = cv.map(|_, y| clamp(1.0 - (y - 5.2).abs() / 0.6));
    cv.over1(c.named("shadow")?, &zip(&under, &ends, |u, e| u * e * 0.4));
    let dc = c.get(spec.get("divider").unwrap_or(&json!("orn")))?;
    cv.over1(dc, &zip(&line, &ends, |l, e| l * e));
    put(&mut out, "divider", &cv, true, false);

    // badge: the theme's shape and material, ringed in its badge rim
    let mut cv = Canvas::new(si, [13.0, 14.0, 14.0, 14.0], (2, 0));
    let ow = m
        .get("outline")
        .map(|o| opt(o, "width", 0.0))
        .unwrap_or(1.0);
    let mut bm = m.clone();
    let rim_c = spec.get("badge_rim").cloned().unwrap_or(json!("#ffffff"));
    bm["outline"] = json!({ "color": rim_c, "width": shape.whole(ow.max(1.5)) });
    let face = if spec["palette"].get("badge").is_some() {
        c.pair("badge")?
    } else {
        (hexc("#d83834")?, hexc("#a00c1a")?)
    };
    let bo = shape.whole(ow.max(1.5)) + 0.5;
    let b = [bo, bo, cv.wf() - bo, cv.hf() - bo];
    piece(
        &mut cv, &mut c, &bm, &shape, "badge", b, face, 0.0, false, true, None, 0.6,
    )?;
    put(&mut out, "badge", &cv, true, false);

    // tint pieces, lit as the material lights an edge
    let (hard, metal) = (m.get("bevel").is_some(), m.get("rim").is_some());
    let mut cv = Canvas::new(si, [sl; 4], (4, 4));
    let b = [2.0, 2.0, cv.wf() - 2.0, cv.hf() - 2.0];
    let body = shape.dist(&cv, b, "slot", 0.8);
    let rw = 2.25f64.max(ow + 0.75);
    let rg = ring(&cv, &body, rw);
    let glen = if hard { 3.0 } else { 7.0 };
    let ci2 = cv.cover(&add(&body, rw));
    let inner = zip(&body, &ci2, |b, c| clamp(1.0 + (b + rw) / glen) * c);
    let l = lit(&cv, &body, shape.span(&cv));
    let tone = |l: f64, soft: (f64, f64), strong: (f64, f64)| {
        if hard {
            soft.0 + soft.1 * lit_step(l)
        } else if metal {
            strong.0 + strong.1 * l
        } else {
            soft.0 + soft.1 * l
        }
    };
    let grey = Col::Each(
        l.iter()
            .map(|&l| {
                let g = if hard {
                    0.7 + 0.3 * lit_step(l)
                } else {
                    tone(l, (0.78, 0.22), (0.6, 0.4))
                };
                [g, g, g, 1.0]
            })
            .collect(),
    );
    cv.over(&grey, &zip(&rg, &inner, |r, i| clamp(r + i * 0.45)));
    put(&mut out, "slot-rim", &cv, false, true);

    let mut cv = Canvas::new(si, [ci, ci + 1.0, ci, ci + 1.0], (4, 4));
    let b = [1.0, 1.0, cv.wf() - 1.0, cv.hf() - 1.0];
    let body = shape.dist(&cv, b, "chip", 0.6);
    let cb = cv.cover(&body);
    cv.over1([0.16, 0.16, 0.16, 1.0], &cb);
    let bw = if hard {
        key(&m["bevel"], "width")?
    } else {
        1.0
    };
    let l = lit(&cv, &body, shape.span(&cv));
    let rim = Col::Each(
        l.iter()
            .map(|&l| {
                let r = tone(l, (0.17, 0.36), (0.12, 0.5));
                [r, r, r, 1.0]
            })
            .collect(),
    );
    let c1 = cv.cover(&add(&body, 1.0));
    cv.over(&rim, &c1);
    let h = cv.hf();
    let g = cv.vgrad(
        [0.33, 0.33, 0.33, 1.0],
        [0.24, 0.24, 0.24, 1.0],
        3.0,
        h - 3.0,
    );
    let c2 = cv.cover(&add(&body, 1.0 + bw));
    cv.over(&g, &c2);
    put(&mut out, "chip-bevel", &cv, true, true);
    Ok(out)
}

/// The spec's cache key: a hash of its canonical JSON and the generator's version.
pub fn spec_key(spec: &Value) -> String {
    let mut h = blake3::Hasher::new();
    h.update(b"ui_art 1\n");
    h.update(serde_json::to_string(spec).unwrap_or_default().as_bytes());
    h.finalize().to_hex()[..32].to_owned()
}

/// Draws `spec` into `<cache>/<key>/` (its PNGs under `art/` and
/// `images.json`) unless that directory is there already, and returns it.
pub fn draw_cached(spec: &Value, cache: &Path) -> Result<PathBuf, String> {
    let dir = cache.join(spec_key(spec));
    if dir.join("images.json").is_file() {
        return Ok(dir);
    }
    let tmp = cache.join(format!("{}.tmp{}", spec_key(spec), std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("art")).map_err(|e| format!("{}: {e}", tmp.display()))?;
    let mut images = serde_json::Map::new();
    for p in draw(spec)? {
        let path = tmp.join("art").join(format!("{}.png", p.name));
        image::RgbaImage::from_raw(p.width, p.height, p.rgba)
            .ok_or("ui art: a piece's bytes do not match its size")?
            .save(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        images.insert(p.name, p.entry);
    }
    std::fs::write(
        tmp.join("images.json"),
        serde_json::to_string_pretty(&images).unwrap_or_default(),
    )
    .map_err(|e| e.to_string())?;
    match std::fs::rename(&tmp, &dir) {
        Ok(()) => Ok(dir),
        // another cook drew the same spec first
        Err(_) if dir.join("images.json").is_file() => {
            let _ = std::fs::remove_dir_all(&tmp);
            Ok(dir)
        }
        Err(e) => Err(format!("{}: {e}", dir.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fnv(data: &[u8]) -> String {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in data {
            h = (h ^ b as u64).wrapping_mul(0x100000001b3);
        }
        format!("{h:016x}")
    }

    /// Every piece of every reference spec has the bytes the Python spec
    /// draws (`theme-art.py --parity` wrote the digests), and every
    /// stretched row and column of it is the same.
    ///
    /// Ways this could fail: an operation reordered or fused (one rounding
    /// apart flips a byte), a default that differs from the Python's, a
    /// piece whose stretch band varies.
    #[test]
    fn the_port_draws_the_python_specs_bytes() {
        let specs: Value = serde_json::from_str(include_str!(
            "../../../../templates/tools/theme-art-specs.json"
        ))
        .unwrap();
        let want: Value =
            serde_json::from_str(include_str!("../../tests/ui_art_parity.json")).unwrap();
        let mut wrong = Vec::new();
        for (name, spec) in specs.as_object().unwrap() {
            let pieces = draw(spec).unwrap();
            assert_eq!(
                pieces.len(),
                want[name].as_object().unwrap().len(),
                "{name}"
            );
            for p in pieces {
                let mut bytes = p.width.to_le_bytes().to_vec();
                bytes.extend(p.height.to_le_bytes());
                bytes.extend(&p.rgba);
                if want[name][&p.name] != fnv(&bytes) {
                    wrong.push(format!("{name}/{}", p.name));
                }
                // the stretch band: identical columns, identical rows (a
                // tiled piece varies along it, and tiles it)
                if p.entry["repeat"] == "tile" {
                    continue;
                }
                let w = p.width as usize;
                let [t, _, _, l] =
                    [0, 1, 2, 3].map(|i| p.entry["slice"][i].as_u64().unwrap() as usize);
                let px = |x: usize, y: usize| &p.rgba[(y * w + x) * 4..(y * w + x) * 4 + 4];
                for y in 0..p.height as usize {
                    assert_eq!(px(l, y), px(l + 1, y), "{name}/{} column", p.name);
                }
                if p.entry["slice"][0].as_u64().unwrap() as usize
                    + p.entry["slice"][2].as_u64().unwrap() as usize
                    + 1
                    < p.height as usize
                {
                    for x in 0..w {
                        assert_eq!(px(x, t), px(x, t + 1), "{name}/{} row", p.name);
                    }
                }
            }
        }
        assert!(wrong.is_empty(), "differ from the Python: {wrong:?}");
    }

    /// Every value of the theme dimension data (`templates/themes/dimensions/`)
    /// names files that exist and are pinned, fonts by the family names the
    /// files themselves give, and only keys the default pack declares; every
    /// material, shape and ornament draws through the generator.
    ///
    /// Ways this could fail: a family typed as it is spelt on a website
    /// rather than in the file (tactical's display face never applied), a
    /// clip or texture renamed or never pinned, a token or slot name the
    /// pack does not have, an art layer the generator does not know.
    #[test]
    fn the_theme_dimensions_name_real_things_and_draw() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let templates = root.join("templates");
        let read = |p: &Path| -> Value {
            serde_json::from_str(&std::fs::read_to_string(p).unwrap())
                .unwrap_or_else(|e| panic!("{}: {e}", p.display()))
        };
        let pack = read(&root.join("crates/og-ui-next/default/pack.json"));
        let pinned: Vec<String> = read(&templates.join("og-assets.json"))["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["path"].as_str().unwrap().to_owned())
            .collect();
        let mut bad = Vec::new();
        let file = |bad: &mut Vec<String>, at: &str, name: &str| -> Option<PathBuf> {
            let rel = name.strip_prefix('@').unwrap_or(name);
            let p = templates.join(rel);
            if !pinned.iter().any(|f| f == rel) {
                bad.push(format!("{at}: {name} is not pinned"));
            }
            if p.exists() {
                Some(p)
            } else {
                bad.push(format!(
                    "{at}: {name} is missing (bun scripts/assets.ts ensure templates)"
                ));
                None
            }
        };
        let keys = |v: &Value| -> Vec<String> {
            v.as_object()
                .map(|o| o.keys().cloned().collect())
                .unwrap_or_default()
        };
        let dims = templates.join("themes/dimensions");
        let values = |dim: &str| -> Vec<(String, Value)> {
            let mut out: Vec<_> = std::fs::read_dir(dims.join(dim))
                .unwrap()
                .map(|e| e.unwrap().path())
                .map(|p| {
                    (
                        p.file_stem().unwrap().to_string_lossy().into_owned(),
                        read(&p),
                    )
                })
                .collect();
            out.sort_by(|a, b| a.0.cmp(&b.0));
            out
        };
        let unknown = |bad: &mut Vec<String>, at: &str, got: &Value, of: &Value| {
            for k in keys(got) {
                if of.get(&k).is_none() {
                    bad.push(format!("{at}: the default pack has no `{k}`"));
                }
            }
        };

        // type: fonts exist, and the families named are the files' own
        for (name, v) in values("type") {
            let at = format!("type/{name}");
            let mut fonts = og_text::fonts::Collection::new();
            let mut have = Vec::new();
            for f in v["fonts"].as_array().unwrap() {
                let Some(p) = file(&mut bad, &at, f.as_str().unwrap()) else {
                    continue;
                };
                let id = fonts
                    .add(std::sync::Arc::new(std::fs::read(&p).unwrap()))
                    .unwrap();
                have.push(fonts.family_name(id).unwrap().to_owned());
            }
            let named = v["families"]
                .as_object()
                .unwrap()
                .values()
                .chain(v.get("cjk"));
            for family in named.filter_map(Value::as_str) {
                if !have.iter().any(|h| h == family) {
                    bad.push(format!(
                        "{at}: no font is family \"{family}\" (the files: {have:?})"
                    ));
                }
            }
            unknown(
                &mut bad,
                &at,
                &v["fontFamily"],
                &pack["theme"]["fontFamily"],
            );
            unknown(&mut bad, &at, &v["tracking"], &pack["theme"]["tracking"]);
        }
        for (name, v) in values("sound") {
            let at = format!("sound/{name}");
            unknown(&mut bad, &at, &v["sounds"], &pack["sounds"]);
            for cue in v["sounds"].as_object().unwrap().values() {
                for c in cue["clips"].as_array().into_iter().flatten() {
                    file(&mut bad, &at, c.as_str().unwrap());
                }
            }
        }
        for (name, v) in values("effects") {
            for t in v["textures"].as_array().unwrap() {
                file(&mut bad, &format!("effects/{name}"), t.as_str().unwrap());
            }
        }
        for (name, v) in values("icons") {
            let at = format!("icons/{name}");
            let set = v["set"].as_str().unwrap();
            if !templates.join(set.trim_start_matches('@')).exists() {
                bad.push(format!("{at}: no icon set {set}"));
            }
            unknown(&mut bad, &at, &v["colors"], &pack["theme"]["colors"]);
        }
        let slots: Vec<String> = keys(&values("motion")[0].1["variants"]);
        for (name, v) in values("motion") {
            let at = format!("motion/{name}");
            unknown(&mut bad, &at, &v["variants"], &pack["variants"]);
            if keys(&v["variants"]) != slots {
                bad.push(format!("{at}: sets other motion slots than {slots:?}"));
            }
        }
        for (name, v) in values("shape") {
            if keys(&v["radius"]) != keys(&pack["theme"]["radius"]) {
                bad.push(format!(
                    "shape/{name}: radius keys differ from the default pack's"
                ));
            }
        }

        // the art: each material, shape and ornament draws (with the parity
        // fixture's first palette standing in for a resolved colour)
        let specs: Value = serde_json::from_str(include_str!(
            "../../../../templates/tools/theme-art-specs.json"
        ))
        .unwrap();
        let base = specs.as_object().unwrap().values().next().unwrap().clone();
        let mut draws = |at: String, material: &Value, shape: &Value, ornament: Option<&Value>| {
            let mut spec = base.clone();
            spec["material"] = material.clone();
            spec["shape"] = shape.clone();
            spec["ornament"] = ornament.map_or(
                json!({}),
                |o| json!({"button": o, "panel": o, "plate": o, "slot": o, "chip": o}),
            );
            if let Err(e) = draw(&spec) {
                bad.push(format!("{at}: {e}"));
            }
        };
        let round = values("shape")
            .into_iter()
            .find(|s| s.0 == "round")
            .unwrap()
            .1;
        let flat = values("material")
            .into_iter()
            .find(|s| s.0 == "flat")
            .unwrap()
            .1;
        for (name, v) in values("material") {
            draws(format!("material/{name}"), &v["art"], &round["art"], None);
        }
        for (name, v) in values("shape") {
            draws(format!("shape/{name}"), &flat["art"], &v["art"], None);
        }
        for (name, v) in values("ornament") {
            draws(
                format!("ornament/{name}"),
                &flat["art"],
                &round["art"],
                Some(&v["art"]),
            );
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }
}
