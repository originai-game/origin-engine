//! Drop shadows off the plane y = 0 (`SpriteDesc.dropShadowHeight`, the
//! og.Terrain under a sprite): the runtime hands the renderer each
//! shadow's ground.
//!
//! A project with a 32 m terrain rising 0.25 m per metre along x (an
//! 8-bit ramp from 0 to 8 m) is cooked once; a renderer that draws nothing
//! records each frame's sprite shadows.
//!
//! Ways this could fail: a shadow over the terrain keeps the plane y = 0
//! (its ground's height 0); the fitted plane is level on the ramp (no
//! slope handed on); an explicit dropShadowHeight above the terrain is
//! overridden by the lower terrain; a sprite off the terrain gets the
//! terrain's edge; og.render.sprite drops its dropShadowHeight.
//!
//! The same project has a generated island (`generate { sea 1.5 }`):
//! og.terrain.load answers its sea level, and none for the ramp.
//!
//! The 2D profile has no terrain (tests/profile2d.rs).
#![cfg(not(feature = "profile2d"))]

use og_render::{RenderScene, ShadowGround, Sprite};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_render_sprite, og_terrain_load, og_world_insert, og_world_spawn,
    set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

/// The ramp's height at world x (the terrain centred on 0, 32 m wide).
fn ramp(x: f32) -> f32 {
    (x + 16.0) / 32.0 * 8.0
}

/// The project, cooked once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-drop-shadows-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        let (w, h) = (33u32, 33u32);
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
        write("terrain/ramp.png", &png);
        write(
            "terrain/ramp.ogt",
            b"type terrain\nversion 1\n---\nheightmap \"ramp.png\"\nheight 0 8\nsize 32 32\ncollision false\n",
        );
        write(
            "terrain/isle.ogt",
            b"type terrain\nversion 1\n---\nheight -2 6\nsize 64 64\ncollision false\ngenerate {\n  samples 33 33\n  mask island 0.5\n  sea 1.5\n}\n",
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

/// The last frame's world sprites and their shadows' grounds.
#[derive(Default)]
struct Seen {
    shadows: Vec<(Sprite, ShadowGround)>,
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
        self.0.borrow_mut().shadows = scene
            .sprite_shadows
            .iter()
            .map(|s| (scene.sprites[s.sprite as usize], s.ground))
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

fn frame(engine: u32) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

fn spawn(engine: u32, at: [f32; 3], components: &[(&str, Value)]) {
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
}

#[test]
fn drop_shadows_lie_on_the_terrain_or_their_height() {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    frame(engine);
    let info = value(og_terrain_load(engine, "terrain/ramp.ogt".into()));
    assert_eq!(info["seaLevel"], Value::Null, "a heightmap has no sea");
    let isle = value(og_terrain_load(engine, "terrain/isle.ogt".into()));
    assert_eq!(isle["seaLevel"], json!(1.5), "{isle}");
    assert_eq!(
        (isle["minHeight"].as_f64(), isle["columns"].as_u64()),
        (Some(-2.0), Some(33))
    );
    spawn(
        engine,
        [0.0, 0.0, 0.0],
        &[("og.Terrain", json!({ "data": info["data"] }))],
    );
    let sprite = |extra: Value| {
        let mut v =
            json!({ "width": 1, "height": 2, "anchor": { "x": 0.5, "y": 0 }, "dropShadow": 0.5 });
        v.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        v
    };
    // On the ramp; on a deck above it; off the terrain.
    spawn(
        engine,
        [4.0, ramp(4.0), 0.0],
        &[("og.Sprite", sprite(json!({})))],
    );
    spawn(
        engine,
        [-8.0, 7.0, 0.0],
        &[("og.Sprite", sprite(json!({ "dropShadowHeight": 7.0 })))],
    );
    spawn(
        engine,
        [40.0, 0.0, 0.0],
        &[("og.Sprite", sprite(json!({})))],
    );
    // An immediate sprite over the ramp, its own height below the terrain.
    let desc = json!({ "position": { "x": -4.0, "y": ramp(-4.0), "z": 2.0 }, "width": 1, "height": 2, "dropShadow": 0.4, "dropShadowHeight": 1.0 });
    value(og_render_sprite(engine, Some(desc.to_string())));
    frame(engine);
    let s = seen.borrow();
    let ground_at = |x: f32| {
        s.shadows
            .iter()
            .find(|(sp, _)| (sp.position.x - x).abs() < 1e-3)
            .unwrap_or_else(|| panic!("a shadow at x {x}: {:?}", s.shadows.len()))
            .1
    };
    assert_eq!(s.shadows.len(), 4, "every shadow reaches the renderer");
    // The ramp at 4 m (8-bit heights: within a step of 8/255 m), rising
    // 0.25 per metre along x and level along z.
    let on = ground_at(4.0);
    assert!((on.height - ramp(4.0)).abs() < 0.05, "{on:?}");
    let [rx, rz] = on.slope();
    assert!((rx - 0.25).abs() < 0.03 && rz.abs() < 0.03, "{rx} {rz}");
    // The deck is above the ramp there (2 m): it stays.
    assert_eq!(ground_at(-8.0), ShadowGround::level(7.0));
    // Off the terrain: its own height, 0.
    assert_eq!(ground_at(40.0), ShadowGround::level(0.0));
    // The immediate sprite's 1 m is below the ramp's 3 m: the terrain.
    let queued = ground_at(-4.0);
    assert!((queued.height - ramp(-4.0)).abs() < 0.05, "{queued:?}");
}
