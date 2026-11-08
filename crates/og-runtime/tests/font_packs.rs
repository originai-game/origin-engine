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

const PACKS: [&str; 6] = [
    "latin",
    "arabic",
    "japanese",
    "chinese-simplified",
    "chinese-traditional",
    "emoji",
];

/// A project whose UI face is a Latin-only display font, with five packs
/// installed as `og font add` installs them; cooked once per process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| cooked("og-runtime-font-packs", None))
}

/// The same project with every pack face cooked as an atlas font for
/// `MIXED`'s characters: the baseline a run-time face is measured against.
fn atlas_project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| cooked("og-runtime-font-packs-atlas", Some(MIXED)))
}

/// The mixed-script label the frames draw.
const MIXED: &str = "はじめる 骨 · ليلى 门 · Start";

/// The emoji label: single emoji, a skin tone, a ZWJ family and a flag
/// (each sequence one glyph the face's GSUB forms), and a heart with its
/// emoji presentation selector.
const EMOJI: &str = "Victory 🏆🔥⭐ 👍🏽 👨‍👩‍👧 🇳🇿 ❤️";

fn cooked(name: &str, atlas: Option<&str>) -> PathBuf {
    {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("{name}-{}", std::process::id()));
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
            if let Some(text) = atlas {
                for e in std::fs::read_dir(&to).unwrap() {
                    let path = e.unwrap().path();
                    if path.extension().is_some_and(|x| x == "ogt") {
                        // The emoji face cooks its colour glyphs as images.
                        let colour = std::fs::read_to_string(&path)
                            .unwrap()
                            .contains("color true");
                        let (chars, colour) = if colour {
                            (
                                EMOJI.replace(char::is_alphabetic, ""),
                                "  color true\n  color_px 64\n",
                            )
                        } else {
                            (format!("{text} \u{3000}").replace('"', ""), "")
                        };
                        std::fs::write(
                            path,
                            format!("type import\nversion 1\n---\nfont {{\n  charset\n  chars {chars:?}\n  em_px 48\n  px_range 16\n{colour}}}\n"),
                        )
                        .unwrap();
                    }
                }
            }
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    }
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
    start_in(project(), locale)
}

fn start_in(root: &Path, locale: &str) -> (u32, impl Fn()) {
    set_hooks();
    let system = og_system_define_system(
        "Status".into(),
        json!({ "reads": [], "writes": [] }).to_string(),
        BUILD,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [system] }).to_string()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
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

/// What the backend saw of the last frame.
#[derive(Default)]
struct Seen {
    frame: Option<og_render::RgbaImage>,
    /// Milliseconds the backend spent waiting on the GPU (not the engine's CPU).
    waited: f64,
    /// The frame's GPU milliseconds, from timestamp queries (all passes).
    gpu: f64,
    /// The UI overlay pass's share of it.
    gpu_ui: f64,
}

/// og-render's headless renderer behind the engine. Each frame waits for
/// the GPU and reads its timestamps; `capture` reads the frame back.
struct Offscreen {
    renderer: og_render::Renderer,
    seen: Rc<RefCell<Seen>>,
    read: bool,
}

fn now_ms() -> f64 {
    static T0: OnceLock<std::time::Instant> = OnceLock::new();
    T0.get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs_f64()
        * 1e3
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
        let t = std::time::Instant::now();
        let frames = self.renderer.gpu_timings(true);
        let mut seen = self.seen.borrow_mut();
        if let Some(f) = frames.last() {
            seen.gpu = f.passes.iter().map(|p| p.ms()).sum();
            seen.gpu_ui = f
                .passes
                .iter()
                .filter(|p| p.name.contains("overlay") || p.name.contains("ui"))
                .map(|p| p.ms())
                .sum();
        }
        if self.read {
            seen.frame = Some(self.renderer.read_offscreen().map_err(|e| e.to_string())?);
        }
        seen.waited = t.elapsed().as_secs_f64() * 1e3;
        Ok(())
    }
    fn capture(
        &mut self,
        _: &og_render::RenderScene,
        _: u32,
        _: u32,
    ) -> Option<og_render::RgbaImage> {
        self.seen.borrow().frame.clone()
    }
}

