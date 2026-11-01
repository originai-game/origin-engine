//! A version 2 style's tokens (docs/ui.md §12.2): the named values every
//! component draws from, beside the 33 palette roles of [`crate::Palette`].
//!
//! - **Colour**: rarity tiers, eight team colours, damage and element
//!   types and eight chart series; each status (the palette's `success`,
//!   `warn`, `danger`, `info`) and rarity colour paired with a [`Cue`], an
//!   icon or a pattern, so no meaning is carried by colour alone (XAG 103).
//! - **Type**: seven steps (display, title, heading, body, label, caption,
//!   numeric) per [`Breakpoint`], each with size, line height, tracking,
//!   weight and the font role it takes.
//! - **Space, size and radius** scales, **stroke** widths, **elevation**
//!   (a shadow set per level) and the **shape language** (a corner shape
//!   and radius per element class).
//! - **Motion**: named springs (duration and bounce), eases, durations,
//!   stagger and the overshoot the style allows.
//! - **Feedback**: per UI event of §7.8, a sound or a set chosen without
//!   repeats, volume and pitch ranges and a haptic pattern.
//!
//! All of it is `Copy` plain data the [`crate::Theme`] carries. Version 2
//! tokens are carried, not yet consumed: the widgets still read the v1
//! values (`palette`, `radius`, `type_ramp`, `durations_ms`, `art`), which a
//! version 2 file sets from the same text (its shape classes give the
//! radii, its motion durations the durations).
//!
//! The grammar is [`crate::style_file::parse_style`]'s; [`StyleTokens::derive`]
//! makes a complete set from a palette and scheme (the prototype style's,
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
        /// Buttons, tabs, fields' labels.
        Label "label",
        /// Hints, metadata, footnotes.
        Caption "caption",
        /// Scores, timers, prices, counters.
        Numeric "numeric",
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
    /// A named ease.
    EaseName {
        Standard "standard",
        Enter "enter",
        Exit "exit",
        Emphasis "emphasis",
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
    /// 100 to 900, as CSS.
    pub weight: u16,
    pub font: FontRole,
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

/// An ease.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ease {
    /// A cubic Bézier's control points (x1, y1, x2, y2); `linear` is 0 0 1 1.
    Bezier([f32; 4]),
    /// Held on whole steps (pixel styles).
    Steps(u8),
}

/// The motion personality.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionTokens {
    pub springs: [SpringToken; SpringName::COUNT],
    pub eases: [Ease; EaseName::COUNT],
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

impl StyleTokens {
    pub fn space(&self, s: SpaceStep) -> f32 {
        self.space[s.index()]
    }
    pub fn size(&self, s: SizeStep) -> f32 {
        self.size[s.index()]
    }
    pub fn radius(&self, s: RadiusStep) -> f32 {
        self.radius[s.index()]
    }
    pub fn stroke(&self, s: StrokeWidth) -> f32 {
        self.stroke[s.index()]
    }
    pub fn elevation(&self, l: Level) -> &Elevation {
        &self.elevation[l.index()]
    }
    pub fn shape(&self, c: ElementClass) -> ShapeToken {
        self.shape[c.index()]
    }
    pub fn spring(&self, s: SpringName) -> SpringToken {
        self.motion.springs[s.index()]
    }
    pub fn ease(&self, e: EaseName) -> Ease {
        self.motion.eases[e.index()]
    }
    pub fn feedback(&self, e: SoundEvent) -> &Feedback {
        &self.feedback[e.index()]
    }
    pub fn rarity(&self, r: Rarity) -> Swatch {
        self.colors.rarity[r.index()]
    }

    /// A complete set derived from a theme's v1 values: its palette and
    /// scheme (the colours, read on its panel), its radii (the shape
    /// language), its ramp (the type scale), its durations and the motion
    /// and shadows of its art. `icons` are the style's icon names (a status
    /// takes an icon cue when the set has a fitting one, a pattern
    /// otherwise). The prototype style's tokens are this of its palette;
    /// `og style upgrade` writes this for a version 1 style.
    pub fn derive(t: &Theme, icons: &[String]) -> StyleTokens {
        StyleTokens::derive_from(
            V1 {
                palette: &t.palette,
                space: &t.space,
                radius: &t.radius,
                type_ramp: &t.type_ramp,
                durations_ms: t.durations_ms,
                art: &t.art,
            },
            icons,
        )
    }

