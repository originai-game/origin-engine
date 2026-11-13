//! Cooked textures and texture arrays: decoding, transcoding and linking.

use super::*;

/// The encodings the attached renderer takes (`None` before one is attached:
/// textures then load as RGBA8).
pub(crate) fn texture_formats(eng: &Engine) -> Option<og_render::TextureFormats> {
    eng.render
        .borrow()
        .backend
        .as_ref()
        .and_then(|b| b.texture_formats())
}

/// `og.load.texture`: a cooked texture added to the renderer once; the
/// answer is its index in the render state's textures.
pub(crate) fn load_texture(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.assets.borrow().by_path.get(&format!("texture:{id}")) {
        return Ok(i);
    }
    let group = eng.assets.borrow().group_of(&id);
    if let Some(group) = group {
        // A placeholder until its group is resident, then the texture.
        let mut rs = eng.render.borrow_mut();
        let handle = rs.scene.add_texture(crate::world::stream::placeholder());
        rs.textures.push(handle);
        let i = rs.textures.len() - 1;
        drop(rs);
        eng.assets
            .borrow_mut()
            .by_path
            .insert(format!("texture:{id}"), i);
        crate::world::stream::add_texture(eng, &group, &id, handle);
        return Ok(i);
    }
    let (data, pixel) = decode_texture(eng, &id)?;
    let mut rs = eng.render.borrow_mut();
    let handle = rs.scene.add_texture_source(data);
    rs.decoded.ids.insert(handle, id.clone());
    if pixel {
        rs.scene
            .set_texture_filter(handle, og_render::TextureFilter::Nearest);
    }
    rs.textures.push(handle);
    let i = rs.textures.len() - 1;
    drop(rs);
    eng.assets
        .borrow_mut()
        .by_path
        .insert(format!("texture:{id}"), i);
    link_index(eng, &id, handle);
    link_normal(eng, &id, handle);
    link_conformed(eng, &id);
    Ok(i)
}

/// A conformed texture's own palette (the cook's `<id>#palette`, written
/// where the art contract's maxColors or rampSteps cut it): its colours
/// join the renderer's conformed set, so the look's frame quantise leaves
/// this art as the cook made it and steps only what it did not conform.
fn link_conformed(eng: &Engine, id: &str) {
    let name = format!("{id}#palette");
    let colors = {
        let assets = eng.assets.borrow();
        let Ok((bytes, bundle)) = assets.blob(&name) else {
            return;
        };
        bundle.palette(&name, bytes.bytes())
    };
    if let Ok(colors) = colors {
        eng.render.borrow_mut().scene.add_conformed_colors(&colors);
    }
}

/// A 2D texture's normal map: the cook's `<id>#normal` beside it (a
/// `normal` line in its import settings, og-cook's normal2d.rs), added to
/// the renderer only and named its normal, so 2D lighting shades every
/// sprite, tile and 2D rig drawn with it in relief. None without one.
fn link_normal(eng: &Engine, id: &str, texture: og_render::TextureHandle) {
    let name = format!("{id}#normal");
    if eng.assets.borrow().blob(&name).is_err() {
        return;
    }
    let decoded = decode_texture(eng, &name);
    let mut rs = eng.render.borrow_mut();
    match decoded {
        Ok((data, _)) => {
            let h = rs.scene.add_texture_source(data);
            rs.decoded.ids.insert(h, name);
            rs.scene.set_texture_normal(texture, Some(h));
        }
        Err(e) => rs
            .notes
            .push(format!("{name}: not loaded ({e}); the texture is lit flat")),
    }
}

/// A cooked texture decoded through the host's texture hook, and whether
/// its kind is pixel art (sampled nearest).
fn decode_texture(eng: &Engine, id: &str) -> Result<(og_render::TextureSource, bool), String> {
    match decode(eng, id) {
        Fetch::Ready(d) => Ok(d),
        Fetch::Pending => Err(format!("{id}: the texture is still being transcoded")),
        Fetch::Failed(e) => Err(e),
    }
}

