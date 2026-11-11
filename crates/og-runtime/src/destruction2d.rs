//! og.destruction's 2D half: pixel terrain (og.PixelTerrain), the pieces
//! cut from it (og.PixelPiece) and og.PixelDetached, over
//! og_destruct::terrain2d.
//!
//! - **The state** is world state: per og.PixelTerrain its cells
//!   (og_destruct::terrain2d::PixelTerrain), its chunks' collider entities
//!   and meshes; per og.PixelPiece its cells; and the changes queued by
//!   carve2d, fill2d, explode2d and fracture2d, in the `og.destruction2d`
//!   state block (snapshots, restores, replays and the world hash carry
//!   it).
//! - **The system** runs after the game's systems and before og.physics:
//!   it fills a terrain the first tick it sees it, applies the queued
//!   changes in order, cuts loose what no longer stands
//!   (`PixelTerrain::detach`) into dynamic pieces, then rebuilds the
//!   collider of every chunk a change touched: its outline
//!   (`PixelTerrain::chunk_chains`), closed along the chunk's border and
//!   triangulated with og_geom's earcut, registered with physics under a
//!   generated handle ([`og_physics::GENERATED_MESH`], so a save carries
//!   the shape) and named by a static SHAPE_TRIMESH og.Collider, which
//!   collides as the outline in 2D. A piece collides as the convex hull of
//!   its cells (SHAPE_CONVEX). Meshes no collider names any more leave
//!   physics. Integer and f32 `+ - * /` arithmetic in fixed orders only.
//! - **Drawing** ([`extract`]): each terrain and piece is one sprite of a
//!   texture painted from its cells, repainted only when its cells changed
//!   (or a restore replaced them): presentation, never read by the tick.
//!
//! The records are lazy (`super::lazy`): a world that never names them has
//! neither the system nor the block, so its saves and hash are unchanged.

use super::codec::{Layout, RecordKind};
use super::engine::{Engine, Mount, engine, queued_structural, running};
use super::extract::{Hierarchy, RenderState};
use super::wire::{self, Carve2dDesc, Explosion2dDesc, Fill2dDesc, Fracture2dDesc};
use glam::{DVec2, Quat, Vec2, Vec3};
use og_core::{Error, ErrorKind};
use og_destruct::terrain2d::{BEDROCK, Cell, PixelTerrain};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const SYSTEM: &str = "og.destruction2d";
pub(crate) const TERRAIN: &str = "og.PixelTerrain";
pub(crate) const PIECE: &str = "og.PixelPiece";
pub(crate) const DETACHED: &str = "og.PixelDetached";
/// Registered together (`super::lazy`'s TOGETHER).
pub(crate) const RECORDS: [&str; 3] = [TERRAIN, PIECE, DETACHED];

const CAUSE_EXPLOSION: u32 = 2;
const CAUSE_FRACTURE: u32 = 5;
const CAUSE_CARVE: u32 = 6;
/// The cell fill2d and a terrain's first fill write.
const GROUND: Cell = 1;

// ── the state ───────────────────────────────────────────────────────────────

/// How a terrain's cells are drawn (copied into each piece cut from it).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct Look {
    cell: f32,
    color: [f32; 3],
    bedrock: [f32; 3],
    grain: f32,
    seed: u32,
    layer: i32,
}

