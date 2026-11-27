//! The cook end to end on the project template
//! (`tests/fixtures/cook-assets/assets`): its determinism, and what it
//! writes loading back as authored.
//!
//! Ways this could fail, and the test that catches each:
//! - output bytes depend on thread scheduling, map order, timestamps or
//!   uninitialised padding → `every_thread_count_cooks_the_same_bytes`
//!   (cooks of a small template on 1, 2 and 8 threads, three at a time);
//! - QOA, font and scene compilation produce data the loaders cannot read
//!   or that means something else → `sounds_fonts_and_scenes_load_as_authored`.

use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::Bundle;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Typed locale input reaches the native reader with ICU, metadata and ordered
/// physical font IDs intact, including warm-helper invalidation and refusals.
#[test]
fn typed_locales_preserve_messages_and_resolve_font_chains() {
    use og_i18n::{Arg, BundleStrings, Direction};
    let root = crate::temp::TempDir::new("typed-locales");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    std::fs::create_dir_all(root.join("assets/fonts")).unwrap();
    for name in ["first", "second"] {
        std::fs::copy(
            template_assets().join("fonts/NotoSansSC-Regular-subset.ttf"),
            root.join(format!("assets/fonts/{name}.ttf")),
        )
        .unwrap();
    }
    write(
        "lib/messages.ts",
        "export const greeting = '北の{name} — مرحبًا';",
    );
    let declaration = "import {file,ref} from '@origingame/engine/author';\nimport {greeting} from '../../lib/messages.ts';\nexport default file.strings({locale:'ar_eg',fallback:'en_us',base:true,direction:'rtl',fonts:[ref('base/fonts/second'),ref('base/fonts/first')],entries:{hello:greeting,coins:'{n, plural, =0 {None} other {# coins}}'}});";
    write("content/strings/labels.ts", declaration);
    let options = CookOptions::default();
    let mut previous = None;
    for _ in 0..2 {
        let report = cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let bytes = bundle.read("base/strings/labels").unwrap();
        let table = bundle.strings("base/strings/labels", &bytes).unwrap();
        assert_eq!(
            (
                table.locale.as_str(),
                table.fallback.as_deref(),
                table.base,
                table.direction
            ),
            ("ar-EG", Some("en-US"), true, Direction::Rtl)
        );
        assert_eq!(table.fonts, ["fonts/second.ttf", "fonts/first.ttf"]);
        assert_eq!(table.entries["hello"].0, "北の{name} — مرحبًا");
        assert_eq!(
            table.entries["coins"]
                .1
                .format("en", &BTreeMap::from([("n".into(), Arg::Num(3.0))])),
            "3 coins"
        );
        if let Some(previous) = &previous {
            assert_eq!(previous, &bytes.to_vec());
        }
        previous = Some(bytes.to_vec());
    }
    write("lib/messages.ts", "export const greeting = 'New {name}';");
    let report = cook_project(&root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = bundle.read("base/strings/labels").unwrap();
    assert_ne!(previous.unwrap(), bytes.to_vec());
    assert_eq!(
        bundle
            .strings("base/strings/labels", &bytes)
            .unwrap()
            .entries["hello"]
            .0,
        "New {name}"
    );
    for (from, to, expected) in [
        ("base/fonts/second", "base/missing", "base/missing"),
        ("base/fonts/second", "base/strings/labels", "font"),
        ("ar_eg", "bad tag", "locale"),
        ("# coins", "{bad", "coins"),
    ] {
        write("content/strings/labels.ts", &declaration.replace(from, to));
        let error = match cook_project(&root, &options) {
            Err(e) => e.to_string(),
            Ok(r) => r.errors().join("\n"),
        };
        assert!(
            error.contains(expected) && error.contains("content/strings/labels.ts:3"),
            "{error}"
        );
    }
    write("content/strings/labels.ts", declaration);
    write(
        "assets/old.ogt",
        "type strings\nversion 1\n---\nlocale \"en\"\nstring \"hello\" \"Old\"\n",
    );
    assert!(!cook_project(&root, &options).unwrap().ok());
}

/// Public TS cook/load contract: canonical refs, native fields and rejection
/// stay the same on a warm cook, including an imported source's physical ID.
#[test]
fn typed_motion_actions_resolve_refs_and_load_native_fields() {
    let root = crate::temp::TempDir::new("typed-reaction");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "assets/brace.gltf",
        r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"name":"Root"}]}"#,
    );
    write(
        "assets/human.ogt",
        "type rig\nversion 1\n---\nscheme auto\n",
    );
    let declaration = "import {file,ref} from '@origingame/engine/author';\nexport default file['motion-action']({source:ref('base/brace'),clip:'brace-右',rig:ref('base/human'),holdStart:0.375,holdEnd:1.625,hand:'right',handTarget:[0.125,-0.25,0.75]});";
    write("content/actions/brace.ts", declaration);
    write(
        "content/actions/plain.ts",
        "import {file,ref} from '@origingame/engine/author';export default file['motion-action']({source:ref('base/brace'),clip:'idle',holdEnd:0.75});",
    );
    let options = CookOptions::default();
    let mut previous = None;
    for _ in 0..2 {
        let report = cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let bytes = bundle.read("base/actions/brace").unwrap();
        let action = bundle.motion_action("base/actions/brace", &bytes).unwrap();
        assert_eq!(
            (
                action.source.as_str(),
                action.rig.as_str(),
                action.clip.as_str()
            ),
            ("brace.gltf", "human.ogt", "brace-右")
        );
        assert_eq!((action.hold_start, action.hold_end), (0.375, 1.625));
        assert_eq!(action.hand, "right");
        assert_eq!(action.hand_target, [0.125, -0.25, 0.75]);
        let plain = bundle.read("base/actions/plain").unwrap();
        let plain = bundle.motion_action("base/actions/plain", &plain).unwrap();
        assert_eq!((plain.hold_start, plain.hold_end), (0.0, 0.75));
        assert!(plain.rig.is_empty() && plain.hand.is_empty());
        assert_eq!(plain.hand_target, [0.0; 3]);
        if let Some(previous) = &previous {
            assert_eq!(previous, &bytes.to_vec());
        }
        previous = Some(bytes.to_vec());
    }
    for (from, to, expected) in [
        ("holdEnd:1.625", "holdEnd:0.375", "holdStart < holdEnd"),
        ("base/brace", "base/missing", "base/missing"),
        ("base/brace", "base/human", "model"),
    ] {
        write("content/actions/brace.ts", &declaration.replace(from, to));
        let error = match cook_project(&root, &options) {
            Err(e) => e.to_string(),
            Ok(report) => format!("{:?}", report.errors()),
        };
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("content/actions/brace.ts:2"), "{error}");
    }
    write("content/actions/brace.ts", declaration);
    write(
        "assets/old.ogt",
        "type motion-action\nversion 1\n---\nsource \"brace.gltf\"\nclip \"brace\"\nholdEnd 1\n",
    );
    let report = cook_project(&root, &options).unwrap();
    assert!(!report.ok(), "old motion-action author entry was accepted");
}

