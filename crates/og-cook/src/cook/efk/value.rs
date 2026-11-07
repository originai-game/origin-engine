//! Effekseer's parameter values as the editor saves them (`Data/Value/*.cs`,
//! `Data/Define.cs`, MIT; `third_party/effekseer/`): a value the editor left
//! at its default is not written, so every reader takes the default the
//! editor's constructor gives. Times are frames (60 a second).

use super::xml::El;
use glam::Vec3;

/// A uniform random range (`FloatWithRandom`'s `Min` and `Max`; its
/// `Center` is their mean).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rng {
    pub min: f32,
    pub max: f32,
}

impl Rng {
    pub const fn fixed(v: f32) -> Rng {
        Rng { min: v, max: v }
    }

    pub fn c(&self) -> f32 {
        (self.min + self.max) * 0.5
    }

    /// Half the range.
    pub fn h(&self) -> f32 {
        (self.max - self.min) * 0.5
    }

    pub fn scaled(&self, s: f32) -> Rng {
        let (a, b) = (self.min * s, self.max * s);
        Rng {
            min: a.min(b),
            max: a.max(b),
        }
    }
}

pub fn num(e: Option<&El>) -> Option<f32> {
    e.and_then(|e| e.text.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite())
}

pub fn f(e: &El, path: &str, d: f32) -> f32 {
    num(e.at(path)).unwrap_or(d)
}

pub fn int(e: &El, path: &str, d: i32) -> i32 {
    e.text_at(path)
        .and_then(|t| t.trim().parse::<i32>().ok())
        .unwrap_or(d)
}

pub fn boolean(e: &El, path: &str, d: bool) -> bool {
    match e.text_at(path) {
        Some(t) if t.eq_ignore_ascii_case("true") => true,
        Some(t) if t.eq_ignore_ascii_case("false") => false,
        _ => d,
    }
}

/// A `FloatWithRandom` (or `IntWithRandom`, or an older plain number).
pub fn rnd(e: Option<&El>, d: f32) -> Rng {
    let Some(e) = e else {
        return Rng::fixed(d);
    };
    if e.children.is_empty() {
        return Rng::fixed(num(Some(e)).unwrap_or(d));
    }
    // The editor sets the centre first (moving the range with it), then
    // the range; a range it did not write is the default.
    let c = num(e.child("Center"));
    let max = num(e.child("Max")).or(c).unwrap_or(d);
    let min = num(e.child("Min")).or(c).unwrap_or(d);
    Rng {
        min: min.min(max),
        max: min.max(max),
    }
}

pub fn v3(e: Option<&El>, d: Vec3) -> Vec3 {
    let Some(e) = e else { return d };
    Vec3::new(
        num(e.child("X")).unwrap_or(d.x),
        num(e.child("Y")).unwrap_or(d.y),
        num(e.child("Z")).unwrap_or(d.z),
    )
}

pub fn v3rnd(e: Option<&El>, d: Vec3) -> [Rng; 3] {
    [
        rnd(e.and_then(|e| e.child("X")), d.x),
        rnd(e.and_then(|e| e.child("Y")), d.y),
        rnd(e.and_then(|e| e.child("Z")), d.z),
    ]
}

pub fn v2rnd(e: Option<&El>, d: [f32; 2]) -> [Rng; 2] {
    [
        rnd(e.and_then(|e| e.child("X")), d[0]),
        rnd(e.and_then(|e| e.child("Y")), d[1]),
    ]
}

pub fn v2(e: Option<&El>, d: [f32; 2]) -> [f32; 2] {
    let Some(e) = e else { return d };
    [
        num(e.child("X")).unwrap_or(d[0]),
        num(e.child("Y")).unwrap_or(d[1]),
    ]
}

/// `tan` of degrees, by og-core's deterministic sin and cos.
fn tan_deg(d: f32) -> f32 {
    let (s, c) = og_core::math::sin_cos(d.to_radians());
    s / c
}

