//! The solid's topology as the layout reads it: triangle normals, turns across edges, the fan round each vertex.

use super::*;

/// Faces closer to coplanar than this (on unit normals) are one plane
/// when placing a boundary point.
pub(crate) const PLANAR_DOT: f64 = 1.0 - 1e-12;

/// Below this sine two edges count as collinear.
pub(crate) const COLLINEAR_SIN: f64 = 1e-9;

pub(crate) fn tri_normal(s: &Solid, t: usize) -> DVec3 {
    let [a, b, c] = s.triangles[t].map(|v| s.positions[v as usize]);
    (b - a).cross(c - a).normalize_or_zero()
}

/// The turn between the two faces of an edge, radians (0 flat, π/2 a box
/// edge).
pub(crate) fn turn(s: &Solid, ta: usize, tb: usize) -> f64 {
    trig::acos(tri_normal(s, ta).dot(tri_normal(s, tb)).clamp(-1.0, 1.0))
}

/// Directed edge → triangle, and triangles per vertex.
pub(crate) struct Topology {
    pub dir: BTreeMap<(u32, u32), usize>,
    pub at: Vec<Vec<usize>>,
}

pub(crate) fn topology(s: &Solid) -> Topology {
    let mut dir = BTreeMap::new();
    let mut at = vec![Vec::new(); s.positions.len()];
    for (i, t) in s.triangles.iter().enumerate() {
        for k in 0..3 {
            dir.insert((t[k], t[(k + 1) % 3]), i);
            at[t[k] as usize].push(i);
        }
    }
    Topology { dir, at }
}

/// The fan around a vertex: triangle `i` is `(v, spokes[i], spokes[i+1])`.
pub(crate) struct Fan {
    pub(crate) tris: Vec<usize>,
    pub(crate) spokes: Vec<u32>,
}

pub(crate) fn rotate_to(t: [u32; 3], v: u32) -> [u32; 3] {
    let r = (0..3).find(|&k| t[k] == v).unwrap_or(0);
    [t[r], t[(r + 1) % 3], t[(r + 2) % 3]]
}

pub(crate) fn fan(s: &Solid, topo: &Topology, v: u32) -> Result<Fan, BevelError> {
    let here = &topo.at[v as usize];
    let p = s.positions[v as usize];
    let start = *here.iter().min().ok_or(BevelError::NotClosed)?;
    let mut f = Fan {
        tris: Vec::new(),
        spokes: Vec::new(),
    };
    let mut cur = start;
    loop {
        let t = rotate_to(s.triangles[cur], v);
        f.tris.push(cur);
        f.spokes.push(t[1]);
        cur = *topo.dir.get(&(v, t[2])).ok_or(BevelError::NotClosed)?;
        if cur == start {
            break;
        }
        // Each directed edge has one triangle and its twin another (the
        // solid is closed), so the walk is a cycle and comes back to
        // `start`; this guards that invariant.
        if f.tris.len() > here.len() {
            return Err(BevelError::NotClosed);
        }
    }
    if f.tris.len() != here.len() {
        return Err(BevelError::Unsupported(
            p,
            "two cones of faces meet at one vertex",
        ));
    }
    Ok(f)
}

/// Per selected edge: its triangles and four offsets
/// `[a at lo, b at lo, a at hi, b at hi]`, where face `a` holds the
/// directed edge lo → hi.
pub(crate) struct EdgeInfo {
    pub(crate) off: [f64; 4],
}

impl EdgeInfo {
    /// Offset on the gap after the edge (counter-clockwise) at `v`.
    pub(crate) fn after(&self, key: EdgeKey, v: u32) -> f64 {
        if v == key.0 { self.off[0] } else { self.off[3] }
    }
    /// Offset on the gap before the edge at `v`.
    pub(crate) fn before(&self, key: EdgeKey, v: u32) -> f64 {
        if v == key.0 { self.off[1] } else { self.off[2] }
    }
}
