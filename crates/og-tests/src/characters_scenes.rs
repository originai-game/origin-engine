//! The characters family's game shots: GPU skinning, skinned material
//! hooks, per-instance looks and baked crowds, set on the ruined
//! processional way (`atmo_scenes::colonnade`) and in the old town.
//!
//! - `skinning`: a kraken's five tentacles burst through the paving of
//!   the processional way under a low afternoon sun, each posed by its own
//!   palette (`fx::bend_palette`), one mesh and one material.
//! - `skinned-hooks`: the same tentacles drawn by a material shader
//!   (`p2_scenes::ripple_shader`) that ripples, bulges and bands them and
//!   lifts them 0.4 m: their roots stand 0.4 m under the paving so the
//!   shader brings them up to it, banded dark, and their shadows, the prepass and AO
//!   follow the displaced surface.
//! - `model-flash`: the trickster trio with two tentacles rearing behind
//!   them: the middle one flashes white as it is hit, the right one glows amber, and the
//!   right tentacle flashes red; one draw per (mesh, material).
//! - `crowd-culling`: pilgrims queueing at the foot of the town wall, its
//!   front two rows walking before it and three more behind it (hidden by
//!   the occlusion test), and a guard whose palette carries him 12 m from
//!   where his bind pose stands.
//! - `crowd-lods`: a festival day on the old town's avenue: the same
//!   2,100 walkers in twelve outfits (full mesh near, both coarser levels,
//!   walking cards past 37 m) between rows of townhouses and lamps, the
//!   stone gate 60 m down hiding the crowd behind it, a low sun from the
//!   right casting long shadows across them.

use crate::atmo_scenes::colonnade;
use crate::fx_scenes::{
    CROWD_FRAMES, CYLINDER_HEIGHT, bend_palette, skinned_cylinder, walker_clip, walker_parts,
};
use crate::kit;
use crate::sky_scenes::physical_light_under;
use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Camera, Exposure, InstanceFx, JointPalette, MaterialHandle, MeshData, RenderScene, SkinData,
    SkinnedInstance, StandardMaterial, shapes,
};

/// Where the kraken breaks through the processional way.
const PIT: Vec3 = Vec3::new(0.0, 0.0, -9.0);
/// The tentacles' size: [`CYLINDER_HEIGHT`] × this, about 4.3 m.
const TENTACLE_SCALE: f32 = 1.8;

/// A tentacle: [`skinned_cylinder`]'s mesh and skin (`base` thick),
/// tapering to a sixth of that and closing round at its tip, coloured by its vertices: dark
/// maroon at the root to violet at the tip, its pale sucker side (+x,
/// the side its bend curls towards) and a mottle along it.
pub fn tentacle(base: f32) -> (MeshData, SkinData) {
    let (mut mesh, skin) = skinned_cylinder(base, 32, 24);
    let slope = base * 0.84 / CYLINDER_HEIGHT;
    for v in &mut mesh.vertices {
        let p = Vec3::from(v.position);
        let t = (p.y / CYLINDER_HEIGHT).clamp(0.0, 1.0);
        // The last eighth closes round, its cap to a point.
        let close = ((t - 0.88) / 0.12).clamp(0.0, 1.0);
        let taper = (1.0 - 0.84 * t.powf(1.3)) * (1.0 - close * close).max(0.0).sqrt().max(0.04);
        v.position = [p.x * taper, p.y, p.z * taper];
        let side = Vec3::from(v.normal);
        if side.y.abs() < 0.5 {
            // Along the taper, then turning up over the round tip.
            let n = (side + Vec3::Y * slope).normalize();
            v.normal = n.lerp(Vec3::Y, close).normalize().to_array();
        }
        let r = Vec2::new(p.x, p.z).length() / base.max(1e-6);
        let along = if r > 0.01 { p.x / (r * base) } else { 0.0 };
        let mottle = 0.5 + 0.5 * ((p.y * 7.3).sin() * (along * 5.0 + p.y * 2.1).cos());
        let root = Vec3::new(0.09, 0.012, 0.03);
        let tip = Vec3::new(0.24, 0.05, 0.13);
        let pale = Vec3::new(0.62, 0.32, 0.3);
        let mut c = root.lerp(tip, t);
        c = c.lerp(c * 0.7, mottle * 0.5);
        c = c.lerp(pale, ((along - 0.35) / 0.65).clamp(0.0, 1.0) * 0.8);
        v.color = [c.x, c.y, c.z, 1.0];
    }
    (mesh, skin)
}

