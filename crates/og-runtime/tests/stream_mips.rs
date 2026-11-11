//! Texture mip streaming (G3-3 batch 3): a texture cooked with
//! `texture { stream true }` loads as its mip tail, takes its whole chain
//! once it is big on screen, goes back to its tail once it has been small
//! for a while, stays on its tail when the mip budget cannot hold it, and
//! in the probe's resident mode is whole whatever its size on screen. Its
//! reported size is the whole chain's throughout.
//!
//! A 1024² colour texture (tail 256², eleven levels) on a 2 m plane at the
//! world's zero, under a camera at z 3 (it covers about 340 of the 180-pixel
//! view's pixels across: more than its tail's 256) or z 400 (about 2).
//! A renderer that draws nothing records the width of what the plane's
//! texture holds at each frame.
//!
//! Ways this could fail: the tail is never swapped for the whole chain (the
//! footprint is measured wrong, the read never lands, the swap misses the
//! handle); the whole chain stays after the texture has been small past the
//! keep window, or goes at once (it would be read again on every turn of the
//! camera); the budget is ignored; resident mode leaves a small texture on
//! its tail; texture_size reports the tail's 256 while it holds it (a sprite
//! sized from it would change size when the levels arrive).

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_commands, og_engine_write_resource, og_game_game, og_host_begin_bundle,
    og_load_texture, og_render_plane_geometry, og_render_standard_material, og_world_insert,
    og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-stream-mips-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("assets/tex");
        std::fs::create_dir_all(&dir).unwrap();
        let rgba: Vec<u8> = (0..1024u32 * 1024)
            .flat_map(|i| [(i % 1024 / 4) as u8, (i / 1024 / 4) as u8, 128, 255])
            .collect();
        image::save_buffer(
            dir.join("big.png"),
            &rgba,
            1024,
            1024,
            image::ColorType::Rgba8,
        )
        .unwrap();
        std::fs::write(
            dir.join("big.png.ogt"),
            "type import\nversion 1\n---\ntexture {\n  stream true\n}\n",
        )
        .unwrap();
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

/// Per frame, what the drawn plane's texture holds (level 0's width) and
/// the size the scene reports for it.
#[derive(Default)]
struct Seen {
    frames: Vec<(u32, (u32, u32))>,
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
        let t = scene
            .objects
            .instances
            .iter()
            .flat_map(|i| scene.material_textures(i.material))
            .next()
            .ok_or("the plane draws its texture")?;
        let held = scene.texture(t).map_or(0, |d| d.width);
        let size = scene.texture_size(t).unwrap();
        self.0.borrow_mut().frames.push((held, size));
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

struct Game {
    engine: u32,
    seen: Rc<RefCell<Seen>>,
}

impl Game {
    fn new() -> Game {
        let root = project();
        set_hooks(Rc::new(Disk));
        og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        let options = json!({ "root": root.to_str().unwrap() }).to_string();
        let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
        let seen = Rc::new(RefCell::new(Seen::default()));
        set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
        let tex = value(og_runtime::host_impl::answer(og_load_texture(
            engine,
            "tex/big.png".into(),
        )));
        let material = value(og_runtime::host_impl::answer(og_render_standard_material(
            engine,
            Some(json!({ "map": tex }).to_string()),
        )));
        let geometry = value(og_runtime::host_impl::answer(og_render_plane_geometry(
            engine,
            Some(json!({ "width": 2, "height": 2 }).to_string()),
        )));
        let e = value(og_runtime::host_impl::answer(og_world_spawn(engine, None)))
            .as_f64()
            .unwrap();
        value(og_runtime::host_impl::answer(og_world_insert(
            engine,
            e,
            "og.Mesh".into(),
            Some(json!({ "geometry": geometry, "material": material }).to_string()),
        )));
        let g = Game { engine, seen };
        g.camera(3.0);
        g
    }

    fn camera(&self, z: f64) {
        value(og_engine_write_resource(
            self.engine,
            "og.Camera".into(),
            json!({ "position": { "x": 0, "y": 0, "z": z } }).to_string(),
        ));
    }

    fn frames(&self, n: usize) -> u32 {
        for _ in 0..n {
            value(og_engine_frame(self.engine, 1000.0 / 60.0));
        }
        self.held()
    }

    fn held(&self) -> u32 {
        self.seen.borrow().frames.last().unwrap().0
    }

    fn probe(&self, command: &str) {
        let s: Value = serde_json::from_str(&og_engine_probe_commands(
            self.engine,
            format!("0:{command}"),
        ))
        .unwrap();
        assert_eq!(s["scheduled"], json!(1), "{s}");
    }
}

#[test]
fn a_streamed_texture_is_whole_while_big_on_screen() {
    let g = Game::new();
    assert_eq!(g.frames(1), 256, "it loads as its tail");
    assert_eq!(g.frames(3), 1024, "big on screen: its whole chain");
    // Small on screen: whole for the keep window, then its tail again.
    g.camera(400.0);
    assert_eq!(g.frames(60), 1024, "kept while it may come back");
    assert_eq!(g.frames(120), 256, "its tail once small past the window");
    g.camera(3.0);
    assert_eq!(g.frames(3), 1024, "and whole again when big");
    assert!(
        g.seen.borrow().frames.iter().all(|f| f.1 == (1024, 1024)),
        "texture_size is the whole chain's on every frame"
    );
}

#[test]
fn the_budget_holds_it_on_its_tail_and_resident_mode_makes_it_whole() {
    let g = Game::new();
    // 1 MiB: less than the whole chain's 1.33 MiB × 4 in RGBA8.
    g.probe("mip_stream:stream:1");
    assert_eq!(g.frames(6), 256, "over the budget: its tail");
    g.camera(400.0);
    g.probe("mip_stream:resident");
    assert_eq!(g.frames(2), 1024, "resident: whole however small");
    g.probe("mip_stream:stream");
    assert_eq!(g.frames(1), 256, "streamed again, within 1 MiB: its tail");
}
