//! Origin Engine navigation (design §2.7): pathfinding on grids and on a
//! navmesh, flow fields for crowds, steering behaviours, and the `og.nav`
//! system that moves agents, all deterministic (integer costs, ordered
//! ties, the engine's own trigonometry) and covered by snapshots.
//!
//! | Module | What it holds |
//! |---|---|
//! | [`grid`] | [`Grid`] (costs 0 blocked, 1–255), A* and jump point search with no corner cutting, line-of-sight smoothing |
//! | [`flow`] | [`FlowField`]: Dijkstra from target cells, a direction per cell, bilinear sampling |
//! | [`mesh`] | [`Heightfield`] (walkable floors per column), [`NavMesh`] (tiles of convex polygons from monotone regions and their contours), path queries with the funnel, `move_along`, box obstacles that rebuild only their tiles |
//! | `bake` (feature `bake`) | the voxeliser og-cook runs: triangles to a heightfield, with ledge, headroom and radius filters |
//! | [`steer`] | seek, flee, arrive, pursue, evade, wander, separation, avoidance, accelerate |
//! | [`system`] | the `og.NavAgent` and `og.Steering` records, [`Navs`] (an engine's grids, meshes and fields), [`install`] and the `og.nav` system and state block |

#[cfg(feature = "bake")]
pub mod bake;
pub mod flow;
pub mod grid;
pub mod mesh;
pub mod steer;
pub mod system;

pub use flow::FlowField;
pub use grid::{Algorithm, BLOCKED, Grid, GridPath, Scratch};
pub use mesh::{Heightfield, MeshPath, NavMesh, NavParams, Obstacle, Poly, PolyRef, Span, Tile};
pub use system::{
    Nav, NavAgent, NavDesc, NavSystem, Navs, PLANS_PER_TICK, Plane, Raycaster, STATUS_ARRIVED,
    STATUS_IDLE, STATUS_MOVING, STATUS_PARTIAL, STATUS_UNREACHABLE, SYSTEM_NAME, SharedNavs,
    Steering, install, plan,
};
