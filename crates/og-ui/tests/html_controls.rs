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

use crate::support;

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

#[test]
fn meter_regions_follow_the_html_rules() {
    use MeterRegion::*;
    let d = |value, low, high, optimum| GaugeDesc {
        value,
        min: 0.0,
        max: 100.0,
        low: Some(low),
        high: Some(high),
        optimum: Some(optimum),
        ..GaugeDesc::default()
    };
    for (value, optimum, want) in [
        // Optimum high: above high best, the middle next, below low poor.
        (80.0, 90.0, Optimum),
        (75.0, 90.0, Suboptimum),
        (25.0, 90.0, Suboptimum),
        (24.0, 90.0, Poor),
        // Optimum low: the mirror.
        (10.0, 5.0, Optimum),
        (50.0, 5.0, Suboptimum),
        (76.0, 5.0, Poor),
        // Optimum in the middle: either end only suboptimal.
        (50.0, 50.0, Optimum),
        (10.0, 50.0, Suboptimum),
        (90.0, 50.0, Suboptimum),
    ] {
        assert_eq!(
            MeterRegion::of(&d(value, 25.0, 75.0, optimum)),
            want,
            "{value} with {optimum}"
        );
    }
    // Bounds out of order clamp as HTML's do: high under low is low.
    assert_eq!(MeterRegion::of(&d(30.0, 40.0, 20.0, 90.0)), Poor);
    // In a document, the fill is drawn in its region's colour.
    let mut rig = rig();
    let fill = |rig: &Rig| -> Vec<[f32; 4]> {
        let hp = rig.el("Hp").expect("the meter");
        rig.ui
            .frame()
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
    };
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
    parts.extend(modal.ui.parts().iter().flatten().map(|p| format!("{p}")));
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
