//! KTX2 sources with their orientation and swizzle honoured, texture
//! arrays cooked and uploaded as one 2D-array texture, and the `toktx`
//! encoder (when it is on PATH).
//!
//! Ways it could fail, and what catches it:
//! - a `ru` KTX2 cooked as stored (upside down), a swizzle ignored, or an
//!   identity KTX2 re-encoded instead of taken as it is →
//!   `ktx2_sources_are_flipped_and_swizzled_to_rd_rgba_or_taken_as_they_are`
//! - an array's layers out of order, of different sizes accepted, padded
//!   wrongly, or not one `D2Array` texture →
//!   `a_texture_array_cooks_its_layers_and_uploads_them_as_one_array`
//! - the toktx fallback writing another orientation, a broken texture, or
//!   not running at all → `toktx_cooks_the_template_textures` (skipped with
//!   its reason when `toktx` is not on PATH)
//!
//! Evidence: `$OG_EVIDENCE_DIR/P4/cook/textures.json`.

use og_cook::cook::{CookOptions, TextureOptions, cook_project};
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
    let dir = base.join("P4/cook");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), text).unwrap();
}

fn project(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-textures-p4")
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
    let stone = std::fs::read(repo().join("templates/game/assets/textures/stone.png")).unwrap();
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

#[test]
fn a_texture_array_cooks_its_layers_and_uploads_them_as_one_array() {
    let root = project("array");
    let (w, h) = (64, 32);
    let cards: Vec<Vec<u8>> = (0..3).map(|i| card(w, h, i)).collect();
    for (i, c) in cards.iter().enumerate() {
        png(&root.join(format!("assets/tex/l{i}.png")), w, h, c);
    }
    std::fs::write(
        root.join("assets/tex/tiles.ogt"),
        "type texture-array\nversion 1\n---\nkind pixel\nlayer \"l0.png\"\nlayer \"l1.png\"\nlayer \"tex/l2.png\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("assets/tex/colour.ogt"),
        "type texture-array\nversion 1\n---\nlayer \"l2.png\"\nlayer \"l0.png\"\n",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let id = "tex/tiles.ogt";
    let arr = b.texture_array(id, b.read(id).unwrap().as_slice()).unwrap();
    assert_eq!(
        arr.layers,
        [
            "tex/tiles.ogt#layer0",
            "tex/tiles.ogt#layer1",
            "tex/tiles.ogt#layer2"
        ]
    );
    assert_eq!(
        (
            arr.meta.width,
            arr.meta.height,
            arr.meta.layers,
            arr.meta.levels
        ),
        (64, 32, 3, 1)
    );
    // Each layer is a texture of its own too.
    assert_eq!(
        b.load_texture("tex/tiles.ogt#layer1")
            .unwrap()
            .to_texture_data()
            .unwrap()
            .rgba,
        cards[1]
    );

    // Uploaded: one D2Array texture holding every layer, in order.
    let mut renderer = og_render::Renderer::new_blocking(&og_render::RenderDesc {
        width: 64,
        height: 64,
        ..Default::default()
    })
    .unwrap();
    let mut scene = og_render::RenderScene::new();
    let pixel = scene
        .add_texture_array(arr.to_texture_array(&b, None).unwrap())
        .unwrap();
    let cid = "tex/colour.ogt";
    let colour = b
        .texture_array(cid, b.read(cid).unwrap().as_slice())
        .unwrap();
    let formats = renderer.info().texture_formats.clone();
    let compressed = scene
        .add_texture_array(colour.to_texture_array(&b, Some(&formats)).unwrap())
        .unwrap();
    let stats = renderer
        .render(&scene, &og_render::RenderTarget::Offscreen)
        .unwrap();
    let info = renderer.texture_array_info(pixel).unwrap();
    assert_eq!((info.layers, info.gpu_layers, info.levels), (3, 3, 1));
    assert_eq!(info.format, og_render::TextureFormat::Rgba8);
    for (i, c) in cards.iter().enumerate() {
        let read = renderer
            .read_texture_array_layer(pixel, i as u32, 0)
            .unwrap();
        assert_eq!(&read.pixels, c, "layer {i} of the array");
    }
    let cinfo = renderer.texture_array_info(compressed).unwrap();
    // Two layers need no padding (one, or a multiple of six, would).
    assert_eq!((cinfo.layers, cinfo.gpu_layers, cinfo.levels), (2, 2, 7));
    assert!(
        stats.texture_bytes >= info.bytes + cinfo.bytes,
        "counted in the frame's texture bytes"
    );

    // Layers of different sizes are refused, naming both.
    png(&root.join("assets/tex/big.png"), 128, 32, &card(128, 32, 1));
    std::fs::write(
        root.join("assets/tex/bad.ogt"),
        "type texture-array\nversion 1\n---\nlayer \"l0.png\"\nlayer \"big.png\"\n",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errs = report.errors().join("\n");
    assert!(
        errs.contains("big.png' is 128x32") && errs.contains("64x32"),
        "{errs}"
    );
    evidence(
        "textures-array.json",
        format!(
            "{{\"pixel\": {{\"layers\": {}, \"gpu_layers\": {}, \"levels\": {}, \"format\": \"{}\", \"bytes\": {}}}, \"colour\": {{\"layers\": {}, \"gpu_layers\": {}, \"levels\": {}, \"format\": \"{}\", \"bytes\": {}}}, \"frame_texture_bytes\": {}}}\n",
            info.layers,
            info.gpu_layers,
            info.levels,
            info.format.name(),
            info.bytes,
            cinfo.layers,
            cinfo.gpu_layers,
            cinfo.levels,
            cinfo.format.name(),
            cinfo.bytes,
            stats.texture_bytes
        ),
    );
}

#[test]
fn toktx_cooks_the_template_textures() {
    let version = match std::process::Command::new("toktx")
        .arg("--version")
        .output()
    {
        Ok(out) if out.status.success() => format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout).trim(),
            String::from_utf8_lossy(&out.stderr).trim()
        ),
        _ => {
            eprintln!(
                "SKIPPED toktx_cooks_the_template_textures: `toktx` is not on PATH (KTX-Software, https://github.com/KhronosGroup/KTX-Software/releases)"
            );
            return;
        }
    };
    let root = project("toktx");
    let tex = repo().join("templates/game/assets/textures");
    for f in ["stone.png", "stone_n.png", "stone_n.png.ogt"] {
        std::fs::copy(tex.join(f), root.join("assets/tex").join(f)).unwrap();
    }
    let options = CookOptions {
        texture: TextureOptions {
            force_toktx: true,
            ..TextureOptions::default()
        },
        ..CookOptions::default()
    };
    let report = cook_project(&root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    assert_eq!(
        report.texture_encoder,
        format!("toktx {version}"),
        "the version is in every texture's stamp"
    );
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let mut rows = Vec::new();
    for (id, floor) in [("tex/stone.png", 30.0), ("tex/stone_n.png", 35.0)] {
        let t = b.load_texture(id).unwrap();
        let info = ktx2::parse(&t.ktx2).unwrap();
        assert_ne!(
            info.orientation.as_deref(),
            Some("ru"),
            "{id}: toktx's orientation"
        );
        let src = image::open(root.join("assets").join(id))
            .unwrap()
            .to_rgba8();
        let got = t.to_texture_data().unwrap();
        let p = psnr(&got.rgba, src.as_raw());
        let flipped = flip_swizzle(src.as_raw(), src.width(), src.height(), true, [0, 1, 2, 3]);
        let pf = psnr(&got.rgba, &flipped);
        assert!(p >= floor, "{id}: {p:.1} dB < {floor}");
        assert!(
            p > pf,
            "{id}: rows top to bottom ({p:.1} dB, flipped {pf:.1})"
        );
        rows.push(format!(
            "{{\"id\": \"{id}\", \"psnr\": {p:.2}, \"psnr_flipped\": {pf:.2}, \"levels\": {}, \"orientation\": \"{}\"}}",
            info.levels.len(),
            info.orientation.as_deref().unwrap_or("absent")
        ));
    }
    // toktx's own `ru` output (rows bottom first), as a KTX2 source.
    let out = root.join("ru.ktx2");
    let st = std::process::Command::new("toktx")
        .args([
            "--t2",
            "--encode",
            "etc1s",
            "--lower_left_maps_to_s0t0",
            "--assign_oetf",
            "srgb",
        ])
        .arg(&out)
        .arg(tex.join("stone.png"))
        .status()
        .unwrap();
    assert!(st.success());
    let ru = std::fs::read(&out).unwrap();
    assert_eq!(ktx2::parse(&ru).unwrap().orientation.as_deref(), Some("ru"));
    std::fs::write(root.join("assets/tex/ru.ktx2"), &ru).unwrap();
    let report = cook_project(&root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let got = b
        .load_texture("tex/ru.ktx2")
        .unwrap()
        .to_texture_data()
        .unwrap();
    let src = image::open(tex.join("stone.png")).unwrap().to_rgba8();
    let p = psnr(&got.rgba, src.as_raw());
    assert!(p >= 28.0, "toktx's ru stone, flipped back: {p:.1} dB");
    rows.push(format!("{{\"id\": \"tex/ru.ktx2\", \"psnr\": {p:.2}}}"));
    evidence(
        "textures-toktx.json",
        format!(
            "{{\"toktx\": \"{version}\", \"textures\": [{}]}}\n",
            rows.join(", ")
        ),
    );
}
