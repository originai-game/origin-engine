//! The depth wave's render-lod scenes (docs/design.md §10.6): mesh levels of
//! detail picked by projected error (`lod-levels`, cooked: in
//! og-tests-cooked's depth_render_scenes), a closed room whose
//! chrome sphere reflects a reflection probe (`probe-interior`), and
//! screen-space reflections on a wet floor (`ssr-wet-floor`).
//!
//! - `probe-interior`: a closed 10 × 4 × 10 m tavern hall (a red cloth on
//!   the back wall, a blue-washed left wall, limewash on the right and
//!   front, a plank floor and beamed ceiling, a hearth, the bar, kegs and
//!   crates) under the procedural sky, lit by the hearth and a lantern,
//!   with a silvered gazing ball on a stump and a reflection probe filling
//!   the room. The ball shows the hall, not the sky: without the probe it
//!   reflects the sky's blue.
//! - `ssr-wet-floor`: a mirror-smooth dark floor under a
//!   black background, a red sphere and a lit white panel standing on it,
//!   PostEffects.ssr on. The floor below each shows its reflection; with
//!   the environment black and no probe, nothing else puts it there.

use glam::{Mat4, Quat, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Exposure, LightDesc, LightKind, ProbeMode,
    ReflectionProbe, RenderScene, ScreenSpaceReflections, StandardMaterial, shapes,
};

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

/// The chrome sphere's centre and radius in `probe-interior`.
pub const CHROME: (Vec3, f32) = (Vec3::new(-0.8, 1.0, 0.0), 1.0);

/// `probe-interior` (module docs); `probe` false leaves the probe out.
pub fn probe_scene(probe: bool) -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    // The sun is off: the hall is closed, its light the hearth's and a
    // lantern's.
    scene.lighting.light = DirectionalLight {
        direction_to_light: Vec3::new(-0.4, 0.7, 0.5),
        intensity: 0.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.lighting.environment.intensity = 1.0;
    scene.frame.camera = Camera {
        position: Vec3::new(0.3, 1.6, 4.2),
        target: Vec3::new(-0.6, 1.0, 0.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    // A tavern's hall, 10 × 4 × 10 m: a plank floor and ceiling on beams,
    // limewashed rubble walls, the back one hung with a red cloth, the
    // left one washed blue.
    let stone = |s: &mut RenderScene, tint: [f32; 3]| kit::surface_tinted(s, "stone", tint);
    let lime = stone(&mut scene, [1.1, 1.04, 0.92]);
    let red = stone(&mut scene, [1.25, 0.3, 0.2]);
    let blue = stone(&mut scene, [0.45, 0.6, 1.2]);
    let planks = kit::surface_tinted(&mut scene, "planks", [0.85, 0.75, 0.65]);
    for (m, min, max) in [
        (
            planks,
            Vec3::new(-5.0, -0.2, -5.0),
            Vec3::new(5.0, 0.0, 5.0),
        ),
        (planks, Vec3::new(-5.1, 4.0, -5.1), Vec3::new(5.1, 4.2, 5.1)),
        (red, Vec3::new(-5.0, 0.0, -5.1), Vec3::new(5.0, 4.0, -4.9)),
        (blue, Vec3::new(-5.1, 0.0, -5.0), Vec3::new(-4.9, 4.0, 5.0)),
        (lime, Vec3::new(4.9, 0.0, -5.0), Vec3::new(5.1, 4.0, 5.0)),
        (lime, Vec3::new(-5.0, 0.0, 4.9), Vec3::new(5.0, 4.0, 5.1)),
    ] {
        let mesh = scene.add_mesh(kit::block(min, max, 1.8));
        scene.add_instance(mesh, m, Mat4::IDENTITY);
    }
    for x in [-3.3f32, -1.1, 1.1, 3.3] {
        let beam = scene.add_mesh(kit::block(
            Vec3::new(x - 0.15, 3.7, -4.9),
            Vec3::new(x + 0.15, 4.0, 4.9),
            1.2,
        ));
        scene.add_instance(beam, planks, Mat4::IDENTITY);
    }
    // The hearth in the back wall's left half: a stone chimney breast, its
    // fire a glowing bed of embers.
    for (min, max) in [
        (Vec3::new(-4.2, 0.0, -4.9), Vec3::new(-3.9, 1.4, -4.3)),
        (Vec3::new(-2.7, 0.0, -4.9), Vec3::new(-2.4, 1.4, -4.3)),
        (Vec3::new(-4.3, 1.4, -4.95), Vec3::new(-2.3, 4.0, -4.2)),
    ] {
        let mesh = scene.add_mesh(kit::block(min, max, 1.4));
        scene.add_instance(mesh, lime, Mat4::IDENTITY);
    }
    let embers = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.1, 0.05, 0.03),
        emissive: Vec3::new(1.0, 0.42, 0.12),
        emissiveIntensity: 4.0,
        roughness: 0.9,
        ..Default::default()
    });
    boxed_with(
        &mut scene,
        embers,
        Vec3::new(-3.3, 0.08, -4.6),
        Vec3::new(1.4, 0.16, 0.5),
    );
    // The bar on the right with its kegs and crates, a table of barrels.
    let counter = kit::prop(&mut scene, "counter");
    counter.place(&mut scene, Vec3::new(3.6, 0.0, -1.0), -1.571, 1.2);
    let barrel = kit::prop(&mut scene, "winebarrel");
    for (at, turn) in [
        (Vec3::new(4.3, 0.0, -3.9), 0.3),
        (Vec3::new(3.4, 0.0, -4.2), 1.2),
        (Vec3::new(3.85, 1.0, -4.05), 0.7),
        (Vec3::new(2.2, 0.0, 1.6), 0.4),
    ] {
        barrel.place(&mut scene, at, turn, 1.0);
    }
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-4.1, 0.0, 1.8), 0.2, 0.7);
    crate_.place(&mut scene, Vec3::new(-4.0, 0.7, 1.75), -0.1, 0.65);
    crate_.place(&mut scene, Vec3::new(4.2, 0.0, 1.2), 0.35, 0.7);
    barrel.place(&mut scene, Vec3::new(-2.0, 0.0, -3.9), 0.9, 1.0);
    // A trestle table with its benches.
    for (min, max) in [
        (Vec3::new(0.7, 0.72, 0.4), Vec3::new(1.7, 0.8, 2.8)),
        (Vec3::new(0.8, 0.0, 0.6), Vec3::new(1.6, 0.72, 0.7)),
        (Vec3::new(0.8, 0.0, 2.5), Vec3::new(1.6, 0.72, 2.6)),
        (Vec3::new(0.15, 0.4, 0.5), Vec3::new(0.45, 0.46, 2.7)),
        (Vec3::new(0.2, 0.0, 0.7), Vec3::new(0.4, 0.4, 0.8)),
        (Vec3::new(0.2, 0.0, 2.4), Vec3::new(0.4, 0.4, 2.5)),
        (Vec3::new(1.95, 0.4, 0.5), Vec3::new(2.25, 0.46, 2.7)),
        (Vec3::new(2.0, 0.0, 0.7), Vec3::new(2.2, 0.4, 0.8)),
        (Vec3::new(2.0, 0.0, 2.4), Vec3::new(2.2, 0.4, 2.5)),
    ] {
        let mesh = scene.add_mesh(kit::block(min, max, 1.0));
        scene.add_instance(mesh, planks, Mat4::IDENTITY);
    }
    // The tavern's curiosity: a silvered gazing ball on a stone plinth.
    let foot = CHROME.0.y - 0.5 * CHROME.1;
    let plinth = scene.add_mesh(kit::block(
        Vec3::new(CHROME.0.x - 0.4, 0.0, CHROME.0.z - 0.4),
        Vec3::new(CHROME.0.x + 0.4, foot, CHROME.0.z + 0.4),
        1.0,
    ));
    scene.add_instance(plinth, lime, Mat4::IDENTITY);
    let sphere = scene.add_mesh(shapes::sphere(64, 32));
    let chrome = scene.add_standard_material(mat(Vec3::splat(0.95), 1.0, 0.02));
    scene.add_instance(
        sphere,
        chrome,
        Mat4::from_scale_rotation_translation(Vec3::splat(CHROME.1), Quat::IDENTITY, CHROME.0),
    );
    // The hearth's warm light and a lantern's by the bar.
    for (at, c, i) in [
        (Vec3::new(-3.3, 0.8, -4.0), Vec3::new(1.0, 0.55, 0.25), 40.0),
        (Vec3::new(3.0, 3.2, 2.0), Vec3::new(1.0, 0.8, 0.55), 30.0),
    ] {
        scene.lighting.lights.push(LightDesc {
            kind: LightKind::Point,
            position: at,
            color: c,
            intensity: i,
            range: 12.0,
            radius: 0.0,
            cast_shadows: false,
            mobility: og_render::LightMobility::Movable,
        });
    }
    if probe {
        let mut p = ReflectionProbe::new(1, Vec3::new(0.0, 1.6, 0.0), Vec3::new(10.0, 4.2, 10.0));
        p.mode = ProbeMode::Baked;
        p.fade = 0.5;
        scene.lighting.reflection_probes.push(p);
    }
    scene
}

