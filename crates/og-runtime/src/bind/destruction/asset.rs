//! Reading a model's fracture asset and giving physics its chunk hulls.

use super::*;

/// Reads model `id`'s fracture asset (`<id>#fracture`), when the cook wrote
/// one, for the model loaded as asset `index`, and gives physics its
/// chunks' hulls. A model without one forgets any it had (a hot reload).
pub(crate) fn load_asset(eng: &Engine, id: &str, index: usize) -> Result<(), String> {
    let handle = crate::world::assets::handle(index);
    let blob = format!("{id}#fracture");
    let bytes = {
        let assets = eng.assets.borrow();
        let has = assets
            .bundle_for(id)
            .is_ok_and(|(b, _)| b.output(&blob).is_ok());
        if !has {
            lock(&eng.modules.destruction.assets).remove(&handle);
            return Ok(());
        }
        let (bytes, bundle) = assets.blob(&blob)?;
        bundle
            .fracture(&blob, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let a = FractureAsset::from_bytes(&bytes).map_err(|e| format!("{blob}: {e}"))?;
    if a.chunks.len() > 0x1_0000 || index > 0xff_ffff {
        return Err(format!(
            "{blob}: {} chunks, past the 65536 a model holds",
            a.chunks.len()
        ));
    }
    if eng.state.borrow().physics.is_some() {
        for (i, c) in a.chunks.iter().enumerate() {
            let mesh = og_physics::MeshDesc {
                positions: c.hull.clone(),
                indices: Vec::new(),
            };
            crate::bind::modules::set_mesh(eng, hull_handle(handle, i as u32), mesh)?;
        }
    }
    lock(&eng.modules.destruction.assets).insert(handle, Arc::new(a));
    Ok(())
}
