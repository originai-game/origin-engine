//! og.ui.configure takes a partial UiDesc: a field left out keeps the last
//! call's value (or a look's style), `null` clears one, and a desc that
//! leaves the UI no font at all warns that it draws no text (lantern-vale
//! FINDINGS #11: configuring sounds after a look's style dropped the style,
//! and with no font every glyph vanished silently).
//!
//! A style names its assets relative to its own file first (FINDINGS #6: a
//! copied pack had to be rewritten): `copied/style.ogt` names `Body.ttf`
//! beside it, `../mini/Body.ttf` up a directory and `styles/mini/Body.ttf`
//! from assets/, and both the cook and og.load.style find all three.
//!
//! A project with two styles (`styles/mini/style.ogt`, a light scheme whose body
//! font is named from assets/; `styles/copied/style.ogt`, relative ids) is
//! cooked once.
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
        let font = repo().join("templates/styles/fantasy/Newsreader-Medium.ttf");
        write("styles/mini/Body.ttf", &std::fs::read(font).unwrap());
        write(
            "styles/mini/Body.ttf.ogt",
            b"type import\nversion 1\n---\nfont {\n  charset latin\n  em_px 32\n  px_range 4\n}\n",
        );
        // A style sets every role and radius: the prototype's, written out.
        let values = |scheme: og_ui::Scheme| {
            let t = og_ui::Theme::new(scheme);
            let mut out = String::from("palette {\n");
            for n in og_ui::Palette::NAMES {
                let c = t.palette.get(n).unwrap();
                let [r, g, b] = c.rgb;
                out += &format!("  {n} \"#{r:02x}{g:02x}{b:02x}{:02x}\"\n", (c.alpha * 255.0).round() as u8);
            }
            out + "}\nradius {\n  small 1\n  control 2\n  bar 1\n  panel 3\n}\n"
        };
        write(
            "styles/mini/style.ogt",
            format!(
                "type style\nversion 1\n---\nname \"mini\"\nscheme light\n{}fonts {{\n  body \"styles/mini/Body.ttf\"\n}}\n",
                values(og_ui::Scheme::Light)
            )
            .as_bytes(),
        );
        let font = repo().join("templates/styles/fantasy/Cinzel-Bold.ttf");
        write("styles/copied/Body.ttf", &std::fs::read(font).unwrap());
        write(
            "styles/copied/Body.ttf.ogt",
            b"type import\nversion 1\n---\nfont {\n  charset latin\n  em_px 32\n  px_range 4\n}\n",
        );
        write(
            "styles/copied/style.ogt",
            format!(
                "type style\nversion 1\n---\nname \"copied\"\n{}fonts {{\n  body \"Body.ttf\"\n  display \"../mini/Body.ttf\"\n  numeric \"styles/mini/Body.ttf\"\n}}\n",
                values(og_ui::Scheme::Dark)
            )
            .as_bytes(),
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
    value(og_ui_configure(engine, Some(desc.to_string())));
}

fn theme(engine: u32) -> Value {
    serde_json::from_str(&og_ui_theme(engine)).unwrap()
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
    let style = value(og_load_style(e, "styles/mini/style.ogt".into()));
    configure(e, json!({ "style": style, "contrast": "high" }));
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
    assert_eq!(t["scheme"].as_str(), Some("light"), "the style's scheme");
    // A call with nothing keeps everything.
    value(og_ui_configure(e, None));
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
    let w = warnings(e);
    assert!(
        w.iter()
            .any(|w| w.contains("no font") && w.contains("og.load.font") && w.contains("style")),
        "{w:?}"
    );
    // A desc that is not an object is refused.
    let r: Value = serde_json::from_str(&og_ui_configure(e, Some("3".into()))).unwrap();
    assert_eq!(r["ok"], json!(false), "{r}");
}

#[test]
fn a_style_names_its_assets_relative_to_itself_or_from_assets() {
    let e = engine();
    let style = value(og_load_style(e, "styles/copied/style.ogt".into()));
    configure(e, json!({ "style": style }));
    assert_eq!(theme(e)["style"].as_str(), Some("copied"));
    assert!(
        warnings(e).iter().all(|w| !w.contains("no font")),
        "its body font, beside it, loaded"
    );
}
