//! R6's grass (og-render's `grass.rs`, og.Grass): the island's meadow
//! with a default field (a zero-value og.Grass is a meadow) seen near, from
//! the middle distance, out to the horizon, against the sun, and in wind.

use crate::terrain_scenes::{camera_at, island_height, island_view};
use glam::Vec3;
use og_render::{Background, DirectionalLight, Grass, GrassDraw, RenderScene, Wind};

/// Towards the sun of `grass-backlit`: west, 12 degrees up.
pub const BACKLIT_SUN: Vec3 = Vec3::new(-0.978, 0.208, 0.0);

/// Whether the sun towards `sun` reaches island point `p` (no terrain on
/// the way, marched 2 m at a time out to 3 km).
pub fn sun_reaches(p: Vec3, sun: Vec3) -> bool {
    let sun = sun.normalize();
    (0..1500).all(|i| {
        let q = p + sun * (0.5 + 2.0 * i as f32);
        q.y >= island_height(q.x, q.z)
    })
}

/// The wind of `grass-wind`: along +x and a little +z, gusting.
pub fn wind() -> Wind {
    Wind {
        strength: 0.6,
        direction: Vec3::new(1.0, 0.0, 0.3),
        speed: 4.0,
        gust: 0.7,
    }
}

/// The island's meadow with a default og.Grass over layer 0 (the grass),
/// casting into the first cascade, seen from `view`:
/// - `near`: at eye height on the meadow, the boulders ahead;
/// - `mid`: 12 m up, across 100 m of meadow;
/// - `far`: 40 m up towards the mountains, the meadow running to them;
/// - `backlit`: at eye height looking into a low sun.
pub fn grass_scene(view: &str) -> RenderScene {
    let mut scene = island_view("near");
    let ground = |x: f32, z: f32| island_height(x, z);
    let (x, z) = (-80.0, 150.0);
    match view {
        "mid" => {
            scene.camera = camera_at(
                Vec3::new(x, ground(x, z) + 12.0, z),
                Vec3::new(x - 30.0, ground(x - 30.0, z - 90.0) + 2.0, z - 90.0),
            );
        }
        "far" => {
            scene.camera = camera_at(
                Vec3::new(x, ground(x, z) + 40.0, z + 40.0),
                Vec3::new(-110.0, ground(-110.0, 60.0) + 10.0, 60.0),
            );
        }
        "backlit" => {
            // A low sun ahead of the camera, west over the meadow where no
            // ridge stands in its way (`sun_reaches`): the blades are lit
            // from behind.
            let sun = BACKLIT_SUN.normalize();
            // Late afternoon: a warm sun, its disc and glow drawn per pixel.
            let warm = Vec3::new(1.0, 0.82, 0.58);
            scene.light = DirectionalLight {
                direction_to_light: sun,
                color: warm,
                ..scene.light
            };
            scene.environment.sky.sun_direction = sun;
            scene.environment.sky.sun_color = warm;
            scene.environment.background = Background::Procedural {
                sun_direction: sun,
                sun_color: warm,
            };
            // Towards the sun, a little to its right so its disc is in frame.
            scene.camera = camera_at(
                Vec3::new(x, ground(x, z) + 1.2, z),
                Vec3::new(x - 30.0, ground(x - 30.0, z - 5.0) + 0.6, z - 5.0),
            );
        }
        _ => {}
    }
    scene.grass.push(GrassDraw {
        terrain: 0,
        grass: Grass {
            cast_shadows: true,
            ..Default::default()
        },
    });
    scene
}

/// `grass-wind`: the near view in [`wind`] at `tick` (60 Hz).
pub fn grass_wind_scene(tick: u64) -> RenderScene {
    let mut scene = grass_scene("near");
    scene.wind = Some(wind());
    scene.tick = tick;
    scene
}
