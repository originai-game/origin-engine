//! `og frame measure <png> [--region x,y,w,h] [--against x,y,w,h] [--units px|fraction]
//! [--metric contrast,highpass,mean,ramp] [--rows N] [--json]`: the numbers a
//! reviewer reads off a captured frame, computed the same way on every
//! machine (the `art-review` guide), so a hold that quotes one can be
//! checked by measuring the same region again.
//!
//! The region (default the whole frame) is in pixels of the PNG, or with
//! `--units fraction` in fractions of its width and height (a reviewer's
//! box); `--against` names a reference region (the ground beside a label,
//! an untouched strip beside a treated one). Each metric:
//!
//! - **contrast**: the WCAG 2 ratio of the region's ink to its ground. The
//!   ground is the median relative luminance of `--against` (else of the
//!   region); the ink is the mean of the region's darkest or brightest 5 %
//!   of pixels, whichever side lies farther from the ground: the glyph
//!   cores of a line of text, not its antialiased edges.
//! - **highpass**: high-pass energy, the mean absolute difference between
//!   the frame's grey (Rec. 709 luma of the sRGB values, 0 to 1) and its
//!   Gaussian blur of σ 2 px, over the region; with `--against`, the same
//!   over the reference and the ratio of the two (texture, grain, brush
//!   marks or noise against a flat or treated area).
//! - **mean**: the mean sRGB colour (and hex), the mean grey and its
//!   standard deviation.
//! - **ramp**: the mean grey of each band of `--rows` rows (default 10) from
//!   the region's top, and the largest step between neighbouring bands with
//!   where it lies (banding in a gradient, a fade's evenness).
//!
//! Every number is computed from the PNG's 8-bit sRGB values alone (alpha
//! ignored), so the same file gives the same numbers anywhere. Exit 0, 2 on
//! a usage error, 1 when the image cannot be read or the region lies off it.

use serde_json::{Value, json};
use std::process::ExitCode;

const USAGE: &str = "usage: og frame measure <png> [--region x,y,w,h] [--against x,y,w,h] [--units px|fraction] [--metric contrast,highpass,mean,ramp] [--rows N] [--json]";

/// Every metric, in the order they print.
pub const METRICS: [&str; 4] = ["contrast", "highpass", "mean", "ramp"];

/// The share of the region's pixels the ink is read from (each side).
const INK_SHARE: f64 = 0.05;

/// The σ of the high-pass's blur, px.
const SIGMA: f64 = 2.0;

/// A frame as 8-bit sRGB pixels.
pub struct Frame {
    pub w: u32,
    pub h: u32,
    pub rgb: Vec<[u8; 3]>,
}

/// A rectangle in whole pixels: x, y, width, height.
pub type Rect = [u32; 4];

