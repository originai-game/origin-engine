//! What a streaming group loaded without a stand-in (an environment map, a
//! texture array, a palette, a look) is let go when the group unloads and
//! comes back when it is resident again; one the frame still uses is kept,
//! with a warning naming it, until the frame stops using it.
//!
//! A project with one manual group "zone" (`env/studio.ogt`,
//! `tex/frames.ogt`, `palettes/dusk.hex` and `looks/dusk.ogt`, which
//! quantises to that palette) is cooked once; a renderer that draws nothing
//! records what each frame names and which of the handles it has seen are
//! still in the scene.
//!
//! Ways this could fail: the unload leaves the map, the array or the
//! palette in the renderer (the live counts stay 1), or frees one the
//! frame still lights, quantises or draws with (og-render would refuse
//! the frame, and the counts drop under it); the kept ones are never let
//! go once unused; nothing warns, or the warning does not name the asset;
//! a released handle still draws (og.render.sprite of the array, applying
//! the look) instead of being refused with its group; the reload does not
//! bring them back under the same game handles. Broken once by freeing
//! without the in-use check: the second test's frame after the unload
//! names an array the scene no longer holds (og-render refuses such a
//! frame: "texture array 0 was removed").

use og_render::{EnvironmentMapHandle, PaletteHandle, RenderScene, TextureArrayHandle};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_engine_take_log,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, og_load_environment,
    og_load_look, og_load_texture_array, og_render_apply_look, og_render_sprite, og_stream_release,
    og_stream_request, og_stream_status, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const SCENE: &str = r#"type scene
version 1
---
name "Zone"
stream "zone" {
  asset "env/studio.ogt"
  asset "tex/frames.ogt"
  asset "palettes/dusk.hex"
  asset "looks/dusk.ogt"
}
"#;

/// The project, cooked once per test process (nextest runs each test in
/// its own, so the directory carries the process id: two tests rebuilding
/// one directory at once raced, and one read a file the other had just
/// removed).
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-stream-release-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        let read = |path: &str| std::fs::read(repo().join(path)).unwrap();
        write("env/studio.hdr", &read("crates/og-tests/assets/env/studio.hdr"));
        write(
            "env/studio.ogt",
            b"type environment\nversion 1\n---\nsource \"studio.hdr\"\nsize 16\nsamples 16\n",
        );
        let png = read("templates/fx-showcase/assets/textures/toon.png");
        write("tex/f0.png", &png);
        write("tex/f1.png", &png);
        write(
            "tex/frames.ogt",
            b"type texture-array\nversion 1\n---\nkind pixel\nlayer \"f0.png\"\nlayer \"f1.png\"\n",
        );
        write(
            "palettes/dusk.hex",
            &read("packages/engine/tests/fixtures/look/assets/palettes/dusk.hex"),
        );
        write(
            "looks/dusk.ogt",
            b"type look\nversion 1\n---\nname \"Dusk\"\nStylise {\n  posterize 6\n  palette \"palettes/dusk.hex\"\n}\n",
        );
        write(
            "looks/plain.ogt",
            b"type look\nversion 1\n---\nname \"Plain\"\nStylise {\n}\n",
        );
        write("scenes/zone.ogt", SCENE.as_bytes());
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

/// Reads the cooked files from disk.
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

