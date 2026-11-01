use super::effects_layers::stripes;
use super::*;

/// `ui-text-effects`: outline out, in and across; a gradient fill; stacked
/// shadows; all four together.
pub fn text_effects(
    font: impl FnOnce(&mut RenderScene) -> Result<FontHandle, String>,
) -> Result<RenderScene, String> {
    let mut err = None;
    let scene = overlay_scene(|scene, o| {
        let font = match font(scene) {
            Ok(f) => f,
            Err(e) => {
                err = Some(e);
                return;
            }
        };
        stripes(o);
        o.ui.shapes.push(UiShape::rect(
            [30.0, 30.0, 1220.0, 660.0],
            24.0,
            [0.06, 0.065, 0.08, 0.92],
        ));
        let line = |o: &mut Overlay, s: &str, y: f32, color: [f32; 4]| -> u32 {
            o.texts.push(Text {
                position: Vec3::new(70.0, y, 0.0),
                size: 72.0,
                color,
                layer: 1,
                ..Text::new(font, s)
            });
            o.texts.len() as u32 - 1
        };
        let white = [1.0; 4];
        let orange = rgb(255, 109, 0);
        let outline = |width, align| UiTextOutline {
            width,
            color: rgb(20, 20, 40),
            align,
        };
        let t = line(o, "Outline out 原点", 120.0, white);
        o.ui.text_effects.push(UiTextEffects {
            text: t,
            outline: Some(UiTextOutline {
                color: orange,
                ..outline(4.0, UiAlign::Outside)
            }),
            ..UiTextEffects::default()
        });
        let t = line(o, "Outline in", 230.0, white);
        o.ui.text_effects.push(UiTextEffects {
            text: t,
            outline: Some(UiTextOutline {
                color: orange,
                ..outline(1.5, UiAlign::Inside)
            }),
            ..UiTextEffects::default()
        });
        let t = line(o, "Gradient fill", 340.0, white);
        o.ui.text_effects.push(UiTextEffects {
            text: t,
            fill: Some(gradient(
                UiGradientKind::Linear,
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 1.0),
                &[(0.0, rgb(255, 241, 118)), (1.0, rgb(255, 61, 0))],
            )),
            fill_box: [70.0, 340.0 - 60.0, 700.0, 80.0],
            ..UiTextEffects::default()
        });
        let t = line(o, "Two shadows", 450.0, white);
        o.ui.text_effects.push(UiTextEffects {
            text: t,
            shadows: vec![
                UiTextShadow {
                    offset: Vec2::new(4.0, 4.0),
                    blur: 0.0,
                    color: rgb(213, 0, 249),
                },
                UiTextShadow {
                    offset: Vec2::new(0.0, 10.0),
                    blur: 14.0,
                    color: [0.0, 0.0, 0.0, 0.9],
                },
            ],
            ..UiTextEffects::default()
        });
        let t = line(o, "VICTORY 原点", 590.0, white);
        o.ui.text_effects.push(UiTextEffects {
            text: t,
            shadows: vec![UiTextShadow {
                offset: Vec2::new(0.0, 6.0),
                blur: 10.0,
                color: [0.0, 0.0, 0.0, 0.8],
            }],
            outline: Some(UiTextOutline {
                width: 3.0,
                color: rgb(62, 39, 35),
                align: UiAlign::Outside,
            }),
            fill: Some(gradient(
                UiGradientKind::Linear,
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 1.0),
                &[
                    (0.0, rgb(255, 245, 157)),
                    (0.55, rgb(255, 193, 7)),
                    (1.0, rgb(230, 81, 0)),
                ],
            )),
            fill_box: [70.0, 590.0 - 60.0, 700.0, 80.0],
            ..UiTextEffects::default()
        });
    });
    match err {
        Some(e) => Err(e),
        None => Ok(scene),
    }
}

