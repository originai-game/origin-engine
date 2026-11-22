//! A narrow spot still lights fog between the froxel columns' centre rays.

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, Fog, FogVolume, LightDesc, LightKind, Material, Projection, RenderDesc,
    RenderScene, VolumetricFog, VolumetricMethod, render_to_rgba, shapes,
};

#[test]
#[ignore = "requires GPU"]
fn global_distance_fog_keeps_the_plain_start_ramp_and_opacity_contract() {
    let mut scene = RenderScene::new();
    scene.frame.camera = Camera {
        position: Vec3::ZERO,
        target: -Vec3::Z,
        projection: Projection::Perspective {
            fov_y_degrees: 62.0,
        },
        far: 500.0,
        ..Default::default()
    };
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.environment.intensity = 0.0;
    scene.lighting.light.intensity = 0.0;
    scene.lighting.light.cast_shadows = false;
    let material = scene.add_material(Material {
        base_color: [0.0, 0.0, 0.0, 1.0],
        emissive: [0.05; 3],
        ..Default::default()
    });
    // Opaque panels inside and beyond the 48 m grid; the sample pixels
    // also distinguish Euclidean camera distance from view depth.
    for (x, width, depth) in [
        (0.0, 1000.0, 180.0),
        (-53.0, 100.0, 24.0),
        (35.0, 60.0, 75.0),
    ] {
        let mesh = scene.add_mesh(shapes::plane_xy(width, 1000.0));
        scene.add_instance(
            mesh,
            material,
            Mat4::from_translation(Vec3::new(x, 0.0, -depth)),
        );
    }
    let color = Vec3::new(0.3, 0.45, 0.7);
    for method in [VolumetricMethod::Froxel, VolumetricMethod::Raymarch] {
        let desc = RenderDesc {
            width: 128,
            height: 72,
            volumetric: Some(method),
            ..Default::default()
        };
        for (start, end, max_opacity) in [
            (250.0, 0.0, 1.0),
            (11.7, 0.0, 1.0),
            (11.7, 31.3, 1.0),
            (11.7, 93.1, 1.0),
            (11.7, 93.1, 0.35),
            (0.0, 0.0001, 1.0),
            (0.0, 0.0, 0.0),
        ] {
            let fog = Fog {
                color,
                sun_color: color,
                density: 0.008,
                start,
                end,
                max_opacity,
                ..Default::default()
            };
            scene.atmosphere.fog = Some(fog);
            scene.atmosphere.volumetric_fog = None;
            let plain = render_to_rgba(&desc, &scene).expect("plain fog control");
            scene.atmosphere.volumetric_fog = Some(VolumetricFog {
                ambient: color,
                density: fog.density,
                start,
                end,
                max_opacity,
                scattering: 0.0,
                ..Default::default()
            });
            let lit = render_to_rgba(&desc, &scene).expect("lit fog without direct light");
            for x in [10, 64, 100] {
                let at = (36 * 128 + x) * 4;
                for c in 0..3 {
                    assert!(
                        plain.pixels[at + c].abs_diff(lit.pixels[at + c]) <= 3,
                        "{method:?}: start={start} end={end} cap={max_opacity} pixel {x} channel {c}: plain {} volume {}",
                        plain.pixels[at + c],
                        lit.pixels[at + c]
                    );
                }
            }
        }
        // Moving the distance start must not move height fog or a local
        // volume. The ordinary fog is replaced in both of these views.
        for local in [false, true] {
            scene.atmosphere.volumetric_fog = Some(VolumetricFog {
                ambient: color,
                density: 0.0,
                height_density: if local { 0.0 } else { 0.015 },
                scattering: 0.0,
                ..Default::default()
            });
            scene.atmosphere.fog_volumes = if local {
                vec![FogVolume {
                    at: og_env::volume::Placement::at(Vec3::ZERO),
                    volume: None,
                    color: Vec3::ONE,
                    density: 0.015,
                    scattering: 0.0,
                }]
            } else {
                Vec::new()
            };
            let zero = render_to_rgba(&desc, &scene).expect("independent fog");
            scene.atmosphere.volumetric_fog.as_mut().unwrap().start = 250.0;
            let delayed = render_to_rgba(&desc, &scene).expect("distance fog delayed");
            assert_eq!(
                zero.pixels, delayed.pixels,
                "{method:?}: start changed independent fog (local={local})"
            );
        }
        scene.atmosphere.fog_volumes.clear();
    }
}

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
    let image = render_to_rgba(
        &RenderDesc {
            width: 640,
            height: 360,
            ..Default::default()
        },
        &scene,
    )
    .expect("fog frame");
    let lit = image
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|c| c[0] > 0 || c[1] > 0 || c[2] > 0)
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
                image.pixels[at..at + 3].iter().any(|c| *c > 0),
                "{method:?}: the beam vanished in column {x}"
            );
        }
        // Outside the cone, behind its apex, and beyond its finite range
        // stay dark: a global fog glow cannot satisfy the beam check.
        for (x, y) in [(320, 150), (200, 179), (500, 179)] {
            let at = (y * 640 + x) * 4;
            assert_eq!(&image.pixels[at..at + 3], &[0, 0, 0], "{method:?}: {x},{y}");
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
            image.pixels[at..at + 3].iter().any(|c| *c > 0),
            "{method:?}: the eye inside a spot lost its beam"
        );
    }
}
