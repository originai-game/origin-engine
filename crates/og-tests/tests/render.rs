//! Renderer behaviour read from frames: standard materials (transparency
//! sorted back to front, double-sided, unlit, a reflected edit reaching the
//! frame), compressed texture formats by tier, the shadow map size and
//! cascade count, and bounded, cancellable capture.

use glam::{Mat4, Vec3};
use og_core::reflect::{Layout, Reflect, from_bytes, to_bytes};
use og_render::{
    Background, Camera, ColorSpace, DebugView, FrameStats, Material, RenderDesc, RenderError,
    RenderScene, RenderTarget, Renderer, RgbaImage, Side, StandardMaterial, TextureCompression,
    TextureFormat, Tonemap, shapes,
};
use og_tests::{blocks, compare, scenes};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn frame(desc: &RenderDesc, scene: &RenderScene) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    (renderer.read_offscreen().expect("capture"), stats)
}

fn pixel(img: &RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

fn center(img: &RgbaImage) -> [u8; 3] {
    pixel(img, img.width / 2, img.height / 2)
}

fn luma(p: &[u8; 4]) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// An unlit 2D-like setup: black background, no tonemap, exposure 0, a
/// camera above the origin looking straight down (+x right, -z up).
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

fn srgb(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        linear * 12.92 * 255.0
    } else {
        (1.055 * linear.powf(1.0 / 2.4) - 0.055) * 255.0
    }
}

fn close(actual: [u8; 3], expected: [f32; 3], within: f32) -> bool {
    actual
        .iter()
        .zip(expected)
        .all(|(&a, e)| (a as f32 - e).abs() <= within)
}

/// Three half-transparent planes stacked under the camera, submitted nearest
/// first: drawn back to front, the nearest (red) dominates the blend; drawn
/// in submission order, the farthest (blue) would.
#[test]
fn transparent_draws_are_sorted_back_to_front() {
    let mut scene = flat_scene();
    let plane = scene.add_mesh(shapes::plane(4.0, 1.0));
    let pane = |scene: &mut RenderScene, color: Vec3| {
        scene.add_standard_material(StandardMaterial {
            color,
            opacity: 0.5,
            transparent: 1,
            unlit: 1,
            ..Default::default()
        })
    };
    let red = pane(&mut scene, Vec3::X);
    let green = pane(&mut scene, Vec3::Y);
    let blue = pane(&mut scene, Vec3::Z);
    for (material, height) in [(red, 3.0), (green, 2.0), (blue, 1.0)] {
        scene.add_instance(plane, material, Mat4::from_translation(Vec3::Y * height));
    }
    let (img, stats) = frame(&scenes::base_desc(), &scene);
    let c = center(&img);
    // Blue, then green, then red, each at alpha 0.5 over black.
    let expected = [srgb(0.5), srgb(0.25), srgb(0.125)];
    println!("centre {c:?}, expected {expected:?}");
    assert!(
        close(c, expected, 2.0),
        "centre {c:?}, expected {expected:?}"
    );
    assert_eq!((stats.transparent, stats.transparent_draws), (3, 3));
}

/// One transparent material on instances at several depths: still sorted
/// per instance (same-material neighbours share a draw).
#[test]
fn transparent_instances_of_one_material_are_sorted_too() {
    let mut scene = flat_scene();
    let mut mesh_of = |color: [f32; 4]| {
        let mut mesh = shapes::plane(4.0, 1.0);
        for v in &mut mesh.vertices {
            v.color = color;
        }
        scene.add_mesh(mesh)
    };
    let (r, g) = (mesh_of([1.0, 0.0, 0.0, 1.0]), mesh_of([0.0, 1.0, 0.0, 1.0]));
    let glass = scene.add_standard_material(StandardMaterial {
        opacity: 0.5,
        transparent: 1,
        unlit: 1,
        ..Default::default()
    });
    // Red nearer than green, submitted first.
    scene.add_instance(r, glass, Mat4::from_translation(Vec3::Y * 2.0));
    scene.add_instance(g, glass, Mat4::from_translation(Vec3::Y * 1.0));
    let (img, stats) = frame(&scenes::base_desc(), &scene);
    let c = center(&img);
    let expected = [srgb(0.5), srgb(0.25), 0.0];
    assert!(
        close(c, expected, 2.0),
        "centre {c:?}, expected {expected:?}"
    );
    assert_eq!(stats.transparent, 2);
}

