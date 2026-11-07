//! CSS Easing Level 1 evaluation; no host math or adaptive solver.
use super::Ease;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPosition {
    JumpStart,
    JumpEnd,
    JumpNone,
    JumpBoth,
}

impl StepPosition {
    pub fn name(self) -> &'static str {
        match self {
            Self::JumpStart => "jump-start",
            Self::JumpEnd => "jump-end",
            Self::JumpNone => "jump-none",
            Self::JumpBoth => "jump-both",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "start" | "jump-start" => Self::JumpStart,
            "end" | "jump-end" => Self::JumpEnd,
            "jump-none" => Self::JumpNone,
            "jump-both" => Self::JumpBoth,
            _ => return None,
        })
    }
}

impl Ease {
    /// Validate authored controls before storing or evaluating them.
    pub fn is_valid(self) -> bool {
        match self {
            Self::CubicBezier(p) => {
                p.iter().all(|x| x.is_finite())
                    && (0.0..=1.0).contains(&p[0])
                    && (0.0..=1.0).contains(&p[2])
            }
            Self::Steps { count, position } => {
                count > 0 && (position != StepPosition::JumpNone || count > 1)
            }
            _ => true,
        }
    }

    /// CSS keywords are exact CSS curves, not similarly named GSAP eases.
    pub fn from_css(text: &str) -> Option<Self> {
        let text = text.trim().to_ascii_lowercase();
        let ease = match text.as_str() {
            "linear" => Self::CubicBezier([0.0, 0.0, 1.0, 1.0]),
            "ease" => Self::CubicBezier([0.25, 0.1, 0.25, 1.0]),
            "ease-in" => Self::CubicBezier([0.42, 0.0, 1.0, 1.0]),
            "ease-out" => Self::CubicBezier([0.0, 0.0, 0.58, 1.0]),
            "ease-in-out" => Self::CubicBezier([0.42, 0.0, 0.58, 1.0]),
            "step-start" => Self::Steps {
                count: 1,
                position: StepPosition::JumpStart,
            },
            "step-end" => Self::Steps {
                count: 1,
                position: StepPosition::JumpEnd,
            },
            _ => {
                let (function, args) = text.split_once('(')?;
                let args = args.strip_suffix(')')?;
                let args: Vec<_> = args.split(',').map(str::trim).collect();
                match (function, args.as_slice()) {
                    ("cubic-bezier", [x1, y1, x2, y2]) => Self::CubicBezier([
                        x1.parse().ok()?,
                        y1.parse().ok()?,
                        x2.parse().ok()?,
                        y2.parse().ok()?,
                    ]),
                    ("steps", [count]) => Self::Steps {
                        count: count.parse().ok()?,
                        position: StepPosition::JumpEnd,
                    },
                    ("steps", [count, position]) => Self::Steps {
                        count: count.parse().ok()?,
                        position: StepPosition::from_name(position)?,
                    },
                    _ => return None,
                }
            }
        };
        ease.is_valid().then_some(ease)
    }

    /// Lossless timeline text for both named and parameterized curves.
    pub fn to_text(self) -> String {
        match self {
            Self::CubicBezier([x1, y1, x2, y2]) => format!("cubic-bezier({x1}, {y1}, {x2}, {y2})"),
            Self::Steps { count, position } => format!("steps({count}, {})", position.name()),
            _ => self.name().into(),
        }
    }

    pub fn from_text(text: &str) -> Option<Self> {
        Self::from_name(text).or_else(|| {
            if text.starts_with("cubic-bezier(") || text.starts_with("steps(") {
                Self::from_css(text)
            } else {
                None
            }
        })
    }

    /// CSS sampling, including extrapolation and the before flag at a step
    /// discontinuity. Named GSAP curves retain their clamped semantics.
    /// Controls must pass [`Self::is_valid`] at the input boundary.
    pub fn sample(self, t: f32, before: bool) -> f32 {
        match self {
            Self::CubicBezier(p) => bezier(p, f64::from(t)) as f32,
            Self::Steps { count, position } => {
                let scaled = f64::from(t) * f64::from(count);
                let mut current = scaled.floor();
                if matches!(position, StepPosition::JumpStart | StepPosition::JumpBoth) {
                    current += 1.0;
                }
                if before && scaled == scaled.floor() {
                    current -= 1.0;
                }
                if t >= 0.0 {
                    current = current.max(0.0);
                }
                let jumps = f64::from(count)
                    + match position {
                        StepPosition::JumpNone => -1.0,
                        StepPosition::JumpBoth => 1.0,
                        _ => 0.0,
                    };
                if t <= 1.0 {
                    current = current.min(jumps);
                }
                (current / jumps) as f32
            }
            _ => self.apply(t),
        }
    }
}

fn bezier(p: [f32; 4], t: f64) -> f64 {
    let [x1, y1, x2, y2] = p.map(f64::from);
    if t < 0.0 {
        return t * if x1 > 0.0 {
            y1 / x1
        } else if x2 > 0.0 {
            y2 / x2
        } else {
            0.0
        };
    }
    if t > 1.0 {
        return 1.0
            + (t - 1.0)
                * if x2 < 1.0 {
                    (1.0 - y2) / (1.0 - x2)
                } else if x1 < 1.0 {
                    (1.0 - y1) / (1.0 - x1)
                } else {
                    0.0
                };
    }
    if t == 0.0 || t == 1.0 {
        return t;
    }
    let coordinate = |a: f64, b: f64, u: f64| {
        let v = 1.0 - u;
        3.0 * v * v * u * a + 3.0 * v * u * u * b + u * u * u
    };
    let (mut low, mut high) = (0.0, 1.0);
    // Cover the full f32 exponent range plus its significand, including
    // subnormal inputs; a short bisection loses tiny nonzero progress.
    for _ in 0..192 {
        let u = (low + high) * 0.5;
        let x = coordinate(x1, x2, u);
        if x == t {
            low = u;
            high = u;
        } else if x < t {
            low = u;
        } else {
            high = u;
        }
    }
    coordinate(y1, y2, (low + high) * 0.5)
}
