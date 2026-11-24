//! UV unwrap for texturing: charts, a planar parametrization per chart,
//! and packing into the unit square. Written here (xatlas, MIT, could be
//! ported but is about ten thousand lines; `docs/research.md` §9).
//!
//! 1. **Charts.** Triangles are taken largest first as seeds. A chart
//!    grows from its seed across edges shared by two triangles, taking a
//!    neighbour whose normal is within `angle` of the seed's, and whose
//!    projection overlaps no triangle already in the chart (decided
//!    exactly with `orient2d`). So no chart folds over itself.
//! 2. **Parametrization.** Each chart is projected along its seed's
//!    normal: every triangle keeps its winding, and lengths shrink by at
//!    most cos `angle`.
//!    With [`Unwrap::conformal`], charts that bend are first flattened
//!    keeping their angles ([`conformal`]) where that works; the rest
//!    is projected. With [`Unwrap::seams`], a chart is first all the
//!    smooth surface round its seed, cut open along seams until it
//!    flattens ([`seamed`]).
//! 3. **Packing.** Each chart is turned to its smallest bounding rectangle
//!    (rotating calipers over its exact convex hull), laid wide, and
//!    packed tallest first by a bottom-left skyline. One scale (texels per
//!    mm) holds for every chart; the largest that fits is found by
//!    bisection. Charts sit `padding` texels apart at the given
//!    resolution.
//!
//! The result depends only on the solid and the desc. Geometry, names
//! and skin weights are unchanged; the UVs, per triangle corner, go into
//! the solid's [`Solid::uvs`] (replacing any it had), so chart borders are
//! seams, and a writer that splits a vertex at a seam gives each copy the
//! vertex's weights.

use core::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

use crate::math::predicates::orient2d;
use crate::math::{DVec2, DVec3};
use crate::mesh::{Solid, edge_key};
use og_core::math::double;

use crate::tools::intersect::tris_overlap_2d;

mod charts;
mod conformal;
mod pack;
mod seams;
pub(crate) use charts::*;
pub(crate) use conformal::*;
pub(crate) use pack::*;
pub(crate) use seams::*;

/// How to unwrap. The zero value is a 66° chart angle, a 1024-texel
/// atlas and 2 texels between charts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Unwrap {
    /// Largest angle in degrees between a chart's faces and its
    /// projection direction, below 90; 0 is 66 (Blender's smart project).
    pub angle: f64,
    /// Texels across the square atlas the padding is measured in; 0 is
    /// 1024.
    pub resolution: u32,
    /// Texels between charts (half that to the atlas edge); 0 is 2.
    pub padding: f64,
    /// Charts share projection directions chosen over the whole solid
    /// ([`directions`]), as Blender's Smart UV Project chooses them: all
    /// that faces one way is projected one way, wherever it is. Off, a
    /// chart is projected along its own seed's normal.
    pub shared: bool,
    /// Charts that bend are flattened keeping their angles (least
    /// squares conformal maps) where that neither folds nor stretches an
    /// area past twice or under half; what is left is projected as
    /// without it. A curved surface then takes fewer, larger charts.
    pub conformal: bool,
    /// As [`Unwrap::conformal`], and a chart is first tried as all the
    /// surface joined to its seed across edges no sharper than `angle`,
    /// cut open along seams chosen here ([`seamed`]) until it flattens:
    /// a closed or tubular surface becomes one chart with seams in it,
    /// not several charts.
    pub seams: bool,
}

/// Texture coordinates for a solid.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Uvs {
    /// Per triangle, its three corners' coordinates in the unit square,
    /// (0, 0) at the bottom left (glTF flips v on export).
    pub corners: Vec<[DVec2; 3]>,
    /// Per triangle, its chart.
    pub charts: Vec<u32>,
    /// How many charts.
    pub chart_count: usize,
    /// UV units per millimetre on the surface (before the projection's
    /// shrinking): the texel density is this × the resolution.
    pub scale: f64,
}

/// Why a solid could not be unwrapped.
#[derive(Clone, Debug, PartialEq)]
pub enum UnwrapError {
    /// The angle, resolution or padding is out of range.
    BadDesc(String),
    /// The solid has no triangles.
    Empty,
    /// The charts' padding alone does not fit in the atlas: fewer charts
    /// (a larger angle), a finer resolution or less padding.
    TooManyCharts(usize),
}

