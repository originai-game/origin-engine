//! og.ui.configure takes a partial UiDesc: a field left out keeps the last
//! call's value (or a look's style), `null` clears one, and a desc that
//! leaves the UI no font at all warns that it draws no text (lantern-vale
//! FINDINGS #11: configuring sounds after a look's style dropped the style,
//! and with no font every glyph vanished silently).
//!
//! CSS packs resolve font URLs beside their individual CSS files, including
//! nested imports. Absolute URLs start at assets/. Packs load by directory.
//!
//! Ways this could fail: a later call that sets only `reducedMotion` or
//! `contrast` drops the style (the theme's style name goes empty) or resets
//! the fields the earlier call set; `null` does not clear; a fontless UI
//! says nothing; a UI with a font warns anyway; the cook refuses the
//! relative ids ("names asset(s) not under assets/") or the runtime looks
//! for them at the root of assets/.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_take_log, og_game_game,
    og_host_begin_bundle, og_load_style, og_ui_configure, og_ui_theme, set_hooks,
};
use serde_json::{Value, json};
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
            .join(format!("og-runtime-ui-configure-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        let font = repo().join("templates/styles/shared/Newsreader-Medium.ttf");
        write("styles/mini/Body.ttf", &std::fs::read(font).unwrap());
        write(
            "styles/mini/Body.ttf.ogt",
            b"type import\nversion 1\n---\nfont {\n  charset latin\n  em_px 32\n  px_range 4\n}\n",
        );
        write(
            "styles/mini/main.css",
            br#"@font-face { font-family: Mini; src: url(Body.ttf); }
                button::part(base) { opacity: 0.37; }
                button::part(text) { font-family: Mini; }"#,
        );
        let font = repo().join("templates/styles/shared/Cinzel-Bold.ttf");
        write("styles/copied/Body.ttf", &std::fs::read(font).unwrap());
        write(
            "styles/copied/Body.ttf.ogt",
            b"type import\nversion 1\n---\nfont {\n  charset latin\n  em_px 32\n  px_range 4\n}\n",
        );
        write(
            "styles/copied/main.css",
            br#"@import "nested/fonts.css";
                button::part(text) { font-family: Copied, Mini; }"#,
        );
        write(
            "styles/copied/nested/fonts.css",
            br#"@font-face { font-family: Copied; src: url(../Body.ttf); }
                @font-face { font-family: Mini; src: url(../../mini/Body.ttf); }
                @font-face { font-family: Numeric; src: url(/styles/mini/Body.ttf); }"#,
        );
        let glyph = br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><path d="M0 0H2V4H0Z" fill="white"/></svg>"#;
        write("icons/glyph.svg", glyph);
        write("icons/mask.svg", glyph);
        write("ui/nested/pointer.css", br#"#Pointer { mask-image: url(../../icons/mask.svg); }"#);
        write("ui/pointer.svelte", br#"<meta name="screen" content="Pointer" />
<div id="Pointer" style="cursor: url(missing.svg), url(../icons/glyph.svg) 1 2, crosshair"></div>
<style>@import "nested/pointer.css";
#Pointer { width: 80px; height: 80px; background: red; }</style>"#);
        write("ui/shape.svelte", br#"<meta name="screen" content="Shape" />
<div id="Root"><div id="F"></div><div id="Line"><span id="A"></span></div></div>
<style>#Root { width: 300px; }
#F { float: left; width: 80px; height: 40px; shape-outside: url(../icons/glyph.svg); }
#Line { font-size: 0; }
#A { display: inline-block; width: 100px; height: 20px; }</style>"#);
        write(
            "ui/vitals.svelte",
            br#"<meta name="screen" content="Vitals" />
<div id="Vitals"><StatBar value={50} max={100} /></div>"#,
        );
        // The game's code names a built-in screen by its id, which bundles it.
        std::fs::write(
            root.join("project.ogt"),
            "type project\nversion 1\n---\nname \"ui configure\"\nid \"ui-configure\"\nentry \"main.ts\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("main.ts"),
            "export const PAUSE = \"@ui/screens/pause.svelte\";\n",
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

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn engine() -> u32 {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    value(og_engine_create(game, options)).as_u64().unwrap() as u32
}

fn configure(engine: u32, desc: Value) {
    value(og_runtime::host_impl::answer(og_ui_configure(
        engine,
        Some(desc.to_string()),
    )));
}

fn theme(engine: u32) -> Value {
    serde_json::from_str(&og_runtime::host_impl::json(og_ui_theme(engine))).unwrap()
}

/// The warnings logged since the last call.
fn warnings(engine: u32) -> Vec<String> {
    let log: Value = serde_json::from_str(&og_engine_take_log(engine)).unwrap();
    log.as_array()
        .unwrap()
        .iter()
        .filter(|l| l["warning"] == json!(true))
        .map(|l| l["text"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn configure_keeps_the_fields_a_call_leaves_out() {
    let e = engine();
    let button = og_ui::dress::PartRef::new("Button", "base", og_ui::tokens::ElementClass::Button);
    let opacity = || {
        og_runtime::shared_ui(e)
            .unwrap()
            .lock()
            .resolve_part(&button, [80.0, 32.0])
            .unwrap()
            .opacity
    };
    let style = value(og_runtime::host_impl::answer(og_load_style(
        e,
        "styles/mini".into(),
    )));
    configure(
        e,
        json!({ "style": style, "contrast": "high", "scheme": "light" }),
    );
    assert_eq!(opacity(), 0.37, "the directory CSS applies");
    let t = theme(e);
    assert_eq!(
        (t["style"].as_str(), t["contrast"].as_str()),
        (Some("mini"), Some("high"))
    );
    // Sounds and reduced motion later, as a settings screen does: the style
    // and the contrast stay.
    configure(e, json!({ "reducedMotion": true }));
    let t = theme(e);
    assert_eq!(t["style"].as_str(), Some("mini"), "the style stays: {t}");
    assert_eq!(t["contrast"].as_str(), Some("high"), "the contrast stays");
    assert_eq!(t["reducedMotion"], json!(true));
    assert_eq!(
        t["scheme"].as_str(),
        Some("light"),
        "configured scheme stays"
    );
    assert_eq!(opacity(), 0.37, "pack CSS survives partial configure");
    // A call with nothing keeps everything.
    value(og_runtime::host_impl::answer(og_ui_configure(e, None)));
    assert_eq!(theme(e)["style"].as_str(), Some("mini"));
    // null clears a field to its default.
    configure(e, json!({ "contrast": null }));
    let t = theme(e);
    assert_eq!(
        (t["contrast"].as_str(), t["style"].as_str()),
        (Some("standard"), Some("mini"))
    );
    assert_eq!(t["reducedMotion"], json!(true), "still reduced");
    assert!(
        warnings(e).iter().all(|w| !w.contains("no font")),
        "the style's body font draws"
    );
    // Without the style the UI has no font at all: a warning names the fix.
    configure(e, json!({ "style": null }));
    assert_eq!(theme(e)["style"].as_str(), Some(""));
    assert_eq!(opacity(), 1.0, "clearing the pack removes its CSS");
    let w = warnings(e);
    assert!(
        w.iter()
            .any(|w| w.contains("no font") && w.contains("og.load.font") && w.contains("style")),
        "{w:?}"
    );
    // A desc that is not an object is refused.
    let r: Value = serde_json::from_str(&og_runtime::host_impl::answer(og_ui_configure(
        e,
        Some("3".into()),
    )))
    .unwrap();
    assert_eq!(r["ok"], json!(false), "{r}");
}

#[test]
fn a_style_names_its_assets_relative_to_itself_or_from_assets() {
    let e = engine();
    let style = value(og_runtime::host_impl::answer(og_load_style(
        e,
        "styles/copied".into(),
    )));
    configure(e, json!({ "style": style }));
    assert_eq!(theme(e)["style"].as_str(), Some("copied"));
    let ui = og_runtime::shared_ui(e).unwrap();
    let ui = ui.lock();
    assert_eq!(ui.fonts().families.len(), 3);
    assert!(ui.fonts().families.iter().all(|f| f.font.handle.is_some()));
    assert!(
        warnings(e).iter().all(|w| !w.contains("no font")),
        "its body font, beside it, loaded"
    );
}

#[test]
fn shape_outside_images_take_their_contour_from_cooked_alpha() {
    let e = engine();
    let style = value(og_runtime::host_impl::answer(og_load_style(
        e,
        "styles/mini".into(),
    )));
    configure(e, json!({ "style": style }));
    value(og_runtime::host_impl::answer(og_runtime::og_load_document(
        e,
        "ui/shape.svelte".into(),
    )));
    value(og_runtime::host_impl::answer(og_runtime::og_ui_push(
        e,
        "Shape".into(),
        None,
    )));
    og_runtime::og_engine_frame(e, 1000.0 / 60.0);
    let ui = og_runtime::shared_ui(e).unwrap();
    let ui = ui.lock();
    let x = |id: &str| {
        ui.probe_elements()
            .into_iter()
            .find(|el| el.id == id)
            .unwrap_or_else(|| panic!("no {id}"))
            .x
    };
    // The cooked SVG's left half is opaque: the line starts at half the
    // float's 80px width, not after its box.
    assert_eq!(x("A") - x("Root"), 40.0);
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

/// A screen that uses a built-in component mounts it with only its own
/// load: the game never loads `@ui/components/stat-bar.svelte`. Ways it could fail:
/// the component undefined (the frame warns "no component 'StatBar'" and
/// draws nothing in its place).
#[test]
fn a_screen_mounts_the_built_in_components_it_uses() {
    let e = engine();
    value(og_runtime::host_impl::answer(og_runtime::og_load_document(
        e,
        "ui/vitals.svelte".into(),
    )));
    value(og_runtime::host_impl::answer(og_runtime::og_ui_push(
        e,
        "Vitals".into(),
        None,
    )));
    og_runtime::og_engine_frame(e, 1000.0 / 60.0);
    let ui = og_runtime::shared_ui(e).unwrap();
    let ui = ui.lock();
    let elements = ui.probe_elements();
    assert!(
        elements.iter().any(|el| el.id == "Track"),
        "{:?}",
        elements.iter().map(|el| &el.id).collect::<Vec<_>>()
    );
    assert!(
        ui.frame()
            .warnings
            .iter()
            .all(|w| !w.contains("no component")),
        "{:?}",
        ui.frame().warnings
    );
    drop(ui);
    assert!(og_runtime::og_engine_destroy(e));
}

/// A built-in screen mounts by its id without a copy: the game's code
/// names `@ui/screens/pause.svelte`, so the cook bundles it with its
/// strings, and the game loads and pushes it as it does its own. Ways it
/// could fail: the screen not bundled (the load fails), or its strings
/// not (its buttons show their keys).
#[test]
fn a_built_in_screen_mounts_without_a_copy() {
    let e = engine();
    value(og_runtime::host_impl::answer(og_runtime::og_load_document(
        e,
        "@ui/screens/pause.svelte".into(),
    )));
    value(og_runtime::host_impl::answer(og_runtime::og_ui_push(
        e,
        "pause".into(),
        None,
    )));
    og_runtime::og_engine_frame(e, 1000.0 / 60.0);
    let ui = og_runtime::shared_ui(e).unwrap();
    let ui = ui.lock();
    let elements = ui.probe_elements();
    let resume = elements
        .iter()
        .find(|el| el.id == "Resume")
        .unwrap_or_else(|| panic!("{:?}", elements.iter().map(|el| &el.id).collect::<Vec<_>>()));
    assert_eq!(resume.text, "Resume");
    drop(ui);
    assert!(og_runtime::og_engine_destroy(e));
}

#[test]
fn svg_asset_urls_supply_cursor_pixels_and_masks() {
    struct Backend;
    impl og_runtime::RenderBackend for Backend {
        fn tier(&self) -> String {
            "test".into()
        }
        fn size(&self) -> (u32, u32) {
            (100, 100)
        }
        fn render(&mut self, _: &og_render::RenderScene) -> Result<(), String> {
            Ok(())
        }
        fn capture(
            &mut self,
            _: &og_render::RenderScene,
            _: u32,
            _: u32,
        ) -> Option<og_render::RgbaImage> {
            None
        }
    }
    let e = engine();
    og_runtime::set_render_backend(e, Some(Box::new(Backend))).unwrap();
    let style = value(og_runtime::host_impl::answer(og_load_style(
        e,
        "styles/mini".into(),
    )));
    configure(e, json!({ "style": style }));
    value(og_runtime::host_impl::answer(og_runtime::og_load_document(
        e,
        "ui/pointer.svelte".into(),
    )));
    value(og_runtime::host_impl::answer(og_runtime::og_ui_push(
        e,
        "Pointer".into(),
        None,
    )));
    value(og_runtime::host_impl::answer(
        og_runtime::og_commands_schedule(e, "0:touch_move:90,90".into()),
    ));
    og_runtime::og_engine_frame(e, 1000.0 / 60.0);
    value(og_runtime::host_impl::answer(
        og_runtime::og_commands_schedule(e, "0:touch_move:20,20".into()),
    ));
    let frame: Value =
        serde_json::from_str(&og_runtime::og_engine_frame(e, 1000.0 / 60.0)).unwrap();
    assert_eq!(frame["ok"], true, "{frame}");
    assert_eq!(frame["cursor"], "crosshair", "{frame}");
    let image = &frame["cursorImage"];
    assert_eq!(
        (image["width"].as_u64(), image["height"].as_u64()),
        (Some(4), Some(4)),
        "{frame}"
    );
    assert_eq!(image["hotspot"], json!([1, 2]));
    let rgba = og_core::base64::decode(image["rgba"].as_str().unwrap()).unwrap();
    for y in 0..4 {
        assert_eq!(&rgba[y * 16..y * 16 + 8], &[255; 8]);
        assert_eq!((rgba[y * 16 + 11], rgba[y * 16 + 15]), (0, 0));
    }
    let ui = og_runtime::shared_ui(e).unwrap();
    let ui = ui.lock();
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    assert!(ui.frame().items.iter().any(|item| matches!(
        &item.cmd,
        og_ui::DrawCmd::EndLayer {
            blend: og_ui::paint::Blend::DestinationIn,
            ..
        }
    )));
    drop(ui);
    assert!(og_runtime::og_engine_destroy(e));
}
