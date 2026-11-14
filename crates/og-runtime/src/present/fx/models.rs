//! Posed and crowd models: their skinned meshes, skeletons and baked clips.

use super::*;

/// Makes the skinned copies of a model's primitives the first time it is
/// drawn posed (both og.load.gltf and og.load.animation of it are needed).
fn skinned_meshes(
    scene: &mut og_render::RenderScene,
    info: &ModelInfo,
    skin: &mut SkinnedModel,
    prims: usize,
) -> Vec<Option<MeshHandle>> {
    if let Some(m) = &skin.meshes {
        return m.clone();
    }
    let mut out = vec![None; prims];
    let mut levels = vec![Vec::new(); prims];
    for (p, slot) in out.iter_mut().enumerate() {
        let stream = skin.streams.iter().find(|s| s.primitive == p);
        let morph = skin.morphs.iter().find(|m| m.primitive == p);
        if stream.is_none() && morph.is_none() {
            continue;
        }
        // Its coarser levels' skinned copies: each vertex skinned as the
        // full mesh's vertex it is (no blendshapes: only crowds draw them).
        for (level, sources) in info.lod_sources.get(p).into_iter().flatten() {
            let Some(mesh) = scene.mesh(*level).cloned() else {
                break;
            };
            let full = info.meshes.get(p).and_then(|h| scene.mesh(*h));
            let data = match stream {
                Some(s)
                    if sources.len() == mesh.vertices.len()
                        && full.is_some_and(|f| f.vertices.len() == s.joints.len()) =>
                {
                    SkinData {
                        joints: sources.iter().map(|&v| s.joints[v as usize]).collect(),
                        weights: sources.iter().map(|&v| s.weights[v as usize]).collect(),
                    }
                }
                Some(_) => break,
                None => SkinData {
                    joints: vec![[0; 4]; mesh.vertices.len()],
                    weights: vec![[1.0, 0.0, 0.0, 0.0]; mesh.vertices.len()],
                },
            };
            levels[p].push(scene.add_skinned_mesh(mesh, data));
        }
        let Some(mesh) = info.meshes.get(p).and_then(|h| scene.mesh(*h)).cloned() else {
            continue;
        };
        let n = mesh.vertices.len();
        let data = match stream {
            Some(s) if s.joints.len() == n => SkinData {
                joints: s.joints.clone(),
                weights: s.weights.clone(),
            },
            // Morph targets alone: every vertex on joint 0 with full weight
            // (an identity palette keeps the bind pose).
            _ => SkinData {
                joints: vec![[0; 4]; n],
                weights: vec![[1.0, 0.0, 0.0, 0.0]; n],
            },
        };
        let h = scene.add_skinned_mesh(mesh, data);
        if let Some(m) = morph {
            let take = m.positions.len().min(og_render::MAX_MORPH_TARGETS);
            scene.set_morph_targets(
                h,
                MorphTargets {
                    positions: m.positions[..take].to_vec(),
                    normals: m.normals.as_ref().map(|n| n[..take].to_vec()),
                },
            );
        }
        *slot = Some(h);
    }
    skin.meshes = Some(out.clone());
    skin.levels = levels;
    out
}

/// The entities with og.Skeleton (to draw their models posed).
pub(crate) fn skeletons(
    world: &World,
    layout: Option<&Layout>,
    fx: &mut FxState,
) -> std::collections::BTreeSet<u64> {
    fx.pose_issues.clear();
    fx.pose_sources = rows(world, layout)
        .into_iter()
        .filter_map(|(e, v)| Some((e, record::Skeleton::from_json(&v).ok()?.asset)))
        .collect();
    fx.pose_sources.keys().copied().collect()
}

/// One og.Model's own look and shadow flags, carried by each of its
/// instances: its tint and flash, `castShadow` and `receiveShadow`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ModelLook {
    pub fx: og_render::InstanceFx,
    pub cast: bool,
    pub receive: bool,
}

