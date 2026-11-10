use super::*;

/// Where a terrain's tiles come from: a whole field in memory (its tiles
/// made when drawn), or tiles handed over one by one (streaming).
#[derive(Clone, Debug)]
pub(crate) enum TerrainSource {
    /// The field, its height range (measured once, when it was set) and
    /// its height errors (`terrain_error.rs`; built when first asked for).
    Data(TerrainData, (f32, f32), FieldErrors),
    Stream {
        layout: TerrainLayout,
        range: (f32, f32),
        tiles: og_core::IdMap<u32, Arc<TerrainTile>>,
    },
}

/// A field's height errors, built when first asked for.
pub(crate) type FieldErrors = Arc<std::sync::OnceLock<Arc<crate::terrain_error::Decimation>>>;

impl TerrainSource {
    /// A field in memory, its range measured.
    pub(crate) fn data(d: TerrainData) -> TerrainSource {
        let range = d.range();
        TerrainSource::Data(d, range, FieldErrors::default())
    }

    /// `d`, a field of the same shape as this one's edited inside `lo`..`hi`
    /// (the terrain's space): its errors built again only there, when this
    /// one's were built.
    pub(crate) fn edited(&self, d: TerrainData, lo: Vec2, hi: Vec2) -> TerrainSource {
        let errors = FieldErrors::default();
        if let TerrainSource::Data(_, _, old) = self
            && let Some(old) = old.get()
        {
            let top = Vec2::new((d.columns - 1) as f32, (d.rows - 1) as f32);
            let sample = |v: Vec2| ((v + d.size * 0.5) / d.cell()).clamp(Vec2::ZERO, top);
            let (a, b) = (sample(lo).floor(), sample(hi).ceil());
            let lo = (a.x as u32, a.y as u32);
            let hi = (b.x as u32, b.y as u32);
            let mut e = (**old).clone();
            e.update(&|x, z| d.heights[(z * d.columns + x) as usize], lo, hi);
            let _ = errors.set(Arc::new(e));
        }
        let range = d.range();
        TerrainSource::Data(d, range, errors)
    }

    /// Its height errors' blocks for `layout` (`terrain_error.rs`): a
    /// field's, built the first time; a streamed terrain's, its tiles'
    /// (each tile's, at its own level; a finer one where it has come).
    pub(crate) fn error_blocks(
        &self,
        layout: &TerrainLayout,
    ) -> Vec<crate::terrain_error::ErrorBlock> {
        use crate::terrain_error::{Decimation, ERROR_BLOCK, ErrorBlock};
        let cell = layout.cell();
        let half = layout.size * 0.5;
        let metres = |x: u32, z: u32| Vec2::new(x as f32, z as f32) * cell - half;
        match self {
            TerrainSource::Data(d, _, errors) => {
                let e = errors.get_or_init(|| {
                    let n = (d.columns.max(d.rows) - 1).max(2);
                    let levels = 32 - (n - 1).leading_zeros();
                    Arc::new(Decimation::new(d.columns, d.rows, levels, &|x, z| {
                        d.heights[(z * d.columns + x) as usize]
                    }))
                });
                e.blocks()
                    .map(|(lo, hi, step, error)| ErrorBlock {
                        lo: metres(lo.0, lo.1),
                        hi: metres(hi.0, hi.1),
                        step,
                        error,
                    })
                    .collect()
            }
            TerrainSource::Stream { tiles, .. } => {
                let t = layout.tile;
                let mut out = Vec::new();
                for (&index, tile) in tiles.iter() {
                    let Some((k, tx, tz)) = layout.locate(index) else {
                        continue;
                    };
                    let step = layout.step(k);
                    let fine = |g: u32, n: u32| (g as u64 * step as u64).min(n as u64 - 1) as u32;
                    let mut at = 0usize;
                    for (j, b) in Decimation::tile_shape(t) {
                        let span = ERROR_BLOCK << j;
                        for bz in 0..b {
                            for bx in 0..b {
                                let error = tile.error.get(at).copied().unwrap_or(0.0);
                                at += 1;
                                let (x0, z0) = (tx * t + bx * span, tz * t + bz * span);
                                let (x1, z1) =
                                    ((x0 + span).min((tx + 1) * t), (z0 + span).min((tz + 1) * t));
                                out.push(ErrorBlock {
                                    lo: metres(fine(x0, layout.columns), fine(z0, layout.rows)),
                                    hi: metres(fine(x1, layout.columns), fine(z1, layout.rows)),
                                    step: k + j,
                                    error,
                                });
                            }
                        }
                    }
                }
                out
            }
        }
    }

    /// The tile size a field in memory is drawn with: [`TERRAIN_TILE`], or
    /// larger for a field too big for the tile table.
    pub(crate) fn layout(&self) -> Result<TerrainLayout, String> {
        match self {
            TerrainSource::Stream { layout, .. } => Ok(*layout),
            TerrainSource::Data(d, ..) => {
                d.check()?;
                let mut tile = TERRAIN_TILE;
                loop {
                    match d.layout(tile) {
                        Ok(l) => return Ok(l),
                        Err(e) if tile >= 1024 => return Err(e),
                        Err(_) => tile *= 2,
                    }
                }
            }
        }
    }

    pub(crate) fn range(&self) -> (f32, f32) {
        match self {
            TerrainSource::Stream { range, .. } => *range,
            TerrainSource::Data(_, range, _) => *range,
        }
    }
}
