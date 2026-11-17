//! R6's wind on placed models (og-render's `wind_shader`, og-runtime's
//! `og.Model.sway`): pines on a hill in a gale, their materials running
//! the wind hook, beside the stones of a ruin, which stand still.

use crate::kit;
use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Grass, GrassDraw, MaterialHandle,
    MeshHandle, ProceduralSky, Projection, RenderScene, Terrain, TerrainData, TerrainLayer, Wind,
};

/// Where the first swaying pine and the still column stand (their roots),
/// and the pine's height.
pub const SWAYING_AT: Vec3 = Vec3::new(-5.0, 0.0, 0.0);
pub const STILL_AT: Vec3 = Vec3::new(6.5, 0.0, 2.0);
pub const PINE: f32 = 12.0;
/// The pines' `og.Model.sway`: how much of og.Wind's lean they take.
pub const SWAY: f32 = 1.0;

/// The wind of `wind-models`: along +x, a strong lean in gusts.
pub fn wind() -> Wind {
    Wind {
        strength: 0.15,
        direction: Vec3::X,
        speed: 3.0,
        gust: 0.5,
        ..Wind::default()
    }
}

/// The hilltop: a gentle swell over a rise of 3 m that is level again
/// 130 m out, where the kit's hills take over (m).
fn hill(x: f32, z: f32) -> f32 {
    let rise = 1.0 - crate::fx_scenes::smooth(0.0, 130.0, (x * x + z * z).sqrt());
    0.5 * (x * 0.06 + 0.4).sin() * (z * 0.05).cos() + 3.0 * rise
}

/// The kit's pine `height` metres tall with its root at its origin, the
/// size in its vertices: the wind hook leans a model by its own units
/// (`og.Wind.strength` a metre of height), so a model that sways is one
/// made in metres. Its material is a copy running the wind shader, its
/// parameters `w`'s at `tick` and the pine's top.
fn swaying_pine(
    scene: &mut RenderScene,
    height: f32,
    w: &Wind,
    tick: u64,
) -> (MeshHandle, MaterialHandle) {
    let pine = kit::prop(scene, "pine");
    let (mesh, material) = pine.rooted(scene, height);
    let look = scene.material(material).expect("the pine's bark").clone();
    let material = scene.add_material(look);
    let shader = scene.add_shader(og_render::wind_shader());
    scene.set_material_shader(material, Some(shader));
    let phase = og_render::wind_phases(w, tick, 60.0);
    let k = std::f32::consts::TAU / og_render::WIND_WAVELENGTH;
    scene.set_shader_param(material, "wind", &[1.0, 0.0, w.strength * SWAY, w.gust]);
    scene.set_shader_param(material, "wave", &[phase[0], phase[1], k, height]);
    (mesh, material)
}

/// `wind-models` at `tick` (60 Hz): a hilltop meadow in [`wind`], three
/// pines leaning downwind (+x, to the right) under og.Wind's hook, their
/// tops about two metres over, the grass (og.Grass) leaning with them, and a
/// ruin's column, arch and stones upright among them; the pines' shadows
/// lean with them. With `blowing` false the wind's strength is 0 and all
/// stand upright. Instance 0 is the still column, instance 1 the first
/// swaying pine (`tests/terrain/wind_models.rs` reads its material).
pub fn wind_scene(blowing: bool, tick: u64) -> RenderScene {
    let mut scene = RenderScene::new();
    let sun = Vec3::new(-0.45, 0.5, 0.62).normalize();
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.93, 0.82),
        intensity: 3.2,
        max_shadow_distance: 90.0,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            ..Default::default()
        },
        intensity: 0.8,
        background: Background::Sky,
        ..Environment::default()
    };
    // Out past where the kit's hills leave the ground.
    let (n, size) = (257u32, 512.0f32);
    let cell = size / (n - 1) as f32;
    let heights: Vec<f32> = (0..n * n)
        .map(|i| {
            let (c, r) = ((i % n) as f32, (i / n) as f32);
            hill(c * cell - size * 0.5, r * cell - size * 0.5)
        })
        .collect();
    let terrain = scene.add_terrain(TerrainData {
        heights: heights.into(),
        ..TerrainData::flat(glam::Vec2::splat(size), n, n)
    });
    let (grass, grass_normal) = kit::layer_maps(&mut scene, "grass");
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            albedo: Some(grass),
            normal: Some(grass_normal),
            color: Vec3::new(0.72, 1.05, 0.5),
            roughness: 0.95,
            tile: 3.5,
            anti_tiling: 0.5,
            macro_variation: 0.5,
            ..Default::default()
        }],
        ..Terrain::new(terrain)
    });
    scene.world.grass.push(GrassDraw {
        terrain: 0,
        grass: Grass {
            cast_shadows: true,
            ..Default::default()
        },
    });
    let w = Wind {
        strength: if blowing { wind().strength } else { 0.0 },
        ..wind()
    };
    scene.atmosphere.wind = blowing.then_some(w);
    let ground = |at: Vec3| Vec3::new(at.x, hill(at.x, at.z), at.z);

    // The still column first, then the pines (the order the test reads).
    let column = kit::prop(&mut scene, "column");
    assert_eq!(column.parts().len(), 1, "the column is one part");
    column.place(&mut scene, ground(STILL_AT), 0.4, 6.0);
    let (tall, tall_bark) = swaying_pine(&mut scene, PINE, &w, tick);
    scene.add_instance(tall, tall_bark, Mat4::from_translation(ground(SWAYING_AT)));
    let (short, short_bark) = swaying_pine(&mut scene, 8.5, &w, tick);
    for (at, yaw, mesh, bark) in [
        (Vec3::new(-11.5, 0.0, -6.0), 2.1f32, short, short_bark),
        (Vec3::new(1.5, 0.0, -11.0), 4.0, tall, tall_bark),
        (Vec3::new(-19.0, 0.0, -16.0), 0.9, tall, tall_bark),
        (Vec3::new(15.0, 0.0, -21.0), 5.2, short, short_bark),
        (Vec3::new(-28.0, 0.0, -30.0), 3.0, short, short_bark),
        (Vec3::new(26.0, 0.0, -34.0), 1.4, tall, tall_bark),
    ] {
        scene.add_instance(
            mesh,
            bark,
            Mat4::from_translation(ground(at)) * Mat4::from_rotation_y(yaw),
        );
    }
    // What is left of the ruin: stones the wind does not move.
    let arch = kit::prop(&mut scene, "arch");
    arch.place(&mut scene, ground(Vec3::new(11.5, 0.0, -5.0)), 0.5, 7.0);
    let broken = kit::prop(&mut scene, "column_broken");
    broken.place(&mut scene, ground(Vec3::new(9.5, 0.0, 4.5)), 2.6, 2.2);
    let boulder = kit::prop(&mut scene, "boulder");
    for (x, z, height, yaw) in [
        (-8.5f32, 3.5f32, 1.3f32, 0.7f32),
        (2.5, 2.0, 0.8, 3.9),
        (-2.0, -5.5, 1.7, 5.0),
    ] {
        let foot = ground(Vec3::new(x, 0.0, z)) - Vec3::Y * 0.25 * height;
        boulder.place(&mut scene, foot, yaw, height);
    }
    kit::hills(&mut scene, None);
    scene.frame.tick = tick;
    let eye = ground(Vec3::new(1.0, 0.0, 21.0));
    scene.frame.camera = Camera {
        position: eye + Vec3::Y * 1.7,
        target: Vec3::new(0.5, 5.6, 0.0),
        projection: Projection::Perspective {
            fov_y_degrees: 45.0,
        },
        ..Default::default()
    };
    scene
}