/// An emblem in the SVG subset: a gradient shield with a stroked rim, a
/// star with round joins, a dashed ring of arcs, and an even-odd hole.
pub const EMBLEM_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120">
  <defs>
    <linearGradient id="metal" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#ffe082"/>
      <stop offset="1" stop-color="#ff6f00"/>
    </linearGradient>
    <radialGradient id="glow" cx="60" cy="56" r="40" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#4fc3f7"/>
      <stop offset="1" stop-color="#1a237e"/>
    </radialGradient>
  </defs>
  <path d="M60 6 L108 24 C108 70 90 98 60 114 C30 98 12 70 12 24 Z" fill="url(#metal)" stroke="#3e2723" stroke-width="4" stroke-linejoin="round"/>
  <path fill-rule="evenodd" fill="url(#glow)" d="M60 22 a34 34 0 1 0 0.01 0 Z M60 38 a18 18 0 1 1 -0.01 0 Z"/>
  <polygon points="60,40 65,53 79,53 68,61 72,75 60,67 48,75 52,61 41,53 55,53" fill="#fffde7" stroke="#ff6f00" stroke-width="2" stroke-linejoin="round"/>
  <circle cx="60" cy="56" r="44" fill="none" stroke="#fff" stroke-opacity="0.7" stroke-width="2" stroke-dasharray="6 4"/>
  <g transform="translate(60 102) rotate(45)"><rect x="-5" y="-5" width="10" height="10" rx="2" fill="#3e2723"/></g>
</svg>"##;

/// `ui-vector`: the emblem at 64, 160 and 360 CSS px, turned 20° at 160,
/// and tinted at 96.
#[cfg(not(target_arch = "wasm32"))]
pub fn vector() -> Result<RenderScene, String> {
    let data = og_cook::cook::vector::tessellate(EMBLEM_SVG, None)?;
    let mut err = None;
    let scene = overlay_scene(|scene, o| {
        let v = scene.add_ui_vector(data);
        let draw = |o: &mut Overlay, rect: [f32; 4], transform: [f32; 4], tint: [f32; 4]| {
            o.ui.vectors.push(og_render::UiVectorDraw {
                vector: v,
                rect,
                transform,
                tint,
                layer: 0,
                clip: None,
            });
        };
        let id = [1.0, 0.0, 0.0, 1.0];
        o.ui.shapes.push(UiShape::rect(
            [20.0, 20.0, 1240.0, 680.0],
            24.0,
            [0.08, 0.09, 0.12, 1.0],
        ));
        draw(o, [60.0, 60.0, 64.0, 64.0], id, [1.0; 4]);
        draw(o, [160.0, 60.0, 160.0, 160.0], id, [1.0; 4]);
        let (s, c) = 20f32.to_radians().sin_cos();
        draw(o, [360.0, 60.0, 160.0, 160.0], [c, s, -s, c], [1.0; 4]);
        draw(o, [60.0, 260.0, 96.0, 96.0], id, [0.4, 0.9, 1.0, 1.0]);
        draw(o, [600.0, 40.0, 640.0, 640.0], id, [1.0; 4]);
        if o.ui.vectors.is_empty() {
            err = Some("no vector drawn".to_string());
        }
    });
    match err {
        Some(e) => Err(e),
        None => Ok(scene),
    }
}

