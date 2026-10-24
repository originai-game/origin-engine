//! `og compare <reference.png> <render.png> [--mask <mask.png>] [--align]
//! [--key <ΔE00>] [--sheet <out.png>] [--json]`: the deterministic half of a
//! model review (the og-model skill). It reads a reference image and a frame
//! the probe captured (`screenshot`, `view`, `turntable`), and prints:
//!
//! - **silhouette IoU**: the object's pixels in both over the pixels in
//!   either. An image with transparency is the object where alpha ≥ 128;
//!   an opaque one is the object where a pixel is more than `--key`
//!   (ΔE00, default 8) from its background, the per-channel median of its
//!   border pixels (a render's clear colour, a photo's backdrop).
//! - **ΔE00 per region**: CIEDE2000 between the reference's and the
//!   render's colour (sRGB, D65) over the pixels both images cover, per
//!   region of `--mask` (an image the reference's size whose every opaque
//!   colour but black is one region, named by its hex), or over the whole
//!   overlap without one: mean and 95th percentile, and each region's own
//!   IoU.
//! - **the sheet**: one PNG, the reference beside the render (as compared)
//!   beside their silhouettes overlaid (both grey, reference only red,
//!   render only blue), labelled with the IoU and the overall ΔE00, so a
//!   reviewer judges one image. By default `<render>.sheet.png` beside the
//!   render.
//!
//! The render is compared in the reference's frame: resampled to its size
//! when the aspect ratios agree within 1 %, refused otherwise (render at the
//! reference's `--size`). `--align` instead scales the render's silhouette
//! box uniformly to the reference's box area and centres it there, for a
//! reference whose framing is not the view's; the framing then no longer
//! counts against the IoU. Nothing here reads a clock or a random number:
//! the same two files give the same numbers and the same sheet bytes.

use image::{Rgba, RgbaImage};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og compare <reference.png> <render.png> [--mask <mask.png>] [--align] [--key <ΔE00>] [--sheet <out.png>] [--json]";

/// What `og compare` was asked.
#[derive(Debug, PartialEq)]
struct Args {
    reference: PathBuf,
    render: PathBuf,
    mask: Option<PathBuf>,
    align: bool,
    key: f64,
    sheet: Option<PathBuf>,
    json: bool,
}

fn parse(args: &[&str]) -> Result<Args, String> {
    let mut paths = Vec::new();
    let (mut mask, mut sheet, mut align, mut json, mut key) = (None, None, false, false, 8.0);
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--align" => align = true,
            "--json" => json = true,
            flag @ ("--mask" | "--sheet" | "--key") => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| format!("{flag} needs a value. {USAGE}"))?;
                match flag {
                    "--mask" => mask = Some(PathBuf::from(v)),
                    "--sheet" => sheet = Some(PathBuf::from(v)),
                    _ => {
                        key = v
                            .parse()
                            .ok()
                            .filter(|k: &f64| k.is_finite() && *k > 0.0)
                            .ok_or_else(|| format!("--key {v}: a ΔE00 above 0, such as 8"))?
                    }
                }
                i += 1;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option `{other}`. {USAGE}"));
            }
            path => paths.push(PathBuf::from(path)),
        }
        i += 1;
    }
    let [reference, render] = <[PathBuf; 2]>::try_from(paths).map_err(|p| {
        format!(
            "two images, the reference and the render ({} given). {USAGE}",
            p.len()
        )
    })?;
    Ok(Args {
        reference,
        render,
        mask,
        align,
        key,
        sheet,
        json,
    })
}

pub fn run(args: &[&str]) -> ExitCode {
    let a = match parse(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("og compare: {e}");
            return ExitCode::from(2);
        }
    };
    match compare_files(&a) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og compare: {e}");
            ExitCode::FAILURE
        }
    }
}

fn load(path: &Path) -> Result<RgbaImage, String> {
    Ok(image::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .to_rgba8())
}

