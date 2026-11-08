//! A version 2 style's tokens (docs/ui.md §3.1): the named values every
//! component draws from, beside the 33 palette roles of [`crate::Palette`].
//!
//! - **Colour**: rarity tiers, eight team colours, damage and element
//!   types and eight chart series; each status (the palette's `success`,
//!   `warn`, `danger`, `info`) and rarity colour paired with a [`Cue`], an
//!   icon or a pattern, so no meaning is carried by colour alone (XAG 103).
//! - **Type**: semantic roles (hero, display, title, heading, body, label,
//!   caption, numeric, button) per [`Breakpoint`], with size, line height, tracking,
//!   weight and the font role it takes.
//! - **Space, size and radius** scales, **stroke** widths, **elevation**
//!   (a shadow set per level) and the **shape language** (a corner shape
//!   and radius per element class).
//! - **Motion**: named springs (duration and bounce), eases, durations,
//!   stagger and the overshoot the style allows.
//! - **Feedback**: per UI event of §7.8, a sound or a set chosen without
//!   repeats, volume and pitch ranges and a haptic pattern.
//!
//! All of it is `Copy` plain data the [`crate::Theme`] carries. The legacy
//! type ramp aliases this hierarchy at the frame's breakpoint; shape
//! classes give the radii and motion tokens the durations.
//!
//! The grammar is [`crate::style_file::parse_style`]'s; [`StyleTokens::derive`]
//! makes a complete set from a palette and scheme (the default style's,
//! and `og style upgrade`'s for a version 1 style), and
//! [`StyleTokens::entries`] writes one back as `.ogt` blocks.

use crate::layout::FontRole;
use crate::style::{Breakpoint, Durations, Srgba, Theme, TypeStep};
use crate::style_file::{P, Shadow, StylePart, parse_color};
use og_core::ogt::{Block, Entry, Number, Value};

