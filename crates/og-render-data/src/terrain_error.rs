//! How far each clipmap level's detail must reach: the height error a
//! coarser level makes, seen on screen.
//!
//! A clipmap level of spacing `2^j` field samples draws the field's
//! samples every `2^j` and the triangles between them, so between its
//! vertices it misses whatever the finer samples rise above or sink below
//! them: its error there (a peak between two of its vertices is cut
//! down). Seen from `d` metres an error of `e` metres spans `e / d`
//! radians, `e / (d × pixel)` pixels. Selection by projected geometric
//! error is how chunked and continuous terrain LOD choose a node's level
//! (Lindstrom et al. 1996; Ulrich, "Rendering Massive Terrains using
//! Chunked Level of Detail Control", 2002; Strugar, "Continuous
//! Distance-Dependent Level of Detail for Rendering Heightmaps", 2009,
//! whose quadtree nodes are chosen by distance and error alike): here
//! each level's morph band, the place where it hands over to the next
//! coarser level, moves out until that coarser level's error is under the
//! tolerance everywhere past it ([`crate::Terrain::clipmap`]).
//!
//! The errors are kept per block of [`ERROR_BLOCK`] × `ERROR_BLOCK` cells
//! of each decimation: a cell's is the largest distance of a sample in it
//! from its two triangles (split along the diagonal og_core's
//! heightfield_zigzag chooses, as the clipmap draws it), up to cells of
//! `2^EXACT` samples; coarser cells take the larger of their children's
//! and of the finer level's vertices' distance from their triangles. A streamed terrain's come with its tiles
//! ([`crate::TerrainTile::error`], built where the tile is, so the cook and
//! the runtime need nothing new); a field in memory's are built once from
//! it and again only where an edit changed it.

use glam::{Vec2, Vec3};

/// Cells per side of an error block.
pub const ERROR_BLOCK: u32 = 8;

/// Decimations up to `2^EXACT` samples apart have exact errors (every
/// sample read against the cell's triangles); coarser ones an estimate
/// from them, so an edit rebuilds at most a cell of `2^EXACT` samples a
/// side per level.
const EXACT: usize = 7;

/// The most decimations an error table answers (`2^31` samples apart).
pub const ERROR_STEPS: usize = 32;

/// One block's error: its rectangle in the terrain's space (metres from
/// its centre), the decimation it is of (`2^step` field samples between
/// the drawn vertices) and the largest height error there (metres, the
/// terrain's space).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ErrorBlock {
    pub lo: Vec2,
    pub hi: Vec2,
    pub step: u32,
    pub error: f32,
}

/// The decimation errors of a grid of samples: for each decimation `2^j`
/// (`j` from 1), its cells' error bounds and their blocks' largest.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Decimation {
    /// Samples along x and z.
    samples: (u32, u32),
    /// Per decimation `j` (index `j - 1`): cells along x and z and each
    /// cell's bound, row by row.
    cells: Vec<(u32, u32, Vec<f32>)>,
    /// Per decimation, blocks along x and z and each block's largest.
    blocks: Vec<(u32, u32, Vec<f32>)>,
}

/// The height over a cell of corner heights `h` (-x -z, +x -z, -x +z,
/// +x +z) at `f`, on the triangle of its split.
fn split(h: [f32; 4], f: Vec2) -> f32 {
    crate::terrain::split_height(h, f)
}

impl Decimation {
    /// The errors of `sx` × `sz` samples read as `at(x, z)`, for every
    /// decimation up to `levels` (`2^levels` samples apart).
    pub fn new(sx: u32, sz: u32, levels: u32, at: &dyn Fn(u32, u32) -> f32) -> Decimation {
        let mut d = Decimation {
            samples: (sx.max(1), sz.max(1)),
            ..Default::default()
        };
        for j in 1..=levels {
            let l = 1u32 << j;
            let nx = (d.samples.0 - 1).div_ceil(l).max(1);
            let nz = (d.samples.1 - 1).div_ceil(l).max(1);
            d.cells.push((nx, nz, vec![0.0; (nx * nz) as usize]));
            let (bx, bz) = (nx.div_ceil(ERROR_BLOCK), nz.div_ceil(ERROR_BLOCK));
            d.blocks.push((bx, bz, vec![0.0; (bx * bz) as usize]));
        }
        d.update(at, (0, 0), (d.samples.0 - 1, d.samples.1 - 1));
        d
    }

