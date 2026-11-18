//! Reflection probes beyond one room (docs/design.md §2.6; the debts of
//! depth S6, `the former test sample`):
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
        holes: None,
    });
    let layers = island_layers(&mut scene);
    scene.world.terrains.push(Terrain {
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
    scene.world.scatters.push(ScatterDraw {
        terrain: 0,
        mesh: tuft,
        material: grass,
        cast_shadows: true,
        scatter: og_render::TerrainScatter {
            distance: 15.0,
            ..grass_scatter()
        },
        impostor: None,
        impostor_distance: 0.0,
        with: None,
        lods: [None; 2],
    });
    let sun = Vec3::new(0.62, 0.45, 0.4);
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.93, 0.82),
        intensity: 3.2,
        max_shadow_distance: 40.0,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.9, 0.76),
            look: None,
        },
        intensity: 0.7,
        background: Background::Sky,
        ..Environment::default()
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
    scene.lighting.light = DirectionalLight {
        direction_to_light: Vec3::new(-0.3, 0.8, 0.5),
        intensity: 0.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.lighting.environment.intensity = 1.0;
    scene.frame.camera = Camera {
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
        scene.lighting.lights.push(Light {
            kind: LightKind::Point,
            position: at(0.0, 3.3, 1.0),
            color: Vec3::new(1.0, 0.95, 0.85),
            intensity: 25.0,
            range: 9.0,
            radius: 0.0,
            cast_shadows: false,
        });
        if probes {
            let mut p =
                ReflectionProbe::new(1 + k as u64, at(0.0, 1.6, 0.0), Vec3::new(6.0, 4.2, 6.0));
            p.mode = ProbeMode::Baked;
            p.fade = 0.3;
            scene.lighting.reflection_probes.push(p);
        }
    }
    scene
}

