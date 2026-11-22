//! The thin-part facts beside a silhouette's IoU (`og model compare`):
//! how wide each part is in the image's pixels, and how much IoU a
//! one-pixel misfit costs at that size, so a low IoU on thin legs reads
//! as the view's resolution, not as a wrong shape.
//!
//! A part's width is the diameter of the largest disc inside the pixels
//! it shows (the thickness of a leg, a rail or a slat; an even width
//! reads one pixel less), from the
//! Euclidean distance to the nearest pixel of another part or of the
//! background (Felzenszwalb and Huttenlocher's two-pass transform). A
//! part narrower than two pixels has no inside the silhouette can sample
//! (two samples across it, the sampling limit): `resolved` is false.

/// Per part, `(width, edge share)` over `labels` (`w` × `h`, each pixel's
/// part or none): the width in pixels (0 for a part with no pixel), and
/// the share of its pixels on its edge, which a one-pixel misfit along
/// the edge would lose.
pub fn widths(labels: &[Option<usize>], w: usize, h: usize, parts: usize) -> Vec<(f64, f64)> {
    let at = |x: isize, y: isize| {
        (x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h)
            .then(|| labels[y as usize * w + x as usize])
            .flatten()
    };
    // Every pixel that is no part's, or touches another label, is an edge.
    let mut f: Vec<f64> = (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as isize, (i / w) as isize);
            let me = labels[i];
            let edge = me.is_none()
                || [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|(dx, dy)| at(x + dx, y + dy) != me);
            if edge { 0.0 } else { f64::INFINITY }
        })
        .collect();
    squared_distances(&mut f, w, h);
    let mut out = vec![(0.0f64, 0usize, 0usize); parts];
    for (i, l) in labels.iter().enumerate() {
        let Some(k) = l.filter(|&k| k < parts) else {
            continue;
        };
        let o = &mut out[k];
        o.0 = o.0.max(f[i].sqrt());
        o.1 += 1;
        o.2 += usize::from(f[i] == 0.0);
    }
    out.into_iter()
        .map(|(r, n, e)| {
            if n == 0 {
                (0.0, 0.0)
            } else {
                // An edge pixel stands at 0 and the centre r from both
                // sides, so the disc spans 2r plus the centre pixel.
                (2.0 * r + 1.0, e as f64 / n as f64)
            }
        })
        .collect()
}

/// The IoU of `mask` (`w` × `h`) against itself grown by one pixel: the
/// most a silhouette this size keeps when it is off by one pixel all
/// round.
pub fn iou_one_pixel(mask: &[bool], w: usize, h: usize) -> f64 {
    let on = |x: isize, y: isize| {
        x >= 0
            && y >= 0
            && (x as usize) < w
            && (y as usize) < h
            && mask[y as usize * w + x as usize]
    };
    let (mut inside, mut grown) = (0usize, 0usize);
    for y in 0..h as isize {
        for x in 0..w as isize {
            let me = on(x, y);
            inside += usize::from(me);
            grown += usize::from(
                me || [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|(dx, dy)| on(x + dx, y + dy)),
            );
        }
    }
    if grown == 0 {
        1.0
    } else {
        inside as f64 / grown as f64
    }
}

/// `f` (0 at a feature, infinite elsewhere) becomes each pixel's squared
/// distance to the nearest feature: the 1D transform down each column,
/// then along each row.
fn squared_distances(f: &mut [f64], w: usize, h: usize) {
    let mut line = Vec::new();
    for x in 0..w {
        line.clear();
        line.extend((0..h).map(|y| f[y * w + x]));
        let d = lower_envelope(&line);
        for y in 0..h {
            f[y * w + x] = d[y];
        }
    }
    for y in 0..h {
        let d = lower_envelope(&f[y * w..(y + 1) * w]);
        f[y * w..(y + 1) * w].copy_from_slice(&d);
    }
}

/// The 1D squared distance transform of `f`: min over q of (p − q)² + f(q),
/// the lower envelope of the parabolas rooted at each finite sample.
fn lower_envelope(f: &[f64]) -> Vec<f64> {
    let n = f.len();
    let roots: Vec<usize> = (0..n).filter(|&q| f[q].is_finite()).collect();
    if roots.is_empty() {
        return vec![f64::INFINITY; n];
    }
    let cross = |q: usize, v: usize| {
        let (q2, v2) = ((q * q) as f64, (v * v) as f64);
        ((f[q] + q2) - (f[v] + v2)) / (2.0 * (q as f64 - v as f64))
    };
    // z[0] is −∞, so the first parabola is never popped.
    let (mut v, mut z) = (vec![roots[0]], vec![f64::NEG_INFINITY]);
    for &q in &roots[1..] {
        let mut s = cross(q, *v.last().unwrap());
        while s <= *z.last().unwrap() {
            v.pop();
            z.pop();
            s = cross(q, *v.last().unwrap());
        }
        v.push(q);
        z.push(s);
    }
    let mut d = vec![0.0; n];
    let mut k = 0;
    for (p, out) in d.iter_mut().enumerate() {
        while k + 1 < z.len() && z[k + 1] < p as f64 {
            k += 1;
        }
        let q = v[k];
        let dp = p as f64 - q as f64;
        *out = dp * dp + f[q];
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 40 × 20 image: a 3-pixel-wide leg (part 0) beside a 12-pixel
    /// block (part 1), touching it.
    fn leg_and_block() -> (Vec<Option<usize>>, usize, usize) {
        let (w, h) = (40, 20);
        let mut l = vec![None; w * h];
        for y in 2..18 {
            for x in 5..8 {
                l[y * w + x] = Some(0);
            }
            for x in 8..20 {
                l[y * w + x] = Some(1);
            }
        }
        (l, w, h)
    }

    #[test]
    fn a_parts_width_is_its_thickness_in_pixels_beside_another_part() {
        let (l, w, h) = leg_and_block();
        let got = widths(&l, w, h, 3);
        // The leg is 3 wide even where it touches the block; the block's
        // 12 reads as 11 (an even width has no centre pixel).
        assert_eq!(got[0].0, 3.0, "{got:?}");
        assert_eq!(got[1].0, 11.0, "{got:?}");
        // Every pixel of a 3-wide leg but its middle column is on its edge.
        assert!(
            (got[0].1 - (16.0 * 2.0 + 2.0) / 48.0).abs() < 1e-12,
            "{got:?}"
        );
        // A part that shows nothing has no width.
        assert_eq!(got[2], (0.0, 0.0));
    }

    #[test]
    fn a_one_pixel_misfit_costs_a_thin_silhouette_more() {
        let (w, h) = (40, 40);
        let bar = |width: usize| {
            let mut m = vec![false; w * h];
            for y in 5..35 {
                for x in 10..10 + width {
                    m[y * w + x] = true;
                }
            }
            m
        };
        // 30 × 2 grows to 32 × 4: 60 / 124. 30 × 20 grows to 22 wide:
        // 600 / (32 × 22 − 4 corners).
        assert!((iou_one_pixel(&bar(2), w, h) - 60.0 / 124.0).abs() < 1e-12);
        assert!((iou_one_pixel(&bar(20), w, h) - 600.0 / 700.0).abs() < 1e-12);
    }
}
