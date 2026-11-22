//! Charts packed into the unit square with padding.

use super::*;

/// Packs every chart at the largest scale that fits (bisection), tallest
/// first. Returns each chart's corner and the scale.
pub(crate) fn pack_all(sizes: &[DVec2], pad: f64) -> Option<(Vec<DVec2>, f64)> {
    let total: f64 = sizes.iter().map(|s| s.x * s.y).sum();
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by(|&a, &b| {
        sizes[b]
            .y
            .total_cmp(&sizes[a].y)
            .then(sizes[b].x.total_cmp(&sizes[a].x))
            .then(a.cmp(&b))
    });
    let fit = |scale: f64| pack(sizes, &order, scale, pad);
    let mut hi = if total > 0.0 {
        (1.0 / total).sqrt()
    } else {
        let longest = sizes.iter().map(|s| s.x.max(s.y)).fold(0.0, f64::max);
        if longest > 0.0 { 1.0 / longest } else { 1.0 }
    };
    let mut best = (fit(0.0)?, 0.0);
    if let Some(p) = fit(hi) {
        return Some((p, hi));
    }
    let mut lo = 0.0;
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        match fit(mid) {
            Some(p) => {
                best = (p, mid);
                lo = mid;
            }
            None => hi = mid,
        }
    }
    Some(best)
}

/// Places every chart at `scale` by a bottom-left skyline in the unit
/// square; each takes its size × scale plus `pad`. `None` if they do not
/// all fit.
pub(crate) fn pack(sizes: &[DVec2], order: &[usize], scale: f64, pad: f64) -> Option<Vec<DVec2>> {
    // The skyline: (x, height) steps, each running to the next one's x.
    let mut sky: Vec<(f64, f64)> = vec![(0.0, 0.0)];
    let mut at = vec![DVec2::ZERO; sizes.len()];
    for &c in order {
        let (w, h) = (sizes[c].x * scale + pad, sizes[c].y * scale + pad);
        if w > 1.0 || h > 1.0 {
            return None;
        }
        let mut best: Option<(f64, f64, usize)> = None;
        for i in 0..sky.len() {
            let x = sky[i].0;
            if x + w > 1.0 {
                break;
            }
            let mut y: f64 = 0.0;
            let mut j = i;
            while j < sky.len() && sky[j].0 < x + w {
                y = y.max(sky[j].1);
                j += 1;
            }
            if y + h > 1.0 {
                continue;
            }
            if best.is_none_or(|(by, bx, _)| y < by || (y == by && x < bx)) {
                best = Some((y, x, i));
            }
        }
        let (y, x, i) = best?;
        at[c] = DVec2::new(x, y);
        // Raise [x, x + w) to y + h.
        let end = x + w;
        let mut j = i;
        let mut after = sky[i].1;
        while j < sky.len() && sky[j].0 < end {
            after = sky[j].1;
            j += 1;
        }
        let mut next: Vec<(f64, f64)> = sky[..i].to_vec();
        next.push((x, y + h));
        if j < sky.len() && sky[j].0 == end {
            // The next step starts exactly where this one ends.
        } else if end < 1.0 {
            next.push((end, after));
        }
        next.extend_from_slice(&sky[j..]);
        // Merge equal neighbours.
        next.dedup_by(|b, a| a.1 == b.1);
        sky = next;
    }
    Some(at)
}
