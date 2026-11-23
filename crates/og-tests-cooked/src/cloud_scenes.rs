//! The cloudscapes over an authored sky (og-tests `cloud_scenes` holds
//! the ones under the physical sky): a low deck over the pinned AI
//! afternoon sky, cooked as a game's og.Environment is (its upper
//! hemisphere, the ground filled from the horizon, the sun taken out into
//! the light), seen from 600 m looking down 15 degrees onto its tops and
//! from the ground looking up 15 degrees; and that view with the
//! environment at twice its strength (og.Environment.intensity 2), which
//! lights the clouds with twice the light of the sky drawn behind them:
//! evenly bright, their shade lost. Shot for review; no reference.

use glam::Vec3;
use og_cook::runtime::Bundle;
use og_render::{Background, Camera, Clouds, EnvironmentMap, EnvironmentMapData, RenderScene};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The scenes of this module.
pub const CLOUDSCAPE_SCENES: &[&str] = &[
    "cloudscape-above",
    "cloudscape-authored",
    "cloudscape-authored-bright",
];

/// The AI afternoon sky's image: among og-tests' assets where a capture
/// host has it (`OG_TEST_ASSETS`), else og-cook's pinned test asset.
fn sky_image() -> Result<Vec<u8>, String> {
    let beside = og_tests::scenes::assets_dir().join("env/ai-afternoon.png");
    let pinned = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../og-cook/tests/assets/environment/ai-afternoon.png");
    let path = if beside.exists() { beside } else { pinned };
    std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The cooked environment (once per process).
fn afternoon() -> Result<EnvironmentMapData, String> {
    static MAP: OnceLock<Result<EnvironmentMapData, String>> = OnceLock::new();
    MAP.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("og-tests-cloudscape-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("assets/env");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        std::fs::write(dir.join("afternoon.png"), sky_image()?)
            .map_err(|e| format!("afternoon.png: {e}"))?;
        std::fs::write(
            dir.join("afternoon.ogt"),
            "type environment\nversion 1\n---\nsource \"afternoon.png\"\nhemisphere upper\nsun extract\n",
        )
        .map_err(|e| format!("afternoon.ogt: {e}"))?;
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default())
            .map_err(|e| format!("cook: {e}"))?;
        if !report.ok() {
            return Err(format!("cook: {:?}", report.errors()));
        }
        let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
        let map = bundle
            .load_environment("env/afternoon.ogt")
            .map_err(|e| e.to_string())?;
        let _ = std::fs::remove_dir_all(&root);
        Ok(map)
    })
    .clone()
}

/// The deck over the authored afternoon from `height` metres, the camera
/// pitched `pitch` degrees, the environment at `intensity` (its ambient
/// light and its sun's, as og.Environment.intensity scales them; the sky
/// is drawn as it is whatever it is).
pub fn authored_scene(height: f32, pitch: f32, intensity: f32) -> Result<RenderScene, String> {
    let map = afternoon()?;
    let sun = map.sun;
    let mut scene = RenderScene::new();
    let map = scene.add_environment_map(map);
    scene.lighting.environment.intensity = intensity;
    scene.lighting.environment_map = Some(EnvironmentMap::new(map));
    scene.lighting.environment.background = Background::Sky;
    scene.lighting.light.direction_to_light = Vec3::from_array(sun.direction);
    scene.lighting.light.color = Vec3::from_array(sun.color);
    scene.lighting.light.intensity = sun.intensity * intensity;
    // From over the deck a closed sea of it; from under it a broken deck
    // of denser heaps, which hang separate lobes with shaded bases where
    // the thinner, closer deck was one even sheet seen from below.
    let (density, coverage) = if height > 180.0 {
        (0.04, 0.65)
    } else {
        (0.1, 0.45)
    };
    let clouds = Clouds {
        density,
        base: 180.0,
        thickness: 220.0,
        scale: 1800.0,
        coverage,
        layer: 0.0,
        wind: Vec3::ZERO,
        steps: 96,
        ..Clouds::default()
    };
    clouds.light_scene(&mut scene);
    scene.atmosphere.clouds = Some(clouds);
    let pitch = pitch.to_radians();
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, height, 0.0),
        target: Vec3::new(0.0, height + pitch.sin(), -pitch.cos()),
        far: 100_000.0,
        ..Default::default()
    };
    Ok(scene)
}