/// An easing from 0 at t 0 to 1 at t 1 (`EasingType`; `LeftRightSpeed`
/// is the editor's cubic from its start and end speeds, `Utils/Math.cs`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ease {
    Cubic([f32; 3]),
    Kind(i32),
}

impl Ease {
    pub const LINEAR: Ease = Ease::Kind(1);

    /// The easing of an easing parameter block (`Type`, `StartSpeed`, `EndSpeed`).
    pub fn read(e: Option<&El>) -> Ease {
        let Some(e) = e else {
            return Ease::LINEAR;
        };
        match int(e, "Type", 0) {
            0 => {
                let g1 = tan_deg(f(e, "StartSpeed", 0.0) + 45.0);
                let g2 = tan_deg(f(e, "EndSpeed", 0.0) + 45.0);
                let c = g1;
                let a = g2 - g1 - (1.0 - c) * 2.0;
                let b = (g2 - g1 - a * 3.0) / 2.0;
                Ease::Cubic([a, b, c])
            }
            k => Ease::Kind(k),
        }
    }

    pub fn eval(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let p = |t: f32, n: i32| (0..n).fold(1.0, |a, _| a * t);
        let io = |t: f32, n: i32| {
            if t < 0.5 {
                p(2.0 * t, n) * 0.5
            } else {
                1.0 - p(2.0 - 2.0 * t, n) * 0.5
            }
        };
        let bounce = |t: f32| {
            let (n, d) = (7.5625, 2.75);
            if t < 1.0 / d {
                n * t * t
            } else if t < 2.0 / d {
                let t = t - 1.5 / d;
                n * t * t + 0.75
            } else if t < 2.5 / d {
                let t = t - 2.25 / d;
                n * t * t + 0.9375
            } else {
                let t = t - 2.625 / d;
                n * t * t + 0.984375
            }
        };
        let (c1, c3) = (1.70158, 2.70158);
        match *self {
            Ease::Cubic([a, b, c]) => a * t * t * t + b * t * t + c * t,
            Ease::Kind(k) => match k {
                10 | 20 | 30 | 40 => p(t, k / 10 + 1),
                11 | 21 | 31 | 41 => 1.0 - p(1.0 - t, k / 10 + 1),
                12 | 22 | 32 | 42 => io(t, k / 10 + 1),
                50 => c3 * t * t * t - c1 * t * t,
                51 => {
                    let u = t - 1.0;
                    1.0 + c3 * u * u * u + c1 * u * u
                }
                52 => {
                    let c2 = c1 * 1.525;
                    if t < 0.5 {
                        (2.0 * t) * (2.0 * t) * ((c2 + 1.0) * 2.0 * t - c2) / 2.0
                    } else {
                        let u = 2.0 * t - 2.0;
                        (u * u * ((c2 + 1.0) * u + c2) + 2.0) / 2.0
                    }
                }
                60 => 1.0 - bounce(1.0 - t),
                61 => bounce(t),
                62 => {
                    if t < 0.5 {
                        (1.0 - bounce(1.0 - 2.0 * t)) / 2.0
                    } else {
                        (1.0 + bounce(2.0 * t - 1.0)) / 2.0
                    }
                }
                _ => t,
            },
        }
    }
}

/// One channel of an f-curve: keys of (frame, value), read as linear between
/// keys (the editor's bezier handles are not followed), constant beyond.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FCurve {
    pub keys: Vec<(f32, f32)>,
    /// The mean of the random offset (`OffsetMin`, `OffsetMax`).
    pub offset: f32,
    /// Frames are percents of the life (`Timeline` 1).
    pub percent: bool,
    /// The value without keys.
    pub default: f32,
}

impl FCurve {
    fn read(e: Option<&El>, percent: bool, default: f32) -> FCurve {
        let mut c = FCurve {
            percent,
            default,
            ..FCurve::default()
        };
        let Some(e) = e else { return c };
        for k in &e.children {
            if k.name.starts_with("Key") {
                c.keys.push((f(k, "Frame", 0.0), f(k, "Value", 0.0)));
            }
        }
        c.keys.sort_by(|a, b| a.0.total_cmp(&b.0));
        c.offset = (f(e, "OffsetMin", 0.0) + f(e, "OffsetMax", 0.0)) * 0.5;
        c
    }

