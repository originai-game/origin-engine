//! R6's wind on placed models (og-render's `wind_shader`, og-runtime's
//! `og.Model.sway`): two 6 m columns standing on a floor, the red one's
//! material running the wind hook, the blue one still.

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, ProceduralSky, Projection, RenderScene,
    StandardMaterial, Wind, shapes,
};

/// Where the columns stand (their roots), and their height.
pub const SWAYING_AT: Vec3 = Vec3::new(-2.0, 0.0, 0.0);
pub const STILL_AT: Vec3 = Vec3::new(2.0, 0.0, 0.0);
pub const COLUMN: f32 = 6.0;

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

/// `wind-models` at `tick` (60 Hz): with `blowing` false the wind's
/// strength is 0 and both columns stand upright.
pub fn wind_scene(blowing: bool, tick: u64) -> RenderScene {
    let mut scene = RenderScene::new();
    let sun = Vec3::new(-0.3, 0.8, 0.5).normalize();
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        max_shadow_distance: 30.0,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            ..Default::default()
        },
        intensity: 0.8,
        background: Background::Color(Vec3::new(0.55, 0.7, 0.9)),
        ..Environment::default()
    };
    let floor = scene.add_mesh(shapes::cuboid(Vec3::new(16.0, 0.2, 10.0)));
    let grey = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.5),
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(
        floor,
        grey,
        Mat4::from_translation(Vec3::new(0.0, -0.1, -1.0)),
    );
    // A column with its root at its origin (the wind moves what is above),
    // stacked from 24 blocks so it has vertices all the way up to bend at
    // (a single box's eight corners would only shear it straight).
    const BLOCKS: usize = 24;
    let step = COLUMN / BLOCKS as f32;
    let mut column = og_render::MeshData {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    for k in 0..BLOCKS {
        let mut block = shapes::cuboid(Vec3::new(0.5, step, 0.5));
        let base = column.vertices.len() as u32;
        for v in &mut block.vertices {
            v.position[1] += step * (k as f32 + 0.5);
        }
        column.vertices.extend(block.vertices);
        column
            .indices
            .extend(block.indices.iter().map(|i| i + base));
    }
    let column = scene.add_mesh(column);
    let red = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.8, 0.08, 0.05),
        roughness: 0.8,
        ..Default::default()
    });
    let blue = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.05, 0.1, 0.8),
        roughness: 0.8,
        ..Default::default()
    });
    let shader = scene.add_shader(og_render::wind_shader());
    scene.set_material_shader(red, Some(shader));
    let w = Wind {
        strength: if blowing { wind().strength } else { 0.0 },
        ..wind()
    };
    let phase = og_render::wind_phases(&w, tick, 60.0);
    let k = std::f32::consts::TAU / og_render::WIND_WAVELENGTH;
    scene.set_shader_param(red, "wind", &[1.0, 0.0, w.strength, w.gust]);
    scene.set_shader_param(red, "wave", &[phase[0], phase[1], k, COLUMN]);
    scene.add_instance(column, red, Mat4::from_translation(SWAYING_AT));
    scene.add_instance(column, blue, Mat4::from_translation(STILL_AT));
    scene.frame.tick = tick;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 3.5, 14.0),
        target: Vec3::new(0.0, 3.0, 0.0),
        projection: Projection::Perspective {
            fov_y_degrees: 45.0,
        },
        ..Default::default()
    };
    scene
}
