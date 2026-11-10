//! CSS for exposed widget parts: authored hosts, scoped rules, text,
//! fills, borders and glass. Transitions and keyframes must advance by
//! ticks, retarget from the current value and replay after restoration.
//! Unmatched parts have no class-dependent appearance fallback.

use crate::support::*;
use og_ui::*;

#[test]
fn tabs_and_select_parts_keep_boxes_states_and_host_input() {
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
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
    let mut selected = 1;
    let tick = |ui: &mut Ui, input, selected: &mut i32| {
        ui.begin(input, Viewport::new(600.0, 400.0));
        ui.open("Root", Element::column());
        *selected = ui
            .segmented(
                "Tabs",
                &SegmentedDesc {
                    selected: *selected,
                    items: vec![
                        SegmentItem {
                            label: "First".into(),
                            ..Default::default()
                        },
                        SegmentItem {
                            label: "Second".into(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            )
            .selected;
        ui.dropdown(
            "Select",
            &DropdownDesc {
                selected: 0,
                options: vec![DropdownOption {
                    label: "Option".into(),
                    group: "Group".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        );
        ui.close();
        ui.end();
    };
    for _ in 0..3 {
        tick(&mut ui, InputFrame::default(), &mut selected);
    }
    let indicator = ui.element("Root/Tabs/Strip/Track/Thumb").unwrap();
    assert_eq!(indicator.kind, "part");
    assert_eq!(indicator.height, 7.0);
    for (host, part, role) in [
        ("Root/Tabs/Strip/Track/Item#0", "tab", "radio"),
        ("Root/Select/Trigger", "trigger", "combobox"),
    ] {
        assert_eq!(ui.element(host).unwrap().role, role);
        let visual = ui.element(&format!("{host}/::part({part})#0")).unwrap();
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
    let first = ui.element("Root/Tabs/Strip/Track/Item#0").unwrap();
    let pointer =
        og_core::math::Vec2::new(first.x + first.width / 2.0, first.y + first.height / 2.0);
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
            &mut selected,
        );
    }
    assert_eq!(selected, 0);
    tick(&mut ui, InputFrame::default(), &mut selected);
    let trigger = ui.element("Root/Select/Trigger").unwrap();
    let pointer = og_core::math::Vec2::new(
        trigger.x + trigger.width / 2.0,
        trigger.y + trigger.height / 2.0,
    );
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
            &mut selected,
        );
    }
    for _ in 0..15 {
        tick(&mut ui, InputFrame::default(), &mut selected);
    }
    let option = ui.elements().iter().find(|e| e.role == "option").unwrap();
    let visual = ui
        .element(&format!("{}/::part(option)#0", option.path))
        .unwrap();
    assert!(visual.role.is_empty());
    assert_eq!(visual.kind, "part");
    let list = ui.element("Select.list/::part(list)#0").unwrap();
    let group = ui.element("Select.list/List/Group#0").unwrap();
    assert_eq!(list.kind, "part");
    assert!(list.role.is_empty());
    assert_eq!(group.kind, "part");
    assert_eq!(group.height, 37.0);
    assert_eq!(group.x - list.x, 16.0);
    let text = ui.element("Select.list/List/Group#0/Text").unwrap();
    assert_eq!(text.x - group.x, 19.0);
    assert!(ui.frame().items.iter().any(|i| {
        i.cmd
            .filled()
            .is_some_and(|(_, c)| c == [1.0, 0.0, 1.0, 1.0])
    }));
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

/// Two sliders and a switch in a 600×400 frame; answers the stepped
/// slider's value.
fn range_tick(ui: &mut Ui, input: InputFrame, value: &mut f32) {
    ui.begin(input, Viewport::new(600.0, 400.0));
    ui.open(
        "Root",
        Element {
            padding: Insets::all(20.0),
            gap: 30.0,
            ..Element::column()
        },
    );
    ui.value_slider(
        "Plain",
        0.3,
        &ValueSliderDesc {
            width: Some(Size::px(200.0)),
            ..Default::default()
        },
    );
    *value = ui
        .value_slider(
            "Ticks",
            *value,
            &ValueSliderDesc {
                max: 10.0,
                step: 1.0,
                ticks: true,
                label: "Volume".into(),
                show_value: true,
                bubble: true,
                marks: vec![
                    SliderMark {
                        value: 0.0,
                        label: "Lo".into(),
                    },
                    SliderMark {
                        value: 10.0,
                        label: "Hi".into(),
                    },
                ],
                ..Default::default()
            },
        )
        .value;
    ui.toggle_switch("Switch", true, &ToggleSwitchDesc::default());
    ui.close();
    ui.end();
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
    let e = ui.element(path).unwrap_or_else(|| panic!("{path}"));
    [e.x, e.y, e.width, e.height]
}

#[test]
fn undressed_range_parts_keep_their_boxes_and_draws() {
    // The boxes and fills the slider drew before its parts were boxes.
    let mut ui = range_ui(None);
    let mut value = 4.0;
    for _ in 0..3 {
        range_tick(&mut ui, InputFrame::default(), &mut value);
    }
    for (path, kind, expected) in [
        ("Root/Plain/Track", "box", [20.0, 44.0, 200.0, 44.0]),
        (
            "Root/Plain/Track/::part(reach)#0",
            "part",
            [20.0, 44.0, 200.0, 44.0],
        ),
        ("Root/Plain/Track/Rail", "part", [33.0, 63.0, 174.0, 6.0]),
        (
            "Root/Plain/Track/Rail/Fill",
            "part",
            [33.0, 63.0, 52.0, 6.0],
        ),
        ("Root/Plain/Track/Knob", "box", [72.0, 53.0, 26.0, 26.0]),
        (
            "Root/Plain/Track/Knob/::part(knob)#0",
            "part",
            [72.0, 53.0, 26.0, 26.0],
        ),
        (
            "Root/Ticks/Track/Rail/Tick#0",
            "part",
            [32.0, 158.0, 2.0, 12.0],
        ),
        (
            "Root/Ticks/Track/Rail/Tick#10",
            "part",
            [206.0, 158.0, 2.0, 12.0],
        ),
        ("Root/Ticks/Track/Knob", "box", [90.0, 151.0, 26.0, 26.0]),
    ] {
        assert_eq!(rect(&ui, path), expected, "{path}");
        let e = ui.element(path).unwrap();
        assert_eq!(e.kind == "part", kind == "part", "{path}: {}", e.kind);
        if kind == "part" {
            assert!(e.role.is_empty(), "{path}: {}", e.role);
        }
    }
    assert_eq!(ui.element("Root/Plain/Track").unwrap().role, "slider");
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
    let track = ui.element("Root/Ticks/Track").unwrap().clone();
    let at = og_core::math::Vec2::new(track.x + track.width * 0.62, track.y + track.height / 2.0);
    let press = |down, pressed| InputFrame {
        pointer: at,
        pointer_inside: true,
        down,
        pressed,
        ..Default::default()
    };
    range_tick(&mut ui, press(true, true), &mut value);
    for _ in 0..4 {
        range_tick(&mut ui, press(true, false), &mut value);
    }
    assert_eq!(value, 6.0, "pointer input reaches the slider host");
    assert_eq!(
        rect(&ui, "Root/Ticks/Track/Knob"),
        [124.0, 151.0, 26.0, 26.0]
    );
    assert_eq!(
        rect(&ui, "Root/Ticks/Track/Knob/Core"),
        [132.0, 159.0, 11.0, 11.0]
    );
    let bubble = rect(&ui, "Root/Ticks/Track/Bubble");
    assert_eq!([bubble[0], bubble[2], bubble[3]], [121.0, 24.0, 26.0]);
    assert!(bubble[1] > 113.0, "entering, the bubble rises: {bubble:?}");
    for _ in 0..30 {
        range_tick(&mut ui, press(true, false), &mut value);
    }
    let settled = rect(&ui, "Root/Ticks/Track/Bubble");
    assert!((settled[1] - 112.0).abs() < 0.01, "{settled:?}");
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

#[test]
fn range_part_css_sizes_and_states_its_boxes() {
    let mut ui = range_ui(Some(
        "input[type=range]::part(track) { height: 10px; }
         input[type=range]::part(reach) { background: #0000ff; }
         input[type=range]::part(bubble) { padding: 0 11px; border: 2px solid #123456; }
         input[type=range]::part(knob) { width: 30px; }
         input[type=range]::part(knob):active { background: #00ff00; }
         input[type=range]::part(core) { background: #ff00ff; }",
    ));
    let mut value = 4.0;
    for _ in 0..3 {
        range_tick(&mut ui, InputFrame::default(), &mut value);
    }
    let reach = ui.element("Root/Plain/Track/::part(reach)#0").unwrap();
    let reach = [reach.x, reach.y, reach.width, reach.height];
    assert_eq!(ui.element("Root/Plain/Track/Rail").unwrap().height, 10.0);
    let blue = Srgba::rgb(0, 0, 255).linear();
    assert!(
        ui.frame().items.iter().any(|i| i
            .cmd
            .filled()
            .is_some_and(|(r, c)| c == blue && [r.x, r.y, r.width, r.height] == reach)),
        "the reach paints its own box"
    );
    assert_eq!(
        ui.element("Root/Plain/Track/Knob/::part(knob)#0")
            .unwrap()
            .width,
        30.0
    );
    let green = Srgba::rgb(0, 255, 0).linear();
    let magenta = Srgba::rgb(255, 0, 255).linear();
    let has = |ui: &Ui, c: [f32; 4]| {
        ui.frame()
            .items
            .iter()
            .any(|i| i.cmd.filled().is_some_and(|(_, f)| f == c))
    };
    assert!(!has(&ui, green) && !has(&ui, magenta));
    let track = ui.element("Root/Ticks/Track").unwrap().clone();
    let at = og_core::math::Vec2::new(track.x + track.width * 0.62, track.y + track.height / 2.0);
    // Held past the knob's state motion.
    for k in 0..30 {
        let pressed = k == 0;
        range_tick(
            &mut ui,
            InputFrame {
                pointer: at,
                pointer_inside: true,
                down: true,
                pressed,
                ..Default::default()
            },
            &mut value,
        );
    }
    assert_eq!(value, 6.0);
    assert!(has(&ui, green), "the held knob is :active");
    assert!(has(&ui, magenta), "the core takes its part's CSS");
    let bubble = ui.element("Root/Ticks/Track/Bubble").unwrap();
    let text = ui.element("Root/Ticks/Track/Bubble/Text").unwrap();
    assert_eq!(bubble.width, text.width + 2.0 * (11.0 + 2.0));
    assert_eq!(text.x - bubble.x, 13.0);
}

#[test]
fn widget_part_placement_follows_the_writing_direction() {
    // A part's own insets are in its layout's terms: in a right-to-left
    // frame the fill and the switch's knob stay on their track.
    let mut ui = range_ui(None);
    ui.set_direction(og_i18n::Direction::Rtl);
    let mut value = 4.0;
    for _ in 0..3 {
        range_tick(&mut ui, InputFrame::default(), &mut value);
    }
    assert_eq!(
        rect(&ui, "Root/Plain/Track/Rail"),
        [393.0, 63.0, 174.0, 6.0]
    );
    assert_eq!(
        rect(&ui, "Root/Plain/Track/Rail/Fill"),
        [515.0, 63.0, 52.0, 6.0]
    );
    assert_eq!(
        rect(&ui, "Root/Ticks/Track/Rail/Tick#0"),
        [566.0, 158.0, 2.0, 12.0]
    );
    let track = rect(&ui, "Root/Switch/Track");
    let knob = rect(&ui, "Root/Switch/Track/Knob");
    assert!(
        knob[0] >= track[0] && knob[0] + knob[2] <= track[0] + track[2],
        "{knob:?} in {track:?}"
    );
    assert!(
        knob[0] - track[0] < 5.0,
        "on, it sits at the left end: {knob:?}"
    );
}

/// A progress bar, a meter and a gauge at 1.5× in a 257 px column.
fn bars(ui: &mut Ui) {
    ui.begin(
        InputFrame::default(),
        Viewport {
            dpr: 1.5,
            ..Viewport::new(800.0, 500.0)
        },
    );
    ui.open(
        "Root",
        Element {
            padding: Insets::all(13.0),
            gap: 9.0,
            width: Size::px(257.0),
            ..Element::column()
        },
    );
    ui.progress(
        "Load",
        0.6,
        ProgressDesc {
            width: Some(Size::px(141.0)),
        },
    );
    ui.meter(
        "Hp",
        23.0,
        100.0,
        &MeterDesc {
            kind: MeterKind::Health,
            label: "HP".into(),
            segments: 4,
            ..Default::default()
        },
    );
    ui.gauge(
        "Gauge",
        &GaugeDesc {
            value: 0.2,
            low: Some(0.3),
            high: Some(0.7),
            optimum: Some(0.9),
            ..Default::default()
        },
    );
    ui.close();
    ui.end();
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
    let mut ui = ui_with_font(test_font_metrics());
    for _ in 0..3 {
        bars(&mut ui);
    }
    for (host, role) in [
        ("Root/Load", "progressbar"),
        ("Root/Hp", "meter"),
        ("Root/Gauge", "meter"),
    ] {
        let h = ui.element(host).unwrap();
        assert_eq!(h.role, role, "{host}");
        let track = ui.element(&format!("{host}/::part(track)#0")).unwrap();
        assert_eq!(track.kind, "part");
        assert!(track.role.is_empty());
        assert_eq!(
            [track.x, track.y, track.width, track.height],
            [h.x, h.y, h.width, h.height]
        );
    }
    // The fills are fractions of the track's device-pixel box, as before
    // the tracks were boxes: 60% of 140.668 px ends on the 98 px edge.
    let fills = fill_rects(&ui);
    let load = ui.element("Root/Load").unwrap();
    let expected = [load.x, load.y, 98.0 - load.x, 8.0];
    assert!(fills.contains(&expected), "{expected:?} in {fills:?}");
    let hp = ui.element("Root/Hp").unwrap();
    assert!(
        fills
            .iter()
            .any(|r| r[1] == hp.y && r[2] == 1.0 * 2.0 && r[3] == hp.height),
        "a separator over the meter's track: {fills:?}"
    );
    assert_eq!(ui.element("Root/Hp").unwrap().text, "HP");
}

#[test]
fn bar_track_css_sizes_the_box_its_fill_is_drawn_in() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css(
        "progress::part(track) { height: 4px; background: #0000ff; }
         meter::part(track) { height: 6px; }",
    )
    .unwrap();
    for _ in 0..3 {
        bars(&mut ui);
    }
    let track = ui.element("Root/Load/::part(track)#0").unwrap();
    assert_eq!(track.height, 4.0);
    let fills = fill_rects(&ui);
    let load = ui.element("Root/Load").unwrap();
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
    let hp = ui.element("Root/Hp/::part(track)#0").unwrap();
    assert_eq!(hp.height, 6.0);
    assert!(
        fills
            .iter()
            .any(|r| r[1] == hp.y && r[3] == 6.0 && r[2] < hp.width),
        "the meter's fill is the track's height: {fills:?}"
    );
}

#[test]
fn choice_parts_have_independent_css_layout_and_paint() {
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    ui.set_part_css(r#"
input { color: #d3b517; font-size: 21px; }
input[type=checkbox]::part(box) { width: 34px; height: 30px; background: #fb0307; translate: 7px 0; opacity: 0.5; border: none; transition: none; }
input[type=checkbox][switch]::part(track) { width: 70px; height: 32px; background: #03fb07; border: none; transition: none; }
input[type=radio]::part(ring) { width: 38px; height: 26px; background: #0307fb; border: none; transition: none; }
input[type=radio]::part(chosen) { width: 17px; height: 13px; background: #fb07fb; border: none; }
"#).unwrap();
    for _ in 0..3 {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
        ui.open("Root", Element::column());
        ui.check_field(
            "Check",
            true,
            &CheckFieldDesc {
                label: "Check".into(),
                ..Default::default()
            },
        );
        ui.toggle_switch(
            "Switch",
            true,
            &ToggleSwitchDesc {
                label: "Switch".into(),
                ..Default::default()
            },
        );
        ui.radio_group(
            "Radio",
            &RadioGroupDesc {
                selected: 0,
                options: vec![RadioOption {
                    label: "Radio".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        );
        ui.radio_group(
            "Cards",
            &RadioGroupDesc {
                variant: RadioVariant::Cards,
                selected: 0,
                options: vec![RadioOption {
                    label: "Card".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        );
        ui.close();
        ui.end();
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        for (path, width, height) in [
            ("Root/Check/Box", 34.0, 30.0),
            ("Root/Switch/Track", 70.0, 32.0),
            ("Root/Radio/Options/Option#0/Ring", 38.0, 26.0),
            ("Root/Cards/Options/Option#0/Chosen", 17.0, 13.0),
        ] {
            let part = ui.element(path).unwrap();
            assert_eq!((part.width, part.height), (width, height), "{path}");
        }
        let fills: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .map(|(_, c)| c)
            .collect();
        for rgb in [[251, 3, 7], [3, 251, 7], [3, 7, 251], [251, 7, 251]] {
            let expected = Srgba::rgb(rgb[0], rgb[1], rgb[2]).linear();
            assert!(
                fills.iter().any(|c| c[..3] == expected[..3]),
                "{rgb:?}: {fills:?}"
            );
        }
        let row = ui.element("Root/Check/::part(row)#0").unwrap();
        assert_eq!(ui.element("Root/Check").unwrap().role, "checkbox");
        assert!(row.role.is_empty());
        for label in ["Check", "Switch", "Radio", "Card"] {
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
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    ui.define_document(
        UiDocument::parse(
            "part-box.svelte",
            r#"<meta name="screen" content="parts" data-layer="hud">
{#if show}<button id="Button" onclick={activate}>Go</button>{/if}
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
    ui.push_screen("parts", serde_json::json!({"show":true}))
        .unwrap();
    for _ in 0..3 {
        tick(&mut ui, InputFrame::default());
        let host = ui.element("parts/Button").unwrap();
        let part = ui.element("parts/Button/::part(base)#0").unwrap();
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
    let part = ui.element("parts/Button/::part(base)#0").unwrap();
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
    ui.set_screen_vm("parts", serde_json::json!({"show":false}));
    tick(&mut ui, InputFrame::default());
    assert!(ui.element("parts/Button").is_none());
    assert!(ui.element("parts/Button/::part(base)#0").is_none());
    ui.set_screen_vm("parts", serde_json::json!({"show":true}));
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
    let mut scene = og_render::RenderScene::new();
    let body = add_test_font(&mut scene);
    let display = add_test_font(&mut scene);
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
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
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
            .filter_map(|it| match &it.cmd {
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
    let run = |ui: &mut Ui, poem: &str, plain: &str| {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 300.0));
        ui.column("Page", Element::column(), |ui| {
            ui.text_field(
                "Verse",
                &mut poem.to_string(),
                &FieldDesc {
                    placeholder: "A line".into(),
                    width: Some(Size::Fixed(320.0)),
                    bare: true,
                    tight: true,
                    component: Some("input.poem".into()),
                    ..FieldDesc::default()
                },
            );
            ui.text_field("Plain", &mut plain.to_string(), &FieldDesc::default());
        });
        ui.end();
    };
    run(&mut ui, "", "x");
    let verse = ui
        .elements()
        .iter()
        .find(|e| e.path == "Page/Verse")
        .cloned()
        .unwrap();
    assert_eq!(
        verse.height, lg.line_height,
        "one line of its lg text, no inset"
    );
    let t = texts(&ui);
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
    run(&mut ui, "Spring", "x");
    let line = texts(&ui)
        .into_iter()
        .find(|x| x.0 == "Spring")
        .expect("the value");
    assert_eq!(line.2, lg.size);
    assert!(
        near(line.3, og_ui::style_file::parse_color("#c0392b").unwrap()),
        "{:?}",
        line.3
    );
    let e = ui
        .set_part_css("input.poem::part(frame) { -og-ink: red; }")
        .unwrap_err();
    assert!(e.contains("-og-ink"), "{e}");
    // On a touch screen a tight field is still a finger's target.
    let touch = Viewport {
        coarse_pointer: true,
        ..Viewport::new(600.0, 300.0)
    };
    ui.begin(InputFrame::default(), touch);
    ui.column("Page", Element::column(), |ui| {
        let tight = FieldDesc {
            bare: true,
            tight: true,
            ..FieldDesc::default()
        };
        ui.text_field("Verse", &mut String::new(), &tight);
        let area = TextAreaDesc {
            rows: 1,
            tight: true,
            ..TextAreaDesc::default()
        };
        ui.text_area("Line", &mut String::new(), &area);
    });
    ui.end();
    for path in ["Page/Verse", "Page/Line"] {
        let e = ui
            .elements()
            .iter()
            .find(|e| e.path == path)
            .cloned()
            .unwrap();
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
                "heading::part(flourish) { width: 12px; height: 5px; color: #ff00ff; opacity: 0.4; }\n",
            )
        } else {
            ""
        });
        let mut scene = og_render::RenderScene::new();
        let mut fonts = ui.fonts().clone();
        fonts.body = add_test_font(&mut scene);
        ui.set_fonts(fonts);
        let texture = scene.add_texture(og_render::TextureData {
            width: 2,
            height: 2,
            rgba: vec![255; 16],
            color_space: og_render::ColorSpace::Srgb,
            mips: og_render::Mips::None,
        });
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
        ui.begin(InputFrame::default(), Viewport::new(200.0, 130.0));
        ui.column("Content", Element::column(), |ui| {
            ui.label(
                "Title",
                "Go",
                LabelDesc {
                    heading: true,
                    align: TextAlign::Center,
                    step: Some(TypeStep {
                        size: 16.0,
                        ..theme.type_ramp.body
                    }),
                    ..Default::default()
                },
            );
            ui.scroll(
                "List",
                ScrollDesc {
                    width: Size::Fixed(100.0),
                    height: Size::Fixed(50.0),
                    axes: ScrollAxes::Both,
                    ..Default::default()
                },
                |ui| {
                    ui.open(
                        "Content",
                        Element {
                            width: Size::Fixed(200.0),
                            height: Size::Fixed(120.0),
                            ..Default::default()
                        },
                    );
                    ui.close();
                },
            );
        });
        ui.end();
        let frame = ui.frame();
        assert_eq!(ui.element("List").unwrap().width, 100.0);
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

#[test]
fn a_disabled_touch_controls_opacity_is_authored_by_its_state_dressing() {
    let mut ui = dressed(concat!(
        "actionbutton::part(face) { opacity: 0.8; }\n",
        "actionbutton::part(face):disabled { opacity: 0.2; }\n",
    ));
    for disabled in [false, true, false] {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.action_button(
            "Fire",
            "Fire",
            &ActionButtonDesc {
                disabled,
                primary: true,
                ..ActionButtonDesc::default()
            },
        );
        ui.end();
        let element = ui.element("Fire").unwrap();
        let rect = Rect::new(element.x, element.y, element.width, element.height);
        let fills = fills_in(&ui, rect);
        assert!(!fills.is_empty(), "the action face is drawn");
        let expected = if disabled { 0.2 } else { 0.8 };
        assert!(
            fills.iter().all(|c| (c[3] - expected).abs() < 0.001),
            "{fills:?}"
        );
    }
}

/// `fill none` on a control's state clears the control's own fill: a quiet
/// button's disabled state, dressed `fill none`, draws no box (it drew the
/// control fill, louder than the clear quiet button at rest), while a
/// disabled secondary button the rule does not name keeps its fill.
#[test]
fn fill_none_clears_a_controls_own_fill() {
    let mut ui = dressed("button.quiet::part(base):disabled { background: none; }\n");
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    for (id, variant) in [
        ("Quiet", ButtonVariant::Quiet),
        ("Plain", ButtonVariant::Secondary),
    ] {
        ui.button(
            id,
            "Equip",
            ButtonDesc {
                variant,
                disabled: true,
                ..ButtonDesc::default()
            },
        );
    }
    ui.end();
    let fills = |ui: &Ui, id: &str| {
        let e = ui.element(id).unwrap();
        fills_in(ui, Rect::new(e.x, e.y, e.width, e.height))
    };
    assert_eq!(
        fills(&ui, "Quiet"),
        Vec::<[f32; 4]>::new(),
        "the cleared quiet button"
    );
    assert!(
        !fills(&ui, "Plain").is_empty(),
        "a disabled secondary keeps its fill"
    );
}

/// A game's draw (`set_draw`) paints over its element's background: a rim
/// stroked on an element with a background draws after the background
/// quad, so the background never covers it (it showed only at the corners
/// the radius left uncovered).
#[test]
fn a_draw_strokes_over_its_elements_background() {
    let vp = Viewport::new(400.0, 200.0);
    let mut ui = Ui::new(UiDesc::default());
    // Twice: the second frame reuses the background's kept commands.
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        ui.open(
            "Rimmed",
            Element {
                width: Size::Fixed(120.0),
                height: Size::Fixed(60.0),
                background: Some(Srgba::rgb(20, 40, 200)),
                radius: 12.0,
                ..Element::default()
            },
        );
        ui.set_draw(og_ui::paint::ElementDraw {
            strokes: vec![og_ui::paint::Stroke {
                width: 3.0,
                align: og_ui::paint::StrokeAlign::Inside,
                dash: [0.0; 2],
                dash_offset: 0.0,
                dash_cap: og_ui::paint::Cap::Butt,
                paint: og_ui::paint::Paint::Solid(Srgba::rgb(255, 255, 255).into()),
            }],
            ..Default::default()
        });
        ui.close();
        ui.end();
        // Painter order: level, then emission.
        let items = &ui.frame().items;
        let order = |want: &dyn Fn(&DrawCmd) -> bool| {
            items
                .iter()
                .enumerate()
                .find(|(_, it)| want(&it.cmd))
                .map(|(k, it)| (it.level, k))
        };
        let background =
            order(&|c| matches!(c, DrawCmd::Quad { .. })).expect("the background quad");
        let rim = order(&|c| {
            matches!(c, DrawCmd::Shape { shape, .. }
                if matches!(shape.mode, og_render::UiMode::Stroke(_)))
        })
        .expect("the rim");
        assert!(
            rim > background,
            "rim {rim:?} under background {background:?}"
        );
    }
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

/// A kit's text element that names its part takes the part's `-og-ink` and
/// `-og-step` (docs/ui.md §3.2), qualified by its classes, as a field's
/// `text` part does: a pack inks a killfeed's names by team. Wrong: the
/// style refused `ink` on a kit part, the ink reached no text or every
/// part's text, a qualified dressing lost to the bare one, or text that
/// names no part took it.
#[test]
fn a_kit_text_part_takes_its_ink_and_step() {
    use og_ui::reactive::SourceId;
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
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
            .find_map(|it| match &it.cmd {
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
    use og_ui::dress::PartRef;
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css("button::part(base) { opacity: 0.2; background: #000; transition: opacity 200ms linear, background 200ms linear; } button::part(base):hover { opacity: 1; background: #fff; }").unwrap();
    let step = |ui: &mut Ui, hovered| {
        ui.begin(InputFrame::default(), Viewport::new(100.0, 100.0));
        ui.column(
            "owner",
            Element {
                width: Size::px(80.0),
                height: Size::px(40.0),
                part: Some(PartRef::of("Button", "base", None).flag_if(hovered, "hover")),
                ..Element::column()
            },
            |_| {},
        );
        ui.end();
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        ui.frame()
            .items
            .iter()
            .find_map(|item| item.cmd.filled().map(|(_, color)| color))
            .expect("the part's painted box")
    };
    assert_eq!(step(&mut ui, false)[3], 0.2);
    assert_eq!(step(&mut ui, true)[3], 0.2);
    for _ in 0..5 {
        step(&mut ui, true);
    }
    let half = step(&mut ui, true);
    assert!((half[3] - 0.6).abs() < 0.001, "{half:?}");
    assert!(half[0] > 0.0 && half[0] < 1.0, "{half:?}");
    let saved = ui.save_state();
    let rest = (0..4).map(|_| step(&mut ui, false)).collect::<Vec<_>>();
    ui.restore_state(&saved).unwrap();
    for expected in rest {
        assert_eq!(step(&mut ui, false), expected);
    }
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
    use og_ui::dress::PartRef;
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css("button::part(base) { background: red; animation: pulse-ring 200ms 100ms linear 2 alternate both; } @keyframes pulse-ring { from { opacity: 0.2; } to { opacity: 0.8; } }").unwrap();
    let step = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(100.0, 100.0));
        ui.column(
            "owner",
            Element {
                width: Size::px(80.0),
                height: Size::px(40.0),
                part: Some(PartRef::of("Button", "base", None)),
                ..Element::column()
            },
            |_| {},
        );
        ui.end();
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        ui.frame()
            .items
            .iter()
            .find_map(|i| i.cmd.filled().map(|(_, c)| c[3]))
            .unwrap()
    };
    let samples: Vec<_> = (0..19).map(|_| step(&mut ui)).collect();
    for (tick, expected) in [(0, 0.2), (5, 0.2), (12, 0.5), (18, 0.8)] {
        assert!((samples[tick] - expected).abs() < 0.001, "{samples:?}");
    }
    let saved = ui.save_state();
    let back: Vec<_> = (0..13).map(|_| step(&mut ui)).collect();
    assert!(
        (back[5] - 0.5).abs() < 0.001 && (back[11] - 0.2).abs() < 0.001,
        "{back:?}"
    );
    ui.restore_state(&saved).unwrap();
    for expected in back {
        assert_eq!(step(&mut ui), expected);
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
    let og_render::UiMode::Backdrop {
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
    assert!(matches!(mode, og_render::UiMode::Backdrop { blur: 0.0, glass, .. } if glass.solid));
    assert_eq!(tint[3], 1.0);
}

#[test]
#[ignore = "requires GPU"]
fn css_parts_render_their_background_border_and_shadow() {
    use og_render::*;
    use og_ui::dress::PartRef;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    ui.set_part_css(include_str!("fixtures/parts.css")).unwrap();
    let (width, height) = (464, 96);
    ui.begin(
        InputFrame::default(),
        Viewport::new(width as f32, height as f32),
    );
    ui.row(
        "States",
        Element {
            padding: Insets::all(24.0),
            gap: 24.0,
            ..Element::row()
        },
        |ui| {
            let base = PartRef::of("Button", "base", None);
            for (name, part) in [
                ("Rest", base),
                ("Hover", base.flag("hover")),
                (
                    "Epic",
                    base.flag("primary")
                        .flag("hover")
                        .attr("data-rarity", "epic"),
                ),
            ] {
                let draw = ui.resolve_part(&part, [120.0, 40.0]).unwrap();
                ui.column(
                    name,
                    Element {
                        width: Size::px(120.0),
                        height: Size::px(40.0),
                        ..Element::column()
                    },
                    |ui| ui.set_draw(draw),
                );
            }
        },
    );
    ui.end();
    let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width,
        height,
        ..Default::default()
    })
    .unwrap();
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    image::save_buffer(
        out_dir().join("css-parts.png"),
        &pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();
    for (x, expected) in [
        (24, [254, 220, 186]),
        (84, [0x12, 0x34, 0x56]),
        (228, [255, 255, 0]),
        (372, [0, 0, 255]),
    ] {
        let at = (44 * width as usize + x) * 4;
        for c in 0..3 {
            assert!(
                (i16::from(pixels[at + c]) - expected[c]).abs() <= 3,
                "pixel at {x},44: {:?}",
                &pixels[at..at + 4]
            );
        }
    }
    let shadow = (67 * width as usize + 84) * 4;
    let ground = (85 * width as usize + 84) * 4;
    assert!(
        pixels[shadow] < pixels[ground],
        "the outer shadow darkens the ground"
    );
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
    let mut ui = dressed(
        "progress::part(track) { background: #000; }
         progress::part(fill) { background: #fff; animation: signal 1200ms linear infinite alternate; }
         @keyframes signal { from { opacity: 0.65; } to { opacity: 1; } }",
    );
    let step = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(300.0, 100.0));
        ui.progress("Meter", 0.5, ProgressDesc::default());
        ui.end();
        ui.frame()
            .items
            .iter()
            .filter_map(|item| item.cmd.filled())
            .find(|(_, c)| c[0] > 0.9)
            .expect("white progress fill")
            .1[3]
    };
    let first = step(&mut ui);
    for _ in 0..17 {
        step(&mut ui);
    }
    let quarter = step(&mut ui);
    assert!((first - 0.65).abs() < 0.001, "{first}");
    assert!((quarter - 0.7375).abs() < 0.001, "{quarter}");
    let saved = ui.save_state();
    let later: Vec<_> = (0..65).map(|_| step(&mut ui)).collect();
    assert!(later[53] > later[64], "alternate reverses after 72 ticks");
    ui.restore_state(&saved).unwrap();
    for expected in later {
        assert_eq!(step(&mut ui), expected);
    }
    ui.set_part_css(
        "progress::part(fill) { background: #fff; animation: finish 100ms linear forwards; }
         @keyframes finish { from { opacity: 0.65; } to { opacity: 1; } }",
    )
    .unwrap();
    for _ in 0..10 {
        step(&mut ui);
    }
    assert_eq!(step(&mut ui), 1.0);
    assert_eq!(
        ui.frame().stats.painted,
        0,
        "finished animation reuses paint"
    );
    ui.set_part_css(
        "progress::part(fill) { background: #fff; animation: again 1200ms linear infinite; }
         @keyframes again { from { opacity: 0.65; } to { opacity: 1; } }",
    )
    .unwrap();
    for _ in 0..3 {
        step(&mut ui);
    }
    ui.set_part_css("progress::part(fill) { background: #fff; opacity: 0.8; }")
        .unwrap();
    for _ in 0..3 {
        step(&mut ui);
    }
    assert!((step(&mut ui) - 0.8).abs() < 0.001);
    assert_eq!(
        ui.frame().stats.painted,
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
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    ui.set_part_css(
        "input::part(frame) { padding: 0 11px; border: 2px solid #123456; background: #00ff00; }
         input::part(frame):focus-visible { background: #0000ff; }
         input::part(frame):invalid { background: #ff0000; }
         input[type=number]::part(button) { background: #ff00ff; }
         input::part(reveal) { background: #00ffff; padding: 3px; }
         textarea::part(frame) { padding: 7px; background: #808000; }",
    )
    .unwrap();
    let mut values = [
        "Hi".to_string(),
        "42".into(),
        "pw".into(),
        "x".into(),
        "Area".into(),
    ];
    let tick = |ui: &mut Ui, input, values: &mut [String; 5]| {
        ui.begin(input, Viewport::new(600.0, 600.0));
        ui.open("Root", Element::column());
        let [name, number, secret, bad, area] = values;
        ui.text_field("Name", name, &FieldDesc::default());
        ui.text_field(
            "Count",
            number,
            &FieldDesc {
                input_type: InputType::Number,
                ..Default::default()
            },
        );
        ui.text_field(
            "Secret",
            secret,
            &FieldDesc {
                input_type: InputType::Password,
                ..Default::default()
            },
        );
        ui.text_field(
            "Bad",
            bad,
            &FieldDesc {
                invalid: true,
                ..Default::default()
            },
        );
        ui.text_area("Notes", area, &TextAreaDesc::default());
        ui.close();
        ui.end();
    };
    let press = |ui: &mut Ui, path: &str, dx: f32, values: &mut [String; 5]| {
        let e = ui.element(path).unwrap().clone();
        let pointer = og_core::math::Vec2::new(e.x + e.width * dx, e.y + e.height / 2.0);
        for down in [true, false] {
            tick(
                ui,
                InputFrame {
                    pointer,
                    pointer_inside: true,
                    down,
                    pressed: down,
                    released: !down,
                    ..Default::default()
                },
                values,
            );
        }
    };
    for _ in 0..3 {
        tick(&mut ui, InputFrame::default(), &mut values);
    }
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
    for (host, part, role) in [
        ("Root/Name", "frame", "textbox"),
        ("Root/Notes", "frame", "textbox"),
        ("Root/Count/Increment", "button", "button"),
        ("Root/Secret/Reveal", "reveal", "button"),
    ] {
        let h = ui.element(host).unwrap();
        assert_eq!(h.role, role, "{host}");
        let p = ui.element(&format!("{host}/::part({part})#0")).unwrap();
        assert_eq!(p.kind, "part", "{host}");
        assert!(p.role.is_empty(), "{host}");
        assert_eq!((p.x, p.y, p.width, p.height), (h.x, h.y, h.width, h.height));
    }
    // The frame's padding and border inset the text; its background is
    // its own box's, by state.
    let name = ui.element("Root/Name").unwrap().clone();
    let hi = texts(&ui).into_iter().find(|t| t.0 == "Hi").unwrap();
    assert_eq!(hi.1 - name.x, 13.0);
    assert!(fill(&ui, [0, 255, 0]).iter().any(|r| r.y == name.y));
    let bad = ui.element("Root/Bad").unwrap().clone();
    assert!(fill(&ui, [255, 0, 0]).iter().any(|r| r.y == bad.y));
    let notes = ui.element("Root/Notes").unwrap().clone();
    let area = texts(&ui).into_iter().find(|t| t.0 == "Area").unwrap();
    assert_eq!(area.1 - notes.x, 7.0);
    assert!(fill(&ui, [128, 128, 0]).iter().any(|r| r.y == notes.y));
    let inc = ui.element("Root/Count/Increment").unwrap().clone();
    assert!(fill(&ui, [255, 0, 255]).iter().any(|r| r.x == inc.x));
    assert!(fill(&ui, [0, 255, 255]).len() == 1);
    // Presses and typing reach the hosts.
    press(&mut ui, "Root/Count/Increment", 0.5, &mut values);
    assert_eq!(values[1], "43");
    press(&mut ui, "Root/Secret/Reveal", 0.5, &mut values);
    assert_eq!(
        ui.element("Root/Secret/Reveal").unwrap().text,
        "Hide password"
    );
    press(&mut ui, "Root/Name", 0.9, &mut values);
    assert!(ui.focus_path(0).ends_with("/Name"));
    tick(
        &mut ui,
        InputFrame {
            text: "!".into(),
            ..Default::default()
        },
        &mut values,
    );
    assert_eq!(values[0], "Hi!");
    for _ in 0..20 {
        tick(&mut ui, InputFrame::default(), &mut values);
    }
    assert!(fill(&ui, [0, 0, 255]).iter().any(|r| r.y == name.y));
    assert!(!fill(&ui, [0, 255, 0]).iter().any(|r| r.y == name.y));
}

/// An undressed text control draws as before its frame became a box: one
/// ground at the host's box, its text at the host's inset, no fill of the
/// host's own. Wrong: the host and its frame both painted, or the text
/// moved by the frame's border.
#[test]
fn undressed_text_frames_keep_the_hosts_box_and_inset() {
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    let mut value = "Hello".to_string();
    let mut invalid = "Bad".to_string();
    for _ in 0..3 {
        ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
        ui.open("Root", Element::column());
        ui.text_field("Name", &mut value, &FieldDesc::default());
        ui.text_field(
            "Bad",
            &mut invalid,
            &FieldDesc {
                invalid: true,
                ..Default::default()
            },
        );
        ui.close();
        ui.end();
    }
    let pad = ui.theme().space.md;
    let control = ui.theme().palette.control.linear();
    for (path, text) in [("Root/Name", "Hello"), ("Root/Bad", "Bad")] {
        let host = ui.element(path).unwrap().clone();
        let frame = ui.element(&format!("{path}/::part(frame)#0")).unwrap();
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
