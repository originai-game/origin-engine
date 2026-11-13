//! The engine's default style: the colour roles of a game UI and the values
//! of the design language the engine shares with origin-agent
//! (`packages/design`, docs/ui.md §3): the neutral ladder, a
//! neutral primary set apart by fill and weight (2026-11-02), hue only where
//! it means something, flat glass on floating chrome only, and borderless
//! controls told apart by tone.
//!
//! **Generated** from `packages/design/tokens/*.css` by
//! `crates/og-ui/scripts/gen-theme.sh`; never edit it by hand (`tests/theme.rs`
//! fails on drift). The default style is origin-agent's look and the one a
//! game draws in until it picks another (`crate::style_file`, `type style`,
//! usually the one its look names); a game may ship it as it is.
//!
//! Lengths are og-ui units (CSS pixels at a unit scale of 1), durations
//! milliseconds, colours sRGB as authored (straight alpha); the translucent
//! fills are the tokens' `linear` values, restated for the UI pass's
//! linear-light blending. `crate::style` converts them for the renderer.

pub use og_css::Srgba;

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
    /// Selected rows, tabs and cards; independent of a primary action.
    selection "selection",
    /// Ink on the selection ground.
    on_selection "onSelection",
    /// Decorative emphasis, independent of selection and primary actions.
    accent "accent",
    /// HUD text drawn directly over the scene.
    scene_ink "sceneInk",
    /// The scheme's plate behind HUD content.
    scene_plate "scenePlate",
    /// The scheme's separation shadow under scene text.
    scene_shadow "sceneShadow",
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

/// The default style's light scheme.
pub const LIGHT: Palette = Palette {
    backdrop: c(0xf2, 0xf2, 0xf7),
    well: c(0xe9, 0xe9, 0xee),
    panel: c(0xff, 0xff, 0xff),
    panel_hover: c(0xe5, 0xe5, 0xea),
    panel_raised: c(0xff, 0xff, 0xff),
    line: c(0xdc, 0xdc, 0xe1),
    line_strong: c(0xc4, 0xc4, 0xc9),
    ink: c(0x18, 0x18, 0x1a),
    ink_muted: c(0x46, 0x46, 0x4c),
    ink_faint: c(0x63, 0x63, 0x69),
    ink_inverse: c(0xfa, 0xfa, 0xfa),
    control: ca(0x78, 0x78, 0x80, 0.169),
    control_strong: ca(0x78, 0x78, 0x80, 0.271),
    primary: c(0x1c, 0x1c, 0x1e),
    primary_hover: c(0x3a, 0x3a, 0x3c),
    primary_active: c(0x00, 0x00, 0x00),
    on_primary: c(0xfa, 0xfa, 0xfa),
    primary_soft: ca(0x00, 0x00, 0x00, 0.131),
    highlight: c(0x18, 0x18, 0x1a),
    selection: c(0xc4, 0xc4, 0xc9),
    on_selection: c(0x18, 0x18, 0x1a),
    accent: c(0x18, 0x18, 0x1a),
    scene_ink: c(0x18, 0x18, 0x1a),
    scene_plate: c(0xff, 0xff, 0xff),
    scene_shadow: c(0x00, 0x00, 0x00),
    focus: c(0x3a, 0x3a, 0x3c),
    knob: c(0xff, 0xff, 0xff),
    hud: ca(0xfa, 0xfa, 0xfc, 0.86),
    scrim: ca(0x00, 0x00, 0x00, 0.416),
    shadow: c(0x00, 0x00, 0x00),
    sheen: ca(0xff, 0xff, 0xff, 0.5),
    success: c(0x1a, 0x7a, 0x37),
    warn: c(0xc2, 0x41, 0x0c),
    danger: c(0xb2, 0x00, 0x11),
    info: c(0x00, 0x62, 0xc4),
    health: c(0x1b, 0x7a, 0x38),
    mana: c(0x00, 0x62, 0xc4),
    xp: c(0x7d, 0x3f, 0xc4),
    gold: c(0x8a, 0x5a, 0x00),
};

