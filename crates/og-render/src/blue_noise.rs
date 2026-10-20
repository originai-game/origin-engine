//! The blue-noise threshold tile `DitherPattern::BlueNoise` reads
//! (stylise.wgsl `dither_at`): 64 × 64 thresholds, each of the 256 levels
//! on 16 texels, laid out by void and cluster (Ulichney 1993) so every
//! level's texels are spread as evenly as the tile allows and the tile
//! repeats without a seam. Made once by [`tests::void_and_cluster`] (a
//! deterministic routine: integer energies, a fixed seed, ties to the
//! lowest index) and compiled in as its 4096 bytes, uploaded as one
//! `R8Unorm` texture with the post passes; the test checks the committed
//! bytes are the routine's (`OG_BLESS_BLUE_NOISE=1` writes them again).

/// The tile's side in texels.
pub(crate) const BLUE_NOISE_SIZE: u32 = 64;

/// The thresholds, row-major, one byte a texel (`R8Unorm`).
pub(crate) static BLUE_NOISE: &[u8; (BLUE_NOISE_SIZE * BLUE_NOISE_SIZE) as usize] =
    include_bytes!("../data/blue-noise-64.r8");

#[cfg(test)]
mod tests {
    use super::*;

    /// A 64-bit xorshift* step.
    fn next(state: &mut u64) -> u64 {
        *state ^= *state >> 12;
        *state ^= *state << 25;
        *state ^= *state >> 27;
        state.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// Void and cluster on an `n` × `n` torus with a Gaussian of `sigma`
    /// texels: the rank of every texel (0 first), as thresholds of 256
    /// levels.
    pub(super) fn void_and_cluster(n: usize, sigma: f64, seed: u64) -> Vec<u8> {
        let count = n * n;
        // The filter as integers over wrapped offsets, so sums are exact.
        let mut lut = vec![0u64; count];
        for dy in 0..n {
            for dx in 0..n {
                let wx = dx.min(n - dx) as f64;
                let wy = dy.min(n - dy) as f64;
                let f = (-(wx * wx + wy * wy) / (2.0 * sigma * sigma)).exp();
                lut[dy * n + dx] = (f * (1u64 << 24) as f64).round() as u64;
            }
        }
        let offset = |a: usize, b: usize| {
            let (ax, ay) = (a % n, a / n);
            let (bx, by) = (b % n, b / n);
            ((ax + n - bx) % n) + ((ay + n - by) % n) * n
        };
        let add = |energy: &mut [u64], at: usize| {
            for (p, e) in energy.iter_mut().enumerate() {
                *e += lut[offset(p, at)];
            }
        };
        let sub = |energy: &mut [u64], at: usize| {
            for (p, e) in energy.iter_mut().enumerate() {
                *e -= lut[offset(p, at)];
            }
        };
        // The tightest cluster: the set texel of most energy; the largest
        // void: the empty texel of least. Ties go to the lowest index.
        let tightest = |set: &[bool], energy: &[u64], want: bool| {
            let mut best = usize::MAX;
            for p in 0..count {
                if set[p] == want && (best == usize::MAX || energy[p] > energy[best]) {
                    best = p;
                }
            }
            best
        };
        let largest_void = |set: &[bool], energy: &[u64]| {
            let mut best = usize::MAX;
            for p in 0..count {
                if !set[p] && (best == usize::MAX || energy[p] < energy[best]) {
                    best = p;
                }
            }
            best
        };

        // A random tenth of the texels, then moved from clusters into voids
        // until the move would put a texel back where it was.
        let mut state = seed;
        let mut set = vec![false; count];
        let ones = count / 10;
        let mut placed = 0;
        while placed < ones {
            let p = (next(&mut state) % count as u64) as usize;
            if !set[p] {
                set[p] = true;
                placed += 1;
            }
        }
        let mut energy = vec![0u64; count];
        for (p, _) in set.iter().enumerate().filter(|(_, on)| **on) {
            add(&mut energy, p);
        }
        loop {
            let c = tightest(&set, &energy, true);
            set[c] = false;
            sub(&mut energy, c);
            let v = largest_void(&set, &energy);
            set[v] = true;
            add(&mut energy, v);
            if v == c {
                break;
            }
        }
        let initial = set.clone();
        let initial_energy = energy.clone();
        let mut rank = vec![0usize; count];

        // Phase 1: the initial texels ranked by removing the tightest cluster.
        for r in (0..ones).rev() {
            let c = tightest(&set, &energy, true);
            set[c] = false;
            sub(&mut energy, c);
            rank[c] = r;
        }
        // Phase 2: up to half full, filling the largest void.
        let mut set = initial;
        let mut energy = initial_energy;
        for r in ones..count / 2 {
            let v = largest_void(&set, &energy);
            set[v] = true;
            add(&mut energy, v);
            rank[v] = r;
        }
        // Phase 3: past half, the empty texels are the minority: fill the
        // tightest cluster of them (by their own energy).
        let mut empty = vec![0u64; count];
        for (p, _) in set.iter().enumerate().filter(|(_, on)| !**on) {
            add(&mut empty, p);
        }
        for r in count / 2..count {
            let c = tightest(&set, &empty, false);
            set[c] = true;
            sub(&mut empty, c);
            rank[c] = r;
        }
        rank.iter().map(|&r| (r * 256 / count) as u8).collect()
    }

    fn generated() -> Vec<u8> {
        void_and_cluster(BLUE_NOISE_SIZE as usize, 1.5, 0x6f67_626c_7565)
    }

    #[test]
    fn the_tile_is_the_generators() {
        let tile = generated();
        if std::env::var_os("OG_BLESS_BLUE_NOISE").is_some() {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/blue-noise-64.r8");
            std::fs::write(path, &tile).expect("write the tile");
        }
        assert!(
            tile.as_slice() == BLUE_NOISE.as_slice(),
            "data/blue-noise-64.r8 is not void_and_cluster's output (OG_BLESS_BLUE_NOISE=1 writes it)"
        );
    }

    #[test]
    fn every_level_is_used_equally_and_spread_evenly() {
        let mut histogram = [0u32; 256];
        for &v in BLUE_NOISE.iter() {
            histogram[v as usize] += 1;
        }
        assert!(histogram.iter().all(|&c| c == 16), "{histogram:?}");
        // Blue noise is locally even: every 8 × 8 block (on the torus, at
        // every offset) averages close to the middle (8.6 levels at worst),
        // where the same values shuffled stray by 31.
        let n = BLUE_NOISE_SIZE as usize;
        let mut worst = 0.0f64;
        for oy in 0..n {
            for ox in 0..n {
                let mut sum = 0u32;
                for y in 0..8 {
                    for x in 0..8 {
                        sum += BLUE_NOISE[((oy + y) % n) * n + (ox + x) % n] as u32;
                    }
                }
                worst = worst.max((sum as f64 / 64.0 - 127.5).abs());
            }
        }
        assert!(worst < 12.0, "an 8 × 8 block strays {worst:.2} levels");
        // Neighbours differ: no two 4-neighbours are within 8 levels.
        let mut close = 0;
        for y in 0..n {
            for x in 0..n {
                let a = BLUE_NOISE[y * n + x] as i32;
                let b = BLUE_NOISE[y * n + (x + 1) % n] as i32;
                let c = BLUE_NOISE[((y + 1) % n) * n + x] as i32;
                close += ((a - b).abs() < 8) as u32 + ((a - c).abs() < 8) as u32;
            }
        }
        // The same values shuffled: 492 pairs.
        assert_eq!(close, 0, "neighbour pairs within 8 levels");
    }
}
