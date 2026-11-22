//! World text as a billboard (`TextDesc.facing`, `FloatingTextDesc.facing`):
//! the runtime hands the renderer each faced world text at its index
//! (`RenderScene::text_facing`) with the point it turns about, and a
//! floating text's facing survives a snapshot.
//!
//! A floating text with no font of its own draws with the UI's: a line
//! mixing a style's Latin body face with its Chinese companion reaches the
//! renderer with every character's glyph, centred on its point (rhythm
//! archetype: a judgement word drawn whole in the body face lost every
//! Chinese character).
//!
//! Ways this could fail: the facing lands on another text's index; an
//! aligned text is shifted along the world x axis instead of laid out
//! about its point (the renderer then turns it about its left end, off
//! its point); a screen text faces; a snapshot drops the facing.

use og_render::{RenderScene, SpriteFacing, TextFacing};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_font, og_load_style, og_render_floating_text, og_render_text,
    og_ui_configure, og_world_restore, og_world_snapshot, set_hooks, set_render_backend,
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
        let style = repo().join("templates/styles/shared");
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

/// One world text as the renderer got it: its position's x, its facing,
/// its glyphs' count and their left and right ends (em from the point).
#[derive(Clone, Debug)]
struct Drawn {
    x: f32,
    facing: Option<TextFacing>,
    glyphs: usize,
    span: [f32; 2],
}

/// The last frame's world texts, in the order drawn, and the screen texts.
#[derive(Default)]
struct Seen {
    texts: Vec<Drawn>,
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
            .sprite
            .texts
            .iter()
            .enumerate()
            .map(|(i, t)| Drawn {
                x: t.position.x,
                facing: scene
                    .sprite
                    .text_facing
                    .iter()
                    .rev()
                    .find(|f| f.text == i as u32)
                    .copied(),
                glyphs: t.glyphs.len(),
                span: t.glyphs.iter().fold([f32::MAX, f32::MIN], |[l, r], g| {
                    [l.min(g.rect[0]), r.max(g.rect[2])]
                }),
            })
            .collect();
        seen.screen = scene.ui.overlay.as_ref().map_or(0, |o| o.texts.len());
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
    let font = value(og_runtime::host_impl::answer(og_load_font(
        engine,
        "fonts/ChakraPetch-BoldItalic.ttf".into(),
    )));
    value(og_engine_frame(engine, 1000.0 / 60.0));

    let text = |t: &str, extra: Value| {
        let mut d = json!({ "font": font, "text": t, "position": { "x": 5, "y": 1, "z": 0 } });
        d.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        value(og_runtime::host_impl::answer(og_render_text(
            engine,
            Some(d.to_string()),
        )));
    };
    text("flat", json!({ "align": "center" }));
    text("name", json!({ "align": "center", "facing": "cameraY" }));
    text("sign", json!({ "facing": "camera" }));
    text("hud", json!({ "facing": "camera", "screen": true }));
    value(og_runtime::host_impl::answer(og_render_floating_text(
        engine,
        Some(
            json!({ "font": font, "text": "-12", "position": { "x": -3, "y": 0, "z": 0 },
                    "spread": 0, "facing": "camera" })
            .to_string(),
        ),
    )));
    value(og_engine_frame(engine, 0.0));
    // The floating text first, then the queued ones in order (flat, name,
    // sign).
    let seen_now = |i: usize| seen.borrow().texts.get(i).cloned().expect("drawn");
    let centred = |d: &Drawn| (d.span[0] + d.span[1]).abs() < 0.1 * (d.span[1] - d.span[0]);
    let facing = |d: &Drawn| d.facing.map(|f| f.facing);
    let flat = seen_now(1);
    assert!(
        flat.x == 5.0 && flat.facing.is_none() && centred(&flat),
        "laid out about its point: {flat:?}"
    );
    let name = seen_now(2);
    assert!(
        name.x == 5.0 && facing(&name) == Some(SpriteFacing::CameraY) && centred(&name),
        "turned about its point, centred on it: {name:?}"
    );
    let sign = seen_now(3);
    assert!(
        sign.x == 5.0
            && facing(&sign) == Some(SpriteFacing::Camera)
            && !centred(&sign)
            // A field's quad reaches past its ink by the field's padding.
            && sign.span[0] > -0.2,
        "starting at its point: {sign:?}"
    );
    let float = seen_now(0);
    assert!(
        float.x == -3.0
            && facing(&float) == Some(SpriteFacing::Camera)
            && float.glyphs == 3
            && centred(&float),
        "a floating text centred on its point: {float:?}"
    );
    assert_eq!(seen.borrow().screen, 1, "the screen text is the overlay's");

