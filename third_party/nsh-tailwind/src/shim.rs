//! Definitions copied unchanged from nsh-core (paths in each comment) that the
//! vendored modules name; see PATCHED.md.
#![allow(dead_code)]
use crate::{color::{Hsla, Rgba, transparent_black}, geometry::{Bounds, Points}, style::{Color, FontVariant}};
use crate::animation::Driver;
use std::{rc::Rc, sync::Arc};

// nsh-core/src/element/animated.rs
/// ln(1000): a decay of e^(-x) falls to a thousandth at this x.
const SETTLE: f32 = 6.907_755;

/// How close to 1 a damping ratio is taken as critical, where the
/// underdamped and overdamped forms divide by almost 0.
const CRITICAL: f32 = 1e-3;

// nsh-core/src/element/animated.rs
/// How a spring moves (TS's defaults: stiffness 170, damping 26, mass 1):
/// an animated value's ([`AnimatedValue::spring`]) and a layout
/// animation's ([`LayoutTransition::spring`](crate::LayoutTransition::spring)).
///
/// Made as `gpui-base`'s `Spring` is, from how long it takes and how much it
/// overshoots ([`Spring::new`], [`Spring::damping_ratio`]), or from a
/// stiffness, as Material 3's motion tokens give it
/// ([`Spring::from_stiffness`]); its fields are the physics either way.
///
/// ```
/// use nsh_core::element::Spring;
///
/// // `gpui-component`'s `spring_move`: a 280 ms response, a little springy.
/// let spring = Spring::new(280.0).damping_ratio(0.85);
/// assert!((spring.settle_ms() - 362.2).abs() < 0.1);
/// // Material 3 Expressive's fast spatial spring.
/// let spring = Spring::from_stiffness(800.0).damping_ratio(0.6);
/// assert!((spring.settle_ms() - 407.0).abs() < 0.1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    /// Stiffness `k`.
    pub stiffness: f32,
    /// Damping coefficient `c` (not a ratio: the ratio is `c / (2 √(k m))`,
    /// [`Spring::damping_ratio`] sets it).
    pub damping: f32,
    /// Mass `m`.
    pub mass: f32,
    /// The velocity it starts with (per second); `None` keeps the current
    /// one.
    pub velocity: Option<f32>,
    /// The highest frame rate it needs, in Hz (TS's `frameRate`); `None`:
    /// the screen's highest ([`Driver::frame_rate`]).
    pub frame_rate: Option<f32>,
}

// nsh-core/src/element/animated.rs
impl Default for Spring {
    fn default() -> Self {
        Self {
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
            velocity: None,
            frame_rate: None,
        }
    }
}

// nsh-core/src/element/animated.rs
impl Spring {
    /// `gpui-base`'s `Spring::new(response)`: a mass of 1 that would swing
    /// once in `response_ms` undamped, critically damped (no overshoot).
    /// The response is the scale the motion is felt at, not when it stops
    /// ([`Spring::settle_ms`]).
    pub fn new(response_ms: f32) -> Self {
        let omega = std::f32::consts::TAU / (response_ms / 1000.0);
        Self {
            stiffness: omega * omega,
            damping: 2.0 * omega,
            mass: 1.0,
            velocity: None,
            frame_rate: None,
        }
    }

    /// A mass of 1 on a spring of `stiffness`, critically damped: a design
    /// system's stiffness token, its damping ratio set with
    /// [`Spring::damping_ratio`].
    pub fn from_stiffness(stiffness: f32) -> Self {
        Self {
            stiffness,
            damping: 2.0 * stiffness.max(0.0).sqrt(),
            mass: 1.0,
            velocity: None,
            frame_rate: None,
        }
    }

    /// Damped at `ratio` (`gpui-base`'s `with_damping`): 1 does not
    /// overshoot, below 1 it passes its target and comes back, above 1 it
    /// approaches slowly. Sets [`Spring::damping`] from the stiffness and
    /// mass it has, so set those first.
    pub fn damping_ratio(mut self, ratio: f32) -> Self {
        self.damping = 2.0 * ratio * (self.stiffness * self.mass).max(0.0).sqrt();
        self
    }

    /// How far it has gone, from 0 to 1, `ms` milliseconds after it was
    /// released at rest: the damped oscillator's closed form, past 1 when
    /// it is underdamped. A spring that never settles (no damping, or a
    /// stiffness or mass that is not positive) is at 1 at once.
    pub fn progress(self, ms: f32) -> f32 {
        let Some((zeta, omega)) = self.oscillator() else {
            return 1.0;
        };
        if ms <= 0.0 {
            return 0.0;
        }
        let t = ms / 1000.0;
        if (zeta - 1.0).abs() < CRITICAL {
            return 1.0 - (1.0 + omega * t) * (-omega * t).exp();
        }
        if zeta < 1.0 {
            let decay = zeta * omega;
            let wd = omega * (1.0 - zeta * zeta).sqrt();
            return 1.0 - (-decay * t).exp() * ((wd * t).cos() + decay / wd * (wd * t).sin());
        }
        let root = omega * (zeta * zeta - 1.0).sqrt();
        let (slow, fast) = (zeta * omega - root, zeta * omega + root);
        1.0 - (fast * (-slow * t).exp() - slow * (-fast * t).exp()) / (fast - slow)
    }