fn compare_files(a: &Args) -> Result<(), String> {
    let reference = load(&a.reference)?;
    let render = load(&a.render)?;
    let mask = a.mask.as_deref().map(load).transpose()?;
    let c = compare(&reference, &render, mask.as_ref(), a.align, a.key)?;
    let sheet_path = a.sheet.clone().unwrap_or_else(|| {
        let stem = a
            .render
            .file_stem()
            .map_or("render".into(), |s| s.to_string_lossy().into_owned());
        a.render.with_file_name(format!("{stem}.sheet.png"))
    });
    sheet(&reference, &c)
        .save(&sheet_path)
        .map_err(|e| format!("{}: {e}", sheet_path.display()))?;
    if a.json {
        let mut v = c.to_json();
        v["sheet"] = json!(sheet_path.display().to_string());
        println!("{v}");
    } else {
        print!("{}", c.report());
        println!("sheet {}", sheet_path.display());
    }
    Ok(())
}

// ── colour ──────────────────────────────────────────────────────────────────

/// CIE L*a*b* (D65) of an 8-bit sRGB colour.
pub fn lab(rgb: [u8; 3]) -> [f64; 3] {
    let lin = |c: u8| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (lin(rgb[0]), lin(rgb[1]), lin(rgb[2]));
    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;
    let f = |t: f64| {
        const E: f64 = 216.0 / 24389.0;
        const K: f64 = 24389.0 / 27.0;
        if t > E {
            t.cbrt()
        } else {
            (K * t + 16.0) / 116.0
        }
    };
    let (fx, fy, fz) = (f(x / 0.95047), f(y), f(z / 1.08883));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// CIEDE2000 colour difference (Sharma, Wu and Dalal 2005), kL = kC = kH = 1.
pub fn delta_e00(a: [f64; 3], b: [f64; 3]) -> f64 {
    use std::f64::consts::PI;
    let deg = |r: f64| r * 180.0 / PI;
    let rad = |d: f64| d * PI / 180.0;
    let [l1, a1, b1] = a;
    let [l2, a2, b2] = b;
    let c_bar = ((a1 * a1 + b1 * b1).sqrt() + (a2 * a2 + b2 * b2).sqrt()) / 2.0;
    let c7 = c_bar.powi(7);
    let g = 0.5 * (1.0 - (c7 / (c7 + 25f64.powi(7))).sqrt());
    let (a1p, a2p) = ((1.0 + g) * a1, (1.0 + g) * a2);
    let (c1p, c2p) = ((a1p * a1p + b1 * b1).sqrt(), (a2p * a2p + b2 * b2).sqrt());
    let hue = |b: f64, a: f64| {
        if b == 0.0 && a == 0.0 {
            0.0
        } else {
            deg(b.atan2(a)).rem_euclid(360.0)
        }
    };
    let (h1p, h2p) = (hue(b1, a1p), hue(b2, a2p));
    let dl = l2 - l1;
    let dc = c2p - c1p;
    let dh = if c1p * c2p == 0.0 {
        0.0
    } else if (h2p - h1p).abs() <= 180.0 {
        h2p - h1p
    } else if h2p - h1p > 180.0 {
        h2p - h1p - 360.0
    } else {
        h2p - h1p + 360.0
    };
    let dhh = 2.0 * (c1p * c2p).sqrt() * rad(dh / 2.0).sin();
    let l_bar = (l1 + l2) / 2.0;
    let cp_bar = (c1p + c2p) / 2.0;
    let h_bar = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= 180.0 {
        (h1p + h2p) / 2.0
    } else if h1p + h2p < 360.0 {
        (h1p + h2p + 360.0) / 2.0
    } else {
        (h1p + h2p - 360.0) / 2.0
    };
    let t = 1.0 - 0.17 * rad(h_bar - 30.0).cos()
        + 0.24 * rad(2.0 * h_bar).cos()
        + 0.32 * rad(3.0 * h_bar + 6.0).cos()
        - 0.20 * rad(4.0 * h_bar - 63.0).cos();
    let d_theta = 30.0 * (-((h_bar - 275.0) / 25.0).powi(2)).exp();
    let cp7 = cp_bar.powi(7);
    let rc = 2.0 * (cp7 / (cp7 + 25f64.powi(7))).sqrt();
    let l50 = (l_bar - 50.0).powi(2);
    let sl = 1.0 + 0.015 * l50 / (20.0 + l50).sqrt();
    let sc = 1.0 + 0.045 * cp_bar;
    let sh = 1.0 + 0.015 * cp_bar * t;
    let rt = -rad(2.0 * d_theta).sin() * rc;
    let (x, y, z) = (dl / sl, dc / sc, dhh / sh);
    (x * x + y * y + z * z + rt * y * z).sqrt()
}

// ── silhouettes and alignment ───────────────────────────────────────────────

fn rgb(p: &Rgba<u8>) -> [u8; 3] {
    [p[0], p[1], p[2]]
}

/// The per-channel median of the border pixels.
fn background(img: &RgbaImage) -> [u8; 3] {
    let (w, h) = img.dimensions();
    let mut border = Vec::new();
    for x in 0..w {
        border.push(rgb(img.get_pixel(x, 0)));
        border.push(rgb(img.get_pixel(x, h - 1)));
    }
    for y in 1..h.saturating_sub(1) {
        border.push(rgb(img.get_pixel(0, y)));
        border.push(rgb(img.get_pixel(w - 1, y)));
    }
    let mut out = [0u8; 3];
    for (c, o) in out.iter_mut().enumerate() {
        let mut v: Vec<u8> = border.iter().map(|p| p[c]).collect();
        v.sort_unstable();
        *o = v[v.len() / 2];
    }
    out
}

/// Which pixels are the object (row-major), and the background colour.
fn silhouette(img: &RgbaImage, key: f64) -> (Vec<bool>, [u8; 3]) {
    let bg = background(img);
    if img.pixels().any(|p| p[3] < 255) {
        return (img.pixels().map(|p| p[3] >= 128).collect(), bg);
    }
    let bg_lab = lab(bg);
    (
        img.pixels()
            .map(|p| delta_e00(lab(rgb(p)), bg_lab) > key)
            .collect(),
        bg,
    )
}

/// The box (x0, y0, x1, y1), exclusive ends, of the set pixels.
fn bbox(mask: &[bool], w: u32) -> Option<(f64, f64, f64, f64)> {
    let mut b: Option<(u32, u32, u32, u32)> = None;
    for (i, _) in mask.iter().enumerate().filter(|(_, m)| **m) {
        let (x, y) = (i as u32 % w, i as u32 / w);
        b = Some(match b {
            None => (x, y, x + 1, y + 1),
            Some((a, c, d, e)) => (a.min(x), c.min(y), d.max(x + 1), e.max(y + 1)),
        });
    }
    b.map(|(a, c, d, e)| (f64::from(a), f64::from(c), f64::from(d), f64::from(e)))
}

/// `src` resampled bilinearly into a `w`×`h` image, each output pixel
/// centre taken from `map(x, y)` in source pixel coordinates (centres at
/// .5); outside the source, `fill`.
fn warp(
    src: &RgbaImage,
    w: u32,
    h: u32,
    fill: Rgba<u8>,
    map: impl Fn(f64, f64) -> (f64, f64),
) -> RgbaImage {
    let (sw, sh) = src.dimensions();
    let mut out = RgbaImage::from_pixel(w, h, fill);
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = map(f64::from(x) + 0.5, f64::from(y) + 0.5);
            let (fx, fy) = (sx - 0.5, sy - 0.5);
            if fx < -0.5 || fy < -0.5 || fx > f64::from(sw) - 0.5 || fy > f64::from(sh) - 0.5 {
                continue;
            }
            let (x0, y0) = (fx.floor(), fy.floor());
            let (tx, ty) = (fx - x0, fy - y0);
            let at = |xi: f64, yi: f64| {
                let xi = xi.clamp(0.0, f64::from(sw - 1)) as u32;
                let yi = yi.clamp(0.0, f64::from(sh - 1)) as u32;
                *src.get_pixel(xi, yi)
            };
            let (a, b, c, d) = (
                at(x0, y0),
                at(x0 + 1.0, y0),
                at(x0, y0 + 1.0),
                at(x0 + 1.0, y0 + 1.0),
            );
            let mut p = [0u8; 4];
            for k in 0..4 {
                let top = f64::from(a[k]) * (1.0 - tx) + f64::from(b[k]) * tx;
                let bottom = f64::from(c[k]) * (1.0 - tx) + f64::from(d[k]) * tx;
                p[k] = (top * (1.0 - ty) + bottom * ty).round().clamp(0.0, 255.0) as u8;
            }
            out.put_pixel(x, y, Rgba(p));
        }
    }
    out
}

