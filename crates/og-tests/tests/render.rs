//! Renderer behaviour read from frames: standard materials (transparency
//! sorted back to front, or order independent, double-sided, unlit, a reflected edit reaching the
//! frame), compressed texture formats by tier, the shadow map size and
//! cascade count, and bounded, cancellable capture.

use glam::{Mat4, Vec3};
use og_core::reflect::Reflect;
use og_render::{
    Background, Camera, ColorSpace, FrameStats, RenderDesc, RenderError, RenderScene, RenderTarget,
    Renderer, RgbaImage, StandardMaterial, TextureCompression, TextureFormat, shapes,
};
use og_tests::{blocks, compare, scenes};
use std::path::PathBuf;

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

/// The reflected schema is the IDL's `render.StandardMaterialDesc`: every
/// IDL field exists under the same three.js name with the same default.
/// Fields the IDL does not carry yet are listed (and only those).
#[test]
fn the_material_schema_matches_the_idl() {
    let schema = StandardMaterial::schema();
    schema.validate().expect("a valid schema");
    let idl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../og-api/api/render.ogt");
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
                // The effects (crates/og-render/README.md §From TypeScript).
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
    scene.lighting.environment.background = Background::Color(Vec3::splat(0.02));
    scene.lighting.light.cast_shadows = false;
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
#[ignore = "requires GPU"]
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

/// The linked lists composite equal depths by the fragments' keys, never by
/// the order the draws arrived in (`oit_list.wgsl` `oit_before`): four
/// coplanar blended quads facing the camera, each its own draw at its own
/// transform, submitted in one order and in the reverse (the back-to-front
/// sort keeps submission order for equal depths), draw the same frame. A
/// linked-list pipeline that lost the vertex stages' key (`oit.wgsl`
/// `OIT_KEY`) composites them in arrival order instead.
#[test]
#[ignore = "requires GPU"]
fn linked_lists_order_equal_depths_by_key_not_by_draw() {
    let desc = RenderDesc {
        width: 256,
        height: 256,
        transparency: og_render::Transparency::LinkedList,
        ..scenes::base_desc()
    };
    let quads = [
        (Vec3::new(1.0, 0.1, 0.1), -0.45),
        (Vec3::new(0.1, 1.0, 0.1), -0.15),
        (Vec3::new(0.1, 0.1, 1.0), 0.15),
        (Vec3::new(1.0, 1.0, 0.1), 0.45),
    ];
    let draw = |order: [usize; 4]| {
        let mut scene = RenderScene::new();
        scene.camera = Camera {
            position: Vec3::new(0.0, 0.0, 4.0),
            target: Vec3::ZERO,
            exposure: og_render::Exposure::linear(1.0),
            ..Default::default()
        };
        let quad = scene.add_mesh(shapes::plane_xy(1.0, 1.0));
        for i in order {
            let (color, x) = quads[i];
            let m = scene.add_standard_material(StandardMaterial {
                color,
                opacity: 0.5,
                transparent: 1,
                side: 2,
                ..Default::default()
            });
            scene.add_instance(quad, m, Mat4::from_translation(Vec3::new(x, 0.0, 0.0)));
        }
        let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
        if renderer.info().transparency != og_render::Transparency::LinkedList {
            return None;
        }
        let stats = renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        assert!(stats.transparency_layers > 0, "the linked lists ran");
        Some(renderer.read_offscreen().expect("capture"))
    };
    let Some(forward) = draw([0, 1, 2, 3]) else {
        println!("no linked lists on this device");
        return;
    };
    let backward = draw([3, 2, 1, 0]).expect("the same device");
    // All four overlap at the centre, two by two further out.
    let c = forward.width / 2;
    assert_ne!(
        pixel(&forward, c, c),
        pixel(&forward, 2, 2),
        "the quads drawn"
    );
    let differ = forward
        .pixels
        .chunks(4)
        .zip(backward.pixels.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(differ, 0, "pixels that depend on the draw order");
}
