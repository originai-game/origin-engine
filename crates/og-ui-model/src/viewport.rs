//! The play surface the UI lays out on: its size, safe area, covered
//! region and preferences, its size class, and the og-ui unit it lays out
//! in.

use crate::{Insets, Rect};

/// The widest the HUD band is by default, over its height: 21:9, so a
/// 32:9 surface's HUD corners sit where a 21:9 one's do, centred
/// ([`Ui::hud_area`]).
pub const HUD_MAX_ASPECT: f32 = 21.0 / 9.0;

/// The play surface the UI lays out in, in CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    /// Device pixels per CSS pixel; rectangles snap to device pixels.
    pub dpr: f32,
    /// The device's unsafe margins (notch, home indicator); the root pads
    /// by them, and CSS `env(safe-area-inset-*)` reads them.
    pub safe_area: Insets,
    /// The surface's size with the browser's own controls (address bar,
    /// toolbars) shown and hidden, CSS's small and large viewports (CSS
    /// Values 4 §6.1.2): what `svh` and `lvh` read, while `vh` and `dvh`
    /// read `height`. 0: the surface's size (native hosts have no such
    /// controls).
    pub small: [f32; 2],
    pub large: [f32; 2],
    /// A touch screen or other coarse pointer (raises controls to touch size).
    pub coarse_pointer: bool,
    /// The page prefers reduced motion (`prefers-reduced-motion: reduce`):
    /// the theme's `reduced_motion` is on whatever the game configured.
    pub reduced_motion: bool,
    /// The page prefers more contrast (`prefers-contrast: more`): the
    /// theme is its high-contrast variant whatever the game configured.
    pub high_contrast: bool,
    /// The page prefers less transparency (`prefers-reduced-transparency:
    /// reduce`): the theme's `reduced_transparency` is on whatever the game
    /// configured (glass parts draw as opaque surfaces, scrims and
    /// translucent fills flat: `Theme::to_reduced_transparency`).
    pub reduced_transparency: bool,
    /// Text controls take macOS's editing keys: Command for the shortcuts
    /// (A, C, X, Z) and with ←/→ for the line's ends (↑/↓ the text's),
    /// Option with ←/→, Backspace and Delete by words, Control+A and E the
    /// line's ends (emacs). Otherwise Control takes the shortcuts and moves
    /// by words (Alt too), Home and End go to the line's ends and Command
    /// only copies, cuts, selects all and undoes. The web shell sets it on
    /// Apple platforms (`navigator.platform`); native hosts leave it off, so
    /// a script edits the same on every OS.
    pub mac_keys: bool,
    /// Where the host's own control covers the surface (the web shell's
    /// round Back, top left of a published page), in CSS pixels; empty is
    /// none. Documents' `env(safe-area-inset-*)` clear it
    /// ([`Viewport::env_insets`]), and anchored groups keep out of it: a
    /// group it would overlap moves along the edge it sits on, past it.
    pub covered: Rect,
    /// The surface presents the Display P3 gamut: CSS colours outside sRGB
    /// keep their gamut and `(color-gamut: p3)` matches. The host sets it
    /// from the renderer it attached (`RenderInfo::wide_gamut`).
    pub wide_gamut: bool,
    /// The surface presents extended range: `(dynamic-range: high)`
    /// matches (`RenderInfo::hdr`).
    pub high_range: bool,
}

impl Default for Viewport {
    fn default() -> Viewport {
        Viewport {
            width: 1280.0,
            height: 720.0,
            dpr: 1.0,
            safe_area: Insets::default(),
            small: [0.0; 2],
            large: [0.0; 2],
            coarse_pointer: false,
            reduced_motion: false,
            high_contrast: false,
            reduced_transparency: false,
            mac_keys: false,
            covered: Rect::default(),
            wide_gamut: false,
            high_range: false,
        }
    }
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Viewport {
        Viewport {
            width,
            height,
            ..Viewport::default()
        }
    }

    /// This surface in og-ui units of `unit` CSS px (docs/ui.md §6.1):
    /// its size, safe area and covered region over `unit`, and `unit`
    /// times as many device pixels to each, so what is laid out on it and
    /// drawn `unit` times larger lands on the same device pixels.
    pub fn zoomed(&self, unit: f32) -> Viewport {
        if unit == 1.0 {
            return *self;
        }
        let k = 1.0 / unit;
        let s = self.safe_area;
        Viewport {
            width: self.width * k,
            height: self.height * k,
            dpr: self.dpr * unit,
            small: self.small.map(|v| v * k),
            large: self.large.map(|v| v * k),
            safe_area: Insets {
                left: s.left * k,
                top: s.top * k,
                right: s.right * k,
                bottom: s.bottom * k,
            },
            covered: Rect::new(
                self.covered.x * k,
                self.covered.y * k,
                self.covered.width * k,
                self.covered.height * k,
            ),
            ..*self
        }
    }

