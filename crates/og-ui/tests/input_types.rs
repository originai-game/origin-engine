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
//! - an old core tag (`<field>`, `<slider>`, `<toggle>`, `<divider>`,
//!   `<image>`, a bare `<label>`) accepted, or refused without naming its
//!   HTML form → `old_tags_are_refused_with_their_html_form`
//! - `required`, `minlength`, `pattern`, `min`/`max`/`step` not matching
//!   `:invalid` (or a valid value matching it); `readonly` letting typing
//!   through or blocking copy; `<textarea rows>` not sizing it; a
//!   `<label>`-wrapped checkbox not binding `checked` →
//!   `html_constraints_readonly_rows_and_label_checkboxes`

use crate::support;

use og_core::glam::Vec2;
use og_ui::*;
use serde_json::json;
use support::*;

#[path = "input_types/rig.rs"]
mod rig;
use rig::*;

/// A number input `doc` over store `value`, laid out in `viewport`.
fn number(doc: &str, value: &str, viewport: Viewport, font: UiFont) -> Doc {
    let mut d = Doc::with_font(doc, &[("value", json!(value)), ("ro", json!(false))], font);
    d.viewport = viewport;
    d.step(InputFrame::default());
    d
}

fn value(d: &mut Doc) -> String {
    d.read("value").as_str().unwrap_or_default().to_string()
}

#[test]
fn number_options_wrap_and_step_without_rewriting_formatted_text() {
    let mut d = number(
        r#"<meta name="component" content="t">
<div><input id="Number" type="number" min="0" max="1" step="0.1" data-wrap data-large-step="0.3" data-format="{value, number} kg" bind:value={$value} /></div>
<style>#Number { width: 200px; }</style>"#,
        "1",
        Viewport::new(300.0, 100.0),
        test_font(),
    );
    // Unfocused it shows its format; the value stays the raw text. (A
    // document's format is a message, which has no fixed decimals.)
    assert_eq!(value(&mut d), "1");
    assert!(d.ui.frame().items.iter().any(|item| matches!(&item.cmd,
        DrawCmd::Text { text, .. } if text == "1 kg")));
    for (shift, expected) in [(false, "0"), (true, "0.3")] {
        let button = d.get("Number/Increment");
        let pointer = Vec2::new(
            button.x + button.width / 2.0,
            button.y + button.height / 2.0,
        );
        for down in [true, false] {
            d.step(InputFrame {
                pointer,
                pointer_inside: true,
                down,
                pressed: down,
                released: !down,
                shift,
                ..Default::default()
            });
        }
        assert_eq!(value(&mut d), expected);
    }
    for (key, expected) in [(UiKey::Up, "0.6"), (UiKey::Down, "0.3")] {
        d.step(InputFrame {
            keys: vec![key],
            shift: true,
            ..Default::default()
        });
        assert!(
            d.ui.focus_path(0).ends_with("/Number"),
            "{}",
            d.ui.focus_path(0)
        );
        assert_eq!(
            value(&mut d),
            expected,
            "Shift arrows use the configured large step"
        );
    }
}

#[test]
fn number_buttons_repeat_while_held_and_accelerate() {
    let mut d = number(
        r#"<meta name="component" content="t">
<div><input id="Number" type="number" step="0.25" data-large-step="1.5" readonly={$ro} bind:value={$value} /></div>
<style>#Number { width: 277px; }</style>"#,
        "0",
        Viewport::new(1280.0, 720.0),
        test_font_metrics(),
    );
    for (button, sign, readonly) in [
        ("Increment", 1.0, false),
        ("Decrement", -1.0, false),
        ("Increment", 1.0, true),
    ] {
        d.set("ro", json!(readonly));
        let rect = d.get(&format!("Number/{button}"));
        let pointer = Vec2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
        let rate = d.ui.ticks_per_second();
        let mut changes = Vec::new();
        for frame in 0..rate * 2 {
            let before: f64 = value(&mut d).parse().unwrap();
            d.step(InputFrame {
                pointer,
                pointer_inside: true,
                down: true,
                pressed: frame == 0,
                ..Default::default()
            });
            let delta = value(&mut d).parse::<f64>().unwrap() - before;
            if delta != 0.0 {
                let age = f64::from(frame) / f64::from(rate);
                assert!(!readonly);
                assert!(frame == 0 || age > 0.4, "no repeat before the hold delay");
                assert_eq!(delta, sign * if age > 1.0 { 1.5 } else { 0.25 });
                changes.push(age);
            }
        }
        if !readonly {
            assert_eq!(changes[0], 0.0, "a press steps immediately");
            assert!(changes[1] > 0.4 && changes[1] < 0.5);
            assert!(
                changes
                    .windows(2)
                    .any(|w| w[0] > 0.5 && w[1] < 1.6 && w[1] - w[0] >= 0.12)
            );
            assert!(
                changes.windows(2).any(|w| w[0] > 1.6 && w[1] - w[0] < 0.1),
                "long holds repeat faster"
            );
        }
        let before = value(&mut d);
        d.step(InputFrame {
            pointer,
            pointer_inside: true,
            released: true,
            ..Default::default()
        });
        assert_eq!(
            value(&mut d),
            before,
            "release must not add another click step"
        );
    }
    let mut rig = Rig::with(
        r#"<meta name="component" content="form"><div><input id="Number" type="number" bind:value={$value} /></div>"#,
        &["value"],
    );
    rig.set("value", json!("0"));
    let pointer = rig.centre("Number/Increment");
    for frame in 0..90 {
        rig.step(InputFrame {
            pointer,
            pointer_inside: true,
            down: true,
            pressed: frame == 0,
            ..Default::default()
        });
    }
    assert!(
        rig.store("value").as_str().unwrap().parse::<f64>().unwrap() > 10.0,
        "retained buttons keep repeating between input events"
    );
}

