//! Local-space terrain brushes, shared by authored processing and live edits.
use super::{TerrainControl, TerrainData};
use glam::{Vec2, Vec3};
use std::sync::Arc;

/// Modes match the public TerrainBrush declaration: raise, lower,
/// flatten, smooth, paint, hole, fill. All distances are terrain metres.
pub struct Brush {
    pub mode: u32,
    pub center: Vec3,
    pub radius: f32,
    pub falloff: f32,
    pub strength: f32,
    pub layer: usize,
    pub pages: usize,
}

/// The brush's weight at distance `d` from its centre: 1 inside the
/// unfaded part, smoothly to 0 at the radius.
fn weight(d: f32, radius: f32, falloff: f32) -> f32 {
    if d >= radius {
        return 0.0;
    }
    let inner = radius * (1.0 - falloff.clamp(0.0, 1.0));
    if d <= inner {
        return 1.0;
    }
    let t = (d - inner) / (radius - inner).max(1e-6);
    let s = 1.0 - t;
    s * s * (3.0 - 2.0 * s)
}

/// Applies one brush to checked terrain data, returning touched samples.
pub fn apply(d: &mut TerrainData, b: &Brush) -> u32 {
    let Brush {
        mode,
        center,
        radius,
        falloff,
        strength,
        layer,
        pages,
    } = *b;
    let c = Vec2::new(center.x, center.z);
    if mode == 4 {
        return paint(d, c, radius, falloff, strength.min(1.0), layer, pages);
    }
    if mode >= 5 {
        return cut(d, c, radius, falloff, strength.min(1.0), mode == 5);
    }
    let half = d.size * 0.5;
    let cell = d.cell();
    let (cols, rows) = (d.columns as i64, d.rows as i64);
    let to_grid = |p: Vec2| ((p + half) / cell).to_array();
    let [gx0, gz0] = to_grid(c - Vec2::splat(radius));
    let [gx1, gz1] = to_grid(c + Vec2::splat(radius));
    let (c0, c1) = (
        (gx0.floor() as i64).max(0),
        (gx1.ceil() as i64).min(cols - 1),
    );
    let (r0, r1) = (
        (gz0.floor() as i64).max(0),
        (gz1.ceil() as i64).min(rows - 1),
    );
    // Smoothing reads the unedited field, independent of visitation order.
    let old = d.heights.clone();
    let heights = Arc::make_mut(&mut d.heights);
    let mut touched = 0;
    let target = center.y;
    for r in r0..=r1 {
        for col in c0..=c1 {
            let p = Vec2::new(col as f32, r as f32) * cell - half;
            let w = weight((p - c).length(), radius, falloff);
            if w <= 0.0 {
                continue;
            }
            let k = (r * cols + col) as usize;
            let h = old[k];
            heights[k] = match mode {
                0 => h + strength * w,
                1 => h - strength * w,
                2 => h + (target - h) * (strength.min(1.0) * w),
                _ => {
                    let at = |c: i64, r: i64| {
                        old[(r.clamp(0, rows - 1) * cols + c.clamp(0, cols - 1)) as usize]
                    };
                    let mean =
                        (at(col - 1, r) + at(col + 1, r) + at(col, r - 1) + at(col, r + 1)) * 0.25;
                    h + (mean - h) * (strength.min(1.0) * w)
                }
            };
            touched += 1;
        }
    }
    touched
}

/// Cut or fill cells whose centres weigh at least half.
fn cut(d: &mut TerrainData, c: Vec2, radius: f32, falloff: f32, strength: f32, hole: bool) -> u32 {
    let (cx, cz) = d.cells();
    let mut holes = d
        .holes
        .as_ref()
        .map_or_else(|| vec![0u8; (cx * cz) as usize], |h| h.to_vec());
    let cell = d.cell();
    let half = d.size * 0.5;
    let mut touched = 0;
    let (c0, c1) = under(c.x - radius, c.x + radius, half.x, cell.x, cx);
    let (r0, r1) = under(c.y - radius, c.y + radius, half.y, cell.y, cz);
    for r in r0..r1 {
        for col in c0..c1 {
            let p = (Vec2::new(col as f32, r as f32) + 0.5) * cell - half;
            if weight((p - c).length(), radius, falloff) * strength < 0.5 {
                continue;
            }
            holes[(r * cx + col) as usize] = u8::from(hole);
            touched += 1;
        }
    }
    d.holes = holes.contains(&1).then(|| holes.into());
    touched
}

fn paint(
    d: &mut TerrainData,
    c: Vec2,
    radius: f32,
    falloff: f32,
    strength: f32,
    layer: usize,
    pages: usize,
) -> u32 {
    let control = d.control.get_or_insert_with(|| {
        let (w, h) = (d.columns.clamp(2, 1024), d.rows.clamp(2, 1024));
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..w * h {
            rgba.extend_from_slice(&[255, 0, 0, 0]);
        }
        TerrainControl {
            width: w,
            height: h,
            rgba: rgba.into(),
        }
    });
    let mut rgba = control.rgba.to_vec();
    let page = (control.width * control.height * 4) as usize;
    let pages = pages.max(control.pages() as usize);
    rgba.resize(page * pages, 0);
    let px = d.size / Vec2::new(control.width as f32, control.height as f32);
    let half = d.size * 0.5;
    let mut touched = 0;
    let (x0, x1) = under(c.x - radius, c.x + radius, half.x, px.x, control.width);
    let (y0, y1) = under(c.y - radius, c.y + radius, half.y, px.y, control.height);
    for y in y0..y1 {
        for x in x0..x1 {
            let p = (Vec2::new(x as f32, y as f32) + 0.5) * px - half;
            let w = weight((p - c).length(), radius, falloff) * strength;
            if w <= 0.0 {
                continue;
            }
            let k = ((y * control.width + x) * 4) as usize;
            for ch in 0..4 * pages {
                let at = (ch / 4) * page + k + ch % 4;
                let v = rgba[at] as f32 / 255.0;
                let to = if ch == layer { 1.0 } else { 0.0 };
                rgba[at] = ((v + (to - v) * w) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            touched += 1;
        }
    }
    control.rgba = rgba.into();
    touched
}

fn under(a: f32, b: f32, half: f32, size: f32, n: u32) -> (u32, u32) {
    let at = |v: f32| (((v + half) / size).max(0.0) as u32).min(n);
    (at(a).saturating_sub(1), (at(b) + 2).min(n))
}
