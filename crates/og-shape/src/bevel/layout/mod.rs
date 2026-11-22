//! The bevel core: offsets, boundary points, profiles, strips and the
//! rebuilt faces (`docs/research.md` §7).
//!
//! Vocabulary (after Blender's published design, §2.2):
//! - Around a vertex the triangles form a fan; a **spoke** is an edge out
//!   of the vertex. Bevelled spokes split the fan into **gaps**; real
//!   unbevelled spokes split a gap into **wedges** (one per face).
//! - Each gap gets one **boundary point** (a terminal vertex, with one
//!   bevelled edge, gets two). Every triangle corner at the vertex is
//!   replaced by its gap's point, or, on the end face of a terminal
//!   edge, by a piece of the edge's profile.
//! - Each bevelled edge end gets a **chain**: its profile, from the gap
//!   after the edge to the gap before it (counter-clockwise seen from
//!   outside). An edge's **strip** joins the chains at its two ends.
//! - Where three or more bevelled edges meet, the chains bound a hole
//!   filled by a corner **patch** (`patch.rs`).

use std::collections::{BTreeMap, BTreeSet};

use crate::math::{DVec2, DVec3, predicates};

use crate::mesh::{EdgeKey, Solid, edge_key};
use og_core::math::double;

use crate::bevel::patch::{self, Corner};
use crate::bevel::profile;
use crate::bevel::{BevelError, Miter};

mod check;
mod cones;
mod general;
mod rows;
mod runs;
mod tips;
mod wide;

mod build;
mod faces;
mod finish;
mod topology;
pub(crate) use build::*;
pub(crate) use faces::*;
pub(crate) use finish::*;
pub(crate) use topology::*;

/// What the new faces are called.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Fillet,
    Chamfer,
}

impl Kind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Kind::Fillet => "fillet",
            Kind::Chamfer => "chamfer",
        }
    }
}

/// How far the offset lines sit from each edge, on its two faces.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Offsets {
    /// The same on both faces.
    Equal(f64),
    /// The first on the face that carries one of the edge's names (the
    /// cap), the second on the other.
    Two(f64, f64),
    /// The first on the cap face; the chamfer face meets the cap at this
    /// angle (radians).
    Angle(f64, f64),
}

/// A resolved bevel request.
#[derive(Clone)]
pub(crate) struct Spec {
    pub edges: Vec<EdgeKey>,
    pub offsets: Offsets,
    /// 0: from the radius (`crate::profile::curve::segments`).
    pub segments: u32,
    /// The superellipse exponent (2 a circle, 1 straight).
    pub exponent: f64,
    pub kind: Kind,
    pub miters: Miters,
    /// Even out each bevel side's width where its ends slide along
    /// unbevelled edges.
    pub loop_slide: bool,
    /// The checked input's triangles (`check.rs`); `None` until checked.
    pub checked: Option<std::sync::Arc<Vec<[DVec3; 3]>>>,
}

/// Miter kinds and their spread (0: the gap's offset).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Miters {
    pub inner: Miter,
    pub outer: Miter,
    pub spread: f64,
}

pub(crate) fn bevel(s: &Solid, spec: &Spec) -> Result<Solid, BevelError> {
    if let Some((solid, edges)) = input(s, spec)? {
        return bevel_input(
            &solid,
            &Spec {
                edges,
                checked: Some(std::sync::Arc::new(check::triangles(&solid))),
                ..spec.clone()
            },
        );
    }
    bevel_input(
        s,
        &Spec {
            checked: Some(std::sync::Arc::new(check::triangles(s))),
            ..spec.clone()
        },
    )
}

