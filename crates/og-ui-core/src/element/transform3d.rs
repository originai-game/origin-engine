//! CSS Transforms 1 and 2 beyond the individual properties (docs/ui.md
//! §2): the `transform` list with every 2D and 3D function, a
//! `transform-origin` with lengths and a z, `perspective` and
//! `perspective-origin`, `transform-style`, `backface-visibility` and
//! `transform-box`. Matrices follow the specs' column-vector convention
//! (x' = M·x), in CSS px with y down, built with the engine's
//! deterministic sine and cosine.

use crate::layout::Rect;

/// A 4×4 matrix by rows: `m[row][column]`.
pub type Mat4 = [[f32; 4]; 4];

pub const IDENTITY: Mat4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// `a × b`: `b` first, then `a`.
pub fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut out = [[0.0; 4]; 4];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, v) in row.iter_mut().enumerate() {
            *v = (0..4).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    out
}

pub fn translate(x: f32, y: f32, z: f32) -> Mat4 {
    let mut m = IDENTITY;
    m[0][3] = x;
    m[1][3] = y;
    m[2][3] = z;
    m
}

/// A 2D affine map as a 4×4.
pub fn from_affine(a: &crate::element::Affine) -> Mat4 {
    [
        [a.a, a.c, 0.0, a.e],
        [a.b, a.d, 0.0, a.f],
        [0.0, 0.0, 1.0, 0.0],
        [a.g, a.h, 0.0, a.i],
    ]
}

/// The map of the plane z = 0: rows and columns x, y and w.
pub fn project(m: &Mat4) -> crate::element::Affine {
    crate::element::Affine::from_rows([
        [m[0][0], m[0][1], m[0][3]],
        [m[1][0], m[1][1], m[1][3]],
        [m[3][0], m[3][1], m[3][3]],
    ])
}

/// CSS Transforms 2 §6.1's flattening: the map z = 0 keeps.
pub fn flatten(m: &Mat4) -> Mat4 {
    let mut f = *m;
    for k in 0..4 {
        f[2][k] = 0.0;
        f[k][2] = 0.0;
    }
    f[2][2] = 1.0;
    f
}

/// A 2D affine map in 3D space: it moves nothing off the plane z = 0.
pub fn is_flat(m: &Mat4) -> bool {
    m[2] == [0.0, 0.0, 1.0, 0.0] && m[0][2] == 0.0 && m[1][2] == 0.0 && m[3] == [0.0, 0.0, 0.0, 1.0]
}

/// Where `(x, y, z)` lands: x, y and z over w, and w.
pub fn apply(m: &Mat4, p: [f32; 3]) -> ([f32; 3], f32) {
    let v = [p[0], p[1], p[2], 1.0];
    let row = |r: usize| (0..4).map(|k| m[r][k] * v[k]).sum::<f32>();
    let w = row(3);
    let k = if w.abs() > 1e-9 { 1.0 / w } else { 0.0 };
    ([row(0) * k, row(1) * k, row(2) * k], w)
}

/// A length and a percentage of a reference size: `px + frac · size`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LenPct {
    pub px: f32,
    pub frac: f32,
}

impl LenPct {
    pub fn at(&self, size: f32) -> f32 {
        self.px + self.frac * size
    }
}

/// One `transform` function.
#[derive(Clone, Debug, PartialEq)]
pub enum TransformFn {
    /// `translate`, `translateX/Y/Z`, `translate3d` (x and y against the
    /// reference box).
    Translate(LenPct, LenPct, f32),
    /// `scale`, `scaleX/Y/Z`, `scale3d`.
    Scale(f32, f32, f32),
    /// `rotate`, `rotateX/Y/Z`, `rotate3d`: an axis and radians.
    Rotate([f32; 3], f32),
    /// `skew`, `skewX`, `skewY`: radians.
    Skew(f32, f32),
    /// `perspective(<length>)`; `None` for `none` (infinite).
    Perspective(Option<f32>),
    /// `matrix` and `matrix3d`, by rows.
    Matrix(Mat4),
}

