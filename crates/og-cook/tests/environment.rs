//! Environment maps through the cook and back (phase 4): a `type
//! environment` .ogt and its equirectangular HDR cook into og-render's
//! prefiltered map, byte for byte what og-render's `bake` makes of the same
//! image and settings, which the runtime loads; the image is part of the
//! environment (not listed as ignored, its edit recooks it); a bad file is
//! refused by name.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

fn project(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-environment")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

fn cook(root: &Path) -> og_cook::Report {
    let report = cook_project(root, &CookOptions::default()).expect("cook");
    assert!(report.ok(), "{:?}", report.errors());
    report
}

// ── cube map sources ─────────────────────────────────────────────────────

/// The studio as six `n`² faces: per texel, 4 x 4 bilinear samples of the
/// equirectangular image (og-render's mapping) at og-render's face
/// directions.
fn studio_cube(hdr: &[u8], n: u32) -> Vec<f32> {
    use std::f32::consts::PI;
    let img = image::load_from_memory(hdr).unwrap().to_rgb32f();
    let (w, h) = (img.width() as i64, img.height() as i64);
    let px = |x: i64, y: i64| {
        let p = img.get_pixel(x.rem_euclid(w) as u32, y.clamp(0, h - 1) as u32);
        glam::Vec3::new(p[0], p[1], p[2])
    };
    let sample = |d: glam::Vec3| {
        let u = d.z.atan2(d.x) / (2.0 * PI) + 0.5;
        let v = 0.5 - d.y.clamp(-1.0, 1.0).asin() / PI;
        let (x, y) = (u * w as f32 - 0.5, v * h as f32 - 0.5);
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (x0, y0) = (x0 as i64, y0 as i64);
        let top = px(x0, y0).lerp(px(x0 + 1, y0), fx);
        let bottom = px(x0, y0 + 1).lerp(px(x0 + 1, y0 + 1), fx);
        top.lerp(bottom, fy)
    };
    let mut rgb = Vec::new();
    for face in 0..6 {
        for j in 0..n {
            for i in 0..n {
                let mut sum = glam::Vec3::ZERO;
                for sj in 0..4 {
                    for si in 0..4 {
                        let x = 2.0 * (i as f32 + (si as f32 + 0.5) / 4.0) / n as f32 - 1.0;
                        let y = 1.0 - 2.0 * (j as f32 + (sj as f32 + 0.5) / 4.0) / n as f32;
                        sum += sample(og_render::face_direction(face, x, y));
                    }
                }
                rgb.extend((sum / 16.0).to_array());
            }
        }
    }
    rgb
}

/// A non-negative f32 as half-float bits, rounded to nearest.
fn f16(v: f32) -> u16 {
    let v = v.clamp(0.0, 65504.0);
    if v < 2f32.powi(-14) {
        return (v / 2f32.powi(-24)).round() as u16;
    }
    let e = v.log2().floor() as i32;
    let m = ((v / 2f32.powi(e) - 1.0) * 1024.0).round() as u32;
    let (e, m) = if m == 1024 { (e + 1, 0) } else { (e, m) };
    (((e + 15) as u32) << 10 | m) as u16
}

const SS_NONE: u32 = 0;
const SS_ZSTD: u32 = 2;
const SS_ZLIB: u32 = 3;

/// A KTX2 cube map of one level: the header, the level index, a basic
/// data format descriptor, the orientation when given, then the level
/// (stored plain, Zstandard or zlib).
fn ktx2_cube(vk: u32, size: u32, level: &[u8], ss: u32, orientation: Option<&str>) -> Vec<u8> {
    ktx2_chain(vk, size, &[level.to_vec()], ss, orientation)
}

/// [`ktx2_cube`] with a mip chain: `levels[k]` the six faces of level k,
/// stored smallest first as KTX2 lays them out.
fn ktx2_chain(
    vk: u32,
    size: u32,
    levels: &[Vec<u8>],
    ss: u32,
    orientation: Option<&str>,
) -> Vec<u8> {
    let stored: Vec<Vec<u8>> = levels
        .iter()
        .map(|level| match ss {
            SS_ZSTD => ruzstd::encoding::compress_to_vec(
                &level[..],
                ruzstd::encoding::CompressionLevel::Fastest,
            ),
            SS_ZLIB => miniz_oxide::deflate::compress_to_vec_zlib(level, 6),
            _ => level.clone(),
        })
        .collect();
    let type_size: u32 = match vk {
        90 | 97 => 2,
        143 | 144 => 1,
        _ => 4,
    };
    let model: u32 = if vk >= 143 { 133 } else { 1 };
    let mut dfd = Vec::new();
    for w in [
        44u32,
        0,
        2 | (40 << 16),
        model | (1 << 8) | (1 << 16),
        0,
        0,
        0,
    ] {
        dfd.extend(w.to_le_bytes());
    }
    dfd.extend([0u8; 16]);
    let mut kvd = Vec::new();
    if let Some(o) = orientation {
        let kv = format!("KTXorientation\0{o}\0");
        kvd.extend((kv.len() as u32).to_le_bytes());
        kvd.extend(kv.as_bytes());
        while kvd.len() % 4 != 0 {
            kvd.push(0);
        }
    }
    let dfd_at = 80 + 24 * levels.len();
    let kvd_at = dfd_at + dfd.len();
    let mut data_at = kvd_at + kvd.len();
    while !data_at.is_multiple_of(16) {
        data_at += 1;
    }
    // The smallest level first; each level 16-aligned.
    let mut offsets = vec![0usize; levels.len()];
    let mut at = data_at;
    for k in (0..levels.len()).rev() {
        offsets[k] = at;
        at += stored[k].len();
        while !at.is_multiple_of(16) {
            at += 1;
        }
    }
    let mut out = og_cook::ktx2::IDENTIFIER.to_vec();
    for v in [
        vk,
        type_size,
        size,
        size,
        0,
        0,
        6,
        levels.len() as u32,
        ss,
        dfd_at as u32,
        dfd.len() as u32,
        kvd_at as u32,
        kvd.len() as u32,
    ] {
        out.extend(v.to_le_bytes());
    }
    out.extend(0u64.to_le_bytes());
    out.extend(0u64.to_le_bytes());
    for k in 0..levels.len() {
        for v in [
            offsets[k] as u64,
            stored[k].len() as u64,
            levels[k].len() as u64,
        ] {
            out.extend(v.to_le_bytes());
        }
    }
    out.extend(&dfd);
    out.extend(&kvd);
    for k in (0..levels.len()).rev() {
        out.resize(offsets[k], 0);
        out.extend(&stored[k]);
    }
    out
}

/// Six faces as RGBA16F texels.
fn rgba16f(rgb: &[f32]) -> Vec<u8> {
    rgb.chunks(3)
        .flat_map(|c| [f16(c[0]), f16(c[1]), f16(c[2]), 0x3C00])
        .flat_map(u16::to_le_bytes)
        .collect()
}

/// Six faces as RGB9E5 texels.
fn rgb9e5(rgb: &[f32]) -> Vec<u8> {
    rgb.chunks(3)
        .flat_map(|c| og_render::rgb9e5([c[0], c[1], c[2]]).to_le_bytes())
        .collect()
}

/// Six `n`² faces as unsigned BC6H blocks of mode 11 (one region, 10-bit
/// endpoints): per block the channels' half-float extremes, per texel the
/// nearest of the 16 weights.
fn bc6h(rgb: &[f32], n: usize) -> Vec<u8> {
    const W: [i64; 16] = [0, 4, 9, 13, 17, 21, 26, 30, 34, 38, 43, 47, 51, 55, 60, 64];
    let mut out = Vec::new();
    for face in 0..6 {
        for by in 0..n / 4 {
            for bx in 0..n / 4 {
                let texel = |i: usize| -> [i64; 3] {
                    let (x, y) = (bx * 4 + i % 4, by * 4 + i / 4);
                    let at = ((face * n + y) * n + x) * 3;
                    [0, 1, 2].map(|c| i64::from(f16(rgb[at + c])))
                };
                // An endpoint `e` decodes to 31 e + 15 (and 1023 to 0x7BFF).
                let mut lo = [1023i64; 3];
                let mut hi = [0i64; 3];
                for i in 0..16 {
                    for (c, h) in texel(i).iter().enumerate() {
                        let x = (*h - 15) as f64 / 31.0;
                        lo[c] = lo[c].min((x.floor() as i64).clamp(0, 1023));
                        hi[c] = hi[c].max((x.ceil() as i64).clamp(0, 1023));
                    }
                }
                // What an endpoint decodes to, as half-float bits before the final scale.
                let un = |e: i64| match e {
                    0 => 0,
                    1023 => 0xFFFF,
                    e => ((e << 16) + 0x8000) >> 10,
                };
                let mut v: u128 = 0b00011;
                let mut at = 5;
                let mut put = |value: i64, bits: u32| {
                    v |= (value as u128) << at;
                    at += bits;
                };
                for e in lo.iter().chain(hi.iter()) {
                    put(*e, 10);
                }
                for i in 0..16 {
                    let t = texel(i);
                    let best = (0..if i == 0 { 8 } else { 16 })
                        .min_by_key(|&k| {
                            (0..3)
                                .map(|c| {
                                    let x = (un(lo[c]) * (64 - W[k]) + un(hi[c]) * W[k] + 32) >> 6;
                                    ((x * 31) >> 6) - t[c]
                                })
                                .map(|d| d * d)
                                .sum::<i64>()
                        })
                        .unwrap();
                    put(best as i64, if i == 0 { 3 } else { 4 });
                }
                out.extend(v.to_le_bytes());
            }
        }
    }
    out
}

/// A face's texels as a Radiance .hdr.
fn hdr_face(rgb: &[f32], n: u32) -> Vec<u8> {
    let px: Vec<image::Rgb<f32>> = rgb
        .chunks(3)
        .map(|c| image::Rgb([c[0], c[1], c[2]]))
        .collect();
    let mut out = Vec::new();
    image::codecs::hdr::HdrEncoder::new(&mut out)
        .encode(&px, n as usize, n as usize)
        .unwrap();
    out
}

/// The mean relative difference over every texel channel of every level,
/// the worst of the prefiltered levels (level 0, the radiance at 32², has
/// the two resamplings' differences at the studio's sharp panel edges),
/// and the worst relative difference of the irradiance along the six axes.
fn compare(
    a: &og_render::EnvironmentMapData,
    b: &og_render::EnvironmentMapData,
) -> (f32, f32, f32) {
    assert_eq!((a.size, a.texels.len()), (b.size, b.texels.len()));
    let (mut sum, mut worst) = (0f32, 0f32);
    let level0 = 6 * (a.size * a.size) as usize;
    let mut worst0 = 0f32;
    for (k, (p, q)) in a.texels.iter().zip(&b.texels).enumerate() {
        let (p, q) = (og_render::rgb9e5_decode(*p), og_render::rgb9e5_decode(*q));
        for c in 0..3 {
            let d = (p[c] - q[c]).abs() / p[c].max(q[c]).max(0.05);
            sum += d;
            if k < level0 {
                worst0 = worst0.max(d);
            } else {
                worst = worst.max(d);
            }
        }
    }
    println!("  level 0 worst {:.2} %", worst0 * 100.0);
    let mut irr = 0f32;
    for n in [
        glam::Vec3::X,
        -glam::Vec3::X,
        glam::Vec3::Y,
        -glam::Vec3::Y,
        glam::Vec3::Z,
        -glam::Vec3::Z,
    ] {
        let (x, y) = (
            og_render::bake::irradiance(a, n),
            og_render::bake::irradiance(b, n),
        );
        irr = irr.max(((x - y).abs() / x.max(glam::Vec3::splat(0.05))).max_element());
    }
    (sum / (3 * a.texels.len()) as f32, worst, irr)
}

const SETTINGS: &str = "size 32\nintensity 1.5\nrotation 30\nsamples 64\n";

/// The studio as an equirectangular image and as a cube map made from it
/// (a 64² cube: an RGBA16F KTX2, Zstandard-supercompressed) bakes to the
/// same map within the resampling between them; the cube as RGB9E5, as
/// BC6H (zlib) and as six .hdr faces bakes as the RGBA16F cube does,
/// within each format's precision. Measured: equirect vs RGBA16F mean
/// 0.20 % per texel channel, the prefiltered levels' worst 1.5 %, the
/// irradiance 0.17 % (level 0's worst, 29 %, is a panel edge both paths
/// resample differently); RGB9E5 0.016 %; BC6H 0.7 % and 1.6 % in
/// irradiance (this test's crude one-region encoder); six .hdr faces 0.19 %.
#[test]
fn a_cube_map_bakes_as_its_equirectangular_studio() {
    let hdr = og_tests::scenes::asset("env/studio.hdr")
        .unwrap()
        .into_owned();
    let n = 64u32;
    let cube = studio_cube(&hdr, n);
    let face = |f: usize| &cube[f * (n * n * 3) as usize..(f + 1) * (n * n * 3) as usize];
    let names = ["px", "nx", "py", "ny", "pz", "nz"];
    let faces: Vec<(String, Vec<u8>)> = (0..6)
        .map(|f| (format!("env/faces/{}.hdr", names[f]), hdr_face(face(f), n)))
        .collect();
    let env = |source: &str| format!("type environment\nversion 1\n---\n{source}\n{SETTINGS}");
    let mut files: Vec<(String, Vec<u8>)> = vec![
        ("env/studio.hdr".into(), hdr.clone()),
        ("env/studio.ogt".into(), env("source \"studio.hdr\"").into_bytes()),
        ("env/half.ktx2".into(), ktx2_cube(97, n, &rgba16f(&cube), SS_ZSTD, Some("rd"))),
        ("env/half.ogt".into(), env("source \"half.ktx2\"").into_bytes()),
        ("env/shared.ktx2".into(), ktx2_cube(123, n, &rgb9e5(&cube), SS_NONE, None)),
        ("env/shared.ogt".into(), env("source \"shared.ktx2\"").into_bytes()),
        ("env/bc6h.ktx2".into(), ktx2_cube(143, n, &bc6h(&cube, n as usize), SS_ZLIB, None)),
        ("env/bc6h.ogt".into(), env("source \"bc6h.ktx2\"").into_bytes()),
        (
            "env/faces.ogt".into(),
            env("faces \"faces/px.hdr\" \"faces/nx.hdr\" \"faces/py.hdr\" \"faces/ny.hdr\" \"faces/pz.hdr\" \"faces/nz.hdr\"")
                .into_bytes(),
        ),
    ];
    files.extend(faces);
    let refs: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(p, b)| (p.as_str(), b.as_slice()))
        .collect();
    let root = project("cube", &refs);
    let report = cook(&root);
    assert!(
        report.ignored.is_empty(),
        "the images are the environments': {:?}",
        report.ignored
    );
    let mut ids: Vec<String> = report
        .cooked_outputs()
        .iter()
        .map(|o| o.id.clone())
        .collect();
    ids.sort();
    assert_eq!(
        ids,
        [
            "env/bc6h.ogt",
            "env/faces.ogt",
            "env/half.ogt",
            "env/shared.ogt",
            "env/studio.ogt"
        ]
    );
    let notes: Vec<String> = report.assets.iter().flat_map(|a| a.notes.clone()).collect();
    assert!(
        notes
            .iter()
            .any(|n| n.contains("a RGBA16F cube map of 6 × 64²")),
        "{notes:?}"
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let load = |id: &str| bundle.load_environment(id).unwrap();
    let (studio, half) = (load("env/studio.ogt"), load("env/half.ogt"));
    // Exactly og-render's bake of the decoded cube.
    let decoded: Vec<f32> = cube
        .iter()
        .map(|v| og_cook::cook::bc6h::half(f16(*v)))
        .collect();
    let baked = og_render::bake::bake_cube(
        &og_render::bake::CubeFaces {
            size: n,
            rgb: &decoded,
        },
        &og_render::bake::BakeDesc {
            size: 32,
            intensity: 1.5,
            rotation_degrees: 30.0,
            samples: 64,
            sun: false,
        },
    )
    .unwrap();
    assert_eq!(
        half, baked,
        "the RGBA16F cube's map is bake_cube of its texels"
    );
    let report_line = |what: &str, (mean, worst, irr): (f32, f32, f32)| {
        println!(
            "{what}: mean {:.3} %, worst {:.2} %, irradiance {:.3} %",
            mean * 100.0,
            worst * 100.0,
            irr * 100.0
        );
        (mean, worst, irr)
    };
    let (mean, worst, irr) = report_line("equirect vs RGBA16F", compare(&studio, &half));
    assert!(
        mean < 0.005 && worst < 0.03 && irr < 0.005,
        "{mean} {worst} {irr}"
    );
    let (mean, worst, irr) =
        report_line("RGB9E5 vs RGBA16F", compare(&half, &load("env/shared.ogt")));
    assert!(
        mean < 0.001 && worst < 0.02 && irr < 0.001,
        "{mean} {worst} {irr}"
    );
    let (mean, worst, irr) = report_line("BC6H vs RGBA16F", compare(&half, &load("env/bc6h.ogt")));
    // The test's encoder is crude (one region, endpoints at the block's
    // extremes): a block across a panel's edge loses most.
    assert!(
        mean < 0.015 && worst < 0.08 && irr < 0.03,
        "{mean} {worst} {irr}"
    );
    let (mean, worst, irr) = report_line(
        "six .hdr faces vs RGBA16F",
        compare(&half, &load("env/faces.ogt")),
    );
    assert!(
        mean < 0.01 && worst < 0.05 && irr < 0.01,
        "{mean} {worst} {irr}"
    );

    // One face edited: only the environment of the faces cooks again.
    let brighter: Vec<f32> = face(2).iter().map(|v| v * 1.1).collect();
    std::fs::write(root.join("assets/env/faces/py.hdr"), hdr_face(&brighter, n)).unwrap();
    let again = cook(&root);
    let cooked: Vec<&str> = again
        .cooked_outputs()
        .iter()
        .filter(|o| o.cooked)
        .map(|o| o.id.as_str())
        .collect();
    assert_eq!(cooked, vec!["env/faces.ogt"]);
}

/// A KTX2 cube whose own mip chain is already prefiltered, with
/// `prefiltered true`, is taken as it is: the bake's own five RGB9E5 levels
/// of the studio written as a chain cook back to the same map texel for
/// texel (filtering them again would blur every level past the first), with
/// the irradiance projected from level 0; a source that cannot be a chain
/// is refused by what is wrong.
#[test]
fn a_prefiltered_ktx2_chain_is_taken_as_it_is() {
    let hdr = og_tests::scenes::asset("env/studio.hdr")
        .unwrap()
        .into_owned();
    let img = image::load_from_memory(&hdr).unwrap().to_rgb32f();
    let desc = og_render::bake::BakeDesc {
        size: 32,
        intensity: 1.0,
        rotation_degrees: 0.0,
        samples: 64,
        sun: false,
    };
    let baked = og_render::bake::bake(
        &og_render::bake::Equirect {
            width: img.width(),
            height: img.height(),
            rgb: img.as_raw(),
        },
        &desc,
    )
    .unwrap();
    let mut levels = Vec::new();
    let mut at = 0;
    for m in 0..og_render::ENVIRONMENT_MAP_LEVELS {
        let count = 6 * ((32u32 >> m) as usize).pow(2);
        levels.push(
            baked.texels[at..at + count]
                .iter()
                .flat_map(|t| t.to_le_bytes())
                .collect::<Vec<u8>>(),
        );
        at += count;
    }
    let chain = ktx2_chain(123, 32, &levels, SS_ZSTD, Some("rd"));
    let head = "type environment\nversion 1\n---\nsource \"chain.ktx2\"\n";
    let root = project(
        "prefiltered",
        &[
            ("env/chain.ktx2", &chain),
            (
                "env/chain.ogt",
                format!("{head}prefiltered true\n").as_bytes(),
            ),
            ("env/again.ktx2", &chain),
            (
                "env/again.ogt",
                "type environment\nversion 1\n---\nsource \"again.ktx2\"\nsize 32\nsamples 64\n"
                    .as_bytes(),
            ),
        ],
    );
    let report = cook(&root);
    let notes: Vec<String> = report.assets.iter().flat_map(|a| a.notes.clone()).collect();
    assert!(
        notes
            .iter()
            .any(|n| n.contains("its 5 prefiltered levels taken as they are")),
        "{notes:?}"
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let taken = bundle.load_environment("env/chain.ogt").unwrap();
    assert_eq!(taken.texels, baked.texels, "the chain is the map's levels");
    for n in [
        glam::Vec3::Y,
        glam::Vec3::X,
        -glam::Vec3::Z,
        glam::Vec3::NEG_Y,
    ] {
        let (a, b) = (
            og_render::bake::irradiance(&taken, n),
            og_render::bake::irradiance(&baked, n),
        );
        assert!(
            (a - b).abs().max_element() <= 0.005 * b.max_element(),
            "irradiance {a} {b}"
        );
    }
    // Without prefiltered true the cook resamples level 0 and filters it
    // again: the rougher levels are the radiance filtered twice over.
    let again = bundle.load_environment("env/again.ogt").unwrap();
    let level0 = 6 * 32 * 32;
    assert_ne!(again.texels[level0..], baked.texels[level0..]);

    let cook_err = |name: &str, text: &str, files: &[(&str, &[u8])]| -> String {
        let mut all: Vec<(&str, &[u8])> = vec![("e/x.ogt", text.as_bytes())];
        all.extend_from_slice(files);
        let root = project(name, &all);
        let report = cook_project(&root, &CookOptions::default()).expect("cook runs");
        report.errors().join("\n")
    };
    let head = "type environment\nversion 1\n---\n";
    let one = ktx2_chain(123, 32, &levels[..1], SS_NONE, None);
    let e = cook_err(
        "one-level",
        &format!("{head}source \"c.ktx2\"\nprefiltered true\n"),
        &[("e/c.ktx2", &one)],
    );
    assert!(
        e.contains("has one level") && e.contains("mip chain"),
        "{e}"
    );
    let e = cook_err(
        "prefiltered-size",
        &format!("{head}source \"c.ktx2\"\nprefiltered true\nsize 64\n"),
        &[("e/c.ktx2", &chain)],
    );
    assert!(e.contains("size 64") && e.contains("32²"), "{e}");
    let e = cook_err(
        "prefiltered-samples",
        &format!("{head}source \"c.ktx2\"\nprefiltered true\nsamples 64\n"),
        &[("e/c.ktx2", &chain)],
    );
    assert!(e.contains("samples is the cook's prefilter"), "{e}");
    let e = cook_err(
        "prefiltered-hdr",
        &format!("{head}source \"studio.hdr\"\nprefiltered true\n"),
        &[("e/studio.hdr", &hdr)],
    );
    assert!(e.contains("prefiltered true takes a KTX2 cube map"), "{e}");
    let e = cook_err(
        "prefiltered-word",
        &format!("{head}source \"c.ktx2\"\nprefiltered yes\n"),
        &[("e/c.ktx2", &chain)],
    );
    assert!(e.contains("'prefiltered' takes true or false"), "{e}");
}

/// `sun extract` takes the image's sun out of the map and stores it in the
/// blob: the cooked map is og-render's bake with `sun`, light and texels,
/// the report says what it found, and the same image without the key keeps
/// its sun. A prefiltered chain, already filtered with its sun in it, and a
/// word that is neither are refused.
#[test]
fn sun_extract_stores_the_images_sun_beside_the_map() {
    let (w, h) = (512u32, 256u32);
    let mut px = vec![image::Rgb([0.8f32, 0.9, 1.0]); (w * h) as usize];
    for p in &mut px[(w * h / 2) as usize..] {
        *p = image::Rgb([0.2, 0.2, 0.2]);
    }
    for (x, y) in [(300, 60), (301, 60), (300, 61), (301, 61)] {
        px[(y * w + x) as usize] = image::Rgb([30000.0, 27000.0, 21000.0]);
    }
    let mut hdr = Vec::new();
    image::codecs::hdr::HdrEncoder::new(&mut hdr)
        .encode(&px, w as usize, h as usize)
        .unwrap();
    let head = "type environment\nversion 1\n---\n";
    let settings = "source \"day.hdr\"\nsize 32\nsamples 16\n";
    let root = project(
        "sun",
        &[
            ("env/day.hdr", &hdr),
            (
                "env/sun.ogt",
                format!("{head}{settings}sun extract\n").as_bytes(),
            ),
            ("env/whole.ogt", format!("{head}{settings}").as_bytes()),
        ],
    );
    let report = cook(&root);
    let notes: Vec<String> = report.assets.iter().flat_map(|a| a.notes.clone()).collect();
    assert!(
        notes.iter().any(|n| n.contains("sun extract: intensity")),
        "{notes:?}"
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (sun, whole) = (
        bundle.load_environment("env/sun.ogt").unwrap(),
        bundle.load_environment("env/whole.ogt").unwrap(),
    );
    let img = image::load_from_memory(&hdr).unwrap().to_rgb32f();
    let bake = |sun: bool| {
        og_render::bake::bake(
            &og_render::bake::Equirect {
                width: w,
                height: h,
                rgb: img.as_raw(),
            },
            &og_render::bake::BakeDesc {
                size: 32,
                intensity: 1.0,
                rotation_degrees: 0.0,
                samples: 16,
                sun,
            },
        )
        .unwrap()
    };
    assert_eq!(sun, bake(true), "the cooked map is the bake with its sun");
    assert_eq!(whole, bake(false));
    assert_eq!(whole.sun, og_render::EnvironmentSun::default());
    // Above the horizon, and the light an upward face lost (nine
    // harmonics hold a sun's cosine to a few percent).
    assert!(
        sun.sun.intensity > 1.0 && sun.sun.direction[1] > 0.5,
        "{:?}",
        sun.sun
    );
    assert_eq!(sun.sun.color[0], 1.0);
    let up = |m: &og_render::EnvironmentMapData| og_render::bake::irradiance(m, glam::Vec3::Y);
    let lost = (up(&whole).x - up(&sun).x) * std::f32::consts::PI;
    let gave = sun.sun.intensity * sun.sun.direction[1];
    assert!(
        (lost / gave - 1.0).abs() < 0.1,
        "lost {lost}, the sun gave {gave}"
    );

    let cook_err = |name: &str, text: &str, files: &[(&str, &[u8])]| -> String {
        let mut all: Vec<(&str, &[u8])> = vec![("e/x.ogt", text.as_bytes())];
        all.extend_from_slice(files);
        let root = project(name, &all);
        let report = cook_project(&root, &CookOptions::default()).expect("cook runs");
        report.errors().join("\n")
    };
    let e = cook_err(
        "sun-word",
        &format!("{head}source \"day.hdr\"\nsun maybe\n"),
        &[("e/day.hdr", &hdr)],
    );
    assert!(
        e.contains("'sun' takes extract") && e.contains("keep"),
        "{e}"
    );
    let e = cook_err(
        "sun-prefiltered",
        &format!("{head}source \"c.ktx2\"\nprefiltered true\nsun extract\n"),
        &[("e/c.ktx2", b"")],
    );
    assert!(e.contains("sun extract takes the sun out before"), "{e}");
}

fn ldr_image(
    w: u32,
    h: u32,
    pixel: impl Fn(u32, u32) -> [u8; 3],
    format: image::ImageFormat,
) -> Vec<u8> {
    let img = image::RgbImage::from_fn(w, h, |x, y| image::Rgb(pixel(x, y)));
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut out, format)
        .unwrap();
    out.into_inner()
}

/// A grey sRGB source must light as 0.216 linear, not 0.502. Both source
/// layouts and all three LDR codecs reach the public bundle loader.
#[test]
fn ldr_panorama_and_faces_light_in_linear_space() {
    for (ext, format) in [
        ("png", image::ImageFormat::Png),
        ("jpg", image::ImageFormat::Jpeg),
        ("webp", image::ImageFormat::WebP),
    ] {
        let pano = ldr_image(65, 32, |_, _| [128, 128, 128], format);
        let face = ldr_image(16, 16, |_, _| [128, 128, 128], format);
        let env = format!(
            "type environment\nversion 1\n---\nsource \"sky.{ext}\"\nsize 16\nsamples 16\nexpand 0\n"
        );
        let cube = format!(
            "type environment\nversion 1\n---\nfaces \"f.{ext}\" \"f.{ext}\" \"f.{ext}\" \"f.{ext}\" \"f.{ext}\" \"f.{ext}\"\nsize 16\nsamples 16\nexpand 0\n"
        );
        let root = project(
            &format!("ldr-{ext}"),
            &[
                ("sky.ogt", env.as_bytes()),
                ("cube.ogt", cube.as_bytes()),
                (&format!("sky.{ext}"), &pano),
                (&format!("f.{ext}"), &face),
            ],
        );
        cook(&root);
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        for name in ["sky.ogt", "cube.ogt"] {
            let map = bundle.load_environment(name).unwrap();
            let light = og_render::bake::irradiance(&map, glam::Vec3::Y);
            assert!((light.x - 0.21586).abs() < 0.004, "{ext} {name}: {light}");
            assert_eq!(
                map.texels, map.background,
                "no expansion: light and authored colours agree"
            );
        }
    }
    // Asymmetric faces catch a projection that duplicates, swaps or flips
    // axes while a constant-grey cube still looks right.
    let colors = [
        [128, 0, 0],
        [0, 128, 0],
        [0, 0, 128],
        [128, 128, 0],
        [128, 0, 128],
        [0, 128, 128],
    ];
    let images: Vec<_> = colors
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            (
                format!("{i}.png"),
                ldr_image(16, 16, |_, _| c, image::ImageFormat::Png),
            )
        })
        .collect();
    let mut files: Vec<(&str, &[u8])> = images
        .iter()
        .map(|(n, b)| (n.as_str(), b.as_slice()))
        .collect();
    files.push(("cube.ogt", b"type environment\nversion 1\n---\nfaces \"0.png\" \"1.png\" \"2.png\" \"3.png\" \"4.png\" \"5.png\"\nsize 16\nsamples 16\nexpand 0\n"));
    let root = project("ldr-oriented-faces", &files);
    cook(&root);
    let map = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_environment("cube.ogt")
        .unwrap();
    for (face, color) in colors.iter().enumerate() {
        let got = og_render::rgb9e5_decode(map.background[face * 256 + 8 * 16 + 8]);
        for c in 0..3 {
            let expected = if color[c] == 0 { 0.0 } else { 0.21586 };
            assert!((got[c] - expected).abs() < 0.004, "face {face}: {got:?}");
        }
    }
}

