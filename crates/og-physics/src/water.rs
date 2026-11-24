//! Water and the bodies in it. Every kind answers the same questions the
//! same way: where its surface is at a point ([`level`], and [`ripple`] for
//! the waves bodies raised), what crossed it this tick (og.Splash) and what
//! cuts it now (the renderer's disturbance, [`Wakes`]).
//!
//! - og.Water ([`WaterBody`]): a lake, a sea, a river or still water over
//!   a mesh ([`WaterMesh`]), a function of the place and the time. Bodies cross and cut it ([`submersion`]); it lifts
//!   only the hulls of og.Buoyancy.
//! - og.WaterSurface ([`SurfaceState`]): a height field for 3D (the wave
//!   equation on a grid) that the bodies crossing it splash, with a box
//!   buoyancy of its own.
//! - og.WaterLine ([`LineState`]): conservative 1D shallow water for 2D
//!   side-scrollers, with finite-depth, finite-width buoyancy.
//!
//! All are stepped by the og.physics system before the rigid-body step.
//! The heights and which bodies are under are part of the og.physics state
//! block.
//!
//! Determinism: the fields are `+ - * /` on f32 in index order, no
//! host transcendental functions (damping uses og-core's deterministic
//! exponential approximation); a sea is
//! og-core's own deterministic function.

use og_core::glam::Vec3;

mod body;
pub(crate) mod clip;
mod line;
mod mesh;
mod query;
pub(crate) mod submersion;
mod surface;
mod wakes;

pub use body::WaterBody;
pub(crate) use line::LineState;
pub use mesh::WaterMesh;
pub use query::WaterAt;
pub(crate) use query::{at, level, ripple};
pub(crate) use submersion::Under;
pub(crate) use surface::SurfaceState;
pub use wakes::{Wakes, WaterCut};

/// A body as water sees it: its box, velocity and whether it floats.
#[derive(Clone, Debug)]
pub(crate) struct Floater {
    pub entity: u64,
    pub min: Vec3,
    pub max: Vec3,
    pub centre: Vec3,
    pub velocity: Vec3,
    /// Dynamic bodies float and slow down; kinematic ones (characters) only splash.
    pub dynamic: bool,
    /// It floats by its own hull (og.Buoyancy): the water takes its splash
    /// and leaves its lift and its cut to that.
    pub hull: bool,
    /// Planar hull triangles in world axes, relative to `centre`.
    pub triangles: Vec<[Vec3; 3]>,
    pub volume: f32,
}

impl Floater {
    /// Displaced volume (area in 2D), its centre, and waterplane area.
    pub fn immersed(&self, level: f32, depth: f32, min: Vec3, max: Vec3) -> (f32, Vec3, f32) {
        if depth <= 0.0 {
            return (0.0, self.centre, 0.0);
        }
        if !self.triangles.is_empty() {
            let clipped = clip::bounded(&self.triangles, min - self.centre, max - self.centre);
            let (volume, centre, area) = clip::immersed(&clipped, level - self.centre.y, depth);
            return (volume, centre + self.centre, area);
        }
        let low = self.min.y.max(level - depth);
        let high = self.max.y.min(level);
        let x0 = self.min.x.max(min.x);
        let x1 = self.max.x.min(max.x);
        let z0 = self.min.z.max(min.z);
        let z1 = self.max.z.min(max.z);
        let width = ((x1 - x0) / (self.max.x - self.min.x).max(1e-6)).clamp(0.0, 1.0);
        let across = if self.max.z > self.min.z {
            ((z1 - z0) / (self.max.z - self.min.z)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let fraction =
            ((high - low) / (self.max.y - self.min.y).max(1e-6)).clamp(0.0, 1.0) * width * across;
        (
            self.volume * fraction,
            Vec3::new((x0 + x1) * 0.5, (low + high) * 0.5, (z0 + z1) * 0.5),
            (x1 - x0).max(0.0) * (z1 - z0).max(0.0),
        )
    }

    /// Exact vertical intersection length for planar hulls. A half-open
    /// barycentric rule counts shared triangle edges once.
    pub fn column_depth(&self, x: f32, z: f32, level: f32, depth: f32) -> f32 {
        let q = Vec3::new(x, level, z) - self.centre;
        let mut length = 0.0;
        if self.triangles.is_empty() {
            return if x >= self.min.x && x < self.max.x && z >= self.min.z && z < self.max.z {
                (self.max.y.min(level) - self.min.y.max(level - depth)).max(0.0)
            } else {
                0.0
            };
        }
        for &[a, b, c] in &self.triangles {
            let ab = b - a;
            let ac = c - a;
            let det = ab.x * ac.z - ab.z * ac.x;
            if det.abs() < 1e-9 {
                continue;
            }
            let d = q - a;
            let u = (d.x * ac.z - d.z * ac.x) / det;
            let v = (ab.x * d.z - ab.z * d.x) / det;
            let edge = |a: Vec3, b: Vec3| {
                let (a, b) = if det > 0.0 { (a, b) } else { (b, a) };
                let e = (b.x - a.x) * (q.z - a.z) - (b.z - a.z) * (q.x - a.x);
                e > 0.0 || (e == 0.0 && (b.z > a.z || (b.z == a.z && b.x < a.x)))
            };
            if edge(a, b) && edge(b, c) && edge(c, a) {
                let y = a.y + ab.y * u + ac.y * v;
                length +=
                    (y - (q.y - depth)).clamp(0.0, depth) * if det < 0.0 { 1.0 } else { -1.0 };
            }
        }
        length.max(0.0)
    }
}

/// A crossing of the surface, for og.Splash.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Crossing {
    pub entity: u64,
    pub point: Vec3,
    pub speed: f32,
}
