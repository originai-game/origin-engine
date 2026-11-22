//! Triangles into pixels: coverage by integer edge functions, perspective-correct weights, clipping.

use super::*;

pub(crate) struct Target {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgb: Vec<[u8; 3]>,
    pub(crate) depth: Vec<f64>,
    pub(crate) ground: Vec<bool>,
}

/// One projected corner: fixed-point screen position, depth, a value to
/// interpolate (light), and 1 / depth for perspective.
#[derive(Clone, Copy)]
pub(crate) struct Corner {
    pub(crate) x: i64,
    pub(crate) y: i64,
    pub(crate) z: f64,
    pub(crate) light: f64,
    pub(crate) iz: f64,
}

/// A covered pixel.
pub(crate) struct Frag {
    /// Pixel index.
    pub(crate) i: usize,
    /// Depth.
    pub(crate) z: f64,
    /// Interpolated light.
    pub(crate) light: f64,
    /// The corners' weights, in the order the corners were given.
    pub(crate) w: [f64; 3],
    /// The weights one pixel right and one pixel down (past the edges
    /// too), when asked for; else zero.
    pub(crate) wx: [f64; 3],
    pub(crate) wy: [f64; 3],
}

/// Whether pixel centres exactly on the edge `a → b` belong to the
/// triangle (the top-left rule, for counter-clockwise area > 0 in y-down
/// screen space). Opposite directions disagree, so a shared edge's pixels
/// go to one side only.
pub(crate) fn owns(dx: i64, dy: i64) -> bool {
    dy < 0 || (dy == 0 && dx > 0)
}

pub(crate) fn edge(a: &Corner, b: &Corner, px: i64, py: i64) -> i64 {
    (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)
}

/// Perspective-correct weights from screen barycentrics.
pub(crate) fn correct(l: [f64; 3], c: &[Corner; 3]) -> [f64; 3] {
    let a = [l[0] * c[0].iz, l[1] * c[1].iz, l[2] * c[2].iz];
    let s = a[0] + a[1] + a[2];
    if s == 0.0 {
        return l;
    }
    [a[0] / s, a[1] / s, a[2] / s]
}