/// [`decode_texture`], with a texture the host is still transcoding
/// answered `Pending`.
pub(super) fn decode(eng: &Engine, id: &str) -> Fetch<(og_render::TextureSource, bool)> {
    let formats = texture_formats(eng);
    let assets = eng.assets.borrow();
    let tex = match assets
        .blob(id)
        .and_then(|(bytes, bundle)| bundle.texture(id, bytes.bytes()).map_err(|e| e.to_string()))
    {
        Ok(t) => t,
        Err(e) => return Fetch::Failed(e),
    };
    let pixel = tex.is_pixel_art();
    match hooks().map(|h| h.texture(&tex, formats.as_ref())) {
        Some(Fetch::Ready(d)) => Fetch::Ready((d, pixel)),
        Some(Fetch::Pending) => Fetch::Pending,
        Some(Fetch::Failed(e)) => Fetch::Failed(format!("{id}: {e}")),
        None => Fetch::Failed("no host is installed to decode textures".into()),
    }
}

/// The cooked textures in the renderer and the device encodings they
/// were last checked against. A renderer made again after a lost device
/// may take other encodings than the old one did (another adapter, a
/// browser that came back without BC): what it does not take is decoded
/// again from the bundle for it ([`retranscode`]), never uploaded as the
/// old device's encoding.
#[derive(Default)]
pub(crate) struct Decoded {
    /// Each scene texture decoded from a cooked texture, with its asset id.
    pub ids: HashMap<og_render::TextureHandle, String>,
    /// Colour textures a material was given as a normal map, each warned
    /// about once (`api::render`).
    pub color_normals: std::collections::HashSet<og_render::TextureHandle>,
    /// The encodings of the last check.
    formats: Option<og_render::TextureFormats>,
    /// Textures the host is still transcoding for the new device, drawn
    /// as the streaming placeholder meanwhile.
    pub(super) pending: Vec<(og_render::TextureHandle, String)>,
    /// Texture arrays (by load index) let go of until the host has
    /// transcoded their layers, added again by [`resolve_texture_arrays`].
    arrays_pending: Vec<usize>,
}

/// Before a frame, after the backend recovered from a lost device: when
/// the device's texture encodings changed since the last frame, every
/// cooked texture and texture array the renderer holds in an encoding the
/// device does not take is decoded again from the bundle through the
/// host's texture hook, for the new encodings. Natively the hook
/// transcodes at once, so the frame draws as it did before the loss; the
/// web host's Worker transcodes in the background, and those textures
/// draw as the placeholder (arrays not at all) until they land.
pub(crate) fn retranscode(eng: &Engine) {
    let Some(formats) = texture_formats(eng) else {
        return;
    };
    let takes_upload = |u: &og_render::TextureUpload| formats.supported.contains(&u.desc.format);
    let takes = |s: &og_render::TextureSource| match s {
        og_render::TextureSource::Encoded(u) => takes_upload(u),
        _ => true,
    };
    let (stale, arrays) = {
        let mut rs = eng.render.borrow_mut();
        let rs = &mut *rs;
        let mut stale = std::mem::take(&mut rs.decoded.pending);
        let mut arrays = std::mem::take(&mut rs.decoded.arrays_pending);
        if rs.decoded.formats.as_ref() != Some(&formats) {
            rs.decoded.formats = Some(formats.clone());
            let scene = &rs.scene;
            rs.decoded.ids.retain(|h, _| scene.contains_texture(*h));
            stale.extend(
                rs.decoded
                    .ids
                    .iter()
                    .filter(|(h, _)| scene.texture_upload(**h).is_some_and(|u| !takes_upload(u)))
                    .map(|(h, id)| (*h, id.clone())),
            );
            for (i, a) in rs.texture_arrays.iter().enumerate() {
                let old = a
                    .handle
                    .and_then(|h| scene.texture_array(h))
                    .is_some_and(|d| !d.layers.iter().all(takes));
                if old {
                    arrays.push(i);
                }
            }
        }
        (stale, arrays)
    };
    let mut pending = Vec::new();
    for (h, id) in stale {
        let decoded = decode(eng, &id);
        let mut rs = eng.render.borrow_mut();
        match decoded {
            Fetch::Ready((source, _)) => {
                crate::world::stream::replace_texture(&mut rs.scene, h, source);
            }
            Fetch::Pending => {
                rs.scene.set_texture(h, crate::world::stream::placeholder());
                pending.push((h, id));
            }
            Fetch::Failed(e) => {
                rs.scene.set_texture(h, crate::world::stream::placeholder());
                rs.notes.push(format!(
                    "{id}: not decoded again for the new GPU device ({e}); drawn as a placeholder"
                ));
            }
        }
        // A texture streamed by its mips no longer holds its whole chain.
        drop(rs);
        crate::world::stream::mips::reset(eng, h);
    }
    let mut arrays_pending = Vec::new();
    for i in arrays {
        let Some((id, handle)) = eng
            .render
            .borrow()
            .texture_arrays
            .get(i)
            .map(|a| (a.id.clone(), a.handle))
        else {
            continue;
        };
        let Some(h) = handle else {
            // Let go of below on an earlier frame: added again once its
            // layers decode.
            if let Ok((added, _)) = add_texture_array(eng, &id) {
                eng.render.borrow_mut().texture_arrays[i].handle = Some(added);
            } else {
                arrays_pending.push(i);
            }
            continue;
        };
        let data = texture_array_data(eng, &id);
        let mut rs = eng.render.borrow_mut();
        let replaced = data.and_then(|(d, _)| rs.scene.set_texture_array(h, d));
        if replaced.is_err() {
            rs.scene.remove_texture_array(h);
            rs.texture_arrays[i].handle = None;
            arrays_pending.push(i);
        }
    }
    let mut rs = eng.render.borrow_mut();
    rs.decoded.pending = pending;
    rs.decoded.arrays_pending = arrays_pending;
}

