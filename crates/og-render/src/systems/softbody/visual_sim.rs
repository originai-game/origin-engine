//! Visual bodies (og-physics' SIMULATION_VISUAL): cloth, soft volumes and
//! garments the renderer simulates on the GPU for looks only, with nothing
//! read back. XPBD in small steps, one workgroup per body; the method and
//! its sources are in `shaders/visual_body.wgsl` and
//! third_party/visual-bodies.md.
//!
//! The host hands in, each frame, the body's rest shape where its entity
//! is this tick (a cloth's or soft volume's coarse mesh through
//! [`crate::RenderScene::set_simulation_surface`]; a garment's skinned
//! pose is posed on the GPU), and the scene's colliders. The rest shape is
//! three things at once: the rest lengths and volume, the targets of the
//! pinned points (moved between the frame's two over its substeps), and
//! the state a reset starts from. Ticks drive it: a frame steps the ticks
//! since the last one it drew (up to [`crate::fields::MAX_CATCH_UP`]); a world that went
//! back, a new body or a larger jump starts again from the rest shape and
//! simulates [`PREROLL`] ticks before it is drawn, so it appears settled.
//!
//! Bindings: one compute-only layout of five storage buffers (the body's
//! header and topology, the frame's colliders, its state, its
//! self-collision grid and the points it draws into), beside the
//! existing layouts, none of which change.

use crate::surface::SimulationSurface;
use crate::{Deferred, MeshData, SceneCollider};
use glam::{Mat4, Vec3};
use std::sync::Arc;

pub(crate) const WGSL: &str = include_str!("../../../shaders/visual_body.wgsl");
const COLLIDERS: &str = wgsl!("scene_colliders.wgsl");

const SURFACE_OUT: &str = "
struct Vertex { pn: vec4<f32>, nuv: vec4<f32>, color: vec4<f32>, tangent: vec4<f32> }
@group(0) @binding(4) var<storage, read_write> output: array<Vertex>;
fn emit(i: u32, p: vec3<f32>) { output[i].pn = vec4<f32>(p, output[i].pn.w); }
";
const GARMENT_OUT: &str = "
@group(0) @binding(4) var<storage, read_write> output: array<vec4<f32>>;
fn emit(i: u32, p: vec3<f32>) { output[i] = vec4<f32>(p, 1.0); }
";

/// The solver for simulation surfaces (writing their coarse vertices).
pub(crate) fn surface_source() -> String {
    format!("{COLLIDERS}\n{WGSL}{SURFACE_OUT}")
}

/// The solver for garments (writing their posed points).
pub(crate) fn garment_source() -> String {
    format!("{COLLIDERS}\n{WGSL}{GARMENT_OUT}")
}

/// The ticks a body that starts again simulates before it is drawn.
pub const PREROLL: u32 = 60;
/// Neighbours a point keeps for self-collision.
const NEIGHBOURS: u32 = 16;
/// Compliance (m/N, every point of unit mass) of a constraint of
/// stiffness 0.5; 1 is rigid (0), and it grows with ((1 - s) / s)² below.
const COMPLIANCE: f32 = 2e-6;
/// A cloth cell's shear compliance: soft, so a sheet wraps what it falls on.
const SHEAR_COMPLIANCE: f32 = 5e-4;

/// How a simulation surface is simulated on the GPU
/// ([`crate::RenderScene::set_visual_body`]), or a garment
/// ([`crate::GarmentDeformation::visual`]). Rates are per tick, lengths in
/// metres.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualBody {
    /// The points held to their place in the rest shape (cloth pins,
    /// `pinTop`), ascending; a garment's are those of reach 0.
    pub pinned: Arc<[u32]>,
    /// A garment's reach per point: how far it may stray from its skinned
    /// target (0: on it). Empty for a surface.
    pub reach: Arc<[f32]>,
    /// A garment's backstop: how far behind its target along the skin's
    /// normal a point may go.
    pub backstop: f32,
    /// Small solver steps per tick, each one pass over the constraints.
    pub substeps: u32,
    /// The velocity a point keeps per tick.
    pub damping: f32,
    /// The acceleration of gravity (og.Physics.gravity times the scale).
    pub gravity: Vec3,
    /// A steady wind and its gusting (a cloth's push along its normal).
    pub wind: Vec3,
    pub gust: f32,
    /// The distance points keep from the scene's colliders (0: none).
    pub radius: f32,
    /// The share of its slide a point touching a collider loses a step.
    pub friction: f32,
    /// The distance points keep from the body's other points (0: none).
    pub thickness: f32,
    /// How stiff its edges are (1 rigid), and its bending (0 none).
    pub stiffness: f32,
    pub bending: f32,
    /// The volume kept as a share of the rest volume, and how firmly (a
    /// soft volume's; 0 for cloth).
    pub pressure: f32,
    pub volume_stiffness: f32,
    /// The 2D profile's plane: points keep this z.
    pub plane: Option<f32>,
    /// A garment's rest shape and skin normals (None for a surface).
    pub garment: Option<VisualGarment>,
}

