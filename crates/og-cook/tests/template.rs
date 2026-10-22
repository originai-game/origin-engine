//! The cook and the runtime loaders, end to end on the project template
//! (`templates/game/assets`): cook it, load what came out, compare with the
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

use og_blob::BlobError;
use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::{Bundle, LoadError};
use og_core::hash::StableHasher;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

mod support;

use support::{copy_dir, template_assets};

fn evidence_dir() -> PathBuf {
    let base = std::env::var_os("OG_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("evidence"));
    let dir = base.join("M4");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A fresh project directory holding a copy of the template fixture
/// (`support::stage_template`: the template's assets with the trickster).
fn fresh_project(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    support::stage_template(&dir.join("assets"));
    dir
}

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

fn tree_hash(files: &BTreeMap<String, Vec<u8>>) -> u64 {
    let mut h = StableHasher::new();
    for (k, v) in files {
        h.write_str(k);
        h.write_u64(v.len() as u64);
        h.write(v);
    }
    h.finish()
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

#[test]
fn cooking_twice_gives_identical_bytes() {
    let first = cooked();
    let root = fresh_project("determinism");
    let options = CookOptions {
        threads: 1,
        ..Default::default()
    };
    let second = cook_project(&root, &options).unwrap();
    assert!(second.ok(), "{:?}", second.errors());
    // Every file but the cook times (wall-clock milliseconds, beside the
    // manifest and never shipped) is the same.
    let a = files(&first.root.join("cooked"));
    let b = files(&root.join("cooked"));
    let differences = differences(&a, &b);
    assert!(
        differences.is_empty(),
        "the all-cores and one-thread cooks differ:\n{}",
        differences.join("\n")
    );
    let (mut a, mut b) = (a, b);
    a.remove(og_cook::COOK_TIMES);
    b.remove(og_cook::COOK_TIMES);
    let (ha, hb) = (tree_hash(&a), tree_hash(&b));
    assert_eq!(ha, hb);
    // Every output of the second cook was produced from scratch.
    assert_eq!(
        second.cooked_outputs().len(),
        a.len() - 1,
        "every output but the manifest"
    );

    let ratio = first.cooked_bytes as f64 / first.source_bytes as f64;
    let ev = evidence_dir();
    std::fs::write(ev.join("cook-report.md"), &first.report_markdown).unwrap();
    std::fs::write(ev.join("cook-report.json"), &first.report_json).unwrap();
    let per_file: Vec<String> = a
        .iter()
        .map(|(k, v)| format!("    \"{k}\": \"{:016x}\"", og_core::hash::hash_bytes(v)))
        .collect();
    std::fs::write(
        ev.join("cook-determinism.json"),
        format!(
            "{{\n  \"what\": \"FNV-1a 64 over every (path, bytes) of cooked/, two cooks of templates/game/assets in fresh directories\",\n  \"cook_all_cores\": \"{ha:016x}\",\n  \"cook_one_thread\": \"{hb:016x}\",\n  \"files\": {},\n  \"source_bytes\": {},\n  \"cooked_bytes\": {},\n  \"ratio\": {ratio:.4},\n  \"cook_ms_all_cores\": {:.0},\n  \"cook_ms_one_thread\": {:.0},\n  \"per_file\": {{\n{}\n  }}\n}}\n",
            a.len(),
            first.source_bytes,
            first.cooked_bytes,
            first.elapsed_ms,
            second.elapsed_ms,
            per_file.join(",\n")
        ),
    )
    .unwrap();
    assert!(
        ratio <= 0.60,
        "cooked template is {:.1} % of its sources (budget 60 %)",
        ratio * 100.0
    );
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

/// Thirty cooks of one project, ten each on one, two and eight threads,
/// the three series at once (so every cook shares the machine with two
/// others and its pool's schedule moves from run to run): every file each
/// writes is the first one-thread cook's bytes, the cook times masked
/// (`differences`).
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
    // Thirteen outputs, the manifest and the cook times.
    assert_eq!(reference.len(), 15, "{:?}", reference.keys());
    let failures: Vec<String> = std::thread::scope(|s| {
        let series: Vec<_> = [1usize, 2, 8]
            .into_iter()
            .map(|threads| {
                let (base, reference, cook) = (&base, &reference, &cook);
                s.spawn(move || {
                    let mut out = Vec::new();
                    for run in 0..10 {
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

/// Renames the first glTF material in the JSON chunk (same length, so the
/// binary chunk does not move).
fn rename_material(glb: &mut [u8]) {
    let needle = b"\"name\":\"material\"";
    let at = glb
        .windows(needle.len())
        .position(|w| w == needle)
        .expect("trickster's material name");
    glb[at + 8] = b'M';
}

#[test]
fn a_gltf_edit_recooks_exactly_its_outputs() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook/incremental");
    let _ = std::fs::remove_dir_all(&root);
    copy_dir(&cooked().root, &root);
    let before = files(&root.join("cooked"));

    // Unchanged sources: nothing is cooked, nothing is written.
    let noop = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(
        noop.cooked_outputs().is_empty(),
        "{:?}",
        noop.cooked_outputs()
    );
    assert_eq!(files(&root.join("cooked")), before);
    // Each asset's time is the one recorded when it last cooked.
    let recorded = og_cook::CookTimes::read(&root.join("cooked"));
    assert_eq!(recorded.assets.len(), noop.assets.len());
    for a in &noop.assets {
        assert_eq!(
            Some(a.cook_ms),
            recorded.assets.get(&a.source).copied(),
            "{}",
            a.source
        );
    }
    let slowest = noop.slowest(10);
    assert_eq!(slowest.len(), 10.min(noop.assets.len()));
    assert!(slowest.windows(2).all(|w| w[0].cook_ms >= w[1].cook_ms));
    assert_eq!(slowest[0].source, recorded.slowest(1)[0].0);
    assert!(noop.to_markdown().contains("## The slowest ten"));

    let glb = root.join("assets/models/trickster.glb");
    let mut bytes = std::fs::read(&glb).unwrap();
    rename_material(&mut bytes);
    std::fs::write(&glb, &bytes).unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok());
    let cooked: Vec<&str> = report
        .cooked_outputs()
        .iter()
        .map(|o| o.id.as_str())
        .collect();
    assert_eq!(
        cooked,
        ["models/trickster.glb", "models/trickster.glb#anim"],
        "only the glTF's own blobs depend on its bytes (the #anim blob re-cooks the same primitives)"
    );

    let after = files(&root.join("cooked"));
    let changed: Vec<&String> = after
        .keys()
        .filter(|k| before.get(*k) != after.get(*k))
        .collect();
    assert_eq!(
        changed,
        [
            ".cook-times.ogt",
            ".cook.ogt",
            "models/trickster.glb.anim.ogb",
            "models/trickster.glb.ogb"
        ]
    );
    assert_eq!(before.len(), after.len());
    // Only the glTF's time moved: this run's, its two outputs' summed.
    let times = og_cook::CookTimes::read(&root.join("cooked"));
    let ms: f64 = report.cooked_outputs().iter().map(|o| o.ms).sum();
    let trickster = times.assets["models/trickster.glb"];
    assert!(
        (trickster - ms).abs() <= 0.05,
        "{trickster} recorded, {ms} cooked"
    );
    for (source, t) in &times.assets {
        if source != "models/trickster.glb" {
            assert_eq!(Some(t), recorded.assets.get(source), "{source}");
        }
    }

    let b = Bundle::open(&root.join("cooked")).unwrap();
    let model = b.load_model("assets/models/trickster.glb").unwrap();
    assert_eq!(model.materials[0].name, "Material");
}

#[test]
fn a_stale_blob_is_refused_with_both_stamps() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook/stale");
    let _ = std::fs::remove_dir_all(&root);
    copy_dir(&cooked().root, &root);
    let old_blob = std::fs::read(root.join("cooked/models/trickster.glb.ogb")).unwrap();
    let old_stamp = Bundle::open(&root.join("cooked"))
        .unwrap()
        .output("models/trickster.glb")
        .unwrap()
        .stamp;

    let glb = root.join("assets/models/trickster.glb");
    let mut bytes = std::fs::read(&glb).unwrap();
    rename_material(&mut bytes);
    std::fs::write(&glb, &bytes).unwrap();
    cook_project(&root, &CookOptions::default()).unwrap();
    // Put the old cook back behind the new manifest's back.
    std::fs::write(root.join("cooked/models/trickster.glb.ogb"), &old_blob).unwrap();

    let b = Bundle::open(&root.join("cooked")).unwrap();
    let new_stamp = b.output("models/trickster.glb").unwrap().stamp;
    assert_ne!(old_stamp, new_stamp);
    let err = b.load_model("models/trickster.glb").unwrap_err();
    assert_eq!(
        err,
        LoadError::Blob(BlobError::StampMismatch {
            path: "cooked/models/trickster.glb.ogb".into(),
            expected: new_stamp,
            found: old_stamp,
        })
    );
    let text = err.to_string();
    assert!(text.contains("cooked/models/trickster.glb.ogb"), "{text}");
    assert!(
        text.contains(&format!("{old_stamp:016x}")) && text.contains(&format!("{new_stamp:016x}")),
        "{text}"
    );
    let api: og_core::Error = err.into();
    assert_eq!(api.kind(), og_core::ErrorKind::Schema);

    // A blob of another type under a texture id is refused too.
    assert!(matches!(
        b.texture("models/trickster.glb", &old_blob),
        Err(LoadError::Invalid { .. })
    ));
    // An id the cook never produced.
    assert!(matches!(
        b.load_model("models/nope.glb"),
        Err(LoadError::NotCooked { .. })
    ));
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
                for p in model.meshes[m].primitives.clone() {
                    for v in &model.primitives[p].vertices {
                        let w = world[n].transform_point3(glam::Vec3::from(v.position));
                        lo = lo.min(w);
                        hi = hi.max(w);
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

fn psnr(a: &[u8], b: &[u8]) -> f64 {
    assert_eq!(a.len(), b.len());
    let mut se = 0.0f64;
    let mut n = 0usize;
    for (pa, pb) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0.iter()) {
        for c in 0..3 {
            let d = pa[c] as f64 - pb[c] as f64;
            se += d * d;
            n += 1;
        }
    }
    let mse = se / n as f64;
    if mse == 0.0 {
        f64::INFINITY
    } else {
        10.0 * (255.0f64 * 255.0 / mse).log10()
    }
}

fn png_rgba(path: &Path) -> (u32, u32, Vec<u8>) {
    let img = image::open(path).unwrap().to_rgba8();
    (img.width(), img.height(), img.into_raw())
}

#[test]
fn textures_decode_close_to_their_sources() {
    let b = bundle();
    let mut lines = Vec::new();
    let mut check = |id: &str, src: (u32, u32, Vec<u8>), min_db: f64, codec: u32| {
        let t = b.load_texture(id).unwrap();
        assert_eq!(t.meta.codec, codec, "{id}");
        assert!(t.meta.levels > 1, "{id}: a mip chain");
        let data = t.to_texture_data().unwrap();
        assert_eq!((data.width, data.height), (src.0, src.1), "{id}");
        let db = psnr(&data.rgba, &src.2);
        lines.push(format!(
            "{id}: {}x{}, PSNR {db:.2} dB (bound {min_db} dB)",
            src.0, src.1
        ));
        assert!(db >= min_db, "{id}: PSNR {db:.2} dB < {min_db} dB");
        // A vertically flipped decode must score far worse (rows top to bottom).
        let w = src.0 as usize * 4;
        let flipped: Vec<u8> = data.rgba.chunks_exact(w).rev().flatten().copied().collect();
        assert!(
            psnr(&flipped, &src.2) < db - 3.0,
            "{id}: orientation is not distinguishable"
        );
    };
    let dir = template_assets().join("textures");
    check(
        "textures/stone.png",
        png_rgba(&dir.join("stone.png")),
        30.0,
        og_cook::format::codec::ETC1S,
    );
    check(
        "textures/stone_n.png",
        png_rgba(&dir.join("stone_n.png")),
        38.0,
        og_cook::format::codec::UASTC,
    );
    // The glTF's embedded images, matched through the material slots.
    let (doc, _, images) = source_doc("trickster.glb");
    let model = b.load_model("models/trickster.glb").unwrap();
    let m = doc.materials().next().unwrap();
    let cm = &model.materials[0];
    let slots = [
        (
            m.pbr_metallic_roughness()
                .base_color_texture()
                .unwrap()
                .texture()
                .source()
                .index(),
            cm.base_color_texture,
            27.0,
            0,
        ),
        (
            m.pbr_metallic_roughness()
                .metallic_roughness_texture()
                .unwrap()
                .texture()
                .source()
                .index(),
            cm.metallic_roughness_texture,
            35.0,
            1,
        ),
        (
            m.normal_texture().unwrap().texture().source().index(),
            cm.normal_texture,
            35.0,
            1,
        ),
    ];
    for (image, tex, bound, codec) in slots {
        let img = &images[image];
        let rgba = image::DynamicImage::from(
            image::RgbImage::from_raw(img.width, img.height, img.pixels.clone())
                .unwrap_or_else(|| panic!("image {image} is RGB8")),
        )
        .to_rgba8()
        .into_raw();
        check(
            &model.textures[tex.unwrap()].id,
            (img.width, img.height, rgba),
            bound,
            codec,
        );
    }
    std::fs::write(
        evidence_dir().join("texture-psnr.md"),
        format!(
            "# Cooked texture PSNR (RGB, level 0)\n\n{}\n",
            lines
                .iter()
                .map(|l| format!("- {l}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )
    .unwrap();
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

#[test]
fn cooked_models_render_the_gltf_reference_scene() {
    use og_render::gltf::load_glb;
    let b = bundle();
    // og-tests' scene: its camera, lights, post settings and placements; only
    // the models are swapped for the cooked ones.
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
    let mut scene = og_render::RenderScene::new();
    let mut parts = Vec::new();
    for id in ["models/trickster.glb", "models/dropship.glb"] {
        let (model, textures) = b.load_model_with_textures(id).unwrap();
        parts.push(model.add_to(&mut scene, &textures));
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
    let expected = src.instances.len();
    scene.camera = src.camera;
    scene.light = src.light;
    scene.lights = src.lights.clone();
    scene.environment = src.environment;
    scene.post = src.post;
    scene.debug = src.debug;
    gfx.scene = scene;
    assert_eq!(
        gfx.scene.instances.len(),
        expected,
        "every reference instance has a cooked counterpart"
    );
    let rendered = og_tests::gfx::render_built(&mut gfx).unwrap();
    let tier = rendered.info.backend_tier();
    let reference_path = og_tests::gfx::reference_path(&tier, "gltf-reference");
    let reference = og_tests::gfx::load_png(&reference_path)
        .unwrap_or_else(|e| panic!("no gltf-reference frame for tier {tier}: {e}"));
    let tolerance = og_tests::compare::Tolerance {
        rmse: 0.02,
        ..gfx.tolerance
    };
    let cmp = og_tests::compare::compare(
        &rendered.image.pixels,
        &reference.pixels,
        rendered.image.width,
        rendered.image.height,
        &tolerance,
    )
    .unwrap();
    let ev = evidence_dir();
    og_tests::gfx::save_png(&ev.join("cooked-gltf-reference.png"), &rendered.image).unwrap();
    let diff = og_render::RgbaImage {
        width: rendered.image.width,
        height: rendered.image.height,
        pixels: cmp.diff_image.clone(),
    };
    og_tests::gfx::save_png(&ev.join("cooked-gltf-reference.diff.png"), &diff).unwrap();
    std::fs::write(
        ev.join("cooked-gltf-reference.json"),
        format!(
            "{{\"tier\": \"{tier}\", \"reference\": \"crates/og-tests/references/{tier}/gltf-reference.png\", \"rmse\": {:.5}, \"rmse_bound\": 0.02, \"max_channel_diff\": {}, \"outliers\": {}, \"pixels\": {}, \"draw_calls\": {}, \"instances\": {}, \"triangles\": {}}}\n",
            cmp.rmse,
            cmp.max_channel_diff,
            cmp.outliers,
            cmp.pixels,
            rendered.stats.draw_calls,
            rendered.stats.instances,
            rendered.stats.triangles
        ),
    )
    .unwrap();
    assert!(
        cmp.rmse <= 0.02,
        "RMSE {:.5} > 0.02 against {}",
        cmp.rmse,
        reference_path.display()
    );
}
