//! Phase 3's culling scenes (perf only, no reference): a city of 40,000
//! buildings on a 200 x 200 grid, sixteen (mesh, material) batches, seen from
//! the street (most of it outside the view or behind the nearest blocks)
//! and from the air (most of it in view). The CPU-bound case of design §10:
//! many instances, few batches, a frame whose cost is the culling and the
//! submission rather than the GPU (`tests/evidence/P3/culling/`).

use glam::{Mat4, Vec3};
use og_render::{Camera, DirectionalLight, Exposure, Material, RenderScene, shapes};

/// Buildings a side of the grid.
pub const CITY_SIDE: u32 = 200;
/// Metres between building centres (the avenues run between them).
pub const CITY_SPACING: f32 = 8.0;

/// Buildings a side: [`CITY_SIDE`], or `OG_GFX_CITY_SIDE` (rounded up to an
/// even number, so the avenue stays between two columns) to time the
/// culling paths across instance counts
/// (`tests/evidence/P4/web-host-tools/culling-threshold.md`).
pub fn city_side() -> u32 {
    std::env::var("OG_GFX_CITY_SIDE")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|&n| n >= 2)
        .map_or(CITY_SIDE, |n| n.div_ceil(2) * 2)
}

fn hash(i: u32, salt: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9e37_79b9) ^ salt.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65535.0
}

/// The city with the camera at `eye` looking at `target`.
pub fn city(eye: Vec3, target: Vec3) -> RenderScene {
    let mut scene = RenderScene::new();
    let side = city_side();
    let extent = side as f32 * CITY_SPACING + 40.0;
    crate::scenes::ground(&mut scene, extent, 4.0, 0.9);
    let meshes = [
        scene.add_mesh(shapes::cuboid(Vec3::new(5.0, 1.0, 5.0))),
        scene.add_mesh(shapes::cuboid(Vec3::new(6.0, 1.0, 4.0))),
        scene.add_mesh(shapes::cuboid(Vec3::new(4.0, 1.0, 6.0))),
        scene.add_mesh(shapes::cuboid(Vec3::new(3.5, 1.0, 3.5))),
    ];
    let materials = [
        [0.72, 0.70, 0.66],
        [0.55, 0.42, 0.36],
        [0.40, 0.45, 0.52],
        [0.80, 0.78, 0.70],
    ]
    .map(|c| {
        scene.add_material(Material {
            base_color: [c[0], c[1], c[2], 1.0],
            roughness: 0.85,
            ..Default::default()
        })
    });
    let half = (side as f32 - 1.0) / 2.0;
    for i in 0..side * side {
        let (gx, gz) = ((i % side) as f32 - half, (i / side) as f32 - half);
        let height = 4.0 + hash(i, 1).powi(3) * 40.0;
        let mesh = meshes[(hash(i, 2) * 3.999) as usize];
        let material = materials[(hash(i, 3) * 3.999) as usize];
        // An even side: the columns sit at odd multiples of half a cell, so
        // the camera's avenue (x = 0) runs between two of them.
        let at = Vec3::new(gx * CITY_SPACING, height / 2.0, gz * CITY_SPACING);
        scene.add_instance(
            mesh,
            material,
            Mat4::from_translation(at) * Mat4::from_scale(Vec3::new(1.0, height, 1.0)),
        );
    }
    scene.camera = Camera {
        position: eye,
        target,
        near: 0.1,
        far: 2000.0,
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    let sun = Vec3::new(0.5, 0.7, 0.3);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 4.0,
        max_shadow_distance: 300.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene
}

/// At eye height in the avenue, looking down it.
pub fn city_street() -> RenderScene {
    city(Vec3::new(0.0, 1.7, 40.0), Vec3::new(0.0, 4.0, -400.0))
}

/// 300 m over the city's corner, looking across it.
pub fn city_aerial() -> RenderScene {
    let side = city_side() as f32 * CITY_SPACING / 2.0;
    city(
        Vec3::new(side + 120.0, 300.0, side + 120.0),
        Vec3::new(0.0, 0.0, 0.0),
    )
}

/// Walks the street camera 0.5 m a frame down the avenue (the occluders change).
pub fn walk(scene: &mut RenderScene, frame: u32) {
    let z = 40.0 - frame as f32 * 0.5;
    scene.camera.position.z = z;
    scene.camera.target.z = z - 440.0;
}