macro_rules! names {
    ($(#[doc = $doc:literal])* $ty:ident { $($(#[doc = $d:literal])* $v:ident $n:literal,)* }) => {
        $(#[doc = $doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $ty {
            $($(#[doc = $d])* $v,)*
        }

        impl $ty {
            /// Every value, in file order.
            pub const ALL: &'static [$ty] = &[$($ty::$v,)*];
            /// How many there are.
            pub const COUNT: usize = [$($n,)*].len();

            /// Its name in a style file.
            pub fn name(self) -> &'static str {
                match self {
                    $($ty::$v => $n,)*
                }
            }

            pub fn from_name(name: &str) -> Option<$ty> {
                match name {
                    $($n => Some($ty::$v),)*
                    _ => None,
                }
            }

            pub fn index(self) -> usize {
                self as usize
            }

            /// Every name, comma separated (for messages).
            pub fn names() -> String {
                [$($n,)*].join(", ")
            }
        }
    };
}

pub(crate) use names;

names! {
    /// A status the palette colours (`success`, `warn`, `danger`, `info`).
    Status {
        Success "success",
        Warn "warn",
        Danger "danger",
        Info "info",
    }
}

names! {
    /// A rarity tier, lowest first.
    Rarity {
        Common "common",
        Uncommon "uncommon",
        Rare "rare",
        Epic "epic",
        Legendary "legendary",
        Mythic "mythic",
    }
}

names! {
    /// A damage or element type.
    DamageType {
        Physical "physical",
        Fire "fire",
        Ice "ice",
        Lightning "lightning",
        Poison "poison",
        Holy "holy",
        Shadow "shadow",
        Arcane "arcane",
    }
}

names! {
    /// A pattern a cue draws (in a badge, a frame's corner, a chart's fill).
    Pattern {
        Dots "dots",
        Stripes "stripes",
        Hatch "hatch",
        Crosshatch "crosshatch",
        Chevrons "chevrons",
        Grid "grid",
        Waves "waves",
        Diamonds "diamonds",
        Stars "stars",
        Checks "checks",
    }
}

names! {
    /// A step of the type scale.
    TypeRole {
        /// Title screens, results, the one big word.
        Display "display",
        /// Screen and dialog titles.
        Title "title",
        /// Section headings, card titles.
        Heading "heading",
        /// Running text.
        Body "body",
        /// Tabs, fields' labels, a HUD's dense labels and key hints.
        Label "label",
        /// Hints, metadata, footnotes.
        Caption "caption",
        /// Scores, timers, prices, counters.
        Numeric "numeric",
        /// A button's words; the `label` step when a style leaves it out
        /// (a pixel face sets it at body, where `label` is a HUD's 12).
        Button "button",
        /// The largest title or result, above display.
        Hero "hero",
    }
}

names! {
    /// A step of the space scale (the v1 `Space` names).
    SpaceStep {
        Xxs "xxs",
        Xs "xs",
        Sm "sm",
        Md "md",
        Lg "lg",
        Xl "xl",
        Xxl "xxl",
        Xl3 "xl3",
    }
}

names! {
    /// A step of the size scale (icons, avatars, thumbnails) and the
    /// smallest touch target.
    SizeStep {
        Xs "xs",
        Sm "sm",
        Md "md",
        Lg "lg",
        Xl "xl",
        Xxl "xxl",
        /// The smallest touch target.
        Touch "touch",
    }
}

names! {
    /// A step of the radius scale.
    RadiusStep {
        Xs "xs",
        Sm "sm",
        Md "md",
        Lg "lg",
        Xl "xl",
        /// Fully round ends.
        Pill "pill",
    }
}

names! {
    /// A stroke width.
    StrokeWidth {
        Hairline "hairline",
        Thin "thin",
        Regular "regular",
        Thick "thick",
        /// The focus ring.
        Focus "focus",
    }
}

names! {
    /// An elevation level, lowest first.
    Level {
        /// Cards and raised rows.
        Raised "raised",
        /// Menus, tooltips, popovers.
        Floating "floating",
        /// Dialogs and sheets.
        Overlay "overlay",
        /// Toasts and what is dragged.
        Top "top",
    }
}

names! {
    /// An element class the shape language gives a corner.
    ElementClass {
        Panel "panel",
        Dialog "dialog",
        Card "card",
        Tooltip "tooltip",
        Button "button",
        Field "field",
        Chip "chip",
        Badge "badge",
        Tab "tab",
        Slot "slot",
        Bar "bar",
        Hud "hud",
        Avatar "avatar",
    }
}

names! {
    /// How a corner is drawn.
    CornerShape {
        Round "round",
        /// A continuous (superellipse) corner.
        Squircle "squircle",
        /// A square notch cut into the corner.
        Notch "notch",
        /// A 45° cut.
        Bevel "bevel",
        Square "square",
    }
}

names! {
    /// A named spring of the motion personality (§7.7).
    SpringName {
        /// Hover, press, small state changes.
        Ui "ui",
        /// Panels, sheets, cards moving in.
        Panel "panel",
        /// Rewards, reveals, what should be noticed.
        Emphasis "emphasis",
        /// Casual flourishes.
        Playful "playful",
    }
}

names! {
    /// A style role selecting a shared timing curve, not a curve name.
    EaseRole {
        Standard "standard",
        Enter "enter",
        Exit "exit",
        Emphasis "emphasis",
        /// Built-in widget arrivals, meter trails and cascades.
        WidgetEnter "widgetEnter",
        /// Built-in widget departures.
        WidgetExit "widgetExit",
    }
}

names! {
    /// A UI event that plays a sound and a haptic (§7.8).
    SoundEvent {
        Hover "hover",
        Press "press",
        Focus "focus",
        Select "select",
        Confirm "confirm",
        Cancel "cancel",
        Back "back",
        Open "open",
        Close "close",
        Tab "tab",
        ToggleOn "toggleOn",
        ToggleOff "toggleOff",
        SliderTick "sliderTick",
        ScrollTick "scrollTick",
        Typewriter "typewriter",
        CounterTick "counterTick",
        Notification "notification",
        Warning "warning",
        Error "error",
        Success "success",
        Reward "reward",
        RevealCommon "revealCommon",
        RevealUncommon "revealUncommon",
        RevealRare "revealRare",
        RevealEpic "revealEpic",
        RevealLegendary "revealLegendary",
        RevealMythic "revealMythic",
    }
}

names! {
    /// A named haptic pattern.
    HapticPreset {
        /// A light click: slider steps, selection.
        Tick "tick",
        /// A firm click: confirm.
        Tap "tap",
        /// A heavy, low knock: an impact, a big reveal.
        Thud "thud",
        Double "double",
        /// A sustained buzz.
        Buzz "buzz",
        Success "success",
        Warning "warning",
        Error "error",
    }
}

/// Team and player colours.
pub const TEAMS: usize = 8;
/// Chart series colours.
pub const SERIES: usize = 8;
/// Sounds a feedback set chooses from.
pub const SET_SOUNDS: usize = 4;
/// Pulses in a haptic pattern.
pub const PULSES: usize = 4;
/// Shadows in an elevation level.
pub const LEVEL_SHADOWS: usize = 3;

/// What carries a colour's meaning besides its hue.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Cue {
    /// Colour only (allowed for teams and damage types, never a status or rarity).
    #[default]
    None,
    /// A cell of the style's icon atlas, by its index in the `icons` names.
    Icon(u16),
    Pattern(Pattern),
    /// A count of marks (stars, gems): rarity as a number.
    Pips(u8),
}

/// A colour and its cue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Swatch {
    pub color: Srgba,
    pub cue: Cue,
}

