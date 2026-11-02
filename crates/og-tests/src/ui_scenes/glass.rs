//! Flat glass (`docs/ui.md` §5.3, og-render's `ui::glass`): the design
//! language's three weights with their hairlines and shadows over a
//! bright, a dark and a busy backdrop, a dialog over the blurred scrim with
//! flat controls on it, and the two fallbacks.
//!
//! - `ui-glass-bright`, `ui-glass-dark`, `ui-glass-busy`: the same chrome
//!   on each backdrop, the light scheme on the left half and the dark on
//!   the right: a `glass` bar with a current item, a `glass-thick` menu
//!   with a selected row, a `glass-thin` play disc and a `glass-thick` toast.
//! - `ui-glass-dialog`: a `glass-thick` dialog (lift) over the scrim,
//!   with a field and flat buttons, on the dark backdrop.
//! - `ui-glass-fallback`: the dark chrome and a dialog under reduced
//!   transparency (left: solid, the scrim not blurred) and increased
//!   contrast (right: solid with a strong edge).
//!
//! Controls on glass are flat fills, never a second glass or a border.
//! [`glass_inks`] lists every text's box and ink, so a test reads the
//! contrast the glass achieves under it.

use super::*;
use og_render::{UiElevation, UiGlassFallback, UiGlassStyle, UiGlassWeight};

/// Which glass scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlassScene {
    Bright,
    Dark,
    Busy,
    Dialog,
    Fallback,
}

impl GlassScene {
    pub const ALL: [GlassScene; 5] = [
        GlassScene::Bright,
        GlassScene::Dark,
        GlassScene::Busy,
        GlassScene::Dialog,
        GlassScene::Fallback,
    ];

    pub fn name(self) -> &'static str {
        match self {
            GlassScene::Bright => "ui-glass-bright",
            GlassScene::Dark => "ui-glass-dark",
            GlassScene::Busy => "ui-glass-busy",
            GlassScene::Dialog => "ui-glass-dialog",
            GlassScene::Fallback => "ui-glass-fallback",
        }
    }
}

/// A text on glass: its box (CSS px), its ink (sRGB bytes), what it is.
#[derive(Clone, Debug, PartialEq)]
pub struct GlassInk {
    pub label: String,
    pub rect: [f32; 4],
    pub ink: [u8; 3],
    /// What the material promises 4.5:1 for: the scheme's `fg` on the
    /// weights that carry text (`glass` and `glass-thick`, a selected row
    /// or current item on them included) or on solid glass.
    pub held: bool,
    /// It lies on `glass-thin`, which carries icons and play discs only,
    /// never text.
    pub on_thin: bool,
}

/// The design language's colours of one scheme (sRGB bytes, alpha).
#[derive(Clone, Copy)]
struct Tokens {
    fg: [u8; 3],
    muted: [u8; 3],
    fill: f32,
    fill_strong: f32,
    /// The clear control tint inside `glass` and `glass-thin`.
    control: ([u8; 3], f32),
    accent: [u8; 3],
    accent_fg: [u8; 3],
    accent_ink: [u8; 3],
}

const DARK: Tokens = Tokens {
    fg: [0xf5, 0xf5, 0xf7],
    muted: [0xae, 0xae, 0xb2],
    fill: 0.24,
    fill_strong: 0.36,
    // The platform's `material-control` is white at 13 % blended over
    // sRGB-encoded values (CSS); the UI pass blends flat fills in linear
    // light, where 13 % white over dark glass reads about three times as
    // strong, so the scene takes the value that draws as CSS's does there
    // (docs/ui.md §5.3's palette notes).
    control: ([255, 255, 255], 0.05),
    accent: [0xff, 0xb2, 0x24],
    accent_fg: [0x1d, 0x13, 0x00],
    accent_ink: [0xff, 0xb2, 0x24],
};

const LIGHT: Tokens = Tokens {
    fg: [0x18, 0x18, 0x1a],
    muted: [0x5f, 0x5f, 0x66],
    fill: 0.12,
    fill_strong: 0.2,
    control: ([120, 120, 128], 0.16),
    accent: [0xf5, 0xa3, 0x0a],
    accent_fg: [0x1d, 0x13, 0x00],
    accent_ink: [0x8f, 0x56, 0x00],
};

fn rgba(c: [u8; 3], a: f32) -> [f32; 4] {
    let mut v = rgb(c[0], c[1], c[2]);
    v[3] = a;
    v
}