/// A visual garment's shapes this frame, in the world: its welded points'
/// bind shape where the entity is (the rest lengths) and the skin's
/// normals at its targets (the backstop's direction).
#[derive(Clone, Debug, PartialEq)]
pub struct VisualGarment {
    pub rest: Arc<Vec<Vec3>>,
    pub normals: Arc<Vec<Vec3>>,
}

impl Default for VisualBody {
    fn default() -> Self {
        VisualBody {
            pinned: Arc::from([]),
            reach: Arc::from([]),
            backstop: 0.0,
            substeps: 16,
            damping: 0.985,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            wind: Vec3::ZERO,
            gust: 0.0,
            radius: 0.0,
            friction: 0.0,
            thickness: 0.0,
            stiffness: 1.0,
            bending: 0.0,
            pressure: 0.0,
            volume_stiffness: 0.0,
            plane: None,
            garment: None,
        }
    }
}

/// What a frame does with a body: start again (from its rest shape, with
/// the preroll), and how many ticks it steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Plan {
    pub reset: bool,
    pub steps: u32,
    /// The first tick stepped (the gust's clock).
    pub first: u64,
}

/// The frame's plan for a body last stepped to `last` (`None`: new) at
/// `tick`.
pub(crate) fn plan(last: Option<u64>, tick: u64) -> Plan {
    // The preroll settles the body, so it is as long at the world's first
    // tick as at any other: its clock starts before the tick drawn.
    let steps = crate::fields::catch_up::catch_up(last, tick, 1);
    if steps.restart {
        Plan {
            reset: true,
            steps: PREROLL,
            first: tick.wrapping_sub(u64::from(PREROLL)),
        }
    } else {
        Plan {
            reset: false,
            steps: steps.count as u32,
            first: steps.first,
        }
    }
}

/// The kind word of the header.
fn kind(surface: Option<SimulationSurface>, body: &VisualBody) -> u32 {
    match surface {
        Some(SimulationSurface::Cloth { .. }) => 0,
        Some(SimulationSurface::SoftBody) if body.plane.is_some() => 2,
        Some(SimulationSurface::SoftBody) => 1,
        None => 3,
    }
}

/// What an edge holds, each with its own compliance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hold {
    Stretch = 0,
    /// A cloth cell's diagonals: softer than its sides, so it can wrap
    /// a curved surface (cloth shears easily, it hardly stretches).
    Shear = 1,
    Bend = 2,
}

/// An edge: its points, its rest length and what it holds.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Edge {
    a: u32,
    b: u32,
    rest: f32,
    hold: Hold,
}

