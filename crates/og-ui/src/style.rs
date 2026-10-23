//! The theme: the values a widget reads its look from (the prototype style's
//! in [`crate::theme`], a style file's over them) with the few conversions
//! the renderer needs.

use crate::theme as tokens;
pub use crate::theme::{
    Density, Durations, Palette, Radius, Space, Spring, Srgba, TypeRamp, TypeStep,
};

/// The contrast a widget that picks its own ink holds it to on what it is
/// drawn on (a badge, a meter's label): WCAG's 4.5:1 for body text.
pub const READABLE: f32 = 4.5;

impl Srgba {
    /// Linear RGBA (straight alpha), what og-render's sprites and text take.
    pub fn linear(self) -> [f32; 4] {
        let c = |v: u8| {
            let s = f32::from(v) / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        [c(self.rgb[0]), c(self.rgb[1]), c(self.rgb[2]), self.alpha]
    }

    /// The same colour with its alpha multiplied by `k`.
    pub fn fade(self, k: f32) -> Srgba {
        Srgba {
            rgb: self.rgb,
            alpha: self.alpha * k,
        }
    }

    /// An opaque colour from 8-bit sRGB channels (for game content; theme
    /// colours come from the palette).
    pub const fn rgb(r: u8, g: u8, b: u8) -> Srgba {
        Srgba {
            rgb: [r, g, b],
            alpha: 1.0,
        }
    }

    /// This colour composited over an opaque `below` (straight alpha), opaque.
    pub fn over(self, below: Srgba) -> Srgba {
        let a = self.alpha.clamp(0.0, 1.0);
        let mix = |t: u8, b: u8| (f32::from(t) * a + f32::from(b) * (1.0 - a)).round() as u8;
        Srgba::rgb(
            mix(self.rgb[0], below.rgb[0]),
            mix(self.rgb[1], below.rgb[1]),
            mix(self.rgb[2], below.rgb[2]),
        )
    }

    /// This colour composited over an opaque `below` in linear light, as
    /// og-render blends it, opaque.
    pub fn over_linear(self, below: Srgba) -> Srgba {
        let (t, b) = (self.linear(), below.linear());
        let a = self.alpha.clamp(0.0, 1.0);
        let e = |k: usize| {
            let v = t[k] * a + b[k] * (1.0 - a);
            let s = if v <= 0.003_130_8 {
                v * 12.92
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            };
            (s.clamp(0.0, 1.0) * 255.0).round() as u8
        };
        Srgba::rgb(e(0), e(1), e(2))
    }

    /// This colour at alpha `k` over `below` as a browser composites it (in
    /// sRGB, a CSS `rgb(… / 16%)` wash), kept translucent for og-render,
    /// which composites in linear light: the alpha whose linear blend over
    /// `below` has the browser's luminance, so the art under it still
    /// shows. A linear 16 % of a light ink over a dark ground lifts the
    /// ground to a sixth of the ink's luminance (no ink then reads 4.5:1 on
    /// it, white included, on the darker packs); the browser's 16 % lifts
    /// it a few percent. Over a light ground it darkens more than linear.
    pub fn tint_over(self, below: Srgba, k: f32) -> Srgba {
        let (lc, lb) = (self.luminance(), below.luminance());
        let mixed = self.fade(k).over(below).luminance();
        if (lc - lb).abs() < 1e-4 {
            return self.fade(k);
        }
        self.fade(((mixed - lb) / (lc - lb)).clamp(0.0, 1.0) / self.alpha.max(1e-4))
    }

    /// Mixed towards `to` by `t` (0–1), keeping this colour's alpha.
    pub fn mix(self, to: Srgba, t: f32) -> Srgba {
        let t = t.clamp(0.0, 1.0);
        let m = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
        Srgba {
            rgb: [
                m(self.rgb[0], to.rgb[0]),
                m(self.rgb[1], to.rgb[1]),
                m(self.rgb[2], to.rgb[2]),
            ],
            alpha: self.alpha,
        }
    }

    /// This ink, or the least mix of it towards its own end of the scale
    /// (white for an ink lighter than `ground`, black for a darker one) that
    /// reads `bar`:1 on the opaque `ground`; `None` when even that end does
    /// not. Value before hue: the ink lightens or darkens, keeping its hue
    /// as far as the bar allows. Called with the two swapped, it moves the
    /// ground away from the ink instead.
    pub fn readable_on(self, ground: Srgba, bar: f32) -> Option<Srgba> {
        if self.contrast(ground) >= bar {
            return Some(self);
        }
        let end = if self.luminance() >= ground.luminance() {
            Srgba::WHITE
        } else {
            Srgba::BLACK
        };
        if end.contrast(ground) < bar {
            return None;
        }
        // The contrast grows with the mix: the least one at the bar.
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..16 {
            let mid = (lo + hi) / 2.0;
            if self.mix(end, mid).contrast(ground) >= bar {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        // Rounded to bytes, a step may still sit just under: step on.
        let mut t = hi;
        let mut c = self.mix(end, t);
        while c.contrast(ground) < bar && t < 1.0 {
            t = (t + 1.0 / 255.0).min(1.0);
            c = self.mix(end, t);
        }
        Some(c)
    }

    /// The WCAG contrast ratio of this colour and `other` (1 to 21).
    pub fn contrast(self, other: Srgba) -> f32 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// WCAG relative luminance (0 black – 1 white).
    pub fn luminance(self) -> f32 {
        let c = self.linear();
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    }

    pub const WHITE: Srgba = Srgba::rgb(255, 255, 255);
    pub const BLACK: Srgba = Srgba::rgb(0, 0, 0);
    pub const TRANSPARENT: Srgba = Srgba {
        rgb: [0, 0, 0],
        alpha: 0.0,
    };
}

/// Light or dark: which variant of the prototype style the theme starts
/// from (a style file sets every colour itself).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scheme {
    Light,
    /// Dark is the default.
    #[default]
    Dark,
}

/// Which control sizes apply: by the input the player holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DensityMode {
    /// Touch on a coarse pointer or a surface under 768 CSS pixels, mouse otherwise.
    #[default]
    Auto,
    /// A mouse and keyboard.
    Mouse,
    /// A gamepad across a room.
    Pad,
    /// A finger.
    Touch,
}

impl DensityMode {
    pub fn name(self) -> &'static str {
        match self {
            DensityMode::Auto => "auto",
            DensityMode::Mouse => "mouse",
            DensityMode::Pad => "pad",
            DensityMode::Touch => "touch",
        }
    }