/// The colour tokens beyond the palette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTokens {
    /// The cue of each palette status colour.
    pub status: [Cue; Status::COUNT],
    pub rarity: [Swatch; Rarity::COUNT],
    pub team: [Swatch; TEAMS],
    pub damage: [Swatch; DamageType::COUNT],
    pub series: [Srgba; SERIES],
}

/// One step of the type scale at one breakpoint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeToken {
    /// CSS px.
    pub size: f32,
    /// CSS px.
    pub line_height: f32,
    /// Latin tracking, em.
    pub tracking_em: f32,
    /// Tracking for text containing CJK characters, em.
    pub cjk_tracking_em: f32,
    /// 100 to 900, as CSS.
    pub weight: u16,
    pub font: FontRole,
    /// OpenType features of the role (`… display smcp tnum`).
    pub features: crate::shape::Features,
    /// `case upper`: uppercase in the active locale before measurement.
    pub uppercase: bool,
}

impl TypeToken {
    /// The renderer's step, scaled once by the player's text preference.
    pub fn step(self, scale: f32) -> TypeStep {
        let size = self.size * scale;
        let line_height = self.line_height * scale;
        TypeStep {
            size,
            line_height,
            tracking_em: self.tracking_em,
            cjk_line_height: line_height.max((size * 1.45).round()),
            cjk_tracking_em: self.cjk_tracking_em,
            features: self.features,
            weight: self.weight,
            uppercase: self.uppercase,
        }
    }
}

impl ColorTokens {
    /// The cue of colour token `name` (`danger`, `epic`, `team3`, `fire`;
    /// [`colour_token_names`]), or `None` for no such token or a series.
    pub fn cue(&self, name: &str) -> Option<Cue> {
        if let Some(s) = Status::from_name(name) {
            return Some(self.status[s.index()]);
        }
        if let Some(r) = Rarity::from_name(name) {
            return Some(self.rarity[r.index()].cue);
        }
        if let Some(d) = DamageType::from_name(name) {
            return Some(self.damage[d.index()].cue);
        }
        let k = name.strip_prefix("team")?.parse::<usize>().ok()?;
        (1..=TEAMS).contains(&k).then(|| self.team[k - 1].cue)
    }
}

/// The type scale: every [`TypeRole`] at every [`Breakpoint`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeScale {
    pub steps: [[TypeToken; Breakpoint::ALL.len()]; TypeRole::COUNT],
}

