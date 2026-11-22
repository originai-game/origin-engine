//! Baked texels spread past chart borders so filtering never reads the empty atlas.

/// The dilation: each texel's layer (0 filled to begin with, `k` filled
/// by layer `k`, `u32::MAX` never) and the layers' texels in increasing
/// index. Layer `k` is the empty texels next (8-neighbours) to a texel of
/// a lower layer; `limit` layers (0: until none is left).
pub(crate) fn dilation(filled: &[bool], n: usize, limit: u32) -> (Vec<u32>, Vec<Vec<usize>>) {
    let mut level: Vec<u32> = filled
        .iter()
        .map(|&f| if f { 0 } else { u32::MAX })
        .collect();
    let mut layers: Vec<Vec<usize>> = Vec::new();
    let mut frontier: Vec<usize> = (0..n * n)
        .filter(|&k| level[k] == u32::MAX && neighbours(k, n).any(|j| level[j] == 0))
        .collect();
    while !frontier.is_empty() && (limit == 0 || (layers.len() as u32) < limit) {
        let l = layers.len() as u32 + 1;
        for &k in &frontier {
            level[k] = l;
        }
        let mut next: Vec<usize> = frontier
            .iter()
            .flat_map(|&k| neighbours(k, n))
            .filter(|&j| level[j] == u32::MAX)
            .collect();
        next.sort_unstable();
        next.dedup();
        layers.push(core::mem::replace(&mut frontier, next));
    }
    (level, layers)
}

/// The 8-neighbours of texel `k` inside the `n` × `n` grid.
pub(crate) fn neighbours(k: usize, n: usize) -> impl Iterator<Item = usize> {
    let (x, y) = ((k % n) as i64, (k / n) as i64);
    (-1..=1i64)
        .flat_map(|dy| (-1..=1i64).map(move |dx| (dx, dy)))
        .filter(|&(dx, dy)| dx != 0 || dy != 0)
        .filter_map(move |(dx, dy)| {
            let (xx, yy) = (x + dx, y + dy);
            (xx >= 0 && yy >= 0 && (xx as usize) < n && (yy as usize) < n)
                .then(|| yy as usize * n + xx as usize)
        })
}

/// Fills an image's texels layer by layer, each the rounded mean of its
/// neighbours of lower layers, channel by channel. Texel `y · n + x` is the
/// image's row `n − 1 − y`.
pub(crate) fn dilate(rgba: &mut [u8], n: usize, level: &[u32], layers: &[Vec<usize>]) {
    let px = |k: usize| ((n - 1 - k / n) * n + k % n) * 4;
    for (l, layer) in layers.iter().enumerate() {
        let l = l as u32 + 1;
        for &k in layer {
            let mut sum = [0u32; 4];
            let mut m = 0u32;
            for j in neighbours(k, n) {
                if level[j] < l {
                    m += 1;
                    for (c, s) in sum.iter_mut().enumerate() {
                        *s += u32::from(rgba[px(j) + c]);
                    }
                }
            }
            for (c, s) in sum.iter().enumerate() {
                rgba[px(k) + c] = ((s + m / 2) / m) as u8;
            }
        }
    }
}