/// A public navigation bake roundtrip, including imported buffer changes,
/// module refs, profile selection, ordered areas and full-width link IDs.
#[test]
fn typed_navigation_bakes_captured_geometry_and_preserves_profiles_and_links() {
    use og_ai::nav::cooked::BundleNav;
    use og_core::glam::Vec3;
    let root = crate::temp::TempDir::new("typed-nav");
    crate::init_project(&root);
    let write = |file: &str, text: &str| {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'nav',entry:'src/main.ts',modules:[{id:'north',root:'modules/north',dependsOn:['base']}]});",
    );
    write("assets/platform.gltf", &serde_json::json!({
        "asset":{"version":"2.0"}, "scene":0, "scenes":[{"nodes":[0]}],
        "nodes":[{"mesh":0}], "meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],
        "buffers":[{"uri":"platform.bin","byteLength":72}],
        "bufferViews":[{"buffer":0,"byteLength":72}],
        "accessors":[{"bufferView":0,"componentType":5126,"count":6,"type":"VEC3","min":[-5,0,-3],"max":[5,3,3]}]
    }).to_string());
    write(
        "assets/scene.ogt",
        "type scene\nversion 1\n---\nentity \"east\" {\nTransform {\nposition 20 1 0\n}\nCollider {\nsize 6 2 6\n}\n}\n",
    );
    let declaration = "import {file,ref} from '@origingame/engine/author';\nexport default file.navmesh({
        scene:ref('base/scene'),models:[{model:ref('base/platform'),transform:{position:[2,0,0],scale:[2,1,1]}}],
        boxes:[{center:[-20,0,0],size:[6,2,6]}],
        agents:[{name:'large',radius:0.75,height:2.5},{radius:0,height:1}],
        cellSize:0.5,cellHeight:0.25,tileSize:16,
        bounds:{min:[-26,-2,-8],max:[26,12,8]},
        areas:[{id:3,shape:{kind:'box',center:[0,0,0],size:[10,10,6],yaw:0.2}},
               {id:5,shape:{kind:'cylinder',center:[0,0,0],radius:2,height:8}},
               {id:7,shape:{kind:'convex',points:[[-1,-1],[1,-1],[1,1],[-1,1]],bottom:0,top:8}}],
        links:[{id:16777217,start:[-2,1,0],end:[2,1,0],kind:'door',bidirectional:false,cost:2.75,duration:0.625,open:false,traverse:'game'}]
    });";
    write("modules/north/content/navigation.ts", declaration);
    let options = CookOptions {
        threads: 2,
        ..Default::default()
    };
    let mut previous = None;
    for height in [1.0f32, 3.0] {
        let positions = [
            [-5., height, -3.],
            [-5., height, 3.],
            [5., height, 3.],
            [-5., height, -3.],
            [5., height, 3.],
            [5., height, -3.],
        ];
        std::fs::write(
            root.join("assets/platform.bin"),
            positions
                .into_iter()
                .flatten()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        for warm in [false, true] {
            let report = cook_project(&root, &options).unwrap();
            assert!(report.ok(), "{:?}", report.errors());
            let b = Bundle::open(&root.join("cooked")).unwrap();
            let bytes = b.read("north/navigation").unwrap();
            assert_eq!(
                b.navmesh_profiles("north/navigation", &bytes).unwrap(),
                ["", "large"]
            );
            let (field, tiles) = b.navmesh("north/navigation", &bytes).unwrap();
            let sample = |x: f32| {
                let column = ((x - field.origin.x) / field.params.cell_size) as i32;
                let row = ((0.0 - field.origin.z) / field.params.cell_size) as i32;
                let spans: Vec<_> = field
                    .column(column, row)
                    .map(|i| {
                        field.origin.y + field.spans[i].floor as f32 * field.params.cell_height
                    })
                    .collect();
                assert_eq!(spans.len(), 1, "x={x}: {spans:?}");
                spans[0]
            };
            assert!((sample(0.0) - height).abs() <= 0.25);
            assert!((sample(-7.0) - height).abs() <= 0.25, "model scale missing");
            assert!(
                (sample(11.0) - height).abs() <= 0.25,
                "model position missing"
            );
            assert!((sample(20.0) - 2.0).abs() <= 0.25, "scene geometry missing");
            assert!((sample(-20.0) - 1.0).abs() <= 0.25, "explicit box missing");
            let areas = b.navmesh_areas("north/navigation", &bytes).unwrap();
            assert_eq!(
                areas.iter().map(|(_, id)| *id).collect::<Vec<_>>(),
                [3, 5, 7]
            );
            let mesh = og_ai::nav::NavMesh::from_tiles_with_areas(field, tiles, areas).unwrap();
            let (poly, _) = mesh
                .nearest(Vec3::new(0.0, height, 0.0), Vec3::splat(0.5))
                .expect("walkable model surface");
            assert_eq!(
                mesh.poly(poly).unwrap().1.area,
                7,
                "last overlapping area wins"
            );
            let links = b.navmesh_links("north/navigation", &bytes).unwrap();
            let (id, link) = &links[0];
            assert_eq!(
                (*id, link.bidirectional, link.game, link.open),
                (16777217, false, true, false)
            );
            assert_eq!((link.cost, link.duration), (2.75, 0.625));
            assert!(matches!(link.kind, og_ai::nav::LinkKind::Door));
            let profile = b
                .navmesh_profile("north/navigation", &bytes, "large")
                .unwrap()
                .unwrap();
            let (large, _) = b.navmesh("north/navigation", profile).unwrap();
            assert_eq!(
                (large.params.agent_radius, large.params.agent_height),
                (0.75, 2.5)
            );
            if warm {
                assert_eq!(previous.as_ref().unwrap(), &bytes.to_vec());
            } else if let Some(previous) = &previous {
                assert_ne!(previous, &bytes.to_vec(), "stale external buffer");
            }
            previous = Some(bytes.to_vec());
        }
    }
    for (suffix, expected) in [
        ("agents:[{},{}]", "agent profile '' is used twice"),
        ("boxes:[{quaternion:[0,0,0,0]}]", "zero length"),
        (
            "links:[{id:1,start:[0,0,0],end:[1,0,0]},{id:1,start:[0,0,0],end:[2,0,0]}]",
            "link id 1 is used twice",
        ),
    ] {
        write(
            "modules/north/content/navigation.ts",
            &format!(
                "import {{file}} from '@origingame/engine/author';\nexport default file.navmesh({{{suffix}}});"
            ),
        );
        let result = cook_project(&root, &options);
        let error = match result {
            Err(e) => e.to_string(),
            Ok(report) => format!("{:?}", report.errors()),
        };
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("content/navigation.ts:2"), "{error}");
    }
    write("modules/north/content/navigation.ts", declaration);
    write("assets/old.ogt", "type navmesh\nversion 1\n---\n");
    let report = cook_project(&root, &options).unwrap();
    assert!(!report.ok(), "old navigation author entry was accepted");
}

