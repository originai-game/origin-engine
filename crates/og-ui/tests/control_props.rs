//! The HTML and CSS a document's controls take beyond their tag (docs/ui.md
//! §2.4): a range's `<datalist>` marks, `data-fill-origin` and `disabled`;
//! a checkbox's caption side, `<small>` description, control size and
//! `disabled`; tabs' size and `disabled`; a dialog's `closedby` and
//! `appearance: none`; a select's `aria-invalid` and
//! `::picker(select) { max-height }`; an image's `object-view-box`; and
//! `data-og-instant`. Each is built both ways a document is (immediate and
//! mounted; a dialog's closing writes back to a mounted store).
//!
//! Ways these could fail, each covered below:
//! - a `<datalist>` drawn as content, its options' ticks missing or at the
//!   wrong fractions, an option labelled by its value shown as text, the
//!   fill growing from min when `data-fill-origin` names another value, a
//!   disabled range still taking a drag, or a `list` naming nothing
//!   accepted → `a_range_takes_its_datalist_origin_and_disabled`
//! - a caption on the wrong side of its switch for the source order, a
//!   label's `flex-direction: row-reverse` ignored, a `<small>` merged into
//!   the caption, styled track dimensions or `disabled` dropped →
//!   `a_checkbox_follows_its_label_order_size_and_description`
//! - a modal closing on a backdrop press without `closedby="any"`, Escape
//!   closing a `closedby="none"` modal, an in-place dialog taking Escape by
//!   default (HTML's `none`), `appearance: none` keeping the frame →
//!   `dialogs_close_as_closedby_says`
//! - `aria-invalid` not marking a select's trigger, `::picker(select)`'s
//!   max-height not bounding the open list, or its box properties not
//!   styling it → `a_select_takes_aria_invalid_and_its_picker_height`
//! - `object-view-box: inset()` cropping the wrong sides of the texture →
//!   `an_image_shows_its_object_view_box`
//! - `data-og-instant` not settling the first entrance, or settling a
//!   later one → `data_og_instant_settles_only_the_first_entrance`
//! - a button's `text-align` ignored (its label always centred), a button
//!   inheriting its parent's `text-align` (HTML's sheet centres it), or a
//!   `<menu>` item's label centred → `a_button_label_takes_text_align`

use crate::html_text::{Doc, both};

use og_core::glam::Vec2;
use og_ui::*;
use serde_json::{Value, json};

/// The element `id` under the element whose path holds `/root/` (or ends
/// in it).
fn under(d: &Doc, root: &str, id: &str) -> UiElement {
    d.ui.probe_elements()
        .into_iter()
        .find(|e| {
            e.id == id
                && (e.path.contains(&format!("/{root}/"))
                    || e.path.starts_with(&format!("{root}/")))
        })
        .unwrap_or_else(|| panic!("no {id} under {root}"))
}

fn count_under(d: &Doc, root: &str, prefix: &str) -> Vec<UiElement> {
    let mut v: Vec<UiElement> =
        d.ui.probe_elements()
            .into_iter()
            .filter(|e| e.id.starts_with(prefix) && e.path.contains(&format!("/{root}/")))
            .collect();
    v.sort_by(|a, b| a.x.total_cmp(&b.x));
    v
}

fn refused(text: &str) -> String {
    match UiDocument::parse("ui/t.svelte", text) {
        Ok(_) => panic!("accepted: {text}"),
        Err(e) => format!("{e:?}"),
    }
}

#[test]
fn data_hit_slop_extends_reach_without_changing_boxes_or_priority() {
    let source = r#"<meta name="component" content="t">
<div id="Root">
  <div id="A" interactive="true" data-hit-slop="8" onclick={a}></div>
  <div id="B" interactive="true" data-hit-slop="8" onclick={b}></div>
</div>
<style>
#Root { width: 300px; height: 200px; }
#A, #B { position: absolute; top: 100px; width: 20px; height: 20px; }
#A { left: 100px; }
#B { left: 126px; }
</style>"#;
    both(source, &[], |d| {
        for (x, y, expected) in [
            (110.0, 94.0, "a"),
            (121.0, 110.0, "a"),
            (124.0, 110.0, "b"),
            (128.0, 110.0, "b"),
        ] {
            let events = d.click_at(Vec2::new(x, y));
            let clicks: Vec<_> = events
                .iter()
                .filter(|e| e.name == "a" || e.name == "b")
                .map(|e| e.name.as_str())
                .collect();
            assert_eq!(clicks, [expected]);
        }
        assert!(d.click_at(Vec2::new(110.0, 91.0)).is_empty());
        let a = d.get("A");
        assert_eq!((a.x, a.y, a.width, a.height), (100.0, 100.0, 20.0, 20.0));
    });
    for declaration in [
        "-og-hit-slop: 8px",
        "-og-fill-origin: 0",
        "-og-min-gap: 20",
        "-og-select: cycle",
        "-og-sheet-detents: 40%",
        "-og-sheet-edge: inline-end",
        "-og-control-size: large",
    ] {
        assert!(refused(&format!("<meta name=\"component\" content=\"t\"><div></div><style>div {{ {declaration}; }}</style>")).contains(declaration.split(':').next().unwrap()));
    }
    assert!(
        refused(r#"<meta name="component" content="t"><dialog data-detents="-1px"></dialog>"#)
            .contains("detent")
    );
}

#[test]
fn root_relative_text_sizes_follow_the_root_not_the_parent() {
    for mounted in [false, true] {
        for scale in [1.0, 1.5] {
            let source = r#"<meta name="component" content="t"><div><p id="Root">Root</p><p id="Parent">Parent</p><p id="Short">Short</p></div><style>div { font-size: 10px; } p { margin: 0; } #Root { font-size: 1.75rem; } #Parent { font-size: 1.75em; } #Short { font: 1.75rem/2 sans-serif; }</style>"#;
            let mut scene = og_render::RenderScene::new();
            let mut d = Doc::with_font(
                source,
                &[],
                mounted,
                crate::support::add_test_font(&mut scene),
            );
            let mut theme = *d.ui.theme();
            theme.type_ramp.body.size = 14.0;
            theme.text_scale = scale;
            d.ui.set_theme(theme);
            d.step(InputFrame::default());
            let sizes: Vec<_> =
                d.ui.frame()
                    .items
                    .iter()
                    .filter_map(|i| match &i.cmd {
                        DrawCmd::Text { text, size, .. } => Some((text.as_str(), *size)),
                        _ => None,
                    })
                    .collect();
            assert!(sizes.contains(&("Root", 1.75 * 14.0 * scale)), "{sizes:?}");
            assert!(sizes.contains(&("Parent", 17.5)), "{sizes:?}");
            assert!(sizes.contains(&("Short", 1.75 * 14.0 * scale)), "{sizes:?}");
        }
    }
}

#[test]
fn scene_text_shadows_follow_the_font_size() {
    for mounted in [false, true] {
        for (font, size, line, offset, blur) in [
            ("font: 12px/1.5 sans-serif", 12.0, 18.0, 1.0, 1.0),
            ("font: 20px/1.5 sans-serif", 20.0, 30.0, 1.0, 1.6),
            ("font: 48px/1.5 sans-serif", 48.0, 72.0, 1.92, 3.84),
            ("font-size: 2em", 20.0, 40.0, 1.0, 1.6),
            ("font-size: 2em; line-height: inherit", 20.0, 40.0, 1.0, 1.6),
            ("font-size: 200%; line-height: 1.5", 20.0, 30.0, 1.0, 1.6),
            (
                "font: var(--case-font); font-size: 20px",
                20.0,
                30.0,
                1.0,
                1.6,
            ),
        ] {
            let draw = |style: &str| {
                let source = format!(
                    r#"<meta name="component" content="t"><div><p id="Ink">Scene ink</p></div><style>div {{ font: 10px/2 sans-serif; --case-font: 12px/1.5 sans-serif; }} #Ink {{ margin: 0; {font}; color: #f5f5f7; {style} }}</style>"#
                );
                let mut scene = og_render::RenderScene::new();
                let d = Doc::with_font(
                    &source,
                    &[],
                    mounted,
                    crate::support::add_test_font(&mut scene),
                );
                assert!(
                    d.ui.frame().warnings.is_empty(),
                    "{:?}",
                    d.ui.frame().warnings
                );
                let items =
                    d.ui.frame()
                        .items
                        .iter()
                        .map(|i| (i.cmd.clone(), i.transform))
                        .collect::<Vec<_>>();
                assert!(!items.is_empty());
                items
            };
            let expected = draw(&format!("text-shadow: 0 {offset}px {blur}px #000000;"));
            let actual = draw("text-shadow: 0 max(1px, 0.04em) max(1px, 0.08em) #000000;");
            assert_ne!(actual, draw("text-shadow: none;"));
            assert_eq!(expected, actual, "size {size}, mounted {mounted}");
            assert!(actual.iter().any(|(cmd, _)| matches!(cmd, DrawCmd::StyledText { text, .. } if matches!(text.as_ref(), DrawCmd::Text { size: s, .. } if *s == size))));
            assert_eq!(
                draw("text-shadow: 0 0.1lh 1px #000000;"),
                draw(&format!("text-shadow: 0 {}px 1px #000000;", line * 0.1)),
                "{font}, mounted {mounted}"
            );
        }
    }
    assert!(refused(r#"<meta name="component" content="t"><span>Text</span><style>span { -og-on-scene: true; }</style>"#).contains("-og-on-scene"));
}

#[test]
fn fit_min_measures_text_with_a_literal_or_authored_step() {
    for mounted in [false, true] {
        let draw = |minimum: &str| {
            let source = format!(
                r#"<meta name="component" content="t"><p id="Title">The hidden treasures of the drowned bell tower</p><style>#Title {{ width: 100px; margin: 0; font: 48px/1.2 sans-serif; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; --floor: caption; {minimum} }}</style>"#
            );
            let mut scene = og_render::RenderScene::new();
            let mut d = Doc::with_font(
                &source,
                &[],
                mounted,
                crate::support::add_test_font(&mut scene),
            );
            d.step(InputFrame::default());
            let text: Vec<_> =
                d.ui.frame()
                    .items
                    .iter()
                    .filter_map(|i| match &i.cmd {
                        DrawCmd::Text { text, size, .. } => Some((text.clone(), *size)),
                        _ => None,
                    })
                    .collect();
            assert_eq!(text.len(), 1);
            (text, d.ui.theme().type_ramp.caption.size)
        };
        let (literal, minimum) = draw("-og-fit-min: caption;");
        assert_eq!(literal[0].1, minimum);
        assert_eq!(literal, draw("-og-fit-min: var(--floor);").0);
        assert_eq!(draw("").0[0].1, 48.0);
    }
    refused(
        r#"<meta name="component" content="t"><p>Text</p><style>p { -og-fit-min: mystery; }</style>"#,
    );
}

fn key(d: &mut Doc, k: UiKey) {
    d.step(InputFrame {
        keys: vec![k],
        ..InputFrame::default()
    });
    for _ in 0..30 {
        d.step(InputFrame::default());
    }
}

/// A range input with `low` and `high` is two knobs on one track: a press
/// takes the nearer knob, each knob steps on its own, and neither passes
/// the other by less than `data-min-gap`.
///
/// Ways it could fail: a press always moving one knob, a step on the high
/// knob moving the low one, the knobs crossing or closing past the gap, or
/// only one value written back.
#[test]
fn a_range_with_low_and_high_is_two_knobs() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <input type="range" id="Price" min="0" max="100" step="10" data-min-gap="20" bind:low={$lo} bind:high={$hi} />
</div>
<style>
#Page { padding: 20px; }
#Price { width: 400px; }
</style>"#;
    let knobs = |d: &mut Doc| {
        (
            d.read("lo").as_f64().unwrap(),
            d.read("hi").as_f64().unwrap(),
        )
    };
    let d = &mut Doc::new(text, &[("lo", json!(20)), ("hi", json!(80))], true);
    {
        d.step(InputFrame::default());
        assert_eq!(under(d, "Price", "Value").text, "20 – 80");
        let track = under(d, "Price", "Track");
        let knob = under(d, "Price", "Low").width;
        let at = |f: f32| {
            Vec2::new(
                track.x + knob / 2.0 + f * (track.width - knob),
                track.y + track.height / 2.0,
            )
        };
        d.click_at(at(0.9));
        assert_eq!(knobs(d), (20.0, 90.0), "the nearer knob, high");
        d.click_at(at(0.3));
        assert_eq!(knobs(d), (30.0, 90.0), "the nearer knob, low");
        // The high knob steps down to the low one less the gap, no further.
        d.click("High");
        for _ in 0..6 {
            key(d, UiKey::Left);
        }
        assert_eq!(knobs(d), (30.0, 50.0));
    }
}

/// A text input's `list` names a `<datalist>` of suggestions: a list under
/// the field while it has the focus, those containing the text with the
/// prefix matches first, and choosing one writes it to the field.
///
/// Ways it could fail: suggestions shown while the field is not focused, a
/// filter that keeps everything or misorders prefix matches, a choice not
/// written back, the list staying open after it, or a datalist's bound
/// `items` ignored.
#[test]
fn a_text_input_list_suggests_from_its_datalist() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <input type="search" id="Q" list="Fruit" bind:value={$q} />
  <datalist id="Fruit">
    <option value="Apple"></option><option value="Grape"></option>
    <option value="fruit-apricot" label="Apricot"></option><option value="Banana"></option>
    <option value="Unavailable" disabled></option>
  </datalist>
</div>
<style>#Page { padding: 20px; } #Q { width: 300px; }</style>"#;
    let options = |d: &Doc| -> Vec<String> {
        d.ui.probe_elements()
            .into_iter()
            .filter(|e| e.path.contains("~list/Option#") && !e.text.is_empty())
            .map(|e| e.text)
            .collect()
    };
    let d = &mut Doc::new(text, &[("q", json!(""))], true);
    assert!(
        options(d).is_empty(),
        "closed until the field has the focus"
    );
    d.click("Q");
    d.step(InputFrame::default());
    assert_eq!(options(d), ["Apple", "Grape", "Apricot", "Banana"]);
    d.set("q", json!("ap"));
    d.step(InputFrame::default());
    assert_eq!(
        options(d),
        ["Apple", "Apricot", "Grape"],
        "prefix matches first"
    );
    let apricot =
        d.ui.probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("~list/Option#1"))
            .unwrap();
    d.click_at(Vec2::new(
        apricot.x + apricot.width / 2.0,
        apricot.y + apricot.height / 2.0,
    ));
    d.step(InputFrame::default());
    assert_eq!(
        d.read("q"),
        json!("fruit-apricot"),
        "selection writes the value, not its label"
    );
    assert!(options(d).is_empty(), "closed once one is chosen");
    let bound = r#"<meta name="component" content="t">
<div id="Page"><input id="Q" list="L" bind:value={$q} /><datalist id="L" items={$names}></datalist></div>"#;
    let d = &mut Doc::new(
        bound,
        &[("q", json!("")), ("names", json!(["Silk", "Salt"]))],
        false,
    );
    d.click("Q");
    d.step(InputFrame::default());
    assert_eq!(options(d), ["Silk", "Salt"]);
}

