//! The screens og-ui's tests drive and its reference frames show: the
//! sample screen, the game widget set, the text fields and the mixed-script
//! labels, built through og-ui alone. `scenes.rs` makes render scenes of
//! them; the browser tiers (crates/og-tests-web) compile both. Nothing
//! here reads a file: the test font comes in as bytes ([`FontFiles`]).

use og_core::glam::Vec2;
use og_ui::*;

/// The game state the sample screen edits.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub music: bool,
    pub volume: f32,
    pub search: String,
    pub loading: f32,
    /// Every click, as the clicked element's path.
    pub clicks: Vec<String>,
    pub submitted: Vec<String>,
}

impl Default for Sample {
    fn default() -> Sample {
        Sample {
            music: true,
            volume: 0.6,
            search: String::new(),
            loading: 0.42,
            clicks: Vec::new(),
            submitted: Vec::new(),
        }
    }
}

pub const ITEMS: u32 = 12;

/// The play surface of the sample: 800×500 CSS pixels at 1.5× (1200×750
/// device pixels; compact density, rectangles snapped to 2/3 CSS px).
pub fn viewport() -> Viewport {
    Viewport {
        width: 800.0,
        height: 500.0,
        dpr: 1.5,
        ..Viewport::default()
    }
}

/// A game menu and a store: every widget kind, a scroll list, a wrapping row.
pub fn sample_screen(ui: &mut Ui, s: &mut Sample) {
    let t = *ui.theme();
    ui.row(
        "Screen",
        Element {
            width: Size::grow(),
            height: Size::grow(),
            padding: Insets::all(t.space.xl),
            gap: t.space.lg,
            background: Some(t.palette.backdrop),
            ..Element::default()
        },
        |ui| {
            ui.panel(
                "Menu",
                PanelDesc {
                    width: Size::Fixed(220.0),
                    height: Size::grow(),
                    ..PanelDesc::default()
                },
                |ui| {
                    ui.label(
                        "Title",
                        "Origin",
                        LabelDesc {
                            step: Some(t.type_ramp.display_sm),
                            ..LabelDesc::default()
                        },
                    );
                    ui.label(
                        "Subtitle",
                        "A small game made with Origin Engine.",
                        LabelDesc {
                            step: Some(t.type_ramp.meta),
                            color: Some(t.palette.ink_muted),
                            ..LabelDesc::default()
                        },
                    );
                    let grow = Some(Size::grow());
                    for (id, text, variant) in [
                        ("Start", "Start", ButtonVariant::Primary),
                        ("Options", "Options", ButtonVariant::Secondary),
                        ("Quit", "Quit", ButtonVariant::Danger),
                    ] {
                        if ui
                            .button(
                                id,
                                text,
                                ButtonDesc {
                                    variant,
                                    width: grow,
                                    ..ButtonDesc::default()
                                },
                            )
                            .clicked
                        {
                            s.clicks.push(id.to_owned());
                        }
                    }
                    let music = ui.toggle_switch(
                        "Music",
                        s.music,
                        &ToggleSwitchDesc {
                            label: "Music".into(),
                            ..ToggleSwitchDesc::default()
                        },
                    );
                    s.music = music.on;
                    if music.response.clicked {
                        s.clicks.push("Music".into());
                    }
                    s.volume = ui
                        .value_slider("Volume", s.volume, &ValueSliderDesc::default())
                        .value;
                    ui.progress("Loading", s.loading, ProgressDesc::default());
                },
            );
            ui.panel(
                "Store",
                PanelDesc {
                    width: Size::grow(),
                    height: Size::grow(),
                    ..PanelDesc::default()
                },
                |ui| {
                    ui.row(
                        "Header",
                        Element {
                            width: Size::grow(),
                            cross: Cross::Center,
                            gap: t.space.sm,
                            ..Element::default()
                        },
                        |ui| {
                            ui.label(
                                "Heading",
                                "Store",
                                LabelDesc {
                                    step: Some(t.type_ramp.xl),
                                    ..LabelDesc::default()
                                },
                            );
                            ui.label(
                                "Engine",
                                "原点引擎",
                                LabelDesc {
                                    color: Some(t.palette.ink_muted),
                                    width: Size::grow(),
                                    ..LabelDesc::default()
                                },
                            );
                            ui.label(
                                "Coins",
                                "1250 coins",
                                LabelDesc {
                                    step: Some(t.type_ramp.meta),
                                    color: Some(t.palette.highlight),
                                    nowrap: true,
                                    ..LabelDesc::default()
                                },
                            );
                        },
                    );
                    let r = ui.text_field(
                        "Search",
                        &mut s.search,
                        &FieldDesc {
                            placeholder: "Search items".into(),
                            ..FieldDesc::default()
                        },
                    );
                    if r.submitted {
                        s.submitted.push(s.search.clone());
                    }
                    ui.scroll(
                        "List",
                        ScrollDesc {
                            width: Size::grow(),
                            height: Size::grow(),
                            gap: t.space.xs,
                            ..ScrollDesc::default()
                        },
                        |ui| {
                            for n in 0..ITEMS {
                                ui.row(
                                    ui_idi("Row", n),
                                    Element {
                                        width: Size::grow(),
                                        padding: Insets::xy(t.space.md, t.space.xs),
                                        cross: Cross::Center,
                                        background: Some(t.palette.control),
                                        radius: t.radius.small,
                                        ..Element::default()
                                    },
                                    |ui| {
                                        ui.label(
                                            "Name",
                                            &format!("Item {n}"),
                                            LabelDesc {
                                                width: Size::grow(),
                                                ..LabelDesc::default()
                                            },
                                        );
                                        let buy = ui.button(
                                            "Buy",
                                            "Buy",
                                            ButtonDesc {
                                                size: ControlSize::Small,
                                                ..ButtonDesc::default()
                                            },
                                        );
                                        ui.tooltip(&format!("{} coins", 100 + 10 * n));
                                        if buy.clicked {
                                            s.clicks.push(format!("Row#{n}/Buy"));
                                        }
                                    },
                                );
                            }
                        },
                    );
                    ui.row(
                        "Tags",
                        Element {
                            width: Size::grow(),
                            wrap: true,
                            gap: t.space.xs,
                            ..Element::default()
                        },
                        |ui| {
                            for (k, tag) in [
                                "Sword", "Shield", "Potion", "Bow", "Helmet", "Ring", "Boots",
                                "Map",
                            ]
                            .iter()
                            .enumerate()
                            {
                                if ui
                                    .button(
                                        ui_idi("Tag", k as u32),
                                        tag,
                                        ButtonDesc {
                                            size: ControlSize::Small,
                                            variant: ButtonVariant::Quiet,
                                            ..ButtonDesc::default()
                                        },
                                    )
                                    .clicked
                                {
                                    s.clicks.push(format!("Tag#{k}"));
                                }
                            }
                        },
                    );
                },
            );
        },
    );
}