/// The constraints between a body's points: a cloth grid's structural,
/// shear and skip-one bending links; a soft volume's surface edges; a
/// ring's neighbours; a garment's edges and the far corners across each
/// shared edge (garment.rs's bends).
fn edges(
    surface: Option<SimulationSurface>,
    body: &VisualBody,
    rest: &[Vec3],
    triangles: &[[u32; 3]],
) -> Vec<Edge> {
    let mut out = Vec::new();
    let mut link = |a: u32, b: u32, hold: Hold| {
        out.push(Edge {
            a: a.min(b),
            b: a.max(b),
            rest: (rest[b as usize] - rest[a as usize]).length(),
            hold,
        })
    };
    match surface {
        Some(SimulationSurface::Cloth { columns, rows }) => {
            for y in 0..rows {
                for x in 0..columns {
                    let i = y * columns + x;
                    if x + 1 < columns {
                        link(i, i + 1, Hold::Stretch);
                    }
                    if y + 1 < rows {
                        link(i, i + columns, Hold::Stretch);
                        if x + 1 < columns {
                            link(i, i + columns + 1, Hold::Shear);
                            link(i + 1, i + columns, Hold::Shear);
                        }
                    }
                    if x + 2 < columns {
                        link(i, i + 2, Hold::Bend);
                    }
                    if y + 2 < rows {
                        link(i, i + 2 * columns, Hold::Bend);
                    }
                }
            }
        }
        Some(SimulationSurface::SoftBody) if body.plane.is_some() => {
            let n = rest.len() as u32;
            for i in 0..n {
                link(i, (i + 1) % n, Hold::Stretch);
            }
        }
        _ => {
            let mut pairs: std::collections::BTreeMap<(u32, u32), Vec<u32>> = Default::default();
            for &[a, b, c] in triangles {
                if a == b || b == c || c == a {
                    continue;
                }
                for (p, q, far) in [(a, b, c), (b, c, a), (c, a, b)] {
                    pairs.entry((p.min(q), p.max(q))).or_default().push(far);
                }
            }
            for &(a, b) in pairs.keys() {
                link(a, b, Hold::Stretch);
            }
            if surface.is_none() {
                let mut bends: Vec<(u32, u32)> = pairs
                    .values()
                    .filter_map(|far| match far[..] {
                        [c, d] if c != d => Some((c.min(d), c.max(d))),
                        _ => None,
                    })
                    .collect();
                bends.sort_unstable();
                bends.dedup();
                for (a, b) in bends {
                    link(a, b, Hold::Bend);
                }
            }
        }
    }
    out
}

/// Greedy edge colouring: no two edges of a colour share a point, so a
/// colour's edges solve in parallel. The edges by colour, and each
/// colour's (first edge, count).
fn colour(edges: &[Edge], points: usize) -> (Vec<Edge>, Vec<(u32, u32)>) {
    let mut used = vec![0u128; points];
    let mut colours = Vec::with_capacity(edges.len());
    let mut most = 0;
    for e in edges {
        let taken = used[e.a as usize] | used[e.b as usize];
        let c = (!taken).trailing_zeros().min(127);
        used[e.a as usize] |= 1 << c;
        used[e.b as usize] |= 1 << c;
        colours.push(c);
        most = most.max(c + 1);
    }
    let mut out = Vec::with_capacity(edges.len());
    let mut table = Vec::with_capacity(most as usize);
    for c in 0..most {
        let first = out.len() as u32;
        out.extend(
            edges
                .iter()
                .zip(&colours)
                .filter(|&(_, &k)| k == c)
                .map(|(e, _)| *e),
        );
        table.push((first, out.len() as u32 - first));
    }
    (out, table)
}

/// The enclosed volume of a closed surface (relative to its first point).
fn volume(rest: &[Vec3], triangles: &[[u32; 3]]) -> f32 {
    let o = rest.first().copied().unwrap_or_default();
    triangles
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| rest[i as usize] - o);
            a.dot(b.cross(c)) / 6.0
        })
        .sum()
}

/// The area of a ring in the xy plane.
fn area(rest: &[Vec3]) -> f32 {
    let n = rest.len();
    (0..n)
        .map(|i| {
            let (a, b) = (rest[i] - rest[0], rest[(i + 1) % n] - rest[0]);
            (a.x * b.y - b.x * a.y) * 0.5
        })
        .sum()
}