impl fmt::Display for UnwrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnwrapError::BadDesc(w) => write!(f, "unwrap: {w}"),
            UnwrapError::Empty => f.write_str("unwrap: the solid has no triangles"),
            UnwrapError::TooManyCharts(n) => write!(
                f,
                "unwrap: {n} charts do not fit in the atlas with their padding"
            ),
        }
    }
}

impl std::error::Error for UnwrapError {}

/// Unwraps `s` into charts packed in the unit square, sets its
/// [`Solid::uvs`] to the result and returns the charts. On an error the
/// solid is unchanged.
pub fn unwrap(s: &mut Solid, d: &Unwrap) -> Result<Uvs, UnwrapError> {
    let u = charts(s, d)?;
    s.uvs = Some(u.corners.clone());
    Ok(u)
}

/// The desc with its defaults (angle aside): resolution and padding; or
/// why it cannot unwrap `s`.
pub(crate) fn check(s: &Solid, d: &Unwrap) -> Result<(u32, f64), UnwrapError> {
    let angle = if d.angle == 0.0 { 66.0 } else { d.angle };
    if !(angle > 0.0 && angle < 90.0) {
        return Err(UnwrapError::BadDesc(
            "angle must be between 0 and 90 degrees".into(),
        ));
    }
    let resolution = if d.resolution == 0 {
        1024
    } else {
        d.resolution
    };
    let padding = if d.padding == 0.0 { 2.0 } else { d.padding };
    if !(padding > 0.0 && padding.is_finite()) {
        return Err(UnwrapError::BadDesc("padding must be positive".into()));
    }
    if s.triangles.is_empty() {
        return Err(UnwrapError::Empty);
    }
    Ok((resolution, padding))
}

/// The charts of `s` (largest triangle first as seeds, grown while the
/// normal stays within the angle and the projection does not fold): each
/// chart's triangles, each triangle's chart, and each triangle projected
/// on its chart's plane (millimetres).
pub(crate) fn form(s: &Solid, d: &Unwrap) -> Formed {
    let formed = form_as(s, d);
    if !(d.conformal || d.seams) {
        return formed;
    }
    // Flattened charts are kept where they are fewer than projected
    // ones: on a rough surface they can be more.
    let plain = form_as(
        s,
        &Unwrap {
            conformal: false,
            seams: false,
            ..*d
        },
    );
    if plain.0.len() <= formed.0.len() {
        plain
    } else {
        formed
    }
}

/// What [`form`] gives: each chart's triangles, each triangle's chart,
/// and each triangle on its chart's plane (millimetres).
/// Two measures (areas, distances, facings) closer than this, relative to
/// the larger or to 1, are one: a symmetric surface has many that are
/// equal in exact arithmetic and differ only in their last bits, and the
/// lower index decides between them, so no chart depends on those bits.
pub(crate) const TIE: f64 = 1e-9;

/// Whether `a` is larger than `b` by more than a [`TIE`].
pub(crate) fn exceeds(a: f64, b: f64) -> bool {
    if !(a.is_finite() && b.is_finite()) {
        return a > b;
    }
    a - b > TIE * a.abs().max(b.abs()).max(1.0)
}

/// `x` on a grid of [`TIE`]s of `top`, for sorting: measures that are one
/// sort as equal, and then by index.
pub(crate) fn level(x: f64, top: f64) -> i64 {
    (x / (top.abs().max(f64::MIN_POSITIVE) * TIE)).round() as i64
}

pub(crate) type Formed = (Vec<Vec<u32>>, Vec<u32>, Vec<[DVec2; 3]>);

