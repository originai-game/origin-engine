//! The assets an element draws with, `og.load.vector` and
//! `og.load.uiMaterial` (`docs/ui.md` §3.3), and their hot reload.

use crate::engine::engine;
use crate::world::assets::handle;
use og_ui::paint::*;
use og_ui::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// What a world's renderer holds of this module.
#[derive(Default)]
pub(crate) struct UiDrawState {
    /// Vectors by VectorArt handle index, and the asset each came from.
    vectors: Vec<UiVectorHandle>,
    vector_paths: BTreeMap<String, usize>,
    /// UI materials by UiMaterial handle index, their parameters' defaults,
    /// and the asset each came from.
    materials: Vec<(UiMaterialHandle, [[f32; 4]; og_render::UI_MATERIAL_PARAMS])>,
    material_paths: BTreeMap<String, (usize, u64)>,
}

/// The UI tree plus the loaded materials' own parameter declarations.
pub(crate) struct MaterialProbe {
    pub ui: Box<dyn og_probe::UiTree>,
    pub render: std::rc::Rc<std::cell::RefCell<crate::present::extract::RenderState>>,
}

impl og_probe::UiTree for MaterialProbe {
    fn elements(&self, world: &og_ecs::World) -> Vec<og_probe::UiElement> {
        self.ui.elements(world)
    }

    fn surface(&self, world: &og_ecs::World) -> Option<(f32, f32, f32)> {
        self.ui.surface(world)
    }

    fn style(&self, world: &og_ecs::World) -> Option<String> {
        self.ui.style(world)
    }

    fn screens(&self, world: &og_ecs::World) -> Vec<String> {
        self.ui.screens(world)
    }

    fn extra(&self, world: &og_ecs::World) -> Option<serde_json::Map<String, Value>> {
        let mut extra = self.ui.extra(world).unwrap_or_default();
        let render = self.render.borrow();
        let materials: Vec<Value> = render.ui_draw.material_paths.iter().map(|(id, (i, _))| {
            let (handle, values) = &render.ui_draw.materials[*i];
            let material = &render.scene.ui.ui_materials[handle.index()];
            json!({ "path": id, "parameters": material.params.iter().enumerate().map(|(slot, p)| {
                let count = p.kind.count();
                json!({ "name": p.name, "type": p.kind.wgsl(), "default": p.default[..count], "value": values[slot][..count] })
            }).collect::<Vec<_>>() })
        }).collect();
        extra.insert("materials".into(), json!(materials));
        Some(extra)
    }
}

/// Tune a material's defaults, without cooking or recompiling its pipeline.
/// Explicit per-layer parameter values still take precedence.
pub(crate) fn tune_material(
    eng: &crate::engine::Engine,
    path: &str,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let mut render = eng.render.borrow_mut();
    let &(i, _) = render
        .ui_draw
        .material_paths
        .get(&id)
        .ok_or_else(|| format!("UI material '{path}' is not loaded"))?;
    let handle = render.ui_draw.materials[i].0;
    let material = &render.scene.ui.ui_materials[handle.index()];
    let (slot, kind) = material
        .param(name)
        .ok_or_else(|| format!("UI material '{path}' has no parameter '{name}'"))?;
    let mut values = material.params[slot].default;
    if !value.trim().is_empty() {
        let numbers: Vec<f32> = value
            .split_whitespace()
            .map(|s| {
                s.parse::<f32>()
                    .map_err(|_| format!("'{s}' is not a number"))
            })
            .collect::<Result<_, _>>()?;
        if numbers.len() != kind.count() || numbers.iter().any(|n| !n.is_finite()) {
            return Err(format!("'{name}' takes {} finite numbers", kind.count()));
        }
        values[..numbers.len()].copy_from_slice(&numbers);
    }
    render.ui_draw.materials[i].1[slot] = values;
    let defaults = render.ui_draw.materials[i].1;
    drop(render);
    if let Some(ui) = &eng.state.borrow().ui {
        ui.lock().define_material(&id, handle, defaults);
    }
    Ok(())
}

