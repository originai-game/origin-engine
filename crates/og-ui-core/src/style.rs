//! The theme: the values a widget reads its look from (the default style's
//! in [`crate::theme`], a style file's over them) with the few conversions
//! the renderer needs.

pub mod css_color;
pub mod dress;
pub mod paint_entry;
pub mod sound_css;
pub mod style_file;
pub mod tokens;
use crate::theme;
pub use crate::theme::{
    Controls, Density, Durations, Palette, Radius, Space, Spring, Srgba, State, TypeRamp, TypeStep,
};

/// The control sizes of each input (CSS `--density-compact-*`,
/// `--density-pad-*`, `--density-touch-*`); [`Theme::sizes`] takes the
/// input's, which documents read as `--control-sm` … `--section`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Densities {
    pub mouse: Density,
    pub pad: Density,
    pub touch: Density,
}

/// The contrast a widget that picks its own ink holds it to on what it is
/// drawn on (a badge, a meter's label): WCAG's 4.5:1 for body text.
pub const READABLE: f32 = 4.5;

/// The colour roles a style declares of its own (docs/ui.md §3.1): a
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

/// The largest player text scale (docs/ui.md §6.1): 150 %, the bar every
/// screen is held to, until 200 % returns as its own batch (the owner,
/// 2026-11-04). A larger UiDesc.textScale lays out at this.
pub const MAX_TEXT_SCALE: f32 = 1.5;

/// Everything a widget reads its look from: a style's values and its art.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub scheme: Scheme,
    pub palette: Palette,
    /// The style's own colour roles (docs/ui.md §3.1), read with the
    /// palette's by [`Theme::color`]; the default style declares none.
    pub roles: Roles,
    pub space: Space,
    pub radius: Radius,
    pub type_ramp: TypeRamp,
    pub density: DensityMode,
    pub durations_ms: Durations,
    /// How far a pressed control shrinks.
    pub press_scale: f32,
    /// How a control answers: its hover wash, press, lift, disabled
    /// opacity and touch target (CSS `--state-*`, `--touch-target`).
    pub state: State,
    /// The control sizes of each input ([`Theme::sizes`] picks one).
    pub densities: Densities,
    /// The curves controls enter and leave on, cubic Bézier control
    /// points (CSS `--motion-easing-out`, `--motion-easing-exit`).
    pub ease_out: [f32; 4],
    pub ease_exit: [f32; 4],
    /// The built-in controls' own sizes, ratios and motion (CSS
    /// `--<control>-<property>`, docs/ui.md §3.1).
    pub controls: Controls,
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
    /// The player's colour vision setting (docs/ui.md §3.2, IDL
    /// `UiDesc.colorVision`): the frame, scene and UI alike, goes through
    /// [`og_draw::ColorFilter`] for it, and document CSS reads it as
    /// `@media (-og-color-vision: …)`.
    pub color_vision: og_draw::ColorVision,
    /// Correct for `color_vision` (the player's setting) or simulate it
    /// (checking a screen).
    pub color_vision_filter: og_draw::ColorVisionFilter,
    /// The deficiency's severity, 0 to 1 (1: a dichromat's).
    pub color_vision_strength: f32,
    /// The player's text scale (docs/ui.md §6.1), 1 to 2: every text size
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
    /// The og-ui unit in effect, in CSS px (docs/ui.md §6.1): the UI lays
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
    /// The style's motion personality (docs/ui.md §3.1): named springs,
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
            Scheme::Light => theme::LIGHT,
            Scheme::Dark => theme::DARK,
        };
        let art = crate::style_file::Art::default();
        let derived = crate::tokens::StyleTokens::derive_from(
            crate::tokens::V1 {
                palette: &palette,
                space: &theme::SPACE,
                radius: &theme::RADIUS,
                type_ramp: &theme::TYPE_RAMP,
                durations_ms: theme::DURATIONS_MS,
                art: &art,
            },
            &[],
        );
        Theme {
            scheme,
            palette,
            roles: Roles::NONE,
            space: theme::SPACE,
            radius: theme::RADIUS,
            type_ramp: derived.typography.ramp(Breakpoint::Desktop, 1.0),
            density: DensityMode::Auto,
            durations_ms: theme::DURATIONS_MS,
            press_scale: theme::PRESS_SCALE,
            state: theme::STATE,
            densities: Densities {
                mouse: theme::MOUSE,
                pad: theme::PAD,
                touch: theme::TOUCH,
            },
            ease_out: theme::EASE_OUT,
            ease_exit: theme::EASE_EXIT,
            controls: theme::CONTROLS,
            contrast: Contrast::Standard,
            outline: false,
            reduced_motion: false,
            reduced_transparency: false,
            color_vision: og_draw::ColorVision::Typical,
            color_vision_filter: og_draw::ColorVisionFilter::Correct,
            color_vision_strength: 1.0,
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
        use crate::tokens::{DamageType, Rarity};
        self.palette
            .get(name)
            .or_else(|| self.roles.get(name))
            .or_else(|| {
                if let Some(r) = Rarity::from_name(name) {
                    Some(self.tokens.colors.rarity[r.index()].color)
                } else if let Some(d) = DamageType::from_name(name) {
                    Some(self.tokens.colors.damage[d.index()].color)
                } else if let Some(n) = name
                    .strip_prefix("team")
                    .and_then(|n| n.parse::<usize>().ok())
                {
                    n.checked_sub(1)
                        .and_then(|i| self.tokens.colors.team.get(i))
                        .map(|s| s.color)
                } else if let Some(n) = name
                    .strip_prefix("series")
                    .and_then(|n| n.parse::<usize>().ok())
                {
                    n.checked_sub(1)
                        .and_then(|i| self.tokens.colors.series.get(i))
                        .copied()
                } else {
                    None
                }
            })
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
            &mut p.selection,
            &mut p.scene_plate,
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

    /// The filter the frame goes through for the player's colour vision
    /// (`RenderScene::color_filter`); `None` for typical vision.
    pub fn color_filter(&self) -> Option<og_draw::ColorFilter> {
        og_draw::ColorFilter::new(
            self.color_vision,
            self.color_vision_filter,
            self.color_vision_strength,
        )
    }

    /// The og-ui unit of a play surface (docs/ui.md §6.1): on a mouse or
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

    /// This theme with its type ramp at `text_scale` (clamped to 1 to
    /// [`MAX_TEXT_SCALE`]): every step's size and leadings scaled, whatever
    /// scale the ramp carried before.
    pub fn to_text_scale(self) -> Theme {
        let mut t = self;
        let s = if t.text_scale.is_finite() {
            t.text_scale.clamp(1.0, MAX_TEXT_SCALE)
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
    pub fn with_dense_atlases(mut self, dense: [Option<og_draw::TextureHandle>; 2]) -> Theme {
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

    /// The density a viewport takes: its CSS name (`compact`, `pad`,
    /// `touch`) and its sizes before the text scale.
    pub fn active_density(&self, viewport: &crate::Viewport) -> (&'static str, Density) {
        let touch = match self.density {
            DensityMode::Mouse => false,
            DensityMode::Pad => return ("pad", self.densities.pad),
            DensityMode::Touch => true,
            DensityMode::Auto => viewport.coarse_pointer || viewport.width < 768.0,
        };
        if touch {
            ("touch", self.densities.touch)
        } else {
            ("compact", self.densities.mouse)
        }
    }

    /// The control sizes for a viewport.
    pub fn sizes(&self, viewport: &crate::Viewport) -> Density {
        let (_, d) = self.active_density(viewport);
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
