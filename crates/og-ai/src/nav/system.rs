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

use crate::nav::flow::FlowField;
use crate::nav::grid::{Algorithm, Grid, Scratch};
use crate::nav::mesh::{
    Filter, Link, LinkKind, NONE as NO_POLY, NavMesh, PathLink, PolyRef, Shape,
};
use crate::steer;
use og_core::IdMap;
use og_core::glam::{Quat, Vec2, Vec3};
use og_core::{Error, ErrorKind, Reflect, Result};
use og_ecs::{Entity, QueryDesc, StateBlock, SystemContext, SystemDesc, World};
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};
mod block;
mod tick;
use tick::*;

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
/// Crossing an off-mesh link (a jump, a ladder, a door, a teleport).
pub const STATUS_ON_LINK: u32 = 5;

/// Sent when an agent reaches an off-mesh link and starts across it (G2-4;
/// registered only once a navmesh has a link, [`enable_links`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.NavLinkReached")]
pub struct NavLinkReached {
    /// The agent.
    pub agent: u64,
    /// The navmesh.
    pub mesh: u64,
    /// The link's id.
    pub link: u32,
    /// Its kind: 0 jump, 1 ladder, 2 door, 3 teleport.
    pub kind: u32,
    /// Where the agent leaves the surface, and where it lands.
    pub start: Vec3,
    pub end: Vec3,
    /// The game moves the agent across (`og.nav.finishLink` ends it).
    pub game: bool,
}

/// Registers `og.NavLinkReached` and grants og.nav its sends: what a host
/// calls the first time a navmesh has an off-mesh link.
pub fn enable_links(world: &mut World) -> Result<()> {
    world.register::<NavLinkReached>()?;
    world.grant(SYSTEM_NAME, &[NavLinkReached::NAME])
}

/// An agent part way across a link.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Traverse {
    pub link: u32,
    pub kind: LinkKind,
    pub from: Vec3,
    pub to: Vec3,
    /// Seconds since it left `from`, and what the crossing takes.
    pub elapsed: f32,
    pub duration: f32,
    /// The game moves it; `finished` once the game said it is over.
    pub game: bool,
    pub finished: bool,
    /// The path corner the link starts at.
    pub corner: u32,
    /// The polygon it lands on.
    pub land: u32,
}

impl Traverse {
    /// Where an engine-moved agent is now: on an arc for a jump, straight otherwise.
    fn at(&self) -> Vec3 {
        let t = if self.duration > 0.0 {
            (self.elapsed / self.duration).min(1.0)
        } else {
            1.0
        };
        let mut p = self.from.lerp(self.to, t);
        if self.kind == LinkKind::Jump {
            let d = Vec2::new(self.to.x - self.from.x, self.to.z - self.from.z).length();
            let h = 0.5 + 0.25 * d;
            p.y += 4.0 * h * t * (1.0 - t);
        }
        p
    }
}

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

/// Which navmesh areas an agent crosses and at what cost (G2-4). Registered
/// with the world only once a game uses it ([`enable_filters`]), so a world
/// without it keeps its type table and hash.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.NavFilter")]
pub struct NavFilter {
    /// Per area id (0 is ground, then the areas a navmesh marks), the cost of a metre in it; past the list's length 1, and at least 0.01.
    #[reflect(name = "areaCosts")]
    pub area_costs: og_core::List<f32, 64>,
    /// Area ids the agent never enters (0 to 63).
    pub excluded: og_core::List<u32, 64>,
}

impl NavFilter {
    /// The path filter it describes.
    pub fn filter(&self) -> Filter {
        let mut excluded = 0u64;
        for &a in self.excluded.as_slice() {
            if a < 64 {
                excluded |= 1u64 << a;
            }
        }
        Filter {
            costs: self.area_costs.as_slice().to_vec(),
            excluded,
        }
    }
}

