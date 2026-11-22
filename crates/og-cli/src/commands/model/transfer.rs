//! `og model bake <variant> --from <base>`: a variant's base colour
//! carried over from its baked base, never projected again. A variant is
//! the base program at other parameter values, so its parts are the
//! base's parts, moved and resized; each keeps its own chart layout (its
//! own atlas) and its texel density.
//!
//! Steps: [`base_texels`] reads every painted texel of the base (its
//! point on the model, its normal, its part and its colour) from the
//! base's maps; [`Anchors`] gives each part its own metric frame (the
//! corner of its box with the least x, y and z, metres, unscaled), so a
//! point keeps its distance from that corner whatever the part's size;
//! [`Nearest`] finds, for each variant texel, the base texel of the same
//! part nearest that point (facing the same way where one does). A
//! variant point inside the base part's box is `transferred`; one where
//! the part grew past the base reads the nearest region of the same part
//! and is `extended`. A part the base has not is filled from its
//! neighbours on the model (`fill_unpainted`).
//!
//! The report: `parts` (per part: `texels`, `transferred`, `extended`, as
//! fractions), `coverage` per material and `output`, as `bake` writes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use glam::{IVec3, Vec3};
use og_bake::projection::{Paint, Texel, Texels, fill_unpainted, padded};
use og_shape::tools::Scene;
use serde_json::{Value, json};

use super::bake::{Surface, encode, surface, write};
use super::source::{Input, program_of};
use super::{Failure, Files};

/// What to carry over. The zero value has no input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransferDesc {
    /// The variant: a `type model` naming a program, or a program.
    pub input: Input,
    /// The baked base: a `type model` whose materials have maps.
    pub base: PathBuf,
    /// Texels across each texture; 0 is the base's atlas size.
    pub size: u32,
    /// Where to write; `None` beside the variant.
    pub out: Option<PathBuf>,
}

/// A painted texel of the base.
struct BaseTexel {
    position: Vec3,
    normal: Vec3,
    rgb: [u8; 3],
}

/// Each part's frame: the least corner of its box, and its box.
#[derive(Default)]
struct Anchors(BTreeMap<String, (Vec3, Vec3)>);

impl Anchors {
    fn of(scene: &Scene, surf: &Surface) -> Anchors {
        let mut out: BTreeMap<String, (Vec3, Vec3)> = BTreeMap::new();
        for (t, &part) in surf.parts.iter().enumerate() {
            let name = &scene.parts[part as usize].name;
            let e = out
                .entry(name.clone())
                .or_insert((Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)));
            for p in &surf.positions[t * 3..t * 3 + 3] {
                e.0 = e.0.min(*p);
                e.1 = e.1.max(*p);
            }
        }
        Anchors(out)
    }
}

/// The base texels of one part, in the part's frame, in a grid of cells
/// for each of the six ways a texel faces most (its normal's largest
/// component and its sign).
struct Nearest {
    texels: Vec<BaseTexel>,
    cell: f32,
    cells: [BTreeMap<(i32, i32, i32), Vec<u32>>; 6],
}

/// Which of the six ways `n` faces most.
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

    /// The colour of the texel nearest `p` among those facing as `n`
    /// most does, else among all; ties to the lower index. `p` lies in
    /// the part's box, so the nearest is a few cells away.
    fn colour(&self, p: Vec3, n: Vec3) -> Option<[u8; 3]> {
        let own = facing(n);
        let found = if self.cells[own].is_empty() {
            (0..6).filter_map(|k| self.search(k, p)).min()
        } else {
            self.search(own, p)
        };
        found.map(|(_, i)| self.texels[i as usize].rgb)
    }

    /// The nearest texel of grid `k` to `p`: (its squared distance's
    /// bits, its index).
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

/// Every painted texel of the base, by part, in each part's frame; and
/// the base parts' boxes in their frames.
fn base_texels(scene: &Scene, size: u32) -> Result<BTreeMap<String, (Nearest, Vec3)>, Failure> {
    let surf = surface(scene)?;
    let anchors = Anchors::of(scene, &surf);
    let mut by_part: BTreeMap<String, Vec<BaseTexel>> = BTreeMap::new();
    for (m, (name, material)) in surf.materials.iter().enumerate() {
        let Some(map) = &material.map else { continue };
        let img = image::load_from_memory(&map.bytes)
            .map_err(|e| Failure::Failed(format!("base material {name:?}: {}: {e}", map.name)))?
            .to_rgb8();
        if img.width() != size || img.height() != size {
            return Err(Failure::Failed(format!(
                "base material {name:?}: its map is {}×{}, its atlas {size}",
                img.width(),
                img.height()
            )));
        }
        let t = Texels::rasterise(&surf.level(Some(m as u16)), size, 0);
        for i in 0..t.len() {
            let part = &scene.parts[surf.parts[t.triangle[i] as usize] as usize].name;
            let k = t.index[i];
            let px = img.get_pixel(k % size, k / size);
            by_part.entry(part.clone()).or_default().push(BaseTexel {
                position: t.position[i] - anchors.0[part].0,
                normal: t.normal[i],
                rgb: px.0,
            });
        }
    }
    Ok(by_part
        .into_iter()
        .map(|(part, texels)| {
            let (lo, hi) = anchors.0[&part];
            (part, (Nearest::new(texels, hi - lo), hi - lo))
        })
        .collect())
}

