//! Effect materials (og-render's `EffectMaterial`, an `.ogt` of `type
//! effect`): loaded once per path with the textures and hook shader they
//! name, for emitters (`effect "<id>"`) to draw with.

use std::collections::BTreeMap;

/// The effect materials loaded, by load index, and their paths.
#[derive(Default)]
pub(crate) struct EffectState {
    pub handles: Vec<og_render::EffectMaterialHandle>,
    pub paths: BTreeMap<String, usize>,
}

/// An effect material's renderer handle, loaded once per path.
pub(crate) fn load_effect(
    eng: &super::engine::Engine,
    path: &str,
) -> Result<og_render::EffectMaterialHandle, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().effects.paths.get(&id) {
        return Ok(eng.render.borrow().effects.handles[i]);
    }
    let text = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .effect_text(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let material = og_bundle::effect::parse_effect(
        &text,
        &mut og_bundle::effect::EffectLinks {
            texture: &mut |tex| {
                let i = super::assets::load_texture(eng, tex)?;
                Ok(eng.render.borrow().textures.get(i).copied())
            },
            shader: &mut |s| {
                let i = super::atmosphere::load_shader(eng, s)?;
                let rs = eng.render.borrow();
                let h = rs.atmo.shaders[i];
                let params = rs.scene.shaders()[h.index()].params.clone();
                Ok(Some((h, params)))
            },
        },
    )
    .map_err(|e| format!("{id}: {e}"))?;
    let mut rs = eng.render.borrow_mut();
    let h = rs.scene.add_effect_material(material);
    rs.effects.handles.push(h);
    let i = rs.effects.handles.len() - 1;
    rs.effects.paths.insert(id, i);
    Ok(h)
}