/// Covers a triangle and calls `plot` for each pixel it covers.
/// `perspective` corrects the weights by 1 / depth; `derivs` fills
/// [`Frag::wx`] and [`Frag::wy`].
pub(crate) fn raster(
    size: (u32, u32),
    mut c: [Corner; 3],
    perspective: bool,
    derivs: bool,
    mut plot: impl FnMut(&Frag),
) {
    let mut area = edge(&c[0], &c[1], c[2].x, c[2].y);
    if area == 0 {
        return;
    }
    let swapped = area < 0;
    if swapped {
        c.swap(1, 2);
        area = -area;
    }
    // Back to the caller's corner order.
    let order = |w: [f64; 3]| if swapped { [w[0], w[2], w[1]] } else { w };
    let (w, h) = (i64::from(size.0), i64::from(size.1));
    let lo_x = c.iter().map(|v| v.x).min().unwrap_or(0);
    let hi_x = c.iter().map(|v| v.x).max().unwrap_or(0);
    let lo_y = c.iter().map(|v| v.y).min().unwrap_or(0);
    let hi_y = c.iter().map(|v| v.y).max().unwrap_or(0);
    // Pixels whose centre (px + 1/2) can lie inside.
    let x0 = (lo_x.div_euclid(SUB)).max(0);
    let x1 = (hi_x.div_euclid(SUB)).min(w - 1);
    let y0 = (lo_y.div_euclid(SUB)).max(0);
    let y1 = (hi_y.div_euclid(SUB)).min(h - 1);
    // Edge k is opposite corner k.
    let pairs = [(c[1], c[2]), (c[2], c[0]), (c[0], c[1])];
    let bias: [i64; 3] = pairs.map(|(a, b)| if owns(b.x - a.x, b.y - a.y) { 0 } else { 1 });
    // Each edge function's change one pixel right and one pixel down.
    let step_x: [i64; 3] = pairs.map(|(a, b)| -(b.y - a.y) * SUB);
    let step_y: [i64; 3] = pairs.map(|(a, b)| (b.x - a.x) * SUB);
    let inv = 1.0 / area as f64;
    let weights = |e: [i64; 3]| {
        let l = [e[0] as f64 * inv, e[1] as f64 * inv, e[2] as f64 * inv];
        order(if perspective { correct(l, &c) } else { l })
    };
    for py in y0..=y1 {
        let cy = py * SUB + SUB / 2;
        for px in x0..=x1 {
            let cx = px * SUB + SUB / 2;
            let e0 = edge(&c[1], &c[2], cx, cy);
            let e1 = edge(&c[2], &c[0], cx, cy);
            let e2 = edge(&c[0], &c[1], cx, cy);
            if e0 - bias[0] < 0 || e1 - bias[1] < 0 || e2 - bias[2] < 0 {
                continue;
            }
            let (l0, l1, l2) = (e0 as f64 * inv, e1 as f64 * inv, e2 as f64 * inv);
            let (z, light, wt) = if perspective {
                let k = correct([l0, l1, l2], &c);
                (
                    k[0] * c[0].z + k[1] * c[1].z + k[2] * c[2].z,
                    k[0] * c[0].light + k[1] * c[1].light + k[2] * c[2].light,
                    order(k),
                )
            } else {
                (
                    l0 * c[0].z + l1 * c[1].z + l2 * c[2].z,
                    l0 * c[0].light + l1 * c[1].light + l2 * c[2].light,
                    order([l0, l1, l2]),
                )
            };
            let (wx, wy) = if derivs {
                let e = [e0, e1, e2];
                (
                    weights([0, 1, 2].map(|k| e[k] + step_x[k])),
                    weights([0, 1, 2].map(|k| e[k] + step_y[k])),
                )
            } else {
                ([0.0; 3], [0.0; 3])
            };
            plot(&Frag {
                i: (py * w + px) as usize,
                z,
                light,
                w: wt,
                wx,
                wy,
            });
        }
    }
}

/// `v` in 0..=1 to a byte, rounding half up.
pub(crate) fn byte(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8
}

pub(crate) fn scale(rgb: [u8; 3], k: u32, d: u32) -> [u8; 3] {
    rgb.map(|c| (u32::from(c) * k / d) as u8)
}

/// `w`-weighted sum of three 2-vectors.
pub(crate) fn mix2(w: [f64; 3], v: &[DVec2; 3]) -> DVec2 {
    v[0] * w[0] + v[1] * w[1] + v[2] * w[2]
}

/// The part of a polygon in the plane z = const inside the axis-aligned
/// square `lo`..`hi` (x and y), by Sutherland–Hodgman.
pub(crate) fn clip_square(poly: &[DVec3], lo: DVec3, hi: DVec3) -> Vec<DVec3> {
    let mut out: Vec<DVec3> = poly.to_vec();
    // (axis, bound, keep below the bound)
    for (axis, bound, below) in [
        (0, lo.x, false),
        (0, hi.x, true),
        (1, lo.y, false),
        (1, hi.y, true),
    ] {
        let inside = |p: DVec3| {
            let v = if axis == 0 { p.x } else { p.y };
            if below { v <= bound } else { v >= bound }
        };
        let cut = |a: DVec3, b: DVec3| {
            let (va, vb) = if axis == 0 { (a.x, b.x) } else { (a.y, b.y) };
            a + (b - a) * ((bound - va) / (vb - va))
        };
        let input = std::mem::take(&mut out);
        for (i, &b) in input.iter().enumerate() {
            let a = input[(i + input.len() - 1) % input.len()];
            match (inside(a), inside(b)) {
                (true, true) => out.push(b),
                (true, false) => out.push(cut(a, b)),
                (false, true) => {
                    out.push(cut(a, b));
                    out.push(b);
                }
                (false, false) => {}
            }
        }
        if out.is_empty() {
            break;
        }
    }
    out
}
