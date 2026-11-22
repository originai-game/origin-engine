//! Named generators: solids built from [`crate::profile::Profile`]s, with
//! sand3d's names on caps, walls and edges (`docs/design.md` §4).
//!
//! | generator | faces | edges |
//! |---|---|---|
//! | [`extrude`] | `bottom`, `top`, `side` (`inner` on hole walls too) | `bottom`, `top`, `side` (vertical, at corners) |
//! | [`revolve`] | `side`; `start`, `end` when not a full turn | `rim` (at corners), `start`, `end` |
//! | primitives | as the generator they are made with | |
//!
//! Turned solids are about +y by default; [`Axis::Z`] turns them about +z
//! instead, and [`sphere_desc`], [`torus_desc`] and [`dome_desc`] then rest
//! on z = 0 (sand3d's). More generators: [`crate::generators::loft`], [`crate::generators::body`],
//! [`crate::generators::wire`], [`crate::generators::parts`] (gear solid, knurl).
//!
//! Every generator returns a canonical [`Solid`] (`Solid::canonicalize`).
//!
//! UVs: [`textured`] builds any of these generators with [`Solid::uvs`]
//! on every face, in millimetres on the surface, by one rule (the
//! generators themselves give none, as before UVs existed):
//! - caps (`top`, `bottom`, `start`, `end`, a dome's `bottom`) are planar:
//!   the profile's own coordinates (x, y), or for a turned solid's flat
//!   base its x and depth;
//! - an extrude's walls: u is the arc length along the profile's contour
//!   from its first point, v the height above the bottom;
//! - a turned solid's walls: u is the arc length round the axis at the
//!   vertex's radius (radius × angle), v the arc length along the
//!   profile's contour;
//! - every face's UVs run counter-clockwise as the face does seen from
//!   outside: a face that would read mirrored has its u negated
//!   (decided per surface by `orient2d`).
//!
//! The other generators, by the same rule (`docs/research.md` §21.1),
//! with their seams: [`crate::generators::paths::sweep`] and `thread` (walls u round
//! each section, v along the path, seam along the contour's first point;
//! caps in the section's placed coordinates), [`crate::generators::loft::loft`]
//! (u round each ring, v up; seam along each ring's first sample; caps
//! x, y), [`crate::generators::wire::wire`] (u round, v along; seam along the ring's
//! first point, and across ring 0 on a closed wire; caps in the ring's
//! circle), [`crate::generators::body::body`] (u round each section from +x, v
//! along y; seam along +x), [`crate::generators::parts::knurl`] (the crest cylinder:
//! u = radius × angle from +x, v = z), [`crate::generators::parts::gear_solid`] (caps
//! x, y; walls as an extrude's, v = z; seam at the outline's first
//! point), [`wedge`] (an extrude's).
//!
//! Each contour's closing segment ends at its full length, not at 0, so a
//! closed wall has one seam where its contour starts (and a full turn one
//! where its angle does).

use crate::math::predicates::orient2d;
use crate::math::{DVec2, DVec3};
use crate::mesh::{CornerUvs, Solid};
use crate::profile::curve::segments;
use crate::profile::{Circle, Contour, Profile, ProfileError, Rect, circle, rect};
use og_core::math::double;

mod extrude;
mod primitives;
mod revolve;
pub use extrude::*;
pub use primitives::*;
pub use revolve::*;

