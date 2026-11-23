//! The cloudscapes: the views a cloud renderer is judged on, each a frame
//! of mostly sky. Under the physical sky: scattered cumulus from the
//! ground at noon and into a sunset, an overcast, the layer from above
//! it, and one view of each other kind of cloud. They are shot for review (`og-gfx cloudscape-noon --out`), never
//! compared with a reference: none is in `SCENES`. The two over an
//! authored sky cook their environment (og-tests-cooked `cloud_scenes`).

use crate::sky_scenes::{clouded, physical_light_under, physical_sky_scene};
use glam::Vec3;
use og_render::{CloudKind, Clouds, PhysicalAtmosphere, RenderScene};

/// The scenes of this module.
pub const CLOUDSCAPE_SCENES: &[&str] = &[
    "cloudscape-noon",
    "cloudscape-sunset",
    "cloudscape-overcast",
    "cloudscape-aloft",
    "cloudscape-stratocumulus",
    "cloudscape-stratus",
    "cloudscape-cumulonimbus",
];

/// `scene`'s camera lifted over the colonnade's capitals and pitched
/// `pitch` degrees up along the bearing it had, so the sky fills the frame
/// over the hills.
pub fn sky_view(mut scene: RenderScene, height: f32, pitch: f32) -> RenderScene {
    let camera = &mut scene.frame.camera;
    let view = camera.target - camera.position;
    let bearing = Vec3::new(view.x, 0.0, view.z).normalize_or(Vec3::NEG_Z);
    let pitch = pitch.to_radians();
    camera.position = Vec3::new(0.0, height, 30.0);
    camera.target = camera.position + bearing * pitch.cos() + Vec3::Y * pitch.sin();
    camera.far = 100_000.0;
    scene
}

/// The scene called `name` of [`CLOUDSCAPE_SCENES`].
pub fn cloudscape_scene(name: &str) -> RenderScene {
    let earth = PhysicalAtmosphere::EARTH;
    match name {
        // Fair-weather cumulus two fifths over the sky, the sun high
        // behind the camera's left.
        "cloudscape-noon" => sky_view(
            clouded(
                physical_sky_scene(12.5, earth, false),
                Clouds {
                    coverage: 0.4,
                    ..Clouds::default()
                },
            ),
            14.0,
            26.0,
        ),
        // The same sky into the sun, twenty minutes before it sets.
        "cloudscape-sunset" => sky_view(
            clouded(
                physical_sky_scene(17.7, earth, true),
                Clouds {
                    coverage: 0.4,
                    ..Clouds::default()
                },
            ),
            14.0,
            12.0,
        ),
        // A closed grey deck 900 m up.
        "cloudscape-overcast" => sky_view(
            clouded(
                physical_sky_scene(13.0, earth, false),
                Clouds {
                    coverage: 0.95,
                    density: 0.06,
                    color: Vec3::splat(0.8),
                    base: 900.0,
                    layer: 0.0,
                    ..Clouds::default()
                },
            ),
            14.0,
            20.0,
        ),
        // Over broken cumulus from 300 m above the layer's top, looking
        // down 15 degrees towards the afternoon sun's side.
        "cloudscape-aloft" => sky_view(
            clouded(
                physical_sky_scene(15.0, earth, false),
                Clouds {
                    coverage: 0.6,
                    ..Clouds::default()
                },
            ),
            3600.0,
            -15.0,
        ),
        // A broken sheet of rolls 1200 m up, in the middle of the morning,
        // at the clear sky's exposure (`clouded` opens the camera for the
        // light under a deck, and the sheet's bases, the sun through
        // them, were drawn white).
        "cloudscape-stratocumulus" => {
            let clear = physical_sky_scene(10.0, earth, false);
            let exposure = clear.frame.camera.exposure;
            let mut scene = clouded(
                clear,
                Clouds {
                    kind: CloudKind::Stratocumulus,
                    coverage: 0.7,
                    base: 1200.0,
                    thickness: 1200.0,
                    layer: 0.0,
                    ..Clouds::default()
                },
            );
            scene.frame.camera.exposure = exposure;
            sky_view(scene, 14.0, 24.0)
        }
        // A sea of stratus from 500 m over its top, in the afternoon, at
        // the clear sky's exposure (`clouded` opens the camera for the
        // light under a deck).
        "cloudscape-stratus" => {
            let clear = physical_sky_scene(15.0, earth, false);
            let exposure = clear.frame.camera.exposure;
            let mut scene = clouded(
                clear,
                Clouds {
                    kind: CloudKind::Stratus,
                    coverage: 0.9,
                    base: 800.0,
                    thickness: 1600.0,
                    ..Clouds::default()
                },
            );
            scene.frame.camera.exposure = exposure;
            sky_view(scene, 1700.0, -8.0)
        }
        // Storm towers through 9 km of air, a few of them over the hills,
        // the sun 40 degrees up behind the camera's right shoulder, on the
        // faces it sees.
        "cloudscape-cumulonimbus" => sky_view(
            clouded(
                {
                    let mut scene = physical_sky_scene(12.5, earth, false);
                    let camera = scene.frame.camera;
                    let view = camera.target - camera.position;
                    let back = -Vec3::new(view.x, 0.0, view.z).normalize_or(Vec3::Z);
                    let side = Vec3::new(-back.z, 0.0, back.x);
                    let level = (back * 0.87 + side * 0.5).normalize();
                    physical_light_under(&mut scene, level * 0.766 + Vec3::Y * 0.643);
                    scene
                },
                Clouds {
                    kind: CloudKind::Cumulonimbus,
                    coverage: 0.3,
                    density: 0.06,
                    base: 1000.0,
                    thickness: 9000.0,
                    scale: 24_000.0,
                    layer: 0.0,
                    ..Clouds::default()
                },
            ),
            14.0,
            22.0,
        ),
        other => panic!("{other} is no cloudscape"),
    }
}
