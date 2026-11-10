use crate::layout::Rect;

/// A map of CSS px, DOMMatrix's `a`–`f` with a projective row `g h i`:
/// `x' = (a·x + c·y + e) / w`, `y' = (b·x + d·y + f) / w`,
/// `w = g·x + h·y + i` (y down). It is affine (`g = h = 0`, `i = 1`)
/// except where a CSS 3D transform puts an element on a plane in
/// perspective; the draw then lays that element's layer through it
/// ([`crate::ui`]'s 3D planes) and hit tests map through its inverse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
    pub g: f32,
    pub h: f32,
    pub i: f32,
}

impl Default for Affine {
    fn default() -> Affine {
        Affine::IDENTITY
    }
}

impl Affine {
    pub const IDENTITY: Affine = Affine {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
        g: 0.0,
        h: 0.0,
        i: 1.0,
    };

    /// The 2D affine map `[a, b, c, d, e, f]`.
    pub const fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Affine {
        Affine {
            a,
            b,
            c,
            d,
            e,
            f,
            ..Affine::IDENTITY
        }
    }

    pub const fn translate(x: f32, y: f32) -> Affine {
        Affine {
            e: x,
            f: y,
            ..Affine::IDENTITY
        }
    }

    pub const fn scale(sx: f32, sy: f32) -> Affine {
        Affine {
            a: sx,
            d: sy,
            ..Affine::IDENTITY
        }
    }

    /// A clockwise turn on screen by `angle` radians (y down), with the
    /// engine's deterministic sine and cosine.
    pub fn rotate(angle: f32) -> Affine {
        let (s, c) = og_core::math::sin_cos(angle);
        Affine {
            a: c,
            b: s,
            c: -s,
            d: c,
            ..Affine::IDENTITY
        }
    }

    /// CSS `skew(x, y)`, radians.
    pub fn skew(x: f32, y: f32) -> Affine {
        let tan = |v: f32| {
            if v == 0.0 {
                return 0.0;
            }
            let (s, c) = og_core::math::sin_cos(v);
            s / c
        };
        Affine {
            b: tan(y),
            c: tan(x),
            ..Affine::IDENTITY
        }
    }

    /// `self × o`: `o` first, then `self`.
    pub fn mul(&self, o: &Affine) -> Affine {
        if self.is_affine() && o.is_affine() {
            return Affine {
                a: self.a * o.a + self.c * o.b,
                b: self.b * o.a + self.d * o.b,
                c: self.a * o.c + self.c * o.d,
                d: self.b * o.c + self.d * o.d,
                e: self.a * o.e + self.c * o.f + self.e,
                f: self.b * o.e + self.d * o.f + self.f,
                ..Affine::IDENTITY
            };
        }
        let (l, r) = (self.rows(), o.rows());
        let at = |i: usize, j: usize| (0..3).map(|k| l[i][k] * r[k][j]).sum::<f32>();
        Affine::from_rows([
            [at(0, 0), at(0, 1), at(0, 2)],
            [at(1, 0), at(1, 1), at(1, 2)],
            [at(2, 0), at(2, 1), at(2, 2)],
        ])
    }

    /// Its 3×3 matrix by rows (`[[a, c, e], [b, d, f], [g, h, i]]`).
    pub fn rows(&self) -> [[f32; 3]; 3] {
        [
            [self.a, self.c, self.e],
            [self.b, self.d, self.f],
            [self.g, self.h, self.i],
        ]
    }

    pub fn from_rows(m: [[f32; 3]; 3]) -> Affine {
        Affine {
            a: m[0][0],
            c: m[0][1],
            e: m[0][2],
            b: m[1][0],
            d: m[1][1],
            f: m[1][2],
            g: m[2][0],
            h: m[2][1],
            i: m[2][2],
        }
    }

    /// No projective part: it keeps parallel lines parallel.
    pub fn is_affine(&self) -> bool {
        self.g == 0.0 && self.h == 0.0 && self.i == 1.0
    }

    /// `self` about the point `(px, py)`: `T(p) · self · T(-p)`.
    pub fn about(&self, px: f32, py: f32) -> Affine {
        Affine::translate(px, py)
            .mul(self)
            .mul(&Affine::translate(-px, -py))
    }

