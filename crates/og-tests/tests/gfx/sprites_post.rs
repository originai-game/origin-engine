use super::*;

/// A sprite drawn from an array shows the layer it names, sampled with the
/// array's filter; a layer past the array's count, or an array that was
/// removed, is refused.
#[test]
#[ignore = "requires GPU"]
fn a_sprite_draws_the_layer_of_its_texture_array() {
    use og_render::{
        ColorSpace, SpriteArray, TextureArrayData, TextureData, TextureFilter, TextureSource,
    };
    let solid = |rgb: [u8; 3]| {
        TextureSource::Rgba(TextureData {
            width: 4,
            height: 4,
            rgba: [rgb[0], rgb[1], rgb[2], 255].repeat(16),
            color_space: ColorSpace::Srgb,
            mips: og_render::Mips::None,
        })
    };
    let mut scene = RenderScene::new();
    let array = scene
        .add_texture_array(TextureArrayData {
            layers: vec![solid([255, 0, 0]), solid([0, 255, 0]), solid([0, 0, 255])],
        })
        .expect("three layers");
    assert!(scene.set_texture_array_filter(array, TextureFilter::Nearest));
    for (i, (x, layer)) in [(-3.0, 2u32), (3.0, 1)].into_iter().enumerate() {
        scene.add_sprite(Sprite {
            position: Vec3::new(x, 0.0, 0.0),
            size: Vec2::splat(4.0),
            ..Default::default()
        });
        scene.sprite_arrays.push(SpriteArray {
            sprite: i as u32,
            array,
            layer,
        });
    }
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    let desc = scenes::base_desc();
    let (img, stats) = frame(&desc, &scene, 1);
    let at = |x: u32| {
        let i = ((img.height / 2 * img.width + x) * 4) as usize;
        [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
    };
    assert_eq!(
        at(img.width * 3 / 10),
        [0, 0, 255],
        "the left sprite: layer 2"
    );
    assert_eq!(
        at(img.width * 7 / 10),
        [0, 255, 0],
        "the right sprite: layer 1"
    );
    assert_eq!(stats.sprite_draws, 1, "both layers in one draw");

    let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
    scene.sprite_arrays[0].layer = 3;
    match renderer.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::InvalidData(m)) => assert!(m.contains("layer 3"), "{m}"),
        other => panic!("a layer past the array: {:?}", other.map(|_| ())),
    }
    scene.sprite_arrays[0].layer = 0;
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("frame");
    assert!(scene.remove_texture_array(array));
    match renderer.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::InvalidHandle(m)) => {
            assert!(m.contains("texture array 0 was removed"), "{m}")
        }
        other => panic!("a sprite of a removed array: {:?}", other.map(|_| ())),
    }
}

/// Solid layers for array tests: red, green, blue.
fn rgb_array(scene: &mut RenderScene) -> og_render::TextureArrayHandle {
    use og_render::{ColorSpace, TextureArrayData, TextureData, TextureSource};
    let solid = |rgb: [u8; 3]| {
        TextureSource::Rgba(TextureData {
            width: 4,
            height: 4,
            rgba: [rgb[0], rgb[1], rgb[2], 255].repeat(16),
            color_space: ColorSpace::Srgb,
            mips: og_render::Mips::None,
        })
    };
    let array = scene
        .add_texture_array(TextureArrayData {
            layers: vec![solid([255, 0, 0]), solid([0, 255, 0]), solid([0, 0, 255])],
        })
        .expect("three layers");
    scene.set_texture_array_filter(array, og_render::TextureFilter::Nearest);
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    array
}

