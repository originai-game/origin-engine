//! `og model compare <model> --image <view.png> --view <view>`: how a
//! model fits a picture of it, as facts with no thresholds.
//!
//! The model is drawn in the named view ([`super::views`]) at the
//! image's size, unlit (each pixel its material's colour), and its
//! silhouette is aligned to the image's as `og compare --align` does: the
//! render's silhouette box scaled uniformly to the image's box area and
//! centred on it, so a concept's framing does not count against the fit.
//! The image's silhouette is its alpha (≥ 128) when it has any, else what
//! differs from its border colour. Reported:
//!
//! - `iou`: the silhouettes' intersection over their union, aligned.
//! - `aspect`: the silhouette box's width over height, the model's and
//!   the image's, and their ratio (model over image): a part too tall or
//!   too wide shows here even when the IoU is high.
//! - `parts`: per part, the pixels where it and the image overlap, and
//!   CIEDE2000 between its mean drawn colour and the image's mean colour
//!   over those pixels (`null` where the part is hidden or off the
//!   image's silhouette); and the thin-part facts ([`super::thin`]):
//!   `widthPx`, the part's width in the image's pixels (0 where it shows
//!   no pixel: hidden, or too thin for the silhouette), `edgeShare`, the share of its pixels on its edge, and
//!   `resolved`, whether it is at least two pixels wide.
//! - `iouOnePixel`: the IoU the model's own silhouette keeps when it is
//!   off by one pixel all round: the most a fit this size can show, so a
//!   low `iou` on thin legs reads as the view's resolution.
//!
//! `--image <view>=<png>`, repeated, compares every view in one call:
//! `views` (each view's facts as above) and the same facts ranked
//! (`byIou`, `byAspect`, `byColour`, `byWidth`), so one pass fixes them
//! all.

use image::RgbaImage;
use og_shape::mesh::Solid;
use og_shape::tools::{ColorSpace, Material, Part, Scene};
use serde_json::{Value, json};
use std::path::Path;

use super::args::{Args, IMPORT_OPTIONS};
use super::preview::with_scene;
use super::source::{Local, Source, load_uvs};
use super::views::{self, Look, View};
use super::{Failure, failed};
use crate::commands::compare::{aligned, delta_e00, lab, silhouette};

/// Runs `og model compare`.
pub fn run(a: &Args) -> Result<String, Failure> {
    a.only(&[
        "--image",
        "--view",
        "--uvs",
        IMPORT_OPTIONS[0],
        IMPORT_OPTIONS[1],
        IMPORT_OPTIONS[2],
        IMPORT_OPTIONS[3],
    ])?;
    let source = load_uvs(&Local, &a.input()?, &a.import()?, a.switch("--uvs"))?;
    let images = a.values("--image")?;
    // `--image <view>=<png>`, repeated: every view in one call.
    let pairs: Vec<(View, &str)> = images
        .iter()
        .filter_map(|v| {
            let (view, path) = v.split_once('=')?;
            View::parse(view).ok().map(|w| (w, path))
        })
        .collect();
    let read = |image: &str| {
        image::open(Path::new(image))
            .map(|i| i.to_rgba8())
            .map_err(|e| Failure::Failed(format!("{image}: {e}")))
    };
    if !pairs.is_empty() {
        if pairs.len() != images.len() || a.value(&["--view"])?.is_some() {
            return Err(Failure::Usage(
                "give every image as --image <view>=<png>, or one --image <png> with --view".into(),
            ));
        }
        let mut views = Vec::new();
        for (view, path) in &pairs {
            views.push(compare(&source, view, &read(path)?)?);
        }
        return Ok(summary(views).to_string());
    }
    let [image] = images[..] else {
        return Err(Failure::Usage(
            "no image: --image <view.png> --view <view>, or --image <view>=<png> for each view"
                .into(),
        ));
    };
    let view = View::parse(a.value(&["--view"])?.unwrap_or("front"))?;
    Ok(compare(&source, &view, &read(image)?)?.to_string())
}

