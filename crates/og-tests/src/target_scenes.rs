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
    put(
        shapes::plane(12.0, 6.0),
        colour(0.55, 0.52, 0.48),
        Mat4::IDENTITY,
    );
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
    scene.add_instance(
        ghost,
        magenta,
        Mat4::from_translation(Vec3::new(-1.0, 0.35, 2.0)),
    );
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

/// A full custom shader (og-render `custom.rs`): a dissolve cut through a
/// sphere and a panel by `amount`, the glowing edge above the cut, the low
/// sun's shadows on the ground cut the same way (the shadow passes draw
/// through the shader's vertex and fragment entries).
pub fn full_dissolve_scene(amount: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let sun = Vec3::new(-0.7, 0.55, 0.3);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.5,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.5;
    let ground = scene.add_mesh(shapes::plane(16.0, 4.0));
    let grey = scene.add_standard_material(colour(0.6, 0.6, 0.6));
    scene.add_instance(ground, grey, Mat4::IDENTITY);
    let shader = scene.add_shader(og_render::example_full_shader());
    let dissolve = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.2, 0.45, 0.9),
        roughness: 0.4,
        alphaTest: 0.5,
        side: 2,
        ..Default::default()
    });
    scene.set_material_shader(dissolve, Some(shader));
    scene.set_shader_param(dissolve, "amount", &[amount]);
    let sphere = scene.add_mesh(shapes::sphere(48, 24));
    scene.add_instance(
        sphere,
        dissolve,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(1.2),
            Quat::IDENTITY,
            Vec3::new(-1.2, 1.3, 0.0),
        ),
    );
    // A thin panel facing the sun: its holes are its shadow's.
    let panel = scene.add_mesh(shapes::cuboid(Vec3::new(2.2, 2.4, 0.06)));
    scene.add_instance(
        panel,
        dissolve,
        Mat4::from_rotation_translation(Quat::from_rotation_y(-1.17), Vec3::new(1.4, 1.25, -0.5)),
    );
    scene.camera = Camera {
        position: Vec3::new(1.0, 3.5, 7.5),
        target: Vec3::new(0.5, 0.8, 0.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    scene
}

/// What a secondary camera draws besides opaque meshes (`views-layers`):
/// a translucent cyan glass box, a pink world sprite, a green particle
/// fountain and a yellow line loop, each on layer 1 alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewKind {
    Transparent,
    Sprite,
    Particles,
    Lines,
}

impl ViewKind {
    pub const ALL: [ViewKind; 4] = [
        ViewKind::Transparent,
        ViewKind::Sprite,
        ViewKind::Particles,
        ViewKind::Lines,
    ];
}

/// Puts `kind` into `scene` on layer 1 alone (mask 2), in front of the
/// monitor's corner camera and of the main camera.
fn add_view_kind(scene: &mut RenderScene, kind: ViewKind) {
    match kind {
        ViewKind::Transparent => {
            let glass = scene.add_mesh(shapes::cuboid(Vec3::splat(1.1)));
            let m = scene.add_standard_material(StandardMaterial {
                color: Vec3::new(0.1, 0.9, 0.95),
                emissive: Vec3::new(0.05, 0.35, 0.4),
                opacity: 0.6,
                transparent: 1,
                ..Default::default()
            });
            scene.add_instance(glass, m, Mat4::from_translation(Vec3::new(-2.6, 0.7, 2.4)));
            let i = scene.instances.len() - 1;
            // Out of the shadow maps: the main frame keeps no trace of it.
            scene.instances[i].cast_shadows = false;
            scene.instance_layers.resize(i + 1, 1);
            scene.instance_layers[i] = 2;
        }
        ViewKind::Sprite => {
            scene.add_sprite(og_render::Sprite {
                position: Vec3::new(1.4, 1.3, 2.6),
                size: glam::Vec2::new(1.2, 1.2),
                color: [1.0, 0.2, 0.65, 1.0],
                ..Default::default()
            });
            let i = scene.sprites.len() - 1;
            scene.sprite_layers.resize(i + 1, 1);
            scene.sprite_layers[i] = 2;
        }
        ViewKind::Particles => {
            let e = scene.add_emitter(og_render::ParticleEmitter {
                rate: 90.0,
                lifetime: [1.4, 1.6],
                speed: [0.5, 0.7],
                size: [0.45, 0.6],
                gravity: Vec3::ZERO,
                color: [0.1, 0.85, 0.15, 1.0],
                blend: og_render::ParticleBlend::Alpha,
                ..Default::default()
            });
            scene.particles.push(og_render::ParticleSystem::new(
                7,
                e,
                Mat4::from_translation(Vec3::new(2.0, 0.2, 2.4)),
                0x51e8,
                0,
            ));
            scene.particle_layers = vec![2; scene.particles.len()];
        }
        ViewKind::Lines => {
            let c = Vec3::new(3.6, 1.2, 3.4);
            scene.lines.push(og_render::Lines {
                points: vec![
                    c + Vec3::new(-0.8, -0.8, 0.0),
                    c + Vec3::new(0.8, -0.8, 0.0),
                    c + Vec3::new(0.8, 0.8, 0.0),
                    c + Vec3::new(-0.8, 0.8, 0.0),
                ],
                color: [1.0, 0.95, 0.1, 1.0],
                mode: og_render::LineMode::Loop,
                width: 0.12,
                world_width: true,
                ..Default::default()
            });
            let i = scene.lines.len() - 1;
            scene.line_layers.resize(i + 1, 1);
            scene.line_layers[i] = 2;
        }
    }
}