/// Source detail and ordinary minification survive a small lighting bake.
#[test]
fn authored_background_has_its_own_resolution_and_box_mips() {
    for (width, size, count) in [
        (95, 16usize, 2046),
        (97, 32, 8190),
        (600, 128, 131070),
        (800, 256, 524286),
    ] {
        let sky = ldr_image(
            width,
            width / 2,
            |x, y| [if (x / 8 + y / 8) % 2 == 0 { 240 } else { 16 }, 70, 40],
            image::ImageFormat::Png,
        );
        let env = |size| {
            format!(
                "type environment\nversion 1\n---\nsource \"sky.png\"\nsun keep\nsize {size}\nsamples 16\n"
            )
        };
        let root = project(
            &format!("ldr-detail-{width}"),
            &[
                ("sky.png", &sky),
                ("low.ogt", env(16).as_bytes()),
                ("high.ogt", env(32).as_bytes()),
            ],
        );
        cook(&root);
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let low = bundle.load_environment("low.ogt").unwrap();
        let high = bundle.load_environment("high.ogt").unwrap();
        assert_eq!((low.size, high.size), (16, 32));
        assert_eq!(
            low.background_size, size as u32,
            "round to nearest, not up or down unconditionally"
        );
        assert_eq!(
            low.background.len(),
            count,
            "complete box chain through 1×1"
        );
        assert_eq!(
            low.background, high.background,
            "lighting size cannot soften the background"
        );
        let rgb = |i| og_render::rgb9e5_decode(low.background[i]);
        let base = &low.background[..6 * size * size];
        assert!(base.iter().any(|&p| og_render::rgb9e5_decode(p)[0] > 0.85));
        assert!(base.iter().any(|&p| og_render::rgb9e5_decode(p)[0] < 0.01));
        // A mip texel averages its four parents in linear light, not a
        // GGX lobe over unrelated directions. Asymmetric samples cross
        // the source's fine checker at different places on every face.
        for face in 0..6 {
            for (x, y) in [(3, 5), (size / 4 - 3, size / 4 + 1)] {
                let parent = |dx, dy| rgb((face * size + 2 * y + dy) * size + 2 * x + dx);
                let got = rgb(6 * size * size + (face * (size / 2) + y) * (size / 2) + x);
                for c in 0..3 {
                    let expected =
                        (parent(0, 0)[c] + parent(1, 0)[c] + parent(0, 1)[c] + parent(1, 1)[c])
                            * 0.25;
                    assert!(
                        (got[c] - expected).abs() < 0.004,
                        "face {face} ({x},{y}) {got:?} vs {expected}"
                    );
                }
            }
        }
    }
}

