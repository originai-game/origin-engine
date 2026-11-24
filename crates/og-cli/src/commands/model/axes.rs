//! `og model preview --axes`: the engine's axes drawn over each view at the
//! model's origin, labelled, so the nose can be checked before the model
//! is used: `+Z FRONT` (blue; the way the object faces and travels, the
//! facing guide), `+Y UP` (green) and `+X` (red; the object's left). In
//! the engine's frame (glTF's: metres, +Y up), whatever the file's own
//! axes were read as. A model whose nose is not on the blue arrow needs
//! `model { front … up … }` in its import settings.

use og_shape::math::DVec3;
use og_shape::tools::{Image, ViewCamera};

/// The axes as drawn, in the scene's frame (kernel: glTF x, −z, y), with
/// their colour and label.
const AXES: [(DVec3, [u8; 3], &str); 3] = [
    (DVec3::new(1.0, 0.0, 0.0), [230, 40, 40], "+X"),
    (DVec3::new(0.0, 0.0, 1.0), [40, 200, 60], "+Y UP"),
    (DVec3::new(0.0, -1.0, 0.0), [40, 110, 255], "+Z FRONT"),
];

/// What the report says was drawn.
pub const ABOUT: &str = r#"{"origin":[0,0,0],"frame":"engine (glTF): +Z front, +Y up, +X left","colors":{"+X":"red","+Y UP":"green","+Z FRONT":"blue"}}"#;

/// Where `p` lands in the view, in pixels (`None` behind a perspective
/// camera).
fn project(c: &ViewCamera, p: DVec3) -> Option<(f64, f64)> {
    let forward = (c.target - c.position).normalize();
    let right = forward.cross(c.up).normalize();
    let up = right.cross(forward);
    let (w, h) = (f64::from(c.width), f64::from(c.height));
    let (x, y) = match c.fov {
        None => {
            let d = p - c.target;
            let k = h / c.ortho_height;
            (d.dot(right) * k, d.dot(up) * k)
        }
        Some(fov) => {
            let d = p - c.position;
            let z = d.dot(forward);
            if z <= 1e-9 {
                return None;
            }
            let k = h / 2.0 / (fov / 2.0).tan() / z;
            (d.dot(right) * k, d.dot(up) * k)
        }
    };
    Some((w / 2.0 + x, h / 2.0 - y))
}

/// The length of an arrow in scene units: about a third of the view.
fn length(c: &ViewCamera) -> f64 {
    match c.fov {
        None => 0.38 * c.ortho_height,
        Some(fov) => 0.38 * 2.0 * (fov / 2.0).tan() * (c.target - c.position).length(),
    }
}

fn plot(im: &mut Image, x: i64, y: i64, rgb: [u8; 3]) {
    if x < 0 || y < 0 || x >= i64::from(im.width) || y >= i64::from(im.height) {
        return;
    }
    let at = ((y as usize) * im.width as usize + x as usize) * 4;
    im.rgba[at..at + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
}

/// A line `t` pixels thick.
fn line(im: &mut Image, a: (f64, f64), b: (f64, f64), t: i64, rgb: [u8; 3]) {
    let n = (b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil().max(1.0) as i64;
    for i in 0..=n {
        let f = i as f64 / n as f64;
        let (x, y) = (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f);
        for dy in -t / 2..=t / 2 {
            for dx in -t / 2..=t / 2 {
                plot(im, x.round() as i64 + dx, y.round() as i64 + dy, rgb);
            }
        }
    }
}

/// 5 × 7 glyphs, a row a byte (bit 4 the left column).
fn glyph(c: char) -> [u8; 7] {
    match c {
        '+' => [0, 4, 4, 31, 4, 4, 0],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        _ => [0; 7],
    }
}

/// `text` at (x, y), its top left, each glyph pixel `s` pixels, on a dark
/// plate so it reads over any model.
fn label(im: &mut Image, x: i64, y: i64, s: i64, text: &str, rgb: [u8; 3]) {
    let w = text.chars().count() as i64 * 6 * s;
    for py in y - s..y + 8 * s {
        for px in x - s..x + w {
            plot(im, px, py, [16, 18, 24]);
        }
    }
    for (i, c) in text.chars().enumerate() {
        for (row, bits) in glyph(c).iter().enumerate() {
            for col in 0..5 {
                if bits & (16 >> col) != 0 {
                    for dy in 0..s {
                        for dx in 0..s {
                            plot(
                                im,
                                x + (i as i64 * 6 + col) * s + dx,
                                y + row as i64 * s + dy,
                                rgb,
                            );
                        }
                    }
                }
            }
        }
    }
}

/// The axes drawn over one view taken by `c`.
pub fn draw(im: &mut Image, c: &ViewCamera) {
    let Some(o) = project(c, DVec3::ZERO) else {
        return;
    };
    let len = length(c);
    let t = (i64::from(im.height) / 200).max(1) + 1;
    let s = (i64::from(im.height) / 256).max(1);
    for (dir, rgb, text) in AXES {
        let Some(e) = project(c, dir * len) else {
            continue;
        };
        line(im, o, e, t, rgb);
        // The head: two short strokes back from the tip.
        let (dx, dy) = (e.0 - o.0, e.1 - o.1);
        let n = (dx * dx + dy * dy).sqrt();
        if n > 4.0 {
            let (ux, uy) = (dx / n, dy / n);
            let k = (n * 0.15).min(12.0 * s as f64);
            for side in [-1.0, 1.0] {
                let back = (
                    e.0 - ux * k - uy * k * 0.5 * side,
                    e.1 - uy * k + ux * k * 0.5 * side,
                );
                line(im, e, back, t, rgb);
            }
        }
        let gap = 4.0 * s as f64;
        let (lx, ly) = if n > 1.0 {
            (e.0 + dx / n * gap, e.1 + dy / n * gap)
        } else {
            (e.0 + gap, e.1)
        };
        // Clamp so a label near an edge stays whole.
        let w = text.chars().count() as i64 * 6 * s;
        let x = (lx as i64 - if dx < 0.0 { w } else { 0 }).clamp(s, i64::from(im.width) - w - s);
        let y =
            (ly as i64 - if dy < 0.0 { 8 * s } else { 0 }).clamp(s, i64::from(im.height) - 8 * s);
        label(im, x, y, s, text, rgb);
    }
}
