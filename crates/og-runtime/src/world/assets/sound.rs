//! Cooked sounds.

use super::*;

pub(crate) fn load_sound(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(i) = eng.assets.borrow().sounds.iter().position(|s| s.id == id) {
        return Ok(i);
    }
    let group = eng.assets.borrow().group_of(&id);
    if let Some(group) = group {
        // Silence of its shape until its group is resident.
        let (sound, handle) = crate::world::stream::add_sound(eng, &group, &id)?;
        let mut assets = eng.assets.borrow_mut();
        assets.sounds.push(sound);
        assets.sound_handles.push(handle);
        return Ok(assets.sounds.len() - 1);
    }
    let sound = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .sound(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    // The engine's audio bank holds the sound the mixer plays; its handle is
    // the one AudioSource.sound names.
    let handle = match og_audio::link(eng.id) {
        // Cooked in chunks: the first is here, the others are read as the
        // playheads near them (stream::sound_chunks).
        Some(link) if sound.chunked() => link
            .bank()
            .insert_chunked(
                &sound.id,
                &sound.qoa,
                sound.meta.chunk_frames,
                sound.meta.chunk_count,
            )
            .map_err(|e| e.reason().to_string())?,
        Some(link) if sound.is_opus() => link
            .bank()
            .insert_opus(&sound.id, &sound.opus)
            .map_err(|e| e.reason().to_string())?,
        Some(link) => link
            .bank()
            .insert_qoa(&sound.id, &sound.qoa)
            .map_err(|e| e.reason().to_string())?,
        None => return Err(format!("{id}: this engine has no audio")),
    };
    #[cfg(not(target_arch = "wasm32"))]
    if sound.chunked()
        && let Some(link) = og_audio::link(eng.id)
    {
        link.bank()
            .set_chunk_loader(handle, chunk_loader(&eng.assets.borrow(), &sound));
    }
    let mut assets = eng.assets.borrow_mut();
    assets.sounds.push(sound);
    assets.sound_handles.push(handle);
    Ok(assets.sounds.len() - 1)
}

/// Natively, a chunk a voice reaches before stream::sound_chunks put it in
/// the bank (a long advance mixed at once) is read from its file there
/// and then, on whichever thread mixes, its type and cook stamp checked.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn chunk_loader(assets: &Assets, sound: &CookedSound) -> og_audio::ChunkLoader {
    let files: Vec<Option<(String, u64)>> = (0..sound.meta.chunk_count)
        .map(|n| {
            let id = og_bundle::runtime::chunk_id(&sound.id, n);
            let (bundle, dir) = assets.bundle_for(&id).ok()?;
            let out = bundle.output(&id).ok()?;
            let path = hooks()?.asset_file(&format!("{dir}/{}", out.file));
            Some((path, out.stamp))
        })
        .collect();
    std::sync::Arc::new(move |n| {
        let (path, stamp) = files.get(n as usize)?.as_ref()?;
        let bytes = std::fs::read(path).ok()?;
        match og_bundle::runtime::sound_chunk_of(&bytes, path, *stamp) {
            Ok((k, qoa)) if k == n => Some(qoa),
            _ => None,
        }
    })
}

/// The `sounds` index of a sound handle.
pub(crate) fn sound_index(eng: &Engine, handle: f64) -> Option<usize> {
    let bits = handle as u64;
    (handle >= 0.0 && handle.fract() == 0.0)
        .then(|| {
            eng.assets
                .borrow()
                .sound_handles
                .iter()
                .position(|h| *h == bits)
        })
        .flatten()
}

// ── scene spawning ──────────────────────────────────────────────────────────