/// A palette swap's index texture (docs/art.md §2.4,
/// `crate::present::art::ArtState::index_of`): `texture` itself when its id is an
/// index texture (`<id>#index`), else the cook's `<id>#index` beside it,
/// added to the renderer only (never a Texture handle of the game's, so
/// the handles a game holds are the same with and without the contract).
fn link_index(eng: &Engine, id: &str, texture: og_render::TextureHandle) {
    let index = if id.ends_with("#index") {
        Some(texture)
    } else {
        let name = format!("{id}#index");
        let known = eng.assets.borrow().blob(&name).is_ok();
        match known.then(|| decode_texture(eng, &name)) {
            Some(Ok((data, pixel))) => {
                let mut rs = eng.render.borrow_mut();
                let h = rs.scene.add_texture_source(data);
                if pixel {
                    rs.scene
                        .set_texture_filter(h, og_render::TextureFilter::Nearest);
                }
                rs.decoded.ids.insert(h, name);
                Some(h)
            }
            _ => None,
        }
    };
    if let Some(index) = index {
        eng.render.borrow_mut().art.index_of.insert(texture, index);
    }
}

/// Reads and decodes a cooked texture array's layers through the host's
/// texture hook, and adds them to the renderer as one array (sampled
/// nearest when its kind is pixel art).
fn add_texture_array(
    eng: &Engine,
    id: &str,
) -> Result<(og_render::TextureArrayHandle, og_render::TextureArrayInfo), String> {
    let (data, pixel) = texture_array_data(eng, id)?;
    let info = data.info().map_err(|e| format!("{id}: {e}"))?;
    let mut rs = eng.render.borrow_mut();
    let added = rs
        .scene
        .add_texture_array(data)
        .map_err(|e| format!("{id}: {e}"))?;
    if pixel {
        rs.scene
            .set_texture_array_filter(added, og_render::TextureFilter::Nearest);
    }
    Ok((added, info))
}