/// The report (see the module doc).
pub fn transfer(files: &dyn Files, d: &TransferDesc) -> Result<String, Failure> {
    let Input::Path(input) = &d.input else {
        return Err(Failure::Usage("no input: give the variant".into()));
    };
    let not_program = |p: &Path| {
        Failure::Usage(format!(
            "{}: --from carries a baked program's maps to a variant; give program models",
            p.display()
        ))
    };
    let (base, base_read) = program_of(files, &d.base)?.ok_or_else(|| not_program(&d.base))?;
    let (variant, read) = program_of(files, input)?.ok_or_else(|| not_program(input))?;
    if base.atlas == 0 || !base.materials.iter().any(|m| m.map.is_some()) {
        return Err(Failure::Failed(format!(
            "{}: not baked (no atlas and no maps): og model bake it with --project first",
            d.base.display()
        )));
    }
    if base.file != variant.file {
        return Err(Failure::Usage(format!(
            "{} builds {}, the base {}: a variant is the base's program",
            input.display(),
            variant.file,
            base.file
        )));
    }
    let size = if d.size == 0 { base.atlas } else { d.size };
    let (texels, _) = base
        .scene(base.atlas, &base_read, |scene, _| {
            base_texels(scene, base.atlas).map_err(|e| e.to_string())
        })
        .map_err(Failure::Failed)?;
    let ((parts, coverage, maps), _) = variant
        .scene(size, &read, |scene, _| {
            carry(scene, &texels, size).map_err(|e| e.to_string())
        })
        .map_err(Failure::Failed)?;
    let transferred: f64 = parts.iter().filter_map(|p| p["transferred"].as_f64()).sum();
    if transferred == 0.0 {
        return Err(Failure::Failed(format!(
            "no part of {} is in the base: nothing to carry over",
            input.display()
        )));
    }
    let output = write(
        files,
        input,
        Some(&variant),
        size,
        &Vec::new(),
        &maps,
        d.out.as_deref(),
    )?;
    Ok(json!({
        "size": size,
        "from": d.base.display().to_string(),
        "parts": parts,
        "coverage": coverage,
        "output": output,
    })
    .to_string())
}

/// The parts' report, the coverage per material, and each material's
/// texture as PNG.
type Carried = (Vec<Value>, Vec<Value>, Vec<(String, Vec<u8>)>);

/// Paints the variant's textures from the base's texels.
fn carry(
    scene: &Scene,
    base: &BTreeMap<String, (Nearest, Vec3)>,
    size: u32,
) -> Result<Carried, Failure> {
    let surf = surface(scene)?;
    let anchors = Anchors::of(scene, &surf);
    // Per part: texels, transferred, extended.
    let mut counts: BTreeMap<String, [usize; 3]> = BTreeMap::new();
    let mut coverage = Vec::new();
    let mut maps = Vec::new();
    for (m, (name, material)) in surf.materials.iter().enumerate() {
        let t = Texels::rasterise(&surf.level(Some(m as u16)), size, 0);
        let mut paint = Paint::new(&t);
        for i in 0..t.len() {
            let part = &scene.parts[surf.parts[t.triangle[i] as usize] as usize].name;
            let c = counts.entry(part.clone()).or_default();
            c[0] += 1;
            let Some((near, extent)) = base.get(part) else {
                continue;
            };
            let q = t.position[i] - anchors.0[part].0;
            // A point where the part grew reads the nearest region of the
            // base part: its box's nearest point.
            let Some(rgb) = near.colour(q.clamp(Vec3::ZERO, *extent), t.normal[i]) else {
                continue;
            };
            paint.texels[i] = Texel { rgb, quality: 1.0 };
            // Past the base part's box by more than a millimetre: grown.
            let grown = q.cmplt(Vec3::splat(-1e-3)).any() || q.cmpgt(*extent + 1e-3).any();
            c[if grown { 2 } else { 1 }] += 1;
        }
        let carried = paint.coverage();
        let filled = fill_unpainted(&t, &mut paint, 0);
        coverage.push(json!({
            "material": name,
            "texels": t.len(),
            "carried": carried,
            "filled": filled as f64 / t.len().max(1) as f64,
        }));
        if carried > 0.0 {
            let rgb = padded(&t, &paint, 4, super::measure::colour(material));
            maps.push((name.clone(), encode(&rgb, size)?));
        }
    }
    let parts = counts
        .into_iter()
        .map(|(part, [n, moved, grown])| {
            let share = |k: usize| k as f64 / n.max(1) as f64;
            json!({
                "part": part,
                "texels": n,
                "transferred": share(moved),
                "extended": share(grown),
                "inBase": base.contains_key(&part),
            })
        })
        .collect();
    Ok((parts, coverage, maps))
}