/// What the frames named, every handle seen, and how many of those the
/// last frame's scene still holds.
#[derive(Default)]
struct Seen {
    env: Option<EnvironmentMapHandle>,
    palette: Option<PaletteHandle>,
    arrays: usize,
    every_env: Vec<EnvironmentMapHandle>,
    every_palette: Vec<PaletteHandle>,
    every_array: Vec<TextureArrayHandle>,
    live: [usize; 3],
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
        let mut s = self.0.borrow_mut();
        s.env = scene.environment_map.map(|m| m.map);
        s.palette = scene.stylise.quantize.and_then(|q| q.palette);
        s.arrays = scene.sprite_arrays.len();
        if let Some(h) = s.env.filter(|h| !s.every_env.contains(h)) {
            s.every_env.push(h);
        }
        if let Some(h) = s.palette.filter(|h| !s.every_palette.contains(h)) {
            s.every_palette.push(h);
        }
        for a in &scene.sprite_arrays {
            if !s.every_array.contains(&a.array) {
                s.every_array.push(a.array);
            }
        }
        // Every reference is live (og-render refuses a frame naming a removed one).
        assert!(s.env.is_none_or(|h| scene.contains_environment_map(h)));
        assert!(s.palette.is_none_or(|h| scene.contains_palette(h)));
        assert!(
            scene
                .sprite_arrays
                .iter()
                .all(|a| scene.contains_texture_array(a.array))
        );
        s.live = [
            s.every_env
                .iter()
                .filter(|h| scene.contains_environment_map(**h))
                .count(),
            s.every_palette
                .iter()
                .filter(|h| scene.contains_palette(**h))
                .count(),
            s.every_array
                .iter()
                .filter(|h| scene.contains_texture_array(**h))
                .count(),
        ];
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

fn answer(text: String) -> Result<Value, String> {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    if v["ok"] == json!(true) {
        Ok(v["value"].clone())
    } else {
        Err(v["reason"].as_str().unwrap_or(&text).to_string())
    }
}

fn value(text: String) -> Value {
    answer(text).unwrap_or_else(|e| panic!("{e}"))
}

struct Game {
    engine: u32,
    seen: Rc<RefCell<Seen>>,
    env: f64,
    array: f64,
    dusk: f64,
    plain: f64,
}

impl Game {
    fn frame(&self) {
        value(og_engine_frame(self.engine, 1000.0 / 60.0));
    }

    fn state(&self) -> String {
        let v = value(og_stream_status(self.engine, "zone".into()));
        v["state"].as_str().unwrap_or_default().to_string()
    }

    /// Frames until the group is resident (and one more for what it holds).
    fn resident(&self) {
        value(og_stream_request(self.engine, "zone".into()));
        for _ in 0..20 {
            self.frame();
            if self.state() == "resident" {
                self.frame();
                return;
            }
        }
        panic!("zone never became resident: {}", self.state());
    }

    fn sprite(&self) -> Result<Value, String> {
        let desc = json!({ "array": self.array, "arrayLayer": 1, "width": 2, "height": 2 });
        answer(og_render_sprite(self.engine, Some(desc.to_string())))
    }

    fn environment(&self, map: f64) {
        let v = json!({ "map": map }).to_string();
        value(og_engine_write_resource(
            self.engine,
            "og.Environment".into(),
            v,
        ));
    }

    /// The map lights the frame, the dusk look quantises it to the
    /// palette, and a sprite draws from the array.
    fn use_all(&self) {
        self.environment(self.env);
        value(og_render_apply_look(self.engine, self.dusk));
        self.sprite().expect("a sprite of the array");
        self.frame();
        self.sprite().expect("a sprite of the array");
        self.frame();
    }

    /// Nothing the group holds is used any more.
    fn use_none(&self) {
        self.environment(0.0);
        value(og_render_apply_look(self.engine, self.plain));
        self.frame();
        self.frame();
    }

    fn live(&self) -> [usize; 3] {
        self.seen.borrow().live
    }

