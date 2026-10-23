//! Baked lighting (depth wave S7): the lightmaps and light volumes
//! `og bake lightmaps` wrote for a scene, applied to the scene's static
//! models and to everything inside its light volumes; and og.RectLight.
//!
//! When a scene is instantiated (`assets::spawn_scene`) its lightmap file
//! (`<scene>.lightmap.ogt`, cooked to an `og.Lightmap` blob) is read, if the
//! bundle has one, and checked against the cooked scene and models it was
//! baked from (their hashes). A matching bake gives each static model
//! entity its lightmapped meshes (the model's parts re-indexed by the
//! lightmap's vertices, each with its lightmap uvs) and hands the pages and
//! the volumes to the renderer; a missing or stale one is reported by
//! `og.render.bakedLighting` with its reason, and the scene draws unbaked.
//! Every instantiated scene's bake is drawn at once, each static model with
//! its own scene's pages (og-render binds them per batch); a bake is let go
//! when its static models are all despawned.

use super::extract::Parts;
use super::fx::{color, f, on};
use glam::{Mat4, Vec3};
use og_cook::runtime::SceneDesc;
use og_render::{BakedLighting, BakedMesh, LightmapPages, LightmapUvs, MeshData, ProbeVolume};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

/// One instantiated scene's bake, as reported.
#[derive(Clone, Debug, Default)]
pub(crate) struct SceneBake {
    pub scene: String,
    /// Empty when drawn.
    pub reason: String,
    pub models: u32,
    pub pages: u32,
    pub texels: u32,
    pub volumes: u32,
    pub probes: u32,
    /// What the scene's bake draws (`None`: it draws unbaked).
    pub drawn: Option<DrawnBake>,
}

/// A drawn bake: the static model entities it lightmaps, their meshes with
/// the scene's pages, and the scene's light volumes.
#[derive(Clone, Debug, Default)]
pub(crate) struct DrawnBake {
    pub entities: Vec<u64>,
    pub meshes: std::collections::HashMap<usize, BakedMesh>,
    pub volumes: Vec<ProbeVolume>,
    /// Whether one of its static models is alive (a bake without static
    /// models, only light volumes, always is).
    pub live: bool,
}

/// The baked lighting of the scenes instantiated so far.
#[derive(Default)]
pub(crate) struct BakedState {
    pub scenes: Vec<SceneBake>,
    /// Static model entities drawn with their lightmap: their parts.
    pub overrides: BTreeMap<u64, Parts>,
    /// What the renderer draws: every drawn bake's meshes and volumes.
    pub lighting: BakedLighting,
    /// Whether RenderSettings.bakedLighting was on in the last frame.
    pub enabled: bool,
}

impl BakedState {
    /// The renderer's [`BakedLighting`] again from the live drawn bakes, in
    /// the order their scenes were instantiated (their volumes in that
    /// order).
    fn gather(&mut self) {
        let mut lighting = BakedLighting::default();
        for d in self
            .scenes
            .iter()
            .filter_map(|s| s.drawn.as_ref())
            .filter(|d| d.live)
        {
            lighting
                .meshes
                .extend(d.meshes.iter().map(|(&k, v)| (k, v.clone())));
            lighting.volumes.extend(d.volumes.iter().cloned());
        }
        self.lighting = lighting;
    }
}

/// og.RectLight at world transform `m`: width and height along its local
/// x and y, emitting along its local -Z; its range where it would light a
/// white surface facing it at one hundredth of one.
pub(crate) fn rect_light(m: Mat4, v: &Value) -> og_render::Light {
    let half_width = m.transform_vector3(Vec3::X) * f(v, "width", 10.0).max(0.0) * 0.5;
    let half_height = m.transform_vector3(Vec3::Y) * f(v, "height", 10.0).max(0.0) * 0.5;
    let c = color(&v["color"], Vec3::ONE);
    let intensity = f(v, "intensity", 1.0).max(0.0);
    let area = 4.0 * half_width.cross(half_height).length();
    let reach = (c.max_element().max(0.0) * intensity * area / 0.01).sqrt();
    og_render::Light {
        kind: og_render::LightKind::Rect {
            half_width,
            half_height,
            two_sided: on(v, "twoSided", false),
        },
        position: m.transform_point3(Vec3::ZERO),
        color: c,
        intensity,
        range: reach
            .max((half_width + half_height).length() * 2.0)
            .max(0.1),
        cast_shadows: on(v, "castShadow", false),
    }
}

