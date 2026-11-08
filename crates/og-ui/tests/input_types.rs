//! A field's `type`, `inputmode` and `autocomplete` (docs/ui.md §2.3),
//! through a document, tick by tick: what is drawn, read, copied and
//! validated, and what the host is told for its virtual keyboard.
//!
//! Ways these could fail, each covered below:
//! - a password drawn, probed or read to a screen reader in clear; copied
//!   or cut by the shortcut (or offered to the host as a selection); the
//!   reveal toggle stealing the focus (a phone's keyboard closing), not
//!   showing the text, or forgetting it over a save and restore →
//!   `a_password_is_masked_uncopyable_and_revealed_by_its_toggle`
//! - a number field typing letters, or a number, email or url that is not
//!   one not matching `:invalid` (or an empty one matching it) →
//!   `types_filter_and_validate_by_the_html_rules`
//! - the host told the wrong keyboard: a type's default lost, an explicit
//!   `inputmode` ignored, `autocomplete` dropped →
//!   `the_host_gets_the_keyboard_and_autofill_hints`

use crate::support;

use og_core::glam::Vec2;
use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::{Value, json};
use support::*;

const FORM: &str = r#"<meta name="component" content="form">
<column id="Form">
  <field id="Pin" type="password" autocomplete="current-password" bind:value={$pin} />
  <field id="Age" type="number" bind:value={$age} />
  <field id="Mail" type="email" inputmode="text" bind:value={$mail} />
  <field id="Site" type="url" bind:value={$site} />
</column>
<style>
#Form { gap: 8px; padding: 8px; }
field { width: 300px; }
field:invalid { width: 277px; }
</style>"#;

const STORES: [&str; 4] = ["pin", "age", "mail", "site"];

/// The form mounted retained over four writable stores.
struct Rig {
    ui: Ui,
}

impl Rig {
    fn new() -> Rig {
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_document(UiDocument::parse("ui/form.svelte", FORM).expect("the form parses"));
        let mut rig = Rig { ui };
        let mut inputs = std::collections::BTreeMap::new();
        for name in STORES {
            let source = SourceId::new("test", name);
            rig.apply(json!({"op": "signal", "source": source, "initial": ""}));
            inputs.insert(name.to_string(), source);
        }
        rig.ui
            .mount_document(SourceId::new("test", "form"), "form", inputs, &bind::NoEnv)
            .expect("the form mounts");
        rig.step(InputFrame::default());
        rig
    }

    fn apply(&mut self, command: Value) -> Value {
        let reply = wire::decode(
            &self
                .ui
                .apply_reactive(&wire::encode(&json!([command])).unwrap()),
        )
        .unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
        reply[0]["value"].clone()
    }

    /// A store's value.
    fn store(&mut self, name: &str) -> Value {
        self.apply(json!({"op": "read", "source": SourceId::new("test", name)}))["value"].clone()
    }

    fn set(&mut self, name: &str, value: Value) {
        self.apply(json!({"op": "set", "source": SourceId::new("test", name), "value": value}));
    }

    fn step(&mut self, input: InputFrame) {
        self.ui.begin(input, Viewport::new(800.0, 600.0));
        self.ui
            .build_retained_documents(&bind::NoEnv)
            .expect("the form builds");
        self.ui.end();
    }

    fn el(&self, id: &str) -> UiElement {
        self.ui
            .frame()
            .element(id)
            .cloned()
            .unwrap_or_else(|| panic!("{id}"))
    }