/// A piece's collider settings, the terrain's when it was cut.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct Body {
    friction: f32,
    density: f32,
    layers: u32,
    mask: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Ground {
    grid: PixelTerrain,
    /// The width, height and chunk it was filled at: another starts it over.
    shape: [u32; 3],
    /// Where its colliders stand: its og.Transform's position.
    origin: [f32; 3],
    /// The bits of what its colliders were built with (cell size,
    /// tolerance, friction, layers, mask): another rebuilds them all.
    built: [u32; 5],
    look: Look,
    /// Per chunk with a collider: its entity and its mesh handle.
    colliders: BTreeMap<u32, (u64, u64)>,
    /// Counts changes to its cells (what drawing repaints on).
    revision: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Piece {
    terrain: u64,
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    /// Its cell (0, 0)'s place in the terrain it was cut from (its grain).
    grain: [i32; 2],
    look: Look,
    body: Body,
    mesh: u64,
    tick: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Op {
    Carve {
        entity: u64,
        point: [f32; 2],
        radius: f32,
    },
    Fill {
        entity: u64,
        point: [f32; 2],
        radius: f32,
        bedrock: bool,
    },
    Explode {
        point: [f32; 2],
        radius: f32,
        debris: f32,
        cells: u32,
        speed: f32,
    },
    Fracture {
        entity: u64,
        pieces: u32,
    },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct State {
    terrains: BTreeMap<u64, Ground>,
    pieces: BTreeMap<u64, Piece>,
    ops: Vec<Op>,
    /// The generated meshes the colliders name.
    meshes: BTreeSet<u64>,
}

/// An engine's state and how many times a restore replaced it (drawing's
/// cache is good for one epoch only; not world state).
#[derive(Default)]
struct Shared {
    state: State,
    epoch: u64,
}

/// og.destruction2d on one engine: the state its world's block saves,
/// shared with its system, its calls and extraction.
#[derive(Clone, Default)]
pub(crate) struct ModuleState(Arc<Mutex<Shared>>);

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}

// ── reading records ─────────────────────────────────────────────────────────

fn num(v: &Value, k: &str, d: f32) -> f32 {
    v[k].as_f64().map_or(d, |x| x as f32)
}

fn uint(v: &Value, k: &str, d: u32) -> u32 {
    v[k].as_f64().map_or(d, |x| x as u32)
}

fn rgb(v: &Value, d: [f32; 3]) -> [f32; 3] {
    [num(v, "r", d[0]), num(v, "g", d[1]), num(v, "b", d[2])]
}

fn v3(v: &Value) -> Vec3 {
    Vec3::new(num(v, "x", 0.0), num(v, "y", 0.0), num(v, "z", 0.0))
}

fn jv(p: Vec3) -> Value {
    json!({ "x": p.x, "y": p.y, "z": p.z })
}

/// An og.PixelTerrain as the tick reads it.
struct Settings {
    shape: [u32; 3],
    ground: f32,
    bedrock: u32,
    tolerance: f32,
    min_piece: u32,
    max_pieces: u32,
    look: Look,
    body: Body,
}

impl Settings {
    fn of(v: &Value) -> Settings {
        Settings {
            shape: [
                uint(v, "width", 256).clamp(1, 4096),
                uint(v, "height", 128).clamp(1, 4096),
                uint(v, "chunk", 32).clamp(4, 256),
            ],
            ground: num(v, "ground", 0.5).clamp(0.0, 1.0),
            bedrock: uint(v, "bedrock", 1),
            tolerance: num(v, "tolerance", 0.5).clamp(0.0, 4.0),
            min_piece: uint(v, "minPiece", 4),
            max_pieces: uint(v, "maxPieces", 64).max(1),
            look: Look {
                cell: num(v, "cellSize", 0.0625).max(0.001),
                color: rgb(&v["color"], [0.30, 0.18, 0.09]),
                bedrock: rgb(&v["bedrockColor"], [0.10, 0.10, 0.11]),
                grain: num(v, "grain", 0.15).clamp(0.0, 1.0),
                seed: uint(v, "seed", 0),
                layer: v["layer"].as_f64().map_or(0, |x| x as i32),
            },
            body: Body {
                friction: num(v, "friction", 0.6).max(0.0),
                density: num(v, "density", 1000.0).max(1.0),
                layers: uint(v, "layers", 1),
                mask: v["mask"].as_f64().map_or(u32::MAX, |x| x as u32),
            },
        }
    }

    fn built(&self) -> [u32; 5] {
        [
            self.look.cell.to_bits(),
            self.tolerance.to_bits(),
            self.body.friction.to_bits(),
            self.body.layers,
            self.body.mask,
        ]
    }

    /// A terrain filled as it starts: bedrock rows, then ground.
    fn filled(&self, origin: [f32; 3]) -> Ground {
        let [w, h, chunk] = self.shape;
        let rows = ((self.ground * h as f32).round() as u32).min(h);
        let rock = self.bedrock.min(h);
        let mut cells = vec![0; (w * h) as usize];
        for y in 0..h {
            let c = if y < rock {
                BEDROCK
            } else if y < rows {
                GROUND
            } else {
                continue;
            };
            cells[(y * w) as usize..((y + 1) * w) as usize].fill(c);
        }
        Ground {
            grid: PixelTerrain::from_cells(w, h, chunk, cells).expect("sized above"),
            shape: self.shape,
            origin,
            built: self.built(),
            look: self.look,
            colliders: BTreeMap::new(),
            revision: 0,
        }
    }
}

// ── geometry ────────────────────────────────────────────────────────────────

fn splitmix(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// A generated mesh's handle: FNV-1a over its geometry's 32-bit words with
/// [`og_physics::GENERATED_MESH`] set, so equal meshes share one and a
/// replay or a restore names the same handle.
fn mesh_handle(mesh: &og_physics::MeshDesc) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |x: u32| {
        h = (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01b3);
    };
    for c in mesh.positions.iter().flat_map(|p| p.to_array()) {
        eat(c.to_bits());
    }
    for &i in mesh.indices.iter().flatten() {
        eat(i);
    }
    og_physics::GENERATED_MESH | (h & (og_physics::GENERATED_MESH - 1))
}

/// A point on the border of the rectangle `r` (x0, y0, x1, y1), as its
/// distance clockwise from (x0, y0): up the left side, along the top, down
/// the right side, back along the bottom.
fn perimeter(r: [i64; 4], p: [i64; 2]) -> Option<i64> {
    let [x0, y0, x1, y1] = r;
    let (w, h) = (x1 - x0, y1 - y0);
    if p[0] == x0 && (y0..=y1).contains(&p[1]) {
        Some(p[1] - y0)
    } else if p[1] == y1 && (x0..=x1).contains(&p[0]) {
        Some(h + p[0] - x0)
    } else if p[0] == x1 && (y0..=y1).contains(&p[1]) {
        Some(h + w + y1 - p[1])
    } else if p[1] == y0 && (x0..=x1).contains(&p[0]) {
        Some(2 * h + w + x1 - p[0])
    } else {
        None
    }
}

/// Chunk `k`'s solid as closed loops in cell units: its outline's chains
/// (solid on their right), the open ones joined clockwise along the
/// chunk's border (the solid there is cut at the border), and the whole
/// chunk when solid ground runs on past every side with no outline across
/// it. Outer loops run clockwise, holes counter-clockwise.
fn chunk_loops(grid: &PixelTerrain, k: u32, tolerance: f32) -> Vec<Vec<[f32; 2]>> {
    let c = grid.chunk_size();
    let (cx, _) = grid.chunks();
    let (x0, y0) = ((k % cx) * c, (k / cx) * c);
    let (x1, y1) = ((x0 + c).min(grid.width()), (y0 + c).min(grid.height()));
    let r = [x0 as i64, y0 as i64, x1 as i64, y1 as i64];
    let corners = [
        (0, [x0, y0]),
        ((y1 - y0) as i64, [x0, y1]),
        ((y1 - y0 + x1 - x0) as i64, [x1, y1]),
        ((2 * (y1 - y0) + x1 - x0) as i64, [x1, y0]),
    ];
    let around = 2 * ((x1 - x0) + (y1 - y0)) as i64;
    let mut loops = Vec::new();
    let mut open: Vec<(Vec<[f32; 2]>, i64, i64)> = Vec::new();
    for chain in grid.chunk_chains(k, tolerance) {
        let n = chain.len();
        if n > 2 && chain[0] == chain[n - 1] {
            loops.push(chain[..n - 1].to_vec());
            continue;
        }
        let at = |p: [f32; 2]| perimeter(r, [p[0] as i64, p[1] as i64]);
        match (at(chain[0]), at(chain[n - 1])) {
            (Some(s), Some(e)) => open.push((chain, s, e)),
            // Not on the border (it cannot be): kept as it is.
            _ => loops.push(chain),
        }
    }
    let mut used = vec![false; open.len()];
    for first in 0..open.len() {
        if used[first] {
            continue;
        }
        let mut poly: Vec<[f32; 2]> = Vec::new();
        let mut cur = first;
        loop {
            used[cur] = true;
            poly.extend_from_slice(&open[cur].0);
            let end = open[cur].2;
            // The next chain starting clockwise from here (this one's own
            // start closes the loop).
            let next = (0..open.len())
                .filter(|&j| !used[j] || j == first)
                .min_by_key(|&j| ((open[j].1 - end).rem_euclid(around), j))
                .expect("the first chain is a candidate");
            let gap = (open[next].1 - end).rem_euclid(around);
            let mut passed: Vec<(i64, [u32; 2])> = corners
                .iter()
                .map(|&(t, p)| ((t - end).rem_euclid(around), p))
                .filter(|&(d, _)| d > 0 && d < gap)
                .collect();
            passed.sort();
            poly.extend(passed.iter().map(|(_, p)| [p[0] as f32, p[1] as f32]));
            if next == first {
                break;
            }
            cur = next;
        }
        poly.dedup();
        if poly.len() > 2 && poly[0] == poly[poly.len() - 1] {
            poly.pop();
        }
        loops.push(poly);
    }
    if open.is_empty() && runs_on(grid, r) {
        loops.push(vec![
            [x0 as f32, y0 as f32],
            [x0 as f32, y1 as f32],
            [x1 as f32, y1 as f32],
            [x1 as f32, y0 as f32],
        ]);
    }
    loops.retain(|l| l.len() > 2);
    loops
}

/// Whether solid ground in rectangle `r` runs on into a neighbouring
/// chunk's (a solid cell on its border with a solid one across it).
fn runs_on(grid: &PixelTerrain, r: [i64; 4]) -> bool {
    let [x0, y0, x1, y1] = r;
    let s = |x: i64, y: i64| grid.get(x, y) != 0;
    (y0..y1).any(|y| (s(x0, y) && s(x0 - 1, y)) || (s(x1 - 1, y) && s(x1, y)))
        || (x0..x1).any(|x| (s(x, y0) && s(x, y0 - 1)) || (s(x, y1 - 1) && s(x, y1)))
}

fn area(l: &[[f32; 2]]) -> f32 {
    og_destruct::terrain2d::polygon_area(l)
}

/// Whether `p` is inside loop `l` (even-odd).
fn inside(l: &[[f32; 2]], p: [f32; 2]) -> bool {
    let mut c = false;
    for i in 0..l.len() {
        let (a, b) = (l[i], l[(i + 1) % l.len()]);
        if (a[1] > p[1]) != (b[1] > p[1]) {
            let x = a[0] + (p[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1]);
            if p[0] < x {
                c = !c;
            }
        }
    }
    c
}

/// Chunk `k`'s collision mesh, in metres from the chunk's corner: its
/// loops triangulated (each outer with the holes inside it), so its
/// outline is the loops. None for a chunk with no solid.
fn chunk_mesh(
    grid: &PixelTerrain,
    k: u32,
    tolerance: f32,
    cell: f32,
) -> Option<og_physics::MeshDesc> {
    let loops = chunk_loops(grid, k, tolerance);
    let c = grid.chunk_size();
    let (cx, _) = grid.chunks();
    let corner = [((k % cx) * c) as f32, ((k / cx) * c) as f32];
    let (outers, holes): (Vec<_>, Vec<_>) = loops.into_iter().partition(|l| area(l) < 0.0);
    // Each hole to the smallest outer round it.
    let mut of: Vec<Vec<usize>> = vec![Vec::new(); outers.len()];
    for (h, hole) in holes.iter().enumerate() {
        // A point just inside the hole's first edge's left (the empty side).
        let (a, b) = (hole[0], hole[1]);
        let probe = [
            (a[0] + b[0]) * 0.5 - (b[1] - a[1]) * 0.25,
            (a[1] + b[1]) * 0.5 + (b[0] - a[0]) * 0.25,
        ];
        let best = (0..outers.len())
            .filter(|&o| inside(&outers[o], probe))
            .min_by(|&i, &j| {
                (-area(&outers[i]))
                    .total_cmp(&-area(&outers[j]))
                    .then(i.cmp(&j))
            });
        if let Some(o) = best {
            of[o].push(h);
        }
    }
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let d = |p: [f32; 2]| DVec2::new(p[0] as f64, p[1] as f64);
    for (o, outer) in outers.iter().enumerate() {
        let mut contour: Vec<DVec2> = outer.iter().map(|&p| d(p)).collect();
        let mut hs: Vec<Vec<DVec2>> = of[o]
            .iter()
            .map(|&h| holes[h].iter().map(|&p| d(p)).collect())
            .collect();
        let tris = og_geom::shape::triangulate_shape(&mut contour, &mut hs);
        let base = positions.len() as u32;
        for p in contour.iter().chain(hs.iter().flatten()) {
            positions.push(Vec3::new(
                (p.x as f32 - corner[0]) * cell,
                (p.y as f32 - corner[1]) * cell,
                0.0,
            ));
        }
        indices.extend(
            tris.iter()
                .map(|t| [base + t[0] as u32, base + t[1] as u32, base + t[2] as u32]),
        );
    }
    (!indices.is_empty()).then_some(og_physics::MeshDesc { positions, indices })
}

/// A piece's convex mesh: the corners of its cells' outline, in metres
/// from its cell (0, 0)'s corner.
fn piece_mesh(width: u32, height: u32, cells: &[Cell], cell: f32) -> og_physics::MeshDesc {
    let grid = PixelTerrain::from_cells(width, height, width.max(height), cells.to_vec())
        .expect("a piece's cells fill its rectangle");
    let mut positions: Vec<Vec3> = Vec::new();
    for chain in grid.chunk_chains(0, 0.0) {
        for p in chain {
            let v = Vec3::new(p[0] * cell, p[1] * cell, 0.0);
            if !positions.contains(&v) {
                positions.push(v);
            }
        }
    }
    og_physics::MeshDesc {
        positions,
        indices: Vec::new(),
    }
}

/// Solid cells cut from a grid: their cells row by row from `origin` (in
/// the grid's cells), `width` wide, 0 where empty.
struct Sub {
    origin: [i32; 2],
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    count: u32,
}

impl Sub {
    /// Its centre in its own cells.
    fn centre(&self) -> Vec2 {
        let (mut sx, mut sy) = (0.0f32, 0.0f32);
        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[(y * self.width + x) as usize] != 0 {
                    sx += x as f32 + 0.5;
                    sy += y as f32 + 0.5;
                }
            }
        }
        let n = self.count.max(1) as f32;
        Vec2::new(sx / n, sy / n)
    }
}

