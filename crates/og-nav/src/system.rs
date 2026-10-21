//! The ECS side: the `og.NavAgent` and `og.Steering` records, the registry
//! of grids, navmeshes and flow fields an engine holds, and the `og.nav`
//! system that moves agents every tick.
//!
//! Per tick, in query order: an agent that is `seeking` re-plans when its
//! target moved more than `repathDistance`, every `repathTicks`, or when
//! its grid or navmesh changed; it follows the path's corners (arriving at
//! the last), or a flow field's direction; the steering behaviours of its
//! `og.Steering` are added by weight; the velocity turns towards the result
//! by at most `acceleration · dt` and is capped at `speed`. An agent with an
//! `og.Velocity` (a physics body) gets its velocity written and physics
//! moves it; any other agent's `og.Transform` moves (along the navmesh
//! surface on a navmesh). The time is the scaled tick, so a hit stop holds
//! agents with everything else.
//!
//! The paths and the registry's edits (grid cells, obstacles, flow targets)
//! are the `og.nav` state block: snapshots restore them, the world hash
//! covers them.

use crate::flow::FlowField;
use crate::grid::{Algorithm, Grid, Scratch};
use crate::mesh::{NONE as NO_POLY, NavMesh, Obstacle, PolyRef};
use crate::steer;
use og_core::glam::{Quat, Vec2, Vec3};
use og_core::{Error, ErrorKind, Reflect, Result};
use og_ecs::{Entity, QueryDesc, StateBlock, SystemContext, SystemDesc, World};
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

/// The system's and the state block's name.
pub const SYSTEM_NAME: &str = "og.nav";

/// `NavAgent.status`: not seeking.
pub const STATUS_IDLE: u32 = 0;
/// Moving along a path (or a flow field).
pub const STATUS_MOVING: u32 = 1;
/// Within `stopDistance` of the target.
pub const STATUS_ARRIVED: u32 = 2;
/// No path reaches the target (or the agent is off its grid or navmesh).
pub const STATUS_UNREACHABLE: u32 = 3;
/// The target is unreachable; moving to the reachable point nearest it.
pub const STATUS_PARTIAL: u32 = 4;

/// An agent that moves itself: towards `target` while `seeking`, by a path
/// over `nav` (a grid, navmesh or flow field handle; 0 goes straight).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.NavAgent")]
pub struct NavAgent {
    /// The grid, navmesh or flow field it plans on (og.nav.grid, og.nav.load, og.nav.flowField); 0 moves straight at the target.
    pub nav: u64,
    /// Where it goes while seeking.
    pub target: Vec3,
    /// Moves towards the target while true.
    pub seeking: bool,
    /// Top speed.
    #[reflect(min = 0, max = 100, default = 3.5, unit = "mps", tunable)]
    pub speed: f32,
    /// How fast it changes velocity.
    #[reflect(min = 0, max = 1000, default = 20, unit = "mps2", tunable)]
    pub acceleration: f32,
    /// Its radius: paths keep this clear of grid walls; separation uses it.
    #[reflect(min = 0, max = 10, default = 0.4, unit = "m", tunable)]
    pub radius: f32,
    /// It has arrived within this distance of the target.
    #[reflect(
        name = "stopDistance",
        min = 0,
        max = 10,
        default = 0.1,
        unit = "m",
        tunable
    )]
    pub stop_distance: f32,
    /// It slows down inside this distance of the target.
    #[reflect(
        name = "slowRadius",
        min = 0,
        max = 20,
        default = 1,
        unit = "m",
        tunable
    )]
    pub slow_radius: f32,
    /// Re-plans this often while seeking (0: only when the target moves).
    #[reflect(
        name = "repathTicks",
        min = 0,
        max = 3600,
        default = 30,
        unit = "ticks"
    )]
    pub repath_ticks: u32,
    /// Re-plans when the target moved farther than this from the last plan.
    #[reflect(name = "repathDistance", min = 0, max = 100, default = 0.5, unit = "m")]
    pub repath_distance: f32,
    /// Pulls grid paths straight where nothing is in the way.
    #[reflect(default = 1)]
    pub smooth: bool,
    /// Turns the entity to face where it moves (3D: about +Y, its +Z forward; 2D: about +Z, its +X forward).
    #[reflect(name = "faceMovement")]
    pub face_movement: bool,
    /// How fast it turns when facing its movement.
    #[reflect(name = "turnSpeed", min = 0, max = 100, default = 10, unit = "radps")]
    pub turn_speed: f32,
    /// Moves the entity (its og.Velocity, or its og.Transform) while true; false keeps planning but leaves the motion to the game (a knock-back, a stun).
    #[reflect(default = 1)]
    pub drive: bool,
    /// NAV_IDLE, NAV_MOVING, NAV_ARRIVED, NAV_UNREACHABLE or NAV_PARTIAL (written by og.nav).
    pub status: u32,
    /// Its velocity (written by og.nav).
    pub velocity: Vec3,
    /// The path corner it heads for (written by og.nav).
    pub waypoint: Vec3,
    /// The path length left (written by og.nav).
    pub remaining: f32,
    /// Paths planned so far (written by og.nav).
    pub plans: u32,
}

