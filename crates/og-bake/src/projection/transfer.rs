//! Part-local metric texture transfer, shared by cook and `og model bake --from`.
use super::{Level, Paint, Texel, Texels};
use glam::{IVec3, Vec3};
use std::collections::BTreeMap;

struct BaseTexel {
    position: Vec3,
    normal: Vec3,
    rgb: [u8; 3],
}

fn bounds(level: &Level, parts: &[String]) -> BTreeMap<String, (Vec3, Vec3)> {
    let mut out: BTreeMap<String, (Vec3, Vec3)> = BTreeMap::new();
    for (t, part) in parts.iter().enumerate() {
        let e = out
            .entry(part.clone())
            .or_insert((Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)));
        for p in level.corners(t).map(|i| level.positions[i as usize]) {
            e.0 = e.0.min(p);
            e.1 = e.1.max(p);
        }
    }
    out
}

struct Nearest {
    texels: Vec<BaseTexel>,
    cell: f32,
    cells: [BTreeMap<(i32, i32, i32), Vec<u32>>; 6],
}

fn facing(n: Vec3) -> usize {
    let a = n.abs();
    let k = if a.x >= a.y && a.x >= a.z {
        0
    } else if a.y >= a.z {
        1
    } else {
        2
    };
    k * 2 + usize::from(n[k] < 0.0)
}

impl Nearest {
    fn new(texels: Vec<BaseTexel>, extent: Vec3) -> Nearest {
        let cell = (extent.max_element() / 32.0).max(1e-4);
        let mut cells: [BTreeMap<(i32, i32, i32), Vec<u32>>; 6] = Default::default();
        for (i, t) in texels.iter().enumerate() {
            let c = (t.position / cell).floor().as_ivec3();
            cells[facing(t.normal)]
                .entry((c.x, c.y, c.z))
                .or_default()
                .push(i as u32);
        }
        Nearest {
            texels,
            cell,
            cells,
        }
    }

    fn colour(&self, p: Vec3, n: Vec3) -> Option<[u8; 3]> {
        let own = facing(n);
        let found = if self.cells[own].is_empty() {
            (0..6).filter_map(|k| self.search(k, p)).min()
        } else {
            self.search(own, p)
        };
        found.map(|(_, i)| self.texels[i as usize].rgb)
    }

    fn search(&self, k: usize, p: Vec3) -> Option<(u32, u32)> {
        let cells = &self.cells[k];
        if cells.is_empty() {
            return None;
        }
        let c = (p / self.cell).floor().as_ivec3();
        let mut best = (f32::INFINITY, u32::MAX);
        for r in 0..=40 {
            for z in c.z - r..=c.z + r {
                for y in c.y - r..=c.y + r {
                    for x in c.x - r..=c.x + r {
                        if (IVec3::new(x, y, z) - c).abs().max_element() != r {
                            continue;
                        }
                        for &j in cells.get(&(x, y, z)).into_iter().flatten() {
                            let d = (self.texels[j as usize].position - p).length_squared();
                            if (d, j) < best {
                                best = (d, j);
                            }
                        }
                    }
                }
            }
            // Every texel past ring r is at least r cells away.
            if best.1 != u32::MAX && best.0.sqrt() <= r as f32 * self.cell {
                break;
            }
        }
        (best.1 != u32::MAX).then_some((best.0.to_bits(), best.1))
    }
}

/// Painted texels indexed by named part in its own metric frame. No fitting or rescaling.
pub struct Transfer(BTreeMap<String, (Nearest, Vec3)>);

/// Counts before hole filling, aggregated by named part.
#[derive(Clone, Debug, Default)]
pub struct PartTransfer {
    /// Part name, stable across variants.
    pub name: String,
    /// Covered destination texels.
    pub texels: usize,
    /// Texels inside the base part's bounds, with the existing 1 mm tolerance.
    pub transferred: usize,
    /// Texels beyond the base part's bounds, using its nearest region.
    pub extended: usize,
    /// Whether the base has painted texels of this part.
    pub in_base: bool,
}

impl Transfer {
    /// Rasterise the base maps. `parts` names each triangle; `maps` is one square
    /// RGB map per material label, with `size * size` pixels, or none.
    pub fn new(
        level: &Level,
        parts: &[String],
        maps: &[Option<Vec<[u8; 3]>>],
        size: u32,
    ) -> Result<Self, String> {
        if size == 0
            || parts.len() != level.triangles()
            || maps.len() != level.labels.len()
            || maps
                .iter()
                .flatten()
                .any(|m| m.len() != size as usize * size as usize)
        {
            return Err("texture transfer needs one part per triangle and one square base map per material label".into());
        }
        let anchors = bounds(level, parts);
        let mut by_part: BTreeMap<String, Vec<BaseTexel>> = BTreeMap::new();
        for (m, map) in maps.iter().enumerate() {
            let Some(map) = map else { continue };
            let own = only_material(level, m as u16);
            let t = Texels::rasterise(&own, size, 0);
            for i in 0..t.len() {
                let part = &parts[t.triangle[i] as usize];
                by_part.entry(part.clone()).or_default().push(BaseTexel {
                    position: t.position[i] - anchors[part].0,
                    normal: t.normal[i],
                    rgb: map[t.index[i] as usize],
                });
            }
        }
        Ok(Self(
            by_part
                .into_iter()
                .map(|(name, texels)| {
                    let (lo, hi) = anchors[&name];
                    (name, (Nearest::new(texels, hi - lo), hi - lo))
                })
                .collect(),
        ))
    }

    /// Paint destination texels in each named part's own unscaled metric frame.
    /// The caller retains its existing hole-fill, padding and image encoding policy.
    pub fn paint(
        &self,
        level: &Level,
        parts: &[String],
        t: &Texels,
    ) -> Result<(Paint, Vec<PartTransfer>), String> {
        if parts.len() != level.triangles() || t.triangle.iter().any(|i| *i as usize >= parts.len())
        {
            return Err("texture transfer destination parts do not match its triangles".into());
        }
        let anchors = bounds(level, parts);
        let mut counts: BTreeMap<String, PartTransfer> = BTreeMap::new();
        let mut paint = Paint::new(t);
        for i in 0..t.len() {
            let part = &parts[t.triangle[i] as usize];
            let c = counts.entry(part.clone()).or_insert_with(|| PartTransfer {
                name: part.clone(),
                in_base: self.0.contains_key(part),
                ..Default::default()
            });
            c.texels += 1;
            let Some((near, extent)) = self.0.get(part) else {
                continue;
            };
            let q = t.position[i] - anchors[part].0;
            let Some(rgb) = near.colour(q.clamp(Vec3::ZERO, *extent), t.normal[i]) else {
                continue;
            };
            paint.texels[i] = Texel { rgb, quality: 1.0 };
            let grown = q.cmplt(Vec3::splat(-1e-3)).any() || q.cmpgt(*extent + 1e-3).any();
            if grown {
                c.extended += 1;
            } else {
                c.transferred += 1;
            }
        }
        Ok((paint, counts.into_values().collect()))
    }
}

fn only_material(level: &Level, material: u16) -> Level {
    // Collapse excluded triangles without changing UVs of shared vertices.
    Level::new(
        level.positions.clone(),
        level.normals.clone(),
        level.uvs.clone(),
        level
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .enumerate()
            .flat_map(|(i, t)| {
                if level.triangle_label[i] == material {
                    [t[0], t[1], t[2]]
                } else {
                    [t[0]; 3]
                }
            })
            .collect(),
        level.triangle_label.clone(),
        level.labels.clone(),
    )
}
