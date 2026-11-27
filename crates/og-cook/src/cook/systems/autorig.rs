//! Layered-art rigging from explicit, captured PSD/PSB or PNG parts.
//! The art contract (docs/art.md §2.8) supplies part names and layouts;
//! this mechanism keeps its atlas, joints and clip construction native.
use super::texture::Image;
use crate::cook::source_schemas::media::{clips::Filter, rig2d::Parts};
use og_anim::{Rig2dAsset, SheetFilter};
use std::collections::BTreeMap;

mod atlas;
mod clips;
mod creature;
mod front;
mod mesh;
mod parts;
mod quadruped;
mod side;
mod skeleton;
mod template;
#[cfg(test)]
mod tests;

pub(crate) use parts::{Part, folder_files as part_files};

/// The suffix of the atlas texture produced by the same declaration.
pub(crate) fn atlas_id(id: &str) -> String {
    format!("{id}#atlas")
}

pub(crate) fn build(
    id: &str,
    input: &Parts,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<(Rig2dAsset, Image, Vec<String>), String> {
    let template = template::get(&input.template).ok_or_else(|| {
        format!(
            "'{}' is not a rig layout; one of {}",
            input.template,
            template::NAMES.join(", ")
        )
    })?;
    let mut notes = Vec::new();
    let art = parts::read(&input.parts, files, template, &mut notes)?;
    let mut pivots = Vec::new();
    for (name, at) in &input.pivots {
        if !art.parts.iter().any(|p| &p.name == name) {
            return Err(format!(
                "pivot for '{name}', which is not a part of {}",
                art.label
            ));
        }
        pivots.push((name.clone(), *at));
    }
    let packed = atlas::pack(&art.parts).map_err(|e| format!("{}: {e}", art.label))?;
    let mut rig = skeleton::rig(&art, template, &pivots, &packed.rects)?;
    rig.texture = atlas_id(id);
    rig.size = [packed.image.width, packed.image.height];
    // The template's height in world units (a humanoid two, a fox 1.2),
    // whatever the art's resolution.
    rig.unit = input
        .unit
        .unwrap_or_else(|| (art.drawn_height() / template.height).max(1.0));
    rig.filter = match input.filter {
        Some(Filter::Nearest) => SheetFilter::Nearest,
        _ => SheetFilter::Linear,
    };
    let measure = clips::measure(&rig, template.legs, art.drawn_height());
    rig.clips = match template.name {
        "humanoid-side" => side::clips(&rig, &measure),
        "humanoid-front" => front::clips(&rig, &measure),
        "quadruped" => quadruped::clips(&rig, &measure),
        _ => creature::clips(&rig, &measure),
    };
    notes.push(format!(
        "auto-rigged {} as {}: {} parts, {} clips, a {}x{} atlas",
        art.label,
        template.name,
        art.parts.len(),
        rig.clips.len(),
        packed.image.width,
        packed.image.height
    ));
    Ok((rig, packed.image, notes))
}
