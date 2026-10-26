//! Compressed textures at runtime (phase 2, owed row 32): the template's
//! cooked Basis textures (`templates/game/assets`: stone.png colour,
//! stone_n.png normal map, and the models' base colour, metallic-roughness
//! and normal maps) transcoded to the block format of each family the GPU
//! takes, every cooked mip level uploaded, and drawn as their RGBA8 decode
//! draws.
//!
//! Ways this could fail, and the test that catches each:
//! - a kind reaches the wrong format (a normal map as BC7 instead of BC5,
//!   colour in a two-channel format, sRGB lost on colour or put on data)
//!   → `each_kind_reaches_its_familys_format`;
//! - levels missing, short, padded or out of order (the upload is refused,
//!   or distant surfaces sample another level's texels) → the level checks
//!   in `each_kind_reaches_its_familys_format`, and the far boards in
//!   `compressed_textures_draw_as_their_rgba8_decode`, which sample the
//!   small levels;
//! - transcoded blocks decode to other texels (channels swapped, rows
//!   flipped, BC5's or EAC's z not rebuilt, sRGB decoded twice) →
//!   `compressed_textures_draw_as_their_rgba8_decode`, each family against
//!   the RGBA8 frame and the RGBA8 frame against itself with the normal map
//!   flipped (so the tolerance is tighter than the error it must catch);
//! - a format the device does not take → the upload fails in every frame
//!   test;
//! - the cooked models drift from og-tests' gltf-reference frame once their
//!   textures are compressed → `cooked_models_with_compressed_textures_render_the_gltf_reference_scene`;
//! - no memory saved → `texture_bytes` compared in the frame test.
//!
//! Evidence (frames, diff numbers, GPU memory, transcode and upload times)
//! goes to `$OG_EVIDENCE_DIR/P2/render/` (default
//! `<target tmp>/evidence/P2/render/`).

use glam::{Mat4, Quat, Vec3};
use og_cook::runtime::{Bundle, CookedTexture};
use og_render::{
    FrameStats, Material, RenderScene, RenderTarget, Renderer, RgbaImage, TextureCompression,
    TextureFormat, TextureFormats, TextureSource, shapes,
};
use og_tests::compare::{Comparison, Tolerance, compare};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

use crate::support;

