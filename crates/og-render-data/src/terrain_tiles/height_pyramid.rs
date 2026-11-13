//! A height field's coarser levels, and the tile heights and horizons read from them.

use super::*;

/// A field's coarser levels (1 up): each level's samples, the finer
/// level's under a centred tent filter, clamped at the field's edges; and
/// beside them the highest field height under each sample's footprint
/// (the same 3 × 3 of the finer level), which the horizon reads.
#[derive(Clone, Debug, PartialEq)]
pub struct HeightPyramid {
    /// Level k + 1's samples along x and z and its heights, row-major.
    pub levels: Vec<(u32, u32, Vec<f32>)>,
    /// Level k + 1's maxima, laid out as its heights.
    pub maxima: Vec<Vec<f32>>,
}

impl HeightPyramid {
    /// The pyramid of a field read as `field(column, row)`, for `layout`.
    pub fn new(layout: &TerrainLayout, field: impl Fn(u32, u32) -> f32) -> HeightPyramid {
        let mut levels: Vec<(u32, u32, Vec<f32>)> = Vec::new();
        let mut maxima: Vec<Vec<f32>> = Vec::new();
        let (mut fx, mut fz) = (layout.columns as i64, layout.rows as i64);
        for k in 1..layout.levels {
            let (sx, sz) = layout.samples(k);
            let samples = (0..sz as i64).flat_map(|z| (0..sx as i64).map(move |x| (x, z)));
            let out = {
                let at = Self::reader(&field, levels.last(), fx, fz);
                samples
                    .clone()
                    .map(|(x, z)| Self::tent(&at, fx, fz, x, z))
                    .collect()
            };
            let peaks = {
                let finer = levels
                    .last()
                    .map(|(w, h, _)| (*w, *h, maxima.last().expect("beside")));
                let at = Self::max_reader(&field, finer, fx, fz);
                samples
                    .map(|(x, z)| Self::peak(&at, fx, fz, x, z))
                    .collect()
            };
            levels.push((sx, sz, out));
            maxima.push(peaks);
            (fx, fz) = (sx as i64, sz as i64);
        }
        HeightPyramid { levels, maxima }
    }