    fn warnings(&self) -> Vec<String> {
        let log: Value = serde_json::from_str(&og_engine_take_log(self.engine)).unwrap();
        log.as_array()
            .unwrap()
            .iter()
            .filter(|e| e["warning"] == json!(true))
            .map(|e| e["text"].as_str().unwrap_or_default().to_string())
            .collect()
    }
}

/// An engine over the cooked project with the group resident and its
/// assets loaded.
fn game() -> Game {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let mut g = Game {
        engine,
        seen,
        env: 0.0,
        array: 0.0,
        dusk: 0.0,
        plain: value(og_load_look(engine, "looks/plain.ogt".into()))
            .as_f64()
            .unwrap(),
    };
    g.frame();
    assert_eq!(g.state(), "unloaded");
    let e = answer(og_load_environment(engine, "env/studio.ogt".into()));
    let pending = e.expect("a grouped map's handle is pending until resident");
    let a = answer(og_load_texture_array(engine, "tex/frames.ogt".into()));
    assert!(
        a.as_ref().is_err_and(|e| e.contains("zone")),
        "a grouped array loads once resident: {a:?}"
    );
    g.resident();
    g.env = pending.as_f64().unwrap();
    g.array = value(og_load_texture_array(engine, "tex/frames.ogt".into()))["array"]
        .as_f64()
        .unwrap();
    g.dusk = value(og_load_look(engine, "looks/dusk.ogt".into()))
        .as_f64()
        .unwrap();
    g.warnings();
    g
}

#[test]
fn an_unloaded_group_lets_go_of_its_maps_arrays_palettes_and_looks_and_a_reload_brings_them_back() {
    let g = game();
    g.use_all();
    {
        let s = g.seen.borrow();
        assert!(s.env.is_some() && s.palette.is_some() && s.arrays == 1);
    }
    assert_eq!(
        g.live(),
        [1, 1, 1],
        "map, palette and array in the renderer"
    );
    g.use_none();
    assert_eq!(g.live(), [1, 1, 1], "still loaded while resident");

    value(og_stream_release(g.engine, "zone".into()));
    g.frame();
    g.frame();
    assert_eq!(g.state(), "unloaded");
    assert_eq!(g.live(), [0, 0, 0], "the unload freed all three");
    let kept: Vec<String> = g
        .warnings()
        .into_iter()
        .filter(|w| w.contains("is kept"))
        .collect();
    assert!(kept.is_empty(), "nothing was in use: {kept:?}");
    // The game's handles stay, refused until the group is back.
    let e = g.sprite().expect_err("an array whose group unloaded");
    assert!(
        e.contains("tex/frames.ogt") && e.contains("not resident"),
        "{e}"
    );
    let e =
        answer(og_render_apply_look(g.engine, g.dusk)).expect_err("a look whose group unloaded");
    assert!(
        e.contains("looks/dusk.ogt") && e.contains("not resident"),
        "{e}"
    );

    // Resident again: the same handles draw again, from new renderer slots.
    g.resident();
    g.use_all();
    let s = g.seen.borrow();
    assert_eq!(s.live, [1, 1, 1], "back in the renderer");
    assert_eq!(s.every_env.len(), 2, "a new renderer handle for the map");
    assert_eq!(s.every_palette.len(), 2);
    assert_eq!(s.every_array.len(), 2);
    assert_eq!(s.arrays, 1);
}

#[test]
fn what_the_frame_still_uses_is_kept_with_a_warning_until_it_is_not() {
    let g = game();
    g.use_all();
    // The group goes while the frame lights, quantises and draws with it.
    value(og_stream_release(g.engine, "zone".into()));
    g.sprite().expect("queued before the unload");
    g.frame();
    g.sprite()
        .expect("the array is kept while a sprite draws from it");
    g.frame();
    assert_eq!(g.state(), "unloaded");
    assert_eq!(g.live(), [1, 1, 1], "nothing freed under the frame");
    let warnings = g.warnings();
    for id in [
        "env/studio.ogt",
        "tex/frames.ogt",
        "palettes/dusk.hex",
        "looks/dusk.ogt",
    ] {
        assert_eq!(
            warnings
                .iter()
                .filter(|w| w.contains("'zone' unloaded") && w.contains(&format!("{id} is kept")))
                .count(),
            1,
            "one warning names {id}: {warnings:?}"
        );
    }
    // Once the frame stops using them they go, without another warning.
    g.use_none();
    g.frame();
    assert_eq!(g.live(), [0, 0, 0], "let go once unused");
    assert!(
        !g.warnings().iter().any(|w| w.contains("is kept")),
        "warned once"
    );
    assert!(g.sprite().is_err(), "the array is released");
}