/// The monitor room (`monitor_scene` without its ghost cube) with
/// `kinds` on layer 1 alone; the monitor's camera sees layers `view_layers`.
/// The scene's clocks stand at `tick` (the fountain's age).
pub fn view_kinds_scene(kinds: &[ViewKind], view_layers: u32, tick: u64) -> RenderScene {
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
    for &k in kinds {
        add_view_kind(&mut scene, k);
    }
    scene.views.push(View {
        camera: Camera {
            position: Vec3::new(5.5, 2.8, 5.5),
            target: Vec3::new(-1.0, 0.6, -1.0),
            exposure: Exposure::linear(0.8),
            ..Default::default()
        },
        target: ViewTarget::Texture(target),
        layers: view_layers,
        ..Default::default()
    });
    crate::fx_scenes::tick_all(&mut scene, tick);
    scene
}

/// A render target's depth texture (`views-depth`): a camera at (0, 1.5, 0)
/// looking down -z at a wall whose face is 5.9 m away, with a box whose
/// face is 2.5 m away in the middle, near 0.1 and far 20; the target's
/// depth texture fills the frame through an unlit screen seen by an
/// orthographic main camera, untonemapped at exposure 1, so a pixel shows
/// the normalised linear depth sRGB-encoded. A box on layer 1, which the
/// view's camera leaves out, writes no depth.
pub fn depth_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    scene.environment.sky.sun_direction = Vec3::new(-0.4, 0.8, 0.5);
    scene.post.tonemap = og_render::Tonemap::None;
    let grey = scene.add_standard_material(colour(0.6, 0.6, 0.6));
    let ground = scene.add_mesh(shapes::plane(12.0, 12.0));
    scene.add_instance(ground, grey, Mat4::IDENTITY);
    let wall = scene.add_mesh(shapes::cuboid(Vec3::new(12.0, 3.0, 0.2)));
    scene.add_instance(
        wall,
        grey,
        Mat4::from_translation(Vec3::new(0.0, 1.5, -6.0)),
    );
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(1.0)));
    let red = scene.add_standard_material(colour(0.8, 0.1, 0.1));
    scene.add_instance(cube, red, Mat4::from_translation(Vec3::new(0.0, 1.5, -3.0)));
    // Off the view's layers, between it and the wall's left half.
    scene.add_instance(
        cube,
        red,
        Mat4::from_translation(Vec3::new(-1.6, 1.5, -4.0)),
    );
    let i = scene.instances.len() - 1;
    scene.instance_layers.resize(i + 1, 1);
    scene.instance_layers[i] = 2;
    let target = scene.add_render_target(RenderTargetDesc {
        width: 320,
        height: 180,
        depth: true,
        ..Default::default()
    });
    scene.views.push(View {
        camera: Camera {
            position: Vec3::new(0.0, 1.5, 0.0),
            target: Vec3::new(0.0, 1.5, -6.0),
            near: 0.1,
            far: 20.0,
            ..Default::default()
        },
        target: ViewTarget::Texture(target),
        ..Default::default()
    });
    // The screen, far from the room, filling an orthographic frame.
    let at = Vec3::new(200.0, 0.0, 0.0);
    screen(
        &mut scene,
        target.depth.expect("asked for"),
        16.0,
        9.0,
        Mat4::from_translation(at),
    );
    scene.camera = Camera {
        position: at + Vec3::Z * 5.0,
        target: at,
        projection: og_render::Projection::Orthographic { height: 9.0 },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// What a view draws of the main pass's effects (`view_fx_scene`), each
/// in front of the view's camera and out of the main camera's sight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewFx {
    /// An orange ribbon across a perspective view.
    Ribbon,
    /// A white sprite on a 2D-lit layer under a dark ambient and a warm
    /// light to its upper right (off: the frame without 2D lighting).
    Lit2d,
    /// A blue 2D fluid: a blob of metaball particles.
    Fluid2d,
}

