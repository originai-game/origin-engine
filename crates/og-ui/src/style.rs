//! The theme: the platform's design tokens (generated into [`crate::theme`])
//! with the few conversions the renderer needs. No value is defined here.

use crate::theme as tokens;
pub use crate::theme::{
    Density, Durations, Hues, Palette, Radius, Space, Spring, Srgba, TypeRamp, TypeStep,
};

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

/// Light or dark: which palette the theme uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scheme {
    Light,
    /// Games run on a stage; dark is the default.
    #[default]
    Dark,
}

/// Which control sizes apply (`docs/design-language.md` §Density and type).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DensityMode {
    /// Touch on a coarse pointer or a viewport under 768 CSS pixels, compact otherwise.
    #[default]
    Auto,
    Compact,
    Comfortable,
    Touch,
}

/// A theme variant beyond the scheme (`prefers-contrast: more`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Contrast {
    #[default]
    Standard,
    /// Stronger inks, fills and edges, every control outlined in
    /// `border-strong`: the token palette remapped, no new colour.
    High,
}

/// A named category hue of the tokens (`hue.*`), for a game's accent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HueName {
    Red,
    Orange,
    Yellow,
    Green,
    Mint,
    Teal,
    Cyan,
    Blue,
    Indigo,
    Purple,
    Pink,
    Brown,
}

impl HueName {
    pub const ALL: [HueName; 12] = [
        HueName::Red,
        HueName::Orange,
        HueName::Yellow,
        HueName::Green,
        HueName::Mint,
        HueName::Teal,
        HueName::Cyan,
        HueName::Blue,
        HueName::Indigo,
        HueName::Purple,
        HueName::Pink,
        HueName::Brown,
    ];

    pub fn name(self) -> &'static str {
        match self {
            HueName::Red => "red",
            HueName::Orange => "orange",
            HueName::Yellow => "yellow",
            HueName::Green => "green",
            HueName::Mint => "mint",
            HueName::Teal => "teal",
            HueName::Cyan => "cyan",
            HueName::Blue => "blue",
            HueName::Indigo => "indigo",
            HueName::Purple => "purple",
            HueName::Pink => "pink",
            HueName::Brown => "brown",
        }
    }

    pub fn from_name(name: &str) -> Option<HueName> {
        HueName::ALL.into_iter().find(|h| h.name() == name)
    }

    pub fn of(self, hues: &Hues) -> Srgba {
        match self {
            HueName::Red => hues.red,
            HueName::Orange => hues.orange,
            HueName::Yellow => hues.yellow,
            HueName::Green => hues.green,
            HueName::Mint => hues.mint,
            HueName::Teal => hues.teal,
            HueName::Cyan => hues.cyan,
            HueName::Blue => hues.blue,
            HueName::Indigo => hues.indigo,
            HueName::Purple => hues.purple,
            HueName::Pink => hues.pink,
            HueName::Brown => hues.brown,
        }
    }
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

/// Everything a widget reads its look from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub scheme: Scheme,
    pub palette: Palette,
    pub hues: Hues,
    pub space: Space,
    pub radius: Radius,
    pub type_ramp: TypeRamp,
    pub density: DensityMode,
    pub durations_ms: Durations,
    /// How far a pressed control shrinks (`vars.press-scale`).
    pub press_scale: f32,
    /// Standard or high contrast.
    pub contrast: Contrast,
    /// Controls draw a `border-strong` edge (high contrast).
    pub outline: bool,
    /// `prefers-reduced-motion`: transitions keep opacity and drop movement and scale; skeleton sheens stop.
    pub reduced_motion: bool,
    /// `prefers-reduced-transparency`: glass parts are opaque surfaces (no
    /// blur of the frame under them, their frame at full opacity), and the
    /// palette's scrim and translucent fills are flat
    /// ([`Theme::to_reduced_transparency`]). A game reads it to flatten
    /// its own (IDL `UiTheme.reducedTransparency`).
    pub reduced_transparency: bool,
    /// The game's look beyond the tokens (`crate::skin`); the default dresses nothing.
    pub skin: crate::skin::Skin,
    /// The card edge (`shadow-edge`): inset layers drawn inside the edge of
    /// an opaque surface (a dialog, a panel, a card, a tooltip) in the
    /// token look. In dark it is the edge light (a 0.5 px top highlight and
    /// a hairline inside, which a drop shadow cannot give on black), in
    /// light a hairline.
    pub edge: &'static [crate::theme::InsetShadow],
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::dark()
    }
}

impl Theme {
    pub fn light() -> Theme {
        Theme::new(Scheme::Light)
    }

    pub fn dark() -> Theme {
        Theme::new(Scheme::Dark)
    }

    pub fn new(scheme: Scheme) -> Theme {
        let (palette, hues) = match scheme {
            Scheme::Light => (tokens::LIGHT, tokens::HUES_LIGHT),
            Scheme::Dark => (tokens::DARK, tokens::HUES_DARK),
        };
        Theme {
            scheme,
            palette,
            hues,
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
            edge: match scheme {
                Scheme::Light => tokens::SHADOW_EDGE_LIGHT,
                Scheme::Dark => tokens::SHADOW_EDGE_DARK,
            },
        }
    }

    /// The high-contrast variant of a scheme: `fg-muted` inks take `fg`,
    /// `fg-subtle` takes `fg-muted`, fills step up to `fill-strong` and
    /// `border-strong`, borders to `border-strong`, and controls are
    /// outlined. Only token values are used.
    pub fn high_contrast(scheme: Scheme) -> Theme {
        Theme::new(scheme).to_high_contrast()
    }

