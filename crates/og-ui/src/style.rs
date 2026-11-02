//! The theme: the values a widget reads its look from (the default style's
//! in [`crate::theme`], a style file's over them) with the few conversions
//! the renderer needs.

use crate::theme as tokens;
pub use crate::theme::{
    Density, Durations, Palette, Radius, Space, Spring, Srgba, TypeRamp, TypeStep,
};

/// The contrast a widget that picks its own ink holds it to on what it is
/// drawn on (a badge, a meter's label): WCAG's 4.5:1 for body text.
pub const READABLE: f32 = 4.5;

/// The contrast a button's label holds on its fill as drawn: WCAG AAA's
/// 7:1, the packs' bar (a small MSDF label over shaded art reads about a
/// third lower in the frame, where og polish holds it to 4.5:1).
pub const BUTTON_READABLE: f32 = 7.0;

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

    /// The same colour with its alpha multiplied by `k` (at most opaque).
    pub fn fade(self, k: f32) -> Srgba {
        Srgba {
            rgb: self.rgb,
            alpha: (self.alpha * k).min(1.0),
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

    /// A colour from linear RGBA (straight alpha), rounded to 8-bit sRGB.
    pub fn from_linear(c: [f32; 4]) -> Srgba {
        let e = |v: f32| {
            let s = if v <= 0.003_130_8 {
                v * 12.92
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            };
            (s.clamp(0.0, 1.0) * 255.0).round() as u8
        };
        Srgba {
            rgb: [e(c[0]), e(c[1]), e(c[2])],
            alpha: c[3].clamp(0.0, 1.0),
        }
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

    /// This translucent fill, stated for og-render's linear-light blending
    /// as a theme's are (`packages/design`'s `linear` values), `k` times as
    /// strong as it reads: the CSS alpha it draws like over mid grey (the
    /// tokens' ground for a scrim), times `k`, restated for linear light.
    /// [`Srgba::fade`] scales the linear alpha itself, which overshoots: a
    /// dark scrim's 0.69 (CSS 0.42) times 1.6 is opaque, not CSS 0.67.
    pub fn deepen(self, k: f32) -> Srgba {
        let grey = Srgba::rgb(128, 128, 128);
        let (lc, lb) = (Srgba { alpha: 1.0, ..self }.luminance(), grey.luminance());
        if (lc - lb).abs() < 1e-4 {
            return self.fade(k);
        }
        let css = |s: f32| Srgba { alpha: s, ..self }.over(grey).luminance();
        // The CSS alpha drawing the luminance this one draws in linear light.
        let drawn = lb + (lc - lb) * self.alpha.clamp(0.0, 1.0);
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..24 {
            let mid = (lo + hi) / 2.0;
            if (css(mid) - lb).abs() < (drawn - lb).abs() {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let want = css(((lo + hi) / 2.0 * k).min(1.0));
        Srgba {
            alpha: ((want - lb) / (lc - lb)).clamp(0.0, 1.0),
            ..self
        }
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

/// The colour roles a style declares of its own (docs/ui.md §12.2): a
/// game's corruption, sanity or faction colours beside the palette's named
/// roles, read by name like any role ([`Theme::color`]: `{ token }`,
/// `[color=]`, a dressing's paints, `UiTheme.palette`). Fixed capacity, so
/// the theme stays a value: [`Roles::MAX`] roles of up to [`Roles::NAME`]
/// bytes, in the order the style declares them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Roles {
    names: [[u8; Roles::NAME]; Roles::MAX],
    lens: [u8; Roles::MAX],
    colors: [Srgba; Roles::MAX],
    count: u8,
}

impl Default for Roles {
    fn default() -> Roles {
        Roles::NONE
    }
}

impl Roles {
    /// The most roles a style declares.
    pub const MAX: usize = 16;
    /// The longest role name, in bytes.
    pub const NAME: usize = 32;
    pub const NONE: Roles = Roles {
        names: [[0; Roles::NAME]; Roles::MAX],
        lens: [0; Roles::MAX],
        colors: [Srgba::BLACK; Roles::MAX],
        count: 0,
    };

    /// Adds or replaces a role. `Err` when the table is full or the name
    /// too long (the style parser checks names before).
    pub fn set(&mut self, name: &str, color: Srgba) -> Result<(), String> {
        if let Some(i) = self.index(name) {
            self.colors[i] = color;
            return Ok(());
        }
        if name.len() > Roles::NAME {
            return Err(format!(
                "role name '{name}' is longer than {} bytes",
                Roles::NAME
            ));
        }
        let i = usize::from(self.count);
        if i >= Roles::MAX {
            return Err(format!("a style declares at most {} roles", Roles::MAX));
        }
        self.names[i][..name.len()].copy_from_slice(name.as_bytes());
        self.lens[i] = name.len() as u8;
        self.colors[i] = color;
        self.count += 1;
        Ok(())
    }

    fn name_at(&self, i: usize) -> &str {
        std::str::from_utf8(&self.names[i][..usize::from(self.lens[i])]).unwrap_or("")
    }

    fn index(&self, name: &str) -> Option<usize> {
        (0..usize::from(self.count)).find(|&i| self.name_at(i) == name)
    }

    /// A role's colour by its name.
    pub fn get(&self, name: &str) -> Option<Srgba> {
        self.index(name).map(|i| self.colors[i])
    }

    /// A role's colour by its name, to change it.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Srgba> {
        self.index(name).map(|i| &mut self.colors[i])
    }

    /// Every role and its colour, in the order declared.
    pub fn iter(&self) -> impl Iterator<Item = (&str, Srgba)> + '_ {
        (0..usize::from(self.count)).map(|i| (self.name_at(i), self.colors[i]))
    }

    pub fn len(&self) -> usize {
        usize::from(self.count)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Light or dark: which variant of the default style the theme starts
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
    /// Every size class, in this order (the index of a per-breakpoint array).
    pub const ALL: [Breakpoint; 4] = [
        Breakpoint::PhonePortrait,
        Breakpoint::PhoneLandscape,
        Breakpoint::Tablet,
        Breakpoint::Desktop,
    ];

    pub fn from_name(name: &str) -> Option<Breakpoint> {
        Breakpoint::ALL.into_iter().find(|b| b.name() == name)
    }

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
    /// The style's own colour roles (docs/ui.md §12.2), read with the
    /// palette's by [`Theme::color`]; the default style declares none.
    pub roles: Roles,
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
    /// The player's text scale (docs/ui.md §15.2), 1 to 2: every text size
    /// and leading grows by it (the type ramp here and each role's step,
    /// [`crate::Ui::type_step`]), and control, row and header heights by
    /// the leading body text gains ([`Theme::sizes`]), so layout grows
    /// instead of clipping. A UI applies it from the next frame on.
    pub text_scale: f32,
    /// The scale `type_ramp` already carries: [`Theme::to_text_scale`]
    /// scales by `text_scale` over it, so applying it twice changes
    /// nothing. 1 for a theme as a style made it.
    #[doc(hidden)]
    pub ramp_scale: f32,
    /// The og-ui unit a game pins (IDL `UiDesc.unitScale`), 1 to 2; 0 (the
    /// default) is auto: [`Theme::auto_unit`] of the play surface.
    pub unit_pin: f32,
    /// The og-ui unit in effect, in CSS px (docs/ui.md §15.2): the UI lays
    /// out in units (a viewport of the surface's size over it) and draws
    /// at it, so every length (the style's, a component's, a game's own
    /// `px`) grows by it and the player's text scale multiplies over it.
    /// 1 for a theme as a style made it; the frame's theme has the
    /// surface's ([`crate::Ui::begin`]).
    pub unit: f32,
    /// The style's art (`crate::style_file`: frames, ornaments, fonts, motion,
    /// sounds); the default dresses nothing.
    pub art: crate::style_file::Art,
    /// The style's version 2 tokens (`crate::tokens`): carried, not yet
    /// drawn from. A version 1 style and the default style have the set
    /// [`crate::tokens::StyleTokens::derive`] makes of their values.
    pub tokens: crate::tokens::StyleTokens,
    /// The style's motion personality (docs/ui.md §7.7): named springs,
    /// enter and exit eases, stagger, overshoot, the reduced-motion
    /// fallback. A style file's motion block fills it.
    pub motion: crate::motion::MotionTokens,
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::dark()
    }
}

impl Theme {
    /// The default style, light.
    pub fn light() -> Theme {
        Theme::new(Scheme::Light)
    }

    /// The default style, dark (the default).
    pub fn dark() -> Theme {
        Theme::new(Scheme::Dark)
    }

    pub fn new(scheme: Scheme) -> Theme {
        let palette = match scheme {
            Scheme::Light => tokens::LIGHT,
            Scheme::Dark => tokens::DARK,
        };
        let art = crate::style_file::Art::default();
        let derived = crate::tokens::StyleTokens::derive_from(
            crate::tokens::V1 {
                palette: &palette,
                space: &tokens::SPACE,
                radius: &tokens::RADIUS,
                type_ramp: &tokens::TYPE_RAMP,
                durations_ms: tokens::DURATIONS_MS,
                art: &art,
            },
            &[],
        );
        Theme {
            scheme,
            palette,
            roles: Roles::NONE,
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
            text_scale: 1.0,
            ramp_scale: 1.0,
            unit_pin: 0.0,
            unit: 1.0,
            art,
            motion: crate::motion::MotionTokens::default(),
            tokens: derived,
        }
    }

    /// A colour role by its name: one of the palette's (the IDL's
    /// ThemeColor), else one the style declares ([`Roles`]).
    pub fn color(&self, name: &str) -> Option<Srgba> {
        self.palette.get(name).or_else(|| self.roles.get(name))
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
        t.palette.scrim = p.scrim.deepen(1.5);
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
        // A style's own translucent roles flatten over the panel too.
        let n = t.roles.len();
        for c in &mut t.roles.colors[..n] {
            if c.alpha < 1.0 {
                *c = c.over(ground);
            }
        }
        t.reduced_transparency = true;
        t
    }

    /// The og-ui unit of a play surface (docs/ui.md §15.2): on a mouse or
    /// pad surface its 1080p-equivalent height (the shorter of the height
    /// and the width's 16:9 height, as `og ui check` reads its text sizes)
    /// over 720, 1 to 2, so the smallest step that is 12 px at 720p is
    /// 18 px at 1080p; a touch surface keeps CSS px (1).
    pub fn auto_unit(viewport: &crate::Viewport) -> f32 {
        if viewport.coarse_pointer || viewport.breakpoint().is_phone() {
            return 1.0;
        }
        let h = viewport.height.min(viewport.width * 9.0 / 16.0);
        if h.is_finite() {
            (h / 720.0).clamp(1.0, 2.0)
        } else {
            1.0
        }
    }

    /// This theme recording og-ui unit `unit` (1 to 2): what the frame
    /// lays out in units of, not a change to any length (`crate::Ui::begin`
    /// zooms the whole UI, every length a game or a component authors
    /// with it).
    pub fn to_unit(self, unit: f32) -> Theme {
        let mut t = self;
        t.unit = if unit.is_finite() {
            unit.clamp(1.0, 2.0)
        } else {
            1.0
        };
        t
    }

    /// This theme with its type ramp at `text_scale` (clamped to 1 to 2):
    /// every step's size and leadings scaled, whatever scale the ramp
    /// carried before.
    pub fn to_text_scale(self) -> Theme {
        let mut t = self;
        let s = if t.text_scale.is_finite() {
            t.text_scale.clamp(1.0, 2.0)
        } else {
            1.0
        };
        let k = s / t.ramp_scale;
        if k != 1.0 {
            for step in t.type_ramp.steps_mut() {
                step.size *= k;
                step.line_height *= k;
                step.cjk_line_height *= k;
            }
        }
        t.text_scale = s;
        t.ramp_scale = s;
        t
    }

    /// This theme with its style's atlas at 2× and 3× its texels
    /// ([`crate::Art::atlas_dense`]; `None` where the style has none).
    pub fn with_dense_atlases(mut self, dense: [Option<og_render::TextureHandle>; 2]) -> Theme {
        self.art.atlas_dense = dense;
        self
    }

    /// This theme with a game's own primary colour: its hover and active
    /// tones 15 % towards either end of the scale (a dark primary lightens
    /// hovered and darkens pressed, a light one the other way),
    /// its soft wash at the palette's alpha, its `highlight` (the accent
    /// ink on that wash: the primary moved towards the scheme's ink until it
    /// reads 5:1 there, so a gold game's badges are gold, not the style's
    /// pink), and the ink on
    /// it whichever of the palette's `ink` and `inkInverse` reads better.
    /// `focus` is the style's own and stays.
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
        // The accent ink: the primary moved towards the scheme's ink as
        // little as it takes to read on its own wash over a panel (with a
        // margin: the wash sits on frame art, not the flat panel).
        let wash = p.primary_soft.over_linear(p.panel);
        p.highlight = a
            .readable_on(wash, READABLE + 0.5)
            .filter(|c| {
                (c.luminance() > wash.luminance()) == (p.ink.luminance() > wash.luminance())
            })
            .unwrap_or_else(|| {
                (1..=20)
                    .map(|n| a.mix(p.ink, n as f32 * 0.05))
                    .find(|c| c.contrast(wash) >= READABLE + 0.5)
                    .unwrap_or(p.ink)
            });
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
        let d = match self.density {
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
        };
        // Text at 200 % needs taller controls: each grows by the leading
        // body text gains.
        let grow = (self.ramp_scale - 1.0) * self.type_ramp.body.line_height / self.ramp_scale;
        Density {
            control_lg: d.control_lg + grow,
            control_md: d.control_md + grow,
            control_sm: d.control_sm + grow,
            header: d.header + grow,
            row: d.row + grow,
            ..d
        }
    }
}