/// The render in the reference's frame (see the module doc).
fn in_frame(
    reference: &RgbaImage,
    ref_mask: &[bool],
    render: &RgbaImage,
    align: bool,
    key: f64,
) -> Result<RgbaImage, String> {
    let (w, h) = reference.dimensions();
    let (rw, rh) = render.dimensions();
    if align {
        let (mask, bg) = silhouette(render, key);
        let fill = Rgba([
            bg[0],
            bg[1],
            bg[2],
            if render.pixels().any(|p| p[3] < 255) {
                0
            } else {
                255
            },
        ]);
        let (Some(a), Some(b)) = (bbox(ref_mask, w), bbox(&mask, rw)) else {
            return Err("--align needs an object in both images; one of them is all background (raise or lower --key)".into());
        };
        let s = (((a.2 - a.0) * (a.3 - a.1)) / ((b.2 - b.0) * (b.3 - b.1))).sqrt();
        let (ca, cb) = (
            ((a.0 + a.2) / 2.0, (a.1 + a.3) / 2.0),
            ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0),
        );
        return Ok(warp(render, w, h, fill, |x, y| {
            ((x - ca.0) / s + cb.0, (y - ca.1) / s + cb.1)
        }));
    }
    if (rw, rh) == (w, h) {
        return Ok(render.clone());
    }
    let (ar, aa) = (f64::from(w) / f64::from(h), f64::from(rw) / f64::from(rh));
    if (ar / aa - 1.0).abs() > 0.01 {
        return Err(format!(
            "the render is {rw}x{rh} and the reference {w}x{h}: their shapes differ, so the framing cannot be compared; render at the reference's size (og run --size {w}x{h}) or pass --align"
        ));
    }
    let (sx, sy) = (f64::from(rw) / f64::from(w), f64::from(rh) / f64::from(h));
    Ok(warp(render, w, h, Rgba([0, 0, 0, 0]), |x, y| {
        (x * sx, y * sy)
    }))
}