/// `tests/fixtures/cook-assets/assets`: what the template ships.
fn template_assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/cook-assets/assets")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dst = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dst);
        } else {
            std::fs::copy(e.path(), dst).unwrap();
        }
    }
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

/// The template made small enough to cook thirty times: as the template
/// ships (without the fixture's trickster.glb, whose three 1024² textures
/// are most of a cook; its scene entity is the walker), the two standalone
/// textures cut to 128². Every kind of output and both texture encoders
/// remain.
fn small_template(root: &Path) {
    copy_dir(&template_assets(), &root.join("assets"));
    copy_dir(
        &template_assets().with_file_name("content"),
        &root.join("content"),
    );
    let assets = root.join("assets");
    for t in ["textures/stone.png", "textures/stone_n.png"] {
        let img = image::open(assets.join(t)).unwrap();
        img.resize_exact(128, 128, image::imageops::FilterType::Triangle)
            .save(assets.join(t))
            .unwrap();
    }
}

/// Cooks of one project on one, two and eight threads, the three series
/// at once (so every cook shares the machine with two others and its
/// pool's schedule moves from run to run): every file each writes is the
/// first one-thread cook's bytes, the cook times masked (`differences`).
/// One run a thread count in the quick and functional lanes; ten under
/// `--all` (`OG_GATE_ALL`), whose repeats hunt a race that
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
        crate::init_project(dir);
        copy_dir(&source.join("assets"), &dir.join("assets"));
        copy_dir(&source.join("content"), &dir.join("content"));
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
    assert!(!reference.is_empty());
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