/// The MSDF test font's files (msdf-atlas-gen's PNG, JSON and CSV; the Noto
/// Sans SC subset og-tests commits under `assets/fonts/noto-sans-sc-msdf.*`).
pub struct FontFiles<'a> {
    pub png: &'a [u8],
    pub json: &'a str,
    pub csv: &'a str,
}

/// A Ui with the test font's metrics (and handle when one is given).
pub fn ui_with_font(font: UiFont) -> Ui {
    Ui::new(UiDesc {
        fonts: Fonts {
            body: font,
            ..Fonts::default()
        },
        ..UiDesc::default()
    })
}

/// Ticks of the sample session: focus the search field and type, scroll the
/// list a little, then rest the pointer on a Buy button until its tooltip shows.
pub fn sample_session(ui: &mut Ui, s: &mut Sample) {
    let vp = viewport();
    let step = |ui: &mut Ui, s: &mut Sample, input: InputFrame| {
        ui.begin(input, vp);
        sample_screen(ui, s);
        ui.end();
    };
    step(ui, s, InputFrame::default());
    let at = |ui: &Ui, path: &str| {
        let e = ui
            .element(path)
            .unwrap_or_else(|| panic!("no element {path}"));
        let (x, y) = e.center();
        Vec2::new(x, y)
    };
    let field = at(ui, "Search");
    let press = |p: Vec2| InputFrame {
        pointer: p,
        pointer_inside: true,
        down: true,
        pressed: true,
        ..InputFrame::default()
    };
    let release = |p: Vec2| InputFrame {
        pointer: p,
        pointer_inside: true,
        released: true,
        ..InputFrame::default()
    };
    step(ui, s, press(field));
    step(ui, s, release(field));
    step(
        ui,
        s,
        InputFrame {
            text: "pot".into(),
            ..InputFrame::hover(field.x, field.y)
        },
    );
    let list = at(ui, "List");
    step(
        ui,
        s,
        InputFrame {
            wheel: Vec2::new(0.0, 22.0),
            ..InputFrame::hover(list.x, list.y)
        },
    );
    let buy = at(ui, "Row#2/Buy");
    for _ in 0..40 {
        step(ui, s, InputFrame::hover(buy.x, buy.y));
    }
}

