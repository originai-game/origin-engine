//! CSS for exposed widget parts: authored hosts, scoped rules, text,
//! fills, borders and glass. Transitions and keyframes must advance by
//! ticks, retarget from the current value and replay after restoration.
//! Unmatched parts have no class-dependent appearance fallback.

use crate::support::*;
use og_ui::*;
use serde_json::{Value, json};

/// `body` (a component document's markup and `<style>`) mounted in `ui`
/// with `stores`, laid out in `vp`, built once.
fn mounted(ui: Ui, body: &str, stores: &[(&str, Value)], vp: Viewport) -> Doc {
    let mut ui = ui;
    let text = format!("<meta name=\"component\" content=\"t\">\n{body}");
    ui.define_document(UiDocument::parse("ui/t.svelte", &text).unwrap_or_else(|e| panic!("{e:?}")));
    mount_stores(&mut ui, reactive::SourceId::new("test", "t"), "t", stores);
    let mut d = Doc {
        ui,
        events: Vec::new(),
        viewport: vp,
        focus: None,
    };
    d.step(InputFrame::default());
    d
}

/// The element (in units) whose path is `path` or ends in `/path`.
fn at(ui: &Ui, path: &str) -> UiElement {
    try_at(ui, path).unwrap_or_else(|| panic!("no {path}"))
}

fn try_at(ui: &Ui, path: &str) -> Option<UiElement> {
    let end = format!("/{path}");
    ui.elements()
        .iter()
        .find(|e| e.path == path || e.path.ends_with(&end))
        .cloned()
}

/// A press and release at `pointer`, a tick each.
fn press_at(d: &mut Doc, pointer: og_core::math::Vec2) {
    for down in [true, false] {
        d.step(InputFrame {
            pointer,
            pointer_inside: true,
            down,
            pressed: down,
            released: !down,
            ..Default::default()
        });
    }
}

