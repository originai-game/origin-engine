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

const STUDIO: &str = "type environment\nversion 1\n---\nsource \"studio.hdr\"\nsize 32\nintensity 1.5\nrotation 30\nsamples 64\n";

fn cook(root: &Path) -> og_cook::Report {
    let report = cook_project(root, &CookOptions::default()).expect("cook");
    assert!(report.ok(), "{:?}", report.errors());
    report
}

#[test]
fn an_environment_cooks_into_the_baked_map_and_loads_back() {
    let hdr = og_tests::scenes::asset("env/studio.hdr")
        .unwrap()
        .into_owned();
    let root = project(
        "studio",
        &[
            ("env/studio.ogt", STUDIO.as_bytes()),
            ("env/studio.hdr", &hdr),
        ],
    );
    let report = cook(&root);
    assert!(
        !report.ignored.iter().any(|f| f.ends_with(".hdr")),
        "the image is the environment's: {:?}",
        report.ignored
    );
    let outputs: Vec<String> = report
        .cooked_outputs()
        .iter()
        .map(|o| o.id.clone())
        .collect();
    assert_eq!(outputs, vec!["env/studio.ogt".to_string()]);
    let notes: Vec<String> = report.assets.iter().flat_map(|a| a.notes.clone()).collect();
    assert!(
        notes.iter().any(|n| n.contains("32² cube of 5 levels")),
        "{notes:?}"
    );

    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let map = bundle.load_environment("env/studio.ogt").unwrap();
    let img = image::load_from_memory(&hdr).unwrap().to_rgb32f();
    let baked = og_render::bake::bake(
        &og_render::bake::Equirect {
            width: img.width(),
            height: img.height(),
            rgb: img.as_raw(),
        },
        &og_render::bake::BakeDesc {
            size: 32,
            intensity: 1.5,
            rotation_degrees: 30.0,
            samples: 64,
        },
    )
    .unwrap();
    assert_eq!(
        map, baked,
        "the cooked map is og-render's bake of the image"
    );
    assert_eq!(
        map.texels.len(),
        og_render::EnvironmentMapData::texel_count(32)
    );

    // Nothing changed: nothing cooked; the image edited: the map again.
    let again = cook(&root);
    assert!(!again.cooked_outputs().iter().any(|o| o.cooked));
    let mut edited = hdr.clone();
    let last = edited.len() - 2;
    edited[last] = edited[last].wrapping_add(7);
    std::fs::write(root.join("assets/env/studio.hdr"), &edited).unwrap();
    let third = cook(&root);
    let cooked: Vec<&str> = third
        .cooked_outputs()
        .iter()
        .filter(|o| o.cooked)
        .map(|o| o.id.as_str())
        .collect();
    assert_eq!(cooked, vec!["env/studio.ogt"]);
}