/// `og.load.vector`: a cooked vector, added to the scene once per path.
pub fn og_load_vector(world: u32, path: String) -> Result<serde_json::Value, String> {
    (|| -> Result<Value, String> {
        let eng = engine(world)?;
        Ok(json!(handle(load_vector(&eng, &path)?.0)))
    })()
}

/// A cooked vector added to the scene once per path: its VectorArt index
/// and its handle.
pub(crate) fn load_vector(
    eng: &crate::engine::Engine,
    path: &str,
) -> Result<(usize, UiVectorHandle), String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().ui_draw.vector_paths.get(&id) {
        return Ok((i, eng.render.borrow().ui_draw.vectors[i]));
    }
    let vector = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .vector(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let mut rs = eng.render.borrow_mut();
    let h = rs.scene.add_ui_vector(vector);
    let st = &mut rs.ui_draw;
    st.vectors.push(h);
    let i = st.vectors.len() - 1;
    st.vector_paths.insert(id, i);
    Ok((i, h))
}

/// `og.load.uiMaterial`: a cooked UI material, added to the scene once per path.
pub fn og_load_ui_material(world: u32, path: String) -> Result<serde_json::Value, String> {
    (|| -> Result<Value, String> {
        let eng = engine(world)?;
        Ok(json!(handle(load_material(&eng, &path)?)))
    })()
}

/// A UI material by its asset path, loaded once: its handle and defaults.
pub(crate) fn ui_material(
    eng: &crate::engine::Engine,
    path: &str,
) -> Result<
    (
        og_render::UiMaterialHandle,
        [[f32; 4]; og_render::UI_MATERIAL_PARAMS],
    ),
    String,
> {
    let i = load_material(eng, path)?;
    Ok(eng.render.borrow().ui_draw.materials[i])
}

fn load_material(eng: &crate::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&(i, _)) = eng.render.borrow().ui_draw.material_paths.get(&id) {
        return Ok(i);
    }
    let material = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .ui_material(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let hash = material_hash(eng, &id);
    let defaults = material.defaults();
    let mut rs = eng.render.borrow_mut();
    let h = rs.scene.add_ui_material(material);
    let st = &mut rs.ui_draw;
    st.materials.push((h, defaults));
    let i = st.materials.len() - 1;
    st.material_paths.insert(id, (i, hash));
    Ok(i)
}

fn material_hash(eng: &crate::engine::Engine, id: &str) -> u64 {
    eng.assets
        .borrow()
        .bundle_for(id)
        .ok()
        .and_then(|(b, _)| b.manifest().outputs().find(|o| o.id == id).map(|o| o.hash))
        .unwrap_or(0)
}

/// Refresh after the bundles' manifests, keeping handles used by live layers.
pub(crate) fn refresh(eng: &crate::engine::Engine) -> Vec<String> {
    let paths = eng.render.borrow().ui_draw.material_paths.clone();
    let mut failed = Vec::new();
    for (id, (i, before)) in paths {
        let hash = material_hash(eng, &id);
        if hash == before {
            continue;
        }
        let material = {
            let assets = eng.assets.borrow();
            assets.blob(&id).and_then(|(bytes, bundle)| {
                bundle
                    .ui_material(&id, bytes.bytes())
                    .map_err(|e| e.to_string())
            })
        };
        match material {
            Ok(material) => {
                let mut rs = eng.render.borrow_mut();
                let h = rs.ui_draw.materials[i].0;
                rs.ui_draw.materials[i].1 = material.defaults();
                rs.scene.ui.ui_materials[h.index()] = material;
                rs.ui_draw.material_paths.insert(id, (i, hash));
            }
            Err(e) => failed.push(format!("og dev: {id}: {e} (the old UI material stays)")),
        }
    }
    // A reload creates a new Ui even when the material bytes did not change.
    // Register every loaded handle and its current defaults with CSS again.
    if let Some(ui) = &eng.state.borrow().ui {
        let render = eng.render.borrow();
        let mut ui = ui.lock();
        for (id, (i, _)) in &render.ui_draw.material_paths {
            let (handle, defaults) = render.ui_draw.materials[*i];
            ui.define_material(id, handle, defaults);
        }
    }
    failed
}
