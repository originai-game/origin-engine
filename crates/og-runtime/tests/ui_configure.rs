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
                button::part(text) { font-family: Mini; }
                .once { width: 61px; }"#,
        );
        write(
            "styles/tinted/main.css",
            br#".tinted { width: 1px; }"#,
        );
        write(
            "styles/tinted/pack.json",
            br##"{ "theme": { "colors": { "fg": { "light": "#123456", "dark": "#654321" } }, "radius": { "button": 13 } } }"##,
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
                button::part(text) { font-family: Copied, Mini; }
                .once { width: 62px; }"#,
        );
        write(
            "styles/copied/nested/fonts.css",
            br#"@font-face { font-family: Copied; src: url(../Body.ttf); }
                @font-face { font-family: Mini; src: url(../../mini/Body.ttf); }
                @font-face { font-family: Numeric; src: url(/styles/mini/Body.ttf); }"#,
        );
        std::fs::write(
            root.join("project.ogt"),
            "type project\nversion 1\n---\nname \"ui configure\"\nid \"ui-configure\"\nentry \"main.ts\"\n",
        )
        .unwrap();
        // A TSX UI: the cook bundles the default UI (its font and pack art).
        std::fs::write(
            root.join("main.ts"),
            "import \"@origingame/engine/ui\";\nexport {};\n",
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
    // Clearing the style restores the bundled default face, not a fontless UI.
    configure(e, json!({ "style": null }));
    assert_eq!(theme(e)["style"].as_str(), Some(""));
    assert_eq!(opacity(), 1.0, "clearing the pack removes its CSS");
    let w = warnings(e);
    assert!(w.iter().all(|w| !w.contains("no font")), "{w:?}");
    // A desc that is not an object is refused.
    let r: Value = serde_json::from_str(&og_runtime::host_impl::answer(og_ui_configure(
        e,
        Some("3".into()),
    )))
    .unwrap();
    assert_eq!(r["ok"], json!(false), "{r}");
}

/// og.ui.theme answers the style pack's theme by the design tokens' names:
/// the default pack's in the configured scheme, the game's pack over it,
/// configured seeds over both, and the motion tokens' durations. Ways this
/// could fail: the colours come from one scheme whatever is configured;
/// the game's pack is not laid over the default; run-time seeds reach only
/// old og-ui; a name is og-ui's old role (`ink`, `control`).
#[test]
fn the_theme_is_the_packs_by_its_token_names() {
    let e = engine();
    let default = og_ui_next::pack::Pack::new(&[og_ui_next::pack::Layer::default_pack()]);
    let fg = |s| colour(&default, s, "fg");
    let t = theme(e);
    assert_eq!(t["scheme"], json!("dark"));
    assert_eq!(
        t["colors"]["fg"].as_str(),
        Some(fg(og_ui_next::pack::Scheme::Dark).as_str())
    );
    assert!(
        t["colors"].get("ink").is_none(),
        "no og-ui role names: {}",
        t["colors"]
    );
    assert_eq!(t["durations"]["fast"], json!(180.0));
    // A pinned unit is the one in effect.
    configure(e, json!({ "unitScale": 2 }));
    assert_eq!(theme(e)["unit"], json!(2.0));
    configure(e, json!({ "unitScale": null }));
    assert!(
        t["fontSize"]["body"].as_f64().is_some_and(|s| s > 0.0),
        "{}",
        t["fontSize"]
    );
    configure(e, json!({ "scheme": "light" }));
    let t = theme(e);
    assert_eq!(
        t["colors"]["fg"].as_str(),
        Some(fg(og_ui_next::pack::Scheme::Light).as_str())
    );
    let style = value(og_runtime::host_impl::answer(og_load_style(
        e,
        "styles/tinted".into(),
    )));
    configure(e, json!({ "style": style }));
    let t = theme(e);
    assert_eq!(t["style"], json!("tinted"));
    assert_eq!(
        t["colors"]["fg"],
        json!("#123456"),
        "the game's pack over the default"
    );
    assert_eq!(t["radius"]["button"], json!(13.0));
    assert!(
        t["colors"]["bg"].is_string(),
        "the default's colours under it"
    );
    // Seeds given at run time (a look's, a faction's) derive the palette
    // over the pack, as a pack's own seeds would: #c8102e is linear
    // 0.578, 0.00518, 0.0273.
    configure(e, json!({ "style": null, "scheme": "dark" }));
    let plain = theme(e)["colors"]["accent"].clone();
    configure(
        e,
        json!({ "seeds": { "primary": { "r": 0.578, "g": 0.00518, "b": 0.0273 } } }),
    );
    let seeded = og_ui_next::pack::Pack::new(&[
        og_ui_next::pack::Layer::default_pack(),
        og_ui_next::pack::Layer::parse(r##"{ "seeds": { "primary": "#c8102e" } }"##).unwrap(),
    ]);
    let want = colour(&seeded, og_ui_next::pack::Scheme::Dark, "accent");
    let t = theme(e);
    assert_ne!(t["colors"]["accent"], plain, "the seed recolours");
    assert_eq!(t["colors"]["accent"].as_str(), Some(want.as_str()));
    configure(e, json!({ "seeds": null }));
    assert_eq!(theme(e)["colors"]["accent"], plain, "null clears the seeds");
    assert!(og_runtime::og_engine_destroy(e));
}

/// `pack`'s colour `name` in `scheme`, as og.ui.theme writes a colour.
fn colour(pack: &og_ui_next::pack::Pack, scheme: og_ui_next::pack::Scheme, name: &str) -> String {
    let c = og_css::parse_color(pack.theme(scheme)["colors"][name].as_str().unwrap()).unwrap();
    og_core::color::hex_string([
        c.rgb[0],
        c.rgb[1],
        c.rgb[2],
        (c.alpha * 255.0).round() as u8,
    ])
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