    pub(crate) fn derive_from(t: V1, icons: &[String]) -> StyleTokens {
        let p = t.palette;
        let ground = p.panel_raised.over(p.backdrop);
        // A non-text colour reads 3:1 on the panel (WCAG 1.4.11): the seed
        // keeps its hue and moves along its own lightness until it does.
        let read = |hex: &str| {
            let c = parse_color(hex).expect("a seed colour");
            c.readable_on(ground, 3.0).unwrap_or(p.ink)
        };
        let icon = |name: &str, or: Pattern| {
            icons
                .iter()
                .position(|n| n == name)
                .map_or(Cue::Pattern(or), |i| Cue::Icon(i as u16))
        };
        let swatch = |hex: &str, cue: Cue| Swatch {
            color: read(hex),
            cue,
        };
        let colors = ColorTokens {
            status: [
                icon("check", Pattern::Dots),
                icon("bolt", Pattern::Stripes),
                icon("close", Pattern::Crosshatch),
                Cue::Pattern(Pattern::Waves),
            ],
            rarity: [
                swatch("#9e9e9e", Cue::Pips(1)),
                swatch("#3fbf4f", Cue::Pips(2)),
                swatch("#2f7fef", Cue::Pips(3)),
                swatch("#a335ee", Cue::Pips(4)),
                swatch("#ff8000", Cue::Pips(5)),
                swatch("#e8364f", Cue::Pips(6)),
            ],
            team: [
                "#3d7eff", "#ff4747", "#3fbf5f", "#ffc61a", "#a347ff", "#ff8a1f", "#1fc7c7",
                "#ff5cb0",
            ]
            .map(|h| swatch(h, Cue::None)),
            damage: [
                "#b8b0a0", "#ff6a2b", "#62cfff", "#ffd83a", "#7ed957", "#ffe9a0", "#8a5cf6",
                "#ff4fd8",
            ]
            .map(|h| swatch(h, Cue::None)),
            series: [
                "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948", "#b07aa1",
                "#ff9da7",
            ]
            .map(read),
        };
        let r = t.type_ramp;
        let step = |s: TypeStep, weight: u16, font: FontRole, phone: f32| {
            let desk = TypeToken {
                size: s.size,
                line_height: s.line_height,
                tracking_em: s.tracking_em,
                weight,
                font,
            };
            let small = |k: f32| TypeToken {
                size: (s.size * k).round().max(12.0),
                line_height: (s.line_height * k).round().max(16.0),
                ..desk
            };
            // phonePortrait, phoneLandscape, tablet, desktop.
            [small(phone), small((1.0 + phone) / 2.0), desk, desk]
        };
        let typography = TypeScale {
            steps: [
                step(r.display_lg, 700, FontRole::Display, 0.8),
                step(r.display_sm, 700, FontRole::Display, 0.875),
                step(r.xl, 600, FontRole::Display, 1.0),
                step(r.body, 400, FontRole::Body, 1.0),
                // A button's and a tab's label: version 1 set them in body.
                step(r.body, 600, FontRole::Body, 1.0),
                step(r.caption, 400, FontRole::Body, 1.0),
                step(r.lg, 600, FontRole::Mono, 1.0),
            ],
        };
        let s = t.space;
        let rad = t.radius;
        let shadow = |dy: f32, blur: f32, a: f32| Shadow {
            dx: 0.0,
            dy,
            blur,
            spread: 0.0,
            // Quantised to a byte, as a file writes it.
            color: parse_color(&hex(p.shadow.fade(a))).expect("a colour"),
        };
        let level = |s: Option<Shadow>, or: Shadow| {
            let mut e = Elevation::FLAT;
            e.shadows[0] = s.unwrap_or(or);
            e.count = 1;
            e
        };
        let art = t.art;
        let overlay = level(
            art.shadows[StylePart::Panel.index()],
            shadow(12.0, 32.0, 0.45),
        );
        let mut top = overlay;
        top.shadows[0].dy = (top.shadows[0].dy * 1.5).round();
        top.shadows[0].blur = (top.shadows[0].blur * 1.5).round();
        let elevation = [
            level(art.shadows[StylePart::Card.index()], shadow(1.0, 3.0, 0.3)),
            level(
                art.shadows[StylePart::Tooltip.index()].or(art.shadows[StylePart::Hud.index()]),
                shadow(4.0, 12.0, 0.35),
            ),
            overlay,
            top,
        ];
        let corner = |radius: f32| ShapeToken {
            corner: if radius <= 0.0 {
                CornerShape::Square
            } else {
                CornerShape::Round
            },
            radius,
        };
        let shape = ElementClass::map_all(|c| {
            corner(match c {
                ElementClass::Panel
                | ElementClass::Dialog
                | ElementClass::Card
                | ElementClass::Tooltip => rad.panel,
                ElementClass::Button
                | ElementClass::Field
                | ElementClass::Chip
                | ElementClass::Badge
                | ElementClass::Tab
                | ElementClass::Hud => rad.control,
                ElementClass::Slot | ElementClass::Avatar => rad.small,
                ElementClass::Bar => rad.bar,
            })
        });
        let d = t.durations_ms;
        let springs = [
            SpringToken {
                duration_ms: d.fast,
                bounce: 0.0,
            },
            SpringToken {
                duration_ms: d.base,
                bounce: 0.0,
            },
            SpringToken {
                duration_ms: d.slow,
                bounce: 0.15,
            },
            SpringToken {
                duration_ms: d.spring,
                bounce: 0.3,
            },
        ];
        let mut overshoot = springs
            .iter()
            .map(|s| s.overshoot())
            .chain(
                [art.motion.hover_spring, art.motion.press_spring]
                    .into_iter()
                    .flatten()
                    .map(|s| SpringToken::from_spring(s).overshoot()),
            )
            .fold(0.0f32, f32::max);
        overshoot = ((overshoot * 20.0).ceil() / 20.0).min(1.0);
        let motion = MotionTokens {
            springs,
            eases: [
                Ease::Bezier([0.2, 0.0, 0.0, 1.0]),
                Ease::Bezier(crate::theme::EASE_OUT),
                Ease::Bezier(crate::theme::EASE_EXIT),
                Ease::Bezier([0.34, 1.4, 0.64, 1.0]),
            ],
            durations: d,
            stagger_ms: art.motion.stagger_ms,
            stagger_cap_ms: 300.0,
            overshoot,
        };
        let haptic = |e: SoundEvent| {
            use HapticPreset as H;
            use SoundEvent as E;
            match e {
                E::Press | E::Select | E::Cancel | E::Back | E::Tab => Some(H::Tick),
                E::ToggleOn | E::ToggleOff | E::SliderTick | E::RevealUncommon => Some(H::Tick),
                E::Confirm | E::Notification | E::RevealRare => Some(H::Tap),
                E::Warning => Some(H::Warning),
                E::Error => Some(H::Error),
                E::Success => Some(H::Success),
                E::Reward | E::RevealEpic => Some(H::Double),
                E::RevealLegendary => Some(H::Thud),
                E::RevealMythic => Some(H::Buzz),
                _ => None,
            }
            .map_or(Haptic::NONE, Haptic::preset)
        };
        let feedback = SoundEvent::map_all(|e| Feedback {
            haptic: haptic(e),
            ..Feedback::SILENT
        });
        StyleTokens {
            colors,
            typography,
            space: [s.xxs, s.xs, s.sm, s.md, s.lg, s.xl, s.xxl, s.xl3],
            size: [16.0, 24.0, 32.0, 48.0, 64.0, 96.0, 44.0],
            radius: [
                rad.small,
                rad.control,
                rad.panel,
                (rad.panel * 1.5).round(),
                (rad.panel * 2.0).round(),
                9999.0,
            ],
            stroke: [1.0, 1.5, 2.0, 3.0, 2.0],
            elevation,
            shape,
            motion,
            feedback,
        }
    }
}