/// A reflected edit, as the inspector or Tune makes it (the packed bytes of
/// the `og.StandardMaterial` schema, one field rewritten by name), reaches
/// the next frame of the same renderer.
#[test]
fn a_reflected_material_edit_reaches_the_frame() {
    let mut scene = flat_scene();
    let plane = scene.add_mesh(shapes::plane(4.0, 1.0));
    let material = scene.add_standard_material(StandardMaterial::basic(Vec3::ONE));
    scene.add_instance(plane, material, Mat4::IDENTITY);
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let before = center(&renderer.read_offscreen().expect("capture"));
    assert_eq!(before, [255, 255, 255]);

    let schema = StandardMaterial::schema();
    let layout = Layout::of(&schema);
    let mut bytes = to_bytes(scene.standard_material(material).expect("material")).expect("encode");
    let color = layout.range(schema.field_index("color").expect("a color field"));
    let red: [f32; 3] = [1.0, 0.0, 0.0];
    bytes[color].copy_from_slice(bytemuck_bytes(&red));
    let edited: StandardMaterial = from_bytes(&bytes).expect("decode");
    assert!(scene.set_standard_material(material, edited));
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let after = center(&renderer.read_offscreen().expect("capture"));
    assert_eq!(after, [255, 0, 0], "the edit is drawn");
}

/// A material's texture replaced in place (a streamed group upgrading its
/// placeholder) is drawn by the next frame of the same renderer; a
/// material without that texture is untouched. Ways this could fail: the
/// material's bind group keeps the old view because its own revision did
/// not change.
#[test]
fn a_replaced_texture_reaches_the_materials_that_sample_it() {
    let solid = |rgba: [u8; 4], side: u32| og_render::TextureData {
        width: side,
        height: side,
        rgba: rgba.repeat((side * side) as usize),
        color_space: ColorSpace::Srgb,
        mips: og_render::Mips::None,
    };
    let mut scene = flat_scene();
    let plane = scene.add_mesh(shapes::plane(4.0, 1.0));
    let tex = scene.add_texture(solid([255, 0, 0, 255], 1));
    let material = scene.add_standard_material(StandardMaterial {
        map: tex.slot(),
        unlit: 1,
        ..Default::default()
    });
    scene.add_instance(plane, material, Mat4::IDENTITY);
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut draw = |scene: &RenderScene| {
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
        center(&renderer.read_offscreen().expect("capture"))
    };
    assert_eq!(draw(&scene), [255, 0, 0]);
    assert!(scene.set_texture(tex, solid([0, 255, 0, 255], 16)));
    assert_eq!(draw(&scene), [0, 255, 0], "the replacement is sampled");
}

fn bytemuck_bytes(v: &[f32; 3]) -> &[u8] {
    og_core::bytemuck::cast_slice(v)
}