/// A text input a pad focuses opens the on-screen keyboard: its keys type
/// into the bound value, `enterkeyhint` labels Done, and Done closes it.
/// The pointer's focus opens none.
///
/// Ways it could fail: a keyboard opening for a mouse, keys that type
/// nothing or into a copy the store never sees, Done not closing it, or
/// the field reopening it at once when the focus comes back.
#[test]
fn a_pad_focus_opens_the_on_screen_keyboard() {
    let text = r#"<meta name="component" content="t">
<div id="Page"><input id="Name" bind:value={$name} placeholder="Hero" enterkeyhint="go" /></div>
<style>#Page { padding: 20px; } #Name { width: 300px; }</style>"#;
    let key = |d: &Doc, row: u32, k: u32| {
        d.ui.probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with(&format!("Keys/Row#{row}/Key#{k}")))
    };
    let d = &mut Doc::new(text, &[("name", json!(""))], true);
    d.click("Name");
    d.step(InputFrame::default());
    assert!(key(d, 0, 0).is_none(), "no keyboard for the pointer");
    d.click_at(Vec2::new(700.0, 800.0));
    // A pad moves the focus onto the field (the pad is the device now).
    d.step(InputFrame {
        gamepad: GamepadFrame {
            pressed: 1 << 13,
            held: 1 << 13,
            device: Some(Device::Xbox),
            ..GamepadFrame::default()
        },
        ..InputFrame::default()
    });
    for _ in 0..20 {
        d.step(InputFrame::default());
    }
    assert!(
        d.ui.focus_path(0).ends_with("Keys/Row#0/Key#0"),
        "the keyboard takes the focus at its first key: {}",
        d.ui.focus_path(0)
    );
    let q = key(d, 0, 0).expect("the keyboard opens for a pad");
    assert!(
        d.ui.probe_elements()
            .iter()
            .any(|e| e.path == format!("{}/Label", q.path) && e.text == "q"),
        "the first key types q"
    );
    for _ in 0..2 {
        d.click_at(Vec2::new(q.x + q.width / 2.0, q.y + q.height / 2.0));
    }
    assert_eq!(d.read("name"), json!("qq"));
    // The action row's last key is enter, labelled by enterkeyhint.
    let enter =
        d.ui.probe_elements()
            .into_iter()
            .filter(|e| e.path.contains("Keys/Row#3/Key#"))
            .max_by(|a, b| a.x.total_cmp(&b.x))
            .unwrap();
    assert!(
        d.ui.probe_elements()
            .iter()
            .any(|e| e.path.starts_with(&enter.path) && e.text == "Go"),
        "enter reads Go"
    );
    d.click_at(Vec2::new(
        enter.x + enter.width / 2.0,
        enter.y + enter.height / 2.0,
    ));
    for _ in 0..20 {
        d.step(InputFrame::default());
    }
    assert!(
        key(d, 0, 0).is_none(),
        "Done closes it, and it stays closed"
    );
    assert_eq!(d.read("name"), json!("qq"));

    let number = text.replace("<input id=\"Name\"", "<input type=\"number\" id=\"Name\"");
    let d = &mut Doc::new(&number, &[("name", json!(""))], true);
    d.step(InputFrame {
        gamepad: GamepadFrame {
            pressed: 1 << 13,
            held: 1 << 13,
            device: Some(Device::Xbox),
            ..GamepadFrame::default()
        },
        ..InputFrame::default()
    });
    for _ in 0..20 {
        d.step(InputFrame::default());
    }
    let digit = key(d, 0, 0).expect("number inputs retain their pad keyboard");
    d.click_at(Vec2::new(
        digit.x + digit.width / 2.0,
        digit.y + digit.height / 2.0,
    ));
    assert!(
        d.read("name").as_str().unwrap().parse::<f64>().is_ok(),
        "a numeric key edits the original number input"
    );
}

#[test]
fn a_range_takes_its_datalist_origin_and_disabled() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <input type="range" id="Bal" min="-1" max="1" step="0.1" data-fill-origin="0" list="Steps" value={$bal} />
  <datalist id="Steps">
    <option value="-1" label="Left"></option>
    <option value="0">0</option>
    <option value="1" label="Right"></option>
  </datalist>
  <input type="range" id="Vol" min="0" max="1" disabled bind:value={$vol} />
</div>
<style>
#Page { display: flex; flex-direction: column; gap: 16px; padding: 20px; }
input { width: 400px; }
</style>"#;
    both(text, &[("bal", json!(-0.5)), ("vol", json!(0.25))], |d| {
        // A range places its ticks from its last laid-out width.
        d.step(InputFrame::default());
        assert!(d.el("Steps").is_none(), "a datalist draws nothing");
        // A tick at each option, at its fraction of the track.
        let rail = under(d, "Bal", "Rail");
        let ticks = count_under(d, "Bal", "Tick");
        assert_eq!(ticks.len(), 3, "one tick per option: {ticks:?}");
        for (t, f) in ticks.iter().zip([0.0, 0.5, 1.0]) {
            let centre = t.x + t.width / 2.0 - rail.x;
            assert!(
                (centre - f * rail.width).abs() <= 2.0,
                "tick at {f}: {centre} of {}",
                rail.width
            );
        }
        // Labels under the track; a value-only option is a tick only.
        let marks = count_under(d, "Bal", "Mark#");
        assert_eq!(marks.len(), 3);
        let texts: Vec<String> =
            d.ui.probe_elements()
                .into_iter()
                .filter(|e| {
                    e.path.contains("/Bal/") && e.path.contains("/Marks/") && !e.text.is_empty()
                })
                .map(|e| e.text)
                .collect();
        assert_eq!(texts, ["Left", "Right"]);
        // -0.5 from an origin of 0: the fill spans the quarter left of centre.
        let fill = under(d, "Bal", "Fill");
        assert!(
            (fill.width - rail.width * 0.25).abs() <= 1.5,
            "fill {} of {}",
            fill.width,
            rail.width
        );
        assert!(
            (fill.x - rail.x - rail.width * 0.25).abs() <= 2.0,
            "the fill starts at -0.5: {} from {}",
            fill.x,
            rail.x
        );
        // Disabled: drawn so, and a drag changes nothing.
        let vol = under(d, "Vol", "Track");
        assert!(vol.disabled, "the range is disabled");
        let at = Vec2::new(vol.x + vol.width * 0.9, vol.y + vol.height / 2.0);
        d.click_at(at);
        assert_eq!(
            d.read("vol"),
            json!(0.25),
            "a disabled range takes no press"
        );
    });
    let missing = refused(
        r#"<meta name="component" content="t"><div><input type="range" list="Nope" /></div>"#,
    );
    assert!(missing.contains("names no <datalist>"), "{missing}");
    let words = refused(
        r#"<meta name="component" content="t"><div><input type="range" list="L" /><datalist id="L"><option value="loud"></option></datalist></div>"#,
    );
    assert!(words.contains("lists numbers"), "{words}");
}