impl Default for NavAgent {
    fn default() -> Self {
        NavAgent {
            nav: 0,
            target: Vec3::ZERO,
            seeking: false,
            speed: 3.5,
            acceleration: 20.0,
            radius: 0.4,
            stop_distance: 0.1,
            slow_radius: 1.0,
            repath_ticks: 30,
            repath_distance: 0.5,
            smooth: true,
            face_movement: false,
            turn_speed: 10.0,
            drive: true,
            status: STATUS_IDLE,
            velocity: Vec3::ZERO,
            waypoint: Vec3::ZERO,
            remaining: 0.0,
            plans: 0,
        }
    }
}

/// Steering behaviours an agent adds to its path following, each by weight
/// (0 is off).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Steering")]
pub struct Steering {
    /// Weight of following the path to the target (seek, arrive at the end).
    #[reflect(min = 0, max = 10, default = 1, tunable)]
    pub seek: f32,
    /// Weight of fleeing fleeFrom.
    #[reflect(min = 0, max = 10, tunable)]
    pub flee: f32,
    /// What it flees.
    #[reflect(name = "fleeFrom")]
    pub flee_from: Vec3,
    /// It flees only inside this distance (0: always).
    #[reflect(
        name = "fleeRadius",
        min = 0,
        max = 100,
        default = 6,
        unit = "m",
        tunable
    )]
    pub flee_radius: f32,
    /// Weight of wandering.
    #[reflect(min = 0, max = 10, tunable)]
    pub wander: f32,
    /// The wander circle's radius.
    #[reflect(
        name = "wanderRadius",
        min = 0,
        max = 20,
        default = 1,
        unit = "m",
        tunable
    )]
    pub wander_radius: f32,
    /// How far ahead the wander circle is.
    #[reflect(
        name = "wanderDistance",
        min = 0,
        max = 20,
        default = 2,
        unit = "m",
        tunable
    )]
    pub wander_distance: f32,
    /// How fast the wander point moves round the circle.
    #[reflect(
        name = "wanderJitter",
        min = 0,
        max = 50,
        default = 3,
        unit = "radps",
        tunable
    )]
    pub wander_jitter: f32,
    /// An entity to pursue (seek where it will be); 0: none.
    pub pursue: u64,
    /// Weight of pursuing.
    #[reflect(name = "pursueWeight", min = 0, max = 10, default = 1, tunable)]
    pub pursue_weight: f32,
    /// An entity to evade (flee where it will be); 0: none.
    pub evade: u64,
    /// Weight of evading.
    #[reflect(name = "evadeWeight", min = 0, max = 10, default = 1, tunable)]
    pub evade_weight: f32,
    /// Weight of keeping apart from other agents.
    #[reflect(min = 0, max = 10, tunable)]
    pub separation: f32,
    /// Other agents closer than this push it away.
    #[reflect(
        name = "separationRadius",
        min = 0,
        max = 20,
        default = 1,
        unit = "m",
        tunable
    )]
    pub separation_radius: f32,
    /// Weight of avoiding colliders ahead (three physics rays).
    #[reflect(min = 0, max = 10, tunable)]
    pub avoidance: f32,
    /// How far ahead the avoidance rays reach.
    #[reflect(
        name = "avoidDistance",
        min = 0,
        max = 50,
        default = 2,
        unit = "m",
        tunable
    )]
    pub avoid_distance: f32,
    /// Collision layers the rays hit (0: every layer).
    #[reflect(name = "avoidMask")]
    pub avoid_mask: u32,
    /// The wander angle (written by og.nav).
    #[reflect(name = "wanderAngle")]
    pub wander_angle: f32,
}

impl Default for Steering {
    fn default() -> Self {
        Steering {
            seek: 1.0,
            flee: 0.0,
            flee_from: Vec3::ZERO,
            flee_radius: 6.0,
            wander: 0.0,
            wander_radius: 1.0,
            wander_distance: 2.0,
            wander_jitter: 3.0,
            pursue: 0,
            pursue_weight: 1.0,
            evade: 0,
            evade_weight: 1.0,
            separation: 0.0,
            separation_radius: 1.0,
            avoidance: 0.0,
            avoid_distance: 2.0,
            avoid_mask: 0,
            wander_angle: 0.0,
        }
    }
}

/// The engine's `og.Transform`, mirrored (same name and layout).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Transform")]
pub struct Transform {
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vec3::ZERO,
            quaternion: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

/// The engine's `og.Velocity`, mirrored (same name and layout).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Velocity")]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

/// Which world axes agents move on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plane {
    /// x and y (the 2D profile); z is kept.
    Xy,
    /// x and z (3D); y is kept, or follows the navmesh.
    Xz,
}

