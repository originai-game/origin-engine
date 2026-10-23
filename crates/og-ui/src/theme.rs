//! The engine's own UI values: the colour roles of a game UI and the
//! `prototype` style, the look a game has before it chooses one. These are
//! owned here and by no one else: the platform's design language styles the
//! platform's pages and the player shell, never a game's screens.
//!
//! `prototype` is deliberately a placeholder: greys, square corners, no
//! glass, a single wireframe cyan for focus and highlights. A game that
//! ships it looks unfinished, and `og polish` says so. A game's own look is
//! a style (`crate::style_file`, `type style`), usually the one its look names.
//!
//! Lengths are CSS pixels, durations milliseconds, colours sRGB as authored
//! (straight alpha); `crate::style` converts them for the renderer.

/// An sRGB colour as a style authors it: 8-bit channels, straight alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Srgba {
    pub rgb: [u8; 3],
    pub alpha: f32,
}

const fn c(r: u8, g: u8, b: u8) -> Srgba {
    Srgba {
        rgb: [r, g, b],
        alpha: 1.0,
    }
}

const fn ca(r: u8, g: u8, b: u8, alpha: f32) -> Srgba {
    Srgba { rgb: [r, g, b], alpha }
}

macro_rules! palette {
    ($($(#[doc = $doc:literal])* $field:ident $name:literal,)*) => {
        /// The colour roles of a game UI. Every widget reads its colours
        /// here; a style sets every one of them (`palette { … }`), and
        /// `[color=<role>]`, `{ token: "<role>" }` name them.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct Palette {
            $($(#[doc = $doc])* pub $field: Srgba,)*
        }

        impl Palette {
            /// Every role's name (the IDL's ThemeColor), in this order.
            pub const NAMES: &'static [&'static str] = &[$($name,)*];

            /// A role by its name.
            pub fn get_mut(&mut self, name: &str) -> Option<&mut Srgba> {
                match name {
                    $($name => Some(&mut self.$field),)*
                    _ => None,
                }
            }

            /// A role by its name.
            pub fn get(&self, name: &str) -> Option<Srgba> {
                match name {
                    $($name => Some(self.$field),)*
                    _ => None,
                }
            }
        }
    };
}

palette! {
    /// The screen behind the UI where nothing else is: a menu's ground, a
    /// dialog's surroundings once the scrim is over them.
    backdrop "backdrop",
    /// A recessed ground inside a panel: wells, map frames, empty slots.
    well "well",
    /// Panels and cards.
    panel "panel",
    /// A panel under the pointer (list rows, selectable cards).
    panel_hover "panelHover",
    /// Dialogs, menus, tooltips: what sits above a panel.
    panel_raised "panelRaised",
    /// Hairlines and frames.
    line "line",
    /// Stronger frames: high contrast, map borders, scrollbars.
    line_strong "lineStrong",
    /// The main ink.
    ink "ink",
    /// Secondary ink: captions, keys, inactive tabs.
    ink_muted "inkMuted",
    /// The faintest ink that still reads: placeholders, hints.
    ink_faint "inkFaint",
    /// Ink on a ground of `ink` (a selected chip).
    ink_inverse "inkInverse",
    /// Neutral control fills: secondary buttons, tracks, fields.
    control "control",
    /// A neutral control hovered or pressed; selection.
    control_strong "controlStrong",
    /// The one directed action: primary buttons, a slider's fill, a switch
    /// that is on, the caret.
    primary "primary",
    /// `primary` hovered.
    primary_hover "primaryHover",
    /// `primary` pressed.
    primary_active "primaryActive",
    /// Ink on `primary`.
    on_primary "onPrimary",
    /// A translucent wash of `primary` (a selected row).
    primary_soft "primarySoft",
    /// Emphasised ink: a speaker's name, a price, a link, the player's dot.
    highlight "highlight",
    /// The focus ring of keyboard and gamepad navigation.
    focus "focus",
    /// Slider and switch knobs.
    knob "knob",
    /// HUD capsules over the game (scores, timers, toasts).
    hud "hud",
    /// Behind a dialog.
    scrim "scrim",
    /// Drop shadows.
    shadow "shadow",
    /// The passing sheen of a skeleton.
    sheen "sheen",
    /// Success.
    success "success",
    /// Warnings.
    warn "warn",
    /// Danger and errors.
    danger "danger",
    /// Information.
    info "info",
    /// A health meter.
    health "health",
    /// A mana meter.
    mana "mana",
    /// An experience meter.
    xp "xp",
    /// Currency: coins, gold, prices that can be paid.
    gold "gold",
}

/// The prototype style's light variant.
pub const LIGHT: Palette = Palette {
    backdrop: c(0xe4, 0xe4, 0xe4),
    well: c(0xd8, 0xd8, 0xd8),
    panel: c(0xf4, 0xf4, 0xf4),
    panel_hover: c(0xe8, 0xe8, 0xe8),
    panel_raised: c(0xff, 0xff, 0xff),
    line: c(0xc4, 0xc4, 0xc4),
    line_strong: c(0x8c, 0x8c, 0x8c),
    ink: c(0x16, 0x16, 0x16),
    ink_muted: c(0x50, 0x50, 0x50),
    ink_faint: c(0x60, 0x60, 0x60),
    ink_inverse: c(0xf4, 0xf4, 0xf4),
    control: ca(0x00, 0x00, 0x00, 0.08),
    control_strong: ca(0x00, 0x00, 0x00, 0.16),
    primary: c(0x1e, 0x1e, 0x1e),
    primary_hover: c(0x38, 0x38, 0x38),
    primary_active: c(0x00, 0x00, 0x00),
    on_primary: c(0xf4, 0xf4, 0xf4),
    primary_soft: ca(0x1e, 0x1e, 0x1e, 0.10),
    highlight: c(0x00, 0x62, 0x7a),
    focus: c(0x00, 0x7c, 0x99),
    knob: c(0xff, 0xff, 0xff),
    hud: ca(0xf4, 0xf4, 0xf4, 0.88),
    scrim: ca(0x00, 0x00, 0x00, 0.35),
    shadow: c(0x00, 0x00, 0x00),
    sheen: ca(0xff, 0xff, 0xff, 0.32),
    success: c(0x16, 0x6e, 0x3a),
    warn: c(0x9a, 0x4a, 0x00),
    danger: c(0x9a, 0x0c, 0x14),
    info: c(0x10, 0x52, 0xa8),
    health: c(0x1c, 0x74, 0x32),
    mana: c(0x23, 0x5c, 0xd6),
    xp: c(0x7a, 0x3c, 0xc8),
    gold: c(0x8a, 0x62, 0x00),
};

/// The prototype style's dark variant (the default: games run on a dark
/// screen more often than not).
pub const DARK: Palette = Palette {
    backdrop: c(0x10, 0x10, 0x10),
    well: c(0x16, 0x16, 0x16),
    panel: c(0x1e, 0x1e, 0x1e),
    panel_hover: c(0x2a, 0x2a, 0x2a),
    panel_raised: c(0x26, 0x26, 0x26),
    line: c(0x3a, 0x3a, 0x3a),
    line_strong: c(0x60, 0x60, 0x60),
    ink: c(0xf0, 0xf0, 0xf0),
    ink_muted: c(0xb4, 0xb4, 0xb4),
    ink_faint: c(0x9c, 0x9c, 0x9c),
    ink_inverse: c(0x16, 0x16, 0x16),
    control: ca(0xff, 0xff, 0xff, 0.06),
    control_strong: ca(0xff, 0xff, 0xff, 0.10),
    primary: c(0xe8, 0xe8, 0xe8),
    primary_hover: c(0xff, 0xff, 0xff),
    primary_active: c(0xc8, 0xc8, 0xc8),
    on_primary: c(0x16, 0x16, 0x16),
    primary_soft: ca(0xff, 0xff, 0xff, 0.12),
    highlight: c(0x6a, 0xd8, 0xec),
    focus: c(0x5a, 0xd1, 0xe6),
    knob: c(0xf0, 0xf0, 0xf0),
    hud: ca(0x10, 0x10, 0x10, 0.82),
    scrim: ca(0x00, 0x00, 0x00, 0.55),
    shadow: c(0x00, 0x00, 0x00),
    sheen: ca(0xff, 0xff, 0xff, 0.24),
    success: c(0x4c, 0xc3, 0x8a),
    warn: c(0xf0, 0xa0, 0x50),
    danger: c(0xff, 0x8a, 0x82),
    info: c(0x7a, 0xb8, 0xff),
    health: c(0x5c, 0xcf, 0x6e),
    mana: c(0x5a, 0x9c, 0xff),
    xp: c(0xb0, 0x7c, 0xff),
    gold: c(0xf2, 0xc1, 0x4e),
};

/// The spacing scale, CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Space {
    pub xl3: f32,
    pub lg: f32,
    pub md: f32,
    pub sm: f32,
    pub xl: f32,
    pub xs: f32,
    pub xxl: f32,
    pub xxs: f32,
}

pub const SPACE: Space = Space {
    xl3: 48.0,
    lg: 16.0,
    md: 12.0,
    sm: 8.0,
    xl: 24.0,
    xs: 4.0,
    xxl: 32.0,
    xxs: 2.0,
};

/// The corner radii, CSS pixels. A style sets them; the prototype's are
/// nearly square. A radius at or past half an element's short side draws
/// it round, so a style that wants pills says so with a large `control`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius {
    /// Small parts: checkboxes, slots, tiles, the minimap.
    pub small: f32,
    /// Controls: buttons, fields, chips, badges, tabs, switches.
    pub control: f32,
    /// Bars: progress, meters, slider tracks, scrollbars.
    pub bar: f32,
    /// Panels, cards, dialogs, tooltips.
    pub panel: f32,
}