/// The game widget set's state.
#[derive(Default)]
pub struct Kit {
    pub hp: f32,
    pub tab: usize,
    pub pick: usize,
    pub amount: f32,
    pub mode: usize,
    pub checks: [bool; 2],
}

pub fn kit_screen(ui: &mut Ui, s: &mut Kit) {
    let t = *ui.theme();
    ui.row(
        "Screen",
        Element {
            width: Size::grow(),
            height: Size::grow(),
            padding: Insets::all(t.space.lg),
            gap: t.space.lg,
            background: Some(t.palette.backdrop),
            ..Element::default()
        },
        |ui| {
            ui.panel(
                "Left",
                PanelDesc {
                    width: Size::Fixed(380.0),
                    height: Size::grow(),
                    ..PanelDesc::default()
                },
                |ui| {
                    s.tab = ui
                        .segmented(
                            "Tabs",
                            &SegmentedDesc {
                                items: ["Overview", "Stats", "Gear"]
                                    .map(|l| SegmentItem {
                                        label: l.into(),
                                        ..SegmentItem::default()
                                    })
                                    .to_vec(),
                                selected: s.tab as i32,
                                ..SegmentedDesc::default()
                            },
                        )
                        .selected as usize;
                    ui.row(
                        "Pickers",
                        Element {
                            gap: t.space.md,
                            cross: Cross::Center,
                            ..Element::default()
                        },
                        |ui| {
                            let pick = ui.dropdown(
                                "Difficulty",
                                &DropdownDesc {
                                    options: ["Easy", "Normal", "Hard"]
                                        .map(|l| DropdownOption {
                                            label: l.into(),
                                            ..DropdownOption::default()
                                        })
                                        .to_vec(),
                                    selected: s.pick as i32,
                                    width: Some(Size::Fixed(160.0)),
                                    ..DropdownDesc::default()
                                },
                            );
                            s.pick = pick.selected.max(0) as usize;
                            s.amount = ui
                                .number_stepper(
                                    "Amount",
                                    s.amount,
                                    &NumberStepperDesc {
                                        min: Some(0.0),
                                        max: Some(9.0),
                                        ..NumberStepperDesc::default()
                                    },
                                )
                                .value;
                        },
                    );
                    s.mode = ui
                        .radio_group(
                            "Mode",
                            &RadioGroupDesc {
                                options: ["Solo", "Co-op", "Versus"]
                                    .map(|l| RadioOption {
                                        label: l.into(),
                                        ..RadioOption::default()
                                    })
                                    .to_vec(),
                                selected: s.mode as i32,
                                row: true,
                                ..RadioGroupDesc::default()
                            },
                        )
                        .selected
                        .max(0) as usize;
                    ui.column("Options", Element::column(), |ui| {
                        for (k, label) in ["Show hints", "Hard mode"].into_iter().enumerate() {
                            s.checks[k] = ui
                                .check_field(
                                    ui_idi("Option", k as u32),
                                    s.checks[k],
                                    &CheckFieldDesc {
                                        label: label.into(),
                                        ..CheckFieldDesc::default()
                                    },
                                )
                                .on;
                        }
                    });
                    ui.row(
                        "Chips",
                        Element {
                            gap: t.space.sm,
                            cross: Cross::Center,
                            ..Element::default()
                        },
                        |ui| {
                            ui.chip(
                                "All",
                                "All",
                                ChipDesc {
                                    selected: true,
                                    ..ChipDesc::default()
                                },
                            );
                            ui.chip("Weapons", "Weapons", ChipDesc::default());
                            ui.badge("New", "New", Tone::Primary);
                            ui.badge("Sale", "-20%", Tone::Success);
                        },
                    );
                    ui.divider("Rule", false);
                    ui.meter(
                        "Health",
                        s.hp,
                        100.0,
                        &MeterDesc {
                            label: format!("{} / 100", s.hp),
                            ..MeterDesc::default()
                        },
                    );
                    ui.meter(
                        "Xp",
                        620.0,
                        1000.0,
                        &MeterDesc {
                            kind: MeterKind::Xp,
                            segments: 5,
                            ..MeterDesc::default()
                        },
                    );
                    ui.skeleton(
                        "Skeleton",
                        SkeletonDesc {
                            lines: 2,
                            ..SkeletonDesc::default()
                        },
                    );
                },
            );
            ui.column(
                "Right",
                Element {
                    gap: t.space.lg,
                    ..Element::default()
                },
                |ui| {
                    ui.row(
                        "Rings",
                        Element {
                            gap: t.space.md,
                            ..Element::default()
                        },
                        |ui| {
                            for (k, rem) in [1.0, 0.75, 0.4, 0.0].into_iter().enumerate() {
                                ui.cooldown(
                                    ui_idi("Ring", k as u32),
                                    rem,
                                    &CooldownDesc {
                                        label: format!("{}", k + 1),
                                        ..CooldownDesc::default()
                                    },
                                );
                            }
                        },
                    );
                    ui.row(
                        "Touch",
                        Element {
                            gap: t.space.lg,
                            cross: Cross::End,
                            ..Element::default()
                        },
                        |ui| {
                            ui.joystick("Stick", JoystickDesc::default());
                            ui.dpad("Dpad", DpadDesc::default());
                            ui.action_button(
                                "Jump",
                                "Jump",
                                &ActionButtonDesc {
                                    primary: true,
                                    ..ActionButtonDesc::default()
                                },
                            );
                        },
                    );
                    ui.spinner("Spinner", 0.0);
                },
            );
        },
    );
}

