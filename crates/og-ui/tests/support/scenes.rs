//! The scenes og-ui's reference frames are drawn from (`references/<tier>/`
//! sample-screen, text-glow, glass, widget-set and text-fields-2; and
//! text-mixed-runs, whose fonts og-tests-cooked's `tests/text_gfx.rs` cooks), as scenes ready to
//! render: shared by the native tests (`tests/gfx.rs`, `tests/kit_gfx.rs`,
//! through `support`) and the browser tiers (crates/og-tests-web includes
//! this file, packages/engine/tests/web/reference-gfx.ts renders them), so
//! every tier draws the same frame. Nothing here reads a file: the test
//! font comes in as bytes ([`FontFiles`]).

use og_core::glam::{Vec2, Vec3};
use og_render::{
    Background, ColorSpace, Glyph, Mips, MsdfFont, RenderDesc, RenderScene, TextureData, Tonemap,
};
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

/// Adds the MSDF test font to a scene; the UI font for it, with the font's
/// own ascender and descender.
pub fn add_font(scene: &mut RenderScene, files: &FontFiles) -> UiFont {
    let img = image::load_from_memory(files.png)
        .unwrap_or_else(|e| panic!("noto-sans-sc-msdf.png: {e}"))
        .to_rgba8();
    let (width, height) = img.dimensions();
    let atlas = scene.add_texture(TextureData {
        width,
        height,
        rgba: img.into_raw(),
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    });
    let json: serde_json::Value = serde_json::from_str(files.json).expect("font json parses");
    let number = |a: &str, b: &str| json[a][b].as_f64().unwrap_or_else(|| panic!("{a}.{b}")) as f32;
    let mut glyphs = Vec::new();
    for line in files.csv.lines().filter(|l| !l.trim().is_empty()) {
        let f: Vec<f32> = line
            .split(',')
            .map(|v| v.trim().parse().expect("number"))
            .collect();
        // -yorigin top: plane bounds left, top, right, bottom with y down.
        glyphs.push(Glyph {
            key: char::from_u32(f[0] as u32).expect("codepoint").into(),
            advance: f[1],
            plane_bounds: [f[2], -f[5], f[4], -f[3]],
            atlas_bounds: [f[6], f[7], f[8], f[9]],
        });
    }
    let font = MsdfFont {
        atlas,
        distance_range: number("atlas", "distanceRange"),
        foreground: false,
        silhouette: false,
        line_height: number("metrics", "lineHeight"),
        glyphs,
        kerning: Vec::new(),
    };
    // yOrigin top: the JSON's ascender is negative, its descender positive.
    let metrics = FontMetrics::from_msdf(&font).with_extents(
        -number("metrics", "ascender"),
        -number("metrics", "descender"),
    );
    let handle = scene.add_font(font);
    UiFont {
        handle: Some(handle),
        metrics,
    }
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

/// A UI frame ready to render: the Ui that laid it out (its frame is what
/// the tests assert on), the scene with that frame emitted as its overlay
/// over the theme's background, and the renderer description at the
/// viewport's device size.
pub struct Shot {
    pub ui: Ui,
    pub scene: RenderScene,
    pub desc: RenderDesc,
}

/// Emits `ui`'s frame into `scene`'s overlay on the theme's background
/// (no tonemap) and sizes the renderer to `vp` in device pixels.
pub fn shot(ui: Ui, mut scene: RenderScene, vp: Viewport) -> Shot {
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.lighting.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: (vp.width * vp.dpr) as u32,
        height: (vp.height * vp.dpr) as u32,
        ..RenderDesc::default()
    };
    Shot { ui, scene, desc }
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

/// `references/<tier>/sample-screen.png`: the sample screen after its
/// session, 800×500 at 1.5×.
pub fn sample_screen_shot(font: &FontFiles) -> (Shot, Sample) {
    let vp = viewport();
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    let mut s = Sample::default();
    sample_session(&mut ui, &mut s);
    (shot(ui, scene, vp), s)
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

/// `references/<tier>/widget-set.png`: the widget set after a hit, an open
/// select moved to Hard by Tab, and a toast, 900×520 at 1.5×.
pub fn widget_set_shot(font: &FontFiles) -> Shot {
    let vp = Viewport {
        width: 900.0,
        height: 520.0,
        dpr: 1.5,
        ..Viewport::default()
    };
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    let mut s = Kit {
        hp: 100.0,
        amount: 3.0,
        checks: [true, false],
        pick: 1,
        ..Kit::default()
    };
    let step = |ui: &mut Ui, s: &mut Kit, input: InputFrame| {
        ui.begin(input, vp);
        kit_screen(ui, s);
        ui.end();
    };
    step(&mut ui, &mut s, InputFrame::default());
    // A hit: the meter's trail shows the old value for a beat.
    s.hp = 55.0;
    for _ in 0..4 {
        step(&mut ui, &mut s, InputFrame::default());
    }
    // Open the select with a press: its menu floats over the panel and
    // takes focus; Tab moves it to Hard with the ring and its gap.
    let sel = ui.element("Difficulty").expect("select").clone();
    let (x, y) = sel.center();
    let p = Vec2::new(x, y);
    step(
        &mut ui,
        &mut s,
        InputFrame {
            pointer: p,
            pointer_inside: true,
            down: true,
            pressed: true,
            ..InputFrame::default()
        },
    );
    step(
        &mut ui,
        &mut s,
        InputFrame {
            pointer: p,
            pointer_inside: true,
            released: true,
            ..InputFrame::default()
        },
    );
    step(
        &mut ui,
        &mut s,
        InputFrame {
            keys: vec![UiKey::Tab],
            ..InputFrame::hover(p.x, p.y)
        },
    );
    ui.toast(
        "Achievement unlocked",
        ToastDesc {
            tone: Tone::Success,
            ..ToastDesc::default()
        },
    );
    for _ in 0..20 {
        step(&mut ui, &mut s, InputFrame::hover(p.x, p.y));
    }
    shot(ui, scene, vp)
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

/// `references/<tier>/text-fields-2.png`, 520×330 at 1.5×, two UIs (each
/// with its own focus) in one frame: in the first, "one two three" typed a
/// character a tick into the second field and undone back to "one two ",
/// then "select" in the first field selected by a double click (its caret
/// at the word's end); in the second, a press on the area's first line
/// dragged 30 px below it and held there 6 ticks, the area scrolled about
/// three lines (a line cut part way at its top) and its selection over
/// every line it passed, the press still held.
pub fn text_fields_shot(font: &FontFiles) -> Shot {
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let (ui, area) = text_fields_uis(&font);
    let vp = text_fields_viewport();
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    // The area's UI under the fields' (each band leaves the other's empty).
    area.frame().emit(&EmitDesc::default(), &mut overlay);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.lighting.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: (vp.width * vp.dpr) as u32,
        height: (vp.height * vp.dpr) as u32,
        ..RenderDesc::default()
    };
    Shot { ui, scene, desc }
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

/// A dark card of display-size labels, with or without a style's text
/// glow, as a scene ready to render.
pub fn glow_scene(font: &FontFiles, glow: bool, labels: usize) -> (RenderScene, RenderDesc) {
    let vp = Viewport {
        dpr: 1.0,
        ..Viewport::new(480.0, 200.0f32.max(16.0 + 40.0 * labels as f32))
    };
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    let mut theme = *ui.theme();
    if glow {
        theme.art.text_glows[0] = Some(og_ui::style_file::Glow {
            size: 2.0,
            ink: og_ui::style_file::Ink::Primary,
            alpha: 0.8,
        });
    }
    ui.set_theme(theme);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        let t = *ui.theme();
        ui.column(
            "Card",
            Element {
                width: Size::grow(),
                height: Size::grow(),
                padding: Insets::all(t.space.lg),
                background: Some(t.palette.backdrop),
                ..Element::default()
            },
            |ui| {
                for k in 0..labels {
                    ui.label(
                        ui_idi("Title", k as u32),
                        "Wall 42",
                        LabelDesc {
                            // A shader fixture, independent of the style's
                            // responsive ramp. Tight tracking overlaps quads.
                            step: Some(TypeStep {
                                size: 34.0,
                                line_height: 38.0,
                                tracking_em: -0.1,
                                cjk_line_height: 44.0,
                                cjk_tracking_em: 0.0,
                                features: og_ui::shape::Features::NONE,
                                weight: 400,
                                uppercase: false,
                            }),
                            ..LabelDesc::default()
                        },
                    );
                }
            },
        );
        ui.end();
    }
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.lighting.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: vp.width as u32,
        height: vp.height as u32,
        ..RenderDesc::default()
    };
    (scene, desc)
}

/// A scene of fine vertical stripes (4 px black, 4 px white) under an
/// overlay holding `glass` panels, 320×200 at 1×.
pub fn stripes_scene(glass: &[og_render::Glass]) -> (RenderScene, RenderDesc) {
    let (w, h) = (320u32, 200u32);
    let mut scene = RenderScene::new();
    scene.camera =
        og_render::Camera::orthographic_2d(Vec2::new(w as f32 / 2.0, h as f32 / 2.0), h as f32);
    for k in 0..(w / 8) {
        scene.sprite.sprites.push(og_render::Sprite {
            position: Vec3::new(k as f32 * 8.0 + 2.0, h as f32 / 2.0, 0.0),
            size: Vec2::new(4.0, h as f32),
            color: [1.0, 1.0, 1.0, 1.0],
            ..og_render::Sprite::default()
        });
    }
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let mut overlay = og_render::Overlay::new(w as f32, h as f32, 1.0);
    overlay.glass.extend_from_slice(glass);
    scene.ui.overlay = Some(overlay);
    let desc = RenderDesc {
        width: w,
        height: h,
        ..RenderDesc::default()
    };
    (scene, desc)
}

/// The glass panel of `references/<tier>/glass.png`: a 160×120 rounded
/// rectangle at (80, 40) over the stripes, blur 16, no tint.
pub fn glass_panel() -> og_render::Glass {
    og_render::Glass {
        position: Vec2::new(80.0, 40.0),
        size: Vec2::new(160.0, 120.0),
        radius: 24.0,
        blur: 16.0,
        tint: [0.0, 0.0, 0.0, 0.0],
        alpha: 1.0,
        layer: 0,
        clip: None,
    }
}

/// Overlay content around the reference glass panel: an orange bar on the
/// overlay layer below it, crossing the panel (y 92..108, x 40..280), and
/// a small cyan square on the layer above it, inside the panel at
/// (190, 60), 16 px. The panel blurs the bar where it covers it and
/// leaves the square sharp.
pub fn glass_overlay_content() -> Vec<og_render::Sprite> {
    let solid = |x: f32, y: f32, w: f32, h: f32, color: [f32; 4], layer: i32| og_render::Sprite {
        texture: None,
        position: Vec3::new(x, y, 0.0),
        size: Vec2::new(w, h),
        anchor: Vec2::new(0.0, 0.0),
        color,
        layer,
        ..og_render::Sprite::default()
    };
    vec![
        solid(40.0, 92.0, 240.0, 16.0, [1.0, 0.45, 0.0, 1.0], -1),
        solid(190.0, 60.0, 16.0, 16.0, [0.0, 0.8, 1.0, 1.0], 1),
    ]
}

/// The scene of `references/<tier>/glass.png`: the stripes, the panel
/// (`glass_panel`) and the overlay content below and above it
/// (`glass_overlay_content`).
pub fn glass_reference_scene() -> (RenderScene, RenderDesc) {
    let (mut scene, desc) = stripes_scene(&[glass_panel()]);
    if let Some(o) = scene.ui.overlay.as_mut() {
        o.sprites.extend(glass_overlay_content());
    }
    (scene, desc)
}

/// The colour vision frame (docs/ui.md §3.2) at `width`×`height`: five
/// swatches a dichromat confuses in pairs, over a scene colour, so the
/// filter's test sees both the overlay and the scene. Not a reference: its
/// tests (`tests/color_vision.rs`, packages/engine/tests/web/color-vision.ts)
/// compare each filtered frame with the CPU filter of the unfiltered one.
pub fn color_vision_scene(font: &FontFiles, width: u32, height: u32) -> (RenderScene, RenderDesc) {
    let doc = r#"<meta name="component" content="chips">
<row id="Root"><panel id="R"/><panel id="G"/><panel id="B"/><panel id="Y"/><panel id="M"/></row>
<style>
#Root { padding: 24px; gap: 12px; flex-wrap: wrap; }
panel { width: 120px; height: 120px; }
#R { background: #d03028; } #G { background: #2fa84f; } #B { background: #2f6fe0; }
#Y { background: #e8c83a; } #M { background: #a03ad0; }
</style>"#;
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    ui.define_document(UiDocument::parse("ui/chips.svelte", doc).expect("chips parse"));
    let vp = Viewport::new(width as f32, height as f32);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        ui.component("T", "chips", &mut serde_json::json!({}), &bind::NoEnv);
        ui.end();
    }
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::new(0.6, 0.15, 0.05));
    scene.look.post.tonemap = Tonemap::None;
    (
        scene,
        RenderDesc {
            width,
            height,
            ..Default::default()
        },
    )
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