#[test]
fn sounds_fonts_and_scenes_load_as_authored() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook/template");
    let _ = std::fs::remove_dir_all(&root);
    crate::init_project(&root);
    copy_dir(&template_assets(), &root.join("assets"));
    copy_dir(
        &template_assets().with_file_name("content"),
        &root.join("content"),
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).expect("the manifest loads");
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

    // Font: the file, whole, with its metrics; no atlas.
    let f = b.load_font("fonts/NotoSansSC-Regular-subset.ttf").unwrap();
    let source =
        std::fs::read(template_assets().join("fonts/NotoSansSC-Regular-subset.ttf")).unwrap();
    assert_eq!(f.file.as_deref(), Some(source.as_slice()));
    assert!(f.pages.is_empty() && f.glyphs.is_empty() && f.kerning.is_empty());
    assert!(f.meta.ascender > 0.5 && f.meta.descender < 0.0);

    // Locale data and compiled ICU arguments survive the native blob boundary.
    use og_i18n::BundleStrings;
    let bytes = b.read("base/strings/en").unwrap();
    let strings = b.strings("base/strings/en", &bytes).unwrap();
    assert_eq!(strings.locale, "en");
    assert!(strings.base && strings.fallback.is_none());
    assert_eq!(strings.direction, og_i18n::Direction::Ltr);
    assert_eq!(strings.entries["menu.title"].0, "Origin Engine");
    assert_eq!(strings.entries["menu.spinning"].1.arguments(), ["speed"]);

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