/// The default style's dark scheme (the default: games run on a dark
/// screen more often than not).
pub const DARK: Palette = Palette {
    backdrop: c(0x00, 0x00, 0x00),
    well: c(0x11, 0x11, 0x13),
    panel: c(0x1c, 0x1c, 0x1e),
    panel_hover: c(0x3a, 0x3a, 0x3c),
    panel_raised: c(0x2c, 0x2c, 0x2e),
    line: c(0x38, 0x38, 0x3a),
    line_strong: c(0x54, 0x54, 0x58),
    ink: c(0xf5, 0xf5, 0xf7),
    ink_muted: c(0xc8, 0xc8, 0xcd),
    ink_faint: c(0x9e, 0x9e, 0xa3),
    ink_inverse: c(0x18, 0x18, 0x1a),
    control: ca(0x78, 0x78, 0x80, 0.115),
    control_strong: ca(0x78, 0x78, 0x80, 0.195),
    primary: c(0xe8, 0xe8, 0xed),
    primary_hover: c(0xff, 0xff, 0xff),
    primary_active: c(0xc7, 0xc7, 0xcc),
    on_primary: c(0x11, 0x11, 0x13),
    primary_soft: ca(0xff, 0xff, 0xff, 0.028),
    highlight: c(0xf5, 0xf5, 0xf7),
    selection: c(0x54, 0x54, 0x58),
    on_selection: c(0xf5, 0xf5, 0xf7),
    accent: c(0xf5, 0xf5, 0xf7),
    scene_ink: c(0xf5, 0xf5, 0xf7),
    scene_plate: c(0x2c, 0x2c, 0x2e),
    scene_shadow: c(0x00, 0x00, 0x00),
    focus: c(0xf5, 0xf5, 0xf7),
    knob: c(0xf5, 0xf5, 0xf7),
    hud: ca(0x16, 0x16, 0x18, 0.8),
    scrim: ca(0x00, 0x00, 0x00, 0.687),
    shadow: c(0x00, 0x00, 0x00),
    sheen: ca(0xff, 0xff, 0xff, 0.12),
    success: c(0x3d, 0xd6, 0x8c),
    warn: c(0xff, 0x8a, 0x3d),
    danger: c(0xff, 0x8a, 0x82),
    info: c(0x5c, 0xc8, 0xff),
    health: c(0x30, 0xd1, 0x58),
    mana: c(0x40, 0x9c, 0xff),
    xp: c(0xbf, 0x5a, 0xf2),
    gold: c(0xff, 0xd6, 0x0a),
};

/// The spacing scale, og-ui units.
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

/// The corner radii, og-ui units. A style sets them; the default's are
/// the tokens' `control` (small), `button` (controls), `capsule` (bars,
/// round at any height) and `panel`. A radius at or past half an
/// element's short side draws it round.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius {
    /// Small parts: checkboxes, slots, tiles.
    pub small: f32,
    /// Controls: buttons, fields, chips, badges, tabs, switches.
    pub control: f32,
    /// Bars: progress, meters, slider tracks, scrollbars.
    pub bar: f32,
    /// Panels, cards, dialogs, tooltips.
    pub panel: f32,
}

pub const RADIUS: Radius = Radius {
    small: 4.0,
    control: 6.0,
    bar: 9999.0,
    panel: 12.0,
};

/// One step of the type ramp: size and leading in og-ui units, Latin
/// tracking in em, and the leading Chinese, Japanese and Korean text takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStep {
    pub size: f32,
    pub line_height: f32,
    pub tracking_em: f32,
    pub cjk_line_height: f32,
    /// Tracking of text with CJK characters, in em (CSS `letter-spacing`, which spaces Han too); 0 in the tokens.
    pub cjk_tracking_em: f32,
    /// OpenType features the step turns on (small caps, tabular figures;
    /// the tokens' `fontFeatures`), applied where its font ships them.
    pub features: crate::shape::Features,
    /// The weight (100 to 900) the step's text takes the nearest real face
    /// of (a style's `fonts { body 600 "…" }`); 0: the role's own face.
    pub weight: u16,
    /// Locale-sensitive uppercase before measurement; source text is retained.
    pub uppercase: bool,
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

