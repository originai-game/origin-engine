//! Placing a mesh from elsewhere where a game wants it: which way is up
//! and which way it faces, how large it is, and where its origin sits
//! (`docs/research.md` §31).
//!
//! Entry: [`place`] with a [`Place`]. Three steps, each optional, in this
//! order:
//!
//! 1. **Turn.** [`Place::up`] and [`Place::front`] name the input axes
//!    that are the object's up and its front; the turn takes them to the
//!    kernel's up (+Z) and front (−Y, the side `preview` looks at first).
//!    It is one of the 24 axis-aligned turns: a permutation of the
//!    coordinates with signs, so it rounds nothing. [`Place::turn`] then
//!    turns about the new up, in degrees (counter-clockwise seen from
//!    above); a multiple of 90 is exact too, any other angle goes through
//!    `crate::math::trig`.
//! 2. **Size.** [`Size`]: the longest side, the height, the width or the
//!    depth of the turned box to a length, or a factor. Always uniform.
//! 3. **Pivot.** [`Pivot`]: the centre of the box's bottom face (after
//!    the turn, so "bottom" is the new −Z), the box's centre, or a point
//!    of the input, moved to the origin.
//!
//! Every vertex moves; UVs, face names and surfaces stay as they are, and
//! normals follow (a solid stores none: they are made from the positions
//! on export). A skinned solid keeps its skin: bone rest frames turn and
//! move with the mesh and keep unit axes; a scale moves their origins
//! only, as import takes an armature's scale out of its bones
//! (`docs/research.md` §11.5); inverse bind matrices become
//! `S · IBM · A⁻¹` for the placement `A` and the scale `S`, so a bone's
//! rest frame times its inverse bind matrix is what it was.
//!
//! [`axis_measures`] reports the numbers an agent reads to choose up and
//! front (extents, pivot, the area facing each way, where the mass
//! sits). It reports them and never guesses.

use core::fmt;

use crate::math::trig;
use crate::math::{DVec3, Frame};
use crate::mesh::skin::Mat4;
use crate::mesh::{Bone, Skin, Solid, edge_key};
use std::collections::BTreeMap;

use crate::tools::import::Affine;
use crate::tools::json::Json;
use crate::tools::measure::weld_exact;

mod axes;
pub use axes::*;

/// How large to make the mesh. Every size is uniform: the mesh is never
/// stretched. Lengths are in millimetres, of the box after the turn. The
/// zero value keeps the size.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Size {
    /// Keep the size.
    #[default]
    Keep,
    /// The longest side of the box to this length, mm.
    Longest(f64),
    /// The box's extent along Z (up) to this length, mm.
    Height(f64),
    /// Its extent along X (side to side) to this length, mm.
    Width(f64),
    /// Its extent along Y (front to back) to this length, mm.
    Depth(f64),
    /// Scale by this factor.
    Scale(f64),
}

/// Where the origin goes. The zero value keeps it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Pivot {
    /// Keep the origin where the turn and the scale put it.
    #[default]
    Keep,
    /// The centre of the box's bottom face (lowest Z after the turn): a
    /// prop or a character then stands on z = 0.
    Base,
    /// The centre of the box.
    Center,
    /// This point of the input (its coordinates before the turn, mm).
    Point(DVec3),
}

/// How to place a mesh. The zero value changes nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Place {
    /// The input axis that is the object's up; it becomes +Z.
    pub up: Option<Axis>,
    /// The input axis the object faces; it becomes −Y.
    pub front: Option<Axis>,
    /// Then a turn about the up axis, degrees, counter-clockwise seen
    /// from above.
    pub turn: f64,
    /// The size.
    pub size: Size,
    /// The origin.
    pub pivot: Pivot,
}

/// Why a mesh could not be placed.
#[derive(Clone, Debug, PartialEq)]
pub enum PlaceError {
    /// Up and front along one axis.
    Axes(String),
    /// A size, factor, turn or point that is not a finite positive
    /// number, or a box with no extent to scale.
    Size(String),
    /// No vertices.
    Empty,
}

impl fmt::Display for PlaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaceError::Axes(w) | PlaceError::Size(w) => f.write_str(w),
            PlaceError::Empty => f.write_str("nothing to place: the mesh has no vertices"),
        }
    }
}

impl std::error::Error for PlaceError {}

/// What [`place`] did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlaceReport {
    /// The input axes taken as up and front (as given, or as the
    /// smallest turn takes them), if a turn was asked.
    pub up: Option<Axis>,
    /// The front.
    pub front: Option<Axis>,
    /// Where input +X, +Y and +Z went, if anything turned: by the whole
    /// turn when [`Place::turn`] is a multiple of 90, else by the turn
    /// before it.
    pub axes: [Option<Axis>; 3],
    /// The turn about up, degrees.
    pub turn: f64,
    /// The scale factor (1 when kept).
    pub scale: f64,
    /// The translation applied after the turn and the scale, mm.
    pub moved: DVec3,
    /// The whole placement as one matrix (column-major): what a clip of
    /// the mesh's skin is moved by (`crate::mesh::SkinClip::moved`).
    pub matrix: Mat4,
    /// The input's measures.
    pub before: AxisMeasures,
    /// The output's.
    pub after: AxisMeasures,
}

