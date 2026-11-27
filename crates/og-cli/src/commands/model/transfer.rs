//! `og model bake <variant> --from <base>`: a variant's base colour
//! carried over from its baked base, never projected again. A variant is
//! the base program at other parameter values, so its parts are the
//! base's parts, moved and resized; each keeps its own chart layout (its
//! own atlas) and its texel density.
//!
//! Steps: [`base_texels`] reads every painted texel of the base (its
//! point on the model, its normal, its part and its colour) from the
//! base's maps; og-bake gives each part its own metric frame (the
//! corner of its box with the least x, y and z, metres, unscaled), so a
//! point keeps its distance from that corner whatever the part's size;
//! [`Transfer`] finds, for each variant texel, the base texel of the same
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

use og_bake::projection::{PartTransfer, Texels, Transfer, fill_unpainted, padded};
use og_shape::tools::Scene;
use serde_json::{Value, json};

use super::bake::{encode, surface, write};
use super::source::{Input, program_of};
use super::{Failure, Files};

/// What to carry over. The zero value has no input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransferDesc {
    /// The variant: a checked program model, or standalone kernel JSON.
    pub input: Input,
    /// The baked base: a checked program model whose materials have maps.
    pub base: PathBuf,
    /// Texels across each texture; 0 is the base's atlas size.
    pub size: u32,
    /// Where to write; `None` beside the variant.
    pub out: Option<PathBuf>,
}

/// Every painted texel of the base, by part, in each part's frame; and
/// the base parts' boxes in their frames.
fn base_texels(scene: &Scene, size: u32) -> Result<Transfer, Failure> {
    let surf = surface(scene)?;
    let names: Vec<_> = surf
        .parts
        .iter()
        .map(|p| scene.parts[*p as usize].name.clone())
        .collect();
    let mut maps = Vec::new();
    for (name, material) in &surf.materials {
        let Some(map) = &material.map else {
            maps.push(None);
            continue;
        };
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
        maps.push(Some(img.pixels().map(|p| p.0).collect()));
    }
    Transfer::new(&surf.level(None), &names, &maps, size).map_err(Failure::Failed)
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
fn carry(scene: &Scene, base: &Transfer, size: u32) -> Result<Carried, Failure> {
    let surf = surface(scene)?;
    let names: Vec<_> = surf
        .parts
        .iter()
        .map(|p| scene.parts[*p as usize].name.clone())
        .collect();
    let level = surf.level(None);
    // Per part: texels, transferred, extended.
    let mut counts: BTreeMap<String, PartTransfer> = BTreeMap::new();
    let mut coverage = Vec::new();
    let mut maps = Vec::new();
    for (m, (name, material)) in surf.materials.iter().enumerate() {
        let t = Texels::rasterise(&surf.level(Some(m as u16)), size, 0);
        let (mut paint, parts) = base.paint(&level, &names, &t).map_err(Failure::Failed)?;
        for part in parts {
            let c = counts
                .entry(part.name.clone())
                .or_insert_with(|| PartTransfer {
                    name: part.name.clone(),
                    in_base: part.in_base,
                    ..Default::default()
                });
            c.texels += part.texels;
            c.transferred += part.transferred;
            c.extended += part.extended;
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
        .map(|(part, c)| {
            let (n, moved, grown) = (c.texels, c.transferred, c.extended);
            let share = |k: usize| k as f64 / n.max(1) as f64;
            json!({
                "part": part,
                "texels": n,
                "transferred": share(moved),
                "extended": share(grown),
                "inBase": c.in_base,
            })
        })
        .collect();
    Ok((parts, coverage, maps))
}