impl TypeScale {
    pub fn get(&self, role: TypeRole, at: Breakpoint) -> TypeToken {
        self.steps[role.index()][at as usize]
    }

    /// Compatibility names of the single semantic hierarchy (§17.4).
    pub fn ramp(&self, at: Breakpoint, scale: f32) -> crate::TypeRamp {
        use TypeRole::*;
        let step = |role| self.get(role, at).step(scale);
        crate::TypeRamp {
            xs: step(Caption),
            sm: step(Label),
            base: step(Body),
            body: step(Body),
            caption: step(Caption),
            meta: step(Caption),
            reading: step(Body),
            lg: step(Numeric),
            xl: step(Heading),
            xl2: step(Heading),
            xl3: step(Title),
            xl4: step(Title),
            xl5: step(Display),
            xl6: step(Display),
            xl7: step(Hero),
            xl8: step(Hero),
            display_sm: step(Title),
            display_md: step(Title),
            display_lg: step(Display),
            display_xl: step(Hero),
            display_xl2: step(Hero),
        }
    }
}

/// A level's shadows, drawn under the element in order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Elevation {
    pub shadows: [Shadow; LEVEL_SHADOWS],
    /// How many of `shadows` are set (0: flat).
    pub count: u8,
}

impl Elevation {
    pub const FLAT: Elevation = Elevation {
        shadows: [Shadow {
            dx: 0.0,
            dy: 0.0,
            blur: 0.0,
            spread: 0.0,
            color: Srgba::TRANSPARENT,
        }; LEVEL_SHADOWS],
        count: 0,
    };

    pub fn shadows(&self) -> &[Shadow] {
        &self.shadows[..usize::from(self.count)]
    }
}

/// An element class's corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeToken {
    pub corner: CornerShape,
    /// The radius (round, squircle) or the cut (notch, bevel), CSS px.
    pub radius: f32,
}

/// A spring as a designer sets it: a perceptual duration and a bounce.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringToken {
    /// ms.
    pub duration_ms: f32,
    /// −1 to 1: 0 settles without overshoot, above 0 overshoots, below 0
    /// is overdamped.
    pub bounce: f32,
}

impl SpringToken {
    /// The damping ratio and response the motion code takes.
    pub fn to_spring(self) -> crate::Spring {
        let damping = if self.bounce >= 0.0 {
            1.0 - self.bounce
        } else {
            1.0 / (1.0 + self.bounce)
        };
        crate::Spring {
            damping,
            response_ms: self.duration_ms,
        }
    }

    /// The spring of a damping ratio and response.
    pub fn from_spring(s: crate::Spring) -> SpringToken {
        let bounce = if s.damping <= 1.0 {
            1.0 - s.damping
        } else {
            1.0 / s.damping - 1.0
        };
        SpringToken {
            duration_ms: s.response_ms,
            bounce: round_to(bounce, 1000.0),
        }
    }

    /// How far past its target it swings, as a fraction of the move.
    pub fn overshoot(self) -> f32 {
        let z = self.to_spring().damping;
        if z >= 1.0 {
            0.0
        } else {
            (-std::f32::consts::PI * z / (1.0 - z * z).sqrt()).exp()
        }
    }
}

pub use og_tween::Ease;

/// The motion personality.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionTokens {
    pub springs: [SpringToken; SpringName::COUNT],
    pub eases: [Ease; EaseRole::COUNT],
    pub durations: Durations,
    /// ms between the entrances of a list's items.
    pub stagger_ms: f32,
    /// The most a stagger may add up to, ms.
    pub stagger_cap_ms: f32,
    /// The overshoot every spring of the style stays within (0–1).
    pub overshoot: f32,
}

/// One pulse of a haptic pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pulse {
    /// From the event, ms.
    pub start_ms: f32,
    pub ms: f32,
    /// The low-frequency (heavy) motor, 0–1.
    pub low: f32,
    /// The high-frequency (light) motor, 0–1; a phone's one motor takes the larger.
    pub high: f32,
}

