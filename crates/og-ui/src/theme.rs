//! The engine's default style: the colour roles of a game UI and the values
//! of the design language the engine shares with origin-agent
//! (`packages/design`, docs/ui.md §12.1): the neutral ladder, a
//! neutral primary set apart by fill and weight (2026-11-02), hue only where
//! it means something, flat glass on floating chrome only, and borderless
//! controls told apart by tone.
//!
//! **Generated** from `packages/design/tokens.json` by
//! `crates/og-ui/scripts/gen-theme.sh`; never edit it by hand (`tests/theme.rs`
//! fails on drift). The default style is for origin-agent's surfaces and for
//! prototyping: every game ships its own style (`crate::style_file`, `type
//! style`, usually the one its look names), and `og build --publish`
//! refuses a game still drawn in this one.
//!
//! Lengths are og-ui units (CSS pixels at a unit scale of 1), durations
//! milliseconds, colours sRGB as authored (straight alpha); the translucent
//! fills are the tokens' `linear` values, restated for the UI pass's
//! linear-light blending. `crate::style` converts them for the renderer.

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
/// motion block overrides each (docs/ui.md §7.7).
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

/// The default style's part dressings (docs/ui.md §12.3), in a style
/// file's grammar: flat glass on floating chrome only (`thick` on what
/// carries text over the game, `regular` on HUD capsules and hints), its
/// own hairline and no second border; content flat (cards keep their
/// hairline and cast no shadow); controls borderless, a fill told apart by
/// tone, the focus ring the only outline. One selection grammar, all
/// neutral (2026-11-02): a selected tab is a raised chip (lineStrong, a
/// lit top edge and a short shadow, its label semibold), a selected tile a
/// light rim with a glow and a lift, a checked control the primary's fill,
/// a chosen radio card the raised chip's fill with no ring; a switch,
/// checkbox or radio row draws no outline and no lit top edge, a radio
/// card no hairline (art r4, note 4);
/// a slider's remainder holds 3:1 on the panel. `crate::theme_dressings`
/// parses them.
pub const DRESSINGS: &str = r##"
dress "Panel.frame" {
  fill none
  stroke none
  effect backdrop glass thick elevation lift
}
dress "Sheet.frame" {
  fill none
  stroke none
  effect backdrop glass thick elevation lift
}
dress "Radial.frame" {
  fill none
  stroke none
  effect backdrop glass thick elevation lift
}
dress "Menu.frame" {
  fill none
  stroke none
  effect backdrop glass thick elevation float
}
dress "Popover.frame" {
  fill none
  stroke none
  effect backdrop glass thick elevation float
}
dress "Tooltip.frame" {
  fill none
  stroke none
  effect backdrop glass thick elevation float
}
dress "Chart.tooltip" {
  fill none
  stroke none
  effect backdrop glass thick elevation float
}
dress "Hud.frame" {
  fill none
  stroke none
  effect backdrop glass regular elevation float
}
dress "Minimap.frame" {
  fill "#16161899"
  stroke 0.5 "#ffffff14" inside
  effect dropShadow 0 6 16 0 "#00000073"
}
dress "ScrollArea.frame" {
  stroke none
  effect none
}
dress "ActionBar.hint" {
  fill none
  stroke none
  effect backdrop glass regular elevation float
}
dress "Card.frame" {
  effect none
  shape round 10
}
dress "Button.base" {
  stroke none
  effect none
}
dress "Button.base[ghost]" {
  stroke 1 lineStrong inside
}
dress "Field.frame" {
  stroke none
  effect none
}
dress "Field.frame[invalid]" {
  stroke 1 danger inside
}
dress "Chip.base" {
  stroke none
}
dress "Slot.frame" {
  stroke none
  effect none
}
dress "Slot.frame[selected]" {
  stroke 1 "#ffffff8c" inside
  effect outerGlow 8 "#ffffff1f"
  effect dropShadow 0 6 14 0 "#00000066"
}
dress "Meter.track" {
  effect none
}
dress "Meter.fill" {
  effect none
}
dress "Slider.track" {
  fill "#707075"
  effect none
}
dress "Toggle.track" {
  effect none
}
dress "Toggle.row" {
  stroke none
  effect none
}
dress "Checkbox.row" {
  stroke none
  effect none
}
dress "Radio.option" {
  stroke none
  effect none
}
dress "Radio.card" {
  stroke none
}
dress "Radio.card[selected]" {
  fill selection
}
dress "Radio.chosen" {
  stroke none
}
dress "Tabs.tab[selected]" {
  fill selection
  effect dropShadow 0 2 4 0 "#00000073"
  effect innerShadow 0 1 0 0 "#ffffff26"
}
"##;