impl TransformFn {
    /// Its matrix for a reference box `size`.
    pub fn matrix(&self, size: (f32, f32)) -> Mat4 {
        match *self {
            TransformFn::Translate(x, y, z) => translate(x.at(size.0), y.at(size.1), z),
            TransformFn::Scale(x, y, z) => {
                let mut m = IDENTITY;
                m[0][0] = x;
                m[1][1] = y;
                m[2][2] = z;
                m
            }
            TransformFn::Rotate(axis, angle) => rotate3d(axis, angle),
            TransformFn::Skew(x, y) => {
                let tan = |v: f32| {
                    let (s, c) = og_core::math::sin_cos(v);
                    s / c
                };
                let mut m = IDENTITY;
                m[0][1] = tan(x);
                m[1][0] = tan(y);
                m
            }
            TransformFn::Perspective(d) => perspective(d),
            TransformFn::Matrix(m) => m,
        }
    }
}

/// CSS Transforms 2 §13: a turn by `angle` radians about `axis`
/// (normalised; none at all for a zero axis).
pub fn rotate3d(axis: [f32; 3], angle: f32) -> Mat4 {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len <= 0.0 || !len.is_finite() {
        return IDENTITY;
    }
    let [x, y, z] = axis.map(|v| v / len);
    let (s, c) = og_core::math::sin_cos(angle / 2.0);
    let (sc, sq) = (s * c, s * s);
    [
        [
            1.0 - 2.0 * (y * y + z * z) * sq,
            2.0 * (x * y * sq - z * sc),
            2.0 * (x * z * sq + y * sc),
            0.0,
        ],
        [
            2.0 * (x * y * sq + z * sc),
            1.0 - 2.0 * (x * x + z * z) * sq,
            2.0 * (y * z * sq - x * sc),
            0.0,
        ],
        [
            2.0 * (x * z * sq - y * sc),
            2.0 * (y * z * sq + x * sc),
            1.0 - 2.0 * (x * x + y * y) * sq,
            0.0,
        ],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// `perspective(d)`: a viewer `d` px in front of the plane (`None`, or
/// under 1 px clamped to 1 px as the spec says, for `none`: no effect).
pub fn perspective(d: Option<f32>) -> Mat4 {
    let mut m = IDENTITY;
    if let Some(d) = d {
        m[3][2] = -1.0 / d.max(1.0);
    }
    m
}

/// Which box `transform-origin`, `perspective-origin` and percentages in
/// `transform` are against (`transform-box`). A document element's fill
/// box is its content box and its stroke and view boxes its border box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransformBox {
    ContentBox,
    #[default]
    BorderBox,
}

/// An element's 3D transform properties (all but the individual
/// `translate`, `scale` and `rotate`, which are its [`crate::Transform`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Space3d {
    /// `transform`, applied after the individual properties.
    pub functions: Vec<TransformFn>,
    /// `transform-origin` (`None`: the individual properties' pivot): x
    /// and y against the reference box, and z.
    pub origin: Option<(LenPct, LenPct, f32)>,
    /// `perspective` for its children (`None`: `none`).
    pub perspective: Option<f32>,
    /// `perspective-origin` against its reference box.
    pub perspective_origin: (LenPct, LenPct),
    /// `transform-style: preserve-3d`: its children share its 3D space.
    pub preserve_3d: bool,
    /// `backface-visibility: hidden`.
    pub backface_hidden: bool,
    pub transform_box: TransformBox,
}

impl Default for Space3d {
    fn default() -> Self {
        let half = LenPct { px: 0.0, frac: 0.5 };
        Space3d {
            functions: Vec::new(),
            origin: None,
            perspective: None,
            perspective_origin: (half, half),
            preserve_3d: false,
            backface_hidden: false,
            transform_box: TransformBox::BorderBox,
        }
    }
}