/// One frame's cost: the engine's CPU milliseconds (the frame without the
/// backend's wait on the GPU) and the GPU's.
#[derive(Clone, Copy, Debug, Default)]
struct Cost {
    cpu: f64,
    gpu: f64,
    gpu_ui: f64,
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// An engine in `ja-JP` on `root`, drawing to a `width`×`height` renderer.
fn rig(
    root: &Path,
    width: u32,
    height: u32,
    read: bool,
) -> (
    u32,
    impl Fn() -> Cost,
    Rc<RefCell<Seen>>,
    og_render::RenderInfo,
) {
    let (engine, step) = start_in(root, "ja-JP");
    let seen = Rc::new(RefCell::new(Seen::default()));
    let mut renderer = og_render::Renderer::new_blocking(&og_render::RenderDesc {
        width,
        height,
        ..Default::default()
    })
    .expect("a headless adapter");
    renderer.set_clock(Some(now_ms));
    let info = renderer.info().clone();
    og_runtime::set_render_backend(
        engine,
        Some(Box::new(Offscreen {
            renderer,
            seen: seen.clone(),
            read,
        })),
    )
    .unwrap();
    let s = seen.clone();
    let frame = move || {
        let t = std::time::Instant::now();
        step();
        let wall = t.elapsed().as_secs_f64() * 1e3;
        let s = s.borrow();
        Cost {
            cpu: wall - s.waited,
            gpu: s.gpu,
            gpu_ui: s.gpu_ui,
        }
    };
    (engine, frame, seen, info)
}

/// Steady (median of 30), the first frames that draw `text` (loading
/// the packs only it needs), and steady after (median of 30).
fn costs(root: &Path, text: &str) -> (Cost, Vec<Cost>, Cost, u32) {
    let (engine, frame, _, _) = rig(root, 1280, 720, false);
    let steady = |n: usize| {
        let c: Vec<Cost> = (0..n).map(|_| frame()).collect();
        Cost {
            cpu: median(c.iter().map(|c| c.cpu).collect()),
            gpu: median(c.iter().map(|c| c.gpu).collect()),
            gpu_ui: median(c.iter().map(|c| c.gpu_ui).collect()),
        }
    };
    show("はじめる 骨 · Start");
    steady(10);
    let before = steady(30);
    show(text);
    // The packs load on the first frame, their glyphs are asked for on
    // the second: the most still rendering when any of the frames returns.
    let mut waited = 0;
    let first: Vec<Cost> = (0..12)
        .map(|_| {
            let c = frame();
            waited = waited.max(og_runtime::glyphs_pending(engine));
            std::thread::sleep(std::time::Duration::from_millis(4));
            c
        })
        .collect();
    steady(10);
    let after = steady(30);
    (before, first, after, waited)
}

/// Frame cost of `MIXED` from run-time pack faces against the same packs
/// cooked as atlas fonts for its characters, in one scene. Prints CPU and
/// GPU milliseconds; asserts a run-time face's steady frame within noise of
/// the atlas font's, and that the frames drawing a new script return
/// while its glyphs still render (rendered on the frame, they hitched it:
/// 203 ms).
#[test]
#[ignore = "requires GPU"]
fn run_time_faces_cost_what_atlas_fonts_cost() {
    let show_cost = |c: &Cost| {
        format!(
            "cpu {:.2} ms, gpu {:.3} ms (ui {:.3})",
            c.cpu, c.gpu, c.gpu_ui
        )
    };
    let mut runs = Vec::new();
    for (name, root) in [("run-time", project()), ("atlas", atlas_project())] {
        // Best of three, each a fresh engine: the machine's noise.
        let mut best: Option<(Cost, Vec<Cost>, Cost, u32)> = None;
        for _ in 0..3 {
            let r = costs(root, MIXED);
            if best.as_ref().is_none_or(|b| r.2.cpu < b.2.cpu) {
                best = Some(r);
            }
        }
        let (before, first, after, waited) = best.unwrap();
        if name == "run-time" {
            assert!(waited > 0, "a frame waited for its new glyphs to render");
        }
        println!("{name}: steady before {}", show_cost(&before));
        for (i, c) in first.iter().enumerate() {
            println!("{name}: first-use frame {i} {}", show_cost(c));
        }
        println!("{name}: steady after {}", show_cost(&after));
        runs.push((before, first, after, waited));
    }
    let (rt, at) = (&runs[0], &runs[1]);
    assert!(
        rt.2.cpu <= at.2.cpu * 1.25 + 0.3,
        "run-time steady {:.2} ms against atlas {:.2} ms",
        rt.2.cpu,
        at.2.cpu
    );
    let worst = |r: &(Cost, Vec<Cost>, Cost, u32)| r.1.iter().map(|c| c.cpu).fold(0.0, f64::max);
    println!(
        "worst first-use frame: run-time {:.2} ms ({} glyphs rendered off it), atlas {:.2} ms",
        worst(rt),
        rt.3,
        worst(at)
    );
}

/// `crates/og-ui/references/metal/font-packs-{1280,390}.png`: a Japanese
/// game's label mixing kana and kanji (its locale's pack), a player's
/// Arabic name and a Simplified-only character (packs loaded when first
/// drawn), over the Latin display face, every pack glyph from its face's
/// outlines at run time. Ways it could fail: boxes or blanks where a pack
/// should have loaded; Arabic unjoined or in logical order; 骨 in its
/// Chinese form in a Japanese game (`OG_UI_CAPTURE=1` records).
#[test]
#[ignore = "requires GPU"]
fn mixed_script_text_draws_from_its_packs() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let (engine, frame, seen, info) = rig(project(), width, height, true);
        show(MIXED);
        settle(engine, &frame, 3);
        let w = warnings(engine);
        assert!(w.iter().all(|w| !w.contains("lacks")), "{w:?}");
        assert_eq!(
            chain(engine),
            vec!["japanese", "arabic", "chinese-simplified"]
        );
        let image = seen.borrow_mut().frame.take().expect("a frame drawn");
        check_reference(&format!("font-packs-{width}.png"), &image, &info);
    }
}