    fn centre(&self, id: &str) -> Vec2 {
        let e = self.el(id);
        Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0)
    }

    fn click(&mut self, at: Vec2) {
        for (pressed, released) in [(true, false), (false, true)] {
            self.step(InputFrame {
                pointer: at,
                pointer_inside: true,
                down: pressed,
                pressed,
                released,
                ..InputFrame::default()
            });
        }
        // Past the double-click window.
        for _ in 0..40 {
            self.step(InputFrame::default());
        }
    }

    /// Focuses field `id` by a press near its start.
    fn focus(&mut self, id: &str) {
        let e = self.el(id);
        self.click(Vec2::new(e.x + 6.0, e.y + e.height / 2.0));
    }

    fn typed(&mut self, text: &str) {
        let events = og_input::TextInput::split(text, false);
        self.step(InputFrame::default().with_text(&events));
        self.step(InputFrame::default());
    }

    fn chord(&mut self, key: UiKey) {
        self.step(InputFrame {
            keys: vec![key],
            ctrl: true,
            pointer_inside: true,
            ..InputFrame::default()
        });
    }

    fn a11y(&self, path_end: &str) -> A11yNode {
        self.ui
            .a11y()
            .nodes
            .iter()
            .find(|n| n.path.ends_with(path_end))
            .cloned()
            .unwrap_or_else(|| panic!("no a11y node {path_end}"))
    }
}

#[test]
fn a_password_is_masked_uncopyable_and_revealed_by_its_toggle() {
    let mut rig = Rig::new();
    rig.focus("Pin");
    rig.typed("hunter2");
    assert_eq!(
        rig.store("pin"),
        "hunter2",
        "typing reaches the bound value"
    );
    // The mask: a bullet per character (`*` in a face without one).
    let bullets = rig.el("Pin").text;
    let mask = bullets.chars().next().unwrap();
    assert!(
        matches!(mask, '\u{2022}' | '*') && bullets == mask.to_string().repeat(7),
        "the probe reads the mask, not {bullets:?}"
    );
    assert_eq!(
        rig.a11y("/Pin").value,
        bullets,
        "a screen reader reads the mask"
    );
    let target = rig
        .ui
        .frame()
        .text_input
        .clone()
        .expect("the field has the text focus");
    assert!(target.secret);
    assert_eq!(target.input_type, InputType::Password);
    assert_eq!(target.autocomplete, "current-password");
    // Select all, then copy and cut: nothing reaches the clipboard, the
    // host sees no selection and the text stays.
    rig.chord(UiKey::KeyA);
    assert_eq!(rig.ui.frame().text_input.clone().unwrap().selection, "");
    rig.chord(UiKey::KeyC);
    rig.chord(UiKey::KeyX);
    assert_eq!(
        rig.ui.take_clipboard(),
        None,
        "a password is never copied or cut"
    );
    assert_eq!(rig.store("pin"), "hunter2");
    // The toggle: masked, it offers to show; a press reveals, keeps the
    // field's text focus, and a save and restore keeps it revealed.
    assert_eq!(rig.a11y("/Pin/Reveal").name, "Show password");
    let toggle = rig.centre("Pin/Reveal");
    rig.click(toggle);
    assert_eq!(rig.el("Pin").text, "hunter2", "revealed, the text shows");
    assert_eq!(rig.a11y("/Pin/Reveal").name, "Hide password");
    assert!(
        rig.a11y("/Pin/Reveal")
            .states
            .contains(&"checked".to_string())
    );
    assert_eq!(
        rig.ui
            .frame()
            .text_input
            .as_ref()
            .map(|t| t.path.ends_with("Pin")),
        Some(true),
        "the toggle leaves the text focus on the field"
    );
    // Revealed, still never copied.
    rig.chord(UiKey::KeyA);
    rig.chord(UiKey::KeyC);
    assert_eq!(rig.ui.take_clipboard(), None);
    let saved = rig.ui.save_state();
    rig.click(toggle);
    assert_eq!(rig.el("Pin").text, bullets, "a second press masks it again");
    rig.ui.restore_state(&saved).expect("the state restores");
    rig.step(InputFrame::default());
    assert_eq!(
        rig.el("Pin").text,
        "hunter2",
        "revealed is UI state, saved and restored"
    );
}

