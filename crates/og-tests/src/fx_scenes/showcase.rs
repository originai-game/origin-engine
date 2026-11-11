//! Showcase effects, updates and overlay HUD.

use super::*;

// ---------------------------------------------------------------------------
// The showcase's effects.
// ---------------------------------------------------------------------------

/// The showcase's starting tick (its particles and trails are under way).
pub const SHOWCASE_TICK: u64 = 150;

/// Adds the effects to the showcase scene: a campfire between the figures
/// (with its light), scorch decals under the ships, wisps with trails, an
/// energy tether between the ships, ambient occlusion and the grading
/// stack (the sun is out of frame: god rays have their own scene).
pub fn showcase_effects(scene: &mut RenderScene) {
    let fire = campfire(scene);
    logs(scene, Vec3::new(1.4, 0.0, 1.4));
    light_campfire(scene, &fire, Vec3::new(1.4, 0.0, 1.4), 100);
    scene.lighting.lights.push(Light {
        kind: LightKind::Point,
        position: Vec3::new(1.4, 0.6, 1.4),
        color: Vec3::new(1.0, 0.55, 0.25),
        intensity: 5.0,
        range: 7.0,
        cast_shadows: false,
    });
    let atlas = decal_atlas(scene);
    scene.decal_atlas = Some(atlas);
    for (p, s, r) in [
        (Vec3::new(-5.0, 0.0, -5.0), 4.0, 0.2),
        (Vec3::new(5.5, 0.0, -4.0), 3.6, 1.3),
        (Vec3::new(1.4, 0.0, 1.4), 2.2, 0.0),
    ] {
        scene.decals.push(Decal {
            uv_rect: DECAL_SCORCH,
            ..Decal::on_surface(p, Vec3::Y, Vec2::splat(s), 0.8, r)
        });
    }
    scene.decals.push(Decal {
        uv_rect: DECAL_RUNE,
        emissive: Vec3::new(0.8, 2.0, 3.0),
        ..Decal::on_surface(
            Vec3::new(-1.6, 0.0, 2.4),
            Vec3::Y,
            Vec2::splat(1.6),
            0.4,
            0.5,
        )
    });
    let beam_tex = beam_texture(scene);
    scene.beams.push(Beam {
        start: Vec3::new(-4.4, 0.9, -4.8),
        end: Vec3::new(5.0, 0.9, -3.8),
        width: 0.18,
        color: [0.8, 2.4, 5.0, 1.0],
        texture: Some(beam_tex),
        tile: 0.8,
        scroll: 1.5,
        ..Default::default()
    });
    for (i, c) in [[1.0, 0.45, 0.2], [0.3, 0.7, 1.0]].iter().enumerate() {
        let style = scene.add_trail_style(TrailStyle {
            lifetime: 0.7,
            min_distance: 0.02,
            width: Curve::new(&[(0.0, 0.12), (1.0, 0.0)]),
            color: ColorCurve::new(&[
                (0.0, [c[0] * 5.0, c[1] * 5.0, c[2] * 5.0, 1.0]),
                (1.0, [c[0], c[1], c[2], 0.0]),
            ]),
            ..Default::default()
        });
        scene.trails.push(Trail {
            id: 200 + i as u64,
            style,
            position: showcase_wisp(i, SHOWCASE_TICK),
            tick: SHOWCASE_TICK,
        });
    }
    scene.look.post.ao = Some(AmbientOcclusion::default());
    grade(scene);
    if let Some(g) = &mut scene.look.post.film_grain {
        g.intensity = 0.025;
    }
    scene.look.post.chromatic_aberration = Some(ChromaticAberration { intensity: 1.2 });
    // The title glows warm against the graded sky.
    for t in &mut scene.texts {
        t.color = [2.4, 0.9, 0.25, 1.0];
    }
    // The darker ground meters brighter: hold the sky back.
    scene.camera.exposure = Exposure::Auto {
        compensation: -1.1,
        min_ev: -4.0,
        max_ev: 4.0,
        adapt_speed_up: 0.0,
        adapt_speed_down: 0.0,
    };
    tick_all(scene, SHOWCASE_TICK);
}