/// Header words (visual_body.wgsl's constants).
mod h {
    pub const N: usize = 0;
    pub const KIND: usize = 1;
    pub const COLUMNS: usize = 2;
    pub const ROWS: usize = 3;
    pub const STEPS: usize = 4;
    pub const RESET: usize = 5;
    pub const SUB: usize = 6;
    pub const TICK: usize = 7;
    pub const DT: usize = 8;
    pub const KEEP: usize = 9;
    pub const GRAVITY: usize = 10;
    pub const PLANAR: usize = 13;
    pub const PLANE_Z: usize = 14;
    pub const WIND: usize = 15;
    pub const GUST: usize = 18;
    pub const RADIUS: usize = 19;
    pub const FRICTION: usize = 20;
    pub const THICKNESS: usize = 21;
    pub const ALPHA_STRETCH: usize = 22;
    pub const ALPHA_BEND: usize = 23;
    pub const VOLUME_K: usize = 24;
    pub const PRESSURE: usize = 25;
    pub const REST_VOLUME: usize = 26;
    pub const COLOURS: usize = 27;
    pub const COLOUR_TABLE: usize = 28;
    pub const INV_MASS: usize = 29;
    pub const TETHER: usize = 30;
    pub const TRIANGLES: usize = 31;
    pub const TRIANGLE_COUNT: usize = 32;
    pub const CSR: usize = 33;
    pub const CELLS: usize = 34;
    pub const NEIGHBOURS: usize = 35;
    pub const COLLIDER_LIST: usize = 36;
    pub const REACH: usize = 37;
    pub const BACKSTOP: usize = 38;
    pub const ALPHA_SHEAR: usize = 39;
    pub const WORLD: usize = 40;
    pub const MODEL: usize = 52;
    pub const LEN: usize = 64;
}

/// A body's topology as the GPU reads it, and what decides when to build
/// it again.
#[derive(Debug, PartialEq)]
pub(crate) struct Topology {
    /// Header (its static words filled) and data.
    pub words: Vec<u32>,
    pub points: u32,
    /// Self-collision hash cells (0: none).
    pub cells: u32,
    /// The farthest a point can be from its rest place: the culling margin.
    pub margin: f32,
    key: TopologyKey,
}

#[derive(Debug, PartialEq)]
struct TopologyKey {
    kind: u32,
    points: usize,
    triangles: Vec<[u32; 3]>,
    pinned: Arc<[u32]>,
    reach: Arc<[f32]>,
    thickness: bool,
    /// Every edge's points and rest length: a rest shape whose lengths
    /// moved builds the topology again.
    edges: Vec<(u32, u32, f32)>,
}

/// A rest length's change that builds the topology again.
const REST_TOLERANCE: f32 = 1e-4;

