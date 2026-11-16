//! CSS Motion Path 1 (docs/ui.md §3.2): `offset-path` moves an element
//! along a path in its containing block, `offset-distance` says how far,
//! `offset-rotate` turns it with the path, and `offset-anchor` names the
//! point of its box that rides the path (its `transform-origin` when
//! `auto`). `offset-position` starts a `ray()` (the containing block's
//! centre when `normal`, the element's own anchor when `auto`).
//!
//! The paths are `path("<svg path data>")` and `circle([<n>px])` in px
//! from the containing block's border box (a circle centred in it, from
//! its top clockwise), and `ray(<angle> [<size>])`. Each is one value
//! prop holding its CSS as written, checked when the stylesheet is
//! lowered and parsed once when the element is built; the placement
//! applies the result as one matrix between the individual transform
//! properties and `transform` (CSS Transforms 2 §6.1), against the
//! containing block's laid-out box.

use crate::text_path::{PathShape, TextPath, Walk};
use og_core::glam::Vec2;
use og_ui_model::{Affine, Rect};

/// The props this module reads, by their CSS names.
pub(crate) fn prop_of(css: &str) -> Option<&'static str> {
    Some(match css {
        "offset-path" => "offsetPath",
        "offset-distance" => "offsetDistance",
        "offset-rotate" => "offsetRotate",
        "offset-anchor" => "offsetAnchor",
        "offset-position" => "offsetPosition",
        _ => return None,
    })
}

/// How each prop's value is written (the catalogue's column).
pub(crate) fn form(prop: &str) -> Option<&'static str> {
    Some(match prop {
        "offsetPath" => {
            "none | path(\"<svg path data>\") | circle([<n>px]) | ray(<angle> [closest-side | farthest-side | closest-corner | farthest-corner | sides])"
        }
        "offsetDistance" => "<n>px | <n>%",
        "offsetRotate" => "[auto | reverse] || <angle>",
        "offsetAnchor" => "auto | <position>",
        "offsetPosition" => "normal | auto | <position>",
        _ => return None,
    })
}

/// Checks a declaration's value.
pub(crate) fn check(css: &str, value: &str) -> Result<(), String> {
    let v = value.trim();
    match css {
        "offset-path" => path(v).map(|_| ()),
        "offset-distance" => distance(v).map(|_| ()),
        "offset-rotate" => rotate(v).map(|_| ()),
        "offset-anchor" => anchor(v).map(|_| ()),
        "offset-position" => position(v).map(|_| ()),
        _ => Ok(()),
    }
}

/// An `offset-path`.
#[derive(Clone, Debug, PartialEq)]
pub enum OffsetPath {
    /// `path()` or `circle()`, in px of the containing block's border box.
    Shape(PathShape),
    /// `ray()`: its angle (radians clockwise from up) and size.
    Ray { angle: f32, size: RaySize },
}

/// How long a `ray()` is (CSS Motion Path 1 §2.1.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaySize {
    ClosestSide,
    FarthestSide,
    ClosestCorner,
    FarthestCorner,
    /// To where it leaves the box.
    Sides,
}

/// A `<position>`: per axis, a fraction of the box and px after it.
type Position = [(f32, f32); 2];

/// `offset-position`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Start {
    Normal,
    Auto,
    At(Position),
}

/// An element's motion path, its props parsed ([`crate::Props::offset`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Offset {
    pub path: OffsetPath,
    /// px, or a share of the path's length when `percent`.
    pub distance: f32,
    pub percent: bool,
    /// `auto` (or `reverse`, which adds half a turn) follows the path's
    /// direction; `angle` (radians) is added to it, or is the whole turn.
    pub auto: bool,
    pub angle: f32,
    /// `None`: `auto`, the transform origin.
    pub anchor: Option<Position>,
    pub start: Start,
}

impl Offset {
    /// The motion path from its props' CSS; `None` for `offset-path: none`.
    pub fn from_css(
        path_css: &str,
        distance_css: Option<&str>,
        rotate_css: Option<&str>,
        anchor_css: Option<&str>,
        position_css: Option<&str>,
    ) -> Result<Option<Offset>, String> {
        let Some(path) = path(path_css.trim())? else {
            return Ok(None);
        };
        let (distance, percent) = distance_css.map_or(Ok((0.0, false)), |v| distance(v.trim()))?;
        let (auto, angle) = rotate_css.map_or(Ok((true, 0.0)), |v| rotate(v.trim()))?;
        let anchor = anchor_css.map_or(Ok(None), |v| anchor(v.trim()))?;
        let start = position_css.map_or(Ok(Start::Normal), |v| position(v.trim()))?;
        Ok(Some(Offset {
            path,
            distance,
            percent,
            auto,
            angle,
            anchor,
            start,
        }))
    }