// ── the comparison ──────────────────────────────────────────────────────────

/// One region's numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    /// `#rrggbb` of its mask colour, or `overlap` without a mask.
    pub name: String,
    /// Its pixels.
    pub pixels: usize,
    /// Its pixels the object covers in both images (where ΔE00 is read).
    pub both: usize,
    /// Its silhouette IoU.
    pub iou: f64,
    pub mean: f64,
    pub p95: f64,
}

/// What `og compare` measured.
#[derive(Clone, Debug)]
pub struct Comparison {
    pub width: u32,
    pub height: u32,
    pub iou: f64,
    pub reference_pixels: usize,
    pub render_pixels: usize,
    pub both_pixels: usize,
    /// ΔE00 over the whole overlap.
    pub mean: f64,
    pub p95: f64,
    pub regions: Vec<Region>,
    /// The render as compared (the reference's size).
    pub render: RgbaImage,
    pub ref_mask: Vec<bool>,
    pub render_mask: Vec<bool>,
}

fn iou(a: usize, b: usize, both: usize) -> f64 {
    let union = a + b - both;
    if union == 0 {
        1.0
    } else {
        both as f64 / union as f64
    }
}

fn stats(mut d: Vec<f64>) -> (f64, f64) {
    if d.is_empty() {
        return (0.0, 0.0);
    }
    let mean = d.iter().sum::<f64>() / d.len() as f64;
    d.sort_by(f64::total_cmp);
    let k = ((d.len() as f64 * 0.95).ceil() as usize).clamp(1, d.len()) - 1;
    (mean, d[k])
}