/// The neutral fill (Apple's system grey) at `a`.
fn fill(a: f32) -> [f32; 4] {
    rgba([120, 120, 128], a)
}

/// What a scene draws on: the overlay, its font (none when the text is
/// left out) and the inks it records.
struct Painter<'a> {
    o: &'a mut Overlay,
    font: Option<FontHandle>,
    inks: Vec<GlassInk>,
    /// The ink held to 4.5:1 where the next texts are ([`GlassInk::held`]).
    held: Option<[u8; 3]>,
    /// The next texts lie on `glass-thin`.
    on_thin: bool,
}

/// Layers: the backdrop 0, the scrim 1, glass 2, controls on it 3, text 4
/// (a dialog's 5 to 7).
impl Painter<'_> {
    fn glass(&mut self, r: [f32; 4], radius: f32, style: UiGlassStyle, layer: i32) {
        let (mode, tint) = style.backdrop();
        self.o.ui.shapes.push(UiShape {
            mode,
            paint: solid(tint),
            layer,
            ..UiShape::rect(r, radius, [1.0; 4])
        });
    }

    fn flat(&mut self, r: [f32; 4], radius: f32, color: [f32; 4], layer: i32) {
        self.o.ui.shapes.push(UiShape {
            layer,
            ..UiShape::rect(r, radius, color)
        });
    }

    /// `s` at `size` px with its baseline at `y`; its box for the contrast
    /// is the em box over the baseline, `w` wide.
    #[allow(clippy::too_many_arguments)]
    fn text(&mut self, s: &str, x: f32, y: f32, size: f32, w: f32, ink: [u8; 3], layer: i32) {
        self.inks.push(GlassInk {
            label: s.to_string(),
            rect: [x, y - 0.8 * size, w, 0.95 * size],
            ink,
            held: self.held == Some(ink),
            on_thin: self.on_thin,
        });
        if let Some(font) = self.font {
            self.o.texts.push(Text {
                position: Vec3::new(x, y, 0.0),
                size,
                color: rgba(ink, 1.0),
                layer,
                ..Text::new(font, s)
            });
        }
    }

    /// The chrome of one scheme in the half from `x0` (640 CSS px wide).
    fn chrome(&mut self, x0: f32, dark: bool, fallback: UiGlassFallback) {
        let t = if dark { DARK } else { LIGHT };
        let style = |weight, elevation| UiGlassStyle {
            weight,
            elevation,
            dark,
            fallback,
        };
        let solid = fallback != UiGlassFallback::None;
        let thick = Some(t.fg);
        self.held = thick;
        // A `glass` bar; its current item a flat clear tint, concentric
        // (inset 6: radius 12 - 6).
        let bar = [x0 + 24.0, 24.0, 592.0, 52.0];
        self.glass(
            bar,
            12.0,
            style(UiGlassWeight::Regular, UiElevation::Raised),
            2,
        );
        let current = if solid {
            fill(t.fill_strong)
        } else {
            rgba(t.control.0, t.control.1)
        };
        // The current item carries the scheme's fg (an amber label on it is
        // a palette pair the ui-scale stream fixes, docs/ui.md §5.3); the
        // amber one is a neighbour.
        self.flat([bar[0] + 6.0, bar[1] + 6.0, 96.0, 40.0], 6.0, current, 3);
        for (i, label) in ["Play", "Library", "Store", "Friends"].iter().enumerate() {
            let ink = if i == 1 { t.accent_ink } else { t.fg };
            let x = bar[0] + 24.0 + i as f32 * 104.0;
            self.text(label, x, bar[1] + 32.0, 17.0, 64.0, ink, 4);
        }
        // A `glass-thick` menu (float): 12 outer, rows of 8 inside a 4
        // inset, the selected one on fill-strong.
        let rows = ["New game", "Continue", "Load save", "Settings", "Quit"];
        let menu = [x0 + 24.0, 104.0, 260.0, 8.0 + 40.0 * rows.len() as f32];
        self.held = thick;
        self.glass(
            menu,
            12.0,
            style(UiGlassWeight::Thick, UiElevation::Float),
            2,
        );
        for (i, label) in rows.iter().enumerate() {
            let y = menu[1] + 4.0 + 40.0 * i as f32;
            if i == 1 {
                self.flat(
                    [menu[0] + 4.0, y, menu[2] - 8.0, 40.0],
                    8.0,
                    fill(t.fill_strong),
                    3,
                );
            }
            self.text(label, menu[0] + 18.0, y + 26.0, 17.0, 120.0, t.fg, 4);
            let key = ["N", "C", "L", ",", "Q"][i];
            self.text(key, menu[0] + 222.0, y + 26.0, 15.0, 12.0, t.muted, 4);
        }
        // A `glass-thin` play disc (float) over art: an icon, never text.
        let disc = [x0 + 320.0, 108.0, 52.0, 52.0];
        self.glass(
            disc,
            26.0,
            style(UiGlassWeight::Thin, UiElevation::Float),
            2,
        );
        self.o.ui.shapes.push(UiShape {
            geometry: UiGeometry::Polygon {
                sides: 3,
                inner: 1.0,
                rounding: 1.5,
                rotation: std::f32::consts::FRAC_PI_2,
            },
            layer: 3,
            ..UiShape::rect(
                [disc[0] + 15.0, disc[1] + 14.0, 24.0, 24.0],
                0.0,
                rgba(t.fg, 1.0),
            )
        });
        // A `glass-thick` toast: a flat dot, its line and a quiet action.
        let toast = [x0 + 300.0, 560.0, 316.0, 56.0];
        self.held = thick;
        self.glass(
            toast,
            6.0,
            style(UiGlassWeight::Thick, UiElevation::Raised),
            2,
        );
        self.o.ui.shapes.push(UiShape {
            geometry: UiGeometry::Ellipse,
            layer: 3,
            ..UiShape::rect(
                [toast[0] + 18.0, toast[1] + 22.0, 12.0, 12.0],
                0.0,
                rgba([0x30, 0xd1, 0x58], 1.0),
            )
        });
        self.text(
            "Game saved",
            toast[0] + 44.0,
            toast[1] + 34.0,
            17.0,
            110.0,
            t.fg,
            4,
        );
        self.text(
            "Undo",
            toast[0] + 250.0,
            toast[1] + 34.0,
            17.0,
            40.0,
            t.accent_ink,
            4,
        );
    }

    /// A `glass-thick` dialog (lift) centred on `cx`, over the scrim.
    fn dialog(&mut self, cx: f32, dark: bool, fallback: UiGlassFallback) {
        let t = if dark { DARK } else { LIGHT };
        let d = [cx - 220.0, 190.0, 440.0, 300.0];
        self.held = Some(t.fg);
        self.glass(
            d,
            12.0,
            UiGlassStyle {
                weight: UiGlassWeight::Thick,
                elevation: UiElevation::Lift,
                dark,
                fallback,
            },
            5,
        );
        self.text(
            "Rename save",
            d[0] + 28.0,
            d[1] + 48.0,
            24.0,
            160.0,
            t.fg,
            7,
        );
        self.text(
            "The old name stays in the history.",
            d[0] + 28.0,
            d[1] + 84.0,
            17.0,
            280.0,
            t.muted,
            7,
        );
        // A field: a soft neutral fill, no border.
        let field = [d[0] + 28.0, d[1] + 112.0, d[2] - 56.0, 44.0];
        self.flat(field, 6.0, fill(t.fill), 6);
        self.text(
            "Harbour, day 12",
            field[0] + 14.0,
            field[1] + 28.0,
            17.0,
            140.0,
            t.fg,
            7,
        );
        // A segmented control: the fill track and the thumb.
        let seg = [d[0] + 28.0, d[1] + 172.0, 240.0, 36.0];
        self.flat(seg, 6.0, fill(t.fill), 6);
        let thumb = if dark {
            rgba([0x63, 0x63, 0x66], 1.0)
        } else {
            [1.0; 4]
        };
        self.flat([seg[0] + 2.0, seg[1] + 2.0, 78.0, 32.0], 5.0, thumb, 6);
        for (i, label) in ["Slot 1", "Slot 2", "Slot 3"].iter().enumerate() {
            let x = seg[0] + 16.0 + 80.0 * i as f32;
            self.text(label, x, seg[1] + 24.0, 15.0, 48.0, t.fg, 7);
        }
        // Buttons: a secondary neutral fill and the one flat amber primary.
        let by = d[1] + d[3] - 64.0;
        let cancel = [d[0] + d[2] - 248.0, by, 104.0, 40.0];
        self.flat(cancel, 6.0, fill(t.fill), 6);
        self.text("Cancel", cancel[0] + 25.0, by + 26.0, 17.0, 56.0, t.fg, 7);
        let save = [d[0] + d[2] - 132.0, by, 104.0, 40.0];
        self.flat(save, 6.0, rgba(t.accent, 1.0), 6);
        self.text(
            "Rename",
            save[0] + 22.0,
            by + 26.0,
            17.0,
            60.0,
            t.accent_fg,
            7,
        );
    }

    /// The scrim over everything under the dialog: it dims and blurs.
    fn scrim(&mut self, dark: bool, fallback: UiGlassFallback, r: [f32; 4]) {
        let (mode, tint) = og_render::ui_scrim(dark, fallback);
        self.o.ui.shapes.push(UiShape {
            mode,
            paint: solid(tint),
            layer: 1,
            ..UiShape::rect(r, 0.0, [1.0; 4])
        });
    }
}

