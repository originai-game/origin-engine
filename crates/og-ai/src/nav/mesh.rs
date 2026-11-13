//! The navmesh: convex polygons over the walkable surface of a level, in
//! tiles, built from a walkable heightfield (the cook voxelises a level's
//! static geometry into one, `crate::nav::bake`), queried for paths (A* over
//! polygons, then the funnel through their shared edges) and walked by
//! agents (`move_along`).
//!
//! The build is the voxel-and-contour pipeline, per tile of
//! `tile_size × tile_size` cells:
//!
//! 1. **Regions**: a row sweep gives every walkable span a region so each
//!    region is monotone along z (one run of cells per row), which makes
//!    its outline a simple polygon without holes.
//! 2. **Contours**: each region's outline on the cell lattice, every edge
//!    tagged with what lies across it (a wall, another region, another
//!    tile); vertices are kept where that changes, where the outline turns,
//!    and all along tile borders; wall runs are simplified (Douglas–Peucker,
//!    `max_error` cells) while the outline stays simple.
//! 3. **Polygons**: ear clipping in integer lattice coordinates, then
//!    triangles merged into convex polygons of at most [`MAX_VERTS`].
//! 4. **Links**: polygons sharing an edge (exactly, by lattice coordinates,
//!    at heights within the agent's climb) are neighbours, inside a tile and
//!    across tiles.
//!
//! A runtime obstacle (`set_obstacle`, `carve` for any [`Shape`]) blocks
//! the spans under it, grown by the agent radius, and rebuilds only the
//! tiles it touches. Areas (`set_area`, and the baked ones the mesh is
//! built with) mark spans with an area id (0 is ground): regions never mix
//! areas, so every polygon has one ([`Poly::area`]), and a path's [`Filter`]
//! weighs each area's length by its cost or keeps out of it. Off-mesh
//! links ([`Link`]: a jump, a ladder, a door, a teleport) join two points
//! the surface does not: A* crosses an open one at its cost, and a path
//! splits at each, its corners funnelled per stretch ([`MeshPath::links`]).
//! Everything is integer or f32 arithmetic in a fixed order: the same
//! heightfield builds the same polygons on every host.

mod build;
mod edit;
mod links;
mod path;
mod query;

pub use crate::nav::shape::Shape;
pub use links::{Link, LinkKind, PathLink};
use og_core::IdMap;
use og_core::glam::{Vec2, Vec3};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

/// The most vertices a polygon keeps after merging.
pub const MAX_VERTS: usize = 12;
/// No polygon / no link.
pub const NONE: u32 = u32::MAX;

/// How the walkable surface was measured and which agent it is for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavParams {
    /// Cell width on x and z.
    pub cell_size: f32,
    /// Cell height on y.
    pub cell_height: f32,
    /// The agent's height: less headroom than this is not walkable.
    pub agent_height: f32,
    /// The agent's radius: walls are kept this far away.
    pub agent_radius: f32,
    /// The highest step the agent climbs.
    pub agent_climb: f32,
    /// The steepest walkable slope, degrees.
    pub max_slope: f32,
    /// Cells per tile side.
    pub tile_size: u32,
    /// Wall simplification, in cells.
    pub max_error: f32,
}

impl Default for NavParams {
    fn default() -> Self {
        NavParams {
            cell_size: 0.25,
            cell_height: 0.1,
            agent_height: 1.8,
            agent_radius: 0.4,
            agent_climb: 0.4,
            max_slope: 45.0,
            tile_size: 32,
            max_error: 1.3,
        }
    }
}

impl NavParams {
    pub fn climb_cells(&self) -> u32 {
        (self.agent_climb / self.cell_height).floor().max(0.0) as u32
    }
    pub fn height_cells(&self) -> u32 {
        (self.agent_height / self.cell_height).ceil().max(1.0) as u32
    }
    pub fn radius_cells(&self) -> u32 {
        (self.agent_radius / self.cell_size).ceil().max(0.0) as u32
    }
}

/// One walkable floor in a column: its height and the headroom above it,
/// in cell heights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub floor: u32,
    pub clear: u32,
}

/// The walkable floors of a level, column by column (`width × depth`
/// columns of `cell_size`, the lowest corner at `origin`).
#[derive(Clone, Debug, PartialEq)]
pub struct Heightfield {
    pub params: NavParams,
    pub origin: Vec3,
    pub width: u32,
    pub depth: u32,
    /// Per column (x fastest): the first span index and the count.
    pub columns: Vec<[u32; 2]>,
    pub spans: Vec<Span>,
}