/// A discontinuous weight at a seam stretches two coincident triangles;
/// a disconnected, healthy triangle must retain its authored weights.
/// Exercise import through cooked animation data, not the relaxation helper.
#[test]
fn skin_relax_is_explicit_local_and_deterministic() {
    use serde_json::json;
    let root = crate::temp::TempDir::new("skin-relax");
    crate::init_project(&root);
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    let positions: [[f32; 3]; 9] = [
        [0.0, 1.0, 0.0],
        [0.03, 1.0, 0.0],
        [0.0, 1.03, 0.0],
        [0.0, 1.0, 0.0],
        [0.03, 1.0, 0.0],
        [0.0, 1.03, 0.0],
        [3.0, 0.02, 0.0],
        [3.001, 0.02, 0.0],
        [3.0, 0.021, 0.0],
    ];
    let joints: [[u16; 4]; 9] = std::array::from_fn(|i| [u16::from(i % 3 == 0), 0, 0, 0]);
    let weights = [[1.0f32, 0.0, 0.0, 0.0]; 9];
    let mut bytes = Vec::new();
    let mut views = Vec::new();
    for data in [
        bytemuck::cast_slice(&positions),
        bytemuck::cast_slice(&joints),
        bytemuck::cast_slice(&weights),
    ] {
        views.push(json!({"buffer":0,"byteOffset":bytes.len(),"byteLength":data.len()}));
        bytes.extend_from_slice(data);
    }
    let mut source = json!({
        "asset":{"version":"2.0"}, "buffers":[{"uri":"mesh.bin","byteLength":bytes.len()}],
        "bufferViews":views,
        "accessors":[
            {"bufferView":0,"componentType":5126,"count":9,"type":"VEC3","min":[0,0.02,0],"max":[3.001,1.03,0]},
            {"bufferView":1,"componentType":5123,"count":9,"type":"VEC4"},
            {"bufferView":2,"componentType":5126,"count":9,"type":"VEC4"}
        ],
        "meshes":[{"primitives":[{"attributes":{"POSITION":0,"JOINTS_0":1,"WEIGHTS_0":2}}]}],
        "nodes":[{"children":[1,4]}, {"children":[2]}, {"children":[3]}, {},
            {"children":[5]}, {"children":[6]}, {}, {"mesh":0,"skin":0}, {"mesh":0,"skin":1}],
        "skins":[{"joints":[0,1,2,3,4,5,6]}, {"joints":[0,1]}],
        "scenes":[{"nodes":[0,7,8]}],"scene":0
    });
    std::fs::write(assets.join("mesh.bin"), &bytes).unwrap();
    std::fs::write(assets.join("mesh.gltf"), source.to_string()).unwrap();
    let cook = |setting: &str, threads| {
        std::fs::write(
            assets.join("mesh.gltf.ogt"),
            format!("type import\nversion 1\n---\nmodel {{\n lods 1\n {setting}\n}}\n"),
        )
        .unwrap();
        let report = cook_project(
            &root,
            &CookOptions {
                threads,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let output = report
            .assets
            .iter()
            .flat_map(|a| &a.outputs)
            .find(|o| o.id == "mesh.gltf#anim")
            .unwrap();
        let bytes = std::fs::read(root.join("cooked").join(&output.file)).unwrap();
        let animation =
            og_anim::cooked::CookedAnimation::from_bytes("mesh.gltf#anim", &bytes).unwrap();
        (report, animation.skin_streams, bytes)
    };
    let (off, authored, _) = cook("", 1);
    assert!(
        off.warnings
            .iter()
            .any(|w| w.contains("skin stretch") && w.contains("relaxSkin"))
    );
    let (_, explicit_off, _) = cook("relaxSkin false", 1);
    assert_eq!(authored, explicit_off);
    let (on, repaired, first) = cook("relaxSkin true", 1);
    assert!(!on.warnings.iter().any(|w| w.contains("skin stretch")));
    assert!(
        on.assets
            .iter()
            .flat_map(|a| &a.notes)
            .any(|n| n.contains("6 of 9 vertices reweighted; 0 remaining"))
    );
    assert_ne!(authored, repaired);
    let stream = &repaired[0];
    // The second binding has no branch probes and only two joints. Even
    // zero-weight slots must remain valid for that binding of the shared mesh.
    assert!(stream.joints.iter().flatten().all(|&j| j < 2));
    assert_eq!(
        stream
            .weights
            .iter()
            .filter(|&&w| w == [1.0, 0.0, 0.0, 0.0])
            .count(),
        3
    );
    for ws in &stream.weights {
        assert!((ws.iter().sum::<f32>() - 1.0).abs() < 1e-5);
    }
    std::fs::remove_dir_all(root.join("cooked")).unwrap();
    let (again, _, second) = cook("relaxSkin true", 8);
    assert_eq!(first, second, "fresh parallel cook must be byte-identical");
    assert!(!again.warnings.iter().any(|w| w.contains("skin stretch")));
    let (cached, _, _) = cook("relaxSkin true", 8);
    assert!(
        cached
            .assets
            .iter()
            .flat_map(|a| &a.notes)
            .any(|n| n.contains("6 of 9 vertices reweighted; 0 remaining"))
    );
    assert_eq!(std::fs::read(assets.join("mesh.bin")).unwrap(), bytes);

    // The same mesh in quantised-accessor units, with dequantisation in its
    // inverse binds as gltfpack writes it. The tiny disconnected triangle is
    // below 8 cm in model space even though its accessor-space edges are huge.
    let scaled = positions.map(|p| p.map(|v| v * 32768.0));
    bytes[..std::mem::size_of_val(&scaled)].copy_from_slice(bytemuck::cast_slice(&scaled));
    let s = 1.0f32 / 32768.0;
    let inverse = [
        s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    source["bufferViews"].as_array_mut().unwrap().push(json!({
        "buffer":0,"byteOffset":bytes.len(),"byteLength":7 * 64
    }));
    bytes.extend_from_slice(bytemuck::cast_slice(&[inverse; 7]));
    source["buffers"][0]["byteLength"] = json!(bytes.len());
    source["accessors"][0]["min"] = json!([0.0, 0.02 * 32768.0, 0.0]);
    source["accessors"][0]["max"] = json!([3.001 * 32768.0, 1.03 * 32768.0, 0.0]);
    for (skin, count) in [7, 2].into_iter().enumerate() {
        source["skins"][skin]["inverseBindMatrices"] = json!(3 + skin);
        source["accessors"].as_array_mut().unwrap().push(json!({
            "bufferView":3,"componentType":5126,"count":count,"type":"MAT4"
        }));
    }
    std::fs::write(assets.join("mesh.bin"), &bytes).unwrap();
    std::fs::write(assets.join("mesh.gltf"), source.to_string()).unwrap();
    let (_, scaled_authored, _) = cook("", 1);
    assert_eq!(authored, scaled_authored);
    let (scaled_on, scaled_repaired, _) = cook("relaxSkin true", 1);
    assert!(
        !scaled_on
            .warnings
            .iter()
            .any(|w| w.contains("skin stretch"))
    );
    assert_eq!(
        repaired, scaled_repaired,
        "skin weights must not depend on accessor units"
    );
}