/// `user-select-<width>`: a drag from the wrapped paragraph's second line
/// into the next paragraph, its selection under the text, drawn by the
/// host's own og-ui (wasm in Chrome, native on Metal). Panics where the
/// drag did not select what it must.
pub fn user_select_scene(font: &FontFiles, width: u32, height: u32) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    ui.define_document(UiDocument::parse("ui/sel.svelte", USER_SELECT_DOC).expect("sel parses"));
    let vp = Viewport::new(width as f32, height as f32);
    let step = |ui: &mut Ui, p: Vec2, down: bool, pressed: bool, released: bool| {
        ui.begin(
            InputFrame {
                pointer: p,
                pointer_inside: true,
                down,
                pressed,
                released,
                ..InputFrame::default()
            },
            vp,
        );
        ui.component("T", "sel", &mut serde_json::json!({}), &bind::NoEnv);
        ui.end();
    };
    step(&mut ui, Vec2::ZERO, false, false, false);
    step(&mut ui, Vec2::ZERO, false, false, false);
    let at = |ui: &Ui, id: &str, fx: f32, fy: f32| {
        let e = ui.element(id).expect(id);
        Vec2::new(e.x + e.width * fx, e.y + e.height * fy)
    };
    let (from, to) = (at(&ui, "Wrap", 0.3, 0.3), at(&ui, "Second", 0.5, 0.5));
    step(&mut ui, from, true, true, false);
    for k in 1..=4 {
        step(
            &mut ui,
            from + (to - from) * (k as f32 / 4.0),
            true,
            false,
            false,
        );
    }
    step(&mut ui, to, false, false, true);
    let selected = ui.probe_extra()["selection"]["text"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(
        selected.contains("jumps over the lazy dog\nSecond"),
        "the drag selected {selected:?}"
    );
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.lighting.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
    (
        scene,
        RenderDesc {
            width,
            height,
            ..Default::default()
        },
    )
}

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

