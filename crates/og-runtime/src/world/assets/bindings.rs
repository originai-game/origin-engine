//! Resolve authored content identities with the registered column's loader.
//! Cooked rows never contain process-local asset handles.
use super::*;
use crate::{bind, host::api, present};
use og_ecs::World;

pub(super) fn preload(eng: &Engine, records: &[(String, Map<String, Value>)]) -> Vec<String> {
    // Runtime bundles fetch on demand and acquire handles only at the held
    // spawn tick, never according to one host's file-arrival timing.
    fn boot(a: &Assets, value: &Value) -> bool {
        match value {
            Value::Object(fields) => {
                if let Some(id) = fields.get("$ogRef").and_then(Value::as_str) {
                    let mut pending = vec![id];
                    let mut seen = std::collections::BTreeSet::new();
                    while let Some(id) = pending.pop() {
                        if !seen.insert(id) {
                            continue;
                        }
                        let Ok(owner) = a.source(id, None) else {
                            return false;
                        };
                        if Some(owner) != a.default_bundle {
                            return false;
                        }
                        let Ok((bundle, _)) = a.bundle_from(id, Some(owner)) else {
                            return false;
                        };
                        let Ok(asset) = bundle.asset(id) else {
                            return false;
                        };
                        if asset.outputs.iter().any(|o| o.group.is_some()) {
                            return false;
                        }
                        pending.extend(asset.dependencies.iter().map(String::as_str));
                    }
                    true
                } else {
                    fields.values().all(|v| boot(a, v))
                }
            }
            Value::Array(values) => values.iter().all(|v| boot(a, v)),
            _ => true,
        }
    }
    let layouts = eng.state.borrow().layouts.clone();
    let mut failed = Vec::new();
    for (name, fields) in records {
        let fields = {
            let assets = eng.assets.borrow();
            fields
                .iter()
                .filter(|(_, v)| boot(&assets, v))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        };
        if let Some(layout) = layouts
            .get(name)
            .or_else(|| layouts.get(&format!("og.{name}")))
            && let Err(e) = eng
                .world()
                .and_then(|world| apply(eng, world, layout, &fields, &mut json!({})))
        {
            failed.push(e);
        }
    }
    failed
}

pub(crate) fn apply(
    eng: &Engine,
    world: &mut World,
    layout: &Layout,
    fields: &Map<String, Value>,
    value: &mut Value,
) -> Result<(), String> {
    for (name, reference) in fields {
        let (_, field) = layout
            .field(name)
            .ok_or_else(|| format!("{}.{}: no registered column", layout.schema, name))?;
        value[name] = load(eng, world, &field.column, reference)
            .map_err(|e| format!("{}.{}: {e}", layout.schema, name))?;
    }
    Ok(())
}

fn load(eng: &Engine, world: &mut World, column: &str, value: &Value) -> Result<Value, String> {
    if let Some(values) = value.as_array() {
        return values.iter().map(|v| load(eng, world, column, v)).collect();
    }
    if column == "ClipAsset" {
        let path = value["asset"]["$ogRef"]
            .as_str()
            .ok_or("clip needs an asset reference")?;
        let name = value["clip"]
            .as_str()
            .ok_or("clip needs an explicit name")?;
        let animation = bind::anim::load_animation(eng, path)?;
        return animation["clips"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|c| c["name"].as_str() == Some(name))
            .map(|c| c["clip"].clone())
            .ok_or_else(|| format!("{path}: no animation clip '{name}'"));
    }
    let path = value["$ogRef"]
        .as_str()
        .ok_or("expected a content reference")?
        .to_owned();
    let engine = eng.id;
    Ok(match column {
        "Texture" => api::og_load_texture(engine, path)?,
        "TextureArray" => api::og_load_texture_array(engine, path)?,
        "Sound" => api::og_load_sound(engine, path)?,
        "EnvironmentMap" => api::og_load_environment(engine, path)?,
        "Post" => present::views::og_load_post(engine, path)?,
        "EffectMaterial" => api::og_load_effect(engine, path)?,
        "ParticleEmitter" => api::og_load_emitter(engine, path)?,
        "FlipbookSheet" => present::feel::og_load_flipbook(engine, path)?["sheet"].clone(),
        "VfxAsset" => {
            let index = bind::vfx::load(eng, &path)?;
            crate::engine::lazy::ensure(eng, world, bind::vfx::COMPONENT)?;
            json!(bind::vfx::handle(index))
        }
        "TerrainData" => json!(bind::terrain::load(eng, &path)?.data),
        "Video" => json!(bind::video::og_load_video(engine, path)?.video),
        "TilemapAsset" => present::feel::og_load_tilemap(engine, path)?["map"].clone(),
        "TimelineAsset" => json!(bind::timeline::load_granted(eng, &path, Some(world))? as f64),
        "Rig2D" => bind::skeleton2d::og_load_rig2d(engine, path)?["rig"].clone(),
        "Rig" => bind::anim::og_load_rig(engine, path)?,
        "GraphAsset" => bind::anim::og_load_animgraph(engine, path)?,
        "SkeletonAsset" => bind::anim::load_animation(eng, &path)?["skeleton"].clone(),
        "Behaviour" => json!(bind::ai::load(engine, &path, Some(world))?.behaviour),
        "Palette" => api::og_load_palette(engine, path)?,
        "Lut" => api::og_load_lut(engine, path)?,
        _ => return Err(format!("{column} is not a loadable content column")),
    })
}
