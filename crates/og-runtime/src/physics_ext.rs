//! The physics beyond rigid bodies seen from game code (the E5 round): the
//! readouts of the last step (`og.physics.ropePoints`, `clothPoints`,
//! `fluidParticles`, `waterHeight`, `vehicleWheels`, `ragdollPose`), the
//! ragdoll rig (`setRagdollRig`, `ragdollRigFromSkeleton`), and drawing a
//! ragdoll: a model posed by og-anim whose entity has a rig made from its
//! skeleton is drawn with its joints blended towards the bones' bodies by
//! og.Ragdoll.blend. Rigs are assets, like meshes: kept by the engine and
//! given again to a reloaded world's physics.

use super::engine::{Engine, engine};
use super::extract::RenderState;
use super::fx::{f, rows};
use super::{ok, ok_void, parse, refuse};
use glam::{Mat4, Quat, Vec3};
use og_ecs::{Entity, World};
use og_physics::{PhysicsSystem, RagdollBone, RagdollRig};
use serde_json::{Value, json};

fn physics(eng: &Engine) -> Result<PhysicsSystem, String> {
    eng.state
        .borrow()
        .physics
        .clone()
        .ok_or_else(|| "this engine has no physics".to_string())
}

fn entity(e: f64) -> Result<Entity, String> {
    if e.fract() != 0.0 || !(0.0..9_007_199_254_740_992.0).contains(&e) {
        return Err(format!("{e} is not an entity"));
    }
    Ok(Entity::from_bits(e as u64))
}

fn v3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
}

fn q4(q: Quat) -> Value {
    json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w })
}