#[test]
fn tabs_and_select_parts_keep_boxes_states_and_host_input() {
    let mut ui = ui_with_font(test_font());
    ui.set_part_css(
        "tabs::part(track) { background: #123456; padding: 5px; }
         tabs::part(indicator) { background: #654321; height: 7px; }
         tabs::part(tab) { background: #ff0000; padding: 3px; }
         tabs::part(tab):checked { background: #00ff00; }
         select::part(trigger) { background: #0000ff; padding: 4px; }
         select::part(list) { padding: 13px; border: 3px solid #123456; }
         select::part(group) { height: 37px; padding-left: 19px; }
         select::part(option) { background: #ff00ff; padding: 6px; }",
    )
    .unwrap();
    let mut d = mounted(
        ui,
        r#"<column id="Root">
  <tabs id="Tabs" items="First Second" bind:selected={$tab}></tabs>
  <select id="Select" bind:value={$choice}><optgroup label="Group"><option value="o">Option</option></optgroup></select>
</column>"#,
        &[("tab", json!(1)), ("choice", json!("o"))],
        Viewport::new(600.0, 400.0),
    );
    for _ in 0..2 {
        d.step(InputFrame::default());
    }
    let ui = &d.ui;
    let indicator = at(ui, "Tabs/Strip/Track/Thumb");
    assert_eq!(indicator.kind, "part");
    assert_eq!(indicator.height, 7.0);
    for (host, part, role) in [
        ("Tabs/Strip/Track/Item#0", "tab", "radio"),
        ("Select/Trigger", "trigger", "combobox"),
    ] {
        assert_eq!(at(ui, host).role, role);
        let visual = at(ui, &format!("{host}/::part({part})#0"));
        assert_eq!(visual.kind, "part");
        assert!(visual.role.is_empty());
    }
    for rgb in [[255, 0, 0], [0, 255, 0], [0, 0, 255]] {
        let expected = Srgba::rgb(rgb[0], rgb[1], rgb[2]).linear();
        assert!(
            ui.frame()
                .items
                .iter()
                .any(|i| i.cmd.filled().is_some_and(|(_, c)| c == expected)),
            "{rgb:?}"
        );
    }
    let first = at(ui, "Tabs/Strip/Track/Item#0");
    let pointer =
        og_core::math::Vec2::new(first.x + first.width / 2.0, first.y + first.height / 2.0);
    press_at(&mut d, pointer);
    d.step(InputFrame::default());
    assert_eq!(d.read("tab"), json!(0));
    let trigger = at(&d.ui, "Select/Trigger");
    let pointer = og_core::math::Vec2::new(
        trigger.x + trigger.width / 2.0,
        trigger.y + trigger.height / 2.0,
    );
    press_at(&mut d, pointer);
    for _ in 0..15 {
        d.step(InputFrame::default());
    }
    let ui = &d.ui;
    let option = ui.elements().iter().find(|e| e.role == "option").unwrap();
    let visual = at(ui, &format!("{}/::part(option)#0", option.path));
    assert!(visual.role.is_empty());
    assert_eq!(visual.kind, "part");
    let list = at(ui, "Select.list/::part(list)#0");
    let group = at(ui, "Select.list/List/Group#0");
    assert_eq!(list.kind, "part");
    assert!(list.role.is_empty());
    assert_eq!(group.kind, "part");
    assert_eq!(group.height, 37.0);
    assert_eq!(group.x - list.x, 16.0);
    let text = at(ui, "Select.list/List/Group#0/Text");
    assert_eq!(text.x - group.x, 19.0);
    assert!(ui.frame().items.iter().any(|i| {
        i.cmd
            .filled()
            .is_some_and(|(_, c)| c == [1.0, 0.0, 1.0, 1.0])
    }));
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

/// Two sliders and a switch in a 600×400 frame; the stepped slider's
/// value is the store `v`.
const RANGES: &str = r#"<column id="Root">
  <input type="range" id="Plain" min="0" max="1" value="0.3" />
  <input type="range" id="Ticks" min="0" max="10" step="1" list="Marks" aria-label="Volume" bind:value={$v} />
  <datalist id="Marks"><option value="0" label="Lo"></option><option value="10" label="Hi"></option></datalist>
  <input type="checkbox" switch id="Switch" checked />
</column>
<style>
#Root { padding: 20px; gap: 30px; }
#Plain { width: 200px; }
</style>"#;

/// [`RANGES`] in `ui`, built three times.
fn ranges(ui: Ui) -> Doc {
    let mut d = mounted(ui, RANGES, &[("v", json!(4))], Viewport::new(600.0, 400.0));
    for _ in 0..2 {
        d.step(InputFrame::default());
    }
    d
}

/// The default style, or `css` as the whole part stylesheet.
fn range_ui(css: Option<&str>) -> Ui {
    let mut ui = ui_with_font(test_font_metrics());
    if let Some(css) = css {
        ui.set_part_css(css).unwrap();
    }
    ui
}

fn rect(ui: &Ui, path: &str) -> [f32; 4] {
    let e = at(ui, path);
    [e.x, e.y, e.width, e.height]
}

/// The stepped slider's value now.
fn value(d: &mut Doc) -> f64 {
    d.read("v").as_f64().expect("a number")
}

#[test]
fn undressed_range_parts_keep_their_boxes_and_draws() {
    // The boxes and fills the slider drew before its parts were boxes.
    let mut d = ranges(range_ui(None));
    let ui = &d.ui;
    // Its datalist's options tick 0 and 10: the first and last of the
    // eleven steps' ticks.
    for (path, kind, expected) in [
        ("Plain/Track", "box", [20.0, 44.0, 200.0, 44.0]),
        (
            "Plain/Track/::part(reach)#0",
            "part",
            [20.0, 44.0, 200.0, 44.0],
        ),
        ("Plain/Track/Rail", "part", [33.0, 63.0, 174.0, 6.0]),
        ("Plain/Track/Rail/Fill", "part", [33.0, 63.0, 52.0, 6.0]),
        ("Plain/Track/Knob", "box", [72.0, 53.0, 26.0, 26.0]),
        (
            "Plain/Track/Knob/::part(knob)#0",
            "part",
            [72.0, 53.0, 26.0, 26.0],
        ),
        ("Ticks/Track/Rail/Tick#0", "part", [32.0, 158.0, 2.0, 12.0]),
        ("Ticks/Track/Rail/Tick#1", "part", [206.0, 158.0, 2.0, 12.0]),
        ("Ticks/Track/Knob", "box", [90.0, 151.0, 26.0, 26.0]),
    ] {
        assert_eq!(rect(ui, path), expected, "{path}");
        let e = at(ui, path);
        assert_eq!(e.kind == "part", kind == "part", "{path}: {}", e.kind);
        if kind == "part" {
            assert!(e.role.is_empty(), "{path}: {}", e.role);
        }
    }
    assert_eq!(at(ui, "Plain/Track").role, "slider");
    // The rail, fill and passed tick in their roles' colours, the knob's
    // own fill under its CSS rim.
    let fills: Vec<_> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| i.cmd.filled())
        .map(|(r, _)| [r.x, r.y, r.width, r.height])
        .collect();
    for expected in [
        [33.0, 63.0, 174.0, 6.0],
        [33.0, 63.0, 52.0, 6.0],
        [32.0, 158.0, 2.0, 12.0],
        [72.0, 53.0, 26.0, 26.0],
    ] {
        assert!(fills.contains(&expected), "{expected:?} in {fills:?}");
    }
    // A press on the track: the knob follows, its core shows, the
    // bubble fades in over it (its presence is the part's) and keeps
    // the width its painted rim had.
    let track = at(ui, "Ticks/Track");
    let at_ = og_core::math::Vec2::new(track.x + track.width * 0.62, track.y + track.height / 2.0);
    let press = |down, pressed| InputFrame {
        pointer: at_,
        pointer_inside: true,
        down,
        pressed,
        ..Default::default()
    };
    d.step(press(true, true));
    for _ in 0..4 {
        d.step(press(true, false));
    }
    assert_eq!(value(&mut d), 6.0, "pointer input reaches the slider host");
    let ui = &d.ui;
    assert_eq!(rect(ui, "Ticks/Track/Knob"), [124.0, 151.0, 26.0, 26.0]);
    assert_eq!(
        rect(ui, "Ticks/Track/Knob/Core"),
        [132.0, 159.0, 11.0, 11.0]
    );
    let bubble = rect(ui, "Ticks/Track/Bubble");
    assert_eq!([bubble[0], bubble[2], bubble[3]], [121.0, 24.0, 26.0]);
    assert!(bubble[1] > 113.0, "entering, the bubble rises: {bubble:?}");
    for _ in 0..30 {
        d.step(press(true, false));
    }
    let ui = &d.ui;
    let settled = rect(ui, "Ticks/Track/Bubble");
    assert!((settled[1] - 112.0).abs() < 0.01, "{settled:?}");
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

#[test]
fn range_part_css_sizes_and_states_its_boxes() {
    let mut d = ranges(range_ui(Some(
        "input[type=range]::part(track) { height: 10px; }
         input[type=range]::part(reach) { background: #0000ff; }
         input[type=range]::part(bubble) { padding: 0 11px; border: 2px solid #123456; }
         input[type=range]::part(knob) { width: 30px; }
         input[type=range]::part(knob):active { background: #00ff00; }
         input[type=range]::part(core) { background: #ff00ff; }",
    )));
    let ui = &d.ui;
    let reach = rect(ui, "Plain/Track/::part(reach)#0");
    assert_eq!(at(ui, "Plain/Track/Rail").height, 10.0);
    let blue = Srgba::rgb(0, 0, 255).linear();
    assert!(
        ui.frame().items.iter().any(|i| i
            .cmd
            .filled()
            .is_some_and(|(r, c)| c == blue && [r.x, r.y, r.width, r.height] == reach)),
        "the reach paints its own box"
    );
    assert_eq!(at(ui, "Plain/Track/Knob/::part(knob)#0").width, 30.0);
    let green = Srgba::rgb(0, 255, 0).linear();
    let magenta = Srgba::rgb(255, 0, 255).linear();
    let has = |ui: &Ui, c: [f32; 4]| {
        ui.frame()
            .items
            .iter()
            .any(|i| i.cmd.filled().is_some_and(|(_, f)| f == c))
    };
    assert!(!has(ui, green) && !has(ui, magenta));
    let track = at(ui, "Ticks/Track");
    let pointer =
        og_core::math::Vec2::new(track.x + track.width * 0.62, track.y + track.height / 2.0);
    // Held past the knob's state motion.
    for k in 0..30 {
        let pressed = k == 0;
        d.step(InputFrame {
            pointer,
            pointer_inside: true,
            down: true,
            pressed,
            ..Default::default()
        });
    }
    assert_eq!(value(&mut d), 6.0);
    let ui = &d.ui;
    assert!(has(ui, green), "the held knob is :active");
    assert!(has(ui, magenta), "the core takes its part's CSS");
    let bubble = at(ui, "Ticks/Track/Bubble");
    let text = at(ui, "Ticks/Track/Bubble/Text");
    assert_eq!(bubble.width, text.width + 2.0 * (11.0 + 2.0));
    assert_eq!(text.x - bubble.x, 13.0);
}

#[test]
fn widget_part_placement_follows_the_writing_direction() {
    // A part's own insets are in its layout's terms: in a right-to-left
    // frame the fill and the switch's knob stay on their track.
    let mut ui = range_ui(None);
    ui.set_direction(og_i18n::Direction::Rtl);
    let d = ranges(ui);
    let ui = &d.ui;
    assert_eq!(rect(ui, "Plain/Track/Rail"), [393.0, 63.0, 174.0, 6.0]);
    assert_eq!(rect(ui, "Plain/Track/Rail/Fill"), [515.0, 63.0, 52.0, 6.0]);
    assert_eq!(
        rect(ui, "Ticks/Track/Rail/Tick#0"),
        [566.0, 158.0, 2.0, 12.0]
    );
    let track = rect(ui, "Switch/Track");
    let knob = rect(ui, "Switch/Track/Knob");
    assert!(
        knob[0] >= track[0] && knob[0] + knob[2] <= track[0] + track[2],
        "{knob:?} in {track:?}"
    );
    assert!(
        knob[0] - track[0] < 5.0,
        "on, it sits at the left end: {knob:?}"
    );
}

/// A progress bar and a meter at 1.5× in a 257 px column, built three
/// times.
fn bars(ui: Ui) -> Doc {
    let mut d = mounted(
        ui,
        r#"<column id="Root">
  <progress id="Load" value="0.6"></progress>
  <meter id="Gauge" value="0.2" low="0.3" high="0.7" optimum="0.9"></meter>
</column>
<style>
#Root { padding: 13px; gap: 9px; width: 257px; }
#Load { width: 141px; }
</style>"#,
        &[],
        Viewport {
            dpr: 1.5,
            ..Viewport::new(800.0, 500.0)
        },
    );
    for _ in 0..2 {
        d.step(InputFrame::default());
    }
    d
}

fn fill_rects(ui: &Ui) -> Vec<[f32; 4]> {
    ui.frame()
        .items
        .iter()
        .filter_map(|i| i.cmd.filled())
        .map(|(r, _)| [r.x, r.y, r.width, r.height])
        .collect()
}

#[test]
fn bar_tracks_are_part_boxes_and_fills_keep_their_device_pixels() {
    let d = bars(ui_with_font(test_font_metrics()));
    let ui = &d.ui;
    for (host, role) in [("Load", "progressbar"), ("Gauge", "meter")] {
        let h = at(ui, host);
        assert_eq!(h.role, role, "{host}");
        let track = at(ui, &format!("{host}/::part(track)#0"));
        assert_eq!(track.kind, "part");
        assert!(track.role.is_empty());
        assert_eq!(
            [track.x, track.y, track.width, track.height],
            [h.x, h.y, h.width, h.height]
        );
    }
    // The fills are fractions of the track's device-pixel box, as before
    // the tracks were boxes: 60% of 140.668 px ends on the 98 px edge.
    let fills = fill_rects(ui);
    let load = at(ui, "Load");
    let expected = [load.x, load.y, 98.0 - load.x, 8.0];
    assert!(fills.contains(&expected), "{expected:?} in {fills:?}");
}

#[test]
fn bar_track_css_sizes_the_box_its_fill_is_drawn_in() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css(
        "progress::part(track) { height: 4px; background: #0000ff; }
         meter::part(track) { height: 6px; }",
    )
    .unwrap();
    let d = bars(ui);
    let ui = &d.ui;
    let track = at(ui, "Load/::part(track)#0");
    assert_eq!(track.height, 4.0);
    let fills = fill_rects(ui);
    let load = at(ui, "Load");
    assert!(
        fills.contains(&[load.x, load.y, 98.0 - load.x, 4.0]),
        "the fill is the track's height: {fills:?}"
    );
    let blue = Srgba::rgb(0, 0, 255).linear();
    assert!(ui.frame().items.iter().any(|i| {
        i.cmd
            .filled()
            .is_some_and(|(r, c)| c == blue && r.height == 4.0)
    }));
    let gauge = at(ui, "Gauge/::part(track)#0");
    assert_eq!(gauge.height, 6.0);
    assert!(
        fills
            .iter()
            .any(|r| r[1] == gauge.y && r[3] == 6.0 && r[2] < gauge.width),
        "the meter's fill is the track's height: {fills:?}"
    );
}

/// A date input in a 600×500 frame: a stepper per part, the year first;
/// its value is the store `when`. Built three times.
fn steppers(ui: Ui) -> Doc {
    let mut d = mounted(
        ui,
        r#"<column id="Root"><input type="date" id="When" bind:value={$when} /></column>
<style>#Root { padding: 13px; }</style>"#,
        &[("when", json!("2000-01-01"))],
        Viewport::new(600.0, 500.0),
    );
    for _ in 0..2 {
        d.step(InputFrame::default());
    }
    d
}

#[test]
fn stepper_buttons_and_value_are_part_boxes_with_their_rims_inside() {
    let mut d = steppers(ui_with_font(test_font_metrics()));
    let ui = &d.ui;
    for (host, part, role) in [
        ("When/Year/Frame/Dec", "button", "button"),
        ("When/Year/Frame/Value", "value", "spinbutton"),
        ("When/Year/Frame/Inc", "button", "button"),
    ] {
        assert_eq!(at(ui, host).role, role, "{host}");
        let visual = at(ui, &format!("{host}/::part({part})#0"));
        assert_eq!(visual.kind, "part");
        assert!(visual.role.is_empty());
    }
    // The default 1.5 px rim sits inside the value's inset, as it was
    // painted: the frame is its two buttons and its value side by side,
    // 44 px tall (after the part's label), and the rim moves no text: a
    // four-digit year is narrower than the value's least width, so it is
    // centred in it.
    let frame = rect(ui, "When/Year/Frame");
    let [dec, value, inc] =
        ["Dec", "Value", "Inc"].map(|p| rect(ui, &format!("When/Year/Frame/{p}")));
    assert_eq!([frame[1], frame[3]], [13.0, 44.0]);
    assert_eq!(frame[2], dec[2] + value[2] + inc[2]);
    assert_eq!(value[0], frame[0] + dec[2]);
    let text = rect(ui, "When/Year/Frame/Value/Text");
    assert!(
        (text[0] - value[0] - (value[2] - text[2]) / 2.0).abs() <= 0.5,
        "{text:?} centred in {value:?}"
    );
    // The button host still takes the press.
    let inc = at(ui, "When/Year/Frame/Inc");
    press_at(
        &mut d,
        og_core::math::Vec2::new(inc.x + inc.width / 2.0, inc.y + inc.height / 2.0),
    );
    d.step(InputFrame::default());
    assert_eq!(d.read("when"), json!("2001-01-01"));
}

#[test]
fn stepper_part_css_pads_borders_and_states_its_boxes() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css(
        "input[type=date]::part(value) { padding: 0 20px; border: 2px solid #123456; }
         input[type=date]::part(button):hover { background: #00ff00; transition: none; }",
    )
    .unwrap();
    let mut d = steppers(ui);
    let ui = &d.ui;
    let text = at(ui, "When/Year/Frame/Value/Text");
    let part = at(ui, "When/Year/Frame/Value/::part(value)#0");
    assert_eq!(part.width, text.width + 2.0 * (20.0 + 2.0));
    assert_eq!(text.x - part.x, 22.0);
    let green = Srgba::rgb(0, 255, 0).linear();
    let inc = at(ui, "When/Year/Frame/Inc");
    let pointer = og_core::math::Vec2::new(inc.x + inc.width / 2.0, inc.y + inc.height / 2.0);
    for _ in 0..3 {
        d.step(InputFrame {
            pointer,
            pointer_inside: true,
            ..Default::default()
        });
    }
    assert!(
        d.ui.frame().items.iter().any(|i| i
            .cmd
            .filled()
            .is_some_and(|(r, c)| c == green && r.x == inc.x && r.width == inc.width)),
        "the hovered button's box takes its :hover"
    );
}

#[test]
fn choice_parts_have_independent_css_layout_and_paint() {
    // (A radio group's cards variant, `input[type=radio]::part(chosen)`, is
    // no document's: no element builds it.)
    let mut ui = ui_with_font(test_font());
    ui.set_part_css(r#"
input { color: #d3b517; font-size: 21px; }
input[type=checkbox]::part(box) { width: 34px; height: 30px; background: #fb0307; translate: 7px 0; opacity: 0.5; border: none; transition: none; }
input[type=checkbox][switch]::part(track) { width: 70px; height: 32px; background: #03fb07; border: none; transition: none; }
input[type=radio]::part(ring) { width: 38px; height: 26px; background: #0307fb; border: none; transition: none; }
input[type=checkbox]::part(mark) { color: #2f7a11; }
"#).unwrap();
    let mut d = mounted(
        ui,
        r#"<column id="Root">
  <label><input type="checkbox" id="Check" checked /> Check</label>
  <label><input type="checkbox" switch id="Switch" checked /> Switch</label>
  <label><input type="radio" id="Radio" value="a" bind:group={$pick} /> Radio</label>
</column>"#,
        &[("pick", json!("a"))],
        Viewport::new(600.0, 400.0),
    );
    for _ in 0..3 {
        d.step(InputFrame::default());
        let ui = &d.ui;
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        for (path, width, height) in [
            ("Check/Box", 34.0, 30.0),
            ("Switch/Track", 70.0, 32.0),
            ("Radio/Options/Option#0/Ring", 38.0, 26.0),
        ] {
            let part = at(ui, path);
            assert_eq!((part.width, part.height), (width, height), "{path}");
        }
        let fills: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .map(|(_, c)| c)
            .collect();
        // The tick takes its mark's ink.
        for rgb in [[251, 3, 7], [3, 251, 7], [3, 7, 251], [47, 122, 17]] {
            let expected = Srgba::rgb(rgb[0], rgb[1], rgb[2]).linear();
            assert!(
                fills.iter().any(|c| c[..3] == expected[..3]),
                "{rgb:?}: {fills:?}"
            );
        }
        let row = at(ui, "Check/::part(row)#0");
        assert_eq!(at(ui, "Check").role, "checkbox");
        assert!(row.role.is_empty());
        for label in ["Check", "Switch", "Radio"] {
            let text = ui
                .frame()
                .items
                .iter()
                .find_map(|item| {
                    match match &item.cmd {
                        DrawCmd::StyledText { text, .. } => &**text,
                        cmd => cmd,
                    } {
                        DrawCmd::Text {
                            text, size, color, ..
                        } if text == label => Some((*size, *color)),
                        _ => None,
                    }
                })
                .expect("choice label");
            assert_eq!(text, (21.0, Srgba::rgb(211, 181, 23).linear()), "{label}");
        }
    }
}

#[test]
fn button_parts_keep_separate_boxes_and_host_event_ownership() {
    let tick = |ui: &mut Ui, input| {
        ui.begin(input, Viewport::new(600.0, 400.0));
        ui.end();
    };
    let mut ui = ui_with_font(test_font());
    ui.define_document(
        UiDocument::parse(
            "part-box.svelte",
            r#"<meta name="screen" content="parts" data-layer="hud">
<div id="Host">{#if $show}<button id="Button" onclick={activate}>Go</button>{/if}</div>
<style>
#Button {
  width: 180px; height: 60px; translate: 24px 0;
  color: blue; font-size: 22px;
  justify-content: flex-start; align-items: flex-start;
}
button::part(base) {
  width: 80px; height: 22px; translate: 12px 8px;
  background: #ff0000; padding: 0;
}
</style>"#,
        )
        .unwrap(),
    );
    mount_screen(&mut ui, "parts", &[("show", serde_json::json!(true))]);
    for _ in 0..3 {
        tick(&mut ui, InputFrame::default());
        let host = ui.element("parts/Host/Button").unwrap();
        let part = ui.element("parts/Host/Button/::part(base)#0").unwrap();
        assert_eq!((host.width, host.height), (180.0, 60.0));
        assert_eq!((part.width, part.height), (80.0, 22.0));
        assert_eq!((part.x - host.x, part.y - host.y), (12.0, 8.0));
        assert_eq!(host.role, "button");
        assert!(part.role.is_empty());
        let text = ui.frame().items.iter().find_map(|item| {
            let cmd = match &item.cmd {
                DrawCmd::StyledText { text, .. } => text.as_ref(),
                cmd => cmd,
            };
            match cmd {
                DrawCmd::Text {
                    text, size, color, ..
                } if text == "Go" => Some((*size, *color)),
                _ => None,
            }
        });
        assert_eq!(text, Some((22.0, [0.0, 0.0, 1.0, 1.0])));
    }
    ui.set_part_css(
        "button::part(base) { animation: fade 200ms linear forwards; }
        @keyframes fade { from { opacity: 0.2; } to { opacity: 0.8; } }",
    )
    .unwrap();
    let mut alpha = Vec::new();
    for _ in 0..15 {
        tick(&mut ui, InputFrame::default());
        alpha.push(
            ui.frame()
                .items
                .iter()
                .find_map(|item| {
                    item.cmd
                        .filled()
                        .filter(|(_, color)| color[0] == 1.0 && color[1] == 0.0)
                        .map(|(_, color)| color[3])
                })
                .unwrap(),
        );
    }
    for (at, expected) in [(0, 0.2), (6, 0.5), (12, 0.8)] {
        assert!((alpha[at] - expected).abs() < 0.001, "{alpha:?}");
    }
    assert_eq!(
        ui.frame().stats.painted,
        0,
        "settled part returns to cached commands"
    );
    ui.take_doc_events();
    let part = ui.element("parts/Host/Button/::part(base)#0").unwrap();
    let pointer = og_core::math::Vec2::new(part.x + 10.0, part.y + 10.0);
    for down in [true, false] {
        tick(
            &mut ui,
            InputFrame {
                pointer,
                pointer_inside: true,
                down,
                pressed: down,
                released: !down,
                ..Default::default()
            },
        );
    }
    assert_eq!(
        ui.take_doc_events()
            .into_iter()
            .filter(|event| event.kind == "emit")
            .map(|event| event.name)
            .collect::<Vec<_>>(),
        ["activate"]
    );
    set_store(&mut ui, "show", serde_json::json!(false));
    tick(&mut ui, InputFrame::default());
    assert!(ui.element("parts/Host/Button").is_none());
    assert!(ui.element("parts/Host/Button/::part(base)#0").is_none());
    set_store(&mut ui, "show", serde_json::json!(true));
    tick(&mut ui, InputFrame::default());
    assert_eq!(ui.elements().iter().filter(|e| e.kind == "part").count(), 1);
}

#[test]
fn registry_parts_on_text_hosts_are_boxes_and_hosts_accept_color() {
    for (markup, css) in [
        ("<h2>Title</h2>", "h2::part(flourish) { background: red; }"),
        ("<button>Go</button>", "button { color: red; }"),
        ("<hr />", "hr { color: red; }"),
    ] {
        og_ui::style_pack::StylePack::validate_source("kit.css", css, Vec::new()).unwrap();
        let source =
            format!("<meta name=\"component\" content=\"kit\">{markup}<style>{css}</style>");
        UiDocument::parse("kit.svelte", &source).unwrap();
    }
}

#[test]
fn host_font_and_color_reach_button_ink_and_divider_output() {
    let mut handles = Handles::default();
    let body = handles.test_font();
    let display = handles.test_font();
    let body_handle = body.handle.unwrap();
    let display_handle = display.handle.unwrap();
    let mut ui = Ui::new(UiDesc {
        fonts: Fonts {
            body,
            families: vec![FamilyFace {
                family: "Fantasy Display".into(),
                weight: (100, 900),
                font: display,
                ..Default::default()
            }],
            ..Default::default()
        },
        ..Default::default()
    });
    ui.define_document(UiDocument::parse("host-ink.svelte", r#"
<meta name="screen" content="host-ink" data-layer="hud">
<column><button id="Inherited">Inherited</button><button id="Override">Override</button><hr id="Rule" /></column>
<style>
button { color: #d12345; font-family: "Fantasy Display"; }
#Override::part(base) { color: blue; font-family: sans-serif; }
hr { width: 173px; color: #23d145; }
</style>"#).unwrap());
    ui.push_screen("host-ink", serde_json::json!({})).unwrap();
    for _ in 0..3 {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
        ui.end();
        for (label, expected_font, expected_color) in [
            (
                "Inherited",
                display_handle,
                Srgba::rgb(209, 35, 69).linear(),
            ),
            ("Override", body_handle, Srgba::rgb(0, 0, 255).linear()),
        ] {
            let actual = ui.frame().items.iter().find_map(|item| {
                let cmd = match &item.cmd {
                    DrawCmd::StyledText { text, .. } => &**text,
                    cmd => cmd,
                };
                match cmd {
                    DrawCmd::Text {
                        text, font, color, ..
                    } if text == label => Some((*font, *color)),
                    _ => None,
                }
            });
            assert_eq!(actual, Some((expected_font, expected_color)), "{label}");
        }
        let expected = Srgba::rgb(35, 209, 69).linear();
        assert!(
            ui.frame().items.iter().any(|item| item
                .cmd
                .filled()
                .is_some_and(|(r, c)| r.width == 173.0 && c == expected)),
            "hr color must reach the drawn rule"
        );
    }
}

/// The drawn text commands whose text is `label`: font, ink and tracking (em).
fn drawn_text(ui: &Ui, label: &str) -> Vec<(og_draw::FontHandle, [f32; 4], f32)> {
    ui.frame()
        .items
        .iter()
        .map(|item| match &item.cmd {
            DrawCmd::StyledText { text, .. } => &**text,
            cmd => cmd,
        })
        .filter_map(|cmd| match cmd {
            DrawCmd::Text {
                text,
                font,
                color,
                tracking,
                ..
            } if text == label => Some((*font, *color, *tracking)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_parts_text_properties_reach_the_widget_text_inside_it() {
    // CSS inheritance: a tab's label, a select's value and a date
    // stepper's value are text inside their part, so the font, case,
    // spacing and ink a pack's hooks declare on the part reach them, as
    // they reach a part's own text (a number field's value).
    let mut handles = Handles::default();
    let body = handles.test_font();
    let display = handles.test_font();
    let display_handle = display.handle.unwrap();
    let mut ui = Ui::new(UiDesc {
        fonts: Fonts {
            body,
            families: vec![FamilyFace {
                family: "Fantasy Display".into(),
                weight: (100, 900),
                font: display,
                ..Default::default()
            }],
            ..Default::default()
        },
        ..Default::default()
    });
    ui.set_part_css(&format!(
        "{DEFAULT_CSS}
         :root {{ --tabs-font: \"Fantasy Display\"; --tabs-transform: uppercase; --tabs-tracking: 3px;
                  --tabs-tab-ink: #d12345; --field-font: \"Fantasy Display\"; }}"
    ))
    .unwrap();
    let d = mounted(
        ui,
        r#"<column id="Root">
  <tabs id="Tabs" items="First Second" bind:selected={$tab}></tabs>
  <select id="Select" bind:value={$choice}><option value="o">Choose</option></select>
  <input type="number" id="Number" value="3" min="0" max="9" />
  <input type="date" id="When" value="2000-01-01" />
</column>"#,
        &[("tab", json!(1)), ("choice", json!("o"))],
        Viewport::new(600.0, 400.0),
    );
    let ui = &d.ui;
    let ink = Srgba::rgb(209, 35, 69).linear();
    for (label, selected) in [("FIRST", false), ("SECOND", true)] {
        let drawn = drawn_text(ui, label);
        assert!(!drawn.is_empty(), "{label} drawn uppercase");
        for (font, color, tracking) in drawn {
            assert_eq!(font, display_handle, "{label}: the tab's family");
            // The selected tab takes `--tabs-tab-selected-ink`, unset here.
            assert_eq!(color == ink, !selected, "{label}: the tab's ink");
            assert!(
                tracking > 0.1,
                "{label}: the tab's 3 px tracking, got {tracking} em"
            );
        }
    }
    for label in ["Choose", "3", "2000"] {
        let drawn = drawn_text(ui, label);
        assert!(!drawn.is_empty(), "{label} drawn");
        assert!(
            drawn.iter().all(|(font, ..)| *font == display_handle),
            "{label}: the part's family"
        );
    }
}

#[test]
fn part_host_negation_and_part_states_select_distinct_paint() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css(
        "button::part(base):hover { opacity: 0.3; border: 1px solid red; }
         button.primary::part(base) {
           opacity: 0.7; border-image: linear-gradient(white, black) 1 / 2px;
         }
         button:not(.primary)::part(base) { opacity: 0.9; }
         button:not(.primary)::part(base):hover { opacity: 0.4; }",
    )
    .unwrap();
    for (primary, hover, expected) in [
        (true, false, 0.7),
        (true, true, 0.7),
        (false, false, 0.9),
        (false, true, 0.4),
    ] {
        let part = dress::PartRef::of("button", "base", None)
            .flag_if(primary, "primary")
            .flag_if(hover, "hover");
        let draw = ui.resolve_part(&part, [80.0, 30.0]).unwrap();
        assert_eq!(draw.opacity, expected);
        assert_eq!(
            draw.boxes.is_some_and(|boxes| boxes.border.is_some()),
            primary
        );
    }
}

#[test]
fn retained_part_nodes_animate_without_rebuilding_the_host() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(
        UiDocument::parse(
            "retained.svelte",
            r#"
        <meta name="component" content="retained">
        <button>Go</button>
        <style>
        button::part(base) { background: red; animation: fade 200ms linear forwards; }
        @keyframes fade { from { opacity: 0.2; } to { opacity: 0.8; } }
        </style>"#,
        )
        .unwrap(),
    );
    ui.mount_document(
        reactive::SourceId::new("test", "retained"),
        "retained",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let mut alpha = Vec::new();
    for _ in 0..15 {
        ui.begin(InputFrame::default(), Viewport::new(300.0, 100.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        alpha.push(
            ui.frame()
                .items
                .iter()
                .find_map(|item| item.cmd.filled().map(|(_, c)| c[3]))
                .unwrap(),
        );
    }
    for (at, expected) in [(0, 0.2), (6, 0.5), (12, 0.8)] {
        assert!((alpha[at] - expected).abs() < 0.001, "{alpha:?}");
    }
    assert_eq!(ui.frame().stats.painted, 0);
}

/// A tight field has no inset and its text's line for a height, so a
/// caller's own box holds the line exactly (the 大观园 page's verse box);
/// its style's text parts set the text's step and inks per component,
/// and an `ink` on a frame is refused. Wrong: the text took the
/// body step and ink, the placeholder inkFaint, inside a control's inset.
#[test]
fn a_tight_field_takes_its_components_text_parts() {
    let css = r##"input.poem::part(text) {
  color: #c0392b;
  font-size: 24px;
  line-height: 32px;
}
input.poem::part(placeholder) {
  color: #7f8c8d;
}
"##;
    let mut ui = ui_with_font(test_font());
    ui.set_part_css(css).unwrap();
    let lg = TypeStep {
        size: 24.0,
        line_height: 32.0,
        ..ui.theme().type_ramp.lg
    };
    let body = ui.theme().type_ramp.body;
    let texts = |ui: &Ui| -> Vec<(String, f32, f32, [f32; 4])> {
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match it.cmd.unstyled() {
                DrawCmd::Text {
                    text,
                    x,
                    size,
                    color,
                    ..
                } => Some((text.clone(), *x, *size, *color)),
                DrawCmd::StyledText { text, .. } => match &**text {
                    DrawCmd::Text {
                        text,
                        x,
                        size,
                        color,
                        ..
                    } => Some((text.clone(), *x, *size, *color)),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    };
    let mut d = mounted(
        ui,
        r#"<column id="Page">
  <input id="Verse" tight component="input.poem" placeholder="A line" bind:value={$poem} />
  <input id="Plain" bind:value={$plain} />
</column>
<style>#Verse { width: 320px; appearance: none; }</style>"#,
        &[("poem", json!("")), ("plain", json!("x"))],
        Viewport::new(600.0, 300.0),
    );
    let ui = &d.ui;
    let verse = at(ui, "Verse");
    assert_eq!(
        verse.height, lg.line_height,
        "one line of its lg text, no inset"
    );
    let t = texts(ui);
    let hint = t.iter().find(|x| x.0 == "A line").expect("the placeholder");
    assert!(
        (hint.1 - verse.x).abs() < 0.5,
        "no inset: {} at {}",
        hint.1,
        verse.x
    );
    assert_eq!(hint.2, lg.size, "the placeholder takes the text's step");
    assert!(
        near(hint.3, og_ui::style_file::parse_color("#7f8c8d").unwrap()),
        "{:?}",
        hint.3
    );
    let plain = t
        .iter()
        .find(|x| x.0 == "x")
        .expect("the plain field's value");
    assert_eq!(plain.2, body.size, "an undressed field keeps the body step");
    assert!(near(plain.3, ui.theme().palette.ink));
    d.set("poem", json!("Spring"));
    let line = texts(&d.ui)
        .into_iter()
        .find(|x| x.0 == "Spring")
        .expect("the value");
    assert_eq!(line.2, lg.size);
    assert!(
        near(line.3, og_ui::style_file::parse_color("#c0392b").unwrap()),
        "{:?}",
        line.3
    );
    let e =
        d.ui.set_part_css("input.poem::part(frame) { -og-ink: red; }")
            .unwrap_err();
    assert!(e.contains("-og-ink"), "{e}");
    // On a touch screen a tight field is still a finger's target.
    let touch = Viewport {
        coarse_pointer: true,
        ..Viewport::new(600.0, 300.0)
    };
    let d = mounted(
        ui_with_font(test_font()),
        r#"<column id="Page">
  <input id="Verse" tight />
  <textarea id="Line" tight rows="1"></textarea>
</column>
<style>#Verse { appearance: none; }</style>"#,
        &[],
        touch,
    );
    for path in ["Verse", "Line"] {
        let e = at(&d.ui, path);
        assert!(
            e.height >= 44.0 - 0.01,
            "{path} is {} tall on a touch screen",
            e.height
        );
    }
}

#[test]
fn scroll_bars_and_heading_flourishes_have_independent_dressings() {
    for authored in [false, true] {
        let mut ui = dressed(if authored {
            concat!(
                "scroll[axis=horizontal]::part(bar) { background: #ff00ff; height: 7px; }\n",
                "scroll[axis=vertical]::part(bar) { background: #00ffff; width: 9px; }\n",
                "h4::part(flourish) { width: 12px; height: 5px; color: #ff00ff; opacity: 0.4; }\n",
            )
        } else {
            ""
        });
        let mut handles = Handles::default();
        let mut fonts = ui.fonts().clone();
        fonts.body = handles.test_font();
        ui.set_fonts(fonts);
        let texture = handles.texture();
        let mut theme = *ui.theme();
        theme.art.atlas = Some(texture);
        theme.art.flourish = Some(og_ui::style_file::Ornament {
            uv: [0.0, 0.0, 1.0, 1.0],
            w: 10.0,
            h: 10.0,
            ink: og_ui::style_file::Ink::Authored,
            offset: 0.0,
        });
        ui.set_theme(theme);
        // A centred 16 px heading over a list that scrolls both ways, in a
        // column as wide as the frame.
        let d = mounted(
            ui,
            r#"<column id="Content">
  <h4 id="Title">Go</h4>
  <scroll id="List"><div id="Inner"></div></scroll>
</column>
<style>
#Content { width: 200px; }
#Title { text-align: center; font-size: 16px; width: stretch; margin: 0; }
#List { width: 100px; height: 50px; overflow: auto; }
#Inner { width: 200px; height: 120px; flex: none; }
</style>"#,
            &[],
            Viewport::new(200.0, 130.0),
        );
        let ui = &d.ui;
        let frame = ui.frame();
        assert_eq!(at(ui, "List").width, 100.0);
        let flourishes: Vec<_> = frame
            .items
            .iter()
            .filter_map(|item| match &item.cmd {
                DrawCmd::Image { rect, color, .. } => Some((*rect, *color)),
                _ => None,
            })
            .collect();
        assert_eq!(
            flourishes.len(),
            if authored { 2 } else { 0 },
            "small headings opt in only when dressed"
        );
        if authored {
            for (rect, color) in flourishes {
                assert_eq!((rect.width, rect.height), (12.0, 5.0));
                assert_eq!(color, Srgba::rgb(255, 0, 255).fade(0.4).linear());
            }
            let fills: Vec<_> = frame
                .items
                .iter()
                .filter_map(|item| item.cmd.filled())
                .collect();
            assert!(
                fills
                    .iter()
                    .any(|(r, c)| r.height == 7.0 && *c == Srgba::rgb(255, 0, 255).linear())
            );
            assert!(
                fills
                    .iter()
                    .any(|(r, c)| r.width == 9.0 && *c == Srgba::rgb(0, 255, 255).linear())
            );
        }
    }
}

/// `fill none` on a control's state clears the control's own fill: a quiet
/// button's disabled state, dressed `fill none`, draws no box (it drew the
/// control fill, louder than the clear quiet button at rest), while a
/// disabled secondary button the rule does not name keeps its fill.
#[test]
fn fill_none_clears_a_controls_own_fill() {
    let ui = dressed("button.quiet::part(base):disabled { background: none; }\n");
    let d = mounted(
        ui,
        r#"<column>
  <button id="Quiet" variant="quiet" disabled>Equip</button>
  <button id="Plain" disabled>Equip</button>
</column>"#,
        &[],
        Viewport::new(800.0, 600.0),
    );
    let fills = |ui: &Ui, id: &str| {
        let e = at(ui, id);
        fills_in(ui, Rect::new(e.x, e.y, e.width, e.height))
    };
    assert_eq!(
        fills(&d.ui, "Quiet"),
        Vec::<[f32; 4]>::new(),
        "the cleared quiet button"
    );
    assert!(
        !fills(&d.ui, "Plain").is_empty(),
        "a disabled secondary keeps its fill"
    );
}

/// A document element's `part` attribute is the part its style dresses
/// (docs/ui.md §3.2), with its classes and ARIA states as qualifiers and a
/// control's own states kept: a kit's `<button part="Loadout.item">` is
/// `Loadout::item:hover` under the pointer. A core frame with a qualifier,
/// literal or bound (`Slot.frame[rarity=epic]`), needs no `parts` entry and
/// takes the style's rarity dressing. Wrong: the attribute reached no draw
/// (a bare box, a `Button::base` button), a class or `aria-selected` never
/// qualified it, the control lost its hover and variant, or a core frame
/// was refused as a part the document does not list.
/// A part list names an element by each of its names (CSS Shadow Parts
/// §4.1): `::part()` reaches it by any of them, the document's `[part~=]`
/// by its own, and it wears the role of its last name (docs/ui.md §3.2).
/// Wrong: a list matched as one name, a role read from the first name, or
/// a name missing from the `parts` line let through.
#[test]
fn a_part_list_names_its_element_by_each_name_and_wears_the_last_role() {
    let mut d = mounted(
        dressed(
            "div::part(t\\.portrait) { width: 70px; }\n\
             :root { --picture-fill: #00ff00; }",
        ),
        r#"<meta name="parts" content="t.portrait t.picture t.badge">
<column id="Root">
  <div id="Face" part="t.portrait t.picture">a</div>
  <div id="Other" part="t.badge">b</div>
</column>
<style>#Root { align-items: flex-start; } [part~="t.portrait"] { height: 30px; }</style>"#,
        &[],
        Viewport::new(400.0, 300.0),
    );
    d.step(InputFrame::default());
    let face = at(&d.ui, "Face");
    assert_eq!([face.width, face.height], [70.0, 30.0]);
    let fills = fills_in(&d.ui, Rect::new(face.x, face.y, face.width, face.height));
    let green = og_ui::style_file::parse_color("#00ff00").unwrap();
    assert!(fills.iter().any(|f| near(*f, green)), "{fills:?}");
    let bad = UiDocument::parse(
        "ui/u.svelte",
        r#"<meta name="component" content="u"><meta name="parts" content="u.portrait">
<div part="u.portrait u.picture">a</div>"#,
    );
    assert!(
        format!("{bad:?}").contains("'u.picture' is not in the document's `parts` line"),
        "{bad:?}"
    );
}

#[test]
fn a_document_part_takes_its_dressing_classes_and_states() {
    use og_ui::reactive::SourceId;
    let mut ui = dressed(concat!(
        "div::part(box) { background: #00ff00; }\n",
        "div.hot::part(box) { background: #ffff00; }\n",
        "button::part(item) { background: #00ffff; }\n",
        "button::part(item):hover { background: #ffffff; }\n",
        "button[aria-selected=true]::part(item) { background: #ff8800; }\n",
        "div[data-rarity=epic]::part(frame) { background: #ff00ff; }\n",
        "div[data-rarity=rare]::part(frame) { background: #0000ff; }\n",
    ));
    let doc = r#"<meta name="component" content="kit">
<meta name="parts" content="box item frame">
<div id="Root">
  <div id="Box" part="box">a</div>
  <div id="Hot" part="box" class:hot={true}>b</div>
  <button id="Item" part="item">c</button>
  <button id="Picked" part="item" aria-selected={true}>d</button>
  <div id="Epic" part="frame" data-rarity="epic">e</div>
  <div id="Rare" part="frame" data-rarity={"rare"}>f</div>
</div>
<style>
#Root { width: stretch; padding: 8px; gap: 8px; }
#Box, #Hot, #Epic, #Rare { width: 100px; height: 30px; }
</style>"#;
    ui.define_document(UiDocument::parse("ui/kit.svelte", doc).unwrap());
    ui.mount_document(
        SourceId::new("test", "kit"),
        "kit",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let step = |ui: &mut Ui, pointer: Option<(f32, f32)>| {
        let input = InputFrame {
            pointer: pointer.map_or(Default::default(), |(x, y)| og_core::glam::Vec2::new(x, y)),
            pointer_inside: pointer.is_some(),
            ..InputFrame::default()
        };
        ui.begin(input, Viewport::new(600.0, 400.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
    };
    for _ in 0..3 {
        step(&mut ui, None);
    }
    let part = |ui: &Ui, id: &str| -> String {
        let k = ui
            .elements()
            .iter()
            .position(|e| e.path.ends_with(&format!("/{id}")))
            .unwrap_or_else(|| panic!("{id}"));
        ui.parts()[k].map(|p| p.to_string()).unwrap_or_default()
    };
    let fill = |ui: &Ui, id: &str| -> Vec<[f32; 4]> {
        let e = ui
            .elements()
            .iter()
            .find(|e| e.path.ends_with(&format!("/{id}")))
            .unwrap()
            .clone();
        fills_in(ui, Rect::new(e.x, e.y, e.width, e.height))
    };
    let has = |fills: Vec<[f32; 4]>, hex: &str| {
        let c = og_ui::style_file::parse_color(hex).unwrap();
        fills.iter().any(|f| near(*f, c))
    };
    assert!(part(&ui, "Box").ends_with("::part(box)"));
    assert!(has(fill(&ui, "Box"), "#00ff00"), "{:?}", fill(&ui, "Box"));
    assert!(part(&ui, "Hot").contains(".hot"), "{}", part(&ui, "Hot"));
    assert!(has(fill(&ui, "Hot"), "#ffff00"));
    let item = part(&ui, "Item");
    assert!(
        item.starts_with("button.secondary") && item.ends_with("::part(item)"),
        "the control's variant stays: {item}"
    );
    assert!(has(fill(&ui, "Item"), "#00ffff"));
    assert!(part(&ui, "Picked").contains("[aria-selected=true]"));
    assert!(has(fill(&ui, "Picked"), "#ff8800"));
    assert!(part(&ui, "Epic").contains("[data-rarity=epic]"));
    assert!(has(fill(&ui, "Epic"), "#ff00ff"), "{:?}", fill(&ui, "Epic"));
    assert!(part(&ui, "Rare").contains("[data-rarity=rare]"));
    assert!(has(fill(&ui, "Rare"), "#0000ff"), "{:?}", fill(&ui, "Rare"));
    let e = ui
        .elements()
        .iter()
        .find(|e| e.path.ends_with("/Item"))
        .unwrap()
        .clone();
    for _ in 0..30 {
        step(&mut ui, Some((e.x + e.width / 2.0, e.y + e.height / 2.0)));
    }
    assert!(
        part(&ui, "Item").contains(":hover"),
        "{}",
        part(&ui, "Item")
    );
    assert!(has(fill(&ui, "Item"), "#ffffff"), "{:?}", fill(&ui, "Item"));
}

/// A button a component names for a role (`t.cell`) wears the role and its
/// tokens (`--cell-fill`), as
/// `appearance: none` hands a browser's button to its page's rules: the
/// button's own `button::part(base)` dress does not reach it, while a
/// button named for no role (`t.buy`) keeps it. Its focus ring is its
/// `focus` part alone and draws no plate. Wrong: the cell took the
/// button's plate over its role, or the ring repainted the base's fill
/// over the focused cell.
/// A role part on a team (`data-team=a|b`) wears `--team1` or `--team2`
/// as its ink and rim, whatever its role; off a team it keeps the role's
/// ink. Wrong: the team colour on one role only, or on a part with no role.
#[test]
fn a_role_part_on_a_team_wears_the_teams_colour() {
    let mut ui = ui_with_font(test_font());
    ui.set_part_css(&format!(
        "{DEFAULT_CSS}\n:root {{ --team1: #ff0000; --team2: #0000ff; --ink-label: #00ff00; }}"
    ))
    .unwrap();
    let mut d = mounted(
        ui,
        r#"<meta name="parts" content="t.heading t.figure t.caption">
<column id="Root">
  <h2 part="t.heading" data-team="a">Wolves</h2>
  <span part="t.figure" data-team="b">12</span>
  <h2 part="t.heading">Neutral</h2>
  <span part="t.caption" data-team="a">Plain</span>
</column>"#,
        &[],
        Viewport::new(400.0, 300.0),
    );
    d.step(InputFrame::default());
    let ink = |want: &str| {
        d.ui.frame().items.iter().find_map(|i| {
            let text = match &i.cmd {
                DrawCmd::StyledText { text, .. } => text.as_ref(),
                other => other,
            };
            match text {
                DrawCmd::Text { text, color, .. } if text == want => Some(*color),
                _ => None,
            }
        })
    };
    let colour = |hex: &str| og_ui::style_file::parse_color(hex).unwrap();
    for (text, hex, wears) in [
        ("Wolves", "#ff0000", true),
        ("12", "#0000ff", true),
        ("Neutral", "#00ff00", true),
        ("Plain", "#ff0000", false),
    ] {
        let got = ink(text).unwrap_or_else(|| panic!("{text} drawn"));
        assert_eq!(near(got, colour(hex)), wears, "{text}: {got:?}");
    }
}

#[test]
fn a_part_named_for_a_role_wears_the_role_not_its_controls_dress() {
    let mut d = mounted(
        dressed(
            "button::part(base) { background: #ff0000; padding: 0 20px; }\n\
             :root { --cell-fill: #00ff00; }",
        ),
        r#"<meta name="parts" content="t.buy t.cell">
<column id="Root">
  <button id="Buy" part="t.buy">Buy</button>
  <button id="Cell" part="t.cell">Cell</button>
</column>
<style>#Root { align-items: flex-start; }</style>"#,
        &[],
        Viewport::new(400.0, 300.0),
    );
    let fill = |ui: &Ui, id: &str| {
        let e = at(ui, id);
        fills_in(ui, Rect::new(e.x, e.y, e.width, e.height))
    };
    let has = |fills: &[[f32; 4]], hex: &str| {
        let c = og_ui::style_file::parse_color(hex).unwrap();
        fills.iter().any(|f| near(*f, c))
    };
    d.step(InputFrame::default());
    assert!(
        has(&fill(&d.ui, "Buy"), "#ff0000"),
        "{:?}",
        fill(&d.ui, "Buy")
    );
    let cell = fill(&d.ui, "Cell");
    assert!(has(&cell, "#00ff00") && !has(&cell, "#ff0000"), "{cell:?}");
    assert!(
        at(&d.ui, "Cell").width < at(&d.ui, "Buy").width,
        "the base's padding stays off the cell"
    );
    for _ in 0..2 {
        d.step(InputFrame {
            keys: vec![UiKey::Tab],
            ..Default::default()
        });
        d.step(InputFrame::default());
    }
    assert!(
        d.ui.focus_path(0).ends_with("/Cell"),
        "{}",
        d.ui.focus_path(0)
    );
    for _ in 0..10 {
        d.step(InputFrame::default());
    }
    let cell = fill(&d.ui, "Cell");
    assert!(!has(&cell, "#ff0000"), "the ring drew the base: {cell:?}");
}

/// The outside strokes (focus rings) drawn: whether each has CSS corners
/// and its first corner radius.
fn ring_strokes(ui: &Ui) -> Vec<(bool, f32)> {
    ui.frame()
        .items
        .iter()
        .filter_map(|item| match &item.cmd {
            DrawCmd::Shape { shape, .. } => match (&shape.geometry, &shape.mode) {
                (og_draw::UiGeometry::Rect { radii, corner }, og_draw::UiMode::Stroke(stroke))
                    if stroke.align == og_draw::UiAlign::Outside =>
                {
                    Some((matches!(corner, og_draw::UiCorner::Css(_)), radii[0]))
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// A focus ring is an outline, so it follows its control's corners and
/// their shape (CSS UI 4 §5.1): a pack's bevelled button rings bevelled.
/// `--button-focus-radius` and `--button-focus-corner-shape` give the ring
/// its own. Wrong: a round ring around a bevelled button.
#[test]
fn a_focus_ring_follows_its_buttons_corner_shape() {
    let ring = |pack: &str| {
        let mut ui = ui_with_font(test_font());
        ui.set_part_css(&format!(
            "{DEFAULT_CSS}\n:root {{ --button-radius: 10px; {pack} }}"
        ))
        .unwrap();
        let mut d = mounted(
            ui,
            r#"<column id="Root"><button id="B">Go</button></column>"#,
            &[],
            Viewport::new(400.0, 300.0),
        );
        d.step(InputFrame {
            keys: vec![UiKey::Tab],
            ..Default::default()
        });
        for _ in 0..10 {
            d.step(InputFrame::default());
        }
        assert!(d.ui.focus_path(0).ends_with("/B"), "{}", d.ui.focus_path(0));
        ring_strokes(&d.ui)
    };
    assert_eq!(
        ring(""),
        vec![(false, 10.0 + 2.0)],
        "a round button rings round"
    );
    let bevelled = ring("--button-corner-shape: bevel;");
    assert!(
        matches!(bevelled.as_slice(), [(true, r)] if *r > 10.0),
        "a bevelled button rings bevelled: {bevelled:?}"
    );
    let own = ring("--button-corner-shape: bevel; --button-focus-radius: 3px;");
    assert!(
        matches!(own.as_slice(), [(false, r)] if *r < 10.0),
        "the ring's own radius: {own:?}"
    );
}

/// A style pack restyles the core controls by setting their tokens alone
/// (docs/ui.md §3.1, Control hooks): a fill, an ink, a rim with its width
/// and a padding reach the button, checkbox and field, and a variant keeps
/// its own hook. Wrong: a hook no rule reads, a primary button wearing the
/// secondary's fill, or a rim drawn with no width set.
#[test]
fn a_pack_restyles_core_controls_through_their_tokens_alone() {
    let doc = r#"<column id="Root">
  <button id="B">Go</button>
  <button id="P" variant="primary">Go</button>
  <input id="C" type="checkbox" checked />
  <input id="T" bind:value={$t} />
  <badge id="G">New</badge>
</column>
<style>#Root { align-items: flex-start; gap: 4px; }</style>"#;
    let pack = ":root {
        --button-fill: #00ff00; --button-primary-fill: #ff8800; --button-pad-inline: 30px;
        --checkbox-checked-fill: #ff00ff; --field-fill: #445566; --badge-neutral-ink: #ff0000;
    }";
    let mount = |css: &str| {
        let mut ui = ui_with_font(test_font());
        ui.set_part_css(&format!("{DEFAULT_CSS}\n{css}")).unwrap();
        let mut d = mounted(ui, doc, &[("t", json!("hi"))], Viewport::new(400.0, 300.0));
        d.step(InputFrame::default());
        d
    };
    let (plain, packed) = (mount(""), mount(pack));
    let colour = |hex: &str| og_ui::style_file::parse_color(hex).unwrap();
    let has = |d: &Doc, id: &str, hex: &str| {
        let e = at(&d.ui, id);
        fills_in(&d.ui, Rect::new(e.x, e.y, e.width, e.height))
            .iter()
            .any(|f| near(*f, colour(hex)))
    };
    let ink = |d: &Doc, want: &str| {
        d.ui.frame().items.iter().find_map(|i| {
            let text = match &i.cmd {
                DrawCmd::StyledText { text, .. } => text.as_ref(),
                other => other,
            };
            match text {
                DrawCmd::Text { text, color, .. } if text == want => Some(*color),
                _ => None,
            }
        })
    };
    assert!(has(&packed, "B", "#00ff00"));
    assert!(has(&packed, "P", "#ff8800") && !has(&packed, "P", "#00ff00"));
    assert!(has(&packed, "C/Box", "#ff00ff"));
    assert!(has(&packed, "T", "#445566"));
    assert!(
        ink(&packed, "New").is_some_and(|c| near(c, colour("#ff0000"))),
        "the badge's ink"
    );
    assert!(!has(&plain, "B", "#00ff00"));
    assert!(ink(&plain, "New").is_some_and(|c| !near(c, colour("#ff0000"))));
    assert_eq!(
        at(&packed.ui, "B").width - at(&plain.ui, "B").width,
        2.0 * (30.0 - 16.0),
        "the inline padding on both sides"
    );
}

/// A document button's variant (`variant="primary"`) is one of its classes
/// for the pack's host rules as for its parts' (docs/ui.md §3.2):
/// `button.primary` inks the label as `button.primary::part(base)` fills
/// the plate, and a bound variant stays out of rules for other classes
/// (`.art`). Wrong: the variant's plate under the plain button's ink.
#[test]
fn a_document_buttons_variant_inks_its_label() {
    let mut ui = ui_with_font(test_font());
    ui.set_icons(icons(og_draw::TextureHandle::from_slot(1)));
    let mut d = mounted(
        ui,
        r#"<column id="Root"><button variant="primary">Pri</button><button variant="danger">Dan</button><button variant="ghost">Gho</button><button class="primary">Cls</button><button>Pla</button><button variant={$v}>Bnd</button><footer class="actions"><button id="Confirm" variant="primary" disabled={!$ok}><kbd class="key" data-action="details"></kbd><span>Lck</span></button></footer></column>
<style>.actions button { gap: 4px; } .art { object-fit: cover; }</style>"#,
        &[("ok", json!(true)), ("v", json!("danger"))],
        Viewport::new(400.0, 700.0),
    );
    // The pack comes after the documents, as a game's `og.ui.configure`.
    let pack = og_ui::style_pack::StylePack::parse(
        "styles/test",
        vec![(
            "styles/test/main.css".into(),
            ":root { --button-ink: #ff0000; --button-primary-ink: #00ff00;
               --button-danger-ink: #0000ff; --button-ghost-ink: #ffff00; }"
                .into(),
        )],
    )
    .unwrap();
    d.ui.set_style_pack(&pack).unwrap();
    for _ in 0..60 {
        d.step(InputFrame::default());
    }
    let ink = |want: &str| {
        d.ui.frame().items.iter().find_map(|i| {
            let text = match &i.cmd {
                DrawCmd::StyledText { text, .. } => text.as_ref(),
                other => other,
            };
            match text {
                DrawCmd::Text { text, color, .. } if text == want => Some(*color),
                _ => None,
            }
        })
    };
    let colour = |hex: &str| og_ui::style_file::parse_color(hex).unwrap();
    for (label, hex) in [
        ("Pri", "#00ff00"),
        ("Dan", "#0000ff"),
        ("Gho", "#ffff00"),
        ("Cls", "#00ff00"),
        ("Pla", "#ff0000"),
        ("Bnd", "#0000ff"),
        ("Lck", "#00ff00"),
    ] {
        let got = ink(label).unwrap_or_else(|| panic!("{label} drawn"));
        assert!(near(got, colour(hex)), "{label}: {got:?}, want {hex}");
    }
}

/// A widget inside a document's button keeps its own ink when the library
/// carries computed values (any document declaring `list-style-type`):
/// the frame's `color`, which the button only inherits, stays under the
/// keycap's ink as without carrying. Wrong: the frame's ink on the keycap.
#[test]
fn a_carried_colour_stays_under_a_widgets_own_ink() {
    let keycap = |carry: bool| {
        let style = if carry {
            "#Z { list-style-type: none; }"
        } else {
            ""
        };
        let d = mounted(
            ui_with_font(test_font()),
            &format!(
                r#"<meta name="parts" content="t.frame">
<column id="Root" part="t.frame"><button variant="primary"><kbd>X</kbd><span>Go</span></button><div id="Z"></div></column>
<style>{style}</style>"#
            ),
            &[],
            Viewport::new(400.0, 300.0),
        );
        d.ui.frame().items.iter().find_map(|i| match &i.cmd {
            DrawCmd::Text { text, color, .. } if text == "X" => Some(*color),
            _ => None,
        })
    };
    let alone = keycap(false).expect("the keycap drawn");
    assert_eq!(keycap(true), Some(alone));
}

/// A button's content without a colour of its own takes the button's ink
/// as that ink moves, as slotted content inherits from its slot: a pack set
/// after mount and a scheme switch reach it, under the default style's
/// colour transition. Wrong: the ink the button had when it was built.
#[test]
fn a_buttons_content_follows_its_ink() {
    let mut d = mounted(
        ui_with_font(test_font()),
        r##"<column id="Root"><button variant="primary"><span>Pri</span></button><button><div><span>Pla</span></div></button><button><span style="color: #123456">Own</span></button></column>"##,
        &[],
        Viewport::new(400.0, 300.0),
    );
    let pack = og_ui::style_pack::StylePack::parse(
        "styles/test",
        vec![(
            "styles/test/main.css".into(),
            ":root { --button-ink: #ff0000; --button-primary-ink: #00ff00; }
             :root[data-scheme=light] { --button-ink: #ff00ff; --button-primary-ink: #00ffff; }"
                .into(),
        )],
    )
    .unwrap();
    d.ui.set_style_pack(&pack).unwrap();
    let colour = |hex: &str| og_ui::style_file::parse_color(hex).unwrap();
    for (light, pri, pla) in [(false, "#00ff00", "#ff0000"), (true, "#00ffff", "#ff00ff")] {
        if light {
            d.ui.set_theme(Theme::new(Scheme::Light));
        }
        for _ in 0..30 {
            d.step(InputFrame::default());
        }
        let ink = |want: &str| {
            d.ui.frame()
                .items
                .iter()
                .find_map(|i| match i.cmd.unstyled() {
                    DrawCmd::Text { text, color, .. } if text == want => Some(*color),
                    _ => None,
                })
        };
        for (label, hex) in [("Pri", pri), ("Pla", pla), ("Own", "#123456")] {
            let got = ink(label).unwrap_or_else(|| panic!("{label} drawn"));
            assert!(
                near(got, colour(hex)),
                "light {light}: {label}: {got:?}, want {hex}"
            );
        }
    }
}

/// A quiet or ghost button is text on the surface, so it takes inks of its
/// own: a pack whose plates are dark in a light scheme inks them dark while
/// its plate buttons keep a light label. Wrong: a text button hovered in the
/// plate's hover ink, or a plate button taking the text button's.
#[test]
fn quiet_and_ghost_buttons_take_their_own_inks() {
    let mut ui = ui_with_font(test_font());
    ui.set_part_css(&format!(
        "{DEFAULT_CSS}\n:root {{ --button-hover-ink: #ff0000; --button-quiet-ink: #00ff00;
           --button-ghost-hover-ink: #0000ff; }}"
    ))
    .unwrap();
    let mut d = mounted(
        ui,
        r#"<column id="Root"><button id="G" variant="ghost">Ghost</button><button id="Q" variant="quiet">Quiet</button><button id="P">Plate</button></column>
<style>#Root { align-items: flex-start; } button { width: 80px; height: 40px; }</style>"#,
        &[],
        Viewport::new(300.0, 300.0),
    );
    let ink = |d: &Doc, want: &str| {
        d.ui.frame().items.iter().find_map(|i| {
            let text = match &i.cmd {
                DrawCmd::StyledText { text, .. } => text.as_ref(),
                other => other,
            };
            match text {
                DrawCmd::Text { text, color, .. } if text == want => Some(*color),
                _ => None,
            }
        })
    };
    let colour = |hex: &str| og_ui::style_file::parse_color(hex).unwrap();
    let hover = |d: &mut Doc, id: &str| {
        let e = at(&d.ui, id);
        for _ in 0..60 {
            d.step(InputFrame::hover(e.x + e.width / 2.0, e.y + e.height / 2.0));
        }
    };
    d.step(InputFrame::default());
    assert!(
        near(ink(&d, "Quiet").unwrap(), colour("#00ff00")),
        "quiet at rest"
    );
    hover(&mut d, "G");
    assert!(
        near(ink(&d, "Ghost").unwrap(), colour("#0000ff")),
        "ghost hovered"
    );
    hover(&mut d, "P");
    assert!(
        near(ink(&d, "Plate").unwrap(), colour("#ff0000")),
        "plate hovered"
    );
}

/// An inherited value reaches what reads it through elements that only
/// hand it down: a custom property set on a container by a bound class
/// restyles a grandchild under a plain `<div>` when the class goes. Wrong:
/// the grandchild keeps the value its first build read.
#[test]
fn an_inherited_change_reaches_through_elements_between() {
    let mut d = mounted(
        ui_with_font(test_font()),
        r##"<column id="Root"><div class:v={$on}><div><p class="u">Var</p></div></div></column>
<style>.v { --x: #ff8800; } .u { color: var(--x, #0000ff); }</style>"##,
        &[("on", json!(true))],
        Viewport::new(400.0, 300.0),
    );
    let colour = |hex: &str| og_ui::style_file::parse_color(hex).unwrap();
    for (on, hex) in [(true, "#ff8800"), (false, "#0000ff"), (true, "#ff8800")] {
        d.set("on", json!(on));
        d.step(InputFrame::default());
        let got = text_ink(&d, "Var").expect("Var drawn");
        assert!(near(got, colour(hex)), "on {on}: {got:?}, want {hex}");
    }
}

/// The ink of the drawn text `want`.
fn text_ink(d: &Doc, want: &str) -> Option<[f32; 4]> {
    d.ui.frame().items.iter().find_map(|i| {
        let text = match &i.cmd {
            DrawCmd::StyledText { text, .. } => text.as_ref(),
            other => other,
        };
        match text {
            DrawCmd::Text { text, color, .. } if text == want => Some(*color),
            _ => None,
        }
    })
}

/// A kit's text element that names its part takes the part's `-og-ink` and
/// `-og-step` (docs/ui.md §3.2), qualified by its classes, as a field's
/// `text` part does: a pack inks a killfeed's names by team. Wrong: the
/// style refused `ink` on a kit part, the ink reached no text or every
/// part's text, a qualified dressing lost to the bare one, or text that
/// names no part took it.
#[test]
fn a_kit_text_part_takes_its_ink_and_step() {
    use og_ui::reactive::SourceId;
    let mut ui = ui_with_font(test_font());
    ui.set_part_css(concat!(
        "span::part(name) { color: #c0392b; font-size: 24px; }\n",
        "span.ally::part(name) { color: #2e86de; }\n",
    ))
    .unwrap();
    let doc = r#"<meta name="component" content="kit">
<meta name="parts" content="name">
<div id="Root">
  <span id="Foe" part="name">Foe</span>
  <span id="Ally" part="name" class:ally={true}>Ally</span>
  <span id="Free">Free</span>
</div>"#;
    ui.define_document(UiDocument::parse("ui/kit.svelte", doc).unwrap());
    ui.mount_document(
        SourceId::new("test", "kit"),
        "kit",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    for _ in 0..3 {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
    }
    let text = |want: &str| -> (f32, [f32; 4]) {
        ui.frame()
            .items
            .iter()
            .find_map(|it| match it.cmd.unstyled() {
                DrawCmd::Text {
                    text, size, color, ..
                } if text == want => Some((*size, *color)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no text {want}"))
    };
    let colour = |hex: &str| og_ui::style_file::parse_color(hex).unwrap();
    let (lg, body) = (24.0, ui.theme().type_ramp.body.size);
    let foe = text("Foe");
    assert!(near(foe.1, colour("#c0392b")), "{:?}", foe.1);
    assert_eq!(foe.0, lg, "the part's step");
    let ally = text("Ally");
    assert!(
        near(ally.1, colour("#2e86de")),
        "the qualified ink wins: {:?}",
        ally.1
    );
    assert_eq!(ally.0, lg, "the bare dressing's step still applies");
    let free = text("Free");
    assert!(near(free.1, ui.theme().palette.ink), "{:?}", free.1);
    assert_eq!(free.0, body);
}

#[test]
fn css_parts_share_the_css_cascade_and_markup_hosts() {
    use og_ui::dress::PartRef;
    use og_ui::paint::{ColorRef, Effect, Paint, ShapeKind};
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css(include_str!("fixtures/parts.css")).unwrap();
    let mut rebuilt = ui_with_font(test_font_metrics());
    rebuilt
        .set_part_css("button::part(base) { background: red; }")
        .unwrap();
    rebuilt.inherit_part_css(&ui);
    ui.set_part_css("").unwrap();
    let mut ui = rebuilt;
    let part = PartRef::of("Button", "base", None);
    let color = |hex: &str| Paint::Solid(ColorRef::Literal(style_file::parse_color(hex).unwrap()));
    let draw = ui.resolve_part(&part, [120.0, 40.0]).unwrap();
    assert_eq!(draw.fills, vec![color("#123456")]);
    assert!(matches!(draw.shape, ShapeKind::Rect { radii: Some(r), .. } if r == [7.0; 4]));
    assert!(matches!(
        draw.effects.as_slice(),
        [Effect::DropShadow {
            offset: [1.0, 3.0],
            blur: 5.0,
            spread: 2.0,
            ..
        }]
    ));
    assert_eq!(
        draw.borders.as_ref().unwrap().borders[0].unwrap().width,
        2.0
    );
    let hover = part.flag("primary").flag("hover");
    assert_eq!(
        ui.resolve_part(&hover, [120.0, 40.0]).unwrap().fills,
        vec![color("#ffff00")]
    );
    let epic = hover.attr("data-rarity", "epic").flag("pressed");
    let draw = ui.resolve_part(&epic, [120.0, 40.0]).unwrap();
    assert_eq!(draw.fills, vec![color("#0000ff")]);
    assert_eq!(draw.opacity, 0.4);
    ui.set_theme(Theme::new(Scheme::Light));
    assert_eq!(
        ui.resolve_part(&part, [120.0, 40.0]).unwrap().fills,
        vec![color("#abcdef")]
    );
    for (component, name, on, expected) in [
        ("Slider", "track", false, "#112233"),
        ("Checkbox", "base", true, "#334455"),
        ("Toggle", "base", true, "#556677"),
        ("Dropdown", "base", false, "#778899"),
        ("TextEdit", "frame", false, "#99aabb"),
    ] {
        let p = PartRef::of(component, name, None).flag_if(on, "on");
        assert_eq!(
            ui.resolve_part(&p, [80.0, 30.0]).unwrap().fills,
            vec![color(expected)],
            "{component}"
        );
    }
    let slot = ui
        .resolve_part(&PartRef::of("Slot", "frame", None), [120.0, 40.0])
        .unwrap();
    let layers = slot.boxes.unwrap();
    assert_eq!(layers.backgrounds.len(), 1);
    assert!(layers.border.is_some());
    assert!(matches!(
        slot.effects.as_slice(),
        [Effect::BackdropFilter(_)]
    ));
    let empty = ui
        .resolve_part(&PartRef::of("Button", "unknown", None), [120.0, 40.0])
        .unwrap();
    assert!(empty.fills.is_empty() && empty.effects.is_empty() && empty.borders.is_none());
    for invalid in [
        "Button::base",
        "Button.base",
        "Button::part(base)",
        "button::part(base).primary",
    ] {
        assert!(
            ui.set_part_css(&format!("{invalid} {{ background: red; }}"))
                .is_err(),
            "{invalid}"
        );
    }
    let doc = UiDocument::parse(
        "parts.svelte",
        r#"<meta name="component" content="parts"><button>OK</button><style>button::part(base):hover { background: red; }</style>"#,
    )
    .unwrap();
    assert_eq!(doc.part_rules.len(), 1);
}

#[test]
fn part_transitions_retarget_and_restore_the_rendered_value() {
    let mut d = base_part(
        "button::part(base) { opacity: 0.2; background: #000; transition: opacity 200ms linear, background 200ms linear; } button::part(base):hover { opacity: 1; background: #fff; }",
    );
    let step = |d: &mut Doc, hovered| {
        d.step(over(hovered));
        assert!(
            d.ui.frame().warnings.is_empty(),
            "{:?}",
            d.ui.frame().warnings
        );
        first_fill(&d.ui)
    };
    assert_eq!(step(&mut d, false)[3], 0.2);
    assert_eq!(step(&mut d, true)[3], 0.2);
    for _ in 0..5 {
        step(&mut d, true);
    }
    let half = step(&mut d, true);
    assert!((half[3] - 0.6).abs() < 0.001, "{half:?}");
    assert!(half[0] > 0.0 && half[0] < 1.0, "{half:?}");
    let saved = d.ui.save_state();
    let rest = (0..4).map(|_| step(&mut d, false)).collect::<Vec<_>>();
    d.ui.restore_state(&saved).unwrap();
    for expected in rest {
        assert_eq!(step(&mut d, false), expected);
    }
}

/// A part's rim set by a `border` shorthand holding `var()`, with a hover
/// rule that writes `border-color`: on leaving the hover, the rim's
/// colour transitions between the two rim colours, so it never shows the
/// ink (`currentColor`, the border colour's initial value) and settles on
/// the shorthand's colour.
///
/// Ways it could fail: the transition reading the `border-color` the
/// hover rule no longer sets as `currentColor` (its initial), so the rim
/// runs to the ink while the transition lasts and only the last frame is
/// right.
#[test]
fn a_parts_rim_from_a_shorthand_transitions_between_the_rim_colours() {
    let mut ui = ui_with_font(test_font_metrics());
    // currentColor is white; the rim is blue and the hover's red. A move
    // between blue and red never passes through white.
    ui.set_part_css(
        "button::part(base) { color: #ffffff; transition: border-color 300ms linear; } \
         button::part(base):hover { border-color: #ff0000; } \
         button.ghost::part(base) { --rim: #3366ff; border: 2px solid var(--rim); }",
    )
    .unwrap();
    let mut d = mounted(
        ui,
        r#"<column><button id="B" class="ghost"></button></column>
<style>#B { width: 80px; height: 40px; }</style>"#,
        &[],
        Viewport::new(100.0, 100.0),
    );
    // The rim's stroke colour this frame.
    let rim = |d: &Doc| -> [f32; 4] {
        d.ui.frame()
            .items
            .iter()
            .find_map(|item| match &item.cmd {
                og_ui::DrawCmd::Shape { shape, .. }
                    if matches!(shape.mode, og_draw::UiMode::Stroke(_)) =>
                {
                    match shape.paint {
                        og_draw::UiPaint::Solid(c) => Some(c),
                        _ => None,
                    }
                }
                _ => None,
            })
            .expect("the rim's stroke")
    };
    let blue = Srgba::rgb(0x33, 0x66, 0xff).linear();
    let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01);
    for _ in 0..20 {
        d.step(over(false));
    }
    assert!(near(rim(&d), blue), "rest rim {:?}", rim(&d));
    for _ in 0..30 {
        d.step(over(true));
    }
    // While the transition runs after leaving, the rim is a mix of red and
    // blue: its green stays low and it is never the white ink.
    for _ in 0..40 {
        d.step(over(false));
        assert!(
            d.ui.frame().warnings.is_empty(),
            "{:?}",
            d.ui.frame().warnings
        );
        let c = rim(&d);
        assert!(c[1] < 0.3, "rim green channel {c:?}: the ink leaked in");
    }
    // It settles on the shorthand's blue.
    for _ in 0..300 {
        d.step(over(false));
    }
    assert!(near(rim(&d), blue), "settled rim {:?}", rim(&d));
}

/// One `button` 80 × 40 at the top left of a 100 × 100 frame, its part
/// CSS `css` alone.
fn base_part(css: &str) -> Doc {
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css(css).unwrap();
    mounted(
        ui,
        r#"<column><button id="B"></button></column>
<style>#B { width: 80px; height: 40px; }</style>"#,
        &[],
        Viewport::new(100.0, 100.0),
    )
}

/// The input of a tick with the pointer over the button or away.
fn over(hovered: bool) -> InputFrame {
    if hovered {
        InputFrame::hover(40.0, 20.0)
    } else {
        InputFrame::default()
    }
}

/// The first box filled (the button's base part's).
fn first_fill(ui: &Ui) -> [f32; 4] {
    ui.frame()
        .items
        .iter()
        .find_map(|item| item.cmd.filled().map(|(_, color)| color))
        .expect("the part's painted box")
}

/// A part's style is kept from tick to tick while nothing it is resolved
/// from changes; a transition that ran in between is one such thing. Left
/// the tick after it was entered, the part runs back from where its hover
/// transition got to instead of taking its unhovered style as it was kept.
#[test]
fn a_part_left_right_after_it_is_entered_transitions_back() {
    let mut d = base_part(
        "button::part(base) { opacity: 0.2; background: #000; transition: opacity 200ms linear; } button::part(base):hover { opacity: 1; }",
    );
    let mut step = |hovered| {
        d.step(over(hovered));
        first_fill(&d.ui)[3]
    };
    for _ in 0..3 {
        assert_eq!(step(false), 0.2);
    }
    // The hover transition starts from 0.2 and moves one tick.
    step(true);
    let back = step(false);
    assert!(back > 0.2 && back < 0.6, "{back}");
}

#[test]
fn document_part_css_is_scoped_and_inherits_authored_host_facts() {
    use og_ui::reactive::SourceId;
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css("button::part(base) { background: blue; }")
        .unwrap();
    for (name, css) in [
        (
            "styled",
            "button { --paint: #ff00ff; } button.primary[data-a=a][data-b=b][data-c=c][data-d=d]::part(base) { background: var(--paint); }",
        ),
        ("plain", ""),
    ] {
        let text = format!(
            r#"<meta name="component" content="{name}"><button id="Go" class="primary" data-a="a" data-b="b" data-c="c" data-d="d">Go</button><style>{css}</style>"#
        );
        ui.define_document(UiDocument::parse(&format!("{name}.svelte"), &text).unwrap());
        ui.mount_document(
            SourceId::new("test", name),
            name,
            Default::default(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
    }
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let fills: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .map(|(_, c)| c)
            .collect();
        assert!(
            fills.iter().any(|c| near(*c, Srgba::rgb(255, 0, 255))),
            "{fills:?}"
        );
        assert!(
            fills.iter().any(|c| near(*c, Srgba::rgb(0, 0, 255))),
            "{fills:?}"
        );
    }
}

#[test]
fn part_keyframes_keep_delay_direction_and_replay() {
    let mut d = base_part(
        "button::part(base) { background: red; animation: pulse-ring 200ms 100ms linear 2 alternate both; } @keyframes pulse-ring { from { opacity: 0.2; } to { opacity: 0.8; } }",
    );
    let step = |d: &mut Doc| {
        d.step(InputFrame::default());
        assert!(
            d.ui.frame().warnings.is_empty(),
            "{:?}",
            d.ui.frame().warnings
        );
        first_fill(&d.ui)[3]
    };
    // The mount's build is the first tick.
    let mut samples = vec![first_fill(&d.ui)[3]];
    samples.extend((1..19).map(|_| step(&mut d)));
    for (tick, expected) in [(0, 0.2), (5, 0.2), (12, 0.5), (18, 0.8)] {
        assert!((samples[tick] - expected).abs() < 0.001, "{samples:?}");
    }
    let saved = d.ui.save_state();
    let back: Vec<_> = (0..13).map(|_| step(&mut d)).collect();
    assert!(
        (back[5] - 0.5).abs() < 0.001 && (back[11] - 0.2).abs() < 0.001,
        "{back:?}"
    );
    d.ui.restore_state(&saved).unwrap();
    for expected in back {
        assert_eq!(step(&mut d), expected);
    }
}

#[test]
fn default_css_glass_uses_css_filters_and_material_fallbacks() {
    use og_ui::dress::PartRef;
    use og_ui::paint::Effect;
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css(DEFAULT_CSS).unwrap();
    for value in ["glass thick lift", "var(--author-material)"] {
        let css = format!("\npanel::part(frame) {{ -og-materail: {value}; }}");
        let error = ui.set_part_css(&css).unwrap_err();
        assert!(error.contains("<part-css>:2:"), "{error}");
        assert!(error.contains("-og-materail"), "{error}");
        let document =
            format!("<meta name=\"component\" content=\"typo\"><panel/>\n<style>{css}</style>");
        let error = UiDocument::parse("ui/typo.svelte", &document).unwrap_err();
        let error = error.join("\n");
        assert!(error.contains("ui/typo.svelte:3:"), "{error}");
        assert!(error.contains("-og-materail"), "{error}");
    }
    let part = PartRef::parse("panel.glass::part(frame)", None).unwrap();
    let default = ui.resolve_part(&part, [120.0, 80.0]).unwrap();
    assert!(default.fills.is_empty());
    assert!(matches!(
        default.effects.as_slice(),
        [Effect::Backdrop { .. }]
    ));
    ui.set_part_css("panel::part(frame) { -og-material: glass thick lift; backdrop-filter: blur(7px) saturate(0); }")
        .unwrap();
    let draw = ui.resolve_part(&part, [120.0, 80.0]).unwrap();
    let [
        Effect::Backdrop {
            blur,
            saturation,
            brightness,
            glass,
            ..
        },
    ] = draw.effects.as_slice()
    else {
        panic!("one material applies the filter exactly once");
    };
    let (mode, _) = glass.resolve(*blur, *saturation, *brightness, [0.0; 4], false, ui.theme());
    let og_draw::UiMode::Backdrop {
        blur: actual_blur,
        saturation: actual_saturation,
        glass: material,
        ..
    } = mode
    else {
        panic!("a glass material");
    };
    assert_eq!(actual_blur, 14.0, "CSS blur, not the thick preset's blur");
    assert_eq!(actual_saturation, 1.0, "the matrix applies saturation once");
    let matrix = material.matrix.unwrap();
    for row in matrix.rows {
        assert_eq!(row, [0.213, 0.715, 0.072, 0.0]);
    }
    let mut reduced = *ui.theme();
    reduced.reduced_transparency = true;
    let (mode, tint) = glass.resolve(*blur, *saturation, *brightness, [0.0; 4], false, &reduced);
    assert!(matches!(mode, og_draw::UiMode::Backdrop { blur: 0.0, glass, .. } if glass.solid));
    assert_eq!(tint[3], 1.0);
}

#[test]
fn reflected_part_constructors_keep_typed_hosts_and_state_semantics() {
    use og_ui::doc::schema;
    use og_ui::paint::{ColorRef, Paint};
    let ui = dressed(
        "input[type=checkbox]::part(track) { background: #112233; }
         input[type=checkbox][switch]::part(track):checked { background: #345678; }
         tabs::part(tab):checked { background: #456789; }
         select[aria-selected=true]::part(option) { background: #56789a; }",
    );
    for (host, name, flag, expected) in [
        ("input[type=checkbox][switch]", "track", "on", "#345678"),
        ("tabs", "tab", "selected", "#456789"),
        ("select", "option", "selected", "#56789a"),
    ] {
        let spec = schema::part(host, name).unwrap();
        let part = spec.reference(None).flag(flag);
        let draw = ui.resolve_part(&part, [100.0, 40.0]).unwrap();
        assert_eq!(
            draw.fills,
            vec![Paint::Solid(ColorRef::Literal(
                og_ui::style_file::parse_color(expected).unwrap()
            ))],
            "{}",
            spec.selector
        );
    }
    assert!(schema::part("input[type=checkbox]", "track").is_none());
    for (state, expected) in [
        ("", StylePart::Button),
        ("hover", StylePart::ButtonHover),
        ("active", StylePart::ButtonPressed),
        ("pressed", StylePart::ButtonPressed),
        ("disabled", StylePart::ButtonDisabled),
    ] {
        let part = schema::parts::BUTTON_BASE.reference(None).flag(state);
        assert_eq!(StylePart::from_part(&part), Some(expected));
        let legacy = dress::PartRef {
            component: "Button",
            ..part
        };
        assert_eq!(StylePart::from_part(&legacy), Some(expected));
    }
    let json = schema::to_json();
    assert!(json["parts"].as_array().unwrap().iter().any(|p| {
        p["host"] == "tabs"
            && p["name"] == "tab"
            && p["states"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s == "checked")
    }));
}

#[test]
fn progress_part_keyframes_advance_without_host_changes() {
    let ui = dressed(
        "progress::part(track) { background: #000; }
         progress::part(fill) { background: #fff; animation: signal 1200ms linear infinite alternate; }
         @keyframes signal { from { opacity: 0.65; } to { opacity: 1; } }",
    );
    let mut d = mounted(
        ui,
        r#"<column><progress id="Meter" value="0.5"></progress></column>"#,
        &[],
        Viewport::new(300.0, 100.0),
    );
    let fill = |ui: &Ui| {
        ui.frame()
            .items
            .iter()
            .filter_map(|item| item.cmd.filled())
            .find(|(_, c)| c[0] > 0.9)
            .expect("white progress fill")
            .1[3]
    };
    let step = |d: &mut Doc| {
        d.step(InputFrame::default());
        fill(&d.ui)
    };
    // The mount's build is the first tick.
    let first = fill(&d.ui);
    for _ in 0..17 {
        step(&mut d);
    }
    let quarter = step(&mut d);
    assert!((first - 0.65).abs() < 0.001, "{first}");
    assert!((quarter - 0.7375).abs() < 0.001, "{quarter}");
    let saved = d.ui.save_state();
    let later: Vec<_> = (0..65).map(|_| step(&mut d)).collect();
    assert!(later[53] > later[64], "alternate reverses after 72 ticks");
    d.ui.restore_state(&saved).unwrap();
    for expected in later {
        assert_eq!(step(&mut d), expected);
    }
    d.ui.set_part_css(
        "progress::part(fill) { background: #fff; animation: finish 100ms linear forwards; }
         @keyframes finish { from { opacity: 0.65; } to { opacity: 1; } }",
    )
    .unwrap();
    for _ in 0..10 {
        step(&mut d);
    }
    assert_eq!(step(&mut d), 1.0);
    assert_eq!(
        d.ui.frame().stats.painted,
        0,
        "finished animation reuses paint"
    );
    d.ui.set_part_css(
        "progress::part(fill) { background: #fff; animation: again 1200ms linear infinite; }
         @keyframes again { from { opacity: 0.65; } to { opacity: 1; } }",
    )
    .unwrap();
    for _ in 0..3 {
        step(&mut d);
    }
    d.ui.set_part_css("progress::part(fill) { background: #fff; opacity: 0.8; }")
        .unwrap();
    for _ in 0..3 {
        step(&mut d);
    }
    assert!((step(&mut d) - 0.8).abs() < 0.001);
    assert_eq!(
        d.ui.frame().stats.painted,
        0,
        "removed animation reuses paint"
    );
}

/// Text fields, text areas and the number spinner keep their frame and
/// buttons as persistent part boxes: the part CSS lays out and paints them,
/// their states select rules, and the hosts keep focus, typing, presses and
/// semantics. Wrong: the frame was painted on the host and its padding,
/// border and state rules did not move the text or reach the buttons.
#[test]
fn text_field_and_spinner_parts_keep_boxes_states_and_host_input() {
    let mut ui = ui_with_font(test_font());
    ui.set_part_css(
        "input::part(frame) { padding: 0 11px; border: 2px solid #123456; background: #00ff00; }
         input::part(frame):focus-visible { background: #0000ff; }
         input::part(frame):hover { background: #ffff00; }
         input::part(frame):invalid { background: #ff0000; }
         input[type=number]::part(button) { background: #ff00ff; }
         input::part(reveal) { background: #00ffff; padding: 3px; }
         textarea::part(frame) { padding: 7px; background: #808000; }",
    )
    .unwrap();
    let mut d = mounted(
        ui,
        r#"<column id="Root">
  <input id="Name" bind:value={$name} />
  <input id="Count" type="number" bind:value={$count} />
  <input id="Secret" type="password" bind:value={$secret} />
  <input id="Bad" aria-invalid="true" bind:value={$bad} />
  <textarea id="Notes" bind:value={$area}></textarea>
</column>"#,
        &[
            ("name", json!("Hi")),
            ("count", json!("42")),
            ("secret", json!("pw")),
            ("bad", json!("x")),
            ("area", json!("Area")),
        ],
        Viewport::new(600.0, 600.0),
    );
    for _ in 0..2 {
        d.step(InputFrame::default());
    }
    let press = |d: &mut Doc, path: &str, dx: f32| {
        let e = at(&d.ui, path);
        press_at(
            d,
            og_core::math::Vec2::new(e.x + e.width * dx, e.y + e.height / 2.0),
        );
    };
    let texts = |ui: &Ui| -> Vec<(String, f32)> {
        ui.frame()
            .items
            .iter()
            .filter_map(|item| match &item.cmd {
                DrawCmd::StyledText { text, .. } => match text.as_ref() {
                    DrawCmd::Text { text, x, .. } => Some((text.clone(), *x)),
                    _ => None,
                },
                DrawCmd::Text { text, x, .. } => Some((text.clone(), *x)),
                _ => None,
            })
            .collect()
    };
    let fill = |ui: &Ui, rgb: [u8; 3]| {
        let c = Srgba::rgb(rgb[0], rgb[1], rgb[2]).linear();
        ui.frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .filter(|(_, color)| *color == c)
            .map(|(r, _)| r)
            .collect::<Vec<_>>()
    };
    let ui = &d.ui;
    for (host, part, role) in [
        ("Name", "frame", "textbox"),
        ("Notes", "frame", "textbox"),
        ("Count/Increment", "button", "button"),
        ("Secret/Reveal", "reveal", "button"),
    ] {
        let h = at(ui, host);
        assert_eq!(h.role, role, "{host}");
        let p = at(ui, &format!("{host}/::part({part})#0"));
        assert_eq!(p.kind, "part", "{host}");
        assert!(p.role.is_empty(), "{host}");
        assert_eq!((p.x, p.y, p.width, p.height), (h.x, h.y, h.width, h.height));
    }
    // The frame's padding and border inset the text; its background is
    // its own box's, by state.
    let name = at(ui, "Name");
    let hi = texts(ui).into_iter().find(|t| t.0 == "Hi").unwrap();
    assert_eq!(hi.1 - name.x, 13.0);
    assert!(fill(ui, [0, 255, 0]).iter().any(|r| r.y == name.y));
    let bad = at(ui, "Bad");
    assert!(fill(ui, [255, 0, 0]).iter().any(|r| r.y == bad.y));
    let notes = at(ui, "Notes");
    let area = texts(ui).into_iter().find(|t| t.0 == "Area").unwrap();
    assert_eq!(area.1 - notes.x, 7.0);
    assert!(fill(ui, [128, 128, 0]).iter().any(|r| r.y == notes.y));
    let inc = at(ui, "Count/Increment");
    assert!(fill(ui, [255, 0, 255]).iter().any(|r| r.x == inc.x));
    // A pointer over a field is :hover on its frame, as on a browser's.
    assert!(fill(ui, [255, 255, 0]).is_empty());
    for _ in 0..2 {
        d.step(InputFrame {
            pointer: og_core::math::Vec2::new(
                name.x + name.width / 2.0,
                name.y + name.height / 2.0,
            ),
            pointer_inside: true,
            ..Default::default()
        });
    }
    assert!(fill(&d.ui, [255, 255, 0]).iter().any(|r| r.y == name.y));
    d.step(InputFrame::default());
    let ui = &d.ui;
    assert!(fill(ui, [0, 255, 255]).len() == 1);
    // Presses and typing reach the hosts.
    press(&mut d, "Count/Increment", 0.5);
    d.step(InputFrame::default());
    let count = d.read("count");
    assert!(count == json!("43") || count == json!(43), "{count}");
    press(&mut d, "Secret/Reveal", 0.5);
    assert_eq!(at(&d.ui, "Secret/Reveal").text, "Hide password");
    press(&mut d, "Name", 0.9);
    assert!(d.ui.focus_path(0).ends_with("/Name"));
    d.step(InputFrame {
        text: "!".into(),
        ..Default::default()
    });
    d.step(InputFrame::default());
    assert_eq!(d.read("name"), json!("Hi!"));
    for _ in 0..20 {
        d.step(InputFrame::default());
    }
    assert!(fill(&d.ui, [0, 0, 255]).iter().any(|r| r.y == name.y));
    assert!(!fill(&d.ui, [0, 255, 0]).iter().any(|r| r.y == name.y));
}

/// An undressed text control draws as before its frame became a box: one
/// ground at the host's box, its text at the host's inset, no fill of the
/// host's own. Wrong: the host and its frame both painted, or the text
/// moved by the frame's border.
#[test]
fn undressed_text_frames_keep_the_hosts_box_and_inset() {
    let d = mounted(
        ui_with_font(test_font()),
        r#"<column id="Root">
  <input id="Name" bind:value={$name} />
  <input id="Bad" aria-invalid="true" bind:value={$bad} />
</column>"#,
        &[("name", json!("Hello")), ("bad", json!("Bad"))],
        Viewport::new(600.0, 400.0),
    );
    let mut d = d;
    for _ in 0..2 {
        d.step(InputFrame::default());
    }
    let ui = &d.ui;
    let pad = ui.theme().space.md;
    let control = ui.theme().palette.control.linear();
    for (path, text) in [("Name", "Hello"), ("Bad", "Bad")] {
        let host = at(ui, path);
        let frame = at(ui, &format!("{path}/::part(frame)#0"));
        assert_eq!(
            (frame.x, frame.y, frame.width, frame.height),
            (host.x, host.y, host.width, host.height)
        );
        let grounds: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .filter(|(r, c)| *c == control && r.y == host.y)
            .collect();
        assert_eq!(grounds.len(), 1, "{path}: {grounds:?}");
        assert_eq!(grounds[0].0.width, host.width);
        let x = ui
            .frame()
            .items
            .iter()
            .find_map(|item| match &item.cmd {
                DrawCmd::StyledText { text: t, .. } => match t.as_ref() {
                    DrawCmd::Text { text: s, x, .. } if s == text => Some(*x),
                    _ => None,
                },
                DrawCmd::Text { text: s, x, .. } if s == text => Some(*x),
                _ => None,
            })
            .unwrap();
        assert_eq!(x - host.x, pad, "{path}");
    }
}

/// A pack's `background: <gradient>` on a panel or dialog part replaces the
/// widget's own ground, as the shorthand resets the colour to transparent:
/// a glass pack's scene shows through it.
#[test]
fn a_gradient_part_background_drops_the_widgets_ground() {
    for doc in [
        "<panel><p>Hi</p></panel>",
        "<dialog open><p>Hi</p></dialog>",
    ] {
        let ui = dressed(
            "panel::part(frame), dialog::part(panel) { background: linear-gradient(rgb(255 0 0 / 0.25), rgb(0 0 255 / 0.25)); box-shadow: none; }",
        );
        let text = format!(r#"<meta name="component" content="t">{doc}"#);
        let mut d = Doc::with_ui(ui, &text);
        d.step(InputFrame::default());
        d.step(InputFrame::default());
        let (mut solid, mut gradient) = (0, 0);
        for item in &d.ui.frame().items {
            if let DrawCmd::Shape { shape, .. } = &item.cmd {
                let fill = matches!(shape.mode, og_draw::UiMode::Fill);
                match shape.paint {
                    og_draw::UiPaint::Solid(c) if c[3] > 0.0 && fill => solid += 1,
                    og_draw::UiPaint::GradientImage(_) => gradient += 1,
                    _ => {}
                }
            }
        }
        assert_eq!((solid, gradient), (0, 1), "{doc}");
    }
}

/// A keycap reads on what its element's CSS background draws under it,
/// as on a widget's own fill: a `<kbd>` on a light gradient face (a hero
/// button's) in the dark theme takes an ink that holds 4.5:1 on its
/// translucent cap over that face. Fails when the badge reads the panel
/// behind the face instead (the dark theme's pale muted ink on a pale
/// face, 1.16:1).
#[test]
fn a_keycap_reads_on_a_css_gradient_ground() {
    let mut ui = ui_with_font(test_font());
    ui.set_theme(Theme::dark());
    let face = Srgba::rgb(0xe0, 0xe0, 0xe5);
    let mut d = Doc::with_ui(
        ui,
        r#"<meta name="component" content="t">
<div id="Face"><kbd>K</kbd></div>
<style>#Face { padding: 16px; background: linear-gradient(#e0e0e5, #e0e0e5); }</style>"#,
    );
    d.step(InputFrame::default());
    d.step(InputFrame::default());
    let items = &d.ui.frame().items;
    let (k, ink, centre) = items
        .iter()
        .enumerate()
        .find_map(|(k, it)| match it.cmd.unstyled() {
            DrawCmd::Text {
                text,
                color,
                bounds,
                ..
            } if text == "K" => Some((
                k,
                *color,
                (
                    bounds.x + bounds.width / 2.0,
                    bounds.y + bounds.height / 2.0,
                ),
            )),
            _ => None,
        })
        .expect("the keycap's text");
    let over = |top: [f32; 4], below: [f32; 3]| -> [f32; 3] {
        [0, 1, 2].map(|i| top[i] * top[3] + below[i] * (1.0 - top[3]))
    };
    let l = face.linear();
    let mut ground = [l[0], l[1], l[2]];
    for it in &items[..k] {
        if let Some((rect, c)) = it.cmd.filled()
            && rect.x <= centre.0
            && centre.0 <= rect.x + rect.width
            && rect.y <= centre.1
            && centre.1 <= rect.y + rect.height
        {
            ground = over(c, ground);
        }
    }
    let srgb = |c: [f32; 3]| Srgba::from_linear([c[0], c[1], c[2], 1.0]);
    let ratio = srgb(over(ink, ground)).contrast(srgb(ground));
    assert!(ratio >= 4.5, "the cap reads {ratio:.2}:1");
}
