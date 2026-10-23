//! Reflection probes beyond one room (docs/design.md §2.6; the debts of
//! depth S6, `tests/evidence/debt/render-lod`):
//!
//! - `probe-two-rooms`: two open-fronted rooms side by side, the left one
//!   red, the right one blue, each with its reflection probe and a chrome
//!   sphere; both spheres are one mesh and one material. Each sphere
//!   reflects its own room: a probe is picked per instance, so the shared
//!   batch is split by room (`og_render`'s `probe_select.rs`). Picked per
//!   batch, by the centre of both spheres' bounds (between the rooms, in
//!   neither box), both would reflect the sky.
//! - `probe-outdoor`: a small rolling meadow (a 48 m terrain with the
//!   island's layers and grass scattered over it), a probe
//!   over it with a chrome sphere beside its capture point, five skinned cylinders posed to the
//!   probe's +x side, a translucent red pane to its -z side. The probe's
//!   capture draws all of them (the skinned ones in their pose, the
//!   terrain and grass below, the pane over what is behind it), so the
//!   sphere reflects the figures, the meadow and the pane; and the terrain
//!   and grass inside the box are lit by the probe, blended with the
//!   environment towards its faces.
//! - SSR reflectivity (no reference frames; `tests/probes.rs` reads the
//!   frames): S6's `ssr-wet-floor` with the floor's roughness given by its
//!   map (factor 1, map 0.04) or by a terrain layer of roughness 0.05. The
//!   prepass writes the roughness the lit pass shades, so both floors
//!   mirror the sphere; from the factor alone, or nothing for terrain, they
//!   would reflect nothing.

use crate::fx_scenes::{bend_palette, skinned_cylinder};
use crate::terrain_scenes::{grass_scatter, grass_tuft, island_layers};
use glam::{Mat4, Quat, Vec2, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Exposure, Light, LightKind, Material,
    ProbeMode, ProceduralSky, ReflectionProbe, RenderScene, ScatterDraw, SkinnedInstance,
    StandardMaterial, Terrain, TerrainControl, TerrainData, shapes,
};
use std::sync::Arc;

/// The meadow's side (m) and its samples per side.
const MEADOW: f32 = 48.0;
const MEADOW_SAMPLES: u32 = 97;

/// The meadow's height at (x, z): gentle rolls under a metre.
pub fn meadow_height(x: f32, z: f32) -> f32 {
    0.35 * (x * 0.23).sin() * (z * 0.17).cos() + 0.2 * (x * 0.11 + z * 0.07).sin()
}

/// The meadow: its terrain (the island's layers, all grass) and the
/// island's grass to 15 m, under a low sun.
fn meadow() -> RenderScene {
    let mut scene = RenderScene::new();
    let n = MEADOW_SAMPLES;
    let cell = MEADOW / (n - 1) as f32;
    let mut heights = Vec::with_capacity((n * n) as usize);
    for r in 0..n {
        for c in 0..n {
            let (x, z) = (
                -MEADOW * 0.5 + c as f32 * cell,
                -MEADOW * 0.5 + r as f32 * cell,
            );
            heights.push(meadow_height(x, z));
        }
    }
    let handle = scene.add_terrain(TerrainData {
        size: Vec2::splat(MEADOW),
        columns: n,
        rows: n,
        heights: Arc::from(heights),
        // All grass (red): the island's control map elsewhere paints sand.
        control: Some(TerrainControl {
            width: 2,
            height: 2,
            rgba: Arc::from([255u8, 0, 0, 0].repeat(4)),
        }),
    });
    let layers = island_layers(&mut scene);
    scene.terrains.push(Terrain {
        layers,
        height_blend: 0.6,
        ..Terrain::new(handle)
    });
    let tuft = scene.add_mesh(grass_tuft());
    let grass = scene.add_material(Material {
        base_color: [0.2, 0.33, 0.08, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    scene.scatters.push(ScatterDraw {
        terrain: 0,
        mesh: tuft,
        material: grass,
        scatter: og_render::TerrainScatter {
            distance: 15.0,
            ..grass_scatter()
        },
    });
    let sun = Vec3::new(0.62, 0.45, 0.4);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.93, 0.82),
        intensity: 3.2,
        max_shadow_distance: 40.0,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.9, 0.76),
            look: None,
        },
        intensity: 0.7,
        background: Background::Sky,
    };
    scene
}

fn mat(color: Vec3, metalness: f32, roughness: f32) -> StandardMaterial {
    StandardMaterial {
        color,
        metalness,
        roughness,
        ..Default::default()
    }
}

fn boxed(scene: &mut RenderScene, m: StandardMaterial, at: Vec3, size: Vec3) {
    let mesh = scene.add_mesh(shapes::cuboid(size));
    let h = scene.add_standard_material(m);
    scene.add_instance(mesh, h, Mat4::from_translation(at));
}