/// The reflected schema is the IDL's `render.StandardMaterialDesc`: every
/// IDL field exists under the same three.js name with the same default.
/// Fields the IDL does not carry yet are listed (and only those).
#[test]
fn the_material_schema_matches_the_idl() {
    let schema = StandardMaterial::schema();
    schema.validate().expect("a valid schema");
    let idl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../og-api/api.ogt");
    let idl = std::fs::read_to_string(&idl_path).expect("the IDL");
    let start = idl
        .find("struct StandardMaterialDesc {")
        .expect("StandardMaterialDesc in the IDL");
    let body = &idl[start..];
    let end = body.find("\n  }\n").expect("the struct's end");
    let mut fields: Vec<(String, Vec<f64>)> = Vec::new();
    for line in body[..end].lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("field ") {
            let name = rest.split_whitespace().next().expect("a name").to_string();
            fields.push((name, Vec::new()));
        } else if let Some(rest) = t.strip_prefix("default ") {
            let values = rest
                .split_whitespace()
                .map(|v| match v {
                    "true" => 1.0,
                    "false" => 0.0,
                    n => n.parse().expect("a number"),
                })
                .collect();
            fields.last_mut().expect("a field").1 = values;
        }
    }
    assert!(fields.len() >= 15, "{} IDL fields", fields.len());
    let zero = |n: usize| vec![0.0; n];
    for (name, default) in &fields {
        let f = schema
            .fields
            .iter()
            .find(|f| &f.name == name)
            .unwrap_or_else(|| panic!("IDL field {name} is missing from og.StandardMaterial"));
        let lanes = f.ty.lanes();
        let ours = f.default.clone().unwrap_or_else(|| zero(lanes));
        let theirs = if default.is_empty() {
            zero(lanes)
        } else {
            default.clone()
        };
        assert_eq!(ours, theirs, "default of {name}");
    }
    let extra: Vec<&str> = schema
        .fields
        .iter()
        .map(|f| f.name.as_str())
        .filter(|n| !fields.iter().any(|(m, _)| m == n))
        .collect();
    println!("og.StandardMaterial fields the IDL does not have yet: {extra:?}");
    for n in &extra {
        assert!(
            [
                "normalScale",
                "aoMapIntensity",
                "unlit",
                // The E1 effects round (crates/og-render/README.md §IDL additions).
                "toon",
                "gradientMap",
                "rimColor",
                "rimIntensity",
                "rimPower",
                "flashColor",
                "flash",
                "dissolve",
                "dissolveMap",
                "dissolveEdgeColor",
                "dissolveEdgeWidth",
                "dissolveScale",
                "outlineColor",
                "outlineWidth",
            ]
            .contains(n),
            "{n} is in neither the IDL nor the list of fields awaiting it"
        );
    }
}

/// `StandardMaterial::default()` is the schema's defaults (what an
/// inspector's reset produces).
#[test]
fn the_default_material_is_the_schema_default() {
    let schema = StandardMaterial::schema();
    let layout = Layout::of(&schema);
    let bytes = to_bytes(&StandardMaterial::default()).expect("encode");
    for (i, f) in schema.fields.iter().enumerate() {
        // Compared as the f32 the field holds (three.js's 1.3 is not a
        // binary fraction).
        let lanes: Vec<f64> = f
            .default
            .clone()
            .unwrap_or_else(|| vec![0.0; f.ty.lanes()])
            .into_iter()
            .map(|v| v as f32 as f64)
            .collect();
        let raw = &bytes[layout.range(i)];
        let values: Vec<f64> = raw
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| match f.ty.name() {
                "u32" => u32::from_le_bytes(*c) as f64,
                _ => f32::from_le_bytes(*c) as f64,
            })
            .collect();
        assert_eq!(values, lanes, "field {}", f.name);
    }
}

/// A plane seen from below: a front-sided material culls it, a double-sided
/// one draws it, a back-sided one draws it too.
#[test]
fn double_sided_materials_draw_back_faces() {
    let draw = |side: Side| {
        let mut scene = flat_scene();
        scene.camera.position = Vec3::new(0.0, -10.0, 0.0);
        let plane = scene.add_mesh(shapes::plane(4.0, 1.0));
        let m = scene.add_standard_material(StandardMaterial {
            side: side as u32,
            ..StandardMaterial::basic(Vec3::ONE)
        });
        scene.add_instance(plane, m, Mat4::IDENTITY);
        center(&frame(&scenes::base_desc(), &scene).0)
    };
    assert_eq!(draw(Side::Front), [0, 0, 0], "back face culled");
    assert_eq!(draw(Side::Double), [255, 255, 255]);
    assert_eq!(draw(Side::Back), [255, 255, 255]);
}

/// Unlit draws the colour exactly, whatever the lights; lit does not.
#[test]
fn unlit_materials_ignore_lighting() {
    let draw = |unlit: u32, intensity: f32| {
        let mut scene = flat_scene();
        scene.light.intensity = intensity;
        let plane = scene.add_mesh(shapes::plane(4.0, 1.0));
        let m = scene.add_standard_material(StandardMaterial {
            color: Vec3::new(0.5, 0.25, 0.125),
            unlit,
            ..Default::default()
        });
        scene.add_instance(plane, m, Mat4::IDENTITY);
        center(&frame(&scenes::base_desc(), &scene).0)
    };
    let expected = [srgb(0.5), srgb(0.25), srgb(0.125)];
    assert!(close(draw(1, 3.0), expected, 1.0));
    assert!(close(draw(1, 30.0), expected, 1.0));
    assert_ne!(
        draw(0, 3.0),
        draw(0, 30.0),
        "lit materials respond to the light"
    );
}