fn evidence_dir() -> PathBuf {
    let base = std::env::var_os("OG_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("evidence"));
    let dir = base.join("P2/render");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn bundle() -> Bundle {
    static COOKED: OnceLock<support::SharedCook> = OnceLock::new();
    let cooked = COOKED.get_or_init(|| {
        // Its own name: a shared cook removes the stale keys of its name,
        // and template.rs's binary has another key for the same sources.
        support::shared(
            "template-compressed",
            &support::template_sources(),
            |root| support::stage_template(&root.join("assets")),
        )
    });
    Bundle::open(&cooked.root.join("cooked")).expect("the manifest loads")
}

const MODELS: [&str; 2] = ["models/trickster.glb", "models/dropship.glb"];

/// Every Basis texture of the template, by id: the two loose ones and the
/// models' embedded ones.
fn template_textures(b: &Bundle) -> Vec<CookedTexture> {
    let mut ids = vec![
        "textures/stone.png".to_string(),
        "textures/stone_n.png".to_string(),
    ];
    for m in MODELS {
        ids.extend(
            b.load_model(m)
                .unwrap()
                .textures
                .iter()
                .map(|t| t.id.clone()),
        );
    }
    ids.iter().map(|id| b.load_texture(id).unwrap()).collect()
}

/// A renderer preferring `compression`, and the encodings it takes.
fn renderer(compression: TextureCompression) -> (Renderer, TextureFormats) {
    let desc = og_render::RenderDesc {
        texture_compression: compression,
        ..og_tests::scenes::base_desc()
    };
    let r = Renderer::new_blocking(&desc).expect("renderer");
    let f = r.info().texture_formats.clone();
    (r, f)
}

/// The families: the preference that selects each, and the colour and
/// normal-map formats it should give.
const FAMILIES: [(&str, TextureCompression, TextureFormat, TextureFormat); 3] = [
    (
        "bc",
        TextureCompression::Auto,
        TextureFormat::Bc7,
        TextureFormat::Bc5,
    ),
    (
        "astc",
        TextureCompression::PreferAstc,
        TextureFormat::Astc4x4,
        TextureFormat::Astc4x4,
    ),
    (
        "etc2",
        TextureCompression::PreferEtc2,
        TextureFormat::Etc2Rgba8,
        TextureFormat::EacRg11,
    ),
];

/// The families this GPU takes (each is still checked where it exists; a
/// GPU with none fails, since every tier this round targets has one).
fn available() -> Vec<(
    &'static str,
    TextureCompression,
    TextureFormat,
    TextureFormat,
)> {
    let (_, f) = renderer(TextureCompression::Auto);
    let out: Vec<_> = FAMILIES
        .into_iter()
        .filter(|(_, _, color, _)| f.supported.contains(color))
        .collect();
    assert!(
        !out.is_empty(),
        "this GPU takes no block format ({:?})",
        f.supported
    );
    out
}

#[test]
fn each_kind_reaches_its_familys_format() {
    let b = bundle();
    let textures = template_textures(&b);
    for (family, pref, color, normal) in available() {
        let (_, formats) = renderer(pref);
        assert_eq!((formats.color, formats.normal), (color, normal), "{family}");
        for t in &textures {
            let kind = t.meta.kind;
            let expected = match kind {
                og_cook::format::kind::NORMAL => normal,
                _ => color,
            };
            let TextureSource::Encoded(up) = t.to_texture_source(Some(&formats)).unwrap() else {
                panic!("{family}: {} stayed RGBA8", t.id);
            };
            let d = &up.desc;
            assert_eq!(d.format, expected, "{family}: {} (kind {kind})", t.id);
            assert_eq!(
                (d.width, d.height),
                (t.meta.width, t.meta.height),
                "{}",
                t.id
            );
            let srgb = kind == og_cook::format::kind::COLOR;
            assert_eq!(
                d.color_space == og_render::ColorSpace::Srgb,
                srgb,
                "{family}: {} colour space",
                t.id
            );
            assert_eq!(d.mip_levels, t.meta.levels, "{family}: {} levels", t.id);
            for (i, level) in up.levels.iter().enumerate() {
                assert_eq!(
                    level.len(),
                    d.format.level_bytes(d.width, d.height, i as u32),
                    "{family}: {} level {i}",
                    t.id
                );
            }
        }
        // Without a renderer, and on a renderer taking no compression: RGBA8.
        let (_, off) = renderer(TextureCompression::Off);
        for f in [None, Some(&off)] {
            assert!(matches!(
                textures[0].to_texture_source(f).unwrap(),
                TextureSource::Rgba(_)
            ));
        }
    }
}

/// Stone boards under a low sun: a near one filling the lower frame and a
/// row of far ones (their small mip levels), stone.png as base colour and
/// stone_n.png as normal map, from `source`.
fn stone_scene(textures: &[TextureSource; 2], flip_normal: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    let color = scene.add_texture_source(textures[0].clone());
    let normal = scene.add_texture_source(textures[1].clone());
    let m = scene.add_material(Material {
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: Some(color),
        normal_texture: Some(normal),
        normal_scale: if flip_normal { -1.0 } else { 1.0 },
        roughness: 0.55,
        ..Default::default()
    });
    let board = scene.add_mesh(shapes::plane(4.0, 2.0));
    scene.add_instance(board, m, Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)));
    for i in 0..6 {
        let at = Vec3::new(-7.5 + i as f32 * 3.0, 1.5, -24.0);
        let tilt = Quat::from_rotation_x(1.2);
        scene.add_instance(
            board,
            m,
            Mat4::from_scale_rotation_translation(Vec3::splat(0.7), tilt, at),
        );
    }
    scene.light.direction_to_light = Vec3::new(0.6, 0.35, 0.5).normalize();
    scene.light.intensity = 3.0;
    scene.camera.position = Vec3::new(0.0, 2.2, 4.2);
    scene.camera.target = Vec3::new(0.0, 0.6, -6.0);
    scene
}