/// The two rooms' centres along x (each 6 × 4 × 6 m, open towards +z).
pub const ROOMS: [f32; 2] = [-3.6, 3.6];
/// The chrome spheres' radius and height.
pub const SPHERE: (f32, f32) = (1.2, 1.3);

/// `probe-two-rooms` (module docs); `probes` false leaves both probes out.
pub fn two_rooms_scene(probes: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(-0.3, 0.8, 0.5),
        intensity: 0.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.environment.intensity = 1.0;
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.9, 8.5),
        target: Vec3::new(0.0, 1.4, 0.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    let sphere = scene.add_mesh(shapes::sphere(48, 24));
    let chrome = scene.add_standard_material(mat(Vec3::splat(0.95), 1.0, 0.02));
    for (k, &x) in ROOMS.iter().enumerate() {
        let wall = if k == 0 {
            mat(Vec3::new(0.8, 0.12, 0.08), 0.0, 0.8)
        } else {
            mat(Vec3::new(0.1, 0.25, 0.85), 0.0, 0.8)
        };
        let at = |dx: f32, y: f32, z: f32| Vec3::new(x + dx, y, z);
        boxed(
            &mut scene,
            wall,
            at(0.0, 2.0, -3.0),
            Vec3::new(6.0, 4.0, 0.2),
        );
        boxed(
            &mut scene,
            wall,
            at(-3.0, 2.0, 0.0),
            Vec3::new(0.2, 4.0, 6.0),
        );
        boxed(
            &mut scene,
            wall,
            at(3.0, 2.0, 0.0),
            Vec3::new(0.2, 4.0, 6.0),
        );
        boxed(
            &mut scene,
            wall,
            at(0.0, 4.1, 0.0),
            Vec3::new(6.2, 0.2, 6.2),
        );
        boxed(
            &mut scene,
            mat(Vec3::splat(0.3), 0.0, 0.9),
            at(0.0, -0.1, 0.0),
            Vec3::new(6.0, 0.2, 6.0),
        );
        scene.add_instance(
            sphere,
            chrome,
            Mat4::from_scale_rotation_translation(
                Vec3::splat(SPHERE.0),
                Quat::IDENTITY,
                at(0.0, SPHERE.1, 0.0),
            ),
        );
        scene.lights.push(Light {
            kind: LightKind::Point,
            position: at(0.0, 3.3, 1.0),
            color: Vec3::new(1.0, 0.95, 0.85),
            intensity: 25.0,
            range: 9.0,
            cast_shadows: false,
        });
        if probes {
            let mut p =
                ReflectionProbe::new(1 + k as u64, at(0.0, 1.6, 0.0), Vec3::new(6.0, 4.2, 6.0));
            p.mode = ProbeMode::Baked;
            p.blend_distance = 0.3;
            scene.reflection_probes.push(p);
        }
    }
    scene
}

/// What `probe-outdoor` puts in its probe's box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outdoor {
    /// The skinned cylinders and the red pane.
    pub content: bool,
    /// The probe (`None`: no probe), and its intensity.
    pub probe: Option<f32>,
}

/// The probe's position in `probe-outdoor`: 1.6 m over the meadow's middle.
pub fn outdoor_probe() -> Vec3 {
    Vec3::new(0.0, meadow_height(0.0, 0.0) + 1.6, 0.0)
}