#[test]
fn number_spinner_targets_fit_inside_the_field() {
    for coarse in [false, true] {
        let mut vp = Viewport::new(1280.0, 844.0);
        vp.coarse_pointer = coarse;
        let d = number(
            r#"<meta name="component" content="t">
<column><input id="Number" type="number" bind:value={$value} /><input id="Sibling" /></column>
<style>#Number { width: 277px; }</style>"#,
            "0.5",
            vp,
            test_font_metrics(),
        );
        let field = d.get("Number");
        assert_eq!(field.height, d.get("Sibling").height);
        let up = d.get("Number/Increment");
        let down = d.get("Number/Decrement");
        let minimum = if coarse { 44.0 } else { 24.0 };
        for button in [&up, &down] {
            assert!(button.width >= minimum);
            assert_eq!(button.y, field.y);
            assert_eq!(button.height, field.height);
            assert!(button.x + button.width <= field.x + field.width);
        }
        // The cells end at the field's end: no strip of its plate past them.
        assert_eq!(up.x + up.width, field.x + field.width);
        assert!(down.x + down.width <= up.x, "targets must not overlap");
    }
    let mut rig = Rig::with(
        r#"<meta name="component" content="form"><div><input id="Number" type="number" bind:value={$value} /></div>"#,
        &["value"],
    );
    for (width, coarse, target) in [
        (1280.0, false, 24.0),
        (390.0, false, 44.0),
        (1280.0, true, 44.0),
        (1280.0, false, 24.0),
    ] {
        let mut viewport = Viewport::new(width, 844.0);
        viewport.coarse_pointer = coarse;
        rig.ui.begin(InputFrame::default(), viewport);
        rig.ui.build_retained_documents(&bind::NoEnv).unwrap();
        rig.ui.end();
        assert_eq!(
            rig.el("Number/Increment").width,
            target,
            "retained targets follow the viewport"
        );
    }
}