impl Space3d {
    /// The `transform` list's matrix about the origin, for an element
    /// whose reference box is `r` and whose individual properties pivot
    /// at the fractions `pivot`.
    pub fn matrix(&self, r: &Rect, pivot: (f32, f32)) -> Mat4 {
        if self.functions.is_empty() {
            return IDENTITY;
        }
        let (ox, oy, oz) = self.origin_at(r, pivot);
        let mut m = translate(ox, oy, oz);
        for f in &self.functions {
            m = mul(&m, &f.matrix((r.width, r.height)));
        }
        mul(&m, &translate(-ox, -oy, -oz))
    }

    /// The origin in layout px.
    pub fn origin_at(&self, r: &Rect, pivot: (f32, f32)) -> (f32, f32, f32) {
        match self.origin {
            Some((x, y, z)) => (r.x + x.at(r.width), r.y + y.at(r.height), z),
            None => (r.x + pivot.0 * r.width, r.y + pivot.1 * r.height, 0.0),
        }
    }

    /// The perspective its children are seen in (identity for none).
    pub fn perspective_matrix(&self, r: &Rect) -> Mat4 {
        let Some(d) = self.perspective else {
            return IDENTITY;
        };
        let (x, y) = (
            r.x + self.perspective_origin.0.at(r.width),
            r.y + self.perspective_origin.1.at(r.height),
        );
        mul(
            &mul(&translate(x, y, 0.0), &perspective(Some(d))),
            &translate(-x, -y, 0.0),
        )
    }

    /// Changes the plane of its children or itself in 3D.
    pub fn is_3d(&self) -> bool {
        !self.functions.is_empty()
            || self.perspective.is_some()
            || self.preserve_3d
            || self.backface_hidden
            || self.origin.is_some_and(|o| o.2 != 0.0)
    }
}

// ---- CSS

/// The properties this module reads.
pub const LONGHANDS: [&str; 7] = [
    "transform",
    "transform-origin",
    "perspective",
    "perspective-origin",
    "transform-style",
    "backface-visibility",
    "transform-box",
];

fn number(word: &str) -> Option<f32> {
    word.parse::<f32>().ok().filter(|v| v.is_finite())
}

/// A length in px, or a percentage when `pct` (0 alone is a length).
fn len_pct(word: &str, pct: bool) -> Option<LenPct> {
    if let Some(v) = word.strip_suffix("px").and_then(number) {
        return Some(LenPct { px: v, frac: 0.0 });
    }
    if pct && let Some(v) = word.strip_suffix('%').and_then(number) {
        return Some(LenPct {
            px: 0.0,
            frac: v / 100.0,
        });
    }
    (number(word) == Some(0.0)).then_some(LenPct::default())
}

fn length(word: &str) -> Option<f32> {
    len_pct(word, false).map(|l| l.px)
}

/// An angle in radians: deg, rad, grad or turn (0 alone too).
pub fn angle(word: &str) -> Option<f32> {
    let w = word.trim();
    for (unit, k) in [
        ("deg", std::f32::consts::PI / 180.0),
        ("grad", std::f32::consts::PI / 200.0),
        ("rad", 1.0),
        ("turn", std::f32::consts::TAU),
    ] {
        if let Some(v) = w.strip_suffix(unit).and_then(number) {
            return Some(v * k);
        }
    }
    (number(w) == Some(0.0)).then_some(0.0)
}

/// A scale factor: a number or a percentage.
fn factor(word: &str) -> Option<f32> {
    word.strip_suffix('%')
        .and_then(number)
        .map(|v| v / 100.0)
        .or_else(|| number(word))
}