/// Every view's facts, and the same facts ranked so one pass can fix them
/// all: `byIou` (the views, least overlap first), `byAspect` (the views,
/// farthest aspect ratio from 1 first), `byColour` (each part's largest
/// colour distance over the views, farthest first) and `byWidth` (each
/// part's narrowest width over the views, thinnest first). Rankings, no
/// thresholds.
fn summary(views: Vec<Value>) -> Value {
    let name = |v: &Value| v["view"].clone();
    let mut by_iou: Vec<&Value> = views.iter().collect();
    let iou = |v: &Value| v["iou"].as_f64().unwrap_or(0.0);
    by_iou.sort_by(|a, b| iou(a).total_cmp(&iou(b)));
    let off = |v: &Value| {
        v["aspect"]["ratio"]
            .as_f64()
            .map_or(0.0, |r| (r.ln()).abs())
    };
    let mut by_aspect: Vec<&Value> = views.iter().collect();
    by_aspect.sort_by(|a, b| off(b).total_cmp(&off(a)));
    let mut colour: Vec<(String, f64, Value)> = Vec::new();
    for v in &views {
        for p in v["parts"].as_array().into_iter().flatten() {
            let (Some(part), Some(d)) = (p["part"].as_str(), p["colourDistance"].as_f64()) else {
                continue;
            };
            match colour.iter_mut().find(|c| c.0 == part) {
                Some(c) if c.1 >= d => {}
                Some(c) => *c = (part.into(), d, name(v)),
                None => colour.push((part.into(), d, name(v))),
            }
        }
    }
    colour.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    // Each part's narrowest showing over the views, thinnest first.
    let mut width: Vec<(String, f64, Value, Value)> = Vec::new();
    for v in &views {
        for p in v["parts"].as_array().into_iter().flatten() {
            let (Some(part), Some(d)) = (p["part"].as_str(), p["widthPx"].as_f64()) else {
                continue;
            };
            if d <= 0.0 {
                continue;
            }
            match width.iter_mut().find(|c| c.0 == part) {
                Some(c) if c.1 <= d => {}
                Some(c) => *c = (part.into(), d, name(v), p["resolved"].clone()),
                None => width.push((part.into(), d, name(v), p["resolved"].clone())),
            }
        }
    }
    width.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    json!({
        "byIou": by_iou.iter().map(|v| json!({ "view": name(v), "iou": v["iou"] })).collect::<Vec<_>>(),
        "byAspect": by_aspect.iter().map(|v| json!({ "view": name(v), "ratio": v["aspect"]["ratio"] })).collect::<Vec<_>>(),
        "byColour": colour.iter().map(|(p, d, v)| json!({ "part": p, "colourDistance": d, "view": v })).collect::<Vec<_>>(),
        "byWidth": width.iter().map(|(p, d, v, r)| json!({ "part": p, "widthPx": d, "view": v, "resolved": r })).collect::<Vec<_>>(),
        "views": views,
    })
}

/// A part's ID as a flat colour, 8 apart per channel so the unlit render
/// gives it back exactly.
fn id_colour(k: usize) -> [u8; 3] {
    let id = k + 1;
    [
        (id & 31) as u8 * 8,
        ((id >> 5) & 31) as u8 * 8,
        ((id >> 10) & 31) as u8 * 8,
    ]
}

fn id_of(p: [u8; 3]) -> Option<usize> {
    let c = p.map(|v| (usize::from(v) + 4) / 8);
    (c[0] | c[1] << 5 | c[2] << 10).checked_sub(1)
}

fn rgba(i: og_shape::tools::Image) -> Result<RgbaImage, Failure> {
    RgbaImage::from_raw(i.width, i.height, i.rgba)
        .ok_or_else(|| Failure::Failed("the view's image has the wrong size".into()))
}