/// A cooked texture array's layers, decoded through the host's texture
/// hook into the encodings the device takes, and whether its kind is pixel
/// art (sampled nearest).
pub(crate) fn texture_array_data(
    eng: &Engine,
    id: &str,
) -> Result<(og_render::TextureArrayData, bool), String> {
    let formats = texture_formats(eng);
    let (array, layers, pixel) = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(id)?;
        let array = bundle
            .texture_array(id, bytes.bytes())
            .map_err(|e| e.to_string())?;
        let mut layers = Vec::with_capacity(array.layers.len());
        let mut pixel = false;
        for l in &array.layers {
            let (bytes, bundle) = assets.blob(l)?;
            let tex = bundle
                .texture(l, bytes.bytes())
                .map_err(|e| e.to_string())?;
            pixel = tex.is_pixel_art();
            match hooks().map(|h| h.texture(&tex, formats.as_ref())) {
                Some(Fetch::Ready(d)) => layers.push(d),
                Some(Fetch::Pending) => {
                    return Err(format!("{l}: the layer is still being transcoded"));
                }
                Some(Fetch::Failed(e)) => return Err(format!("{l}: {e}")),
                None => return Err("no host is installed to decode textures".into()),
            }
        }
        (array, layers, pixel)
    };
    if layers.len() != array.meta.layers as usize {
        return Err(format!(
            "{id}: {} layers, its meta says {}",
            layers.len(),
            array.meta.layers
        ));
    }
    Ok((og_render::TextureArrayData { layers }, pixel))
}

/// `og.load.textureArray`: a cooked `type texture-array`, its layers
/// decoded through the host's texture hook (each an `og.Texture` the web
/// Worker transcoded with the bundle) and uploaded as one 2D-array
/// texture. Loading a path again answers the same array. One in a
/// streaming group's pack loads once its group is resident; when the
/// group unloads the renderer lets the array go (`crate::world::stream`) and the
/// handle draws nothing until the group is resident again.
pub(crate) fn load_texture_array(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(v) = eng.assets.borrow().texture_arrays.get(&id) {
        return Ok(v.clone());
    }
    load_new_texture_array(eng, &id)
}

/// [`load_texture_array`]'s load index (what a TextureArray handle and
/// RenderState::texture_arrays index): for the engine's own loads of an
/// array (a flipbook's, an emitter's).
pub(crate) fn load_texture_array_index(eng: &Engine, path: &str) -> Result<usize, String> {
    let v = load_texture_array(eng, path)?;
    <crate::host::wire::TextureArrayInfo as crate::host::wire::FromJson>::from_json(&v)
        .ok()
        .and_then(|info| index_of(info.array as f64))
        .ok_or_else(|| format!("{path}: no texture array handle"))
}

fn load_new_texture_array(eng: &Engine, id: &str) -> Result<Value, String> {
    let id = id.to_string();
    let (added, info) = add_texture_array(eng, &id)?;
    let index = {
        let mut rs = eng.render.borrow_mut();
        rs.texture_arrays.push(LoadedArray {
            id: id.clone(),
            handle: Some(added),
            layers: info.layers,
            size: (info.width, info.height),
        });
        rs.texture_arrays.len() - 1
    };
    let value = json!({
        "array": handle(index) as f64,
        "width": info.width,
        "height": info.height,
        "layers": info.layers,
        "levels": info.levels,
        "format": info.format.name(),
        "bytes": info.bytes as f64,
    });
    eng.assets
        .borrow_mut()
        .texture_arrays
        .insert(id, value.clone());
    Ok(value)
}

/// At a tick boundary: adds again the texture arrays whose streaming group
/// is resident again after an unload released them.
pub(crate) fn resolve_texture_arrays(eng: &Engine) {
    let pending: Vec<(usize, String)> = eng
        .render
        .borrow()
        .texture_arrays
        .iter()
        .enumerate()
        .filter(|(_, a)| a.handle.is_none())
        .map(|(i, a)| (i, a.id.clone()))
        .collect();
    for (i, id) in pending {
        if let Ok((h, _)) = add_texture_array(eng, &id) {
            eng.render.borrow_mut().texture_arrays[i].handle = Some(h);
        }
    }
}