/// Compares `render` with `reference` (both RGBA8), regions from `mask`.
pub fn compare(
    reference: &RgbaImage,
    render: &RgbaImage,
    mask: Option<&RgbaImage>,
    align: bool,
    key: f64,
) -> Result<Comparison, String> {
    let (w, h) = reference.dimensions();
    if w == 0 || h == 0 || render.width() == 0 || render.height() == 0 {
        return Err("an image is empty".into());
    }
    let (ref_mask, _) = silhouette(reference, key);
    let framed = in_frame(reference, &ref_mask, render, align, key)?;
    let (render_mask, _) = silhouette(&framed, key);
    let n = (w * h) as usize;
    let delta: Vec<Option<f64>> = (0..n)
        .map(|i| {
            (ref_mask[i] && render_mask[i]).then(|| {
                let (x, y) = (i as u32 % w, i as u32 / w);
                delta_e00(
                    lab(rgb(reference.get_pixel(x, y))),
                    lab(rgb(framed.get_pixel(x, y))),
                )
            })
        })
        .collect();
    let count = |sel: &dyn Fn(usize) -> bool| -> (usize, usize, usize) {
        (0..n).filter(|&i| sel(i)).fold((0, 0, 0), |(a, b, c), i| {
            (
                a + ref_mask[i] as usize,
                b + render_mask[i] as usize,
                c + (ref_mask[i] && render_mask[i]) as usize,
            )
        })
    };
    let (ra, rb, both) = count(&|_| true);
    let (mean, p95) = stats(delta.iter().flatten().copied().collect());
    let mut regions = Vec::new();
    match mask {
        None => regions.push(Region {
            name: "overlap".into(),
            pixels: n,
            both,
            iou: iou(ra, rb, both),
            mean,
            p95,
        }),
        Some(m) => {
            let m = if m.dimensions() == (w, h) {
                m.clone()
            } else {
                let (mw, mh) = m.dimensions();
                if (f64::from(w) / f64::from(h) / (f64::from(mw) / f64::from(mh)) - 1.0).abs()
                    > 0.01
                {
                    return Err(format!(
                        "the mask is {mw}x{mh} and the reference {w}x{h}; draw the mask over the reference, its size"
                    ));
                }
                image::imageops::resize(m, w, h, image::imageops::FilterType::Nearest)
            };
            let label: Vec<Option<[u8; 3]>> = m
                .pixels()
                .map(|p| (p[3] >= 128 && rgb(p) != [0, 0, 0]).then(|| rgb(p)))
                .collect();
            let mut colours: Vec<[u8; 3]> = label.iter().flatten().copied().collect();
            colours.sort_unstable();
            colours.dedup();
            if colours.len() > 64 {
                return Err(format!(
                    "the mask has {} colours; a mask is flat regions (at most 64), drawn without antialiasing",
                    colours.len()
                ));
            }
            for c in colours {
                let inside = |i: usize| label[i] == Some(c);
                let (a, b, both) = count(&inside);
                let (mean, p95) = stats(
                    (0..n)
                        .filter(|&i| inside(i))
                        .filter_map(|i| delta[i])
                        .collect(),
                );
                regions.push(Region {
                    name: format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]),
                    pixels: (0..n).filter(|&i| inside(i)).count(),
                    both,
                    iou: iou(a, b, both),
                    mean,
                    p95,
                });
            }
        }
    }
    Ok(Comparison {
        width: w,
        height: h,
        iou: iou(ra, rb, both),
        reference_pixels: ra,
        render_pixels: rb,
        both_pixels: both,
        mean,
        p95,
        regions,
        render: framed,
        ref_mask,
        render_mask,
    })
}

