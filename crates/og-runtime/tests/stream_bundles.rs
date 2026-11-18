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