#[test]
fn types_filter_and_validate_by_the_html_rules() {
    let mut rig = Rig::new();
    let width = |rig: &Rig, id: &str| rig.el(id).width;
    // Empty: no field is invalid.
    for id in ["Age", "Mail", "Site"] {
        assert_eq!(width(&rig, id), 300.0, "an empty {id} is valid");
    }
    rig.focus("Age");
    rig.typed("12ab.5x");
    assert_eq!(
        rig.store("age"),
        "12.5",
        "a number field types only a number's characters"
    );
    assert_eq!(width(&rig, "Age"), 300.0);
    rig.typed("e");
    assert_eq!(rig.store("age"), "12.5e");
    assert_eq!(width(&rig, "Age"), 277.0, "12.5e is no number: :invalid");
    rig.focus("Mail");
    rig.typed("ada@");
    assert_eq!(width(&rig, "Mail"), 277.0, "ada@ is no email");
    rig.typed("example.org");
    assert_eq!(width(&rig, "Mail"), 300.0, "ada@example.org is one");
    rig.focus("Site");
    rig.typed("example.org");
    assert_eq!(width(&rig, "Site"), 277.0, "a url needs a scheme");
    rig.set("site", json!("https://example.org/a"));
    rig.step(InputFrame::default());
    assert_eq!(width(&rig, "Site"), 300.0, "a bound value is validated too");
}

#[test]
fn the_host_gets_the_keyboard_and_autofill_hints() {
    let mut rig = Rig::new();
    let mut hints = Vec::new();
    for id in ["Pin", "Age", "Mail", "Site"] {
        rig.focus(id);
        let t = rig.ui.frame().text_input.clone().expect(id);
        hints.push((id, t.input_type.name(), t.inputmode.name(), t.secret));
    }
    assert_eq!(
        hints,
        [
            ("Pin", "password", "text", true),
            ("Age", "number", "decimal", false),
            // An explicit inputmode wins over the type's.
            ("Mail", "email", "text", false),
            ("Site", "url", "url", false),
        ]
    );
}

/// `references/metal/input-types-{1280,390}.png`: the form with a masked
/// password, its reveal eye, an invalid number and a valid email, the
/// password focused. Ways it could fail: the mask drawn as text, the eye
/// missing or over the text, the invalid frame not dressed.
#[test]
#[ignore = "requires GPU"]
fn input_types_match_their_reference_frames() {
    use og_render::*;
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/form.svelte", FORM).unwrap());
        let mut rig = Rig { ui };
        let mut inputs = std::collections::BTreeMap::new();
        for (name, value) in STORES.into_iter().zip(["", "12.5e", "ada@example.org", ""]) {
            let source = SourceId::new("test", name);
            rig.apply(json!({"op": "signal", "source": source, "initial": value}));
            inputs.insert(name.to_string(), source);
        }
        rig.ui
            .mount_document(SourceId::new("test", "form"), "form", inputs, &bind::NoEnv)
            .unwrap();
        rig.step(InputFrame::default());
        rig.focus("Pin");
        rig.typed("hunter2");
        for _ in 0..30 {
            rig.step(InputFrame::default());
        }
        let vp = Viewport::new(width as f32, height as f32);
        rig.ui.begin(InputFrame::default(), vp);
        rig.ui.build_retained_documents(&bind::NoEnv).unwrap();
        rig.ui.end();
        let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
        rig.ui.frame().emit(&EmitDesc::default(), &mut overlay);
        scene.overlay = Some(overlay);
        let bg = rig.ui.theme().palette.backdrop.linear();
        scene.environment.background =
            Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
        scene.post.tonemap = Tonemap::None;
        let mut renderer = Renderer::new_blocking(&RenderDesc {
            width,
            height,
            ..Default::default()
        })
        .expect("a headless adapter");
        let stats = renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        assert_eq!(stats.missing_glyphs, 0);
        let image = renderer.read_offscreen().unwrap();
        let name = format!("input-types-{width}.png");
        image::save_buffer(
            out_dir().join(&name),
            &image.pixels,
            width,
            height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let own = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references/metal")
            .join(&name);
        if std::env::var_os("OG_UI_CAPTURE").is_some() {
            record_reference(&own, width, height, &image.pixels);
        }
        let expected = image::open(&own)
            .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", own.display()))
            .to_rgba8();
        let (rmse, _) = compare(&image.pixels, expected.as_raw(), renderer.info(), &own)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        println!("{name}: RMSE {rmse:.6}");
    }
}
