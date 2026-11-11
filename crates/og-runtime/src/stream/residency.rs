//! Streamed asset upgrades, stand-ins and retained resources.

use super::*;

/// What one slot's upgrade puts in place.
enum Full {
    Model {
        meshes: Vec<MeshData>,
        model: Box<og_bundle::runtime::CookedModel>,
        textures: Vec<TextureSource>,
    },
    Texture(TextureSource, bool),
    /// The cooked sound: whole, or the head of one cooked in chunks.
    Sound(Box<CookedSound>),
}

/// A texture over its stand-in, in whichever form the host decoded it to.
pub(crate) fn replace_texture(
    scene: &mut og_render::RenderScene,
    h: og_render::TextureHandle,
    src: TextureSource,
) {
    match src {
        TextureSource::Rgba(d) => scene.set_texture(h, d),
        TextureSource::Encoded(u) => scene.set_texture_upload(h, u),
        // A render target is never streamed: the renderer owns its pixels.
        TextureSource::Target(_) => false,
    };
}

fn decode_texture(
    bundle: &og_bundle::runtime::Bundle,
    files: &Streams,
    group: &str,
    id: &str,
    formats: Option<&og_render::TextureFormats>,
) -> Result<Option<(TextureSource, bool)>, String> {
    let out = bundle.output(id).map_err(|e| e.to_string())?;
    let bytes = super::assets::aligned(files.file(group, &out.file)?);
    let tex = bundle
        .texture(id, bytes.bytes())
        .map_err(|e| e.to_string())?;
    let pixel = tex.is_pixel_art();
    match hooks().map(|h| h.texture(&tex, formats)) {
        Some(super::assets::Fetch::Ready(d)) => Ok(Some((d, pixel))),
        Some(super::assets::Fetch::Pending) => Ok(None),
        Some(super::assets::Fetch::Failed(e)) => Err(format!("{id}: {e}")),
        None => Err("no host is installed to decode textures".into()),
    }
}