/// A haptic pattern: up to four pulses.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Haptic {
    pub pulses: [Pulse; PULSES],
    pub count: u8,
}

impl Haptic {
    pub const NONE: Haptic = Haptic {
        pulses: [Pulse {
            start_ms: 0.0,
            ms: 0.0,
            low: 0.0,
            high: 0.0,
        }; PULSES],
        count: 0,
    };

    pub fn pulses(&self) -> &[Pulse] {
        &self.pulses[..usize::from(self.count)]
    }

    fn of(p: &[(f32, f32, f32, f32)]) -> Haptic {
        let mut h = Haptic::NONE;
        for (k, &(start_ms, ms, low, high)) in p.iter().enumerate() {
            h.pulses[k] = Pulse {
                start_ms,
                ms,
                low,
                high,
            };
        }
        h.count = p.len() as u8;
        h
    }

    pub fn preset(p: HapticPreset) -> Haptic {
        match p {
            HapticPreset::Tick => Haptic::of(&[(0.0, 10.0, 0.0, 0.35)]),
            HapticPreset::Tap => Haptic::of(&[(0.0, 18.0, 0.2, 0.6)]),
            HapticPreset::Thud => Haptic::of(&[(0.0, 60.0, 0.9, 0.3)]),
            HapticPreset::Double => Haptic::of(&[(0.0, 18.0, 0.2, 0.6), (70.0, 18.0, 0.2, 0.6)]),
            HapticPreset::Buzz => Haptic::of(&[(0.0, 120.0, 0.5, 0.5)]),
            HapticPreset::Success => Haptic::of(&[(0.0, 20.0, 0.1, 0.5), (90.0, 30.0, 0.3, 0.8)]),
            HapticPreset::Warning => Haptic::of(&[(0.0, 40.0, 0.5, 0.5), (100.0, 40.0, 0.5, 0.5)]),
            HapticPreset::Error => Haptic::of(&[
                (0.0, 60.0, 0.8, 0.4),
                (90.0, 60.0, 0.8, 0.4),
                (180.0, 60.0, 0.8, 0.4),
            ]),
        }
    }
}

/// What a UI event plays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Feedback {
    /// Sound handles the loader filled (0: not loaded); the style file's
    /// asset ids are [`crate::StyleFile::feedback_sounds`].
    pub sounds: [u64; SET_SOUNDS],
    /// How many sounds the set has (0: silent); more than one plays one
    /// at random, never the same twice in a row.
    pub count: u8,
    /// The volume range a play picks from (0–1).
    pub volume: [f32; 2],
    /// The pitch range a play picks from (1: as recorded).
    pub pitch: [f32; 2],
    pub haptic: Haptic,
}

impl Feedback {
    pub const SILENT: Feedback = Feedback {
        sounds: [0; SET_SOUNDS],
        count: 0,
        volume: [1.0, 1.0],
        pitch: [1.0, 1.0],
        haptic: Haptic::NONE,
    };
}

/// Everything §12.2 names, complete.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyleTokens {
    pub colors: ColorTokens,
    pub typography: TypeScale,
    /// By [`SpaceStep`], CSS px.
    pub space: [f32; SpaceStep::COUNT],
    /// By [`SizeStep`], CSS px.
    pub size: [f32; SizeStep::COUNT],
    /// By [`RadiusStep`], CSS px.
    pub radius: [f32; RadiusStep::COUNT],
    /// By [`StrokeWidth`], CSS px.
    pub stroke: [f32; StrokeWidth::COUNT],
    /// By [`Level`].
    pub elevation: [Elevation; Level::COUNT],
    /// By [`ElementClass`].
    pub shape: [ShapeToken; ElementClass::COUNT],
    pub motion: MotionTokens,
    /// By [`SoundEvent`].
    pub feedback: [Feedback; SoundEvent::COUNT],
}

fn round_to(v: f32, k: f32) -> f32 {
    (v * k).round() / k
}

mod convert;
mod parse;
pub use convert::*;
pub use parse::colour_token_names;
pub(crate) use parse::*;
