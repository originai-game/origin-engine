//! og.Garment from the runtime's side (R9 cloth): the system that poses
//! each garment by its model's skin before og.physics steps it, and the
//! drawing of the simulated points as the garment primitives' own vertices.
//!
//! A garment is one glTF mesh of the entity's og.Model (`Garment.mesh`).
//! Its primitives' vertices, welded where they share a position (a seam),
//! are the rig's points ([`og_physics::GarmentRig`]), each skinned as its
//! first vertex is, and free as that vertex's colour's red. Every tick,
//! after og.anim and before og.physics, the system skins the points with
//! the tick's joint palette on the CPU and hands them, in the world, to
//! physics as the garment's targets ([`og_physics::GarmentPose`]). The
//! points physics leaves are drawn as the offsets from those targets on
//! top of the frame's own pose (so a frame between ticks keeps the pinned
//! edge on the body), through a copy of each primitive with no skin
//! weights (`og_render::SkinnedInstance::deformed`). Only cooked data and
//! the tick's pose feed the simulation; nothing drawn reads back into it.

use super::engine::engine;
use super::fx::{FxState, SkinnedModel};
use glam::{Mat4, Vec3};
use og_core::Reflect;
use og_ecs::{Entity, QueryDesc, SystemDesc, World};
use og_physics::{Garment, GarmentPose, GarmentRig, Parent, PhysicsSystem, Transform};
use og_render::{MeshHandle, RenderScene, SkinData};
use std::collections::BTreeMap;

/// The name of the garment pose system.
pub(crate) const SYSTEM: &str = "og.garments";

/// One model mesh made into a garment: its rig and how its primitives'
/// vertices map onto the rig's points.
#[derive(Clone, Debug)]
pub(crate) struct Weld {
    /// The rig's key for og.physics (the model id and mesh, hashed).
    pub key: u64,
    pub rig: GarmentRig,
    /// The model primitives of the mesh, ascending.
    pub prims: Vec<usize>,
    /// Per primitive (as `prims`), each vertex's point.
    pub point_of: Vec<Vec<u32>>,
    /// Per point: its first vertex's joints and weights.
    pub skin: Vec<([u16; 4], [f32; 4])>,
    /// Per point: its first vertex's normal at rest.
    pub normal: Vec<Vec3>,
    /// Per primitive, per vertex: 1 when its rest normal faces as its
    /// point's, -1 when it is the other side's (a two-sided garment's back).
    pub side: Vec<Vec<f32>>,
    /// Per primitive: its copy with no skin weights, made when first drawn.
    pub zero: Vec<Option<MeshHandle>>,
    pub draw: Vec<std::sync::Arc<og_render::GarmentTopology>>,
}

/// The garment state the system and the drawing share (in the render
/// state's effects, beside the skins it is made from).
#[derive(Clone, Debug, Default)]
pub(crate) struct Garments {
    /// Welds by (model asset handle bits, glTF mesh).
    pub welds: BTreeMap<(u64, u32), Weld>,
    /// Each entity's garment (model asset, mesh, component), as the last
    /// tick posed it.
    pub posed: BTreeMap<u64, (u64, u32, Garment)>,
}