    /// Its small and large viewports' sizes (`small`, `large`, else its own).
    pub fn extremes(&self) -> [[f32; 2]; 2] {
        let or = |v: [f32; 2]| {
            [
                if v[0] > 0.0 { v[0] } else { self.width },
                if v[1] > 0.0 { v[1] } else { self.height },
            ]
        };
        [or(self.small), or(self.large)]
    }

    /// The size class of this surface.
    pub fn breakpoint(&self) -> Breakpoint {
        Breakpoint::of(self.width, self.height)
    }

    /// What CSS `env(safe-area-inset-*)` reads: the safe area with the
    /// host's covered region folded into the one edge whose inset clears
    /// it for the least surface, as a phone folds its status bar into the
    /// top inset in portrait and its notch into a side one in landscape.
    /// The web shell's Back, top left, widens the left inset on a
    /// landscape surface and deepens the top one on a portrait one, so a
    /// HUD placed at the insets is clear of it at every corner.
    pub fn env_insets(&self) -> Insets {
        let mut s = self.safe_area;
        let c = self.covered;
        if c.width <= 0.0 || c.height <= 0.0 {
            return s;
        }
        // Each edge's inset to clear it, and the surface that inset loses.
        let needs = [
            (c.x + c.width - s.left, self.height),
            (self.width - c.x - s.right, self.height),
            (c.y + c.height - s.top, self.width),
            (self.height - c.y - s.bottom, self.width),
        ];
        let (k, (grow, _)) = needs
            .into_iter()
            .enumerate()
            .min_by(|(_, (a, la)), (_, (b, lb))| (a.max(0.0) * la).total_cmp(&(b.max(0.0) * lb)))
            .expect("four edges");
        let grow = grow.max(0.0);
        match k {
            0 => s.left += grow,
            1 => s.right += grow,
            2 => s.top += grow,
            _ => s.bottom += grow,
        }
        s
    }

    /// What CSS `env(-og-hud-inset-*)` reads: [`Viewport::env_insets`]
    /// with the left and right insets grown alike until the area between
    /// them is at most `max_aspect` times its height wide (UiDesc.hudMaxAspect;
    /// 0 or not finite: no narrower), so a HUD document placed at them sits
    /// in the HUD band on an ultrawide and stays clear of the host's
    /// covered region everywhere.
    pub fn hud_insets(&self, max_aspect: f32) -> Insets {
        let mut s = self.env_insets();
        if !(max_aspect.is_finite() && max_aspect > 0.0) {
            return s;
        }
        let width = self.width - s.left - s.right;
        let widest = max_aspect * (self.height - s.top - s.bottom).max(0.0);
        if width > widest {
            let grow = (width - widest) / 2.0;
            s.left += grow;
            s.right += grow;
        }
        s
    }

    /// The safe area, in CSS pixels.
    pub fn safe_rect(&self) -> Rect {
        Rect::new(
            self.safe_area.left,
            self.safe_area.top,
            (self.width - self.safe_area.horizontal()).max(0.0),
            (self.height - self.safe_area.vertical()).max(0.0),
        )
    }

    /// The HUD band ([`Ui::hud_area`]): the safe area at most `max_aspect`
    /// times its height wide, centred; the whole safe area where it is no
    /// wider, or for a `max_aspect` of 0 (or not finite).
    pub fn hud_rect(&self, max_aspect: f32) -> Rect {
        let safe = self.safe_rect();
        if !(max_aspect.is_finite() && max_aspect > 0.0) {
            return safe;
        }
        let widest = max_aspect * safe.height;
        if safe.width <= widest {
            return safe;
        }
        Rect::new(
            safe.x + (safe.width - widest) / 2.0,
            safe.y,
            widest,
            safe.height,
        )
    }

    pub fn dpr(&self) -> f32 {
        if self.dpr.is_finite() && self.dpr > 0.0 {
            self.dpr
        } else {
            1.0
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

/// The og-ui unit of a play surface (docs/ui.md §6.1): on a mouse or
/// pad surface its 1080p-equivalent height (the shorter of the height
/// and the width's 16:9 height, as `og ui check` reads its text sizes)
/// over 720, 1 to 2, so the smallest step that is 12 px at 720p is
/// 18 px at 1080p; a touch surface keeps CSS px (1).
pub fn auto_unit(viewport: &Viewport) -> f32 {
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