fn boxed_with(scene: &mut RenderScene, m: og_render::MaterialHandle, at: Vec3, size: Vec3) {
    let mesh = scene.add_mesh(shapes::cuboid(size));
    scene.add_instance(mesh, m, Mat4::from_translation(at));
}

pub use crate::SSR_FRAMES;

/// `ssr-wet-floor` (module docs); `ssr` false turns the reflections off.
pub fn ssr_scene(ssr: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.environment.intensity = 0.0;
    let key = Vec3::new(0.3, 0.8, 0.6);
    scene.lighting.light = DirectionalLight {
        direction_to_light: key,
        intensity: 2.5,
        ..Default::default()
    };
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.4, 5.0),
        target: Vec3::new(0.0, 0.6, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    boxed(
        &mut scene,
        mat(Vec3::splat(0.04), 0.0, 0.04),
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(14.0, 0.2, 14.0),
    );
    let sphere = scene.add_mesh(shapes::sphere(48, 24));
    let red = scene.add_standard_material(mat(Vec3::new(0.85, 0.08, 0.05), 0.0, 0.4));
    scene.add_instance(
        sphere,
        red,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(0.8),
            Quat::IDENTITY,
            Vec3::new(-1.0, 0.8, 0.0),
        ),
    );
    let panel = scene.add_standard_material(StandardMaterial {
        color: Vec3::ONE,
        emissive: Vec3::new(1.0, 0.95, 0.8),
        emissiveIntensity: 2.0,
        ..Default::default()
    });
    let slab = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 2.0, 0.2)));
    scene.add_instance(
        slab,
        panel,
        Mat4::from_translation(Vec3::new(1.4, 1.0, -0.8)),
    );
    if ssr {
        scene.look.post.ssr = Some(ScreenSpaceReflections {
            max_distance: 10.0,
            steps: 48,
            ..Default::default()
        });
    }
    scene
}
