//! The core `<select>`, `<dialog>`, `<details>` and `<meter>` (docs/ui.md
//! §2.4), through a mounted document, tick by tick.
//!
//! Ways these could fail, each covered below:
//! - a select showing the wrong option for its bound value, writing back
//!   the label instead of the value, letting a disabled option be picked,
//!   losing its groups, ignoring the arrows, typeahead or the pad's A and
//!   B, or leaving the focus in the closed list →
//!   `a_select_binds_its_value_by_keys_typeahead_pad_and_pointer`
//! - a modal dialog not trapping the focus, Escape or B not closing it
//!   (or closing it without writing `open` back), the focus not returning
//!   to its opener; a non-modal one taking Back while the focus is
//!   elsewhere → `dialogs_close_on_back_and_return_the_focus`
//! - a closed disclosure laying out or reading its content, its summary
//!   not toggling it (bound or literal), or the summary's state not
//!   reaching the accessibility tree →
//!   `details_shows_its_content_only_while_open`
//! - a meter's region wrong at either side of `low`/`high` for an optimum
//!   in each of the three places, or its range missing for a screen reader
//!   → `meter_regions_follow_the_html_rules`
//! - a control dressed under another part name than docs/ui.md §2.4
//!   lists, so a pack's `Component::part` rule never reaches it →
//!   `each_control_is_dressed_by_its_named_parts`
//! - `dialog::backdrop`'s background or box properties not reaching the
//!   scrim (or reaching an in-place dialog's page)
//!   → `the_backdrop_pseudo_element_fills_the_scrim`
//! - `<option>` or `<summary>` accepted out of place, or a select's
//!   `{#each}` accepted though it cannot list → `misplaced_parts_are_refused`
//! - `::file-selector-button` styling the whole file input or nothing, the
//!   names not beside the button, or a press on the names not opening the
//!   picker → `file_selector_button_is_the_file_inputs_button_box`

use crate::support;

use og_core::glam::Vec2;
use og_ui::reactive::SourceId;
use og_ui::*;
use serde_json::json;
use support::*;

#[path = "html_controls/rig.rs"]
mod rig;
use rig::*;

fn rig() -> Rig {
    Rig::new(ui_with_font(test_font_metrics()), &STORES)
}

const PAD_A: u32 = 0;
const PAD_B: u32 = 1;
const PAD_DOWN: u32 = 13;

#[test]
fn a_select_binds_its_value_by_keys_typeahead_pad_and_pointer() {
    let mut rig = rig();
    assert_eq!(rig.a11y("Pick/Trigger").value, "", "nothing chosen");
    rig.set("choice", json!("b"));
    assert_eq!(
        rig.a11y("Pick/Trigger").value,
        "Blue",
        "the bound value shows its option's label"
    );
    // The pointer: open, the groups are there, a disabled option is not picked.
    rig.click("Pick/Trigger");
    assert!(
        rig.focus().ends_with("Option#2"),
        "opens at the chosen: {}",
        rig.focus()
    );
    assert!(rig.ui.element_by_text("Warm").is_some() && rig.ui.element_by_text("Cool").is_some());
    rig.click("List/Option#1");
    assert_eq!(rig.store("choice"), "b", "a disabled option is not picked");
    rig.click("List/Option#3");
    assert_eq!(
        rig.store("choice"),
        "c",
        "the value, not the label, is written"
    );
    assert!(
        rig.focus().ends_with("Pick/Trigger"),
        "the focus returns: {}",
        rig.focus()
    );
    // Keys: Down opens; typeahead moves to Red; Enter picks it.
    rig.key(UiKey::Down);
    assert!(rig.focus().ends_with("Option#3"), "{}", rig.focus());
    rig.typed("r");
    assert!(
        rig.focus().ends_with("Option#0"),
        "typeahead: {}",
        rig.focus()
    );
    rig.key(UiKey::Enter);
    assert_eq!(rig.store("choice"), "r");
    // Escape closes without changing it, the focus on the trigger.
    rig.key(UiKey::Down);
    rig.key(UiKey::Down);
    rig.key(UiKey::Escape);
    assert_eq!(rig.store("choice"), "r", "Escape changes nothing");
    assert!(rig.focus().ends_with("Pick/Trigger"), "{}", rig.focus());
    // The pad: A opens, the d-pad moves past the disabled option, A picks,
    // B closes another time without a change.
    rig.pad(PAD_A);
    rig.pad(PAD_DOWN);
    assert!(
        rig.focus().ends_with("Option#2"),
        "skips Orange: {}",
        rig.focus()
    );
    rig.pad(PAD_A);
    assert_eq!(rig.store("choice"), "b");
    rig.pad(PAD_A);
    rig.pad(PAD_DOWN);
    rig.pad(PAD_B);
    assert_eq!(rig.store("choice"), "b");
    assert!(rig.focus().ends_with("Pick/Trigger"), "{}", rig.focus());
}