/// Puts a resident group's assets in place of their stand-ins. `Ok(false)`:
/// a texture is still being transcoded (nothing changed yet).
pub(super) fn upgrade(eng: &Engine, group: &str) -> Result<bool, String> {
    let formats = super::assets::texture_formats(eng);
    let fulls: Vec<(usize, Full)> = {
        let a = eng.assets.borrow();
        let Some(i) = a.default_bundle else {
            return Ok(true);
        };
        let super::assets::Payload::Bundle { bundle, .. } = &a.entries[i].payload else {
            return Ok(true);
        };
        let s = &a.stream;
        let mut out = Vec::new();
        for (k, (slot, up)) in s.slots.get(group).into_iter().flatten().enumerate() {
            if *up {
                continue;
            }
            let id = slot.id();
            let out_file = &bundle.output(id).map_err(|e| e.to_string())?.file;
            let bytes = super::assets::aligned(s.file(group, out_file)?);
            let full = match slot {
                Slot::Model { .. } => {
                    let model = bundle.model(id, bytes.bytes()).map_err(|e| e.to_string())?;
                    let mut textures = Vec::with_capacity(model.textures.len());
                    for t in &model.textures {
                        match decode_texture(bundle, s, group, &t.id, formats.as_ref())? {
                            Some((d, _)) => textures.push(d),
                            None => return Ok(false),
                        }
                    }
                    Full::Model {
                        meshes: model.primitives.iter().map(|p| p.mesh(0)).collect(),
                        model: Box::new(model),
                        textures,
                    }
                }
                Slot::Texture { .. } => {
                    match decode_texture(bundle, s, group, id, formats.as_ref())? {
                        Some((d, pixel)) => Full::Texture(d, pixel),
                        None => return Ok(false),
                    }
                }
                Slot::Sound { .. } => {
                    let sound = bundle.sound(id, bytes.bytes()).map_err(|e| e.to_string())?;
                    Full::Sound(Box::new(sound))
                }
            };
            out.push((k, full));
        }
        out
    };
    let link = og_audio::link(eng.id);
    let mut a = eng.assets.borrow_mut();
    let mut rs = eng.render.borrow_mut();
    // Sounds cooked in chunks put in place: their stand-in's entry learns
    // its chunks (the playheads ask for them from now on), after the loop.
    let mut chunked: Vec<CookedSound> = Vec::new();
    let Some(slots) = a.stream.slots.get_mut(group) else {
        return Ok(true);
    };
    for (k, full) in fulls {
        let (slot, up) = &mut slots[k];
        match (slot, full) {
            (
                Slot::Model {
                    meshes,
                    materials,
                    textures,
                    ..
                },
                Full::Model {
                    meshes: full,
                    model,
                    textures: data,
                },
            ) => {
                for (i, d) in data.into_iter().enumerate() {
                    match textures.get(i) {
                        Some(&h) => {
                            replace_texture(&mut rs.scene, h, d);
                        }
                        None => textures.push(rs.scene.add_texture_source(d)),
                    }
                }
                for (h, t) in textures.iter().zip(&model.textures) {
                    rs.decoded.ids.insert(*h, t.id.clone());
                }
                for (((h, _), m), prim) in meshes.iter().zip(full).zip(&model.primitives) {
                    rs.scene.set_mesh(*h, m);
                    // The full model's cooked BVH replaces the proxy's.
                    super::geometry::adopt_cooked_bvh(&mut rs, *h, prim)?;
                }
                for (i, (h, _)) in materials.iter().enumerate().take(model.materials.len()) {
                    if model.materials[i].physical.present == 0 {
                        rs.scene.set_material(*h, model.material(i, textures));
                    } else {
                        rs.scene
                            .set_standard_material(*h, model.standard_material(i, textures));
                    }
                }
            }
            (Slot::Texture { id, handle }, Full::Texture(d, pixel)) => {
                replace_texture(&mut rs.scene, *handle, d);
                rs.decoded.ids.insert(*handle, id.clone());
                if pixel {
                    rs.scene
                        .set_texture_filter(*handle, og_render::TextureFilter::Nearest);
                }
            }
            (Slot::Sound { id, handle }, Full::Sound(sound)) => {
                if let Some(l) = &link {
                    if sound.chunked() {
                        // Its first chunk now; the others are read ahead of
                        // its playheads (sound_chunks), as in the first group.
                        l.bank()
                            .replace_chunked(
                                *handle,
                                id,
                                &sound.qoa,
                                sound.meta.chunk_frames,
                                sound.meta.chunk_count,
                            )
                            .map_err(|e| e.reason().to_string())?;
                        chunked.push(*sound);
                    } else if sound.is_opus() {
                        l.bank()
                            .replace_opus(*handle, id, &sound.opus)
                            .map_err(|e| e.reason().to_string())?;
                    } else {
                        l.bank()
                            .replace_qoa(*handle, id, &sound.qoa)
                            .map_err(|e| e.reason().to_string())?;
                    }
                }
            }
            _ => {}
        }
        *up = true;
    }
    for sound in chunked {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(l) = &link
            && let Some(i) = a.sounds.iter().position(|s| s.id == sound.id)
        {
            l.bank()
                .set_chunk_loader(a.sound_handles[i], super::assets::chunk_loader(&a, &sound));
        }
        if let Some(entry) = a.sounds.iter_mut().find(|s| s.id == sound.id) {
            *entry = sound;
        }
    }
    Ok(true)
}

/// The 1x1 a streamed texture shows until its group is resident.
pub(crate) fn placeholder() -> TextureData {
    TextureData {
        width: 1,
        height: 1,
        rgba: vec![128, 128, 128, 255],
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::None,
    }
}