#[test]
fn switch_parts_drive_knob_mark_and_travel_after_resize() {
    let source = r#"<meta name="component" content="t">
<div><input type="checkbox" switch id="Switch" class:large={$large} bind:checked={$on} /></div>
<style>
#Switch { width: 240px; height: 80px; }
#Switch::part(row) { padding: 0; }
#Switch::part(track) { width: 80px; height: 40px; }
#Switch::part(knob) { width: 32px; height: 32px; }
#Switch.large::part(track) { width: 50%; height: 75%; }
#Switch.large::part(knob) { width: 40%; height: 80%; }
</style>"#;
    both(
        source,
        &[("large", json!(false)), ("on", json!(false))],
        |d| {
            let mut theme = *d.ui.theme();
            theme.reduced_motion = true;
            d.ui.set_theme(theme);
            for (large, on, diameter, inset, travel, tick) in [
                (false, false, 32.0, 4.0, 0.0, 20.0),
                (false, true, 32.0, 4.0, 40.0, 20.0),
                (true, true, 48.0, 7.0, 58.0, 30.0),
                (true, false, 48.0, 7.0, 0.0, 30.0),
            ] {
                d.set("large", json!(large));
                d.set("on", json!(on));
                let track = under(d, "Switch", "Track");
                let knob = under(d, "Switch", "Knob");
                let mark = under(d, "Switch", "Mark");
                let glyph = under(d, "Switch", "Tick");
                let scale = if on { 1.0 } else { 0.5 };
                assert_eq!(
                    (mark.width, mark.height),
                    (diameter * scale, diameter * scale)
                );
                assert_eq!((glyph.width, glyph.height), (tick * scale, tick * scale));
                assert!((knob.x - track.x - inset - travel).abs() < 0.01);
                assert!((knob.y - track.y - inset).abs() < 0.01);
            }
            let geometry = |d: &Doc| {
                ["Track", "Knob", "Mark", "Tick"].map(|id| {
                    let node = under(d, "Switch", id);
                    (node.x, node.y, node.width, node.height)
                })
            };
            let expected = geometry(d);
            let saved = d.ui.save_state();
            d.ui.restore_state(&saved).unwrap();
            d.step(InputFrame::default());
            assert_eq!(geometry(d), expected, "same-instance restore");
            let mut fresh = Doc::new(
                source,
                &[("large", json!(true)), ("on", json!(false))],
                d.mounted,
            );
            fresh.ui.set_theme(*d.ui.theme());
            fresh.ui.restore_state(&saved).unwrap();
            fresh.step(InputFrame::default());
            assert_eq!(geometry(&fresh), expected, "fresh-instance restore");
            let automatic = source
                .lines()
                .filter(|line| !line.contains("::part(knob)"))
                .collect::<Vec<_>>()
                .join("\n");
            let automatic = Doc::new(
                &automatic,
                &[("large", json!(true)), ("on", json!(true))],
                d.mounted,
            );
            let knob = under(&automatic, "Switch", "Knob");
            let tick = under(&automatic, "Switch", "Tick");
            assert_eq!(
                (knob.width, knob.height),
                (46.0, 46.0),
                "unauthored knob follows the 60px track on the first frame"
            );
            assert_eq!((tick.width, tick.height), (29.0, 29.0));
        },
    );
}

#[test]
fn a_checkbox_follows_its_label_order_size_and_description() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <label id="After"><input type="checkbox" switch bind:checked={$a}> After</label>
  <label id="Before">Before <input type="checkbox" switch><small>Plays in menus</small></label>
  <label id="Flip" class="rev">Flipped <input type="checkbox" switch></label>
  <input type="checkbox" switch id="Late"><label for="Late">Late</label>
  <label id="Big" class="big">Big <input type="checkbox" switch disabled></label>
  <label id="Small">Small <input type="checkbox" switch></label>
</div>
<style>
#Page { display: flex; flex-direction: column; align-items: start; gap: 12px; padding: 20px; }
.rev { flex-direction: row-reverse; }
.big::part(track) { width: 55px; height: 32px; }
</style>"#;
    both(text, &[("a", json!(false))], |d| {
        let side = |d: &Doc, id: &str| {
            let label = under(d, id, "Label");
            let track = under(d, id, "Track");
            if label.x < track.x { "start" } else { "end" }
        };
        assert_eq!(
            side(d, "After"),
            "end",
            "a caption written after the switch sits after it"
        );
        assert_eq!(
            side(d, "Before"),
            "start",
            "a caption written first sits first"
        );
        assert_eq!(side(d, "Flip"), "end", "row-reverse swaps it");
        assert_eq!(side(d, "Late"), "end", "a <label for> after its checkbox");
        let before = under(d, "Before", "Label");
        assert_eq!(before.text, "Before", "the <small> is not in the caption");
        let note = under(d, "Before", "Description");
        assert_eq!(note.text, "Plays in menus");
        assert!(note.y > before.y, "the description is under the caption");
        let (big, small) = (under(d, "Big", "Track"), under(d, "Small", "Track"));
        assert!(
            big.height > small.height,
            "the styled track is larger: {} vs {}",
            big.height,
            small.height
        );
        assert!(d.get("Big").disabled, "disabled reaches the switch");
    });
    let two = refused(
        r#"<meta name="component" content="t"><div><label>A <input type="checkbox"><small>x</small><small>y</small></label></div>"#,
    );
    assert!(two.contains("one plain <small>"), "{two}");
}