/// The 4-connected islands of a set of cells, each a [`Sub`], in the order
/// of their lowest cell (bottom row first).
fn components(cells: &[(i32, i32, Cell)]) -> Vec<Sub> {
    if cells.is_empty() {
        return Vec::new();
    }
    let (mut lx, mut ly, mut hx, mut hy) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for &(x, y, _) in cells {
        (lx, ly, hx, hy) = (lx.min(x), ly.min(y), hx.max(x), hy.max(y));
    }
    let (w, h) = ((hx - lx + 1) as usize, (hy - ly + 1) as usize);
    let mut grid = vec![0 as Cell; w * h];
    for &(x, y, c) in cells {
        grid[(y - ly) as usize * w + (x - lx) as usize] = c;
    }
    let mut seen = vec![false; w * h];
    let mut out = Vec::new();
    for start in 0..w * h {
        if grid[start] == 0 || seen[start] {
            continue;
        }
        seen[start] = true;
        let mut members = vec![start];
        let mut stack = vec![start];
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            let next = [
                (x > 0).then(|| i - 1),
                (x + 1 < w).then(|| i + 1),
                (y > 0).then(|| i - w),
                (y + 1 < h).then(|| i + w),
            ];
            for j in next.into_iter().flatten() {
                if grid[j] != 0 && !seen[j] {
                    seen[j] = true;
                    members.push(j);
                    stack.push(j);
                }
            }
        }
        let (mut ax, mut ay, mut bx, mut by) = (w, h, 0, 0);
        for &i in &members {
            let (x, y) = (i % w, i / w);
            (ax, ay, bx, by) = (ax.min(x), ay.min(y), bx.max(x), by.max(y));
        }
        let (sw, sh) = (bx - ax + 1, by - ay + 1);
        let mut sub = vec![0; sw * sh];
        for &i in &members {
            sub[(i / w - ay) * sw + (i % w - ax)] = grid[i];
        }
        out.push(Sub {
            origin: [lx + ax as i32, ly + ay as i32],
            width: sw as u32,
            height: sh as u32,
            cells: sub,
            count: members.len() as u32,
        });
    }
    out
}

