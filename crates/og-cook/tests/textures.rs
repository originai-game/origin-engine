//! KTX2 sources with their orientation and swizzle honoured, and streamed
//! textures cooked as a mip tail and a whole chain.
//!
//! Ways it could fail, and what catches it:
//! - a `ru` KTX2 cooked as stored (upside down), a swizzle ignored, or an
//!   identity KTX2 re-encoded instead of taken as it is →
//!   `ktx2_sources_are_flipped_and_swizzled_to_rd_rgba_or_taken_as_they_are`
//! - a streamed texture's tail and chain from two encodes, a texture inside
//!   the tail split, or stream accepted where it cannot apply →
//!   `a_streamed_texture_cooks_its_mip_tail_and_its_whole_chain`
//! - an alpha-tested material's base colour cooked with the encoder's own
//!   mips (its leaves averaged away when minified), or every texture of the
//!   image, or its other slots, cooked as a cut-out →
//!   `an_alpha_tested_base_colour_is_a_cut_out_that_keeps_its_coverage`
//!
//! Evidence: `$OG_EVIDENCE_DIR/cook/textures-*.json`.

use og_cook::cook::{CookOptions, cook_project};
use og_cook::ktx2;
use og_cook::runtime::Bundle;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn evidence(name: &str, text: String) {
    let base = std::env::var_os("OG_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("evidence"));
    let dir = base.join("cook");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), text).unwrap();
}

fn project(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-textures")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/tex")).unwrap();
    root
}

/// A test card: every texel different, so a flip or a swizzle shows.
fn card(w: u32, h: u32, seed: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            out.extend_from_slice(&[
                (x * 255 / (w - 1)) as u8,
                (y * 255 / (h - 1)) as u8,
                ((x + y + seed * 40) % 256) as u8,
                if (x / 4 + y / 4 + seed).is_multiple_of(2) {
                    255
                } else {
                    200
                },
            ]);
        }
    }
    out
}

fn png(path: &Path, w: u32, h: u32, rgba: &[u8]) {
    image::save_buffer(path, rgba, w, h, image::ColorType::Rgba8).unwrap();
}

fn flip_swizzle(rgba: &[u8], w: u32, h: u32, flip_y: bool, sw: [usize; 4]) -> Vec<u8> {
    let mut out = vec![0u8; rgba.len()];
    for y in 0..h {
        let sy = if flip_y { h - 1 - y } else { y };
        for x in 0..w {
            let (s, d) = (((sy * w + x) * 4) as usize, ((y * w + x) * 4) as usize);
            for c in 0..4 {
                out[d + c] = rgba[s + sw[c]];
            }
        }
    }
    out
}

/// PSNR over RGB.
fn psnr(a: &[u8], b: &[u8]) -> f64 {
    let (mut sum, mut n) = (0f64, 0f64);
    for (p, q) in a.chunks(4).zip(b.chunks(4)) {
        for c in 0..3 {
            let d = f64::from(p[c]) - f64::from(q[c]);
            sum += d * d;
            n += 1.0;
        }
    }
    let mse = sum / n;
    if mse == 0.0 {
        return f64::INFINITY;
    }
    10.0 * (255.0f64 * 255.0 / mse).log10()
}