/// `probe-two-rooms` as a shot of a game: two open-fronted shops side by
/// side on the old town's street at noon, a dyer's hung in red cloth on
/// the left and an apothecary washed blue on the right, each with its
/// reflection probe and a silvered gazing ball on a plinth in [`ROOMS`]'
/// places; both balls are one mesh and one material. Each ball reflects
/// its own shop: a probe is picked per instance. [`two_rooms_scene`] is
/// the plain fixture `tests/probes.rs` reads.
pub fn two_shops_scene() -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let setts = kit::surface_tinted(&mut scene, "cobbles", [0.88, 0.86, 0.82]);
    let street = scene.add_mesh(kit::slab(
        Vec2::new(-30.0, 3.0),
        Vec2::new(30.0, 30.0),
        0.0,
        2.0,
    ));
    scene.add_instance(street, setts, Mat4::IDENTITY);
    let planks = kit::surface_tinted(&mut scene, "planks", [0.85, 0.75, 0.65]);
    let lime = kit::surface_tinted(&mut scene, "stone", [1.1, 1.04, 0.92]);
    let dyed = [
        kit::surface_tinted(&mut scene, "stone", [1.3, 0.25, 0.18]),
        kit::surface_tinted(&mut scene, "stone", [0.3, 0.5, 1.25]),
    ];
    let block = |s: &mut RenderScene, m, min: Vec3, max: Vec3| {
        let mesh = s.add_mesh(kit::block(min, max, 1.6));
        s.add_instance(mesh, m, Mat4::IDENTITY);
    };
    // The house front the shops open from: its storeys above them and
    // the piers between, to either side the neighbours' walls.
    block(
        &mut scene,
        lime,
        Vec3::new(-14.0, 4.2, 2.8),
        Vec3::new(14.0, 9.0, 3.2),
    );
    for x in [-7.0f32, 0.0, 7.0] {
        block(
            &mut scene,
            lime,
            Vec3::new(x - 0.6, 0.0, 2.8),
            Vec3::new(x + 0.6, 4.2, 3.4),
        );
    }
    block(
        &mut scene,
        lime,
        Vec3::new(-14.0, 0.0, 2.8),
        Vec3::new(-7.6, 4.2, 3.2),
    );
    block(
        &mut scene,
        lime,
        Vec3::new(7.6, 0.0, 2.8),
        Vec3::new(14.0, 4.2, 3.2),
    );
    let sphere = scene.add_mesh(shapes::sphere(48, 24));
    let chrome = scene.add_standard_material(mat(Vec3::splat(0.95), 1.0, 0.02));
    let crate_ = kit::prop(&mut scene, "crate");
    let barrel = kit::prop(&mut scene, "winebarrel");
    for (k, &x) in ROOMS.iter().enumerate() {
        let at = |dx: f32, y: f32, z: f32| Vec3::new(x + dx, y, z);
        let wall = dyed[k];
        block(&mut scene, planks, at(-3.0, -0.2, -3.0), at(3.0, 0.0, 3.0));
        block(&mut scene, planks, at(-3.1, 4.0, -3.1), at(3.1, 4.2, 3.0));
        block(&mut scene, wall, at(-3.0, 0.0, -3.1), at(3.0, 4.0, -2.9));
        block(&mut scene, wall, at(-3.1, 0.0, -3.0), at(-2.9, 4.0, 2.8));
        block(&mut scene, wall, at(2.9, 0.0, -3.0), at(3.1, 4.0, 2.8));
        // Shelves on the back wall, the wares under them.
        for y in [1.6f32, 2.5] {
            block(
                &mut scene,
                planks,
                at(-2.6, y, -2.9),
                at(2.6, y + 0.06, -2.5),
            );
        }
        crate_.place(&mut scene, at(-2.2, 0.0, -2.3), 0.15, 0.6);
        crate_.place(&mut scene, at(-2.15, 0.6, -2.35), -0.1, 0.55);
        barrel.place(&mut scene, at(2.2, 0.0, -2.2), 0.6 + k as f32, 1.0);
        barrel.place(&mut scene, at(2.3, 0.0, -1.1), 1.4, 1.0);
        // The plinth and the ball on it.
        block(
            &mut scene,
            lime,
            at(-0.45, 0.0, -0.45),
            at(0.45, SPHERE.1 - 0.5 * SPHERE.0, 0.45),
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
        scene.lighting.lights.push(Light {
            kind: LightKind::Point,
            position: at(0.0, 3.3, 1.0),
            color: Vec3::new(1.0, 0.85, 0.65),
            intensity: 25.0,
            range: 9.0,
            radius: 0.0,
            cast_shadows: false,
        });
        let mut p = ReflectionProbe::new(1 + k as u64, at(0.0, 1.6, 0.0), Vec3::new(6.0, 4.2, 6.0));
        p.mode = ProbeMode::Baked;
        p.fade = 0.3;
        scene.lighting.reflection_probes.push(p);
    }
    // A market barrow in the street.
    crate::lighting_scenes::wares(&mut scene, Vec3::new(-1.0, 0.0, 6.4), 0.1);
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(-0.3, 0.8, 0.5));
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.9, 10.5),
        target: Vec3::new(0.0, 1.6, 0.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// `probe-outdoor` as a shot of a game: a cottage garden on the same
/// meadow, a silvered gazing ball 1 m across on a plinth beside the
/// probe's capture point, five villagers striding past on the probe's +x side (skinned,
/// each at its own phase of the walk), a red stained-glass screen between
/// two columns on its -z side, pines and a ruined arch behind. The probe's
/// capture draws them all, so the ball reflects the villagers in their
/// pose, the meadow and grass, and the screen over what is behind it.
/// [`outdoor_scene`] is the plain fixture `tests/probes.rs` reads.
pub fn garden_scene() -> RenderScene {
    use crate::kit;
    let mut scene = meadow();
    let p = outdoor_probe();
    let ground = meadow_height;
    let ball = Vec3::new(p.x - 1.2, ground(p.x - 1.2, p.z) + 1.05, p.z);
    // The meadow runs on past the terrain into the hills.
    let grass = kit::surface_tinted(&mut scene, "grass", [0.86, 1.0, 0.72]);
    let h = MEADOW * 0.5 - 0.5;
    for (min, max) in [
        (Vec2::new(-260.0, -260.0), Vec2::new(-h, 260.0)),
        (Vec2::new(h, -260.0), Vec2::new(260.0, 260.0)),
        (Vec2::new(-h, -260.0), Vec2::new(h, -h)),
        (Vec2::new(-h, h), Vec2::new(h, 260.0)),
    ] {
        let m = scene.add_mesh(kit::slab(min, max, -0.05, 7.0));
        scene.add_instance(m, grass, Mat4::IDENTITY);
    }
    kit::hills(&mut scene, None);
    let stone = kit::surface_tinted(&mut scene, "stone", [0.95, 0.9, 0.84]);
    let plinth = scene.add_mesh(kit::block(
        Vec3::new(ball.x - 0.3, ground(ball.x, ball.z) - 0.2, ball.z - 0.3),
        Vec3::new(ball.x + 0.3, ball.y - 0.48, ball.z + 0.3),
        1.0,
    ));
    scene.add_instance(plinth, stone, Mat4::IDENTITY);
    let sphere = scene.add_mesh(shapes::sphere(48, 24));
    let chrome = scene.add_standard_material(mat(Vec3::splat(0.95), 1.0, 0.02));
    scene.add_instance(
        sphere,
        chrome,
        Mat4::from_scale_rotation_translation(Vec3::splat(1.0), Quat::IDENTITY, ball),
    );
    // The villagers on the path past the probe's +x side.
    let (mesh, skin) = crate::fx_scenes::walker_mesh(16, 4);
    let walker = scene.add_skinned_mesh(mesh, skin);
    for i in 0..5 {
        let z = p.z - 3.2 + i as f32 * 1.5;
        let x = p.x + 2.6 + (i as f32 * 1.7).sin() * 0.3;
        let cloth = scene.add_material(Material {
            base_color: [
                [0.55, 0.18, 0.12, 1.0],
                [0.2, 0.3, 0.45, 1.0],
                [0.45, 0.4, 0.25, 1.0],
                [0.3, 0.42, 0.22, 1.0],
                [0.5, 0.45, 0.4, 1.0],
            ][i],
            roughness: 0.8,
            ..Default::default()
        });
        scene.objects.joint_palettes.push(og_render::JointPalette {
            matrices: crate::fx_scenes::walker_palette(i as f32 * 0.21),
        });
        scene.objects.skinned_instances.push(SkinnedInstance::new(
            walker,
            cloth,
            Mat4::from_translation(Vec3::new(x, ground(x, z), z))
                * Mat4::from_rotation_y(std::f32::consts::PI),
            i as u32,
        ));
    }
    // The stained-glass screen between two columns on the -z side.
    let pane = scene.add_mesh(shapes::cuboid(Vec3::new(3.0, 2.4, 0.05)));
    let glass = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(1.0, 0.08, 0.05),
        emissive: Vec3::new(0.5, 0.03, 0.02),
        transparent: 1,
        opacity: 0.6,
        roughness: 0.2,
        ..Default::default()
    });
    let screen = Vec3::new(p.x, p.y + 0.2, p.z - 2.4);
    scene.add_instance(pane, glass, Mat4::from_translation(screen));
    let column = kit::prop(&mut scene, "column");
    for dx in [-1.75f32, 1.75] {
        let x = screen.x + dx;
        column.place(
            &mut scene,
            Vec3::new(x, ground(x, screen.z) - 0.1, screen.z),
            0.3,
            3.6,
        );
    }
    let lintel = scene.add_mesh(kit::block(
        Vec3::new(screen.x - 2.0, screen.y + 1.25, screen.z - 0.25),
        Vec3::new(screen.x + 2.0, screen.y + 1.55, screen.z + 0.25),
        1.0,
    ));
    scene.add_instance(lintel, stone, Mat4::IDENTITY);
    // Pines along the garden's back, an arch and boulders.
    let pine = kit::prop(&mut scene, "pine");
    for (x, z, hgt) in [
        (-9.0f32, -12.0f32, 11.0f32),
        (-4.0, -15.0, 13.0),
        (3.0, -13.0, 10.0),
        (8.0, -16.0, 12.0),
        (13.0, -10.0, 11.0),
        (-14.0, -6.0, 12.0),
    ] {
        pine.place(
            &mut scene,
            Vec3::new(x, ground(x, z) - 0.1, z),
            x * 0.3,
            hgt,
        );
    }
    kit::prop(&mut scene, "arch").place(
        &mut scene,
        Vec3::new(-7.0, ground(-7.0, -6.0) - 0.1, -6.0),
        0.5,
        5.0,
    );
    let boulder = kit::prop(&mut scene, "boulder");
    boulder.place(
        &mut scene,
        Vec3::new(-3.4, ground(-3.4, 1.2) - 0.2, 1.2),
        1.0,
        0.7,
    );
    boulder.place(
        &mut scene,
        Vec3::new(4.8, ground(4.8, -3.0) - 0.2, -3.0),
        2.3,
        0.9,
    );
    scene.frame.camera = Camera {
        position: Vec3::new(p.x - 1.4, ground(p.x - 1.4, p.z + 4.6) + 1.9, p.z + 4.6),
        target: Vec3::new(p.x + 0.5, ground(p.x + 0.5, p.z - 0.8) + 1.1, p.z - 0.8),
        far: 2500.0,
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    let mut r = ReflectionProbe::new(7, p, Vec3::new(8.0, 5.0, 8.0));
    r.mode = ProbeMode::Baked;
    r.fade = 0.5;
    scene.lighting.reflection_probes.push(r);
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
    scene.frame.camera = Camera {
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
                .objects
                .joint_palettes
                .push(bend_palette(-0.3 + i as f32 * 0.15, i as f32 * 0.3));
            scene.objects.skinned_instances.push(SkinnedInstance::new(
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
        r.fade = 0.5;
        scene.lighting.reflection_probes.push(r);
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
    scene.objects.instances.remove(0);
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
                holes: None,
            });
            scene.world.terrains.push(Terrain {
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
    scene.objects.joint_palettes.push(og_render::JointPalette {
        matrices: vec![joint; crate::fx_scenes::CYLINDER_BONES],
    });
    scene.objects.skinned_instances.push(SkinnedInstance::new(
        cylinder,
        chrome,
        transform,
        scene.objects.joint_palettes.len() as u32 - 1,
    ));
    scene
}

/// `probe-outdoor` without its figures and pane, and with (`on`) what the
/// main pass draws over the opaque scene around the probe: a red world
/// sprite facing it on its -z side, a yellow line loop on its +x side and a
/// green particle fountain on its -x side.
pub fn probe_effects_scene(on: bool) -> RenderScene {
    let mut scene = outdoor_scene(Outdoor {
        content: false,
        probe: Some(1.0),
    });
    let p = outdoor_probe();
    if on {
        scene.add_sprite(og_render::Sprite {
            position: p + Vec3::new(0.0, 0.0, -2.5),
            size: Vec2::new(2.4, 2.4),
            color: [1.0, 0.08, 0.06, 1.0],
            ..Default::default()
        });
        let c = p + Vec3::new(2.5, 0.0, 0.0);
        scene.effects.lines.push(og_render::Lines {
            points: vec![
                c + Vec3::new(0.0, -0.9, -0.9),
                c + Vec3::new(0.0, -0.9, 0.9),
                c + Vec3::new(0.0, 0.9, 0.9),
                c + Vec3::new(0.0, 0.9, -0.9),
            ],
            color: [1.0, 0.9, 0.05, 1.0],
            mode: og_render::LineMode::Loop,
            width: 0.35,
            world_width: true,
            ..Default::default()
        });
        let e = scene.add_emitter(og_render::ParticleEmitter {
            rate: 200.0,
            lifetime: [1.4, 1.6],
            speed: [0.4, 0.6],
            size: [1.2, 1.5],
            gravity: Vec3::ZERO,
            color: [0.1, 0.95, 0.15, 1.0],
            blend: og_render::ParticleBlend::Alpha,
            ..Default::default()
        });
        scene.effects.particles.push(og_render::ParticleSystem::new(
            11,
            e,
            Mat4::from_translation(p + Vec3::new(-2.5, -0.6, 0.0)),
            0x7a11,
            0,
        ));
    }
    crate::fx_scenes::tick_all(&mut scene, 60);
    scene
}

/// `probe-outdoor` without its figures and pane, and with (`on`) a pond
/// of red-tinted water 0.4 m under the probe, 6 m square: its -y face sees
/// the water where it saw the meadow.
pub fn probe_water_scene(on: bool) -> RenderScene {
    let mut scene = outdoor_scene(Outdoor {
        content: false,
        probe: Some(1.0),
    });
    if on {
        let p = outdoor_probe();
        scene.world.water.push(og_render::Water {
            transform: Mat4::from_translation(p - Vec3::Y * 0.4),
            size: Vec2::new(6.0, 6.0),
            shallow: Vec3::new(0.9, 0.2, 0.15),
            deep: Vec3::new(0.5, 0.04, 0.03),
            ..Default::default()
        });
    }
    scene
}

/// The probe of [`probe_floor_scene`]: its id and capture point, 1.2 m
/// over a yellow floor.
pub const FLOOR_PROBE: (u64, Vec3) = (7, Vec3::new(0.0, 1.2, 0.0));

/// A yellow floor 10 m square (its top at y 0) under a high sun and a dark
/// grey background, with a baked probe at [`FLOOR_PROBE`]: its -y face sees
/// the floor. The scenes of the probe faces' depth (`tests/probes.rs`) put
/// water, soft particles or a 3D fluid between the probe and the floor.
pub fn probe_floor_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    scene.lighting.light = DirectionalLight {
        direction_to_light: Vec3::new(0.3, 1.0, 0.2),
        intensity: 3.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.lighting.environment.intensity = 0.6;
    scene.lighting.environment.background = Background::Color(Vec3::splat(0.05));
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 3.0, 6.0),
        target: Vec3::ZERO,
        ..Default::default()
    };
    boxed(
        &mut scene,
        mat(Vec3::new(0.9, 0.75, 0.05), 0.0, 0.9),
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(10.0, 0.2, 10.0),
    );
    let (id, at) = FLOOR_PROBE;
    scene
        .lighting
        .reflection_probes
        .push(ReflectionProbe::new(id, at, Vec3::new(10.0, 4.0, 10.0)));
    scene
}

/// Clear water 0.15 m deep over [`probe_floor_scene`]'s floor, 6 m square,
/// its deep colour a dark blue and no foam: seen through, it shows the
/// floor's yellow; with no depth below, its deep colour.
pub fn add_floor_water(scene: &mut RenderScene) {
    scene.world.water.push(og_render::Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 0.15, 0.0)),
        size: Vec2::new(6.0, 6.0),
        shallow: Vec3::ONE,
        deep: Vec3::new(0.02, 0.05, 0.3),
        foam_width: 0.0,
        ..Default::default()
    });
}