/// A sunlit sky over pale hills: bright almost everywhere.
fn bright_backdrop(o: &mut Overlay) {
    let ground = UiPaint::Gradient(gradient(
        UiGradientKind::Linear,
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        &[
            (0.0, rgb(150, 205, 250)),
            (0.55, rgb(232, 244, 255)),
            (1.0, rgb(255, 246, 214)),
        ],
    ));
    o.ui.shapes.push(shape(
        rect([0.0; 4], UiCorner::Round),
        [0.0, 0.0, W, H],
        ground,
    ));
    let glow = |c: [f32; 4]| {
        UiPaint::Gradient(gradient(
            UiGradientKind::Radial,
            Vec2::new(0.5, 0.5),
            Vec2::new(0.5, 0.5),
            &[(0.0, c), (1.0, [c[0], c[1], c[2], 0.0])],
        ))
    };
    o.ui.shapes.push(shape(
        UiGeometry::Ellipse,
        [880.0, -120.0, 420.0, 420.0],
        glow(rgb(255, 250, 225)),
    ));
    for (x, y, w, h) in [
        (40.0, 220.0, 420.0, 140.0),
        (520.0, 320.0, 520.0, 160.0),
        (300.0, 60.0, 300.0, 90.0),
    ] {
        o.ui.shapes
            .push(shape(UiGeometry::Ellipse, [x, y, w, h], glow([1.0; 4])));
    }
    for (x, y, w, h, c) in [
        (-200.0, 520.0, 900.0, 420.0, rgb(196, 226, 170)),
        (520.0, 560.0, 1000.0, 400.0, rgb(214, 234, 186)),
    ] {
        o.ui.shapes
            .push(shape(UiGeometry::Ellipse, [x, y, w, h], solid(c)));
    }
}