    // The floating text's facing is world state: a snapshot keeps it.
    let save = value(og_runtime::host_impl::answer(og_world_snapshot(
        engine, None,
    )));
    value(og_runtime::host_impl::answer(og_world_restore(
        engine,
        save.as_str().unwrap().to_string(),
        None,
    )));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(
        facing(&seen_now(0)),
        Some(SpriteFacing::Camera),
        "restored facing"
    );
}

/// A project whose style has a Latin body face and a Chinese companion, and
/// a Chinese string table (the cook brings the companion only then).
fn mixed_project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-text-mixed-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        let neon = repo().join("templates/styles/shared");
        write(
            "styles/mix/Body.ttf",
            &std::fs::read(neon.join("ChakraPetch-BoldItalic.ttf")).unwrap(),
        );
        write(
            "styles/mix/Body.ttf.ogt",
            &std::fs::read(neon.join("ChakraPetch-BoldItalic.ttf.ogt")).unwrap(),
        );
        write(
            "styles/mix/pack.json",
            br#"{
                "fonts": ["Body.ttf", "@styles/cjk/NotoSansSC-Regular.ttf"],
                "theme": { "fontFamily": { "sans": "\"Chakra Petch\", \"Noto Sans SC\"" } }
            }"#,
        );
        write(
            "strings/zh-CN.ogt",
            "type strings\nversion 1\n---\nlocale \"zh-CN\"\nstring \"judge.perfect\" \"完美\"\n"
                .as_bytes(),
        );
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

#[test]
fn a_floating_text_mixing_a_companion_draws_each_run_in_its_font() {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": mixed_project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let style = value(og_runtime::host_impl::answer(og_load_style(
        engine,
        "styles/mix".into(),
    )));
    let body = value(og_runtime::host_impl::answer(og_load_font(
        engine,
        "styles/mix/Body.ttf".into(),
    )));
    let chinese = value(og_runtime::host_impl::answer(og_load_font(
        engine,
        "@styles/cjk/NotoSansSC-Regular.ttf".into(),
    )));
    value(og_runtime::host_impl::answer(og_ui_configure(
        engine,
        Some(json!({ "style": style, "font": body, "fallbackFonts": [chinese] }).to_string()),
    )));
    value(og_engine_frame(engine, 1000.0 / 60.0));

    for (text, x, facing) in [("完美 x2", 4.0, None), ("美完 x3", -2.0, Some("camera"))] {
        let mut d = json!({ "text": text, "position": { "x": x, "y": 0, "z": 0 }, "spread": 0, "size": 1, "pop": 0 });
        if let Some(f) = facing {
            d["facing"] = json!(f);
        }
        value(og_runtime::host_impl::answer(og_render_floating_text(
            engine,
            Some(d.to_string()),
        )));
    }
    value(og_engine_frame(engine, 0.0));
    let seen = seen.borrow();
    // Every character drawn, the Chinese from the companion, each line
    // centred on its point.
    let [flat, faced] = [&seen.texts[0], &seen.texts[1]];
    for (d, x) in [(flat, 4.0), (faced, -2.0)] {
        assert_eq!(d.glyphs, 4, "完美 and x2 whole: {d:?}");
        assert_eq!(d.x, x);
        assert!(
            (d.span[0] + d.span[1]).abs() < 0.1 * (d.span[1] - d.span[0]),
            "centred: {d:?}"
        );
    }
    assert!(flat.facing.is_none());
    assert_eq!(faced.facing.map(|f| f.facing), Some(SpriteFacing::Camera));
}