/// Tells the host that `group` let go of its pack and textures.
pub(super) fn release(eng: &Engine, group: &str) {
    let Some(h) = hooks() else {
        return;
    };
    let (pack, textures) = {
        let a = eng.assets.borrow();
        let Some(i) = a.stream.find(group) else {
            return;
        };
        let g = &a.stream.groups[i];
        let textures = a
            .default_bundle
            .and_then(|b| match &a.entries[b].payload {
                super::assets::Payload::Bundle { bundle, .. } => Some(
                    bundle
                        .manifest()
                        .outputs()
                        .filter(|o| o.group.as_deref() == Some(group) && o.ty == "og.Texture")
                        .map(|o| o.id.clone())
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        (format!("{}/{}", g.dir, g.desc.file), textures)
    };
    // The host drops it when it lands; its slot is free now.
    eng.assets.borrow_mut().stream.fetch.in_flight.remove(&pack);
    h.release_stream(&pack, &textures);
}

/// Puts a group's stand-ins back and drops its pack (the host's too), then
/// lets go of its assets without a stand-in.
pub(super) fn downgrade(eng: &Engine, group: &str) {
    put_stand_ins_back(eng, group);
    release_held(eng, group);
    super::cell_groups::forget_scenes(eng, group);
}

fn put_stand_ins_back(eng: &Engine, group: &str) {
    release(eng, group);
    let link = og_audio::link(eng.id);
    let mut a = eng.assets.borrow_mut();
    if let Some(i) = a.stream.find(group) {
        a.stream.groups[i].files.clear();
    }
    let mut rs = eng.render.borrow_mut();
    for (slot, up) in a.stream.slots.get_mut(group).into_iter().flatten() {
        if !*up {
            continue;
        }
        match slot {
            Slot::Model {
                meshes,
                materials,
                textures,
                ..
            } => {
                for (h, m) in meshes.iter() {
                    rs.scene.set_mesh(*h, m.clone());
                }
                for (h, m) in materials.iter() {
                    rs.scene.set_standard_material(*h, *m);
                }
                // The full model's own textures go (its materials are the
                // proxy's again, untextured): the renderer frees them and
                // the next load fills the same slots under new generations.
                for h in textures.drain(..) {
                    rs.scene.remove_texture(h);
                }
            }
            Slot::Texture { handle, .. } => {
                rs.scene.set_texture(*handle, placeholder());
            }
            Slot::Sound { handle, .. } => {
                if let Some(l) = &link {
                    l.bank().silence(*handle);
                }
            }
        }
        *up = false;
    }
}

// ── assets without a stand-in ───────────────────────────────────────────

/// Every environment map, texture array, palette and look loaded from
/// `group`'s pack that the renderer still holds.
fn held_of(eng: &Engine, group: &str) -> Vec<(Held, String)> {
    let loaded: Vec<(Held, String)> = {
        let rs = eng.render.borrow();
        let maps = rs.env.maps.iter().enumerate();
        let maps = maps.filter(|(_, (_, h))| h.is_some());
        let arrays = rs.texture_arrays.iter().enumerate();
        let arrays = arrays.filter(|(_, a)| a.handle.is_some());
        let palettes = rs.fx.look.palettes.iter().enumerate();
        let palettes = palettes.filter(|(_, (_, h))| h.is_some());
        let looks = rs.fx.look.looks.iter().enumerate();
        let looks = looks.filter(|(_, l)| l.resident);
        let luts = rs.fx.luts.iter().enumerate();
        let luts = luts.filter(|(_, l)| l.handle.is_some());
        maps.map(|(i, (id, _))| (Held::Environment(i), id.clone()))
            .chain(arrays.map(|(i, a)| (Held::TextureArray(i), a.id.clone())))
            .chain(palettes.map(|(i, (id, _))| (Held::Palette(i), id.clone())))
            .chain(looks.map(|(i, l)| (Held::Look(i), l.id.clone())))
            .chain(luts.map(|(i, l)| (Held::Lut(i), l.id.clone())))
            .collect()
    };
    let a = eng.assets.borrow();
    loaded
        .into_iter()
        .filter(|(_, id)| a.group_of(id).as_deref() == Some(group))
        .collect()
}

/// Why the frame still needs `held` at tick boundary `tick` (`None`: it
/// does not): what the last frame drew with (its sprites, overlay sprites
/// and particle emitters for an array), and the sprites queued for the
/// next one.
fn in_use(rs: &super::extract::RenderState, held: Held, tick: u64) -> Option<&'static str> {
    match held {
        Held::Environment(i) => {
            let h = rs.env.maps.get(i)?.1?;
            (rs.scene.lighting.environment_map.map(|m| m.map) == Some(h))
                .then_some("og.Environment.map lights the frame with it")
        }
        Held::TextureArray(i) => {
            let h = rs.texture_arrays.get(i)?.handle?;
            // The queue is cleared lazily: one older than the last tick is stale.
            let queued = rs.texts_tick + 1 >= tick;
            let overlay = rs.scene.ui.overlay.iter().flat_map(|o| &o.sprite_arrays);
            let drawn = rs
                .scene
                .sprite_arrays
                .iter()
                .chain(overlay)
                .any(|s| s.array == h)
                || queued
                    && rs
                        .fx
                        .sprites
                        .iter()
                        .any(|q| q.array.map(|a| a.0) == Some(h));
            if drawn {
                return Some("a sprite draws from it");
            }
            let emitted = rs
                .scene
                .effects
                .particles
                .iter()
                .any(|p| rs.scene.emitter(p.emitter).and_then(|e| e.array) == Some(h));
            if emitted {
                return Some("a particle emitter draws from it");
            }
            let slot = h.slot();
            let sampled = rs
                .materials
                .iter()
                .filter_map(|&m| rs.scene.standard_material(m))
                .any(|m| m.mapArray == slot || m.normalMapArray == slot);
            if sampled {
                return Some("a material samples it (mapArray, normalMapArray)");
            }
            let layered = rs
                .terrain
                .loaded
                .iter()
                .any(|t| t.layer_arrays.contains(&Some(i)));
            layered.then_some("a terrain's layers sample it (layers, layerNormals)")
        }
        Held::Palette(i) => {
            let h = rs.fx.look.palettes.get(i)?.1?;
            (rs.scene.look.stylise.quantize.and_then(|q| q.palette) == Some(h))
                .then_some("Stylise.palette quantises the frame to it")
        }
        Held::Look(i) => (rs.fx.look.applied == Some(i)).then_some("it is the look applied"),
        Held::Lut(i) => {
            let h = rs.fx.luts.get(i)?.handle?;
            (rs.scene.look.post.color_grading.map(|g| g.lut) == Some(h))
                .then_some("PostEffects.lut grades the frame with it")
        }
    }
}

/// What `held` takes on the GPU (a look's values take none).
fn bytes_of(rs: &super::extract::RenderState, held: Held) -> u64 {
    let scene = &rs.scene;
    match held {
        Held::Environment(i) => rs
            .env
            .maps
            .get(i)
            .and_then(|m| m.1)
            .and_then(|h| scene.environment_map_data(h))
            .map_or(0, |m| {
                4 * og_render::EnvironmentMapData::texel_count(m.size) as u64
            }),
        Held::TextureArray(i) => rs
            .texture_arrays
            .get(i)
            .and_then(|a| a.handle)
            .and_then(|h| scene.texture_array(h))
            .and_then(|d| d.info().ok())
            .map_or(0, |info| info.bytes),
        Held::Palette(_) => og_render::MAX_PALETTE_COLORS as u64 * 4,
        Held::Look(_) => 0,
        Held::Lut(i) => rs
            .fx
            .luts
            .get(i)
            .and_then(|l| l.handle)
            .and_then(|h| scene.lut(h))
            .map_or(0, |l| u64::from(l.size).pow(3) * 4),
    }
}

/// Lets `held` go: out of the renderer, its handle pending.
fn free(rs: &mut super::extract::RenderState, held: Held) {
    match held {
        Held::Environment(i) => {
            if let Some(h) = rs.env.maps.get_mut(i).and_then(|m| m.1.take()) {
                rs.scene.remove_environment_map(h);
            }
        }
        Held::TextureArray(i) => {
            if let Some(h) = rs.texture_arrays.get_mut(i).and_then(|a| a.handle.take()) {
                rs.scene.remove_texture_array(h);
            }
        }
        Held::Palette(i) => {
            if let Some(h) = rs.fx.look.palettes.get_mut(i).and_then(|p| p.1.take()) {
                rs.scene.remove_palette(h);
            }
        }
        Held::Look(i) => {
            if let Some(l) = rs.fx.look.looks.get_mut(i) {
                l.values = Vec::new();
                l.resident = false;
            }
        }
        Held::Lut(i) => {
            if let Some(h) = rs.fx.luts.get_mut(i).and_then(|l| l.handle.take()) {
                rs.scene.remove_lut(h);
            }
        }
    }
}

/// The unloaded `group`'s environment maps, texture arrays, palettes,
/// looks and LUTs: freed, or kept with a warning while the frame uses them.
fn release_held(eng: &Engine, group: &str) {
    let tick = tick_of(eng);
    for (held, id) in held_of(eng, group) {
        let (why, bytes) = {
            let mut rs = eng.render.borrow_mut();
            match in_use(&rs, held, tick) {
                Some(why) => (why, bytes_of(&rs, held)),
                None => {
                    free(&mut rs, held);
                    continue;
                }
            }
        };
        let mut a = eng.assets.borrow_mut();
        if a.stream.retained.iter().any(|r| r.held == held) {
            continue;
        }
        a.stream.retained.push(Retained {
            group: group.to_string(),
            held,
            id: id.clone(),
            bytes,
        });
        drop(a);
        eng.state.borrow_mut().log.push(json!({
            "tick": tick,
            "warning": true,
            "text": format!(
                "streaming group '{group}' unloaded, but {id} is kept: {why}; it is let go once the frame stops using it"
            ),
        }));
    }
}

/// At a tick boundary: the kept assets the frame no longer uses are let
/// go; those whose group is resident again stay as they are.
pub(super) fn retry_retained(eng: &Engine) {
    let tick = tick_of(eng);
    let retained = std::mem::take(&mut eng.assets.borrow_mut().stream.retained);
    let mut keep = Vec::new();
    for r in retained {
        if eng.assets.borrow().stream.resident(&r.group) {
            continue;
        }
        let mut rs = eng.render.borrow_mut();
        if in_use(&rs, r.held, tick).is_some() {
            keep.push(r);
        } else {
            free(&mut rs, r.held);
        }
    }
    eng.assets.borrow_mut().stream.retained.extend(keep);
}

/// At a tick boundary: the environment maps, texture arrays, palettes,
/// looks and LUTs a resident group's pack holds that are not in the
/// renderer (their group unloaded them, or they were loaded before it was
/// resident) are added again under their handles.
pub(crate) fn resolve(eng: &Engine) {
    super::environment::resolve(eng);
    super::assets::resolve_texture_arrays(eng);
    super::look::resolve(eng);
    super::fx::resolve_luts(eng);
}

// ── the loads' side ─────────────────────────────────────────────────────

/// A grouped model's proxy was added: its meshes and materials are the
/// handles the full model takes over.
pub(crate) fn add_model(
    eng: &Engine,
    group: &str,
    id: &str,
    meshes: Vec<(MeshHandle, MeshData)>,
    materials: Vec<(MaterialHandle, StandardMaterial)>,
) {
    push(
        eng,
        group,
        Slot::Model {
            id: id.to_string(),
            meshes,
            materials,
            textures: Vec::new(),
        },
    );
}

pub(crate) fn add_texture(eng: &Engine, group: &str, id: &str, handle: TextureHandle) {
    push(
        eng,
        group,
        Slot::Texture {
            id: id.to_string(),
            handle,
        },
    );
}

/// A grouped sound: silence of its shape in the bank, and the stand-in the
/// tick side reads (its rate, channels and length; no samples).
pub(crate) fn add_sound(eng: &Engine, group: &str, id: &str) -> Result<(CookedSound, u64), String> {
    let shape = {
        let a = eng.assets.borrow();
        let i = a.stream.find(group);
        i.and_then(|i| {
            a.stream.groups[i]
                .desc
                .sounds
                .iter()
                .find(|s| s.id == id)
                .cloned()
        })
        .ok_or_else(|| {
            format!("{id}: its group '{group}' does not list its shape; run `og cook`")
        })?
    };
    let link = og_audio::link(eng.id).ok_or_else(|| format!("{id}: this engine has no audio"))?;
    let handle = link
        .bank()
        .insert_silent(id, shape.sample_rate, shape.channels as usize, shape.frames)
        .map_err(|e| e.reason().to_string())?;
    push(
        eng,
        group,
        Slot::Sound {
            id: id.to_string(),
            handle,
        },
    );
    let sound = CookedSound {
        id: id.to_string(),
        meta: og_bundle::format::SoundMeta {
            sample_rate: shape.sample_rate,
            channels: shape.channels,
            frames: shape.frames.min(u32::MAX as u64) as u32,
            ..Default::default()
        },
        qoa: Vec::new(),
        opus: Vec::new(),
    };
    Ok((sound, handle))
}
