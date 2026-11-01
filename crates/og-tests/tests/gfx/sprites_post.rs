use super::*;

#[test]
fn bloom_matches_its_reference() {
    check("bloom");
}

/// Bloom spreads light from the emissive spheres into the dark around them.
#[test]
fn bloom_spreads_light_beyond_the_emitters() {
    let g = built("bloom");
    let (bloomed, with) = frame(&g.desc, &g.scene, 1);
    let mut scene = g.scene;
    scene.post.bloom = None;
    let (plain, without) = frame(&g.desc, &scene, 1);
    assert!(
        with.draw_calls > without.draw_calls + 4,
        "bloom passes: {} vs {} draws",
        with.draw_calls,
        without.draw_calls
    );
    let brighter = bloomed
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(plain.pixels.as_chunks::<4>().0.iter())
        .filter(|(a, b)| luma(a) > luma(b) + 10.0)
        .count();
    let frac = brighter as f64 / (bloomed.width * bloomed.height) as f64;
    println!("{:.1}% of pixels brighter with bloom", frac * 100.0);
    assert!(frac > 0.05, "bloom brightened only {:.2}%", frac * 100.0);
}

#[test]
fn exposure_matches_its_reference() {
    check("exposure");
}

/// Metered exposure depends on the frame only: the same scene lit 16x
/// brighter comes out at the same brightness, where a manual exposure does not.
#[test]
fn auto_exposure_meters_the_frame() {
    let desc = scenes::base_desc();
    let (dim, _) = frame(&desc, &scenes::dim_scene(1.0), 1);
    let (bright, _) = frame(&desc, &scenes::dim_scene(16.0), 1);
    let (dim2, _) = frame(&desc, &scenes::dim_scene(1.0), 1);
    let (a, b) = (mean_luma(&dim), mean_luma(&bright));
    println!("auto: mean luma {a:.1} (x1) vs {b:.1} (x16)");
    assert!(
        (a / b - 1.0).abs() < 0.08,
        "auto exposure: {a:.1} vs {b:.1}"
    );
    assert_eq!(dim.pixels, dim2.pixels, "the same frame meters the same");
    let manual = |k: f32| {
        let mut s = scenes::dim_scene(k);
        s.camera.exposure = Exposure::Manual { ev: 2.0 };
        mean_luma(&frame(&desc, &s, 1).0)
    };
    let (ma, mb) = (manual(1.0), manual(16.0));
    println!("manual: mean luma {ma:.1} (x1) vs {mb:.1} (x16)");
    assert!(mb > 2.0 * ma, "manual exposure: {ma:.1} vs {mb:.1}");
}

/// 20,000 sprites submitted one by one, drawn in a handful of draws.
#[test]
fn sprites_20k_match_their_reference_in_a_handful_of_draws() {
    let report = check("sprites-20k");
    let s = report.stats;
    assert_eq!(s.sprites, 20_000);
    assert!(s.sprite_draws <= 6, "{} sprite draws", s.sprite_draws);
    assert_eq!(s.draw_calls, s.sprite_draws + 1, "sprites, then post");
}