/// Reciprocal crowd avoidance for an agent (G2-4, ORCA): beside an
/// og.NavAgent, its velocity each tick is the one nearest its path's (and
/// steering's) that keeps it clear of its nearest agents for `horizon`
/// seconds and of the navmesh's walls for `obstacle_horizon`. Registered
/// with the world only once a game uses it ([`enable_crowds`]): an agent
/// without it moves as before (separation only).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Crowd")]
pub struct Crowd {
    /// How many nearest agents it avoids.
    #[reflect(min = 0, max = 32, default = 8, tunable)]
    pub neighbours: u32,
    /// Seconds ahead it keeps clear of other agents.
    #[reflect(min = 0, max = 10, default = 2, unit = "s", tunable)]
    pub horizon: f32,
    /// Seconds ahead it keeps clear of walls (a navmesh's edges).
    #[reflect(
        name = "obstacleHorizon",
        min = 0,
        max = 10,
        default = 0.5,
        unit = "s",
        tunable
    )]
    pub obstacle_horizon: f32,
    /// Its share of each avoidance with another crowd agent.
    #[reflect(min = 0, max = 1, default = 0.5, tunable)]
    pub weight: f32,
}

impl Default for Crowd {
    fn default() -> Self {
        Crowd {
            neighbours: 8,
            horizon: 2.0,
            obstacle_horizon: 0.5,
            weight: 0.5,
        }
    }
}

/// Registers `og.Crowd` and grants og.nav its reads.
pub fn enable_crowds(world: &mut World) -> Result<()> {
    world.register::<Crowd>()?;
    world.grant(SYSTEM_NAME, &[Crowd::NAME])
}

/// Registers `og.NavFilter` and grants og.nav its reads: what a host calls
/// the first time a game uses the component.
pub fn enable_filters(world: &mut World) -> Result<()> {
    world.register::<NavFilter>()?;
    world.grant(SYSTEM_NAME, &[NavFilter::NAME])
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
    pub entries: IdMap<u64, Nav>,
    next: u64,
    /// Per entry made in a world whose zero og.WorldOrigin had moved, that
    /// world's nav origin ([`NavSystem::origin`]) as it was made: its data
    /// is in the frame the world had then (R12). An entry without one
    /// (loaded from a file, or made before any shift) is in the world's
    /// first frame.
    frames: IdMap<u64, Vec3>,
}

impl Navs {
    /// Stores `nav` under a fresh handle (`index + 2^32`).
    pub fn insert(&mut self, nav: Nav) -> u64 {
        self.next += 1;
        let h = self.next | (1u64 << 32);
        self.entries.insert(h, nav);
        h
    }

    /// Stores `nav`, made in the frame of a world whose nav origin is
    /// `at` ([`NavSystem::origin`]): its coordinates are that world's then.
    pub fn insert_at(&mut self, nav: Nav, at: Vec3) -> u64 {
        let h = self.insert(nav);
        if at != Vec3::ZERO {
            self.frames.insert(h, at);
        }
        h
    }

    /// What to add to a point of entry `h`'s data for the place it is in a
    /// world whose nav origin is `origin` (subtract it from a world point
    /// for the entry's): how far the world's zero moved since the entry's
    /// frame. Exact: both are multiples of og.WorldOrigin's step.
    pub fn offset(&self, h: u64, origin: Vec3) -> Vec3 {
        origin - self.frames.get(&h).copied().unwrap_or(Vec3::ZERO)
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
    /// The navmesh polygon the agent stands on; [`crate::nav::mesh::NONE`] until it is
    /// located (an agent that has never planned stands on none).
    pub poly: u32,
    pub status: u32,
    /// The links the path crosses (G2-4).
    pub links: Vec<PathLink>,
    /// The link it is crossing now.
    pub traverse: Option<Traverse>,
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
            links: Vec::new(),
            traverse: None,
        }
    }
}