/// The tentacles' flesh: wet (a clear coat over a satin skin), coloured
/// by the mesh's vertices.
pub fn tentacle_skin() -> StandardMaterial {
    StandardMaterial {
        color: Vec3::ONE,
        roughness: 0.5,
        clearcoat: 0.7,
        clearcoatRoughness: 0.12,
        ..Default::default()
    }
}

/// The five tentacles' places round the pit: (x, z, yaw).
const TENTACLES: [(f32, f32, f32); 5] = [
    (-2.6, 0.6, 0.5),
    (-1.3, -0.5, 0.2),
    (0.0, 0.2, 1.4),
    (1.4, -0.4, 2.6),
    (2.7, 0.7, 3.9),
];

/// The paving the kraken broke: boulders thrown up round the pit.
fn pit(scene: &mut RenderScene) {
    let boulder = kit::prop(scene, "boulder");
    for (x, z, yaw, h) in [
        (-3.6, 1.6, 0.4, 0.7),
        (-0.6, 1.4, 2.0, 0.45),
        (3.3, 1.8, 1.1, 0.6),
        (0.9, -1.9, 3.0, 0.8),
    ] {
        boulder.place(scene, PIT + Vec3::new(x, 0.0, z), yaw, h);
    }
}

/// The kraken's five tentacles round [`PIT`], `sink` metres into the
/// ground, each posed by its own palette (bending 0.62 to -0.7 a bone,
/// twisting 0 to 0.8), with `material`.
fn tentacles(scene: &mut RenderScene, material: MaterialHandle, sink: f32) {
    let (mesh, skin) = tentacle(0.3);
    let mesh = scene.add_skinned_mesh(mesh, skin);
    for (i, &(x, z, yaw)) in TENTACLES.iter().enumerate() {
        let bend = [0.62, 0.3, -0.12, -0.42, -0.7][i];
        scene
            .objects
            .joint_palettes
            .push(bend_palette(bend, i as f32 * 0.2));
        scene.objects.skinned_instances.push(SkinnedInstance::new(
            mesh,
            material,
            Mat4::from_translation(PIT + Vec3::new(x, -sink, z))
                * Mat4::from_rotation_y(yaw)
                * Mat4::from_scale(Vec3::splat(TENTACLE_SCALE)),
            i as u32,
        ));
    }
}

/// The processional way late in the afternoon: a low sun from the left,
/// beyond the tentacles, under the physical sky.
fn storm_light(scene: &mut RenderScene) {
    colonnade(scene);
    physical_light_under(scene, Vec3::new(-0.75, 0.2, -0.3));
    scene.lighting.light.cast_shadows = true;
}

/// `skinning` (module docs).
pub fn kraken_road() -> RenderScene {
    let mut scene = RenderScene::new();
    storm_light(&mut scene);
    pit(&mut scene);
    let flesh = scene.add_standard_material(tentacle_skin());
    tentacles(&mut scene, flesh, 0.0);
    scene.frame.camera = kraken_camera();
    scene
}

fn kraken_camera() -> Camera {
    Camera {
        position: Vec3::new(1.4, 1.3, -0.2),
        target: PIT + Vec3::new(-0.2, 2.5, 0.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.3 },
        ..Default::default()
    }
}

/// `skinned-hooks` (module docs): [`kraken_road`]'s tentacles through
/// `p2_scenes::ripple_shader`, banded pale on the maroon, outlined, their
/// roots sunk the 0.4 m the shader lifts them, under GTAO.
pub fn kraken_hooks() -> RenderScene {
    let mut scene = RenderScene::new();
    storm_light(&mut scene);
    pit(&mut scene);
    let ripple = scene.add_shader(crate::p2_scenes::ripple_shader());
    let m = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.42, 0.1, 0.2),
        roughness: 0.4,
        outlineWidth: 2.0,
        outlineColor: Vec3::new(0.05, 0.02, 0.04),
        ..Default::default()
    });
    scene.set_material_shader(m, Some(ripple));
    scene.set_shader_param(m, "offset", &[0.0, 0.4, 0.0]);
    scene.set_shader_param(m, "bulge", &[0.05]);
    scene.set_shader_param(m, "stripe", &[0.1, 0.02, 0.05]);
    tentacles(&mut scene, m, 0.4 * TENTACLE_SCALE);
    scene.look.post.ao = Some(og_render::AmbientOcclusion {
        radius: 0.6,
        intensity: 1.0,
        method: og_render::AoMethod::Gtao,
        ..Default::default()
    });
    scene.frame.tick = 30;
    scene.frame.camera = kraken_camera();
    scene
}