impl Topology {
    pub(crate) fn build(
        surface: Option<SimulationSurface>,
        body: &VisualBody,
        rest: &[Vec3],
        triangles: &[[u32; 3]],
    ) -> Topology {
        let n = rest.len();
        let kind = kind(surface, body);
        let edges = edges(surface, body, rest, triangles);
        let (coloured, table) = colour(&edges, n);
        let mut w = vec![0u32; h::LEN];
        w[h::N] = n as u32;
        w[h::KIND] = kind;
        if let Some(SimulationSurface::Cloth { columns, rows }) = surface {
            w[h::COLUMNS] = columns;
            w[h::ROWS] = rows;
        }
        w[h::COLOURS] = table.len() as u32;
        w[h::COLOUR_TABLE] = w.len() as u32;
        let base = (w.len() + table.len() * 2) as u32;
        for &(first, count) in &table {
            w.extend([base + first * 4, count]);
        }
        for e in &coloured {
            w.extend([e.a, e.b, e.rest.to_bits(), e.hold as u32]);
        }
        let mut inv = vec![1.0f32; n];
        for &p in body.pinned.iter() {
            if let Some(m) = inv.get_mut(p as usize) {
                *m = 0.0;
            }
        }
        for (m, &r) in inv.iter_mut().zip(body.reach.iter()) {
            if r <= 0.0 {
                *m = 0.0;
            }
        }
        w[h::INV_MASS] = w.len() as u32;
        w.extend(inv.iter().map(|m| m.to_bits()));
        // Long-range attachments to the nearest pin at rest (not a garment's:
        // its targets hold it).
        let mut margin = 0.0f32;
        if kind != 3 && !body.pinned.is_empty() {
            w[h::TETHER] = w.len() as u32;
            for (i, p) in rest.iter().enumerate() {
                let nearest = body
                    .pinned
                    .iter()
                    .filter(|&&k| (k as usize) < n)
                    .map(|&k| (k, (rest[k as usize] - *p).length()))
                    .min_by(|a, b| a.1.total_cmp(&b.1));
                match nearest {
                    Some((k, d)) if inv[i] > 0.0 => {
                        w.extend([k, d.to_bits()]);
                        margin = margin.max(2.0 * d);
                    }
                    _ => w.extend([u32::MAX, 0]),
                }
            }
        } else if kind == 3 {
            margin = body.reach.iter().fold(0.0, |m, &r| m.max(r));
        } else {
            // Unpinned: nothing holds it near its rest shape.
            margin = 8.0;
        }
        let mut rest_volume = 0.0;
        // The triangles and each point's incident ones: a soft volume's
        // volume, a cloth's and a garment's aerodynamics.
        if kind != 2 && !triangles.is_empty() {
            w[h::TRIANGLES] = w.len() as u32;
            w[h::TRIANGLE_COUNT] = triangles.len() as u32;
            w.extend(triangles.iter().flatten());
            let mut incident = vec![Vec::new(); n];
            for (t, tri) in triangles.iter().enumerate() {
                for &p in tri {
                    incident[p as usize].push(t as u32);
                }
            }
            w[h::CSR] = w.len() as u32;
            let mut end = 0;
            for f in &incident {
                w.push(end);
                end += f.len() as u32;
            }
            w.push(end);
            w.extend(incident.into_iter().flatten());
        }
        if kind == 1 {
            rest_volume = volume(rest, triangles);
        } else if kind == 2 {
            rest_volume = area(rest);
        }
        w[h::REST_VOLUME] = rest_volume.to_bits();
        if kind == 3 {
            w[h::REACH] = w.len() as u32;
            w.extend(body.reach.iter().map(|r| r.max(0.0).to_bits()));
        }
        let cells = if body.thickness > 0.0 {
            (2 * n as u32).max(64).next_power_of_two()
        } else {
            0
        };
        w[h::CELLS] = cells;
        w[h::NEIGHBOURS] = NEIGHBOURS;
        Topology {
            words: w,
            points: n as u32,
            cells,
            margin: margin + body.radius.max(0.0) + body.thickness.max(0.0),
            key: TopologyKey {
                kind,
                points: n,
                triangles: triangles.to_vec(),
                pinned: body.pinned.clone(),
                reach: body.reach.clone(),
                thickness: body.thickness > 0.0,
                edges: edges.iter().map(|e| (e.a, e.b, e.rest)).collect(),
            },
        }
    }

    /// Whether `build` from these would give this topology.
    pub(crate) fn matches(
        &self,
        surface: Option<SimulationSurface>,
        body: &VisualBody,
        rest: &[Vec3],
        triangles: &[[u32; 3]],
    ) -> bool {
        let k = &self.key;
        k.kind == kind(surface, body)
            && k.points == rest.len()
            && k.triangles == triangles
            && k.pinned == body.pinned
            && k.reach == body.reach
            && k.thickness == (body.thickness > 0.0)
            && k.edges.iter().all(|&(a, b, r)| {
                let now = (rest[b as usize] - rest[a as usize]).length();
                (now - r).abs() <= REST_TOLERANCE * r.max(1.0)
            })
    }

    /// The grid buffer's words: counts, starts, the sorted points, each
    /// point's neighbour count and its neighbours.
    fn grid_words(&self) -> u64 {
        if self.cells == 0 {
            4
        } else {
            u64::from(2 * self.cells + self.points * (2 + NEIGHBOURS))
        }
    }
}