/// Ground fill is independent of a sky's blue zenith. A sky-only source
/// has no measured lower-half cues, including in incremental reports.
#[test]
fn upper_hemisphere_fills_ground() {
    let sky = ldr_image(
        64,
        16,
        |x, y| {
            if y < 12 && x == 0 {
                [0, 0, 0]
            } else if y < 12 && x == 63 {
                [255, 255, 255]
            } else if y < 12 {
                [40, 80, 160]
            } else {
                [128, 100, 60]
            }
        },
        image::ImageFormat::Png,
    );
    let env = |lines: &str| {
        format!(
            "type environment\nversion 1\n---\nsize 16\nsamples 16\nsun keep\nexpand 0\n{lines}\n"
        )
    };
    let root = project(
        "ldr-upper",
        &[
            ("sky.png", &sky),
            (
                "upper.ogt",
                env("source \"sky.png\"\nhemisphere upper\nground 0.1 0.2 0.03").as_bytes(),
            ),
            (
                "horizon.ogt",
                env("source \"sky.png\"\nhemisphere upper").as_bytes(),
            ),
        ],
    );
    let report = cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let down = |name: &str| {
        let map = bundle.load_environment(name).unwrap();
        og_render::rgb9e5_decode(map.background[3 * 16 * 16 + 8 * 16 + 8])
    };
    let ground = down("upper.ogt");
    assert!(
        (ground[0] - 0.1).abs() < 0.001
            && (ground[1] - 0.2).abs() < 0.001
            && (ground[2] - 0.03).abs() < 0.001,
        "{ground:?}"
    );
    let ground = down("horizon.ogt");
    assert!(
        (ground[0] - 0.21586).abs() < 0.004 && ground[0] > ground[2] * 3.0,
        "horizon, not zenith: {ground:?}"
    );
    let incremental = cook(&root);
    for report in [&report, &incremental] {
        let stats = og_cook::cook::stats::cook_report_json(
            report,
            &root.join("cooked"),
            &Default::default(),
        );
        for name in ["upper.ogt", "horizon.ogt"] {
            let asset = stats["assets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["source"] == name)
                .unwrap();
            let facts = &asset["environment"]["facts"];
            assert!(facts.get("lowerMirrorMean").unwrap().is_null());
            assert!(facts.get("lowerBlueFraction").unwrap().is_null());
            assert!(
                facts["lowerHemisphere"]
                    .as_str()
                    .unwrap()
                    .contains("no lower hemisphere")
            );
            assert_eq!(
                facts["clippedFraction"],
                12.0 / 1024.0,
                "measure the original source, not the filled sphere"
            );
            assert_eq!(facts["wrapMean"], 0.75);
        }
    }
}

/// The horizon must retain azimuthal colour, transition rather than step,
/// and reach the explicit ground colour, not an average of the columns.
#[test]
fn upper_horizon_eases_each_column_to_ground() {
    let sky = ldr_image(
        1024,
        256,
        |x, _| {
            if x < 512 {
                [64, 128, 192]
            } else {
                [192, 64, 32]
            }
        },
        image::ImageFormat::Png,
    );
    let root = project(
        "ldr-upper-transition",
        &[
            ("sky.png", &sky),
            (
                "sky.ogt",
                b"type environment\nversion 1\n---\nsize 16\nsamples 16\nsun keep\nexpand 0\nsource \"sky.png\"\nhemisphere upper\nground 0.1 0.2 0.03\n",
            ),
        ],
    );
    cook(&root);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let map = bundle.load_environment("sky.ogt").unwrap();
    assert_eq!(map.background_size, 256);
    let ground = [0.1, 0.2, 0.03];
    // +Z and -Z face centres see the two different horizon colours.
    // Rows 128, 134 and 148 look about 0.2°, 2.9° and 9.1° below it.
    for (face, horizon) in [
        (4, [0.52712, 0.05127, 0.01444]),
        (5, [0.05127, 0.21586, 0.52712]),
    ] {
        let pixel =
            |y: usize| og_render::rgb9e5_decode(map.background[face * 256 * 256 + y * 256 + 128]);
        let (near, middle, far) = (pixel(128), pixel(134), pixel(148));
        for c in 0..3 {
            assert!((near[c] - horizon[c]).abs() < 0.015, "horizon: {near:?}");
            let midpoint = (horizon[c] + ground[c]) * 0.5;
            let tolerance = (horizon[c] - ground[c]).abs() * 0.2 + 0.002;
            assert!((middle[c] - midpoint).abs() < tolerance, "ease: {middle:?}");
            assert!((far[c] - ground[c]).abs() < 0.002, "ground: {far:?}");
        }
    }
}

#[test]
fn ldr_sun_is_removed_from_lighting_not_the_authored_background() {
    let sky = ldr_image(
        512,
        256,
        |x, y| {
            let r = (x as f32 - 300.0).hypot(y as f32 - 70.0);
            if r < 2.5 {
                [255, 255, 255]
            } else {
                let glow = (40.0 * (-r * r / 200.0).exp()) as u8;
                [100 + glow, 130 + glow, 170 + glow]
            }
        },
        image::ImageFormat::Png,
    );
    let env = |sun: &str| {
        format!(
            "type environment\nversion 1\n---\nsource \"sky.png\"\nsize 32\nsamples 16\nexpand 0\n{sun}\n"
        )
    };
    let root = project(
        "ldr-sun",
        &[
            ("sky.png", &sky),
            ("sun.ogt", env("").as_bytes()),
            ("keep.ogt", env("sun keep").as_bytes()),
            ("boost.ogt", env("sunBoost 3\nintensity 2").as_bytes()),
        ],
    );
    let report = cook(&root);
    assert!(
        report
            .assets
            .iter()
            .flat_map(|a| &a.notes)
            .any(|n| n.contains("clippedFraction"))
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let sun = bundle.load_environment("sun.ogt").unwrap();
    let keep = bundle.load_environment("keep.ogt").unwrap();
    let boost = bundle.load_environment("boost.ogt").unwrap();
    assert!(
        (boost.sun.intensity / sun.sun.intensity - 6.0).abs() < 0.001,
        "both sunBoost and cooked intensity must scale the directional light"
    );
    assert!(
        sun.sun.intensity > 0.0 && sun.sun.direction[1] > 0.5 && sun.sun.angular_size > 0.0,
        "{:?}",
        sun.sun
    );
    assert_eq!(
        sun.background, keep.background,
        "the sun stays visible in the authored sky"
    );
    let peak = |m: &og_render::EnvironmentMapData| {
        m.texels[..6 * 32 * 32]
            .iter()
            .map(|p| og_render::rgb9e5_decode(*p)[0])
            .fold(0.0, f32::max)
    };
    assert!(
        peak(&sun) < peak(&keep) - 0.05,
        "the prefilter must not retain the disc"
    );
    let incremental = cook(&root);
    assert!(!incremental.cooked_outputs().iter().any(|o| o.cooked));
    let stats = og_cook::cook::stats::cook_report_json(
        &incremental,
        &root.join("cooked"),
        &Default::default(),
    );
    let entry = stats["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["source"] == "sun.ogt")
        .unwrap();
    assert_eq!(entry["environment"]["facts"]["width"], 512);
    assert!(entry["environment"]["facts"]["lowerMirrorMean"].is_number());
    assert!(entry["environment"]["facts"]["lowerBlueFraction"].is_number());
    assert!(
        entry["environment"]["facts"]["clippedFraction"]
            .as_f64()
            .unwrap()
            > 0.0
    );
    assert_eq!(
        entry["environment"]["sun"]["intensity"],
        serde_json::json!(sun.sun.intensity)
    );
}

#[test]
fn near_equirect_resamples_but_a_distant_aspect_names_its_size() {
    let near = ldr_image(65, 32, |_, _| [128, 128, 128], image::ImageFormat::Png);
    let far = ldr_image(65, 40, |_, _| [128, 128, 128], image::ImageFormat::Png);
    let root = project(
        "ldr-aspect",
        &[
            ("near.png", &near),
            ("far.png", &far),
            (
                "near.ogt",
                b"type environment\nversion 1\n---\nsource \"near.png\"\nsize 16\nsamples 16\n",
            ),
            (
                "far.ogt",
                b"type environment\nversion 1\n---\nsource \"far.png\"\nsize 16\nsamples 16\n",
            ),
        ],
    );
    let report = cook_project(&root, &Default::default()).unwrap();
    let error = report
        .assets
        .iter()
        .find(|a| a.source == "far.ogt")
        .unwrap()
        .error
        .as_ref()
        .unwrap();
    assert!(
        error.contains("65 × 40") && error.contains("2:1"),
        "{error}"
    );
    assert!(
        Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_environment("near.ogt")
            .is_ok()
    );
}

/// A space sky's clipped stars tie with its sun disc pixel for pixel; the
/// sun is the white region holding the most light, so a one-pixel star near
/// the horizon (the last clipped pixel in the image's order) is not taken
/// for it.
#[test]
fn an_ldr_space_sky_takes_its_disc_not_a_clipped_star() {
    let stars = [(40, 20), (210, 95), (470, 60), (120, 124), (400, 126)];
    let png = ldr_image(
        512,
        256,
        move |x, y| {
            let (dx, dy) = (x as f32 - 300.0, y as f32 - 70.0);
            if dx.hypot(dy) < 4.0 || stars.contains(&(x, y)) {
                [255, 255, 255]
            } else {
                [8, 10, 24]
            }
        },
        image::ImageFormat::Png,
    );
    let root = project(
        "ldr-space-sun",
        &[
            ("space.png", &png),
            (
                "space.ogt",
                b"type environment\nversion 1\n---\nsource \"space.png\"\nsize 32\nsamples 16\nexpand 0\n",
            ),
        ],
    );
    let report = cook(&root);
    let stats =
        og_cook::cook::stats::cook_report_json(&report, &root.join("cooked"), &Default::default());
    let sun = stats["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["source"] == "space.ogt")
        .unwrap()["environment"]["sun"]
        .clone();
    // The disc's centre row 70 of 256 is 90 - 70 / 256 * 180 = 40.8 degrees up.
    let up = sun["direction"][1].as_f64().unwrap();
    assert!(
        (up.asin().to_degrees() - 40.8).abs() < 1.5,
        "the sun is the disc 40.8 degrees up: {sun}"
    );
}

/// When `sun extract` finds no sun in an LDR sky, the cook's message and
/// report name why: nothing bright enough, white clouds merged with the
/// disc (a clipped area), or a disc drawn too large. A small white disc in
/// the same sky is measured.
#[test]
fn an_ldr_sky_without_a_measurable_sun_names_the_cause() {
    let sky = |paint: fn(f32, f32) -> Option<[u8; 3]>| {
        ldr_image(
            512,
            256,
            move |x, y| {
                paint(x as f32 - 300.0, y as f32 - 70.0).unwrap_or({
                    let glow = (y as f32 * 0.2) as u8;
                    [90 + glow, 120 + glow, 170]
                })
            },
            image::ImageFormat::Png,
        )
    };
    let small = sky(|dx, dy| (dx.hypot(dy) < 2.5).then_some([255, 255, 255]));
    let dim = sky(|dx, dy| (dx.hypot(dy) < 2.5).then_some([200, 200, 200]));
    let large = sky(|dx, dy| (dx.hypot(dy) < 20.0).then_some([255, 255, 255]));
    // The disc, with white cumulus lumps running off it to the right.
    let clouds = sky(|dx, dy| {
        let lump = |cx: f32, cy: f32, r: f32| (dx - cx).hypot(dy - cy) < r;
        (dx.hypot(dy) < 2.5
            || lump(4.0, 1.0, 4.0)
            || lump(14.0, 6.0, 7.0)
            || lump(28.0, 2.0, 9.0)
            || lump(44.0, 10.0, 8.0)
            || lump(60.0, 4.0, 10.0))
        .then_some([255, 255, 255])
    });
    let env = |source: &str| {
        format!(
            "type environment\nversion 1\n---\nsource \"{source}\"\nsize 32\nsamples 16\nexpand 0\n"
        )
    };
    let root = project(
        "ldr-no-sun",
        &[
            ("small.png", &small),
            ("dim.png", &dim),
            ("large.png", &large),
            ("clouds.png", &clouds),
            ("small.ogt", env("small.png").as_bytes()),
            ("dim.ogt", env("dim.png").as_bytes()),
            ("large.ogt", env("large.png").as_bytes()),
            ("clouds.ogt", env("clouds.png").as_bytes()),
        ],
    );
    let report = cook(&root);
    let notes = |source: &str| {
        report
            .assets
            .iter()
            .find(|a| a.source == source)
            .unwrap()
            .notes
            .join("\n")
    };
    assert!(
        notes("small.ogt").contains("sun extract: intensity"),
        "{}",
        notes("small.ogt")
    );
    for (source, cause) in [
        ("dim.ogt", "none bright enough"),
        ("large.ogt", "a disc too large"),
        ("clouds.ogt", "clipped area"),
    ] {
        let n = notes(source);
        assert!(
            n.contains("no measurable sun was found (") && n.contains(cause),
            "{source}: {n}"
        );
    }
    let stats =
        og_cook::cook::stats::cook_report_json(&report, &root.join("cooked"), &Default::default());
    let sun = |source: &str| {
        stats["assets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["source"] == source)
            .unwrap()["environment"]["sun"]
            .clone()
    };
    assert!(sun("small.ogt")["missing"].is_null());
    assert!(
        sun("large.ogt")["missing"]
            .as_str()
            .is_some_and(|m| m.starts_with("a disc too large")),
        "{}",
        sun("large.ogt")
    );
}

/// A space sky whose sun is drawn cream, below white, holds white only in
/// its star dots. A star is no sun: one smaller than a disc 1° across is
/// never measured, and a larger white speck that a cream disc outshines is
/// a star beside a sun drawn too dim. Both skies say why, with the cream
/// disc's colour, rather than putting the sun where a star is.
#[test]
fn an_ldr_sky_with_a_cream_disc_does_not_take_a_star_for_its_sun() {
    let sky = |stars: &'static [(u32, u32)]| {
        ldr_image(
            512,
            256,
            move |x, y| {
                let (dx, dy) = (x as f32 - 300.0, y as f32 - 70.0);
                if dx.hypot(dy) < 6.0 {
                    [250, 235, 170]
                } else if stars.contains(&(x, y)) {
                    [255, 255, 255]
                } else {
                    [8, 10, 24]
                }
            },
            image::ImageFormat::Png,
        )
    };
    let dots = sky(&[(40, 20), (210, 95), (470, 60), (120, 30)]);
    let speck = sky(&[(40, 20), (41, 20), (40, 21), (41, 21), (470, 60)]);
    let env = |source: &str| {
        format!(
            "type environment\nversion 1\n---\nsource \"{source}\"\nsize 32\nsamples 16\nexpand 0\n"
        )
    };
    let root = project(
        "ldr-cream-sun",
        &[
            ("dots.png", &dots),
            ("speck.png", &speck),
            ("dots.ogt", env("dots.png").as_bytes()),
            ("speck.ogt", env("speck.png").as_bytes()),
        ],
    );
    let report = cook(&root);
    let stats =
        og_cook::cook::stats::cook_report_json(&report, &root.join("cooked"), &Default::default());
    for source in ["dots.ogt", "speck.ogt"] {
        let asset = stats["assets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["source"] == source)
            .unwrap()
            .clone();
        assert!(
            asset["environment"]["sun"]["missing"].is_string(),
            "{source}: a star is not the sun: {asset}"
        );
        let notes = report
            .assets
            .iter()
            .find(|a| a.source == source)
            .unwrap()
            .notes
            .join("\n");
        assert!(
            notes.contains("none bright enough")
                && notes.contains("disc above the horizon is sRGB 0.98"),
            "{source}: the note names the cream disc: {notes}"
        );
    }
}