/// The polygon of `mesh` an agent at `pos` stands on: `poly` while it still
/// holds the agent (under it on x and z, its surface within the agent's
/// height), else the nearest within the agent's height and radius, else
/// [`crate::nav::mesh::NONE`] (off the mesh). An agent that has never planned, or that the
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
    paths: IdMap<u64, PathState>,
    /// How far og.WorldOrigin has moved the world's zero (R12, the sum of
    /// its shifts): an entry of [`Navs`] is read at
    /// [`Navs::offset`] of it.
    origin: Vec3,
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
    /// How far og.WorldOrigin has moved this world's zero, as the nav data
    /// sees it ([`Navs::offset`]).
    pub fn origin(&self) -> Vec3 {
        lock(&self.state).origin
    }

    /// The path an agent follows now (for the probe's dumps).
    pub fn path_of(&self, e: Entity) -> Option<PathState> {
        lock(&self.state).paths.get(&e.to_bits()).cloned()
    }

    /// Ends a game crossing of a link (`og.nav.finishLink`): the agent lands
    /// at the link's end as og.nav next runs. False when it is not on one
    /// the game moves.
    pub fn finish_link(&self, e: Entity) -> bool {
        let mut st = lock(&self.state);
        match st
            .paths
            .get_mut(&e.to_bits())
            .and_then(|p| p.traverse.as_mut())
        {
            Some(t) if t.game => {
                t.finished = true;
                true
            }
            _ => false,
        }
    }

    /// Every agent's path now, by entity bits (the debug view's `nav`).
    pub fn paths(&self) -> Vec<(u64, PathState)> {
        lock(&self.state)
            .paths
            .iter()
            .map(|(e, p)| (*e, p.clone()))
            .collect()
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
    let mut h = og_core::hash::StableHasher::new();
    h.write_u64(e);
    h.write_u64(tick);
    og_core::hash::unit24_of64(h.finish()) * 2.0 - 1.0
}

/// What [`plan`] answers: world corners, a status, the navmesh polygon
/// the start stands on and the links the path crosses.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub corners: Vec<Vec3>,
    pub status: u32,
    pub poly: u32,
    pub links: Vec<PathLink>,
}

impl From<(Vec<Vec3>, u32, u32)> for Plan {
    fn from((corners, status, poly): (Vec<Vec3>, u32, u32)) -> Plan {
        Plan {
            corners,
            status,
            poly,
            links: Vec::new(),
        }
    }
}

/// Plans a path on `nav` from `from` to `to` for `agent` (its radius and
/// smoothing) through `filter`: world corners and a status.
pub fn plan(
    nav: &Nav,
    plane: Plane,
    from: Vec3,
    to: Vec3,
    agent: &NavAgent,
    filter: &Filter,
    scratch: &mut Scratch,
) -> Plan {
    let (radius, smooth) = (agent.radius, agent.smooth);
    match nav {
        Nav::Grid(g) => {
            let (a, b) = (plane.to_plane(from), plane.to_plane(to));
            let reach = 4;
            let (Some(sc), Some(gc)) = (g.nearest_walkable(a, reach), g.nearest_walkable(b, reach))
            else {
                return (Vec::new(), STATUS_UNREACHABLE, NO_POLY).into();
            };
            let Some(path) = g.find_path(sc, gc, Algorithm::Jps, scratch) else {
                return (Vec::new(), STATUS_UNREACHABLE, NO_POLY).into();
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
                .into()
        }
        Nav::Mesh(m) => {
            let extent = Vec3::new(2.0, m.field.params.agent_height.max(2.0), 2.0);
            match m.find_path_filtered(from, to, extent, filter) {
                Some(p) => {
                    let status = if p.complete {
                        STATUS_MOVING
                    } else {
                        STATUS_PARTIAL
                    };
                    Plan {
                        corners: p.points,
                        status,
                        poly: p.polys.first().copied().unwrap_or(NO_POLY),
                        links: p.links,
                    }
                }
                None => (Vec::new(), STATUS_UNREACHABLE, NO_POLY).into(),
            }
        }
        Nav::Flow { .. } => (Vec::new(), STATUS_MOVING, NO_POLY).into(),
    }
}

// ── the state block ────────────────────────────────────────────────────────

struct NavBlock(NavSystem);
