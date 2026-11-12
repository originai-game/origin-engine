use super::*;

/// [`TerrainData::height`] and [`TerrainData::normal`] over a field read
/// sample by sample (`sample(column, row)`, both in range): the runtime
/// samples a streamed terrain's decoded tiles with it, and gets the same
/// answers as from the whole field.
pub struct TerrainSampler<F: Fn(u32, u32) -> f32> {
    pub size: Vec2,
    pub columns: u32,
    pub rows: u32,
    pub sample: F,
}

impl TerrainSampler<fn(u32, u32) -> f32> {
    /// A sampler over a field in memory.
    pub fn of(d: &TerrainData) -> TerrainSampler<impl Fn(u32, u32) -> f32 + '_> {
        TerrainSampler {
            size: d.size,
            columns: d.columns,
            rows: d.rows,
            sample: move |c: u32, r: u32| d.heights[(r * d.columns + c) as usize],
        }
    }
}

impl<F: Fn(u32, u32) -> f32> TerrainSampler<F> {
    fn at(&self, c: u32, r: u32) -> f32 {
        (self.sample)(c.min(self.columns - 1), r.min(self.rows - 1))
    }

    fn cell(&self) -> Vec2 {
        self.size
            / Vec2::new(
                self.columns.max(2) as f32 - 1.0,
                self.rows.max(2) as f32 - 1.0,
            )
    }

    /// Sample coordinates of a point given relative to the terrain's centre.
    fn grid(&self, local: Vec2) -> Vec2 {
        let t = (local + self.size * 0.5) / self.cell();
        t.clamp(
            Vec2::ZERO,
            Vec2::new(self.columns as f32 - 1.0, self.rows as f32 - 1.0),
        )
    }

    /// The cell (column, row) under `local`: the quad whose -x, -z corner is
    /// that sample; `None` off the terrain.
    pub fn cell_of(&self, local: Vec2) -> Option<(u32, u32)> {
        let t = (local + self.size * 0.5) / self.cell();
        let n = Vec2::new(self.columns as f32 - 1.0, self.rows as f32 - 1.0);
        if !(t.x >= 0.0 && t.y >= 0.0 && t.x <= n.x && t.y <= n.y) {
            return None;
        }
        let c = (t.x as u32).min(self.columns - 2);
        let r = (t.y as u32).min(self.rows - 2);
        Some((c, r))
    }

    /// As [`TerrainData::height`].
    pub fn height(&self, local: Vec2) -> f32 {
        let t = self.grid(local);
        let i = t
            .floor()
            .min(Vec2::new(self.columns as f32 - 2.0, self.rows as f32 - 2.0));
        let f = (t - i).clamp(Vec2::ZERO, Vec2::ONE);
        let (c, r) = (i.x as u32, i.y as u32);
        let h00 = self.at(c, r);
        let h10 = self.at(c + 1, r);
        let h01 = self.at(c, r + 1);
        let h11 = self.at(c + 1, r + 1);
        if !og_core::math::heightfield_zigzag(h00, h10, h01, h11) {
            if f.x + f.y <= 1.0 {
                h00 + (h10 - h00) * f.x + (h01 - h00) * f.y
            } else {
                h11 + (h01 - h11) * (1.0 - f.x) + (h10 - h11) * (1.0 - f.y)
            }
        } else if f.y >= f.x {
            h00 + (h11 - h01) * f.x + (h01 - h00) * f.y
        } else {
            h00 + (h10 - h00) * f.x + (h11 - h10) * f.y
        }
    }

    fn sample_normal(&self, c: u32, r: u32) -> Vec3 {
        let cell = self.cell();
        let (c0, c1) = (c.saturating_sub(1), (c + 1).min(self.columns - 1));
        let (r0, r1) = (r.saturating_sub(1), (r + 1).min(self.rows - 1));
        let hx = (self.at(c1, r) - self.at(c0, r)) / ((c1 - c0).max(1) as f32 * cell.x);
        let hz = (self.at(c, r1) - self.at(c, r0)) / ((r1 - r0).max(1) as f32 * cell.y);
        Vec3::new(-hx, 1.0, -hz).normalize()
    }

    /// As [`TerrainData::normal`].
    pub fn normal(&self, local: Vec2) -> Vec3 {
        let t = self.grid(local);
        let i = t
            .floor()
            .min(Vec2::new(self.columns as f32 - 2.0, self.rows as f32 - 2.0));
        let f = (t - i).clamp(Vec2::ZERO, Vec2::ONE);
        let (c, r) = (i.x as u32, i.y as u32);
        let lo = self
            .sample_normal(c, r)
            .lerp(self.sample_normal(c + 1, r), f.x);
        let hi = self
            .sample_normal(c, r + 1)
            .lerp(self.sample_normal(c + 1, r + 1), f.x);
        lo.lerp(hi, f.y).normalize_or(Vec3::Y)
    }
}
