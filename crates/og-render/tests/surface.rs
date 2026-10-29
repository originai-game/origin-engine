//! The material style's made surface (`MaterialStyle::surface`, the IDL's
//! `MaterialStyle.surface`): off at zero, and where it is on a lit ball
//! takes its bumps (its normal bent in the mesh's own space) instead of
//! drawing as the plain material.

use glam::{Mat4, Vec3};
use og_render::{
    Camera, Material, MaterialStyle, MaterialSurface, RenderDesc, RenderScene, render_to_rgba,
    shapes,
};

fn ball(style: MaterialStyle) -> RenderScene {
    let mut scene = RenderScene::new();
    let ball = scene.add_mesh(shapes::sphere(48, 32));
    let clay = scene.add_material(Material {
        base_color: [0.7, 0.45, 0.3, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    scene.add_instance(ball, clay, Mat4::from_scale(Vec3::splat(2.0)));
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.4, 3.2),
        target: Vec3::ZERO,
        ..Default::default()
    };
    scene.stylise.material = style;
    scene
}

/// The mean absolute step between horizontal neighbours over the ball's
/// pixels (those that are not the background's): how much the shading
/// varies from one pixel to the next.
fn roughness(rgba: &[u8], width: usize, background: [u8; 4]) -> f64 {
    let px = |i: usize| &rgba[i * 4..i * 4 + 4];
    let (mut sum, mut n) = (0.0, 0usize);
    for i in 0..rgba.len() / 4 - 1 {
        if (i + 1) % width == 0 || px(i) == background || px(i + 1) == background {
            continue;
        }
        sum += (0..3)
            .map(|c| (px(i)[c] as f64 - px(i + 1)[c] as f64).abs())
            .sum::<f64>();
        n += 1;
    }
    sum / n.max(1) as f64
}

#[test]
fn a_made_surface_bends_a_lit_ball_and_is_off_at_zero_strength() {
    let desc = RenderDesc {
        width: 192,
        height: 144,
        ..Default::default()
    };
    let Ok(plain) = render_to_rgba(&desc, &ball(MaterialStyle::default())) else {
        eprintln!("no adapter: skipped");
        return;
    };
    let background: [u8; 4] = plain.pixels[..4].try_into().unwrap();
    let clay = |strength: f32| MaterialStyle {
        surface: MaterialSurface::Clay,
        surface_strength: strength,
        ..Default::default()
    };
    // Off at zero: a surface at strength 0 is the plain frame, byte for byte.
    let zero = render_to_rgba(&desc, &ball(clay(0.0))).expect("renders");
    assert_eq!(plain.pixels, zero.pixels, "strength 0 changed the frame");
    // On: lumps and dents in the ball's own space vary the shading from
    // pixel to pixel far more than the smooth sphere does.
    let made = render_to_rgba(&desc, &ball(clay(1.0))).expect("renders");
    let (a, b) = (
        roughness(&plain.pixels, 192, background),
        roughness(&made.pixels, 192, background),
    );
    assert!(b > a * 1.5 + 0.5, "clay {b:.2} against plain {a:.2}");
}