#[test]
fn ktx2_sources_are_flipped_and_swizzled_to_rd_rgba_or_taken_as_they_are() {
    let root = project("ktx2");
    let (w, h) = (32, 16);
    let src = card(w, h, 0);
    // An RGBA8 KTX2 written bottom row first, with its channels stored as
    // BGRA: KTXorientation `ru`, KTXswizzle `bgra`.
    let stored = flip_swizzle(&src, w, h, true, [2, 1, 0, 3]);
    let plain = ktx2::write_rgba8_zlib(w, h, &stored, true, false, "test");
    let tagged =
        ktx2::with_metadata(&plain, &[("KTXorientation", "ru"), ("KTXswizzle", "bgra")]).unwrap();
    let info = ktx2::parse(&tagged).unwrap();
    assert_eq!(
        (info.orientation.as_deref(), info.swizzle.as_deref()),
        (Some("ru"), Some("bgra"))
    );
    assert_eq!(
        ktx2::decode_rgba8(&tagged, 0).unwrap().2,
        stored,
        "metadata leaves the texels"
    );
    std::fs::write(root.join("assets/tex/upside.ktx2"), &tagged).unwrap();
    std::fs::write(
        root.join("assets/tex/upside.ktx2.ogt"),
        "type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n",
    )
    .unwrap();
    // The same texels as stored, with no metadata (rd, rgba): taken as they are.
    let as_is = ktx2::write_rgba8_zlib(w, h, &src, true, false, "test");
    std::fs::write(root.join("assets/tex/plain.ktx2"), &as_is).unwrap();
    // A Basis ETC1S KTX2 the cook made from a PNG, tagged `ru`: decoded,
    // flipped and encoded again.
    let stone =
        std::fs::read(repo().join("tests/fixtures/cook-assets/assets/textures/stone.png")).unwrap();
    std::fs::write(root.join("assets/tex/stone.png"), &stone).unwrap();
    let first = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(first.ok(), "{:?}", first.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let stone_ktx = b.load_texture("tex/stone.png").unwrap().ktx2;
    let flipped_stone = ktx2::with_metadata(&stone_ktx, &[("KTXorientation", "ru")]).unwrap();
    std::fs::write(root.join("assets/tex/stone_ru.ktx2"), &flipped_stone).unwrap();
    std::fs::write(root.join("assets/tex/stone_rd.ktx2"), &stone_ktx).unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let notes: Vec<String> = first
        .assets
        .iter()
        .chain(&report.assets)
        .flat_map(|a| a.notes.clone())
        .collect();
    let b = Bundle::open(&root.join("cooked")).unwrap();

    let upside = b.load_texture("tex/upside.ktx2").unwrap();
    let px = upside.to_texture_data().unwrap().rgba;
    assert_eq!(
        px, src,
        "flipped to rd and swizzled back to rgba, lossless as pixel art"
    );
    let plain_t = b.load_texture("tex/plain.ktx2").unwrap();
    assert_eq!(
        plain_t.ktx2, as_is,
        "an rd/rgba RGBA8 KTX2 is taken as it is"
    );
    let rd = b.load_texture("tex/stone_rd.ktx2").unwrap();
    assert_eq!(rd.ktx2, stone_ktx, "an rd Basis KTX2 is taken as it is");
    let decoded_stone = b
        .load_texture("tex/stone.png")
        .unwrap()
        .to_texture_data()
        .unwrap();
    let ru = b
        .load_texture("tex/stone_ru.ktx2")
        .unwrap()
        .to_texture_data()
        .unwrap();
    let (sw, sh) = (decoded_stone.width, decoded_stone.height);
    let upright = flip_swizzle(&decoded_stone.rgba, sw, sh, true, [0, 1, 2, 3]);
    let p_right = psnr(&ru.rgba, &upright);
    let p_wrong = psnr(&ru.rgba, &decoded_stone.rgba);
    assert!(
        p_right > 30.0,
        "the ru stone re-encoded the right way up: {p_right:.1} dB"
    );
    assert!(
        p_right > p_wrong + 5.0,
        "flipped: {p_right:.1} dB against {p_wrong:.1} unflipped"
    );
    assert!(
        notes
            .iter()
            .any(|n| n.contains("flipped y to rd") && n.contains("swizzled")),
        "{notes:?}"
    );
    assert!(notes.iter().any(|n| n.contains("taken as it is")));
    evidence(
        "textures-ktx2.json",
        format!(
            "{{\"rgba8_ru_bgra\": \"lossless round trip\", \"basis_ru_psnr_upright\": {p_right:.2}, \"basis_ru_psnr_as_stored\": {p_wrong:.2}, \"identity_taken_as_is\": true}}\n"
        ),
    );
}

/// `texture { stream true }` (G3-3): a large texture cooks its mip tail as
/// `<id>` and its whole chain as `<id>#mips`, from one encode; a texture
/// within the tail is not split.
#[test]
fn a_streamed_texture_cooks_its_mip_tail_and_its_whole_chain() {
    let root = project("stream");
    let import = |file: &str, body: &str| {
        std::fs::write(
            root.join(format!("assets/tex/{file}.ogt")),
            format!("type import\nversion 1\n---\ntexture {{\n{body}}}\n"),
        )
        .unwrap();
    };
    png(
        &root.join("assets/tex/big.png"),
        1024,
        1024,
        &card(1024, 1024, 0),
    );
    import("big.png", "  stream true\n");
    png(
        &root.join("assets/tex/bump.png"),
        1024,
        1024,
        &card(1024, 1024, 1),
    );
    import("bump.png", "  kind normal\n  stream true\n");
    png(
        &root.join("assets/tex/small.png"),
        128,
        128,
        &card(128, 128, 2),
    );
    import("small.png", "  stream true\n");
    // A model's textures, by `model { stream_textures true }`: 512, so
    // its tail starts at level 1.
    png(
        &root.join("assets/tex/rock.png"),
        512,
        512,
        &card(512, 512, 3),
    );
    std::fs::write(
        root.join("assets/tex/m.gltf"),
        r#"{"asset":{"version":"2.0"},"images":[{"uri":"rock.png"}],"textures":[{"source":0}],"materials":[{"pbrMetallicRoughness":{"baseColorTexture":{"index":0}}}]}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("assets/tex/m.gltf.ogt"),
        "type import\nversion 1\n---\nmodel {\n  stream_textures true\n}\n",
    )
    .unwrap();
    // In a streaming group: the group's pack holds the whole chain.
    png(
        &root.join("assets/tex/far.png"),
        512,
        512,
        &card(512, 512, 4),
    );
    import("far.png", "  stream true\n");
    std::fs::write(
        root.join("assets/world.ogt"),
        "type scene\nversion 1\n---\nname \"World\"\nstream \"far\" {\n  asset \"tex/far.png\"\n}\n",
    )
    .unwrap();
    let report =
        og_cook::cook::cook_project_with_texture_cache(&root, &CookOptions::default(), None)
            .unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let manifest = og_cook::runtime::Manifest::parse(
        &std::fs::read_to_string(root.join("cooked/.cook.ogt")).unwrap(),
    )
    .unwrap();
    let stamp = |id: &str| {
        manifest
            .outputs()
            .find(|o| o.id == id)
            .unwrap_or_else(|| panic!("no output {id}"))
            .stamp
    };
    let mut lines = Vec::new();
    for (id, codec) in [("tex/big.png", "etc1s"), ("tex/bump.png", "uastc")] {
        let mips_id = og_cook::runtime::mips_id(id);
        let tail = b.load_texture(id).unwrap();
        let whole = b.load_texture(&mips_id).unwrap();
        assert_ne!(stamp(id), stamp(&mips_id), "{id}: the two outputs' stamps");
        // The whole chain is the ordinary blob: every level, no `mips`.
        assert_eq!(
            (whole.meta.width, whole.meta.height, whole.meta.levels),
            (1024, 1024, 11),
            "{id}#mips"
        );
        assert_eq!(whole.mips, None, "{id}#mips");
        // The tail: level 2 (256) down, the whole chain's size beside it.
        assert_eq!(
            (tail.meta.width, tail.meta.height, tail.meta.levels),
            (256, 256, 9),
            "{id}"
        );
        assert_eq!(
            (tail.meta.kind, tail.meta.codec, tail.meta.color_space),
            (whole.meta.kind, whole.meta.codec, whole.meta.color_space)
        );
        assert_eq!(
            tail.mips,
            Some(og_cook::runtime::MipChain {
                width: 1024,
                height: 1024,
                levels: 11
            }),
            "{id}"
        );
        let (ti, wi) = (
            ktx2::parse(&tail.ktx2).unwrap(),
            ktx2::parse(&whole.ktx2).unwrap(),
        );
        assert_eq!((ti.width, ti.height, ti.levels.len()), (256, 256, 9));
        let level = |bytes: &[u8], l: &ktx2::Level| bytes[l.offset..l.offset + l.length].to_vec();
        assert_eq!(
            level(&tail.ktx2, &ti.levels[0]),
            level(&whole.ktx2, &wi.levels[2]),
            "{id}: the tail's level 0 is the whole chain's level 2"
        );
        for (n, l) in ti.levels.iter().enumerate() {
            assert_eq!(
                level(&tail.ktx2, l),
                level(&whole.ktx2, &wi.levels[n + 2]),
                "{id}: tail level {n}"
            );
        }
        let decoded = tail.to_texture_data().unwrap();
        assert_eq!((decoded.width, decoded.height), (256, 256), "{id}");
        assert_eq!(decoded.rgba.len(), 256 * 256 * 4);
        lines.push(format!(
            "\"{id}\": {{\"codec\": \"{codec}\", \"tail_bytes\": {}, \"whole_bytes\": {}}}",
            tail.ktx2.len(),
            whole.ktx2.len()
        ));
    }
    // A glTF's texture is cooked once per image and use, under the id its
    // model names (`_gltf/<image hash>-<kind>-stream#tex`).
    let gltf_tex = b.load_model("tex/m.gltf").unwrap().textures[0].id.clone();
    assert!(gltf_tex.starts_with("_gltf/") && gltf_tex.ends_with("-stream#tex"));
    let (tail, whole) = (
        b.load_texture(&gltf_tex).unwrap(),
        b.load_texture(&og_cook::runtime::mips_id(&gltf_tex))
            .unwrap(),
    );
    assert_eq!(
        (
            tail.meta.width,
            tail.meta.levels,
            whole.meta.width,
            whole.meta.levels
        ),
        (256, 9, 512, 10)
    );
    assert_eq!(
        tail.mips,
        Some(og_cook::runtime::MipChain {
            width: 512,
            height: 512,
            levels: 10
        })
    );
    let (ti, wi) = (
        ktx2::parse(&tail.ktx2).unwrap(),
        ktx2::parse(&whole.ktx2).unwrap(),
    );
    let (t0, w1) = (&ti.levels[0], &wi.levels[1]);
    assert_eq!(
        tail.ktx2[t0.offset..t0.offset + t0.length],
        whole.ktx2[w1.offset..w1.offset + w1.length],
        "the model texture's tail starts at level 1"
    );
    // Within the tail: one ordinary output, and the report says why.
    let small = b.load_texture("tex/small.png").unwrap();
    assert_eq!(
        (small.meta.width, small.meta.levels, small.mips),
        (128, 8, None)
    );
    assert!(manifest.outputs().all(|o| o.id != "tex/small.png#mips"));
    let notes: Vec<&String> = report.assets.iter().flat_map(|a| &a.notes).collect();
    assert!(
        notes
            .iter()
            .any(|n| n.starts_with("tex/small.png: not streamed by level")),
        "{notes:?}"
    );
    let far = manifest.outputs().find(|o| o.id == "tex/far.png").unwrap();
    assert_eq!(far.group.as_deref(), Some("far"));
    assert!(manifest.outputs().all(|o| o.id != "tex/far.png#mips"));
    assert_eq!(b.load_texture("tex/far.png").unwrap().meta.levels, 10);
    assert!(
        notes.iter().any(|n| n
            .starts_with("tex/far.png: not streamed by level: it is in streaming group \"far\"")),
        "{notes:?}"
    );
    evidence(
        "textures-stream.json",
        format!("{{{}}}\n", lines.join(", ")),
    );

    // Refused: stream on pixel art (no mips), and a value not a boolean.
    let bad = project("stream-refused");
    png(&bad.join("assets/tex/a.png"), 8, 8, &card(8, 8, 0));
    png(&bad.join("assets/tex/b.png"), 8, 8, &card(8, 8, 0));
    std::fs::write(
        bad.join("assets/tex/a.png.ogt"),
        "type import\nversion 1\n---\ntexture {\n  kind pixel\n  stream true\n}\n",
    )
    .unwrap();
    std::fs::write(
        bad.join("assets/tex/b.png.ogt"),
        "type import\nversion 1\n---\ntexture {\n  stream yes\n}\n",
    )
    .unwrap();
    let r = og_cook::cook::cook_project_with_texture_cache(&bad, &CookOptions::default(), None)
        .unwrap();
    let err = |rel: &str| {
        r.assets
            .iter()
            .find(|a| a.source == rel)
            .and_then(|a| a.error.clone())
            .unwrap_or_default()
    };
    assert!(
        err("tex/a.png").contains("kind pixel"),
        "{:?}",
        err("tex/a.png")
    );
    assert!(
        err("tex/b.png").starts_with("line 5: stream takes true"),
        "{:?}",
        err("tex/b.png")
    );
}

/// The bamboo spray card of texture.rs's tests (daguanyuan's zhu_spray, 1024
/// x 384, at 32 x 12): green leaves in rows 4 to 7, three columns in four,
/// the rest transparent.
fn bamboo_card() -> Vec<u8> {
    (0..12u32)
        .flat_map(|y| (0..32u32).map(move |x| (4..8).contains(&y) && x % 4 != 3))
        .flat_map(|leaf| [60, 140, 50, if leaf { 255 } else { 0 }])
        .collect()
}

#[cfg(feature = "basis")]
#[test]
fn an_alpha_tested_base_colour_is_a_cut_out_that_keeps_its_coverage() {
    use basis_universal::{TranscodeParameters, Transcoder, TranscoderTextureFormat};
    let root = project("cutout");
    png(&root.join("assets/tex/zhu.png"), 32, 12, &bamboo_card());
    // One image under three materials: alpha-tested (glTF MASK, which also
    // samples it as its metallic-roughness), opaque, and blended (which
    // also samples it as its emissive).
    std::fs::write(
        root.join("assets/tex/m.gltf"),
        r#"{"asset":{"version":"2.0"},"images":[{"uri":"zhu.png"}],"textures":[{"source":0}],"materials":[
{"name":"cut","alphaMode":"MASK","alphaCutoff":0.35,"pbrMetallicRoughness":{"baseColorTexture":{"index":0},"metallicRoughnessTexture":{"index":0}}},
{"name":"solid","pbrMetallicRoughness":{"baseColorTexture":{"index":0}}},
{"name":"blend","alphaMode":"BLEND","pbrMetallicRoughness":{"baseColorTexture":{"index":0}},"emissiveTexture":{"index":0},"emissiveFactor":[1,1,1]}]}"#,
    )
    .unwrap();
    let report =
        og_cook::cook::cook_project_with_texture_cache(&root, &CookOptions::default(), None)
            .unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let m = b.load_model("tex/m.gltf").unwrap();
    let id = |t: Option<usize>| m.textures[t.expect("a texture")].id.clone();
    let (cut, solid, blend) = (&m.materials[0], &m.materials[1], &m.materials[2]);
    // The alpha-tested material's base colour is a texture of its own; the
    // opaque and blended materials' base colour and the emissive are one
    // ordinary texture, and the metallic-roughness its own kind.
    let cutout = id(cut.base_color_texture);
    let plain = id(solid.base_color_texture);
    assert!(cutout.ends_with("-0-cutout90#tex"), "{cutout}");
    assert!(plain.ends_with("-0#tex"), "{plain}");
    assert_eq!(id(blend.base_color_texture), plain);
    assert_eq!(id(blend.emissive_texture), plain);
    assert!(id(cut.metallic_roughness_texture).ends_with("-2#tex"));
    assert_eq!(m.textures.len(), 3);
    // Their levels, each one's share of texels at or above the cut-off:
    // the cut-out keeps the card's (0.25) to 1 x 1; the ordinary texture,
    // its leaf rows spread over the transparent ones by the encoder's filter,
    // has none from 4 x 1 down.
    let coverage = |id: &str| -> Vec<f32> {
        let basis = og_cook::ktx2::to_basis(&b.load_texture(id).unwrap().ktx2).unwrap();
        let mut t = Transcoder::new();
        t.prepare_transcoding(&basis).unwrap();
        let levels = (0..t.image_level_count(&basis, 0))
            .map(|level_index| {
                let rgba = t
                    .transcode_image_level(
                        &basis,
                        TranscoderTextureFormat::RGBA32,
                        TranscodeParameters {
                            image_index: 0,
                            level_index,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                let texels = rgba.as_chunks::<4>().0;
                texels.iter().filter(|p| p[3] >= 90).count() as f32 / texels.len() as f32
            })
            .collect();
        t.end_transcoding();
        levels
    };
    let (c, p) = (coverage(&cutout), coverage(&plain));
    assert_eq!((c.len(), p.len()), (6, 6), "32x12 to 1x1");
    assert_eq!((c[0], p[0]), (0.25, 0.25));
    assert!(c.iter().all(|&c| c >= 0.25), "cut-out {c:?}");
    assert!(p[3..].iter().all(|&p| p == 0.0), "ordinary {p:?}");
}