fn answer(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

/// A readout of the last step for `e`, refused with `what` when it has none.
fn readout<T>(
    world: u32,
    e: f64,
    what: &str,
    read: impl FnOnce(&PhysicsSystem, Entity) -> Option<T>,
    wire: impl FnOnce(T) -> Value,
) -> String {
    answer((|| {
        let eng = engine(world)?;
        let p = physics(&eng)?;
        let e = entity(e)?;
        read(&p, e).map(wire).ok_or_else(|| {
            format!(
                "entity {} has no {what} (or it has not stepped yet)",
                e.to_bits()
            )
        })
    })())
}

pub fn og_physics_rope_points(world: u32, e: f64) -> String {
    readout(
        world,
        e,
        "og.Rope",
        |p, e| p.rope_points(e),
        |pts| json!(pts.into_iter().map(v3).collect::<Vec<_>>()),
    )
}

pub fn og_physics_cloth_points(world: u32, e: f64) -> String {
    readout(
        world,
        e,
        "og.Cloth",
        |p, e| p.cloth_points(e),
        |c| json!(c.points.into_iter().map(v3).collect::<Vec<_>>()),
    )
}

pub fn og_physics_fluid_particles(world: u32, e: f64) -> String {
    readout(
        world,
        e,
        "og.Fluid2D",
        |p, e| p.fluid_particles(e),
        |pts| {
            json!(
                pts.into_iter()
                    .map(|p| json!({ "x": p.x, "y": p.y }))
                    .collect::<Vec<_>>()
            )
        },
    )
}

pub fn og_physics_fluid3d_particles(world: u32, e: f64) -> String {
    readout(
        world,
        e,
        "og.Fluid3D",
        |p, e| p.fluid3d_particles(e),
        |pts| json!(pts.into_iter().map(v3).collect::<Vec<_>>()),
    )
}

pub fn og_physics_vehicle_wheels(world: u32, e: f64) -> String {
    readout(
        world,
        e,
        "og.Vehicle",
        |p, e| p.vehicle_wheels(e),
        |wheels| {
            json!(
            wheels
                .into_iter()
                .map(|w| json!({ "position": v3(w.position), "rotation": q4(w.quaternion), "contact": w.contact }))
                .collect::<Vec<_>>()
        )
        },
    )
}

pub fn og_physics_ragdoll_pose(world: u32, e: f64) -> String {
    readout(
        world,
        e,
        "og.Ragdoll",
        |p, e| p.ragdoll_pose(e),
        |bones| {
            json!(
                bones
                    .into_iter()
                    .map(|(p, q)| json!({ "position": v3(p), "rotation": q4(q) }))
                    .collect::<Vec<_>>()
            )
        },
    )
}

/// The entity's position from its og.Transform (water lives at the top of
/// the hierarchy, as physics sees it).
fn position_of(eng: &Engine, e: Entity) -> Result<Vec3, String> {
    let w = eng.world()?;
    let layout = eng
        .state
        .borrow()
        .layouts
        .get("og.Transform")
        .cloned()
        .ok_or("og.Transform is not registered")?;
    let id = w
        .id_by_name(&layout.schema)
        .map_err(|e| e.reason().to_string())?;
    let bytes = w
        .get_bytes(e, id)
        .map_err(|_| format!("entity {} has no og.Transform", e.to_bits()))?;
    Ok(vec3_of(&layout.decode(&bytes)["position"], Vec3::ZERO))
}

pub fn og_physics_water_height(world: u32, e: f64, x: f64, z: f64) -> String {
    answer((|| {
        let eng = engine(world)?;
        let p = physics(&eng)?;
        let e = entity(e)?;
        let at = position_of(&eng, e)?;
        let (x, z) = (x as f32, z as f32);
        if let Some(w) = p.water_heights(e) {
            let n = w.resolution as usize;
            let cell = w.size / (n - 1) as f32;
            let gx = (x - (at.x - w.size * 0.5)) / cell;
            let gz = (z - (at.z - w.size * 0.5)) / cell;
            if !(gx >= 0.0 && gz >= 0.0 && gx <= (n - 1) as f32 && gz <= (n - 1) as f32) {
                return Ok(json!(at.y));
            }
            let (i, j) = ((gx as usize).min(n - 2), (gz as usize).min(n - 2));
            let (fx, fz) = (gx - i as f32, gz - j as f32);
            let hh = |a: usize, b: usize| w.heights[b * n + a];
            let top = hh(i, j) + (hh(i + 1, j) - hh(i, j)) * fx;
            let bottom = hh(i, j + 1) + (hh(i + 1, j + 1) - hh(i, j + 1)) * fx;
            return Ok(json!(at.y + top + (bottom - top) * fz));
        }
        if let Some(line) = p.water_line(e) {
            let layout = eng.state.borrow().layouts.get("og.WaterLine").cloned();
            let w = eng.world()?;
            let width = rows(w, layout.as_ref())
                .into_iter()
                .find(|(id, _)| *id == e.to_bits())
                .map_or(12.0, |(_, v)| f(&v, "width", 12.0));
            let n = line.len();
            if n < 2 {
                return Ok(json!(at.y));
            }
            let g = (x - (at.x - width * 0.5)) / width * (n - 1) as f32;
            if !(g >= 0.0 && g <= (n - 1) as f32) {
                return Ok(json!(at.y));
            }
            let i = (g as usize).min(n - 2);
            let t = g - i as f32;
            return Ok(json!(at.y + line[i] + (line[i + 1] - line[i]) * t));
        }
        Err(format!(
            "entity {} has no og.WaterSurface or og.WaterLine (or it has not stepped yet)",
            e.to_bits()
        ))
    })())
}

// ── ragdoll rigs ────────────────────────────────────────────────────────

/// A rig as the engine keeps it: what physics simulates, and the skeleton
/// joints its bones stand for (drawing).
#[derive(Clone, Debug)]
pub(crate) struct KeptRig {
    pub rig: RagdollRig,
    pub skeleton: u64,
    pub joints: Vec<i32>,
}

fn vec3_of(v: &Value, default: Vec3) -> Vec3 {
    if !v.is_object() {
        return default;
    }
    Vec3::new(
        v["x"].as_f64().unwrap_or(0.0) as f32,
        v["y"].as_f64().unwrap_or(0.0) as f32,
        v["z"].as_f64().unwrap_or(0.0) as f32,
    )
}

fn quat_of(v: &Value) -> Quat {
    let q = Quat::from_xyzw(
        v["x"].as_f64().unwrap_or(0.0) as f32,
        v["y"].as_f64().unwrap_or(0.0) as f32,
        v["z"].as_f64().unwrap_or(0.0) as f32,
        v["w"].as_f64().unwrap_or(0.0) as f32,
    );
    if q.length_squared() < 1e-12 {
        Quat::IDENTITY
    } else {
        q.normalize()
    }
}

/// A RagdollRigDesc into a kept rig.
fn rig_of(d: &Value) -> Result<KeptRig, String> {
    let bones = d["bones"].as_array().cloned().unwrap_or_default();
    if bones.is_empty() {
        return Err("a ragdoll rig needs at least one bone".into());
    }
    let mut rig = RagdollRig::default();
    let mut joints = Vec::with_capacity(bones.len());
    for (i, b) in bones.iter().enumerate() {
        let parent = b["parent"].as_i64().unwrap_or(-1);
        if parent >= i as i64 {
            return Err(format!(
                "bone {i}: its parent ({parent}) must come before it"
            ));
        }
        rig.bones.push(RagdollBone {
            parent: (parent >= 0).then_some(parent as u32),
            rest: (vec3_of(&b["position"], Vec3::ZERO), quat_of(&b["rotation"])),
            length: b["length"].as_f64().map_or(0.25, |l| l as f32),
            axis: vec3_of(&b["axis"], Vec3::Y),
        });
        joints.push(b["joint"].as_i64().unwrap_or(-1) as i32);
    }
    Ok(KeptRig {
        rig,
        skeleton: d["skeleton"].as_f64().unwrap_or(0.0) as u64,
        joints,
    })
}

pub fn og_physics_set_ragdoll_rig(world: u32, e: f64, desc: Option<String>) -> String {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let p = physics(&eng)?;
        let e = entity(e)?;
        let kept = rig_of(&parse(desc.as_deref())?)?;
        p.set_ragdoll_rig(e, kept.rig.clone())
            .map_err(|err| err.reason().to_string())?;
        eng.render.borrow_mut().atmo.rigs.insert(e.to_bits(), kept);
        Ok(())
    })();
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// A rig from a skeleton's first skin: a bone per joint, parents first,
/// each from its joint's rest pose in model space to its first child joint
/// (a leaf gets 60% of its parent's length, pointing on).
pub(crate) fn rig_from_skeleton(s: &og_anim::SkeletonAsset) -> Result<Value, String> {
    let skin = s.skins.first().ok_or("the skeleton has no skin")?;
    let model = s.model_space(&s.rest_pose());
    let joint_of = |node: u32| skin.joints.iter().position(|&n| n == node);
    // The nearest ancestor that is a joint.
    let parent_joint = |node: u32| {
        let mut n = s.nodes[node as usize].parent;
        while n != og_anim::NONE {
            if let Some(j) = joint_of(n) {
                return Some(j);
            }
            n = s.nodes[n as usize].parent;
        }
        None
    };
    // Parents first: by depth, then joint order.
    let depth = |mut j: usize| {
        let mut d = 0;
        while let Some(p) = parent_joint(skin.joints[j]) {
            j = p;
            d += 1;
            if d > 512 {
                break;
            }
        }
        d
    };
    let mut order: Vec<usize> = (0..skin.joints.len()).collect();
    order.sort_by_key(|&j| (depth(j), j));
    let bone_of = |j: usize| order.iter().position(|&o| o == j);
    let mut bones: Vec<Value> = Vec::with_capacity(order.len());
    let mut lengths: Vec<f32> = Vec::with_capacity(order.len());
    for &j in &order {
        let node = skin.joints[j];
        let at = model[node as usize];
        let child = (0..skin.joints.len()).find(|&c| parent_joint(skin.joints[c]) == Some(j));
        let parent = parent_joint(node).and_then(bone_of);
        let (length, axis) = match child {
            Some(c) => {
                let d = model[skin.joints[c] as usize].translation - at.translation;
                (d.length(), at.rotation.inverse() * d)
            }
            None => {
                let pl = parent.map_or(0.2, |p| lengths[p] * 0.6);
                let dir = parent_joint(node)
                    .map(|p| at.translation - model[skin.joints[p] as usize].translation)
                    .unwrap_or(Vec3::Y);
                (pl, at.rotation.inverse() * dir)
            }
        };
        let length = length.max(0.02);
        lengths.push(length);
        let axis = axis.normalize_or(Vec3::Y);
        bones.push(json!({
            "parent": parent.map_or(-1, |p| p as i64),
            "position": v3(at.translation),
            "rotation": q4(at.rotation),
            "length": length,
            "axis": v3(axis),
            "joint": j,
        }));
    }
    Ok(json!({ "bones": bones }))
}