/// The whole placement: `p ↦ scale · yaw(axes(p)) + t`.
#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub(crate) axes: AxisTurn,
    /// cos and sin of a turn about Z that is not a quarter turn.
    pub(crate) yaw: Option<(f64, f64)>,
    pub(crate) scale: f64,
    pub(crate) t: DVec3,
}

impl Motion {
    fn rotate(&self, v: DVec3) -> DVec3 {
        let v = self.axes.apply(v);
        match self.yaw {
            None => v,
            Some((c, s)) => DVec3::new(c * v.x - s * v.y, s * v.x + c * v.y, v.z),
        }
    }

    fn point(&self, p: DVec3) -> DVec3 {
        let mut q = self.rotate(p);
        if self.scale != 1.0 {
            q *= self.scale;
        }
        if self.t != DVec3::ZERO {
            q += self.t;
        }
        q
    }

    /// The 4×4 matrix of the inverse placement, column-major.
    fn inverse_matrix(&self) -> Mat4 {
        // The rotation's columns are its images of the unit axes; its
        // inverse is its transpose. A⁻¹(q) = Rᵀ (q − t) / s.
        let cols = [DVec3::X, DVec3::Y, DVec3::Z].map(|e| self.rotate(e));
        let mut m = [0.0; 16];
        for c in 0..3 {
            for r in 0..3 {
                // Row r of Rᵀ is column r of R.
                m[4 * c + r] = cols[r].to_array()[c] / self.scale;
            }
        }
        for r in 0..3 {
            m[12 + r] = -(0..3)
                .map(|c| m[4 * c + r] * self.t.to_array()[c])
                .sum::<f64>();
        }
        m[15] = 1.0;
        m
    }
}

/// `a · b`, column-major.
pub fn mat_mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut out = [0.0; 16];
    for c in 0..4 {
        for r in 0..4 {
            out[4 * c + r] = (0..4).map(|k| a[4 * k + r] * b[4 * c + k]).sum();
        }
    }
    out
}

#[doc(hidden)]
pub fn moved(s: &Solid, m: &Motion) -> Solid {
    let mut out = s.clone();
    // A morph offset is turned and scaled, not shifted.
    if let Some(morphs) = &mut out.morphs {
        for t in &mut morphs.targets {
            for (d, p) in t.deltas.iter_mut().zip(&s.positions) {
                *d = m.point(*p + *d) - m.point(*p);
            }
        }
    }
    for p in &mut out.positions {
        *p = m.point(*p);
    }
    if let Some(skin) = &s.skin {
        let inverse = m.inverse_matrix();
        let mut scale = [0.0; 16];
        for k in 0..3 {
            scale[5 * k] = m.scale;
        }
        scale[15] = 1.0;
        out.skin = Some(Skin {
            bones: skin
                .bones
                .iter()
                .map(|b| Bone {
                    name: b.name.clone(),
                    parent: b.parent,
                    rest: Frame {
                        origin: m.point(b.rest.origin),
                        x: m.rotate(b.rest.x),
                        y: m.rotate(b.rest.y),
                        z: m.rotate(b.rest.z),
                    },
                    inverse_bind: mat_mul(&mat_mul(&scale, &b.inverse_bind), &inverse),
                })
                .collect(),
            weights: skin.weights.clone(),
        });
    }
    out
}

/// A solid turned by an axis turn, exactly: its positions, and its skin
/// as [`place`] turns one. Import uses it for a skinned part.
pub fn turned(s: &Solid, t: &AxisTurn) -> Solid {
    moved(
        s,
        &Motion {
            axes: *t,
            yaw: None,
            scale: 1.0,
            t: DVec3::ZERO,
        },
    )
}

#[doc(hidden)]
pub fn bounds(parts: &[Solid], f: impl Fn(DVec3) -> DVec3) -> Option<(DVec3, DVec3)> {
    let mut it = parts.iter().flat_map(|s| s.positions.iter()).map(|&p| f(p));
    let first = it.next()?;
    Some(it.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p))))
}

