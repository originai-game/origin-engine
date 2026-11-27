//! A narrow spot still lights fog between the froxel columns' centre rays.

use glam::Vec3;
use og_render::{
    Background, Camera, LightDesc, LightKind, Projection, RenderDesc, RenderScene, VolumetricFog,
    VolumetricMethod, render_to_rgba,
};

#[test]
#[ignore = "requires GPU"]
fn a_subfroxel_spot_beam_does_not_disappear_between_columns() {
    let mut scene = RenderScene::new();
    scene.frame.camera = Camera {
        position: Vec3::ZERO,
        target: -Vec3::Z,
        projection: Projection::Perspective {
            fov_y_degrees: 60.0,
        },
        ..Default::default()
    };
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.environment.intensity = 0.0;
    scene.lighting.light.intensity = 0.0;
    scene.lighting.light.cast_shadows = false;
    scene.atmosphere.volumetric_fog = Some(VolumetricFog {
        density: 0.1,
        scattering: 1.0,
        anisotropy: 0.0,
        distance: 20.0,
        ..Default::default()
    });
    // At z=-10 the adjacent centre rays lie at y=±0.06415 m.
    // This beam is between them, visible at 640x360 (up to 1.75 pixels
    // across), but no centre ray intersects its cone before its range.
    scene.lighting.lights.push(LightDesc {
        kind: LightKind::Spot {
            direction: Vec3::X,
            angle_degrees: 0.007f32.to_degrees(),
            penumbra: 0.5,
        },
        position: Vec3::new(-2.0, 0.032, -10.0),
        intensity: 100_000.0,
        range: 4.0,
        ..Default::default()
    });
    let desc = RenderDesc {
        width: 640,
        height: 360,
        ..Default::default()
    };
    // The display pass dithers even black to 0 or 1. Compare with the
    // same unlit pixel, so neither dither nor a global fog glow can pass
    // for a beam (or fail an outside-cone check).
    scene.lighting.lights[0].intensity = 0.0;
    let dark = render_to_rgba(&desc, &scene).expect("unlit fog frame");
    scene.lighting.lights[0].intensity = 100_000.0;
    let image = render_to_rgba(&desc, &scene).expect("fog frame");
    let lit = image
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(dark.pixels.as_chunks::<4>().0)
        .filter(|(lit, dark)| lit[..3] != dark[..3])
        .count();
    assert!(
        lit > 0,
        "the visible spot beam vanished between centre rays"
    );

    // Every column in this interior stretch crosses the cone. Neither
    // method may lose it between lateral or depth samples; spreading one
    // cell's light across its neighbours does not fill these columns.
    scene.frame.camera.position.y = 0.016;
    scene.frame.camera.target.y = 0.016;
    for method in [VolumetricMethod::Froxel, VolumetricMethod::Raymarch] {
        let image = render_to_rgba(
            &RenderDesc {
                width: 640,
                height: 360,
                volumetric: Some(method),
                ..Default::default()
            },
            &scene,
        )
        .expect("fog frame");
        for x in 300..340 {
            let at = (179 * 640 + x) * 4;
            assert!(
                image.pixels[at..at + 3] != dark.pixels[at..at + 3],
                "{method:?}: the beam vanished in column {x}"
            );
        }
        // Outside the cone, behind its apex, and beyond its finite range
        // stay dark: a global fog glow cannot satisfy the beam check.
        for (x, y) in [(320, 150), (200, 179), (500, 179)] {
            let at = (y * 640 + x) * 4;
            assert_eq!(
                image.pixels[at..at + 3],
                dark.pixels[at..at + 3],
                "{method:?}: {x},{y}"
            );
        }
    }

    // The eye inside a wider cone is the other sign of the intersection
    // quadratic. Clipping must retain its forward half, not reject it.
    scene.lighting.lights[0].position = Vec3::new(0.0, 0.016, 1.0);
    scene.lighting.lights[0].kind = LightKind::Spot {
        direction: -Vec3::Z,
        angle_degrees: 20.0,
        penumbra: 0.5,
    };
    for method in [VolumetricMethod::Froxel, VolumetricMethod::Raymarch] {
        let image = render_to_rgba(
            &RenderDesc {
                width: 640,
                height: 360,
                volumetric: Some(method),
                ..Default::default()
            },
            &scene,
        )
        .expect("fog from inside the cone");
        let at = (179 * 640 + 320) * 4;
        assert!(
            image.pixels[at..at + 3] != dark.pixels[at..at + 3],
            "{method:?}: the eye inside a spot lost its beam"
        );
    }
}