impl ViewFx {
    pub const ALL: [ViewFx; 3] = [ViewFx::Ribbon, ViewFx::Lit2d, ViewFx::Fluid2d];
}

/// A dark grey wall at z = -1 and the effect `fx` (`on` false: the scene
/// without it), drawn by a viewport view over the frame's right half
/// (x 0.5..1) looking down -z from (0, 0, 6); the main camera looks the
/// other way, along +z, so the left half never shows the effect.
pub fn view_fx_scene(fx: ViewFx, on: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(0.2, 0.5, 1.0),
        intensity: 1.5,
        ..Default::default()
    };
    scene.environment.intensity = 0.3;
    scene.environment.background = og_render::Background::Color(Vec3::splat(0.02));
    let wall = scene.add_mesh(shapes::cuboid(Vec3::new(20.0, 20.0, 0.2)));
    let grey = scene.add_standard_material(colour(0.25, 0.25, 0.25));
    scene.add_instance(
        wall,
        grey,
        Mat4::from_translation(Vec3::new(0.0, 0.0, -1.1)),
    );
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 6.0),
        target: Vec3::new(0.0, 0.0, 12.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 6.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    if on {
        match fx {
            ViewFx::Ribbon => {
                let point = |x: f32| og_render::RibbonPoint {
                    position: Vec3::new(x, 0.3 * x.sin(), 0.0),
                    width: 0.6,
                    color: [1.0, 0.45, 0.1, 1.0],
                };
                scene.ribbons.push(og_render::Ribbon {
                    points: (0..=8).map(|i| point(-2.0 + i as f32 * 0.5)).collect(),
                    texture: None,
                    uv: og_render::RibbonUv::Stretch,
                    blend: og_render::ParticleBlend::Alpha,
                });
            }
            ViewFx::Lit2d => {
                scene.lighting_2d = Some(og_render::Lighting2d {
                    ambient: Vec3::splat(0.05),
                    ..Default::default()
                });
                scene.lights_2d.push(og_render::Light2d {
                    position: glam::Vec2::new(0.8, 0.6),
                    color: Vec3::new(1.0, 0.7, 0.4),
                    intensity: 2.0,
                    radius: 2.2,
                    ..Default::default()
                });
            }
            ViewFx::Fluid2d => {
                let particles: Vec<glam::Vec2> = (0..48)
                    .map(|i| {
                        let a = i as f32 * 2.399;
                        let r = 1.2 * ((i as f32 + 0.5) / 48.0).sqrt();
                        glam::Vec2::new(r * a.cos(), r * a.sin())
                    })
                    .collect();
                scene.metaballs.push(og_render::Metaballs {
                    particles: particles.into(),
                    radius: 0.45,
                    ..Default::default()
                });
            }
        }
    }
    if fx == ViewFx::Lit2d {
        scene.add_sprite(og_render::Sprite {
            position: Vec3::ZERO,
            size: glam::Vec2::new(3.0, 3.0),
            color: [0.9, 0.9, 0.9, 1.0],
            ..Default::default()
        });
    }
    scene.views.push(View {
        camera,
        target: ViewTarget::Viewport([0.5, 0.0, 0.5, 1.0]),
        ..Default::default()
    });
    crate::fx_scenes::tick_all(&mut scene, 60);
    scene
}