/// The input checked (`docs/research.md` §7.7): closed, every edge between
/// two faces, and not crossing itself. A solid that crosses itself only
/// where a boolean left points closer than 1e-9 of the offset (a sliver
/// lying across its neighbour) is welded there first, as a wide bevel's
/// pass 0 does; the welded solid comes back. Otherwise the result could
/// not be told apart from a bevel that crossed itself, so it is refused.
pub(crate) fn input(s: &Solid, spec: &Spec) -> Result<Option<(Solid, Vec<EdgeKey>)>, BevelError> {
    if !s.measure().closed {
        let mut count: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for t in &s.triangles {
            for k in 0..3 {
                *count.entry((t[k], t[(k + 1) % 3])).or_default() += 1;
            }
        }
        for (&(a, c), &n) in &count {
            let back = count.get(&(c, a)).copied().unwrap_or(0);
            if n > 1 || back > 1 {
                return Err(BevelError::NotManifold(
                    s.positions[a as usize],
                    s.positions[c as usize],
                ));
            }
        }
        return Err(BevelError::NotClosed);
    }
    let crossings = crate::boolean::selfcheck::self_crossings(&s.positions, &s.triangles);
    let Some(&(i, _)) = crossings.first() else {
        return Ok(None);
    };
    let at = s.positions[s.triangles[i as usize][0] as usize];
    let topo = topology(s);
    let (info, _) = edge_offsets(s, &topo, spec)?;
    let largest = info.values().flat_map(|e| e.off).fold(0.0f64, f64::max);
    let (w, edges) =
        wide::weld(s, &spec.edges, 1e-9 * largest).ok_or(BevelError::SelfCrossing(at))?;
    if let Some(&(i, _)) =
        crate::boolean::selfcheck::self_crossings(&w.positions, &w.triangles).first()
    {
        return Err(BevelError::SelfCrossing(
            w.positions[w.triangles[i as usize][0] as usize],
        ));
    }
    Ok(Some((w, edges)))
}

pub(crate) fn bevel_input(s: &Solid, spec: &Spec) -> Result<Solid, BevelError> {
    // A vertex where two cones of faces touch is split into one per cone.
    if let Some((solid, edges)) = cones::split(s, &spec.edges) {
        return bevel_runs(
            &solid,
            &Spec {
                edges,
                ..spec.clone()
            },
        );
    }
    bevel_runs(s, spec)
}

/// Why one layout attempt failed: an error, or a result that would tilt
/// old faces off their planes (`docs/research.md` §7.7), with the vertex
/// and the bands (bevelled edge, its face `a`) whose faces it tilts.
pub(crate) enum Fail {
    Error(BevelError),
    Tilted(DVec3, BTreeSet<(EdgeKey, bool)>),
}

impl From<BevelError> for Fail {
    fn from(e: BevelError) -> Self {
        Fail::Error(e)
    }
}

/// How far a rebuilt triangle's corner may leave the triangle's plane, as
/// a fraction of the largest offset: rounding only, for the faces as
/// given (the classic layout keeps every corner it replaces in its
/// triangle's plane, to 1.5e-11 in the scan) ...
pub(crate) const TILT_EXACT: f64 = 1e-9;
/// ... and a tenth where a wide bevel's pass 1 or 2 already moved a
/// vertex, by up to that (`wide.rs`).
pub(crate) const TILT_PASS1: f64 = 0.1;

pub(crate) const TILTED: &str = "a bevel along a face whose triangles at this vertex are not in one plane (a twisted or converging face), where the bevel would tilt them";