#[test]
fn dialogs_close_on_back_and_return_the_focus() {
    let mut rig = rig();
    assert!(
        rig.el("Ask/Panel").is_none(),
        "a closed dialog is not laid out"
    );
    rig.click("Opener");
    assert!(rig.focus().ends_with("Opener"));
    rig.set("ask", json!(true));
    rig.settle();
    assert!(rig.el("Ask/Panel").is_some());
    assert_eq!(rig.a11y("Ask/Panel").name, "Quit?");
    assert!(
        rig.focus().starts_with("Ask/"),
        "the focus moves in: {}",
        rig.focus()
    );
    // The trap: Tab past the last button stays inside.
    for _ in 0..4 {
        rig.key(UiKey::Tab);
        assert!(rig.focus().starts_with("Ask/"), "trapped: {}", rig.focus());
    }
    rig.key(UiKey::Escape);
    assert_eq!(
        rig.store("ask"),
        false,
        "Escape closes it and writes open back"
    );
    assert!(rig.el("Ask/Panel").is_none());
    assert!(
        rig.focus().ends_with("Opener"),
        "back to the opener: {}",
        rig.focus()
    );
    rig.set("ask", json!(true));
    rig.settle();
    rig.pad(PAD_B);
    assert_eq!(rig.store("ask"), false, "B closes it too");
    // Non-modal: in place; Back elsewhere is not its own.
    rig.click("Opener");
    rig.set("note", json!(true));
    assert!(rig.el("Note").is_some());
    rig.key(UiKey::Escape);
    assert_eq!(
        rig.store("note"),
        true,
        "Back with the focus outside leaves it open"
    );
    rig.click("Ok");
    assert!(rig.focus().ends_with("Ok"));
    rig.key(UiKey::Escape);
    assert_eq!(rig.store("note"), false, "Back inside closes it");
    assert!(rig.el("Note/Ok").is_none());
    assert!(
        rig.focus().ends_with("Opener"),
        "the focus returns: {}",
        rig.focus()
    );
}

#[test]
fn details_shows_its_content_only_while_open() {
    let mut rig = rig();
    assert!(
        rig.el("Body").is_none(),
        "closed: its content is not laid out"
    );
    assert_eq!(rig.a11y("More/Summary").name, "More options");
    assert!(
        !rig.a11y("More/Summary")
            .states
            .contains(&"expanded".to_string())
    );
    rig.click("More/Summary");
    assert_eq!(
        rig.store("more"),
        true,
        "the summary toggles the bound open"
    );
    assert!(rig.el("Body").is_some());
    assert!(
        rig.a11y("More/Summary")
            .states
            .contains(&"expanded".to_string())
    );
    rig.set("more", json!(false));
    assert!(rig.el("Body").is_none(), "the store closes it");
    // Literal: the player's toggle is kept as UI state; Enter on the
    // focused summary toggles it as a press does.
    assert!(rig.el("LitBody").is_none());
    rig.click("Lit/Summary");
    assert!(rig.el("LitBody").is_some(), "a press opens a literal one");
    let saved = rig.ui.save_state();
    rig.key(UiKey::Enter);
    assert!(rig.el("LitBody").is_none(), "Enter closes it");
    rig.ui.restore_state(&saved).unwrap();
    rig.settle();
    assert!(rig.el("LitBody").is_some(), "its state saves and restores");
}

