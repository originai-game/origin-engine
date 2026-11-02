//! A spatial hash over entities' positions, for neighbourhood queries on
//! entities without colliders: everything within a radius, the k nearest.
//!
//! The index is a snapshot of positions taken when it is built
//! ([`crate::World::spatial`], [`crate::SystemContext::spatial`]): it owns
//! its data and does not follow later writes. Answers are deterministic by
//! construction: positions are read as stored (f32), distances computed in
//! f64, and every answer is ordered by distance, then by entity id, so ties
//! never depend on storage order.

use crate::entity::Entity;
use crate::registry::{FieldRef, SchemaId};
use og_core::reflect::FieldType;
use og_core::{Error, ErrorKind, Result};

/// What a spatial index holds. `field` is a `vec3` (or `vec2`, at z = 0)
/// field of a component: every entity that has that component (and every
/// component of `with`, and none of `without`) is placed at its value.
#[derive(Clone, Debug, PartialEq)]
pub struct SpatialDesc {
    pub field: FieldRef,
    /// The cell size; 0 picks one from the points' spread (about two points
    /// per cell on average). Answers do not depend on it, only speed does.
    pub cell: f32,
    pub with: Vec<SchemaId>,
    pub without: Vec<SchemaId>,
}

/// One answer: an entity and its distance from the query's centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Neighbour {
    pub entity: Entity,
    pub distance: f64,
}

/// A built index. See the module docs.
#[derive(Clone, Debug)]
pub struct SpatialHash {
    /// Points in entity order.
    points: Vec<(Entity, [f64; 3])>,
    cell: f64,
    /// (cell key, point index), sorted.
    cells: Vec<([i64; 3], u32)>,
    /// The bounds of the points (min, max).
    bounds: ([f64; 3], [f64; 3]),
}

/// Cells past this many along an axis per query fall back to a scan.
const MAX_SPAN: i64 = 64;

impl SpatialHash {
    /// An index of `points` (entity, position). `cell` 0 picks one.
    pub fn new(mut points: Vec<(Entity, [f32; 3])>, cell: f32) -> Result<SpatialHash> {
        if !cell.is_finite() || cell < 0.0 {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("a spatial cell size is 0 (automatic) or positive, got {cell}"),
            ));
        }
        og_core::sort::sort_by_key(&mut points, |(e, _)| *e);
        let points: Vec<(Entity, [f64; 3])> = points
            .into_iter()
            .filter(|(_, p)| p.iter().all(|v| v.is_finite()))
            .map(|(e, p)| (e, p.map(f64::from)))
            .collect();
        let mut lo = [0.0f64; 3];
        let mut hi = [0.0f64; 3];
        if let Some((_, first)) = points.first() {
            lo = *first;
            hi = *first;
        }
        for (_, p) in &points {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        let cell = if cell > 0.0 {
            f64::from(cell)
        } else {
            // About two points per cell over the occupied axes.
            let extents: Vec<f64> = (0..3).map(|k| hi[k] - lo[k]).filter(|e| *e > 0.0).collect();
            if extents.is_empty() || points.len() < 2 {
                1.0
            } else {
                let volume: f64 = extents.iter().product();
                let per = (volume * 2.0 / points.len() as f64).powf(1.0 / extents.len() as f64);
                per.max(1e-3)
            }
        };
        let mut cells: Vec<([i64; 3], u32)> = points
            .iter()
            .enumerate()
            .map(|(i, (_, p))| (key(p, cell), i as u32))
            .collect();
        og_core::sort::sort(&mut cells);
        Ok(SpatialHash {
            points,
            cell,
            cells,
            bounds: (lo, hi),
        })
    }

    /// How many entities it holds.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// The cell size it was built with.
    pub fn cell(&self) -> f64 {
        self.cell
    }

    /// Every entity at most `radius` from `center`, nearest first (equal
    /// distances in entity order). A negative or non-finite radius finds
    /// nothing.
    pub fn within(&self, center: [f32; 3], radius: f32) -> Vec<Neighbour> {
        if radius.is_nan() || radius < 0.0 || !center.iter().all(|v| v.is_finite()) {
            return Vec::new();
        }
        self.ball(center.map(f64::from), f64::from(radius))
    }

    fn ball(&self, c: [f64; 3], r: f64) -> Vec<Neighbour> {
        let mut out = Vec::new();
        let lo = key(&[c[0] - r, c[1] - r, c[2] - r], self.cell);
        let hi = key(&[c[0] + r, c[1] + r, c[2] + r], self.cell);
        let span = if r.is_finite() {
            (0..3).map(|k| hi[k] - lo[k] + 1).max().unwrap_or(1)
        } else {
            i64::MAX
        };
        let mut take = |i: u32| {
            let (e, p) = self.points[i as usize];
            let d2 = dist2(&p, &c);
            if d2 <= r * r {
                out.push((d2, e));
            }
        };
        if span > MAX_SPAN || (span as u128).pow(3) > self.cells.len() as u128 {
            // A box of more cells than points: a scan is cheaper.
            for i in 0..self.points.len() {
                take(i as u32);
            }
        } else {
            for x in lo[0]..=hi[0] {
                for y in lo[1]..=hi[1] {
                    let from = self.cells.partition_point(|(k, _)| *k < [x, y, lo[2]]);
                    let to = self.cells.partition_point(|(k, _)| *k <= [x, y, hi[2]]);
                    for &(_, i) in &self.cells[from..to] {
                        take(i);
                    }
                }
            }
        }
        finish(out)
    }

    /// The `k` entities nearest `center`, nearest first (equal distances in
    /// entity order); fewer when it holds fewer.
    pub fn nearest(&self, center: [f32; 3], k: usize) -> Vec<Neighbour> {
        if k == 0 || self.points.is_empty() || !center.iter().all(|v| v.is_finite()) {
            return Vec::new();
        }
        let c = center.map(f64::from);
        // The farthest any point can be: the distance to the farthest corner.
        let (lo, hi) = self.bounds;
        let far = (0..3)
            .map(|i| (c[i] - lo[i]).abs().max((c[i] - hi[i]).abs()))
            .map(|d| d * d)
            .sum::<f64>()
            .sqrt();
        // Grow a ball until it holds k points: then the k nearest are in
        // it. Past the farthest point, every point.
        let mut r = self.cell;
        loop {
            let mut ball = if r >= far {
                self.ball(c, f64::INFINITY)
            } else {
                self.ball(c, r)
            };
            if ball.len() >= k || r >= far {
                ball.truncate(k);
                return ball;
            }
            r *= 2.0;
        }
    }
}

fn key(p: &[f64; 3], cell: f64) -> [i64; 3] {
    p.map(|v| (v / cell).floor().clamp(-1e15, 1e15) as i64)
}

fn dist2(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    (0..3).map(|k| (a[k] - b[k]) * (a[k] - b[k])).sum()
}

fn finish(mut out: Vec<(f64, Entity)>) -> Vec<Neighbour> {
    out.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    out.into_iter()
        .map(|(d2, entity)| Neighbour {
            entity,
            distance: d2.sqrt(),
        })
        .collect()
}

/// A position field's value as three coordinates.
pub(crate) fn position(ty: FieldType, bytes: &[u8]) -> Option<[f32; 3]> {
    let f = |i: usize| f32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().expect("4 bytes"));
    match ty {
        FieldType::Vec3 => Some([f(0), f(1), f(2)]),
        FieldType::Vec2 => Some([f(0), f(1), 0.0]),
        _ => None,
    }
}
