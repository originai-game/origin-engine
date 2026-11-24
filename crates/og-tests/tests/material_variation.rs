//! Authored aging and stochastic 2D textures: observable frames, including
//! physical materials, with no weather simulation to supply either effect.
use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Camera, ColorSpace, DirectionalLight, Exposure, MaterialHandle, Mips, Projection, RenderDesc,
    RenderScene, RenderTarget, Renderer, RgbaImage, StandardMaterial, TextureData, shapes,
};
use og_tests::harness::{luma_at, pixels, save};

fn wall(m: StandardMaterial) -> (RenderScene, MaterialHandle) {
    let mut scene = RenderScene::new();
    scene.lighting.light = DirectionalLight {
        direction_to_light: Vec3::Z,
        intensity: 2.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 4.0, 10.0),
        target: Vec3::new(0.0, 4.0, 0.0),
        projection: Projection::Orthographic { height: 8.0 },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let mut mesh = shapes::plane_xy(8.0, 8.0);
    for v in &mut mesh.vertices {
        v.position[1] += 4.0;
        v.uv = [v.position[0], -v.position[1]];
    }
    let mesh = scene.add_mesh(mesh);
    let material = scene.add_standard_material(m);
    scene.add_instance(mesh, material, Mat4::IDENTITY);
    (scene, material)
}

fn renderer() -> Renderer {
    Renderer::new_blocking(&RenderDesc {
        width: 512,
        height: 512,
        ..Default::default()
    })
    .unwrap()
}

fn capture(renderer: &mut Renderer, scene: &RenderScene) -> RgbaImage {
    renderer.render(scene, &RenderTarget::Offscreen).unwrap();
    renderer.read_offscreen().unwrap()
}

#[test]
#[ignore = "requires GPU"]
fn aging_is_ground_up_in_metric_uv_and_stays_dry_without_weather() {
    let mut renderer = renderer();
    for clearcoat in [0.0, 0.4] {
        let plain = StandardMaterial {
            weathering: 0.0,
            clearcoat,
            ..Default::default()
        };
        let (mut scene, h) = wall(plain);
        let clean = capture(&mut renderer, &scene);
        let mut dirty = StandardMaterial {
            grime: 0.9,
            grimeColor: Vec3::splat(0.03),
            grimeHeight: 2.0,
            ..plain
        };
        // Texture transforms must not rescale or translate the height input.
        dirty.mapRepeat = Vec2::new(0.7, 0.125);
        dirty.mapOffset = Vec2::new(0.3, 0.9);
        scene.set_standard_material(h, dirty);
        let aged = capture(&mut renderer, &scene);
        save("material-variation", &format!("grime-{clearcoat}"), &aged);
        // At 0.49 m and 0.99 m above the base, dirt must fade upwards.
        // Display-space luma after tone mapping is not linear coverage:
        // a fixed brightness ratio tests the tone curve, not the gradient.
        let darkening = |y| luma_at(&clean, 256, y) - luma_at(&aged, 256, y);
        assert!(
            darkening(480) > darkening(448) + 1.0 && darkening(448) > 1.0,
            "ground-up gradient missing: lower {}, upper {}",
            darkening(480),
            darkening(448)
        );
        assert!(
            (luma_at(&aged, 256, 256) - luma_at(&clean, 256, 256)).abs() < 1.0,
            "grime reached above its 2 m height"
        );
        scene.set_standard_material(
            h,
            StandardMaterial {
                stains: 1.0,
                ..plain
            },
        );
        let stained = capture(&mut renderer, &scene);
        save(
            "material-variation",
            &format!("stains-{clearcoat}"),
            &stained,
        );
        let darkened = stained
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .zip(clean.pixels.as_chunks::<4>().0)
            .filter(|(a, b)| a[0] as i32 + 3 < b[0] as i32)
            .count();
        assert!(
            darkened > 512 * 8,
            "dry stains were absent: {darkened} darker pixels"
        );
        scene.frame.tick += 600;
        assert_eq!(
            stained.pixels,
            pixels(&mut renderer, &scene),
            "dry stains moved with time"
        );
    }
}

#[test]
#[ignore = "requires GPU"]
fn stochastic_tiling_breaks_repeats_without_washing_out_the_histogram() {
    let mut renderer = renderer();
    let (mut scene, h) = wall(StandardMaterial {
        unlit: 1,
        ..Default::default()
    });
    // Three unequal populations, constant over 4x4 patches. A linear
    // three-image blend narrows this distribution; an untouched map repeats.
    let rgba: Vec<u8> = (0..32 * 32)
        .flat_map(|i| {
            let patch = (i / 32 / 4) * 8 + (i % 32 / 4);
            let v = match (patch * 37 + 11) % 10 {
                0..=4 => 25,
                5..=7 => 150,
                _ => 240,
            };
            [v, v, v, 255]
        })
        .collect();
    let texture = scene.add_texture(TextureData {
        width: 32,
        height: 32,
        rgba,
        color_space: ColorSpace::Linear,
        mips: Mips::Generate,
    });
    let material = StandardMaterial {
        unlit: 1,
        map: texture.slot(),
        ..Default::default()
    };
    scene.set_standard_material(h, material);
    let plain = capture(&mut renderer, &scene);
    scene.set_standard_material(
        h,
        StandardMaterial {
            stochastic: 1.0,
            ..material
        },
    );
    let shuffled = capture(&mut renderer, &scene);
    save("material-variation", "ordinary-tiling", &plain);
    save("material-variation", "stochastic-tiling", &shuffled);
    let stats = |image: &RgbaImage| {
        let v: Vec<f32> = image
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[0] as f32)
            .collect();
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        let variance = v.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / v.len() as f32;
        let mut step = 0.0;
        for y in 0..512 {
            for x in 0..448 {
                step += (luma_at(image, x, y) - luma_at(image, x + 64, y)).abs();
            }
        }
        (mean, variance, step / (512 * 448) as f32)
    };
    let (a, av, repeats) = stats(&plain);
    let (b, bv, changed) = stats(&shuffled);
    println!(
        "ordinary mean/variance/repeat-step {a}/{av}/{repeats}; stochastic {b}/{bv}/{changed}"
    );
    assert!((a - b).abs() < 16.0, "the colour distribution drifted");
    assert!(
        bv > av * 0.65 && bv < av * 1.35,
        "contrast changed: {av} to {bv}"
    );
    assert!(
        repeats < 1.0 && changed > 8.0,
        "the original one-metre repetition remains"
    );
    assert_eq!(
        shuffled.pixels,
        pixels(&mut renderer, &scene),
        "tiling changed over time"
    );
}