/// The header's per-frame words for `body` this frame.
#[allow(clippy::too_many_arguments)]
pub(crate) fn frame_header(
    words: &mut [u32],
    body: &VisualBody,
    plan: Plan,
    dt: f32,
    list: u32,
    world: Mat4,
) {
    let sub = body.substeps.clamp(1, 128);
    let keep = body.damping.clamp(0.0, 1.0).powf(1.0 / sub as f32);
    let f = |w: &mut [u32], at: usize, v: f32| w[at] = v.to_bits();
    let alpha = |s: f32| {
        let s = s.clamp(0.0, 1.0);
        if s >= 1.0 {
            0.0
        } else {
            COMPLIANCE * ((1.0 - s) / s.max(1e-4)).powi(2)
        }
    };
    words[h::STEPS] = plan.steps;
    words[h::RESET] = u32::from(plan.reset);
    words[h::SUB] = sub;
    words[h::TICK] = plan.first as u32;
    f(words, h::DT, dt);
    f(words, h::KEEP, keep);
    for k in 0..3 {
        f(words, h::GRAVITY + k, body.gravity[k]);
        f(words, h::WIND + k, body.wind[k]);
    }
    words[h::PLANAR] = u32::from(body.plane.is_some());
    f(words, h::PLANE_Z, body.plane.unwrap_or(0.0));
    f(words, h::GUST, body.gust);
    f(words, h::RADIUS, body.radius.max(0.0));
    f(words, h::FRICTION, body.friction);
    f(words, h::THICKNESS, body.thickness.max(0.0));
    f(words, h::ALPHA_STRETCH, alpha(body.stiffness));
    f(
        words,
        h::ALPHA_SHEAR,
        SHEAR_COMPLIANCE / body.stiffness.clamp(1e-3, 1.0),
    );
    f(
        words,
        h::ALPHA_BEND,
        if body.bending > 0.0 {
            alpha(body.bending)
        } else {
            f32::MAX
        },
    );
    f(words, h::VOLUME_K, body.volume_stiffness.clamp(0.0, 1.0));
    f(words, h::PRESSURE, body.pressure.max(0.0));
    words[h::COLLIDER_LIST] = list;
    f(words, h::BACKSTOP, body.backstop.max(0.0));
    let model = world.inverse();
    for (at, m) in [(h::WORLD, world), (h::MODEL, model)] {
        let rows = m.transpose().to_cols_array();
        for (k, v) in rows[..12].iter().enumerate() {
            f(words, at + k, *v);
        }
    }
}

// ---- GPU ----------------------------------------------------------------------

pub(crate) struct VisualPipelines {
    layout: wgpu::BindGroupLayout,
    surface: Deferred<wgpu::ComputePipeline>,
    garment: Deferred<wgpu::ComputePipeline>,
    colliders: wgpu::Buffer,
    /// Bumped when `colliders` is made again (its bind groups with it).
    generation: u64,
    /// This frame's collider list of each visual garment, by its key.
    pub(crate) garment_lists: std::collections::HashMap<(u64, usize), u32>,
}

/// One body's buffers and the clock of its simulation.
pub(crate) struct VisualMesh {
    pub(crate) topology: Topology,
    body: wgpu::Buffer,
    state: wgpu::Buffer,
    grid: wgpu::Buffer,
    group: Option<(u64, wgpu::BindGroup)>,
    /// The tick last stepped.
    last: Option<u64>,
    /// The targets of the frame before (the pins' start this frame), and
    /// a garment's skin normals.
    targets: Vec<[f32; 4]>,
    normals: Vec<[f32; 4]>,
    /// This frame's plan, written by `prepare`.
    plan: Plan,
    /// World bounds (min, max) of where it may be drawn this frame.
    pub(crate) bounds: (Vec3, Vec3),
}

