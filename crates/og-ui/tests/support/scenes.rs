//! The scenes og-ui's reference frames are drawn from (`references/<tier>/`
//! sample-screen, text-glow, glass and widget-set), as scenes ready to
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
            background: Some(t.palette.bg),
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
                            color: Some(t.palette.fg_muted),
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
                    if ui.toggle("Music", &mut s.music, "Music").clicked {
                        s.clicks.push("Music".into());
                    }
                    ui.slider("Volume", &mut s.volume, SliderDesc::default());
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
                                    color: Some(t.palette.fg_muted),
                                    width: Size::grow(),
                                    ..LabelDesc::default()
                                },
                            );
                            ui.label(
                                "Coins",
                                "1250 coins",
                                LabelDesc {
                                    step: Some(t.type_ramp.meta),
                                    color: Some(t.palette.accent_ink),
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
                                        background: Some(t.palette.fill),
                                        radius: t.radius.block,
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
            codepoint: char::from_u32(f[0] as u32).expect("codepoint"),
            advance: f[1],
            plane_bounds: [f[2], -f[5], f[4], -f[3]],
            atlas_bounds: [f[6], f[7], f[8], f[9]],
        });
    }
    let font = MsdfFont {
        atlas,
        distance_range: number("atlas", "distanceRange"),
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
pub fn shot(ui: Ui, mut scene: RenderScene, atlas: og_render::TextureHandle, vp: Viewport) -> Shot {
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    ui.frame().emit(&EmitDesc { atlas, layer: 0 }, &mut overlay);
    scene.overlay = Some(overlay);
    let bg = ui.theme().palette.bg.linear();
    scene.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.post.tonemap = Tonemap::None;
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
            .frame()
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
    let atlas = scene.add_texture(atlas_texture());
    let mut ui = ui_with_font(font);
    let mut s = Sample::default();
    sample_session(&mut ui, &mut s);
    (shot(ui, scene, atlas, vp), s)
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
            background: Some(t.palette.bg),
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
                    ui.tabs(
                        "Tabs",
                        &["Overview", "Stats", "Gear"],
                        &mut s.tab,
                        TabsDesc {
                            fill: true,
                            ..TabsDesc::default()
                        },
                    );
                    ui.row(
                        "Pickers",
                        Element {
                            gap: t.space.md,
                            cross: Cross::Center,
                            ..Element::default()
                        },
                        |ui| {
                            ui.select(
                                "Difficulty",
                                &["Easy", "Normal", "Hard"],
                                &mut s.pick,
                                &SelectDesc::default(),
                            );
                            ui.stepper(
                                "Amount",
                                &mut s.amount,
                                StepperDesc {
                                    min: 0.0,
                                    max: 9.0,
                                    ..StepperDesc::default()
                                },
                            );
                        },
                    );
                    ui.radio(
                        "Mode",
                        &["Solo", "Co-op", "Versus"],
                        &mut s.mode,
                        RadioDesc { horizontal: true },
                    );
                    ui.checkbox_list("Options", &["Show hints", "Hard mode"], &mut s.checks);
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
                            ui.badge("New", "New", Tone::Accent);
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
    let atlas = scene.add_texture(atlas_texture());
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
    let sel = ui.frame().element("Difficulty").expect("select").clone();
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
    shot(ui, scene, atlas, vp)
}

/// A dark card of display-size labels, with or without a skin's text
/// glow, as a scene ready to render.
pub fn glow_scene(font: &FontFiles, glow: bool, labels: usize) -> (RenderScene, RenderDesc) {
    let vp = Viewport {
        dpr: 1.0,
        ..Viewport::new(480.0, 200.0f32.max(16.0 + 40.0 * labels as f32))
    };
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let atlas = scene.add_texture(atlas_texture());
    let mut ui = ui_with_font(font);
    let mut theme = *ui.theme();
    // Tight tracking: glyph quads overlap their neighbours.
    theme.type_ramp.display_md.tracking_em = -0.1;
    if glow {
        theme.skin.text_glows[0] = Some(og_ui::skin::Glow {
            size: 2.0,
            ink: og_ui::skin::Ink::Accent,
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
                background: Some(t.palette.bg),
                ..Element::default()
            },
            |ui| {
                for k in 0..labels {
                    ui.label(
                        ui_idi("Title", k as u32),
                        "Wall 42",
                        LabelDesc {
                            step: Some(t.type_ramp.display_md),
                            ..LabelDesc::default()
                        },
                    );
                }
            },
        );
        ui.end();
    }
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    ui.frame().emit(&EmitDesc { atlas, layer: 0 }, &mut overlay);
    scene.overlay = Some(overlay);
    let bg = ui.theme().palette.bg.linear();
    scene.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.post.tonemap = Tonemap::None;
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
        scene.sprites.push(og_render::Sprite {
            position: Vec3::new(k as f32 * 8.0 + 2.0, h as f32 / 2.0, 0.0),
            size: Vec2::new(4.0, h as f32),
            color: [1.0, 1.0, 1.0, 1.0],
            ..og_render::Sprite::default()
        });
    }
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    let mut overlay = og_render::Overlay::new(w as f32, h as f32, 1.0);
    overlay.glass.extend_from_slice(glass);
    scene.overlay = Some(overlay);
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

/// The reference frames, by their file name under `references/<tier>/`.
pub const REFERENCE_SCENES: [&str; 4] = ["sample-screen", "text-glow", "glass", "widget-set"];

/// The scene and renderer description of reference frame `name`.
pub fn reference_scene(name: &str, font: &FontFiles) -> Result<(RenderScene, RenderDesc), String> {
    Ok(match name {
        "sample-screen" => {
            let (s, _) = sample_screen_shot(font);
            (s.scene, s.desc)
        }
        "text-glow" => glow_scene(font, true, 1),
        "glass" => stripes_scene(&[glass_panel()]),
        "widget-set" => {
            let s = widget_set_shot(font);
            (s.scene, s.desc)
        }
        _ => {
            return Err(format!(
                "no og-ui reference scene {name} (it knows {})",
                REFERENCE_SCENES.join(", ")
            ));
        }
    })
}
