//! Independently cooked parcels must use their own streaming packs, release
//! their resident assets, and never hold the simulation forever. Compare
//! immediate and pending pack reads of the same 16-parcel route tick for tick.

use og_render::{RenderScene, TextureHandle};
use og_runtime::*;
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[global_allocator]
static ALLOC: og_core::alloc::Counting<std::alloc::System> =
    og_core::alloc::Counting(std::alloc::System);

const PARCELS: usize = 16;
const SIDE: u32 = 256;
// cargo test shares this allocator across its test threads; nextest runs
// them in separate processes. Keep heap samples isolated in either runner.
static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn cook(root: &Path, files: &[(&str, Vec<u8>)]) {
    for (name, data) in files {
        let path = root.join("assets").join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, data).unwrap();
    }
    let report = og_cook::cook_project(
        root,
        &og_cook::CookOptions {
            threads: 1,
            schemas: vec![og_core::Schema {
                name: "Model".into(),
                version: 1,
                doc: String::new(),
                fields: [("static", 0.0), ("visible", 1.0)]
                    .into_iter()
                    .map(|(name, on)| {
                        let mut f = og_core::reflect::FieldSchema::new(
                            name,
                            og_core::reflect::FieldType::Bool,
                        );
                        f.default = Some(vec![on]);
                        f
                    })
                    .collect(),
            }],
            ..Default::default()
        },
    )
    .unwrap();
    assert!(report.ok(), "{:?}", report.errors());
}

fn parcel(root: &Path, id: &str, group: &str, x: usize, grouped: bool) {
    let mut seed = x as u32 + 1;
    let image = image::RgbaImage::from_fn(SIDE, SIDE, |_, _| {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let [r, g, b, _] = seed.to_le_bytes();
        image::Rgba([r, g, b, 255])
    });
    let mut png = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).unwrap();
    let stream = if grouped {
        format!("stream \"{group}\" {{\n  cell\n  asset \"{id}/model.ogt\"\n}}\n")
    } else {
        String::new()
    };
    let content = format!(
        "type scene\nversion 1\n---\n{stream}entity \"house-{id}\" {{\n  model \"{id}/model.ogt\"\n}}\n"
    );
    let place = format!(
        "type scene\nversion 1\n---\nentity \"cell-{id}\" {{\n  scene \"{id}/content.ogt\"\n  Transform {{\n    position {x} 0 0\n  }}\n  Volume {{\n    size 20 10 20\n  }}\n  StreamCell {{\n    load 10\n  }}\n}}\n"
    );
    let model = format!(
        "type model\nversion 1\n---\nmaterial \"paint\" {{\n  map \"tex.png\"\n}}\npart \"ball\" {{\n  material \"paint\"\n  sphereGeometry {{\n    radius {}\n    widthSegments 32\n    heightSegments 24\n  }}\n}}\n",
        1.0 + x as f32 / 1000.0
    );
    cook(
        root,
        &[
            (&format!("{id}/tex.png"), png.into_inner()),
            (&format!("{id}/model.ogt"), model.into_bytes()),
            (&format!("{id}/content.ogt"), content.into_bytes()),
            (&format!("{id}/place.ogt"), place.into_bytes()),
        ],
    );
}