    /// Where `(x, y)` lands (behind the eye, where w is not positive, at
    /// infinity).
    pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        let (px, py) = (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        );
        if self.is_affine() {
            return (px, py);
        }
        let w = self.g * x + self.h * y + self.i;
        if w <= 1e-6 {
            return (px.signum() * f32::MAX, py.signum() * f32::MAX);
        }
        (px / w, py / w)
    }

    pub fn is_identity(&self) -> bool {
        *self == Affine::IDENTITY
    }

    /// Translation and scale only: it maps a rectangle to a rectangle.
    pub fn is_axis_aligned(&self) -> bool {
        self.b == 0.0 && self.c == 0.0 && self.is_affine()
    }

    /// The determinant of its linear part (its 3×3's for a projective map).
    pub fn determinant(&self) -> f32 {
        if self.is_affine() {
            return self.a * self.d - self.b * self.c;
        }
        let m = self.rows();
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    /// The inverse (`None` for a map that flattens the plane).
    pub fn invert(&self) -> Option<Affine> {
        let det = self.determinant();
        if det.abs() < 1e-12 || !det.is_finite() {
            return None;
        }
        let k = 1.0 / det;
        if self.is_affine() {
            return Some(Affine {
                a: self.d * k,
                b: -self.b * k,
                c: -self.c * k,
                d: self.a * k,
                e: (self.c * self.f - self.d * self.e) * k,
                f: (self.b * self.e - self.a * self.f) * k,
                ..Affine::IDENTITY
            });
        }
        let m = self.rows();
        let cof = |r0: usize, r1: usize, c0: usize, c1: usize| {
            m[r0][c0] * m[r1][c1] - m[r0][c1] * m[r1][c0]
        };
        Some(Affine::from_rows([
            [
                cof(1, 2, 1, 2) * k,
                -cof(0, 2, 1, 2) * k,
                cof(0, 1, 1, 2) * k,
            ],
            [
                -cof(1, 2, 0, 2) * k,
                cof(0, 2, 0, 2) * k,
                -cof(0, 1, 0, 2) * k,
            ],
            [
                cof(1, 2, 0, 1) * k,
                -cof(0, 2, 0, 1) * k,
                cof(0, 1, 0, 1) * k,
            ],
        ]))
    }

    /// The axis-aligned bounds of `r` mapped.
    pub fn map_rect(&self, r: &Rect) -> Rect {
        if self.is_identity() {
            return *r;
        }
        let pts = [
            self.apply(r.x, r.y),
            self.apply(r.right(), r.y),
            self.apply(r.x, r.bottom()),
            self.apply(r.right(), r.bottom()),
        ];
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for (x, y) in pts {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    }

    /// The map that takes rectangle `from` onto `to` (translate and scale).
    pub fn rect_to_rect(from: &Rect, to: &Rect) -> Affine {
        let sx = if from.width > 0.0 {
            to.width / from.width
        } else {
            1.0
        };
        let sy = if from.height > 0.0 {
            to.height / from.height
        } else {
            1.0
        };
        Affine {
            a: sx,
            d: sy,
            e: to.x - from.x * sx,
            f: to.y - from.y * sy,
            ..Affine::IDENTITY
        }
    }
}

/// A transform applied after layout, about `pivot` (fractions of the
/// element's box; the zero value's pivot is its centre): CSS's
/// `translate(x, y) rotate(r) skew(kx, ky) scale(sx, sy)` with its
/// `transform-origin`. Hit tests, the probe's rectangles and the
/// accessibility tree's bounds follow it; layout does not.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// CSS px.
    pub x: f32,
    pub y: f32,
    /// Fractions of the box's own width and height, added to `x` and `y`
    /// (CSS `translate`'s percentages: -0.5 centres the box on its place).
    pub shift_x: f32,
    pub shift_y: f32,
    /// 1: unscaled.
    pub scale_x: f32,
    pub scale_y: f32,
    /// Radians, clockwise on screen.
    pub rotate: f32,
    /// Radians.
    pub skew_x: f32,
    pub skew_y: f32,
    /// Fractions of the box (0.5, 0.5: the centre).
    pub pivot_x: f32,
    pub pivot_y: f32,
}

impl Default for Transform {
    fn default() -> Transform {
        Transform::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        x: 0.0,
        y: 0.0,
        shift_x: 0.0,
        shift_y: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        rotate: 0.0,
        skew_x: 0.0,
        skew_y: 0.0,
        pivot_x: 0.5,
        pivot_y: 0.5,
    };

    /// Moves nothing (the pivot aside).
    pub fn is_identity(&self) -> bool {
        self.x == 0.0
            && self.y == 0.0
            && self.shift_x == 0.0
            && self.shift_y == 0.0
            && self.scale_x == 1.0
            && self.scale_y == 1.0
            && self.rotate == 0.0
            && self.skew_x == 0.0
            && self.skew_y == 0.0
    }

    /// The map for an element laid out at `rect`.
    pub fn matrix(&self, rect: &Rect) -> Affine {
        if self.is_identity() {
            return Affine::IDENTITY;
        }
        let px = rect.x + self.pivot_x * rect.width;
        let py = rect.y + self.pivot_y * rect.height;
        let mut m = Affine::translate(
            self.x + self.shift_x * rect.width,
            self.y + self.shift_y * rect.height,
        );
        if self.rotate != 0.0 {
            m = m.mul(&Affine::rotate(self.rotate));
        }
        if self.skew_x != 0.0 || self.skew_y != 0.0 {
            m = m.mul(&Affine::skew(self.skew_x, self.skew_y));
        }
        m = m.mul(&Affine::scale(self.scale_x, self.scale_y));
        m.about(px, py)
    }

    /// Mirrored for a right-to-left layout: moves and turns the other way.
    pub fn mirrored(&self) -> Transform {
        Transform {
            x: -self.x,
            shift_x: -self.shift_x,
            rotate: -self.rotate,
            skew_x: -self.skew_x,
            skew_y: -self.skew_y,
            pivot_x: 1.0 - self.pivot_x,
            ..*self
        }
    }
}