    /// This theme's high-contrast variant (see [`Theme::high_contrast`]): a
    /// skin's palette is remapped the same way.
    pub fn to_high_contrast(self) -> Theme {
        let mut t = self;
        let p = t.palette;
        t.palette.fg_muted = p.fg;
        t.palette.fg_subtle = p.fg_muted;
        t.palette.fill = p.fill_strong;
        t.palette.fill_strong = p.border_strong;
        t.palette.border = p.border_strong;
        t.palette.scrim = p.scrim.fade(1.5);
        t.contrast = Contrast::High;
        t.outline = true;
        t
    }

    /// This theme with nothing to see through (`prefers-reduced-
    /// transparency`): the translucent fills (`fill`, `fill-strong`,
    /// `accent-soft`, the `material-*` grounds and `material-control`)
    /// become the opaque colour they make over `surface`, the `scrim` the
    /// opaque colour it makes over `bg` (a dialog's backdrop hides the game
    /// rather than dimming it), and `reduced_transparency` is on (glass is
    /// an opaque surface). Highlights drawn over a surface (sheens, edges,
    /// glows) stay as they are. Only token values are used.
    pub fn to_reduced_transparency(self) -> Theme {
        let mut t = self;
        let p = &mut t.palette;
        let ground = p.surface;
        for c in [
            &mut p.fill,
            &mut p.fill_strong,
            &mut p.accent_soft,
            &mut p.material_regular,
            &mut p.material_thin,
            &mut p.material_thick,
            &mut p.material_control,
        ] {
            if c.alpha < 1.0 {
                *c = c.over(ground);
            }
        }
        if p.scrim.alpha < 1.0 {
            p.scrim = p.scrim.over(p.bg);
        }
        t.reduced_transparency = true;
        t
    }

    /// This theme with a game's own accent. The hover, active, ink, soft and
    /// focus tones are derived from `accent` the way the tokens derive them
    /// from amber (the same mix towards white or black, the same alpha), and
    /// the ink on the accent is the token `accent-fg` or the light `fg`,
    /// whichever reads better.
    pub fn with_accent(mut self, accent: Srgba) -> Theme {
        let base = match self.scheme {
            Scheme::Light => tokens::LIGHT,
            Scheme::Dark => tokens::DARK,
        };
        let amber = base.accent;
        if accent.rgb == amber.rgb {
            // The tokens' own accent keeps the tokens' own tones.
            self.palette.accent = base.accent;
            self.palette.accent_hover = base.accent_hover;
            self.palette.accent_active = base.accent_active;
            self.palette.accent_ink = base.accent_ink;
            self.palette.accent_soft = base.accent_soft;
            self.palette.accent_fg = base.accent_fg;
            self.palette.focus = base.focus;
            return self;
        }
        let derive = |token: Srgba| -> Srgba {
            let sum = |c: Srgba| c.rgb.iter().map(|&v| f32::from(v)).sum::<f32>();
            let (s0, s1) = (sum(amber), sum(token));
            let toward = if s1 >= s0 { Srgba::WHITE } else { Srgba::BLACK };
            let span = sum(toward) - s0;
            let t = if span.abs() < 1.0 {
                0.0
            } else {
                (s1 - s0) / span
            };
            Srgba {
                alpha: token.alpha,
                ..accent.mix(toward, t)
            }
        };
        let a = Srgba {
            alpha: 1.0,
            ..accent
        };
        self.palette.accent = a;
        self.palette.accent_hover = derive(base.accent_hover);
        self.palette.accent_active = derive(base.accent_active);
        self.palette.accent_ink = derive(base.accent_ink);
        self.palette.focus = derive(base.focus);
        self.palette.accent_soft = Srgba {
            alpha: base.accent_soft.alpha,
            ..a
        };
        let dark_ink = base.accent_fg;
        let light_ink = tokens::DARK.fg;
        let contrast = |x: Srgba, y: Srgba| {
            let (l1, l2) = (x.luminance(), y.luminance());
            (l1.max(l2) + 0.05) / (l1.min(l2) + 0.05)
        };
        self.palette.accent_fg = if contrast(a, dark_ink) >= contrast(a, light_ink) {
            dark_ink
        } else {
            light_ink
        };
        self
    }

    /// This theme with one of the token hues as its accent.
    pub fn with_hue(self, hue: HueName) -> Theme {
        let c = hue.of(&self.hues);
        self.with_accent(c)
    }

    /// Frames a duration in milliseconds takes at `ticks_per_second` (at least 1).
    pub fn ticks(ms: f32, ticks_per_second: u32) -> u32 {
        ((ms * ticks_per_second as f32 / 1000.0).ceil() as u32).max(1)
    }

    /// The control sizes for a viewport.
    pub fn sizes(&self, viewport: &crate::Viewport) -> Density {
        match self.density {
            DensityMode::Compact => tokens::COMPACT,
            DensityMode::Comfortable => tokens::COMFORTABLE,
            DensityMode::Touch => tokens::TOUCH,
            DensityMode::Auto => {
                // "coarse pointers and widths under 768px raise controls to touch size"
                if viewport.coarse_pointer || viewport.width < 768.0 {
                    tokens::TOUCH
                } else {
                    tokens::COMPACT
                }
            }
        }
    }
}