#[test]
fn refusals_name_the_fix() {
    let hdr = og_tests::scenes::asset("env/studio.hdr")
        .unwrap()
        .into_owned();
    let cook_err = |name: &str, text: &str| -> String {
        let root = project(
            name,
            &[("e/x.ogt", text.as_bytes()), ("e/studio.hdr", &hdr)],
        );
        let report = cook_project(&root, &CookOptions::default()).expect("cook runs");
        report.errors().join("\n")
    };
    let head = "type environment\nversion 1\n---\n";
    let e = cook_err("none", head);
    assert!(
        e.contains("source \\\"<file>.hdr\\\"") || e.contains("source \"<file>.hdr\""),
        "{e}"
    );
    let e = cook_err("png", &format!("{head}source \"studio.png\"\n"));
    assert!(e.contains(".hdr or OpenEXR .exr"), "{e}");
    let e = cook_err("size", &format!("{head}source \"studio.hdr\"\nsize 100\n"));
    assert!(e.contains("a power of two from 16 to 2048"), "{e}");
    let e = cook_err("key", &format!("{head}source \"studio.hdr\"\nblur 2\n"));
    assert!(e.contains("'blur' is not an environment entry"), "{e}");
    let e = cook_err("missing", &format!("{head}source \"gone.hdr\"\n"));
    assert!(e.contains("gone.hdr"), "{e}");
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

#[test]
fn cube_sources_are_refused_by_what_is_wrong() {
    let n = 8u32;
    let texels = vec![0.5f32; (6 * n * n * 3) as usize];
    let half = rgba16f(&texels);
    let face = hdr_face(&texels[..(n * n * 3) as usize], n);
    let small = hdr_face(&texels[..(4 * 4 * 3) as usize], 4);
    let head = "type environment\nversion 1\n---\n";
    let cook_err = |name: &str, text: &str, files: &[(&str, &[u8])]| -> String {
        let mut all: Vec<(&str, &[u8])> = vec![("e/x.ogt", text.as_bytes())];
        all.extend_from_slice(files);
        let root = project(name, &all);
        let report = cook_project(&root, &CookOptions::default()).expect("cook runs");
        report.errors().join("\n")
    };
    let mut flat = ktx2_cube(97, n, &half, SS_NONE, None);
    flat[36..40].copy_from_slice(&1u32.to_le_bytes());
    let e = cook_err(
        "flat",
        &format!("{head}source \"c.ktx2\"\n"),
        &[("e/c.ktx2", &flat)],
    );
    assert!(e.contains("a cube map (6 faces)"), "{e}");
    let rgba8 = ktx2_cube(37, n, &half, SS_NONE, None);
    let e = cook_err(
        "rgba8",
        &format!("{head}source \"c.ktx2\"\n"),
        &[("e/c.ktx2", &rgba8)],
    );
    assert!(e.contains("vkFormat 37") && e.contains("RGBA16F"), "{e}");
    let up = ktx2_cube(97, n, &half, SS_NONE, Some("ru"));
    let e = cook_err(
        "ru",
        &format!("{head}source \"c.ktx2\"\n"),
        &[("e/c.ktx2", &up)],
    );
    assert!(e.contains("KTXorientation \"ru\""), "{e}");
    let short = ktx2_cube(97, n, &half[..half.len() / 2], SS_NONE, None);
    let e = cook_err(
        "short",
        &format!("{head}source \"c.ktx2\"\n"),
        &[("e/c.ktx2", &short)],
    );
    assert!(e.contains("six 8 x 8 RGBA16F faces take"), "{e}");
    let mut nan = half.clone();
    nan[8 * 9..8 * 9 + 2].copy_from_slice(&0x7E00u16.to_le_bytes());
    let bad = ktx2_cube(97, n, &nan, SS_NONE, None);
    let e = cook_err(
        "nan",
        &format!("{head}source \"c.ktx2\"\n"),
        &[("e/c.ktx2", &bad)],
    );
    assert!(
        e.contains("face 0 (+x) texel (1, 1) is not a finite number"),
        "{e}"
    );
    let five = "faces \"f.hdr\" \"f.hdr\" \"f.hdr\" \"f.hdr\" \"f.hdr\"\n";
    let e = cook_err("five", &format!("{head}{five}"), &[("e/f.hdr", &face)]);
    assert!(e.contains("faces takes six images"), "{e}");
    let sizes = "faces \"f.hdr\" \"f.hdr\" \"f.hdr\" \"g.hdr\" \"f.hdr\" \"f.hdr\"\n";
    let e = cook_err(
        "sizes",
        &format!("{head}{sizes}"),
        &[("e/f.hdr", &face), ("e/g.hdr", &small)],
    );
    assert!(e.contains("g.hdr is 4 × 4 and e/f.hdr is 8 × 8"), "{e}");
    let both = format!("{head}source \"c.ktx2\"\nfaces \"f.hdr\"\n");
    let e = cook_err("both", &both, &[("e/c.ktx2", &rgba8), ("e/f.hdr", &face)]);
    assert!(e.contains("a source or six faces, not both"), "{e}");
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