pub const RADIUS: Radius = Radius {
    small: 1.0,
    control: 2.0,
    bar: 1.0,
    panel: 3.0,
};

/// One step of the type ramp: size and leading in CSS pixels, Latin
/// tracking in em, and the leading Chinese, Japanese and Korean text takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStep {
    pub size: f32,
    pub line_height: f32,
    pub tracking_em: f32,
    pub cjk_line_height: f32,
}

/// The type ramp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeRamp {
    pub xl2: TypeStep,
    pub xl3: TypeStep,
    pub xl4: TypeStep,
    pub xl5: TypeStep,
    pub xl6: TypeStep,
    pub xl7: TypeStep,
    pub xl8: TypeStep,
    pub base: TypeStep,
    /// The default step.
    pub body: TypeStep,
    pub caption: TypeStep,
    pub display_xl2: TypeStep,
    pub display_lg: TypeStep,
    pub display_md: TypeStep,
    pub display_sm: TypeStep,
    pub display_xl: TypeStep,
    pub lg: TypeStep,
    pub meta: TypeStep,
    pub reading: TypeStep,
    pub sm: TypeStep,
    pub xl: TypeStep,
    pub xs: TypeStep,
}

const fn step(size: f32, line_height: f32, cjk_line_height: f32) -> TypeStep {
    TypeStep {
        size,
        line_height,
        tracking_em: 0.0,
        cjk_line_height,
    }
}