impl Plane {
    pub fn to_plane(self, v: Vec3) -> Vec2 {
        match self {
            Plane::Xy => Vec2::new(v.x, v.y),
            Plane::Xz => Vec2::new(v.x, v.z),
        }
    }
    /// A plane vector back in the world, the off-plane axis from `height`.
    pub fn to_world(self, p: Vec2, height: f32) -> Vec3 {
        match self {
            Plane::Xy => Vec3::new(p.x, p.y, height),
            Plane::Xz => Vec3::new(p.x, height, p.y),
        }
    }
    pub fn height(self, v: Vec3) -> f32 {
        match self {
            Plane::Xy => v.z,
            Plane::Xz => v.y,
        }
    }
}

/// One thing agents plan on.
#[derive(Clone, Debug)]
pub enum Nav {
    Grid(Grid),
    Mesh(Box<NavMesh>),
    /// A flow field over grid `grid` towards `targets` (cells).
    Flow {
        grid: u64,
        targets: Vec<(u32, u32)>,
        field: FlowField,
    },
}

impl Nav {
    pub fn kind(&self) -> &'static str {
        match self {
            Nav::Grid(_) => "grid",
            Nav::Mesh(_) => "navmesh",
            Nav::Flow { .. } => "flow",
        }
    }
}

/// The grids, navmeshes and flow fields of one engine, by handle.
#[derive(Debug, Default)]
pub struct Navs {
    pub entries: BTreeMap<u64, Nav>,
    next: u64,
}

impl Navs {
    /// Stores `nav` under a fresh handle (`index + 2^32`).
    pub fn insert(&mut self, nav: Nav) -> u64 {
        self.next += 1;
        let h = self.next | (1u64 << 32);
        self.entries.insert(h, nav);
        h
    }

    pub fn get(&self, h: u64) -> Option<&Nav> {
        self.entries.get(&h)
    }

    pub fn get_mut(&mut self, h: u64) -> Option<&mut Nav> {
        self.entries.get_mut(&h)
    }