/// The system that hands each og.Ragdoll the animation's pose of this
/// tick (after og.anim, before og.physics): every bone of a rig made from a
/// skeleton (`ragdollRigFromSkeleton`) follows its joint, in world space, so
/// a ragdoll at weight 0 tracks the animation and goes limp from it. A bone
/// with no joint, or an entity with no pose yet, keeps the rig's rest pose.
pub(crate) fn schedule_ragdoll_targets(
    world: &mut og_ecs::World,
    engine_id: u32,
    physics: &og_physics::PhysicsSystem,
) -> Result<(), String> {
    use og_physics::{Ragdoll, Transform};
    let physics = physics.clone();
    world
        .register::<Transform>()
        .and_then(|_| world.register::<Ragdoll>())
        .map_err(|e| e.reason().to_string())?;
    world
        .add_system(
            og_ecs::SystemDesc::new(RAGDOLL_TARGETS)
                .read::<Ragdoll>()
                .read::<Transform>()
                .query(
                    og_ecs::QueryDesc::new()
                        .read::<Ragdoll>()
                        .read::<Transform>(),
                ),
            move |ctx| {
                let Ok(eng) = engine(engine_id) else {
                    return Ok(());
                };
                let Some(anim) = eng.state.try_borrow().ok().and_then(|s| s.anim.clone()) else {
                    return Ok(());
                };
                let Ok(render) = eng.render.try_borrow() else {
                    return Ok(());
                };
                let rigs = &render.atmo.rigs;
                if rigs.is_empty() {
                    return Ok(());
                }
                for a in ctx.query(0)? {
                    for &e in a.entities() {
                        let Some(kept) = rigs.get(&e.to_bits()) else {
                            continue;
                        };
                        let (Some(skeleton), Some(pose)) =
                            (anim.skeleton(kept.skeleton), anim.pose(e))
                        else {
                            continue;
                        };
                        let Some(skin) = skeleton.skins.first() else {
                            continue;
                        };
                        let t = ctx.get::<Transform>(e)?;
                        let model = skeleton.model_space(&pose);
                        let targets = kept
                            .rig
                            .bones
                            .iter()
                            .zip(&kept.joints)
                            .map(|(b, &j)| {
                                let (p, q) = usize::try_from(j)
                                    .ok()
                                    .and_then(|j| skin.joints.get(j))
                                    .and_then(|&n| model.get(n as usize))
                                    .map_or(b.rest, |m| (m.translation, m.rotation));
                                (
                                    t.position + t.quaternion * p,
                                    (t.quaternion * q).normalize(),
                                )
                            })
                            .collect();
                        physics.set_ragdoll_targets(e, targets);
                    }
                }
                Ok(())
            },
        )
        .map(|_| ())
        .map_err(|e| e.reason().to_string())
}