    /// Builds again what samples `lo`..=`hi` (x, z) reach, after they
    /// changed (`at`: the samples now).
    pub fn update(&mut self, at: &dyn Fn(u32, u32) -> f32, lo: (u32, u32), hi: (u32, u32)) {
        let (sx, sz) = self.samples;
        let sample = |x: u32, z: u32| at(x.min(sx - 1), z.min(sz - 1));
        for j in 0..self.cells.len() {
            let l = 1u32 << (j + 1);
            let half = l / 2;
            let (nx, nz, _) = self.cells[j];
            // The cells whose samples (their edges included) the change reaches.
            let (cx0, cz0) = ((lo.0.saturating_sub(1)) / l, (lo.1.saturating_sub(1)) / l);
            let (cx1, cz1) = ((hi.0 / l).min(nx - 1), (hi.1 / l).min(nz - 1));
            for cz in cz0..=cz1 {
                for cx in cx0..=cx1 {
                    let (x0, z0) = (cx * l, cz * l);
                    let (x1, z1) = ((x0 + l).min(sx - 1), (z0 + l).min(sz - 1));
                    let h = [
                        sample(x0, z0),
                        sample(x1, z0),
                        sample(x0, z1),
                        sample(x1, z1),
                    ];
                    let span = Vec2::new((x1 - x0).max(1) as f32, (z1 - z0).max(1) as f32);
                    let mut e = 0.0f32;
                    if j < EXACT {
                        // Every sample in the cell against its triangles.
                        for vz in z0..=z1 {
                            for vx in x0..=x1 {
                                let f = Vec2::new((vx - x0) as f32, (vz - z0) as f32) / span;
                                e = e.max((sample(vx, vz) - split(h, f)).abs());
                            }
                        }
                    } else {
                        // Past them, the larger of the finer level's error
                        // under it and how far that level's vertices lie
                        // from its triangles: an estimate (their sum bounds
                        // it, but only loosely: a peak's slopes spread it
                        // the same way at every level).
                        for vz in [z0, z0 + half, z0 + l] {
                            for vx in [x0, x0 + half, x0 + l] {
                                if vx > x1 || vz > z1 {
                                    continue;
                                }
                                let f = Vec2::new((vx - x0) as f32, (vz - z0) as f32) / span;
                                e = e.max((sample(vx, vz) - split(h, f)).abs());
                            }
                        }
                        let (fx, fz, fine) = &self.cells[j - 1];
                        for (ox, oz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                            let (x, z) = (2 * cx + ox, 2 * cz + oz);
                            if x < *fx && z < *fz {
                                e = e.max(fine[(z * fx + x) as usize]);
                            }
                        }
                    }
                    let (nx, ..) = self.cells[j];
                    self.cells[j].2[(cz * nx + cx) as usize] = e;
                }
            }
            // The blocks over those cells.
            let (bx, _, _) = self.blocks[j];
            for z in cz0 / ERROR_BLOCK..=cz1 / ERROR_BLOCK {
                for x in cx0 / ERROR_BLOCK..=cx1 / ERROR_BLOCK {
                    let (nx, nz, cells) = &self.cells[j];
                    let mut m = 0.0f32;
                    for cz in z * ERROR_BLOCK..((z + 1) * ERROR_BLOCK).min(*nz) {
                        for cx in x * ERROR_BLOCK..((x + 1) * ERROR_BLOCK).min(*nx) {
                            m = m.max(cells[(cz * nx + cx) as usize]);
                        }
                    }
                    self.blocks[j].2[(z * bx + x) as usize] = m;
                }
            }
        }
    }

    /// The blocks' errors, decimation after decimation, each block's
    /// rectangle in samples ((x, z) to (x, z), clamped to the grid) and
    /// its decimation `j`.
    pub fn blocks(&self) -> impl Iterator<Item = ((u32, u32), (u32, u32), u32, f32)> + '_ {
        let (sx, sz) = self.samples;
        self.blocks
            .iter()
            .enumerate()
            .flat_map(move |(j, (bx, bz, e))| {
                let span = ERROR_BLOCK << (j + 1);
                (0..*bz).flat_map(move |z| {
                    (0..*bx).map(move |x| {
                        let lo = (x * span, z * span);
                        let hi = (((x + 1) * span).min(sx - 1), ((z + 1) * span).min(sz - 1));
                        (lo, hi, j as u32 + 1, e[(z * bx + x) as usize])
                    })
                })
            })
    }

    /// The blocks' errors laid out as a tile keeps them
    /// ([`crate::TerrainTile::error`]): decimation after decimation, row by row.
    pub fn flat(&self) -> Vec<f32> {
        self.blocks
            .iter()
            .flat_map(|(_, _, e)| e.iter().copied())
            .collect()
    }

    /// Decimations a tile of `tile` quads keeps errors for (`2^j` up to
    /// its size), and how many blocks each has along a side.
    pub fn tile_shape(tile: u32) -> impl Iterator<Item = (u32, u32)> {
        let levels = 31 - tile.max(2).leading_zeros();
        (1..=levels).map(move |j| (j, tile.div_ceil(1 << j).max(1).div_ceil(ERROR_BLOCK)))
    }
}