/// Cells cut into the Voronoi regions of `n` seeds picked among them by
/// `seed` (each cell to its nearest, the earlier seed on a tie), each
/// region split into its 4-connected islands.
fn voronoi(cells: &[(i32, i32, Cell)], n: u32, seed: u64) -> Vec<Sub> {
    if cells.is_empty() {
        return Vec::new();
    }
    let n = (n as usize).clamp(1, cells.len());
    let mut seeds: Vec<(i64, i64)> = Vec::with_capacity(n);
    for i in 0..n as u64 {
        let c = cells[(splitmix(seed ^ i.wrapping_mul(0x9e37_79b9)) % cells.len() as u64) as usize];
        if !seeds.contains(&(c.0 as i64, c.1 as i64)) {
            seeds.push((c.0 as i64, c.1 as i64));
        }
    }
    let mut regions: Vec<Vec<(i32, i32, Cell)>> = vec![Vec::new(); seeds.len()];
    for &c in cells {
        let (x, y) = (c.0 as i64, c.1 as i64);
        let near = (0..seeds.len())
            .min_by_key(|&s| {
                let (dx, dy) = (x - seeds[s].0, y - seeds[s].1);
                (dx * dx + dy * dy, s)
            })
            .expect("one seed at least");
        regions[near].push(c);
    }
    regions.iter().flat_map(|r| components(r)).collect()
}

// ── the system ──────────────────────────────────────────────────────────────

struct Ls {
    terrain: (SchemaId, Layout),
    piece: (SchemaId, Layout),
    detached: (SchemaId, Layout),
    transform: (SchemaId, Layout),
    rigid: (SchemaId, Layout),
    velocity: (SchemaId, Layout),
    collider: (SchemaId, Layout),
}

const WRITES: [&str; 4] = ["og.Transform", "og.RigidBody", "og.Velocity", "og.Collider"];

/// Registers the `og.destruction2d` system and its state block once every
/// one of [`RECORDS`] is (super::lazy registers them one after another).
pub(crate) fn enable(world: &mut World, mount: Mount<'_>) -> og_core::Result<()> {
    if world.systems().contains(&SYSTEM) || RECORDS.iter().any(|n| world.id_by_name(n).is_err()) {
        return Ok(());
    }
    let layout = |n: &str, kind: RecordKind| -> og_core::Result<(SchemaId, Layout)> {
        let id = world.id_by_name(n)?;
        // The IDL's layout, with its defaults (a schema read back has none).
        let layout = og_api::host::RECORDS
            .iter()
            .find(|d| d.schema == n)
            .and_then(|d| Layout::from_decl(d).ok());
        let layout = match layout {
            Some(l) => l,
            None => Layout::from_schema(world.schema(id)?, kind),
        };
        Ok((id, layout))
    };
    use RecordKind::{Component as C, Event as E};
    let ls = Ls {
        terrain: layout(TERRAIN, C)?,
        piece: layout(PIECE, C)?,
        detached: layout(DETACHED, E)?,
        transform: layout("og.Transform", C)?,
        rigid: layout("og.RigidBody", C)?,
        velocity: layout("og.Velocity", C)?,
        collider: layout("og.Collider", C)?,
    };
    let sh = mount.modules.destruction2d.0.clone();
    lock(&sh).state = State::default();
    let state = sh.clone();
    let destruction = mount.modules.destruction.clone();
    let s = |v: &[&str]| v.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            reads: Vec::new(),
            writes: [s(&RECORDS), s(&WRITES)].concat(),
            queries: vec![QueryDesc {
                write: vec![TERRAIN.to_string()],
                ..QueryDesc::default()
            }],
            ..Default::default()
        },
        move |ctx| run(ctx, &destruction, &state, &ls),
    )?;
    if world.systems().contains(&"og.physics") {
        world.move_system_before(SYSTEM, "og.physics")?;
    }
    world.add_state_block(SYSTEM, Box::new(Block(sh)))
}

trait Enc {
    fn enc(&self, value: &Value) -> og_core::Result<Vec<u8>>;
}

impl Enc for Layout {
    fn enc(&self, value: &Value) -> og_core::Result<Vec<u8>> {
        self.encode(None, Some(value)).map_err(invalid)
    }
}

/// A piece about to be spawned.
struct Cut {
    terrain: u64,
    sub: Sub,
    /// Its cell (0, 0)'s grain place.
    grain: [i32; 2],
    look: Look,
    body: Body,
    /// Its cell (0, 0)'s corner and its rotation, in the world.
    pose: (Vec3, Quat),
    velocity: (Vec2, f32),
    cause: u32,
    min: u32,
}

/// The outward push of blasts (point, reach, speed) at a point.
fn pushed(blasts: &[([f32; 2], f32, f32)], at: Vec2) -> Vec2 {
    let mut v = Vec2::ZERO;
    for &(p, reach, speed) in blasts {
        let d = at - Vec2::from(p);
        let dist = d.length();
        if dist >= reach {
            continue;
        }
        let dir = if dist > 1e-6 { d / dist } else { Vec2::Y };
        v += dir * (speed * (1.0 - dist / reach));
    }
    v
}

fn quat_of(t: &Value) -> Quat {
    let q = &t["quaternion"];
    Quat::from_xyzw(
        num(q, "x", 0.0),
        num(q, "y", 0.0),
        num(q, "z", 0.0),
        num(q, "w", 1.0),
    )
    .normalize()
}