    /// The channels of an `FCurveVector3D`/`FCurveColorRGBA` element.
    pub fn channels(e: Option<&El>, names: &[&str], default: f32) -> Vec<FCurve> {
        let keys = e.and_then(|e| e.child("Keys"));
        let percent = keys.is_some_and(|k| int(k, "Timeline", 0) == 1);
        names
            .iter()
            .map(|n| FCurve::read(keys.and_then(|k| k.child(n)), percent, default))
            .collect()
    }

    /// The value at `frame` of an instance living `life` frames.
    pub fn eval(&self, frame: f32, life: f32) -> f32 {
        let x = if self.percent {
            frame / life.max(1e-3) * 100.0
        } else {
            frame
        };
        let v = match self.keys.as_slice() {
            [] => self.default,
            [(_, v)] => *v,
            keys => {
                let i = keys.partition_point(|k| k.0 <= x);
                if i == 0 {
                    keys[0].1
                } else if i == keys.len() {
                    keys[i - 1].1
                } else {
                    let (a, b) = (keys[i - 1], keys[i]);
                    let t = if b.0 > a.0 {
                        (x - a.0) / (b.0 - a.0)
                    } else {
                        0.0
                    };
                    a.1 + (b.1 - a.1) * t
                }
            }
        };
        v + self.offset
    }
}

/// A scalar over an instance's life: fixed, position-velocity-acceleration,
/// eased between two values, or an f-curve.
#[derive(Clone, Debug, PartialEq)]
pub enum Anim {
    Fixed(f32),
    Pva { p: Rng, v: Rng, a: Rng },
    Easing { s: Rng, e: Rng, ease: Ease },
    Curve(FCurve),
}

impl Anim {
    /// Its centre value `frame` frames into a life of `life` frames.
    pub fn at(&self, frame: f32, life: f32) -> f32 {
        match self {
            Anim::Fixed(v) => *v,
            // Effekseer steps p += v + a·k per frame: v·n + a·n(n+1)/2.
            Anim::Pva { p, v, a } => p.c() + v.c() * frame + a.c() * frame * (frame + 1.0) * 0.5,
            Anim::Easing { s, e, ease } => {
                s.c() + (e.c() - s.c()) * ease.eval(frame / life.max(1e-3))
            }
            Anim::Curve(c) => c.eval(frame, life),
        }
    }

    /// Its value range at birth.
    pub fn start(&self) -> Rng {
        match self {
            Anim::Fixed(v) => Rng::fixed(*v),
            Anim::Pva { p, .. } => *p,
            Anim::Easing { s, .. } => *s,
            Anim::Curve(c) => Rng::fixed(c.eval(0.0, 1.0)),
        }
    }

    /// The range of its mean rate per frame over a life of `life` frames
    /// (a velocity for a position).
    pub fn rate(&self, life: f32) -> Rng {
        let l = life.max(1.0);
        match self {
            Anim::Fixed(_) => Rng::fixed(0.0),
            Anim::Pva { v, .. } => *v,
            Anim::Easing { s, e, .. } => Rng {
                min: (e.min - s.max) / l,
                max: (e.max - s.min) / l,
            },
            Anim::Curve(c) => Rng::fixed((c.eval(l, l) - c.eval(0.0, l)) / l),
        }
    }

    /// Its constant acceleration per frame² (PVA only).
    pub fn accel(&self) -> Rng {
        match self {
            Anim::Pva { a, .. } => *a,
            _ => Rng::fixed(0.0),
        }
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, Anim::Fixed(_))
    }
}

/// A vector of three scalars over the life.
pub type Anim3 = [Anim; 3];

pub fn fixed3(v: Vec3) -> Anim3 {
    [Anim::Fixed(v.x), Anim::Fixed(v.y), Anim::Fixed(v.z)]
}