/// Glyphs render off the frame: draws until the chain holds `packs`
/// packs and three frames in a row wait for no glyph.
fn settle(engine: u32, frame: &impl Fn() -> Cost, packs: usize) {
    let mut quiet = 0;
    for _ in 0..2000 {
        frame();
        quiet = if chain(engine).len() == packs && og_runtime::glyphs_pending(engine) == 0 {
            quiet + 1
        } else {
            0
        };
        if quiet == 3 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(quiet, 3, "the glyphs never arrived");
}

/// `crates/og-ui/references/metal/color-emoji-{1280,390}.png`: a label
/// in the Latin display face with emoji from the emoji pack (Noto Color
/// Emoji, COLRv1), loaded when first drawn and painted in colour from the
/// face's COLR table at run time. Ways it could fail: the emoji drawn as
/// distance fields in the text's ink (no colour); a ZWJ family or a flag
/// drawn as its parts; the pack never loaded, so they are warned of and
/// boxed; a colour page's glyphs blank (an atlas never uploaded).
#[test]
#[ignore = "requires GPU"]
fn colour_emoji_draw_from_the_emoji_pack_in_colour() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let (engine, frame, seen, info) = rig(project(), width, height, true);
        show(EMOJI);
        settle(engine, &frame, 2);
        let w = warnings(engine);
        assert!(w.iter().all(|w| !w.contains("lacks")), "{w:?}");
        assert_eq!(chain(engine), vec!["japanese", "emoji"]);
        let image = seen.borrow_mut().frame.take().expect("a frame drawn");
        // Colour: the label's ink is a light grey, and the scene black, so
        // only an emoji painted from its palette is saturated (🔥's orange,
        // 🏆's gold, the flag's red and blue).
        let saturated = image
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| {
                let (hi, lo) = (p[0].max(p[1]).max(p[2]), p[0].min(p[1]).min(p[2]));
                hi > 120 && hi - lo > 90
            })
            .count();
        assert!(
            saturated > 400,
            "{saturated} saturated pixels: no colour emoji"
        );
        check_reference(&format!("color-emoji-{width}.png"), &image, &info);
    }
}

