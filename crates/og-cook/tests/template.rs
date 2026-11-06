//! The cook and the runtime loaders, end to end on the project template
//! (`tests/fixtures/cook-assets/assets`): cook it, load what came out, compare with the
//! sources and render it.
//!
//! Ways this could fail, and the test that catches each:
//! - output bytes depend on thread scheduling, map order, timestamps or
//!   uninitialised padding → `cooking_twice_gives_identical_bytes` (one cook
//!   on all cores, one on one thread, every file compared) and
//!   `every_thread_count_cooks_the_same_bytes` (thirty cooks of a small
//!   template on 1, 2 and 8 threads, three at a time);
//! - the incremental check re-cooks too much (stamps over-broad) or too
//!   little (stamps miss an input), or rewrites unchanged files →
//!   `a_gltf_edit_recooks_exactly_its_outputs`;
//! - a stale blob loads silently, or is refused without naming the file and
//!   both stamps → `a_stale_blob_is_refused_with_both_stamps`;
//! - quantisation, the meshopt codecs (C encoder, Rust decoder), welding or
//!   reordering lose or distort triangles → `meshes_keep_their_triangles_and_bounds`;
//! - Basis encoding, the .basis→KTX2 rebuild or the KTX2→.basis inverse
//!   corrupt texels, swap channels or flip rows → `textures_decode_close_to_their_sources`;
//! - QOA, MSDF and scene compilation produce data the loaders cannot read or
//!   that means something else → `sounds_fonts_and_scenes_load_as_authored`;
//! - the whole chain feeds og-render something that draws differently from
//!   the source models → `cooked_models_render_the_gltf_reference_scene`;
//! - the cooked template is not smaller than its sources → the size check in
//!   `cooking_twice_gives_identical_bytes`.
//!
//! Evidence (the cook report, the two hashes, the rendered frame) goes to
//! `$OG_EVIDENCE_DIR/M4/` (default `<target tmp>/evidence/M4/`).

use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::Bundle;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::support;

use support::{copy_dir, template_assets};

fn files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let e = e.unwrap();
            if e.file_type().unwrap().is_dir() {
                walk(base, &e.path(), out);
            } else {
                let rel = e
                    .path()
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, std::fs::read(e.path()).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

/// What differs between two `cooked/` trees, one line per difference: a
/// file on one side only, a blob's bytes (its size and first differing
/// offset), each differing line of the manifest, and the cook times with
/// the times masked (the same assets, whatever their milliseconds; README
/// §Cook times: the one file two cooks write differently). Empty: the
/// same bytes.
fn differences(a: &BTreeMap<String, Vec<u8>>, b: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
    let mut out = Vec::new();
    for k in a.keys().filter(|k| !b.contains_key(*k)) {
        out.push(format!("cooked/{k}: only in the first cook"));
    }
    for k in b.keys().filter(|k| !a.contains_key(*k)) {
        out.push(format!("cooked/{k}: only in the second cook"));
    }
    for (k, x) in a {
        let Some(y) = b.get(k) else { continue };
        if k == og_cook::COOK_TIMES {
            let masked = |t: &[u8]| {
                og_cook::CookTimes::parse(&String::from_utf8_lossy(t))
                    .map(|c| c.assets.into_keys().collect::<Vec<_>>())
            };
            let (mx, my) = (masked(x), masked(y));
            if mx != my {
                out.push(format!(
                    "cooked/{k}: the assets differ (times masked): {mx:?} against {my:?}"
                ));
            }
        } else if x != y {
            let at = x.iter().zip(y).position(|(p, q)| p != q);
            out.push(format!(
                "cooked/{k}: {} against {} bytes, first difference at {}",
                x.len(),
                y.len(),
                at.map_or("the end of the shorter".to_string(), |i| format!(
                    "byte {i}"
                ))
            ));
            if k == og_cook::runtime::MANIFEST {
                let (tx, ty) = (String::from_utf8_lossy(x), String::from_utf8_lossy(y));
                let (lx, ly): (Vec<&str>, Vec<&str>) = (tx.lines().collect(), ty.lines().collect());
                for i in 0..lx.len().max(ly.len()) {
                    let (p, q) = (lx.get(i), ly.get(i));
                    if p != q {
                        out.push(format!("  line {}: {p:?} against {q:?}", i + 1));
                    }
                }
            }
        }
    }
    out
}

/// The template cooked once (all cores) and shared by the tests, across
/// their processes (`support::shared`).
fn cooked() -> &'static support::SharedCook {
    static COOKED: OnceLock<support::SharedCook> = OnceLock::new();
    COOKED.get_or_init(|| {
        support::shared("template", &support::template_sources(), |root| {
            support::stage_template(&root.join("assets"))
        })
    })
}