    /// How long it takes, in milliseconds, released at rest, to stay within
    /// a thousandth of its travel: how long a layout animation on it lasts.
    /// 0 for a spring that never settles.
    pub fn settle_ms(self) -> f32 {
        let Some((zeta, omega)) = self.oscillator() else {
            return 0.0;
        };
        let seconds = if (zeta - 1.0).abs() < CRITICAL {
            // (1 + ω t) e^(-ω t) falls to 1/1000 at ω t = 9.233.
            9.233 / omega
        } else if zeta < 1.0 {
            SETTLE / (zeta * omega)
        } else {
            // The slow root's term, whose weight is above 1, outlasts the fast one's.
            let root = (zeta * zeta - 1.0).sqrt();
            let weight = (zeta + root) / (2.0 * root);
            (SETTLE + weight.ln()) / (omega * (zeta - root))
        };
        seconds * 1000.0
    }

    /// The damping ratio and the natural angular frequency (radians per
    /// second); `None` when it never settles.
    fn oscillator(self) -> Option<(f32, f32)> {
        let Spring {
            stiffness,
            damping,
            mass,
            ..
        } = self;
        if !(stiffness > 0.0 && mass > 0.0 && damping > 0.0)
            || !(stiffness.is_finite() && mass.is_finite() && damping.is_finite())
        {
            return None;
        }
        let zeta = damping / (2.0 * (stiffness * mass).sqrt());
        Some((zeta, (stiffness / mass).sqrt()))
    }

    /// Needing at most `hz` frames per second (a small, slow spring: 60).
    pub const fn frame_rate(mut self, hz: f32) -> Self {
        self.frame_rate = Some(hz);
        self
    }

    fn driver(self, to: f32) -> Driver {
        Driver::Spring {
            to,
            stiffness: self.stiffness,
            damping: self.damping,
            mass: self.mass,
            velocity: self.velocity,
            frame_rate: self.frame_rate,
        }
    }
}

// nsh-core/src/ops.rs
/// How an image fits its box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResizeMode {
    /// Fill the box, cropping (default).
    #[default]
    Cover,
    /// Fit inside the box, letterboxing.
    Contain,
    /// Stretch to the box.
    Stretch,
    /// Center at natural size, cropped where larger than the box (CSS's
    /// `object-fit: none`).
    Center,
    /// Center at natural size, shrunk to fit the box where larger (CSS's
    /// `object-fit: scale-down`, React Native's `center`).
    ScaleDown,
}

// nsh-core/src/element/style.rs
/// A color for each appearance: the platform switches between them
/// with nothing running. Theme colors are adaptive; an [`Hsla`] (or anything
/// that converts to one) is the same in both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AdaptiveColor {
    /// In light appearance.
    pub light: Hsla,
    /// In dark appearance.
    pub dark: Hsla,
}

impl AdaptiveColor {
    /// Transparent in both.
    pub const TRANSPARENT: Self = Self {
        light: transparent_black(),
        dark: transparent_black(),
    };

    /// `light` and `dark`.
    pub fn new(light: impl Into<Hsla>, dark: impl Into<Hsla>) -> Self {
        Self {
            light: light.into(),
            dark: dark.into(),
        }
    }

    /// `0xRRGGBB` in light appearance and in dark, opaque (`const`s:
    /// `AdaptiveColor::rgb(0xffffff, 0x101113)`).
    pub const fn rgb(light: u32, dark: u32) -> Self {
        Self {
            light: crate::color::rgb(light).to_hsla(),
            dark: crate::color::rgb(dark).to_hsla(),
        }
    }

    /// Both with their alpha multiplied by `opacity`.
    pub fn opacity(self, opacity: f32) -> Self {
        self.map(|c| c.opacity(opacity))
    }

    /// Both through `f` (`theme.primary.map(|c| c.darken(0.1))`).
    pub fn map(self, f: impl Fn(Hsla) -> Hsla) -> Self {
        Self {
            light: f(self.light),
            dark: f(self.dark),
        }
    }
}

impl From<Hsla> for AdaptiveColor {
    fn from(c: Hsla) -> Self {
        Self { light: c, dark: c }
    }
}

impl From<Rgba> for AdaptiveColor {
    fn from(c: Rgba) -> Self {
        Hsla::from(c).into()
    }
}

impl From<Color> for AdaptiveColor {
    fn from(c: Color) -> Self {
        Hsla::from(c).into()
    }
}

// nsh-core/src/animation/graph.rs
/// The evaluated value of a node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationValue {
    /// A number.
    Number(f32),
    /// A color, `0xRRGGBBAA`.
    Color(u32),
}

// nsh-core/src/animation/graph.rs
impl AnimationValue {
    /// The number, or 0 for colors.
    pub fn number(self) -> f32 {
        match self {
            AnimationValue::Number(n) => n,
            AnimationValue::Color(_) => 0.0,
        }
    }
}

// nsh-core/src/element/reactive.rs
/// A value that is set once or bound: what element setters and props take
/// ([`IntoReactive`]).
pub enum Reactive<T> {
    /// A value set once (no binding).
    Static(T),
    /// A value read in a binding: a signal, a memo or a closure, run again
    /// when what it read changes.
    Bound(Rc<dyn Fn() -> T>),
}

// nsh-core/src/element/reactive.rs
/// What can be a [`Reactive`] `T`: a value of a type that becomes a `T`
/// (set once), a [`Signal`], [`Memo`] or [`ReadSignal`] of `T`, or a
/// closure returning what becomes a `T` (bound).
pub trait IntoReactive<T> {
    /// As a [`Reactive`].
    fn into_reactive(self) -> Reactive<T>;
}

// nsh-core/src/props.rs
/// Which touches a view and its children receive (`pointerEvents`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointerEvents {
    /// The view and its children receive touches (default).
    #[default]
    Auto,
    /// Neither the view nor its children receive touches.
    None,
    /// Only the children receive touches; the view itself is transparent to them.
    BoxNone,
    /// Only the view receives touches, never its children.
    BoxOnly,
}