struct Disk {
    delayed: bool,
    reads: RefCell<BTreeSet<String>>,
}
impl Hooks for Disk {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        if self.delayed && path.contains("/stream/") && self.reads.borrow_mut().insert(path.into())
        {
            return Fetch::Pending;
        }
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
struct Held {
    textures: u64,
    meshes: u64,
    instances: usize,
    sixteen: usize,
    thirty_two: usize,
    radius: f32,
}
struct Recorder(Rc<Cell<Held>>);
impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let textures = (0..scene.texture_count())
            .map(TextureHandle::from_index)
            .map(|h| match (scene.texture(h), scene.texture_upload(h)) {
                (Some(t), _) => t.rgba.len() as u64,
                (_, Some(u)) => u.levels.iter().map(|l| l.len() as u64).sum(),
                _ => 0,
            })
            .sum();
        let meshes = (0..scene.mesh_count())
            .filter_map(|i| scene.mesh_handle(i).and_then(|h| scene.mesh(h)))
            .map(|m| (size_of_val(&m.vertices[..]) + size_of_val(&m.indices[..])) as u64)
            .sum();
        let count = |side| {
            (0..scene.texture_count())
                .map(TextureHandle::from_index)
                .filter(|&h| {
                    scene
                        .texture(h)
                        .is_some_and(|t| t.width == side && t.height == side)
                })
                .count()
        };
        let radius = (0..scene.mesh_count())
            .filter_map(|i| scene.mesh_handle(i).and_then(|h| scene.mesh(h)))
            .flat_map(|m| &m.vertices)
            .map(|v| v.position[0].abs())
            .fold(0.0, f32::max);
        self.0.set(Held {
            textures,
            meshes,
            instances: scene.objects.instances.len(),
            sixteen: count(16),
            thirty_two: count(32),
            radius,
        });
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}
fn answer<T>(result: Result<T, String>) -> T {
    result.unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_stream_budget_prices_device_blocks_and_exact_mips_before_admission() {
    let _lock = TEST_LOCK.lock().unwrap();
    use og_render::{TextureFormat as F, TextureFormats};
    struct Device(F, Rc<Cell<u64>>);
    impl RenderBackend for Device {
        fn tier(&self) -> String {
            "none".into()
        }
        fn size(&self) -> (u32, u32) {
            (320, 180)
        }
        fn texture_formats(&self) -> Option<TextureFormats> {
            Some(TextureFormats {
                color: self.0,
                normal: self.0,
                data: self.0,
                supported: vec![F::Rgba8, self.0],
            })
        }
        fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
            let h = TextureHandle::from_index(0);
            if let Some(u) = scene.texture_upload(h) {
                assert_eq!(u.desc.format, self.0);
                self.1.set(u.levels.iter().map(|l| l.len() as u64).sum());
            } else if let Some(t) = scene.texture(h)
                && t.width == 20
            {
                self.1.set(t.rgba.len() as u64);
            }
            Ok(())
        }
        fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
            None
        }
    }
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("stream-device-bytes-{}", std::process::id()));
    let image = image::RgbaImage::from_fn(20, 12, |x, y| {
        image::Rgba([(x * 11) as u8, (y * 19) as u8, 150, 255])
    });
    let mut png = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).unwrap();
    cook(
        &root,
        &[
            ("tex.png", png.into_inner()),
            (
                "world.ogt",
                b"type scene\nversion 1\n---\nstream \"zone\" {\n asset \"tex.png\"\n}\n".to_vec(),
            ),
        ],
    );
    for format in [F::Bc7, F::Astc4x4, F::Etc2Rgba8, F::Rgba8] {
        set_hooks(Rc::new(Disk {
            delayed: false,
            reads: RefCell::default(),
        }));
        og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
            .as_u64()
            .unwrap() as u32;
        let seen = Rc::new(Cell::new(0));
        answer(set_render_backend(
            engine,
            Some(Box::new(Device(format, seen.clone()))),
        ));
        // 20x12, 10x6, 5x3, 2x1, 1x1: blocks round up at every level.
        let (expected, budget) = if format == F::Rgba8 {
            (1272, 1400)
        } else {
            (400, 400)
        };
        answer(og_stream_set_budget(engine, budget as f64));
        answer(og_load_texture(engine, "tex.png".into()));
        answer(og_stream_request(engine, "zone".into()));
        for _ in 0..8 {
            value(og_engine_frame(engine, 1000.0 / 60.0));
        }
        let status = value(host_impl::answer(og_stream_status(engine, "zone".into())));
        assert_eq!(status["state"], "resident", "{format:?}: {status}");
        assert_eq!(status["memory"], expected as f64);
        assert_eq!(
            value(host_impl::answer(og_stream_budget(engine)))["used"],
            expected as f64
        );
        assert_eq!(seen.get(), if format == F::Rgba8 { 960 } else { 400 });
        answer(og_stream_release(engine, "zone".into()));
        value(og_engine_frame(engine, 1000.0 / 60.0));
        assert_eq!(
            value(host_impl::answer(og_stream_budget(engine)))["used"],
            0.0
        );
        assert!(og_engine_destroy(engine));
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn over_budget_cells_report_the_problem_without_freezing_or_drawing_unloaded_members() {
    let _lock = TEST_LOCK.lock().unwrap();
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("stream-budget-cells-{}", std::process::id()));
    let boot = root.join("boot");
    cook(&boot, &[("world.ogt", b"type scene\nversion 1\n---\nentity \"player\" {\n Transform {\n  position 100 0 0\n }\n StreamAnchor {\n  reach 0\n }\n}\n".to_vec())]);
    for id in ["p1", "p2"] {
        parcel(&root.join(id), id, id, 100, true);
    }
    let estimate = og_bundle::runtime::Bundle::open(&root.join("p1/cooked"))
        .unwrap()
        .streams()[0]
        .memory;
    for fits_one in [false, true] {
        set_hooks(Rc::new(Disk {
            delayed: false,
            reads: RefCell::default(),
        }));
        og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        let engine = value(og_engine_create(game, json!({"root": boot}).to_string()))
            .as_u64()
            .unwrap() as u32;
        let held = Rc::new(Cell::new(Held::default()));
        answer(set_render_backend(
            engine,
            Some(Box::new(Recorder(held.clone()))),
        ));
        let g = Game { engine, held };
        g.place("world.ogt");
        answer(og_stream_set_budget(
            engine,
            if fits_one {
                (estimate * 3 / 2) as f64
            } else {
                1.0
            },
        ));
        for id in ["p1", "p2"] {
            g.open(&root.join(id));
            g.place(&format!("{id}/place.ogt"));
        }
        for _ in 0..20 {
            g.frame();
        }
        assert!(
            g.dump()["tick"].as_u64().unwrap() >= 19,
            "budget pressure must not hold the world: {}",
            g.dump()
        );
        assert_eq!(g.loaded("p1"), fits_one);
        assert!(!g.loaded("p2"));
        assert_eq!(
            g.held.get().instances,
            usize::from(fits_one),
            "the submitted frame contains only resident cell members"
        );
        assert_eq!(g.state("p1"), if fits_one { "resident" } else { "failed" });
        assert_eq!(g.state("p2"), if fits_one { "queued" } else { "failed" });
        let log: Value = serde_json::from_str(&og_engine_take_log(engine)).unwrap();
        let warnings = log
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| {
                v["text"]
                    .as_str()
                    .is_some_and(|s| s.contains("budget") && s.contains("cell"))
            })
            .count();
        assert_eq!(
            warnings,
            if fits_one { 1 } else { 2 },
            "each blocked cell reports the budget problem once: {log}"
        );
        for _ in 0..20 {
            g.frame();
        }
        let log: Value = serde_json::from_str(&og_engine_take_log(engine)).unwrap();
        assert!(
            !log.as_array().unwrap().iter().any(|v| v["text"]
                .as_str()
                .is_some_and(|s| s.contains("budget") && s.contains("cell"))),
            "no repeated budget warning: {log}"
        );
        answer(og_stream_set_budget(engine, (estimate * 3) as f64));
        for _ in 0..8 {
            g.frame();
        }
        assert!(
            g.loaded("p1") && g.loaded("p2"),
            "a larger budget recovers both cells"
        );
        assert_eq!(g.held.get().instances, 2);
        // A later eviction must retire the loaded flag and its members too.
        answer(og_stream_set_budget(engine, 1.0));
        for _ in 0..3 {
            g.frame();
        }
        assert!(!g.loaded("p1") && !g.loaded("p2"));
        assert_eq!(g.held.get().instances, 0);
        assert!(og_engine_destroy(engine));
    }
    std::fs::remove_dir_all(root).unwrap();
}