pub(crate) fn bevel_runs(s: &Solid, spec: &Spec) -> Result<Solid, BevelError> {
    let (at, tilted) = match bevel_once(s, spec, None, TILT_EXACT) {
        Ok(out) => return Ok(out),
        Err(Fail::Error(BevelError::TooLarge(at))) => (at, false),
        Err(Fail::Tilted(at, _)) => (at, true),
        Err(Fail::Error(e)) => return Err(e),
    };
    {
        {
            // Straight runs may be cut into pieces shorter than the bevel.
            let dissolved = runs::dissolve(s, &spec.edges);
            if let Some((solid, edges)) = &dissolved {
                let spec = Spec {
                    edges: edges.clone(),
                    ..spec.clone()
                };
                if let Ok(out) = bevel_once(solid, &spec, None, TILT_EXACT) {
                    return Ok(out);
                }
            }
            // An offset line that shrinks a run of edges to a point.
            if let Ok(out) = bevel_poles(s, spec) {
                return Ok(out);
            }
            // A bevel wider than the faces' own segments.
            let first = wide::bevel(s, spec);
            if first.is_ok() {
                return first;
            }
            if let Some((solid, edges)) = &dissolved {
                let spec = Spec {
                    edges: edges.clone(),
                    ..spec.clone()
                };
                if let Ok(out) = wide::bevel(solid, &spec) {
                    return Ok(out);
                }
            }
            if tilted {
                // Cut every corner on its folds, in each triangle's plane.
                match layout(s, spec, None, TILT_EXACT, true, false) {
                    Ok(out) => Ok(out),
                    Err(Fail::Error(e @ BevelError::LonePiece(..))) => Err(e),
                    Err(_) => match rows::bevel(s, spec) {
                        Ok(out) => Ok(out),
                        Err(Some(at)) => {
                            Err(tips::lone_piece(s, spec).unwrap_or(BevelError::TooLarge(at)))
                        }
                        Err(None) => Err(tips::lone_piece(s, spec)
                            .unwrap_or(BevelError::Unsupported(at, TILTED))),
                    },
                }
            } else {
                match rows::bevel(s, spec) {
                    Ok(out) => Ok(out),
                    Err(_) => Err(tips::lone_piece(s, spec).unwrap_or(BevelError::TooLarge(at))),
                }
            }
        }
    }
}

/// Per selected edge, its offsets; and the segment count.
pub(crate) fn edge_offsets(
    s: &Solid,
    topo: &Topology,
    spec: &Spec,
) -> Result<(BTreeMap<EdgeKey, EdgeInfo>, u32), BevelError> {
    let mut edges = BTreeMap::new();
    let mut max_turn = 0.0f64;
    let mut max_off = 0.0f64;
    for &key in &spec.edges {
        let (lo, hi) = key;
        let (Some(&ta), Some(&tb)) = (topo.dir.get(&(lo, hi)), topo.dir.get(&(hi, lo))) else {
            return Err(BevelError::NotClosed);
        };
        let (pa, pb) = (s.positions[lo as usize], s.positions[hi as usize]);
        if s.surfaces[ta] == s.surfaces[tb] {
            return Err(BevelError::NotAnEdge(pa, pb));
        }
        let tags = s.edge_tags.get(&key).cloned().unwrap_or_default();
        let carries = |t: usize| s.face_tags[t].iter().any(|x| tags.contains(x));
        let a_first = carries(ta) || !carries(tb);
        let th = turn(s, ta, tb);
        max_turn = max_turn.max(th);
        let (d1, d2) = match spec.offsets {
            Offsets::Equal(d) => (d, d),
            Offsets::Two(d, e) => (d, e),
            Offsets::Angle(d, angle) => {
                // The faces meet at π − turn; the chamfer leaves the cap at
                // `angle`: law of sines in the cross-section.
                let inner = double::PI - th;
                (d, d * double::sin(angle) / double::sin(inner + angle))
            }
        };
        if !(d1.is_finite() && d2.is_finite() && d1 > 0.0 && d2 > 0.0) {
            return Err(BevelError::TooLarge(pa));
        }
        max_off = max_off.max(d1).max(d2);
        let off = if a_first {
            [d1, d2, d1, d2]
        } else {
            [d2, d1, d2, d1]
        };
        edges.insert(key, EdgeInfo { off });
    }
    let n = if spec.segments > 0 {
        spec.segments
    } else {
        match spec.kind {
            Kind::Chamfer => 1,
            Kind::Fillet => {
                let n = crate::profile::curve::segments(max_off, max_turn);
                n + n % 2
            }
        }
    };
    Ok((edges, n))
}

/// One layout attempt; `tilt` as for [`TILT_EXACT`].
pub(crate) fn bevel_once(
    s: &Solid,
    spec: &Spec,
    wide: Option<&wide::Wide>,
    tilt: f64,
) -> Result<Solid, Fail> {
    layout(s, spec, wide, tilt, false, false)
}