/// A glTF-shaped material and its standard form draw the same pixels.
#[test]
fn gltf_materials_draw_as_their_standard_form() {
    let build = |standard: bool| {
        let mut scene = RenderScene::new();
        let sphere = scene.add_mesh(shapes::sphere(32, 16));
        let checker = scene.add_texture(shapes::checker(256, 8));
        let gltf = Material {
            base_color: [0.9, 0.5, 0.3, 1.0],
            base_color_texture: Some(checker),
            metallic: 0.3,
            roughness: 0.6,
            emissive: [0.05, 0.0, 0.0],
            ..Default::default()
        };
        let m = if standard {
            scene.add_standard_material(StandardMaterial::from(&gltf))
        } else {
            scene.add_material(gltf)
        };
        scene.add_instance(sphere, m, Mat4::IDENTITY);
        frame(&scenes::base_desc(), &scene).0
    };
    assert_eq!(build(false).pixels, build(true).pixels);
}

/// The 8 colours of the compressed-texture test grid.
fn grid() -> Vec<Vec<[u8; 4]>> {
    let palette = [
        [220, 40, 40, 255],
        [40, 200, 60, 255],
        [50, 70, 230, 255],
        [240, 230, 60, 255],
        [250, 250, 250, 255],
        [20, 20, 20, 255],
        [200, 90, 220, 255],
        [60, 210, 210, 255],
    ];
    (0..16)
        .map(|y| (0..16).map(|x| palette[(x / 2 + y / 2 * 3) % 8]).collect())
        .collect()
}

/// A normal-map grid: tilts in x and y per 2x2 blocks.
fn normal_grid() -> Vec<Vec<[u8; 4]>> {
    (0..16)
        .map(|y| {
            (0..16)
                .map(|x| {
                    let (nx, ny) = ([90u8, 128, 166][x / 2 % 3], [100u8, 128, 156][y / 2 % 3]);
                    [nx, ny, 0, 255]
                })
                .collect()
        })
        .collect()
}

/// The RGBA8 form of a normal grid as the shader sees it: z from x and y.
fn with_z(mut upload: og_render::TextureUpload) -> og_render::TextureUpload {
    for level in &mut upload.levels {
        for px in level.as_chunks_mut::<4>().0.iter_mut() {
            let (x, y) = (
                px[0] as f32 / 255.0 * 2.0 - 1.0,
                px[1] as f32 / 255.0 * 2.0 - 1.0,
            );
            let z = (1.0 - x * x - y * y).max(0.0).sqrt();
            px[2] = ((z * 0.5 + 0.5) * 255.0).round() as u8;
        }
    }
    upload
}

/// A tilted textured plane: an unlit one showing the colour texture and a
/// lit one with the normal map, both with mips in view.
fn texture_scene(color: og_render::TextureUpload, normal: og_render::TextureUpload) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.environment.background = Background::Color(Vec3::splat(0.02));
    scene.light.cast_shadows = false;
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.0, 6.0),
        target: Vec3::new(0.0, 0.0, -2.0),
        ..Default::default()
    };
    let plane = scene.add_mesh(shapes::plane(6.0, 3.0));
    let map = scene.add_texture_upload(color);
    let normal_map = scene.add_texture_upload(normal);
    let unlit = scene.add_standard_material(StandardMaterial {
        map: map.slot(),
        unlit: 1,
        ..Default::default()
    });
    let lit = scene.add_standard_material(StandardMaterial {
        normalMap: normal_map.slot(),
        roughness: 0.4,
        ..Default::default()
    });
    scene.add_instance(
        plane,
        unlit,
        Mat4::from_translation(Vec3::new(-3.1, 0.0, -3.0)),
    );
    scene.add_instance(
        plane,
        lit,
        Mat4::from_translation(Vec3::new(3.1, 0.0, -3.0)),
    );
    scene
}