    /// The offset transform of border box `r` in containing block `cb`,
    /// with its transform origin at `origin` (all in the same space): its
    /// anchor moved onto the path's point and turned about it. Identity
    /// where an open path has no point.
    pub fn matrix(&self, r: Rect, cb: Rect, origin: (f32, f32)) -> Affine {
        let at = |p: &Position, b: Rect| {
            Vec2::new(
                b.x + p[0].0 * b.width + p[0].1,
                b.y + p[1].0 * b.height + p[1].1,
            )
        };
        let anchor = self
            .anchor
            .as_ref()
            .map_or(Vec2::new(origin.0, origin.1), |p| at(p, r));
        let (point, dir) = match &self.path {
            OffsetPath::Shape(shape) => {
                let (subs, closed) = TextPath {
                    shape: shape.clone(),
                    offset: 0.0,
                    percent: false,
                    reverse: false,
                }
                .polyline(Vec2::new(cb.width, cb.height));
                let walk = Walk::new(&subs, closed);
                let d = if self.percent {
                    self.distance * walk.length
                } else {
                    self.distance
                };
                // An open path holds its ends (§3.2); a closed one wraps.
                let d = if closed { d } else { d.clamp(0.0, walk.length) };
                let Some((p, dir)) = walk.at(d) else {
                    return Affine::IDENTITY;
                };
                (Vec2::new(cb.x, cb.y) + p, dir)
            }
            OffsetPath::Ray { angle, size } => {
                let start = match self.start {
                    Start::Normal => Vec2::new(cb.x + cb.width / 2.0, cb.y + cb.height / 2.0),
                    Start::Auto => anchor,
                    Start::At(p) => at(&p, cb),
                };
                let (s, c) = og_core::math::sin_cos(*angle);
                let way = Vec2::new(s, -c);
                let d = if self.percent {
                    self.distance * ray_length(start, way, *size, cb)
                } else {
                    self.distance
                };
                (start + way * d, og_core::math::atan2(way.y, way.x))
            }
        };
        let turn = if self.auto {
            dir + self.angle
        } else {
            self.angle
        };
        let shift = point - anchor;
        Affine::translate(shift.x, shift.y).mul(&Affine::rotate(turn).about(anchor.x, anchor.y))
    }
}

/// A ray's length from `start` heading `way` in box `b` (§2.1.1).
fn ray_length(start: Vec2, way: Vec2, size: RaySize, b: Rect) -> f32 {
    let (l, t, r, btm) = (b.x, b.y, b.x + b.width, b.y + b.height);
    let sides = [
        (start.x - l).abs(),
        (r - start.x).abs(),
        (start.y - t).abs(),
        (btm - start.y).abs(),
    ];
    let corners =
        [(l, t), (r, t), (r, btm), (l, btm)].map(|(x, y)| start.distance(Vec2::new(x, y)));
    match size {
        RaySize::ClosestSide => sides.into_iter().fold(f32::INFINITY, f32::min),
        RaySize::FarthestSide => sides.into_iter().fold(0.0, f32::max),
        RaySize::ClosestCorner => corners.into_iter().fold(f32::INFINITY, f32::min),
        RaySize::FarthestCorner => corners.into_iter().fold(0.0, f32::max),
        RaySize::Sides => {
            // Where it leaves the box; none from outside it.
            if !(l..=r).contains(&start.x) || !(t..=btm).contains(&start.y) {
                return 0.0;
            }
            let along = |p: f32, lo: f32, hi: f32, w: f32| {
                if w > 0.0 {
                    (hi - p) / w
                } else if w < 0.0 {
                    (lo - p) / w
                } else {
                    f32::INFINITY
                }
            };
            along(start.x, l, r, way.x).min(along(start.y, t, btm, way.y))
        }
    }
}

fn path(v: &str) -> Result<Option<OffsetPath>, String> {
    if let Some(arg) = v
        .strip_prefix("ray(")
        .and_then(|r| r.strip_suffix(')'))
        .map(str::trim)
    {
        let mut angle = None;
        let mut size = RaySize::ClosestSide;
        for w in arg.split_whitespace() {
            match w {
                "closest-side" => size = RaySize::ClosestSide,
                "farthest-side" => size = RaySize::FarthestSide,
                "closest-corner" => size = RaySize::ClosestCorner,
                "farthest-corner" => size = RaySize::FarthestCorner,
                "sides" => size = RaySize::Sides,
                w if angle.is_none() && crate::transform3d::angle(w).is_some() => {
                    angle = crate::transform3d::angle(w);
                }
                w => {
                    return Err(format!(
                        "offset-path: ray() takes an angle and a size, not '{w}' (contain and at <position> are not supported)"
                    ));
                }
            }
        }
        let angle = angle.ok_or("offset-path: ray() needs an angle (ray(45deg))")?;
        return Ok(Some(OffsetPath::Ray { angle, size }));
    }
    match crate::text_path::parse_shape(v) {
        Ok(shape) => Ok(shape.map(OffsetPath::Shape)),
        Err(_) => Err(format!(
            "offset-path is none, path(\"<svg path data>\"), circle([<n>px]) or ray(<angle> [<size>]), not '{v}'"
        )),
    }
}

fn distance(v: &str) -> Result<(f32, bool), String> {
    crate::text_path::parse_offset(v)
        .map_err(|_| format!("offset-distance is <n>px or <n>%, not '{v}'"))
}

fn rotate(v: &str) -> Result<(bool, f32), String> {
    let (mut auto, mut half, mut angle) = (false, false, None);
    for w in v.split_whitespace() {
        match w {
            "auto" if !auto => auto = true,
            "reverse" if !auto => (auto, half) = (true, true),
            w if angle.is_none() && crate::transform3d::angle(w).is_some() => {
                angle = crate::transform3d::angle(w);
            }
            _ => {
                return Err(format!(
                    "offset-rotate is [auto | reverse] || <angle>, not '{v}'"
                ));
            }
        }
    }
    if !auto && angle.is_none() {
        return Err("offset-rotate: auto, reverse or an angle".into());
    }
    let half = if half { std::f32::consts::PI } else { 0.0 };
    Ok((auto, angle.unwrap_or(0.0) + half))
}

fn anchor(v: &str) -> Result<Option<Position>, String> {
    if v == "auto" {
        return Ok(None);
    }
    crate::css::image_css::object_position(v)
        .map(Some)
        .map_err(|e| e.replace("object-position", "offset-anchor"))
}

fn position(v: &str) -> Result<Start, String> {
    match v {
        "normal" => Ok(Start::Normal),
        "auto" => Ok(Start::Auto),
        _ => crate::css::image_css::object_position(v)
            .map(Start::At)
            .map_err(|e| e.replace("object-position", "offset-position")),
    }
}