/// The fields of `text-fields-2` and the text they hold.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fields {
    pub words: String,
    pub undo: String,
    pub notes: String,
}

/// A column of two fields (a double click's word, an undone edit) and a
/// three-row text area, `top` CSS px down the surface.
pub fn fields_screen(ui: &mut Ui, f: &mut Fields, top: f32) {
    let t = *ui.theme();
    ui.column(
        "Fields",
        Element {
            width: Size::grow(),
            padding: Insets {
                left: t.space.lg,
                top: t.space.lg + top,
                right: t.space.lg,
                bottom: 0.0,
            },
            gap: t.space.sm,
            ..Element::column()
        },
        |ui| {
            let field = FieldDesc {
                width: Some(Size::Fixed(320.0)),
                ..FieldDesc::default()
            };
            ui.text_field("Words", &mut f.words, &field);
            ui.text_field("Undo", &mut f.undo, &field);
            ui.text_area(
                "Notes",
                &mut f.notes,
                &TextAreaDesc {
                    width: Some(Size::Fixed(320.0)),
                    rows: 3,
                    ..TextAreaDesc::default()
                },
            );
        },
    );
}

/// The surface of `text-fields-2`.
pub fn text_fields_viewport() -> Viewport {
    Viewport {
        width: 520.0,
        height: 330.0,
        dpr: 1.5,
        ..Viewport::default()
    }
}