/// Per compression preference: `RenderInfo::texture_formats` names the
/// family's formats when the device has them (BC7/BC5 first on desktop,
/// ASTC or ETC2/EAC when preferred, RGBA8 when off); textures uploaded in
/// those formats with their cooked mips draw the same frame as the same
/// texels in RGBA8; a format the device was not set up for is refused.
#[test]
fn compressed_formats_are_chosen_per_tier_and_draw_like_rgba8() {
    let mut table = Vec::new();
    for preference in [
        TextureCompression::Auto,
        TextureCompression::PreferAstc,
        TextureCompression::PreferEtc2,
        TextureCompression::Off,
    ] {
        let desc = RenderDesc {
            texture_compression: preference,
            ..scenes::base_desc()
        };
        let renderer = Renderer::new_blocking(&desc).expect("renderer");
        let info = renderer.info().clone();
        let formats = info.texture_formats.clone();
        drop(renderer);
        let supported = |f: TextureFormat| formats.supported.contains(&f);
        // The family the preference names wins whenever the device has it.
        let expected = match preference {
            TextureCompression::Off => (TextureFormat::Rgba8, TextureFormat::Rgba8),
            TextureCompression::Auto if supported(TextureFormat::Bc7) => {
                (TextureFormat::Bc7, TextureFormat::Bc5)
            }
            TextureCompression::PreferAstc if supported(TextureFormat::Astc4x4) => {
                (TextureFormat::Astc4x4, TextureFormat::Astc4x4)
            }
            TextureCompression::PreferEtc2 if supported(TextureFormat::Etc2Rgba8) => {
                (TextureFormat::Etc2Rgba8, TextureFormat::EacRg11)
            }
            _ => (formats.color, formats.normal),
        };
        assert_eq!((formats.color, formats.normal), expected, "{preference:?}");
        assert_eq!(formats.data, formats.color);
        if info.adapter.contains("Apple") && info.backend == og_render::wgpu::Backend::Metal {
            // Apple silicon has all three families.
            for f in [
                TextureFormat::Bc7,
                TextureFormat::Astc4x4,
                TextureFormat::Etc2Rgba8,
            ] {
                assert_eq!(supported(f), preference != TextureCompression::Off, "{f:?}");
            }
        }
        let (color, color_ref) = blocks::block_texture(formats.color, ColorSpace::Srgb, &grid());
        let (normal, normal_ref) =
            blocks::block_texture(formats.normal, ColorSpace::Linear, &normal_grid());
        let normal_ref = if formats.normal.two_channel() {
            with_z(normal_ref)
        } else {
            normal_ref
        };
        let (compressed, _) = frame(&desc, &texture_scene(color, normal));
        let (reference, _) = frame(&desc, &texture_scene(color_ref, normal_ref));
        let c = compare::compare(
            &compressed.pixels,
            &reference.pixels,
            reference.width,
            reference.height,
            &Default::default(),
        )
        .expect("same size");
        // The texture is really drawn: many distinct colours on the left plane.
        let mut distinct = std::collections::HashSet::new();
        for y in (300..700).step_by(7) {
            for x in (100..600).step_by(7) {
                distinct.insert(pixel(&compressed, x, y));
            }
        }
        println!(
            "{preference:?} on {} {:?}: color {:?}, normal {:?}, supported {:?}; vs RGBA8 rmse {:.5}, max diff {}, {} colours",
            info.adapter,
            info.backend,
            formats.color,
            formats.normal,
            formats.supported,
            c.rmse,
            c.max_channel_diff,
            distinct.len()
        );
        assert!(distinct.len() > 20, "{} distinct colours", distinct.len());
        assert!(c.rmse < 0.004, "{preference:?}: rmse {:.5}", c.rmse);
        table.push(format!(
            "| {preference:?} | {:?} | {:?} | {:?} | {:.5} | {} |",
            formats.color, formats.normal, formats.supported, c.rmse, c.max_channel_diff
        ));
        // A format outside the device's set is refused, not drawn wrong.
        if let Some(missing) = TextureFormat::ALL
            .into_iter()
            .find(|f| !formats.supported.contains(f))
        {
            let (upload, _) = blocks::block_texture(missing, ColorSpace::Linear, &normal_grid());
            let mut scene = RenderScene::new();
            let t = scene.add_texture_upload(upload);
            let m = scene.add_standard_material(StandardMaterial {
                map: t.slot(),
                ..Default::default()
            });
            let plane = scene.add_mesh(shapes::plane(1.0, 1.0));
            scene.add_instance(plane, m, Mat4::IDENTITY);
            let mut r = Renderer::new_blocking(&desc).expect("renderer");
            match r.render(&scene, &RenderTarget::Offscreen) {
                Err(RenderError::Unsupported(why)) => {
                    assert!(why.contains(&format!("{missing:?}")), "{why}")
                }
                other => panic!("{missing:?} on {preference:?}: {other:?}"),
            }
        }
    }
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-render");
    std::fs::create_dir_all(&out).expect("out dir");
    let text = format!(
        "| preference | color | normal | supported | RMSE vs RGBA8 | max diff |\n|---|---|---|---|---|---|\n{}\n",
        table.join("\n")
    );
    std::fs::write(out.join("texture-formats.md"), &text).expect("write");
    print!("{text}");
}

