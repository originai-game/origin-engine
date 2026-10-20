//! Resources can be removed (og-render's `RenderScene::remove_mesh`,
//! `remove_texture`, `remove_material`, `remove_font`,
//! `remove_environment_map`, `remove_texture_array`, `remove_palette`): a
//! removed slot is
//! freed on the GPU before the next frame, the next add of its kind reuses
//! it under a new generation, and an old handle never reaches the new
//! resource: the setters refuse it and a frame that still draws with it is
//! refused (`RenderError::InvalidHandle`). A material that sampled a
//! removed texture draws without it, as a material that never had one.
//! Broken once by keeping a slot's generation when it is emptied: the
//! reused slot's handle equals the removed one's, so nothing could tell
//! a stale handle from the new mesh's (the generation check fails). The
//! environment maps, texture arrays and palettes a streaming group loaded
//! leave the frame's texture bytes when removed (broken once by summing
//! the cache's arrays whether or not their slot was emptied: the bytes
//! stay).

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, ColorSpace, DitherPattern, ENVIRONMENT_MAP_MIN_SIZE, EnvironmentMap,
    EnvironmentMapData, Glyph, MAX_PALETTE_COLORS, MsdfFont, Palette, Quantize, RenderError,
    RenderScene, RenderTarget, Renderer, RgbaImage, Sprite, SpriteArray, StandardMaterial, Text,
    TextureArrayData, TextureData, TextureSource, Tonemap, rgb9e5, shapes,
};
use og_tests::scenes;

fn center(img: &RgbaImage) -> [u8; 3] {
    let i = ((img.height / 2 * img.width + img.width / 2) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

/// Unlit, black background, looking straight down at the origin.
fn flat_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    scene.light.cast_shadows = false;
    scene.camera = Camera {
        position: Vec3::new(0.0, 10.0, 0.0),
        target: Vec3::ZERO,
        up: Vec3::NEG_Z,
        ..Default::default()
    };
    scene
}

fn solid(rgba: [u8; 4], side: u32) -> TextureData {
    TextureData {
        width: side,
        height: side,
        rgba: rgba.repeat((side * side) as usize),
        color_space: ColorSpace::Srgb,
        mips: og_render::Mips::None,
    }
}

fn unlit(color: [f32; 3], map: u32) -> StandardMaterial {
    StandardMaterial {
        color: Vec3::from_array(color),
        map,
        unlit: 1,
        ..Default::default()
    }
}

fn draw(renderer: &mut Renderer, scene: &RenderScene) -> Result<RgbaImage, RenderError> {
    renderer.render(scene, &RenderTarget::Offscreen)?;
    Ok(renderer.read_offscreen().expect("capture"))
}

#[test]
fn a_removed_mesh_is_refused_and_its_slot_reused_under_a_new_generation() {
    let mut scene = flat_scene();
    let plane = scene.add_mesh(shapes::plane(4.0, 1.0));
    let red = scene.add_standard_material(unlit([1.0, 0.0, 0.0], 0));
    scene.add_instance(plane, red, Mat4::IDENTITY);
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    assert_eq!(
        center(&draw(&mut renderer, &scene).expect("frame")),
        [255, 0, 0]
    );

    assert!(scene.remove_mesh(plane));
    assert!(!scene.contains_mesh(plane));
    assert!(!scene.remove_mesh(plane), "removed twice");
    assert!(
        !scene.set_mesh(plane, shapes::plane(4.0, 1.0)),
        "a stale handle is refused"
    );
    assert!(scene.mesh(plane).is_none());
    match draw(&mut renderer, &scene) {
        Err(RenderError::InvalidHandle(m)) => assert!(m.contains("mesh 0 was removed"), "{m}"),
        other => panic!("a frame drawing a removed mesh: {:?}", other.map(|_| ())),
    }

    // The slot comes back with a new generation: the old handle stays stale.
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(1.0)));
    assert_eq!(cube.index(), plane.index(), "the freed slot is reused");
    assert_ne!(cube.generation(), plane.generation());
    assert!(!scene.contains_mesh(plane) && scene.contains_mesh(cube));
    assert!(
        draw(&mut renderer, &scene).is_err(),
        "the old instance still names the old mesh"
    );
    scene.instances.clear();
    scene.add_instance(cube, red, Mat4::IDENTITY);
    let reused = draw(&mut renderer, &scene).expect("frame");

    // Byte for byte what a scene that only ever had the cube draws.
    let mut fresh_scene = flat_scene();
    let c = fresh_scene.add_mesh(shapes::cuboid(Vec3::splat(1.0)));
    let m = fresh_scene.add_standard_material(unlit([1.0, 0.0, 0.0], 0));
    fresh_scene.add_instance(c, m, Mat4::IDENTITY);
    let mut fresh = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let expected = draw(&mut fresh, &fresh_scene).expect("frame");
    assert!(
        reused.pixels == expected.pixels,
        "the reused slot draws the new mesh"
    );
}