    pub fn from_name(name: &str) -> Option<DensityMode> {
        Some(match name {
            "auto" => DensityMode::Auto,
            "mouse" => DensityMode::Mouse,
            "pad" => DensityMode::Pad,
            "touch" => DensityMode::Touch,
            _ => return None,
        })
    }
}

/// A theme variant beyond the scheme (`prefers-contrast: more`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Contrast {
    #[default]
    Standard,
    /// Stronger inks, fills and lines, every control outlined in
    /// `lineStrong`: the style's palette remapped, no new colour.
    High,
}

/// The play surface's size class, for layouts that change shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breakpoint {
    /// The shorter side under 600 CSS px, taller than wide.
    PhonePortrait,
    /// The shorter side under 600 CSS px, wider than tall.
    PhoneLandscape,
    /// The shorter side 600 or more, under 1024 wide.
    Tablet,
    /// 1024 wide or more (with the shorter side 600 or more).
    #[default]
    Desktop,
}

impl Breakpoint {
    pub fn of(width: f32, height: f32) -> Breakpoint {
        let short = width.min(height);
        if short < 600.0 {
            if height >= width {
                Breakpoint::PhonePortrait
            } else {
                Breakpoint::PhoneLandscape
            }
        } else if width < 1024.0 {
            Breakpoint::Tablet
        } else {
            Breakpoint::Desktop
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Breakpoint::PhonePortrait => "phonePortrait",
            Breakpoint::PhoneLandscape => "phoneLandscape",
            Breakpoint::Tablet => "tablet",
            Breakpoint::Desktop => "desktop",
        }
    }

    /// A phone in either orientation.
    pub fn is_phone(self) -> bool {
        matches!(self, Breakpoint::PhonePortrait | Breakpoint::PhoneLandscape)
    }
}

/// Everything a widget reads its look from: a style's values and its art.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub scheme: Scheme,
    pub palette: Palette,
    pub space: Space,
    pub radius: Radius,
    pub type_ramp: TypeRamp,
    pub density: DensityMode,
    pub durations_ms: Durations,
    /// How far a pressed control shrinks.
    pub press_scale: f32,
    /// Standard or high contrast.
    pub contrast: Contrast,
    /// Controls draw a `lineStrong` edge (high contrast).
    pub outline: bool,
    /// `prefers-reduced-motion`: transitions keep opacity and drop movement and scale; skeleton sheens stop.
    pub reduced_motion: bool,
    /// `prefers-reduced-transparency`: glass parts are opaque surfaces (no
    /// blur of the frame under them, their frame at full opacity), and the
    /// palette's scrim and translucent fills are flat
    /// ([`Theme::to_reduced_transparency`]). A game reads it to flatten
    /// its own (IDL `UiTheme.reducedTransparency`).
    pub reduced_transparency: bool,
    /// The style's art (`crate::skin`: frames, ornaments, fonts, motion,
    /// sounds); the default dresses nothing.
    pub skin: crate::skin::Skin,
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::dark()
    }
}

impl Theme {
    /// The prototype style, light.
    pub fn light() -> Theme {
        Theme::new(Scheme::Light)
    }