fn showcase_wisp(i: usize, tick: u64) -> Vec3 {
    let t = tick as f32 / 60.0;
    let a = t * 2.2 + i as f32 * std::f32::consts::PI;
    Vec3::new(
        -0.2 + a.cos() * 1.3,
        2.6 + (t * 3.0 + i as f32).sin() * 0.3,
        0.8 + a.sin() * 1.3,
    )
}

/// The showcase's per-frame update: clocks and wisps.
pub fn showcase_update(scene: &mut RenderScene, frame: u32) {
    let tick = SHOWCASE_TICK + frame as u64;
    tick_all(scene, tick);
    for (i, t) in scene.trails.iter_mut().enumerate() {
        t.position = showcase_wisp(i, tick);
    }
}

/// A HUD in the screen-space overlay over a 3D scene (the `overlay` scene):
/// a panel with a title and a health bar, a rotated marker, a list scrolled
/// half out of its clip rectangle (rows and their text cut at the edge), and
/// a crosshair; 1280×720 CSS pixels at `scale` 0 (fit to the output).
pub fn overlay_hud(scene: &mut RenderScene) -> Result<(), String> {
    let font = crate::scenes::load_font(scene)?;
    let mut o = og_render::Overlay::new(1280.0, 720.0, 0.0);
    let solid = |x: f32, y: f32, w: f32, h: f32, color: [f32; 4], layer: i32| Sprite {
        texture: None,
        position: Vec3::new(x, y, 0.0),
        size: Vec2::new(w, h),
        anchor: Vec2::ZERO,
        color,
        layer,
        ..Default::default()
    };
    let text = |s: &str, x: f32, y: f32, size: f32, color: [f32; 4], layer: i32| Text {
        position: Vec3::new(x, y, 0.0),
        size,
        color,
        layer,
        ..Text::new(font, s)
    };
    // The panel, the title and the health bar (exact token-like colours).
    o.sprites.push(solid(
        24.0,
        24.0,
        320.0,
        120.0,
        [0.02, 0.02, 0.025, 0.85],
        0,
    ));
    o.texts.push(text(
        "Origin 原点",
        44.0,
        64.0,
        30.0,
        [1.0, 1.0, 1.0, 1.0],
        1,
    ));
    o.sprites
        .push(solid(44.0, 92.0, 280.0, 20.0, [0.1, 0.1, 0.1, 1.0], 1));
    o.sprites
        .push(solid(44.0, 92.0, 196.0, 20.0, [0.9, 0.18, 0.05, 1.0], 2));
    o.texts.push(text(
        "HP 70/100",
        48.0,
        132.0,
        16.0,
        [0.8, 0.8, 0.8, 1.0],
        1,
    ));
    // A marker turned 30° clockwise about its centre.
    o.sprites.push(Sprite {
        anchor: Vec2::splat(0.5),
        rotation: 30f32.to_radians(),
        ..solid(1180.0, 90.0, 60.0, 60.0, [1.0, 0.73, 0.0, 1.0], 0)
    });
    // A list in a 260×150 clip, scrolled 23 px: its first and last rows cut.
    let clip = o.add_clip([980.0, 300.0, 260.0, 150.0]);
    o.sprites.push(solid(
        980.0,
        300.0,
        260.0,
        150.0,
        [0.02, 0.02, 0.025, 0.85],
        0,
    ));
    for row in 0..5 {
        let y = 300.0 - 23.0 + row as f32 * 44.0;
        o.sprites.push(Sprite {
            clip: Some(clip),
            ..solid(988.0, y, 244.0, 38.0, [0.2, 0.22, 0.26, 1.0], 1)
        });
        o.texts.push(Text {
            clip: Some(clip),
            ..text(&format!("Item {row}"), 1000.0, y + 27.0, 20.0, [1.0; 4], 2)
        });
    }
    // A crosshair at the centre.
    o.sprites
        .push(solid(638.0, 348.0, 4.0, 24.0, [1.0, 1.0, 1.0, 0.9], 0));
    o.sprites
        .push(solid(628.0, 358.0, 24.0, 4.0, [1.0, 1.0, 1.0, 0.9], 0));
    scene.ui.overlay = Some(o);
    Ok(())
}