pub fn at3(a: &Anim3, frame: f32, life: f32) -> Vec3 {
    Vec3::new(
        a[0].at(frame, life),
        a[1].at(frame, life),
        a[2].at(frame, life),
    )
}

/// The Fixed / PVA / Easing / FCurve parameter of a location, rotation or
/// scale block, by its `Type` (`fixed`, `pva`, `easing`, `fcurve` give each
/// type's element name and the vector inside it).
pub struct Kinds<'a> {
    pub fixed: (&'a str, &'a str),
    pub pva: (&'a str, [&'a str; 3]),
    pub easing: &'a str,
    pub fcurve: &'a str,
}

pub fn anim3(e: &El, ty: i32, k: &Kinds<'_>, ids: [i32; 4], d: Vec3) -> Option<Anim3> {
    let [fixed, pva, easing, fcurve] = ids;
    Some(if ty == fixed {
        fixed3(v3(e.at(k.fixed.0).and_then(|f| f.child(k.fixed.1)), d))
    } else if ty == pva {
        let b = e.child(k.pva.0);
        let p = v3rnd(b.and_then(|b| b.child(k.pva.1[0])), d);
        let v = v3rnd(b.and_then(|b| b.child(k.pva.1[1])), Vec3::ZERO);
        let a = v3rnd(b.and_then(|b| b.child(k.pva.1[2])), Vec3::ZERO);
        std::array::from_fn(|i| Anim::Pva {
            p: p[i],
            v: v[i],
            a: a[i],
        })
    } else if ty == easing {
        let b = e.child(k.easing);
        let s = v3rnd(b.and_then(|b| b.child("Start")), d);
        let en = v3rnd(b.and_then(|b| b.child("End")), d);
        let ease = Ease::read(b);
        std::array::from_fn(|i| Anim::Easing {
            s: s[i],
            e: en[i],
            ease,
        })
    } else if ty == fcurve {
        let c = FCurve::channels(
            e.at(k.fcurve).and_then(|b| b.child("FCurve")),
            &["X", "Y", "Z"],
            0.0,
        );
        let mut it = c.into_iter().map(Anim::Curve);
        [it.next()?, it.next()?, it.next()?]
    } else {
        return None;
    })
}

/// A colour, 0..1 per channel as the editor's 0..255 (gamma, not linear).
pub type Rgba = [f32; 4];

/// A `Color`'s channels (`R G B A`, 0 to 255).
pub fn color(e: Option<&El>, d: [f32; 4]) -> Rgba {
    let Some(e) = e else {
        return d.map(|c| c / 255.0);
    };
    let ch = |n: &str, d: f32| num(e.child(n)).unwrap_or(d) / 255.0;
    [ch("R", d[0]), ch("G", d[1]), ch("B", d[2]), ch("A", d[3])]
}

/// A `ColorWithRandom`'s ranges (0..1), HSV ones turned to RGB at their ends.
pub fn color_rnd(e: Option<&El>, d: [f32; 4]) -> [Rng; 4] {
    let ch = |n: &str, d: f32| rnd(e.and_then(|e| e.child(n)), d).scaled(1.0 / 255.0);
    let c = [ch("R", d[0]), ch("G", d[1]), ch("B", d[2]), ch("A", d[3])];
    if e.is_some_and(|e| int(e, "ColorSpace", 0) == 1) {
        let lo = hsv([c[0].min, c[1].min, c[2].min]);
        let hi = hsv([c[0].max, c[1].max, c[2].max]);
        return [
            Rng {
                min: lo[0].min(hi[0]),
                max: lo[0].max(hi[0]),
            },
            Rng {
                min: lo[1].min(hi[1]),
                max: lo[1].max(hi[1]),
            },
            Rng {
                min: lo[2].min(hi[2]),
                max: lo[2].max(hi[2]),
            },
            c[3],
        ];
    }
    c
}

