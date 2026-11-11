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

use og_core::glam::Vec2;
use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::{Value, json};
use support::*;

pub(crate) const FORM: &str = r#"<meta name="component" content="form">
<column id="Form">
  <button id="Opener">Open</button>
  <details id="More" bind:open={$more}><summary>More options</summary><text id="Body">Hidden text</text></details>
  <details id="Lit"><summary>Literal</summary><text id="LitBody">Shown text</text></details>
  <meter id="Hp" value={$hp} min="0" max="100" low="25" high="75" optimum="90" />
  <dialog id="Ask" modal bind:open={$ask} aria-label="Quit?"><button id="Yes">Yes</button><button id="No">No</button></dialog>
  <dialog id="Note" bind:open={$note} closedby="closerequest" aria-label="Note"><button id="Ok">Ok</button></dialog>
  <select id="Pick" bind:value={$choice}>
    <optgroup label="Warm">
      <option value="r">Red</option>
      <option value="o" disabled>Orange</option>
    </optgroup>
    <optgroup label="Cool">
      <option value="b">Blue</option>
      <option value="c">Cyan</option>
    </optgroup>
  </select>
</column>
<style>
#Form { gap: 8px; padding: 8px; }
select { width: 260px; }
</style>"#;

pub(crate) const STORES: [(&str, Value); 5] = [
    ("choice", Value::Null),
    ("more", Value::Bool(false)),
    ("hp", Value::Null),
    ("ask", Value::Bool(false)),
    ("note", Value::Bool(false)),
];

pub(crate) struct Rig {
    pub ui: Ui,
}

impl Rig {
    pub fn new(ui: Ui, initial: &[(&str, Value)]) -> Rig {
        let mut ui = ui;
        ui.define_document(UiDocument::parse("ui/form.svelte", FORM).expect("the form parses"));
        let mut rig = Rig { ui };
        let mut inputs = std::collections::BTreeMap::new();
        for (name, value) in initial {
            let source = SourceId::new("test", name);
            rig.apply(json!({"op": "signal", "source": source, "initial": value}));
            inputs.insert(name.to_string(), source);
        }
        rig.ui
            .mount_document(
                SourceId::new("test", "form"),
                "form",
                inputs,
                Default::default(),
                &bind::NoEnv,
            )
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

    fn store(&mut self, name: &str) -> Value {
        self.apply(json!({"op": "read", "source": SourceId::new("test", name)}))["value"].clone()
    }

    fn set(&mut self, name: &str, value: Value) {
        self.apply(json!({"op": "set", "source": SourceId::new("test", name), "value": value}));
        self.step(InputFrame::default());
        self.step(InputFrame::default());
    }

    pub fn step(&mut self, input: InputFrame) {
        self.step_at(input, Viewport::new(800.0, 900.0));
    }

    pub fn step_at(&mut self, input: InputFrame, vp: Viewport) {
        self.ui.begin(input, vp);
        self.ui
            .build_retained_documents(&bind::NoEnv)
            .expect("the form builds");
        self.ui.end();
    }

    fn el(&self, id: &str) -> Option<UiElement> {
        self.ui.element(id).cloned()
    }

    fn click(&mut self, id: &str) {
        let e = self.el(id).unwrap_or_else(|| panic!("no {id}"));
        let at = Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0);
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
        self.settle();
    }

    /// Past the double-click window and every transition.
    fn settle(&mut self) {
        for _ in 0..40 {
            self.step(InputFrame::default());
        }
    }

    fn key(&mut self, key: UiKey) {
        self.step(InputFrame {
            keys: vec![key],
            ..InputFrame::default()
        });
        self.settle();
    }

    fn pad(&mut self, button: u32) {
        self.step(InputFrame {
            gamepad: GamepadFrame {
                pressed: 1 << button,
                ..GamepadFrame::default()
            },
            ..InputFrame::default()
        });
        self.settle();
    }

    fn typed(&mut self, text: &str) {
        self.step(InputFrame {
            text: text.into(),
            ..InputFrame::default()
        });
        self.settle();
    }

    fn focus(&self) -> String {
        self.ui.focus_path(0).to_string()
    }

    fn a11y(&self, path_end: &str) -> A11yNode {
        self.ui
            .a11y()
            .nodes
            .iter()
            .find(|n| n.path == path_end || n.path.ends_with(&format!("/{path_end}")))
            .cloned()
            .unwrap_or_else(|| panic!("no a11y node {path_end}"))
    }
}

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
                        og_render::UiPaint::Solid(c) => Some(c),
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

