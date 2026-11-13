//! glTF files: their images decoded, their primitives converted one at a time.

use super::*;

/// A glTF on its way: its images decoding, its primitives converted one
/// at a time.
pub(super) struct Gltf {
    pub(super) file: og_bundle::runtime::UncookedGltf,
    pub(super) images: Vec<(String, Image)>,
    pub(super) prims: Vec<og_bundle::runtime::CookedPrimitive>,
    pub(super) error: Option<String>,
}

/// `og.load.gltfData`: reads the glTF and checks it against its budget
/// (its size, its triangles, each image's header), starts its images'
/// decodes and answers its (pending) asset index.
pub(crate) fn gltf_data(
    eng: &Engine,
    name: &str,
    bytes: Vec<u8>,
    desc: &Value,
) -> Result<usize, String> {
    let budget = Budget::from_desc(desc)?;
    check_size(name, &bytes, &budget)?;
    let mut file =
        og_bundle::runtime::UncookedGltf::read(name, &bytes).map_err(|e| format!("{name}: {e}"))?;
    drop(bytes);
    if file.triangles > budget.triangles as u64 {
        return Err(format!(
            "{name}: {} triangles is over LoadBudget.triangles ({}); simplify the model (a decimate modifier, fewer subdivisions) or raise the budget",
            file.triangles, budget.triangles
        ));
    }
    let images = std::mem::take(&mut file.images);
    let mut headers = Vec::with_capacity(images.len());
    for (what, bytes) in &images {
        headers.push(check_image(&format!("{name}: {what}"), bytes, &budget)?);
    }
    Ok(queue(eng, name, &budget, |u| {
        let images = images
            .into_iter()
            .zip(headers)
            .map(|((what, bytes), header)| (what, start(u, bytes, header)))
            .collect();
        Work::Gltf(Box::new(Gltf {
            file,
            images,
            prims: Vec::new(),
            error: None,
        }))
    }))
}

/// A converted glTF and its decoded images into the renderer, through the
/// path a cooked model takes ([`crate::world::assets::insert_model`]).
pub(super) fn put_gltf(eng: &Engine, name: &str, entry: usize, g: Gltf) -> Result<(), String> {
    if let Some(e) = g.error {
        return Err(e);
    }
    let images = g
        .images
        .into_iter()
        .map(|(what, image)| take_image(image, &what))
        .collect::<Result<Vec<_>, _>>()?;
    use og_bundle::format::kind;
    let textures: Vec<og_render::TextureSource> = g
        .file
        .textures
        .iter()
        .map(|(t, i)| {
            let d = &images[*i];
            og_render::TextureSource::Rgba(og_render::TextureData {
                width: d.width,
                height: d.height,
                rgba: d.rgba.clone(),
                color_space: match t.kind {
                    kind::COLOR | kind::PIXEL => og_render::ColorSpace::Srgb,
                    _ => og_render::ColorSpace::Linear,
                },
                mips: match t.kind {
                    kind::PIXEL => og_render::Mips::None,
                    _ => og_render::Mips::Generate,
                },
            })
        })
        .collect();
    let model = g.file.finish(g.prims);
    let (_, handles) = crate::world::assets::insert_model(
        eng,
        name,
        None,
        model,
        &textures,
        None,
        Some(entry),
        false,
    )?;
    if let Some(owned) = eng.assets.borrow_mut().uncooked.owned.get_mut(&entry) {
        *owned = handles;
    }
    Ok(())
}
