//! The spatial grid that refuses a collapse folding the surface into a near neighbour.

use super::*;

/// A triangle near a fan: its index, corners, positions and box.
pub(crate) type Near = (u32, [u32; 3], [DVec3; 3], (DVec3, DVec3));

/// A triangle's box.
pub(crate) fn bounds(t: [DVec3; 3]) -> (DVec3, DVec3) {
    (t[0].min(t[1]).min(t[2]), t[0].max(t[1]).max(t[2]))
}

/// A spatial hash over the live triangles' boxes: each triangle is listed
/// in the buckets of the cells its box covers (cells about twice a mean
/// triangle across); a triangle covering more than [`Grid::MAX_CELLS`]
/// cells is listed apart and returned by every query. Rebuilt with larger
/// cells each time the triangle count halves, so a cell keeps a few
/// triangles. Queries return candidates sorted, so nothing depends on
/// the order triangles came and went.
pub(crate) struct Grid {
    pub(crate) lo: DVec3,
    pub(crate) h: f64,
    pub(crate) buckets: Vec<Vec<u32>>,
    pub(crate) big: Vec<u32>,
    /// Live triangles when built.
    pub(crate) built: usize,
}

impl Grid {
    /// Cells a triangle may cover and still be listed in them.
    pub(crate) const MAX_CELLS: i64 = 512;
    /// Cells a query may visit before it scans every bucket instead.
    pub(crate) const MAX_QUERY: i64 = 8192;

    pub(crate) fn new(m: &Mesh) -> Grid {
        let live: Vec<u32> = (0..m.tris.len() as u32)
            .filter(|&t| m.alive[t as usize])
            .collect();
        let (mut lo, mut hi) = (DVec3::INFINITY, DVec3::NEG_INFINITY);
        let mut extent = 0.0;
        for &t in &live {
            let (a, b) = bounds(m.tris[t as usize].map(|v| m.pos[v as usize]));
            lo = lo.min(a);
            hi = hi.max(b);
            extent += (b - a).max_element();
        }
        let span = (hi - lo).max_element();
        let mut h = 2.0 * extent / live.len().max(1) as f64;
        // At most about 2^20 cells a side.
        if !(h > 0.0 && h.is_finite()) || span / h > (1u64 << 20) as f64 {
            h = (span / (1u64 << 20) as f64).max(f64::MIN_POSITIVE);
        }
        let mut g = Grid {
            lo: if lo.is_finite() { lo } else { DVec3::ZERO },
            h,
            buckets: vec![Vec::new(); (2 * live.len()).max(16).next_power_of_two()],
            big: Vec::new(),
            built: live.len(),
        };
        for t in live {
            g.insert(m, t);
        }
        g
    }

    pub(crate) fn cells(&self, lo: DVec3, hi: DVec3, max: i64) -> Option<([i64; 3], [i64; 3])> {
        let c = |p: DVec3| {
            let q = ((p - self.lo) / self.h).floor();
            [q.x as i64, q.y as i64, q.z as i64]
        };
        let (a, b) = (c(lo), c(hi));
        let n = (0..3).fold(1i64, |n, k| n.saturating_mul(b[k] - a[k] + 1));
        (n <= max).then_some((a, b))
    }

    pub(crate) fn bucket(&self, c: [i64; 3]) -> usize {
        let h = (c[0] as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (c[1] as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
            ^ (c[2] as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
        ((h ^ (h >> 29)) as usize) & (self.buckets.len() - 1)
    }

    pub(crate) fn each_bucket(
        &self,
        lo: DVec3,
        hi: DVec3,
        max: i64,
        mut f: impl FnMut(usize),
    ) -> bool {
        let Some((a, b)) = self.cells(lo, hi, max) else {
            return false;
        };
        for x in a[0]..=b[0] {
            for y in a[1]..=b[1] {
                for z in a[2]..=b[2] {
                    f(self.bucket([x, y, z]));
                }
            }
        }
        true
    }

    pub(crate) fn insert(&mut self, m: &Mesh, t: u32) {
        let (lo, hi) = bounds(m.tris[t as usize].map(|v| m.pos[v as usize]));
        let mut at = Vec::new();
        if self.each_bucket(lo, hi, Self::MAX_CELLS, |b| at.push(b)) {
            for b in at {
                self.buckets[b].push(t);
            }
        } else {
            self.big.push(t);
        }
    }

    /// Takes `t` out, with the box it was inserted with.
    pub(crate) fn remove(&mut self, m: &Mesh, t: u32) {
        let (lo, hi) = bounds(m.tris[t as usize].map(|v| m.pos[v as usize]));
        let mut at = Vec::new();
        if self.each_bucket(lo, hi, Self::MAX_CELLS, |b| at.push(b)) {
            for b in at {
                self.buckets[b].retain(|&x| x != t);
            }
        } else {
            self.big.retain(|&x| x != t);
        }
    }

    /// The triangles listed near the box, sorted, each once.
    pub(crate) fn near(&self, lo: DVec3, hi: DVec3, out: &mut Vec<u32>) {
        let mut at = Vec::new();
        if self.each_bucket(lo, hi, Self::MAX_QUERY, |b| at.push(b)) {
            for b in at {
                out.extend(&self.buckets[b]);
            }
            out.extend(&self.big);
        } else {
            // A box over many cells: every triangle.
            for b in &self.buckets {
                out.extend(b);
            }
            out.extend(&self.big);
        }
        out.sort_unstable();
        out.dedup();
    }
}