impl Heightfield {
    pub fn column(&self, x: i32, z: i32) -> std::ops::Range<usize> {
        if x < 0 || z < 0 || x >= self.width as i32 || z >= self.depth as i32 {
            return 0..0;
        }
        let [first, count] = self.columns[z as usize * self.width as usize + x as usize];
        first as usize..(first + count) as usize
    }

    /// Whether an agent can step between two floors of neighbouring columns.
    pub fn connected(&self, a: Span, b: Span) -> bool {
        let climb = self.params.climb_cells() as i64;
        let height = self.params.height_cells() as i64;
        let (af, bf) = (a.floor as i64, b.floor as i64);
        let top = (af + a.clear as i64).min(bf + b.clear as i64);
        (af - bf).abs() <= climb && top - af.max(bf) >= height
    }

    /// A flat floor of `width × depth` cells at height 0 with open sky:
    /// for tests and procedural levels.
    pub fn flat(params: NavParams, origin: Vec3, width: u32, depth: u32) -> Heightfield {
        let n = (width * depth) as usize;
        Heightfield {
            params,
            origin,
            width,
            depth,
            columns: (0..n as u32).map(|i| [i, 1]).collect(),
            spans: vec![
                Span {
                    floor: 0,
                    clear: u32::MAX / 4
                };
                n
            ],
        }
    }

    /// The same heightfield with the columns `blocked` answers true for
    /// emptied (walls carved into a flat floor, a procedural level).
    pub fn without(&self, mut blocked: impl FnMut(u32, u32) -> bool) -> Heightfield {
        let mut columns = Vec::with_capacity(self.columns.len());
        let mut spans = Vec::new();
        for z in 0..self.depth {
            for x in 0..self.width {
                let first = spans.len() as u32;
                if !blocked(x, z) {
                    for s in self.column(x as i32, z as i32) {
                        spans.push(self.spans[s]);
                    }
                }
                columns.push([first, spans.len() as u32 - first]);
            }
        }
        Heightfield {
            columns,
            spans,
            ..self.clone()
        }
    }

    /// World position of a lattice vertex at a floor height.
    pub fn world(&self, ix: i32, iz: i32, floor: u32) -> Vec3 {
        Vec3::new(
            self.origin.x + ix as f32 * self.params.cell_size,
            self.origin.y + floor as f32 * self.params.cell_height,
            self.origin.z + iz as f32 * self.params.cell_size,
        )
    }
}

/// A box that blocks the navmesh while it is set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Obstacle {
    pub min: Vec3,
    pub max: Vec3,
}

/// A convex polygon of a tile.
#[derive(Clone, Debug, PartialEq)]
pub struct Poly {
    /// Indices into the tile's vertices, counter-clockwise seen from +y.
    pub verts: Vec<u16>,
    /// Per edge `verts[i] → verts[i+1]`: the neighbour across it, or [`NONE`].
    pub links: Vec<u32>,
    pub region: u32,
    /// Its area (0: ground).
    pub area: u8,
}

/// One tile's polygons.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tile {
    pub verts: Vec<Vec3>,
    /// The lattice coordinates of each vertex (cells from the heightfield origin).
    pub lattice: Vec<(i32, i32)>,
    pub polys: Vec<Poly>,
}

/// A polygon reference: tile index × 65536 + polygon index.
pub type PolyRef = u32;

pub fn poly_ref(tile: usize, poly: usize) -> PolyRef {
    ((tile as u32) << 16) | poly as u32
}

pub fn split_ref(r: PolyRef) -> (usize, usize) {
    ((r >> 16) as usize, (r & 0xffff) as usize)
}

/// Which areas a path may cross and what each costs per metre (an
/// agent's `og.NavFilter`); the default is every area at 1.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filter {
    /// Per area id, its cost (past the list: 1). At least [`MIN_COST`].
    pub costs: Vec<f32>,
    /// Bit `a` set: area `a` (0 to 63) is never entered.
    pub excluded: u64,
}

/// The least area cost a filter takes.
pub const MIN_COST: f32 = 0.01;

impl Filter {
    /// The cost of a metre in area `a`.
    pub fn cost(&self, a: u8) -> f32 {
        self.costs
            .get(a as usize)
            .map_or(1.0, |&c| if c.is_finite() { c.max(MIN_COST) } else { 1.0 })
    }

