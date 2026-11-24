//! A 2D texture's normal map (R11, docs/art.md): the `normal` line of its
//! import settings' `texture` block names one, `normal "hero_n.png"` (a
//! tangent-space normal map beside it, red towards +u, green up the
//! image), or asks the cook to make one, `normal auto`. Either is cooked
//! as `<id>#normal`, a normal-kind texture, which the runtime loads beside
//! the texture and the 2D lighting shades with (og-render's light map
//! direction strips).
//!
//! `auto` raises the drawn shape from its silhouette: the height is a
//! bevel rising over the first few pixels in from the transparent edge (a
//! chamfer distance to the nearest clear pixel), plus a little of the
//! colour's luminance for surface detail; the normal is the height's
//! slope. Clear pixels, and an image with no clear pixel at all away from
//! its detail, are flat.

use super::texture::Image;

/// What a texture's `normal` line asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum NormalSetting {
    /// `normal auto`.
    Auto,
    /// `normal "<file>"`, relative to the texture's directory.
    File(String),
}

/// The `normal` line of the `texture` block, if any.
pub fn setting(block: Option<&og_core::ogt::Block>) -> Result<Option<NormalSetting>, String> {
    let Some(e) = block.and_then(|b| b.get("normal")) else {
        return Ok(None);
    };
    match e.values.first() {
        Some(v) if v.as_word() == Some("auto") => Ok(Some(NormalSetting::Auto)),
        Some(_) => match e.string(0) {
            Ok(s) if !s.is_empty() => Ok(Some(NormalSetting::File(s.to_string()))),
            _ => Err(format!(
                "line {}: normal takes auto or a quoted file name (normal \"hero_n.png\")",
                e.line
            )),
        },
        None => Err(format!(
            "line {}: normal takes auto or a quoted file name",
            e.line
        )),
    }
}

/// The bevel's width in pixels for an image of this size.
fn bevel(width: u32, height: u32) -> f32 {
    (width.min(height) as f32 / 12.0).clamp(2.0, 16.0)
}

/// Chamfer (3-4) distances in pixels from each pixel to the nearest clear
/// one (alpha under half), capped at `cap`; an image with no clear pixel is
/// `cap` everywhere.
fn distances(img: &Image, cap: f32) -> Vec<f32> {
    let (w, h) = (img.width as usize, img.height as usize);
    let big = (cap * 3.0).ceil() + 8.0;
    let mut d: Vec<f32> = (0..w * h)
        .map(|i| if img.rgba[i * 4 + 3] < 128 { 0.0 } else { big })
        .collect();
    let at = |d: &Vec<f32>, x: isize, y: isize| -> f32 {
        if x < 0 || y < 0 || x >= w as isize || y >= h as isize {
            big
        } else {
            d[y as usize * w + x as usize]
        }
    };
    for y in 0..h as isize {
        for x in 0..w as isize {
            let i = y as usize * w + x as usize;
            let v = d[i]
                .min(at(&d, x - 1, y) + 3.0)
                .min(at(&d, x, y - 1) + 3.0)
                .min(at(&d, x - 1, y - 1) + 4.0)
                .min(at(&d, x + 1, y - 1) + 4.0);
            d[i] = v;
        }
    }
    for y in (0..h as isize).rev() {
        for x in (0..w as isize).rev() {
            let i = y as usize * w + x as usize;
            let v = d[i]
                .min(at(&d, x + 1, y) + 3.0)
                .min(at(&d, x, y + 1) + 3.0)
                .min(at(&d, x + 1, y + 1) + 4.0)
                .min(at(&d, x - 1, y + 1) + 4.0);
            d[i] = v;
        }
    }
    d.into_iter().map(|v| (v / 3.0).min(cap)).collect()
}

/// `normal auto`: a normal map from the image's silhouette and luminance.
pub fn auto(img: &Image) -> Image {
    let (w, h) = (img.width as usize, img.height as usize);
    let b = bevel(img.width, img.height);
    let dist = distances(img, b);
    let height: Vec<f32> = (0..w * h)
        .map(|i| {
            if img.rgba[i * 4 + 3] < 128 {
                return 0.0;
            }
            let t = (dist[i] / b).clamp(0.0, 1.0);
            let rise = t * t * (3.0 - 2.0 * t);
            let p = &img.rgba[i * 4..i * 4 + 3];
            let luma = (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32) / 255.0;
            rise + 0.08 * luma
        })
        .collect();
    let at = |x: isize, y: isize| -> f32 {
        let x = x.clamp(0, w as isize - 1) as usize;
        let y = y.clamp(0, h as isize - 1) as usize;
        height[y * w + x]
    };
    let strength = 0.75 * b;
    let mut rgba = Vec::with_capacity(w * h * 4);
    for y in 0..h as isize {
        for x in 0..w as isize {
            let i = y as usize * w + x as usize;
            let (nx, ny) = if img.rgba[i * 4 + 3] < 128 {
                (0.0, 0.0)
            } else {
                // Towards +u where the height falls to the right; up the
                // image (smaller y) where it falls going up.
                (
                    -(at(x + 1, y) - at(x - 1, y)) * 0.5 * strength,
                    (at(x, y + 1) - at(x, y - 1)) * 0.5 * strength,
                )
            };
            let len = (nx * nx + ny * ny + 1.0).sqrt();
            let c = |v: f32| ((v / len * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8;
            rgba.extend_from_slice(&[c(nx), c(ny), c(1.0), 255]);
        }
    }
    Image {
        width: img.width,
        height: img.height,
        rgba,
    }
}