#[test]
fn button_dimensions_follow_media_and_text_scale() {
    let source = r#"<meta name="component" content="t"><div><button id="Action">Apply</button></div><style>
#Action, #Action::part(base) { height: 2rem; }
#Action::part(base) { padding: 0 calc(6rem / 7); }
@media (pointer: coarse) { #Action, #Action::part(base) { height: calc(22rem / 7); } }
@media (-og-input: pad) { #Action, #Action::part(base) { height: calc(20rem / 7); } }
</style>"#;
    both(source, &[], |d| {
        let original = *d.ui.theme();
        for (density, scale, height) in [
            (DensityMode::Mouse, 1.0, 28.0),
            (DensityMode::Touch, 1.0, 44.0),
            (DensityMode::Pad, 1.0, 40.0),
            (DensityMode::Pad, 1.5, 60.0),
        ] {
            let mut theme = original;
            theme.density = density;
            theme.text_scale = scale;
            d.ui.set_theme(theme);
            d.step(InputFrame::default());
            assert_eq!(d.get("Action").height, height);
            assert!(!d.get("Action").clipped);
        }
        let mut theme = original;
        theme.density = DensityMode::Auto;
        theme.text_scale = 1.0;
        d.ui.set_theme(theme);
        d.viewport.width = 390.0;
        d.viewport.coarse_pointer = false;
        d.step(InputFrame::default());
        assert_eq!(
            d.get("Action").height,
            28.0,
            "a narrow mouse window keeps fine-pointer sizing"
        );
        d.viewport.coarse_pointer = true;
        d.step(InputFrame::default());
        assert_eq!(
            d.get("Action").height,
            44.0,
            "a phone's coarse pointer selects touch sizing"
        );
    });
}

#[test]
fn dialogs_close_as_closedby_says() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <dialog id="Std" modal bind:open={$std} aria-label="Std"><button id="InStd">Ok</button></dialog>
  <dialog id="Any" modal closedby="any" bind:open={$any} aria-label="Any"><button id="InAny">Ok</button></dialog>
  <dialog id="Lock" modal closedby="none" bind:open={$lock} aria-label="Lock"><button id="InLock">Ok</button></dialog>
  <dialog id="Place" bind:open={$place} aria-label="Place"><button id="InPlace">Ok</button></dialog>
  <dialog id="Bare" bind:open={$bare} aria-label="Bare"><button id="InBare">Ok</button></dialog>
</div>
<style>
#Bare { appearance: none; }
</style>"#;
    let stores = [
        ("std", json!(false)),
        ("any", json!(false)),
        ("lock", json!(false)),
        ("place", json!(false)),
        ("bare", json!(false)),
    ];
    // Closing writes `open` back to its store: a mounted document's.
    {
        let d = &mut Doc::new(text, &stores, true);
        let corner = Vec2::new(6.0, 6.0);
        // HTML's default for a modal: closerequest. The backdrop keeps it.
        d.set("std", json!(true));
        for _ in 0..30 {
            d.step(InputFrame::default());
        }
        d.click_at(corner);
        assert_eq!(
            d.read("std"),
            json!(true),
            "a press outside leaves a default modal open"
        );
        d.click("InStd");
        key(d, UiKey::Escape);
        assert_eq!(d.read("std"), json!(false), "Escape closes it");
        // any: light dismiss.
        d.set("any", json!(true));
        // A backdrop press counts once the modal has entered.
        for _ in 0..30 {
            d.step(InputFrame::default());
        }
        d.click_at(corner);
        assert_eq!(
            d.read("any"),
            json!(false),
            "closedby=any closes on a press outside"
        );
        // none: only the document closes it.
        d.set("lock", json!(true));
        d.click("InLock");
        key(d, UiKey::Escape);
        d.click_at(corner);
        assert_eq!(
            d.read("lock"),
            json!(true),
            "closedby=none ignores Escape and the backdrop"
        );
        d.set("lock", json!(false));
        // In place: HTML's default is none, even with the focus inside.
        d.set("place", json!(true));
        d.click("InPlace");
        key(d, UiKey::Escape);
        assert_eq!(
            d.read("place"),
            json!(true),
            "an in-place dialog keeps Escape by default"
        );
        // appearance: none drops its frame: the button sits at its corner.
        d.set("bare", json!(true));
        let (bare, inside) = (d.get("Bare"), d.get("InBare"));
        assert!(
            (inside.x - bare.x).abs() < 0.5 && (inside.y - bare.y).abs() < 0.5,
            "no padding: {inside:?} in {bare:?}"
        );
        assert!(!bare.opaque, "no fill of its own");
        let framed = (d.get("InPlace").x - d.get("Place").x).abs();
        assert!(framed > 4.0, "a framed one keeps its padding: {framed}");
    }
}

#[test]
fn datalist_filter_uses_translated_labels() {
    struct Env;
    impl bind::Env for Env {
        fn lookup(&self, _: &str) -> Option<Value> {
            None
        }
        fn translate(&self, key: &str) -> Option<String> {
            (key == "map.bay").then(|| "Baie".into())
        }
    }
    for obsolete in ["-og-select-filter: on", "-og-select-filter: off"] {
        let source = format!(
            "<meta name=\"component\" content=\"t\"><select/><style>select {{ {obsolete}; }}</style>"
        );
        assert!(UiDocument::parse("ui/t.svelte", &source).is_err());
    }
    for internal in ["suggestion-labels", "suggestion-keys"] {
        let source = format!("<meta name=\"component\" content=\"t\"><input {internal}=\"Bay\" />");
        assert!(UiDocument::parse("ui/t.svelte", &source).is_err());
    }
    let text = r#"<meta name="component" content="t">
<div><input id="Q" list="Maps" value="bai" placeholder="Find a map" />
<datalist id="Maps"><option value="map-17" i18n="map.bay">Bay</option><option value="map-82">Fort</option></datalist></div>"#;
    for mounted in [false, true] {
        let mut ui = crate::support::ui_with_font(crate::support::test_font_metrics());
        ui.define_document(UiDocument::parse("ui/t.svelte", text).unwrap());
        if mounted {
            ui.mount_document(
                reactive::SourceId::new("test", "t"),
                "t",
                Default::default(),
                Default::default(),
                &Env,
            )
            .unwrap();
        }
        for _ in 0..3 {
            ui.begin(InputFrame::default(), Viewport::new(800.0, 900.0));
            if mounted {
                ui.build_retained_documents(&Env).unwrap();
            } else {
                ui.component("T", "t", &mut json!({}), &Env);
            }
            let field = ui.probe_elements().into_iter().find(|e| e.id == "Q");
            if let Some(field) = field {
                ui.set_focus(&field.path);
            }
            ui.end();
        }
        let options: Vec<_> = ui
            .probe_elements()
            .into_iter()
            .filter(|e| e.path.contains("~list/Option#") && !e.text.is_empty())
            .map(|e| e.text)
            .collect();
        assert_eq!(
            options,
            ["Baie"],
            "filter matches translated labels, mounted={mounted}"
        );
    }
}

#[test]
fn a_select_takes_aria_invalid_and_its_picker_height() {
    let options: String = (0..20)
        .map(|i| format!("<option value=\"{i}\">Option {i}</option>"))
        .collect();
    let text = format!(
        r#"<meta name="component" content="t">
<div id="Page">
  <select id="Bad" aria-invalid={{$bad}} bind:value={{$v}}>{options}</select>
  <small id="Why">Pick one</small>
</div>
<style>
#Page {{ display: flex; flex-direction: column; gap: 8px; padding: 20px; }}
select {{ width: 260px; }}
select::picker(select) {{ max-height: 150px; padding: 9px; }}
</style>"#
    );
    both(&text, &[("bad", json!(true)), ("v", Value::Null)], |d| {
        let parts: Vec<String> =
            d.ui.parts()
                .iter()
                .flatten()
                .map(|p| format!("{p}"))
                .collect();
        assert!(
            parts
                .iter()
                .any(|p| p.starts_with("select::part(trigger)") && p.contains("invalid")),
            "aria-invalid marks the trigger: {parts:?}"
        );
        assert!(
            d.el("Why").is_some_and(|e| e.text == "Pick one"),
            "its message is the document's"
        );
        d.click("Bad/Trigger");
        let list =
            d.ui.probe_elements()
                .into_iter()
                .find(|e| e.id == "List" && e.path.contains("Bad"))
                .expect("the open list");
        assert!(
            list.height <= 150.5 && list.height > 100.0,
            "the picker's max-height bounds it: {}",
            list.height
        );
        // Its other box properties style the open list as a box's do.
        let trigger = d.el("Bad/Trigger").expect("the trigger");
        assert!(
            (list.x - trigger.x - 9.0).abs() < 0.5,
            "the picker's padding insets its rows: {} vs {}",
            list.x,
            trigger.x
        );
        d.set("bad", json!(false));
        let parts: Vec<String> =
            d.ui.parts()
                .iter()
                .flatten()
                .map(|p| format!("{p}"))
                .collect();
        assert!(
            !parts
                .iter()
                .any(|p| p.starts_with("Dropdown.trigger") && p.contains("invalid")),
            "unmarked once valid: {parts:?}"
        );
    });
}

#[test]
fn an_image_shows_its_object_view_box() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <img id="Crop" alt="" style:--u0={0.4} style:--v0={0.1} style:--u1={0.8} style:--v1={0.7} />
  <img id="Raw" alt="" />
</div>
<style>
img { width: 64px; height: 64px; }
#Crop { object-view-box: inset(calc(var(--v0) * 100%) calc((1 - var(--u1)) * 100%) calc((1 - var(--v1)) * 100%) calc(var(--u0) * 100%)); }
#Raw { object-view-box: inset(0% 50% 0% 0%); }
</style>"#;
    both(text, &[], |d| {
        let rect_of = |d: &Doc, id: &str| {
            let e = d.get(id);
            d.ui.frame()
                .items
                .iter()
                .find_map(|item| match item.cmd {
                    DrawCmd::Image { rect, uv, .. }
                        if (rect.x - e.x).abs() < 0.5 && (rect.y - e.y).abs() < 0.5 =>
                    {
                        Some(uv)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("no image drawn for {id}"))
        };
        let close = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5);
        let crop = rect_of(d, "Crop");
        assert!(
            close(crop, [0.4, 0.1, 0.8, 0.7]),
            "inset(top right bottom left): {crop:?}"
        );
        let raw = rect_of(d, "Raw");
        assert!(
            close(raw, [0.0, 0.0, 0.5, 1.0]),
            "right half cropped: {raw:?}"
        );
    });
    let px = refused(
        r#"<meta name="component" content="t"><div><img alt="" /></div><style>img { object-view-box: inset(4px); }</style>"#,
    );
    assert!(px.contains("percentages"), "{px}");
    let removed = refused(
        r#"<meta name="component" content="t"><img alt="" /><style>img { -og-uv: 0 0 0.5 1; }</style>"#,
    );
    assert!(removed.contains("-og-uv"), "{removed}");
}

#[test]
fn data_og_instant_settles_only_the_first_entrance() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  {#if $shown}<div id="Gate" data-og-instant class="in">Ready</div>{/if}
  {#if $shown}<div id="Plain" class="in">Ready</div>{/if}
</div>
<style>
.in { -og-enter: fade 0px 400ms; width: 100px; height: 40px; background: #ffffff; }
</style>"#;
    both(text, &[("shown", json!(true))], |d| {
        // Built at the first frame: a frame on, the instant one is settled
        // and the plain one is still fading in.
        d.step(InputFrame::default());
        d.step(InputFrame::default());
        // Its fill's alpha as drawn: the fade's opacity.
        let opacity = |d: &Doc, id: &str| {
            let e = d.get(id);
            d.ui.frame()
                .items
                .iter()
                .find_map(|item| match item.cmd {
                    DrawCmd::Quad { rect, color, .. }
                        if (rect.x - e.x).abs() < 0.5
                            && (rect.y - e.y).abs() < 0.5
                            && (rect.width - e.width).abs() < 0.5 =>
                    {
                        Some(color[3])
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("no quad for {id} {e:?}"))
        };
        assert!(
            opacity(d, "Gate") > 0.99,
            "settled at once: {}",
            opacity(d, "Gate")
        );
        assert!(
            opacity(d, "Plain") < 0.9,
            "the plain one enters: {}",
            opacity(d, "Plain")
        );
        // Hidden, then shown again: now both play their entrance.
        d.set("shown", json!(false));
        for _ in 0..60 {
            d.step(InputFrame::default());
        }
        d.set("shown", json!(true));
        assert!(
            opacity(d, "Gate") < 0.9,
            "a later entrance plays: {}",
            opacity(d, "Gate")
        );
    });
}

/// `<button>` holding content (G16): its icon, text and badge lie in a row
/// centred in its frame and its press is the button's. Ways it could fail:
/// the content laid out beside the button instead of in it, a column
/// instead of a row, or a press on the icon not clicking the button.
#[test]
fn a_button_holds_its_content_in_a_row() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <button id="Buy" variant="primary" onclick={buy}><og-icon id="Coin" icon="coin" />Buy<badge id="N">3</badge></button>
  <button id="Plain">Plain</button>
</div>"#;
    both(text, &[], |d| {
        let (buy, coin, n) = (d.get("Buy"), d.get("Coin"), d.get("N"));
        let words =
            d.ui.probe_elements()
                .into_iter()
                .find(|e| e.path.contains("/Buy/") && e.text == "Buy")
                .expect("the text in the button");
        for e in [&coin, &words, &n] {
            assert!(
                e.x >= buy.x
                    && e.x + e.width <= buy.x + buy.width + 0.5
                    && e.y >= buy.y
                    && e.y + e.height <= buy.y + buy.height + 0.5,
                "{} inside the button: {e:?} in {buy:?}",
                e.id
            );
        }
        assert!(coin.x < words.x && words.x < n.x, "a row in source order");
        let mid = |e: &UiElement| e.y + e.height / 2.0;
        assert!((mid(&coin) - mid(&buy)).abs() < 1.0 && (mid(&words) - mid(&buy)).abs() < 1.0);
        assert_eq!(
            d.get("Plain").text,
            "Plain",
            "a text-only button is unchanged"
        );
        let events = d.click("Coin");
        assert!(
            events.iter().any(|e| e.name == "buy"),
            "a press on its icon clicks it: {events:?}"
        );
    });
}

/// A `<button>` whose content stands taller than its control height (a
/// loadout row: a name over its detail) grows to hold it, as an HTML
/// button does; its control height stays the least it takes, and an
/// authored `height` still holds. Ways it could fail: the content spilling
/// below a fixed control height, a short content button shrinking under
/// its control height, or the authored height lost to the fit.
#[test]
fn a_button_grows_to_hold_tall_content() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <button id="Row"><column id="Col"><text id="Name">Name</text><text id="Detail">Detail</text></column></button>
  <button id="Short"><og-icon id="Ic" icon="coin" /></button>
  <button id="Given"><column><text>Name</text><text>Detail</text></column></button>
</div>
<style>
#Page { display: flex; flex-direction: column; align-items: flex-start; }
#Name, #Detail { font-size: 40px; }
#Given { height: 30px; }
</style>"#;
    both(text, &[], |d| {
        let (row, col) = (d.get("Row"), d.get("Col"));
        assert!(
            col.height >= 60.0 && col.y >= row.y && col.y + col.height <= row.y + row.height + 0.5,
            "the row holds its two lines: {col:?} in {row:?}"
        );
        let plain = d.get("Short").height;
        assert!(
            plain >= 24.0,
            "a short content button keeps its control height: {plain}"
        );
        assert_eq!(d.get("Given").height, 30.0, "an authored height holds");
    });
}

/// A declared action's handler on each `{#each}` row acts for the row
/// holding the focus only, as a key event reaches the focused element and
/// its ancestors; a handler outside the rows is the page's and acts
/// wherever the focus is. Ways it could fail: every row's handler firing
/// at once (the loadout's per-row Equip), the focused row's handler
/// missed when the focus is on an element inside the row rather than the
/// row itself, a row firing while the focus is outside every row, or the
/// page's handler scoped away.
#[test]
fn row_action_handlers_take_the_action_for_the_focused_row() {
    // A screen on a layer: a mounted document takes actions through its layer.
    let text = r#"<meta name="screen" content="t" data-layer="hud">
<meta name="action:details" content="Equip">
<meta name="action:back" content="Back">
<div id="Page">
  {#each $rows as r (r.id)}
    <div id="Row" ondetails={equip}><button id="Pick">{r.id}</button></div>
  {/each}
  <button id="Back" onback={back}>Back</button>
</div>"#;
    let rows = json!([{"id": "a"}, {"id": "b/2"}, {"id": "c"}]);
    both(text, &[("rows", rows.clone())], |d| {
        let fire = |d: &mut Doc, focus: &str, action: &str| -> Vec<(String, String)> {
            let path =
                d.ui.probe_elements()
                    .into_iter()
                    .find(|e| {
                        let (row, leaf) = focus.split_once(' ').unwrap_or(("", focus));
                        e.path.contains(row) && e.path.ends_with(leaf)
                    })
                    .unwrap_or_else(|| panic!("no {focus}"))
                    .path;
            d.focus = Some(path.clone());
            d.step(InputFrame::default());
            d.step(InputFrame::default());
            assert_eq!(d.ui.focus_path(0), path);
            d.events.clear();
            d.ui.fire_action(action);
            d.step(InputFrame::default());
            d.step(InputFrame::default());
            d.events
                .drain(..)
                .filter(|e| e.kind == "emit")
                .map(|e| (e.name, e.row_key))
                .collect()
        };
        assert_eq!(
            fire(d, "/Row#b /Pick", "details"),
            [("equip".to_string(), "b/2".to_string())],
            "the focused row's handler alone, its key holding a slash"
        );
        assert_eq!(
            fire(d, "/Row#c /Pick", "details"),
            [("equip".into(), "c".into())]
        );
        assert_eq!(fire(d, "/Back", "details"), [], "no row holds the focus");
        assert_eq!(
            fire(d, "/Row#a /Pick", "back"),
            [("back".to_string(), String::new())],
            "the page's handler acts from inside a row"
        );
    });
}

/// `<tabs icons badges>`: each tab draws its icon before its label and its
/// count badge after it; an empty entry draws none. The tabPrev and tabNext
/// actions (Q and E, LB and RB) step through the tabs.
///
/// Ways it could fail: the lists ignored, a badge on every tab, a badge
/// the tab's width leaves out (cut by its neighbour), or a bumper that
/// stops at the last tab.
#[test]
fn tabs_take_icons_and_badges() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <tabs id="Tabs" items="Gear Quests" icons="sword shield" badges={$badges} bind:selected={$tab}></tabs>
</div>"#;
    let d = &mut Doc::new(
        text,
        &[("tab", json!(0)), ("badges", json!(["", "3"]))],
        false,
    );
    let el = |d: &Doc, end: &str| {
        d.ui.probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with(end))
    };
    assert!(el(d, "Item#0/Icon").is_some(), "the first tab has its icon");
    assert!(el(d, "Item#0/Badge").is_none(), "an empty badge is none");
    let (badge, tab) = (
        el(d, "Item#1/Badge").expect("the second tab's badge"),
        el(d, "Item#1").unwrap(),
    );
    assert_eq!(badge.text, "3");
    assert!(
        badge.x + badge.width <= tab.x + tab.width + 0.5,
        "the tab holds its badge: {badge:?} in {tab:?}"
    );
    // The bumpers: tabNext steps, wrapping past the last; tabPrev back.
    let d = &mut Doc::new(text, &[("tab", json!(0)), ("badges", Value::Null)], true);
    let press = |d: &mut Doc, key: &'static str| {
        d.step(InputFrame {
            keys_held: vec![key],
            keys_pressed: vec![key],
            ..InputFrame::default()
        });
        d.step(InputFrame::default());
    };
    press(d, "KeyE");
    assert_eq!(d.read("tab"), json!(1), "tabNext steps");
    press(d, "KeyE");
    assert_eq!(d.read("tab"), json!(0), "and wraps");
    press(d, "KeyQ");
    assert_eq!(d.read("tab"), json!(1), "tabPrev steps back, wrapping");
}

/// ARIA in documents (G4): `role` and the `aria-*` states and values reach
/// the accessibility tree, `aria-labelledby`/`aria-describedby` read the
/// text of the elements they name (and follow it when it is bound), and
/// a role og.ui has not is refused. Ways it could fail: a role kept to the
/// kind's, a state dropped (pressed, current, busy, expanded), a range
/// without its bounds, a referenced label's text not followed.
#[test]
fn aria_attributes_reach_the_accessibility_tree() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <h2 id="Title">{$title}</h2>
  <p id="Help">Pick a tab</p>
  <div id="Tabs" role="tablist" aria-labelledby="Title" aria-describedby="Help">
    <button id="One" role="tab" aria-selected={$tab == 1} aria-current="page">One</button>
    <button id="Two" role="tab" aria-selected={$tab == 2}>Two</button>
  </div>
  <button id="Chip" aria-pressed={$on} aria-busy="true">Filter</button>
  <div id="Bar" role="progressbar" aria-valuemin="0" aria-valuemax="10" aria-valuenow={$n} aria-valuetext="3 of 10">x</div>
  <div id="Sep" role="separator" aria-expanded="false">Or</div>
</div>"#;
    both(
        text,
        &[
            ("title", json!("Modes")),
            ("tab", json!(1)),
            ("on", json!(true)),
            ("n", json!(3)),
        ],
        |d| {
            let tabs = d.a11y("Tabs");
            assert_eq!(
                (tabs.role.as_str(), tabs.name.as_str()),
                ("tablist", "Modes")
            );
            assert_eq!(tabs.description, "Pick a tab");
            let one = d.a11y("One");
            assert_eq!(one.role, "tab");
            assert!(
                one.states.contains(&"selected".into()) && one.states.contains(&"current".into()),
                "{:?}",
                one.states
            );
            assert!(!d.a11y("Two").states.contains(&"selected".into()));
            let chip = d.a11y("Chip");
            assert!(
                chip.states.contains(&"pressed".into()) && chip.states.contains(&"busy".into()),
                "{:?}",
                chip.states
            );
            let bar = d.a11y("Bar");
            assert_eq!(
                (bar.role.as_str(), bar.value.as_str()),
                ("progressbar", "3 of 10")
            );
            assert_eq!(
                (bar.value_min, bar.value_max, bar.value_now),
                (Some(0.0), Some(10.0), Some(3.0))
            );
            let sep = d.a11y("Sep");
            assert_eq!(sep.role, "separator");
            assert!(sep.states.contains(&"collapsed".into()));
            d.set("tab", json!(2));
            d.set("title", json!("Maps"));
            assert!(
                d.a11y("Two").states.contains(&"selected".into()),
                "a bound state follows"
            );
            assert_eq!(
                d.a11y("Tabs").name,
                "Maps",
                "a named element's bound text follows"
            );
        },
    );
    let bad = refused(r#"<meta name="component" content="t"><div role="banana">x</div>"#);
    assert!(bad.contains("not an accessibility role"), "{bad}");
}

/// Device-following glyphs (G7): `<kbd data-action>` shows the action's
/// key on a keyboard and its button on a pad, rebuilt when the device
/// changes; `$ui` carries the device, the coarse pointer and the action
/// bar; `data-og-rebind` captures the next key into its bound value.
/// Ways it could fail: a glyph stuck on the device it was built for, `$ui`
/// not reaching a mounted or an immediate document, the capture writing
/// Escape or never ending.
#[test]
fn kbd_glyphs_follow_the_device_and_rebind() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <kbd id="J" data-action="jump"></kbd>
  <kbd id="Esc">Esc</kbd>
  <p id="Dev">{$ui.device}</p>
  <p id="Count">{$ui.coarsePointer ? "touch" : "fine"}</p>
  <kbd id="R" data-action="jump" data-og-rebind bind:value={$key}></kbd>
</div>"#;
    both(text, &[("key", json!("KeyE"))], |d| {
        d.ui.define_action(ActionDesc {
            name: "jump".into(),
            keys: vec!["KeyE".into()],
            buttons: vec!["a".into()],
            label: "Jump".into(),
            ..ActionDesc::default()
        });
        d.ui.set_device(og_input::Device::KeyboardMouse);
        d.step(InputFrame::default());
        d.step(InputFrame::default());
        let key_cap = d.get("J").text;
        assert_eq!(d.get("Esc").text, "Esc");
        assert_eq!(d.get("Dev").text, "keyboardMouse");
        assert_eq!(d.get("Count").text, "fine");
        assert_eq!(d.a11y("J").name, "Jump", "named by the action");
        d.ui.set_device(og_input::Device::Xbox);
        d.step(InputFrame::default());
        d.step(InputFrame::default());
        assert_ne!(d.get("J").text, key_cap, "the glyph follows the device");
        assert_eq!(d.get("Dev").text, "xbox");
        d.ui.set_device(og_input::Device::KeyboardMouse);
        d.click("R");
        d.step(InputFrame {
            keys_pressed: vec!["KeyQ"],
            keys_held: vec!["KeyQ"],
            ..InputFrame::default()
        });
        d.step(InputFrame::default());
        if d.mounted {
            assert_eq!(d.read("key"), json!("KeyQ"), "the next key is its binding");
        }
        d.click("R");
        d.step(InputFrame {
            keys_pressed: vec!["Escape"],
            ..InputFrame::default()
        });
        d.step(InputFrame::default());
        if d.mounted {
            assert_eq!(d.read("key"), json!("KeyQ"), "Escape cancels");
        }
    });
}

