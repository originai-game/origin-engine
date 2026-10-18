//! The sample screen every og-ui integration test drives, the MSDF test font
//! and the reference-frame comparison (og-tests' tolerance rules).
#![allow(dead_code)]

use og_render::{ColorSpace, Glyph, Mips, MsdfFont, RenderScene, TextureData};
use og_ui::*;
use std::path::PathBuf;

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

fn fonts_dir() -> PathBuf {
    // The MSDF test atlas og-tests commits (Noto Sans SC subset, OFL).
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../og-tests/assets/fonts")
}

/// Adds the MSDF test font (msdf-atlas-gen's PNG, CSV and JSON) to a scene;
/// the UI font for it, with the font's own ascender and descender.
pub fn add_test_font(scene: &mut RenderScene) -> UiFont {
    let dir = fonts_dir();
    let png = dir.join("noto-sans-sc-msdf.png");
    let img = image::open(&png)
        .unwrap_or_else(|e| panic!("{}: {e}", png.display()))
        .to_rgba8();
    let (width, height) = img.dimensions();
    let atlas = scene.add_texture(TextureData {
        width,
        height,
        rgba: img.into_raw(),
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    });
    let json = std::fs::read_to_string(dir.join("noto-sans-sc-msdf.json")).expect("font json");
    let json: serde_json::Value = serde_json::from_str(&json).expect("font json parses");
    let number = |a: &str, b: &str| json[a][b].as_f64().unwrap_or_else(|| panic!("{a}.{b}")) as f32;
    let csv = std::fs::read_to_string(dir.join("noto-sans-sc-msdf.csv")).expect("font csv");
    let mut glyphs = Vec::new();
    for line in csv.lines().filter(|l| !l.trim().is_empty()) {
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

/// The test font's metrics with nothing to draw with (tests without a renderer).
pub fn test_font_metrics() -> UiFont {
    let mut scene = RenderScene::new();
    UiFont {
        handle: None,
        ..add_test_font(&mut scene)
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

/// og-tests' rule: a pixel is an outlier when a channel differs by more than
/// 32/255; the frame fails above 0.5 % outliers or an RGB RMSE above 0.01.
pub fn compare(actual: &[u8], reference: &[u8]) -> Result<(f64, f64), String> {
    if actual.len() != reference.len() {
        return Err(format!("size {} vs {}", actual.len(), reference.len()));
    }
    let mut sum = 0.0f64;
    let mut outliers = 0usize;
    let pixels = actual.len() / 4;
    for (a, r) in actual
        .as_chunks::<4>()
        .0
        .iter()
        .zip(reference.as_chunks::<4>().0.iter())
    {
        let mut out = false;
        for c in 0..3 {
            let d = (f64::from(a[c]) - f64::from(r[c])) / 255.0;
            sum += d * d;
            out |= a[c].abs_diff(r[c]) > 32;
        }
        outliers += usize::from(out);
    }
    let rmse = (sum / (pixels * 3) as f64).sqrt();
    let fraction = outliers as f64 / pixels as f64;
    let mut failures = Vec::new();
    if rmse > 0.01 {
        failures.push(format!("RMSE {rmse:.5} > 0.01"));
    }
    if fraction > 0.005 {
        failures.push(format!("{:.3}% outlier pixels > 0.5%", fraction * 100.0));
    }
    if failures.is_empty() {
        Ok((rmse, fraction))
    } else {
        Err(failures.join("; "))
    }
}

/// Where tests write what they produce: `$CARGO_TARGET_TMPDIR/og-ui`.
pub fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-ui");
    std::fs::create_dir_all(&dir).expect("out dir");
    dir
}

/// og-input's layer behind og-probe's `InputSink`, until og-probe adopts
/// og-input: the probe's injected events (the same protocol 5 shape) are
/// handed to `og_input::InputLayer`, so the world holds og-input's `og.Input`.
pub struct ProbeInput(pub og_input::InputLayer);

impl og_probe::InputSink for ProbeInput {
    fn attach(&mut self, world: &mut og_ecs::World) -> og_core::Result<()> {
        og_input::InputSink::attach(&mut self.0, world)
    }

    fn inject(&mut self, world: &mut og_ecs::World, event: &og_probe::InjectedInput) -> bool {
        match og_input::InjectedInput::from_json(&event.to_json()) {
            Some(e) => og_input::InputSink::inject(&mut self.0, world, &e),
            None => false,
        }
    }

    fn end_tick(&mut self, world: &mut og_ecs::World) {
        og_input::InputSink::end_tick(&mut self.0, world);
    }

    fn accepted(&self) -> u32 {
        og_input::InputSink::accepted(&self.0)
    }
}