/// `ui-world` (`docs/ui.md` §14, og-render's `ui::mass`): mass elements and
/// occluders over a lit 3D scene. The camera looks down -z from (0, 2, 12)
/// at a wall 4 m wide and 4 m tall at z = 0 (centred on the screen); the
/// mass elements' tick is 100.
///
/// - Damage numbers: 300 in a fan over the ground, each started 0 to 47
///   ticks ago of a 48-tick pop, rise and fade, every tenth a critical hit
///   at 1.6 times the size, white, gold or red, outlined.
/// - Nameplates, a string per instance, faded to a quarter behind the
///   scene: `Brenna` at (0, 2, -5) behind the wall, `Aldric` at (5, 2, -5)
///   against the sky, `Corin` at (-5, 2, -5) on the left.
/// - Overhead bars (screen space, 120×12 CSS px from (80, 80) down in 24
///   px rows): fills of 0.5, 0.25 and 0.9 with ghosts of 0.75, 0.6 and 0.
/// - Icons: the four quadrants of the sprite atlas at 48 px, top right.
/// - Occluders: two 160×60 panels at the screen's centre row, the left one
///   (centred at 320, 360) at a depth of 20 m with only sky behind it, the
///   right (centred at 640, 360) at 20 m behind the wall (12 m), hidden;
///   both over the numbers.
/// - A score `+12.3k` in screen space, compact, top centre.
/// - Confetti: 120 cyan particles from (300, 200), 20 ticks into a 40-tick
///   life, flung 60 to 180 px with drag, gravity and spin.
pub fn world(
    font: impl FnOnce(&mut RenderScene) -> Result<FontHandle, String>,
    atlas: impl FnOnce(&mut RenderScene) -> TextureHandle,
) -> Result<RenderScene, String> {
    use og_render::{
        Camera, Material, Projection, UiMassBatch, UiMassEase, UiMassInstance, UiMassKey,
        UiMassKind, UiMassSpace, UiMassTemplate, UiOccluder, UiOcclusion, shapes,
    };
    use std::sync::Arc;
    let mut scene = RenderScene::new();
    let font = font(&mut scene)?;
    let atlas = atlas(&mut scene);
    crate::scenes::ground(&mut scene, 60.0, 1.0, 0.8);
    let wall = scene.add_mesh(shapes::cuboid(Vec3::new(4.0, 4.0, 0.4)));
    let stone = scene.add_material(Material {
        base_color: [0.55, 0.5, 0.45, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    scene.add_instance(
        wall,
        stone,
        og_core::glam::Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)),
    );
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.0, 12.0),
        target: Vec3::new(0.0, 2.0, 0.0),
        up: Vec3::Y,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        near: 0.1,
        far: 200.0,
        ..Default::default()
    };
    scene.environment.background = Background::Color(Vec3::new(0.25, 0.35, 0.55));
    scene.post.tonemap = Tonemap::None;
    let mut o = Overlay::new(W, H, 0.0);
    o.ui.tick = 100;
    let outline = Some((2.0, [0.02, 0.02, 0.03, 1.0]));
    // Damage numbers.
    let numbers = Arc::new(UiMassTemplate {
        kind: UiMassKind::Number {
            decimals: 0,
            plus: false,
            compact: false,
        },
        font: Some(font),
        size: Vec2::new(22.0, 22.0),
        pivot: Vec2::new(0.5, 1.0),
        lifetime: 48,
        keys: vec![
            UiMassKey {
                t: 0.0,
                scale: 1.8,
                ..Default::default()
            },
            UiMassKey {
                t: 0.15,
                scale: 1.0,
                offset: Vec2::new(0.0, -10.0),
                ease: UiMassEase::Back,
                ..Default::default()
            },
            UiMassKey {
                t: 0.7,
                offset: Vec2::new(0.0, -36.0),
                ease: UiMassEase::Out,
                ..Default::default()
            },
            UiMassKey {
                t: 1.0,
                offset: Vec2::new(0.0, -48.0),
                alpha: 0.0,
                ..Default::default()
            },
        ],
        drift: 14.0,
        distance: Vec3::new(12.0, 0.5, 2.0),
        outline,
        ..Default::default()
    });
    let colors = [
        [1.0, 1.0, 1.0, 1.0],
        [1.0, 0.75, 0.15, 1.0],
        [1.0, 0.25, 0.2, 1.0],
    ];
    let instances = (0..300u32)
        .map(|i| {
            let a = i as f32 * 2.399_963;
            let r = 2.0 + (i as f32).sqrt() * 0.75;
            UiMassInstance {
                position: [
                    a.cos() * r,
                    0.5 + (i % 7) as f32 * 0.3,
                    -a.sin().abs() * r + 4.0,
                ],
                value: (17 + (i * 7919) % 983) as f32,
                color: colors[(i % 3) as usize],
                start: 100 - (i * 13) % 48,
                seed: i,
                scale: if i % 10 == 0 { 1.6 } else { 1.0 },
                ..Default::default()
            }
        })
        .collect();
    o.ui.mass.push(UiMassBatch {
        template: numbers,
        space: UiMassSpace::World,
        instances,
        layer: 10,
        clip: None,
    });
    // Nameplates.
    let names = Arc::new(UiMassTemplate {
        kind: UiMassKind::Text {
            strings: vec!["Aldric".into(), "Brenna".into(), "Corin".into()],
        },
        font: Some(font),
        size: Vec2::new(28.0, 28.0),
        pivot: Vec2::new(0.5, 0.5),
        occlusion: UiOcclusion::Fade { alpha: 0.25 },
        outline,
        ..Default::default()
    });
    let plate = |p: [f32; 3], s: f32| UiMassInstance {
        position: p,
        value: s,
        color: [0.6, 1.0, 0.6, 1.0],
        scale: 1.0,
        ..Default::default()
    };
    o.ui.mass.push(UiMassBatch {
        template: names,
        space: UiMassSpace::World,
        instances: vec![
            plate([5.0, 2.0, -5.0], 0.0),
            plate([0.0, 2.0, -5.0], 1.0),
            plate([-5.0, 2.0, -5.0], 2.0),
        ],
        layer: 11,
        clip: None,
    });
    // Overhead bars, in screen space.
    let bars = Arc::new(UiMassTemplate {
        kind: UiMassKind::Bar {
            back: [0.02, 0.02, 0.03, 0.85],
            ghost: [1.0, 0.9, 0.9, 1.0],
            radius: 6.0,
        },
        size: Vec2::new(120.0, 12.0),
        pivot: Vec2::ZERO,
        ..Default::default()
    });
    let bar = |y: f32, v: f32, ghost: f32| UiMassInstance {
        position: [80.0, y, 0.0],
        value: v,
        color: [0.1, 0.8, 0.2, 1.0],
        scale: 1.0,
        aux: [0.0, 0.0, ghost, 0.0],
        ..Default::default()
    };
    o.ui.mass.push(UiMassBatch {
        template: bars,
        space: UiMassSpace::Screen,
        instances: vec![
            bar(80.0, 0.5, 0.75),
            bar(104.0, 0.25, 0.6),
            bar(128.0, 0.9, 0.0),
        ],
        layer: 12,
        clip: None,
    });
    // Icons.
    let icons = Arc::new(UiMassTemplate {
        kind: UiMassKind::Icon {
            frames: vec![
                [0.0, 0.0, 0.5, 0.5],
                [0.5, 0.0, 1.0, 0.5],
                [0.0, 0.5, 0.5, 1.0],
                [0.5, 0.5, 1.0, 1.0],
            ],
        },
        texture: Some(atlas),
        size: Vec2::new(48.0, 48.0),
        pivot: Vec2::ZERO,
        ..Default::default()
    });
    o.ui.mass.push(UiMassBatch {
        template: icons,
        space: UiMassSpace::Screen,
        instances: (0..4)
            .map(|i| UiMassInstance {
                position: [1000.0 + 56.0 * i as f32, 60.0, 0.0],
                value: i as f32,
                color: [1.0; 4],
                scale: 1.0,
                ..Default::default()
            })
            .collect(),
        layer: 12,
        clip: None,
    });
    // A compact score.
    let score = Arc::new(UiMassTemplate {
        kind: UiMassKind::Number {
            decimals: 0,
            plus: true,
            compact: true,
        },
        font: Some(font),
        size: Vec2::new(40.0, 40.0),
        pivot: Vec2::new(0.5, 0.0),
        outline,
        ..Default::default()
    });
    o.ui.mass.push(UiMassBatch {
        template: score,
        space: UiMassSpace::Screen,
        instances: vec![UiMassInstance {
            position: [640.0, 40.0, 0.0],
            value: 12_345.0,
            color: [1.0, 0.85, 0.3, 1.0],
            scale: 1.0,
            ..Default::default()
        }],
        layer: 12,
        clip: None,
    });
    // A confetti burst in screen space, half way through its life.
    let confetti = Arc::new(UiMassTemplate {
        kind: UiMassKind::Particle {
            shape: og_render::UiParticleShape::Square,
        },
        size: Vec2::new(10.0, 6.0),
        pivot: Vec2::new(0.5, 0.5),
        lifetime: 40,
        gravity: 60.0,
        drag: true,
        keys: vec![
            UiMassKey {
                t: 0.0,
                ..Default::default()
            },
            UiMassKey {
                t: 1.0,
                alpha: 0.0,
                ease: UiMassEase::In,
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    o.ui.mass.push(UiMassBatch {
        template: confetti,
        space: UiMassSpace::Screen,
        instances: (0..120u32)
            .map(|i| {
                let a = i as f32 * 2.399_963;
                let r = 60.0 + (i % 5) as f32 * 30.0;
                UiMassInstance {
                    position: [300.0, 200.0, 0.0],
                    color: [0.1, 0.9, 1.0, 1.0],
                    start: 80,
                    seed: i,
                    scale: 1.0,
                    velocity: [a.cos() * r, a.sin() * r, 1.0 + (i % 3) as f32, a],
                    ..Default::default()
                }
            })
            .collect(),
        layer: 12,
        clip: None,
    });
    // Occluded panels.
    for (x, depth) in [(320.0, 20.0), (640.0, 20.0)] {
        let rect = o.add_clip([0.0, 0.0, W, H]);
        o.ui.occluders.push(UiOccluder {
            rect,
            at: Vec2::new(x, 360.0),
            depth,
            alpha: 0.0,
            bias: 0.25,
            soft: 0.5,
        });
        o.ui.shapes.push(UiShape {
            clip: Some(rect),
            layer: 20,
            ..UiShape::rect([x - 80.0, 330.0, 160.0, 60.0], 12.0, [0.9, 0.3, 0.6, 1.0])
        });
    }
    scene.overlay = Some(o);
    Ok(scene)
}

/// `ui-panel` (`docs/ui.md` §14, world panels): a UI layer drawn into a
/// render target instead of the overlay, the target the emissive map of a
/// 3 × 2 m quad turned 25° about y in front of the camera. The panel's
/// rect, 480 × 320 CSS px, is on the screen at (760, 380), where nothing
/// of it may show (a panel composites nowhere); its content, in layer 200:
/// a teal card filling it, a green bar across its lower third, a red disc
/// at its top left. Two frames: the scene samples the target the frame
/// after the UI drew it.
pub fn panel() -> RenderScene {
    use og_render::{Camera, Material, Projection, RenderTargetDesc, TargetFormat, shapes};
    let mut scene = RenderScene::new();
    crate::scenes::ground(&mut scene, 40.0, 1.0, 0.8);
    let target = scene.add_render_target(RenderTargetDesc {
        width: 480,
        height: 320,
        format: TargetFormat::Rgba16Float,
        depth: false,
    });
    let quad = scene.add_mesh(shapes::plane_xy(3.0, 2.0));
    let screen = scene.add_material(Material {
        base_color: [0.0, 0.0, 0.0, 1.0],
        emissive: [1.0; 3],
        emissive_texture: Some(target.color),
        ..Default::default()
    });
    scene.add_instance(
        quad,
        screen,
        og_core::glam::Mat4::from_rotation_translation(
            og_core::glam::Quat::from_rotation_y(25f32.to_radians()),
            Vec3::new(0.0, 1.5, 0.0),
        ),
    );
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.5, 5.0),
        target: Vec3::new(0.0, 1.5, 0.0),
        up: Vec3::Y,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        near: 0.1,
        far: 100.0,
        ..Default::default()
    };
    scene.environment.background = Background::Color(Vec3::new(0.25, 0.35, 0.55));
    scene.post.tonemap = Tonemap::None;
    let mut o = Overlay::new(W, H, 0.0);
    let (x, y) = (760.0, 380.0);
    o.ui.layers.push(og_render::UiLayer {
        capture: [200, 209],
        rect: [x, y, 480.0, 320.0],
        material: og_render::UiMaterial::Plain,
        params: [[0.0; 4]; og_render::UI_MATERIAL_PARAMS],
        mask: None,
        target: Some(target.color),
    });
    let mut put = |s: UiShape| {
        o.ui.shapes.push(UiShape { layer: 200, ..s });
    };
    put(UiShape::rect(
        [x, y, 480.0, 320.0],
        0.0,
        [0.0, 0.35, 0.4, 1.0],
    ));
    put(UiShape::rect(
        [x + 40.0, y + 220.0, 400.0, 50.0],
        12.0,
        [0.1, 0.9, 0.2, 1.0],
    ));
    put(UiShape {
        geometry: og_render::UiGeometry::Ellipse,
        ..UiShape::rect([x + 30.0, y + 30.0, 80.0, 80.0], 0.0, [0.95, 0.1, 0.1, 1.0])
    });
    scene.overlay = Some(o);
    scene
}