    /// Whether area `a` is kept out of.
    pub fn excludes(&self, a: u8) -> bool {
        a < 64 && self.excluded & (1u64 << a) != 0
    }

    /// The cheapest cost of any area (1 when none is cheaper).
    fn least(&self) -> f32 {
        (0..self.costs.len()).fold(1.0f32, |m, a| m.min(self.cost(a as u8)))
    }
}

/// A path through the navmesh.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshPath {
    /// The corners, start and end included.
    pub points: Vec<Vec3>,
    /// The links it crosses, in order.
    pub links: Vec<PathLink>,
    /// The polygons crossed.
    pub polys: Vec<PolyRef>,
    /// Polygons taken off the open list.
    pub expanded: u32,
    /// False when the end was not reachable and the path stops at the
    /// reachable polygon nearest it.
    pub complete: bool,
}

/// The navmesh: a heightfield, its runtime obstacles and its tiles.
#[derive(Clone, Debug, PartialEq)]
pub struct NavMesh {
    pub field: Heightfield,
    pub tiles_x: u32,
    pub tiles_z: u32,
    pub tiles: Vec<Tile>,
    obstacles: IdMap<u32, Shape>,
    /// The areas it was built with (the cook's), in order.
    base_areas: Vec<(Shape, u8)>,
    /// The areas set at run time, by id, after the base ones.
    areas: IdMap<u32, (Shape, u8)>,
    /// Per span: blocked by an obstacle now.
    blocked: Vec<bool>,
    /// Per span: its area.
    span_area: Vec<u8>,
    /// The links it was built with, and its links now, by id.
    base_links: IdMap<u32, Link>,
    links: IdMap<u32, Link>,
    /// Per link, the polygons its start and end stand on (a link with an
    /// end off the mesh is left out), and per polygon the links leaving it.
    link_ends: IdMap<u32, (PolyRef, PolyRef)>,
    link_from: BTreeMap<PolyRef, Vec<(u32, bool)>>,
    /// Per span: its region in its tile (0: none).
    span_region: Vec<u32>,
    /// Per tile: each region's rows of cells.
    rows: Vec<RegionRows>,
    /// Bumped by every rebuild.
    version: u64,
}

/// Directed lattice edge → the polygon edges on it: (polygon, edge, mid height).
type EdgeMap = BTreeMap<((i32, i32), (i32, i32)), Vec<(PolyRef, usize, f32)>>;

/// A tile's regions: region → its rows, each `(z, [(x, span)])`.
type RegionRows = IdMap<u32, Vec<(i32, Vec<(i32, usize)>)>>;

fn cross(o: (i64, i64), a: (i64, i64), b: (i64, i64)) -> i64 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

impl NavMesh {
    /// Builds every tile of a heightfield.
    pub fn build(field: Heightfield) -> NavMesh {
        NavMesh::build_with_areas(field, Vec::new())
    }

    /// [`Self::build`] with areas marked (later entries over earlier ones).
    pub fn build_with_areas(field: Heightfield, areas: Vec<(Shape, u8)>) -> NavMesh {
        let mut mesh = NavMesh::empty(field, areas);
        for t in 0..mesh.tiles.len() {
            mesh.tiles[t] = mesh.build_tile(t);
        }
        mesh.link();
        mesh
    }

    /// A navmesh from tiles built earlier (the cook's), linked here.
    pub fn from_tiles(field: Heightfield, tiles: Vec<Tile>) -> Option<NavMesh> {
        NavMesh::from_tiles_with_areas(field, tiles, Vec::new())
    }

    /// [`Self::from_tiles`] for tiles built with `areas`.
    pub fn from_tiles_with_areas(
        field: Heightfield,
        tiles: Vec<Tile>,
        areas: Vec<(Shape, u8)>,
    ) -> Option<NavMesh> {
        let mut mesh = NavMesh::empty(field, areas);
        if tiles.len() != mesh.tiles.len() {
            return None;
        }
        mesh.tiles = tiles;
        mesh.link();
        mesh.resolve_links();
        Some(mesh)
    }

