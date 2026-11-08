//! The HTML and CSS a document's controls take beyond their tag (docs/ui.md
//! §2.4): a range's `<datalist>` marks, `-og-fill-origin` and `disabled`;
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
//!   fill growing from min when `-og-fill-origin` names another value, a
//!   disabled range still taking a drag, or a `list` on a text input (or
//!   naming nothing) accepted → `a_range_takes_its_datalist_origin_and_disabled`
//! - a caption on the wrong side of its switch for the source order, a
//!   label's `flex-direction: row-reverse` ignored, a `<small>` merged into
//!   the caption, `-og-control-size` or `disabled` dropped →
//!   `a_checkbox_follows_its_label_order_size_and_description`
//! - a modal closing on a backdrop press without `closedby="any"`, Escape
//!   closing a `closedby="none"` modal, an in-place dialog taking Escape by
//!   default (HTML's `none`), `appearance: none` keeping the frame →
//!   `dialogs_close_as_closedby_says`
//! - `aria-invalid` not marking a select's trigger, `::picker(select)`'s
//!   max-height not bounding the open list, or the pseudo-element taking a
//!   property it cannot honour → `a_select_takes_aria_invalid_and_its_picker_height`
//! - `object-view-box: inset()` cropping the wrong sides of the texture →
//!   `an_image_shows_its_object_view_box`
//! - `data-og-instant` not settling the first entrance, or settling a
//!   later one → `data_og_instant_settles_only_the_first_entrance`

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

fn key(d: &mut Doc, k: UiKey) {
    d.step(InputFrame {
        keys: vec![k],
        ..InputFrame::default()
    });
    for _ in 0..30 {
        d.step(InputFrame::default());
    }
}

#[test]
fn a_range_takes_its_datalist_origin_and_disabled() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <input type="range" id="Bal" min="-1" max="1" step="0.1" list="Steps" value={$bal} />
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
#Bal { -og-fill-origin: 0; }
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
    let on_text = refused(
        r#"<meta name="component" content="t"><div><input type="text" list="L" /><datalist id="L"><option value="1"></option></datalist></div>"#,
    );
    assert!(on_text.contains("combobox"), "{on_text}");
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
fn a_checkbox_follows_its_label_order_size_and_description() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <label id="After"><input type="checkbox" switch bind:checked={$a}> After</label>
  <label id="Before">Before <input type="checkbox" switch><small>Plays in menus</small></label>
  <label id="Flip" class="rev">Flipped <input type="checkbox" switch></label>
  <input type="checkbox" id="Late"><label for="Late">Late</label>
  <label id="Big" class="big">Big <input type="checkbox" switch disabled></label>
  <label id="Small">Small <input type="checkbox" switch></label>
</div>
<style>
#Page { display: flex; flex-direction: column; align-items: start; gap: 12px; padding: 20px; }
.rev { flex-direction: row-reverse; }
.big { -og-control-size: large; }
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
            "-og-control-size: large is larger: {} vs {}",
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
select::picker(select) {{ max-height: 150px; }}
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
                .any(|p| p.starts_with("Dropdown.trigger") && p.contains("invalid")),
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
    let colour = refused(
        r#"<meta name="component" content="t"><div><select><option>a</option></select></div><style>select::picker(select) { color: red; }</style>"#,
    );
    assert!(colour.contains("takes max-height"), "{colour}");
}

#[test]
fn an_image_shows_its_object_view_box() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <img id="Crop" alt="" />
  <img id="Raw" alt="" />
</div>
<style>
img { width: 64px; height: 64px; }
#Crop { object-view-box: inset(10% 20% 30% 40%); }
#Raw { -og-uv: 0 0 0.5 1; }
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
            "-og-uv as written: {raw:?}"
        );
    });
    let px = refused(
        r#"<meta name="component" content="t"><div><img alt="" /></div><style>img { object-view-box: inset(4px); }</style>"#,
    );
    assert!(px.contains("percentages"), "{px}");
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
  <button id="Buy" variant="primary" onclick={buy}><icon id="Coin" icon="coin" />Buy<badge id="N">3</badge></button>
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

/// Rich inline content (G6): `<img>`, `<icon>` and `<kbd data-action>`
/// among a paragraph's words draw as inline objects, a `<span part>` takes
/// its `::part()` size, line height and tracking (doc/mf2.rs tests the
/// translated markup). Ways it could fail: an inline object refused or
/// drawn as text, a part's size ignored.
#[test]
fn text_holds_inline_objects_sized_parts_and_translated_markup() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <p id="Line">Press <kbd data-action="jump"></kbd> for <img src="coin" alt=""> <span part="big">gold</span></p>
  <p id="Said">plain</p>
</div>
<style>
p::part(big) { font-size: 40px; line-height: 48px; letter-spacing: 0.1em; }
</style>"#;
    let doc = UiDocument::parse("ui/t.svelte", text).unwrap_or_else(|e| panic!("{e:?}"));
    let line = doc.nodes.iter().find(|n| n.id == "Line").unwrap();
    assert_eq!(line.inline.iter().filter(|r| r.atom.is_some()).count(), 2);
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
