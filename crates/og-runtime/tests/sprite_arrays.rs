//! Texture arrays through the runtime's sprite paths: an og.Sprite naming
//! an array (and a layer), an og.Flipbook of an `array true` sheet (its
//! frame becoming the layer), a screen sprite of an array (the overlay),
//! and a sprite-mode particle emitter naming an array.
//!
//! A project with a two-layer texture array, a five-frame `array true`
//! flipbook (the topdown template's 80 x 16 slime strip, a four-frame clip
//! at 10 fps) and an
//! emitter drawing from the array is cooked once; a renderer that draws
//! nothing records what each frame's scene names.
//!
//! Ways this could fail: og.Sprite ignores its array (no array entry, or
//! the wrong layer); the flipbook draws its texture's regions instead of
//! the array's layers, or a layer that does not follow og.Flipbook.frame;
//! a screen sprite of an array is refused or lands in the world; the
//! emitter's array is lost on its way to the renderer. Broken once by
//! drawing og.Sprite from its texture whatever its array: the first test
//! sees no array entry.

use og_render::{RenderScene, SpriteArray, TextureArrayHandle};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_emitter, og_load_flipbook, og_load_texture_array,
    og_render_sprite, og_world_get, og_world_insert, og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The project, cooked once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-sprite-arrays-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        let png = std::fs::read(repo().join("templates/fx-showcase/assets/textures/toon.png"))
            .unwrap();
        write("tex/f0.png", &png);
        write("tex/f1.png", &png);
        write(
            "tex/frames.ogt",
            b"type texture-array\nversion 1\n---\nkind pixel\nlayer \"f0.png\"\nlayer \"f1.png\"\n",
        );
        let slime = std::fs::read(repo().join("templates/archetypes/topdown/assets/sprites/slime.png"))
            .unwrap();
        write("sprites/strip.png", &slime);
        write(
            "sprites/run.ogt",
            b"type flipbook\nversion 1\n---\ntexture \"strip.png\"\narray true\ngrid 16 16 5 1\nclip run {\n  frames 0 1 2 3\n  fps 10\n}\n",
        );
        write(
            "fx/motes.ogt",
            b"type particles\nversion 1\n---\nrender sprite 1\nrate 30\narray \"tex/frames.ogt\" random\nround true\n",
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

/// What the last frame's scene named.
#[derive(Default)]
struct Seen {
    /// (array, layer) of every array sprite, world and overlay.
    world: Vec<(TextureArrayHandle, u32)>,
    overlay: Vec<(TextureArrayHandle, u32)>,
    /// The arrays of the frame's particle systems' emitters.
    emitters: Vec<Option<TextureArrayHandle>>,
    textured: usize,
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
        let pairs = |list: &[SpriteArray]| list.iter().map(|a| (a.array, a.layer)).collect();
        let mut s = self.0.borrow_mut();
        s.world = pairs(&scene.sprite_arrays);
        s.overlay = scene
            .overlay
            .as_ref()
            .map_or_else(Vec::new, |o| pairs(&o.sprite_arrays));
        s.emitters = scene
            .particles
            .iter()
            .map(|p| scene.emitter(p.emitter).and_then(|e| e.array))
            .collect();
        s.textured = scene.sprites.iter().filter(|s| s.texture.is_some()).count();
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

fn engine() -> (u32, Rc<RefCell<Seen>>) {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    value(og_engine_frame(engine, 1000.0 / 60.0));
    (engine, seen)
}

fn frame(engine: u32) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

fn spawn(engine: u32, components: &[(&str, Value)]) -> f64 {
    let e = value(og_world_spawn(engine, Some("{}".into())))
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
fn an_og_sprite_draws_the_layer_of_its_array() {
    let (engine, seen) = engine();
    let array = value(og_load_texture_array(engine, "tex/frames.ogt".into()))["array"]
        .as_f64()
        .unwrap();
    spawn(
        engine,
        &[(
            "og.Sprite",
            json!({ "array": array, "arrayLayer": 1, "width": 2, "height": 2 }),
        )],
    );
    // Past the last layer: the last.
    spawn(
        engine,
        &[("og.Sprite", json!({ "array": array, "arrayLayer": 9 }))],
    );
    frame(engine);
    let s = seen.borrow();
    assert_eq!(s.world.len(), 2, "both sprites draw from the array");
    assert_eq!(s.world[0].1, 1);
    assert_eq!(s.world[1].1, 1, "clamped to the last layer");
    assert_eq!(s.world[0].0, s.world[1].0, "one array, one run");
    assert_eq!(s.textured, 0);
}

#[test]
fn an_array_flipbook_draws_its_frame_as_the_layer() {
    let (engine, seen) = engine();
    let info = value(og_load_flipbook(engine, "sprites/run.ogt".into()));
    assert_eq!(info["texture"], json!(0.0), "no texture: {info}");
    assert!(info["array"].as_f64().unwrap() > 0.0, "{info}");
    assert_eq!(info["frames"], json!(5));
    let e = spawn(
        engine,
        &[
            ("og.Sprite", json!({})),
            ("og.Flipbook", json!({ "sheet": info["sheet"] })),
        ],
    );
    let mut layers = Vec::new();
    for _ in 0..20 {
        frame(engine);
        let fb = value(og_world_get(engine, e, "og.Flipbook".into()));
        let s = seen.borrow();
        assert_eq!(s.world.len(), 1, "the sprite draws from the sheet's array");
        assert_eq!(
            s.world[0].1,
            fb["frame"].as_u64().unwrap() as u32,
            "the layer is the frame"
        );
        layers.push(s.world[0].1);
        assert_eq!(s.textured, 0);
    }
    layers.dedup();
    assert!(layers.len() >= 3, "the layer follows the clip: {layers:?}");
}

#[test]
fn screen_sprites_and_emitters_draw_from_arrays() {
    let (engine, seen) = engine();
    let array = value(og_load_texture_array(engine, "tex/frames.ogt".into()))["array"]
        .as_f64()
        .unwrap();
    let desc =
        json!({ "array": array, "arrayLayer": 1, "screen": true, "width": 64, "height": 64 });
    value(og_render_sprite(engine, Some(desc.to_string())));
    let emitter = value(og_load_emitter(engine, "fx/motes.ogt".into()));
    spawn(
        engine,
        &[("og.ParticleSystem", json!({ "emitter": emitter }))],
    );
    frame(engine);
    let s = seen.borrow();
    assert_eq!(s.overlay.len(), 1, "the screen sprite is the overlay's");
    assert_eq!(s.overlay[0].1, 1);
    assert!(s.world.is_empty());
    assert_eq!(s.emitters.len(), 1);
    assert_eq!(
        s.emitters[0],
        Some(s.overlay[0].0),
        "the emitter draws from the array"
    );
}
