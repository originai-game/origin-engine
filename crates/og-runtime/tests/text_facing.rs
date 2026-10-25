//! World text as a billboard (`TextDesc.facing`, `FloatingTextDesc.facing`):
//! the runtime hands the renderer each faced world text at its index
//! (`RenderScene::text_facing`) with the point it turns about, and a
//! floating text's facing survives a snapshot.
//!
//! Ways this could fail: the facing lands on another text's index; an
//! aligned text keeps the runtime's shift along the world x axis (the
//! renderer then turns it about its left end, off its point); a screen
//! text faces; a snapshot drops the facing, or a snapshot without a faced
//! text changes its bytes.

use og_render::{RenderScene, SpriteFacing, TextFacing};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_font, og_render_floating_text, og_render_text, og_world_restore,
    og_world_snapshot, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A project with the kit's neon display face, cooked once per process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-text-facing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("assets/fonts");
        std::fs::create_dir_all(&dir).unwrap();
        let style = repo().join("templates/styles/neon");
        for f in [
            "ChakraPetch-BoldItalic.ttf",
            "ChakraPetch-BoldItalic.ttf.ogt",
        ] {
            std::fs::copy(style.join(f), dir.join(f)).unwrap();
        }
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

/// The last frame's world texts as (text, x of its position, its facing).
#[derive(Default)]
struct Seen {
    texts: Vec<(String, f32, Option<TextFacing>)>,
    screen: usize,
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
        let mut seen = self.0.borrow_mut();
        seen.texts = scene
            .texts
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let facing = scene
                    .text_facing
                    .iter()
                    .rev()
                    .find(|f| f.text == i as u32)
                    .copied();
                (t.text.clone(), t.position.x, facing)
            })
            .collect();
        seen.screen = scene.overlay.as_ref().map_or(0, |o| o.texts.len());
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

#[test]
fn faced_world_text_reaches_the_renderer_with_its_pivot() {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let font = value(og_load_font(
        engine,
        "fonts/ChakraPetch-BoldItalic.ttf".into(),
    ));
    value(og_engine_frame(engine, 1000.0 / 60.0));

    let text = |t: &str, extra: Value| {
        let mut d = json!({ "font": font, "text": t, "position": { "x": 5, "y": 1, "z": 0 } });
        d.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        value(og_render_text(engine, Some(d.to_string())));
    };
    text("flat", json!({ "align": "center" }));
    text("name", json!({ "align": "center", "facing": "cameraY" }));
    text("sign", json!({ "facing": "camera" }));
    text("hud", json!({ "facing": "camera", "screen": true }));
    value(og_render_floating_text(
        engine,
        Some(
            json!({ "font": font, "text": "-12", "position": { "x": -3, "y": 0, "z": 0 },
                    "spread": 0, "facing": "camera" })
            .to_string(),
        ),
    ));
    value(og_engine_frame(engine, 0.0));
    let seen_now = |name: &str| {
        seen.borrow()
            .texts
            .iter()
            .find(|t| t.0 == name)
            .cloned()
            .unwrap_or_else(|| panic!("{name} drawn"))
    };
    let flat = seen_now("flat");
    assert!(
        flat.1 < 5.0 && flat.2.is_none(),
        "centred by a shift: {flat:?}"
    );
    let faced = |facing, align| {
        Some(TextFacing {
            text: 0,
            facing,
            align,
        })
    };
    let pivot =
        |t: (String, f32, Option<TextFacing>)| (t.1, t.2.map(|f| TextFacing { text: 0, ..f }));
    assert_eq!(
        pivot(seen_now("name")),
        (5.0, faced(SpriteFacing::CameraY, 0.5)),
        "turned about its point, the renderer centring it"
    );
    assert_eq!(
        pivot(seen_now("sign")),
        (5.0, faced(SpriteFacing::Camera, 0.0))
    );
    assert_eq!(
        pivot(seen_now("-12")),
        (-3.0, faced(SpriteFacing::Camera, 0.5)),
        "a floating text centred on its point"
    );
    assert_eq!(seen.borrow().screen, 1, "the screen text is the overlay's");

    // The floating text's facing is world state: a snapshot keeps it.
    let save = value(og_world_snapshot(engine));
    value(og_world_restore(
        engine,
        save.as_str().unwrap().to_string(),
        None,
    ));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(
        seen_now("-12").2.map(|f| f.facing),
        Some(SpriteFacing::Camera),
        "restored facing"
    );
}