#[test]
fn a_removed_texture_leaves_its_materials_and_a_removed_material_is_refused() {
    let mut scene = flat_scene();
    let plane = scene.add_mesh(shapes::plane(4.0, 1.0));
    let tex = scene.add_texture(solid([0, 255, 0, 255], 64));
    let material = scene.add_standard_material(unlit([1.0, 1.0, 1.0], tex.slot()));
    scene.add_instance(plane, material, Mat4::IDENTITY);
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    assert_eq!(
        center(&draw(&mut renderer, &scene).expect("frame")),
        [0, 255, 0]
    );

    assert!(scene.remove_texture(tex));
    assert!(!scene.contains_texture(tex));
    assert!(
        !scene.set_texture(tex, solid([0, 0, 255, 255], 1)),
        "a stale handle is refused"
    );
    assert_eq!(scene.standard_material(material).expect("material").map, 0);
    // The material now draws as one that never had a map: white.
    assert_eq!(
        center(&draw(&mut renderer, &scene).expect("frame")),
        [255, 255, 255]
    );
    // A new texture takes the slot; the material does not pick it up.
    let blue = scene.add_texture(solid([0, 0, 255, 255], 1));
    assert_eq!(blue.index(), tex.index());
    assert_eq!(
        center(&draw(&mut renderer, &scene).expect("frame")),
        [255, 255, 255]
    );

    assert!(scene.remove_material(material));
    assert!(!scene.set_standard_material(material, unlit([1.0, 0.0, 0.0], 0)));
    match draw(&mut renderer, &scene) {
        Err(RenderError::InvalidHandle(m)) => assert!(m.contains("material 0 was removed"), "{m}"),
        other => panic!(
            "a frame drawing a removed material: {:?}",
            other.map(|_| ())
        ),
    }
    let red = scene.add_standard_material(unlit([1.0, 0.0, 0.0], 0));
    assert_eq!(red.index(), material.index());
    scene.instances.clear();
    scene.add_instance(plane, red, Mat4::IDENTITY);
    assert_eq!(
        center(&draw(&mut renderer, &scene).expect("frame")),
        [255, 0, 0]
    );
}

/// A small uniform environment map (the smallest size), every texel `rgb`.
fn environment(rgb: [f32; 3]) -> EnvironmentMapData {
    let size = ENVIRONMENT_MAP_MIN_SIZE;
    EnvironmentMapData {
        size,
        texels: vec![rgb9e5(rgb); EnvironmentMapData::texel_count(size)],
        irradiance: [[0.0; 3]; 9],
    }
}

fn layers(n: usize) -> TextureArrayData {
    TextureArrayData {
        layers: (0..n)
            .map(|i| TextureSource::Rgba(solid([40 * i as u8, 200, 90, 255], 8)))
            .collect(),
    }
}