/// The name of the ragdoll target system.
pub(crate) const RAGDOLL_TARGETS: &str = "og.ragdollTargets";

pub fn og_physics_ragdoll_rig_from_skeleton(world: u32, skeleton: f64) -> String {
    answer((|| {
        let eng = engine(world)?;
        let anim = eng
            .state
            .borrow()
            .anim
            .clone()
            .ok_or("this engine has no animation")?;
        let s = anim
            .skeleton(skeleton as u64)
            .ok_or_else(|| format!("{skeleton} is not a loaded skeleton (og.load.animation)"))?;
        let mut rig = rig_from_skeleton(&s)?;
        rig["skeleton"] = json!(skeleton);
        Ok(rig)
    })())
}

/// The ragdolls' drawing: each posed model (palette `palettes[e]`) whose
/// entity has a rig from its skeleton gets its joints blended towards the
/// bones' bodies by og.Ragdoll.blend.
pub(crate) fn draw_ragdolls(
    world: &World,
    rs: &mut RenderState,
    world_of: &mut dyn FnMut(u64) -> Mat4,
) {
    let RenderState {
        scene,
        fx,
        draw,
        atmo,
        ..
    } = rs;
    let (Some(physics), Some(anim)) = (draw.atmo.physics.as_ref(), fx.anim.as_ref()) else {
        return;
    };
    let layout = draw.atmo.ragdoll.as_ref();
    for (e, v) in rows(world, layout) {
        let blend = f(&v, "blend", 0.0).clamp(0.0, 1.0);
        let (Some(kept), Some(&palette)) = (atmo.rigs.get(&e), atmo.palettes.get(&e)) else {
            continue;
        };
        if blend <= 0.0 {
            continue;
        }
        let (Some(skeleton), Some(bodies)) = (
            anim.skeleton(kept.skeleton),
            physics.ragdoll_pose(Entity::from_bits(e)),
        ) else {
            continue;
        };
        let Some(skin) = skeleton.skins.first() else {
            continue;
        };
        let Some(matrices) = scene
            .joint_palettes
            .get_mut(palette as usize)
            .map(|p| &mut p.matrices)
        else {
            continue;
        };
        let to_model = world_of(e).inverse();
        for (bone, &(p, q)) in bodies.iter().enumerate() {
            let Some(j) = kept.joints.get(bone).and_then(|&j| usize::try_from(j).ok()) else {
                continue;
            };
            let (Some(m), Some(ib)) = (matrices.get_mut(j), skin.inverse_bind.get(j)) else {
                continue;
            };
            let animated = *m * ib.inverse();
            let simulated = to_model * Mat4::from_rotation_translation(q, p);
            let (sa, ra, ta) = animated.to_scale_rotation_translation();
            let (_, rb, tb) = simulated.to_scale_rotation_translation();
            let blended =
                Mat4::from_scale_rotation_translation(sa, ra.slerp(rb, blend), ta.lerp(tb, blend));
            *m = blended * *ib;
        }
    }
}