/// The two UIs of `text-fields-2` after their input ([`text_fields_shot`]):
/// the fields' and the area's.
pub fn text_fields_uis(font: &UiFont) -> (Ui, Ui) {
    let vp = text_fields_viewport();
    let at = |p: Vec2, down: bool, pressed: bool, released: bool| InputFrame {
        pointer: p,
        pointer_inside: true,
        down,
        pressed,
        released,
        ..InputFrame::default()
    };
    let center = |ui: &Ui, id: &str| {
        let e = ui.element(id).expect(id);
        Vec2::new(e.x + e.width - 4.0, e.y + e.height / 2.0)
    };

    let mut ui = ui_with_font(font.clone());
    let mut f = Fields {
        words: "drag to select words".into(),
        ..Fields::default()
    };
    let step = |ui: &mut Ui, f: &mut Fields, input: InputFrame| {
        ui.begin(input, vp);
        fields_screen(ui, f, 0.0);
        ui.end();
    };
    step(&mut ui, &mut f, InputFrame::default());
    let p = center(&ui, "Undo");
    step(&mut ui, &mut f, at(p, true, true, false));
    step(&mut ui, &mut f, at(p, false, false, true));
    for c in "one two three".chars() {
        step(
            &mut ui,
            &mut f,
            InputFrame {
                text: c.to_string(),
                ..InputFrame::default()
            },
        );
    }
    step(
        &mut ui,
        &mut f,
        InputFrame {
            keys: vec![UiKey::KeyZ],
            ctrl: true,
            ..InputFrame::default()
        },
    );
    // A double click inside "select" (the test face's advances at body size).
    let words = ui.element("Words").expect("words").clone();
    let t = *ui.theme();
    let m = &font.metrics;
    let x = words.x + t.space.md + m.measure("drag to sel", t.type_ramp.body.size);
    let p = Vec2::new(x, words.y + words.height / 2.0);
    for _ in 0..2 {
        step(&mut ui, &mut f, at(p, true, true, false));
        step(&mut ui, &mut f, at(p, false, false, true));
    }
    step(&mut ui, &mut f, InputFrame::default());

    let mut area = ui_with_font(font.clone());
    let mut g = Fields {
        notes: (1..=9)
            .map(|k| format!("Line {k} of the notes"))
            .collect::<Vec<_>>()
            .join("\n"),
        ..Fields::default()
    };
    let step = |ui: &mut Ui, f: &mut Fields, input: InputFrame| {
        ui.begin(input, vp);
        fields_screen(ui, f, 120.0);
        ui.end();
    };
    step(&mut area, &mut g, InputFrame::default());
    let notes = area.element("Notes").expect("notes").clone();
    let start = Vec2::new(notes.x + 30.0, notes.y + 14.0);
    step(&mut area, &mut g, at(start, true, true, false));
    let below = Vec2::new(notes.x + 120.0, notes.y + notes.height + 30.0);
    for _ in 0..6 {
        step(&mut area, &mut g, at(below, true, false, false));
    }

    (ui, area)
}

/// The labels of `text-mixed-runs`: lines that mix Latin with CJK, Arabic,
/// Hebrew and Devanagari, each drawn a run per font (`Fonts::link`).
pub const MIXED_LINES: [&str; 5] = [
    "Level 1-2 古い水車小屋",
    "Score 42: مرحبا بك",
    "Player: שלום, नमस्ते!",
    "Found 冒険 and مغامرة in one line",
    "A wrapped paragraph of Latin words with 古い水車小屋 inside it, and مرحبا بك too, broken where it meets the column's edge.",
];