/// Whether a scene has something to bake: a Model row with `static` set,
/// or a LightVolume.
fn bakeable(scene: &SceneDesc) -> bool {
    scene.components.iter().any(|t| {
        let name = t.name.trim_start_matches("og.");
        if name == "LightVolume" {
            return !t.rows.is_empty();
        }
        if name != "Model" {
            return false;
        }
        let Some(i) = t.schema.fields.iter().position(|f| f.name == "static") else {
            return false;
        };
        let range = og_core::reflect::Layout::of(&t.schema).range(i);
        t.rows.iter().any(|(_, row)| {
            row.get(range.clone())
                .is_some_and(|b| b.iter().any(|&x| x != 0))
        })
    })
}

/// Reads the bake of `scene` just spawned as `spawned` (one entity per
/// scene entity, in order).
pub(crate) fn scene_spawned(
    eng: &super::engine::Engine,
    scene: &SceneDesc,
    spawned: &[og_ecs::Entity],
) {
    let lightmap_id = og_cook::lightmap::lightmap_id(&scene.id);
    let mut report = SceneBake {
        scene: scene.id.clone(),
        ..Default::default()
    };
    let loaded = (|| -> Result<Option<og_cook::runtime::CookedLightmap>, String> {
        let assets = eng.assets.borrow();
        let Ok((bundle, _)) = assets.bundle_for(&lightmap_id) else {
            return Ok(None);
        };
        if bundle.output(&lightmap_id).is_err() {
            return Ok(None);
        }
        let (bytes, bundle) = assets.blob(&lightmap_id)?;
        let lm = bundle
            .lightmap(&lightmap_id, bytes.bytes())
            .map_err(|e| e.to_string())?;
        let scene_hash = bundle.output(&scene.id).map(|o| o.hash).unwrap_or(0);
        if lm.file.scene_hash != scene_hash {
            return Err(format!(
                "its bake is stale: {} changed since og bake lightmaps (run it again)",
                scene.id
            ));
        }
        for m in &lm.file.models {
            let hash = bundle.output(&m.asset).map(|o| o.hash).unwrap_or(0);
            if hash != m.hash {
                return Err(format!(
                    "its bake is stale: the static model {} changed since og bake lightmaps (run it again)",
                    m.asset
                ));
            }
        }
        Ok(Some(lm))
    })();
    let lm = match loaded {
        Ok(Some(lm)) => lm,
        Ok(None) => {
            if bakeable(scene) {
                report.reason =
                    format!("the scene has no bake (og bake lightmaps writes {lightmap_id})");
                eng.render.borrow_mut().baked.scenes.push(report);
            }
            return;
        }
        Err(reason) => {
            report.reason = reason;
            eng.render.borrow_mut().baked.scenes.push(report);
            return;
        }
    };
    let by_name: BTreeMap<&str, og_ecs::Entity> = scene
        .entities
        .iter()
        .zip(spawned)
        .filter(|(e, _)| !e.name.is_empty())
        .map(|(e, s)| (e.name.as_str(), *s))
        .collect();
    // The models' parts, loaded by the spawn.
    let mut models: Vec<(og_ecs::Entity, Parts, &og_cook::lightmap::BakedModel)> = Vec::new();
    for m in &lm.file.models {
        let Some(&entity) = by_name.get(m.entity.as_str()) else {
            report.reason = format!(
                "its bake is stale: the static model entity '{}' is no longer in the scene",
                m.entity
            );
            eng.render.borrow_mut().baked.scenes.push(report);
            return;
        };
        let index = match super::assets::load_gltf(eng, &m.asset) {
            Ok(i) => i,
            Err(e) => {
                report.reason = e;
                eng.render.borrow_mut().baked.scenes.push(report);
                return;
            }
        };
        let rs = eng.render.borrow();
        let Some(parts) = rs.models.get(&super::assets::handle(index)) else {
            continue;
        };
        models.push((entity, parts.clone(), m));
    }
    let mut rs = eng.render.borrow_mut();
    let mut overrides = Vec::new();
    let mut meshes = std::collections::HashMap::new();
    for (entity, parts, m) in &models {
        if parts.len() != m.parts.len() {
            report.reason = format!(
                "its bake is stale: {} has {} parts, the bake {}",
                m.asset,
                parts.len(),
                m.parts.len()
            );
            rs.baked.scenes.push(report);
            return;
        }
        let mut baked_parts = Vec::with_capacity(parts.len());
        for (&(mesh, material, local), bp) in parts.iter().zip(&m.parts) {
            let Some(data) = rs.scene.mesh(mesh) else {
                continue;
            };
            if data.vertices.len() != bp.vertices as usize
                || bp.xref.iter().any(|&x| x as usize >= data.vertices.len())
            {
                report.reason = format!(
                    "its bake is stale: {}'s vertices changed since the bake",
                    m.asset
                );
                rs.baked.scenes.push(report);
                return;
            }
            let lit = MeshData {
                vertices: bp.xref.iter().map(|&x| data.vertices[x as usize]).collect(),
                indices: bp.indices.clone(),
            };
            let handle = rs.scene.add_mesh(lit);
            meshes.insert(handle.index(), Arc::new(LightmapUvs { uvs: bp.uv.clone() }));
            baked_parts.push((handle, material, local));
        }
        overrides.push((entity.to_bits(), baked_parts));
    }
    let to_f32 = og_cook::lightmap::f16_to_f32;
    let pages = Arc::new(LightmapPages {
        width: lm.width,
        height: lm.height,
        pages: lm
            .pages
            .iter()
            .map(|p| {
                p.as_chunks::<4>()
                    .0
                    .iter()
                    .map(|t| [to_f32(t[0]), to_f32(t[1]), to_f32(t[2]), to_f32(t[3])])
                    .collect()
            })
            .collect(),
    });
    let volumes: Vec<ProbeVolume> = lm
        .file
        .volumes
        .iter()
        .map(|v| ProbeVolume {
            min: Vec3::from(v.min),
            max: Vec3::from(v.max),
            counts: v.counts,
            probes: Arc::new(v.probes.clone()),
        })
        .collect();
    report.models = overrides.len() as u32;
    report.pages = lm.pages.len() as u32;
    report.texels = lm.width.max(lm.height);
    report.volumes = volumes.len() as u32;
    report.probes = volumes.iter().map(|v| v.probes.len() as u32).sum();
    report.drawn = Some(DrawnBake {
        entities: overrides.iter().map(|(e, _)| *e).collect(),
        meshes: meshes
            .into_iter()
            .map(|(k, uvs)| {
                (
                    k,
                    BakedMesh {
                        pages: pages.clone(),
                        uvs,
                    },
                )
            })
            .collect(),
        volumes,
        live: true,
    });
    let state = &mut rs.baked;
    state.overrides.extend(overrides);
    state.scenes.push(report);
    state.gather();
}

