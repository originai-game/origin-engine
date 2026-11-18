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
        self.0.set(Held { textures, meshes });
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