/// Billboards (`RenderScene::sprite_facing`): the scene's reference (upright
/// trees and figures with sun-facing drop shadows, a turned camera-facing
/// sign, a 2D-lit figure, translucent puffs, and the side view's billboards
/// turned to its own camera); the one behind the camera culled by its
/// bounds; and the translucent puffs drawn back to front whatever order
/// they are submitted in, so swapping them leaves the frame byte for byte.
#[test]
fn billboards_face_each_camera_sort_back_to_front_and_cull_by_bounds() {
    let report = check("sprites-billboard");
    let s = report.stats;
    assert_eq!(
        (s.sprites, s.sprites_culled),
        (8, 1),
        "nine sprites, the tree behind the camera culled"
    );
    let scene = built("sprites-billboard");
    let (image, _) = frame(&scene.desc, &scene.scene, 1);
    let mut swapped = built("sprites-billboard").scene;
    // The two puffs (sprites 5 and 6, both facing the camera) swapped.
    swapped.sprites.swap(5, 6);
    let (other, _) = frame(&scene.desc, &swapped, 1);
    let differ = image
        .pixels
        .chunks(4)
        .zip(other.pixels.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(differ, 0, "{differ} pixels moved with the submission order");
}

/// A glass pane between two solid billboards: the red one behind it, the
/// green one in front, both turned to the camera; `pane` false leaves the
/// pane out. Seen from (0, 1, 6) looking along -z.
fn billboards_and_glass(pane: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.environment.background = Background::Color(Vec3::splat(0.02));
    scene.post.tonemap = Tonemap::None;
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.0, 6.0),
        target: Vec3::new(0.0, 1.0, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    if pane {
        let mesh = scene.add_mesh(shapes::cuboid(Vec3::new(5.0, 3.0, 0.05)));
        let glass = scene.add_standard_material(og_render::StandardMaterial {
            color: Vec3::new(0.1, 0.2, 1.0),
            emissive: Vec3::new(0.05, 0.1, 0.6),
            transparent: 1,
            opacity: 0.5,
            ..Default::default()
        });
        scene.add_instance(
            mesh,
            glass,
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        );
    }
    for (x, z, color) in [
        (-1.2, -2.0, [1.0, 0.05, 0.05, 1.0]),
        (1.2, 2.0, [0.05, 1.0, 0.05, 1.0]),
    ] {
        scene.add_sprite(Sprite {
            position: Vec3::new(x, 1.0, z),
            size: Vec2::splat(1.0),
            anchor: Vec2::splat(0.5),
            color,
            ..Default::default()
        });
        scene.sprite_facing.push(og_render::SpriteFacing::Camera);
    }
    scene
}

/// Billboards sort among the transparent meshes: the red billboard behind
/// the glass pane shows through it (its pixels take the glass's blue), the
/// green one in front draws over it (its pixels are the same with the pane
/// as without). Before the debt round 3 every sprite drew after the
/// transparent meshes, so the red one drew over the glass too and its
/// pixels were the same with the pane as without.
#[test]
fn billboards_sort_among_transparent_meshes() {
    let desc = scenes::base_desc();
    let (with, _) = frame(&desc, &billboards_and_glass(true), 1);
    let (without, _) = frame(&desc, &billboards_and_glass(false), 1);
    // The billboards' centres on screen: x = 640 ± 1.2 / (depth · tan 22.5°) · 360 px.
    let at = |img: &RgbaImage, depth: f32, x: f32| {
        let px = 640.0 + x / (depth * 22.5f32.to_radians().tan()) * 360.0;
        let i = ((360 * img.width + px as u32) * 4) as usize;
        [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
    };
    let (back, back0) = (at(&with, 8.0, -1.2), at(&without, 8.0, -1.2));
    let (front, front0) = (at(&with, 4.0, 1.2), at(&without, 4.0, 1.2));
    println!("behind the glass {back:?} (no glass {back0:?}); in front {front:?} ({front0:?})");
    assert!(
        back[2] > back0[2] + 40 && back[0] < back0[0],
        "the red billboard through the glass: {back:?} against {back0:?}"
    );
    assert_eq!(front, front0, "the green billboard over the glass");
}

/// World text faces the camera (`RenderScene::text_facing`): seen almost
/// edge on, along -x, text in the xy plane shows nearly nothing; turned
/// upright towards the camera (`CameraY`) it shows its glyphs, and centred
/// on its pivot (`align` 0.5) they stand about the middle of the frame,
/// where the pivot is, rather than to its right. Before the debt round 3
/// world text had no facing.
#[test]
fn world_text_faces_the_camera() {
    let desc = scenes::base_desc();
    // Bright pixels, their mean column, and the glyphs drawn.
    let lit = |facing: og_render::SpriteFacing, align: f32| {
        let mut scene = RenderScene::new();
        let font = scenes::load_font(&mut scene).expect("the test font");
        scene.environment.background = Background::Color(Vec3::ZERO);
        scene.post.tonemap = Tonemap::None;
        scene.camera = Camera {
            position: Vec3::new(6.0, 0.5, 0.05),
            target: Vec3::new(0.0, 0.5, 0.0),
            exposure: Exposure::linear(1.0),
            ..Default::default()
        };
        scene.texts.push(Text {
            size: 1.2,
            ..Text::new(font, "OG text")
        });
        scene.text_facing.push(og_render::TextFacing {
            text: 0,
            facing,
            align,
        });
        let (img, stats) = frame(&desc, &scene, 1);
        let columns: Vec<u32> = (0..img.width * img.height)
            .filter(|&i| img.pixels[i as usize * 4] > 128)
            .map(|i| i % img.width)
            .collect();
        let mean = columns.iter().sum::<u32>() as f32 / columns.len().max(1) as f32;
        (columns.len(), mean, stats.glyphs)
    };
    let (flat, _, glyphs) = lit(og_render::SpriteFacing::None, 0.0);
    let (faced, start, faced_glyphs) = lit(og_render::SpriteFacing::CameraY, 0.0);
    let (_, centred, _) = lit(og_render::SpriteFacing::CameraY, 0.5);
    println!(
        "bright pixels: flat {flat}, faced {faced}; mean column from the start {start:.0}, centred {centred:.0}; glyphs {glyphs}, {faced_glyphs}"
    );
    assert_eq!((glyphs, faced_glyphs), (6, 6), "the six glyphs either way");
    assert!(
        faced > 4000 && faced > 8 * flat.max(1),
        "flat {flat}, faced {faced}"
    );
    assert!(
        (centred - 640.0).abs() < 40.0 && start > 640.0 + 150.0,
        "centred on the pivot at column {centred:.0}, from its start {start:.0}"
    );
}

/// Sprites drawn from texture arrays (`RenderScene::sprite_arrays`): the
/// sixteen flipbook sprites (eight layers, two rows) are one draw, the six
/// added ones from a second array another, the plain textured strip a
/// third. Broken once by giving each layer its own run: 8 + 3 + 1 draws.
#[test]
fn sprites_array_match_their_reference_in_one_draw_per_array() {
    let report = check("sprites-array");
    let s = report.stats;
    assert_eq!(s.sprites, 23);
    assert_eq!(
        s.sprite_draws, 3,
        "one run per array and blend, and the strip"
    );
}

/// Materials sampling texture arrays: eight cubes of one tile material on
/// four layers (and the material's own) are one batch; the blended panes
/// of another array material one more; with the ground, the plain cube and
/// the normal-array sphere, five (mesh, material) pairs for 13 instances.
#[test]
fn materials_array_match_their_reference_in_one_batch_per_material() {
    let report = check("materials-array");
    let s = report.stats;
    assert_eq!((s.batches, s.instances), (5, 13));
}

/// A sprite drawn from an array shows the layer it names, sampled with the
/// array's filter; a layer past the array's count, or an array that was
/// removed, is refused.
#[test]
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

/// Texture arrays through every sprite path (`sprites-array-fx`): the
/// glowing flipbook sprites and the flipbook particles are one run, the
/// smooth array's particles another, the strip a third, and the overlay's
/// four array sprites one draw beside its plain one.
#[test]
fn sprites_array_fx_match_their_reference_in_one_draw_per_array() {
    let report = check("sprites-array-fx");
    let s = report.stats;
    assert_eq!(s.overlay_sprites, 5);
    assert_eq!(
        s.overlay_draws, 2,
        "the overlay's array run and its plain sprite"
    );
    assert_eq!(
        s.sprite_draws, 5,
        "the strip, the flipbook (sprites and particles), the motes, and the overlay's two"
    );
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

/// An array sprite with a glow lights the glow buffer around it (the
/// array's glow pipeline), and an overlay sprite draws the layer it names
/// in screen space; a layer past the array is refused there too. Broken
/// once by leaving array sprites out of the glow pass: no halo.
#[test]
fn array_sprites_glow_and_draw_in_the_overlay() {
    use og_render::{Glow, Overlay, SpriteArray, SpriteFx};
    let mut scene = RenderScene::new();
    let array = rgb_array(&mut scene);
    scene.add_sprite(Sprite {
        size: Vec2::splat(2.0),
        ..Default::default()
    });
    scene.sprite_arrays.push(SpriteArray {
        sprite: 0,
        array,
        layer: 1,
    });
    scene.post.glow = Some(Glow { intensity: 1.0 });
    let desc = scenes::base_desc();
    // Just outside the sprite's right edge (2 of 10 units tall: 72 px each side).
    let (cx, cy) = (640, 360);
    let edge = cx + 72 + 12;
    let (plain, _) = frame(&desc, &scene, 1);
    assert_eq!(pixel(&plain, cx, cy), [0, 255, 0], "layer 1");
    assert_eq!(pixel(&plain, edge, cy), [0, 0, 0], "no glow");
    scene.sprite_fx.push(SpriteFx {
        sprite: 0,
        glow: 2.0,
        ..Default::default()
    });
    let (glowing, stats) = frame(&desc, &scene, 1);
    let halo = pixel(&glowing, edge, cy);
    assert!(
        halo[1] > 20 && halo[0] < halo[1] / 4,
        "the layer's green glows past the edge: {halo:?}"
    );
    assert_eq!(stats.sprite_draws, 1);

    let mut overlay = Overlay::new(1280.0, 720.0, 1.0);
    for (x, layer) in [(100.0, 2u32), (300.0, 0)] {
        let i = overlay.sprites.len() as u32;
        overlay.sprites.push(Sprite {
            position: Vec3::new(x, 100.0, 0.0),
            size: Vec2::splat(80.0),
            ..Default::default()
        });
        overlay.sprite_arrays.push(SpriteArray {
            sprite: i,
            array,
            layer,
        });
    }
    scene.overlay = Some(overlay);
    let (img, stats) = frame(&desc, &scene, 1);
    assert_eq!(pixel(&img, 100, 100), [0, 0, 255], "overlay layer 2");
    assert_eq!(pixel(&img, 300, 100), [255, 0, 0], "overlay layer 0");
    assert_eq!(
        stats.overlay_draws, 1,
        "one draw for the array's overlay sprites"
    );
    scene.overlay.as_mut().unwrap().sprite_arrays[1].layer = 5;
    let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
    match renderer.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::InvalidData(m)) => assert!(m.contains("layer 5"), "{m}"),
        other => panic!("an overlay layer past the array: {:?}", other.map(|_| ())),
    }
}

/// A sprite-mode emitter with a texture array draws each particle from a
/// layer: by age (every particle of one burst is the same age, so one
/// layer shows) or at random (several show), in one draw; `round` gives
/// them the soft dot's edge (a big particle's corner stays dark); a
/// billboard emitter with an array is refused. Broken once by drawing the
/// particles without their array (as untextured soft dots): no layer's
/// colour shows.
#[test]
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
    assert_eq!(stats.sprites, 40);
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