/// `model-flash` (module docs): the trickster trio (one glTF model, its
/// materials shared) at the front, two tentacles rearing behind them.
pub fn kraken_duel() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    storm_light(&mut scene);
    pit(&mut scene);
    let model = crate::scenes::load("trickster.glb")?;
    let parts = model.add_to(&mut scene);
    let scale = 1.6 / (model.max - model.min).max_element().max(1e-3);
    let looks = [
        InstanceFx::NONE,
        InstanceFx {
            flash: 0.6,
            ..InstanceFx::NONE
        },
        InstanceFx {
            tint: Vec3::new(1.0, 0.6, 0.25),
            ..InstanceFx::NONE
        },
    ];
    for (i, fx) in looks.into_iter().enumerate() {
        let x = -1.6 + i as f32 * 1.6;
        let m = Mat4::from_translation(
            PIT + Vec3::new(x, -model.min.y * scale, 3.6 - (i % 2) as f32 * 0.5),
        ) * Mat4::from_rotation_y(crate::scenes::TRICKSTER_TO_Z + x * 0.15)
            * Mat4::from_scale(Vec3::splat(scale));
        for &(mesh, material) in &parts.parts {
            scene.objects.instances.push(og_render::Instance {
                mesh,
                material,
                transform: m,
                material_layer: None,
                fx,
                cast_shadows: true,
                receive_shadows: true,
                sway: 0.0,
            });
        }
    }
    let (mesh, skin) = tentacle(0.3);
    let mesh = scene.add_skinned_mesh(mesh, skin);
    let flesh = scene.add_standard_material(tentacle_skin());
    for i in 0..2u32 {
        scene.objects.joint_palettes.push(bend_palette(0.45, 0.3));
        let x = -1.4 + i as f32 * 2.8;
        let mut inst = SkinnedInstance::new(
            mesh,
            flesh,
            Mat4::from_translation(PIT + Vec3::new(x, 0.0, 0.2))
                * Mat4::from_rotation_y(i as f32 * std::f32::consts::PI)
                * Mat4::from_scale(Vec3::splat(TENTACLE_SCALE)),
            i,
        );
        if i == 1 {
            inst.fx = InstanceFx {
                flash: 0.5,
                flash_color: Vec3::new(1.0, 0.1, 0.05),
                ..InstanceFx::NONE
            };
        }
        scene.objects.skinned_instances.push(inst);
    }
    scene.frame.camera = Camera {
        position: PIT + Vec3::new(0.6, 1.5, 8.4),
        target: PIT + Vec3::new(0.0, 1.7, 1.5),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.3 },
        ..Default::default()
    };
    Ok(scene)
}

/// Outfits of muted cloth: (trousers, shirt).
const OUTFITS: [([f32; 3], [f32; 3]); 6] = [
    ([0.16, 0.18, 0.28], [0.62, 0.22, 0.16]),
    ([0.30, 0.24, 0.18], [0.18, 0.32, 0.55]),
    ([0.12, 0.12, 0.13], [0.78, 0.72, 0.58]),
    ([0.45, 0.42, 0.36], [0.22, 0.42, 0.26]),
    ([0.20, 0.26, 0.40], [0.55, 0.45, 0.18]),
    ([0.26, 0.22, 0.20], [0.30, 0.30, 0.34]),
];
/// Skin in three tones.
const SKINS: [[f32; 3]; 3] = [[0.86, 0.66, 0.52], [0.52, 0.36, 0.26], [0.7, 0.5, 0.38]];