fn run(
    ctx: &mut SystemContext<'_>,
    destruction: &super::destruction::ModuleState,
    sh: &Mutex<Shared>,
    l: &Ls,
) -> og_core::Result<()> {
    let physics = super::destruction::physics(destruction);
    let tick = ctx.time().tick;
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let mut g = lock(sh);
    let st = &mut g.state;
    let mut gone: Vec<u64> = Vec::new();
    let mut settings: BTreeMap<u64, Settings> = BTreeMap::new();
    // Every terrain: filled the first tick it is seen (or when its shape
    // changes), its colliders rebuilt when what they are built from moves.
    for e in rows {
        let key = e.to_bits();
        let s = Settings::of(&l.terrain.1.decode(&ctx.get_bytes(e, l.terrain.0)?));
        let origin = ctx
            .get_bytes(e, l.transform.0)
            .map_or(Vec3::ZERO, |b| v3(&l.transform.1.decode(&b)["position"]))
            .to_array();
        if st.terrains.get(&key).is_none_or(|t| t.shape != s.shape) {
            if let Some(old) = st.terrains.remove(&key) {
                gone.extend(old.colliders.values().map(|c| c.0));
            }
            st.terrains.insert(key, s.filled(origin));
        }
        let t = st.terrains.get_mut(&key).expect("inserted above");
        t.look = s.look;
        if t.origin != origin || t.built != s.built() {
            t.origin = origin;
            t.built = s.built();
            let [w, h, chunk] = t.shape;
            t.grid = PixelTerrain::from_cells(w, h, chunk, t.grid.cells().to_vec())
                .expect("its own cells");
        }
        settings.insert(key, s);
    }
    let dropped: Vec<u64> = st
        .terrains
        .keys()
        .filter(|k| !settings.contains_key(k))
        .copied()
        .collect();
    for k in dropped {
        let t = st.terrains.remove(&k).expect("listed above");
        gone.extend(t.colliders.values().map(|c| c.0));
    }
    let before = st.pieces.len();
    st.pieces.retain(|k, _| ctx.is_alive(Entity::from_bits(*k)));
    let mut any = before != st.pieces.len() || !gone.is_empty();

    // The queued changes, in the order they were made.
    let ops = std::mem::take(&mut st.ops);
    let mut changed: BTreeSet<u64> = BTreeSet::new();
    let mut exploded: BTreeSet<u64> = BTreeSet::new();
    let mut blasts: Vec<([f32; 2], f32, f32)> = Vec::new();
    let mut cuts: Vec<Cut> = Vec::new();
    let to_cell = |t: &Ground, p: [f32; 2]| {
        (
            (p[0] - t.origin[0]) / t.look.cell,
            (p[1] - t.origin[1]) / t.look.cell,
        )
    };
    let corner_of = |t: &Ground, o: [i32; 2]| {
        Vec3::new(
            t.origin[0] + o[0] as f32 * t.look.cell,
            t.origin[1] + o[1] as f32 * t.look.cell,
            t.origin[2],
        )
    };
    for (i, op) in ops.iter().enumerate() {
        let op_seed = splitmix(tick ^ (i as u64) << 40);
        match *op {
            Op::Carve {
                entity,
                point,
                radius,
            }
            | Op::Fill {
                entity,
                point,
                radius,
                ..
            } => {
                let Some(t) = st.terrains.get_mut(&entity) else {
                    continue;
                };
                let value = match op {
                    Op::Fill { bedrock: true, .. } => BEDROCK,
                    Op::Fill { .. } => GROUND,
                    _ => 0,
                };
                let (x, y) = to_cell(t, point);
                if t.grid.circle(x, y, radius / t.look.cell, value) > 0 {
                    changed.insert(entity);
                }
            }
            Op::Explode {
                point,
                radius,
                debris,
                cells,
                speed,
            } => {
                let reach = radius + debris;
                blasts.push((point, reach.max(1e-3), speed));
                for (&k, t) in st.terrains.iter_mut() {
                    let s = &settings[&k];
                    let (x, y) = to_cell(t, point);
                    let (r, rd) = (radius / t.look.cell, reach / t.look.cell);
                    let carved = t.grid.circle(x, y, r, 0);
                    // The ring round the hole, cut into pieces.
                    let (r2, rd2) = (r * r, rd * rd);
                    let lo = |c: f32| (c - rd).floor().max(0.0) as u32;
                    let x1 = ((x + rd).ceil().max(0.0) as u32).min(t.grid.width());
                    let y1 = ((y + rd).ceil().max(0.0) as u32).min(t.grid.height());
                    let mut ring = Vec::new();
                    for cy in lo(y)..y1 {
                        for cx in lo(x)..x1 {
                            let (dx, dy) = (cx as f32 + 0.5 - x, cy as f32 + 0.5 - y);
                            let d2 = dx * dx + dy * dy;
                            let c = t.grid.get(cx as i64, cy as i64);
                            if d2 > r2 && d2 <= rd2 && c != 0 && c != BEDROCK {
                                ring.push((cx as i32, cy as i32, c));
                            }
                        }
                    }
                    if carved == 0 && ring.is_empty() {
                        continue;
                    }
                    changed.insert(k);
                    exploded.insert(k);
                    for &(cx, cy, _) in &ring {
                        t.grid.set(cx as u32, cy as u32, 0);
                    }
                    let n = (ring.len() as u32).div_ceil(cells.max(1));
                    let seed = op_seed ^ u64::from(t.look.seed) ^ k;
                    for sub in voronoi(&ring, n, seed) {
                        let at = corner_of(t, sub.origin).truncate() + sub.centre() * t.look.cell;
                        cuts.push(Cut {
                            terrain: k,
                            grain: sub.origin,
                            pose: (corner_of(t, sub.origin), Quat::IDENTITY),
                            velocity: (pushed(&blasts[blasts.len() - 1..], at), 0.0),
                            look: t.look,
                            body: s.body,
                            cause: CAUSE_EXPLOSION,
                            min: s.min_piece,
                            sub,
                        });
                    }
                }
                // The pieces already loose in reach.
                for (&k, p) in &st.pieces {
                    let e = Entity::from_bits(k);
                    let (Ok(tb), Ok(vb)) = (
                        ctx.get_bytes(e, l.transform.0),
                        ctx.get_bytes(e, l.velocity.0),
                    ) else {
                        continue;
                    };
                    let tr = l.transform.1.decode(&tb);
                    let half =
                        Vec3::new(p.width as f32, p.height as f32, 0.0) * (0.5 * p.look.cell);
                    let at = (v3(&tr["position"]) + quat_of(&tr) * half).truncate();
                    let push = pushed(&blasts[blasts.len() - 1..], at);
                    if push == Vec2::ZERO {
                        continue;
                    }
                    let mut vel = l.velocity.1.decode(&vb);
                    let lin = v3(&vel["linear"]) + push.extend(0.0);
                    vel["linear"] = jv(lin);
                    ctx.set_bytes(e, l.velocity.0, &l.velocity.1.enc(&vel)?)?;
                }
            }
            Op::Fracture { entity, pieces } => {
                if let Some(t) = st.terrains.get_mut(&entity) {
                    let s = &settings[&entity];
                    let mut all = Vec::new();
                    for y in 0..t.grid.height() {
                        for x in 0..t.grid.width() {
                            let c = t.grid.get(x as i64, y as i64);
                            if c != 0 && c != BEDROCK {
                                all.push((x as i32, y as i32, c));
                            }
                        }
                    }
                    for &(x, y, _) in &all {
                        t.grid.set(x as u32, y as u32, 0);
                    }
                    if !all.is_empty() {
                        changed.insert(entity);
                    }
                    let seed = op_seed ^ u64::from(t.look.seed) ^ entity;
                    for sub in voronoi(&all, pieces, seed) {
                        cuts.push(Cut {
                            terrain: entity,
                            grain: sub.origin,
                            pose: (corner_of(t, sub.origin), Quat::IDENTITY),
                            velocity: (Vec2::ZERO, 0.0),
                            look: t.look,
                            body: s.body,
                            cause: CAUSE_FRACTURE,
                            min: s.min_piece,
                            sub,
                        });
                    }
                } else if let Some(p) = st.pieces.get(&entity) {
                    let e = Entity::from_bits(entity);
                    let tr = l.transform.1.decode(&ctx.get_bytes(e, l.transform.0)?);
                    let (pos, rot) = (v3(&tr["position"]), quat_of(&tr));
                    let vel = ctx
                        .get_bytes(e, l.velocity.0)
                        .map(|b| l.velocity.1.decode(&b))
                        .unwrap_or(Value::Null);
                    let (lin, spin) = (v3(&vel["linear"]).truncate(), v3(&vel["angular"]).z);
                    let mut all = Vec::new();
                    for y in 0..p.height {
                        for x in 0..p.width {
                            let c = p.cells[(y * p.width + x) as usize];
                            if c != 0 {
                                all.push((x as i32, y as i32, c));
                            }
                        }
                    }
                    let cell = p.look.cell;
                    let whole = Vec3::new(p.width as f32, p.height as f32, 0.0) * (0.5 * cell);
                    let seed = op_seed ^ u64::from(p.look.seed) ^ entity;
                    let min = settings.get(&p.terrain).map_or(1, |s| s.min_piece);
                    for sub in voronoi(&all, pieces, seed) {
                        let local =
                            Vec3::new(sub.origin[0] as f32, sub.origin[1] as f32, 0.0) * cell;
                        let r = rot * (local + sub.centre().extend(0.0) * cell - whole);
                        // v + ω × r, in the plane.
                        let v = lin + Vec2::new(-r.y, r.x) * spin;
                        cuts.push(Cut {
                            terrain: p.terrain,
                            grain: [p.grain[0] + sub.origin[0], p.grain[1] + sub.origin[1]],
                            pose: (pos + rot * local, rot),
                            velocity: (v, spin),
                            look: p.look,
                            body: p.body,
                            cause: CAUSE_FRACTURE,
                            min,
                            sub,
                        });
                    }
                    st.pieces.remove(&entity);
                    ctx.despawn(e)?;
                    queued_structural();
                    any = true;
                }
            }
        }
    }

    // What no longer stands falls.
    for &k in &changed {
        let t = st.terrains.get_mut(&k).expect("changed terrains exist");
        let s = &settings[&k];
        for island in t.grid.detach() {
            let count = island.cells.iter().filter(|&&c| c != 0).count() as u32;
            let sub = Sub {
                origin: [island.origin[0] as i32, island.origin[1] as i32],
                width: island.width,
                height: island.height,
                cells: island.cells,
                count,
            };
            let at = corner_of(t, sub.origin).truncate() + sub.centre() * t.look.cell;
            cuts.push(Cut {
                terrain: k,
                grain: sub.origin,
                pose: (corner_of(t, sub.origin), Quat::IDENTITY),
                velocity: (pushed(&blasts, at), 0.0),
                look: t.look,
                body: s.body,
                cause: if exploded.contains(&k) {
                    CAUSE_EXPLOSION
                } else {
                    CAUSE_CARVE
                },
                min: s.min_piece,
                sub,
            });
        }
    }
    any |= !cuts.is_empty();
    for cut in cuts {
        let centre = cut.pose.0 + cut.pose.1 * (cut.sub.centre().extend(0.0) * cut.look.cell);
        let (terrain, count, cause) = (cut.terrain, cut.sub.count, cut.cause);
        let piece = if count < cut.min.max(1) {
            0
        } else {
            spawn_piece(ctx, l, physics.as_ref(), st, cut, tick)?
        };
        let ev = json!({
            "terrain": terrain as f64, "piece": piece as f64, "cells": count,
            "point": jv(centre), "cause": cause,
        });
        ctx.send_bytes(l.detached.0, &l.detached.1.enc(&ev)?)?;
    }

    // Each chunk a change touched collides as its new outline.
    for (&k, t) in st.terrains.iter_mut() {
        let dirty = t.grid.take_dirty();
        if dirty.is_empty() && !changed.contains(&k) {
            continue;
        }
        any = true;
        t.revision += 1;
        let s = &settings[&k];
        let c = t.grid.chunk_size();
        let (cx, _) = t.grid.chunks();
        for chunk in dirty {
            if let Some((e, _)) = t.colliders.remove(&chunk) {
                gone.push(e);
            }
            let Some(mesh) = chunk_mesh(&t.grid, chunk, s.tolerance, t.look.cell) else {
                continue;
            };
            let handle = mesh_handle(&mesh);
            if let Some(p) = &physics {
                p.set_mesh(handle, mesh)?;
            }
            let at = Vec3::new(
                t.origin[0] + ((chunk % cx) * c) as f32 * t.look.cell,
                t.origin[1] + ((chunk / cx) * c) as f32 * t.look.cell,
                t.origin[2],
            );
            let collider = json!({
                "shape": og_physics::SHAPE_TRIMESH, "mesh": handle as f64,
                "friction": s.body.friction, "layers": s.body.layers, "mask": s.body.mask,
            });
            let e = ctx.spawn_bytes(vec![
                (
                    l.transform.0,
                    l.transform.1.enc(&json!({ "position": jv(at) }))?,
                ),
                (l.collider.0, l.collider.1.enc(&collider)?),
            ])?;
            queued_structural();
            t.colliders.insert(chunk, (e.to_bits(), handle));
        }
        let solid = t.grid.cells().iter().filter(|&&c| c != 0).count() as u32;
        let e = Entity::from_bits(k);
        let mut v = l.terrain.1.decode(&ctx.get_bytes(e, l.terrain.0)?);
        if uint(&v, "solid", 0) != solid {
            v["solid"] = json!(solid);
            ctx.set_bytes(e, l.terrain.0, &l.terrain.1.enc(&v)?)?;
        }
    }

    // Past a terrain's maxPieces, its oldest pieces go.
    for (&k, s) in &settings {
        let mut mine: Vec<(u64, u64)> = st
            .pieces
            .iter()
            .filter(|(_, p)| p.terrain == k)
            .map(|(&e, p)| (p.tick, e))
            .collect();
        if mine.len() <= s.max_pieces as usize {
            continue;
        }
        mine.sort();
        for &(_, e) in &mine[..mine.len() - s.max_pieces as usize] {
            st.pieces.remove(&e);
            ctx.despawn(Entity::from_bits(e))?;
            queued_structural();
            any = true;
        }
    }
    for e in gone {
        let e = Entity::from_bits(e);
        if ctx.is_alive(e) {
            ctx.despawn(e)?;
            queued_structural();
        }
    }
    if any {
        // Generated meshes no collider names any more leave physics.
        let named: BTreeSet<u64> = st
            .terrains
            .values()
            .flat_map(|t| t.colliders.values().map(|c| c.1))
            .chain(st.pieces.values().map(|p| p.mesh))
            .collect();
        if let Some(p) = &physics {
            for unused in st.meshes.difference(&named) {
                p.remove_mesh(*unused);
            }
        }
        st.meshes = named;
    }
    Ok(())
}