/// Malformed uploads are errors with the reason.
#[test]
fn malformed_texture_uploads_are_refused() {
    let (mut upload, _) = blocks::block_texture(TextureFormat::Rgba8, ColorSpace::Srgb, &grid());
    upload.levels[2].pop();
    let mut scene = RenderScene::new();
    scene.add_texture_upload(upload);
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    match r.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::InvalidData(why)) => assert!(why.contains("level 2"), "{why}"),
        other => panic!("{other:?}"),
    }
}

/// A pole's shadow on the ground, sun low: the scene, and the desc with a
/// given cascade size.
fn pole_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let ground = scene.add_mesh(shapes::plane(40.0, 1.0));
    let pole = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 6.0, 1.0)));
    let grey = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.8),
        ..Default::default()
    });
    scene.add_instance(ground, grey, Mat4::IDENTITY);
    scene.add_instance(pole, grey, Mat4::from_translation(Vec3::Y * 3.0));
    scene.light.direction_to_light = Vec3::new(-0.8, 0.45, 0.3);
    scene.light.max_shadow_distance = 40.0;
    scene.environment.intensity = 0.2;
    scene.camera = Camera {
        position: Vec3::new(4.0, 7.0, 10.0),
        target: Vec3::new(5.0, 0.0, -1.0),
        ..Default::default()
    };
    scene
}

/// Pixels in the shadow edge's ramp: neither fully lit nor fully shadowed.
fn penumbra(lit: &RgbaImage, unshadowed: &RgbaImage) -> usize {
    lit.pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(unshadowed.pixels.as_chunks::<4>().0.iter())
        .filter(|(a, b)| {
            let (a, b) = (luma(a), luma(b));
            b > 40.0 && a / b > 0.6 && a / b < 0.93
        })
        .count()
}

/// The cascade size is the shadow's resolution: at 256² the edge of the
/// pole's shadow ramps over many more pixels than at 2048².
#[test]
fn shadow_map_size_sets_the_shadow_resolution() {
    let mut scene = pole_scene();
    let base = scenes::base_desc();
    let mut ramps = Vec::new();
    for size in [256, 2048] {
        let desc = RenderDesc {
            shadow_map_size: size,
            shadow_cascades: 1,
            ..base
        };
        let renderer = Renderer::new_blocking(&desc).expect("renderer");
        assert_eq!(renderer.info().shadow_map_size, size);
        drop(renderer);
        scene.light.cast_shadows = true;
        let (lit, stats) = frame(&desc, &scene);
        assert_eq!(stats.shadow_passes, 1);
        scene.light.cast_shadows = false;
        let (unshadowed, _) = frame(&desc, &scene);
        let shadowed = lit
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .zip(unshadowed.pixels.as_chunks::<4>().0.iter())
            .filter(|(a, b)| luma(a) < 0.6 * luma(b))
            .count();
        let ramp = penumbra(&lit, &unshadowed);
        println!("shadow map {size}²: {shadowed} shadowed pixels, {ramp} in the edge ramp");
        og_tests::gfx::save_png(
            &PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                .join(format!("og-render/shadow-{size}.png")),
            &lit,
        )
        .expect("save");
        assert!(
            shadowed > 2_000,
            "{size}: the pole casts a shadow ({shadowed})"
        );
        ramps.push(ramp);
    }
    assert!(
        ramps[0] > 3 * ramps[1],
        "edge ramp {} pixels at 256² vs {} at 2048²",
        ramps[0],
        ramps[1]
    );
}

