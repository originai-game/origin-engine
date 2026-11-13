//! Sampling a function of the life (t 0 to 1) into at most `max` keys: the
//! ends and the times that must stay (a fade's corner), then the sample the
//! keys so far miss most, until they all fall within `tolerance` or the keys
//! run out. Deterministic: fixed samples, ties to the earliest.

const SAMPLES: usize = 64;

/// `(t, value)` keys of `f` (any number of channels), sorted by t.
pub fn keys<const N: usize>(
    f: impl Fn(f32) -> [f32; N],
    keep: &[f32],
    max: usize,
    tolerance: f32,
) -> Vec<(f32, [f32; N])> {
    let mut ts: Vec<f32> = vec![0.0, 1.0];
    for &t in keep {
        if t > 0.0 && t < 1.0 && ts.iter().all(|k| (k - t).abs() > 1e-3) {
            ts.push(t);
        }
    }
    og_core::sort::sort_by_key(&mut ts, |t| t.to_bits());
    ts.truncate(max.max(2));
    let samples: Vec<(f32, [f32; N])> = (0..=SAMPLES)
        .map(|i| {
            let t = i as f32 / SAMPLES as f32;
            (t, f(t))
        })
        .collect();
    loop {
        let keys: Vec<(f32, [f32; N])> = ts.iter().map(|&t| (t, f(t))).collect();
        if keys.len() >= max {
            return keys;
        }
        let mut worst = (tolerance, None);
        for &(t, v) in &samples {
            let e = (0..N)
                .map(|c| (lerp(&keys, t, c) - v[c]).abs())
                .fold(0.0, f32::max);
            if e > worst.0 {
                worst = (e, Some(t));
            }
        }
        match worst.1 {
            Some(t) if ts.iter().all(|k| (k - t).abs() > 1e-4) => {
                ts.push(t);
                og_core::sort::sort_by_key(&mut ts, |t| t.to_bits());
            }
            _ => return keys,
        }
    }
}

fn lerp<const N: usize>(keys: &[(f32, [f32; N])], t: f32, c: usize) -> f32 {
    let i = keys.partition_point(|k| k.0 <= t);
    if i == 0 {
        return keys[0].1[c];
    }
    if i == keys.len() {
        return keys[i - 1].1[c];
    }
    let (a, b) = (keys[i - 1], keys[i]);
    a.1[c] + (b.1[c] - a.1[c]) * (t - a.0) / (b.0 - a.0).max(1e-6)
}

/// Whether every key holds the same value (within `eps`).
pub fn flat<const N: usize>(keys: &[(f32, [f32; N])], eps: f32) -> bool {
    keys.iter()
        .all(|k| (0..N).all(|c| (k.1[c] - keys[0].1[c]).abs() <= eps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_is_two_keys_and_a_corner_is_kept() {
        let k = keys(|t| [2.0 * t], &[], 8, 1e-3);
        assert_eq!(k.len(), 2);
        // A fade in over the first 10 %: its corner is a key.
        let k = keys(|t| [(t / 0.1).min(1.0)], &[0.1], 8, 1e-3);
        assert_eq!(k.iter().map(|k| k.0).collect::<Vec<_>>(), [0.0, 0.1, 1.0]);
        // A curve is approximated within the key budget.
        let k = keys(|t| [t * t], &[], 4, 1e-4);
        assert_eq!(k.len(), 4);
    }
}