/// The classic layout with each run of edges that the offset line
/// swallows on a face shrunk to a point (a round corner of the fillet's
/// own radius): the strips along it end in that point, as fans, and the
/// faces beside keep their planes.
pub(crate) fn bevel_poles(s: &Solid, spec: &Spec) -> Result<Solid, Fail> {
    layout(s, spec, None, TILT_EXACT, false, true)
}

/// One layout attempt; with `folds`, every fold of a face counts as a
/// real edge, so a corner on a face whose triangles at it are not one
/// plane is cut in each triangle's own plane (`docs/research.md` §7.8).
pub(crate) fn layout(
    s: &Solid,
    spec: &Spec,
    wide: Option<&wide::Wide>,
    tilt: f64,
    folds: bool,
    weld: bool,
) -> Result<Solid, Fail> {
    if !s.measure().closed {
        return Err(BevelError::NotClosed.into());
    }
    let topo = topology(s);
    let (edges, n) = edge_offsets(s, &topo, spec)?;
    let mut b = Build {
        s,
        topo,
        edges,
        pos: s.positions.clone(),
        repl: BTreeMap::new(),
        chains: BTreeMap::new(),
        new_tris: Vec::new(),
        units: 0,
        joins: Vec::new(),
        smooth: Vec::new(),
        patches: Vec::new(),
        samples: BTreeMap::new(),
        n,
        exponent: spec.exponent,
        kind: spec.kind,
        miters: spec.miters,
        wide,
        tilt,
        folds,
        poles: BTreeMap::new(),
        checked: spec.checked.clone(),
    };
    let verts: BTreeSet<u32> = b.edges.keys().flat_map(|&(x, y)| [x, y]).collect();
    if spec.loop_slide {
        b.slide(&verts)?;
    }
    for &v in &verts {
        b.vertex(v)?;
    }
    if weld {
        b.swallowed_runs();
    }
    // Strips.
    let mut strip_of: BTreeMap<EdgeKey, usize> = BTreeMap::new();
    let mut inserts: BTreeMap<(usize, u32), Vec<u32>> = BTreeMap::new();
    let keys: Vec<EdgeKey> = b.edges.keys().copied().collect();
    for key in keys {
        let (u, w) = key;
        let cu = b.chains[&(u, w)].clone();
        let mut cw = b.chains[&(w, u)].clone();
        cw.reverse();
        if cu.len() != cw.len() {
            return Err(BevelError::Unsupported(s.positions[u as usize], "uneven profiles").into());
        }
        let stations = wide.and_then(|w| w.stations.get(&key));
        let point = |c: &[u32]| c.iter().all(|&x| x == c[0]);
        let mut profiles = vec![cu.clone()];
        if stations.is_none() && point(&cu) && point(&cw) {
            // Both ends taper to a tip (`general.rs`): the width grows
            // inside the edge (`tips.rs`, `docs/research.md` §7.8).
            let (mid, ends) = b.taper(key)?;
            profiles.extend(mid);
            for (at, pts) in ends {
                inserts.insert(at, pts);
            }
        } else if let Some(st) = stations {
            let (mid, insert) = b.stations(key, st, &cu, &cw)?;
            profiles.extend(mid);
            inserts.insert(insert.0, insert.1);
        }
        profiles.push(cw);
        let unit = b.unit();
        strip_of.insert(key, unit);
        for pair in profiles.windows(2) {
            let (cu, cw) = (&pair[0], &pair[1]);
            for k in 0..cu.len() - 1 {
                // A tip's profile is one point: skip the folded triangles.
                for t in [[cw[k], cu[k], cu[k + 1]], [cw[k], cu[k + 1], cw[k + 1]]] {
                    if t[0] != t[1] && t[1] != t[2] && t[2] != t[0] {
                        b.new_tris.push((t, unit));
                    }
                }
            }
        }
    }
    for &(e0, e1) in &b.smooth {
        b.joins.push((strip_of[&e0], strip_of[&e1]));
    }
    if b.kind == Kind::Fillet {
        for (unit, ks) in &b.patches {
            for k in ks {
                b.joins.push((*unit, strip_of[k]));
            }
        }
    }
    finish(b, &inserts)
}
