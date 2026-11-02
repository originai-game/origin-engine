//! Material tracks: presentation, not world state. Every frame, before the
//! scene is extracted, each active timeline's material tracks (and those of
//! the timelines it nests) are evaluated at its time and written into the
//! materials they drive; a material no track drives any more is put back as
//! it was. Nothing is accumulated or saved, so a snapshot restored mid-way
//! draws the restored time's material on its next frame, and seeking draws
//! the sought time's.

use super::*;
use og_timeline::{FieldValue, FieldsKind, FieldsTrack};

/// The StandardMaterialDesc fields a material track sets by name (others
/// are the material shader's parameters).
const STANDARD: &[&str] = &[
    "color",
    "metalness",
    "roughness",
    "emissive",
    "emissiveIntensity",
    "opacity",
    "transparent",
    "alphaTest",
    "envMapIntensity",
    "normalScale",
    "aoMapIntensity",
    "rimColor",
    "rimIntensity",
    "rimPower",
    "flashColor",
    "flash",
    "dissolve",
    "dissolveEdgeColor",
    "dissolveEdgeWidth",
    "dissolveScale",
    "outlineColor",
    "outlineWidth",
    "clearcoat",
    "clearcoatRoughness",
    "sheen",
    "sheenColor",
    "sheenRoughness",
    "transmission",
    "thickness",
    "attenuationDistance",
    "attenuationColor",
    "ior",
    "iridescence",
    "anisotropy",
    "anisotropyRotation",
    "mapOffset",
    "mapRepeat",
    "mapRotation",
];

/// What one material is driven to this frame.
type Driven = BTreeMap<u32, (og_render::MaterialHandle, Vec<(String, FieldValue)>)>;

/// Applies every active timeline's material tracks to the scene.
pub(crate) fn present(world: &World, rs: &mut super::super::extract::RenderState) {
    let engine_id = rs.draw.engine;
    let runs: Vec<(u64, i64)> = lock(&shared(engine_id))
        .runs
        .values()
        .filter(|r| r.active)
        .map(|r| (r.asset, r.num))
        .collect();
    let reg = registry(engine_id);
    if runs.is_empty() && lock(&reg).material_base.is_empty() {
        return;
    }
    let dt = world.time().dt;
    let rate = if dt > 0.0 {
        (1.0 / f64::from(dt)).round().max(1.0) as i64
    } else {
        60
    };
    let den = (rate * STEPS) as f64;
    let mut driven = Driven::new();
    for (asset, num) in runs {
        let Some(t) = lock(&reg).timeline(asset) else {
            continue;
        };
        let lt = lock(&t);
        collect(
            world,
            rs,
            engine_id,
            &lt,
            (num as f64 / den) as f32,
            0,
            &mut driven,
        );
    }
    let mut r = lock(&reg);
    // Materials no track drives any more: back as they were.
    let released: Vec<u32> = r
        .material_base
        .keys()
        .filter(|k| !driven.contains_key(k))
        .copied()
        .collect();
    for k in released {
        if let Some(b) = r.material_base.remove(&k) {
            rs.scene.set_standard_material(b.handle, b.standard);
            for (name, v) in &b.params {
                rs.scene.set_shader_param(b.handle, name, v);
            }
        }
    }
    for (k, (h, values)) in driven {
        let Some(current) = rs.scene.standard_material(h).copied() else {
            continue;
        };
        let base = r.material_base.entry(k).or_insert_with(|| MaterialBase {
            handle: h,
            standard: current,
            params: Vec::new(),
        });
        let mut desc = serde_json::Map::new();
        let mut params = Vec::new();
        for (name, v) in values {
            if STANDARD.contains(&name.as_str()) {
                let j = match &v {
                    FieldValue::Word(w) => json!(w == "true"),
                    FieldValue::Numbers(n) => match n.as_slice() {
                        [x] => json!(x),
                        [x, y] => json!({ "x": x, "y": y }),
                        [r, g, b, ..] => json!({ "r": r, "g": g, "b": b }),
                        [] => continue,
                    },
                };
                desc.insert(name, j);
            } else if let FieldValue::Numbers(n) = v {
                params.push((name, n));
            }
        }
        // From the material as it was, so a field no key sets stays its own.
        let next = super::super::api::material(
            base.standard,
            &Value::Object(desc),
            &rs.textures,
            &rs.texture_arrays,
        )
        .unwrap_or(base.standard);
        if next != current {
            rs.scene.set_standard_material(h, next);
        }
        for (name, n) in params {
            if !base.params.iter().any(|(p, _)| *p == name)
                && let Some((shader, values)) = rs.scene.material_shader(h)
                && let Some((slot, _)) = rs.scene.shaders()[shader.index()].param(&name)
            {
                base.params.push((name.clone(), values[slot]));
            }
            rs.scene.set_shader_param(h, &name, &n);
        }
    }
}