fn channel(c: u8) -> f64 {
    let c = c as f64 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 2's relative luminance of an sRGB colour.
pub fn luminance(p: [u8; 3]) -> f64 {
    0.2126 * channel(p[0]) + 0.7152 * channel(p[1]) + 0.0722 * channel(p[2])
}

/// WCAG 2's contrast ratio of two relative luminances.
pub fn ratio(a: f64, b: f64) -> f64 {
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// Rec. 709 luma of the sRGB values, 0 to 1 (ImageMagick's `-colorspace gray`).
pub fn grey(p: [u8; 3]) -> f64 {
    (0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64) / 255.0
}

fn round(x: f64, places: i32) -> f64 {
    let k = 10f64.powi(places);
    (x * k).round() / k
}

fn hex(c: [f64; 3]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        c[0].round() as u8,
        c[1].round() as u8,
        c[2].round() as u8
    )
}

impl Frame {
    fn at(&self, x: u32, y: u32) -> [u8; 3] {
        self.rgb[(y * self.w + x) as usize]
    }

    fn pixels(&self, r: Rect) -> impl Iterator<Item = [u8; 3]> + '_ {
        (r[1]..r[1] + r[3]).flat_map(move |y| (r[0]..r[0] + r[2]).map(move |x| self.at(x, y)))
    }

    /// The frame's grey, blurred by a Gaussian of σ `SIGMA` (separable,
    /// radius 3σ, edges clamped): the low-pass the high-pass subtracts.
    fn blurred_grey(&self) -> Vec<f64> {
        let r = (3.0 * SIGMA).ceil() as i64;
        let k: Vec<f64> = (-r..=r)
            .map(|i| (-(i * i) as f64 / (2.0 * SIGMA * SIGMA)).exp())
            .collect();
        let sum: f64 = k.iter().sum();
        let k: Vec<f64> = k.iter().map(|v| v / sum).collect();
        let (w, h) = (self.w as i64, self.h as i64);
        let g: Vec<f64> = self.rgb.iter().map(|p| grey(*p)).collect();
        let mut tmp = vec![0.0; g.len()];
        for y in 0..h {
            for x in 0..w {
                tmp[(y * w + x) as usize] = (-r..=r)
                    .map(|i| k[(i + r) as usize] * g[(y * w + (x + i).clamp(0, w - 1)) as usize])
                    .sum();
            }
        }
        let mut out = vec![0.0; g.len()];
        for y in 0..h {
            for x in 0..w {
                out[(y * w + x) as usize] = (-r..=r)
                    .map(|i| k[(i + r) as usize] * tmp[((y + i).clamp(0, h - 1) * w + x) as usize])
                    .sum();
            }
        }
        out
    }
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// The contrast metric over `region` with the ground from `against` (else
/// the region itself).
pub fn contrast(f: &Frame, region: Rect, against: Option<Rect>) -> Value {
    let mut px: Vec<(f64, [u8; 3])> = f.pixels(region).map(|p| (luminance(p), p)).collect();
    px.sort_by(|a, b| a.0.total_cmp(&b.0));
    let ground_px: Vec<[u8; 3]> = f.pixels(against.unwrap_or(region)).collect();
    let mut gl: Vec<f64> = ground_px.iter().map(|p| luminance(*p)).collect();
    let ground = median(&mut gl);
    let chan = |c: usize| {
        let mut v: Vec<f64> = ground_px.iter().map(|p| p[c] as f64).collect();
        median(&mut v)
    };
    let ground_rgb = [chan(0), chan(1), chan(2)];
    let n = ((px.len() as f64 * INK_SHARE).ceil() as usize).max(1);
    let side = |s: &[(f64, [u8; 3])]| {
        let l = s.iter().map(|p| p.0).sum::<f64>() / s.len() as f64;
        let c = |k: usize| s.iter().map(|p| p.1[k] as f64).sum::<f64>() / s.len() as f64;
        (l, [c(0), c(1), c(2)])
    };
    let dark = side(&px[..n]);
    let bright = side(&px[px.len() - n..]);
    let (ink, ink_rgb, which) = if ratio(bright.0, ground) >= ratio(dark.0, ground) {
        (bright.0, bright.1, "bright")
    } else {
        (dark.0, dark.1, "dark")
    };
    json!({
        "ratio": round(ratio(ink, ground), 2),
        "ink": { "luminance": round(ink, 4), "colour": hex(ink_rgb), "side": which },
        "ground": { "luminance": round(ground, 4), "colour": hex(ground_rgb), "from": if against.is_some() { "against" } else { "region" } },
    })
}

/// The high-pass energy of `region` (and of `against`, with their ratio).
pub fn highpass(f: &Frame, blur: &[f64], region: Rect, against: Option<Rect>) -> Value {
    let energy = |r: Rect| {
        let mut sum = 0.0;
        for y in r[1]..r[1] + r[3] {
            for x in r[0]..r[0] + r[2] {
                let i = (y * f.w + x) as usize;
                sum += (grey(f.rgb[i]) - blur[i]).abs();
            }
        }
        sum / (r[2] as f64 * r[3] as f64)
    };
    let e = energy(region);
    match against {
        Some(a) => {
            let b = energy(a);
            json!({
                "energy": round(e, 5),
                "against": round(b, 5),
                "ratio": if b > 0.0 { Value::from(round(e / b, 3)) } else { Value::Null },
                "sigma": SIGMA,
            })
        }
        None => json!({ "energy": round(e, 5), "sigma": SIGMA }),
    }
}

/// The mean colour and grey of `region`.
pub fn mean(f: &Frame, region: Rect) -> Value {
    let px: Vec<[u8; 3]> = f.pixels(region).collect();
    let n = px.len() as f64;
    let c = |k: usize| px.iter().map(|p| p[k] as f64).sum::<f64>() / n;
    let rgb = [c(0), c(1), c(2)];
    let g: Vec<f64> = px.iter().map(|p| grey(*p)).collect();
    let m = g.iter().sum::<f64>() / n;
    let sd = (g.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / n).sqrt();
    json!({
        "rgb": rgb.map(|v| round(v, 2)),
        "colour": hex(rgb),
        "grey": round(m, 4),
        "greyStdDev": round(sd, 4),
        "luminance": round(px.iter().map(|p| luminance(*p)).sum::<f64>() / n, 4),
    })
}

/// The mean grey of each band of `rows` rows from the region's top and the
/// largest step between neighbouring bands.
pub fn ramp(f: &Frame, region: Rect, rows: u32) -> Value {
    let mut bands = Vec::new();
    let mut y = region[1];
    while y < region[1] + region[3] {
        let y1 = (y + rows).min(region[1] + region[3]);
        let band = [region[0], y, region[2], y1 - y];
        let g: f64 = f.pixels(band).map(grey).sum::<f64>() / (band[2] as f64 * band[3] as f64);
        bands.push((y, y1, g));
        y = y1;
    }
    let mut step = json!(null);
    let mut most = -1.0;
    for w in bands.windows(2) {
        let d = w[1].2 - w[0].2;
        if d.abs() > most {
            most = d.abs();
            step = json!({ "step": round(d, 4), "from": [w[0].0, w[0].1], "to": [w[1].0, w[1].1] });
        }
    }
    json!({
        "rows": rows,
        "bands": bands.iter().map(|(a, b, g)| json!({ "y": [a, b], "grey": round(*g, 4) })).collect::<Vec<_>>(),
        "maxStep": step,
    })
}

/// Parses `x,y,w,h` in pixels or (`fraction`) fractions of the frame, to
/// whole pixels inside it.
pub fn region(s: &str, fraction: bool, w: u32, h: u32) -> Result<Rect, String> {
    let v: Vec<f64> = s
        .split(',')
        .map(|p| p.trim().parse::<f64>())
        .collect::<Result<_, _>>()
        .map_err(|_| format!("`{s}` is not x,y,w,h"))?;
    let [x, y, rw, rh] = v[..] else {
        return Err(format!("`{s}` is not x,y,w,h"));
    };
    let (x0, y0, x1, y1) = if fraction {
        (
            (x * w as f64).round(),
            (y * h as f64).round(),
            ((x + rw) * w as f64).round(),
            ((y + rh) * h as f64).round(),
        )
    } else {
        (x.round(), y.round(), (x + rw).round(), (y + rh).round())
    };
    if x0 < 0.0 || y0 < 0.0 || x1 > w as f64 || y1 > h as f64 || x1 <= x0 || y1 <= y0 {
        return Err(format!(
            "the region {s}{} is not inside the {w}×{h} frame",
            if fraction { " (fractions)" } else { " (px)" }
        ));
    }
    Ok([x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32])
}

/// Reads a PNG (or any image the `image` crate reads) as sRGB pixels.
pub fn open(path: &str) -> Result<Frame, String> {
    let img = image::open(path)
        .map_err(|e| format!("{path}: {e}"))?
        .to_rgb8();
    Ok(Frame {
        w: img.width(),
        h: img.height(),
        rgb: img.pixels().map(|p| p.0).collect(),
    })
}

/// The whole measurement as one JSON document.
pub fn measure(
    f: &Frame,
    region: Rect,
    against: Option<Rect>,
    metrics: &[&str],
    rows: u32,
) -> Value {
    let mut out = serde_json::Map::new();
    let blur = metrics.contains(&"highpass").then(|| f.blurred_grey());
    for m in METRICS.iter().filter(|m| metrics.contains(m)) {
        let v = match *m {
            "contrast" => contrast(f, region, against),
            "highpass" => highpass(f, blur.as_deref().unwrap_or_default(), region, against),
            "mean" => mean(f, region),
            _ => ramp(f, region, rows),
        };
        out.insert(m.to_string(), v);
    }
    json!({
        "size": [f.w, f.h],
        "region": region,
        "against": against,
        "metrics": out,
    })
}

fn text(image: &str, v: &Value) -> String {
    let r = &v["region"];
    let mut s = format!(
        "og frame measure {image} ({}×{}), region {},{} {}×{} px{}\n",
        v["size"][0],
        v["size"][1],
        r[0],
        r[1],
        r[2],
        r[3],
        match v["against"].as_array() {
            Some(a) => format!(" against {},{} {}×{} px", a[0], a[1], a[2], a[3]),
            None => String::new(),
        }
    );
    let m = &v["metrics"];
    if let Some(c) = m.get("contrast") {
        s += &format!(
            "  contrast  {}:1  ink {} ({}, luminance {})  ground {} (luminance {}, from the {})\n",
            c["ratio"],
            c["ink"]["colour"].as_str().unwrap_or(""),
            c["ink"]["side"].as_str().unwrap_or(""),
            c["ink"]["luminance"],
            c["ground"]["colour"].as_str().unwrap_or(""),
            c["ground"]["luminance"],
            c["ground"]["from"].as_str().unwrap_or("")
        );
    }
    if let Some(h) = m.get("highpass") {
        s += &format!("  highpass  {} (σ {} px)", h["energy"], h["sigma"]);
        if !h["against"].is_null() {
            s += &format!("  against {}  ratio {}", h["against"], h["ratio"]);
        }
        s.push('\n');
    }
    if let Some(a) = m.get("mean") {
        s += &format!(
            "  mean      {}  grey {} (sd {})  luminance {}\n",
            a["colour"].as_str().unwrap_or(""),
            a["grey"],
            a["greyStdDev"],
            a["luminance"]
        );
    }
    if let Some(r) = m.get("ramp") {
        let bands: Vec<String> = r["bands"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|b| format!("{}", b["grey"]))
            .collect();
        s += &format!("  ramp      per {} rows: {}\n", r["rows"], bands.join(" "));
        if !r["maxStep"].is_null() {
            let st = &r["maxStep"];
            s += &format!(
                "            largest step {} from y {}–{} to y {}–{}\n",
                st["step"], st["from"][0], st["from"][1], st["to"][0], st["to"][1]
            );
        }
    }
    s
}

pub fn run(args: &[&str]) -> ExitCode {
    let ["measure", rest @ ..] = args else {
        eprintln!("og frame: {USAGE}");
        return ExitCode::from(2);
    };
    let mut image = None;
    let mut reg = None;
    let mut against = None;
    let mut fraction = false;
    let mut metrics: Vec<&str> = METRICS.to_vec();
    let mut rows = 10u32;
    let mut as_json = false;
    let mut i = 0;
    while i < rest.len() {
        let value = rest.get(i + 1).copied();
        match (rest[i], value) {
            ("--json", _) => {
                as_json = true;
                i += 1;
                continue;
            }
            ("--require-checked", _) => {
                i += 1;
                continue;
            }
            ("--region", Some(v)) => reg = Some(v),
            ("--against", Some(v)) => against = Some(v),
            ("--units", Some(v @ ("px" | "fraction"))) => fraction = v == "fraction",
            ("--metric", Some(v)) => {
                metrics = v.split(',').map(str::trim).collect();
                if let Some(bad) = metrics.iter().find(|m| !METRICS.contains(m)) {
                    eprintln!(
                        "og frame measure: no metric `{bad}`; it takes {}",
                        METRICS.join(", ")
                    );
                    return ExitCode::from(2);
                }
            }
            ("--rows", Some(v)) => match v.parse::<u32>() {
                Ok(n) if n > 0 => rows = n,
                _ => {
                    eprintln!("og frame measure: --rows takes a whole number above 0");
                    return ExitCode::from(2);
                }
            },
            (a, _) if a.starts_with('-') => {
                eprintln!("og frame measure: `{a}` (or its value) is not an option. {USAGE}");
                return ExitCode::from(2);
            }
            (a, _) if image.is_none() => {
                image = Some(a);
                i += 1;
                continue;
            }
            (a, _) => {
                eprintln!("og frame measure: one image only (`{a}`). {USAGE}");
                return ExitCode::from(2);
            }
        }
        i += 2;
    }
    let Some(image) = image else {
        eprintln!("og frame measure: name the image. {USAGE}");
        return ExitCode::from(2);
    };
    let f = match open(image) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("og frame measure: {e}");
            return ExitCode::FAILURE;
        }
    };
    let rect = |s: Option<&str>| s.map(|s| region(s, fraction, f.w, f.h)).transpose();
    let (r, a) = match (rect(reg), rect(against)) {
        (Ok(r), Ok(a)) => (r.unwrap_or([0, 0, f.w, f.h]), a),
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("og frame measure: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut v = measure(&f, r, a, &metrics, rows);
    v["image"] = json!(image);
    v["uncheckedCount"] = json!(0);
    if as_json {
        println!("{v}");
    } else {
        print!("{}", text(image, &v));
    }
    ExitCode::SUCCESS
}