/// Rich inline content (G6): `<img>`, `<og-icon>` and `<kbd data-action>`
/// among a paragraph's words draw as inline objects, a `<span part>` takes
/// its `::part()` size, line height and tracking (doc/mf2.rs tests the
/// translated markup). Ways it could fail: an inline object refused or
/// drawn as text, a part's size ignored.
#[test]
fn text_holds_inline_objects_sized_parts_and_translated_markup() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <p id="Line">Press <kbd data-action="jump"></kbd> for <img src="coin" alt=""> <og-icon name="coin"/> <span part="big">gold</span></p>
  <p id="Said">plain</p>
</div>
<style>
p::part(big) { font-size: 40px; line-height: 48px; letter-spacing: 0.1em; }
</style>"#;
    let doc = UiDocument::parse("ui/t.svelte", text).unwrap_or_else(|e| panic!("{e:?}"));
    let line = doc.nodes.iter().find(|n| n.id == "Line").unwrap();
    assert_eq!(line.inline.iter().filter(|r| r.atom.is_some()).count(), 3);
    assert_eq!(
        line.span_rules.len(),
        1,
        "p::part(big) styles the paragraph's span"
    );
    both(text, &[], |d| {
        let line = d.get("Line");
        assert!(line.text.starts_with("Press "), "{}", line.text);
        assert!(!line.text.contains('<'), "no markup as text: {}", line.text);
        // The 40 px part makes the line taller than the plain paragraph.
        assert!(
            line.height >= 47.0,
            "the part's line height: {}",
            line.height
        );
        assert!(d.get("Said").height < 40.0);
    });
    let refused_bound = refused(
        r#"<meta name="component" content="t"><div><p>a <img src={$x} alt=""> b</p></div>"#,
    );
    assert!(refused_bound.contains("literally"), "{refused_bound}");
}