#[test]
fn environment_maps_texture_arrays_and_palettes_leave_the_frame_and_its_bytes() {
    let mut scene = flat_scene();
    let plane = scene.add_mesh(shapes::plane(4.0, 1.0));
    let white = scene.add_standard_material(unlit([1.0, 1.0, 1.0], 0));
    scene.add_instance(plane, white, Mat4::IDENTITY);
    let env = scene.add_environment_map(environment([0.5, 0.6, 0.7]));
    let array = scene.add_texture_array(layers(3)).expect("three layers");
    let palette = scene.add_palette(Palette {
        colors: vec![[0, 0, 0], [255, 255, 255]],
    });
    scene.environment_map = Some(EnvironmentMap::new(env));
    scene.stylise.quantize = Some(Quantize {
        posterize: 0,
        palette: Some(palette),
        dither: DitherPattern::None,
        dither_strength: 0.0,
        dither_scale: 1,
    });
    scene.add_sprite(Sprite {
        position: Vec3::new(0.0, 1.0, 0.0),
        ..Default::default()
    });
    scene.sprite_arrays.push(SpriteArray {
        sprite: 0,
        array,
        layer: 2,
    });
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let full = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("frame")
        .texture_bytes;
    let env_bytes = 4 * EnvironmentMapData::texel_count(ENVIRONMENT_MAP_MIN_SIZE) as u64;
    let array_bytes = renderer.texture_array_info(array).expect("uploaded").bytes;
    let palette_bytes = MAX_PALETTE_COLORS as u64 * 4;
    assert!(full >= env_bytes + array_bytes + palette_bytes, "{full}");

    // Each one in use is refused by the frame that still names it.
    assert!(scene.remove_environment_map(env));
    assert!(!scene.contains_environment_map(env));
    assert!(!scene.remove_environment_map(env), "removed twice");
    assert!(scene.environment_map_data(env).is_none());
    match renderer.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::InvalidHandle(m)) => {
            assert!(m.contains("environment map 0 was removed"), "{m}")
        }
        other => panic!("a frame lit by a removed map: {:?}", other.map(|_| ())),
    }
    scene.environment_map = None;
    assert!(scene.remove_palette(palette));
    assert!(scene.palette(palette).is_none());
    match renderer.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::InvalidHandle(m)) => assert!(m.contains("palette 0 was removed"), "{m}"),
        other => panic!(
            "a frame quantised to a removed palette: {:?}",
            other.map(|_| ())
        ),
    }
    scene.stylise.quantize = None;
    assert!(scene.remove_texture_array(array));
    assert!(!scene.set_texture_array_filter(array, og_render::TextureFilter::Nearest));
    assert!(scene.texture_array(array).is_none());
    match renderer.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::InvalidHandle(m)) => {
            assert!(m.contains("texture array 0 was removed"), "{m}")
        }
        other => panic!("a sprite of a removed array: {:?}", other.map(|_| ())),
    }
    scene.sprite_arrays.clear();

    // Freed: the frame's bytes drop by exactly what the three took.
    let after = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("nothing stale is left")
        .texture_bytes;
    assert_eq!(full - after, env_bytes + array_bytes + palette_bytes);
    assert!(renderer.texture_array_info(array).is_none());

    // The slots come back under new generations; the old handles stay stale.
    let env2 = scene.add_environment_map(environment([0.1, 0.1, 0.1]));
    let array2 = scene.add_texture_array(layers(2)).expect("two layers");
    let palette2 = scene.add_palette(Palette {
        colors: vec![[255, 0, 0]],
    });
    assert_eq!(
        (env2.index(), array2.index(), palette2.index()),
        (env.index(), array.index(), palette.index())
    );
    assert_ne!(env2.generation(), env.generation());
    assert_ne!(array2.generation(), array.generation());
    assert_ne!(palette2.generation(), palette.generation());
    scene.environment_map = Some(EnvironmentMap::new(env));
    assert!(
        renderer.render(&scene, &RenderTarget::Offscreen).is_err(),
        "the old handle does not reach the new map"
    );
    scene.environment_map = Some(EnvironmentMap::new(env2));
    let again = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("frame")
        .texture_bytes;
    let array2_bytes = renderer.texture_array_info(array2).expect("uploaded").bytes;
    assert_eq!(again, after + env_bytes + array2_bytes + palette_bytes);
    assert!(renderer.texture_array_info(array).is_none(), "stale handle");
}

#[test]
fn text_in_a_removed_font_is_refused() {
    let mut scene = flat_scene();
    let atlas = scene.add_texture(solid([255, 255, 255, 255], 4));
    let font = scene.add_font(MsdfFont {
        atlas,
        distance_range: 4.0,
        line_height: 1.2,
        glyphs: vec![Glyph {
            codepoint: 'A',
            advance: 0.6,
            plane_bounds: [0.0, 0.0, 0.5, 0.7],
            atlas_bounds: [0.0, 0.0, 4.0, 4.0],
        }],
        kerning: Vec::new(),
    });
    scene.texts.push(Text {
        font,
        text: "A".into(),
        position: Vec3::ZERO,
        size: 1.0,
        rotation: 0.0,
        color: [1.0; 4],
        layer: 0,
        clip: None,
        tracking: 0.0,
        weight: 0.0,
        glow: None,
        placements: Vec::new(),
    });
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    draw(&mut renderer, &scene).expect("frame");
    assert!(scene.remove_font(font));
    assert!(scene.font(font).is_none());
    match draw(&mut renderer, &scene) {
        Err(RenderError::InvalidHandle(m)) => assert!(m.contains("font 0 was removed"), "{m}"),
        other => panic!("text in a removed font: {:?}", other.map(|_| ())),
    }
    scene.texts.clear();
    // A font's atlas removed under a live font is refused too.
    let font = scene.add_font(MsdfFont {
        atlas,
        distance_range: 4.0,
        line_height: 1.2,
        glyphs: Vec::new(),
        kerning: Vec::new(),
    });
    assert!(scene.contains_font(font));
    assert!(scene.remove_texture(atlas));
    assert!(draw(&mut renderer, &scene).is_err());
    assert!(scene.remove_font(font));
    draw(&mut renderer, &scene).expect("nothing stale is left");
}
