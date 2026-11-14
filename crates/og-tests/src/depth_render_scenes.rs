//! The depth wave's render-lod scenes (docs/design.md §10.6): mesh levels of
//! detail picked by projected error (`lod-levels`, cooked: in
//! og-tests-cooked's depth_render_scenes), a closed room whose
//! chrome sphere reflects a reflection probe (`probe-interior`), and
//! screen-space reflections on a wet floor (`ssr-wet-floor`).
//!
//! - `probe-interior`: a closed 10 × 4 × 10 m room (red back wall, blue
//!   left, cream right and front, white ceiling, a dark floor) under the
//!   procedural sky, lit by two point lights, with a chrome sphere and a
//!   green block inside and a reflection probe filling the room. The
//!   sphere shows the room (the red wall behind the camera's view of it
//!   and the floor), not the sky: without the probe it reflects the sky's
//!   blue.
//! - `ssr-wet-floor`: a mirror-smooth dark floor under a
//!   black background, a red sphere and a lit white panel standing on it,
//!   PostEffects.ssr on. The floor below each shows its reflection; with
//!   the environment black and no probe, nothing else puts it there.

use glam::{Mat4, Quat, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Exposure, Light, LightKind, ProbeMode, ReflectionProbe,
    RenderScene, ScreenSpaceReflections, StandardMaterial, shapes,
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
    let mut scene = RenderScene::new();
    // The sun is off: the room is closed, its light the lamps'.
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
    let wall = |c: Vec3| mat(c, 0.0, 0.8);
    boxed(
        &mut scene,
        wall(Vec3::new(0.75, 0.15, 0.1)),
        Vec3::new(0.0, 2.0, -5.0),
        Vec3::new(10.0, 4.0, 0.2),
    );
    boxed(
        &mut scene,
        wall(Vec3::new(0.12, 0.3, 0.75)),
        Vec3::new(-5.0, 2.0, 0.0),
        Vec3::new(0.2, 4.0, 10.0),
    );
    boxed(
        &mut scene,
        wall(Vec3::new(0.85, 0.8, 0.65)),
        Vec3::new(5.0, 2.0, 0.0),
        Vec3::new(0.2, 4.0, 10.0),
    );
    boxed(
        &mut scene,
        wall(Vec3::new(0.85, 0.8, 0.65)),
        Vec3::new(0.0, 2.0, 5.0),
        Vec3::new(10.0, 4.0, 0.2),
    );
    boxed(
        &mut scene,
        wall(Vec3::new(0.9, 0.9, 0.88)),
        Vec3::new(0.0, 4.1, 0.0),
        Vec3::new(10.2, 0.2, 10.2),
    );
    boxed(
        &mut scene,
        wall(Vec3::new(0.25, 0.22, 0.2)),
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(10.0, 0.2, 10.0),
    );
    boxed(
        &mut scene,
        mat(Vec3::new(0.2, 0.65, 0.3), 0.0, 0.5),
        Vec3::new(2.2, 0.6, -2.0),
        Vec3::splat(1.2),
    );
    let sphere = scene.add_mesh(shapes::sphere(64, 32));
    let chrome = scene.add_standard_material(mat(Vec3::splat(0.95), 1.0, 0.02));
    scene.add_instance(
        sphere,
        chrome,
        Mat4::from_scale_rotation_translation(Vec3::splat(CHROME.1), Quat::IDENTITY, CHROME.0),
    );
    for (at, c) in [
        (Vec3::new(-3.0, 3.2, -3.0), Vec3::new(1.0, 0.75, 0.45)),
        (Vec3::new(3.0, 3.2, 2.0), Vec3::new(0.6, 0.75, 1.0)),
    ] {
        scene.lighting.lights.push(Light {
            kind: LightKind::Point,
            position: at,
            color: c,
            intensity: 30.0,
            range: 12.0,
            cast_shadows: false,
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

/// [`ssr_scene`] with SSR on and its floor a pale metal of `roughness`,
/// traced up to `max_roughness`: the glossy and rough reflections of the
/// lit slab and the red sphere.
pub fn ssr_metal_scene(roughness: f32, max_roughness: f32) -> RenderScene {
    let mut scene = ssr_scene(true);
    let floor = scene.objects.instances[0].material;
    scene.set_standard_material(floor, mat(Vec3::splat(0.9), 1.0, roughness));
    if let Some(ssr) = scene.look.post.ssr.as_mut() {
        ssr.max_roughness = max_roughness;
    }
    scene
}
