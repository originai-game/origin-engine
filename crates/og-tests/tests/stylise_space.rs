//! Stylise's object space (`StyliseSpace::Object`): a pattern laid in each
//! instance's own space moves with the instance. The same hatched box and
//! sphere drawn twice, the second time moved (and the camera with them, so
//! they cover the same pixels): in object space the two frames agree but
//! for rounding at a few edges; in world space (`StyliseSpace::Surface`)
//! the strokes stay where they were in the world and the objects slide
//! through them, so the frames differ wherever there are strokes.
//!
//! Ways this could fail: the object pass is not drawn or not bound (the
//! paint and lines passes read the "no instance" texel and fall back to
//! world space: the object frames differ as the world ones do); it
//! writes the world position or the mesh position without the instance's
//! scale (the strokes slide, or grow with the scale); its depth test
//! misses the prepass's surface (no pixel covered, the same fallback).
//! Broken once by binding the placeholder in place of `object-space`: the
//! object frames differed as the world ones do (1.07 % of the pixels by
//! more than 8/255, against none).

use glam::{Mat4, Quat, Vec3};
use og_render::{
    Antialias, Background, Camera, DirectionalLight, Exposure, Hatching, RenderDesc, RenderScene,
    RenderTarget, Renderer, StandardMaterial, StyliseSpace, shapes,
};
use og_tests::scenes;

fn desc() -> RenderDesc {
    RenderDesc {
        width: 320,
        height: 180,
        antialias: Antialias::Off,
        ..scenes::base_desc()
    }
}

/// A turned, stretched box and a sphere, hatched in `space`, everything
/// (camera included) moved by `offset`.
fn scene(offset: Vec3, space: StyliseSpace) -> RenderScene {
    let mut scene = RenderScene::new();
    let clay = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.8, 0.78, 0.74),
        roughness: 0.9,
        ..Default::default()
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let at = Mat4::from_translation(offset);
    scene.add_instance(
        cube,
        clay,
        at * Mat4::from_scale_rotation_translation(
            Vec3::new(1.8, 0.9, 0.9),
            Quat::from_rotation_y(0.6) * Quat::from_rotation_x(0.3),
            Vec3::new(-0.9, 0.6, 0.0),
        ),
    );
    scene.add_instance(
        sphere,
        clay,
        at * Mat4::from_scale_rotation_translation(
            Vec3::splat(1.4),
            Quat::IDENTITY,
            Vec3::new(1.1, 0.7, -0.3),
        ),
    );
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(0.5, 0.7, 0.45),
        intensity: 3.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.environment.background = Background::Color(Vec3::new(0.86, 0.84, 0.78));
    scene.camera = Camera {
        position: Vec3::new(0.3, 2.0, 5.0) + offset,
        target: Vec3::new(0.0, 0.6, 0.0) + offset,
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene.stylise.hatching = Some(Hatching {
        strength: 0.9,
        scale: 5.0,
        levels: 4,
        angle_degrees: 30.0,
        color: [0.05, 0.045, 0.04],
        space,
        depth: 0.0,
    });
    scene
}

/// The share of pixels where any channel differs by more than 8 of 255.
fn differing(a: &[u8], b: &[u8]) -> f64 {
    let n = a
        .chunks(4)
        .zip(b.chunks(4))
        .filter(|(p, q)| p.iter().zip(*q).any(|(x, y)| x.abs_diff(*y) > 8))
        .count();
    n as f64 / (a.len() / 4) as f64
}

/// The share of pixels that differ between the scene and the scene moved.
fn moved(renderer: &mut Renderer, space: StyliseSpace) -> f64 {
    let mut frame = |offset: Vec3| {
        renderer
            .render(&scene(offset, space), &RenderTarget::Offscreen)
            .expect("frame");
        renderer.read_offscreen().expect("capture").pixels
    };
    let a = frame(Vec3::ZERO);
    let b = frame(Vec3::new(0.37, 0.11, -0.23));
    differing(&a, &b)
}

#[test]
fn a_moving_object_carries_its_strokes_in_object_space() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    let object = moved(&mut renderer, StyliseSpace::Object);
    let world = moved(&mut renderer, StyliseSpace::Surface);
    println!("pixels that differ once moved: object {object:.4}, world {world:.4}");
    assert!(
        world > 0.005,
        "world space: the objects slide through the strokes ({world})"
    );
    assert!(
        object < world / 20.0,
        "object space: the strokes move with the objects ({object})"
    );
}