/// 1, 2 or 3 cascades: that many shadow passes, and the cascade debug view
/// shows only those cascades' colours.
#[test]
fn cascade_count_is_one_to_three() {
    for cascades in [1u32, 2, 3, 0, 9] {
        let desc = RenderDesc {
            shadow_cascades: cascades,
            ..scenes::base_desc()
        };
        let expected = cascades.clamp(1, 3);
        let mut scene = scenes::cascades_200m(true);
        scene.debug = DebugView::Cascades;
        let (img, stats) = frame(&desc, &scene);
        assert_eq!(stats.shadow_passes, expected, "{cascades} requested");
        let mut seen = [0usize; 3];
        for p in img.pixels.as_chunks::<4>().0.iter() {
            let max = *p[..3].iter().max().expect("rgb");
            if max > 100
                && let Some(c) =
                    (0..3).find(|&c| p[c] == max && (0..3).all(|o| o == c || p[o] < 60))
            {
                seen[c] += 1;
            }
        }
        println!("{cascades} cascades requested: pixels per cascade colour {seen:?}");
        for (c, &n) in seen.iter().enumerate() {
            if (c as u32) < expected {
                assert!(n > 5_000, "cascade {c} of {expected}: {n} pixels");
            } else {
                assert!(n < 50, "cascade {c} beyond {expected}: {n} pixels");
            }
        }
    }
}

/// The mobile preset: two 1024² cascades and ASTC/ETC2 first.
#[test]
fn the_mobile_preset_is_reported() {
    let desc = RenderDesc {
        backend: scenes::base_desc().backend,
        ..RenderDesc::mobile()
    };
    let renderer = Renderer::new_blocking(&desc).expect("renderer");
    let info = renderer.info();
    assert_eq!((info.shadow_map_size, info.shadow_cascades), (1024, 2));
    assert_eq!(desc.texture_compression, TextureCompression::PreferAstc);
    if info
        .texture_formats
        .supported
        .contains(&TextureFormat::Astc4x4)
    {
        assert_eq!(info.texture_formats.color, TextureFormat::Astc4x4);
    }
    let mut scene = pole_scene();
    let mut r = renderer;
    let stats = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert_eq!(stats.shadow_passes, 2);
    scene.light.cast_shadows = false;
    assert_eq!(
        r.render(&scene, &RenderTarget::Offscreen)
            .expect("render")
            .shadow_passes,
        0
    );
}

/// Capture is a request: it completes without blocking through
/// `try_finish`, a dropped request is cancelled without disturbing the
/// next, and `wait` is bounded.
#[test]
fn capture_is_bounded_and_cancellable() {
    let mut scene = flat_scene();
    let plane = scene.add_mesh(shapes::plane(4.0, 1.0));
    let m = scene.add_standard_material(StandardMaterial::basic(Vec3::new(0.0, 1.0, 0.0)));
    scene.add_instance(plane, m, Mat4::IDENTITY);
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    // Cancelled: started, then dropped before it completes.
    for _ in 0..3 {
        drop(renderer.begin_capture().expect("begin"));
    }
    renderer.begin_capture().expect("begin").cancel();
    // Non-blocking completion.
    let mut request = renderer.begin_capture().expect("begin");
    let t = Instant::now();
    let image = loop {
        if let Some(image) = request.try_finish(&renderer).expect("capture") {
            break image;
        }
        assert!(
            t.elapsed() < Duration::from_secs(10),
            "capture never completed"
        );
        std::thread::sleep(Duration::from_millis(1));
    };
    assert_eq!(center(&image), [0, 255, 0]);
    // Bounded wait.
    let image = renderer
        .begin_capture()
        .expect("begin")
        .wait(&renderer, Duration::from_secs(5))
        .expect("within the bound");
    assert_eq!(center(&image), [0, 255, 0]);
}