/// HSV (0..1 each, hue a turn) to RGB.
fn hsv([h, s, v]: [f32; 3]) -> [f32; 3] {
    let h = (h.fract() + 1.0).fract() * 6.0;
    let i = h.floor();
    let fr = h - i;
    let (p, q, t) = (
        v * (1.0 - s),
        v * (1.0 - s * fr),
        v * (1.0 - s * (1.0 - fr)),
    );
    match i as i32 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

/// A colour over the life (`StandardColor`, or an older renderer's
/// `ColorAll` with its `_Fixed`, `_Random`, `_Easing`, `_FCurve` beside it).
#[derive(Clone, Debug, PartialEq)]
pub enum Color {
    Fixed(Rgba),
    Random([Rng; 4]),
    Easing {
        s: [Rng; 4],
        e: [Rng; 4],
        ease: Ease,
    },
    Curve(Vec<FCurve>),
    /// A gradient (1.7+), not read: white.
    Gradient,
}

impl Color {
    pub const WHITE: Color = Color::Fixed([1.0; 4]);

    /// A `StandardColor` element (`Type`, `Fixed`, `Random`, `Easing`, `FCurve`).
    pub fn standard(e: Option<&El>, d: [f32; 4]) -> Color {
        let Some(e) = e else {
            return Color::Fixed(d.map(|c| c / 255.0));
        };
        Color::of(
            int(e, "Type", 0),
            e.child("Fixed"),
            e.child("Random"),
            e.child("Easing"),
            e.at("FCurve/FCurve"),
            d,
        )
    }

    /// An older renderer's `<prefix>`, `<prefix>_Fixed`, ... elements.
    pub fn prefixed(e: &El, prefix: &str, d: [f32; 4]) -> Color {
        Color::of(
            int(e, prefix, 0),
            e.child(&format!("{prefix}_Fixed")),
            e.child(&format!("{prefix}_Random")),
            e.child(&format!("{prefix}_Easing")),
            e.at(&format!("{prefix}_FCurve/FCurve")),
            d,
        )
    }

    fn of(
        ty: i32,
        fixed: Option<&El>,
        random: Option<&El>,
        easing: Option<&El>,
        fcurve: Option<&El>,
        d: [f32; 4],
    ) -> Color {
        match ty {
            1 => Color::Random(color_rnd(random, d)),
            2 => Color::Easing {
                s: color_rnd(easing.and_then(|e| e.child("Start")), d),
                e: color_rnd(easing.and_then(|e| e.child("End")), d),
                ease: Ease::read(easing),
            },
            3 => Color::Curve(
                FCurve::channels(fcurve, &["R", "G", "B", "A"], 255.0)
                    .into_iter()
                    .map(|mut c| {
                        for k in &mut c.keys {
                            k.1 /= 255.0;
                        }
                        c.offset /= 255.0;
                        c.default /= 255.0;
                        c
                    })
                    .collect(),
            ),
            4 => Color::Gradient,
            _ => Color::Fixed(color(fixed, d)),
        }
    }

    /// Its centre colour at `frame` of a life of `life` frames.
    pub fn at(&self, frame: f32, life: f32) -> Rgba {
        match self {
            Color::Fixed(c) => *c,
            Color::Random(r) => r.map(|r| r.c()),
            Color::Easing { s, e, ease } => {
                let t = ease.eval(frame / life.max(1e-3));
                std::array::from_fn(|i| s[i].c() + (e[i].c() - s[i].c()) * t)
            }
            Color::Curve(c) => {
                std::array::from_fn(|i| c.get(i).map_or(1.0, |c| c.eval(frame, life)))
            }
            Color::Gradient => [1.0; 4],
        }
    }

    /// Whether its channels vary from one instance to the next.
    pub fn random_spread(&self) -> f32 {
        let spread = |r: &[Rng; 4]| r[..3].iter().map(|r| r.max - r.min).fold(0.0, f32::max);
        match self {
            Color::Random(r) => spread(r),
            Color::Easing { s, e, .. } => spread(s).max(spread(e)),
            _ => 0.0,
        }
    }
}