/// The facts for `source` drawn in `view` against `reference`.
pub fn compare(source: &Source, view: &View, reference: &RgbaImage) -> Result<Value, Failure> {
    let (w, h) = reference.dimensions();
    if w == 0 || h == 0 {
        return Err(Failure::Failed("the image is empty".into()));
    }
    let flat = Look {
        unlit: true,
        edges: false,
    };
    with_scene(source, None, &[], |scene| {
        let (drawn, _) = views::render(scene, view, (w, h), flat)?;
        let drawn = rgba(drawn)?;
        // The same view with each part one flat ID colour.
        let bare: Vec<Solid> = scene
            .parts
            .iter()
            .map(|p| Solid {
                colors: None,
                ..p.solid.clone()
            })
            .collect();
        let ids = Scene {
            name: scene.name.clone(),
            parts: scene
                .parts
                .iter()
                .zip(&bare)
                .enumerate()
                .map(|(k, (p, s))| Part {
                    solid: s,
                    material: Material {
                        color: Some(id_colour(k).map(|c| f64::from(c) / 255.0)),
                        color_space: ColorSpace::Srgb,
                        ..Default::default()
                    },
                    ..p.clone()
                })
                .collect(),
        };
        let (id_image, _) = views::render(&ids, view, (w, h), flat)?;
        let id_image = rgba(id_image)?;
        let (ref_mask, _) = silhouette(reference, 8.0);
        let model_mask: Vec<bool> = drawn.pixels().map(|p| p[3] >= 128).collect();
        let to_model = aligned(&ref_mask, w, &model_mask, w).ok_or_else(|| {
            Failure::Failed(
                "no silhouette to align: the model draws nothing in this view, or the image has no object"
                    .into(),
            )
        })?;
        // Every image pixel's model pixel (nearest), aligned.
        let at = |x: u32, y: u32| -> Option<(u32, u32)> {
            let (mx, my) = to_model(f64::from(x) + 0.5, f64::from(y) + 0.5);
            (mx >= 0.0 && my >= 0.0 && mx < f64::from(w) && my < f64::from(h))
                .then_some((mx as u32, my as u32))
        };
        let n = scene.parts.len();
        let (mut both, mut either) = (0usize, 0usize);
        // Per part: pixels, the image's colour sum and the drawn one's.
        let mut sums = vec![(0usize, [0f64; 3], [0f64; 3]); n];
        // The aligned model in the image's pixels: each pixel's part.
        let mut labels: Vec<Option<usize>> = vec![None; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let r = ref_mask[(y * w + x) as usize];
                let m = at(x, y).filter(|&(mx, my)| drawn.get_pixel(mx, my)[3] >= 128);
                both += usize::from(r && m.is_some());
                either += usize::from(r || m.is_some());
                let Some((mx, my)) = m else {
                    continue;
                };
                let p = id_image.get_pixel(mx, my);
                let Some(k) = id_of([p[0], p[1], p[2]]).filter(|&k| k < n) else {
                    continue;
                };
                labels[(y * w + x) as usize] = Some(k);
                if !r {
                    continue;
                }
                let (rp, dp) = (reference.get_pixel(x, y), drawn.get_pixel(mx, my));
                let s = &mut sums[k];
                s.0 += 1;
                for c in 0..3 {
                    s.1[c] += f64::from(rp[c]);
                    s.2[c] += f64::from(dp[c]);
                }
            }
        }
        let aspect = |mask: &[bool]| {
            crate::commands::compare::bbox(mask, w).map(|b| (b.2 - b.0) / (b.3 - b.1))
        };
        let (am, ai) = (aspect(&model_mask), aspect(&ref_mask));
        let (uw, uh) = (w as usize, h as usize);
        let widths = super::thin::widths(&labels, uw, uh, n);
        let shown: Vec<bool> = labels.iter().map(Option::is_some).collect();
        let parts: Vec<Value> = scene
            .parts
            .iter()
            .zip(&sums)
            .zip(&widths)
            .map(|((p, (k, r, d)), &(width, edge))| {
                let mean = |s: &[f64; 3]| s.map(|v| (v / *k as f64).round().clamp(0.0, 255.0) as u8);
                let distance = (*k > 0).then(|| delta_e00(lab(mean(r)), lab(mean(d))));
                json!({
                    "part": p.name,
                    "material": p.material.name,
                    "pixels": k,
                    "colourDistance": distance,
                    "widthPx": width,
                    "edgeShare": edge,
                    "resolved": width >= 2.0,
                })
            })
            .collect();
        Ok(json!({
            "view": view.name(),
            "iou": if either == 0 { 1.0 } else { both as f64 / either as f64 },
            "iouOnePixel": super::thin::iou_one_pixel(&shown, uw, uh),
            "aspect": {
                "model": am,
                "image": ai,
                "ratio": am.zip(ai).map(|(m, i)| m / i),
            },
            "parts": parts,
        }))
    })
    .map_err(|e| match e {
        Failure::Failed(w) => failed(w),
        u => u,
    })
}