/// Two cells needed by the same tick must not wait on that tick's ordinary
/// placement quota. Pack arrival and extra host boundaries cannot choose
/// which cell loads or change the world hash.
#[test]
fn simultaneous_cell_groups_keep_their_load_tick_across_host_boundaries() {
    let _lock = TEST_LOCK.lock().unwrap();
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("stream-held-groups-{}", std::process::id()));
    let boot = root.join("boot");
    cook(&boot, &[("world.ogt", b"type scene\nversion 1\n---\nentity \"player\" {\n Transform {\n  position 100 0 0\n }\n StreamAnchor {\n  reach 0\n }\n}\n".to_vec())]);
    for id in ["p1", "p2"] {
        parcel(&root.join(id), id, id, 100, true);
    }
    let mut baseline = None;
    for delayed in [false, true] {
        for pattern in ["plain", "probed", "idle", "catch-up"] {
            set_hooks(Rc::new(Disk {
                delayed,
                reads: RefCell::default(),
            }));
            og_host_begin_bundle();
            let game = og_game_game(Some(r#"{"seed":1}"#.into()));
            let engine = value(og_engine_create(game, json!({"root": boot}).to_string()))
                .as_u64()
                .unwrap() as u32;
            let g = Game {
                engine,
                held: Rc::default(),
            };
            g.place("world.ogt");
            for id in ["p1", "p2"] {
                g.open(&root.join(id));
                g.place(&format!("{id}/place.ogt"));
            }
            let mut tick = 0;
            for _ in 0..16 {
                match pattern {
                    "probed" => {
                        og_engine_probe_assertion_events(engine);
                    }
                    "idle" => {
                        value(og_engine_frame(engine, 0.0));
                    }
                    _ => {}
                }
                let steps = if pattern == "catch-up" { 2 } else { 1 }.min(2 - tick);
                value(og_engine_frame(engine, f64::from(steps) * 1000.0 / 60.0));
                tick = g.dump()["tick"].as_u64().unwrap() as u32;
                if tick > 0 {
                    assert!(
                        g.loaded("p1") && g.loaded("p2"),
                        "both cells must load in the first completed tick: {delayed}/{pattern}"
                    );
                }
                if tick == 2 {
                    break;
                }
            }
            assert_eq!(tick, 2, "held tick froze: {delayed}/{pattern}");
            let hash = g.dump()["hash"].clone();
            if let Some(expected) = &baseline {
                assert_eq!(
                    &hash, expected,
                    "cell load timing moved: {delayed}/{pattern}"
                );
            } else {
                baseline = Some(hash);
            }
            assert!(og_engine_destroy(engine));
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

struct Game {
    engine: u32,
    held: Rc<Cell<Held>>,
}
impl Game {
    fn frame(&self) {
        value(og_engine_frame(self.engine, 1000.0 / 60.0));
    }
    fn dump(&self) -> Value {
        serde_json::from_str(&og_engine_probe_dump(self.engine, None)).unwrap()
    }
    fn find(&self, name: &str) -> f64 {
        serde_json::from_str::<Value>(&host_impl::json(og_world_find(self.engine, name.into())))
            .unwrap()
            .as_f64()
            .unwrap_or_else(|| panic!("no entity {name}"))
    }
    fn walk(&self, x: usize) {
        value(host_impl::answer(og_world_set(
            self.engine,
            self.find("player"),
            "og.Transform".into(),
            json!({ "position": { "x": x, "y": 0, "z": 0 } }).to_string(),
        )));
    }
    fn loaded(&self, id: &str) -> bool {
        value(host_impl::answer(og_world_get(
            self.engine,
            self.find(&format!("cell-{id}")),
            "og.StreamCell".into(),
        )))["loaded"]
            == true
    }
    fn state(&self, id: &str) -> String {
        value(host_impl::answer(og_stream_status(self.engine, id.into())))["state"]
            .as_str()
            .unwrap()
            .into()
    }
    fn used(&self) -> f64 {
        value(host_impl::answer(og_stream_budget(self.engine)))["used"]
            .as_f64()
            .unwrap()
    }
    fn open(&self, root: &Path) -> f64 {
        value(host_impl::answer(og_load_bundle(
            self.engine,
            root.join("cooked").to_str().unwrap().into(),
        )))
        .as_f64()
        .unwrap()
    }
    fn place(&self, path: &str) {
        let h = value(host_impl::answer(og_load_scene(self.engine, path.into())))
            .as_f64()
            .unwrap();
        value(host_impl::answer(og_world_instantiate(
            self.engine,
            h,
            None,
        )));
    }
}

fn id(i: usize) -> String {
    // Two simultaneously open versions; no prefix matching or shared slots.
    if i < 2 {
        format!("a-v{}", i + 1)
    } else {
        format!("p{i}-v1")
    }
}

fn route(base: &Path, grouped: bool, delayed: bool) -> Vec<Value> {
    let root = base.join("game");
    cook(&root, &[("world.ogt", b"type scene\nversion 1\n---\nentity \"player\" {\n  Transform {\n    position 0 0 0\n  }\n  StreamAnchor {\n    reach 0\n  }\n}\n".to_vec())]);
    for i in 0..PARCELS {
        parcel(&base.join(id(i)), &id(i), &id(i), (i + 1) * 100, grouped);
    }
    set_hooks(Rc::new(Disk {
        delayed,
        reads: RefCell::default(),
    }));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
        .as_u64()
        .unwrap() as u32;
    let held = Rc::new(Cell::new(Held::default()));
    answer(set_render_backend(
        engine,
        Some(Box::new(Recorder(held.clone()))),
    ));
    let g = Game { engine, held };
    g.place("world.ogt");
    for i in 0..PARCELS {
        let h = g.open(&base.join(id(i)));
        assert_eq!(
            og_load_status(engine, h),
            "ready",
            "{:?}",
            og_load_error(engine, h)
        );
        g.place(&format!("{}/place.ogt", id(i)));
    }
    g.frame();
    g.frame();
    let before = g.held.get();
    let mut hashes = Vec::new();
    for i in 0..PARCELS {
        g.walk((i + 1) * 100);
        // Native I/O must not hold a tick; also captures the web-style gate
        // regression as a bounded failure rather than an infinite test.
        let tick = g.dump()["tick"].as_u64().unwrap();
        for _ in 0..4 {
            g.frame();
            hashes.push(g.dump()["hash"].clone());
        }
        assert!(
            g.dump()["tick"].as_u64().unwrap() > tick,
            "cell {} froze at tick {tick}",
            id(i)
        );
        assert!(g.loaded(&id(i)));
        let full = g.held.get();
        assert!(
            full.textures >= before.textures + u64::from(SIDE * SIDE * 4),
            "{full:?}"
        );
        if grouped {
            assert_eq!(g.state(&id(i)), "resident");
            assert!(g.used() > 0.0);
        }
        g.walk(0);
        for _ in 0..4 {
            g.frame();
            hashes.push(g.dump()["hash"].clone());
        }
        assert!(!g.loaded(&id(i)));
        let away = g.held.get();
        println!(
            "CURVE grouped={grouped} delayed={delayed} parcels={} heap={} textures={} meshes={} used={}",
            i + 1,
            og_core::alloc::heap().unwrap().live,
            away.textures,
            away.meshes,
            g.used()
        );
        if grouped {
            assert_eq!(g.state(&id(i)), "unloaded");
            assert_eq!(g.used(), 0.0);
            assert!(away.textures <= before.textures + 64, "{away:?}");
            assert!(away.meshes < full.meshes, "{away:?} vs {full:?}");
        }
    }
    if grouped {
        parcel(&base.join("duplicate"), "duplicate", "a-v1", 9000, true);
        let h = g.open(&base.join("duplicate"));
        assert_eq!(og_load_status(engine, h), "failed");
        let why = og_load_error(engine, h).unwrap();
        for path in ["a-v1", "a-v1/cooked", "duplicate/cooked"] {
            assert!(why.contains(path), "{why}");
        }
        // A failed open and a repeated open leave the first owner's group
        // usable, including the reload of its evicted scene and textures.
        let h = g.open(&base.join("a-v1"));
        assert_eq!(og_load_status(engine, h), "ready");
        g.walk(100);
        for _ in 0..4 {
            g.frame();
        }
        assert!(g.loaded("a-v1"));
        assert_eq!(g.state("a-v1"), "resident");
        assert!(g.held.get().textures >= u64::from(SIDE * SIDE * 4));
    }
    assert!(og_engine_destroy(engine));
    hashes
}

#[test]
fn runtime_bundles_stream_without_freezing_or_changing_the_world() {
    let _lock = TEST_LOCK.lock().unwrap();
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-stream-bundles-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let ungrouped = route(&base.join("ungrouped"), false, false);
    println!("UNGROUPED_HASHES {}", json!(ungrouped));
    let grouped = route(&base.join("grouped"), true, false);
    let delayed = route(&base.join("delayed"), true, true);
    assert_eq!(
        grouped, delayed,
        "pending pack reads must not change simulation hashes"
    );
    std::fs::remove_dir_all(base).unwrap();
}

/// Full meshes must not leave their cooked raycast BVHs behind. HLOD's
/// baked atlases and proxies live as long as their placements, not forever
/// in the model cache. Keep all bundles open throughout this route.
#[test]
fn hlod_parcels_release_full_bvhs_and_dead_placement_proxies() {
    let _lock = TEST_LOCK.lock().unwrap();
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-hlod-parcels-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let root = base.join("game");
    cook(&root, &[("world.ogt", b"type scene\nversion 1\n---\nentity \"player\" {\n  Transform {\n    position 0 0 0\n  }\n  StreamAnchor {\n    reach 0\n  }\n}\n".to_vec())]);
    for i in 0..PARCELS {
        let id = id(i);
        let parcel_root = base.join(&id);
        parcel(&parcel_root, &id, &id, (i + 1) * 100, true);
        let mut content = format!(
            "type scene\nversion 1\n---\nhlod {{\n  size 32\n  error 0.25\n  texels 256\n}}\nstream \"{id}\" {{\n  cell\n  asset \"{id}/model.ogt\"\n}}\n"
        );
        for n in 0..4 {
            content.push_str(&format!("entity \"house-{id}-{n}\" {{\n  model \"{id}/model.ogt\"\n  static true\n  Transform {{\n    position {} 0 0\n  }}\n}}\n", n * 12));
        }
        let model = std::fs::read_to_string(parcel_root.join(format!("assets/{id}/model.ogt")))
            .unwrap()
            .replace("widthSegments 32", "widthSegments 128")
            .replace("heightSegments 24", "heightSegments 96");
        let duplicate = std::fs::read_to_string(parcel_root.join(format!("assets/{id}/place.ogt")))
            .unwrap()
            .replace(&format!("cell-{id}"), "cell-copy");
        cook(
            &parcel_root,
            &[
                (&format!("{id}/content.ogt"), content.into_bytes()),
                (&format!("{id}/model.ogt"), model.into_bytes()),
                (&format!("{id}/duplicate.ogt"), duplicate.into_bytes()),
            ],
        );
    }
    set_hooks(Rc::new(Disk {
        delayed: false,
        reads: RefCell::default(),
    }));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
        .as_u64()
        .unwrap() as u32;
    let held = Rc::new(Cell::new(Held::default()));
    answer(set_render_backend(
        engine,
        Some(Box::new(Recorder(held.clone()))),
    ));
    let g = Game { engine, held };
    g.place("world.ogt");
    for i in 0..PARCELS {
        assert_eq!(og_load_status(engine, g.open(&base.join(id(i)))), "ready");
        g.place(&format!("{}/place.ogt", id(i)));
    }
    g.frame();
    g.frame();
    let placed = g.held.get();
    assert!(
        placed.instances >= PARCELS,
        "no distant HLOD proxies: {placed:?}"
    );
    assert!(
        placed.textures > PARCELS as u64 * 32 * 1024,
        "no HLOD atlases: {placed:?}"
    );
    let start = og_core::alloc::heap().unwrap().live;
    for i in 0..PARCELS {
        g.walk((i + 1) * 100);
        for _ in 0..4 {
            g.frame();
        }
        assert!(g.loaded(&id(i)));
        assert_eq!(g.state(&id(i)), "resident");
        let full = g.held.get();
        assert!(full.textures >= placed.textures + u64::from(SIDE * SIDE * 4));
        g.walk(0);
        for _ in 0..4 {
            g.frame();
        }
        assert!(!g.loaded(&id(i)));
        assert_eq!(g.used(), 0.0);
        let live = og_core::alloc::heap().unwrap().live;
        let held = g.held.get();
        assert!(held.meshes < full.meshes);
        assert!(held.textures <= placed.textures + 4);
        println!(
            "HLOD_CURVE phase=visited parcels={} heap={live} textures={} meshes={}",
            i + 1,
            held.textures,
            held.meshes
        );
        println!("HLOD_HASH visited={} {}", i + 1, g.dump()["hash"]);
    }
    let end = og_core::alloc::heap().unwrap().live;
    let visited = g.held.get();
    // Two placements share a tree: the first removal must not free the
    // survivor's proxies, but the last removal must release them.
    let first = g.find("cell-a-v1");
    g.place("a-v1/duplicate.ogt");
    g.frame();
    let twice = g.held.get();
    value(host_impl::answer(og_world_despawn(engine, first)));
    g.frame();
    assert_eq!(g.held.get().textures, twice.textures);
    for i in 0..PARCELS {
        value(host_impl::answer(og_world_despawn(
            engine,
            g.find(&if i == 0 {
                "cell-copy".into()
            } else {
                format!("cell-{}", id(i))
            }),
        )));
        g.frame();
        let held = g.held.get();
        println!(
            "HLOD_CURVE phase=unplaced parcels={} heap={} textures={} meshes={}",
            i + 1,
            og_core::alloc::heap().unwrap().live,
            held.textures,
            held.meshes
        );
        println!("HLOD_HASH unplaced={} {}", i + 1, g.dump()["hash"]);
    }
    let away = g.held.get();
    assert_eq!(away.instances, 0, "dead placements still draw: {away:?}");
    // Removed renderer slots keep one clear texel, not their atlases.
    assert!(
        away.textures < 1024,
        "HLOD atlases outlive their placements: {away:?}"
    );
    assert!(
        away.meshes < visited.meshes,
        "HLOD meshes outlive their placements: {away:?}"
    );
    assert!(
        end.saturating_sub(start) + (visited.meshes.saturating_sub(placed.meshes) as usize)
            < PARCELS * 1024 * 1024,
        "visited parcels retain >1 MiB of heap + mesh upload payload each: {start} -> {end}, {placed:?} -> {visited:?}"
    );
    // Reload must rebuild resources rather than returning an empty cached model.
    g.place("a-v1/place.ogt");
    g.frame();
    assert!(g.held.get().instances > 0 && g.held.get().textures > away.textures + 16 * 1024);
    assert!(og_engine_destroy(engine));
    std::fs::remove_dir_all(base).unwrap();
}

/// Deliberately identical cooked ids, but different cell names, geometry,
/// texture sizes and group names. The unique placing scene selects a bundle;
/// its same-path dependencies must keep that source through load and reload.
fn version(root: &Path, v: usize, grouped: bool) {
    let side = 16 * v as u32;
    let image = image::RgbaImage::from_pixel(side, side, image::Rgba([40, 80, 160, 255]));
    let mut png = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).unwrap();
    let stream = if grouped {
        format!("stream \"v{v}\" {{\n  cell\n  asset \"model.ogt\"\n}}\n")
    } else {
        String::new()
    };
    cook(root, &[
        (&format!("v{v}/place.ogt"), format!(
            "type scene\nversion 1\n---\nentity \"cell-v{v}\" {{\n  scene \"content.ogt\"\n  Transform {{\n    position {} 0 0\n  }}\n  Volume {{\n    size 20 10 20\n  }}\n  StreamCell {{\n    load 10\n  }}\n}}\n", v * 100).into_bytes()),
        ("content.ogt", format!("type scene\nversion 1\n---\n{stream}entity \"house-v{v}\" {{\n  model \"model.ogt\"\n}}\n").into_bytes()),
        ("model.ogt", format!("type model\nversion 1\n---\nmaterial \"paint\" {{\n  map \"tex.png\"\n}}\npart \"ball\" {{\n  material \"paint\"\n  sphereGeometry {{\n    radius {v}\n    widthSegments 16\n    heightSegments 12\n  }}\n}}\n").into_bytes()),
        ("tex.png", png.into_inner()),
    ]);
}

#[test]
fn same_path_versions_keep_their_own_scenes_models_textures_and_budget() {
    let _lock = TEST_LOCK.lock().unwrap();
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-bundle-versions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    for grouped in [true, false] {
        for resident in [false, true] {
            let root = base.join(format!("{grouped}-{resident}"));
            let boot = root.join("boot");
            cook(&boot, &[("world.ogt", b"type scene\nversion 1\n---\nentity \"player\" {\n  StreamAnchor {\n    reach 0\n  }\n}\n".to_vec())]);
            for v in 1..=2 {
                version(&root.join(format!("v{v}")), v, grouped);
            }
            set_hooks(Rc::new(Disk {
                delayed: true,
                reads: RefCell::default(),
            }));
            og_host_begin_bundle();
            let game = og_game_game(Some(r#"{"seed":1}"#.into()));
            let engine = value(og_engine_create(game, json!({"root": boot}).to_string()))
                .as_u64()
                .unwrap() as u32;
            let held = Rc::new(Cell::new(Held::default()));
            answer(set_render_backend(
                engine,
                Some(Box::new(Recorder(held.clone()))),
            ));
            let g = Game { engine, held };
            g.place("world.ogt");
            assert_eq!(og_load_status(engine, g.open(&root.join("v1"))), "ready");
            g.place("v1/place.ogt");
            g.walk(100);
            for _ in 0..4 {
                g.frame();
            }
            assert!(g.loaded("v1"));
            let model = |name| {
                value(host_impl::answer(og_world_get(
                    engine,
                    g.find(name),
                    "og.Model".into(),
                )))["asset"]
                    .as_f64()
                    .unwrap()
            };
            let v1 = model("house-v1");
            assert_eq!(g.held.get().sixteen, 1);
            let v1_full = g.held.get();
            if !resident {
                g.walk(0);
                for _ in 0..4 {
                    g.frame();
                }
                assert!(!g.loaded("v1"));
                if grouped {
                    assert_eq!(g.used(), 0.0);
                }
            }
            // Open v2 either while v1 is resident or after its unload. In
            // both cases v1 remains open, listing every same-path id.
            assert_eq!(og_load_status(engine, g.open(&root.join("v2"))), "ready");
            g.place("v2/place.ogt");
            g.walk(200);
            for _ in 0..4 {
                g.frame();
            }
            assert!(g.loaded("v2"));
            assert!(!g.loaded("v1"));
            let v2 = model("house-v2");
            assert_ne!(
                v1, v2,
                "same paths in different bundles must not share load slots"
            );
            assert_eq!(g.held.get().thirty_two, 1, "v2 must decode its own texture");
            assert!(
                (g.held.get().radius - 2.0).abs() < 0.01,
                "v2 must decode its own geometry"
            );
            if grouped {
                assert_eq!(g.state("v1"), "unloaded");
                assert_eq!(g.state("v2"), "resident");
                let v2_bytes =
                    value(host_impl::answer(og_stream_status(engine, "v2".into())))["memory"]
                        .as_f64()
                        .unwrap();
                assert_eq!(g.used(), v2_bytes, "only v2 counts in the shared budget");
                assert_eq!(g.held.get().sixteen, 0, "v1 texture was released");
            }
            // A direct load intentionally keeps boot-first / open-order
            // lookup, not the most recently placed scene's context.
            let direct = value(host_impl::answer(og_load_gltf(
                engine,
                "model.ogt".into(),
                None,
            )))
            .as_f64()
            .unwrap();
            assert_eq!(direct, v1);
            g.walk(100);
            for _ in 0..4 {
                g.frame();
            }
            assert!(g.loaded("v1"));
            assert_eq!(model("house-v1"), v1);
            if grouped {
                assert_eq!(g.state("v2"), "unloaded");
                assert_eq!(g.held.get().thirty_two, 0);
                assert!(
                    g.held.get().textures <= v1_full.textures + 4,
                    "only v2's 1x1 proxy may remain"
                );
                assert_eq!(g.held.get().sixteen, 1);
            }
            assert!(og_engine_destroy(engine));
        }
    }
    std::fs::remove_dir_all(base).unwrap();
}

#[test]
fn missing_scene_dependencies_fall_back_to_boot_then_open_order() {
    let _lock = TEST_LOCK.lock().unwrap();
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-bundle-fallback-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    for boot_has_scene in [true, false] {
        let root = base.join(boot_has_scene.to_string());
        for v in 1..=4 {
            version(&root.join(format!("v{v}")), v, false);
        }
        let boot = root.join("v1");
        cook(&boot, &[("world.ogt", b"type scene\nversion 1\n---\nentity \"player\" {\n  StreamAnchor {\n    reach 0\n  }\n}\n".to_vec())]);
        // A bundle can omit an externally supplied dependency from its
        // manifest. Keep the placing scene's reference, not a broken blob.
        for v in [1, 3] {
            if v == 1 && boot_has_scene {
                continue;
            }
            let manifest = root.join(format!("v{v}/cooked/.cook.ogt"));
            let mut m =
                og_bundle::runtime::Manifest::parse(&std::fs::read_to_string(&manifest).unwrap())
                    .unwrap();
            m.assets.retain(|a| a.source != "content.ogt");
            std::fs::write(manifest, m.to_text()).unwrap();
        }
        set_hooks(Rc::new(Disk {
            delayed: false,
            reads: RefCell::default(),
        }));
        og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        let engine = value(og_engine_create(game, json!({"root": boot}).to_string()))
            .as_u64()
            .unwrap() as u32;
        let held = Rc::new(Cell::new(Held::default()));
        answer(set_render_backend(
            engine,
            Some(Box::new(Recorder(held.clone()))),
        ));
        let g = Game { engine, held };
        g.place("world.ogt");
        for v in [2, 4, 3] {
            assert_eq!(
                og_load_status(engine, g.open(&root.join(format!("v{v}")))),
                "ready"
            );
        }
        g.place("v3/place.ogt");
        g.walk(300);
        for _ in 0..4 {
            g.frame();
        }
        assert!(g.loaded("v3"));
        let selected = if boot_has_scene { 1 } else { 2 };
        g.find(&format!("house-v{selected}"));
        assert!((g.held.get().radius - selected as f32).abs() < 0.01);
        assert_eq!(g.held.get().sixteen, usize::from(boot_has_scene));
        assert_eq!(g.held.get().thirty_two, usize::from(!boot_has_scene));
        assert!(og_engine_destroy(engine));
    }
    std::fs::remove_dir_all(base).unwrap();
}
#[test]
fn pending_model_textures_hide_only_presentation_and_preserve_128_tick_hashes() {
    let _lock = TEST_LOCK.lock().unwrap();
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    struct Pending {
        ready: Arc<AtomicBool>,
        native: bool,
        pack: Option<Rc<Cell<bool>>>,
    }
    impl Hooks for Pending {
        fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
            Ok(String::new())
        }
        fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
            if path.ends_with("/shared.ogb") && self.pack.as_ref().is_some_and(|p| !p.get()) {
                return Fetch::Pending;
            }
            match std::fs::read(path) {
                Ok(b) => Fetch::Ready(b),
                Err(e) => Fetch::Failed(e.to_string()),
            }
        }
        fn texture(
            &self,
            t: &og_bundle::runtime::CookedTexture,
            f: Option<&og_render::TextureFormats>,
        ) -> Fetch<og_render::TextureSource> {
            if !self.ready.load(Ordering::Acquire) {
                return Fetch::Pending;
            }
            match t.to_texture_source(f) {
                Ok(t) => Fetch::Ready(t),
                Err(e) => Fetch::Failed(e.to_string()),
            }
        }
        fn texture_work(
            &self,
            t: &og_bundle::runtime::CookedTexture,
            f: Option<&og_render::TextureFormats>,
        ) -> Option<TextureWork> {
            if !self.native {
                return None;
            }
            let (ready, t, f) = (self.ready.clone(), t.clone(), f.cloned());
            Some(Box::new(move || {
                while !ready.load(Ordering::Acquire) {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                t.to_texture_source(f.as_ref()).map_err(|e| e.to_string())
            }))
        }
    }
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("texture-ready-{}", std::process::id()));
    let root = base.join("game");
    cook(&root, &[("world.ogt", b"type scene\nversion 1\n---\nentity \"player\" {\n Transform {\n  position 0 0 0\n }\n StreamAnchor {\n  reach 0\n }\n}\n".to_vec())]);
    for grouped in [false, true] {
        let path = base.join(if grouped { "grouped" } else { "plain" });
        parcel(&path, "parcel", "parcel", 100, grouped);
        let mut baseline = Vec::new();
        for (delayed, native) in [(false, false), (true, false), (true, true)] {
            let ready = Arc::new(AtomicBool::new(!delayed));
            set_hooks(Rc::new(Pending {
                ready: ready.clone(),
                native,
                pack: None,
            }));
            og_host_begin_bundle();
            let game = og_game_game(Some(r#"{"seed":1}"#.into()));
            let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
                .as_u64()
                .unwrap() as u32;
            let held = Rc::new(Cell::new(Held::default()));
            answer(set_render_backend(
                engine,
                Some(Box::new(Recorder(held.clone()))),
            ));
            let g = Game { engine, held };
            g.place("world.ogt");
            g.open(&path);
            g.place("parcel/place.ogt");
            g.walk(100);
            let mut hashes = Vec::new();
            for tick in 0..128 {
                if tick == 64 {
                    ready.store(true, Ordering::Release);
                }
                g.frame();
                hashes.push(g.dump()["hash"].clone());
                if delayed && tick == 8 {
                    assert!(
                        g.loaded("parcel"),
                        "entities load independently of textures"
                    );
                    assert_eq!(g.held.get().instances, 0, "never draw an untextured model");
                    assert!(!og_loading_idle(engine));
                    if grouped {
                        assert_eq!(g.state("parcel"), "resident");
                        assert!(!og_stream_status(engine, "parcel".into()).unwrap().ready);
                    }
                }
            }
            if delayed {
                assert_eq!(hashes, baseline, "worker timing changed world hashes");
            } else {
                baseline = hashes;
            }
            for _ in 0..2000 {
                g.frame();
                if og_loading_idle(engine) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            assert!(og_loading_idle(engine));
            assert!(g.held.get().instances > 0, "prepared model is revealed");
            assert!(g.held.get().textures >= u64::from(SIDE * SIDE * 4));
            if grouped {
                assert_eq!(g.state("parcel"), "resident");
                assert!(og_stream_status(engine, "parcel".into()).unwrap().ready);
            }
            assert!(og_engine_destroy(engine));
        }
    }
    // A cell's own pack can be ready before the distance group of a model
    // it places. Presentation waits for both, never drawing either a model
    // stand-in or an HLOD proxy with pending maps.
    for hlod in [false, true] {
        let path = base.join(format!("shared-{hlod}"));
        parcel(&path, "parcel", "parcel", 100, true);
        let content = format!(
            "type scene\nversion 1\n---\n{}stream \"parcel\" {{\n cell\n asset \"parcel/own.ogt\"\n}}\nstream \"shared\" {{\n asset \"parcel/model.ogt\"\n distance 100 0 0 10000\n}}\nentity \"house-parcel\" {{\n model \"parcel/model.ogt\"\n static true\n Transform {{\n    position 0 0 0\n }}\n}}\nentity \"house-second\" {{\n model \"parcel/model.ogt\"\n static true\n Transform {{\n position 4 0 0\n }}\n}}\n",
            if hlod {
                "hlod {\n size 32\n error 0.25\n texels 64\n}\n"
            } else {
                ""
            }
        );
        cook(&path, &[
            ("parcel/content.ogt", content.into_bytes()),
            ("parcel/own.ogt", b"type model\nversion 1\n---\npart \"unused\" {\n boxGeometry {\n width 1\n height 1\n depth 1\n }\n}\n".to_vec()),
        ]);
        let mut baseline = Vec::new();
        for (hold_pack, delayed, native) in [
            (false, false, false),
            (false, true, false),
            (false, true, true),
            (true, true, false),
        ] {
            let ready = Arc::new(AtomicBool::new(!delayed));
            let pack = Rc::new(Cell::new(!hold_pack));
            set_hooks(Rc::new(Pending {
                ready: ready.clone(),
                native,
                pack: Some(pack.clone()),
            }));
            og_host_begin_bundle();
            let game = og_game_game(Some(r#"{"seed":1}"#.into()));
            let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
                .as_u64()
                .unwrap() as u32;
            let held = Rc::new(Cell::new(Held::default()));
            answer(set_render_backend(
                engine,
                Some(Box::new(Recorder(held.clone()))),
            ));
            let g = Game { engine, held };
            g.place("world.ogt");
            g.open(&path);
            g.place("parcel/place.ogt");
            if !hlod {
                g.walk(100);
            }
            value(og_engine_write_resource(
                engine,
                "og.Camera".into(),
                json!({"position": {"x":100,"y":0,"z":3000}, "fov":50}).to_string(),
            ));
            let mut hashes = Vec::new();
            for tick in 0..128 {
                if tick == 16 {
                    pack.set(true);
                }
                if hlod && tick == 96 {
                    g.walk(100);
                }
                if tick == 64 {
                    ready.store(true, Ordering::Release);
                }
                g.frame();
                hashes.push(g.dump()["hash"].clone());
                if delayed && (tick == 8 || tick == 40) {
                    if hlod {
                        assert!(!g.loaded("parcel"), "also exercise unloaded-cell HLOD");
                    } else {
                        assert!(g.loaded("parcel"));
                        assert_eq!(g.state("parcel"), "resident");
                    }
                    assert!(
                        !og_stream_status(engine, "parcel".into()).unwrap().ready,
                        "shared group or its texture still pending; hlod={hlod}"
                    );
                    assert!(
                        !og_stream_budget(engine)
                            .unwrap()
                            .groups
                            .iter()
                            .find(|g| g.name == "parcel")
                            .unwrap()
                            .ready
                    );
                    assert_eq!(
                        g.held.get().instances,
                        0,
                        "pending proxies must not draw; hlod={hlod}"
                    );
                }
                if hlod && !native && tick == 80 {
                    assert!(!g.loaded("parcel"));
                    assert!(
                        g.held.get().instances > 0,
                        "prepared unloaded-cell proxies must invalidate kept draws"
                    );
                }
            }
            if !hold_pack {
                if delayed {
                    assert_eq!(hashes, baseline, "shared decode timing changed hashes");
                } else {
                    baseline = hashes;
                }
            }
            for _ in 0..2000 {
                g.frame();
                if og_loading_idle(engine) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            assert!(og_loading_idle(engine));
            assert!(og_stream_status(engine, "parcel".into()).unwrap().ready);
            assert!(
                g.held.get().instances > 0,
                "ready models/proxies must become visible"
            );
            if hlod {
                let _ = og_engine_take_outputs(engine);
                let _: Value =
                    serde_json::from_str(&og_engine_probe_commands(engine, "0:dump_render".into()))
                        .unwrap();
                g.frame();
                let outputs: Value = serde_json::from_str(&og_engine_take_outputs(engine)).unwrap();
                let output = outputs
                    .as_array()
                    .unwrap()
                    .iter()
                    .rev()
                    .find(|o| o["kind"] == "json")
                    .unwrap();
                let render: Value = serde_json::from_str(output["data"].as_str().unwrap()).unwrap();
                assert!(
                    render["hlod"]["proxies"].as_u64().unwrap() > 0,
                    "far fixture must exercise HLOD: {render}"
                );
            }
            assert!(og_engine_destroy(engine));
        }
    }
    // Equal cooked ids in two runtime bundles must prepare independently.
    // Otherwise each poll cancels the other bundle's texture job forever.
    for grouped in [false, true] {
        let versions = base.join(format!("versions-{grouped}"));
        for v in 1..=2 {
            version(&versions.join(format!("v{v}")), v, grouped);
        }
        let ready = Arc::new(AtomicBool::new(false));
        set_hooks(Rc::new(Pending {
            ready: ready.clone(),
            native: true,
            pack: None,
        }));
        og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
            .as_u64()
            .unwrap() as u32;
        let held = Rc::new(Cell::new(Held::default()));
        answer(set_render_backend(
            engine,
            Some(Box::new(Recorder(held.clone()))),
        ));
        let g = Game { engine, held };
        g.place("world.ogt");
        for v in 1..=2 {
            g.open(&versions.join(format!("v{v}")));
            g.place(&format!("v{v}/place.ogt"));
        }
        value(host_impl::answer(og_world_set(
            engine,
            g.find("player"),
            "og.StreamAnchor".into(),
            json!({"reach":200}).to_string(),
        )));
        g.walk(150);
        for _ in 0..8 {
            g.frame();
        }
        assert!(g.loaded("v1") && g.loaded("v2"));
        assert_eq!(g.held.get().instances, 0);
        ready.store(true, Ordering::Release);
        for _ in 0..2000 {
            g.frame();
            if og_loading_idle(engine) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(og_loading_idle(engine));
        assert_eq!(g.held.get().sixteen, 1);
        assert_eq!(g.held.get().thirty_two, 1);
        assert!(g.held.get().instances >= 2);
        assert!(og_engine_destroy(engine));
    }
    std::fs::remove_dir_all(base).unwrap();
}

/// Release-mode CPU profile of an independently cooked cell with four
/// 1024-square material maps. The recorder consumes the real scene but
/// does not upload it to a GPU; GPU frame budgets need a native GPU run too.
#[test]
#[ignore = "release timing repro; cooks four material maps"]
fn cell_load_timing() {
    let _lock = TEST_LOCK.lock().unwrap();
    use std::time::Instant;
    struct Profile;
    thread_local! {
        static DECODE: Cell<f64> = const { Cell::new(0.0) };
    }
    impl Hooks for Profile {
        fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
            Ok(String::new())
        }
        fn now_ms(&self) -> f64 {
            thread_local! { static START: Instant = Instant::now(); }
            START.with(|s| s.elapsed().as_secs_f64() * 1000.0)
        }
        fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
            match std::fs::read(path) {
                Ok(b) => Fetch::Ready(b),
                Err(e) => Fetch::Failed(e.to_string()),
            }
        }
        fn texture_work(
            &self,
            t: &og_bundle::runtime::CookedTexture,
            f: Option<&og_render::TextureFormats>,
        ) -> Option<TextureWork> {
            if std::env::var("OG_CELL_SYNC").is_ok() {
                return None;
            }
            let t = t.clone();
            let f = f.cloned();
            Some(Box::new(move || {
                t.to_texture_source(f.as_ref()).map_err(|e| e.to_string())
            }))
        }
        fn texture(
            &self,
            t: &og_bundle::runtime::CookedTexture,
            f: Option<&og_render::TextureFormats>,
        ) -> Fetch<og_render::TextureSource> {
            let start = Instant::now();
            let result = t.to_texture_source(f);
            DECODE.with(|d| d.set(d.get() + start.elapsed().as_secs_f64() * 1000.0));
            match result {
                Ok(t) => Fetch::Ready(t),
                Err(e) => Fetch::Failed(e.to_string()),
            }
        }
    }
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cell-load-timing");
    let root = base.join("game");
    cook(&root, &[("world.ogt", b"type scene\nversion 1\n---\nentity \"player\" {\n  Transform {\n    position 0 0 0\n  }\n  StreamAnchor {\n    reach 0\n  }\n}\n".to_vec())]);
    let count: u32 = std::env::var("OG_CELL_TEXTURES")
        .ok()
        .map(|s| s.parse().unwrap())
        .unwrap_or(4);
    for (id, grouped, control) in [
        ("plain", false, false),
        ("grouped", true, false),
        ("control", false, true),
    ] {
        let parcel_root = base.join(id);
        parcel(&parcel_root, id, id, 100, grouped);
        let mut files = Vec::new();
        let mut model = "type model\nversion 1\n---\n".to_string();
        for n in 0..count {
            let mut seed = n + 1u32;
            let image = image::RgbaImage::from_fn(1024, 1024, |x, y| {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                let mortar = y % 64 < 3 || (x + (y / 64 % 2) * 64) % 128 < 3;
                let v = if mortar { 70 } else { 120 + (seed % 60) as u8 };
                image::Rgba([v, v / 2, v / 3, 255])
            });
            let mut png = std::io::Cursor::new(Vec::new());
            image.write_to(&mut png, image::ImageFormat::Png).unwrap();
            files.push((format!("{id}/map{n}.png"), png.into_inner()));
            model.push_str(&format!("material \"m{n}\" {{\n map \"map{n}.png\"\n}}\npart \"p{n}\" {{\n material \"m{n}\"\n sphereGeometry {{\n radius 1\n widthSegments 32\n heightSegments 24\n }}\n}}\n"));
        }
        files.push((format!("{id}/model.ogt"), model.into_bytes()));
        let files: Vec<_> = files.iter().map(|(n, b)| (n.as_str(), b.clone())).collect();
        cook(&parcel_root, &files);
        for run in 0..5 {
            set_hooks(Rc::new(Profile));
            og_host_begin_bundle();
            let game = og_game_game(Some(r#"{"seed":1}"#.into()));
            let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
                .as_u64()
                .unwrap() as u32;
            let held = Rc::new(Cell::new(Held::default()));
            answer(set_render_backend(
                engine,
                Some(Box::new(Recorder(held.clone()))),
            ));
            let g = Game { engine, held };
            g.place("world.ogt");
            g.open(&parcel_root);
            g.place(&format!(
                "{id}/{}",
                if control { "content.ogt" } else { "place.ogt" }
            ));
            g.frame();
            if control {
                for _ in 0..1000 {
                    g.frame();
                    if og_loading_idle(engine) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                assert!(og_loading_idle(engine));
            }
            trace_set(engine, true);
            g.walk(100);
            let mut max: f64 = 0.0;
            DECODE.with(|d| d.set(0.0));
            for _ in 0..600 {
                let start = Instant::now();
                g.frame();
                max = max.max(start.elapsed().as_secs_f64() * 1000.0);
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            assert!(control || g.loaded(id));
            assert_eq!(g.held.get().textures, u64::from(count * 1024 * 1024 * 4));
            let trace: Value = serde_json::from_str(&trace_take(engine, false)).unwrap();
            let worker_ms: f64 = trace["events"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|s| s["cat"] == "texture transcode")
                .map(|s| s["dur"].as_f64().unwrap() / 1000.0)
                .sum();
            trace_set(engine, false);
            println!(
                "CELL_TIMING count={count} grouped={grouped} control={control} run={run} max_ms={max:.3} decode_ms={:.3} worker_ms={worker_ms:.3} textures={}",
                DECODE.with(Cell::get),
                g.held.get().textures
            );
            assert!(og_engine_destroy(engine));
        }
    }
    std::fs::remove_dir_all(base).unwrap();
}

/// Answers a parcel bundle's manifest `Fetch::Pending` for its first
/// `waits` reads, as the web host does until the network brings it.
struct LateManifest {
    waits: Cell<u32>,
}
impl Hooks for LateManifest {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        if path.contains("/late/") && path.ends_with(".cook.ogt") && self.waits.get() > 0 {
            self.waits.set(self.waits.get() - 1);
            return Fetch::Pending;
        }
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// A run that opens a bundle and places its scene once the open is ready,
/// its manifest arriving after `waits` reads: each tick's world hash and
/// og.load.completions. With `replay`, that journal is handed back first.
fn late_open(base: &Path, waits: u32, replay: Option<&Value>) -> (Vec<Value>, Value) {
    let root = base.join("game");
    set_hooks(Rc::new(LateManifest {
        waits: Cell::new(waits),
    }));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
        .as_u64()
        .unwrap() as u32;
    let g = Game {
        engine,
        held: Rc::default(),
    };
    if let Some(journal) = replay {
        answer(og_load_replay_completions(engine, journal.to_string()));
    }
    g.place("world.ogt");
    let mut hashes = Vec::new();
    let mut bundle = None;
    let mut placed = false;
    for tick in 0..12 {
        if tick == 3 {
            bundle = Some(g.open(&base.join("late")));
        }
        // Game code acting on the open's status, as a kit system does.
        if let Some(h) = bundle
            && !placed
            && og_load_status(engine, h) == "ready"
        {
            g.place("late/place.ogt");
            placed = true;
        }
        g.frame();
        // Not the probe's dump: a probe call runs a tick boundary of its own.
        hashes.push(json!(og_net_hash(engine)));
    }
    assert!(placed, "the bundle never opened");
    let journal = og_load_completions(engine);
    assert!(og_engine_destroy(engine));
    (hashes, journal)
}

/// On the web a runtime bundle opens when its manifest arrives, a tick the
/// network picks. That tick is journaled, and a replay whose host reads the
/// manifest at once (natively) still opens it there, so a world that acts
/// on the open replays tick for tick.
#[test]
fn a_runtime_bundle_opens_at_its_journaled_tick_in_a_replay() {
    let _lock = TEST_LOCK.lock().unwrap();
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "og-runtime-bundle-open-journal-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&base);
    cook(&base.join("game"), &[("world.ogt", b"type scene\nversion 1\n---\nentity \"player\" {\n  Transform {\n    position 0 0 0\n  }\n  StreamAnchor {\n    reach 0\n  }\n}\n".to_vec())]);
    parcel(&base.join("late"), "late", "late", 100, false);

    let (web, journal) = late_open(&base, 3, None);
    let bundles: Vec<&Value> = journal
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["kind"] == "bundle")
        .collect();
    assert_eq!(bundles.len(), 1, "{journal}");
    assert_eq!(bundles[0]["request"], 0, "{journal}");
    assert_eq!(bundles[0]["ok"], true, "{journal}");
    let opened = bundles[0]["tick"].as_f64().unwrap();
    assert!(
        opened > 3.0,
        "opened at {opened}, pending since tick 3: {journal}"
    );

    let (native, native_journal) = late_open(&base, 0, None);
    assert_ne!(
        web, native,
        "the late open must change the world for this test to see anything"
    );
    assert_eq!(
        native_journal[0]["tick"].as_f64(),
        Some(3.0),
        "{native_journal}"
    );

    let (replayed, replay_journal) = late_open(&base, 0, Some(&journal));
    assert_eq!(
        replayed, web,
        "the replay must open the bundle at its recorded tick"
    );
    assert_eq!(replay_journal, journal);
    std::fs::remove_dir_all(base).unwrap();
}