pub const TYPE_RAMP: TypeRamp = TypeRamp {
    xl2: step(22.0, 28.0, 32.0),
    xl3: step(26.0, 32.0, 36.0),
    xl4: step(30.0, 36.0, 40.0),
    xl5: step(36.0, 40.0, 46.0),
    xl6: step(44.0, 48.0, 54.0),
    xl7: step(52.0, 56.0, 62.0),
    xl8: step(64.0, 64.0, 74.0),
    base: step(16.0, 24.0, 24.0),
    body: step(14.0, 20.0, 20.0),
    caption: step(13.0, 18.0, 18.0),
    display_xl2: step(56.0, 60.0, 68.0),
    display_lg: step(40.0, 44.0, 50.0),
    display_md: step(34.0, 38.0, 44.0),
    display_sm: step(28.0, 32.0, 38.0),
    display_xl: step(48.0, 52.0, 60.0),
    lg: step(18.0, 26.0, 26.0),
    meta: step(13.0, 18.0, 18.0),
    reading: step(15.0, 24.0, 24.0),
    sm: step(14.0, 20.0, 20.0),
    xl: step(20.0, 28.0, 30.0),
    xs: step(12.0, 16.0, 16.0),
};

/// Control, row and header heights and the gutter, CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Density {
    pub control_lg: f32,
    pub control_md: f32,
    pub control_sm: f32,
    pub gutter: f32,
    pub header: f32,
    pub row: f32,
    pub section: f32,
}

/// A mouse and keyboard: the tightest sizes.
pub const MOUSE: Density = Density {
    control_lg: 36.0,
    control_md: 32.0,
    control_sm: 28.0,
    gutter: 16.0,
    header: 48.0,
    row: 36.0,
    section: 24.0,
};

/// A gamepad across a room: larger controls and rows, wider gutters, so
/// a focused control reads from a couch.
pub const PAD: Density = Density {
    control_lg: 52.0,
    control_md: 48.0,
    control_sm: 40.0,
    gutter: 32.0,
    header: 64.0,
    row: 52.0,
    section: 40.0,
};

/// Touch: every control at least 44 CSS px, a finger's target.
pub const TOUCH: Density = Density {
    control_lg: 48.0,
    control_md: 44.0,
    control_sm: 44.0,
    gutter: 16.0,
    header: 52.0,
    row: 48.0,
    section: 24.0,
};

/// Motion durations, milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Durations {
    pub base: f32,
    pub bounce: f32,
    pub fast: f32,
    pub instant: f32,
    pub slow: f32,
    pub spring: f32,
}

pub const DURATIONS_MS: Durations = Durations {
    base: 240.0,
    bounce: 640.0,
    fast: 180.0,
    instant: 120.0,
    slow: 400.0,
    spring: 520.0,
};

/// A spring: damping ratio (1 critically damped, below 1 overshoots) and
/// response (the time to reach the target, not a duration), milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub damping: f32,
    pub response_ms: f32,
}

/// Hover lifts and highlights: no overshoot.
pub const SPRING_DEFAULT: Spring = Spring {
    damping: 1.0,
    response_ms: 350.0,
};
/// A fling's release.
pub const SPRING_MOMENTUM: Spring = Spring {
    damping: 0.8,
    response_ms: 400.0,
};

/// Leaving: cubic Bézier control points (x1, y1, x2, y2).
pub const EASE_EXIT: [f32; 4] = [0.4, 0.0, 1.0, 1.0];
/// Entering: cubic Bézier control points (x1, y1, x2, y2).
pub const EASE_OUT: [f32; 4] = [0.23, 1.0, 0.32, 1.0];

/// How far a pressed control shrinks.
pub const PRESS_SCALE: f32 = 0.96;
