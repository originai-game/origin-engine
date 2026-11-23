//! Small, asset-free optical review fixture. Frames are review artifacts, never references in git.
use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Background, Camera, DepthOfField, DofQuality, Exposure, GateFit, MotionBlur, PhysicalLens,
    Projection, RenderDesc, RenderScene, RenderTarget, Renderer, StandardMaterial, Tonemap, shapes,
};

fn main() {
    let out = std::env::args()
        .nth(1)
        .expect("camera_optics OUTPUT_DIRECTORY");
    std::fs::create_dir_all(&out).unwrap();
    let desc = RenderDesc {
        width: 640,
        height: 360,
        antialias: og_render::Antialias::Off,
        ..og_tests::scenes::base_desc()
    };
    let mut scene = RenderScene::new();
    scene.frame.camera = Camera {
        position: Vec3::ZERO,
        target: -Vec3::Z,
        up: Vec3::Y,
        projection: Projection::Perspective {
            fov_y_degrees: 26.991467,
        },
        near: 0.1,
        far: 100.0,
        exposure: Exposure::linear(1.0),
    };
    scene.lighting.environment.intensity = 0.0;
    scene.lighting.environment.background = Background::Color(Vec3::splat(0.002));
    scene.lighting.light.intensity = 0.0;
    scene.look.post.tonemap = Tonemap::None;
    let white = scene.add_standard_material(StandardMaterial::basic(Vec3::splat(0.8)));
    let red = scene.add_standard_material(StandardMaterial::basic(Vec3::new(0.8, 0.03, 0.01)));
    // Equal angular size, at 2, 5 and 10 metres: asymmetric columns and depth.
    for (column, depth) in [(-0.28, 2.0), (0.0, 5.0), (0.28, 10.0)] {
        let card = scene.add_mesh(shapes::cuboid(Vec3::new(depth * 0.05, depth * 0.14, 0.002)));
        scene.add_instance(
            card,
            white,
            Mat4::from_translation(Vec3::new(column * depth, 0.0, -depth)),
        );
        let dot = scene.add_mesh(shapes::sphere(24, 16));
        scene.add_instance(
            dot,
            red,
            Mat4::from_scale_rotation_translation(
                Vec3::splat(depth * 0.008),
                glam::Quat::IDENTITY,
                Vec3::new(column * depth, depth * 0.10, -depth),
            ),
        );
    }
    let save = |name: &str, renderer: &mut Renderer| {
        let image = renderer.read_offscreen().expect("readback");
        assert!(
            image.pixels.as_chunks::<4>().0.iter().any(|p| p[0] > 120),
            "blank optical fixture"
        );
        image::save_buffer(
            format!("{out}/{name}.png"),
            &image.pixels,
            image.width,
            image.height,
            image::ColorType::Rgba8,
        )
        .unwrap();
    };
    if std::env::args().nth(2).as_deref() == Some("projection") {
        for (name, width, height, gate_fit, shift) in [
            ("physical-centered", 640, 360, GateFit::Vertical, Vec2::ZERO),
            (
                "physical-shifted",
                640,
                360,
                GateFit::Vertical,
                Vec2::new(0.08, -0.1),
            ),
            ("physical-fill", 640, 360, GateFit::Fill, Vec2::ZERO),
            ("physical-portrait", 360, 640, GateFit::Overscan, Vec2::ZERO),
        ] {
            let lens = PhysicalLens {
                focal_length: 50.0,
                sensor_size: Vec2::new(36.0, 24.0),
                gate_fit,
                shift,
                ..Default::default()
            };
            let focal = lens.focal_y(width as f32 / height as f32, 1.0).unwrap();
            scene.frame.camera.projection = Projection::PerspectiveShifted {
                fov_y_degrees: (2.0 * og_core::math::atan(1.0 / focal)).to_degrees(),
                shift,
            };
            let mut renderer = Renderer::new_blocking(&RenderDesc {
                width,
                height,
                ..desc
            })
            .expect("NVIDIA renderer");
            renderer
                .render_capture(&scene, &RenderTarget::Offscreen)
                .unwrap();
            save(name, &mut renderer);
            println!("{name}: {width}x{height}, 50mm/36x24mm, {gate_fit:?}, shift={shift:?}");
        }
        return;
    }
    for (name, dof) in [("sharp", false), ("dof", true)] {
        scene.look.post.depth_of_field = dof.then_some(DepthOfField {
            focus_distance: 5.0,
            focus_range: 0.0,
            max_blur: 0.04,
            falloff: 4.0,
            tilt_shift: 0.0,
            quality: DofQuality::High,
        });
        let mut renderer = Renderer::new_blocking(&desc).expect("NVIDIA renderer");
        renderer
            .render_capture(&scene, &RenderTarget::Offscreen)
            .unwrap();
        save(name, &mut renderer);
    }
    scene.look.post.depth_of_field = None;
    scene.look.post.motion_blur = Some(MotionBlur {
        shutter: 0.5,
        samples: 32,
        max_pixels: 64.0,
    });
    let mut renderer = Renderer::new_blocking(&desc).expect("NVIDIA renderer");
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    scene.objects.previous_transforms = scene
        .objects
        .instances
        .iter()
        .map(|i| i.transform)
        .collect();
    // 60 Hz, 1/120 s centered exposure; translate only the center card by 0.15 m per tick.
    scene.objects.instances[2].transform = Mat4::from_translation(Vec3::new(0.15, 0.0, -5.0));
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    save("object-motion", &mut renderer);
    scene.objects.previous_transforms.clear();
    scene.frame.camera.position.x = 0.15;
    scene.frame.camera.target.x = 0.15;
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    save("camera-motion", &mut renderer);
    println!(
        "optical fixture: 640x360, 50mm, vertical gate 24mm, focus 5m; baseline artistic DoF, shutter 0.5 frame at 60Hz"
    );
}