fn key_of(id: &str, mesh: u32) -> u64 {
    // FNV-1a: stable across hosts and runs.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    // Rebuild saved authoring-space constraints with bind-space lengths.
    for b in b"bind-rest:1"
        .iter()
        .copied()
        .chain(id.bytes())
        .chain(mesh.to_le_bytes())
    {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Welds glTF mesh `mesh` of a loaded model: `None` when the model has no
/// primitive of it, or one of them is not skinned.
pub(crate) fn weld(
    scene: &RenderScene,
    info: &super::fx::ModelInfo,
    skin: &SkinnedModel,
    mesh: u32,
) -> Option<Weld> {
    let prims: Vec<usize> = (0..info.prim_mesh.len())
        .filter(|&p| info.prim_mesh[p] == mesh)
        .collect();
    if prims.is_empty() {
        return None;
    }
    let mut at: BTreeMap<[u32; 3], u32> = BTreeMap::new();
    let mut rig = GarmentRig::default();
    let mut w = Weld {
        key: key_of(&info.id, mesh),
        rig: GarmentRig::default(),
        prims: prims.clone(),
        point_of: Vec::new(),
        skin: Vec::new(),
        normal: Vec::new(),
        side: Vec::new(),
        zero: vec![None; prims.len()],
        draw: Vec::new(),
    };
    for &p in &prims {
        let data = scene.mesh(*info.meshes.get(p)?)?;
        let stream = skin.streams.iter().find(|s| s.primitive == p)?;
        if stream.joints.len() != data.vertices.len() {
            return None;
        }
        let mut of = Vec::with_capacity(data.vertices.len());
        let mut side = Vec::with_capacity(data.vertices.len());
        for (v, vx) in data.vertices.iter().enumerate() {
            let bits = vx.position.map(f32::to_bits);
            let n = Vec3::from(vx.normal);
            let k = *at.entry(bits).or_insert_with(|| {
                rig.rest.push(Vec3::from(vx.position));
                // Red times alpha: alpha keeps the garment's colours (an
                // opaque material does not show it).
                rig.freedom.push(vx.color[0] * vx.color[3]);
                w.skin.push((stream.joints[v], stream.weights[v]));
                w.normal.push(n);
                rig.rest.len() as u32 - 1
            });
            side.push(if n.dot(w.normal[k as usize]) < 0.0 {
                -1.0
            } else {
                1.0
            });
            of.push(k);
        }
        for t in data.indices.as_chunks::<3>().0 {
            let get = |i: u32| of.get(i as usize).copied();
            if let (Some(a), Some(b), Some(c)) = (get(t[0]), get(t[1]), get(t[2])) {
                rig.triangles.push([a, b, c]);
            }
        }
        w.point_of.push(of);
        w.side.push(side);
    }
    rig.check().ok()?;
    w.rig = rig;
    w.draw = w
        .point_of
        .iter()
        .zip(&w.side)
        .map(|(of, side)| {
            std::sync::Arc::new(og_render::GarmentTopology {
                rest: w.rig.rest.clone(),
                normals: w.normal.clone(),
                skin: w.skin.clone(),
                triangles: w.rig.triangles.clone(),
                vertices: of.iter().copied().zip(side.iter().copied()).collect(),
            })
        })
        .collect();
    Some(w)
}

/// A point skinned by `palette` (model space): the shader's blend
/// (`skin_matrix`), weights normalised, none at all the identity.
fn skinned(palette: &[Mat4], (joints, weights): &([u16; 4], [f32; 4])) -> Mat4 {
    let total: f32 = weights.iter().sum();
    if total < 1e-6 || palette.is_empty() {
        return Mat4::IDENTITY;
    }
    let last = palette.len() - 1;
    let mut m = Mat4::ZERO;
    for (j, w) in joints.iter().zip(weights) {
        m += palette[(*j as usize).min(last)] * (w / total);
    }
    m
}

/// The garment's points as `palette` poses them, in the model's space,
/// with their normals.
pub(crate) fn pose(w: &Weld, palette: &[Mat4]) -> (Vec<Vec3>, Vec<Vec3>) {
    let mut points = Vec::with_capacity(w.rig.rest.len());
    let mut normals = Vec::with_capacity(w.rig.rest.len());
    for ((r, s), n) in w.rig.rest.iter().zip(&w.skin).zip(&w.normal) {
        let m = skinned(palette, s);
        points.push(m.transform_point3(*r));
        normals.push(m.transform_vector3(*n).normalize_or_zero());
    }
    (points, normals)
}

/// The system that poses every og.Garment by its model's skin each tick
/// (after og.anim, before og.physics). A garment whose model is not loaded
/// and posed yet, or whose mesh is not skinned, is given no pose: physics
/// keeps its points.
pub(crate) fn schedule(
    world: &mut World,
    engine_id: u32,
    physics: &PhysicsSystem,
) -> Result<(), String> {
    let physics = physics.clone();
    world
        .register::<Transform>()
        .and_then(|_| world.register::<Parent>())
        .map_err(|e| e.reason().to_string())?;
    let mut desc = SystemDesc::new(SYSTEM)
        .read::<Transform>()
        .read::<Parent>()
        .read::<og_anim::Skeleton>();
    // og.Model is the game API's record, registered before the modules.
    if world.id_by_name("og.Model").is_ok() {
        desc.reads.push("og.Model".into());
    }
    world
        .add_system(desc, move |ctx| {
            // og.Garment is lazy: granted to this system when a game names it.
            let (Ok(_), Ok(model_id)) = (ctx.id_by_name(Garment::NAME), ctx.id_by_name("og.Model"))
            else {
                return Ok(());
            };
            let Ok(eng) = engine(engine_id) else {
                return Ok(());
            };
            let (anim, layout) = {
                let Ok(st) = eng.state.try_borrow() else {
                    return Ok(());
                };
                (st.anim.clone(), st.layouts.get("og.Model").cloned())
            };
            let (Some(anim), Some(layout)) = (anim, layout) else {
                return Ok(());
            };
            let Ok(mut render) = eng.render.try_borrow_mut() else {
                return Ok(());
            };
            let rs = &mut *render;
            rs.fx.garments.posed.clear();
            let q = QueryDesc::new().read::<Garment>().read::<Transform>();
            for a in ctx.query_with(&q)? {
                for &e in a.entities() {
                    let g = ctx.get::<Garment>(e)?;
                    let Ok(bytes) = ctx.get_bytes(e, model_id) else {
                        continue;
                    };
                    let Ok(model) = layout.read::<crate::wire::record::Model>(&bytes) else {
                        continue;
                    };
                    let asset = model.asset;
                    let Some(out) = anim.output(e) else {
                        continue;
                    };
                    let Some(w) = welded(&mut rs.fx, &rs.scene, asset, g.mesh) else {
                        continue;
                    };
                    if physics.garment_rig_key(e) != Some(w.key) {
                        let skeleton = ctx.get::<og_anim::Skeleton>(e)?;
                        let asset = anim.skeleton(skeleton.asset).expect("posed skeleton");
                        let mut palette = Vec::new();
                        asset.palette(skeleton.skin, &asset.rest_pose(), &mut palette);
                        let mut rig = w.rig.clone();
                        // Vertex positions precede the bind palette's model
                        // transform; constraints must measure the skinned rest
                        // shape, not fight that transform on the first tick.
                        rig.rest = pose(w, &palette).0;
                        physics.set_garment_rig(e, w.key, rig)?;
                    }
                    let t = ctx.get::<Transform>(e)?;
                    let (p, r, s) = og_core::math::world_pose(
                        e,
                        (t.position, t.quaternion, t.scale),
                        |c| {
                            ctx.get::<Parent>(c)
                                .ok()
                                .map(|p| Entity::from_bits(p.entity))
                        },
                        |p| {
                            ctx.get::<Transform>(p)
                                .ok()
                                .map(|t| (t.position, t.quaternion, t.scale))
                        },
                    );
                    let world = Mat4::from_scale_rotation_translation(s, r, p);
                    let (points, normals) = pose(w, &out.palette);
                    physics.set_garment_pose(
                        e,
                        GarmentPose {
                            targets: points.iter().map(|x| world.transform_point3(*x)).collect(),
                            normals: normals
                                .iter()
                                .map(|n| (r * *n).normalize_or_zero())
                                .collect(),
                            scale: s.abs().max_element(),
                        },
                    );
                    rs.fx.garments.posed.insert(e.to_bits(), (asset, g.mesh, g));
                }
            }
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| e.reason().to_string())
}

/// The weld of a model's mesh, made once its model and skin are loaded.
fn welded<'a>(fx: &'a mut FxState, scene: &RenderScene, asset: u64, mesh: u32) -> Option<&'a Weld> {
    if !fx.garments.welds.contains_key(&(asset, mesh)) {
        let info = fx.models.get(&asset)?;
        let skin = fx.skins.get(&info.id)?;
        let w = weld(scene, info, skin, mesh)?;
        fx.garments.welds.insert((asset, mesh), w);
    }
    fx.garments.welds.get(&(asset, mesh))
}

/// The drawing of `entity`'s garment, when it has one and physics has its
/// points: by model primitive, each garment primitive's weightless copy
/// and its vertices in the instance's space, posed by this frame's
/// `palette` plus each point's offset from its target.
pub(crate) fn deformed(
    fx: &mut FxState,
    scene: &mut RenderScene,
    physics: Option<&PhysicsSystem>,
    entity: u64,
    asset: u64,
    world: Mat4,
    palette: &[Mat4],
) -> BTreeMap<usize, (MeshHandle, og_render::DeformedVertices)> {
    let mut out = BTreeMap::new();
    let Some(&(posed, mesh, garment)) = fx.garments.posed.get(&entity) else {
        return out;
    };
    if posed != asset {
        return out;
    }
    let Some(meshes) = fx.models.get(&asset).map(|i| i.meshes.clone()) else {
        return out;
    };
    let Some(w) = fx.garments.welds.get_mut(&(asset, mesh)) else {
        return out;
    };
    let Some(sim) = physics.and_then(|p| p.garment_points(Entity::from_bits(entity))) else {
        return out;
    };
    let n = w.rig.rest.len();
    if sim.points.len() != n || sim.targets.len() != n {
        return out;
    }
    // SIMULATION_VISUAL: the renderer simulates it from this frame's
    // skinned pose (the tick's points are its targets), for looks only.
    let (points, targets, visual) = if garment.simulation == og_physics::SIMULATION_VISUAL {
        let (posed, normals) = pose(w, palette);
        let turn = Mat4::from_quat(world.to_scale_rotation_translation().1);
        let targets: std::sync::Arc<Vec<Vec3>> =
            std::sync::Arc::new(posed.iter().map(|p| world.transform_point3(*p)).collect());
        let body = visual_body(
            &garment,
            w,
            world,
            normals
                .iter()
                .map(|n| turn.transform_vector3(*n).normalize_or_zero())
                .collect(),
            physics
                .and_then(|p| p.gravity())
                .unwrap_or(Vec3::new(0.0, -9.81, 0.0)),
        );
        (targets.clone(), targets, Some(std::sync::Arc::new(body)))
    } else {
        (
            std::sync::Arc::new(sim.points.to_vec()),
            std::sync::Arc::new(sim.targets.to_vec()),
            None,
        )
    };
    let palette = std::sync::Arc::new(palette.to_vec());
    for (k, &p) in w.prims.iter().enumerate() {
        let zero = match w.zero[k] {
            Some(h) => h,
            None => {
                let Some(data) = meshes.get(p).and_then(|h| scene.mesh(*h)).cloned() else {
                    continue;
                };
                let v = data.vertices.len();
                let h = scene.add_skinned_mesh(
                    data,
                    SkinData {
                        joints: vec![[0; 4]; v],
                        weights: vec![[0.0; 4]; v],
                    },
                );
                w.zero[k] = Some(h);
                h
            }
        };
        let vertices = og_render::DeformedVertices {
            garment: Some(og_render::GarmentDeformation {
                key: (entity, p),
                topology: w.draw[k].clone(),
                palette: palette.clone(),
                points: points.clone(),
                targets: targets.clone(),
                world,
                output_transform: Mat4::IDENTITY,
                visual: visual.clone(),
            }),
            ..Default::default()
        };
        out.insert(p, (zero, vertices));
    }
    out
}

/// A visual garment's solver settings: the component's, with each welded
/// point's reach as the tick's rule gives it (its freedom times
/// `maxDistance`, 0 within `pinTop` of the top at rest) and the skin's
/// normals in the world this frame (`normals`), which hold the backstop.
fn visual_body(
    g: &Garment,
    w: &Weld,
    world: Mat4,
    normals: Vec<Vec3>,
    gravity: Vec3,
) -> og_render::VisualBody {
    let top = w.rig.rest.iter().fold(f32::MIN, |m, p| m.max(p.y));
    let pin = g.pin_top.max(0.0);
    let max = g.max_distance.max(0.0);
    let reach: Vec<f32> = w
        .rig
        .rest
        .iter()
        .zip(&w.rig.freedom)
        .map(|(p, f)| {
            if pin > 0.0 && p.y >= top - pin {
                0.0
            } else {
                f.clamp(0.0, 1.0) * max
            }
        })
        .collect();
    let pinned = (0..reach.len() as u32)
        .filter(|&i| reach[i as usize] <= 0.0)
        .collect();
    // Rest lengths at the drawn scale: the bind shape placed in the world.
    let rest = w
        .rig
        .rest
        .iter()
        .map(|p| world.transform_point3(*p))
        .collect();
    og_render::VisualBody {
        pinned,
        reach: reach.into(),
        backstop: g.backstop.max(0.0),
        substeps: (2 * g.substeps.clamp(1, 16) * g.iterations.clamp(1, 64)).clamp(4, 96),
        damping: g.damping,
        gravity: gravity * g.gravity_scale,
        wind: g.wind,
        gust: g.gust,
        radius: g.radius,
        friction: g.friction,
        stiffness: g.stiffness,
        bending: g.bending,
        garment: Some(og_render::VisualGarment {
            rest: std::sync::Arc::new(rest),
            normals: std::sync::Arc::new(normals),
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_render::{MeshData, Vertex};

    /// A strip of two quads, split along its middle seam (the seam's
    /// vertices in both primitives), hanging from y = 1, red 0 at the top.
    fn strip() -> (RenderScene, super::super::fx::ModelInfo, SkinnedModel) {
        masked_strip(|top| [if top { 0.0 } else { 1.0 }, 1.0, 1.0, 1.0])
    }

    /// The strip with each vertex's colour `color(at the top)`.
    fn masked_strip(
        color: impl Fn(bool) -> [f32; 4],
    ) -> (RenderScene, super::super::fx::ModelInfo, SkinnedModel) {
        let mut scene = RenderScene::new();
        let vx = |x: f32, y: f32| Vertex {
            position: [x, y, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            color: color(y >= 1.0),
            tangent: [1.0, 0.0, 0.0, 1.0],
        };
        let quad = |x0: f32, x1: f32| MeshData {
            vertices: vec![vx(x0, 1.0), vx(x1, 1.0), vx(x0, 0.0), vx(x1, 0.0)],
            indices: vec![0, 2, 1, 1, 2, 3],
        };
        scene.add_mesh(quad(0.0, 0.5));
        scene.add_mesh(quad(0.5, 1.0));
        let info = super::super::fx::ModelInfo {
            id: "cape.glb".into(),
            meshes: (0..2).filter_map(|i| scene.mesh_handle(i)).collect(),
            prim_mesh: vec![3, 3],
            part_nodes: vec![],
            part_shown: vec![],
            part_variants: vec![],
        };
        let stream = |primitive| og_bundle::runtime::SkinStream {
            primitive,
            mesh: 3,
            joints: vec![[0, 1, 0, 0]; 4],
            weights: vec![[0.5, 0.5, 0.0, 0.0]; 4],
        };
        let skin = SkinnedModel {
            streams: vec![stream(0), stream(1)],
            morphs: vec![],
            meshes: None,
            ..SkinnedModel::default()
        };
        (scene, info, skin)
    }

    #[test]
    fn a_seam_welds_into_one_point() {
        let (scene, info, skin) = strip();
        let w = weld(&scene, &info, &skin, 3).expect("welded");
        // 8 vertices, 2 shared along x = 0.5.
        assert_eq!(w.rig.rest.len(), 6);
        assert_eq!(w.rig.triangles.len(), 4);
        assert_eq!(w.point_of[0][1], w.point_of[1][0]);
        assert_eq!(w.point_of[0][3], w.point_of[1][2]);
        // Freedom from the vertex colour's red.
        let top = w
            .rig
            .rest
            .iter()
            .zip(&w.rig.freedom)
            .filter(|(p, _)| p.y >= 1.0);
        assert!(top.clone().count() == 3 && top.into_iter().all(|(_, f)| *f == 0.0));
        assert!(
            weld(&scene, &info, &skin, 2).is_none(),
            "no primitive of mesh 2"
        );
        let unskinned = SkinnedModel {
            streams: skin.streams[..1].to_vec(),
            ..SkinnedModel::default()
        };
        assert!(weld(&scene, &info, &unskinned, 3).is_none());
    }

    /// The freedom painted in the alpha (white colour, so the garment
    /// keeps its texture's colours) pins as the red does; a white strip
    /// with alpha 1 is free everywhere.
    #[test]
    fn the_alpha_pins_as_the_red_does() {
        let (scene, info, skin) = masked_strip(|top| [1.0, 1.0, 1.0, if top { 0.0 } else { 0.6 }]);
        let w = weld(&scene, &info, &skin, 3).expect("welded");
        for (p, f) in w.rig.rest.iter().zip(&w.rig.freedom) {
            let want = if p.y >= 1.0 { 0.0 } else { 0.6 };
            assert!((f - want).abs() < 1e-6, "{p} {f}");
        }
        let (scene, info, skin) = masked_strip(|_| [1.0; 4]);
        let w = weld(&scene, &info, &skin, 3).expect("welded");
        assert!(w.rig.freedom.iter().all(|&f| f == 1.0));
    }

    #[test]
    fn points_are_skinned_as_the_shader_blends() {
        let (scene, info, skin) = strip();
        let w = weld(&scene, &info, &skin, 3).unwrap();
        // Joint 0 still, joint 1 moved up 2 and turned a quarter about y:
        // half of each.
        let turn = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let palette = [
            Mat4::IDENTITY,
            Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)) * turn,
        ];
        let (points, normals) = pose(&w, &palette);
        let r = w.rig.rest[5];
        let want = (r + (turn.transform_point3(r) + Vec3::new(0.0, 2.0, 0.0))) * 0.5;
        assert!(
            (points[5] - want).length() < 1e-5,
            "{:?} vs {want:?}",
            points[5]
        );
        // (0,0,1) halfway to (1,0,0), normalised.
        let n = Vec3::new(1.0, 0.0, 1.0).normalize();
        assert!((normals[5] - n).length() < 1e-5, "{:?}", normals[5]);
    }

    #[test]
    fn the_drawn_vertices_follow_the_simulated_offsets() {
        let (mut scene, info, skin) = strip();
        let mut fx = FxState::default();
        fx.models.insert(7, info.clone());
        fx.skins.insert(info.id.clone(), skin);
        let w = welded(&mut fx, &scene, 7, 3).unwrap().clone();
        fx.garments.posed.insert(1, (7, 3, Garment::default()));
        // No physics: nothing to draw.
        let palette = [Mat4::IDENTITY, Mat4::IDENTITY];
        assert!(deformed(&mut fx, &mut scene, None, 1, 7, Mat4::IDENTITY, &palette).is_empty());
        // The offsets: the weld is what the drawing maps through.
        let (rest, _) = pose(&w, &palette);
        assert_eq!(rest, w.rig.rest);
        let drawn = og_render::GarmentDeformation {
            key: (1, 0),
            topology: w.draw[0].clone(),
            palette: std::sync::Arc::new(palette.to_vec()),
            points: std::sync::Arc::new(rest.iter().map(|p| *p + Vec3::Z * 0.2).collect()),
            targets: std::sync::Arc::new(rest.clone()),
            world: Mat4::IDENTITY,
            output_transform: Mat4::IDENTITY,
            visual: None,
        }
        .evaluate();
        for (p, &point) in drawn.positions.iter().zip(&w.point_of[0]) {
            assert!((*p - rest[point as usize] - Vec3::Z * 0.2).length() < 1e-5);
        }
        assert!(drawn.normals.iter().all(|n| (*n - Vec3::Z).length() < 1e-5));
    }
}