fn pixel(img: &RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

/// A sprite-mode emitter with a texture array draws each particle from a
/// layer: by age (every particle of one burst is the same age, so one
/// layer shows) or at random (several show), in one draw; `round` gives
/// them the soft dot's edge (a big particle's corner stays dark); a
/// billboard emitter with an array is refused. Broken once by drawing the
/// particles without their array (as untextured soft dots): no layer's
/// colour shows.
#[test]
#[ignore = "requires GPU"]
fn array_particles_take_their_layers_by_age_or_at_random() {
    use og_render::{
        Burst, EmitterShape, ParticleEmitter, ParticleRender, ParticleSystem, SheetMode,
        SpriteSheet,
    };
    let run = |mode: SheetMode, round: bool, size: f32, billboard: bool| {
        let mut scene = RenderScene::new();
        let array = rgb_array(&mut scene);
        // Forty small particles over the view, or one big one at the centre.
        let big = size > 1.0;
        let e = scene.add_emitter(ParticleEmitter {
            max_particles: 64,
            shape: match big {
                true => EmitterShape::Point,
                false => EmitterShape::Box {
                    half_extents: Vec3::new(3.0, 3.0, 0.0),
                },
            },
            rate: 0.0,
            bursts: vec![Burst::once(0.0, if big { 1 } else { 40 })],
            looping: false,
            lifetime: [3.0, 3.0],
            speed: [0.0, 0.0],
            size: [size, size],
            gravity: Vec3::ZERO,
            array: Some(array),
            round,
            sheet: SpriteSheet {
                mode,
                ..Default::default()
            },
            blend: og_render::ParticleBlend::Alpha,
            render: if billboard {
                ParticleRender::Billboard
            } else {
                ParticleRender::Sprite { layer: 0 }
            },
            ..Default::default()
        });
        scene
            .particles
            .push(ParticleSystem::new(1, e, Mat4::IDENTITY, 9, 0));
        let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
        // 1.5 s of a 3 s life at 60 ticks a second: the middle layer by age.
        let mut stats = FrameStats::default();
        for t in 0..=90 {
            scene.tick = t;
            scene.particles[0].tick = t;
            stats = renderer.render(&scene, &RenderTarget::Offscreen)?;
        }
        Ok::<_, RenderError>((renderer.read_offscreen().expect("capture"), stats))
    };
    let colours = |img: &RgbaImage| {
        let mut seen = [0usize; 3];
        for p in img.pixels.as_chunks::<4>().0 {
            match [p[0], p[1], p[2]] {
                [255, 0, 0] => seen[0] += 1,
                [0, 255, 0] => seen[1] += 1,
                [0, 0, 255] => seen[2] += 1,
                _ => {}
            }
        }
        seen
    };
    let (img, stats) = run(SheetMode::OverLife { cycles: 1.0 }, false, 0.4, false).expect("frame");
    let seen = colours(&img);
    assert!(
        seen[1] > 1000 && seen[0] == 0 && seen[2] == 0,
        "by age: every particle on layer 1: {seen:?}"
    );
    assert_eq!(stats.particles_cpu, 0, "simulated in compute");
    assert_eq!(stats.sprite_draws, 1, "every layer in one draw");
    let (img, _) = run(SheetMode::Random, false, 0.4, false).expect("frame");
    let seen = colours(&img);
    assert!(
        seen.iter().all(|n| *n > 100),
        "at random: every layer shows: {seen:?}"
    );
    // One big particle at the centre: a round one's corner is dark.
    let (square, _) = run(SheetMode::OverLife { cycles: 1.0 }, false, 9.0, false).expect("frame");
    let (round, _) = run(SheetMode::OverLife { cycles: 1.0 }, true, 9.0, false).expect("frame");
    let corner = |img: &RgbaImage| pixel(img, 640 - 300, 360 - 300);
    assert_eq!(corner(&square), [0, 255, 0], "a square particle's corner");
    assert!(
        corner(&round)[1] < 40,
        "a round one's: {:?}",
        corner(&round)
    );
    match run(SheetMode::Off, false, 0.4, true) {
        Err(RenderError::InvalidData(m)) => assert!(m.contains("sprite-mode"), "{m}"),
        other => panic!("a billboard emitter with an array: {:?}", other.map(|_| ())),
    }
}