impl Comparison {
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "width": self.width,
            "height": self.height,
            "iou": self.iou,
            "referencePixels": self.reference_pixels,
            "renderPixels": self.render_pixels,
            "bothPixels": self.both_pixels,
            "deltaE00": { "mean": self.mean, "p95": self.p95 },
            "regions": self.regions.iter().map(|r| json!({
                "name": r.name, "pixels": r.pixels, "both": r.both, "iou": r.iou,
                "deltaE00": { "mean": r.mean, "p95": r.p95 },
            })).collect::<Vec<_>>(),
        })
    }

    pub fn report(&self) -> String {
        let mut s = format!(
            "silhouette IoU {:.4} ({}x{}: reference {} px, render {} px, both {} px)\nΔE00 over the overlap: mean {:.2}, p95 {:.2}\n",
            self.iou,
            self.width,
            self.height,
            self.reference_pixels,
            self.render_pixels,
            self.both_pixels,
            self.mean,
            self.p95
        );
        s.push_str("region     pixels     both    IoU  ΔE00 mean   p95\n");
        for r in &self.regions {
            s.push_str(&format!(
                "{:<9} {:>7} {:>8} {:>6.4} {:>10.2} {:>5.2}\n",
                r.name, r.pixels, r.both, r.iou, r.mean, r.p95
            ));
        }
        s
    }
}

// ── the sheet ───────────────────────────────────────────────────────────────

/// The tallest a panel is drawn.
const PANEL_HEIGHT: u32 = 400;
const GAP: u32 = 8;
const LABEL: u32 = 22;
const SHEET_BG: Rgba<u8> = Rgba([18, 18, 22, 255]);

