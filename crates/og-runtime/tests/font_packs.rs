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
    og_system_define_system, og_ui_configure,
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
    ROOT.get_or_init(|| cooked("og-runtime-font-packs"))
}

/// The mixed-script label the frames draw.
const MIXED: &str = "はじめる 骨 · ليلى 门 · Start";

/// The emoji label: single emoji, a skin tone, a ZWJ family and a flag
/// (each sequence one glyph the face's GSUB forms), and a heart with its
/// emoji presentation selector.
const EMOJI: &str = "Victory 🏆🔥⭐ 👍🏽 👨‍👩‍👧 🇳🇿 ❤️";

fn cooked(name: &str) -> PathBuf {
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
            "type import\nversion 1\n---\nfont {\n}\n",
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
        let report = og_cook::cook_project(
            &root,
            // Its label is TSX: the cook bundles the default UI's font and
            // pack art, as for code that reaches the TSX UI.
            &og_cook::CookOptions {
                code: og_cook::cook::CodeGraph {
                    tsx: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    }
}

struct Disk;

const BUILD: u32 = 1;

thread_local! {
    static LABEL: RefCell<Option<String>> = const { RefCell::new(None) };
    /// The engines whose UI shows the label.
    static MOUNTED: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    /// Files whose path holds one of these are still being fetched, as the
    /// web reads an on-demand face after the boot.
    static FETCHING: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

/// Shows `text` as a TSX label in engine `e`'s tree, as the TS renderer's
/// op batch does (og-ui-next CONVENTIONS.md §Ops): on the first call a View
/// holding a text run mounted at the root, after it the run's new text.
fn label(e: u32, first: bool, text: &str) {
    let mut ops = Vec::new();
    let u32s = |ops: &mut Vec<u8>, vs: &[u32]| {
        for v in vs {
            ops.extend_from_slice(&v.to_le_bytes());
        }
    };
    let str = |ops: &mut Vec<u8>, s: &str| {
        ops.extend_from_slice(&(s.len() as u32).to_le_bytes());
        ops.extend_from_slice(s.as_bytes());
    };
    let count = if first {
        // create 1 (a View), createText 2, insert 2 into 1, 1 into the root.
        ops.push(1);
        u32s(&mut ops, &[1]);
        ops.push(0);
        ops.push(2);
        u32s(&mut ops, &[2]);
        str(&mut ops, text);
        ops.push(8);
        u32s(&mut ops, &[1, 2, u32::MAX]);
        ops.push(8);
        u32s(&mut ops, &[0, 1, u32::MAX]);
        4u32
    } else {
        ops.push(3);
        u32s(&mut ops, &[2]);
        str(&mut ops, text);
        1
    };
    let mut batch = og_ui_next::tree::MAGIC.to_vec();
    batch.extend_from_slice(&count.to_le_bytes());
    batch.extend_from_slice(&ops);
    let reply = og_runtime::og_engine_ui_patch(e, &batch);
    assert!(
        !reply.starts_with(og_ui_next::tree::REFUSED),
        "the ops apply: {}",
        String::from_utf8_lossy(&reply)
    );
}

impl Hooks for Disk {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0
            && call.id == BUILD
            && let Some(text) = LABEL.with(|l| l.borrow().clone())
        {
            let first = MOUNTED.with(|m| {
                let mut m = m.borrow_mut();
                let first = !m.contains(&call.engine);
                if first {
                    m.push(call.engine);
                }
                first
            });
            label(call.engine, first, &text);
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        if FETCHING.with(|f| f.borrow().iter().any(|p| path.contains(p))) {
            return Fetch::Pending;
        }
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
    let info: Value =
        serde_json::from_str(&og_runtime::host_impl::json(og_i18n_locale(engine))).unwrap();
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
    let face = value(og_runtime::host_impl::answer(og_load_font(
        engine,
        "fonts/Display.ttf".into(),
    )));
    value(og_runtime::host_impl::answer(og_ui_configure(
        engine,
        Some(json!({ "font": face }).to_string()),
    )));
    og_runtime::host_impl::json(og_i18n_set_locale(engine, locale.into()));
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
fn an_undeclared_direct_font_load_is_refused_even_with_native_disk_bytes() {
    let (engine, step) = start("ja-JP");
    let path = "fonts/arabic/NotoSansArabic-Regular.ttf";
    let refused = || {
        let reason = og_load_font(engine, path.into()).unwrap_err();
        assert!(reason.contains(path), "{reason}");
        assert!(reason.contains("the web cannot prefetch"), "{reason}");
        assert!(reason.contains("publish.keep"), "{reason}");
    };
    refused();
    // i18n may load the same face on demand; that must not make the
    // synchronous public contract depend on which host fetched it first.
    og_runtime::host_impl::json(og_i18n_set_locale(engine, "ar".into()));
    show("Start");
    step();
    assert_eq!(chain(engine), vec!["arabic"]);
    refused();
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

    // Hebrew: no face has it (the display face ships whole, so its Thai
    // draws) and no pack installed covers it, so it is still named.
    show("สวัสดี שלום");
    step();
    let w = warnings(engine);
    assert!(w.iter().any(|w| w.contains("U+05E9")), "{w:?}");
    assert!(w.iter().all(|w| !w.contains("U+0E2A")), "{w:?}");
    assert_eq!(
        chain(engine),
        vec!["japanese", "arabic"],
        "no pack for Hebrew"
    );
}

#[test]
fn the_first_frame_waits_until_a_landed_face_draws_what_it_lacked() {
    // Every face with Arabic or Han still being fetched.
    let han = [
        "NotoSansSC",
        "NotoSansTC",
        "NotoSansJP",
        "NotoSerifSC",
        "ZCOOL",
    ];
    FETCHING.with(|f| *f.borrow_mut() = [&["NotoSansArabic"][..], &han].concat());
    let (_engine, step) = start("ar");
    show("ليلى 商店");
    step();
    step();
    assert!(og_runtime::fonts_pending() > 0, "the frame lacks both");
    // The Arabic face lands first: the Han is still missing, and behind
    // the play gate no tick lays the frame out again to say so.
    FETCHING.with(|f| f.borrow_mut().retain(|p| !p.contains("Arabic")));
    assert!(
        og_runtime::fonts_pending() > 0,
        "the first frame still waits for the Han"
    );
    FETCHING.with(|f| f.borrow_mut().clear());
    assert_eq!(
        og_runtime::fonts_pending(),
        0,
        "every face it lacked landed"
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
fn default_ui_uses_instrument_outlines_before_locale_fallbacks() {
    let bundle = og_cook::runtime::Bundle::open(&project().join("cooked")).unwrap();
    let cooked = bundle
        .load_font(og_ui_next::fonts::DEFAULT_FONT_ASSET)
        .expect("a UI game bundles its default face");
    assert!(
        cooked.pages.is_empty(),
        "the default ships outlines, not an atlas"
    );
    let source =
        std::fs::read(repo().join("templates/styles/shared/InstrumentSans[wdth,wght].ttf"))
            .unwrap();
    assert_eq!(cooked.file.as_ref().unwrap(), &source);
    let (engine, step) = start("ja-JP");
    value(og_runtime::host_impl::answer(og_ui_configure(
        engine,
        Some(json!({"font": null, "style": null}).to_string()),
    )));
    show("Start");
    step();
    step();
    // Every role the game named no font for resolves to the default face,
    // ahead of the locale's packs.
    for role in og_runtime::ui_font_roles(engine) {
        assert_eq!(
            og_runtime::ui_text_family(engine, &role).as_deref(),
            Some("Instrument Sans"),
            "{role}"
        );
    }
    let face = value(og_runtime::host_impl::answer(og_load_font(
        engine,
        "fonts/Display.ttf".into(),
    )));
    value(og_runtime::host_impl::answer(og_ui_configure(
        engine,
        Some(json!({"font": face}).to_string()),
    )));
    step();
    let roles = og_runtime::ui_font_roles(engine);
    assert_ne!(
        og_runtime::ui_text_family(engine, &roles[0]).as_deref(),
        Some("Instrument Sans"),
        "the game's explicit font still wins"
    );
    assert_eq!(warnings(engine), Vec::<String>::new());
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
}

/// og-render's headless renderer behind the engine; `capture` reads the
/// frame back.
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
        if self.read {
            let mut seen = self.seen.borrow_mut();
            seen.frame = Some(self.renderer.read_offscreen().map_err(|e| e.to_string())?);
        }
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

/// An engine in `ja-JP` on `root`, drawing to a `width`×`height` renderer.
fn rig(
    root: &Path,
    width: u32,
    height: u32,
    read: bool,
) -> (u32, impl Fn(), Rc<RefCell<Seen>>, og_render::RenderInfo) {
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
    let frame = move || {
        step();
    };
    (engine, frame, seen, info)
}

/// `crates/og-tests/references/vulkan/font-packs-{1280,390}.png`: a Japanese
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

/// Glyphs render off the frame and pipelines compile off it: draws until
/// the chain holds `packs` packs and three frames in a row skip no draw
/// (og-render `frame_skipped`), so the frame read is the scene's whole.
fn settle(engine: u32, frame: &impl Fn(), packs: usize) {
    let mut quiet = 0;
    for _ in 0..2000 {
        frame();
        quiet = if chain(engine).len() == packs && og_render::frame_skipped() == 0 {
            quiet + 1
        } else {
            0
        };
        if quiet == 3 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(
        quiet, 3,
        "the glyphs never arrived, or a draw was still skipped"
    );
}

/// `crates/og-tests/references/vulkan/color-emoji-{1280,390}.png`: a label
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

/// Compares `image` with `crates/og-tests/references/vulkan/<name>`
/// (`OG_UI_CAPTURE=1` records it) under og-tests' rule for this tier,
/// writing it to the test's temporary directory; prints its RMSE and
/// reports a frame outside it (`og_tests::gfx::report_mismatch`).
fn check_reference(name: &str, image: &og_render::RgbaImage, info: &og_render::RenderInfo) {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&out, og_render::png_best(image)).unwrap();
    let own = repo()
        .join("crates/og-tests/references")
        .join(og_tests::compare::RECORDING_TIER)
        .join(name);
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
    assert_eq!(
        expected.dimensions(),
        (image.width, image.height),
        "{name}: the reference is another size"
    );
    if let Some(why) = og_tests::compare::broken(&image.pixels, expected.as_raw()) {
        panic!("{name}: {why} ({})", out.display());
    }
    let c = og_tests::compare::compare(
        &image.pixels,
        expected.as_raw(),
        image.width,
        image.height,
        &tolerance,
    )
    .unwrap();
    println!("{name}: RMSE {:.6} ({})", c.rmse, out.display());
    // A moved frame is reported for review, with its diff, never failed.
    if let Err(e) = c.check(&tolerance) {
        let diff = out.with_extension("diff.png");
        image::save_buffer(
            &diff,
            &c.diff_image,
            image.width,
            image.height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        og_tests::gfx::report_mismatch(&format!(
            "{name}: {e}\n  reference {}\n  actual    {}\n  diff      {}",
            own.display(),
            out.display(),
            diff.display()
        ));
    }
}