/// `transform`'s value: `none` or its functions. Errors name the part.
pub fn parse_transform(value: &str) -> Result<Vec<TransformFn>, String> {
    let value = value.trim();
    if value == "none" {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut rest = value;
    while !rest.is_empty() {
        let open = rest
            .find('(')
            .ok_or_else(|| format!("transform: `{rest}` is not a transform function"))?;
        let name = rest[..open].trim().to_ascii_lowercase();
        let close = rest[open..]
            .find(')')
            .map(|c| open + c)
            .ok_or_else(|| format!("transform: {name}( is not closed"))?;
        let args: Vec<&str> = rest[open + 1..close]
            .split(',')
            .map(str::trim)
            .filter(|a| !a.is_empty())
            .collect();
        out.push(function(&name, &args)?);
        rest = rest[close + 1..].trim_start();
    }
    Ok(out)
}

fn function(name: &str, args: &[&str]) -> Result<TransformFn, String> {
    let bad = || {
        format!(
            "transform: {name}({}) has wrong arguments; see CSS Transforms (lengths in px or %, angles in deg, rad, grad or turn)",
            args.join(", ")
        )
    };
    let n = args.len();
    let lp = |i: usize| len_pct(args[i], true).ok_or_else(bad);
    let len = |i: usize| length(args[i]).ok_or_else(bad);
    let ang = |i: usize| angle(args[i]).ok_or_else(bad);
    let num = |i: usize| number(args[i]).ok_or_else(bad);
    let fac = |i: usize| factor(args[i]).ok_or_else(bad);
    let zero = LenPct::default();
    Ok(match (name, n) {
        ("translate", 1) => TransformFn::Translate(lp(0)?, zero, 0.0),
        ("translate", 2) => TransformFn::Translate(lp(0)?, lp(1)?, 0.0),
        ("translatex", 1) => TransformFn::Translate(lp(0)?, zero, 0.0),
        ("translatey", 1) => TransformFn::Translate(zero, lp(0)?, 0.0),
        ("translatez", 1) => TransformFn::Translate(zero, zero, len(0)?),
        ("translate3d", 3) => TransformFn::Translate(lp(0)?, lp(1)?, len(2)?),
        ("scale", 1) => TransformFn::Scale(fac(0)?, fac(0)?, 1.0),
        ("scale", 2) => TransformFn::Scale(fac(0)?, fac(1)?, 1.0),
        ("scalex", 1) => TransformFn::Scale(fac(0)?, 1.0, 1.0),
        ("scaley", 1) => TransformFn::Scale(1.0, fac(0)?, 1.0),
        ("scalez", 1) => TransformFn::Scale(1.0, 1.0, fac(0)?),
        ("scale3d", 3) => TransformFn::Scale(fac(0)?, fac(1)?, fac(2)?),
        ("rotate" | "rotatez", 1) => TransformFn::Rotate([0.0, 0.0, 1.0], ang(0)?),
        ("rotatex", 1) => TransformFn::Rotate([1.0, 0.0, 0.0], ang(0)?),
        ("rotatey", 1) => TransformFn::Rotate([0.0, 1.0, 0.0], ang(0)?),
        ("rotate3d", 4) => TransformFn::Rotate([num(0)?, num(1)?, num(2)?], ang(3)?),
        ("skew", 1) => TransformFn::Skew(ang(0)?, 0.0),
        ("skew", 2) => TransformFn::Skew(ang(0)?, ang(1)?),
        ("skewx", 1) => TransformFn::Skew(ang(0)?, 0.0),
        ("skewy", 1) => TransformFn::Skew(0.0, ang(0)?),
        ("perspective", 1) if args[0] == "none" => TransformFn::Perspective(None),
        ("perspective", 1) => TransformFn::Perspective(Some(len(0)?.max(0.0))),
        ("matrix", 6) => {
            let v: Vec<f32> = (0..6).map(num).collect::<Result<_, _>>()?;
            TransformFn::Matrix(from_affine(&crate::element::Affine::new(
                v[0], v[1], v[2], v[3], v[4], v[5],
            )))
        }
        ("matrix3d", 16) => {
            let v: Vec<f32> = (0..16).map(num).collect::<Result<_, _>>()?;
            // Column by column.
            let mut m = [[0.0; 4]; 4];
            for (k, x) in v.into_iter().enumerate() {
                m[k % 4][k / 4] = x;
            }
            TransformFn::Matrix(m)
        }
        (
            "translate" | "translatex" | "translatey" | "translatez" | "translate3d" | "scale"
            | "scalex" | "scaley" | "scalez" | "scale3d" | "rotate" | "rotatex" | "rotatey"
            | "rotatez" | "rotate3d" | "skew" | "skewx" | "skewy" | "perspective" | "matrix"
            | "matrix3d",
            _,
        ) => return Err(bad()),
        _ => return Err(format!("transform: {name}() is not a transform function")),
    })
}

/// An origin's x or y: a keyword, a length or a percentage.
fn origin_part(word: &str) -> Option<(LenPct, Option<bool>)> {
    let pct = |f: f32| LenPct { px: 0.0, frac: f };
    Some(match word {
        "left" => (pct(0.0), Some(true)),
        "right" => (pct(1.0), Some(true)),
        "top" => (pct(0.0), Some(false)),
        "bottom" => (pct(1.0), Some(false)),
        "center" => (pct(0.5), None),
        w => (len_pct(w, true)?, None),
    })
}

/// `transform-origin` or `perspective-origin`: one to two positions
/// (keywords in either order), and a z length for `transform-origin`.
pub fn parse_origin(name: &str, value: &str, z: bool) -> Result<(LenPct, LenPct, f32), String> {
    let words: Vec<&str> = value.split_whitespace().collect();
    let bad = || {
        format!(
            "{name}: `{value}`; write one or two of left, center, right, top, bottom, lengths or percentages{}",
            if z { ", then a z length" } else { "" }
        )
    };
    let half = LenPct { px: 0.0, frac: 0.5 };
    let (xy, zw) = match words.len() {
        3 if z => (&words[..2], Some(words[2])),
        1 | 2 => (&words[..], None),
        _ => return Err(bad()),
    };
    let parts: Vec<_> = xy
        .iter()
        .map(|w| origin_part(w))
        .collect::<Option<_>>()
        .ok_or_else(bad)?;
    let (x, y) = match parts.as_slice() {
        [(v, Some(false))] => (half, *v),
        [(v, _)] => (*v, half),
        // `top left`: the vertical keyword first.
        [(a, Some(false)), (b, ax)] if *ax != Some(false) => (*b, *a),
        [(a, ax), (b, bx)] if *ax != Some(false) && *bx != Some(true) => (*a, *b),
        _ => return Err(bad()),
    };
    let z = match zw {
        Some(w) => length(w).ok_or_else(bad)?,
        None => 0.0,
    };
    Ok((x, y, z))
}

/// The element's [`Space3d`] from its winning declarations (name,
/// substituted value); `None` when none applies. Errors are diagnostics.
pub fn resolve(winners: &[(String, String)], diagnostics: &mut Vec<String>) -> Option<Space3d> {
    let mut s = Space3d::default();
    let mut any = false;
    for (name, value) in winners {
        let value = value.trim();
        let result: Result<(), String> = match name.as_str() {
            "transform" => parse_transform(value).map(|f| s.functions = f),
            "transform-origin" => parse_origin(name, value, true).map(|o| s.origin = Some(o)),
            "perspective" => match value {
                "none" => {
                    s.perspective = None;
                    Ok(())
                }
                v => length(v)
                    .filter(|d| *d >= 0.0)
                    .map(|d| s.perspective = Some(d))
                    .ok_or_else(|| format!("perspective: `{v}`; write none or a length in px")),
            },
            "perspective-origin" => {
                parse_origin(name, value, false).map(|(x, y, _)| s.perspective_origin = (x, y))
            }
            "transform-style" => match value {
                "flat" | "preserve-3d" => {
                    s.preserve_3d = value == "preserve-3d";
                    Ok(())
                }
                v => Err(format!("transform-style: `{v}`; write flat or preserve-3d")),
            },
            "backface-visibility" => match value {
                "visible" | "hidden" => {
                    s.backface_hidden = value == "hidden";
                    Ok(())
                }
                v => Err(format!(
                    "backface-visibility: `{v}`; write visible or hidden"
                )),
            },
            "transform-box" => match value {
                "content-box" | "fill-box" => {
                    s.transform_box = TransformBox::ContentBox;
                    Ok(())
                }
                "border-box" | "stroke-box" | "view-box" => {
                    s.transform_box = TransformBox::BorderBox;
                    Ok(())
                }
                v => Err(format!(
                    "transform-box: `{v}`; write content-box, border-box, fill-box, stroke-box or view-box"
                )),
            },
            _ => continue,
        };
        match result {
            Ok(()) => any = true,
            Err(e) => diagnostics.push(e),
        }
    }
    any.then_some(s)
}

/// A node in a 3D space (docs/ui.md §2): where it draws and how its plane
/// lies. A **root** draws itself and its subtree into a layer of its own,
/// in its layout px, laid through `project(m)` onto `into`'s space; any
/// other node draws into `into`'s layer through the affine `project(m)`.
#[derive(Clone, Debug, PartialEq)]
pub struct Plane {
    /// The root whose layer holds it (`layout::NONE`: the screen).
    pub into: usize,
    /// Its layout px into `into`'s px, in 3D.
    pub m: Mat4,
    pub root: bool,
    /// Its children share its 3D space (`transform-style: preserve-3d`).
    pub preserve: bool,
    /// The 3D rendering context it sorts in by depth (a key; 0: none).
    pub context: u64,
    /// A root with `backface-visibility: hidden` seen from behind.
    pub hidden: bool,
    /// Its clip in `into`'s px (a scroll container inside the same layer),
    /// or everything.
    pub clip: Rect,
}

impl Plane {
    /// The map its own commands draw with.
    pub fn draw(&self) -> crate::element::Affine {
        if self.root {
            crate::element::Affine::IDENTITY
        } else {
            project(&self.m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: &Mat4, b: &Mat4) -> bool {
        a.iter()
            .flatten()
            .zip(b.iter().flatten())
            .all(|(x, y)| (x - y).abs() < 1e-5)
    }

    #[test]
    fn functions_build_the_specs_matrices() {
        let size = (200.0, 100.0);
        let m = |v: &str| {
            parse_transform(v)
                .unwrap()
                .iter()
                .fold(IDENTITY, |m, f| mul(&m, &f.matrix(size)))
        };
        // rotateZ(90deg) turns +x onto +y (clockwise on a y-down screen),
        // the same as rotate() and rotate3d(0, 0, 1, …).
        let z = m("rotateZ(90deg)");
        assert!(close(&z, &m("rotate(0.25turn)")));
        assert!(close(&z, &m("rotate3d(0, 0, 2, 100grad)")));
        let (p, _) = apply(&z, [1.0, 0.0, 0.0]);
        assert!((p[0]).abs() < 1e-6 && (p[1] - 1.0).abs() < 1e-6);
        // rotateY(90deg) takes +x to -z (away from the viewer).
        let (p, _) = apply(&m("rotateY(90deg)"), [1.0, 0.0, 0.0]);
        assert!(p[0].abs() < 1e-6 && (p[2] + 1.0).abs() < 1e-6, "{p:?}");
        // Percentages are of the reference box; matrix3d reads by columns.
        assert!(close(
            &m("translate(50%, 10%) translateZ(4px)"),
            &m("matrix3d(1,0,0,0, 0,1,0,0, 0,0,1,0, 100,10,4,1)")
        ));
        // perspective(100px): a point 50 px nearer is seen twice as large.
        let (p, w) = apply(&m("perspective(100px)"), [10.0, 0.0, 50.0]);
        assert!((w - 0.5).abs() < 1e-6 && (p[0] - 20.0).abs() < 1e-5);
        for bad in ["rotate(1)", "spin(1deg)", "translate(1em)", "scale()"] {
            assert!(parse_transform(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn origins_read_keywords_in_either_order() {
        let o = |v: &str| parse_origin("transform-origin", v, true).unwrap();
        let pct = |f: f32| LenPct { px: 0.0, frac: f };
        assert_eq!(o("top left"), (pct(0.0), pct(0.0), 0.0));
        assert_eq!(
            o("right 10px 5px").1,
            LenPct {
                px: 10.0,
                frac: 0.0
            }
        );
        assert_eq!(o("bottom"), (pct(0.5), pct(1.0), 0.0));
        assert_eq!(o("25% 75% -8px").2, -8.0);
        assert!(parse_origin("transform-origin", "left right", true).is_err());
    }
}