/// The UI of `references/<tier>/text-mixed-runs.png` (og-tests-cooked's `tests/text_gfx.rs`
/// cooks its fonts and draws it at `vp`): the mixed lines as labels (the
/// last one wrapping in a 300 px column), rich text with a bold Latin run
/// beside CJK and Arabic spans, and a focused field holding a mixed name
/// with its caret, in `fonts` (a Latin face first, the other scripts'
/// faces as fallbacks).
pub fn mixed_runs_ui(fonts: Fonts, vp: Viewport) -> Ui {
    let mut ui = Ui::new(UiDesc {
        fonts,
        ..UiDesc::default()
    });
    let mut name = "Name 冒険".to_string();
    let mut step = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp);
        let t = *ui.theme();
        ui.column(
            "Mixed",
            Element {
                width: Size::grow(),
                padding: Insets::all(t.space.lg),
                gap: t.space.sm,
                ..Element::column()
            },
            |ui| {
                for (k, line) in MIXED_LINES.iter().enumerate().take(4) {
                    ui.label(ui_idi("Line", k as u32), line, LabelDesc::default());
                }
                ui.label(
                    "Wrapped",
                    MIXED_LINES[4],
                    LabelDesc {
                        width: Size::Fixed(300.0),
                        ..LabelDesc::default()
                    },
                );
                ui.rich_text(
                    "Rich",
                    "[b]Rank 3[/b] 冒険 [color=primary]مغامرة[/color]",
                    RichTextDesc::default(),
                );
                ui.text_field(
                    "Name",
                    &mut name,
                    &FieldDesc {
                        width: Some(Size::Fixed(300.0)),
                        ..FieldDesc::default()
                    },
                );
            },
        );
        ui.end();
    };
    step(&mut ui, InputFrame::default());
    let e = ui.element("Name").expect("name").clone();
    let p = Vec2::new(e.x + e.width - 4.0, e.y + e.height / 2.0);
    for (down, pressed, released) in [(true, true, false), (false, false, true)] {
        step(
            &mut ui,
            InputFrame {
                pointer: p,
                pointer_inside: true,
                down,
                pressed,
                released,
                ..InputFrame::default()
            },
        );
    }
    step(&mut ui, InputFrame::default());
    ui
}

/// The selectable text of `user-select-<width>` (tests/user_select.rs).
pub const USER_SELECT_DOC: &str = r#"<meta name="component" content="sel">
<column id="Doc">
  <p id="Wrap">The quick brown fox jumps over the lazy dog</p>
  <p id="Second">Second paragraph</p>
  <button id="Go">Go</button>
  <p id="Off">Not this one</p>
  <p id="Code">ID-1234</p>
</column>
<style>
#Doc { user-select: text; padding: 16px; gap: 12px; width: 600px; }
#Wrap { width: 80px; }
#Off { user-select: none; }
#Code { user-select: all; }
</style>"#;

/// `text-path-<width>`: an arched title, a badge ringed with text (one
/// ring outside, one inside on the right side) and a wave, each in
/// `-og-text-path` (tests/text_path.rs).
pub const TEXT_PATH_DOC: &str = r#"<meta name="component" content="tp">
<column id="Root">
  <p id="Arch">Legends of the Arch</p>
  <row id="Badges">
    <stack id="Badge"><column id="Seal"/><p id="Outer">GUILD OF MAKERS - EST 2026 -</p><p id="Inner">well met, traveller</p></stack>
    <p id="Wave">riding the wave of words</p>
  </row>
</column>
<style>
#Root { padding: 24px; gap: 16px; }
#Badges { gap: 24px; flex-wrap: wrap; }
#Arch { -og-text-path: path("M0 90 Q170 -20 340 90"); -og-text-path-offset: 50%; text-align: center; font: var(--text-xl); letter-spacing: var(--type-xl-tracking); }
#Badge { width: 180px; height: 180px; }
#Seal { width: 120px; height: 120px; border-radius: 60px; background: #7a2e22; translate: 30px 30px; }
#Outer { -og-text-path: circle(82px); translate: 8px 8px; }
#Inner { -og-text-path: circle(52px); -og-text-path-side: right; -og-text-path-offset: 50%; text-align: center; translate: 38px 38px; }
#Wave { -og-text-path: path("M0 40 C40 0 80 0 120 40 S200 80 240 40"); -og-text-path-offset: 8px; }
</style>"#;
