//! Cooked bytes, not decoded upload copies, back resident GPU textures.

use super::*;

pub(crate) fn release_uploaded_textures(
    eng: &Engine,
    rs: &mut crate::present::extract::RenderState,
) {
    let Some(backend) = &rs.backend else { return };
    let formats = backend.texture_formats();
    for (&texture, id) in &rs.decoded.ids {
        if rs.decoded.pending.iter().any(|(h, _)| *h == texture)
            || !backend.texture_uploaded(&rs.scene, texture)
        {
            continue;
        }
        let owner = rs.decoded.bundles.get(&texture).copied();
        let id = {
            let a = eng.assets.borrow();
            if a.group_from(id, owner)
                .is_some_and(|g| !a.stream.resident(&g))
            {
                continue;
            }
            // A pending swap still displays its old source; a completed
            // swap changes the revision and resets its recovery closure.
            if a.bundle_from(&og_bundle::runtime::mips_id(id), owner)
                .is_ok()
            {
                let Some(id) = crate::world::stream::mips::recovery_id(&a, texture) else {
                    continue;
                };
                id
            } else {
                id.clone()
            }
        };
        let engine_id = eng.id;
        let formats = formats.clone();
        rs.scene.release_texture_source(texture, move || {
            let eng = crate::engine::engine(engine_id)?;
            let a = eng.assets.borrow();
            let (bytes, bundle) = a.blob_from(&id, owner)?;
            let tex = bundle
                .texture(&id, bytes.bytes())
                .map_err(|e| e.to_string())?;
            match hooks().map(|h| h.texture(&tex, formats.as_ref())) {
                Some(Fetch::Ready(source)) => Ok(source),
                Some(Fetch::Pending) => Err(format!("{id}: texture recovery is pending")),
                Some(Fetch::Failed(reason)) => Err(format!("{id}: {reason}")),
                None => Err("no host is installed to recover textures".into()),
            }
        });
    }
}