/// Spawns a cut as an og.PixelPiece: a dynamic body whose collider is the
/// convex hull of its cells.
fn spawn_piece(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    physics: Option<&og_physics::PhysicsSystem>,
    st: &mut State,
    cut: Cut,
    tick: u64,
) -> og_core::Result<u64> {
    let Cut {
        terrain,
        sub,
        grain,
        look,
        body,
        pose: (position, rotation),
        velocity: (linear, angular),
        ..
    } = cut;
    let mesh = piece_mesh(sub.width, sub.height, &sub.cells, look.cell);
    let handle = mesh_handle(&mesh);
    if let Some(p) = physics {
        p.set_mesh(handle, mesh)?;
    }
    let q = json!({ "x": rotation.x, "y": rotation.y, "z": rotation.z, "w": rotation.w });
    let collider = json!({
        "shape": og_physics::SHAPE_CONVEX, "mesh": handle as f64,
        "friction": body.friction, "density": body.density,
        "layers": body.layers, "mask": body.mask,
    });
    let velocity = json!({
        "linear": jv(linear.extend(0.0)),
        "angular": { "x": 0.0, "y": 0.0, "z": angular },
    });
    let piece = json!({
        "terrain": terrain as f64, "width": sub.width, "height": sub.height,
        "cells": sub.count, "tick": tick.min(u64::from(u32::MAX)) as u32,
    });
    let e = ctx.spawn_bytes(vec![
        (
            l.transform.0,
            l.transform
                .1
                .enc(&json!({ "position": jv(position), "quaternion": q }))?,
        ),
        (l.rigid.0, l.rigid.1.enc(&json!({ "kind": "dynamic" }))?),
        (l.velocity.0, l.velocity.1.enc(&velocity)?),
        (l.collider.0, l.collider.1.enc(&collider)?),
        (l.piece.0, l.piece.1.enc(&piece)?),
    ])?;
    queued_structural();
    st.pieces.insert(
        e.to_bits(),
        Piece {
            terrain,
            width: sub.width,
            height: sub.height,
            cells: sub.cells,
            grain,
            look,
            body,
            mesh: handle,
            tick,
        },
    );
    Ok(e.to_bits())
}