/// `title` is a tooltip after a rest and the element's description; a
/// `popovertarget` press toggles its popover over the page against the
/// control, a press outside closes an auto one; an `interestfor` control
/// shows its hint while hovered and hides it once the pointer leaves.
///
/// Ways it could fail: a popover laid out in its place (or drawn while
/// closed), a toggle that only opens, light dismiss missing, a hint shown
/// with no interest or kept after it, a title that never shows, or a
/// mounted popover not rebuilt when another element opens it.
#[test]
fn titles_and_popovers_show_against_their_controls() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <button id="Help" title="Opens the help">?</button>
  <button id="Open" popovertarget="Menu">Menu</button>
  <div id="Menu" popover><button id="Item">One</button></div>
  <button id="Tip" interestfor="Card">i</button>
  <div id="Card" popover="hint"><p id="CardText">Details</p></div>
  <p id="Far">far away</p>
</div>
<style>
#Page { padding: 40px; gap: 40px; }
</style>"#;
    both(text, &[], |d| {
        let hover = |d: &mut Doc, id: &str, ticks: usize| {
            let e = d.get(id);
            let at = Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0);
            for _ in 0..ticks {
                d.step(InputFrame {
                    pointer: at,
                    pointer_inside: true,
                    ..InputFrame::default()
                });
            }
        };
        for _ in 0..5 {
            d.step(InputFrame::default());
        }
        assert!(d.el("Item").is_none(), "a closed popover builds nothing");
        assert!(d.el("CardText").is_none(), "no hint without interest");
        let far = d.get("Far");
        let open = d.get("Open");
        assert!(
            far.y < open.y + open.height + 200.0,
            "closed popovers take no room in the page"
        );
        // title: after a rest, a tooltip; always its description.
        assert_eq!(d.a11y("Help").description, "Opens the help");
        hover(d, "Help", 90);
        assert!(
            d.ui.probe_elements()
                .iter()
                .any(|e| e.text == "Opens the help"),
            "the title shows after a rest"
        );
        // popovertarget toggles; it is placed against its control.
        d.click("Open");
        let item = d.get("Item");
        let open = d.get("Open");
        assert!(
            item.y >= open.y + open.height - 0.5 && (item.x - open.x).abs() < 40.0,
            "under its control: {item:?} vs {open:?}"
        );
        assert_eq!(d.get("Far").y, far.y, "an open popover moves nothing");
        assert!(
            d.a11y("Open").states.iter().any(|s| s == "expanded"),
            "the control is expanded"
        );
        d.click("Open");
        assert!(d.el("Item").is_none(), "a second press closes it");
        d.click("Open");
        assert!(d.el("Item").is_some());
        // An auto popover closes on a press outside.
        d.click("Far");
        assert!(d.el("Item").is_none(), "light dismiss closes it");
        // interestfor: shown while hovered, gone once the pointer leaves.
        hover(d, "Tip", 90);
        let (tip, card) = (d.get("Tip"), d.get("CardText"));
        assert!(
            card.y > tip.y + tip.height - 0.5 || card.y + card.height < tip.y + 0.5,
            "the hint sits off its control: {card:?} vs {tip:?}"
        );
        hover(d, "Far", 20);
        assert!(
            d.el("CardText").is_none(),
            "the hint leaves with the interest"
        );
    });
}

/// The centre of the `::-og-arrow` drawn this frame (red in the test's
/// style), if one is.
fn arrow_at(d: &Doc) -> Option<(f32, f32)> {
    d.ui.frame().items.iter().find_map(|i| match &i.cmd {
        DrawCmd::Bar { center, color, .. }
            if color[0] > 0.9 && color[1] < 0.05 && color[2] < 0.05 =>
        {
            Some(*center)
        }
        _ => None,
    })
}

/// A popover is anchor-positioned against its implicit anchor, the control
/// that opened it: `position-area` picks its side, an inset offsets it,
/// `position-try-fallbacks` flips it off the safe area's edge, and its
/// `::-og-arrow` sits on the edge facing the control; `contextmenu` opens
/// one at the point of a secondary press.
///
/// Ways it could fail: an authored area ignored (today's hang-below
/// placement), an offset dropped, a fallback that never flips, an arrow on
/// the wrong edge or kept while closed, a context menu at its element
/// rather than the pointer, with an arrow or with items of their labels'
/// widths, or a primary press opening it.
#[test]
fn popovers_take_their_area_offset_fallback_arrow_and_point() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <button id="Open" popovertarget="Side">Side</button>
  <div id="Side" popover><p id="SideText">Beside it</p></div>
  <div id="Area" contextmenu="Ctx"><p>Right-click here for the menu</p></div>
  <div id="Gap"></div>
  <button id="Edge" popovertarget="Flip">Edge</button>
  <div id="Flip" popover><p id="FlipText">Flipped</p><p>over</p><p>it</p></div>
  <menu id="Ctx" popover><button id="Copy">Copy</button><button id="PasteAll">Paste all</button></menu>
</div>
<style>
#Page { padding: 40px; gap: 40px; }
#Gap { height: 640px; }
#Side { position-area: right; left: 12px; }
#Side::-og-arrow { width: 10px; background: #ff0000; }
#Flip { position-area: bottom; position-try-fallbacks: flip-block; }
#Ctx::-og-arrow { width: 10px; background: #ff0000; }
</style>"#;
    both(text, &[], |d| {
        for _ in 0..3 {
            d.step(InputFrame::default());
        }
        assert!(arrow_at(d).is_none(), "no arrow while closed");
        d.click("Open");
        d.step(InputFrame::default());
        let (open, side) = (d.get("Open"), d.get("SideText"));
        assert!(
            side.x >= open.x + open.width + 12.0 - 0.5,
            "right of its control, 12 px off: {side:?} vs {open:?}"
        );
        let (ax, ay) = arrow_at(d).expect("the arrow draws");
        assert!(
            (ax - (open.x + open.width + 12.0)).abs() < 1.0
                && (ay - (open.y + open.height / 2.0)).abs() < 1.0,
            "on the edge facing the control, at its middle: {:?} vs {open:?}",
            (ax, ay)
        );
        // At the bottom of the screen, a popover under its control flips above it.
        d.click("Edge");
        let (edge, flip) = (d.get("Edge"), d.get("FlipText"));
        assert!(
            flip.y + flip.height <= edge.y + 0.5,
            "flipped above its control: {flip:?} vs {edge:?}"
        );
        // A secondary press opens the context menu at the pointer.
        let area = d.get("Area");
        let at = Vec2::new(area.x + area.width * 0.75, area.y + area.height / 2.0);
        d.click("Area");
        assert!(d.el("Copy").is_none(), "a primary press opens no menu");
        d.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            secondary_pressed: true,
            ..InputFrame::default()
        });
        d.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            ..InputFrame::default()
        });
        let copy = d.get("Copy");
        assert!(
            (copy.x - at.x).abs() < 24.0 && copy.y >= at.y - 0.5 && copy.y < at.y + 24.0,
            "at the point: {copy:?} vs {at:?}"
        );
        // Its corner at the point, as a desktop's: no arrow, and its items
        // as wide as each other (a menu's rows).
        assert!(arrow_at(d).is_none(), "a context menu draws no arrow");
        assert_eq!(
            copy.width,
            d.get("PasteAll").width,
            "menu items span the menu"
        );
    });
}

/// The width of the `::-og-hold` fill drawn this frame (red in the test's
/// style), or 0.
fn hold_fill_width(d: &Doc) -> f32 {
    d.ui.frame()
        .items
        .iter()
        .find_map(|i| match &i.cmd {
            DrawCmd::Quad { rect, color, .. }
                if color[0] > 0.9 && color[1] < 0.05 && color[2] < 0.05 =>
            {
                Some(rect.width)
            }
            _ => None,
        })
        .unwrap_or(0.0)
}

