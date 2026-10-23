//! og.load.rig2d, og.anim.rig2dInfo and the drawing of og.Skeleton2D.
//!
//! og-anim's `og.skeleton2d` (installed with the feel modules, after the
//! game's systems) plays the clips and solves the IK targets; the rigs live
//! per engine in [`super::feel::FeelAssets`] under handles minted there, and
//! a hot reload registers them again. Extraction poses each skeleton from
//! the same inputs the system read (its rig, clip, time and flipX, its
//! og.Transform, its og.IkTarget2D rows) and draws the triangles as one
//! og-render sprite mesh in the sprite order at its layer, so og.Lighting2D
//! lights it as it lights a sprite there.

use super::engine::{Engine, engine};
use super::extract::{Hierarchy, RenderState};
use super::feel::{feel_of, set_filter};
use super::{ok, refuse};
use glam::{Vec2, Vec3, Vec4};
use og_anim::{IkTarget2D, Rig2dAsset, Skeleton2D, Transform};
use og_ecs::{Entity, QueryDesc, World};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

/// What og.load.rig2d and og.anim.rig2dInfo answer for a rig.
fn info(handle: u64, rig: &Rig2dAsset, texture: usize) -> Value {
    let names = |v: Vec<&String>| v.into_iter().cloned().collect::<Vec<_>>();
    json!({
        "rig": handle as f64,
        "texture": super::assets::handle(texture) as f64,
        "unit": rig.unit,
        "bones": names(rig.bones.iter().map(|b| &b.name).collect()),
        "slots": names(rig.slots.iter().map(|s| &s.name).collect()),
        "constraints": names(rig.iks.iter().map(|k| &k.name).collect()),
        "clips": rig.clips.iter().enumerate().map(|(i, c)| json!({
            "clip": i,
            "name": c.name,
            "duration": c.duration,
            "loop": c.looping,
            "events": c.events.iter().map(|e| e.1.clone()).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

fn load_rig2d(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    let key = format!("rig2d:{id}");
    if let Some(v) = eng.state.borrow().feel_assets.loaded.get(&key) {
        return Ok(v.clone());
    }
    let rig = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle.rig2d(&id, bytes.bytes()).map_err(|e| e.to_string())?
    };
    let texture = super::assets::load_texture(eng, &rig.texture)
        .map_err(|e| format!("{id}: texture {}: {e}", rig.texture))?;
    set_filter(eng, texture, rig.filter == og_anim::SheetFilter::Nearest);
    let feel = feel_of(eng)?;
    let mut st = eng.state.borrow_mut();
    let handle = st.feel_assets.mint();
    feel.skeletons
        .add_rig(handle, rig.clone())
        .map_err(|e| format!("{id}: {}", e.reason()))?;
    feel.rig_textures
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(handle, texture);
    let answer = info(handle, &rig, texture);
    st.feel_assets.rigs.insert(handle, (rig, texture));
    st.feel_assets.loaded.insert(key, answer.clone());
    Ok(answer)
}

pub fn og_load_rig2d(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load_rig2d(&eng, &path)))
}

pub fn og_anim_rig2d_info(world: u32, rig: f64) -> String {
    wire(engine(world).and_then(|eng| {
        let st = eng.state.borrow();
        let h = rig as u64;
        match st.feel_assets.rigs.get(&h) {
            Some((r, t)) if rig >= 0.0 && rig.fract() == 0.0 => Ok(info(h, r, *t)),
            _ => Err(format!(
                "{rig} is not a rig og.load.rig2d answered on this world"
            )),
        }
    }))
}

fn entities_with<T: og_core::Reflect>(world: &World) -> Vec<Entity> {
    let Ok(id) = world.id_by_name(T::NAME) else {
        return Vec::new();
    };
    let Ok(schema) = world.schema(id) else {
        return Vec::new();
    };
    let q = QueryDesc {
        read: vec![schema.name.clone()],
        ..QueryDesc::default()
    };
    match world.query(&q) {
        Ok(all) => all.flat_map(|a| a.entities().to_vec()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Each og.Skeleton2D with a loaded rig, as one sprite mesh at its layer.
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let Some((feel, _)) = rs.feel.feel.clone() else {
        return;
    };
    let skeletons = entities_with::<Skeleton2D>(world);
    if skeletons.is_empty() {
        return;
    }
    let mut targets: BTreeMap<u64, Vec<(u32, Vec2, f32)>> = BTreeMap::new();
    for e in entities_with::<IkTarget2D>(world) {
        if let Ok(t) = world.get::<IkTarget2D>(e) {
            targets
                .entry(t.skeleton)
                .or_default()
                .push((t.constraint, t.target, t.weight));
        }
    }
    let textures = feel
        .rig_textures
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    for e in skeletons {
        let Ok(s) = world.get::<Skeleton2D>(e) else {
            continue;
        };
        let Some(rig) = feel.skeletons.rig(s.rig) else {
            continue;
        };
        let bits = e.to_bits();
        let place = world.get::<Transform>(e).unwrap_or_default();
        let frame = og_anim::skeleton_frame(&rig, &place, s.flip_x);
        let goals = targets.get(&bits).map_or(&[][..], |v| v.as_slice());
        let pose = og_anim::skeleton_pose(&rig, &s, &frame, goals);
        let (corners, indices) = rig.triangles(&pose);
        if indices.is_empty() {
            continue;
        }
        // triangles() answers rig units (pixels over `unit`): the frame
        // without its 1 / unit, then the parent's world matrix.
        let to_parent = frame.mul(&og_anim::Affine2::trs(0.0, 0.0, 0.0, rig.unit, rig.unit));
        let parent = h.parent_world(bits);
        let z = place.position.z;
        let vertices = corners
            .iter()
            .map(|c| {
                let p = to_parent.apply_to(c.position);
                let w = parent * Vec4::new(p[0], p[1], z, 1.0);
                og_render::SpriteMeshVertex {
                    position: Vec3::new(w.x, w.y, w.z),
                    uv: Vec2::from(c.uv),
                    color: c.color,
                }
            })
            .collect();
        rs.scene.sprite_meshes.push(og_render::SpriteMesh {
            texture: textures
                .get(&s.rig)
                .and_then(|&i| rs.textures.get(i).copied()),
            vertices,
            indices,
            layer: s.layer,
            key: 0.0,
            blend: og_render::SpriteBlend::Alpha,
        });
    }
}