/// How far out (Chebyshev metres from the eye, the terrain's space) each
/// decimation's error shows: per `2^step` samples apart, the farthest
/// edge of a block where `error / distance` spans more than `tolerance`
/// pixels of `pixel` radians, 0 where none does. `eye` is in the
/// terrain's space; `range` its height range; `along` and `up` its
/// horizontal and vertical scale to the world; blocks past `far` (the
/// terrain's metres) are not seen.
pub fn error_reach(
    blocks: impl Iterator<Item = ErrorBlock>,
    eye: Vec3,
    range: (f32, f32),
    (along, up): (f32, f32),
    pixel: f32,
    tolerance: f32,
    far: f32,
) -> [f32; ERROR_STEPS] {
    let mut reach = [0.0f32; ERROR_STEPS];
    let eye_xz = Vec2::new(eye.x, eye.z);
    let dy = (eye.y - eye.y.clamp(range.0, range.1)) * up;
    let limit = pixel.max(1e-9) * tolerance.max(1e-3);
    for b in blocks {
        let step = b.step as usize;
        if b.error <= 0.0 || step >= ERROR_STEPS {
            continue;
        }
        let d_xz = (eye_xz.clamp(b.lo, b.hi) - eye_xz).length();
        if d_xz > far {
            continue;
        }
        let d = ((d_xz * along).powi(2) + dy * dy).sqrt().max(1e-3);
        if b.error * up / d > limit {
            let far_edge = (b.lo - eye_xz)
                .abs()
                .max((b.hi - eye_xz).abs())
                .max_element();
            reach[step] = reach[step].max(far_edge);
        }
    }
    reach
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A spike between a coarse level's vertices is that level's error at
    /// least; a plane is drawn exactly at every level.
    #[test]
    fn a_peak_between_vertices_is_the_coarser_levels_error() {
        let n = 33;
        let at = |x: u32, z: u32| if (x, z) == (5, 9) { 10.0 } else { 0.0 };
        let d = Decimation::new(n, n, 5, &at);
        for j in 1..=5 {
            let e = d
                .blocks()
                .filter(|b| b.2 == j)
                .map(|b| b.3)
                .fold(0.0, f32::max);
            assert!((e - 10.0).abs() < 1e-4, "decimation {j}: {e}");
        }
        // A spike on a corner of every level: the coarser triangles spread
        // it over their cells, by at most the spike.
        let at = |x: u32, z: u32| if (x, z) == (0, 0) { 10.0 } else { 0.0 };
        let d = Decimation::new(n, n, 5, &at);
        assert!(
            d.blocks().all(|b| b.3 <= 10.0),
            "{:?}",
            d.blocks().collect::<Vec<_>>()
        );
        let plane = |x: u32, z: u32| 0.5 * x as f32 - 0.25 * z as f32 + 3.0;
        let d = Decimation::new(n, n, 5, &plane);
        assert!(
            d.blocks().all(|b| b.3 < 1e-4),
            "{:?}",
            d.blocks().collect::<Vec<_>>()
        );
    }

    /// An edit builds again only what it reaches, and gives what building
    /// it all again gives.
    #[test]
    fn an_update_matches_a_rebuild() {
        let n = 70;
        let before = |x: u32, z: u32| ((x * 7 + z * 13) % 11) as f32;
        let after = |x: u32, z: u32| {
            if (20..30).contains(&x) && (40..45).contains(&z) {
                (x * z) as f32 * 0.1
            } else {
                before(x, z)
            }
        };
        let mut d = Decimation::new(n, n, 6, &before);
        d.update(&after, (20, 40), (29, 44));
        assert_eq!(d, Decimation::new(n, n, 6, &after));
    }

    /// A block shows where its error spans more than the tolerance, and
    /// the reach is its far edge.
    #[test]
    fn the_reach_is_the_far_edge_of_a_block_whose_error_shows() {
        let block = |x: f32, error: f32| ErrorBlock {
            lo: Vec2::new(x, -5.0),
            hi: Vec2::new(x + 10.0, 5.0),
            step: 3,
            error,
        };
        // 1 m at 100 m: 0.01 rad, 10 pixels of a milliradian.
        let reach = error_reach(
            [block(100.0, 1.0), block(500.0, 0.1)].into_iter(),
            Vec3::ZERO,
            (0.0, 0.0),
            (1.0, 1.0),
            1e-3,
            1.0,
            1e4,
        );
        assert_eq!(reach[3], 110.0);
        assert!(reach.iter().enumerate().all(|(i, r)| i == 3 || *r == 0.0));
    }
}