/// HTML's core controls beyond the switch: a plain checkbox is a box with a
/// check (`indeterminate` a dash), radios share a `bind:group`, a number
/// input stays editable while its buttons step, `<menu>` holds menu items, and
/// `onhold`, `oncontextmenu`, `ontoggle` and `onclose` fire.
///
/// Ways it could fail: a plain checkbox still drawn as a switch, a radio
/// that writes when unchecked or fails to uncheck its sibling, a stepper
/// ignoring max, a hold firing at once or on every tick, a context menu on
/// a primary press, or toggle and close never firing.
#[test]
fn core_controls_choose_step_hold_and_report() {
    for declaration in [
        "-og-stepper: buttons",
        "-og-stepper-wrap: wrap",
        "-og-stepper-large-step: 10",
    ] {
        assert!(refused(&format!("<meta name=\"component\" content=\"t\"><input type=\"number\"/><style>input {{ {declaration}; }}</style>")).contains("-og-stepper"));
    }
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <label><input type="checkbox" id="Box" bind:checked={$box}> Box</label>
  <label><input type="checkbox" id="Mixed" indeterminate> Some</label>
  <label><input type="checkbox" switch id="Sw" data-on="On" data-off="Off"> Switch</label>
  <label><input type="radio" id="Easy" name="lv" value="easy" bind:group={$level}> Easy</label>
  <label><input type="radio" id="Hard" name="lv" value="hard" bind:group={$level}> Hard</label>
  <input type="number" id="Qty" max="3" data-format="{value, number} m" bind:value={$qty}>
  <input type="number" id="Lv" min="0" max="20" step="2" data-wrap data-large-step="6" bind:value={$lv}>
  <menu id="Menu"><button id="Copy">Copy</button></menu>
  <button id="Buy" onhold={buy} data-hold="300" oncontextmenu={more}>Buy</button>
  <details id="More" summary="More" ontoggle={toggled}><p>x</p></details>
</div>
<style>
#Page { gap: 8px; padding: 8px; }
#Buy::-og-hold { background: #ff0000; }
</style>"#;
    let stores = [
        ("box", json!(false)),
        ("level", json!("easy")),
        ("qty", json!("2")),
        ("lv", json!("20")),
    ];
    let mut scene = og_render::RenderScene::new();
    let d = &mut Doc::with_font(
        text,
        &stores,
        true,
        crate::support::add_test_font(&mut scene),
    );
    for _ in 0..5 {
        d.step(InputFrame::default());
    }
    assert_eq!(d.a11y("Box").role, "checkbox", "a plain checkbox is a box");
    assert_eq!(d.a11y("Sw").role, "switch");
    assert_eq!(d.a11y("Sw").value, "Off", "data-off is its value while off");
    assert_eq!(d.a11y("Mixed").value, "mixed", "indeterminate");
    d.click("Box");
    assert_eq!(d.read("box"), json!(true));
    // Radios: the bound group checks one; a press on the other moves it.
    assert_eq!(d.a11y("Easy").role, "radiogroup");
    d.click("Hard");
    assert_eq!(d.read("level"), json!("hard"));
    d.click("Hard");
    assert_eq!(d.read("level"), json!("hard"), "a checked radio stays");
    d.click("Easy");
    assert_eq!(d.read("level"), json!("easy"));
    // The stepper's plus steps to max and no further.
    let plus =
        d.ui.probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("/Qty/Increment"))
            .expect("the stepper's plus");
    let at = Vec2::new(plus.x + plus.width / 2.0, plus.y + plus.height / 2.0);
    d.click_at(at);
    d.click_at(at);
    assert_eq!(d.read("qty"), json!("3"), "steps to max and stops");
    d.click("Copy");
    assert!(
        d.ui.frame()
            .items
            .iter()
            .any(|item| matches!(&item.cmd, DrawCmd::Text { text, .. } if text == "3 m")),
        "data-format presents the value while unfocused"
    );
    // data-wrap goes past max to min.
    let plus =
        d.ui.probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("/Lv/Increment"))
            .expect("the wrapping stepper's plus");
    d.click_at(Vec2::new(
        plus.x + plus.width / 2.0,
        plus.y + plus.height / 2.0,
    ));
    assert_eq!(d.read("lv"), json!("0"), "wraps past max to min");
    d.step(InputFrame {
        keys: vec![og_ui::UiKey::Up],
        shift: true,
        ..InputFrame::default()
    });
    assert_eq!(
        d.read("lv"),
        json!("6"),
        "data-large-step sets Shift adjustment"
    );
    assert_eq!(d.a11y("Copy").role, "menuitem");
    // Hold: fires once, after 300 ms, not at once.
    let buy = d.get("Buy");
    let at = Vec2::new(buy.x + buy.width / 2.0, buy.y + buy.height / 2.0);
    let mut holds = Vec::new();
    let mut fills = Vec::new();
    for tick in 0..40 {
        d.step(InputFrame {
            pointer: at,
            pointer_inside: true,
            down: true,
            pressed: tick == 0,
            ..InputFrame::default()
        });
        if d.events.iter().any(|e| e.name == "buy") {
            holds.push(tick);
        }
        d.events.clear();
        fills.push(hold_fill_width(d));
    }
    d.step(InputFrame {
        pointer: at,
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    });
    assert_eq!(holds.len(), 1, "one hold per press: {holds:?}");
    assert!((15..20).contains(&holds[0]), "at 300 ms: {holds:?}");
    // The `::-og-hold` fill grows from the start edge across the button
    // while held, full at the hold, and leaves with the press.
    assert!(
        fills[2] > 0.0 && fills[2] < fills[10] && fills[10] < buy.width,
        "the fill grows: {fills:?}"
    );
    assert!(
        (fills[30] - buy.width).abs() < 1.0,
        "full at the hold: {fills:?}"
    );
    assert_eq!(hold_fill_width(d), 0.0, "released, no fill");
    // A secondary press asks for the context menu; a primary one does not.
    let events = d.click("Buy");
    assert!(!events.iter().any(|e| e.name == "more"));
    d.step(InputFrame {
        pointer: at,
        pointer_inside: true,
        secondary_pressed: true,
        ..InputFrame::default()
    });
    assert!(d.events.iter().any(|e| e.name == "more"), "{:?}", d.events);
    // ontoggle on a disclosure.
    d.events.clear();
    let events = d.click("More");
    assert!(events.iter().any(|e| e.name == "toggled"), "{events:?}");
}

/// The counter counts to its value, the typewriter fires `onrevealed`
/// once, `data-cycle` steps its options and writes the value back,
/// `og-spotlight` dims the page but for its target, and `<input
/// type="color">` opens a picker whose hex writes `#rrggbb` back.
///
/// Ways it could fail: a count that jumps or never arrives, a reveal shown
/// whole at once or firing every tick, a cycler that does not wrap or skips
/// the bound value, a spotlight without its callout, or a colour input that
/// writes a malformed value.
#[test]
fn counters_reveals_cyclers_spotlights_and_colours() {
    for rejected in [
        "<p count=\"3\" />",
        "<og-counter count-duration=\"200\" />",
        "<p reveal=\"20\" />",
        "<dialog spotlight=\"Target\" />",
        "<p/><style>p { -og-count-duration: 200; }</style>",
        "<p/><style>p { -og-count-decimals: 1; }</style>",
        "<p/><style>p { -og-reveal: 20; }</style>",
        "<dialog/><style>dialog { -og-spotlight: Target; }</style>",
    ] {
        let source = format!("<meta name=\"component\" content=\"t\">{rejected}");
        assert!(
            UiDocument::parse("ui/t.svelte", &source).is_err(),
            "{rejected}"
        );
    }
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <og-counter id="Score" value={$score} duration="200" decimals="1">0</og-counter>
  <og-typewriter id="Line" rate="20" onrevealed={done}>Hello there</og-typewriter>
  <select id="Diff" data-cycle bind:value={$diff}><option value="easy">Easy</option><option value="hard">Hard</option></select>
  <button id="Target">Here</button>
  <og-spotlight id="Tour" bind:open={$tour} target="Target"><div id="Step"><p id="Callout">Press here</p></div></og-spotlight>
  <input type="color" id="Tint" bind:value={$tint}>
</div>
<style>
#Page { gap: 8px; padding: 8px; }
</style>"#;
    let stores = [
        ("score", json!(0)),
        ("diff", json!("easy")),
        ("tint", json!("#3366cc")),
        ("tour", json!(false)),
    ];
    let d = &mut Doc::new(text, &stores, true);
    d.step(InputFrame::default());
    assert!(
        d.get("Line").text.len() < "Hello there".len(),
        "typed out, not whole"
    );
    let mut revealed = 0;
    for _ in 0..60 {
        d.step(InputFrame::default());
        revealed += d.events.iter().filter(|e| e.name == "done").count();
        d.events.clear();
    }
    assert_eq!(d.get("Line").text, "Hello there");
    assert_eq!(revealed, 1, "onrevealed fires once");
    d.set("score", json!(25.5));
    d.step(InputFrame::default());
    let mid: f64 = d.get("Score").text.parse().unwrap_or(-1.0);
    assert!(mid > 0.0 && mid < 25.5, "counting: {mid}");
    for _ in 0..12 {
        d.step(InputFrame::default());
    }
    assert_eq!(
        d.get("Score").text,
        "25.5",
        "duration and decimal attributes apply"
    );
    d.click("Diff/Next");
    assert_eq!(d.read("diff"), json!("hard"));
    d.click("Diff/Next");
    assert_eq!(d.read("diff"), json!("easy"), "wraps");
    d.click("Tint");
    let hex =
        d.ui.probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("/Hex"))
            .expect("the picker opened");
    assert_eq!(hex.text.to_lowercase(), "#3366cc");
    // The spotlight's shade takes presses outside its hole.
    d.set("tour", json!(true));
    for _ in 0..5 {
        d.step(InputFrame::default());
    }
    assert!(
        d.el("Callout").is_some(),
        "the spotlight's callout shows its nested content"
    );
    d.click("Diff/Next");
    assert_eq!(
        d.read("diff"),
        json!("easy"),
        "the page under the shade takes no press"
    );
}

#[test]
fn typewriter_reveals_whole_graphemes_and_replays_completion() {
    let text = r#"<meta name="component" content="t">
<div><og-typewriter id="Line" rate="60" onrevealed={done}>{$words}</og-typewriter></div>"#;
    for mounted in [false, true] {
        let mut d = Doc::new(text, &[("words", json!("Á👩‍🚀Z"))], mounted);
        assert_eq!(d.get("Line").text, "");
        d.step(InputFrame::default());
        assert_eq!(d.get("Line").text, "Á", "the accent stays with its base");
        let saved = d.ui.save_state();
        for replay in 0..2 {
            if replay == 1 {
                d.ui.restore_state(&saved).unwrap();
            }
            d.events.clear();
            d.step(InputFrame::default());
            assert_eq!(d.get("Line").text, "Á👩‍🚀", "the joined emoji is indivisible");
            assert!(!d.events.iter().any(|event| event.name == "done"));
            d.step(InputFrame::default());
            assert_eq!(d.get("Line").text, "Á👩‍🚀Z");
            d.step(InputFrame::default());
            assert_eq!(
                d.events.iter().filter(|event| event.name == "done").count(),
                1
            );
        }
        d.set("words", json!("🇳🇿BC"));
        assert_eq!(d.get("Line").text, "🇳🇿", "new content restarts the reveal");
        let mut theme = *d.ui.theme();
        theme.reduced_motion = true;
        d.ui.set_theme(theme);
        d.step(InputFrame::default());
        assert_eq!(d.get("Line").text, "🇳🇿BC");
    }
}