/// Draws a model posed by og-anim: its skinned primitives as skinned
/// instances under the entity's palette (and blendshape weights), the rest
/// as instances, each with the model's own tint, flash and shadow flags
/// (`look`). A part whose node the pose hides (a visibility channel, or
/// a hidden ancestor: `AnimOutput::drawn`) is not drawn at all: a `.vox`
/// frame folded away costs no draw. `false`
/// when it cannot (no palette yet, no skin data): the caller draws it at
/// its bind pose. On a frame between ticks the pose is blended with the
/// tick before's at `alpha` (`crate::present::interpolate::pose`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn posed_model(
    rs: &mut RenderState,
    entity: u64,
    asset: u64,
    world_matrix: Mat4,
    alpha: Option<f32>,
    parts: &[(MeshHandle, og_render::MaterialHandle, Mat4)],
    look: ModelLook,
    keys: &mut Vec<(u64, u32)>,
) -> bool {
    let physics = rs.draw.physics.clone();
    let RenderState {
        scene,
        fx,
        atmo,
        interp,
        motion,
        notes,
        ..
    } = rs;
    let Some(anim) = fx.anim.as_ref() else {
        return false;
    };
    let Some(mut out) = anim.output(Entity::from_bits(entity)) else {
        return false;
    };
    // og.motion's generated pose, when it has one: between ticks, blended
    // from the tick before's like og-anim's.
    let matrices = match motion.drawn(entity, alpha) {
        Some(m) if m.len() == out.palette.len() => m,
        _ => {
            crate::present::interpolate::pose(interp, entity, alpha, &out.palette, &mut out.weights)
        }
    };
    let Some(info) = fx.models.get(&asset).cloned() else {
        return false;
    };
    let source = fx.pose_sources.get(&entity).and_then(|&h| anim.skeleton(h));
    let reason = match fx.skins.get(&info.id) {
        None => Some("model skin data is not loaded; load its animation asset".to_string()),
        Some(model) if !model.streams.is_empty() => {
            match model.skeleton.skins.get(out.skin as usize) {
                None => Some(format!("model has no skin {}", out.skin)),
                Some(expected) if expected.joints.len() != matrices.len() => Some(format!(
                    "model skin {} needs {} joints but the palette has {}",
                    out.skin, expected.joints.len(), matrices.len(),
                )),
                Some(expected) => source.as_ref().and_then(|source| {
                    let actual = source.skins.get(out.skin as usize)?;
                    let different_order = expected.joints.iter().zip(&actual.joints).any(|(&a, &b)| {
                        model.skeleton.nodes[a as usize].name != source.nodes[b as usize].name
                    });
                    (expected.inverse_bind != actual.inverse_bind
                        || different_order)
                        .then(|| format!("model skin {} and skeleton have different joint order or inverse bind matrices", out.skin))
                }),
            }
        }
        Some(_) => None,
    };
    if let Some(reason) = reason {
        notes.push(format!("og.Model {} on entity {entity}: {reason}", info.id));
        fx.pose_issues.insert(
            entity,
            serde_json::json!({
                "model": info.id, "skin": out.skin, "paletteJoints": matrices.len(),
                "reason": reason,
            }),
        );
    }
    let Some(skin) = fx.skins.get_mut(&info.id) else {
        return false;
    };
    let meshes = skinned_meshes(scene, &info, skin, info.prim_mesh.len());
    let palette = scene.objects.joint_palettes.len() as u32;
    atmo.palettes.insert(entity, palette);
    // og.Garment: its primitives drawn at the simulated points, posed by
    // this frame's palette (`crate::bind::garment::deformed`).
    let mut garment = crate::bind::garment::deformed(
        fx,
        scene,
        physics.as_ref(),
        entity,
        asset,
        world_matrix,
        &matrices,
    );
    scene.objects.joint_palettes.push(JointPalette { matrices });
    // Parts and nodes line up unless the parts are not the model's own
    // (a bake's lightmapped parts): then every part is drawn.
    let nodes = (info.part_nodes.len() == parts.len()).then_some(&info.part_nodes);
    for (k, &(mesh, material, local)) in parts.iter().enumerate() {
        if nodes.is_some_and(|n| out.drawn.get(n[k]) == Some(&false)) {
            continue;
        }
        let p = info.primitive(mesh);
        match meshes.get(p).copied().flatten() {
            Some(skinned) => {
                let mut inst = SkinnedInstance::new(skinned, material, world_matrix, palette);
                inst.fx = look.fx;
                inst.cast_shadows = look.cast;
                inst.receive_shadows = look.receive;
                if let Some((zero, vertices)) = garment.remove(&p) {
                    inst.mesh = zero;
                    inst.deformed = Some(scene.objects.deformed_vertices.len() as u32);
                    scene.objects.deformed_vertices.push(vertices);
                    fx.deformed_keys.push(((entity, p), world_matrix));
                } else if info.prim_mesh.get(p).copied() == Some(out.mesh) {
                    let n = inst.morph_weights.len().min(out.weights.len());
                    inst.morph_weights[..n].copy_from_slice(&out.weights[..n]);
                }
                scene.objects.skinned_instances.push(inst);
            }
            None => {
                scene.objects.instances.push(og_render::Instance {
                    mesh,
                    material,
                    transform: world_matrix * local,
                    material_layer: None,
                    fx: look.fx,
                    cast_shadows: look.cast,
                    receive_shadows: look.receive,
                    sway: 0.0,
                });
                keys.push((entity, k as u32));
            }
        }
    }
    true
}