impl TypeRamp {
    /// Every step, to change them all (a text scale).
    pub fn steps_mut(&mut self) -> [&mut TypeStep; 21] {
        let TypeRamp {
            xl2,
            xl3,
            xl4,
            xl5,
            xl6,
            xl7,
            xl8,
            base,
            body,
            caption,
            display_xl2,
            display_lg,
            display_md,
            display_sm,
            display_xl,
            lg,
            meta,
            reading,
            sm,
            xl,
            xs,
        } = self;
        [
            xl2, xl3, xl4, xl5, xl6, xl7, xl8, base, body, caption, display_xl2, display_lg,
            display_md, display_sm, display_xl, lg, meta, reading, sm, xl, xs,
        ]
    }
}

pub const TYPE_RAMP: TypeRamp = TypeRamp {
    xl2: TypeStep {
        size: 22.0,
        line_height: 28.0,
        tracking_em: -0.015,
        cjk_line_height: 32.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    xl3: TypeStep {
        size: 26.0,
        line_height: 32.0,
        tracking_em: -0.02,
        cjk_line_height: 36.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    xl4: TypeStep {
        size: 30.0,
        line_height: 36.0,
        tracking_em: -0.022,
        cjk_line_height: 40.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    xl5: TypeStep {
        size: 36.0,
        line_height: 40.0,
        tracking_em: -0.025,
        cjk_line_height: 46.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    xl6: TypeStep {
        size: 44.0,
        line_height: 48.0,
        tracking_em: -0.03,
        cjk_line_height: 54.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    xl7: TypeStep {
        size: 52.0,
        line_height: 56.0,
        tracking_em: -0.032,
        cjk_line_height: 62.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    xl8: TypeStep {
        size: 64.0,
        line_height: 64.0,
        tracking_em: -0.035,
        cjk_line_height: 74.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    base: TypeStep {
        size: 16.0,
        line_height: 24.0,
        tracking_em: 0.0,
        cjk_line_height: 24.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    body: TypeStep {
        size: 14.0,
        line_height: 20.0,
        tracking_em: 0.0,
        cjk_line_height: 20.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    caption: TypeStep {
        size: 13.0,
        line_height: 18.0,
        tracking_em: 0.0,
        cjk_line_height: 18.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    display_xl2: TypeStep {
        size: 56.0,
        line_height: 60.0,
        tracking_em: -0.025,
        cjk_line_height: 68.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    display_lg: TypeStep {
        size: 40.0,
        line_height: 44.0,
        tracking_em: -0.02,
        cjk_line_height: 50.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    display_md: TypeStep {
        size: 34.0,
        line_height: 38.0,
        tracking_em: -0.016,
        cjk_line_height: 44.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    display_sm: TypeStep {
        size: 28.0,
        line_height: 32.0,
        tracking_em: -0.012,
        cjk_line_height: 38.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    display_xl: TypeStep {
        size: 48.0,
        line_height: 52.0,
        tracking_em: -0.022,
        cjk_line_height: 60.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    lg: TypeStep {
        size: 18.0,
        line_height: 26.0,
        tracking_em: 0.0,
        cjk_line_height: 26.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    meta: TypeStep {
        size: 13.0,
        line_height: 18.0,
        tracking_em: 0.0,
        cjk_line_height: 18.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    reading: TypeStep {
        size: 15.0,
        line_height: 24.0,
        tracking_em: 0.0,
        cjk_line_height: 24.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    sm: TypeStep {
        size: 14.0,
        line_height: 20.0,
        tracking_em: 0.0,
        cjk_line_height: 20.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    xl: TypeStep {
        size: 20.0,
        line_height: 28.0,
        tracking_em: -0.01,
        cjk_line_height: 30.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
    xs: TypeStep {
        size: 12.0,
        line_height: 16.0,
        tracking_em: 0.0,
        cjk_line_height: 16.0,
        cjk_tracking_em: 0.0,
        features: crate::shape::Features::NONE,
        weight: 0,
        uppercase: false,
    },
};

/// Control, row and header heights and the gutter, og-ui units.
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

/// A mouse and keyboard: the tightest sizes (the tokens' `compact`).
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
/// a focused control reads from a couch (the tokens' `pad`).
pub const PAD: Density = Density {
    control_lg: 52.0,
    control_md: 48.0,
    control_sm: 40.0,
    gutter: 32.0,
    header: 64.0,
    row: 52.0,
    section: 40.0,
};

/// Touch: every control at least 44 CSS px, a finger's target (the
/// tokens' `touch`, its gutter and section `compact`'s).
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

/// The timing table's own numbers (the design language's `motion.timing`;
/// its other rows name the durations, eases and springs above). A style's
/// motion block overrides each (docs/ui.md §3.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timing {
    /// How much a pad's virtual cursor slows and settles over a control, 0–1.
    pub magnet: f32,
    /// The press squash's spring back: its damping over `instant`.
    pub press_damping: f32,
    /// A cascade's step between children and the most it spreads over, ms.
    pub cascade_step_ms: f32,
    pub cascade_cap_ms: f32,
    /// How long a toast stays, and an error's toast, ms.
    pub toast_hold_ms: f32,
    pub toast_danger_ms: f32,
    /// A screen transition covers and reveals over this many `slow` durations.
    pub screen_slows: f32,
}

pub const TIMING: Timing = Timing {
    magnet: 0.5,
    press_damping: 0.55,
    cascade_step_ms: 40.0,
    cascade_cap_ms: 400.0,
    toast_hold_ms: 3200.0,
    toast_danger_ms: 6000.0,
    screen_slows: 2.0,
};

/// A spring: damping ratio (1 critically damped, below 1 overshoots) and
/// response (the time to reach the target, not a duration), milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub damping: f32,
    pub response_ms: f32,
}

/// Overlays, menus and highlights: no overshoot.
pub const SPRING_DEFAULT: Spring = Spring {
    damping: 1.0,
    response_ms: 350.0,
};
/// Sheets and drawers.
pub const SPRING_SHEET: Spring = Spring {
    damping: 0.85,
    response_ms: 300.0,
};
/// A release that carried velocity (a fling, a throw).
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

/// How a control answers (the tokens' `state`; CSS `--state-*`,
/// `--touch-target`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    /// How far hover (half) and press (whole) move a control's fill
    /// towards its wash.
    pub hover_wash: f32,
    /// What a pressed control row shrinks to (a button's part takes
    /// `PRESS_SCALE`).
    pub control_press_scale: f32,
    /// The hover and focus rise, og-ui units (negative is up).
    pub lift: f32,
    /// A disabled control's opacity.
    pub disabled_opacity: f32,
    /// The smallest side a finger's target gets, og-ui units.
    pub touch_target: f32,
}

pub const STATE: State = State {
    hover_wash: 0.1,
    control_press_scale: 0.97,
    lift: -1.0,
    disabled_opacity: 0.5,
    touch_target: 44.0,
};

/// The built-in controls' own sizes, ratios and motion (the tokens'
/// `controls`; CSS `--<control>-<property>`, docs/ui.md §3.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Controls {
    /// `--action-size`, og-ui units.
    pub action_size: f32,
    /// `--checkbox-mark-ratio`, a ratio or number.
    pub checkbox_mark_ratio: f32,
    /// `--checkbox-pop-bounce`, a ratio or number.
    pub checkbox_pop_bounce: f32,
    /// `--checkbox-pop-duration`, ms.
    pub checkbox_pop_duration: f32,
    /// `--checkbox-radius-ratio`, a ratio or number.
    pub checkbox_radius_ratio: f32,
    /// `--checkbox-size`, og-ui units.
    pub checkbox_size: f32,
    /// `--checkbox-unmark-duration`, ms.
    pub checkbox_unmark_duration: f32,
    /// `--details-icon`, og-ui units.
    pub details_icon: f32,
    /// `--dpad-size`, og-ui units.
    pub dpad_size: f32,
    /// `--dropdown-option-row`, og-ui units.
    pub dropdown_option_row: f32,
    /// `--dropdown-row`, og-ui units.
    pub dropdown_row: f32,
    /// `--dropdown-width`, og-ui units.
    pub dropdown_width: f32,
    /// `--joystick-size`, og-ui units.
    pub joystick_size: f32,
    /// `--keyboard-candidate-height`, og-ui units.
    pub keyboard_candidate_height: f32,
    /// `--keyboard-key-height`, og-ui units.
    pub keyboard_key_height: f32,
    /// `--keyboard-key-min-height`, og-ui units.
    pub keyboard_key_min_height: f32,
    /// `--menu-open-duration`, ms.
    pub menu_open_duration: f32,
    /// `--modal-width`, og-ui units.
    pub modal_width: f32,
    /// `--radio-dot-ratio`, a ratio or number.
    pub radio_dot_ratio: f32,
    /// `--radio-pop-bounce`, a ratio or number.
    pub radio_pop_bounce: f32,
    /// `--radio-pop-duration`, ms.
    pub radio_pop_duration: f32,
    /// `--radio-size`, og-ui units.
    pub radio_size: f32,
    /// `--segmented-pad-ratio`, a ratio or number.
    pub segmented_pad_ratio: f32,
    /// `--slider-bubble-exit-duration`, ms.
    pub slider_bubble_exit_duration: f32,
    /// `--slider-bubble-height`, og-ui units.
    pub slider_bubble_height: f32,
    /// `--slider-knob`, og-ui units.
    pub slider_knob: f32,
    /// `--slider-knob-touch`, og-ui units.
    pub slider_knob_touch: f32,
    /// `--slider-rail`, og-ui units.
    pub slider_rail: f32,
    /// `--slider-rail-touch`, og-ui units.
    pub slider_rail_touch: f32,
    /// `--slider-tick-width`, og-ui units.
    pub slider_tick_width: f32,
    /// `--state-press-duration`, ms.
    pub state_press_duration: f32,
    /// `--switch-height-lg`, og-ui units.
    pub switch_height_lg: f32,
    /// `--switch-height-md`, og-ui units.
    pub switch_height_md: f32,
    /// `--switch-height-sm`, og-ui units.
    pub switch_height_sm: f32,
    /// `--switch-inset-ratio`, a ratio or number.
    pub switch_inset_ratio: f32,
    /// `--switch-width-ratio`, a ratio or number.
    pub switch_width_ratio: f32,
}

impl Controls {
    /// Every field by its CSS name (without `--`) and unit (`px`, `ms` or
    /// none), as the root tokens read and write them.
    pub fn fields_mut(&mut self) -> [(&'static str, &'static str, &mut f32); 36] {
        [
            ("action-size", "px", &mut self.action_size),
            ("checkbox-mark-ratio", "", &mut self.checkbox_mark_ratio),
            ("checkbox-pop-bounce", "", &mut self.checkbox_pop_bounce),
            ("checkbox-pop-duration", "ms", &mut self.checkbox_pop_duration),
            ("checkbox-radius-ratio", "", &mut self.checkbox_radius_ratio),
            ("checkbox-size", "px", &mut self.checkbox_size),
            ("checkbox-unmark-duration", "ms", &mut self.checkbox_unmark_duration),
            ("details-icon", "px", &mut self.details_icon),
            ("dpad-size", "px", &mut self.dpad_size),
            ("dropdown-option-row", "px", &mut self.dropdown_option_row),
            ("dropdown-row", "px", &mut self.dropdown_row),
            ("dropdown-width", "px", &mut self.dropdown_width),
            ("joystick-size", "px", &mut self.joystick_size),
            ("keyboard-candidate-height", "px", &mut self.keyboard_candidate_height),
            ("keyboard-key-height", "px", &mut self.keyboard_key_height),
            ("keyboard-key-min-height", "px", &mut self.keyboard_key_min_height),
            ("menu-open-duration", "ms", &mut self.menu_open_duration),
            ("modal-width", "px", &mut self.modal_width),
            ("radio-dot-ratio", "", &mut self.radio_dot_ratio),
            ("radio-pop-bounce", "", &mut self.radio_pop_bounce),
            ("radio-pop-duration", "ms", &mut self.radio_pop_duration),
            ("radio-size", "px", &mut self.radio_size),
            ("segmented-pad-ratio", "", &mut self.segmented_pad_ratio),
            ("slider-bubble-exit-duration", "ms", &mut self.slider_bubble_exit_duration),
            ("slider-bubble-height", "px", &mut self.slider_bubble_height),
            ("slider-knob", "px", &mut self.slider_knob),
            ("slider-knob-touch", "px", &mut self.slider_knob_touch),
            ("slider-rail", "px", &mut self.slider_rail),
            ("slider-rail-touch", "px", &mut self.slider_rail_touch),
            ("slider-tick-width", "px", &mut self.slider_tick_width),
            ("state-press-duration", "ms", &mut self.state_press_duration),
            ("switch-height-lg", "px", &mut self.switch_height_lg),
            ("switch-height-md", "px", &mut self.switch_height_md),
            ("switch-height-sm", "px", &mut self.switch_height_sm),
            ("switch-inset-ratio", "", &mut self.switch_inset_ratio),
            ("switch-width-ratio", "", &mut self.switch_width_ratio),
        ]
    }
}

pub const CONTROLS: Controls = Controls {
    action_size: 64.0,
    checkbox_mark_ratio: 0.78,
    checkbox_pop_bounce: 0.45,
    checkbox_pop_duration: 260.0,
    checkbox_radius_ratio: 0.24,
    checkbox_size: 22.0,
    checkbox_unmark_duration: 90.0,
    details_icon: 16.0,
    dpad_size: 132.0,
    dropdown_option_row: 40.0,
    dropdown_row: 36.0,
    dropdown_width: 240.0,
    joystick_size: 128.0,
    keyboard_candidate_height: 36.0,
    keyboard_key_height: 40.0,
    keyboard_key_min_height: 32.0,
    menu_open_duration: 140.0,
    modal_width: 420.0,
    radio_dot_ratio: 0.46,
    radio_pop_bounce: 0.4,
    radio_pop_duration: 240.0,
    radio_size: 20.0,
    segmented_pad_ratio: 0.08,
    slider_bubble_exit_duration: 90.0,
    slider_bubble_height: 26.0,
    slider_knob: 20.0,
    slider_knob_touch: 26.0,
    slider_rail: 4.0,
    slider_rail_touch: 6.0,
    slider_tick_width: 2.0,
    state_press_duration: 60.0,
    switch_height_lg: 32.0,
    switch_height_md: 28.0,
    switch_height_sm: 22.0,
    switch_inset_ratio: 0.11,
    switch_width_ratio: 1.72,
};

/// The `ui` transition's spring: perceptual duration (ms) and bounce.
pub const MOTION_UI: [f32; 2] = [300.0, 0.0];
/// The `panel` transition's spring: perceptual duration (ms) and bounce.
pub const MOTION_PANEL: [f32; 2] = [420.0, 0.1];
/// The `emphasis` transition's spring: perceptual duration (ms) and bounce.
pub const MOTION_EMPHASIS: [f32; 2] = [520.0, 0.3];
/// The `playful` transition's spring: perceptual duration (ms) and bounce.
pub const MOTION_PLAYFUL: [f32; 2] = [600.0, 0.45];
