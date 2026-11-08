//! Font packs (docs/ui.md §3.4): installed packs join og.i18n's fallback
//! chain with no game code. The locale's own pack is in the chain from the
//! first UI frame; another pack loads only when a frame draws a character
//! that only it covers, and that character is not warned of as missing.
//! A Chinese locale never takes the other script's pack, and a character
//! no pack covers is still named.
//!
//! Ways this could fail: every pack loaded up front (the size cut costs
//! memory instead); a pack never loaded, so mixed-script text warns and
//! draws boxes; zh-TW given the Simplified face for a Han character both
//! cover; a character no pack covers silenced with the rest.
//!
//! Runs with no render backend: the chain and the warnings are CPU state.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_take_log, og_game_game,
    og_host_begin_bundle, og_i18n_locale, og_i18n_set_locale, og_load_font,
    og_system_define_system, og_ui_configure, og_ui_label,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const PACKS: [&str; 5] = [
    "latin",
    "arabic",
    "japanese",
    "chinese-simplified",
    "chinese-traditional",
];

/// A project whose UI face is a Latin-only display font, with five packs
/// installed as `og font add` installs them; cooked once per process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-font-packs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let fonts = root.join("assets/fonts");
        std::fs::create_dir_all(&fonts).unwrap();
        let style = repo().join("templates/styles/shared");
        std::fs::copy(
            style.join("ChakraPetch-BoldItalic.ttf"),
            fonts.join("Display.ttf"),
        )
        .unwrap();
        std::fs::write(
            fonts.join("Display.ttf.ogt"),
            "type import\nversion 1\n---\nfont {\n  charset latin\n  em_px 48\n  px_range 4\n}\n",
        )
        .unwrap();
        for pack in PACKS {
            let from = repo().join("templates/fonts").join(pack);
            let to = fonts.join(pack);
            std::fs::create_dir_all(&to).unwrap();
            for e in std::fs::read_dir(&from).unwrap() {
                let e = e.unwrap();
                std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
            }
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

struct Disk;

const BUILD: u32 = 1;

thread_local! {
    static LABEL: RefCell<Option<String>> = const { RefCell::new(None) };
}

impl Hooks for Disk {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0
            && call.id == BUILD
            && let Some(text) = LABEL.with(|l| l.borrow().clone())
        {
            og_ui_label("Status".into(), text, Some(r#"{"step":"xl4"}"#.into()));
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn warnings(engine: u32) -> Vec<String> {
    let log: Value = serde_json::from_str(&og_engine_take_log(engine)).unwrap();
    log.as_array()
        .unwrap()
        .iter()
        .filter(|l| l["warning"] == json!(true))
        .map(|l| l["text"].as_str().unwrap().to_owned())
        .collect()
}

/// The packs whose faces are in the applied chain, in chain order.
fn chain(engine: u32) -> Vec<String> {
    let info: Value = serde_json::from_str(&og_i18n_locale(engine)).unwrap();
    let info = if info.get("ok").is_some() {
        info["value"].clone()
    } else {
        info
    };
    let mut packs: Vec<String> = Vec::new();
    for f in info["fonts"].as_array().unwrap() {
        let f = f.as_str().unwrap();
        let pack = f
            .strip_prefix("fonts/")
            .and_then(|f| f.split_once('/'))
            .map(|(p, _)| p);
        if let Some(p) = pack
            && !packs.iter().any(|q| q == p)
        {
            packs.push(p.to_owned());
        }
    }
    packs
}

/// An engine showing `LABEL` in the Latin display face, in `locale`.
fn start(locale: &str) -> (u32, impl Fn()) {
    set_hooks();
    let system = og_system_define_system(
        "Status".into(),
        json!({ "reads": [], "writes": [] }).to_string(),
        BUILD,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [system] }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let face = value(og_load_font(engine, "fonts/Display.ttf".into()));
    value(og_ui_configure(
        engine,
        Some(json!({ "font": face }).to_string()),
    ));
    og_i18n_set_locale(engine, locale.into());
    let step = move || {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    };
    (engine, step)
}

fn set_hooks() {
    og_runtime::set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
}

fn show(text: &str) {
    LABEL.with(|l| *l.borrow_mut() = Some(text.into()));
}

#[test]
fn the_locale_pack_is_in_the_chain_and_others_load_on_demand() {
    let (engine, step) = start("ja-JP");
    show("Start");
    step();
    step();
    assert_eq!(
        chain(engine),
        vec!["japanese"],
        "the locale's own pack, alone"
    );
    let _ = warnings(engine);

    // Japanese text: drawn from the locale's pack, nothing missing.
    show("はじめる 設定");
    step();
    step();
    assert_eq!(warnings(engine), Vec::<String>::new());
    assert_eq!(chain(engine), vec!["japanese"]);

    // A player's Arabic name: the Arabic pack loads, and nothing is warned.
    show("ليلى はじめる");
    step();
    step();
    let w = warnings(engine);
    assert!(w.iter().all(|w| !w.contains("U+0644")), "{w:?}");
    assert_eq!(chain(engine), vec!["japanese", "arabic"]);

    // Thai: no pack installed covers it, so it is still named.
    show("สวัสดี");
    step();
    let w = warnings(engine);
    assert!(w.iter().any(|w| w.contains("U+0E2A")), "{w:?}");
    assert_eq!(
        chain(engine),
        vec!["japanese", "arabic"],
        "no pack for Thai"
    );
}

#[test]
fn chinese_locales_never_take_the_other_script() {
    // 骨 is in both Chinese packs (and the Japanese one) with different
    // glyph forms; 门 is Simplified only, 門 Traditional only.
    let (engine, step) = start("zh-TW");
    show("骨 門");
    step();
    step();
    assert_eq!(chain(engine), vec!["chinese-traditional"]);
    let _ = warnings(engine);

    // A Simplified-only character loads the Simplified pack behind the
    // Traditional one: 骨 keeps its Traditional form.
    show("骨 门");
    step();
    step();
    assert_eq!(warnings(engine), Vec::<String>::new());
    let c = chain(engine);
    assert_eq!(
        c.first().map(String::as_str),
        Some("chinese-traditional"),
        "{c:?}"
    );
    assert!(c.iter().any(|p| p == "chinese-simplified"), "{c:?}");

    let (engine, step) = start("zh-CN");
    show("骨 门");
    step();
    step();
    assert_eq!(chain(engine), vec!["chinese-simplified"]);
}

#[test]
fn a_game_with_no_locale_draws_in_the_latin_pack() {
    let (engine, step) = start("");
    show("Ça va? Über");
    step();
    step();
    assert_eq!(chain(engine), vec!["latin"]);
}

/// og-render's headless renderer behind the engine: its last frame kept.
struct Offscreen {
    renderer: og_render::Renderer,
    frame: Rc<RefCell<Option<og_render::RgbaImage>>>,
}

impl og_runtime::RenderBackend for Offscreen {
    fn tier(&self) -> String {
        self.renderer.info().backend_tier()
    }
    fn size(&self) -> (u32, u32) {
        self.renderer.size()
    }
    fn info(&self) -> Option<og_render::RenderInfo> {
        Some(self.renderer.info().clone())
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        self.renderer
            .render(scene, &og_render::RenderTarget::Offscreen)
            .map_err(|e| e.to_string())?;
        *self.frame.borrow_mut() = Some(self.renderer.read_offscreen().map_err(|e| e.to_string())?);
        Ok(())
    }
    fn capture(
        &mut self,
        _: &og_render::RenderScene,
        _: u32,
        _: u32,
    ) -> Option<og_render::RgbaImage> {
        self.frame.borrow().clone()
    }
}

/// `crates/og-ui/references/metal/font-packs-{1280,390}.png`: a Japanese
/// game's label mixing kana and kanji (its locale's pack), a player's
/// Arabic name and a Simplified-only character (packs loaded when first
/// drawn), over the Latin display face, every pack glyph from its face's
/// outlines at run time. Ways it could fail: boxes or blanks where a pack
/// should have loaded; Arabic unjoined or in logical order; 骨 in its
/// Chinese form in a Japanese game. Prints the cost of the frames that
/// load two packs against steady ones (`OG_UI_CAPTURE=1` records).
#[test]
#[ignore = "requires GPU"]
fn mixed_script_text_draws_from_its_packs() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let (engine, step) = start("ja-JP");
        let frame = Rc::new(RefCell::new(None));
        let renderer = og_render::Renderer::new_blocking(&og_render::RenderDesc {
            width,
            height,
            ..Default::default()
        })
        .expect("a headless adapter");
        let info = renderer.info().clone();
        og_runtime::set_render_backend(
            engine,
            Some(Box::new(Offscreen {
                renderer,
                frame: frame.clone(),
            })),
        )
        .unwrap();
        let timed = |n: usize| {
            let t = std::time::Instant::now();
            for _ in 0..n {
                step();
            }
            t.elapsed().as_secs_f64() * 1e3 / n as f64
        };
        show("はじめる 骨");
        timed(4);
        let steady = timed(8);
        show("はじめる 骨 · ليلى 门 · Start");
        let loading = timed(2);
        timed(4);
        let after = timed(8);
        println!(
            "font-packs-{width}: frame {steady:.2} ms steady, {loading:.2} ms for each of the two frames loading arabic and chinese-simplified, {after:.2} ms steady after"
        );
        let w = warnings(engine);
        assert!(w.iter().all(|w| !w.contains("lacks")), "{w:?}");
        assert_eq!(
            chain(engine),
            vec!["japanese", "arabic", "chinese-simplified"]
        );
        let image = frame.borrow_mut().take().expect("a frame drawn");
        let name = format!("font-packs-{width}.png");
        let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(&name);
        std::fs::write(&out, og_render::png_best(&image)).unwrap();
        let own = repo().join("crates/og-ui/references/metal").join(&name);
        if std::env::var_os("OG_UI_CAPTURE").is_some() {
            std::fs::copy(&out, &own).unwrap();
        }
        let expected = image::open(&own)
            .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", own.display()))
            .to_rgba8();
        use og_tests::compare::{Tolerance, tolerance_for};
        let tolerance = tolerance_for(
            Tolerance::default(),
            Tolerance::CROSS_PLATFORM,
            &info.backend_tier(),
            info.software(),
            &own,
        );
        let c = og_tests::compare::compare(
            &image.pixels,
            expected.as_raw(),
            width * height,
            1,
            &tolerance,
        )
        .unwrap();
        c.check(&tolerance)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        println!("{name}: RMSE {:.6} ({})", c.rmse, out.display());
    }
}