/// Places `parts` (one model, in one space) by `d`: every part by the
/// same turn, scale and translation, the box taken over them all. The
/// parts come back in order.
pub fn place(parts: &[Solid], d: &Place) -> Result<(Vec<Solid>, PlaceReport), PlaceError> {
    let axes = AxisTurn::new(d.up, d.front).map_err(PlaceError::Axes)?;
    if !d.turn.is_finite() {
        return Err(PlaceError::Size(format!(
            "turn {} is not a finite angle",
            d.turn
        )));
    }
    // A whole number of quarter turns stays exact.
    let quarters = d.turn / 90.0;
    let (axes_all, yaw) = if quarters == quarters.round() {
        let n = quarters.round().rem_euclid(4.0) as u32;
        (AxisTurn::quarter(n).then(&axes), None)
    } else {
        let (s, c) = trig::sin_cos(trig::to_radians(d.turn));
        (axes, Some((c, s)))
    };
    let mut m = Motion {
        axes: axes_all,
        yaw,
        scale: 1.0,
        t: DVec3::ZERO,
    };
    let (lo, hi) = bounds(parts, |p| m.rotate(p)).ok_or(PlaceError::Empty)?;
    let size = hi - lo;
    let to = |what: &str, target: f64, now: f64| -> Result<f64, PlaceError> {
        if !(target > 0.0 && target.is_finite()) {
            return Err(PlaceError::Size(format!(
                "{what} {target}: give a length above 0"
            )));
        }
        if now <= 0.0 {
            return Err(PlaceError::Size(format!(
                "the mesh has no {what} to scale (0 mm)"
            )));
        }
        Ok(target / now)
    };
    m.scale = match d.size {
        Size::Keep => 1.0,
        Size::Longest(l) => to("longest side", l, size.max_element())?,
        Size::Height(l) => to("height", l, size.z)?,
        Size::Width(l) => to("width", l, size.x)?,
        Size::Depth(l) => to("depth", l, size.y)?,
        Size::Scale(f) => {
            if !(f > 0.0 && f.is_finite()) {
                return Err(PlaceError::Size(format!(
                    "scale {f}: give a factor above 0"
                )));
            }
            f
        }
    };
    let (lo, hi) = (lo * m.scale, hi * m.scale);
    m.t = match d.pivot {
        Pivot::Keep => DVec3::ZERO,
        Pivot::Base => -DVec3::new((lo.x + hi.x) / 2.0, (lo.y + hi.y) / 2.0, lo.z),
        Pivot::Center => -(lo + hi) / 2.0,
        Pivot::Point(p) => {
            if !p.is_finite() {
                return Err(PlaceError::Size("the pivot point is not finite".into()));
            }
            -(m.rotate(p) * m.scale)
        }
    };
    // Exactly 0 rather than −0 in the report.
    m.t += DVec3::ZERO;
    let out: Vec<Solid> = parts.iter().map(|s| moved(s, &m)).collect();
    let turned = d.up.is_some() || d.front.is_some();
    let report = PlaceReport {
        up: turned.then(|| axes_input(&axes, Axis::PosZ)),
        front: turned.then(|| axes_input(&axes, Axis::NegY)),
        axes: if turned || d.turn != 0.0 {
            [Axis::PosX, Axis::PosY, Axis::PosZ].map(|a| Some(axes_all.image(a)))
        } else {
            [None; 3]
        },
        turn: d.turn,
        scale: m.scale,
        moved: m.t,
        matrix: {
            let cols = [DVec3::X, DVec3::Y, DVec3::Z].map(|e| m.rotate(e) * m.scale);
            [
                cols[0].x, cols[0].y, cols[0].z, 0.0, cols[1].x, cols[1].y, cols[1].z, 0.0,
                cols[2].x, cols[2].y, cols[2].z, 0.0, m.t.x, m.t.y, m.t.z, 1.0,
            ]
        },
        before: axis_measures(parts),
        after: axis_measures(&out),
    };
    Ok((out, report))
}

impl PlaceReport {
    /// The report as one line of JSON, lengths in mm: `up`, `front` and
    /// `axes` (where input `+x`, `+y`, `+z` went; null without a turn),
    /// `turn` (degrees), `scale`, `moved`, and the axis measures `before`
    /// and `after`.
    pub fn to_json(&self) -> String {
        let axis = |a: Option<Axis>| a.map_or(Json::Null, |a| Json::Str(a.name().into()));
        let axes = if self.axes.iter().all(Option::is_some) {
            Json::Obj(vec![
                ("+x", axis(self.axes[0])),
                ("+y", axis(self.axes[1])),
                ("+z", axis(self.axes[2])),
            ])
        } else {
            Json::Null
        };
        let doc = Json::Obj(vec![
            ("units", Json::Str("mm".into())),
            ("up", axis(self.up)),
            ("front", axis(self.front)),
            ("axes", axes),
            ("turn", num(self.turn)),
            ("scale", num(self.scale)),
            ("moved", vec3(self.moved)),
            ("before", self.before.json()),
            ("after", self.after.json()),
        ]);
        let mut out = String::new();
        doc.write(&mut out);
        out
    }
}

#[cfg(test)]
mod tests;