/// Frame and size cost of colour emoji painted at run time (the emoji
/// pack as shipped) against the same emoji cooked as images (`color
/// true`, its characters in `chars`). Prints CPU and GPU milliseconds and
/// the cooked bytes; asserts the steady frame within noise of the cooked
/// font's, and that the frames first drawing them return while they are
/// still being painted.
#[test]
#[ignore = "requires GPU"]
fn run_time_emoji_cost_what_cooked_emoji_cost() {
    let show_cost = |c: &Cost| {
        format!(
            "cpu {:.2} ms, gpu {:.3} ms (ui {:.3})",
            c.cpu, c.gpu, c.gpu_ui
        )
    };
    let mut runs = Vec::new();
    for (name, root) in [("run-time", project()), ("cooked", atlas_project())] {
        let bytes: u64 = std::fs::read_dir(root.join("cooked/fonts/emoji"))
            .unwrap()
            .map(|e| e.unwrap().metadata().unwrap().len())
            .sum();
        println!("{name}: emoji pack cooked to {bytes} bytes");
        let mut best: Option<(Cost, Vec<Cost>, Cost, u32)> = None;
        for _ in 0..3 {
            let r = costs(root, EMOJI);
            if best.as_ref().is_none_or(|b| r.2.cpu < b.2.cpu) {
                best = Some(r);
            }
        }
        let (before, first, after, waited) = best.unwrap();
        if name == "run-time" {
            assert!(waited > 0, "a frame waited for its emoji to be painted");
        }
        println!("{name}: steady before {}", show_cost(&before));
        for (i, c) in first.iter().enumerate() {
            println!("{name}: first-use frame {i} {}", show_cost(c));
        }
        println!("{name}: steady after {}", show_cost(&after));
        runs.push((before, first, after, waited));
    }
    let (rt, ck) = (&runs[0], &runs[1]);
    assert!(
        rt.2.cpu <= ck.2.cpu * 1.25 + 0.3,
        "run-time steady {:.2} ms against cooked {:.2} ms",
        rt.2.cpu,
        ck.2.cpu
    );
    let worst = |r: &(Cost, Vec<Cost>, Cost, u32)| r.1.iter().map(|c| c.cpu).fold(0.0, f64::max);
    println!(
        "worst first-use frame: run-time {:.2} ms ({} glyphs painted off it), cooked {:.2} ms",
        worst(rt),
        rt.3,
        worst(ck)
    );
}

/// Compares `image` with `crates/og-ui/references/metal/<name>`
/// (`OG_UI_CAPTURE=1` records it) under og-tests' rule for this tier,
/// writing it to the test's temporary directory; prints its RMSE.
fn check_reference(name: &str, image: &og_render::RgbaImage, info: &og_render::RenderInfo) {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&out, og_render::png_best(image)).unwrap();
    let own = repo().join("crates/og-ui/references/metal").join(name);
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
        image.width * image.height,
        1,
        &tolerance,
    )
    .unwrap();
    c.check(&tolerance)
        .unwrap_or_else(|e| panic!("{name}: {e}"));
    println!("{name}: RMSE {:.6} ({})", c.rmse, out.display());
}