    /// The tiles' regions, no polygons yet.
    fn empty(field: Heightfield, base_areas: Vec<(Shape, u8)>) -> NavMesh {
        let ts = field.params.tile_size.max(4);
        let tiles_x = field.width.div_ceil(ts).max(1);
        let tiles_z = field.depth.div_ceil(ts).max(1);
        let n = (tiles_x * tiles_z) as usize;
        let spans = field.spans.len();
        let mut mesh = NavMesh {
            field,
            tiles_x,
            tiles_z,
            tiles: vec![Tile::default(); n],
            obstacles: IdMap::new(),
            base_areas,
            areas: IdMap::new(),
            blocked: vec![false; spans],
            span_area: Vec::new(),
            base_links: IdMap::new(),
            links: IdMap::new(),
            link_ends: IdMap::new(),
            link_from: BTreeMap::new(),
            span_region: vec![0; spans],
            rows: vec![RegionRows::new(); n],
            version: 1,
        };
        mesh.mark_areas();
        for t in 0..n {
            mesh.assign_regions(t);
        }
        mesh
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    /// Sets the edit count a restore found saved, so the paths planned on
    /// it before the save stay current.
    pub fn restore_version(&mut self, version: u64) {
        self.version = version;
    }

    pub fn poly_count(&self) -> usize {
        self.tiles.iter().map(|t| t.polys.len()).sum()
    }

    /// The box around every polygon's vertices (lowest and highest corners);
    /// a mesh without polygons answers its heightfield's floor rectangle at
    /// the origin's height.
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let mut lo = Vec3::splat(f32::MAX);
        let mut hi = Vec3::splat(f32::MIN);
        for tile in &self.tiles {
            for p in &tile.polys {
                for &v in &p.verts {
                    let v = tile.verts[v as usize];
                    lo = lo.min(v);
                    hi = hi.max(v);
                }
            }
        }
        if lo.x > hi.x {
            let f = &self.field;
            let size = Vec3::new(
                f.width as f32 * f.params.cell_size,
                0.0,
                f.depth as f32 * f.params.cell_size,
            );
            return (f.origin, f.origin + size);
        }
        (lo, hi)
    }

    /// The obstacles set now, by id.
    pub fn obstacles(&self) -> impl Iterator<Item = (u32, &Shape)> {
        self.obstacles.iter().map(|(k, v)| (*k, v))
    }

    /// The areas set at run time, by id (not the baked ones).
    pub fn areas(&self) -> impl Iterator<Item = (u32, &(Shape, u8))> {
        self.areas.iter().map(|(k, v)| (*k, v))
    }

    /// The areas the mesh was built with.
    pub fn base_areas(&self) -> &[(Shape, u8)] {
        &self.base_areas
    }

    /// Marks every span with the area of the last area shape over it: under
    /// its outline on x and z (the cell's centre), its floor from the
    /// agent's climb below the shape's bottom to its top.
    fn mark_areas(&mut self) {
        let p = self.field.params;
        let mut marks = vec![0u8; self.field.spans.len()];
        for (shape, area) in self.base_areas.iter().chain(self.areas.values()) {
            let (min, max) = shape.bounds();
            let (x0, z0, x1, z1) = self.cells_of(min, max);
            for z in z0..z1 {
                for x in x0..x1 {
                    let cx = self.field.origin.x + (x as f32 + 0.5) * p.cell_size;
                    let cz = self.field.origin.z + (z as f32 + 0.5) * p.cell_size;
                    if !shape.covers(cx, cz, 0.0) {
                        continue;
                    }
                    for s in self.field.column(x, z) {
                        let y =
                            self.field.origin.y + self.field.spans[s].floor as f32 * p.cell_height;
                        if y >= min.y - p.agent_climb && y <= max.y {
                            marks[s] = *area;
                        }
                    }
                }
            }
        }
        self.span_area = marks;
    }

    fn walkable(&self, s: usize) -> bool {
        !self.blocked[s]
    }

    /// The span of column (x, z) that `from` steps onto, if any.
    fn step(&self, from: Span, x: i32, z: i32) -> Option<usize> {
        self.field
            .column(x, z)
            .find(|&t| self.walkable(t) && self.field.connected(from, self.field.spans[t]))
    }

    fn tile_rect(&self, t: usize) -> (i32, i32, i32, i32) {
        let ts = self.field.params.tile_size.max(4) as i32;
        let (tx, tz) = (
            (t as u32 % self.tiles_x) as i32,
            (t as u32 / self.tiles_x) as i32,
        );
        let x0 = tx * ts;
        let z0 = tz * ts;
        (
            x0,
            z0,
            (x0 + ts).min(self.field.width as i32),
            (z0 + ts).min(self.field.depth as i32),
        )
    }
}
