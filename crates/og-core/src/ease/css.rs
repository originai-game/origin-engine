//! CSS Easing Level 1 evaluation; no host math or adaptive solver.
use super::Ease;

#[repr(u32)]
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
            Self::Points(p) => p.len() >= 2 && p.iter().flatten().all(|v| v.is_finite()),
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
                    ("linear", stops) => linear(stops)?,
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
            Self::Points(p) => {
                let stops: Vec<String> = p
                    .iter()
                    .map(|[i, o]| format!("{o} {}%", i * 100.0))
                    .collect();
                format!("linear({})", stops.join(", "))
            }
            _ => self.name().into(),
        }
    }

    pub fn from_text(text: &str) -> Option<Self> {
        Self::from_name(text).or_else(|| {
            if text.starts_with("cubic-bezier(")
                || text.starts_with("steps(")
                || text.starts_with("linear(")
            {
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
            // CSS Easing 2 §2.2.2: the segment the input falls in, its
            // first or last extrapolated outside them.
            Self::Points(p) => {
                let mut a = p.iter().rposition(|q| q[0] <= t).unwrap_or(0);
                if a + 1 >= p.len() {
                    a = p.len() - 2;
                }
                let (pa, pb) = (p[a], p[a + 1]);
                if pa[0] == pb[0] {
                    return pb[1];
                }
                pa[1] + (t - pa[0]) / (pb[0] - pa[0]) * (pb[1] - pa[1])
            }
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

/// CSS Easing 2 §2.2.1: `linear()`'s stops (`<number> <percentage>{0,2}`)
/// as canonical points: the first and last inputs default to 0% and 100%,
/// an input under an earlier one is raised to it, and runs without one
/// are spread evenly between their neighbours.
fn linear(stops: &[&str]) -> Option<Ease> {
    let mut points: Vec<(f32, Option<f32>)> = Vec::new();
    for stop in stops {
        let mut words = stop.split_whitespace();
        let mut output = None;
        let mut inputs = Vec::new();
        for w in words.by_ref() {
            if let Some(p) = w.strip_suffix('%') {
                inputs.push(p.parse::<f32>().ok()? / 100.0);
            } else if output.is_none() {
                output = Some(w.parse::<f32>().ok()?);
            } else {
                return None;
            }
        }
        let output = output?;
        if inputs.len() > 2 {
            return None;
        }
        if inputs.is_empty() {
            points.push((output, None));
        }
        for i in inputs {
            points.push((output, Some(i)));
        }
    }
    if points.len() < 2 || points.iter().any(|p| !p.0.is_finite()) {
        return None;
    }
    let n = points.len();
    points[0].1.get_or_insert(0.0);
    points[n - 1].1.get_or_insert(1.0);
    let mut largest = f32::MIN;
    for p in &mut points {
        if let Some(i) = &mut p.1 {
            *i = i.max(largest);
            largest = *i;
        }
    }
    let mut k = 0;
    while k < n {
        if points[k].1.is_some() {
            k += 1;
            continue;
        }
        let start = k - 1;
        let end = (k..n).find(|&j| points[j].1.is_some())?;
        let (a, b) = (points[start].1?, points[end].1?);
        for (step, p) in points[start + 1..end].iter_mut().enumerate() {
            p.1 = Some(a + (b - a) * (step + 1) as f32 / (end - start) as f32);
        }
        k = end;
    }
    let canonical: Vec<[f32; 2]> = points.iter().map(|&(o, i)| [i.unwrap_or(0.0), o]).collect();
    if !canonical.iter().flatten().all(|v| v.is_finite()) {
        return None;
    }
    Some(Ease::linear(&canonical))
}

/// FNV-1a over the points' bits: a `linear()` curve's wire identity.
pub(super) fn points_hash(p: &[[f32; 2]]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for v in p.iter().flatten() {
        for b in v.to_bits().to_le_bytes() {
            h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// Every `linear()` curve made in this process, once each.
static POINTS: std::sync::Mutex<Vec<&'static [[f32; 2]]>> = std::sync::Mutex::new(Vec::new());

/// The interned curve of that hash, if this process has made it.
pub(super) fn interned(hash: u64) -> Option<&'static [[f32; 2]]> {
    let all = POINTS.lock().unwrap_or_else(|e| e.into_inner());
    all.iter().copied().find(|p| points_hash(p) == hash)
}

impl Ease {
    /// A `linear()` curve's identity: its points' hash.
    pub fn points_hash(self) -> Option<u64> {
        match self {
            Ease::Points(p) => Some(points_hash(p)),
            _ => None,
        }
    }

    /// The `linear()` curve of that hash, when this process has made it
    /// (a document defines its curves before a saved state reads them).
    pub fn from_points_hash(hash: u64) -> Option<Ease> {
        interned(hash).map(Ease::Points)
    }

    /// A `linear()` curve of canonical points (inputs ascending). Each
    /// distinct curve is kept for the process's life, as a style's
    /// strings are: documents make a handful.
    pub fn linear(points: &[[f32; 2]]) -> Ease {
        let mut all = POINTS.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(p) = all.iter().copied().find(|p| *p == points) {
            return Ease::Points(p);
        }
        let p: &'static [[f32; 2]] = Box::leak(points.to_vec().into_boxed_slice());
        all.push(p);
        Ease::Points(p)
    }
}

#[cfg(test)]
mod linear_tests {
    use super::*;

    #[test]
    fn linear_stops_canonicalise_and_sample_as_css_easing_2_says() {
        // linear(0, 0.25 75%, 1): (0, 0) (0.75, 0.25) (1, 1).
        let e = Ease::from_css("linear(0, 0.25 75%, 1)").unwrap();
        assert!((e.apply(0.375) - 0.125).abs() < 1e-6);
        assert!((e.apply(0.875) - 0.625).abs() < 1e-6);
        // Two inputs make a flat run; a missing input is spread between
        // its neighbours; an input under an earlier one is raised.
        let Ease::Points(p) = Ease::from_css("linear(0, 0.5 25% 75%, 1)").unwrap() else {
            panic!()
        };
        assert_eq!(p, &[[0.0, 0.0], [0.25, 0.5], [0.75, 0.5], [1.0, 1.0]]);
        let Ease::Points(p) = Ease::from_css("linear(0 20%, 0.1, 0.2, 1 10%)").unwrap() else {
            panic!()
        };
        assert_eq!(p[1][0], 0.2, "{p:?}");
        assert_eq!(p[3][0], 0.2, "raised to the largest earlier input");
        // Same curve, same value; and its wire form round-trips.
        assert_eq!(e, Ease::from_css("linear(0 0%, 0.25 75%, 1 100%)").unwrap());
        assert_eq!(Ease::from_bytes(&e.to_bytes()), Some(e));
        for bad in [
            "linear(1)",
            "linear()",
            "linear(0, x)",
            "linear(0 1% 2% 3%, 1)",
        ] {
            assert_eq!(Ease::from_css(bad), None, "{bad}");
        }
    }
}