macro_rules! map_all {
    ($($ty:ident),*) => {$(
        impl $ty {
            /// `f` of every value, in order.
            pub fn map_all<U>(mut f: impl FnMut($ty) -> U) -> [U; $ty::COUNT] {
                std::array::from_fn(|i| f($ty::ALL[i]))
            }
        }
    )*};
}
map_all!(ElementClass, SoundEvent);

/// The version 1 values a token set is derived from.
pub(crate) struct V1<'a> {
    pub palette: &'a crate::Palette,
    pub space: &'a crate::Space,
    pub radius: &'a crate::Radius,
    pub type_ramp: &'a crate::TypeRamp,
    pub durations_ms: Durations,
    pub art: &'a crate::Art,
}

fn round_to(v: f32, k: f32) -> f32 {
    (v * k).round() / k
}

// ───────────────────────────────────────────────────────────── parsing

/// A cue's icon, by name until the icons are known.
pub(crate) enum CueRef {
    None,
    Icon(String),
    Pattern(Pattern),
    Pips(u8),
}

/// Reads a cue at `values[at..]` (`icon "<name>"`, `pattern <word>`,
/// `pips <n>`), or none when nothing follows.
fn cue_at(p: &P, at: usize) -> Result<CueRef, String> {
    if p.e.values.len() <= at {
        return Ok(CueRef::None);
    }
    let cue =
        match p.word(at)? {
            "icon" => CueRef::Icon(p.string(at + 1)?.to_string()),
            "pattern" => {
                let w = p.word(at + 1)?;
                CueRef::Pattern(Pattern::from_name(w).ok_or_else(|| {
                    p.err(&format!("names the pattern '{w}' ({})", Pattern::names()))
                })?)
            }
            "pips" => {
                let n = p.num(at + 1)?;
                if !(1.0..=9.0).contains(&n) || n.fract() != 0.0 {
                    return Err(p.err("takes pips 1 to 9"));
                }
                CueRef::Pips(n as u8)
            }
            w => {
                return Err(p.err(&format!(
                "pairs its colour with a cue: icon \"<name>\", pattern <{}> or pips <n>, not '{w}'",
                Pattern::names().replace(", ", "|")
            )));
            }
        };
    let used = if matches!(cue, CueRef::None) { 0 } else { 2 };
    if p.e.values.len() > at + used {
        return Err(p.err("takes a colour and one cue"));
    }
    Ok(cue)
}

fn resolve_cue(p: &P, cue: CueRef, icons: &[String]) -> Result<Cue, String> {
    Ok(match cue {
        CueRef::None => Cue::None,
        CueRef::Pattern(x) => Cue::Pattern(x),
        CueRef::Pips(n) => Cue::Pips(n),
        CueRef::Icon(name) => Cue::Icon(icons.iter().position(|n| *n == name).ok_or_else(|| {
            if icons.is_empty() {
                p.err(&format!(
                    "names the icon \"{name}\", but the style has no `icons` names"
                ))
            } else {
                p.err(&format!(
                    "names the icon \"{name}\", which is not one of the style's icons"
                ))
            }
        })? as u16),
    })
}

fn missing_err(p: &P, missing: &[String], what: &str) -> Result<(), String> {
    if missing.is_empty() {
        Ok(())
    } else {
        Err(p.err(&format!("leaves out {}: {what}", missing.join(", "))))
    }
}

/// The `colors { … }` block.
pub(crate) fn parse_colors(p: &P, icons: &[String]) -> Result<ColorTokens, String> {
    let block = p.block()?;
    let black = Swatch {
        color: Srgba::BLACK,
        cue: Cue::None,
    };
    let mut c = ColorTokens {
        status: [Cue::None; Status::COUNT],
        rarity: [black; Rarity::COUNT],
        team: [black; TEAMS],
        damage: [black; DamageType::COUNT],
        series: [Srgba::BLACK; SERIES],
    };
    let mut seen: Vec<&str> = Vec::new();
    for e in &block.entries {
        let q = P { e };
        if seen.contains(&e.key.as_str()) {
            return Err(q.err("is set twice"));
        }
        seen.push(&e.key);
        let key = e.key.as_str();
        let numbered = |prefix: &str, n: usize| {
            key.strip_prefix(prefix)
                .and_then(|k| k.parse::<usize>().ok())
                .filter(|k| (1..=n).contains(k))
                .map(|k| k - 1)
        };
        if let Some(s) = Status::from_name(key) {
            let cue = resolve_cue(&q, cue_at(&q, 0)?, icons)?;
            if cue == Cue::None {
                return Err(q.err(
                    "takes the status colour's cue (icon \"<name>\", pattern <word> or pips <n>): the colour is the palette's",
                ));
            }
            c.status[s.index()] = cue;
        } else if let Some(r) = Rarity::from_name(key) {
            let color = q.color(0)?;
            let cue = resolve_cue(&q, cue_at(&q, 1)?, icons)?;
            if cue == Cue::None {
                return Err(q.err(&format!(
                    "pairs the rarity colour with a cue, so no tier is told by colour alone: `{key} \"#rrggbb\" pips {}`",
                    r.index() + 1
                )));
            }
            c.rarity[r.index()] = Swatch { color, cue };
        } else if let Some(d) = DamageType::from_name(key) {
            c.damage[d.index()] = Swatch {
                color: q.color(0)?,
                cue: resolve_cue(&q, cue_at(&q, 1)?, icons)?,
            };
        } else if let Some(k) = numbered("team", TEAMS) {
            c.team[k] = Swatch {
                color: q.color(0)?,
                cue: resolve_cue(&q, cue_at(&q, 1)?, icons)?,
            };
        } else if let Some(k) = numbered("series", SERIES) {
            if e.values.len() != 1 {
                return Err(q.err("takes one colour"));
            }
            c.series[k] = q.color(0)?;
        } else {
            return Err(q.err(&format!(
                "is not a colour token: a status ({}), a rarity ({}), team1 to team{TEAMS}, a damage type ({}) or series1 to series{SERIES}",
                Status::names(),
                Rarity::names(),
                DamageType::names()
            )));
        }
    }
    let want: Vec<String> = Status::ALL
        .iter()
        .map(|s| s.name().to_string())
        .chain(Rarity::ALL.iter().map(|r| r.name().to_string()))
        .chain((1..=TEAMS).map(|k| format!("team{k}")))
        .chain(DamageType::ALL.iter().map(|d| d.name().to_string()))
        .chain((1..=SERIES).map(|k| format!("series{k}")))
        .filter(|n| !seen.contains(&n.as_str()))
        .collect();
    missing_err(p, &want, "a style sets every colour token")?;
    for (group, cues) in [
        ("status", c.status.to_vec()),
        ("rarity", c.rarity.iter().map(|s| s.cue).collect()),
    ] {
        for (i, a) in cues.iter().enumerate() {
            if let Some(j) = cues[..i].iter().position(|b| b == a) {
                let name = |k: usize| {
                    if group == "status" {
                        Status::ALL[k].name()
                    } else {
                        Rarity::ALL[k].name()
                    }
                };
                return Err(p.err(&format!(
                    "gives {} and {} the same cue: each {group} needs its own, so colour is never the only difference",
                    name(j),
                    name(i)
                )));
            }
        }
    }
    Ok(c)
}