// ── the state block ─────────────────────────────────────────────────────────

struct Block(Arc<Mutex<Shared>>);

const MAGIC: &[u8; 8] = b"DESTR2D1";

impl StateBlock for Block {
    fn save(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(MAGIC);
        out.extend(
            bincode::serialize(&lock(&self.0).state)
                .expect("the og.destruction2d state serialises"),
        );
    }
    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let rest = bytes
            .strip_prefix(MAGIC)
            .ok_or_else(|| invalid("not an og.destruction2d state"))?;
        let s: State = bincode::deserialize(rest).map_err(|e| invalid(e.to_string()))?;
        Ok(Box::new(s))
    }
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut g = lock(&self.0);
        g.state = decoded
            .and_then(|d| d.downcast::<State>().ok())
            .map(|b| *b)
            .unwrap_or_default();
        g.epoch += 1;
    }
    /// A terrain's origin is its root entity's og.Transform.position (its
    /// colliders, roots too, move with the shift); queued points are in
    /// the world.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut g = lock(&self.0);
        let st = &mut g.state;
        for (&k, t) in st.terrains.iter_mut() {
            t.origin = s.moved_of(k, Vec3::from(t.origin)).to_array();
        }
        let m = |p: &mut [f32; 2]| {
            let v = (Vec3::new(p[0], p[1], 0.0).as_dvec3() + s.d).as_vec3();
            *p = [v.x, v.y];
        };
        for op in &mut st.ops {
            match op {
                Op::Carve { point, .. } | Op::Fill { point, .. } | Op::Explode { point, .. } => {
                    m(point)
                }
                Op::Fracture { .. } => {}
            }
        }
    }
}

// ── drawing ─────────────────────────────────────────────────────────────────

/// Extraction's textures of the terrains and pieces, by entity: the
/// texture and the (epoch, revision) its pixels were painted at.
#[derive(Default)]
pub(crate) struct PixelRender {
    textures: BTreeMap<u64, (og_render::TextureHandle, u64, u64)>,
}

fn srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * og_core::math::pow(c, 1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// A grid's pixels, top row first: ground in its colour with a fixed grain
/// per cell (by its place in the terrain), bedrock, empty clear.
fn paint(width: u32, height: u32, cells: &[Cell], grain: [i32; 2], look: &Look) -> Vec<u8> {
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    let rock = look.bedrock.map(srgb);
    for y in 0..height {
        let row = (height - 1 - y) * width;
        for x in 0..width {
            let c = cells[(y * width + x) as usize];
            let px = &mut rgba[((row + x) * 4) as usize..((row + x) * 4 + 4) as usize];
            match c {
                0 => {}
                BEDROCK => px.copy_from_slice(&[rock[0], rock[1], rock[2], 255]),
                _ => {
                    let (gx, gy) = (grain[0] + x as i32, grain[1] + y as i32);
                    let h = splitmix(
                        u64::from(look.seed) << 40 ^ (gx as u32 as u64) << 20 ^ gy as u32 as u64,
                    );
                    let n = (h >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0;
                    let k = 1.0 + look.grain * n;
                    let [r, g, b] = look.color.map(|v| srgb(v * k));
                    px.copy_from_slice(&[r, g, b, 255]);
                }
            }
        }
    }
    rgba
}

/// A terrain or a piece as drawn: a sprite of its cells.
struct Drawable<'a> {
    entity: u64,
    size: [u32; 2],
    cells: &'a [Cell],
    /// Its cell (0, 0)'s place in the terrain (its grain).
    grain: [i32; 2],
    look: &'a Look,
    /// Its cells' revision (a piece's never change).
    revision: u64,
    /// Whether it turns with its og.Transform (a piece does).
    turns: bool,
}

/// Draws every terrain and piece as a sprite of its cells.
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let Ok(sh) = engine(rs.engine).map(|e| e.modules.destruction2d.0.clone()) else {
        return;
    };
    let g = lock(&sh);
    let epoch = g.epoch;
    let mut drawn: Vec<Drawable> = Vec::new();
    for (&k, t) in &g.state.terrains {
        drawn.push(Drawable {
            entity: k,
            size: [t.grid.width(), t.grid.height()],
            cells: t.grid.cells(),
            grain: [0, 0],
            look: &t.look,
            revision: t.revision,
            turns: false,
        });
    }
    for (&k, p) in &g.state.pieces {
        drawn.push(Drawable {
            entity: k,
            size: [p.width, p.height],
            cells: &p.cells,
            grain: p.grain,
            look: &p.look,
            revision: 0,
            turns: true,
        });
    }
    let mut live = BTreeSet::new();
    for d in drawn {
        let Drawable {
            entity: k,
            size: [w, ht],
            cells,
            grain,
            look,
            revision,
            turns,
        } = d;
        if !world.is_alive(Entity::from_bits(k)) {
            continue;
        }
        live.insert(k);
        let painted = rs.pixel.textures.get(&k).copied();
        let texture = match painted {
            Some((t, e, r)) if e == epoch && r == revision => t,
            _ => {
                let data = og_render::TextureData {
                    width: w,
                    height: ht,
                    rgba: paint(w, ht, cells, grain, look),
                    color_space: og_render::ColorSpace::Srgb,
                    mips: og_render::Mips::None,
                };
                let t = match painted {
                    Some((t, ..)) => {
                        rs.scene.set_texture(t, data);
                        t
                    }
                    None => {
                        let t = rs.scene.add_texture(data);
                        rs.scene
                            .set_texture_filter(t, og_render::TextureFilter::Nearest);
                        t
                    }
                };
                rs.pixel.textures.insert(k, (t, epoch, revision));
                t
            }
        };
        let m = h.world_of(k);
        let xa = m.x_axis.truncate();
        rs.scene.sprite.sprites.push(og_render::Sprite {
            texture: Some(texture),
            position: m.w_axis.truncate(),
            size: Vec2::new(w as f32, ht as f32) * look.cell,
            anchor: Vec2::ZERO,
            rotation: if turns {
                og_core::math::atan2(xa.y, xa.x)
            } else {
                0.0
            },
            layer: look.layer,
            ..Default::default()
        });
    }
    drop(g);
    let stale: Vec<u64> = rs
        .pixel
        .textures
        .keys()
        .filter(|k| !live.contains(k))
        .copied()
        .collect();
    for k in stale {
        if let Some((t, ..)) = rs.pixel.textures.remove(&k) {
            rs.scene.remove_texture(t);
        }
    }
}

