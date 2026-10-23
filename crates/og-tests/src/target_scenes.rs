//! The depth wave's render targets (docs/depth.md §8): a security monitor
//! showing the room from a second camera (`views-monitor`), a mirror that
//! reflects the room and the figure in front of it (`views-mirror`),
//! split-screen with two viewport views over a frame (`views-split`), and
//! layer masks: an object on layer 1 only that the main camera skips and
//! the monitor shows (in `views-monitor`).

use glam::{Mat4, Quat, Vec3};
use og_render::{
    Camera, DirectionalLight, Exposure, MirrorQuad, RenderScene, RenderTargetDesc,
    StandardMaterial, View, ViewTarget, shapes,
};

fn colour(r: f32, g: f32, b: f32) -> StandardMaterial {
    StandardMaterial {
        color: Vec3::new(r, g, b),
        roughness: 0.6,
        ..Default::default()
    }
}

/// A walled room with a red box, a green pillar, a blue sphere (the
/// figure) and a yellow box; returns the scene lit by a low sun and a lamp.
pub fn room() -> RenderScene {
    let mut scene = RenderScene::new();
    let sun = Vec3::new(-0.4, 0.8, 0.5);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.6;
    let mut put = |mesh, m: StandardMaterial, t: Mat4| {
        let mesh = scene.add_mesh(mesh);
        let h = scene.add_standard_material(m);
        scene.add_instance(mesh, h, t);
    };
    put(shapes::plane(12.0, 6.0), colour(0.55, 0.52, 0.48), Mat4::IDENTITY);
    // Back and left walls.
    put(
        shapes::cuboid(Vec3::new(12.0, 3.0, 0.2)),
        colour(0.7, 0.68, 0.62),
        Mat4::from_translation(Vec3::new(0.0, 1.5, -6.0)),
    );
    put(
        shapes::cuboid(Vec3::new(0.2, 3.0, 12.0)),
        colour(0.45, 0.55, 0.65),
        Mat4::from_translation(Vec3::new(-6.0, 1.5, 0.0)),
    );
    put(
        shapes::cuboid(Vec3::splat(1.0)),
        colour(0.85, 0.15, 0.1),
        Mat4::from_rotation_translation(Quat::from_rotation_y(0.4), Vec3::new(-2.0, 0.5, -2.0)),
    );
    put(
        shapes::cuboid(Vec3::new(0.6, 2.4, 0.6)),
        colour(0.15, 0.7, 0.2),
        Mat4::from_translation(Vec3::new(2.5, 1.2, -3.0)),
    );
    put(
        shapes::sphere(32, 16),
        colour(0.15, 0.3, 0.9),
        Mat4::from_scale_rotation_translation(
            Vec3::splat(0.9),
            Quat::IDENTITY,
            Vec3::new(0.3, 0.9, 0.5),
        ),
    );
    put(
        shapes::cuboid(Vec3::new(0.8, 0.8, 0.8)),
        colour(0.95, 0.8, 0.1),
        Mat4::from_translation(Vec3::new(3.0, 0.4, 1.5)),
    );
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.6, 7.5),
        target: Vec3::new(0.0, 1.0, 0.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    scene
}

/// A flat unlit screen showing `texture` (a render target's colour).
fn screen(scene: &mut RenderScene, texture: og_render::TextureHandle, w: f32, h: f32, at: Mat4) {
    let mesh = scene.add_mesh(shapes::plane_xy(w, h));
    let m = scene.add_standard_material(StandardMaterial {
        map: texture.slot(),
        unlit: 1,
        ..Default::default()
    });
    scene.add_instance(mesh, m, at);
}

/// The room with a monitor on the back wall showing a security camera in
/// the corner, and a magenta cube on layer 1 only: the monitor's camera
/// (layers 3) shows it, the main camera (layer 0) does not.
pub fn monitor_scene() -> RenderScene {
    let mut scene = room();
    let target = scene.add_render_target(RenderTargetDesc {
        width: 480,
        height: 270,
        ..Default::default()
    });
    screen(
        &mut scene,
        target.color,
        3.2,
        1.8,
        Mat4::from_translation(Vec3::new(0.0, 1.9, -5.85)),
    );
    let ghost = scene.add_mesh(shapes::cuboid(Vec3::splat(0.7)));
    let magenta = scene.add_standard_material(colour(0.9, 0.1, 0.8));
    scene.add_instance(ghost, magenta, Mat4::from_translation(Vec3::new(-1.0, 0.35, 2.0)));
    let i = scene.instances.len() - 1;
    scene.instance_layers.resize(i + 1, 1);
    scene.instance_layers[i] = 2;
    scene.views.push(View {
        camera: Camera {
            position: Vec3::new(5.5, 2.8, 5.5),
            target: Vec3::new(-1.0, 0.6, -1.0),
            exposure: Exposure::linear(0.8),
            ..Default::default()
        },
        target: ViewTarget::Texture(target),
        layers: 3,
        ..Default::default()
    });
    scene
}

/// The room with a mirror on the left wall facing the figure, seen from
/// the side: the mirror shows the figure and the room behind the camera.
pub fn mirror_scene() -> RenderScene {
    let mut scene = room();
    let target = scene.add_render_target(RenderTargetDesc {
        width: 512,
        height: 384,
        format: og_render::TargetFormat::Rgba16Float,
        ..Default::default()
    });
    // On the left wall, facing +x.
    let at = Mat4::from_rotation_translation(
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Vec3::new(-5.85, 1.4, 0.0),
    );
    let (w, h) = (4.0, 2.4);
    let mesh = scene.add_mesh(shapes::plane_xy(w, h));
    // Linear HDR: shown as emitted light, exposed with the frame.
    let m = scene.add_standard_material(StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::ONE,
        emissiveMap: target.color.slot(),
        emissiveIntensity: 0.92,
        ..Default::default()
    });
    scene.add_instance(mesh, m, at);
    scene.camera.position = Vec3::new(3.5, 2.0, 4.5);
    scene.camera.target = Vec3::new(-3.0, 1.2, 0.0);
    scene.views.push(View {
        camera: scene.camera,
        target: ViewTarget::Texture(target),
        mirror: Some(MirrorQuad {
            transform: at,
            width: w,
            height: h,
        }),
        ..Default::default()
    });
    scene
}

/// Split-screen: the left half from the front, the right half from above,
/// each a viewport view over the main frame.
pub fn split_scene() -> RenderScene {
    let mut scene = room();
    let exposure = Exposure::linear(0.8);
    scene.views.push(View {
        camera: Camera {
            position: Vec3::new(-1.0, 2.0, 6.0),
            target: Vec3::new(-0.5, 0.8, 0.0),
            exposure,
            ..Default::default()
        },
        target: ViewTarget::Viewport([0.0, 0.0, 0.5, 1.0]),
        ..Default::default()
    });
    scene.views.push(View {
        camera: Camera {
            position: Vec3::new(0.5, 11.0, 2.5),
            target: Vec3::new(0.5, 0.0, -0.5),
            exposure,
            ..Default::default()
        },
        target: ViewTarget::Viewport([0.5, 0.0, 0.5, 1.0]),
        order: 1,
        ..Default::default()
    });
    scene
}