/// `crowd-culling` (module docs): 7 × 5 walkers playing one baked walk at
/// their own frames, rows 1.6 m apart, the back three behind a town wall
/// 30 m wide and 10 m tall at z -2.6 (past the frame's top and sides: the
/// occlusion test reads up to 2 × 2 texels of the depth pyramid round a
/// box's rectangle, so a wall whose top ran just above the crowd would
/// leave sky in them and hide nothing), and before its gate a guard posed
/// by a palette that carries him 12 m from where his bind pose stands
/// (out of view there): culled by his posed box, he is drawn.
pub fn wall_queue() -> RenderScene {
    let mut scene = RenderScene::new();
    let cobbles = kit::surface(&mut scene, "cobbles");
    let ground = scene.add_mesh(kit::slab(
        Vec2::new(-40.0, -40.0),
        Vec2::new(40.0, 30.0),
        0.0,
        2.4,
    ));
    scene.add_instance(ground, cobbles, Mat4::IDENTITY);
    let stone = kit::surface_tinted(&mut scene, "stone", [0.92, 0.88, 0.8]);
    let wall = scene.add_mesh(kit::block(
        Vec3::new(-15.0, 0.0, -2.8),
        Vec3::new(15.0, 10.0, -2.4),
        2.6,
    ));
    scene.add_instance(wall, stone, Mat4::IDENTITY);
    // Buttresses and a plinth along the wall's foot.
    for x in [-9.0f32, 9.0] {
        let b = scene.add_mesh(kit::block(
            Vec3::new(x - 0.7, 0.0, -2.4),
            Vec3::new(x + 0.7, 10.0, -1.9),
            2.6,
        ));
        scene.add_instance(b, stone, Mat4::IDENTITY);
    }
    // The gate: oak doors in a stone frame behind the guard.
    let planks = kit::surface_tinted(&mut scene, "planks", [0.7, 0.55, 0.42]);
    let doors = scene.add_mesh(kit::block(
        Vec3::new(-1.6, 0.0, -2.42),
        Vec3::new(1.6, 3.8, -2.3),
        1.4,
    ));
    scene.add_instance(doors, planks, Mat4::IDENTITY);
    for (min, max) in [
        (Vec3::new(-2.1, 0.0, -2.4), Vec3::new(-1.6, 4.3, -2.15)),
        (Vec3::new(1.6, 0.0, -2.4), Vec3::new(2.1, 4.3, -2.15)),
        (Vec3::new(-2.1, 3.8, -2.4), Vec3::new(2.1, 4.3, -2.15)),
    ] {
        let jamb = scene.add_mesh(kit::block(min, max, 1.2));
        scene.add_instance(jamb, stone, Mat4::IDENTITY);
    }
    // Banners either side of the gate.
    let cloth = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.5, 0.06, 0.05),
        roughness: 0.85,
        sheen: 0.6,
        sheenColor: Vec3::new(0.9, 0.5, 0.4),
        ..Default::default()
    });
    let banner = scene.add_mesh(shapes::cuboid(Vec3::new(1.2, 3.2, 0.04)));
    for x in [-4.2f32, 4.2] {
        scene.add_instance(
            banner,
            cloth,
            Mat4::from_translation(Vec3::new(x, 6.4, -2.36)),
        );
    }
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-6.4, 0.0, -1.7), 0.3, 0.7);
    crate_.place(&mut scene, Vec3::new(-5.7, 0.0, -1.9), -0.2, 0.7);
    crate_.place(&mut scene, Vec3::new(-6.1, 0.7, -1.85), 0.6, 0.7);
    kit::prop(&mut scene, "barrel").place(&mut scene, Vec3::new(6.0, 0.0, -1.8), 0.4, 1.0);

    // The walker in three pieces (trousers, shirt, skin), each drawn
    // white and coloured by its outfit's material.
    let pieces: Vec<_> = [&[0usize, 3, 4, 5, 6][..], &[1, 7, 9], &[2, 8, 10]]
        .iter()
        .map(|parts| {
            let (mesh, skin) = walker_parts(16, 4, parts, true);
            scene.add_skinned_mesh(mesh, skin)
        })
        .collect();
    let cloth = |scene: &mut RenderScene, c: [f32; 3], roughness: f32| {
        scene.add_material(og_render::Material {
            base_color: [c[0], c[1], c[2], 1.0],
            roughness,
            ..Default::default()
        })
    };
    let outfits: Vec<[MaterialHandle; 3]> = OUTFITS
        .iter()
        .enumerate()
        .map(|(k, &(trousers, shirt))| {
            [
                cloth(&mut scene, trousers, 0.85),
                cloth(&mut scene, shirt, 0.75),
                cloth(&mut scene, SKINS[k % SKINS.len()], 0.6),
            ]
        })
        .collect();
    scene.objects.baked_clips.push(walker_clip(CROWD_FRAMES));
    for z in 0..5 {
        for x in 0..7 {
            let k = z * 7 + x;
            let f = (k as f32 * 0.37) % 1.0 * (CROWD_FRAMES - 1) as f32;
            let jitter = ((k * 37 % 11) as f32 / 10.0 - 0.5) * 0.4;
            let heading = if (k * 13) % 3 == 0 {
                -std::f32::consts::FRAC_PI_2
            } else {
                std::f32::consts::FRAC_PI_2
            } + jitter * 0.6;
            let at = Mat4::from_translation(Vec3::new(
                (x as f32 - 3.0) * 1.4 + jitter,
                0.0,
                -(z as f32) * 1.6,
            )) * Mat4::from_rotation_y(heading);
            let outfit = outfits[(k * 5 + z) % outfits.len()];
            for (&mesh, material) in pieces.iter().zip(outfit) {
                let mut inst = SkinnedInstance::new(mesh, material, at, 0);
                inst.baked = Some(og_render::BakedPose {
                    clip: 0,
                    frame: f,
                    previous_clip: 0,
                    previous_frame: f,
                    previous_transform: at,
                });
                scene.objects.skinned_instances.push(inst);
            }
        }
    }
    // The guard: his bind pose 12 m to the left (out of view), his
    // palette moving every joint 12 m right, to the gate.
    let joints = walker_clip(2).joints as usize;
    scene.objects.joint_palettes.push(JointPalette::from(vec![
        Mat4::from_translation(Vec3::new(
            12.0, 0.0, 0.0
        ));
        joints
    ]));
    let guard = [
        cloth(&mut scene, [0.12, 0.12, 0.14], 0.5),
        cloth(&mut scene, [0.55, 0.06, 0.05], 0.7),
        cloth(&mut scene, SKINS[0], 0.6),
    ];
    for (&mesh, material) in pieces.iter().zip(guard) {
        scene.objects.skinned_instances.push(SkinnedInstance::new(
            mesh,
            material,
            Mat4::from_translation(Vec3::new(-12.0, 0.0, 2.2)),
            0,
        ));
    }
    physical_light_under(&mut scene, Vec3::new(0.6, 0.45, 0.65));
    scene.lighting.light.cast_shadows = true;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 2.6, 8.0),
        target: Vec3::new(0.0, 1.4, -2.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// `crowd-lods` (module docs): [`crate::fx_scenes::lod_crowd`] down the
/// old town's avenue, seen from 7 m up as the fixture is.
pub fn festival_avenue() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::shadow_scenes::meadow(&mut scene, None);
    let cobbles = kit::surface(&mut scene, "cobbles");
    let avenue = scene.add_mesh(kit::slab(
        Vec2::new(-26.0, -200.0),
        Vec2::new(26.0, 20.0),
        0.01,
        2.4,
    ));
    scene.add_instance(avenue, cobbles, Mat4::IDENTITY);
    crate::fx_scenes::lod_crowd(&mut scene);
    // Townhouses facing the avenue down both sides, every 19 m.
    let house = kit::prop(&mut scene, "townhouse");
    for k in 0..9 {
        let z = -6.0 - k as f32 * 19.0;
        house.place(
            &mut scene,
            Vec3::new(-21.0, 0.0, z),
            std::f32::consts::FRAC_PI_2,
            16.0,
        );
        house.place(
            &mut scene,
            Vec3::new(21.0, 0.0, z),
            -std::f32::consts::FRAC_PI_2,
            16.0,
        );
    }
    let lamp = kit::prop(&mut scene, "streetlamp");
    for k in 0..12 {
        let z = -2.0 - k as f32 * 12.0;
        lamp.place(&mut scene, Vec3::new(-16.2, 0.0, z), 0.0, 4.2);
        lamp.place(&mut scene, Vec3::new(16.2, 0.0, z), 0.0, 4.2);
    }
    // The gate across the avenue at 60 m: two piers and a lintel of
    // dressed stone, a red banner on each pier.
    let stone = kit::surface_tinted(&mut scene, "stone", [0.92, 0.88, 0.8]);
    for (min, max) in [
        (Vec3::new(-16.0, 0.0, -61.0), Vec3::new(-7.0, 12.0, -59.0)),
        (Vec3::new(7.0, 0.0, -61.0), Vec3::new(16.0, 12.0, -59.0)),
        (Vec3::new(-16.0, 12.0, -61.0), Vec3::new(16.0, 15.0, -59.0)),
    ] {
        let block = scene.add_mesh(kit::block(min, max, 2.6));
        scene.add_instance(block, stone, Mat4::IDENTITY);
    }
    let cloth = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.5, 0.06, 0.05),
        roughness: 0.85,
        sheen: 0.6,
        sheenColor: Vec3::new(0.9, 0.5, 0.4),
        ..Default::default()
    });
    let banner = scene.add_mesh(shapes::cuboid(Vec3::new(2.4, 7.0, 0.06)));
    for x in [-11.5f32, 11.5] {
        scene.add_instance(
            banner,
            cloth,
            Mat4::from_translation(Vec3::new(x, 7.0, -58.95)),
        );
    }
    physical_light_under(&mut scene, Vec3::new(0.8, 0.45, 0.25));
    scene.lighting.light.cast_shadows = true;
    scene.frame.camera = Camera {
        position: Vec3::new(2.0, 7.0, 4.0),
        target: Vec3::new(0.0, 0.0, -60.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}
