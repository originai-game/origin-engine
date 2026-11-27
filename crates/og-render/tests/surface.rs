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
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 0.4, 3.2),
        target: Vec3::ZERO,
        ..Default::default()
    };
    scene.look.stylise.material = style;
    scene
}

#[test]
#[ignore = "requires GPU"]
fn a_made_surface_bends_a_lit_ball_and_is_off_at_zero_strength() {
    let desc = RenderDesc {
        width: 192,
        height: 144,
        ..Default::default()
    };
    let plain = render_to_rgba(&desc, &ball(MaterialStyle::default())).expect(
        "a GPU adapter: a GPU test, run where there is one (OG_RENDERER=none takes it away)",
    );
    let clay = |strength: f32| MaterialStyle {
        surface: MaterialSurface::Clay,
        surface_strength: strength,
        ..Default::default()
    };
    // Off at zero: a surface at strength 0 is the plain frame, byte for byte.
    let zero = render_to_rgba(&desc, &ball(clay(0.0))).expect("renders");
    assert_eq!(plain.pixels, zero.pixels, "strength 0 changed the frame");
    // On: the surface shader must run. The character of its shading is
    // reviewed on captures, not scored by a pixel-roughness threshold.
    let made = render_to_rgba(&desc, &ball(clay(1.0))).expect("renders");
    assert_ne!(made.pixels, plain.pixels, "strength 1 left the surface off");
}