// ── the calls ───────────────────────────────────────────────────────────────

/// Whether `e` is alive with record `name`: between ticks from the world, inside a
/// system through its context, which reads only what the system declared
/// (the calls are queued for og.destruction2d after the game's systems, so
/// a system may make them: a hit frame carves, a blast in the tick).
fn has(eng: &Engine, name: &str, e: Entity) -> Result<bool, String> {
    if let Some(f) = running(eng.id) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { f.ctx() };
        if !ctx.is_alive(e) {
            return Err(format!("entity {} is not alive", e.to_bits()));
        }
        let Ok(id) = ctx.id_by_name(name) else {
            return Ok(false);
        };
        return match ctx.get_bytes(e, id) {
            Ok(_) => Ok(true),
            Err(err) if err.kind() == ErrorKind::Access => Err(format!(
                "{err}: inside a system, declare {name} among its reads"
            )),
            Err(_) => Ok(false),
        };
    }
    let w = eng.world()?;
    if !w.is_alive(e) {
        return Err(format!("entity {} is not alive", e.to_bits()));
    }
    Ok(w.id_by_name(name)
        .is_ok_and(|id| w.get_bytes(e, id).is_ok()))
}

fn terrain_of(eng: &Engine, entity: f64) -> Result<u64, String> {
    let e = Entity::from_bits(entity as u64);
    if has(eng, TERRAIN, e)? {
        Ok(e.to_bits())
    } else {
        Err(format!("entity {} has no og.PixelTerrain", e.to_bits()))
    }
}

fn point2(p: Vec3) -> Result<[f32; 2], String> {
    if p.is_finite() {
        Ok([p.x, p.y])
    } else {
        Err(format!("the point {p} is not finite"))
    }
}

fn size(r: f32, key: &str) -> Result<f32, String> {
    if r.is_finite() && r >= 0.0 {
        Ok(r)
    } else {
        Err(format!(
            "desc.{key} {r} is not a finite, non-negative number"
        ))
    }
}

fn queue(eng: &Engine, op: Op) {
    lock(&eng.modules.destruction2d.0).state.ops.push(op);
}

pub fn og_destruction_carve2d(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let entity = terrain_of(&eng, entity)?;
        let d: Carve2dDesc = wire::arg(desc.as_deref(), "desc")?;
        queue(
            &eng,
            Op::Carve {
                entity,
                point: point2(d.point)?,
                radius: size(d.radius, "radius")?,
            },
        );
        Ok(())
    })()
}

pub fn og_destruction_fill2d(world: u32, entity: f64, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let entity = terrain_of(&eng, entity)?;
        let d: Fill2dDesc = wire::arg(desc.as_deref(), "desc")?;
        queue(
            &eng,
            Op::Fill {
                entity,
                point: point2(d.point)?,
                radius: size(d.radius, "radius")?,
                bedrock: d.bedrock,
            },
        );
        Ok(())
    })()
}

pub fn og_destruction_explode2d(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let d: Explosion2dDesc = wire::arg(desc.as_deref(), "desc")?;
        let op = Op::Explode {
            point: point2(d.point)?,
            radius: size(d.radius, "radius")?,
            debris: size(d.debris, "debris")?,
            cells: d.piece_cells.max(4),
            speed: size(d.speed, "speed")?,
        };
        // A world with no pixel terrain (its records, registered with
        // og.destruction2d, unknown) has nothing to blow up. Inside a
        // system the system's context answers, as the world cannot.
        let registered = match running(eng.id) {
            // SAFETY: the frame is published while its system runs.
            Some(f) => unsafe { f.ctx() }.id_by_name(TERRAIN).is_ok(),
            None => eng.world()?.systems().contains(&SYSTEM),
        };
        if registered {
            queue(&eng, op);
        }
        Ok(())
    })()
}

pub fn og_destruction_fracture2d(
    world: u32,
    entity: f64,
    desc: Option<String>,
) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        if !has(&eng, TERRAIN, e)? && !has(&eng, PIECE, e)? {
            return Err(format!(
                "entity {} has no og.PixelPiece or og.PixelTerrain",
                e.to_bits()
            ));
        }
        let d: Fracture2dDesc = wire::arg(desc.as_deref(), "desc")?;
        queue(
            &eng,
            Op::Fracture {
                entity: e.to_bits(),
                pieces: d.pieces.clamp(1, 256),
            },
        );
        Ok(())
    })()
}

pub fn og_destruction_cell2d(
    world: u32,
    entity: f64,
    point: String,
) -> Result<serde_json::Value, String> {
    let r = (|| -> Result<u32, String> {
        let eng = engine(world)?;
        let key = terrain_of(&eng, entity)?;
        let p = point2(wire::arg(Some(&point), "point")?)?;
        let g = lock(&eng.modules.destruction2d.0);
        let t = g.state.terrains.get(&key).ok_or_else(|| {
            format!("entity {key}'s og.PixelTerrain is not filled yet (the tick after it is added)")
        })?;
        let (x, y) = (
            ((p[0] - t.origin[0]) / t.look.cell).floor(),
            ((p[1] - t.origin[1]) / t.look.cell).floor(),
        );
        Ok(u32::from(t.grid.get(x as i64, y as i64)))
    })();
    match r {
        Ok(v) => Ok(json!(v)),
        Err(e) => Err(e),
    }
}