    /// Rebuilds every flow field whose grid changed since it was built.
    pub fn refresh_flows(&mut self) {
        let stale: Vec<(u64, u64)> = self
            .entries
            .iter()
            .filter_map(|(h, n)| match n {
                Nav::Flow { grid, field, .. } => match self.entries.get(grid) {
                    Some(Nav::Grid(g)) if g.version() != field.grid_version => Some((*h, *grid)),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        for (h, g) in stale {
            let Some(Nav::Grid(grid)) = self.entries.get(&g).cloned() else {
                continue;
            };
            if let Some(Nav::Flow { targets, field, .. }) = self.entries.get_mut(&h) {
                *field = FlowField::build(&grid, targets);
            }
        }
    }
}

/// An engine's navigation, shared by the runtime's functions and the system.
pub type SharedNavs = Arc<Mutex<Navs>>;

/// What an avoidance ray reports: (distance, surface normal).
pub type Raycaster = Arc<dyn Fn(Vec3, Vec3, f32, u32, Entity) -> Option<(f32, Vec3)> + Send + Sync>;

/// How og.nav joins a world.
#[derive(Clone)]
pub struct NavDesc {
    pub plane: Plane,
    /// The physics ray the avoidance behaviour casts; without one it is off.
    pub raycast: Option<Raycaster>,
    /// Path plans per tick across all agents (0: no limit). Agents over the
    /// budget plan on the next ticks, in query order, so a crowd that gets
    /// a new target at once spreads its planning instead of one long tick.
    pub plans_per_tick: u32,
}

/// An agent farther than this (or four radii) from its path's current leg plans again.
pub const OFF_PATH: f32 = 1.5;

/// The default [`NavDesc::plans_per_tick`].
pub const PLANS_PER_TICK: u32 = 32;

/// A path an agent follows.
#[derive(Clone, Debug, PartialEq)]
pub struct PathState {
    pub corners: Vec<Vec3>,
    pub next: u32,
    /// What the plan was made for.
    pub target: Vec3,
    pub nav: u64,
    pub version: u64,
    /// Ticks since the plan.
    pub age: u32,
    /// The navmesh polygon the agent stands on; [`crate::mesh::NONE`] until it is
    /// located (an agent that has never planned stands on none).
    pub poly: u32,
    pub status: u32,
}

impl Default for PathState {
    fn default() -> Self {
        PathState {
            corners: Vec::new(),
            next: 0,
            target: Vec3::ZERO,
            nav: 0,
            version: 0,
            age: 0,
            poly: NO_POLY,
            status: 0,
        }
    }
}

/// The polygon of `mesh` an agent at `pos` stands on: `poly` while it still
/// holds the agent (under it on x and z, its surface within the agent's
/// height), else the nearest within the agent's height and radius, else
/// [`crate::mesh::NONE`] (off the mesh). An agent that has never planned, or that the
/// game teleported, is located here before the surface moves it.
fn locate(mesh: &NavMesh, poly: u32, pos: Vec3, radius: f32) -> u32 {
    let height = mesh.field.params.agent_height;
    if let Some((tile, p)) = mesh.poly(poly) {
        let on = NavMesh::closest_on(tile, p, pos);
        if Vec2::new(on.x - pos.x, on.z - pos.z).length_squared() < 1e-6
            && (on.y - pos.y).abs() <= height
        {
            return poly;
        }
    }
    let reach = (radius * 2.0).max(1.0);
    mesh.nearest(pos, Vec3::new(reach, height, reach))
        .map_or(NO_POLY, |n| n.0)
}

#[derive(Debug, Default)]
struct State {
    paths: BTreeMap<u64, PathState>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// og.nav on one world: the registry it reads and the agents' paths.
#[derive(Clone)]
pub struct NavSystem {
    pub navs: SharedNavs,
    state: Arc<Mutex<State>>,
}

impl NavSystem {
    /// The path an agent follows now (for the probe's dumps).
    pub fn path_of(&self, e: Entity) -> Option<PathState> {
        lock(&self.state).paths.get(&e.to_bits()).cloned()
    }
}

/// Registers the records, adds `og.nav` to the schedule (after the game's
/// systems, before physics) and its state block.
pub fn install(world: &mut World, navs: SharedNavs, desc: NavDesc) -> Result<NavSystem> {
    world.register::<Transform>()?;
    world.register::<Velocity>()?;
    world.register::<NavAgent>()?;
    world.register::<Steering>()?;
    let sys = NavSystem {
        navs,
        state: Arc::new(Mutex::new(State::default())),
    };
    let run_sys = sys.clone();
    world.add_system(
        SystemDesc::new(SYSTEM_NAME)
            .write::<NavAgent>()
            .write::<Transform>()
            .write::<Velocity>()
            .write::<Steering>()
            .query(QueryDesc::new().write::<NavAgent>().write::<Transform>()),
        move |ctx| run(ctx, &run_sys, &desc),
    )?;
    world.add_state_block(SYSTEM_NAME, Box::new(NavBlock(sys.clone())))?;
    Ok(sys)
}

/// A deterministic value in [-1, 1] from an entity and a tick.
fn noise(e: u64, tick: u64) -> f32 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in e.to_le_bytes().iter().chain(tick.to_le_bytes().iter()) {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    ((h >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
}

/// Plans a path on `nav` from `from` to `to`: world corners and a status.
pub fn plan(
    nav: &Nav,
    plane: Plane,
    from: Vec3,
    to: Vec3,
    radius: f32,
    smooth: bool,
    scratch: &mut Scratch,
) -> (Vec<Vec3>, u32, u32) {
    match nav {
        Nav::Grid(g) => {
            let (a, b) = (plane.to_plane(from), plane.to_plane(to));
            let reach = 4;
            let (Some(sc), Some(gc)) = (g.nearest_walkable(a, reach), g.nearest_walkable(b, reach))
            else {
                return (Vec::new(), STATUS_UNREACHABLE, NO_POLY);
            };
            let Some(path) = g.find_path(sc, gc, Algorithm::Jps, scratch) else {
                return (Vec::new(), STATUS_UNREACHABLE, NO_POLY);
            };
            let end = if g.cell_at(b) == Some(gc) {
                b
            } else {
                g.center(gc.0, gc.1)
            };
            let clearance = (radius / g.cell_size).min(0.45);
            let pts = g.waypoints(&path, a, end, smooth, clearance);
            let h = plane.height(from);
            let status = if g.cell_at(b) == Some(gc) {
                STATUS_MOVING
            } else {
                STATUS_PARTIAL
            };
            (
                pts.into_iter().map(|p| plane.to_world(p, h)).collect(),
                status,
                NO_POLY,
            )
        }
        Nav::Mesh(m) => {
            let extent = Vec3::new(2.0, m.field.params.agent_height.max(2.0), 2.0);
            match m.find_path(from, to, extent) {
                Some(p) => {
                    let status = if p.complete {
                        STATUS_MOVING
                    } else {
                        STATUS_PARTIAL
                    };
                    (
                        p.points,
                        status,
                        p.polys.first().copied().unwrap_or(NO_POLY),
                    )
                }
                None => (Vec::new(), STATUS_UNREACHABLE, NO_POLY),
            }
        }
        Nav::Flow { .. } => (Vec::new(), STATUS_MOVING, NO_POLY),
    }
}

#[derive(Clone, Copy)]
struct Row {
    e: Entity,
    agent: NavAgent,
    pos: Vec3,
    steer: Option<Steering>,
    vel: Option<Velocity>,
}

fn run(ctx: &mut SystemContext<'_>, sys: &NavSystem, desc: &NavDesc) -> Result<()> {
    let plane = desc.plane;
    let tick = ctx.time().tick;
    let dt = ctx.scaled_dt();
    let mut rows: Vec<Row> = Vec::new();
    for a in ctx.query(0)? {
        for &e in a.entities() {
            let agent = ctx.get::<NavAgent>(e)?;
            let t = ctx.get::<Transform>(e)?;
            rows.push(Row {
                e,
                agent,
                pos: t.position,
                steer: ctx.get::<Steering>(e).ok(),
                vel: ctx.get::<Velocity>(e).ok(),
            });
        }
    }
    if rows.is_empty() {
        return Ok(());
    }
    let mut navs = lock(&sys.navs);
    navs.refresh_flows();
    let mut state = lock(&sys.state);
    // Drop the paths of agents that are gone.
    let alive: std::collections::BTreeSet<u64> = rows.iter().map(|r| r.e.to_bits()).collect();
    state.paths.retain(|k, _| alive.contains(k));
    // Positions for separation, bucketed by 2 m cells.
    let cell = 2.0f32;
    let mut buckets: BTreeMap<(i32, i32), Vec<usize>> = BTreeMap::new();
    for (i, r) in rows.iter().enumerate() {
        let p = plane.to_plane(r.pos);
        buckets
            .entry(((p.x / cell).floor() as i32, (p.y / cell).floor() as i32))
            .or_default()
            .push(i);
    }
    let positions: Vec<Vec2> = rows.iter().map(|r| plane.to_plane(r.pos)).collect();
    let mut scratch = Scratch::default();
    let mut budget = if desc.plans_per_tick == 0 {
        u32::MAX
    } else {
        desc.plans_per_tick
    };
    for (i, row) in rows.iter().enumerate() {
        let Row {
            e,
            mut agent,
            pos,
            steer: steering,
            vel,
        } = *row;
        let key = e.to_bits();
        let p2 = plane.to_plane(pos);
        // A body's actual velocity (physics, or the game, may have changed it).
        let mut v2 = plane.to_plane(vel.map_or(agent.velocity, |v| v.linear));
        let mut desired = Vec2::ZERO;
        let mut new_pos = pos;
        let ps = state.paths.entry(key).or_default();
        let nav = navs.get(agent.nav);
        let mut status = if agent.seeking {
            STATUS_MOVING
        } else {
            STATUS_IDLE
        };
        let seek_w = steering.map_or(1.0, |s| s.seek);
        if agent.seeking {
            match nav {
                Some(Nav::Flow { field, .. }) => {
                    let d = field.sample(p2);
                    if field.cost_at(p2) == Some(0) || d == Vec2::ZERO {
                        status = if field.cost_at(p2).is_some() {
                            STATUS_ARRIVED
                        } else {
                            STATUS_UNREACHABLE
                        };
                    } else {
                        desired = d * agent.speed;
                    }
                    agent.waypoint = plane.to_world(p2 + d, plane.height(pos));
                    agent.remaining = field
                        .cost_at(p2)
                        .map_or(0.0, |c| c as f32 / 1000.0 * field.cell_size);
                }
                _ => {
                    let version = match nav {
                        Some(Nav::Grid(g)) => g.version(),
                        Some(Nav::Mesh(m)) => m.version(),
                        _ => 0,
                    };
                    let moved = ps.target.distance(agent.target) > agent.repath_distance.max(1e-4);
                    let due = agent.repath_ticks > 0 && ps.age >= agent.repath_ticks;
                    // Off its path (teleported, knocked away): plan again from here.
                    let off = ps.corners.len() >= 2 && {
                        let n = (ps.next as usize).clamp(1, ps.corners.len() - 1);
                        let (a, b) = (
                            plane.to_plane(ps.corners[n - 1]),
                            plane.to_plane(ps.corners[n]),
                        );
                        let ab = b - a;
                        let t = if ab.length_squared() > 1e-12 {
                            ((p2 - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        p2.distance(a + ab * t) > OFF_PATH.max(agent.radius * 4.0)
                    };
                    let stale = ps.nav != agent.nav
                        || ps.version != version
                        || ps.corners.is_empty()
                        || off;
                    if (moved || due || stale) && budget > 0 {
                        budget -= 1;
                        let (corners, st, poly) = match nav {
                            Some(n) => plan(
                                n,
                                plane,
                                pos,
                                agent.target,
                                agent.radius,
                                agent.smooth,
                                &mut scratch,
                            ),
                            None => (vec![pos, agent.target], STATUS_MOVING, NO_POLY),
                        };
                        ps.corners = corners;
                        ps.next = 1.min(ps.corners.len().saturating_sub(1) as u32);
                        ps.target = agent.target;
                        ps.nav = agent.nav;
                        ps.version = version;
                        ps.age = 0;
                        ps.status = st;
                        if poly != NO_POLY || ps.poly == NO_POLY {
                            ps.poly = poly;
                        }
                        agent.plans += 1;
                    } else {
                        ps.age += 1;
                    }
                    status = ps.status;
                    if ps.corners.len() >= 2 {
                        let last = ps.corners.len() - 1;
                        // A corner is passed within reach of it, or once the agent is
                        // past it towards the next one (fast agents overshoot).
                        let reach = agent
                            .stop_distance
                            .max(agent.radius * 0.5)
                            .max(agent.speed * dt * 1.5)
                            .max(0.05);
                        while (ps.next as usize) < last {
                            let c = plane.to_plane(ps.corners[ps.next as usize]);
                            let n = plane.to_plane(ps.corners[ps.next as usize + 1]);
                            let d = c.distance(p2);
                            let past = (p2 - c).dot(n - c) > 0.0 && d < reach * 4.0 + agent.radius;
                            if d <= reach || past {
                                ps.next += 1;
                            } else {
                                break;
                            }
                        }
                        let wp = ps.corners[ps.next as usize];
                        agent.waypoint = wp;
                        let w2 = plane.to_plane(wp);
                        let goal = plane.to_plane(ps.corners[last]);
                        let mut remaining = w2.distance(p2);
                        for k in ps.next as usize..last {
                            remaining += plane
                                .to_plane(ps.corners[k])
                                .distance(plane.to_plane(ps.corners[k + 1]));
                        }
                        agent.remaining = remaining;
                        if (ps.next as usize) == last
                            && goal.distance(p2) <= agent.stop_distance.max(1e-3)
                        {
                            if status == STATUS_MOVING {
                                status = STATUS_ARRIVED;
                            }
                        } else if (ps.next as usize) == last {
                            desired = steer::arrive(
                                p2,
                                w2,
                                agent.speed,
                                agent.slow_radius,
                                agent.stop_distance,
                            );
                        } else {
                            desired = steer::seek(p2, w2, agent.speed);
                        }
                    } else if status != STATUS_UNREACHABLE {
                        status = STATUS_ARRIVED;
                    }
                }
            }
            desired *= seek_w;
        } else {
            ps.corners.clear();
            ps.age = 0;
            agent.remaining = 0.0;
        }
        // Steering behaviours.
        if let Some(mut s) = steering {
            if s.flee > 0.0 {
                desired += steer::flee(p2, plane.to_plane(s.flee_from), agent.speed, s.flee_radius)
                    * s.flee;
            }
            if s.wander > 0.0 {
                let heading = if v2.length_squared() > 1e-6 {
                    v2
                } else {
                    Vec2::X
                };
                let circle = steer::WanderCircle {
                    radius: s.wander_radius,
                    distance: s.wander_distance,
                    jitter: s.wander_jitter,
                };
                let (w, a) = steer::wander(
                    heading,
                    s.wander_angle,
                    noise(key, tick),
                    circle,
                    dt,
                    agent.speed,
                );
                s.wander_angle = a;
                desired += w * s.wander;
            }
            let mover = |target: u64| -> Option<(Vec2, Vec2)> {
                if target == 0 {
                    return None;
                }
                let t = Entity::from_bits(target);
                if !ctx.is_alive(t) {
                    return None;
                }
                let tp = ctx.get::<Transform>(t).ok()?.position;
                let tv = ctx
                    .get::<NavAgent>(t)
                    .ok()
                    .map(|a| a.velocity)
                    .or_else(|| ctx.get::<Velocity>(t).ok().map(|v| v.linear))
                    .unwrap_or(Vec3::ZERO);
                Some((plane.to_plane(tp), plane.to_plane(tv)))
            };
            if let Some((tp, tv)) = mover(s.pursue) {
                desired += steer::pursue(p2, tp, tv, agent.speed) * s.pursue_weight;
                status = STATUS_MOVING;
            }
            if let Some((tp, tv)) = mover(s.evade) {
                desired += steer::evade(p2, tp, tv, agent.speed, s.flee_radius) * s.evade_weight;
            }
            if s.separation > 0.0 {
                let r = s.separation_radius.max(agent.radius * 2.0);
                let (cx, cy) = ((p2.x / cell).floor() as i32, (p2.y / cell).floor() as i32);
                let reachc = (r / cell).ceil() as i32;
                let mut others = Vec::new();
                for by in cy - reachc..=cy + reachc {
                    for bx in cx - reachc..=cx + reachc {
                        if let Some(list) = buckets.get(&(bx, by)) {
                            for &j in list {
                                if j != i {
                                    others.push(positions[j]);
                                }
                            }
                        }
                    }
                }
                desired += steer::separation(p2, others, r, agent.speed) * s.separation;
            }
            if s.avoidance > 0.0
                && let Some(ray) = &desc.raycast
            {
                let heading = if v2.length_squared() > 1e-4 {
                    v2
                } else {
                    desired
                };
                let origin = pos;
                let a = steer::avoid(heading, s.avoid_distance, agent.speed, |d| {
                    let dir = plane.to_world(d, 0.0);
                    ray(origin, dir, s.avoid_distance, s.avoid_mask, e).map(|(dist, n)| {
                        steer::Feeler {
                            distance: dist,
                            normal: plane.to_plane(n),
                        }
                    })
                });
                desired += a * s.avoidance;
            }
            if s != steering.unwrap() {
                ctx.set(e, s)?;
            }
        }
        if !agent.drive {
            agent.velocity = vel.map_or(Vec3::ZERO, |v| v.linear);
            agent.status = status;
            ctx.set(e, agent)?;
            continue;
        }
        // Velocity and position.
        let speed_cap = agent.speed.max(0.0);
        v2 = steer::accelerate(
            v2,
            desired.clamp_length_max(speed_cap),
            agent.acceleration,
            dt,
            speed_cap,
        );
        if desired == Vec2::ZERO && v2.length_squared() < 1e-6 {
            v2 = Vec2::ZERO;
        }
        let height = plane.height(pos);
        match vel {
            Some(mut v) => {
                let keep = plane.height(v.linear);
                v.linear = plane.to_world(v2, keep);
                ctx.set(e, v)?;
            }
            None => {
                let step = p2 + v2 * dt;
                new_pos = plane.to_world(step, height);
                // An agent at rest is left where it stands; one that moves is
                // walked along the surface from the polygon it stands on.
                if let Some(Nav::Mesh(m)) = nav
                    && dt > 0.0
                    && step != p2
                {
                    let poly = locate(m, ps.poly, pos, agent.radius);
                    ps.poly = poly;
                    if m.poly(poly).is_some() {
                        let (np, at) = m.move_along(poly as PolyRef, pos, new_pos);
                        ps.poly = np;
                        // The motion the surface allowed is the velocity.
                        let moved = plane.to_plane(at) - p2;
                        v2 = if dt > 0.0 { moved / dt } else { v2 };
                        new_pos = at;
                    }
                }
            }
        }
        agent.velocity = plane.to_world(v2, 0.0);
        agent.status = status;
        let mut t = ctx.get::<Transform>(e)?;
        let changed_pos = new_pos != t.position;
        t.position = new_pos;
        let mut changed_rot = false;
        if agent.face_movement && v2.length_squared() > 0.0025 {
            let yaw = match plane {
                Plane::Xz => og_core::math::atan2(v2.x, v2.y),
                Plane::Xy => og_core::math::atan2(v2.y, v2.x),
            };
            let axis = match plane {
                Plane::Xz => Vec3::Y,
                Plane::Xy => Vec3::Z,
            };
            let want = og_core::math::quat_from_axis_angle(axis, yaw);
            let have = t.quaternion.normalize();
            let angle = have.angle_between(want);
            let max = agent.turn_speed * dt;
            let q = if angle <= max || angle < 1e-5 {
                want
            } else {
                have.slerp(want, max / angle)
            };
            changed_rot = q != t.quaternion;
            t.quaternion = q;
        }
        if changed_pos || changed_rot {
            ctx.set(e, t)?;
        }
        ctx.set(e, agent)?;
    }
    Ok(())
}

// ── the state block ────────────────────────────────────────────────────────

struct NavBlock(NavSystem);

/// What a snapshot holds: the paths, and the registry's edits.
#[derive(Debug)]
struct Saved {
    paths: BTreeMap<u64, PathState>,
    grids: Vec<(u64, Vec<u8>)>,
    obstacles: Vec<(u64, Vec<(u32, Obstacle)>)>,
    flows: Vec<(u64, Vec<(u32, u32)>)>,
}

struct Writer<'a>(&'a mut Vec<u8>);
impl Writer<'_> {
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn v3(&mut self, v: Vec3) {
        self.f32(v.x);
        self.f32(v.y);
        self.f32(v.z);
    }
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        if self.at + n > self.b.len() {
            return Err(Error::new(
                ErrorKind::Invalid,
                "the og.nav state block is truncated",
            ));
        }
        let s = &self.b[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn v3(&mut self) -> Result<Vec3> {
        Ok(Vec3::new(self.f32()?, self.f32()?, self.f32()?))
    }
}

impl StateBlock for NavBlock {
    fn save(&self, out: &mut Vec<u8>) {
        let st = lock(&self.0.state);
        let navs = lock(&self.0.navs);
        let mut w = Writer(out);
        w.u32(st.paths.len() as u32);
        for (k, p) in &st.paths {
            w.u64(*k);
            w.u32(p.corners.len() as u32);
            for c in &p.corners {
                w.v3(*c);
            }
            w.u32(p.next);
            w.v3(p.target);
            w.u64(p.nav);
            w.u64(p.version);
            w.u32(p.age);
            w.u32(p.poly);
            w.u32(p.status);
        }
        let grids: Vec<(&u64, &Grid)> = navs
            .entries
            .iter()
            .filter_map(|(h, n)| {
                if let Nav::Grid(g) = n {
                    Some((h, g))
                } else {
                    None
                }
            })
            .collect();
        w.u32(grids.len() as u32);
        for (h, g) in grids {
            w.u64(*h);
            w.u32(g.costs().len() as u32);
            w.0.extend_from_slice(g.costs());
        }
        let meshes: Vec<(&u64, &NavMesh)> = navs
            .entries
            .iter()
            .filter_map(|(h, n)| {
                if let Nav::Mesh(m) = n {
                    Some((h, m.as_ref()))
                } else {
                    None
                }
            })
            .collect();
        w.u32(meshes.len() as u32);
        for (h, m) in meshes {
            w.u64(*h);
            let obs: Vec<(u32, &Obstacle)> = m.obstacles().collect();
            w.u32(obs.len() as u32);
            for (id, o) in obs {
                w.u32(id);
                w.v3(o.min);
                w.v3(o.max);
            }
        }
        let flows: Vec<(&u64, &Vec<(u32, u32)>)> = navs
            .entries
            .iter()
            .filter_map(|(h, n)| {
                if let Nav::Flow { targets, .. } = n {
                    Some((h, targets))
                } else {
                    None
                }
            })
            .collect();
        w.u32(flows.len() as u32);
        for (h, t) in flows {
            w.u64(*h);
            w.u32(t.len() as u32);
            for &(x, y) in t {
                w.u32(x);
                w.u32(y);
            }
        }
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        let mut r = Reader { b: bytes, at: 0 };
        let mut paths = BTreeMap::new();
        for _ in 0..r.u32()? {
            let k = r.u64()?;
            let n = r.u32()? as usize;
            let mut corners = Vec::with_capacity(n.min(4096));
            for _ in 0..n {
                corners.push(r.v3()?);
            }
            let p = PathState {
                corners,
                next: r.u32()?,
                target: r.v3()?,
                nav: r.u64()?,
                version: r.u64()?,
                age: r.u32()?,
                poly: r.u32()?,
                status: r.u32()?,
            };
            paths.insert(k, p);
        }
        let mut grids = Vec::new();
        for _ in 0..r.u32()? {
            let h = r.u64()?;
            let n = r.u32()? as usize;
            grids.push((h, r.take(n)?.to_vec()));
        }
        let mut obstacles = Vec::new();
        for _ in 0..r.u32()? {
            let h = r.u64()?;
            let mut list = Vec::new();
            for _ in 0..r.u32()? {
                list.push((
                    r.u32()?,
                    Obstacle {
                        min: r.v3()?,
                        max: r.v3()?,
                    },
                ));
            }
            obstacles.push((h, list));
        }
        let mut flows = Vec::new();
        for _ in 0..r.u32()? {
            let h = r.u64()?;
            let mut t = Vec::new();
            for _ in 0..r.u32()? {
                t.push((r.u32()?, r.u32()?));
            }
            flows.push((h, t));
        }
        Ok(Box::new(Saved {
            paths,
            grids,
            obstacles,
            flows,
        }))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut st = lock(&self.0.state);
        let Some(saved) = decoded.and_then(|d| d.downcast::<Saved>().ok()) else {
            st.paths.clear();
            return;
        };
        st.paths = saved.paths;
        let mut navs = lock(&self.0.navs);
        for (h, costs) in saved.grids {
            if let Some(Nav::Grid(g)) = navs.get_mut(h)
                && g.costs() != costs.as_slice()
                && costs.len() == g.costs().len()
            {
                for (i, &c) in costs.iter().enumerate() {
                    let (x, y) = (i as u32 % g.width, i as u32 / g.width);
                    g.set(x, y, c);
                }
            }
        }
        for (h, list) in saved.obstacles {
            if let Some(Nav::Mesh(m)) = navs.get_mut(h) {
                let have: Vec<(u32, Obstacle)> = m.obstacles().map(|(k, o)| (k, *o)).collect();
                if have != list {
                    for (k, _) in have {
                        m.set_obstacle(k, None);
                    }
                    for (k, o) in list {
                        m.set_obstacle(k, Some(o));
                    }
                }
            }
        }
        for (h, targets) in saved.flows {
            let grid = match navs.get(h) {
                Some(Nav::Flow { grid, .. }) => *grid,
                _ => continue,
            };
            let Some(Nav::Grid(g)) = navs.get(grid).cloned() else {
                continue;
            };
            if let Some(Nav::Flow {
                targets: t, field, ..
            }) = navs.get_mut(h)
                && *t != targets
            {
                *field = FlowField::build(&g, &targets);
                *t = targets;
            }
        }
        navs.refresh_flows();
    }
}