/// `references/vulkan/html-controls-{1280,390}.png`: the form with the
/// select, last, open below it (its groups, the disabled Orange, the tick
/// on Blue), the bound disclosure open, the meter in its poor region and
/// the dialog in place; `html-controls-modal-{1280,390}.png`: the modal
/// over its scrim (the focus moved in by program, so no ring). Ways it could fail: a
/// list drawn under the page or off its trigger, a group heading missing,
/// a closed disclosure's content showing, a meter in the wrong colour, a
/// modal with no backdrop or its panel off centre.
#[test]
#[ignore = "requires GPU"]
fn html_controls_match_their_reference_frames() {
    use og_render::*;
    for modal in [false, true] {
        for (width, height) in [(1280u32, 720u32), (390, 844)] {
            let mut scene = RenderScene::new();
            let font = add_test_font(&mut scene);
            let mut rig = Rig::new(ui_with_font(font), &STORES);
            let vp = Viewport::new(width as f32, height as f32);
            rig.step_at(InputFrame::default(), vp);
            for (name, value) in [
                ("choice", json!("b")),
                ("hp", json!(20)),
                ("more", json!(true)),
                ("note", json!(true)),
            ] {
                rig.apply(
                    json!({"op": "set", "source": SourceId::new("test", name), "value": value}),
                );
            }
            rig.step_at(InputFrame::default(), vp);
            if modal {
                rig.apply(
                    json!({"op": "set", "source": SourceId::new("test", "ask"), "value": true}),
                );
            } else {
                let e = rig.el("Pick/Trigger").unwrap();
                let at = Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0);
                for (pressed, released) in [(true, false), (false, true)] {
                    rig.step_at(
                        InputFrame {
                            pointer: at,
                            pointer_inside: true,
                            down: pressed,
                            pressed,
                            released,
                            ..InputFrame::default()
                        },
                        vp,
                    );
                }
            }
            for _ in 0..40 {
                rig.step_at(InputFrame::default(), vp);
            }
            let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
            rig.ui.frame().emit(&EmitDesc::default(), &mut overlay);
            scene.ui.overlay = Some(overlay);
            let bg = rig.ui.theme().palette.backdrop.linear();
            scene.lighting.environment.background =
                Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
            scene.look.post.tonemap = Tonemap::None;
            let mut renderer = Renderer::new_blocking(&RenderDesc {
                width,
                height,
                ..Default::default()
            })
            .expect("a headless adapter");
            let stats = renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
            assert_eq!(stats.missing_glyphs, 0);
            let image = renderer.read_offscreen().unwrap();
            let name = format!(
                "html-controls{}-{width}.png",
                if modal { "-modal" } else { "" }
            );
            image::save_buffer(
                out_dir().join(&name),
                &image.pixels,
                width,
                height,
                image::ColorType::Rgba8,
            )
            .unwrap();
            let own = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("references")
                .join(og_tests::compare::RECORDING_TIER)
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
            shape.half.x >= 399.0
                && matches!(shape.paint, og_render::UiPaint::Solid(c) if c == want)
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

/// Input on a mounted document's button runs its control on the node it
/// holds: hovering, pressing, releasing and activating it by key build no
/// element again and evaluate no binding, while the button still shows
/// each state, its click still fires its handler once, and a bound
/// control beside it still writes its store.
///
/// Ways it could fail: the hovered, pressed or focused button's element
/// emitted again each tick (as every control was), the states no longer
/// drawn (the frame not moving on hover or press), the click lost or fired
/// twice, or a content button left on the old path.
#[test]
fn input_on_a_mounted_button_builds_no_element_again() {
    use crate::html_text::Doc;
    use og_core::glam::Vec2;
    use serde_json::json;
    let mut d = Doc::new(
        r#"<meta name="component" content="t">
<column id="Root" gap="8" padding="12">
  <button id="Go" onclick={go}>Go</button>
  <button id="Buy" variant="primary" onclick={buy}><text id="Word">Buy</text></button>
  <label><input type="checkbox" switch id="On" bind:checked={$on}> On</label>
</column>"#,
        &[("on", json!(false))],
        true,
    );
    let pointer = |p: Vec2, down: bool, pressed: bool, released: bool| InputFrame {
        pointer: p,
        pointer_inside: true,
        down,
        pressed,
        released,
        ..Default::default()
    };
    let centre = |d: &Doc, id: &str| {
        let e = d.get(id);
        Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0)
    };
    let away = Vec2::new(700.0, 800.0);
    for _ in 0..4 {
        d.step(pointer(away, false, false, false));
    }
    for (id, handler) in [("Go", "go"), ("Buy", "buy")] {
        let p = centre(&d, id);
        let rest = d.ui.frame().items.clone();
        d.events.clear();
        let mut frames = Vec::new();
        for (what, input) in [
            ("hover", pointer(p, false, false, false)),
            ("hover", pointer(p, false, false, false)),
            ("press", pointer(p, true, true, false)),
            ("held", pointer(p, true, false, false)),
            ("release", pointer(p, false, false, true)),
            ("after", pointer(p, false, false, false)),
            ("leave", pointer(away, false, false, false)),
            ("left", pointer(away, false, false, false)),
        ] {
            d.step(input);
            let stats = d.ui.frame().stats;
            assert_eq!(
                (stats.bindings, stats.reconciled),
                (0, 0),
                "{id} {what}: {stats:?}"
            );
            frames.push(d.ui.frame().items.clone());
        }
        assert!(frames[1] != rest, "{id}: hover is drawn");
        assert!(frames[3] != frames[1], "{id}: the press is drawn");
        let fired: Vec<_> = d
            .events
            .iter()
            .filter(|e| e.name == handler || e.kind == "click")
            .map(|e| (e.kind.as_str(), e.name.as_str()))
            .collect();
        assert_eq!(fired, [("click", id), ("emit", handler)], "{id}");
    }
    // By key: the focus on the first button, Enter activates it.
    d.events.clear();
    d.focus = Some(d.get("Go").path);
    for (what, input) in [
        ("focus", InputFrame::default()),
        ("focused", InputFrame::default()),
        (
            "enter",
            InputFrame {
                keys: vec![UiKey::Enter],
                keys_pressed: vec!["Enter"],
                ..Default::default()
            },
        ),
        ("after", InputFrame::default()),
    ] {
        d.step(input);
        let stats = d.ui.frame().stats;
        assert_eq!(
            (stats.bindings, stats.reconciled),
            (0, 0),
            "key {what}: {stats:?}"
        );
    }
    assert!(d.ui.focus_path(0).ends_with("Go"), "{}", d.ui.focus_path(0));
    assert_eq!(d.events.iter().filter(|e| e.name == "go").count(), 1);
    // The switch beside them still writes its store.
    let p = centre(&d, "On");
    d.step(pointer(p, true, true, false));
    d.step(pointer(p, false, false, true));
    d.step(InputFrame::default());
    assert_eq!(d.read("on"), json!(true));
}