impl VisualPipelines {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let entries: Vec<_> = (0..5)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: binding < 2,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("visual body"),
            entries: &entries,
        });
        let pipeline = |label: &'static str, source: String| {
            let device = device.clone();
            let layout = layout.clone();
            Deferred::new(move || {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(label),
                    layout: Some(
                        &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("visual body"),
                            bind_group_layouts: &[Some(&layout)],
                            immediate_size: 0,
                        }),
                    ),
                    module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some(label),
                        source: wgpu::ShaderSource::Wgsl(source.into()),
                    }),
                    entry_point: Some("solve"),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        VisualPipelines {
            surface: pipeline("visual body", surface_source()),
            garment: pipeline("visual garment", garment_source()),
            colliders: Self::collider_buffer(device, 4096),
            generation: 0,
            garment_lists: Default::default(),
            layout,
        }
    }

    fn collider_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("visual colliders"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Writes the frame's colliders for bodies with world boxes `bounds`;
    /// each body's list offset.
    pub(crate) fn write_colliders(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        colliders: &[SceneCollider],
        bounds: &[(Vec3, Vec3)],
    ) -> Vec<u32> {
        let (data, lists) = crate::scene_colliders::pack(colliders, bounds);
        let bytes: &[u8] = bytemuck::cast_slice(&data);
        if bytes.len() as u64 > self.colliders.size() {
            self.colliders =
                Self::collider_buffer(device, (bytes.len() as u64).next_power_of_two());
            self.generation += 1;
        }
        queue.write_buffer(&self.colliders, 0, bytes);
        lists
    }

    pub(crate) fn mesh(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        topology: Topology,
    ) -> VisualMesh {
        let n = u64::from(topology.points.max(1));
        let body = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("visual body"),
                contents: bytemuck::cast_slice(&topology.words),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            },
        );
        let state = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("visual body state"),
            size: n * 7 * 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let grid = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("visual body grid"),
            size: topology.grid_words() * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        VisualMesh {
            topology,
            body,
            state,
            grid,
            group: None,
            last: None,
            targets: Vec::new(),
            normals: Vec::new(),
            plan: plan(None, 0),
            bounds: (Vec3::ZERO, Vec3::ZERO),
        }
    }

    /// Plans `mesh`'s frame at `tick` and writes its header and targets
    /// (the rest shape where the entity is now: `targets`, with the skin's
    /// `normals` for a garment, empty for a surface), binding it to
    /// `output`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh: &mut VisualMesh,
        body: &VisualBody,
        targets: &[Vec3],
        normals: &[Vec3],
        tick: u64,
        dt: f32,
        list: u32,
        world: Mat4,
        output: &wgpu::Buffer,
    ) {
        let p = plan(mesh.last, tick);
        mesh.plan = p;
        mesh.last = Some(tick);
        let now: Vec<[f32; 4]> = targets.iter().map(|t| t.extend(0.0).to_array()).collect();
        if p.reset || mesh.targets.len() != now.len() {
            mesh.targets.clone_from(&now);
        }
        let n = now.len() as u64;
        let mut both = std::mem::take(&mut mesh.targets);
        both.extend_from_slice(&now);
        queue.write_buffer(&mesh.state, 2 * n * 16, bytemuck::cast_slice(&both));
        mesh.targets = now;
        if !normals.is_empty() {
            let now: Vec<[f32; 4]> = normals.iter().map(|v| v.extend(0.0).to_array()).collect();
            if p.reset || mesh.normals.len() != now.len() {
                mesh.normals.clone_from(&now);
            }
            let mut both = std::mem::take(&mut mesh.normals);
            both.extend_from_slice(&now);
            queue.write_buffer(&mesh.state, 5 * n * 16, bytemuck::cast_slice(&both));
            mesh.normals = now;
        }
        let mut header = mesh.topology.words[..h::LEN].to_vec();
        frame_header(&mut header, body, p, dt, list, world);
        queue.write_buffer(&mesh.body, 0, bytemuck::cast_slice(&header));
        if mesh
            .group
            .as_ref()
            .is_none_or(|(g, _)| *g != self.generation)
        {
            mesh.group = Some((self.generation, self.group(device, mesh, output)));
        }
    }

    fn group(
        &self,
        device: &wgpu::Device,
        mesh: &VisualMesh,
        output: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        let entries: Vec<_> = [&mesh.body, &self.colliders, &mesh.state, &mesh.grid, output]
            .into_iter()
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: b.as_entire_binding(),
            })
            .collect();
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("visual body"),
            layout: &self.layout,
            entries: &entries,
        })
    }

    /// Records `mesh`'s frame (nothing when it steps no tick).
    pub(crate) fn encode(&self, encoder: &mut crate::Encoder, mesh: &VisualMesh, garment: bool) {
        let Some((_, group)) = &mesh.group else {
            return;
        };
        if mesh.plan.steps == 0 && !mesh.plan.reset {
            return;
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("visual body"),
            timestamp_writes: None,
        });
        pass.set_pipeline(if garment {
            self.garment.get()
        } else {
            self.surface.get()
        });
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
}

impl VisualMesh {
    pub(crate) fn bytes(&self) -> u64 {
        self.body.size() + self.state.size() + self.grid.size()
    }
}