/// The solid fills drawn across the middle of meter `hp`.
fn meter_fills(ui: &Ui, hp: &UiElement) -> Vec<[f32; 4]> {
    ui.frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. }
                if (shape.center.y - (hp.y + hp.height / 2.0)).abs() < 1.0
                    && shape.center.x > hp.x
                    && shape.center.x < hp.x + hp.width =>
            {
                match shape.paint {
                    og_draw::UiPaint::Solid(c) => Some(c),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

#[test]
fn meter_regions_follow_the_html_rules() {
    // A meter over its bounds' stores: its region is the colour its fill
    // is drawn in (undressed: success, warn, danger).
    let mut d = Doc::new(
        r#"<meta name="component" content="t">
<column id="Form"><meter id="M" value={$v} min="0" max="100" low={$low} high={$high} optimum={$opt} /></column>"#,
        &[
            ("v", json!(0)),
            ("low", json!(25)),
            ("high", json!(75)),
            ("opt", json!(90)),
        ],
    );
    let palette = d.ui.theme().palette;
    let regions = [
        ("optimum", palette.success.linear()),
        ("suboptimum", palette.warn.linear()),
        ("poor", palette.danger.linear()),
    ];
    let region = |d: &mut Doc, value: f64, low: f64, high: f64, optimum: f64| {
        for (store, v) in [("v", value), ("low", low), ("high", high), ("opt", optimum)] {
            d.set(store, json!(v));
        }
        let fills = meter_fills(&d.ui, &d.get("M"));
        let drawn: Vec<&str> = regions
            .iter()
            .filter(|(_, c)| fills.contains(c))
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(drawn.len(), 1, "one region's colour: {drawn:?}");
        drawn[0]
    };
    for (value, optimum, want) in [
        // Optimum high: above high best, the middle next, below low poor.
        (80.0, 90.0, "optimum"),
        (75.0, 90.0, "suboptimum"),
        (25.0, 90.0, "suboptimum"),
        (24.0, 90.0, "poor"),
        // Optimum low: the mirror.
        (10.0, 5.0, "optimum"),
        (50.0, 5.0, "suboptimum"),
        (76.0, 5.0, "poor"),
        // Optimum in the middle: either end only suboptimal.
        (50.0, 50.0, "optimum"),
        (10.0, 50.0, "suboptimum"),
        (90.0, 50.0, "suboptimum"),
    ] {
        assert_eq!(
            region(&mut d, value, 25.0, 75.0, optimum),
            want,
            "{value} with {optimum}"
        );
    }
    // Bounds out of order clamp as HTML's do: high under low is low.
    assert_eq!(region(&mut d, 30.0, 40.0, 20.0, 90.0), "poor");
    // In the form, the fill is drawn in its region's colour.
    let mut rig = rig();
    let fill = |rig: &Rig| meter_fills(&rig.ui, &rig.el("Hp").expect("the meter"));
    let palette = rig.ui.theme().palette;
    rig.set("hp", json!(20));
    assert_eq!(rig.a11y("Hp").role, "meter");
    assert!(
        fill(&rig).contains(&palette.danger.linear()),
        "20 with optimum 90 is poor"
    );
    rig.set("hp", json!(50));
    assert!(
        fill(&rig).contains(&palette.warn.linear()),
        "50 is suboptimal"
    );
    rig.set("hp", json!(80));
    let now = fill(&rig);
    assert!(now.contains(&palette.success.linear()) && !now.contains(&palette.danger.linear()));
}

/// CSS Pseudo-Elements 4 §3.4 `::file-selector-button`: a file input is
/// its own box holding a button, which the pseudo-element styles, and the
/// names chosen beside it (HTML's file upload state). The input's 4px
/// padding places the button 4px inside it; the rule's red background
/// fills the button's box alone while the input keeps its blue; the
/// rule's 10px inline padding makes the button as wide as a `<button>`
/// with that padding and the same label, and its 6px right margin moves
/// the names 6px further than an unstyled input's. The names say none is
/// chosen, then name the file; a press on them opens the picker, as a
/// press anywhere on the input does, and the input alone is a stop.
/// Ways it could fail: the rule ignored (it styled nothing before), the
/// rule styling the whole input, the button a stop of its own, or only
/// the button taking the press.
#[test]
fn file_selector_button_is_the_file_inputs_button_box() {
    let text = r#"<meta name="component" content="t">
<column>
  <button id="Ref">Choose file…</button>
  {#if $show}
    <input type="file" id="F" bind:value={$files} />
    <input type="file" id="G" />
  {/if}
</column>
<style>
input { padding: 4px; background: #0000ff; }
#Ref { padding: 0 10px; }
#F::file-selector-button { background: #ff0000; padding: 0 10px; margin-right: 6px; }
</style>"#;
    let mut d = support::Doc::new(text, &[("files", json!("")), ("show", json!(false))]);
    d.ui.set_file_picker(true);
    d.set("show", json!(true));
    let fill = |d: &support::Doc, want: [f32; 3]| {
        d.ui.frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .filter(|(_, c)| [c[0], c[1], c[2]] == want)
            .map(|(r, _)| (r.x, r.y, r.width, r.height))
            .collect::<Vec<_>>()
    };
    let (f, button, names) = (d.get("F"), d.get("F/Button"), d.get("F/Names"));
    assert_eq!(
        (button.x - f.x, button.y - f.y),
        (4.0, 4.0),
        "{button:?} in {f:?}"
    );
    assert_eq!(button.text, "Choose file…");
    assert_eq!(
        fill(&d, [1.0, 0.0, 0.0]),
        [(button.x, button.y, button.width, button.height)],
        "the rule fills the button's box"
    );
    assert!(
        fill(&d, [0.0, 0.0, 1.0]).contains(&(f.x, f.y, f.width, f.height)),
        "the input keeps its own background"
    );
    assert_eq!(button.width, d.get("Ref").width, "10px inline padding");
    let (g, gb) = (d.get("G/Names"), d.get("G/Button"));
    assert_eq!(
        (names.x - button.x - button.width) - (g.x - gb.x - gb.width),
        6.0,
        "its right margin"
    );
    assert_eq!(names.text, "No file chosen");
    assert!(d.get("F").states.iter().any(|s| s == "focusable"));
    assert_eq!(d.get("F").role, "button");
    assert!(
        !button.states.iter().any(|s| s == "focusable") && button.role.is_empty(),
        "the button is no stop of its own: {button:?}"
    );
    let events = d.click("F/Names");
    assert!(events.iter().any(|e| e.kind == "filepick"), "{events:?}");
    d.set("files", json!("a.png"));
    assert_eq!(d.get("F/Names").text, "a.png");
}

#[test]
fn misplaced_parts_are_refused() {
    for (bad, why) in [
        ("<option>A</option>", "inside a <select>"),
        ("<summary>A</summary>", "first child of a <details>"),
        (
            "<select bind:value={$a}>{#each $xs as x (x.id)}<option>{x.name}</option>{/each}</select>",
            "items={…} values={…}",
        ),
        (
            "<select bind:value={$a}><option value={$v}>A</option></select>",
            "literal",
        ),
        (
            "<details><text>A</text><summary>B</summary></details>",
            "first child",
        ),
    ] {
        let doc = format!("<meta name=\"component\" content=\"form\">\n<column>{bad}</column>");
        let err = UiDocument::parse("ui/form.svelte", &doc)
            .err()
            .unwrap_or_else(|| panic!("{bad} parses"))
            .join("\n");
        assert!(err.contains(why), "{bad}: {err}");
    }
}

#[test]
fn each_control_is_dressed_by_its_named_parts() {
    let mut rig = rig();
    rig.ui.set_part_css("details::part(summary) { background: #ff00ff; } select[aria-expanded=true]::part(trigger) { background: #00ffff; } select[id=Pick]::part(list) { background: #ff8800; }").unwrap();
    rig.set("more", json!(true));
    rig.set("note", json!(true));
    rig.set("choice", json!("b"));
    rig.click("Pick/Trigger");
    for color in [
        Srgba::rgb(255, 0, 255),
        Srgba::rgb(0, 255, 255),
        Srgba::rgb(255, 136, 0),
    ] {
        assert!(
            rig.ui
                .frame()
                .items
                .iter()
                .any(|item| { item.cmd.filled().is_some_and(|(_, fill)| near(fill, color)) }),
            "nested parts retain their document host after the popup opens: {color:?}"
        );
    }
    let mut parts: std::collections::BTreeSet<String> = rig
        .ui
        .parts()
        .iter()
        .flatten()
        .map(|p| format!("{p}"))
        .collect();
    let mut modal = self::rig();
    modal.set("ask", json!(true));
    modal.settle();
    assert!(modal.el("Ask/Panel").is_some());
    let in_top_layer: Vec<String> = modal
        .ui
        .parts()
        .iter()
        .flatten()
        .map(|p| format!("{p}"))
        .collect();
    // A modal dialog in the top layer keeps its panel part, as one opened
    // in place has it.
    assert!(
        in_top_layer.iter().any(|p| p == "dialog::part(panel):open"),
        "{in_top_layer:?}"
    );
    parts.extend(in_top_layer);
    for want in [
        "select[aria-expanded=true]::part(trigger):open",
        "select::part(list)",
        "select::part(group)",
        "select::part(option)",
        "select::part(option):disabled",
        "details::part(root)",
        "details::part(summary)",
        "details::part(summary):open",
        "details::part(content)",
        "dialog::part(panel):open",
        "dialog::part(backdrop)",
        "meter::part(track)",
    ] {
        assert!(parts.contains(want), "{want} in {parts:?}");
    }
    // The chosen option is :checked, as HTML's selected <option> is.
    assert!(
        parts.iter().any(|p| p.starts_with("select")
            && p.contains("::part(option)")
            && p.contains(":checked")),
        "{parts:?}"
    );
}

/// CSS Pseudo-Elements 4 §5.2 `::details-content`: a details element's
/// expandable content is one box its rules style. Its padding (20px left,
/// 30px top) places the content inside it over the disclosure's own, its
/// background fills it from the details' left edge across its width, and
/// a rule under the originating element's `[open]` wins while it is open
/// (`bind:open`, both ways); closed, the content and its box draw nothing.
/// Ways it could fail: the rules dropped (the content at the disclosure's
/// own inset), the box styled as the whole details rather than its
/// content, or `[open]` not following the element's open state.
#[test]
fn details_content_styles_the_disclosures_content_box() {
    let text = r#"<meta name="component" content="t">
<column>
  <details id="D" summary="More" bind:open={$open}><text id="Body">x</text></details>
</column>
<style>
#D { width: 300px; }
#D::details-content { padding: 30px 0 0 20px; background: #ff0000; }
#D[open]::details-content { background: #00ff00; }
</style>"#;
    let mut d = crate::support::Doc::new(text, &[("open", json!(false))]);
    let fill = |d: &crate::support::Doc, want: [f32; 3]| {
        d.ui.frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .find(|(_, c)| [c[0], c[1], c[2]] == want)
            .map(|(r, _)| r)
    };
    let (red, green) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    assert!(d.el("Body").is_none(), "closed: no content");
    assert!(
        fill(&d, red).is_none() && fill(&d, green).is_none(),
        "closed: no box"
    );
    d.set("open", json!(true));
    let (details, body) = (d.get("D"), d.get("Body"));
    assert!(fill(&d, red).is_none(), "[open] wins while open");
    let r = fill(&d, green).expect("the open content box draws its background");
    assert_eq!(
        (r.x, r.width),
        (details.x, details.width),
        "across the details"
    );
    assert!(r.y > details.y, "below the summary: {r:?} in {details:?}");
    assert_eq!(
        (body.x - r.x, body.y - r.y),
        (20.0, 30.0),
        "its padding places the content"
    );
    d.set("open", json!(false));
    assert!(d.el("Body").is_none(), "closed again: no content");
    assert!(
        fill(&d, red).is_none() && fill(&d, green).is_none(),
        "closed again: no box, and [open] no longer matches"
    );
}

/// CSS Writing Modes 4 with HTML's vertical form controls: a range, a
/// progress bar and a meter in a vertical writing mode stand up along
/// their inline axis, their `height` its length. Left to right, the value
/// runs top to bottom (min at the top); `direction: rtl` runs it bottom to
/// top. A press a quarter of the way down the track (its knob's half kept
/// at each end) sets 25 or 75 of 0–100, the knob's centre sits there, and
/// a quarter-full bar or meter fills the top or bottom 50 of its 200 px.
/// Ways it could fail: the control left lying across, `height` ignored,
/// the pointer read across, or `rtl` not reversing the value.
#[test]
fn vertical_writing_modes_stand_bar_controls_up() {
    let text = r#"<meta name="component" content="t">
<row>
  <input type="range" id="Down" min="0" max="100" step="1" show-value="false" bind:value={$down} />
  <input type="range" id="Up" min="0" max="100" step="1" show-value="false" bind:value={$up} />
  <progress id="PDown" value="0.25"></progress>
  <progress id="PUp" value="0.25"></progress>
  <meter id="MDown" min="0" max="100" value="25"></meter>
  <meter id="MUp" min="0" max="100" value="25"></meter>
</row>
<style>
input, progress, meter { writing-mode: vertical-lr; height: 200px; }
#Up, #PUp, #MUp { direction: rtl; }
</style>"#;
    let mut d = crate::support::Doc::new(text, &[("down", json!(50)), ("up", json!(50))]);
    for (id, store) in [("Down", "down"), ("Up", "up")] {
        let track = d.get(&format!("{id}/Body/Track"));
        assert_eq!(
            track.height, 200.0,
            "{id}: its height is the track's length"
        );
        assert!(track.width < 60.0, "{id}: standing up: {track:?}");
        let knob = d.get(&format!("{id}/Body/Track/Knob")).height;
        let quarter = track.y + knob / 2.0 + 0.25 * (track.height - knob);
        d.click_at(og_core::glam::Vec2::new(
            track.x + track.width / 2.0,
            quarter,
        ));
        let want = if id == "Down" { 25.0 } else { 75.0 };
        assert_eq!(d.read(store).as_f64(), Some(want), "{id}: a quarter down");
        let k = d.get(&format!("{id}/Body/Track/Knob"));
        assert!(
            (k.y + k.height / 2.0 - quarter).abs() <= 0.5,
            "{id}: the knob's centre at the press: {k:?} vs {quarter}"
        );
        // The ends: min at the top left to right, at the bottom right to left.
        d.click_at(og_core::glam::Vec2::new(
            track.x + track.width / 2.0,
            track.y + 1.0,
        ));
        let top = if id == "Down" { 0.0 } else { 100.0 };
        assert_eq!(d.read(store).as_f64(), Some(top), "{id}: its top end");
    }
    for (id, down) in [
        ("PDown", true),
        ("PUp", false),
        ("MDown", true),
        ("MUp", false),
    ] {
        let b = d.get(id);
        assert_eq!(
            (b.height, b.width < 60.0),
            (200.0, true),
            "{id}: standing up: {b:?}"
        );
        let inside = |r: &Rect| {
            r.x >= b.x - 0.5
                && r.right() <= b.x + b.width + 0.5
                && r.y >= b.y - 0.5
                && r.bottom() <= b.y + b.height + 0.5
        };
        let fill =
            d.ui.frame()
                .items
                .iter()
                .filter_map(|i| i.cmd.filled())
                .map(|(r, _)| r)
                .find(|r| inside(r) && r.height < b.height - 1.0)
                .unwrap_or_else(|| panic!("{id}: a fill short of the box"));
        let want_y = if down { b.y } else { b.y + b.height - 50.0 };
        assert_eq!(
            (fill.x, fill.width, fill.y, fill.height),
            (b.x, b.width, want_y, 50.0),
            "{id}: a quarter, from its minimum end"
        );
    }
}

/// A modal is bounded by the safe surface even when author dimensions
/// exceed it. Unmarked content scrolls as a whole; an explicit shell's
/// body scrolls without moving its fixed siblings.
#[test]
fn modal_panels_bound_author_sizes_and_keep_their_content_reachable() {
    for (shell, zoom) in [(false, 1.0), (true, 1.0), (true, 1.25)] {
        let marker = if shell { "data-layout=\"shell\"" } else { "" };
        let body = if shell {
            "data-layout=\"scroll\" data-fill"
        } else {
            ""
        };
        let text = format!(
            r#"<meta name="component" content="form">
<div id="Root"><dialog id="Ask" modal open {marker}>
  <div id="Header">Question</div>
  <div id="Body" {body}><div id="Content"><button id="Last">Last choice</button></div></div>
  <button id="Actions">Accept</button>
</dialog></div>
<style>
#Ask {{ width: 960px; min-width: 900px; min-height: 900px; gap: 8px; padding: 12px; zoom: {zoom}; }}
#Header, #Actions {{ height: 40px; flex-shrink: 0; }}
#Content {{ display: flex; flex-direction: column; justify-content: end; height: 900px; flex-shrink: 0; }}
</style>"#
        );
        let mut ui = ui_with_font(test_font_metrics());
        ui.set_theme(Theme {
            reduced_motion: true,
            text_scale: 1.5,
            ..Theme::dark()
        });
        let mut d = Doc::mounted(ui, &text, &[]);
        let mut vp = Viewport::new(640.0, 320.0);
        vp.dpr = 2.0;
        vp.safe_area = Insets {
            top: 7.0,
            right: 31.0,
            bottom: 19.0,
            left: 13.0,
        };
        d.viewport = vp;
        for _ in 0..3 {
            d.step(InputFrame::default());
        }
        let panel =
            d.ui.elements()
                .iter()
                .find(|e| e.role == "dialog")
                .unwrap_or_else(|| panic!("the modal panel: {:?}", d.ui.elements()))
                .clone();
        let safe = vp.safe_rect();
        assert!(
            !panel.clipped
                && panel.x >= safe.x
                && panel.y >= safe.y
                && panel.x + panel.width <= safe.right() + 0.01
                && panel.y + panel.height <= safe.bottom() + 0.01,
            "shell={shell}: panel {panel:?} must fit {safe:?}"
        );
        let header = d.get("Header");
        let actions = shell.then(|| d.get("Actions"));
        let target = if shell { d.get("Body") } else { panel.clone() };
        d.focus = Some(format!("{}/Actions", panel.path));
        d.step(InputFrame::default());
        d.step(InputFrame {
            keys: vec![UiKey::Tab],
            ..Default::default()
        });
        assert!(!d.get("Last").clipped, "focus reveals the last choice");
        for _ in 0..30 {
            d.step(InputFrame {
                pointer: Vec2::new(target.x + 20.0, target.y + 20.0),
                pointer_inside: true,
                wheel: Vec2::new(0.0, -120.0),
                ..Default::default()
            });
        }
        for _ in 0..30 {
            d.step(InputFrame {
                pointer: Vec2::new(target.x + 20.0, target.y + 20.0),
                pointer_inside: true,
                wheel: Vec2::new(0.0, 120.0),
                ..Default::default()
            });
        }
        let last = d.el("Last").unwrap_or_else(|| {
            panic!(
                "shell={shell}: scroll={:?}, {:?}",
                d.ui.scroll_state(&target.path),
                d.ui.probe_elements()
            )
        });
        assert!(
            !last.clipped,
            "shell={shell}: last choice remains unreachable: {last:?}"
        );
        if shell {
            assert_eq!(
                d.get("Header").y,
                header.y,
                "the explicit shell fixes its header"
            );
            assert_eq!(
                d.get("Actions").y,
                actions.unwrap().y,
                "the explicit shell fixes its actions"
            );
        } else {
            assert!(
                d.el("Header").is_none_or(|e| e.y < header.y),
                "the fallback scrolls the entire panel"
            );
        }
        // The same retained modal must release its ceiling on a larger
        // surface and restore it when the surface shrinks again.
        d.viewport = Viewport::new(1400.0, 1200.0);
        d.step(InputFrame::default());
        assert!(d.ui.element(&panel.path).unwrap().width >= 900.0);
        d.viewport = vp;
        d.step(InputFrame::default());
        assert!(d.ui.element(&panel.path).unwrap().width <= safe.width);
        d.step(InputFrame::default());
        assert!(d.ui.element(&panel.path).unwrap().width <= safe.width);
    }
}

#[test]
fn the_backdrop_pseudo_element_fills_the_scrim() {
    let doc = r#"<meta name="component" content="form">
<column id="Form">
  <dialog id="Ask" modal bind:open={$ask} aria-label="Quit?"><button id="Yes">Yes</button></dialog>
</column>
<style>
dialog::backdrop { background: #336699; padding-top: 200px; }
dialog { width: 200px; }
</style>"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/form.svelte", doc).expect("::backdrop parses"));
    let source = SourceId::new("test", "ask");
    let mut rig = Rig { ui };
    rig.apply(json!({"op": "signal", "source": source, "initial": true}));
    rig.ui
        .mount_document(
            SourceId::new("test", "form"),
            "form",
            [("ask".to_string(), source)].into(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
    rig.settle();
    let want = Srgba::rgb(0x33, 0x66, 0x99).linear();
    let scrim = rig.ui.frame().items.iter().any(|it| match &it.cmd {
        DrawCmd::Shape { shape, .. } => {
            shape.half.x >= 399.0 && matches!(shape.paint, og_draw::UiPaint::Solid(c) if c == want)
        }
        _ => false,
    });
    assert!(scrim, "the scrim is drawn in the ::backdrop fill");
    assert_eq!(
        rig.el("Ask/Panel").unwrap().width,
        200.0,
        "the dialog's own rule still applies"
    );
    // A box property of `::backdrop` lays out the scrim: its top padding
    // moves the panel it centres down by half.
    let panel = rig.el("Ask/Panel").unwrap();
    assert!(panel.height < 200.0, "a small dialog keeps its fit height");
    let middle = rig.ui.viewport().height / 2.0;
    assert!(
        (panel.y + panel.height / 2.0 - middle - 100.0).abs() < 20.0,
        "the scrim's padding-top centres the panel below the middle: {} vs {middle}",
        panel.y + panel.height / 2.0
    );
    let bad = "dialog, dialog::backdrop { background: red; }";
    let doc = format!(
        "<meta name=\"component\" content=\"form\">\n<column><dialog modal></dialog></column>\n<style>{bad}</style>"
    );
    let err = UiDocument::parse("ui/form.svelte", &doc)
        .err()
        .unwrap_or_else(|| panic!("{bad} parses"))
        .join("\n");
    assert!(err.contains("apart"), "{bad}: {err}");
}

/// `::backdrop` inherits from its originating element (CSS Position 4
/// §3.2, as resolved for dialogs): a custom property set on the dialog
/// reaches the scrim's fill through `var()`, over the value of the column
/// around it, which it inherits when the dialog sets none. Ways it could
/// fail: the backdrop inheriting from the root or the dialog's parent
/// instead, or var() left unresolved there.
#[test]
fn the_backdrop_inherits_from_its_dialog() {
    let fill = |dialog: &str, backdrop: &str| {
        let doc = format!(
            r#"<meta name="component" content="form">
<column id="Form" style="--scrim: #ff0000">
  <dialog id="Ask" modal open aria-label="Quit?"><button id="Yes">Yes</button></dialog>
</column>
<style>
dialog {{ {dialog} }}
dialog::backdrop {{ {backdrop} }}
</style>"#
        );
        let d = crate::support::Doc::new(&doc, &[]);
        d.ui.frame()
            .items
            .iter()
            .find_map(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. } if shape.half.x >= 399.0 => match shape.paint {
                    og_draw::UiPaint::Solid(c) => Some(c),
                    _ => None,
                },
                _ => None,
            })
            .expect("a scrim")
    };
    let blue = Srgba::rgb(0, 0, 255).linear();
    assert_eq!(fill("--scrim: #0000ff;", "background: var(--scrim);"), blue);
    // The column around the dialog sets --scrim red: without the dialog's
    // own, the backdrop inherits the column's through the dialog.
    assert_eq!(
        fill("", "background: var(--scrim);"),
        Srgba::rgb(255, 0, 0).linear()
    );
}

/// A customizable select (`appearance: base-select`, HTML's customizable
/// `<select>`) builds from its own children. Its `<button>` is the trigger,
/// and its `<selectedcontent>` shows the chosen option's content, rich
/// content included. A press opens `::picker(select)` under the select,
/// holding the options, and the select matches `:open` while it is.
/// The chosen option matches `:checked`, a disabled one `:disabled`, and
/// pressing a disabled option does nothing. Pressing an enabled one writes
/// its value to the binding, fires the change and closes the picker; a
/// press outside closes it without a change. A select with no button
/// shows the chosen option's label, and with no binding keeps the
/// player's choice as its own.
///
/// Ways it could fail: the native dropdown drawn instead; the picker
/// open from the start or not anchored below its select; the label
/// written back instead of the value; a disabled option chosen; the
/// trigger still showing the old option; `:checked` staying on the old
/// option; light dismiss writing a value; an unbound select losing the
/// choice.
#[test]
fn a_base_select_builds_its_trigger_picker_and_options_from_its_children() {
    use og_core::glam::Vec2;
    let doc = r#"<meta name="component" content="bs">
<column id="Root">
  <select id="S" bind:value={$pick}>
    <button id="Trig"><selectedcontent id="Shown"></selectedcontent></button>
    <option id="A" value="a"><b>alpha</b> one</option>
    <option id="B" value="b" disabled>beta</option>
    <option id="C" value="c">gamma</option>
  </select>
  <select id="U">
    <option id="X">first</option>
    <option id="Y" selected>second</option>
  </select>
  <p id="Away">away</p>
</column>
<style>
select { appearance: base-select; width: 100px; height: 30px; }
select:open { margin-left: 7px; }
option { width: 40px; height: 20px; }
option:checked { width: 60px; }
option:disabled { width: 30px; }
</style>"#;
    let mut d = support::Doc::new(doc, &[("pick", json!("c"))]);
    let texts = |d: &support::Doc, under: &str| -> Vec<String> {
        d.ui.probe_elements()
            .into_iter()
            .filter(|e| e.path.contains(&format!("/{under}/")) && !e.text.is_empty())
            .map(|e| e.text)
            .collect()
    };
    let x0 = d.get("S").x;
    assert!(d.el("A").is_none(), "the picker starts closed");
    assert_eq!(texts(&d, "Shown"), ["gamma"], "the bound option's content");
    assert_eq!(texts(&d, "U"), ["second"], "<option selected>'s label");

    d.click("S");
    let s = d.get("S");
    assert_eq!(s.x, x0 + 7.0, "select:open");
    let (a, b, c) = (d.get("A"), d.get("B"), d.get("C"));
    assert!(
        a.y >= s.y + s.height - 0.5 && a.y < s.y + s.height + 30.0,
        "{a:?} under {s:?}"
    );
    assert!(b.y > a.y && c.y > b.y, "options in order");
    assert_eq!(
        (a.width, b.width, c.width),
        (40.0, 30.0, 60.0),
        ":checked, :disabled"
    );

    // A disabled option takes no press.
    let events = d.click("B");
    assert!(!events.iter().any(|e| e.kind == "changed"), "{events:?}");
    assert_eq!(d.read("pick"), json!("c"));
    assert!(d.el("A").is_some(), "still open");

    let events = d.click("A");
    assert_eq!(d.read("pick"), json!("a"), "the value, not the label");
    assert!(events.iter().any(|e| e.kind == "changed"), "{events:?}");
    assert!(d.el("A").is_none(), "a choice closes the picker");
    assert_eq!(d.get("S").x, x0, "no longer :open");
    assert_eq!(
        texts(&d, "Shown"),
        ["alpha one"],
        "the chosen option's rich content"
    );

    d.click("S");
    assert_eq!(
        (d.get("A").width, d.get("C").width),
        (60.0, 40.0),
        ":checked moved"
    );
    let away = d.get("Away");
    d.click_at(Vec2::new(away.x + away.width + 50.0, away.y + 2.0));
    assert!(d.el("A").is_none(), "a press outside closes it");
    assert_eq!(d.read("pick"), json!("a"));

    // Unbound, with no button: its own value, shown as the chosen label.
    d.click("U");
    d.click("X");
    assert!(d.el("X").is_none());
    assert_eq!(texts(&d, "U"), ["first"]);
}
