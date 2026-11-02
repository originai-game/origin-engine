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

#[cfg(test)]
mod tests {
    use super::*;

    /// A grey disc of radius 12 in a 32 x 32 clear image.
    fn disc() -> Image {
        let mut rgba = vec![0u8; 32 * 32 * 4];
        for y in 0..32 {
            for x in 0..32 {
                let (dx, dy) = (x as f32 - 15.5, y as f32 - 15.5);
                if dx * dx + dy * dy <= 144.0 {
                    rgba[(y * 32 + x) * 4..(y * 32 + x) * 4 + 4]
                        .copy_from_slice(&[128, 128, 128, 255]);
                }
            }
        }
        Image {
            width: 32,
            height: 32,
            rgba,
        }
    }

    fn n(img: &Image, x: usize, y: usize) -> [u8; 3] {
        let i = (y * img.width as usize + x) * 4;
        [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]]
    }

    #[test]
    fn the_rim_leans_outwards_the_middle_and_the_clear_are_flat() {
        let m = auto(&disc());
        // Left rim leans left (red low); right rim right; top rim (small y)
        // up the image (green high); bottom rim down.
        let (l, r, t, b) = (n(&m, 5, 16), n(&m, 26, 16), n(&m, 16, 5), n(&m, 16, 26));
        assert!(l[0] < 100 && r[0] > 156, "{l:?} {r:?}");
        assert!(t[1] > 156 && b[1] < 100, "{t:?} {b:?}");
        assert!((l[1] as i32 - 128).abs() < 12, "{l:?}");
        // The middle (past the bevel) and the clear corner face the viewer.
        for p in [n(&m, 16, 16), n(&m, 0, 0)] {
            assert!(
                (p[0] as i32 - 128).abs() <= 2 && (p[1] as i32 - 128).abs() <= 2 && p[2] >= 253,
                "{p:?}"
            );
        }
    }

    #[test]
    fn an_opaque_image_is_flat_but_for_its_detail() {
        let mut img = Image {
            width: 8,
            height: 8,
            rgba: vec![200; 8 * 8 * 4],
        };
        for p in img.rgba.chunks_mut(4) {
            p[3] = 255;
        }
        assert!(
            auto(&img)
                .rgba
                .chunks(4)
                .all(|p| p[0] == 128 && p[1] == 128)
        );
    }

    #[test]
    fn the_setting_reads_auto_a_file_or_refuses() {
        let block = |text: &str| og_core::ogt::Document::parse(text).unwrap();
        let read = |text: &str| {
            let d = block(text);
            setting(d.body.get("texture").and_then(|e| e.block.as_ref()))
        };
        let head = "type import\nversion 1\n---\n";
        assert_eq!(
            read(&format!("{head}texture {{\n  normal auto\n}}\n")),
            Ok(Some(NormalSetting::Auto))
        );
        assert_eq!(
            read(&format!("{head}texture {{\n  normal \"hero_n.png\"\n}}\n")),
            Ok(Some(NormalSetting::File("hero_n.png".into())))
        );
        assert_eq!(
            read(&format!("{head}texture {{\n  kind color\n}}\n")),
            Ok(None)
        );
        assert!(read(&format!("{head}texture {{\n  normal\n}}\n")).is_err());
    }
}