/// The world box of `points` grown by `margin`.
pub(crate) fn grown_bounds(points: &[Vec3], margin: f32) -> (Vec3, Vec3) {
    let (lo, hi) = points.iter().fold(
        (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
        |(lo, hi), p| (lo.min(*p), hi.max(*p)),
    );
    (lo - Vec3::splat(margin), hi + Vec3::splat(margin))
}

/// The coarse rest mesh's points and triangles.
pub(crate) fn rest_of(mesh: &MeshData) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    (
        mesh.vertices
            .iter()
            .map(|v| Vec3::from(v.position))
            .collect(),
        mesh.indices.as_chunks::<3>().0.to_vec(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_steps_the_ticks_since_the_last_and_starts_again_on_a_jump_or_going_back() {
        assert_eq!(
            plan(None, 100),
            Plan {
                reset: true,
                steps: PREROLL,
                first: 40
            }
        );
        assert_eq!(
            plan(Some(100), 100),
            Plan {
                reset: false,
                steps: 0,
                first: 101
            }
        );
        assert_eq!(
            plan(Some(100), 103),
            Plan {
                reset: false,
                steps: 3,
                first: 101
            }
        );
        assert_eq!(plan(Some(100), 108).steps, 8);
        assert!(plan(Some(100), 109).reset, "a jump past the catch-up");
        assert!(plan(Some(100), 99).reset, "a world that went back");
    }

    fn grid(columns: u32, rows: u32, spacing: f32) -> Vec<Vec3> {
        (0..rows)
            .flat_map(|y| {
                (0..columns).map(move |x| Vec3::new(x as f32, -(y as f32), 0.0) * spacing)
            })
            .collect()
    }

    #[test]
    fn colours_never_share_a_point_and_cover_every_edge() {
        let rest = grid(7, 5, 0.2);
        let cloth = Some(SimulationSurface::Cloth {
            columns: 7,
            rows: 5,
        });
        let body = VisualBody::default();
        let e = edges(cloth, &body, &rest, &[]);
        // Structural 6*5 + 7*4, shear 2*6*4, bending 5*5 + 7*3.
        assert_eq!(e.len(), 30 + 28 + 48 + 25 + 21);
        let (coloured, table) = colour(&e, rest.len());
        assert_eq!(coloured.len(), e.len());
        for &(first, count) in &table {
            let mut seen = std::collections::BTreeSet::new();
            for e in &coloured[first as usize..(first + count) as usize] {
                assert!(
                    seen.insert(e.a) && seen.insert(e.b),
                    "colour shares a point"
                );
            }
        }
        assert!(table.len() <= 16, "{} colours", table.len());
        let diag = e.iter().find(|e| e.a == 0 && e.b == 8).unwrap();
        assert!((diag.rest - 0.2 * std::f32::consts::SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn pinned_points_weigh_nothing_and_the_rest_are_tethered_to_the_nearest_pin() {
        let rest = grid(4, 3, 0.5);
        let cloth = Some(SimulationSurface::Cloth {
            columns: 4,
            rows: 3,
        });
        let body = VisualBody {
            pinned: Arc::from([0u32, 3]),
            ..VisualBody::default()
        };
        let t = Topology::build(cloth, &body, &rest, &[]);
        let w = &t.words;
        let inv = w[h::INV_MASS] as usize;
        let m: Vec<f32> = (0..12).map(|i| f32::from_bits(w[inv + i])).collect();
        assert_eq!(m[0], 0.0);
        assert_eq!(m[3], 0.0);
        assert!(m[1..3].iter().chain(&m[4..]).all(|&x| x == 1.0));
        let tether = w[h::TETHER] as usize;
        // Point 9 (x 1, y 2) is nearest pin 0: sqrt(0.5² + 1²).
        assert_eq!(w[tether + 9 * 2], 0);
        assert!((f32::from_bits(w[tether + 9 * 2 + 1]) - 1.25f32.sqrt()).abs() < 1e-6);
        // Point 10 (x 2) is nearest pin 3.
        assert_eq!(w[tether + 10 * 2], 3);
        assert_eq!(w[tether], u32::MAX, "a pin has no tether");
        assert!(t.matches(cloth, &body, &rest, &[]));
        let moved: Vec<Vec3> = rest
            .iter()
            .map(|p| *p + Vec3::new(3.0, 1.0, -2.0))
            .collect();
        assert!(
            t.matches(cloth, &body, &moved, &[]),
            "a moved rest shape keeps it"
        );
        let wider = grid(4, 3, 0.6);
        assert!(
            !t.matches(cloth, &body, &wider, &[]),
            "new rest lengths rebuild it"
        );
    }
}