/// Why a generator could not build its solid.
#[derive(Clone, Debug, PartialEq)]
pub enum GenError {
    /// The profile was invalid.
    Profile(ProfileError),
    /// A size that must be positive was not (the field named).
    NotPositive(&'static str),
    /// A value was NaN or infinite (the field named).
    NotFinite(&'static str),
    /// A revolved profile crosses the axis (x < 0 somewhere).
    CrossesAxis,
    /// The inputs cannot make a solid; the message says where and why.
    Invalid(String),
}

impl From<ProfileError> for GenError {
    fn from(e: ProfileError) -> Self {
        GenError::Profile(e)
    }
}

impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenError::Profile(e) => write!(f, "profile: {e}"),
            GenError::NotPositive(n) => write!(f, "{n} must be positive"),
            GenError::NotFinite(n) => write!(f, "{n} must be finite"),
            GenError::CrossesAxis => f.write_str("a revolved profile must lie at x >= 0"),
            GenError::Invalid(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for GenError {}

pub(crate) fn positive(v: f64, name: &'static str) -> Result<f64, GenError> {
    if !v.is_finite() {
        Err(GenError::NotFinite(name))
    } else if v <= 0.0 {
        Err(GenError::NotPositive(name))
    } else {
        Ok(v)
    }
}

pub(crate) fn finite(v: f64, name: &'static str) -> Result<f64, GenError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(GenError::NotFinite(name))
    }
}

/// Most segments a turn a `segments` field takes.
pub const MAX_SEGMENTS: u32 = 1024;

/// Checks a `segments` field: 0 (the chord rule) or 3 to [`MAX_SEGMENTS`].
pub(crate) fn segments_field(n: u32, name: &'static str) -> Result<u32, GenError> {
    if n == 0 || (3..=MAX_SEGMENTS).contains(&n) {
        Ok(n)
    } else {
        Err(GenError::Invalid(format!(
            "{name} is {n}: 0 for the chord rule, or 3 to {MAX_SEGMENTS} a turn"
        )))
    }
}

/// Segments for an arc of `radius` sweeping `angle`: the chord rule with
/// `per_turn` 0, else `per_turn`'s share of a turn (at least 1).
pub(crate) fn count(per_turn: u32, radius: f64, angle: f64) -> u32 {
    if per_turn == 0 {
        segments(radius, angle)
    } else {
        let fraction = (angle.abs() / double::TAU).min(1.0);
        ((per_turn as f64 * fraction).ceil() as u32).max(1)
    }
}

/// Faces out (UVs turned with their triangles, then made to read
/// unmirrored), then canonical.
pub(crate) fn finish(mut s: Solid) -> Result<Solid, GenError> {
    if s.measure().volume < 0.0 {
        for i in 0..s.triangles.len() {
            s.reverse_triangle(i);
        }
    }
    unmirror_uvs(&mut s);
    s.canonicalize()
        .map_err(|_| GenError::NotFinite("positions"))?;
    Ok(s)
}

/// Pushes a triangle, with its UVs when the solid is being given some.
pub(crate) fn push(
    s: &mut Solid,
    uvs: bool,
    t: [u32; 3],
    surface: u32,
    tags: &[&str],
    uv: CornerUvs,
) {
    if uvs {
        s.push_triangle_uv(t, surface, tags, uv);
    } else {
        s.push_triangle(t, surface, tags);
    }
}

/// Per surface, negates u where most of its UV triangles run clockwise
/// (a face that would read mirrored), counted exactly by `orient2d`.
pub(crate) fn unmirror_uvs(s: &mut Solid) {
    let Some(uvs) = &mut s.uvs else { return };
    let mut balance: std::collections::BTreeMap<u32, i64> = std::collections::BTreeMap::new();
    for (i, u) in uvs.iter().enumerate() {
        let b = balance.entry(s.surfaces[i]).or_default();
        match orient2d(u[0], u[1], u[2]) {
            core::cmp::Ordering::Greater => *b += 1,
            core::cmp::Ordering::Less => *b -= 1,
            core::cmp::Ordering::Equal => {}
        }
    }
    for (i, u) in uvs.iter_mut().enumerate() {
        if balance[&s.surfaces[i]] < 0 {
            for q in u {
                q.x = -q.x;
            }
        }
    }
}

/// Each point's arc length along a closed contour from its first point,
/// and the whole length: `(at, total)`.
pub(crate) fn arc_lengths(points: &[DVec2]) -> (Vec<f64>, f64) {
    let mut at = Vec::with_capacity(points.len());
    let mut sum = 0.0;
    for i in 0..points.len() {
        at.push(sum);
        sum += (points[(i + 1) % points.len()] - points[i]).length();
    }
    (at, sum)
}

/// The arc length at the end of the segment from point `i0` to the next:
/// `at[i0 + 1]`, or the whole length for the closing segment.
pub(crate) fn arc_end(at: &[f64], total: f64, i0: usize) -> f64 {
    at.get(i0 + 1).copied().unwrap_or(total)
}

/// UVs of a wall stitched between rings of points (`rings[i][j]`: ring
/// `i` along the wall, point `j` round it, every ring closed), in
/// millimetres on the surface: u is the arc length round ring `i` from
/// its point 0 (the closing segment ends at the ring's whole length, so
/// the seam runs along point 0), v the arc length along the line of
/// points `j` from ring 0. A ring of no length (a body's apex) takes the
/// u of the nearest ring that has one. With `wraps`, the wall closes
/// along too: ring `len` is ring 0 again at the whole length (a second
/// seam, across ring 0).
pub(crate) struct RingUvs {
    pub(crate) u: Vec<Vec<f64>>,
    pub(crate) v: Vec<Vec<f64>>,
}

impl RingUvs {
    pub(crate) fn new(pos: &[DVec3], rings: &[Vec<u32>], wraps: bool) -> RingUvs {
        let p = |v: u32| pos[v as usize];
        let mut u: Vec<Vec<f64>> = rings
            .iter()
            .map(|r| {
                let n = r.len();
                let mut at = Vec::with_capacity(n + 1);
                let mut sum = 0.0;
                for j in 0..n {
                    at.push(sum);
                    sum += (p(r[(j + 1) % n]) - p(r[j])).length();
                }
                at.push(sum);
                at
            })
            .collect();
        // Rings of no length borrow the nearest ring's u (later first).
        let len = |r: &Vec<f64>| r.last().copied().unwrap_or(0.0);
        for i in 0..u.len() {
            if len(&u[i]) == 0.0 {
                let near = (1..u.len())
                    .flat_map(|d| [i + d, i.wrapping_sub(d)])
                    .find(|&k| k < u.len() && len(&u[k]) > 0.0);
                if let Some(k) = near {
                    u[i] = u[k].clone();
                }
            }
        }
        let n = rings.first().map_or(0, Vec::len);
        let count = rings.len() + usize::from(wraps);
        let mut v = vec![vec![0.0; n]; count];
        for i in 1..count {
            for j in 0..n {
                let (a, b) = (rings[i - 1][j], rings[i % rings.len()][j]);
                v[i][j] = v[i - 1][j] + (p(b) - p(a)).length();
            }
        }
        RingUvs { u, v }
    }

    /// The corner at ring `i` (up to `len`, the wrap, when it wraps) and
    /// point `j` (up to `n`, the closing end).
    pub(crate) fn at(&self, i: usize, j: usize) -> DVec2 {
        let n = self.v[0].len();
        DVec2::new(self.u[i % self.u.len()][j], self.v[i][j % n])
    }
}

/// The runs of a contour between its corners: `(first, last)` point
/// indices, `last` possibly wrapping past the end. A contour with no
/// corner is one run that closes on itself.
pub(crate) fn runs(c: &Contour) -> Vec<(usize, usize)> {
    let n = c.points.len();
    let corners: Vec<usize> = (0..n).filter(|&i| c.sharp[i]).collect();
    if corners.is_empty() {
        return vec![(0, n)];
    }
    (0..corners.len())
        .map(|k| {
            let a = corners[k];
            let b = corners[(k + 1) % corners.len()];
            (a, if b > a { b } else { b + n })
        })
        .collect()
}

// ── primitives ──────────────────────────────────────────────────────────────

/// A spherical cap `a` in base radius and `h` high, about +y on y = 0.
pub(crate) fn cap(a: f64, h: f64, per_turn: u32, uvs: bool) -> Result<Solid, GenError> {
    let r = (a * a + h * h) / (2.0 * h);
    let from = double::atan2(r - h, a);
    let sweep = double::PI / 2.0 - from;
    let n = count(per_turn, r, sweep);
    let mut pts = vec![DVec2::ZERO];
    pts.extend((0..=n).map(|k| {
        let (s, c) = double::sin_cos(from + sweep * (k as f64 / n as f64));
        DVec2::new(r * c, h - r + r * s)
    }));
    pts[1] = DVec2::new(a, 0.0);
    pts[n as usize + 1] = DVec2::new(0.0, h);
    let mut sharp = vec![false; pts.len()];
    sharp[0] = true;
    sharp[1] = true;
    sharp[n as usize + 1] = true;
    let mut s = revolve_with(
        &Profile::from_contour(Contour { points: pts, sharp }),
        &Revolve {
            segments: per_turn,
            ..Default::default()
        },
        uvs,
    )?;
    flat_bottom(&mut s);
    s.canonicalize()
        .map_err(|_| GenError::NotFinite("positions"))?;
    Ok(s)
}

/// A generator [`textured`] can build.
#[derive(Clone, Copy, Debug)]
pub enum Generator<'a> {
    /// [`extrude`].
    Extrude(&'a Profile, Extrude),
    /// [`revolve`].
    Revolve(&'a Profile, Revolve),
    /// [`cuboid`].
    Box(BoxDesc),
    /// [`cylinder`].
    Cylinder(CylinderDesc),
    /// [`cone_desc`].
    Cone(ConeDesc),
    /// [`tube_desc`].
    Tube(TubeDesc),
    /// [`sphere_desc`].
    Sphere(SphereDesc),
    /// [`torus_desc`].
    Torus(TorusDesc),
    /// [`dome_desc`].
    Dome(DomeDesc),
    /// [`crate::generators::loft::loft`].
    Loft(&'a crate::generators::loft::Loft),
    /// [`crate::generators::loft::loft_along`].
    LoftAlong(&'a crate::generators::loft::LoftAlong),
    /// [`crate::generators::paths::sweep`].
    Sweep(&'a Profile, &'a crate::generators::paths::Sweep),
    /// [`crate::generators::paths::thread`].
    Thread(crate::generators::paths::Thread),
    /// [`crate::generators::wire::wire`].
    Wire(&'a crate::generators::wire::Wire),
    /// [`crate::generators::body::body`].
    Body(&'a crate::generators::body::Body),
    /// [`crate::generators::parts::knurl`].
    Knurl(crate::generators::parts::Knurl),
    /// [`crate::generators::parts::gear_solid`].
    Gear(crate::generators::parts::GearDesc),
    /// [`wedge`]: `(width, height, depth)`.
    Wedge(f64, f64, f64),
}

/// The generator's solid with UVs on every face, by the rule in the
/// module docs: the same positions, triangles and names as the
/// generator alone gives, plus [`Solid::uvs`].
pub fn textured(g: &Generator) -> Result<Solid, GenError> {
    match g {
        Generator::Extrude(p, d) => extrude_with(p, d, true),
        Generator::Revolve(p, d) => revolve_with(p, d, true),
        Generator::Box(d) => cuboid_with(d, true),
        Generator::Cylinder(d) => cylinder_with(d, true),
        Generator::Cone(d) => cone_with(d, true),
        Generator::Tube(d) => tube_with(d, true),
        Generator::Sphere(d) => sphere_with(d, true),
        Generator::Torus(d) => torus_with(d, true),
        Generator::Dome(d) => dome_with(d, true),
        Generator::Loft(d) => crate::generators::loft::loft_with(d, true),
        Generator::LoftAlong(d) => crate::generators::loft::loft_along_with(d, true),
        Generator::Sweep(p, d) => crate::generators::paths::sweep_with(p, d, true),
        Generator::Thread(d) => crate::generators::paths::thread_with(d, true),
        Generator::Wire(d) => crate::generators::wire::wire_with(d, true),
        Generator::Body(d) => crate::generators::body::body_with(d, true),
        Generator::Knurl(d) => crate::generators::parts::knurl_with(d, true),
        Generator::Gear(d) => crate::generators::parts::gear_with(d, true),
        Generator::Wedge(w, h, d) => wedge_with(*w, *h, *d, true),
    }
}

#[cfg(test)]
mod tests;