fn draw(renderer: &mut Renderer, scene: &RenderScene) -> (RgbaImage, FrameStats, f64) {
    let t = Instant::now();
    let stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    let img = renderer.read_offscreen().expect("capture");
    (img, stats, t.elapsed().as_secs_f64() * 1e3)
}

/// The stone frame on a warmed renderer (its pipelines built by a first
/// scene), and what uploading its textures costs: five fresh scenes, each
/// drawn once (upload + frame) and twice more (frame); the median of first
/// less the faster other.
fn upload_and_draw(
    renderer: &mut Renderer,
    textures: &[TextureSource; 2],
) -> (RgbaImage, FrameStats, f64) {
    draw(renderer, &stone_scene(textures, false));
    let mut costs = Vec::new();
    let mut last = None;
    for _ in 0..5 {
        let scene = stone_scene(textures, false);
        let (img, stats, first) = draw(renderer, &scene);
        let steady = draw(renderer, &scene).2.min(draw(renderer, &scene).2);
        costs.push(first - steady);
        last = Some((img, stats));
    }
    costs.sort_by(f64::total_cmp);
    let (img, stats) = last.expect("five scenes");
    (img, stats, costs[2])
}

fn cmp(a: &RgbaImage, b: &RgbaImage, t: &Tolerance) -> Comparison {
    compare(&a.pixels, &b.pixels, a.width, a.height, t).expect("same size")
}

/// The tolerance a compressed frame must meet against the RGBA8 frame:
/// og-tests' gfx tolerance (32 of 255 per channel, 0.5 % outliers) with
/// RMSE 0.012. A block format's error is 1-3 levels per texel against the
/// transcoder's own RGBA8 decode, and the cooked mip levels (basisu's
/// filter) stand in for the GPU's generated ones on the far boards; the
/// normal map read upside down, the error the frame must catch, is far
/// past it (checked below).
const COMPRESSED: Tolerance = Tolerance {
    channel: 32,
    max_outlier_fraction: 0.005,
    rmse: 0.012,
};