/// The material a track drives: its entity's og.Mesh material, else its
/// og.Model's material `index`.
fn material_of(
    world: &World,
    rs: &super::super::extract::RenderState,
    engine_id: u32,
    f: &FieldsTrack,
) -> Option<(u32, og_render::MaterialHandle)> {
    let e = world.find(&f.entity)?;
    let handle = if let Some(mesh) = rs.draw.mesh.as_ref()
        && let Ok(id) = world.id_by_name("og.Mesh")
        && let Ok(b) = world.get_bytes(e, id)
    {
        mesh.decode(&b)["material"].as_f64()?
    } else {
        let model = rs.draw.model.as_ref()?;
        let id = world.id_by_name("og.Model").ok()?;
        let b = world.get_bytes(e, id).ok()?;
        let asset = model.decode(&b)["asset"].as_f64()?;
        let eng = engine(engine_id).ok()?;
        let a = eng.assets.try_borrow().ok()?;
        let entry = assets::index_of(asset).and_then(|i| a.entries.get(i))?;
        match &entry.payload {
            assets::Payload::Gltf { materials, .. } => *materials.get(f.index as usize)? as f64,
            _ => return None,
        }
    };
    let i = assets::index_of(handle)?;
    rs.materials.get(i).map(|h| (i as u32, *h))
}

fn collect(
    world: &World,
    rs: &super::super::extract::RenderState,
    engine_id: u32,
    lt: &Loaded,
    t: f32,
    depth: usize,
    out: &mut Driven,
) {
    for tr in &lt.timeline.tracks {
        match tr {
            Track::Fields(f) if f.kind == FieldsKind::Material => {
                let Some((k, h)) = material_of(world, rs, engine_id, f) else {
                    note(
                        engine_id,
                        format!(
                            "timeline {}: '{}' draws no og.Mesh or og.Model material {} for its material track",
                            lt.id, f.entity, f.index
                        ),
                    );
                    continue;
                };
                let values = &mut out.entry(k).or_insert_with(|| (h, Vec::new())).1;
                for name in f.fields() {
                    if let Some(v) =
                        og_timeline::eval::field_at(f, name, t, og_timeline::eval::Mix::Lerp)
                    {
                        values.retain(|(n, _)| n != name);
                        values.push((name.to_string(), v));
                    }
                }
            }
            Track::Timeline(keys) if depth < og_timeline::NEST_DEPTH => {
                let Some((ki, _, _)) =
                    og_timeline::eval::nested_at(keys, t, lt.timeline.duration, |_| 0.0)
                else {
                    continue;
                };
                let k = &keys[ki];
                let Some(child) = lt.nested.get(&k.play) else {
                    continue;
                };
                let cl = lock(child);
                let ct = og_timeline::eval::child_time(k, t, cl.timeline.duration);
                collect(world, rs, engine_id, &cl, ct, depth + 1, out);
            }
            _ => {}
        }
    }
}