    /// A finer level's maxima (the field's heights when `finer` is
    /// `None`), clamped at its edges.
    fn max_reader<'a>(
        field: &'a impl Fn(u32, u32) -> f32,
        finer: Option<(u32, u32, &'a Vec<f32>)>,
        fx: i64,
        fz: i64,
    ) -> impl Fn(i64, i64) -> f32 + 'a {
        move |i: i64, j: i64| {
            let (i, j) = (i.clamp(0, fx - 1), j.clamp(0, fz - 1));
            match finer {
                None => field(i as u32, j as u32),
                Some((w, _, h)) => h[(j * w as i64 + i) as usize],
            }
        }
    }

    /// The highest of the finer samples a coarser one's tent covers.
    fn peak(at: &impl Fn(i64, i64) -> f32, fx: i64, fz: i64, x: i64, z: i64) -> f32 {
        let ci = (2 * x).min(fx - 1);
        let cj = (2 * z).min(fz - 1);
        let mut v = f32::NEG_INFINITY;
        for b in -1..=1 {
            for a in -1..=1 {
                v = v.max(at(ci + a, cj + b));
            }
        }
        v
    }

    /// A finer level's heights (`fx` × `fz`; the field's when `finer` is
    /// `None`), clamped at its edges.
    fn reader<'a>(
        field: &'a impl Fn(u32, u32) -> f32,
        finer: Option<&'a (u32, u32, Vec<f32>)>,
        fx: i64,
        fz: i64,
    ) -> impl Fn(i64, i64) -> f32 + 'a {
        move |i: i64, j: i64| {
            let (i, j) = (i.clamp(0, fx - 1), j.clamp(0, fz - 1));
            match finer {
                None => field(i as u32, j as u32),
                Some((w, _, h)) => h[(j * *w as i64 + i) as usize],
            }
        }
    }

    /// A coarser sample (`x`, `z`): the finer level's (`at`, `fx` × `fz`)
    /// under it (every second, and the last at the last) under a centred
    /// tent.
    fn tent(at: &impl Fn(i64, i64) -> f32, fx: i64, fz: i64, x: i64, z: i64) -> f32 {
        const W: [f32; 3] = [0.25, 0.5, 0.25];
        let ci = (2 * x).min(fx - 1);
        let cj = (2 * z).min(fz - 1);
        let mut v = 0.0;
        for (b, wz) in W.iter().enumerate() {
            for (a, wx) in W.iter().enumerate() {
                v += wx * wz * at(ci + a as i64 - 1, cj + b as i64 - 1);
            }
        }
        v
    }

    /// Filters again what field samples `lo`..=`hi` (column, row) reach on
    /// every level after an edit inside them (`field`: the edited one):
    /// the pyramid is then [`HeightPyramid::new`]'s of the edited field.
    pub fn update(
        &mut self,
        layout: &TerrainLayout,
        field: impl Fn(u32, u32) -> f32,
        lo: (u32, u32),
        hi: (u32, u32),
    ) {
        let (mut lo, mut hi) = ((lo.0 as i64, lo.1 as i64), (hi.0 as i64, hi.1 as i64));
        let (mut fx, mut fz) = (layout.columns as i64, layout.rows as i64);
        for k in 1..layout.levels as usize {
            let (done, rest) = self.levels.split_at_mut(k - 1);
            let (sx, sz, out) = &mut rest[0];
            let (sx, sz) = (*sx as i64, *sz as i64);
            let at = Self::reader(&field, done.last(), fx, fz);
            let (peaks_done, peaks_rest) = self.maxima.split_at_mut(k - 1);
            let finer = done
                .last()
                .map(|(w, h, _)| (*w, *h, peaks_done.last().expect("beside")));
            let at_max = Self::max_reader(&field, finer, fx, fz);
            let peaks = &mut peaks_rest[0];
            // The samples whose tent (2x - 1 to 2x + 1 of the finer level,
            // the last one's clamped) reads a changed finer one.
            let reach = |a: i64, b: i64, n: i64| {
                (
                    (a - 1).div_euclid(2).clamp(0, n - 1),
                    ((b + 2) / 2).clamp(0, n - 1),
                )
            };
            let (x0, x1) = reach(lo.0, hi.0, sx);
            let (z0, z1) = reach(lo.1, hi.1, sz);
            for z in z0..=z1 {
                for x in x0..=x1 {
                    out[(z * sx + x) as usize] = Self::tent(&at, fx, fz, x, z);
                    peaks[(z * sx + x) as usize] = Self::peak(&at_max, fx, fz, x, z);
                }
            }
            (lo, hi, fx, fz) = ((x0, z0), (x1, z1), sx, sz);
        }
    }

    /// Level `k`'s height at its sample (`x`, `z`), clamped; level 0 is
    /// `field`'s.
    fn at(
        &self,
        field: &impl Fn(u32, u32) -> f32,
        layout: &TerrainLayout,
        k: u32,
        x: i64,
        z: i64,
    ) -> f32 {
        if k == 0 {
            let c = x.clamp(0, layout.columns as i64 - 1) as u32;
            let r = z.clamp(0, layout.rows as i64 - 1) as u32;
            return field(c, r);
        }
        let (sx, sz, h) = &self.levels[k as usize - 1];
        let (x, z) = (x.clamp(0, *sx as i64 - 1), z.clamp(0, *sz as i64 - 1));
        h[(z * *sx as i64 + x) as usize]
    }

    /// Level `k`'s height under field coordinates `s` (in field cells from
    /// the -x, -z corner): bilinear between its samples (every 2^k-th and
    /// the last, whose cell may be shorter: `terrain.wgsl` `level_coord`).
    fn height_at(
        &self,
        field: &impl Fn(u32, u32) -> f32,
        layout: &TerrainLayout,
        k: u32,
        s: Vec2,
    ) -> f32 {
        let step = layout.step(k) as f32;
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let (sx, sz) = layout.samples(k);
        let n = Vec2::new(sx as f32, sz as f32);
        let last = (n - 2.0) * step;
        let span = (quads - last).max(Vec2::splat(1e-6));
        let u = Vec2::select(s.cmpgt(last), n - 2.0 + (s - last) / span, s / step)
            .clamp(Vec2::ZERO, n - 1.0);
        let i = u.floor();
        let f = u - i;
        let (x, z) = (i.x as i64, i.y as i64);
        let h = |x: i64, z: i64| self.at(field, layout, k, x, z);
        let a = h(x, z) + (h(x + 1, z) - h(x, z)) * f.x;
        let b = h(x, z + 1) + (h(x + 1, z + 1) - h(x, z + 1)) * f.x;
        a + (b - a) * f.y
    }

    /// The highest ground near field coordinates `s` on level `k`: the
    /// largest of the four samples around `s` (past level 0, of their
    /// maxima: each the highest of the field a sample's spacing either
    /// side of it), so a peak between two reads is never lowered.
    fn max_at(
        &self,
        field: &impl Fn(u32, u32) -> f32,
        layout: &TerrainLayout,
        k: u32,
        s: Vec2,
    ) -> f32 {
        if k == 0 {
            let (x, z) = (s.x.floor() as i64, s.y.floor() as i64);
            let at = |x: i64, z: i64| self.at(field, layout, 0, x, z);
            return at(x, z)
                .max(at(x + 1, z))
                .max(at(x, z + 1))
                .max(at(x + 1, z + 1));
        }
        let step = layout.step(k) as f32;
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let (sx, sz) = layout.samples(k);
        let n = Vec2::new(sx as f32, sz as f32);
        let last = (n - 2.0) * step;
        let span = (quads - last).max(Vec2::splat(1e-6));
        let u = Vec2::select(s.cmpgt(last), n - 2.0 + (s - last) / span, s / step)
            .clamp(Vec2::ZERO, n - 1.0);
        let (x, z) = (u.x.floor() as i64, u.y.floor() as i64);
        let peaks = &self.maxima[k as usize - 1];
        let at = |x: i64, z: i64| {
            let (x, z) = (x.clamp(0, sx as i64 - 1), z.clamp(0, sz as i64 - 1));
            peaks[(z * sx as i64 + x) as usize]
        };
        at(x, z)
            .max(at(x + 1, z))
            .max(at(x, z + 1))
            .max(at(x + 1, z + 1))
    }

    /// A tile's horizon: at each of its `horizon_side()²` texels (every
    /// [`HORIZON_SPACING`]-th sample of its level and the last), the
    /// highest elevation of the ground seen from that point in each of
    /// [`HORIZON_AZIMUTHS`] directions (k · 45° from +x towards +z), out to
    /// the terrain's edge: a horizon map (Max, "Horizon mapping", 1988),
    /// marched at distances growing by 1/32 per step, each read on the
    /// pyramid's maxima at the level that covers the gap to the next (the
    /// far-field reads of Timonen and Westerholm, "Scalable Height Field
    /// Self-Shadowing", 2010, without their sweep's shared hull). Each
    /// angle is [`HORIZON_BITS`] over -90° to 90° (0 a horizon straight
    /// down, all ones straight up), two to a word from its low bits
    /// ([`HORIZON_WORDS`] words per texel), texel by texel row-major. Seen in the terrain's
    /// own space (its metres, before its transform).
    pub fn horizon(
        &self,
        layout: &TerrainLayout,
        index: u32,
        field: impl Fn(u32, u32) -> f32,
    ) -> Vec<u32> {
        // The march's steps grow by 1/GROWTH of the distance.
        const GROWTH: f32 = 32.0;
        let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
        let side = layout.horizon_side();
        let t = layout.tile as f32;
        let step = layout.step(k) as f32;
        let cell = layout.cell();
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let reach = quads.length();
        let dirs: [Vec2; HORIZON_AZIMUTHS] = std::array::from_fn(|a| {
            let angle = a as f32 * std::f32::consts::TAU / HORIZON_AZIMUTHS as f32;
            Vec2::new(angle.cos(), angle.sin())
        });
        let quantise = |tan: f32| {
            let angle = tan.atan() / std::f32::consts::FRAC_PI_2;
            let top = ((1 << HORIZON_BITS) - 1) as f32;
            ((angle * 0.5 + 0.5) * top).round().clamp(0.0, top) as u32
        };
        let mut out = Vec::with_capacity(layout.horizon_words() as usize);
        for j in 0..side {
            for i in 0..side {
                // The texel's place on its level, then in field cells.
                let g = Vec2::new(
                    tx as f32 * t + i as f32 * t / (side - 1) as f32,
                    tz as f32 * t + j as f32 * t / (side - 1) as f32,
                );
                let s = (g * step).min(quads);
                let h0 = self.height_at(&field, layout, k, s);
                let mut bytes = [0u32; HORIZON_AZIMUTHS];
                for (a, dir) in dirs.iter().enumerate() {
                    let metres = (*dir * cell).length();
                    let mut best = f32::NEG_INFINITY;
                    let mut d = step;
                    while d <= reach {
                        let q = s + *dir * d;
                        if q.x < 0.0 || q.y < 0.0 || q.x > quads.x || q.y > quads.y {
                            break;
                        }
                        // The finest level whose maxima (each a sample's
                        // spacing either side) cover the gap to the next
                        // step: a peak is never stepped over, and is seen
                        // at most an eighth of `d` nearer than it stands
                        // (below), so the horizon errs a little high.
                        let lv = ((d / (2.0 * GROWTH)).log2().ceil().max(0.0) as u32)
                            .min(layout.levels - 1);
                        let h = self.max_at(&field, layout, lv, q);
                        // As if at the nearest the read may have found it
                        // (its four samples' footprint), so a peak is never
                        // seen lower than it stands.
                        let near = (d - 2.0 * layout.step(lv) as f32).max(step.min(d));
                        best = best.max((h - h0) / (near * metres));
                        d = (d * (1.0 + 1.0 / GROWTH)).max(d + step);
                    }
                    bytes[a] = quantise(best);
                }
                for word in bytes.chunks(2) {
                    out.push(word.iter().rev().fold(0, |w, b| w << HORIZON_BITS | b));
                }
            }
        }
        out
    }

    /// A tile's `(tile + 3)²` heights: its level's samples (the field's at
    /// level 0), the border clamped at the terrain's edge.
    pub fn tile_heights(
        &self,
        layout: &TerrainLayout,
        index: u32,
        field: impl Fn(u32, u32) -> f32,
    ) -> Vec<f32> {
        let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
        let t = layout.tile as i64;
        let side = layout.heights_side() as i64;
        let mut out = Vec::with_capacity((side * side) as usize);
        for j in 0..side {
            for i in 0..side {
                let (gx, gz) = (tx as i64 * t + i - 1, tz as i64 * t + j - 1);
                out.push(if k == 0 {
                    let (sx, sz) = layout.samples(0);
                    let c = layout.fine(0, gx.clamp(0, sx as i64 - 1), layout.columns);
                    let r = layout.fine(0, gz.clamp(0, sz as i64 - 1), layout.rows);
                    field(c, r)
                } else {
                    self.at(&field, layout, k, gx, gz)
                });
            }
        }
        out
    }
}
