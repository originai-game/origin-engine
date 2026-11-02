//! Origin Engine navigation (design §2.7): pathfinding on grids and on a
//! navmesh, flow fields for crowds, steering behaviours, and the `og.nav`
//! system that moves agents, all deterministic (integer costs, ordered
//! ties, the engine's own trigonometry) and covered by snapshots.
//!
//! | Module | What it holds |
//! |---|---|
//! | [`grid`] | [`Grid`] (costs 0 blocked, 1–255), A* and jump point search with no corner cutting, line-of-sight smoothing |
//! | [`flow`] | [`FlowField`]: Dijkstra from target cells, a direction per cell, bilinear sampling |
//! | [`mesh`] | [`Heightfield`] (walkable floors per column), [`NavMesh`] (tiles of convex polygons from monotone regions and their contours), path queries with the funnel and [`Filter`]s, `move_along`, obstacles of any [`Shape`] and areas that rebuild only their tiles, off-mesh [`Link`]s |
//! | [`shape`] | [`Shape`]: boxes, turned boxes, cylinders and convex prisms, their cell test and byte form |
//! | `bake` (feature `bake`) | the voxeliser og-cook runs: triangles to a heightfield, with ledge, headroom and radius filters |
//! | [`steer`] | seek, flee, arrive, pursue, evade, wander, separation, avoidance, accelerate |
//! | [`system`] | the `og.NavAgent` and `og.Steering` records, [`Navs`] (an engine's grids, meshes and fields), [`install`] and the `og.nav` system and state block |

#[cfg(feature = "bake")]
pub mod bake;
pub mod flow;
pub mod grid;
pub mod mesh;
pub mod shape;
pub mod steer;
pub mod system;

pub use flow::FlowField;
pub use grid::{Algorithm, BLOCKED, Grid, GridPath, Scratch};
pub use mesh::{
    Filter, Heightfield, Link, LinkKind, MeshPath, NavMesh, NavParams, Obstacle, PathLink, Poly,
    PolyRef, Shape, Span, Tile,
};
pub use system::{
    Nav, NavAgent, NavDesc, NavFilter, NavLinkReached, NavSystem, Navs, OFF_PATH, PLANS_PER_TICK,
    Plan, Plane, Raycaster, STATUS_ARRIVED, STATUS_IDLE, STATUS_MOVING, STATUS_ON_LINK,
    STATUS_PARTIAL, STATUS_UNREACHABLE, SYSTEM_NAME, SharedNavs, Steering, Traverse,
    enable_filters, enable_links, install, plan,
};