/// The settings screen's tree from data: pages as tabs, each entry's
/// control writing its row's value back, Apply and Revert only while
/// changes are pending.
///
/// Ways it could fail: every page's entries shown at once, a tab that does
/// not switch the page, a control writing to the wrong row, or Apply taking
/// a press with nothing pending.
#[test]
fn the_settings_tree_is_data() {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/engine/src/ui/screens/settings.svelte"),
    )
    .unwrap()
    .replace("<meta name=\"strings\" content=\"./settings.en.ogt\">", "");
    let stores = [
        ("note", json!("")),
        (
            "pages",
            json!([{"id":"audio","name":"Audio"},{"id":"video","name":"Video"}]),
        ),
        (
            "entries",
            json!([
                {"id":"music","page":"audio","name":"Music","kind":"range","value":0.5,"min":0,"max":1,"step":0.1},
                {"id":"subs","page":"audio","name":"Subtitles","kind":"toggle","value":false},
                {"id":"quality","page":"video","name":"Quality","kind":"select","value":"Low","options":["Low","High"]}
            ]),
        ),
        ("page", json!("audio")),
        ("pending", json!(false)),
    ];
    let d = &mut Doc::new(&text, &stores, true);
    for _ in 0..5 {
        d.step(InputFrame::default());
    }
    assert!(
        d.el("Toggle").is_some() && d.el("Choice").is_none(),
        "one page shown"
    );
    d.click("Toggle/Track");
    let row = |d: &mut Doc, k: usize| d.read("entries")[k]["value"].clone();
    assert_eq!(row(d, 1), json!(true));
    assert_eq!(row(d, 0), json!(0.5), "its own row");
    let events = d.click("Apply");
    assert!(!events.iter().any(|e| e.name == "apply"), "nothing pending");
    d.set("page", json!("video"));
    assert_eq!(d.read("page"), json!("video"));
    assert!(d.el("Choice").is_some() && d.el("Toggle").is_none());
    d.click("Choice/Next");
    assert_eq!(row(d, 2), json!("High"));
}

/// Rows a store hides and shows inside an unmarked list: the rows'
/// `{#if}` content changes in the same refresh that removes their `<li>`.
///
/// The way it failed: the queued sibling change among a removed row's
/// children named a parent no longer in the selector topology, and the
/// whole build stopped ("missing instantiated parent").
#[test]
fn rows_leaving_with_their_changed_children_restyle_cleanly() {
    let text = r#"<meta name="component" content="t">
<div id="Page"><ul id="L">{#each $rows as e (e.id)}{#if e.page == $page}<li>{#if e.kind == "r"}<p id="N">{e.name}</p><input type="range" id="R" bind:value={e.value}>{/if}{#if e.kind == "t"}<label>{e.name}<input type="checkbox" switch id="T" bind:checked={e.value}></label>{/if}</li>{/if}{/each}</ul></div>
<style>#L { list-style: none; }</style>"#;
    let stores = [
        (
            "rows",
            json!([
            {"id":"a","page":"1","name":"A","kind":"r","value":0.5},
            {"id":"a2","page":"1","name":"A2","kind":"t","value":false},
            {"id":"b","page":"2","name":"B","kind":"t","value":false}]),
        ),
        ("page", json!("1")),
    ];
    let d = &mut Doc::new(text, &stores, true);
    d.set("page", json!("2"));
    let shown: Vec<_> =
        d.ui.probe_elements()
            .into_iter()
            .filter(|e| e.kind == "label" && e.text.starts_with('A') || e.text == "B")
            .map(|e| e.text)
            .collect();
    assert_eq!(shown, ["B"]);
}

/// Every component document the engine ships in place of a TS builder
/// (B-port) builds from its default fixture, both ways a document is built,
/// and its keyed lists show their rows.
///
/// Ways it could fail: a document that no longer parses after a tag or CSS
/// change, a fixture list that builds no rows (a nested `{#each}` a retained
/// build cannot read).
#[test]
fn shipped_component_documents_build_from_their_fixtures() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/engine/src/ui/screens");
    // Each document, and a text from its fixture's list it must show.
    for (name, row) in [
        ("card", "Ember Blade"),
        ("confirm", "Keep playing"),
        ("pagination", "9"),
        ("data-list", "Silk"),
        ("key-values", "Defence"),
        ("calendar", "2"),
        ("tree", "Camera"),
        ("table", "Gulls"),
        ("objective-tracker", "Light the lamps 2/5"),
        ("star-rating", ""),
        ("banner", "Level up!"),
        ("subtitles", "Keeper"),
        ("notifications", "First blood"),
    ] {
        let text = std::fs::read_to_string(root.join(format!("{name}.svelte"))).unwrap();
        let doc = UiDocument::parse(name, &text).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let fixture = doc
            .fixtures
            .iter()
            .find(|(n, _)| n == "default")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("{name}: no default fixture"));
        let stores: Vec<(String, Value)> = fixture
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let stores: Vec<(&str, Value)> = stores
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect();
        both(&text, &stores, |d| {
            let elements = d.ui.probe_elements();
            let shown = if row.is_empty() {
                elements.iter().filter(|e| e.kind == "icon").count() == 3
            } else {
                elements.iter().any(|e| e.text.contains(row))
            };
            assert!(shown, "{name} (mounted: {}): no {row}", d.mounted);
        });
    }
}

/// A search box draws HTML's clear button while it has text, and a press on
/// it empties the field; a field's count, leading icon and suffix are plain
/// markup round a bare input (`{$name.length}/20`, `<og-icon>`, `<span>`).
///
/// Ways it could fail: no clear button, one that shows on an empty box or
/// leaves the text, or a count that does not follow the text.
#[test]
fn search_clears_and_field_adornments_compose() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <input type="search" id="Find" bind:value={$q}>
  <div id="Frame" class="frame">
    <og-icon id="Lead" icon="coin"></og-icon>
    <input id="Gold" class="bare" maxlength="20" bind:value={$name}>
    <span id="Suffix">gold</span>
  </div>
  <small id="Count">{$name.length}/20</small>
</div>
<style>
#Page { gap: 8px; padding: 8px; }
.frame { flex-direction: row; align-items: center; gap: 4px; padding: 0 8px; }
.bare { appearance: none; }
</style>"#;
    let d = &mut Doc::new(text, &[("q", json!("ore")), ("name", json!("Ash"))], true);
    assert!(
        d.el("Find/Clear").is_some(),
        "a search box with text has its clear button"
    );
    d.click("Find/Clear");
    assert_eq!(d.read("q"), json!(""), "the clear button empties it");
    d.step(InputFrame::default());
    assert!(d.el("Find/Clear").is_none(), "an empty search box has none");
    assert_eq!(d.get("Count").text, "3/20");
    let (lead, gold, suffix) = (d.get("Lead"), d.get("Gold"), d.get("Suffix"));
    assert!(
        lead.x < gold.x && gold.x < suffix.x,
        "icon, field, suffix in a row"
    );
    d.set("name", json!("Ashen"));
    d.step(InputFrame::default());
    assert_eq!(d.get("Count").text, "5/20", "the count follows the text");
}

/// `<details name>`, HTML's exclusive accordion: of the disclosures sharing
/// a name one is open at once; opening one closes the other, which writes
/// back, while a disclosure of another name stays as it is.
///
/// Ways it could fail: the name read as a screen reader's label, two open
/// at once, the closed one's store left true, or an unrelated one closed.
#[test]
fn details_name_is_an_exclusive_accordion() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <details id="Rules" name="faq" summary="Rules" bind:open={$rules}><p id="RulesBody">x</p></details>
  <details id="Score" name="faq" summary="Scoring"><p id="ScoreBody">y</p></details>
  <details id="Other" summary="Other" open><p id="OtherBody">z</p></details>
</div>
<style>#Page { gap: 8px; padding: 8px; }</style>"#;
    let d = &mut Doc::new(text, &[("rules", json!(true))], true);
    assert!(d.el("RulesBody").is_some() && d.el("ScoreBody").is_none());
    assert_ne!(
        d.a11y("Score").name,
        "faq",
        "name is the group, not a label"
    );
    let summary = |d: &Doc, id: &str, text: &str| {
        d.ui.probe_elements()
            .into_iter()
            .find(|e| e.path.contains(&format!("/{id}/")) && e.text == text)
            .unwrap_or_else(|| panic!("{id}'s summary"))
    };
    let s = summary(d, "Score", "Scoring");
    d.click_at(Vec2::new(s.x + s.width / 2.0, s.y + s.height / 2.0));
    assert!(d.el("ScoreBody").is_some(), "Scoring opens");
    assert!(d.el("RulesBody").is_none(), "and Rules closes");
    assert_eq!(d.read("rules"), json!(false), "its store says so");
    assert!(d.el("OtherBody").is_some(), "another name is left open");
    let s = summary(d, "Rules", "Rules");
    d.click_at(Vec2::new(s.x + s.width / 2.0, s.y + s.height / 2.0));
    assert!(d.el("RulesBody").is_some() && d.el("ScoreBody").is_none());
    assert_eq!(d.read("rules"), json!(true));
}

/// Where each button's label starts across it, from its drawn text.
fn label_x(d: &Doc, id: &str, text: &str) -> f32 {
    let e = d.get(id);
    let x =
        d.ui.frame()
            .items
            .iter()
            .find_map(|it| match &it.cmd {
                DrawCmd::Text { x, text: t, .. } if t == text => Some(*x),
                DrawCmd::StyledText { text: inner, .. } => match &**inner {
                    DrawCmd::Text { x, text: t, .. } if t == text => Some(*x),
                    _ => None,
                },
                _ => None,
            })
            .unwrap_or_else(|| panic!("no text {text}"));
    x - e.x
}

#[test]
fn a_button_label_takes_text_align() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <button id="Mid">Mid</button>
  <button id="Start">Start</button>
  <div id="Right"><button id="Kept">Kept</button></div>
  <menu id="Menu"><button id="Copy">Copy</button><button id="PasteAll">Paste all</button></menu>
</div>
<style>
#Page { width: 400px; }
button { width: 200px; }
#Start { text-align: start; }
#Right { text-align: right; }
#Menu { width: 300px; padding: 0; margin: 0; }
#Menu > button { width: stretch; }
</style>"#;
    for mounted in [false, true] {
        let mut scene = og_render::RenderScene::new();
        let d = &Doc::with_font(
            text,
            &[],
            mounted,
            crate::support::add_test_font(&mut scene),
        );
        let mid = label_x(d, "Mid", "Mid");
        let start = label_x(d, "Start", "Start");
        assert!(mid > 60.0, "centred by default: {mid}");
        assert!(start < 30.0, "text-align: start: {start}");
        assert!(
            label_x(d, "Kept", "Kept") > 60.0,
            "a button does not inherit its parent's text-align"
        );
        let (copy, paste) = (
            label_x(d, "Copy", "Copy"),
            label_x(d, "PasteAll", "Paste all"),
        );
        assert_eq!(d.get("Copy").width, 300.0);
        assert!(
            copy < 30.0 && copy == paste,
            "menu items start their labels: {copy} {paste}"
        );
    }
}