/// `text-path-<width>`: [`TEXT_PATH_DOC`] at `width`×`height`.
pub fn text_path_scene(font: &FontFiles, width: u32, height: u32) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    ui.define_document(UiDocument::parse("ui/tp.svelte", TEXT_PATH_DOC).expect("tp parses"));
    let vp = Viewport::new(width as f32, height as f32);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        ui.component("T", "tp", &mut serde_json::json!({}), &bind::NoEnv);
        ui.end();
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.lighting.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
    (
        scene,
        RenderDesc {
            width,
            height,
            ..Default::default()
        },
    )
}

/// The reference frames, by their file name under `references/<tier>/`.
pub const REFERENCE_SCENES: [&str; 9] = [
    "sample-screen",
    "text-glow",
    "glass",
    "widget-set",
    "text-fields-2",
    "user-select-1280",
    "user-select-390",
    "text-path-1280",
    "text-path-390",
];

/// The scene and renderer description of reference frame `name`.
pub fn reference_scene(name: &str, font: &FontFiles) -> Result<(RenderScene, RenderDesc), String> {
    Ok(match name {
        "sample-screen" => {
            let (s, _) = sample_screen_shot(font);
            (s.scene, s.desc)
        }
        "text-glow" => glow_scene(font, true, 1),
        "glass" => glass_reference_scene(),
        "widget-set" => {
            let s = widget_set_shot(font);
            (s.scene, s.desc)
        }
        "text-fields-2" => {
            let s = text_fields_shot(font);
            (s.scene, s.desc)
        }
        "color-vision-1280" => color_vision_scene(font, 1280, 720),
        "color-vision-390" => color_vision_scene(font, 390, 844),
        "user-select-1280" => user_select_scene(font, 1280, 720),
        "user-select-390" => user_select_scene(font, 390, 844),
        "text-path-1280" => text_path_scene(font, 1280, 720),
        "text-path-390" => text_path_scene(font, 390, 844),
        _ => {
            return Err(format!(
                "no og-ui reference scene {name} (it knows {})",
                REFERENCE_SCENES.join(", ")
            ));
        }
    })
}