/// A glowing blue wall 4 m to the +x side of [`probe_floor_scene`]'s
/// probe, 3 m tall, beyond [`add_floor_water`]'s far edge: the probe's +x
/// face sees it at the horizon and, in screen space, mirrored in the water
/// below it.
pub fn add_floor_wall(scene: &mut RenderScene) {
    boxed(
        scene,
        StandardMaterial {
            color: Vec3::new(0.02, 0.05, 0.4),
            emissive: Vec3::new(0.1, 0.25, 3.0),
            ..Default::default()
        },
        Vec3::new(4.0, 1.5, 0.0),
        Vec3::new(0.2, 3.0, 8.0),
    );
}

/// A few green particles 1.2 m wide hovering 0.1 to 0.2 m over
/// [`probe_floor_scene`]'s floor, under the probe; `soft`: they fade over
/// 2 m of the depth behind them, so against the floor they nearly vanish.
pub fn add_floor_particles(scene: &mut RenderScene, soft: bool) {
    let e = scene.add_emitter(og_render::ParticleEmitter {
        shape: og_render::EmitterShape::Cone {
            angle_degrees: 0.0,
            radius: 0.6,
        },
        rate: 6.0,
        lifetime: [1.8, 2.0],
        speed: [0.1, 0.1],
        size: [1.2, 1.2],
        gravity: Vec3::ZERO,
        color: [0.1, 0.95, 0.15, 1.0],
        blend: og_render::ParticleBlend::Alpha,
        soft_distance: if soft { 2.0 } else { 0.0 },
        ..Default::default()
    });
    scene.effects.particles.push(og_render::ParticleSystem::new(
        12,
        e,
        Mat4::from_translation(Vec3::new(0.0, 0.1, 0.0)),
        0x50f7,
        0,
    ));
    crate::fx_scenes::tick_all(scene, 60);
}

/// A red 3D fluid ball 0.7 m in radius 1.5 m to the +x side of
/// [`probe_floor_scene`]'s probe, at its height: its +x face sees the
/// fluid where it saw the dark background and the floor.
pub fn add_floor_fluid(scene: &mut RenderScene) {
    let spacing = 0.1;
    let centre = FLOOR_PROBE.1 + Vec3::new(1.5, 0.0, 0.0);
    let n = (0.7 / spacing) as i32;
    let particles: Vec<Vec3> = (-n..=n)
        .flat_map(|x| {
            (-n..=n).flat_map(move |y| {
                (-n..=n).map(move |z| Vec3::new(x as f32, y as f32, z as f32) * spacing)
            })
        })
        .filter(|p| p.length() <= 0.7)
        .map(|p| centre + p)
        .collect();
    scene.world.fluids_3d.push(og_render::Fluid3d {
        particles: particles.into(),
        color: Vec3::new(0.9, 0.08, 0.05),
        opacity: 0.9,
        radius: 0.12,
        volume: spacing * spacing * spacing,
        ..Default::default()
    });
}