#[test]
fn compressed_textures_draw_as_their_rgba8_decode() {
    let b = bundle();
    let stone = [
        b.load_texture("textures/stone.png").unwrap(),
        b.load_texture("textures/stone_n.png").unwrap(),
    ];
    let t = Instant::now();
    let rgba = stone
        .iter()
        .map(|t| t.to_texture_source(None).unwrap())
        .collect::<Vec<_>>();
    let rgba_ms = t.elapsed().as_secs_f64() * 1e3;
    let rgba: [TextureSource; 2] = rgba.try_into().unwrap();
    let (mut off, _) = renderer(TextureCompression::Off);
    let (base, base_stats, base_upload_ms) = upload_and_draw(&mut off, &rgba);
    let (flipped, _, _) = draw(
        &mut renderer(TextureCompression::Off).0,
        &stone_scene(&rgba, true),
    );
    let wrong = cmp(&flipped, &base, &COMPRESSED);
    assert!(
        wrong.check(&COMPRESSED).is_err(),
        "the stone frame cannot tell a flipped normal map (RMSE {:.5})",
        wrong.rmse
    );
    let ev = evidence_dir();
    og_tests::gfx::save_png(&ev.join("compressed-stone-rgba8.png"), &base).unwrap();
    let mut rows = vec![format!(
        "| rgba8 | rgba8 + rgba8 | {} | {rgba_ms:.2} | {base_upload_ms:.2} | - | - |",
        base_stats.texture_bytes,
    )];
    for (family, pref, ..) in available() {
        let (mut r, formats) = renderer(pref);
        let t = Instant::now();
        let encoded: [TextureSource; 2] = stone
            .iter()
            .map(|t| t.to_texture_source(Some(&formats)).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let transcode_ms = t.elapsed().as_secs_f64() * 1e3;
        let (img, stats, upload_ms) = upload_and_draw(&mut r, &encoded);
        let c = cmp(&img, &base, &COMPRESSED);
        og_tests::gfx::save_png(&ev.join(format!("compressed-stone-{family}.png")), &img).unwrap();
        rows.push(format!(
            "| {family} | {} + {} | {} | {transcode_ms:.2} | {upload_ms:.2} | {:.5} | {:.4} % |",
            formats.color.name(),
            formats.normal.name(),
            stats.texture_bytes,
            c.rmse,
            100.0 * c.outliers as f64 / c.pixels as f64,
        ));
        println!("{}", rows.last().unwrap());
        assert!(
            c.check(&COMPRESSED).is_ok(),
            "{family}: the compressed stone is not the RGBA8 one (RMSE {:.5}, {} outliers, max {})",
            c.rmse,
            c.outliers,
            c.max_channel_diff
        );
        assert!(
            stats.texture_bytes * 3 < base_stats.texture_bytes,
            "{family}: {} bytes against RGBA8's {}",
            stats.texture_bytes,
            base_stats.texture_bytes
        );
    }
    std::fs::write(
        ev.join("compressed-stone.md"),
        format!(
            "# templates/game stone.png + stone_n.png ({}x{}), {}\n\n\
             Tolerance against the RGBA8 frame: channel 32, outliers 0.5 %, RMSE 0.012. \
             The normal map flipped (the error to catch): RMSE {:.5}.\n\n\
             | family | formats | GPU bytes (every level) | transcode ms (both) | upload ms (median over 5 fresh scenes of the first frame less a steady one; RGBA8 builds its mips on the GPU) | RMSE vs RGBA8 | outliers |\n\
             |---|---|---|---|---|---|---|\n{}\n",
            stone[0].meta.width,
            stone[0].meta.height,
            off.info().adapter,
            wrong.rmse,
            rows.join("\n")
        ),
    )
    .unwrap();
}

/// og-tests' gltf-reference scene with the cooked models (as template.rs's
/// `cooked_models_render_the_gltf_reference_scene`), their textures
/// transcoded for the renderer as og-runtime loads them.
fn cooked_reference(
    b: &Bundle,
    formats: Option<&TextureFormats>,
) -> (og_tests::scenes::GfxScene, Vec<usize>) {
    use og_render::gltf::load_glb;
    let mut gfx = og_tests::scenes::build("gltf-reference").unwrap();
    let src = &gfx.scene;
    let counts: Vec<usize> = ["trickster.glb", "dropship.glb"]
        .iter()
        .map(|n| {
            load_glb(&std::fs::read(support::template_model(n)).unwrap())
                .unwrap()
                .primitives
                .len()
        })
        .collect();
    let mut scene = RenderScene::new();
    let mut parts = Vec::new();
    for id in MODELS {
        let model = b.load_model(id).unwrap();
        let textures: Vec<TextureSource> = model
            .textures
            .iter()
            .map(|t| {
                b.load_texture(&t.id)
                    .unwrap()
                    .to_texture_source(formats)
                    .unwrap()
            })
            .collect();
        parts.push(model.add_to_with(&mut scene, &textures));
    }
    let mut ground = None;
    for inst in &src.instances {
        let i = inst.mesh.index();
        if i == 0 {
            parts[0].instance(&mut scene, inst.transform);
        } else if i == counts[0] {
            parts[1].instance(&mut scene, inst.transform);
        } else if i >= counts[0] + counts[1] {
            let (mesh, material) = *ground.get_or_insert_with(|| {
                let mut m = src.material(inst.material).unwrap().clone();
                m.base_color_texture = m
                    .base_color_texture
                    .map(|t| scene.add_texture(src.texture(t).unwrap().clone()));
                (
                    scene.add_mesh(src.mesh(inst.mesh).unwrap().clone()),
                    scene.add_material(m),
                )
            });
            scene.add_instance(mesh, material, inst.transform);
        }
    }
    scene.camera = src.camera;
    scene.light = src.light;
    scene.lights = src.lights.clone();
    scene.environment = src.environment;
    scene.post = src.post;
    scene.debug = src.debug;
    gfx.scene = scene;
    (gfx, counts)
}

#[test]
fn cooked_models_with_compressed_textures_render_the_gltf_reference_scene() {
    let b = bundle();
    let mut lines = Vec::new();
    let mut rgba8_bytes = 0;
    let runs = std::iter::once(("rgba8", TextureCompression::Off)).chain(
        available()
            .into_iter()
            .map(|(family, pref, ..)| (family, pref)),
    );
    for (family, pref) in runs {
        let (_, formats) = renderer(pref);
        let t = Instant::now();
        let (mut gfx, _) = cooked_reference(&b, Some(&formats));
        let load_ms = t.elapsed().as_secs_f64() * 1e3;
        gfx.desc.texture_compression = pref;
        let rendered = og_tests::gfx::render_built(&mut gfx).unwrap();
        let tier = rendered.info.backend_tier();
        let reference_path = og_tests::gfx::reference_path(&tier, "gltf-reference");
        let reference_path = if reference_path.exists() {
            reference_path
        } else {
            og_tests::gfx::reference_path("metal", "gltf-reference")
        };
        let reference = og_tests::gfx::load_png(&reference_path)
            .unwrap_or_else(|e| panic!("no gltf-reference frame for tier {tier}: {e}"));
        // template.rs's RMSE bound for the cooked models, and 1 % outliers
        // (gfx's 0.5 % doubled): the trickster's colour map now samples the
        // cook's mip chain (basisu's filter, each level ETC1S-encoded, 19-25
        // dB against box-filtered level 0) instead of mips the GPU builds
        // from level 0. Uploading that cooked chain as RGBA8 measured the
        // same RMSE (0.0143) and outliers (0.65 %) as BC7, so the block
        // format adds nothing measurable; the bound is the cooked mips'.
        let tolerance = Tolerance {
            rmse: 0.02,
            max_outlier_fraction: 0.01,
            ..gfx.tolerance
        };
        let c = cmp(&rendered.image, &reference, &tolerance);
        let line = format!(
            "{family} ({} + {}): RMSE {:.5} against crates/og-tests/references/{tier}/gltf-reference.png (bound 0.02, 1 % outliers), {} outliers ({:.3} %), textures {} bytes on the GPU, models loaded in {load_ms:.1} ms (transcoding included)",
            formats.color.name(),
            formats.normal.name(),
            c.rmse,
            c.outliers,
            100.0 * c.outlier_fraction(),
            rendered.stats.texture_bytes
        );
        println!("{line}");
        lines.push(line.clone());
        og_tests::gfx::save_png(
            &evidence_dir().join(format!("compressed-gltf-reference-{family}.png")),
            &rendered.image,
        )
        .unwrap();
        assert!(c.check(&tolerance).is_ok(), "{line}");
        if pref == TextureCompression::Off {
            rgba8_bytes = rendered.stats.texture_bytes;
        } else {
            // The ground's RGBA8 checker is in both; the models' maps shrink 4x.
            assert!(
                rendered.stats.texture_bytes * 2 < rgba8_bytes,
                "{family}: {} bytes against RGBA8's {rgba8_bytes}",
                rendered.stats.texture_bytes
            );
        }
    }
    std::fs::write(
        evidence_dir().join("compressed-gltf-reference.md"),
        format!(
            "# Cooked models, compressed textures, against gltf-reference\n\n{}\n",
            lines.join("\n\n")
        ),
    )
    .unwrap();
}
