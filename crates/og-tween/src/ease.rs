//! The easing curves: Robert Penner's set (quad, cubic, quart, expo, back,
//! elastic, bounce; in, out, in-out) plus linear, with og-core's
//! deterministic `pow` and `sin`, so a curve gives the same bits on every
//! host.

use og_core::math;

/// An easing curve from 0 at t = 0 to 1 at t = 1 (back and elastic leave
/// the range in between).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ease {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuartIn,
    QuartOut,
    QuartInOut,
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    BackIn,
    BackOut,
    BackInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

/// Every curve, in [`Ease`] order (the code is the index).
pub const EASES: [Ease; 22] = [
    Ease::Linear,
    Ease::QuadIn,
    Ease::QuadOut,
    Ease::QuadInOut,
    Ease::CubicIn,
    Ease::CubicOut,
    Ease::CubicInOut,
    Ease::QuartIn,
    Ease::QuartOut,
    Ease::QuartInOut,
    Ease::ExpoIn,
    Ease::ExpoOut,
    Ease::ExpoInOut,
    Ease::BackIn,
    Ease::BackOut,
    Ease::BackInOut,
    Ease::ElasticIn,
    Ease::ElasticOut,
    Ease::ElasticInOut,
    Ease::BounceIn,
    Ease::BounceOut,
    Ease::BounceInOut,
];

/// The IDL names, in [`EASES`] order.
pub const EASE_NAMES: [&str; 22] = [
    "linear",
    "quadIn",
    "quadOut",
    "quadInOut",
    "cubicIn",
    "cubicOut",
    "cubicInOut",
    "quartIn",
    "quartOut",
    "quartInOut",
    "expoIn",
    "expoOut",
    "expoInOut",
    "backIn",
    "backOut",
    "backInOut",
    "elasticIn",
    "elasticOut",
    "elasticInOut",
    "bounceIn",
    "bounceOut",
    "bounceInOut",
];

const PI: f32 = std::f32::consts::PI;

fn exp2(x: f32) -> f32 {
    math::pow(2.0, x)
}

fn bounce_out(t: f32) -> f32 {
    const N: f32 = 7.5625;
    const D: f32 = 2.75;
    if t < 1.0 / D {
        N * t * t
    } else if t < 2.0 / D {
        let t = t - 1.5 / D;
        N * t * t + 0.75
    } else if t < 2.5 / D {
        let t = t - 2.25 / D;
        N * t * t + 0.9375
    } else {
        let t = t - 2.625 / D;
        N * t * t + 0.984375
    }
}

impl Ease {
    /// The code (index in [`EASES`]).
    pub fn code(self) -> u32 {
        self as u32
    }

    /// The curve of a code, if it is one.
    pub fn from_code(code: u32) -> Option<Ease> {
        EASES.get(code as usize).copied()
    }

    /// The curve of an IDL name (`quadOut`).
    pub fn from_name(name: &str) -> Option<Ease> {
        EASE_NAMES.iter().position(|n| *n == name).map(|i| EASES[i])
    }

    pub fn name(self) -> &'static str {
        EASE_NAMES[self as usize]
    }

    /// The eased value at `t`, clamped to 0..=1 first.
    pub fn apply(self, t: f32) -> f32 {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        let c1 = 1.70158f32;
        let c2 = c1 * 1.525;
        let c3 = c1 + 1.0;
        let c4 = 2.0 * PI / 3.0;
        let c5 = 2.0 * PI / 4.5;
        match self {
            Ease::Linear => t,
            Ease::QuadIn => t * t,
            Ease::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            Ease::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    let u = -2.0 * t + 2.0;
                    1.0 - u * u / 2.0
                }
            }
            Ease::CubicIn => t * t * t,
            Ease::CubicOut => {
                let u = 1.0 - t;
                1.0 - u * u * u
            }
            Ease::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let u = -2.0 * t + 2.0;
                    1.0 - u * u * u / 2.0
                }
            }
            Ease::QuartIn => t * t * t * t,
            Ease::QuartOut => {
                let u = 1.0 - t;
                1.0 - u * u * u * u
            }
            Ease::QuartInOut => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    let u = -2.0 * t + 2.0;
                    1.0 - u * u * u * u / 2.0
                }
            }
            Ease::ExpoIn => {
                if t == 0.0 {
                    0.0
                } else {
                    exp2(10.0 * t - 10.0)
                }
            }
            Ease::ExpoOut => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - exp2(-10.0 * t)
                }
            }
            Ease::ExpoInOut => {
                if t == 0.0 || t == 1.0 {
                    t
                } else if t < 0.5 {
                    exp2(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - exp2(-20.0 * t + 10.0)) / 2.0
                }
            }
            Ease::BackIn => c3 * t * t * t - c1 * t * t,
            Ease::BackOut => {
                let u = t - 1.0;
                1.0 + c3 * u * u * u + c1 * u * u
            }
            Ease::BackInOut => {
                if t < 0.5 {
                    let u = 2.0 * t;
                    u * u * ((c2 + 1.0) * u - c2) / 2.0
                } else {
                    let u = 2.0 * t - 2.0;
                    (u * u * ((c2 + 1.0) * u + c2) + 2.0) / 2.0
                }
            }
            Ease::ElasticIn => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    -exp2(10.0 * t - 10.0) * math::sin((t * 10.0 - 10.75) * c4)
                }
            }
            Ease::ElasticOut => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    exp2(-10.0 * t) * math::sin((t * 10.0 - 0.75) * c4) + 1.0
                }
            }
            Ease::ElasticInOut => {
                if t == 0.0 || t == 1.0 {
                    t
                } else if t < 0.5 {
                    -(exp2(20.0 * t - 10.0) * math::sin((20.0 * t - 11.125) * c5)) / 2.0
                } else {
                    exp2(-20.0 * t + 10.0) * math::sin((20.0 * t - 11.125) * c5) / 2.0 + 1.0
                }
            }
            Ease::BounceIn => 1.0 - bounce_out(1.0 - t),
            Ease::BounceOut => bounce_out(t),
            Ease::BounceInOut => {
                if t < 0.5 {
                    (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
                }
            }
        }
    }
}
