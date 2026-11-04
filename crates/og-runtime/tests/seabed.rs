//! og.Water.seabed (W1): a sea over an og.Terrain feels the depth.
//!
//! A project with a 256 m terrain rising 1 m per 16 along x (from 8 m under
//! the sea's level to 8 m over it, the waterline at x = 0) is cooked once; a
//! breezy og.Water over it names it as its seabed.
//!
//! Ways this could fail: og.physics.waterHeight (what og.Buoyancy rides)
//! keeps the open sea's metres-high swell in half a metre of water; the
//! renderer gets no seabed, or one other than the tick's (a floater then
//! rides waves that are not drawn); the seabed is read off the terrain's
//! wrong place.
#![cfg(not(feature = "profile2d"))]

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_physics_water_height, og_terrain_load, og_world_insert,
    og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

/// The project, cooked once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-seabed-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        let (w, h) = (129u32, 33u32);
        let rgba: Vec<u8> = (0..h)
            .flat_map(|_| {
                (0..w).flat_map(|x| {
                    let v = (x as f32 / (w - 1) as f32 * 255.0).round() as u8;
                    [v, v, v, 255]
                })
            })
            .collect();
        let png = og_runtime::png(&og_render::RgbaImage {
            width: w,
            height: h,
            pixels: rgba,
        });
        write("terrain/beach.png", &png);
        write(
            "terrain/beach.ogt",
            b"type terrain\nversion 1\n---\nheightmap \"beach.png\"\nheight -8 8\nsize 256 64\ncollision false\n",
        );
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

struct Disk;

impl Hooks for Disk {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// The last frame's seabed: its depth at the points asked, and the tick
/// its field was drawn for.
#[derive(Default)]
struct Seen {
    depths: Vec<Option<f32>>,
}

struct Recorder(Rc<RefCell<Seen>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.borrow_mut().depths = [-8.0f32, -64.0, 64.0]
            .iter()
            .map(|&x| {
                let s = scene.water.first()?.shore.as_ref()?;
                let p = glam::Vec2::new(x, 0.0) - s.origin + s.field.origin;
                Some(s.field.sample(p)?.depth)
            })
            .collect();
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn spawn(engine: u32, at: [f32; 3], components: &[(&str, Value)]) -> f64 {
    let desc = json!({ "position": { "x": at[0], "y": at[1], "z": at[2] } });
    let e = value(og_world_spawn(engine, Some(desc.to_string())))
        .as_f64()
        .unwrap();
    for (name, v) in components {
        value(og_world_insert(
            engine,
            e,
            name.to_string(),
            Some(v.to_string()),
        ));
    }
    e
}

#[test]
fn a_sea_over_its_seabed_shoals_for_the_tick_and_the_frame() {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let info = value(og_terrain_load(engine, "terrain/beach.ogt".into()));
    let terrain = spawn(
        engine,
        [0.0, 0.0, 0.0],
        &[("og.Terrain", json!({ "data": info["data"] }))],
    );
    let sea = |seabed: f64| json!({ "size": { "x": 2000, "y": 2000 }, "segments": 256, "wind": { "x": 12, "y": 0 }, "seabed": seabed });
    let water = spawn(engine, [0.0, 0.0, 0.0], &[("og.Water", sea(terrain))]);
    let open = spawn(engine, [0.0, 0.0, 0.0], &[("og.Water", sea(0.0))]);
    // Over a few seconds: the sea's crest to trough at half a metre of water
    // (x = -8) with its seabed and without.
    let mut spans = [(f64::MAX, f64::MIN); 2];
    for _ in 0..240 {
        value(og_engine_frame(engine, 1000.0 / 60.0));
        for (span, e) in spans.iter_mut().zip([water, open]) {
            let h = value(og_physics_water_height(engine, e, -8.0, 0.0))
                .as_f64()
                .unwrap();
            *span = (span.0.min(h), span.1.max(h));
        }
    }
    let [shoaled, unbounded] = spans.map(|(lo, hi)| hi - lo);
    println!("crest to trough in 0.5 m of water: {shoaled} m, the open sea's {unbounded} m");
    assert!(
        shoaled < 0.39 && shoaled < 0.25 * unbounded,
        "the shallows hold {shoaled} m of sea against the open sea's {unbounded} m"
    );
    // Off the beach, deep water: the swell is the open sea's.
    let at = |e: f64| {
        value(og_physics_water_height(engine, e, -600.0, 0.0))
            .as_f64()
            .unwrap()
    };
    assert_eq!(at(water), at(open));
    // The frame draws the tick's seabed, read off the terrain's place:
    // 0.5 m at x = -8, 4 m at -64, dry at 64 (the 8-bit heights within a
    // step of 16/255 m).
    let depths = seen.borrow().depths.clone();
    let want = [0.5, 4.0, -4.0];
    for (got, want) in depths.iter().zip(want) {
        let got = got.expect("a seabed drawn");
        assert!((got - want).abs() < 0.07, "{depths:?}");
    }
}