/// A night forest: deep blue, dark trunks and a few warm lights (the
/// owner's concept, drawn).
fn dark_backdrop(o: &mut Overlay) {
    let ground = UiPaint::Gradient(gradient(
        UiGradientKind::Linear,
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        &[
            (0.0, rgb(18, 32, 66)),
            (0.6, rgb(14, 22, 44)),
            (1.0, rgb(28, 22, 30)),
        ],
    ));
    o.ui.shapes.push(shape(
        rect([0.0; 4], UiCorner::Round),
        [0.0, 0.0, W, H],
        ground,
    ));
    for i in 0..18 {
        let x = -40.0 + i as f32 * 76.0 + (i * 37 % 23) as f32;
        let h = 380.0 + (i * 53 % 7) as f32 * 40.0;
        // A tall fir: a triangle stretched upwards.
        o.ui.shapes.push(UiShape {
            transform: [1.0, 0.0, 0.0, h / 90.0],
            ..shape(
                UiGeometry::Polygon {
                    sides: 3,
                    inner: 1.0,
                    rounding: 2.0,
                    rotation: 0.0,
                },
                [x, H - 0.5 * h - 60.0, 120.0, 120.0],
                solid(rgb(8, 14, 26)),
            )
        });
    }
    let lights = [
        (180.0, 420.0, 90.0),
        (420.0, 560.0, 140.0),
        (700.0, 300.0, 70.0),
        (930.0, 480.0, 160.0),
        (1160.0, 260.0, 110.0),
        (90.0, 140.0, 60.0),
        (600.0, 650.0, 120.0),
    ];
    for (x, y, r) in lights {
        o.ui.shapes.push(shape(
            UiGeometry::Ellipse,
            [x - r, y - r, 2.0 * r, 2.0 * r],
            UiPaint::Gradient(gradient(
                UiGradientKind::Radial,
                Vec2::new(0.5, 0.5),
                Vec2::new(0.5, 0.5),
                &[
                    (0.0, rgb(255, 196, 92)),
                    (0.25, rgba([255, 140, 40], 0.7)),
                    (1.0, rgba([255, 120, 30], 0.0)),
                ],
            )),
        ));
    }
}