/// The frame's baked lighting (extract): what the renderer draws, and the
/// static models' parts when RenderSettings.bakedLighting is on. A bake
/// whose static models are all despawned stops drawing (the renderer lets
/// its pages go) and draws again if they come back (a world restore).
pub(crate) fn apply(
    world: &og_ecs::World,
    scene: &mut og_render::RenderScene,
    state: &mut BakedState,
    enabled: bool,
) {
    let mut changed = false;
    for d in state.scenes.iter_mut().filter_map(|s| s.drawn.as_mut()) {
        let live = d.entities.is_empty()
            || d.entities
                .iter()
                .any(|&e| world.is_alive(og_ecs::Entity::from_bits(e)));
        changed |= live != d.live;
        d.live = live;
    }
    if changed {
        state.gather();
    }
    state.enabled = enabled;
    scene.baked = if enabled {
        state.lighting.clone()
    } else {
        BakedLighting::default()
    };
}

/// `og.render.bakedLighting`.
pub fn og_render_baked_lighting(world: u32) -> String {
    let Ok(eng) = super::engine::engine(world) else {
        return "[]".to_string();
    };
    let rs = eng.render.borrow();
    let list: Vec<Value> = rs
        .baked
        .scenes
        .iter()
        .map(|s| {
            let reason = if !s.reason.is_empty() {
                s.reason.clone()
            } else if s.drawn.as_ref().is_some_and(|d| !d.live) {
                "its static models were despawned".to_string()
            } else if !rs.baked.enabled {
                "RenderSettings.bakedLighting is off".to_string()
            } else {
                String::new()
            };
            json!({
                "scene": s.scene,
                "baked": reason.is_empty(),
                "reason": reason,
                "models": s.models,
                "pages": s.pages,
                "texels": s.texels,
                "volumes": s.volumes,
                "probes": s.probes,
            })
        })
        .collect();
    Value::Array(list).to_string()
}
