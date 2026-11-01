//! Deterministic rig angles, affine transforms and easing.

const PI: f64 = std::f64::consts::PI;
pub(super) const DEG: f64 = PI / 180.0;

/// sin and cos of `x` with basic arithmetic only (the camera systems' series).
pub fn sincos(x: f64) -> (f64, f64) {
    let q = (x / (PI / 2.0)).round();
    let r = x - q * (PI / 2.0);
    let r2 = r * r;
    let mut s = 1.0 / 1_307_674_368_000.0;
    for d in [6_227_020_800.0, 39_916_800.0, 362_880.0, 5040.0, 120.0, 6.0] {
        s = s * -r2 + 1.0 / d;
    }
    let s = r - r * r2 * s;
    let mut c = 1.0 / 87_178_291_200.0;
    for d in [479_001_600.0, 3_628_800.0, 40_320.0, 720.0, 24.0, 2.0] {
        c = c * -r2 + 1.0 / d;
    }
    let c = 1.0 - r2 * c;
    match (q as i64).rem_euclid(4) {
        0 => (s, c),
        1 => (c, -s),
        2 => (-s, -c),
        _ => (-c, s),
    }
}

fn atan_small(x: f64) -> f64 {
    let x2 = x * x;
    let mut s = 0.0;
    for n in (0..=22).rev() {
        s = s * -x2 + 1.0 / (2 * n + 1) as f64;
    }
    x * s
}

fn atan_unit(x: f64) -> f64 {
    if x > 0.414_213_562_373_095_03 {
        PI / 4.0 + atan_small((x - 1.0) / (x + 1.0))
    } else {
        atan_small(x)
    }
}

/// atan2(y, x) with basic arithmetic only.
pub fn atan2(y: f64, x: f64) -> f64 {
    if x == 0.0 && y == 0.0 {
        return 0.0;
    }
    let (ax, ay) = (x.abs(), y.abs());
    let a = if ay <= ax {
        atan_unit(ay / ax)
    } else {
        PI / 2.0 - atan_unit(ax / ay)
    };
    let r = if x < 0.0 { PI - a } else { a };
    if y < 0.0 { -r } else { r }
}

/// An angle wrapped into -pi..pi.
pub(super) fn wrap(a: f64) -> f64 {
    let k = ((a + PI) / (2.0 * PI)).floor();
    a - k * 2.0 * PI
}

/// A 2D affine transform: `(x, y)` maps to `(a x + c y + tx, b x + d y + ty)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine2 {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Affine2 {
    pub const IDENTITY: Affine2 = Affine2 {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    /// Translate, rotate (degrees, counter-clockwise), scale.
    pub fn trs(x: f32, y: f32, degrees: f32, sx: f32, sy: f32) -> Affine2 {
        let (s, c) = sincos(f64::from(degrees) * DEG);
        let (s, c) = (s as f32, c as f32);
        Affine2 {
            a: c * sx,
            b: s * sx,
            c: -s * sy,
            d: c * sy,
            tx: x,
            ty: y,
        }
    }

    /// [`Affine2::trs`] with the y axis turned `skew` degrees further than
    /// the x axis (counter-clockwise); skew 0 is `trs` to the bit.
    pub fn trsk(x: f32, y: f32, degrees: f32, skew: f32, sx: f32, sy: f32) -> Affine2 {
        let mut m = Affine2::trs(x, y, degrees, sx, sy);
        if skew != 0.0 {
            let (s, c) = sincos(f64::from(degrees + skew) * DEG);
            m.c = -(s as f32) * sy;
            m.d = c as f32 * sy;
        }
        m
    }

    pub fn mul(&self, l: &Affine2) -> Affine2 {
        Affine2 {
            a: self.a * l.a + self.c * l.b,
            b: self.b * l.a + self.d * l.b,
            c: self.a * l.c + self.c * l.d,
            d: self.b * l.c + self.d * l.d,
            tx: self.a * l.tx + self.c * l.ty + self.tx,
            ty: self.b * l.tx + self.d * l.ty + self.ty,
        }
    }

    pub fn apply(&self, x: f32, y: f32) -> [f32; 2] {
        [
            self.a * x + self.c * y + self.tx,
            self.b * x + self.d * y + self.ty,
        ]
    }

    /// [`Affine2::apply`] of a point.
    pub fn apply_to(&self, p: [f32; 2]) -> [f32; 2] {
        self.apply(p[0], p[1])
    }

    /// The inverse (a singular transform inverts to the identity).
    pub fn inverse(&self) -> Affine2 {
        let det = self.a * self.d - self.b * self.c;
        if det == 0.0 {
            return Affine2::IDENTITY;
        }
        let i = 1.0 / det;
        Affine2 {
            a: self.d * i,
            b: -self.b * i,
            c: -self.c * i,
            d: self.a * i,
            tx: (self.c * self.ty - self.d * self.tx) * i,
            ty: (self.b * self.tx - self.a * self.ty) * i,
        }
    }

    /// The direction of the x axis, radians.
    pub(super) fn angle(&self) -> f64 {
        atan2(f64::from(self.b), f64::from(self.a))
    }
}

/// Whether a Bézier's numbers make a curve over time: 4 + 6n of them,
/// every x in 0..1, the anchors in time order.
pub fn check_curve(c: &[f32]) -> std::result::Result<(), String> {
    if c.len() < 4 || !(c.len() - 4).is_multiple_of(6) {
        return Err(format!(
            "a curve is x1 y1 x2 y2, then x y x1 y1 x2 y2 per further segment ({} numbers)",
            c.len()
        ));
    }
    if c.iter().step_by(2).any(|x| !(0.0..=1.0).contains(x)) {
        return Err("a curve's x values (time) are within 0..1".into());
    }
    let anchors: Vec<f32> = (1..=(c.len() - 4) / 6).map(|k| c[6 * k - 2]).collect();
    if anchors.windows(2).any(|w| w[1] < w[0]) {
        return Err("a curve's anchors are in time order".into());
    }
    Ok(())
}

/// The value fraction of a Bézier ease at time fraction `x`: the segment
/// whose anchors span `x`, solved for its parameter by bisection.
pub fn bezier(c: &[f32], x: f32) -> f32 {
    let g = |i: usize| f64::from(c[i]);
    let segments = (c.len() - 4) / 6 + 1;
    let anchor = |k: usize| -> (f64, f64) {
        if k == 0 {
            (0.0, 0.0)
        } else if k == segments {
            (1.0, 1.0)
        } else {
            (g(6 * k - 2), g(6 * k - 1))
        }
    };
    let x = f64::from(x).clamp(0.0, 1.0);
    let mut k = 0;
    while k + 1 < segments && anchor(k + 1).0 < x {
        k += 1;
    }
    let (p0, p3) = (anchor(k), anchor(k + 1));
    let (p1, p2) = ((g(6 * k), g(6 * k + 1)), (g(6 * k + 2), g(6 * k + 3)));
    let at = |s: f64| -> (f64, f64) {
        let l = 1.0 - s;
        let (a, b, c, d) = (l * l * l, 3.0 * l * l * s, 3.0 * l * s * s, s * s * s);
        (
            a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
            a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
        )
    };
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..40 {
        let m = (lo + hi) * 0.5;
        if at(m).0 < x {
            lo = m;
        } else {
            hi = m;
        }
    }
    at((lo + hi) * 0.5).1 as f32
}