/// Everything at once: saturated stripes, white and black bars, discs and
/// big type, the worst a glass can sit on.
fn busy_backdrop(o: &mut Overlay, font: Option<FontHandle>) {
    let hues = [
        rgb(255, 59, 48),
        rgb(255, 204, 0),
        rgb(52, 199, 89),
        rgb(0, 122, 255),
        rgb(175, 82, 222),
        rgb(255, 255, 255),
        rgb(10, 10, 12),
    ];
    for i in 0..40 {
        let c = hues[i % hues.len()];
        o.ui.shapes.push(UiShape {
            transform: [0.8, 0.6, -0.6, 0.8],
            ..UiShape::rect([-400.0 + 52.0 * i as f32, -200.0, 26.0, 1200.0], 0.0, c)
        });
    }
    for (i, (x, y, r)) in [
        (200.0, 300.0, 80.0),
        (520.0, 180.0, 60.0),
        (760.0, 420.0, 100.0),
        (1060.0, 220.0, 70.0),
        (320.0, 600.0, 90.0),
        (1100.0, 600.0, 80.0),
    ]
    .into_iter()
    .enumerate()
    {
        let c = hues[(i * 3 + 1) % hues.len()];
        o.ui.shapes.push(shape(
            UiGeometry::Ellipse,
            [x - r, y - r, 2.0 * r, 2.0 * r],
            solid(c),
        ));
    }
    if let Some(font) = font {
        for (i, s) in ["ORIGIN", "原点 GAME", "ARCADE 88"].iter().enumerate() {
            o.texts.push(Text {
                position: Vec3::new(40.0 + 300.0 * i as f32, 280.0 + 190.0 * i as f32, 0.0),
                size: 110.0,
                color: if i == 1 {
                    [0.0, 0.0, 0.0, 1.0]
                } else {
                    [1.0; 4]
                },
                layer: 0,
                ..Text::new(font, *s)
            });
        }
    }
}

/// A glass scene: its render scene and the texts on glass. `font` is
/// called unless `text` is false (the frame the contrast is read from).
pub fn glass_scene(
    which: GlassScene,
    font: impl FnOnce(&mut RenderScene) -> Result<FontHandle, String>,
    text: bool,
) -> Result<(RenderScene, Vec<GlassInk>), String> {
    let mut err = None;
    let mut inks = Vec::new();
    let scene = overlay_scene(|scene, o| {
        let font = if text {
            match font(scene) {
                Ok(f) => Some(f),
                Err(e) => {
                    err = Some(e);
                    return;
                }
            }
        } else {
            None
        };
        match which {
            GlassScene::Bright => bright_backdrop(o),
            GlassScene::Dark | GlassScene::Dialog | GlassScene::Fallback => dark_backdrop(o),
            GlassScene::Busy => busy_backdrop(o, font),
        }
        let mut p = Painter {
            o,
            font,
            inks: Vec::new(),
            held: None,
            on_thin: false,
        };
        let none = UiGlassFallback::None;
        match which {
            GlassScene::Bright | GlassScene::Dark | GlassScene::Busy => {
                p.chrome(0.0, false, none);
                p.chrome(640.0, true, none);
            }
            GlassScene::Dialog => {
                p.chrome(0.0, true, none);
                p.scrim(true, none, [0.0, 0.0, W, H]);
                p.dialog(640.0, true, none);
            }
            GlassScene::Fallback => {
                let rt = UiGlassFallback::ReducedTransparency;
                let hc = UiGlassFallback::IncreasedContrast;
                p.chrome(0.0, true, rt);
                p.chrome(640.0, true, hc);
                p.scrim(true, rt, [0.0, 400.0, 640.0, 320.0]);
                p.scrim(true, hc, [640.0, 400.0, 640.0, 320.0]);
            }
        }
        inks = p.inks;
    });
    match err {
        Some(e) => Err(e),
        None => Ok((scene, inks)),
    }
}