pub(crate) fn form_as(s: &Solid, d: &Unwrap) -> Formed {
    let angle = if d.angle == 0.0 { 66.0 } else { d.angle };
    let n = s.triangles.len();
    let cos_limit = double::cos(double::to_radians(angle));
    let corner = |i: usize| s.triangles[i].map(|v| s.positions[v as usize]);
    let mut normals = Vec::with_capacity(n);
    let mut areas = Vec::with_capacity(n);
    let mut edge_sum = 0.0;
    for i in 0..n {
        let [a, b, c] = corner(i);
        let cr = (b - a).cross(c - a);
        normals.push(unit(cr));
        areas.push(cr.length());
        edge_sum += (b - a).length() + (c - b).length() + (a - c).length();
    }
    let cell = (edge_sum / (3 * n) as f64 * 2.0).max(1e-9);
    let mut by_edge: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (i, t) in s.triangles.iter().enumerate() {
        for k in 0..3 {
            by_edge
                .entry(edge_key(t[k], t[(k + 1) % 3]))
                .or_default()
                .push(i as u32);
        }
    }
    let mut seeds: Vec<usize> = (0..n).collect();
    let top = areas.iter().copied().fold(0.0, f64::max);
    seeds.sort_by_key(|&a| (-level(areas[a], top), a));
    // With shared directions: each triangle's, by index, and the list.
    let shared = d
        .shared
        .then(|| directions(&normals, &areas, &seeds, angle));
    let mut chart_of = vec![u32::MAX; n];
    let mut flat: Vec<[DVec2; 3]> = vec![[DVec2::ZERO; 3]; n];
    let mut charts: Vec<Vec<u32>> = Vec::new();
    let mut tried = vec![false; n];
    let mut work = 0usize;
    if d.conformal || d.seams {
        // Charts over wider turns of the surface, flattened keeping
        // their angles; the triangles they leave are projected below.
        // A chart grows from its seed taking, of the triangles on its
        // rim, the one that faces most as the seed does, so that any
        // first part of it is the flattest part and in one piece; where
        // the whole does not flatten, its first half is tried, then a
        // quarter, then an eighth.
        let cos_wide = double::cos(double::to_radians((angle * 1.6).min(150.0)));
        for &seed in &seeds {
            if work > SOLVE_WORK * n {
                break;
            }
            if chart_of[seed] != u32::MAX || normals[seed] == DVec3::ZERO {
                continue;
            }
            // All that is joined to the seed across edges no sharper
            // than the angle, cut open until it flattens; while only a
            // part of it does and the seed is not in that part, what
            // is left around the seed again.
            let mut taken = false;
            while d.seams && !tried[seed] && work <= SOLVE_WORK * n {
                let mut region = vec![seed as u32];
                tried[seed] = true;
                let mut head = 0;
                while head < region.len() {
                    let i = region[head];
                    head += 1;
                    let t = s.triangles[i as usize];
                    for k in 0..3 {
                        let users = &by_edge[&edge_key(t[k], t[(k + 1) % 3])];
                        if users.len() != 2 || region.len() == SEAM_REGION {
                            continue;
                        }
                        let j = if users[0] == i { users[1] } else { users[0] } as usize;
                        if chart_of[j] != u32::MAX
                            || tried[j]
                            || normals[j] == DVec3::ZERO
                            || normals[j].dot(normals[i as usize]) < cos_limit
                        {
                            continue;
                        }
                        tried[j] = true;
                        region.push(j as u32);
                    }
                }
                if region.len() < 2 {
                    break;
                }
                let cut = Cut {
                    s,
                    by_edge: &by_edge,
                    normals: &normals,
                    areas: &areas,
                    cos_limit,
                    cell,
                };
                let id = charts.len() as u32;
                match seamed(&cut, &region, &mut work) {
                    Seamed::Whole(laid) => {
                        for (&i, f) in region.iter().zip(laid) {
                            chart_of[i as usize] = id;
                            flat[i as usize] = f;
                        }
                        charts.push(region);
                    }
                    Seamed::Part(which, laid) => {
                        for &i in &region {
                            tried[i as usize] = false;
                        }
                        let some: Vec<u32> = which.iter().map(|&k| region[k]).collect();
                        for (&i, f) in some.iter().zip(laid) {
                            chart_of[i as usize] = id;
                            flat[i as usize] = f;
                        }
                        charts.push(some);
                    }
                    Seamed::Nothing => break,
                }
                if chart_of[seed] != u32::MAX {
                    taken = true;
                    break;
                }
            }
            if taken {
                continue;
            }
            let mut rim: std::collections::BTreeSet<(u64, u32)> = std::collections::BTreeSet::new();
            let mut seen: BTreeMap<u32, ()> = BTreeMap::new();
            rim.insert((0, seed as u32));
            seen.insert(seed as u32, ());
            let mut members: Vec<u32> = Vec::new();
            // Where the chart turns a corner: how many triangles it had
            // before one much more turned than any before it.
            let mut corners: Vec<usize> = Vec::new();
            let mut most = 0.0f64;
            while let Some((key, i)) = rim.pop_first() {
                let turned = f64::from_bits(key);
                if turned > most + 0.25 {
                    corners.push(members.len());
                }
                most = most.max(turned);
                members.push(i);
                let t = s.triangles[i as usize];
                for k in 0..3 {
                    let users = &by_edge[&edge_key(t[k], t[(k + 1) % 3])];
                    if users.len() != 2 {
                        continue;
                    }
                    let j = if users[0] == i { users[1] } else { users[0] };
                    let facing = normals[j as usize].dot(normals[seed]);
                    if chart_of[j as usize] != u32::MAX
                        || facing < cos_wide
                        || seen.insert(j, ()).is_some()
                    {
                        continue;
                    }
                    // Least turned from the seed first (1 − cos ≥ 0: its
                    // bits order as its value).
                    rim.insert(((1.0 - facing).max(0.0).to_bits(), j));
                }
            }
            // The whole; then up to each corner, the last first; then
            // halves.
            let mut sizes = vec![members.len()];
            sizes.extend(corners.iter().rev().take(3));
            sizes.extend([2, 4, 8].map(|part| members.len() / part));
            sizes.sort_unstable_by(|a, b| b.cmp(a));
            sizes.dedup();
            for take in sizes {
                if take < 2 {
                    break;
                }
                let some = &members[..take];
                if let Some(laid) = conformal(s, some, &normals, &areas, cell, &mut work) {
                    let id = charts.len() as u32;
                    for (&i, f) in some.iter().zip(laid) {
                        chart_of[i as usize] = id;
                        flat[i as usize] = f;
                    }
                    charts.push(some.to_vec());
                    break;
                }
            }
        }
    }
    for &seed in &seeds {
        if chart_of[seed] != u32::MAX {
            continue;
        }
        let id = charts.len() as u32;
        let axis = match &shared {
            Some((list, of)) => list.get(of[seed] as usize).copied().unwrap_or(DVec3::ZERO),
            None => normals[seed],
        };
        let (u, v) = if axis == DVec3::ZERO {
            (DVec3::X, DVec3::Y)
        } else {
            basis(axis)
        };
        let project = |i: usize| corner(i).map(|p| DVec2::new(p.dot(u), p.dot(v)));
        let mut grid: BTreeMap<Cell, Vec<u32>> = BTreeMap::new();
        let mut members = vec![seed as u32];
        chart_of[seed] = id;
        flat[seed] = project(seed);
        for c in cells(&flat[seed], cell) {
            grid.entry(c).or_default().push(seed as u32);
        }
        let mut head = 0;
        while head < members.len() && axis != DVec3::ZERO {
            let i = members[head] as usize;
            head += 1;
            let t = s.triangles[i];
            for k in 0..3 {
                let users = &by_edge[&edge_key(t[k], t[(k + 1) % 3])];
                if users.len() != 2 {
                    continue;
                }
                let j = if users[0] as usize == i {
                    users[1]
                } else {
                    users[0]
                } as usize;
                // Within the angle of the seed's normal; or, with shared
                // directions, of the same direction as the seed.
                let fits = match &shared {
                    Some((_, of)) => of[j] == of[seed],
                    None => normals[j].dot(axis) >= cos_limit,
                };
                if chart_of[j] != u32::MAX || !fits {
                    continue;
                }
                let fj = project(j);
                if orient2d(fj[0], fj[1], fj[2]) != Ordering::Greater {
                    continue;
                }
                let mut near: Vec<u32> = cells(&fj, cell)
                    .filter_map(|c| grid.get(&c))
                    .flatten()
                    .copied()
                    .collect();
                near.sort_unstable();
                near.dedup();
                if near.iter().any(|&o| tris_overlap_2d(fj, flat[o as usize])) {
                    continue;
                }
                chart_of[j] = id;
                flat[j] = fj;
                for c in cells(&fj, cell) {
                    grid.entry(c).or_default().push(j as u32);
                }
                members.push(j as u32);
            }
        }
        charts.push(members);
    }
    (charts, chart_of, flat)
}