/// `<role> <size> <lineHeight> <tracking em> <weight> <font> [{ <breakpoint> <size> <lineHeight> … }]`.
pub(crate) fn parse_typography(p: &P) -> Result<TypeScale, String> {
    let block = p.block()?;
    let zero = TypeToken {
        size: 0.0,
        line_height: 0.0,
        tracking_em: 0.0,
        weight: 400,
        font: FontRole::Body,
    };
    let mut scale = TypeScale {
        steps: [[zero; 4]; TypeRole::COUNT],
    };
    let mut seen = Vec::new();
    for e in &block.entries {
        let q = P { e };
        let role = TypeRole::from_name(&e.key)
            .ok_or_else(|| q.err(&format!("is not a type step ({})", TypeRole::names())))?;
        if seen.contains(&role) {
            return Err(q.err("is set twice"));
        }
        seen.push(role);
        let size = q.num(0)?;
        let line_height = q.num(1)?;
        let tracking_em = q.num(2)?;
        let weight = q.num(3)?;
        if !(100.0..=900.0).contains(&weight) || weight % 100.0 != 0.0 {
            return Err(q.err("takes a weight of 100 to 900 in hundreds at position 4"));
        }
        let font = match q.word(4)? {
            "body" => FontRole::Body,
            "display" => FontRole::Display,
            "numeric" | "mono" => FontRole::Mono,
            "rounded" => FontRole::Rounded,
            w => {
                return Err(q.err(&format!(
                    "takes the font role body, display, numeric or rounded, not '{w}'"
                )));
            }
        };
        let base = TypeToken {
            size,
            line_height,
            tracking_em,
            weight: weight as u16,
            font,
        };
        check_type(&q, &base)?;
        let mut at = [base; 4];
        if let Some(b) = &e.block {
            for c in &b.entries {
                let r = P { e: c };
                let bp = Breakpoint::from_name(&c.key).ok_or_else(|| {
                    r.err("is not a breakpoint (phonePortrait, phoneLandscape, tablet, desktop)")
                })?;
                let t = TypeToken {
                    size: r.num(0)?,
                    line_height: r.num(1)?,
                    ..base
                };
                check_type(&r, &t)?;
                at[bp as usize] = t;
            }
        }
        scale.steps[role.index()] = at;
    }
    let missing: Vec<String> = TypeRole::ALL
        .iter()
        .filter(|r| !seen.contains(r))
        .map(|r| r.name().to_string())
        .collect();
    missing_err(p, &missing, "a style sets every type step")?;
    Ok(scale)
}

fn check_type(q: &P, t: &TypeToken) -> Result<(), String> {
    if !(6.0..=400.0).contains(&t.size) {
        return Err(q.err("takes a size of 6 to 400 CSS px"));
    }
    if t.line_height < t.size * 0.8 {
        return Err(q.err("has a line height under 0.8 of its size: lines would overlap"));
    }
    if !(-0.2..=0.5).contains(&t.tracking_em) {
        return Err(q.err("takes a tracking of -0.2 to 0.5 em"));
    }
    Ok(())
}