/// A baked clip of og.CrowdAnimation: its index in
/// `RenderScene::baked_clips`, its frames and the clip's seconds.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Baked {
    pub index: u32,
    pub frames: usize,
    pub seconds: f32,
}

/// One og.CrowdAnimation's fields.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CrowdPlay {
    pub skeleton: u64,
    pub skin: u32,
    pub clip: u64,
    pub offset: f32,
    pub speed: f32,
    pub looping: bool,
    pub rate: f32,
}

/// The entities with og.CrowdAnimation and its fields, read in place (a
/// crowd is thousands of rows: no JSON a row).
pub(crate) fn crowds(world: &World, rs: &RenderState) -> IdMap<u64, CrowdPlay> {
    use crate::host::codec::{Rec, Slot};
    let Some(layout) = rs.draw.fx.crowd.as_ref() else {
        return IdMap::new();
    };
    let slot = |key| Slot::of(layout, key);
    let (skeleton, skin, clip) = (slot("skeleton"), slot("skin"), slot("clip"));
    let (offset, speed, looping, rate) =
        (slot("offset"), slot("speed"), slot("loop"), slot("rate"));
    crate::present::extract::column_rows(world, layout)
        .iter()
        .map(|(e, v)| {
            let num = |s: &Slot, d: f64| v.num(s).unwrap_or(d);
            (
                e,
                CrowdPlay {
                    skeleton: num(&skeleton, 0.0) as u64,
                    skin: num(&skin, 0.0) as u32,
                    clip: num(&clip, 0.0) as u64,
                    offset: num(&offset, 0.0) as f32,
                    speed: num(&speed, 1.0) as f32,
                    looping: v.flag(&looping).unwrap_or(true),
                    rate: num(&rate, 30.0) as f32,
                },
            )
        })
        .collect()
}

/// The baked clip of an og.CrowdAnimation (`crowd`): baked from og-anim's
/// skeleton and clip the first time it is drawn, at `rate` frames a
/// second evenly over the clip, and shared from then on; `None` while the
/// skeleton is not loaded, or its skin has no joints.
fn baked_clip(
    fx: &mut FxState,
    scene: &mut og_render::RenderScene,
    crowd: &CrowdPlay,
) -> Option<Baked> {
    let (skeleton, clip, skin) = (crowd.skeleton, crowd.clip, crowd.skin);
    let rate = if crowd.rate.is_finite() {
        crowd.rate.clamp(1.0, 120.0)
    } else {
        30.0
    };
    let key = (skeleton, skin, clip, rate.to_bits());
    if let Some(&b) = fx.baked.get(&key) {
        return Some(b);
    }
    let anim = fx.anim.as_ref()?;
    let skel = anim.skeleton(skeleton)?;
    let clip = anim.clip(clip);
    let seconds = clip.as_ref().map_or(0.0, |c| c.duration.max(0.0));
    let count = (((seconds * rate).ceil() as usize) + 1).clamp(2, 4096);
    let binding = clip.as_ref().map(|c| og_anim::bind(&skel, c));
    let mut frames = Vec::new();
    let mut palette = Vec::new();
    for k in 0..count {
        let mut pose = og_anim::Pose::rest(&skel);
        if let (Some(c), Some(b)) = (&clip, &binding) {
            let t = seconds * k as f32 / (count - 1) as f32;
            og_anim::sample(&skel, c, b, t, og_anim::NONE, &mut pose);
        }
        skel.palette(skin, &pose.nodes, &mut palette);
        frames.extend_from_slice(&palette);
    }
    if palette.is_empty() {
        return None;
    }
    let index = scene.objects.baked_clips.len() as u32;
    scene.objects.baked_clips.push(og_render::BakedClip {
        joints: palette.len() as u32,
        frames,
    });
    let b = Baked {
        index,
        frames: count,
        seconds,
    };
    fx.baked.insert(key, b);
    Some(b)
}