    /// The prototype style, dark (the default).
    pub fn dark() -> Theme {
        Theme::new(Scheme::Dark)
    }

    pub fn new(scheme: Scheme) -> Theme {
        Theme {
            scheme,
            palette: match scheme {
                Scheme::Light => tokens::LIGHT,
                Scheme::Dark => tokens::DARK,
            },
            space: tokens::SPACE,
            radius: tokens::RADIUS,
            type_ramp: tokens::TYPE_RAMP,
            density: DensityMode::Auto,
            durations_ms: tokens::DURATIONS_MS,
            press_scale: tokens::PRESS_SCALE,
            contrast: Contrast::Standard,
            outline: false,
            reduced_motion: false,
            reduced_transparency: false,
            skin: crate::skin::Skin::default(),
        }
    }

    /// The high-contrast variant of a scheme: `inkMuted` takes `ink`,
    /// `inkFaint` takes `inkMuted`, fills step up to `controlStrong` and
    /// `lineStrong`, lines to `lineStrong`, and controls are outlined. Only
    /// the palette's own values are used.
    pub fn high_contrast(scheme: Scheme) -> Theme {
        Theme::new(scheme).to_high_contrast()
    }

    /// This theme's high-contrast variant (see [`Theme::high_contrast`]): a
    /// style's palette is remapped the same way.
    pub fn to_high_contrast(self) -> Theme {
        let mut t = self;
        let p = t.palette;
        t.palette.ink_muted = p.ink;
        t.palette.ink_faint = p.ink_muted;
        t.palette.control = p.control_strong;
        t.palette.control_strong = p.line_strong;
        t.palette.line = p.line_strong;
        t.palette.scrim = p.scrim.fade(1.5);
        t.contrast = Contrast::High;
        t.outline = true;
        t
    }

    /// This theme with nothing to see through (`prefers-reduced-
    /// transparency`): the translucent fills (`control`, `controlStrong`,
    /// `primarySoft`, `hud`) become the opaque colour they make over
    /// `panel`, the `scrim` the opaque colour it makes over `backdrop` (a
    /// dialog's backdrop hides the game rather than dimming it), and
    /// `reduced_transparency` is on (glass is an opaque surface).
    /// Highlights drawn over a surface (sheens, glows) stay as they are.
    pub fn to_reduced_transparency(self) -> Theme {
        let mut t = self;
        let p = &mut t.palette;
        let ground = p.panel;
        for c in [
            &mut p.control,
            &mut p.control_strong,
            &mut p.primary_soft,
            &mut p.hud,
        ] {
            if c.alpha < 1.0 {
                *c = c.over(ground);
            }
        }
        if p.scrim.alpha < 1.0 {
            p.scrim = p.scrim.over(p.backdrop);
        }
        t.reduced_transparency = true;
        t
    }

    /// This theme with a game's own primary colour: its hover and active
    /// tones 15 % towards either end of the scale (a dark primary lightens
    /// hovered and darkens pressed, a light one the other way),
    /// its soft wash at the palette's alpha, and the ink on it whichever of
    /// the palette's `ink` and `inkInverse` reads better. `highlight` and
    /// `focus` are the style's own and stay.
    pub fn with_primary(mut self, primary: Srgba) -> Theme {
        let a = Srgba {
            alpha: 1.0,
            ..primary
        };
        // Hover moves away from the nearer end of the scale, a press towards it.
        let (lift, sink) = if a.luminance() > 0.4 {
            (Srgba::BLACK, Srgba::WHITE)
        } else {
            (Srgba::WHITE, Srgba::BLACK)
        };
        let p = &mut self.palette;
        p.primary = a;
        p.primary_hover = a.mix(lift, 0.15);
        p.primary_active = a.mix(sink, 0.15);
        p.primary_soft = Srgba {
            alpha: p.primary_soft.alpha,
            ..a
        };
        p.on_primary = if a.contrast(p.ink) >= a.contrast(p.ink_inverse) {
            p.ink
        } else {
            p.ink_inverse
        };
        self
    }

    /// Frames a duration in milliseconds takes at `ticks_per_second` (at least 1).
    pub fn ticks(ms: f32, ticks_per_second: u32) -> u32 {
        ((ms * ticks_per_second as f32 / 1000.0).ceil() as u32).max(1)
    }

    /// The control sizes for a viewport.
    pub fn sizes(&self, viewport: &crate::Viewport) -> Density {
        match self.density {
            DensityMode::Mouse => tokens::MOUSE,
            DensityMode::Pad => tokens::PAD,
            DensityMode::Touch => tokens::TOUCH,
            DensityMode::Auto => {
                if viewport.coarse_pointer || viewport.width < 768.0 {
                    tokens::TOUCH
                } else {
                    tokens::MOUSE
                }
            }
        }
    }
}