/// `probe-outdoor` (module docs).
pub fn outdoor_scene(o: Outdoor) -> RenderScene {
    let mut scene = meadow();
    let p = outdoor_probe();
    let ground = meadow_height;
    // Beside the capture point, so the capture's faces see past it.
    let ball = p + Vec3::new(-1.2, -0.8, 0.0);
    scene.camera = Camera {
        position: Vec3::new(p.x + 1.5, ground(p.x + 1.5, p.z + 5.0) + 1.9, p.z + 5.0),
        target: ball,
        far: 200.0,
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    let sphere = scene.add_mesh(shapes::sphere(48, 24));
    let chrome = scene.add_standard_material(mat(Vec3::splat(0.95), 1.0, 0.02));
    scene.add_instance(
        sphere,
        chrome,
        Mat4::from_scale_rotation_translation(Vec3::splat(0.6), Quat::IDENTITY, ball),
    );
    if o.content {
        let (mesh, skin) = skinned_cylinder(0.22, 24, 24);
        let cylinder = scene.add_skinned_mesh(mesh, skin);
        let flesh = scene.add_material(Material {
            base_color: [0.85, 0.15, 0.75, 1.0],
            roughness: 0.45,
            emissive: [0.4, 0.05, 0.35],
            ..Default::default()
        });
        for i in 0..5 {
            let z = p.z - 1.6 + i as f32 * 0.8;
            let x = p.x + 2.6;
            scene
                .joint_palettes
                .push(bend_palette(-0.3 + i as f32 * 0.15, i as f32 * 0.3));
            scene.skinned_instances.push(SkinnedInstance::new(
                cylinder,
                flesh,
                Mat4::from_translation(Vec3::new(x, ground(x, z) - 0.1, z)),
                i,
            ));
        }
        let pane = scene.add_mesh(shapes::cuboid(Vec3::new(3.0, 2.4, 0.05)));
        let glass = scene.add_standard_material(StandardMaterial {
            color: Vec3::new(1.0, 0.05, 0.05),
            emissive: Vec3::new(0.6, 0.02, 0.02),
            transparent: 1,
            opacity: 0.6,
            roughness: 0.3,
            ..Default::default()
        });
        scene.add_instance(
            pane,
            glass,
            Mat4::from_translation(Vec3::new(p.x, p.y + 0.2, p.z - 2.4)),
        );
    }
    if let Some(intensity) = o.probe {
        let mut r = ReflectionProbe::new(7, p, Vec3::new(8.0, 5.0, 8.0));
        r.mode = ProbeMode::Baked;
        r.intensity = intensity;
        r.blend_distance = 0.5;
        scene.reflection_probes.push(r);
    }
    scene
}

/// Which floor [`ssr_floor_scene`] draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SsrFloor {
    /// A box of roughness factor 1, its roughness map 0.04 (`true`) or none.
    Mapped(bool),
    /// A flat terrain of one dark layer of roughness 0.05.
    Terrain,
}

/// S6's `ssr-wet-floor` (`depth_render_scenes::ssr_scene`, SSR on) with
/// `floor` in place of its floor.
pub fn ssr_floor_scene(floor: SsrFloor, ssr: bool) -> RenderScene {
    let mut scene = crate::depth_render_scenes::ssr_scene(ssr);
    // The old floor is the first instance.
    scene.instances.remove(0);
    match floor {
        SsrFloor::Mapped(map) => {
            let texture = map.then(|| {
                // glTF's layout: roughness in green (0.04), metalness in blue (0).
                scene.add_texture(crate::scenes::texture(
                    4,
                    og_render::ColorSpace::Linear,
                    |_, _| [0, 10, 0, 255],
                ))
            });
            let m = scene.add_material(Material {
                base_color: [0.04, 0.04, 0.04, 1.0],
                metallic: 0.0,
                roughness: 1.0,
                metallic_roughness_texture: texture,
                ..Default::default()
            });
            let mesh = scene.add_mesh(shapes::cuboid(Vec3::new(14.0, 0.2, 14.0)));
            scene.add_instance(mesh, m, Mat4::from_translation(Vec3::new(0.0, -0.1, 0.0)));
        }
        SsrFloor::Terrain => {
            let handle = scene.add_terrain(TerrainData {
                size: Vec2::splat(14.0),
                columns: 15,
                rows: 15,
                heights: Arc::from(vec![0.0f32; 15 * 15]),
                control: None,
            });
            scene.terrains.push(Terrain {
                layers: vec![og_render::TerrainLayer {
                    color: Vec3::splat(0.04),
                    roughness: 0.05,
                    ..Default::default()
                }],
                ..Terrain::new(handle)
            });
        }
    }
    scene
}

/// `probe-two-rooms` with a chrome skinned cylinder whose instance stands
/// in the left (red) room and whose palette moves every joint
/// [`ROOMS`]'s span along +x, so its pose stands in the right (blue)
/// room, left of that room's sphere; `posed` false places the same
/// cylinder there by its transform instead, in its bind pose.
pub fn posed_figure_scene(posed: bool) -> RenderScene {
    let mut scene = two_rooms_scene(true);
    let (mesh, skin) = skinned_cylinder(0.35, 24, 32);
    let cylinder = scene.add_skinned_mesh(mesh, skin);
    let chrome = scene.add_material(Material {
        base_color: [0.95, 0.95, 0.95, 1.0],
        metallic: 1.0,
        roughness: 0.05,
        ..Default::default()
    });
    let span = ROOMS[1] - ROOMS[0];
    let shift = Mat4::from_translation(Vec3::X * span);
    let (transform, joint) = if posed {
        (
            Mat4::from_translation(Vec3::new(ROOMS[0] - 1.7, 0.0, 1.0)),
            shift,
        )
    } else {
        (
            Mat4::from_translation(Vec3::new(ROOMS[1] - 1.7, 0.0, 1.0)),
            Mat4::IDENTITY,
        )
    };
    scene.joint_palettes.push(og_render::JointPalette {
        matrices: vec![joint; crate::fx_scenes::CYLINDER_BONES],
    });
    scene.skinned_instances.push(SkinnedInstance::new(
        cylinder,
        chrome,
        transform,
        scene.joint_palettes.len() as u32 - 1,
    ));
    scene
}