/// The reference, the render as compared and the silhouettes overlaid,
/// side by side, each labelled.
pub fn sheet(reference: &RgbaImage, c: &Comparison) -> RgbaImage {
    let (w, h) = (c.width, c.height);
    let overlay = RgbaImage::from_fn(w, h, |x, y| {
        let i = (y * w + x) as usize;
        Rgba(match (c.ref_mask[i], c.render_mask[i]) {
            (true, true) => [150, 150, 150, 255],
            (true, false) => [230, 50, 50, 255],
            (false, true) => [60, 110, 240, 255],
            (false, false) => [0, 0, 0, 255],
        })
    });
    let ph = h.min(PANEL_HEIGHT);
    let pw = ((u64::from(w) * u64::from(ph) / u64::from(h)) as u32).max(1);
    let panel = |img: &RgbaImage| {
        // Transparency over the sheet's mid grey, so a cut-out reads.
        let flat = RgbaImage::from_fn(img.width(), img.height(), |x, y| {
            let p = img.get_pixel(x, y);
            let a = u32::from(p[3]);
            let mix = |c: u8| ((u32::from(c) * a + 96 * (255 - a)) / 255) as u8;
            Rgba([mix(p[0]), mix(p[1]), mix(p[2]), 255])
        });
        if (pw, ph) == (w, h) {
            flat
        } else {
            image::imageops::resize(&flat, pw, ph, image::imageops::FilterType::Triangle)
        }
    };
    let panels = [
        (panel(reference), "REF".to_string()),
        (panel(&c.render), format!("DE00 {:.2}", c.mean)),
        (panel(&overlay), format!("IOU {:.4}", c.iou)),
    ];
    let mut out = RgbaImage::from_pixel(GAP + 3 * (pw + GAP), GAP + ph + LABEL + GAP, SHEET_BG);
    for (k, (img, label)) in panels.iter().enumerate() {
        let x = GAP + k as u32 * (pw + GAP);
        image::imageops::overlay(&mut out, img, i64::from(x), i64::from(GAP));
        crate::contact_sheet::text(&mut out, x + 2, GAP + ph + 4, label, 2);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sharma, Wu and Dalal's CIEDE2000 test data: pairs 1, 2, 3, 7, 17,
    /// 18, 25 and 34 with their published differences.
    #[test]
    fn delta_e00_matches_the_published_test_data() {
        let pairs = [
            ([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485], 2.0425),
            ([50.0, 3.1571, -77.2803], [50.0, 0.0, -82.7485], 2.8615),
            ([50.0, 2.8361, -74.0200], [50.0, 0.0, -82.7485], 3.4412),
            ([50.0, 0.0, 0.0], [50.0, -1.0, 2.0], 2.3669),
            ([50.0, 2.5, 0.0], [73.0, 25.0, -18.0], 27.1492),
            ([50.0, 2.5, 0.0], [61.0, -5.0, 29.0], 22.8977),
            (
                [60.2574, -34.0099, 36.2677],
                [60.4626, -34.1751, 39.4387],
                1.2644,
            ),
            (
                [2.0776, 0.0795, -1.1350],
                [0.9033, -0.0636, -0.5514],
                0.9082,
            ),
        ];
        for (a, b, want) in pairs {
            let got = delta_e00(a, b);
            assert!(
                (got - want).abs() < 5e-5,
                "{a:?} {b:?}: {got}, published {want}"
            );
            assert!((delta_e00(b, a) - want).abs() < 5e-5, "symmetric");
        }
        assert_eq!(delta_e00([40.0, 10.0, -5.0], [40.0, 10.0, -5.0]), 0.0);
    }

    #[test]
    fn lab_matches_the_srgb_primaries() {
        for (rgb, want) in [
            ([255, 255, 255], [100.0, 0.0, 0.0]),
            ([255, 0, 0], [53.2408, 80.0925, 67.2032]),
            ([0, 255, 0], [87.7347, -86.1827, 83.1793]),
            ([0, 0, 255], [32.2970, 79.1875, -107.8602]),
            ([0, 0, 0], [0.0, 0.0, 0.0]),
        ] {
            let got = lab(rgb);
            for k in 0..3 {
                assert!(
                    (got[k] - want[k]).abs() < 0.01,
                    "{rgb:?}: {got:?}, expected {want:?}"
                );
            }
        }
    }

    /// A black 100×80 canvas with a `colour` 40×40 square at (x, 20).
    fn square(x: u32, colour: [u8; 3]) -> RgbaImage {
        RgbaImage::from_fn(100, 80, |px, py| {
            if (x..x + 40).contains(&px) && (20..60).contains(&py) {
                Rgba([colour[0], colour[1], colour[2], 255])
            } else {
                Rgba([0, 0, 0, 255])
            }
        })
    }

    const RED: [u8; 3] = [255, 0, 0];

    #[test]
    fn a_frame_against_itself_is_iou_one_and_no_difference() {
        let a = square(20, RED);
        let c = compare(&a, &a, None, false, 8.0).unwrap();
        assert_eq!(c.iou, 1.0);
        assert_eq!(
            (c.reference_pixels, c.render_pixels, c.both_pixels),
            (1600, 1600, 1600)
        );
        assert_eq!((c.mean, c.p95), (0.0, 0.0));
    }

    #[test]
    fn a_shifted_recoloured_copy_gives_the_computed_values() {
        // 10 px to the right: 30 of 40 columns overlap, 50 in either.
        let reference = square(20, RED);
        let render = square(30, [200, 40, 40]);
        let c = compare(&reference, &render, None, false, 8.0).unwrap();
        assert_eq!(c.both_pixels, 30 * 40);
        assert!((c.iou - 1200.0 / 2000.0).abs() < 1e-12, "{}", c.iou);
        let d = delta_e00(lab(RED), lab([200, 40, 40]));
        assert!(d > 5.0, "a visible difference: {d}");
        assert!(
            (c.mean - d).abs() < 1e-9 && (c.p95 - d).abs() < 1e-9,
            "{} {}",
            c.mean,
            c.p95
        );

        // Two regions: the left half and the right half of the frame.
        let mask = RgbaImage::from_fn(100, 80, |x, _| {
            Rgba(if x < 50 {
                [255, 0, 0, 255]
            } else {
                [0, 255, 0, 255]
            })
        });
        let c = compare(&reference, &render, Some(&mask), false, 8.0).unwrap();
        let names: Vec<_> = c.regions.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["#00ff00", "#ff0000"]);
        let (right, left) = (&c.regions[0], &c.regions[1]);
        // Left (x < 50): reference 20..50, render 30..50; right: 50..60 and 50..70.
        assert_eq!((left.pixels, left.both), (4000, 20 * 40));
        assert!((left.iou - 800.0 / 1200.0).abs() < 1e-12);
        assert_eq!((right.pixels, right.both), (4000, 10 * 40));
        assert!((right.iou - 400.0 / 800.0).abs() < 1e-12);
        assert!((left.mean - d).abs() < 1e-9 && (right.mean - d).abs() < 1e-9);
    }

    #[test]
    fn align_takes_the_framing_out_and_a_size_mismatch_is_refused() {
        let reference = square(20, RED);
        // Shifted and at twice the size: aligned, the same picture.
        let small = square(30, RED);
        let render = RgbaImage::from_fn(200, 160, |x, y| *small.get_pixel(x / 2, y / 2));
        let c = compare(&reference, &render, None, true, 8.0).unwrap();
        assert!(c.iou > 0.99, "{}", c.iou);
        // Not aligned, the same shape resamples: the shift still counts.
        let c = compare(&reference, &render, None, false, 8.0).unwrap();
        assert!((c.iou - 0.6).abs() < 0.02, "{}", c.iou);
        // A different shape is refused with the fix.
        let wide = RgbaImage::from_pixel(160, 80, Rgba([0, 0, 0, 255]));
        let e = compare(&reference, &wide, None, false, 8.0).unwrap_err();
        assert!(e.contains("--size 100x80"), "{e}");
    }

    #[test]
    fn transparency_is_the_silhouette_when_an_image_has_it() {
        // The reference a cut-out (transparent background, a grey square
        // the backdrop key could not tell from a grey background).
        let reference = RgbaImage::from_fn(100, 80, |x, y| {
            let inside = (20..60).contains(&x) && (20..60).contains(&y);
            Rgba([128, 128, 128, if inside { 255 } else { 0 }])
        });
        let c = compare(&reference, &square(20, [128, 128, 128]), None, false, 8.0).unwrap();
        assert_eq!(c.reference_pixels, 1600);
        assert_eq!(c.iou, 1.0);
    }

    #[test]
    fn the_sheet_puts_three_labelled_panels_side_by_side() {
        let reference = square(20, RED);
        let c = compare(&reference, &square(30, RED), None, false, 8.0).unwrap();
        let s = sheet(&reference, &c);
        assert_eq!(
            s.dimensions(),
            (GAP + 3 * (100 + GAP), GAP + 80 + LABEL + GAP)
        );
        let at = |panel: u32, x: u32, y: u32| *s.get_pixel(GAP + panel * (100 + GAP) + x, GAP + y);
        assert_eq!(at(0, 25, 30), Rgba([255, 0, 0, 255]), "the reference");
        assert_eq!(at(1, 25, 30), Rgba([0, 0, 0, 255]), "the render, shifted");
        assert_eq!(at(2, 25, 30), Rgba([230, 50, 50, 255]), "reference only");
        assert_eq!(at(2, 45, 30), Rgba([150, 150, 150, 255]), "both");
        assert_eq!(at(2, 65, 30), Rgba([60, 110, 240, 255]), "render only");
        let label =
            (0..90).any(|dx| s.get_pixel(GAP + 2 * (100 + GAP) + 2 + dx, GAP + 80 + 8)[0] > 200);
        assert!(label, "the IoU is written under the overlay");
        assert_eq!(sheet(&reference, &c), s, "the same bytes again");
    }

    #[test]
    fn arguments() {
        let a = parse(&[
            "ref.png", "shot.png", "--mask", "m.png", "--align", "--json", "--key", "12",
        ])
        .unwrap();
        assert_eq!(a.mask, Some(PathBuf::from("m.png")));
        assert!(a.align && a.json);
        assert_eq!(a.key, 12.0);
        assert!(parse(&["ref.png"]).unwrap_err().contains("two images"));
        assert!(parse(&["a", "b", "--key", "-1"]).is_err());
    }
}