#[test]
fn number_buttons_keep_the_editable_value_and_readonly_selection() {
    let mut rig = Rig::with(
        r#"<meta name="component" content="form">
        <div>
        <input id="Number" type="number" min="-2" max="3" step="0.25" bind:value={$value} />
        <input id="Read" type="number" readonly bind:value={$other} />
        </div>"#,
        &["value", "other"],
    );
    rig.focus("Number");
    rig.typed("1e");
    assert_eq!(rig.store("value"), "1e");
    rig.typed("1");
    assert_eq!(rig.store("value"), "1e1");
    rig.set("value", json!("0.5"));
    rig.step(InputFrame::default());
    rig.click(rig.centre("Number/Increment"));
    assert_eq!(rig.store("value"), "0.75");
    rig.click(rig.centre("Number/Decrement"));
    assert_eq!(rig.store("value"), "0.5");
    rig.set("value", json!("0.6"));
    rig.step(InputFrame::default());
    rig.click(rig.centre("Number/Increment"));
    assert_eq!(rig.store("value"), "0.75");
    rig.focus("Number");
    for (key, shift, expected) in [
        (UiKey::Up, false, "1"),
        (UiKey::Down, false, "0.75"),
        (UiKey::Down, true, "-1.75"),
        (UiKey::Up, true, "0.75"),
    ] {
        rig.step(InputFrame {
            keys: vec![key],
            shift,
            ..Default::default()
        });
        assert_eq!(rig.store("value"), expected);
        assert!(
            rig.ui.frame().text_input.is_some(),
            "arrows keep editing focus"
        );
    }
    rig.set("other", json!("2"));
    rig.step(InputFrame::default());
    rig.focus("Read");
    rig.typed("9");
    rig.click(rig.centre("Read/Increment"));
    rig.step(InputFrame {
        keys: vec![UiKey::Up],
        ..Default::default()
    });
    assert_eq!(rig.store("other"), "2");
    rig.chord(UiKey::KeyA);
    rig.chord(UiKey::KeyC);
    assert_eq!(rig.ui.take_clipboard().as_deref(), Some("2"));
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

#[test]
fn old_tags_are_refused_with_their_html_form() {
    for (old, html) in [
        ("<field bind:value={$a} />", "<input type=\"text\">"),
        ("<slider bind:value={$a} />", "<input type=\"range\">"),
        (
            "<toggle bind:checked={$a} />",
            "<input type=\"checkbox\" switch>",
        ),
        ("<divider />", "<hr>"),
        ("<image src=\"a.png\" />", "<img>"),
        ("<label>Hi</label>", "<text>"),
    ] {
        let doc = format!("<meta name=\"component\" content=\"form\">\n<column>{old}</column>");
        let err = UiDocument::parse("ui/form.svelte", &doc)
            .err()
            .unwrap_or_else(|| panic!("{old} parses"));
        let err = err.join("\n");
        assert!(
            err.contains("is not a core element") && err.contains(html),
            "{old}: {err}"
        );
    }
    for bad in [
        "<textarea multiline bind:value={$a} />",
        "<input pattern=\"[a-\" bind:value={$a} />",
        "<input max-chars=\"4\" bind:value={$a} />",
        "<textarea lines=\"4\" bind:value={$a} />",
    ] {
        let doc = format!("<meta name=\"component\" content=\"form\">\n<column>{bad}</column>");
        assert!(
            UiDocument::parse("ui/form.svelte", &doc).is_err(),
            "{bad} is refused"
        );
    }
}

const CONSTRAINED: &str = r#"<meta name="component" content="form">
<column id="Form">
  <input id="Name" required minlength="3" bind:value={$name} />
  <input id="Code" pattern="[A-Z]{2}[0-9]+" bind:value={$code} />
  <input id="Qty" type="number" min="2" max="10" step="2" bind:value={$qty} />
  <input id="Fixed" readonly bind:value={$fixed} />
  <textarea id="Notes" rows="4" bind:value={$notes} />
  <textarea id="Line" rows="1" bind:value={$line} />
  <label id="Opt"><input type="checkbox" bind:checked={$opt} /> Subtitles</label>
</column>
<style>
#Form { gap: 8px; padding: 8px; }
input, textarea { width: 300px; }
input:invalid, textarea:invalid { width: 277px; }
</style>"#;

#[test]
fn html_constraints_readonly_rows_and_label_checkboxes() {
    let stores = ["name", "code", "qty", "fixed", "notes", "line", "opt"];
    let mut rig = Rig::with(CONSTRAINED, &stores);
    let width = |rig: &Rig, id: &str| rig.el(id).width;
    assert_eq!(
        width(&rig, "Name"),
        277.0,
        "an empty required field is invalid"
    );
    for id in ["Code", "Qty", "Fixed", "Notes"] {
        assert_eq!(width(&rig, id), 300.0, "an empty optional {id} is valid");
    }
    for (store, value, id, valid) in [
        ("name", "Al", "Name", false),
        ("name", "Ada", "Name", true),
        ("code", "AB12", "Code", true),
        ("code", "ab12", "Code", false),
        ("code", "AB12x", "Code", false),
        ("qty", "1", "Qty", false),
        ("qty", "4", "Qty", true),
        ("qty", "5", "Qty", false),
        ("qty", "12", "Qty", false),
        ("qty", "10", "Qty", true),
        ("qty", "4.5", "Qty", false),
    ] {
        rig.set(store, json!(value));
        rig.step(InputFrame::default());
        assert_eq!(
            width(&rig, id) == 300.0,
            valid,
            "{id} = {value:?} valid: {valid}"
        );
    }
    // Readonly: typing is dropped, the text still selects and copies.
    rig.set("fixed", json!("locked"));
    rig.step(InputFrame::default());
    rig.focus("Fixed");
    rig.typed("zz");
    assert_eq!(
        rig.store("fixed"),
        "locked",
        "a readonly field drops typing"
    );
    rig.chord(UiKey::KeyA);
    rig.chord(UiKey::KeyX);
    assert_eq!(rig.store("fixed"), "locked", "nor is it cut");
    let _ = rig.ui.take_clipboard();
    rig.chord(UiKey::KeyA);
    rig.chord(UiKey::KeyC);
    assert_eq!(
        rig.ui.take_clipboard().as_deref(),
        Some("locked"),
        "a readonly field copies"
    );
    // rows: four lines are taller than one.
    assert!(
        rig.el("Notes").height > rig.el("Line").height * 2.0,
        "rows=4 {} vs rows=1 {}",
        rig.el("Notes").height,
        rig.el("Line").height
    );
    // A label wrapping a checkbox is the checkbox, captioned by the label.
    assert_eq!(rig.store("opt"), "");
    let at = rig.centre("Opt");
    rig.click(at);
    assert_eq!(rig.store("opt"), true, "a press on the caption checks it");
    assert_eq!(rig.a11y("/Opt").name, "Subtitles");
}
