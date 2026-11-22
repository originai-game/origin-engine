//! The low mesh's UV triangles rasterized onto the atlas: each texel's triangle and weights, and its tangent frame.

use super::*;

/// Barycentric weights of `p` in the 2D triangle `t` (unclamped); a
/// degenerate triangle gives its first corner.
pub(crate) fn bary2(p: DVec2, t: [DVec2; 3]) -> [f64; 3] {
    let (v0, v1, v2) = (t[1] - t[0], t[2] - t[0], p - t[0]);
    let den = v0.perp_dot(v1);
    if den == 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let v = v2.perp_dot(v1) / den;
    let w = v0.perp_dot(v2) / den;
    [1.0 - v - w, v, w]
}

/// The point of the 2D triangle `t` nearest `p`.
pub(crate) fn closest_2d(p: DVec2, t: [DVec2; 3]) -> DVec2 {
    let lift = |q: DVec2| DVec3::new(q.x, q.y, 0.0);
    let q = closest_on_triangle(lift(p), t.map(lift));
    DVec2::new(q.x, q.y)
}

/// The frame at barycentric `b` of a triangle's corner frames: normal and
/// tangent interpolated and normalised, the tangent made square to the
/// normal, `w` the sign of the interpolated handedness (+1 on a tie).
#[doc(hidden)]
pub fn frame_at(f: &[CornerFrame; 3], b: [f64; 3]) -> CornerFrame {
    let n = unit(f[0].normal * b[0] + f[1].normal * b[1] + f[2].normal * b[2]);
    let t = f[0].tangent * b[0] + f[1].tangent * b[1] + f[2].tangent * b[2];
    let mut t = unit(t - n * n.dot(t));
    if t == DVec3::ZERO {
        t = f[0].tangent;
    }
    let w = f[0].w * b[0] + f[1].w * b[1] + f[2].w * b[2];
    CornerFrame {
        normal: n,
        tangent: t,
        w: if w < 0.0 { -1.0 } else { 1.0 },
    }
}

/// Each texel's owning triangle (`u32::MAX` for none) and its barycentric
/// weights there, with whether it was taken conservatively (by its square,
/// at the nearest point of the triangle to its centre). Texel `y · size +
/// x`, rows from the bottom (v up).
#[allow(clippy::type_complexity)]
#[doc(hidden)]
pub fn rasterize(uv: &[[DVec2; 3]], size: u32) -> (Vec<u32>, Vec<([f64; 3], bool)>) {
    let n = size as usize;
    let sz = f64::from(size);
    let mut owner = vec![u32::MAX; n * n];
    let mut at = vec![([0.0; 3], false); n * n];
    let span = |a: f64, b: f64, grow: f64| {
        let x0 = (a * sz - 0.5 - grow).ceil().max(0.0) as i64;
        let x1 = (b * sz - 0.5 + grow).floor().min(sz - 1.0) as i64;
        (x0, x1)
    };
    // Centres inside, exact, the top-left rule on edges, the lower index
    // first on an overlap.
    for (i, t) in uv.iter().enumerate() {
        let t = match orient2d(t[0], t[1], t[2]) {
            Ordering::Greater => *t,
            Ordering::Less => [t[0], t[2], t[1]],
            Ordering::Equal => continue,
        };
        let lo = t[0].min(t[1]).min(t[2]);
        let hi = t[0].max(t[1]).max(t[2]);
        let (x0, x1) = span(lo.x, hi.x, 0.0);
        let (y0, y1) = span(lo.y, hi.y, 0.0);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let k = y as usize * n + x as usize;
                if owner[k] != u32::MAX {
                    continue;
                }
                let p = DVec2::new((x as f64 + 0.5) / sz, (y as f64 + 0.5) / sz);
                let inside = (0..3).all(|e| {
                    let (a, b) = (t[e], t[(e + 1) % 3]);
                    match orient2d(a, b, p) {
                        Ordering::Greater => true,
                        Ordering::Equal => takes(a, b),
                        Ordering::Less => false,
                    }
                });
                if inside {
                    owner[k] = i as u32;
                    at[k] = (bary2(p, uv[i]), false);
                }
            }
        }
    }
    // Squares overlapped, centres held by none: the nearest triangle.
    let mut near = vec![f64::INFINITY; n * n];
    let mut best: Vec<(usize, u32, DVec2)> = Vec::new();
    let mut slot = vec![u32::MAX; n * n];
    for (i, t) in uv.iter().enumerate() {
        if orient2d(t[0], t[1], t[2]) == Ordering::Equal {
            continue;
        }
        let lo = t[0].min(t[1]).min(t[2]);
        let hi = t[0].max(t[1]).max(t[2]);
        let (x0, x1) = span(lo.x, hi.x, 1.0);
        let (y0, y1) = span(lo.y, hi.y, 1.0);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let k = y as usize * n + x as usize;
                if owner[k] != u32::MAX {
                    continue;
                }
                let (fx, fy) = (x as f64 / sz, y as f64 / sz);
                let e = 1.0 / sz;
                let w = DVec2::new;
                let sq1 = [w(fx, fy), w(fx + e, fy), w(fx + e, fy + e)];
                let sq2 = [w(fx, fy), w(fx + e, fy + e), w(fx, fy + e)];
                if !(tris_overlap_2d(*t, sq1) || tris_overlap_2d(*t, sq2)) {
                    continue;
                }
                let c = w(fx + e / 2.0, fy + e / 2.0);
                let q = closest_2d(c, *t);
                let d2 = (q - c).length_squared();
                if d2 < near[k] {
                    near[k] = d2;
                    if slot[k] == u32::MAX {
                        slot[k] = best.len() as u32;
                        best.push((k, i as u32, q));
                    } else {
                        best[slot[k] as usize] = (k, i as u32, q);
                    }
                }
            }
        }
    }
    for (k, i, q) in best {
        owner[k] = i;
        let b = bary2(q, uv[i as usize]);
        at[k] = (b.map(|v| v.clamp(0.0, 1.0)), true);
    }
    (owner, at)
}
