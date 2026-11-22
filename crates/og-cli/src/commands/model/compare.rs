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
//!   image's silhouette).

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
    let image = a
        .value(&["--image"])?
        .ok_or_else(|| Failure::Usage("no image: --image <view.png>".into()))?;
    let view = View::parse(a.value(&["--view"])?.unwrap_or("front"))?;
    let reference = image::open(Path::new(image))
        .map_err(|e| Failure::Failed(format!("{image}: {e}")))?
        .to_rgba8();
    let source = load_uvs(&Local, &a.input()?, &a.import()?, a.switch("--uvs"))?;
    Ok(compare(&source, &view, &reference)?.to_string())
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
        for y in 0..h {
            for x in 0..w {
                let r = ref_mask[(y * w + x) as usize];
                let m = at(x, y).filter(|&(mx, my)| drawn.get_pixel(mx, my)[3] >= 128);
                both += usize::from(r && m.is_some());
                either += usize::from(r || m.is_some());
                let (Some((mx, my)), true) = (m, r) else {
                    continue;
                };
                let p = id_image.get_pixel(mx, my);
                let Some(k) = id_of([p[0], p[1], p[2]]).filter(|&k| k < n) else {
                    continue;
                };
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
        let parts: Vec<Value> = scene
            .parts
            .iter()
            .zip(&sums)
            .map(|(p, (k, r, d))| {
                let mean = |s: &[f64; 3]| s.map(|v| (v / *k as f64).round().clamp(0.0, 255.0) as u8);
                let distance = (*k > 0).then(|| delta_e00(lab(mean(r)), lab(mean(d))));
                json!({
                    "part": p.name,
                    "material": p.material.name,
                    "pixels": k,
                    "colourDistance": distance,
                })
            })
            .collect();
        Ok(json!({
            "view": view.name(),
            "iou": if either == 0 { 1.0 } else { both as f64 / either as f64 },
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