/// Draws a model played by og.CrowdAnimation (`crowd`) on the GPU: its
/// skinned primitives as skinned instances posed by the shared baked clip
/// at the frame of `seconds` (the world's time, between ticks on a frame
/// drawn between them), with the frame before's clip, frame and transform
/// for TAA; its other parts as instances, as [`posed_model`] draws them.
/// `false` when it cannot (no skin data or skeleton yet): the caller draws
/// the model at its bind pose.
#[allow(clippy::too_many_arguments)]
pub(crate) fn crowd_model(
    rs: &mut RenderState,
    entity: u64,
    asset: u64,
    world_matrix: Mat4,
    seconds: f64,
    crowd: &CrowdPlay,
    parts: &[(MeshHandle, og_render::MaterialHandle, Mat4)],
    lods: Option<&crate::present::extract::ModelLods>,
    look: ModelLook,
    keys: &mut Vec<(u64, u32)>,
) -> bool {
    let RenderState { scene, fx, .. } = rs;
    let Some(info) = fx.models.get(&asset).cloned() else {
        return false;
    };
    let Some(Baked {
        index: clip,
        frames: count,
        seconds: length,
    }) = baked_clip(fx, scene, crowd)
    else {
        return false;
    };
    let Some(skin) = fx.skins.get_mut(&info.id) else {
        return false;
    };
    let meshes = skinned_meshes(scene, &info, skin, info.prim_mesh.len());
    // The clip's time, then its frame (in f64: a world hours old keeps
    // its sub-frame precision).
    let t = f64::from(crowd.offset) + f64::from(crowd.speed) * seconds;
    let length = f64::from(length);
    let t = if length <= 0.0 {
        0.0
    } else if crowd.looping {
        t.rem_euclid(length)
    } else {
        t.clamp(0.0, length)
    };
    let frame = if length > 0.0 {
        (t / length * (count - 1) as f64) as f32
    } else {
        0.0
    };
    let (previous_clip, previous_frame, previous_transform) = fx
        .drawn_baked
        .get(&entity)
        .copied()
        .unwrap_or((clip, frame, world_matrix));
    fx.baked_now.insert(entity, (clip, frame, world_matrix));
    let pose = og_render::BakedPose {
        clip,
        frame,
        previous_clip,
        previous_frame,
        previous_transform,
    };
    // Its skinned parts draw as a member of the frame's crowd of the same
    // parts (og-render's crowd.rs: culled, compacted and drawn on the GPU),
    // each with the model's coarser levels, drawn where their error shows
    // under a pixel; its rigid parts as instances.
    let most = og_render::MAX_CROWD_LEVELS;
    let skinned: Vec<og_render::CrowdPart> = parts
        .iter()
        .filter_map(|&(mesh, material, _)| {
            let p = info.primitive(mesh);
            let coarser = skin.levels.get(p).map_or(&[][..], |l| l.as_slice());
            meshes
                .get(p)
                .copied()
                .flatten()
                .map(|m| og_render::CrowdPart {
                    levels: std::iter::once(m)
                        .chain(coarser.iter().copied())
                        .take(if lods.is_some() { most } else { 1 })
                        .collect(),
                    material,
                })
        })
        .collect();
    if !skinned.is_empty() {
        let mut member = og_render::CrowdMember::new(world_matrix, clip, frame);
        member.pose = pose;
        member.fx = look.fx;
        member.receive_shadows = look.receive;
        let at = scene.objects.crowds.iter().position(|c| {
            c.cast_shadows == look.cast
                && c.parts == skinned
                && c.mirrored() == og_render::mirrors(&world_matrix)
        });
        let row = match at {
            Some(i) => &mut scene.objects.crowds[i],
            None => {
                // The levels' errors and the sphere they are measured
                // from: the model's, as its unposed instances draw by.
                let levels = skinned.iter().map(|p| p.levels.len()).max().unwrap_or(1);
                let (errors, center, radius) = match lods {
                    Some(l) if levels > 1 => (
                        (0..levels).map(|k| l.mesh_error(k)).collect(),
                        l.center,
                        l.radius,
                    ),
                    _ => (vec![0.0], Vec3::ZERO, 0.0),
                };
                scene.objects.crowds.push(og_render::Crowd {
                    parts: skinned,
                    errors,
                    center,
                    radius,
                    cast_shadows: look.cast,
                    ..Default::default()
                });
                scene.objects.crowds.last_mut().expect("just pushed")
            }
        };
        row.members.push(member);
    }
    for (k, &(mesh, material, local)) in parts.iter().enumerate() {
        let p = info.primitive(mesh);
        match meshes.get(p).copied().flatten() {
            Some(_) => {}
            None => {
                scene.objects.instances.push(og_render::Instance {
                    mesh,
                    material,
                    transform: world_matrix * local,
                    material_layer: None,
                    fx: look.fx,
                    cast_shadows: look.cast,
                    receive_shadows: look.receive,
                    sway: 0.0,
                });
                keys.push((entity, k as u32));
            }
        }
    }
    true
}