/// A block of named lengths (`space`, `size`, `radius`, `stroke`), every
/// name required, none negative.
pub(crate) fn parse_lengths<const N: usize>(
    p: &P,
    names: &[&'static str],
    what: &str,
) -> Result<[f32; N], String> {
    let block = p.block()?;
    let mut out = [f32::NAN; N];
    for e in &block.entries {
        let q = P { e };
        let i = names
            .iter()
            .position(|n| *n == e.key)
            .ok_or_else(|| q.err(&format!("is not a {what} name ({})", names.join(", "))))?;
        if !out[i].is_nan() {
            return Err(q.err("is set twice"));
        }
        let v = q.num(0)?;
        if v < 0.0 {
            return Err(q.err("takes a length of 0 or more"));
        }
        out[i] = v;
    }
    let missing: Vec<String> = names
        .iter()
        .zip(out)
        .filter(|(_, v)| v.is_nan())
        .map(|(n, _)| n.to_string())
        .collect();
    missing_err(p, &missing, &format!("a style sets every {what}"))?;
    Ok(out)
}

pub(crate) fn names_of<T: Copy>(all: &[T], name: fn(T) -> &'static str) -> Vec<&'static str> {
    all.iter().map(|v| name(*v)).collect()
}

/// `shadow <dx> <dy> <blur> <spread> "<colour>"` at `values[at..]`.
pub(crate) fn shadow_at(p: &P, at: usize) -> Result<Shadow, String> {
    Ok(Shadow {
        dx: p.num(at)?,
        dy: p.num(at + 1)?,
        blur: p.num(at + 2)?.max(0.0),
        spread: p.num(at + 3)?,
        color: p.color(at + 4)?,
    })
}

/// `elevation { <level> { shadow … } | <level> none }`.
pub(crate) fn parse_elevation(p: &P) -> Result<[Elevation; Level::COUNT], String> {
    let block = p.block()?;
    let mut out = [None; Level::COUNT];
    for e in &block.entries {
        let q = P { e };
        let l = Level::from_name(&e.key)
            .ok_or_else(|| q.err(&format!("is not an elevation level ({})", Level::names())))?;
        if out[l.index()].is_some() {
            return Err(q.err("is set twice"));
        }
        let mut el = Elevation::FLAT;
        match (&e.block, e.values.first().and_then(Value::as_word)) {
            (None, Some("none")) if e.values.len() == 1 => {}
            (Some(b), None) if e.values.is_empty() => {
                for s in &b.entries {
                    let r = P { e: s };
                    if s.key != "shadow" {
                        return Err(r.err("is not `shadow <dx> <dy> <blur> <spread> \"<colour>\"`"));
                    }
                    if usize::from(el.count) == LEVEL_SHADOWS {
                        return Err(r.err(&format!(
                            "is past the {LEVEL_SHADOWS} shadows a level holds"
                        )));
                    }
                    el.shadows[usize::from(el.count)] = shadow_at(&r, 0)?;
                    el.count += 1;
                }
                if el.count == 0 {
                    return Err(q.err("holds no shadow: write `<level> none` for a flat level"));
                }
            }
            _ => return Err(q.err("takes a { shadow … } block, or `none`")),
        }
        out[l.index()] = Some(el);
    }
    let missing: Vec<String> = Level::ALL
        .iter()
        .filter(|l| out[l.index()].is_none())
        .map(|l| l.name().to_string())
        .collect();
    missing_err(p, &missing, "a style sets every elevation level")?;
    Ok(out.map(|e| e.expect("checked")))
}

/// `shape { <class> <corner> <radius px or radius name> }`.
pub(crate) fn parse_shape(
    p: &P,
    radius: &[f32; RadiusStep::COUNT],
) -> Result<[ShapeToken; ElementClass::COUNT], String> {
    let block = p.block()?;
    let mut out: [Option<ShapeToken>; ElementClass::COUNT] = [None; ElementClass::COUNT];
    for e in &block.entries {
        let q = P { e };
        let c = ElementClass::from_name(&e.key).ok_or_else(|| {
            q.err(&format!(
                "is not an element class ({})",
                ElementClass::names()
            ))
        })?;
        if out[c.index()].is_some() {
            return Err(q.err("is set twice"));
        }
        out[c.index()] = Some(shape_at(&q, 0, radius)?);
    }
    let missing: Vec<String> = ElementClass::ALL
        .iter()
        .filter(|c| out[c.index()].is_none())
        .map(|c| c.name().to_string())
        .collect();
    missing_err(p, &missing, "a style gives every element class its corner")?;
    Ok(out.map(|s| s.expect("checked")))
}

/// `<corner> <radius>` at `values[at..]`: the radius in CSS px or a
/// radius scale name.
pub(crate) fn shape_at(
    q: &P,
    at: usize,
    radius: &[f32; RadiusStep::COUNT],
) -> Result<ShapeToken, String> {
    let w = q.word(at)?;
    let corner = CornerShape::from_name(w).ok_or_else(|| {
        q.err(&format!(
            "names the corner '{w}' ({})",
            CornerShape::names()
        ))
    })?;
    let r = match q.e.values.get(at + 1) {
        Some(Value::Word(n)) => {
            radius[RadiusStep::from_name(n)
                .ok_or_else(|| {
                    q.err(&format!(
                        "names the radius '{n}' ({}, or a number)",
                        RadiusStep::names()
                    ))
                })?
                .index()]
        }
        _ => q.num(at + 1)?,
    };
    if r < 0.0 {
        return Err(q.err("takes a radius of 0 or more"));
    }
    if corner == CornerShape::Square && r != 0.0 {
        return Err(q.err("is square with a radius: write `square 0` or another corner"));
    }
    if q.e.values.len() > at + 2 {
        return Err(q.err("takes a corner and a radius"));
    }
    Ok(ShapeToken { corner, radius: r })
}

/// `<duration ms> <bounce>` at `values[at..]`.
pub(crate) fn spring_at(q: &P, at: usize) -> Result<SpringToken, String> {
    let duration_ms = q.num(at)?;
    let bounce = q.num(at + 1)?;
    if !(16.0..=4000.0).contains(&duration_ms) {
        return Err(q.err("takes a duration of 16 to 4000 ms"));
    }
    if !(-0.75..=0.95).contains(&bounce) {
        return Err(q.err("takes a bounce of -0.75 to 0.95"));
    }
    Ok(SpringToken {
        duration_ms,
        bounce,
    })
}

/// What the v2 `motion` block sets besides the v1 widget motion.
pub(crate) struct MotionParse {
    pub springs: [Option<SpringToken>; SpringName::COUNT],
    pub eases: [Option<Ease>; EaseName::COUNT],
    pub durations: [Option<f32>; 6],
    pub stagger: Option<f32>,
    pub stagger_cap: Option<f32>,
    pub overshoot: Option<f32>,
}

/// The v1 duration names, in the order of [`MotionParse::durations`].
pub(crate) const DURATIONS: [&str; 6] = ["instant", "fast", "base", "slow", "spring", "bounce"];

impl MotionParse {
    pub fn new() -> MotionParse {
        MotionParse {
            springs: [None; SpringName::COUNT],
            eases: [None; EaseName::COUNT],
            durations: [None; 6],
            stagger: None,
            stagger_cap: None,
            overshoot: None,
        }
    }

    /// Reads one token line of the motion block; false for a key it does not own.
    pub fn line(&mut self, q: &P) -> Result<bool, String> {
        let e = q.e;
        match e.key.as_str() {
            "spring" => {
                let w = q.word(0)?;
                let s = SpringName::from_name(w).ok_or_else(|| {
                    q.err(&format!("names the spring '{w}' ({})", SpringName::names()))
                })?;
                self.springs[s.index()] = Some(spring_at(q, 1)?);
            }
            "ease" => {
                let w = q.word(0)?;
                let n = EaseName::from_name(w).ok_or_else(|| {
                    q.err(&format!("names the ease '{w}' ({})", EaseName::names()))
                })?;
                let ease = match e.values.get(1).and_then(Value::as_word) {
                    Some("linear") => Ease::Bezier([0.0, 0.0, 1.0, 1.0]),
                    Some("steps") => {
                        let k = q.num(2)?;
                        if !(1.0..=60.0).contains(&k) || k.fract() != 0.0 {
                            return Err(q.err("takes steps 1 to 60"));
                        }
                        Ease::Steps(k as u8)
                    }
                    Some(w) => {
                        return Err(q.err(&format!(
                            "takes four control points, linear or steps <n>, not '{w}'"
                        )));
                    }
                    None => {
                        let b = [q.num(1)?, q.num(2)?, q.num(3)?, q.num(4)?];
                        if !(0.0..=1.0).contains(&b[0]) || !(0.0..=1.0).contains(&b[2]) {
                            return Err(q.err("takes control points whose x is 0 to 1"));
                        }
                        Ease::Bezier(b)
                    }
                };
                self.eases[n.index()] = Some(ease);
            }
            "duration" => {
                let w = q.word(0)?;
                let i = DURATIONS.iter().position(|d| *d == w).ok_or_else(|| {
                    q.err(&format!(
                        "names the duration '{w}' ({})",
                        DURATIONS.join(", ")
                    ))
                })?;
                self.durations[i] = Some(q.num(1)?.max(0.0));
            }
            "stagger" => self.stagger = Some(q.num(0)?.max(0.0)),
            "staggerCap" => self.stagger_cap = Some(q.num(0)?.max(0.0)),
            "overshoot" => {
                let v = q.num(0)?;
                if !(0.0..=1.0).contains(&v) {
                    return Err(q.err("takes the overshoot allowed, 0 to 1"));
                }
                self.overshoot = Some(v);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// The tokens, every one set, every spring within the overshoot.
    pub fn finish(self, p: &P, extra: &[(&str, SpringToken)]) -> Result<MotionTokens, String> {
        let mut missing: Vec<String> = Vec::new();
        for s in SpringName::ALL {
            if self.springs[s.index()].is_none() {
                missing.push(format!("spring {}", s.name()));
            }
        }
        for n in EaseName::ALL {
            if self.eases[n.index()].is_none() {
                missing.push(format!("ease {}", n.name()));
            }
        }
        for (i, d) in DURATIONS.iter().enumerate() {
            if self.durations[i].is_none() {
                missing.push(format!("duration {d}"));
            }
        }
        for (k, v) in [
            ("stagger", self.stagger),
            ("staggerCap", self.stagger_cap),
            ("overshoot", self.overshoot),
        ] {
            if v.is_none() {
                missing.push(k.into());
            }
        }
        missing_err(p, &missing, "a version 2 style sets every motion token")?;
        let overshoot = self.overshoot.expect("checked");
        let springs = self.springs.map(|s| s.expect("checked"));
        let named = SpringName::ALL
            .iter()
            .map(|s| (s.name(), springs[s.index()]))
            .chain(extra.iter().copied());
        for (name, s) in named {
            if s.overshoot() > overshoot + 1e-3 {
                return Err(p.err(&format!(
                    "has the spring {name} overshoot {:.2} past the style's overshoot {overshoot}: lower its bounce or raise overshoot",
                    s.overshoot()
                )));
            }
        }
        let d = self.durations.map(|d| d.expect("checked"));
        Ok(MotionTokens {
            springs,
            eases: self.eases.map(|e| e.expect("checked")),
            durations: Durations {
                instant: d[0],
                fast: d[1],
                base: d[2],
                slow: d[3],
                spring: d[4],
                bounce: d[5],
            },
            stagger_ms: self.stagger.expect("checked"),
            stagger_cap_ms: self.stagger_cap.expect("checked"),
            overshoot,
        })
    }
}

/// `sounds { <event> none|"<id>"… [volume lo hi] [pitch lo hi] haptic <preset>|none|<pulses> }`:
/// the tokens and each event's sound asset ids.
#[allow(clippy::type_complexity)]
pub(crate) fn parse_sounds(
    p: &P,
) -> Result<
    (
        [Feedback; SoundEvent::COUNT],
        [Vec<String>; SoundEvent::COUNT],
    ),
    String,
> {
    let block = p.block()?;
    let mut out = [Feedback::SILENT; SoundEvent::COUNT];
    let mut ids: [Vec<String>; SoundEvent::COUNT] = std::array::from_fn(|_| Vec::new());
    let mut seen = Vec::new();
    for e in &block.entries {
        let q = P { e };
        let ev = SoundEvent::from_name(&e.key)
            .ok_or_else(|| q.err(&format!("is not a UI event ({})", SoundEvent::names())))?;
        if seen.contains(&ev) {
            return Err(q.err("is set twice"));
        }
        seen.push(ev);
        let mut f = Feedback::SILENT;
        let mut i = 0;
        let v = &e.values;
        if v.first().and_then(Value::as_word) == Some("none") {
            i = 1;
        } else {
            while let Some(s) = v.get(i).and_then(Value::as_str) {
                ids[ev.index()].push(s.to_string());
                i += 1;
            }
            if ids[ev.index()].is_empty() {
                return Err(
                    q.err("starts with its sound (\"<asset id>\", several for a set) or none")
                );
            }
            if ids[ev.index()].len() > SET_SOUNDS {
                return Err(q.err(&format!(
                    "names more than the {SET_SOUNDS} sounds a set holds"
                )));
            }
            f.count = ids[ev.index()].len() as u8;
        }
        let mut haptic = None;
        while i < v.len() {
            match q.word(i)? {
                "volume" | "pitch" => {
                    let k = q.word(i)?;
                    let (lo, hi) = (q.num(i + 1)?, q.num(i + 2)?);
                    let ok = lo <= hi
                        && if k == "volume" {
                            (0.0..=1.0).contains(&lo) && (0.0..=1.0).contains(&hi)
                        } else {
                            (0.25..=4.0).contains(&lo) && (0.25..=4.0).contains(&hi)
                        };
                    if !ok {
                        return Err(q.err(&format!(
                            "takes {k} as a low and a high value ({})",
                            if k == "volume" { "0 to 1" } else { "0.25 to 4" }
                        )));
                    }
                    if k == "volume" {
                        f.volume = [lo, hi];
                    } else {
                        f.pitch = [lo, hi];
                    }
                    i += 3;
                }
                "haptic" => {
                    let (h, used) = haptic_at(&q, i + 1)?;
                    haptic = Some(h);
                    i += 1 + used;
                }
                w => {
                    return Err(q.err(&format!("has '{w}' where volume, pitch or haptic goes")));
                }
            }
        }
        f.haptic = haptic.ok_or_else(|| {
            q.err("leaves out its haptic: `haptic <preset>`, `haptic none` or pulses `haptic <start ms> <ms> <low> <high> …`")
        })?;
        out[ev.index()] = f;
    }
    let missing: Vec<String> = SoundEvent::ALL
        .iter()
        .filter(|e| !seen.contains(e))
        .map(|e| e.name().to_string())
        .collect();
    missing_err(
        p,
        &missing,
        "a style says what every UI event plays (`<event> none haptic none` for nothing)",
    )?;
    Ok((out, ids))
}

/// A haptic at `values[at..]` and how many values it took.
fn haptic_at(q: &P, at: usize) -> Result<(Haptic, usize), String> {
    match q.e.values.get(at) {
        Some(Value::Word(w)) if w == "none" => Ok((Haptic::NONE, 1)),
        Some(Value::Word(w)) => HapticPreset::from_name(w)
            .map(|h| (Haptic::preset(h), 1))
            .ok_or_else(|| {
                q.err(&format!(
                    "names the haptic '{w}' ({}, none, or pulses)",
                    HapticPreset::names()
                ))
            }),
        Some(Value::Number(_)) => {
            let n = q.e.values[at..]
                .iter()
                .take_while(|v| v.as_number().is_some())
                .count();
            if n % 4 != 0 || n / 4 > PULSES {
                return Err(q.err(&format!(
                    "takes up to {PULSES} pulses of four numbers: start ms, ms, low and high strength"
                )));
            }
            let mut h = Haptic::NONE;
            let mut last = 0.0;
            for k in 0..n / 4 {
                let b = at + k * 4;
                let pulse = Pulse {
                    start_ms: q.num(b)?,
                    ms: q.num(b + 1)?,
                    low: q.num(b + 2)?,
                    high: q.num(b + 3)?,
                };
                if pulse.start_ms < last
                    || !(1.0..=1000.0).contains(&pulse.ms)
                    || !(0.0..=1.0).contains(&pulse.low)
                    || !(0.0..=1.0).contains(&pulse.high)
                {
                    return Err(q.err(
                        "takes pulses in order of start, 1 to 1000 ms long, strengths 0 to 1",
                    ));
                }
                last = pulse.start_ms;
                h.pulses[k] = pulse;
            }
            h.count = (n / 4) as u8;
            Ok((h, n))
        }
        _ => Err(q.err("takes a haptic after `haptic`")),
    }
}

// ───────────────────────────────────────────────────────────── writing

pub(crate) fn num_value(v: f32) -> Value {
    num(v)
}

fn num(v: f32) -> Value {
    Value::Number(Number::from_f32(round_to(v, 1000.0)).expect("a finite token"))
}

fn word(w: &str) -> Value {
    Value::Word(w.to_string())
}

/// `#rrggbb` or `#rrggbbaa`.
pub fn hex(c: Srgba) -> String {
    let [r, g, b] = c.rgb;
    if c.alpha >= 1.0 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!(
            "#{r:02x}{g:02x}{b:02x}{:02x}",
            (c.alpha * 255.0).round() as u8
        )
    }
}

fn color(c: Srgba) -> Value {
    Value::String(hex(c))
}

fn line(key: &str, values: Vec<Value>) -> Entry {
    Entry {
        key: key.to_string(),
        values,
        block: None,
        line: 0,
    }
}

fn block(key: &str, values: Vec<Value>, entries: Vec<Entry>) -> Entry {
    Entry {
        key: key.to_string(),
        values,
        block: Some(Block { entries }),
        line: 0,
    }
}

fn cue_values(cue: Cue, icons: &[String]) -> Vec<Value> {
    match cue {
        Cue::None => vec![],
        Cue::Icon(i) => vec![
            word("icon"),
            Value::String(icons.get(usize::from(i)).cloned().unwrap_or_default()),
        ],
        Cue::Pattern(x) => vec![word("pattern"), word(x.name())],
        Cue::Pips(n) => vec![word("pips"), num(f32::from(n))],
    }
}

pub(crate) fn shadow_values(s: &Shadow) -> Vec<Value> {
    vec![
        num(s.dx),
        num(s.dy),
        num(s.blur),
        num(s.spread),
        color(s.color),
    ]
}

impl StyleTokens {
    /// The tokens as the blocks of a version 2 style file (`colors`,
    /// `typography`, `space`, `size`, `radius`, `stroke`, `elevation`,
    /// `shape`), and the token lines of its `motion` block and its
    /// `sounds` block given each event's sound ids.
    pub fn entries(
        &self,
        icons: &[String],
        sounds: &[Vec<String>; SoundEvent::COUNT],
    ) -> (Vec<Entry>, Vec<Entry>, Entry) {
        let c = &self.colors;
        let mut colors = Vec::new();
        for s in Status::ALL {
            colors.push(line(s.name(), cue_values(c.status[s.index()], icons)));
        }
        let sw = |k: &str, s: &Swatch| {
            let mut v = vec![color(s.color)];
            v.extend(cue_values(s.cue, icons));
            line(k, v)
        };
        for r in Rarity::ALL {
            colors.push(sw(r.name(), &c.rarity[r.index()]));
        }
        for (k, s) in c.team.iter().enumerate() {
            colors.push(sw(&format!("team{}", k + 1), s));
        }
        for d in DamageType::ALL {
            colors.push(sw(d.name(), &c.damage[d.index()]));
        }
        for (k, s) in c.series.iter().enumerate() {
            colors.push(line(&format!("series{}", k + 1), vec![color(*s)]));
        }
        let mut typo = Vec::new();
        for r in TypeRole::ALL {
            let at = self.typography.steps[r.index()];
            let base = at[Breakpoint::Desktop as usize];
            let font = match base.font {
                FontRole::Body => "body",
                FontRole::Display => "display",
                FontRole::Mono => "numeric",
                FontRole::Rounded => "rounded",
            };
            let overrides: Vec<Entry> = Breakpoint::ALL
                .iter()
                .filter(|b| {
                    let t = at[**b as usize];
                    (t.size, t.line_height) != (base.size, base.line_height)
                })
                .map(|b| {
                    let t = at[*b as usize];
                    line(b.name(), vec![num(t.size), num(t.line_height)])
                })
                .collect();
            let values = vec![
                num(base.size),
                num(base.line_height),
                num(base.tracking_em),
                num(f32::from(base.weight)),
                word(font),
            ];
            typo.push(if overrides.is_empty() {
                line(r.name(), values)
            } else {
                block(r.name(), values, overrides)
            });
        }
        let lengths = |key: &str, names: &[&str], v: &[f32]| {
            block(
                key,
                vec![],
                names
                    .iter()
                    .zip(v)
                    .map(|(n, v)| line(n, vec![num(*v)]))
                    .collect(),
            )
        };
        let elevation = Level::ALL
            .iter()
            .map(|l| {
                let e = &self.elevation[l.index()];
                if e.count == 0 {
                    line(l.name(), vec![word("none")])
                } else {
                    block(
                        l.name(),
                        vec![],
                        e.shadows()
                            .iter()
                            .map(|s| line("shadow", shadow_values(s)))
                            .collect(),
                    )
                }
            })
            .collect();
        let shape = ElementClass::ALL
            .iter()
            .map(|k| {
                let s = self.shape[k.index()];
                line(k.name(), vec![word(s.corner.name()), num(s.radius)])
            })
            .collect();
        let tokens = vec![
            block("colors", vec![], colors),
            block("typography", vec![], typo),
            lengths(
                "space",
                &names_of(SpaceStep::ALL, SpaceStep::name),
                &self.space,
            ),
            lengths("size", &names_of(SizeStep::ALL, SizeStep::name), &self.size),
            lengths(
                "radius",
                &names_of(RadiusStep::ALL, RadiusStep::name),
                &self.radius,
            ),
            lengths(
                "stroke",
                &names_of(StrokeWidth::ALL, StrokeWidth::name),
                &self.stroke,
            ),
            block("elevation", vec![], elevation),
            block("shape", vec![], shape),
        ];
        let m = &self.motion;
        let mut motion = Vec::new();
        for s in SpringName::ALL {
            let t = m.springs[s.index()];
            motion.push(line(
                "spring",
                vec![word(s.name()), num(t.duration_ms), num(t.bounce)],
            ));
        }
        for n in EaseName::ALL {
            let mut v = vec![word(n.name())];
            match m.eases[n.index()] {
                Ease::Bezier([0.0, 0.0, 1.0, 1.0]) => v.push(word("linear")),
                Ease::Bezier(b) => v.extend(b.map(num)),
                Ease::Steps(k) => v.extend([word("steps"), num(f32::from(k))]),
            }
            motion.push(line("ease", v));
        }
        let d = m.durations;
        for (n, v) in DURATIONS
            .iter()
            .zip([d.instant, d.fast, d.base, d.slow, d.spring, d.bounce])
        {
            motion.push(line("duration", vec![word(n), num(v)]));
        }
        motion.push(line("stagger", vec![num(m.stagger_ms)]));
        motion.push(line("staggerCap", vec![num(m.stagger_cap_ms)]));
        motion.push(line("overshoot", vec![num(m.overshoot)]));
        let feedback = SoundEvent::ALL
            .iter()
            .map(|e| {
                let f = &self.feedback[e.index()];
                let ids = &sounds[e.index()];
                let mut v: Vec<Value> = if ids.is_empty() {
                    vec![word("none")]
                } else {
                    ids.iter().map(|s| Value::String(s.clone())).collect()
                };
                if f.volume != [1.0, 1.0] {
                    v.extend([word("volume"), num(f.volume[0]), num(f.volume[1])]);
                }
                if f.pitch != [1.0, 1.0] {
                    v.extend([word("pitch"), num(f.pitch[0]), num(f.pitch[1])]);
                }
                v.push(word("haptic"));
                match HapticPreset::ALL
                    .iter()
                    .find(|h| Haptic::preset(**h) == f.haptic)
                {
                    _ if f.haptic.count == 0 => v.push(word("none")),
                    Some(h) => v.push(word(h.name())),
                    None => {
                        for p in f.haptic.pulses() {
                            v.extend([num(p.start_ms), num(p.ms), num(p.low), num(p.high)]);
                        }
                    }
                }
                line(e.name(), v)
            })
            .collect();
        (tokens, motion, block("sounds", vec![], feedback))
    }
}