fn bundle() -> Bundle {
    Bundle::open(&cooked().root.join("cooked")).expect("the manifest loads")
}

/// The template made small enough to cook thirty times: as the template
/// ships (without the fixture's trickster.glb, whose three 1024² textures
/// are most of a cook; its scene entity is the walker), the two standalone
/// textures cut to 128², the font's charset to ASCII and the template's own
/// characters. Every kind of output and both texture encoders remain, and
/// the font still renders its glyphs on the cook's thread count.
fn small_template(root: &Path) {
    copy_dir(&template_assets(), &root.join("assets"));
    let assets = root.join("assets");
    for t in ["textures/stone.png", "textures/stone_n.png"] {
        let img = image::open(assets.join(t)).unwrap();
        img.resize_exact(128, 128, image::imageops::FilterType::Triangle)
            .save(assets.join(t))
            .unwrap();
    }
    let font = assets.join("fonts/NotoSansSC-Regular-subset.ttf.ogt");
    let text = std::fs::read_to_string(&font).unwrap();
    let text = text.replace("charset latin cjk-punct", "charset ascii");
    assert!(text.contains("charset ascii"), "{text}");
    std::fs::write(&font, text).unwrap();
}

/// Cooks of one project on one, two and eight threads, the three series
/// at once (so every cook shares the machine with two others and its
/// pool's schedule moves from run to run): every file each writes is the
/// first one-thread cook's bytes, the cook times masked (`differences`).
/// One run a thread count in the quick and functional lanes; ten under
/// `--all` (`OG_GATE_ALL`, a wave's end), whose repeats hunt a race that
/// one schedule may miss (the owner, 2026-11-06: 31 cooks were the gate's
/// slowest test).
#[test]
fn every_thread_count_cooks_the_same_bytes() {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook/threads");
    let _ = std::fs::remove_dir_all(&base);
    let source = base.join("source");
    small_template(&source);
    let cook = |dir: &Path, threads: usize| {
        let _ = std::fs::remove_dir_all(dir);
        copy_dir(&source.join("assets"), &dir.join("assets"));
        let options = CookOptions {
            threads,
            ..Default::default()
        };
        let report = cook_project(dir, &options).unwrap();
        assert!(report.ok(), "{threads} threads: {:?}", report.errors());
        assert_eq!(report.ignored, ["fonts/OFL.txt"], "{threads} threads");
        files(&dir.join("cooked"))
    };
    let reference = cook(&base.join("reference"), 1);
    // Fifteen outputs (timelines/intro.ogt with its .srt among them, and
    // the intro's music, audio/sting.ogt), the eleven of the built-in
    // storybook style its look's preset names (its style and seven sounds,
    // the two shared fonts and the icon sheet it draws with), the manifest
    // and the cook times.
    assert_eq!(reference.len(), 28, "{:?}", reference.keys());
    let runs = if std::env::var_os("OG_GATE_ALL").is_some() {
        10
    } else {
        1
    };
    let failures: Vec<String> = std::thread::scope(|s| {
        let series: Vec<_> = [1usize, 2, 8]
            .into_iter()
            .map(|threads| {
                let (base, reference, cook, runs) = (&base, &reference, &cook, runs);
                s.spawn(move || {
                    let mut out = Vec::new();
                    for run in 0..runs {
                        let dir = base.join(format!("t{threads}-{run}"));
                        let got = cook(&dir, threads);
                        for d in differences(reference, &got) {
                            out.push(format!("{threads} threads, run {run}: {d}"));
                        }
                        let _ = std::fs::remove_dir_all(&dir);
                    }
                    out
                })
            })
            .collect();
        series.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    assert!(
        failures.is_empty(),
        "cooks differ from the first one-thread cook:\n{}",
        failures.join("\n")
    );
}

fn source_doc(
    name: &str,
) -> (
    gltf::Document,
    Vec<gltf::buffer::Data>,
    Vec<gltf::image::Data>,
) {
    gltf::import(support::template_model(name)).unwrap()
}

#[test]
fn meshes_keep_their_triangles_and_bounds() {
    let b = bundle();
    for name in ["trickster.glb", "dropship.glb"] {
        let (doc, buffers, _) = source_doc(name);
        let model = b.load_model(&format!("models/{name}")).unwrap();
        let src_prims: Vec<_> = doc
            .meshes()
            .flat_map(|m| m.primitives().collect::<Vec<_>>())
            .collect();
        assert_eq!(
            model.primitives.len(),
            src_prims.len(),
            "{name}: primitive count"
        );
        let mut src_tris = 0;
        let mut cooked_tris = 0;
        for (i, (prim, cooked)) in src_prims.iter().zip(&model.primitives).enumerate() {
            let reader = prim.reader(|bf| Some(&buffers[bf.index()]));
            let pos: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
            let tris = reader
                .read_indices()
                .map_or(pos.len(), |ix| ix.into_u32().count())
                / 3;
            assert_eq!(
                cooked.triangle_count(0),
                tris,
                "{name} primitive {i}: LOD 0 triangles"
            );
            src_tris += tris;
            cooked_tris += cooked.triangle_count(0);
            // Bounds of the decoded vertices against the source positions.
            let (mut slo, mut shi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for p in &pos {
                for a in 0..3 {
                    slo[a] = slo[a].min(p[a]);
                    shi[a] = shi[a].max(p[a]);
                }
            }
            let (mut clo, mut chi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for v in &cooked.vertices {
                for a in 0..3 {
                    clo[a] = clo[a].min(v.position[a]);
                    chi[a] = chi[a].max(v.position[a]);
                }
            }
            let extent = (0..3)
                .map(|a| shi[a] - slo[a])
                .fold(0.0f32, f32::max)
                .max(1e-6);
            for a in 0..3 {
                assert!(
                    (clo[a] - slo[a]).abs() <= 0.01 * extent,
                    "{name} primitive {i}: min[{a}] {} vs {}",
                    clo[a],
                    slo[a]
                );
                assert!(
                    (chi[a] - shi[a]).abs() <= 0.01 * extent,
                    "{name} primitive {i}: max[{a}] {} vs {}",
                    chi[a],
                    shi[a]
                );
            }
            // LODs shrink and stay inside the vertex buffer.
            for w in cooked.lods.windows(2) {
                assert!(
                    w[1].count < w[0].count,
                    "{name} primitive {i}: LODs must shrink"
                );
            }
            // Normals stay unit length after octahedral quantisation.
            for v in cooked.vertices.iter().step_by(97) {
                let n = glam::Vec3::from(v.normal).length();
                assert!(
                    (n - 1.0).abs() < 1e-3,
                    "{name} primitive {i}: normal length {n}"
                );
            }
        }
        assert_eq!(src_tris, cooked_tris);
        // Model bounds (default scene, node transforms) against og-render's
        // own glTF loader, from the decoded vertices.
        let reference =
            og_render::gltf::load_glb(&std::fs::read(support::template_model(name)).unwrap())
                .unwrap();
        let world = model.node_world();
        let (mut lo, mut hi) = (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN));
        for n in model.scene_nodes(model.default_scene.unwrap()) {
            if let Some(m) = model.nodes[n].mesh {
                for at in model.node_draws(&world, n) {
                    for p in model.meshes[m].primitives.clone() {
                        for v in &model.primitives[p].vertices {
                            let w = at.transform_point3(glam::Vec3::from(v.position));
                            lo = lo.min(w);
                            hi = hi.max(w);
                        }
                    }
                }
            }
        }
        let extent = (reference.max - reference.min).max_element();
        assert!(
            (lo - reference.min).abs().max_element() <= 0.01 * extent,
            "{name}: {lo} vs {}",
            reference.min
        );
        assert!(
            (hi - reference.max).abs().max_element() <= 0.01 * extent,
            "{name}: {hi} vs {}",
            reference.max
        );
        assert!((model.min - reference.min).abs().max_element() <= 1e-4 * extent);
        assert!((model.max - reference.max).abs().max_element() <= 1e-4 * extent);
    }
}

#[test]
fn sounds_fonts_and_scenes_load_as_authored() {
    let b = bundle();
    // Sound: QOA decodes to the WAV's length and stays close to it.
    let s = b.load_sound("audio/blip.wav").unwrap();
    let pcm = s.decode().unwrap();
    let mut wav = hound::WavReader::open(template_assets().join("audio/blip.wav")).unwrap();
    let src: Vec<i16> = wav.samples::<i16>().map(Result::unwrap).collect();
    assert_eq!((pcm.channels, pcm.sample_rate), (1, 44100));
    assert_eq!(pcm.samples.len(), src.len());
    let (mut sig, mut err) = (0.0f64, 0.0f64);
    for (a, b) in src.iter().zip(&pcm.samples) {
        sig += (*a as f64).powi(2);
        err += (*a as f64 - *b as f64).powi(2);
    }
    let snr = 10.0 * (sig / err).log10();
    assert!(snr > 20.0, "QOA SNR {snr:.1} dB");

    // Font: the charset and the extra characters, with distance fields whose
    // inside is inside: the centre of '中' (its vertical stroke) is ink,
    // the atlas border around it is not.
    let f = b.load_font("fonts/NotoSansSC-Regular-subset.ttf").unwrap();
    assert!(f.glyph('A').is_some() && f.glyph('é').is_some() && f.glyph('。').is_some());
    assert!(f.glyph('开').is_some(), "chars from the import settings");
    assert!(f.glyph('龍').is_none(), "outside the charset");
    let space = f.glyph(' ').unwrap();
    assert!(space.advance > 0.1 && space.atlas_w == 0);
    let g = f.glyph('中').unwrap();
    let page = &f.pages[g.page as usize];
    let at = |x: u32, y: u32| page.rgba[((y * page.width + x) * 4) as usize..][..4].to_vec();
    let centre = at(g.atlas_x + g.atlas_w / 2, g.atlas_y + g.atlas_h / 2);
    let corner = at(g.atlas_x + 1, g.atlas_y + 1);
    let median = |p: &[u8]| {
        let mut c = [p[0], p[1], p[2]];
        c.sort();
        c[1]
    };
    assert!(
        median(&centre) > 160 && centre[3] > 160,
        "inside: {centre:?}"
    );
    assert!(
        median(&corner) < 96 && corner[3] < 96,
        "outside: {corner:?}"
    );
    assert!(f.meta.ascender > 0.5 && f.meta.descender < 0.0);

    // Kerning: the GPOS `kern` pairs between cooked characters, sorted and
    // unique, in em; Latin pairs the font tightens ("AV", "To") come out
    // negative, a pair it does not kern ("HH") is absent.
    assert!(!f.kerning.is_empty(), "the template font kerns");
    assert!(
        f.kerning
            .windows(2)
            .all(|w| (w[0].left, w[0].right) < (w[1].left, w[1].right)),
        "sorted by (left, right), no duplicate"
    );
    let kern = |a: char, b: char| {
        f.kerning
            .iter()
            .find(|k| k.left == a as u32 && k.right == b as u32)
            .map(|k| k.adjust)
    };
    assert!(kern('A', 'V').is_some_and(|k| k < 0.0) && kern('T', 'o').is_some_and(|k| k < -0.05));
    assert_eq!(kern('H', 'H'), None);
    // Error correction (msdfgen's): between two neighbouring texels that
    // the true distance (alpha) puts on the same side, well away from an
    // edge, the interpolated multi-channel median stays on that side, so no
    // corner draws a false spike or notch where the channels clash.
    let mut clashes = 0;
    for g in f
        .glyphs
        .iter()
        .filter(|g| g.atlas_w > 0 && g.codepoint < 0xF_0000)
    {
        let page = &f.pages[g.page as usize];
        let px = |x: u32, y: u32| &page.rgba[((y * page.width + x) * 4) as usize..][..4];
        for y in g.atlas_y..g.atlas_y + g.atlas_h {
            for x in g.atlas_x..g.atlas_x + g.atlas_w {
                let a = px(x, y);
                for (nx, ny) in [(x + 1, y), (x, y + 1)] {
                    if nx >= g.atlas_x + g.atlas_w || ny >= g.atlas_y + g.atlas_h {
                        continue;
                    }
                    let b = px(nx, ny);
                    let (ta, tb) = (a[3] as i32 - 128, b[3] as i32 - 128);
                    if ta.signum() != tb.signum() || ta.abs() < 24 || tb.abs() < 24 {
                        continue;
                    }
                    let mid: Vec<u8> = (0..3)
                        .map(|c| ((a[c] as u32 + b[c] as u32) / 2) as u8)
                        .collect();
                    if (median(&mid) as i32 - 128).signum() != ta.signum() {
                        clashes += 1;
                    }
                }
            }
        }
    }
    // Measured on this font: 474 such clashes without the correction, 224
    // with it (what remains is the sharp corners the median keeps and the
    // rounded true distance does not); a clash per two glyphs is the bound.
    let drawn = f.glyphs.iter().filter(|g| g.atlas_w > 0).count();
    assert!(clashes * 2 < drawn, "{clashes} clashes over {drawn} glyphs");

    // Scene: entities, nesting, model instances and the Transform table.
    let scene = b.load_scene("scenes/main.ogt").unwrap();
    assert_eq!(scene.name, "Main");
    let names: Vec<&str> = scene.entities.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["ground", "dropship", "trickster", "marker"]);
    assert_eq!(scene.entities[3].parent, Some(2));
    assert_eq!(
        scene.entities[1].instance.as_deref(),
        Some("models/dropship.glb")
    );
    let t = scene
        .components
        .iter()
        .find(|c| c.name == "Transform")
        .unwrap();
    assert!(t.inferred);
    let fields: Vec<&str> = t.schema.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(fields, ["position", "quaternion", "scale"]);
    let marker = t.get(3).unwrap();
    let f32s: Vec<f32> = marker
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect();
    assert_eq!(f32s, [0.0, 2.2, 0.0, 0.0, 0.0, 0.0, 1.0, 0.25, 0.25, 0.25]);
}
