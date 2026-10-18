//! The navmesh: convex polygons over the walkable surface of a level, in
//! tiles, built from a walkable heightfield (the cook voxelises a level's
//! static geometry into one, `crate::bake`), queried for paths (A* over
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
//! A runtime obstacle (`set_obstacle`) blocks the spans under its box,
//! grown by the agent radius, and rebuilds only the tiles it touches.
//! Everything is integer or f32 arithmetic in a fixed order: the same
//! heightfield builds the same polygons on every host.

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

/// A path through the navmesh.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshPath {
    /// The corners, start and end included.
    pub points: Vec<Vec3>,
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
    obstacles: BTreeMap<u32, Obstacle>,
    /// Per span: blocked by an obstacle now.
    blocked: Vec<bool>,
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
type RegionRows = BTreeMap<u32, Vec<(i32, Vec<(i32, usize)>)>>;

fn cross(o: (i64, i64), a: (i64, i64), b: (i64, i64)) -> i64 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

/// What lies across a contour edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Across {
    Wall,
    /// A region of a tile (this one or a neighbour).
    Region(u32, u32),
}

impl NavMesh {
    /// Builds every tile of a heightfield.
    pub fn build(field: Heightfield) -> NavMesh {
        let mut mesh = NavMesh::empty(field);
        for t in 0..mesh.tiles.len() {
            mesh.tiles[t] = mesh.build_tile(t);
        }
        mesh.link();
        mesh
    }

    /// A navmesh from tiles built earlier (the cook's), linked here.
    pub fn from_tiles(field: Heightfield, tiles: Vec<Tile>) -> Option<NavMesh> {
        let mut mesh = NavMesh::empty(field);
        if tiles.len() != mesh.tiles.len() {
            return None;
        }
        mesh.tiles = tiles;
        mesh.link();
        Some(mesh)
    }

    /// The tiles' regions, no polygons yet.
    fn empty(field: Heightfield) -> NavMesh {
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
            obstacles: BTreeMap::new(),
            blocked: vec![false; spans],
            span_region: vec![0; spans],
            rows: vec![RegionRows::new(); n],
            version: 1,
        };
        for t in 0..n {
            mesh.assign_regions(t);
        }
        mesh
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn poly_count(&self) -> usize {
        self.tiles.iter().map(|t| t.polys.len()).sum()
    }

    pub fn obstacles(&self) -> impl Iterator<Item = (u32, &Obstacle)> {
        self.obstacles.iter().map(|(k, v)| (*k, v))
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

    /// Sets, moves (`Some` with an id in use) or removes (`None`) a box
    /// obstacle, and rebuilds the tiles under its old and new boxes. The
    /// tiles rebuilt, in order.
    pub fn set_obstacle(&mut self, id: u32, obstacle: Option<Obstacle>) -> Vec<usize> {
        let old = match obstacle {
            Some(o) => self.obstacles.insert(id, o),
            None => self.obstacles.remove(&id),
        };
        let mut boxes: Vec<Obstacle> = old.into_iter().collect();
        boxes.extend(obstacle);
        self.rebuild_boxes(&boxes)
    }

    /// Rebuilds the tiles that overlap `min..max` (after a change to the
    /// level there): the region rebuild an editor or a procedural level calls.
    pub fn rebuild_region(&mut self, min: Vec3, max: Vec3) -> Vec<usize> {
        self.rebuild_boxes(&[Obstacle { min, max }])
    }

    fn rebuild_boxes(&mut self, boxes: &[Obstacle]) -> Vec<usize> {
        let p = self.field.params;
        let r = p.agent_radius;
        // Spans under every obstacle (grown by the radius), from scratch.
        let mut blocked = vec![false; self.field.spans.len()];
        for o in self.obstacles.values() {
            let (x0, z0, x1, z1) =
                self.cells_of(o.min - Vec3::new(r, 0.0, r), o.max + Vec3::new(r, 0.0, r));
            for z in z0..z1 {
                for x in x0..x1 {
                    for s in self.field.column(x, z) {
                        let sp = self.field.spans[s];
                        let y = self.field.origin.y + sp.floor as f32 * p.cell_height;
                        if y + p.agent_height >= o.min.y && y <= o.max.y + p.agent_climb {
                            blocked[s] = true;
                        }
                    }
                }
            }
        }
        self.blocked = blocked;
        let ts = p.tile_size.max(4) as i32;
        let mut tiles = Vec::new();
        for b in boxes {
            let (x0, z0, x1, z1) =
                self.cells_of(b.min - Vec3::new(r, 0.0, r), b.max + Vec3::new(r, 0.0, r));
            // One cell of margin: a neighbour tile's border cells see across.
            let (tx0, tz0) = (((x0 - 1).max(0)) / ts, ((z0 - 1).max(0)) / ts);
            let (tx1, tz1) = (
                ((x1).min(self.field.width as i32 - 1)) / ts,
                ((z1).min(self.field.depth as i32 - 1)) / ts,
            );
            for tz in tz0..=tz1 {
                for tx in tx0..=tx1 {
                    let t = (tz as u32 * self.tiles_x + tx as u32) as usize;
                    if t < self.tiles.len() && !tiles.contains(&t) {
                        tiles.push(t);
                    }
                }
            }
        }
        tiles.sort_unstable();
        for &t in &tiles {
            self.assign_regions(t);
        }
        // A neighbour's border vertices follow these tiles' regions: rebuild it too.
        let mut rebuild = tiles.clone();
        for &t in &tiles {
            let (tx, tz) = (
                (t as u32 % self.tiles_x) as i32,
                (t as u32 / self.tiles_x) as i32,
            );
            for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let (nx, nz) = (tx + dx, tz + dz);
                if nx >= 0 && nz >= 0 && nx < self.tiles_x as i32 && nz < self.tiles_z as i32 {
                    let n = (nz as u32 * self.tiles_x + nx as u32) as usize;
                    if !rebuild.contains(&n) {
                        rebuild.push(n);
                    }
                }
            }
        }
        rebuild.sort_unstable();
        for &t in &rebuild {
            self.tiles[t] = self.build_tile(t);
        }
        self.link();
        self.version += 1;
        rebuild
    }

    /// The cell rectangle `[x0, x1) × [z0, z1)` a world box covers, clamped.
    fn cells_of(&self, min: Vec3, max: Vec3) -> (i32, i32, i32, i32) {
        let cs = self.field.params.cell_size;
        let o = self.field.origin;
        let x0 = (((min.x - o.x) / cs).floor() as i32).clamp(0, self.field.width as i32);
        let z0 = (((min.z - o.z) / cs).floor() as i32).clamp(0, self.field.depth as i32);
        let x1 = (((max.x - o.x) / cs).ceil() as i32).clamp(0, self.field.width as i32);
        let z1 = (((max.z - o.z) / cs).ceil() as i32).clamp(0, self.field.depth as i32);
        (x0, z0, x1, z1)
    }

    // ── build ────────────────────────────────────────────────────────────

    /// The tile a cell is in.
    fn tile_of(&self, x: i32, z: i32) -> usize {
        let ts = self.field.params.tile_size.max(4) as i32;
        ((z / ts) as u32 * self.tiles_x + (x / ts) as u32) as usize
    }

    /// Monotone regions of one tile: a row sweep in which a run of
    /// connected spans takes the region below it when it is the only run
    /// touching that region, so every region is one run per row.
    fn assign_regions(&mut self, t: usize) {
        let (x0, z0, x1, z1) = self.tile_rect(t);
        let mut region: BTreeMap<usize, u32> = BTreeMap::new();
        let mut next_region = 1u32;
        let mut rows = RegionRows::new();
        for z in z0..z1 {
            struct Sweep {
                nei: u32,
                ns: u32,
                cells: Vec<(i32, usize)>,
            }
            const MIXED: u32 = u32::MAX;
            let mut sweeps: Vec<Sweep> = Vec::new();
            let mut run_of: BTreeMap<usize, usize> = BTreeMap::new();
            let mut prev: BTreeMap<u32, u32> = BTreeMap::new();
            for x in x0..x1 {
                for s in self.field.column(x, z) {
                    if !self.walkable(s) {
                        continue;
                    }
                    let sp = self.field.spans[s];
                    let left = if x > x0 {
                        self.step(sp, x - 1, z)
                            .and_then(|l| run_of.get(&l).copied())
                    } else {
                        None
                    };
                    let run = match left {
                        Some(r) => r,
                        None => {
                            sweeps.push(Sweep {
                                nei: 0,
                                ns: 0,
                                cells: Vec::new(),
                            });
                            sweeps.len() - 1
                        }
                    };
                    if z > z0
                        && let Some(d) = self.step(sp, x, z - 1)
                        && let Some(&nr) = region.get(&d)
                    {
                        let sw = &mut sweeps[run];
                        if sw.nei == 0 || sw.nei == nr {
                            sw.nei = nr;
                            sw.ns += 1;
                            *prev.entry(nr).or_insert(0) += 1;
                        } else {
                            sw.nei = MIXED;
                        }
                    }
                    run_of.insert(s, run);
                    sweeps[run].cells.push((x, s));
                }
            }
            for sw in &mut sweeps {
                let id = if sw.nei != 0
                    && sw.nei != MIXED
                    && prev.get(&sw.nei).copied() == Some(sw.ns)
                {
                    sw.nei
                } else {
                    next_region += 1;
                    next_region - 1
                };
                for &(_, s) in &sw.cells {
                    region.insert(s, id);
                }
                rows.entry(id)
                    .or_default()
                    .push((z, std::mem::take(&mut sw.cells)));
            }
        }
        // Clear the tile's old regions, then set the new ones.
        for z in z0..z1 {
            for x in x0..x1 {
                for s in self.field.column(x, z) {
                    self.span_region[s] = 0;
                }
            }
        }
        for (s, r) in region {
            self.span_region[s] = r;
        }
        self.rows[t] = rows;
    }

    /// Polygonises one tile from its regions.
    pub fn build_tile(&self, t: usize) -> Tile {
        let (x0, z0, x1, z1) = self.tile_rect(t);
        let f = &self.field;
        // 2.–3. Contours, triangles, polygons per region.
        let mut tile = Tile::default();
        let mut vert_index: BTreeMap<(i32, i32, u32), u16> = BTreeMap::new();
        for (&rid, rws) in &self.rows[t] {
            let Some(contour) = self.contour(t, rid, rws, (x0, z0, x1, z1)) else {
                continue;
            };
            let pts: Vec<(i64, i64)> = contour.iter().map(|c| (c.0 as i64, c.1 as i64)).collect();
            let tris = triangulate(&pts);
            let polys = merge(&pts, tris);
            // Vertex heights: the highest floor of this region's cells around the vertex.
            let height = |(ix, iz): (i32, i32)| -> u32 {
                let mut best: Option<u32> = None;
                for (cx, cz) in [(ix - 1, iz - 1), (ix, iz - 1), (ix - 1, iz), (ix, iz)] {
                    if self.tile_of(cx.max(0), cz.max(0)) != t {
                        continue;
                    }
                    for s in f.column(cx, cz) {
                        if self.span_region[s] == rid {
                            best = Some(
                                best.map_or(f.spans[s].floor, |b: u32| b.max(f.spans[s].floor)),
                            );
                        }
                    }
                }
                best.unwrap_or(0)
            };
            for poly in polys {
                let mut verts = Vec::with_capacity(poly.len());
                for &k in &poly {
                    let (ix, iz) = contour[k];
                    let h = height((ix, iz));
                    let key = (ix, iz, h);
                    let idx = *vert_index.entry(key).or_insert_with(|| {
                        tile.verts.push(f.world(ix, iz, h));
                        tile.lattice.push((ix, iz));
                        (tile.verts.len() - 1) as u16
                    });
                    verts.push(idx);
                }
                let n = verts.len();
                tile.polys.push(Poly {
                    verts,
                    links: vec![NONE; n],
                    region: rid,
                });
            }
        }
        tile
    }

    /// A region's outline on the lattice, simplified, counter-clockwise.
    fn contour(
        &self,
        t: usize,
        rid: u32,
        rows: &[(i32, Vec<(i32, usize)>)],
        rect: (i32, i32, i32, i32),
    ) -> Option<Vec<(i32, i32)>> {
        let f = &self.field;
        let _ = rect;
        let zmin = rows.first()?.0;
        let zmax = rows.last()?.0;
        let row = |z: i32| -> &Vec<(i32, usize)> { &rows[(z - zmin) as usize].1 };
        let l = |z: i32| row(z).first().map_or(0, |c| c.0);
        let r = |z: i32| row(z).last().map_or(0, |c| c.0 + 1);
        let span_at = |z: i32, x: i32| -> usize {
            row(z)
                .iter()
                .find(|c| c.0 == x)
                .map_or(row(z)[0].1, |c| c.1)
        };
        // What lies across from the inside span `inside` in cell (ox, oz).
        let across = |inside: usize, ox: i32, oz: i32| -> Across {
            match self.step(f.spans[inside], ox, oz) {
                Some(o) => {
                    let g = self.span_region[o];
                    let ot = self.tile_of(ox, oz) as u32;
                    if g == 0 || (ot as usize == t && g == rid) {
                        Across::Wall
                    } else {
                        Across::Region(ot, g)
                    }
                }
                None => Across::Wall,
            }
        };
        // Unit segments: (from vertex, what lies across).
        let mut segs: Vec<((i32, i32), Across)> = Vec::new();
        let (lb, rb) = (l(zmin), r(zmin));
        for x in lb..rb {
            segs.push(((x, zmin), across(span_at(zmin, x), x, zmin - 1)));
        }
        for z in zmin..=zmax {
            let rz = r(z);
            segs.push(((rz, z), across(span_at(z, rz - 1), rz, z)));
            if z < zmax {
                let rn = r(z + 1);
                if rn > rz {
                    for x in rz..rn {
                        segs.push(((x, z + 1), across(span_at(z + 1, x), x, z)));
                    }
                } else if rn < rz {
                    for x in (rn..rz).rev() {
                        segs.push(((x + 1, z + 1), across(span_at(z, x), x, z + 1)));
                    }
                }
            }
        }
        let (lt, rt) = (l(zmax), r(zmax));
        for x in (lt..rt).rev() {
            segs.push(((x + 1, zmax + 1), across(span_at(zmax, x), x, zmax + 1)));
        }
        for z in (zmin..=zmax).rev() {
            let lz = l(z);
            segs.push(((lz, z + 1), across(span_at(z, lz), lz - 1, z)));
            if z > zmin {
                let lp = l(z - 1);
                if lp < lz {
                    for x in (lp..lz).rev() {
                        segs.push(((x + 1, z), across(span_at(z - 1, x), x, z)));
                    }
                } else if lp > lz {
                    for x in lz..lp {
                        segs.push(((x, z), across(span_at(z, x), x, z - 1)));
                    }
                }
            }
        }
        let n = segs.len();
        if n < 3 {
            return None;
        }
        // Keep corners, changes of what lies across, and every tile-border vertex.
        let dir = |i: usize| {
            let a = segs[i].0;
            let b = segs[(i + 1) % n].0;
            (b.0 - a.0, b.1 - a.1)
        };
        let mut keep: Vec<usize> = Vec::new();
        for i in 0..n {
            let p = (i + n - 1) % n;
            let corner = dir(p) != dir(i);
            let change = segs[p].1 != segs[i].1;
            if corner || change {
                keep.push(i);
            }
        }
        if keep.len() < 3 {
            return None;
        }
        let pts: Vec<(i32, i32)> = keep.iter().map(|&i| segs[i].0).collect();
        let tags: Vec<Across> = keep.iter().map(|&i| segs[i].1).collect();
        Some(simplify_walls(&pts, &tags, self.field.params.max_error))
    }

    /// Links every polygon edge to the polygon sharing it.
    fn link(&mut self) {
        let climb = self.field.params.agent_climb + self.field.params.cell_height * 2.0;
        // Directed lattice edge → (poly, edge, mid y).
        let mut edges: EdgeMap = BTreeMap::new();
        for (t, tile) in self.tiles.iter().enumerate() {
            for (p, poly) in tile.polys.iter().enumerate() {
                let n = poly.verts.len();
                for e in 0..n {
                    let (a, b) = (poly.verts[e] as usize, poly.verts[(e + 1) % n] as usize);
                    let y = (tile.verts[a].y + tile.verts[b].y) * 0.5;
                    edges
                        .entry((tile.lattice[a], tile.lattice[b]))
                        .or_default()
                        .push((poly_ref(t, p), e, y));
                }
            }
        }
        for t in 0..self.tiles.len() {
            for p in 0..self.tiles[t].polys.len() {
                let n = self.tiles[t].polys[p].verts.len();
                for e in 0..n {
                    let tile = &self.tiles[t];
                    let poly = &tile.polys[p];
                    let (a, b) = (poly.verts[e] as usize, poly.verts[(e + 1) % n] as usize);
                    let y = (tile.verts[a].y + tile.verts[b].y) * 0.5;
                    let mut best: Option<(f32, PolyRef)> = None;
                    if let Some(list) = edges.get(&(tile.lattice[b], tile.lattice[a])) {
                        for &(q, _, qy) in list {
                            let d = (qy - y).abs();
                            if q != poly_ref(t, p)
                                && d <= climb
                                && best.is_none_or(|(bd, br)| d < bd || (d == bd && q < br))
                            {
                                best = Some((d, q));
                            }
                        }
                    }
                    self.tiles[t].polys[p].links[e] = best.map_or(NONE, |b| b.1);
                }
            }
        }
    }

    // ── queries ──────────────────────────────────────────────────────────

    pub fn poly(&self, r: PolyRef) -> Option<(&Tile, &Poly)> {
        let (t, p) = split_ref(r);
        let tile = self.tiles.get(t)?;
        Some((tile, tile.polys.get(p)?))
    }

    fn xz(v: Vec3) -> Vec2 {
        Vec2::new(v.x, v.z)
    }

    /// Whether `p` (x, z) is inside a polygon or on its edges.
    fn inside(tile: &Tile, poly: &Poly, p: Vec2) -> bool {
        let n = poly.verts.len();
        (0..n).all(|i| {
            let a = Self::xz(tile.verts[poly.verts[i] as usize]);
            let b = Self::xz(tile.verts[poly.verts[(i + 1) % n] as usize]);
            (b - a).perp_dot(p - a) >= -1e-5
        })
    }

    /// The surface height of a polygon at (x, z): its fan triangle's plane.
    pub fn height_at(tile: &Tile, poly: &Poly, p: Vec2) -> f32 {
        let v = |i: usize| tile.verts[poly.verts[i] as usize];
        let mut best = (f32::MAX, v(0).y);
        for i in 1..poly.verts.len() - 1 {
            let (a, b, c) = (v(0), v(i), v(i + 1));
            let (a2, b2, c2) = (Self::xz(a), Self::xz(b), Self::xz(c));
            let den = (b2 - a2).perp_dot(c2 - a2);
            if den.abs() < 1e-9 {
                continue;
            }
            let u = (p - a2).perp_dot(c2 - a2) / den;
            let w = (b2 - a2).perp_dot(p - a2) / den;
            let outside = (-u).max(0.0) + (-w).max(0.0) + (u + w - 1.0).max(0.0);
            let y = a.y + (b.y - a.y) * u + (c.y - a.y) * w;
            if outside < best.0 {
                best = (outside, y);
            }
            if outside == 0.0 {
                break;
            }
        }
        best.1
    }

    /// The closest point of a polygon to `p` (on its surface when `p` is above or below it).
    pub fn closest_on(tile: &Tile, poly: &Poly, p: Vec3) -> Vec3 {
        let q = Self::xz(p);
        if Self::inside(tile, poly, q) {
            return Vec3::new(p.x, Self::height_at(tile, poly, q), p.z);
        }
        let n = poly.verts.len();
        let mut best = (f32::MAX, Vec3::ZERO);
        for i in 0..n {
            let a = tile.verts[poly.verts[i] as usize];
            let b = tile.verts[poly.verts[(i + 1) % n] as usize];
            let ab = Self::xz(b) - Self::xz(a);
            let len2 = ab.length_squared();
            let t = if len2 > 0.0 {
                ((q - Self::xz(a)).dot(ab) / len2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let c = a + (b - a) * t;
            let d = (Self::xz(c) - q).length_squared();
            if d < best.0 {
                best = (d, c);
            }
        }
        best.1
    }

    /// The polygon nearest `p` within `extent` (half sizes) and the nearest
    /// point on it: polygons under or over `p` first, by height difference.
    pub fn nearest(&self, p: Vec3, extent: Vec3) -> Option<(PolyRef, Vec3)> {
        let (x0, z0, x1, z1) = self.cells_of(p - extent, p + extent);
        let ts = self.field.params.tile_size.max(4) as i32;
        let mut best: Option<(f32, PolyRef, Vec3)> = None;
        for tz in (z0 / ts)..=((z1 - 1).max(0) / ts) {
            for tx in (x0 / ts)..=((x1 - 1).max(0) / ts) {
                if tx < 0 || tz < 0 || tx >= self.tiles_x as i32 || tz >= self.tiles_z as i32 {
                    continue;
                }
                let t = (tz as u32 * self.tiles_x + tx as u32) as usize;
                let tile = &self.tiles[t];
                for (pi, poly) in tile.polys.iter().enumerate() {
                    let c = Self::closest_on(tile, poly, p);
                    let d = c - p;
                    if d.x.abs() > extent.x || d.y.abs() > extent.y || d.z.abs() > extent.z {
                        continue;
                    }
                    let score = d.length_squared();
                    let r = poly_ref(t, pi);
                    if best.is_none_or(|(bs, br, _)| score < bs || (score == bs && r < br)) {
                        best = Some((score, r, c));
                    }
                }
            }
        }
        best.map(|(_, r, c)| (r, c))
    }

    fn edge(&self, r: PolyRef, e: usize) -> (Vec3, Vec3) {
        let (tile, poly) = self.poly(r).expect("a live polygon");
        let n = poly.verts.len();
        (
            tile.verts[poly.verts[e] as usize],
            tile.verts[poly.verts[(e + 1) % n] as usize],
        )
    }

    /// The shortest polygon corridor from `start` to `end` (A* over
    /// polygons through edge midpoints), then the funnel's corners.
    /// `extent` is the search box for the end points' polygons.
    pub fn find_path(&self, start: Vec3, end: Vec3, extent: Vec3) -> Option<MeshPath> {
        let (sref, spt) = self.nearest(start, extent)?;
        let (eref, ept) = self.nearest(end, extent)?;
        let mut g: BTreeMap<PolyRef, (f32, PolyRef, Vec3)> = BTreeMap::new();
        let mut closed: BTreeMap<PolyRef, ()> = BTreeMap::new();
        let mut open: BinaryHeap<Reverse<(u32, u32, PolyRef)>> = BinaryHeap::new();
        let key = |x: f32| x.max(0.0).to_bits();
        g.insert(sref, (0.0, NONE, spt));
        open.push(Reverse((key(spt.distance(ept)), 0, sref)));
        let mut seq = 0u32;
        let mut expanded = 0u32;
        let mut best_end = (spt.distance(ept), sref);
        let mut found = false;
        while let Some(Reverse((_, _, cur))) = open.pop() {
            if closed.insert(cur, ()).is_some() {
                continue;
            }
            expanded += 1;
            if cur == eref {
                found = true;
                break;
            }
            let (gc, _, pos) = g[&cur];
            let h = pos.distance(ept);
            if h < best_end.0 {
                best_end = (h, cur);
            }
            let (_, poly) = self.poly(cur).expect("a live polygon");
            for (e, &nb) in poly.links.iter().enumerate() {
                if nb == NONE || closed.contains_key(&nb) {
                    continue;
                }
                let (a, b) = self.edge(cur, e);
                let mid = (a + b) * 0.5;
                let ng = gc + pos.distance(mid) + if nb == eref { mid.distance(ept) } else { 0.0 };
                if g.get(&nb).is_none_or(|old| ng < old.0) {
                    g.insert(nb, (ng, cur, mid));
                    seq += 1;
                    open.push(Reverse((key(ng + mid.distance(ept)), seq, nb)));
                }
            }
        }
        let last = if found { eref } else { best_end.1 };
        let mut polys = vec![last];
        let mut at = last;
        while g[&at].1 != NONE {
            at = g[&at].1;
            polys.push(at);
        }
        polys.reverse();
        let goal = if found {
            ept
        } else {
            let (tile, poly) = self.poly(last).expect("a live polygon");
            Self::closest_on(tile, poly, ept)
        };
        let points = self.funnel(&polys, spt, goal);
        Some(MeshPath {
            points,
            polys,
            expanded,
            complete: found,
        })
    }

    /// The portal (left, right) from polygon `a` into its neighbour `b`.
    fn portal(&self, a: PolyRef, b: PolyRef) -> Option<(Vec3, Vec3)> {
        let (_, poly) = self.poly(a)?;
        let n = poly.links.len();
        // The run of edges into `b` (one run: the polygon is convex), from
        // its first edge's start to its last edge's end.
        let first = (0..n)
            .find(|&e| poly.links[e] == b && poly.links[(e + n - 1) % n] != b)
            .or_else(|| poly.links.iter().position(|&l| l == b))?;
        let mut last = first;
        while poly.links[(last + 1) % n] == b && (last + 1) % n != first {
            last = (last + 1) % n;
        }
        let (p, _) = self.edge(a, first);
        let (_, q) = self.edge(a, last);
        // Counter-clockwise polygons: walking out through p → q, q is on the left.
        Some((q, p))
    }

    /// The simple stupid funnel algorithm over the corridor's portals.
    pub fn funnel(&self, polys: &[PolyRef], start: Vec3, end: Vec3) -> Vec<Vec3> {
        let mut portals: Vec<(Vec3, Vec3)> = vec![(start, start)];
        for w in polys.windows(2) {
            if let Some(p) = self.portal(w[0], w[1]) {
                portals.push(p);
            }
        }
        portals.push((end, end));
        let tri = |a: Vec3, b: Vec3, c: Vec3| -> f32 {
            let (ax, az) = (b.x - a.x, b.z - a.z);
            let (bx, bz) = (c.x - a.x, c.z - a.z);
            bx * az - ax * bz
        };
        let same = |a: Vec3, b: Vec3| (a.x - b.x).abs() < 1e-6 && (a.z - b.z).abs() < 1e-6;
        let mut path = vec![start];
        let (mut apex, mut left, mut right) = (start, portals[0].0, portals[0].1);
        let (mut left_i, mut right_i) = (0usize, 0usize);
        let mut i = 1;
        let mut guard = portals.len() * 4 + 8;
        while i < portals.len() {
            guard -= 1;
            if guard == 0 {
                break;
            }
            let (pl, pr) = portals[i];
            if tri(apex, right, pr) <= 0.0 {
                if same(apex, right) || tri(apex, left, pr) >= 0.0 {
                    right = pr;
                    right_i = i;
                } else {
                    apex = left;
                    let apex_i = left_i;
                    if !same(*path.last().unwrap(), apex) {
                        path.push(apex);
                    }
                    right = apex;
                    right_i = apex_i;
                    i = apex_i + 1;
                    continue;
                }
            }
            if tri(apex, left, pl) >= 0.0 {
                if same(apex, left) || tri(apex, right, pl) <= 0.0 {
                    left = pl;
                    left_i = i;
                } else {
                    apex = right;
                    let apex_i = right_i;
                    if !same(*path.last().unwrap(), apex) {
                        path.push(apex);
                    }
                    left = apex;
                    left_i = apex_i;
                    i = apex_i + 1;
                    continue;
                }
            }
            i += 1;
        }
        if !same(*path.last().unwrap(), end) || path.len() == 1 {
            path.push(end);
        }
        path
    }

    /// Moves from `start` (on polygon `from`) towards `end` along the
    /// surface: across linked edges into neighbours, sliding along walls.
    /// The polygon reached and the point, on its surface.
    pub fn move_along(&self, from: PolyRef, start: Vec3, end: Vec3) -> (PolyRef, Vec3) {
        let mut cur = from;
        let mut a = Self::xz(start);
        let mut b = Self::xz(end);
        let mut slid = false;
        for _ in 0..24 {
            let Some((tile, poly)) = self.poly(cur) else {
                break;
            };
            if Self::inside(tile, poly, b) {
                let y = Self::height_at(tile, poly, b);
                return (cur, Vec3::new(b.x, y, b.y));
            }
            // The edge the segment leaves through first.
            let n = poly.verts.len();
            let d = b - a;
            // Ranked by: the hit lies on the edge (collinear edges share a
            // line, only one holds the point), then the smallest t.
            let mut exit: Option<(bool, f32, usize)> = None;
            for e in 0..n {
                let p = Self::xz(tile.verts[poly.verts[e] as usize]);
                let q = Self::xz(tile.verts[poly.verts[(e + 1) % n] as usize]);
                let edge = q - p;
                let side_b = edge.perp_dot(b - p);
                if side_b >= 0.0 {
                    continue;
                }
                let den = edge.perp_dot(d);
                let t = if den.abs() > 1e-12 {
                    -edge.perp_dot(a - p) / den
                } else {
                    0.0
                };
                let t = t.clamp(0.0, 1.0);
                let hit = a + d * t;
                let len2 = edge.length_squared();
                let s = if len2 > 0.0 {
                    (hit - p).dot(edge) / len2
                } else {
                    0.0
                };
                let on = (-1e-4..=1.0 + 1e-4).contains(&s);
                let better = match exit {
                    None => true,
                    Some((bon, bt, _)) => (on && !bon) || (on == bon && t < bt),
                };
                if better {
                    exit = Some((on, t, e));
                }
            }
            let exit = exit.map(|(_, t, e)| (t, e));
            let Some((t, e)) = exit else { break };
            let hit = a + d * t;
            let link = poly.links[e];
            if link != NONE {
                cur = link;
                a = hit;
                continue;
            }
            if slid {
                let y = Self::height_at(tile, poly, hit);
                return (cur, Vec3::new(hit.x, y, hit.y));
            }
            // A wall: keep the motion along it.
            let p = Self::xz(tile.verts[poly.verts[e] as usize]);
            let q = Self::xz(tile.verts[poly.verts[(e + 1) % n] as usize]);
            let along = (q - p).normalize_or_zero();
            let rest = (b - hit).dot(along);
            // Nudge inside by a hair so the next test starts in this polygon.
            let inward = Vec2::new(-along.y, along.x) * 1e-4;
            a = hit + inward;
            b = hit + along * rest + inward;
            slid = true;
        }
        let (tile, poly) = self.poly(cur).expect("a live polygon");
        let c = Self::closest_on(tile, poly, Vec3::new(a.x, start.y, a.y));
        (cur, c)
    }
}

/// Douglas–Peucker over each run of wall edges, keeping the outline simple.
fn simplify_walls(pts: &[(i32, i32)], tags: &[Across], max_error: f32) -> Vec<(i32, i32)> {
    let n = pts.len();
    // Anchors: vertices where an edge that is not a wall starts or ends.
    let anchor: Vec<bool> = (0..n)
        .map(|i| tags[i] != Across::Wall || tags[(i + n - 1) % n] != Across::Wall)
        .collect();
    let mut keep = anchor.clone();
    let anchors: Vec<usize> = (0..n).filter(|&i| anchor[i]).collect();
    let chains: Vec<(usize, usize)> = if anchors.is_empty() {
        // All walls: anchor at the lowest-left vertex and the one farthest from it.
        let a = (0..n).min_by_key(|&i| (pts[i].0, pts[i].1)).unwrap_or(0);
        let far = |i: usize| {
            let (dx, dz) = ((pts[i].0 - pts[a].0) as i64, (pts[i].1 - pts[a].1) as i64);
            dx * dx + dz * dz
        };
        let b = (0..n)
            .max_by_key(|&i| (far(i), std::cmp::Reverse(i)))
            .unwrap_or(0);
        keep[a] = true;
        keep[b] = true;
        vec![(a, b), (b, a)]
    } else {
        let k = anchors.len();
        (0..k).map(|i| (anchors[i], anchors[(i + 1) % k])).collect()
    };
    let err2 = (max_error * max_error) as f64;
    for (s, e) in chains {
        // The chain s → e (exclusive of both), walls only.
        let mut idx = Vec::new();
        let mut i = (s + 1) % n;
        while i != e {
            idx.push(i);
            i = (i + 1) % n;
        }
        if idx.is_empty() {
            continue;
        }
        let mut chain_keep = vec![false; idx.len()];
        dp(pts, s, e, &idx, &mut chain_keep, err2);
        let mut trial = keep.clone();
        for (k, &j) in idx.iter().enumerate() {
            trial[j] = chain_keep[k];
        }
        let candidate: Vec<(i32, i32)> = (0..n).filter(|&j| trial[j]).map(|j| pts[j]).collect();
        if candidate.len() >= 3 && simple(&candidate) && area2(&candidate) > 0 {
            keep = trial;
        } else {
            for &j in &idx {
                keep[j] = true;
            }
        }
    }
    (0..n).filter(|&i| keep[i]).map(|i| pts[i]).collect()
}

fn dp(pts: &[(i32, i32)], a: usize, b: usize, idx: &[usize], keep: &mut [bool], err2: f64) {
    if idx.is_empty() {
        return;
    }
    let (ax, az) = (pts[a].0 as f64, pts[a].1 as f64);
    let (bx, bz) = (pts[b].0 as f64, pts[b].1 as f64);
    let (dx, dz) = (bx - ax, bz - az);
    let len2 = dx * dx + dz * dz;
    let mut far = (0.0f64, 0usize);
    for (k, &i) in idx.iter().enumerate() {
        let (px, pz) = (pts[i].0 as f64 - ax, pts[i].1 as f64 - az);
        let d2 = if len2 > 0.0 {
            let t = ((px * dx + pz * dz) / len2).clamp(0.0, 1.0);
            let (qx, qz) = (px - dx * t, pz - dz * t);
            qx * qx + qz * qz
        } else {
            px * px + pz * pz
        };
        if d2 > far.0 {
            far = (d2, k);
        }
    }
    if far.0 > err2 {
        let k = far.1;
        keep[k] = true;
        let m = idx[k];
        let (left, right) = idx.split_at(k);
        let (lk, rk) = keep.split_at_mut(k);
        dp(pts, a, m, left, lk, err2);
        dp(pts, m, b, &right[1..], &mut rk[1..], err2);
    }
}

fn area2(p: &[(i32, i32)]) -> i64 {
    let n = p.len();
    (0..n)
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % n]);
            a.0 as i64 * b.1 as i64 - b.0 as i64 * a.1 as i64
        })
        .sum()
}

/// No two non-adjacent edges touch, and no vertex repeats.
fn simple(p: &[(i32, i32)]) -> bool {
    let n = p.len();
    let q: Vec<(i64, i64)> = p.iter().map(|v| (v.0 as i64, v.1 as i64)).collect();
    for i in 0..n {
        for j in i + 1..n {
            if q[i] == q[j] {
                return false;
            }
        }
    }
    for i in 0..n {
        let (a, b) = (q[i], q[(i + 1) % n]);
        for j in i + 1..n {
            if j == i || (j + 1) % n == i || (i + 1) % n == j {
                continue;
            }
            let (c, d) = (q[j], q[(j + 1) % n]);
            if segments_touch(a, b, c, d) {
                return false;
            }
        }
    }
    true
}

fn segments_touch(a: (i64, i64), b: (i64, i64), c: (i64, i64), d: (i64, i64)) -> bool {
    let d1 = cross(c, d, a);
    let d2 = cross(c, d, b);
    let d3 = cross(a, b, c);
    let d4 = cross(a, b, d);
    if ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0)) {
        return true;
    }
    let on = |p: (i64, i64), q: (i64, i64), r: (i64, i64)| {
        r.0 >= p.0.min(q.0) && r.0 <= p.0.max(q.0) && r.1 >= p.1.min(q.1) && r.1 <= p.1.max(q.1)
    };
    (d1 == 0 && on(c, d, a))
        || (d2 == 0 && on(c, d, b))
        || (d3 == 0 && on(a, b, c))
        || (d4 == 0 && on(a, b, d))
}

/// Ear clipping of a simple counter-clockwise polygon: index triples.
fn triangulate(p: &[(i64, i64)]) -> Vec<[usize; 3]> {
    let mut idx: Vec<usize> = (0..p.len()).collect();
    let mut tris = Vec::new();
    let mut guard = p.len() * p.len() + 8;
    while idx.len() > 3 && guard > 0 {
        guard -= 1;
        let n = idx.len();
        let mut clipped = false;
        for k in 0..n {
            let (a, b, c) = (idx[(k + n - 1) % n], idx[k], idx[(k + 1) % n]);
            if cross(p[a], p[b], p[c]) <= 0 {
                continue;
            }
            let blocked = idx.iter().any(|&v| {
                if v == a || v == b || v == c || p[v] == p[a] || p[v] == p[b] || p[v] == p[c] {
                    return false;
                }
                cross(p[a], p[b], p[v]) >= 0
                    && cross(p[b], p[c], p[v]) >= 0
                    && cross(p[c], p[a], p[v]) >= 0
            });
            if blocked {
                continue;
            }
            tris.push([a, b, c]);
            idx.remove(k);
            clipped = true;
            break;
        }
        if !clipped {
            // Degenerate: drop the flattest vertex.
            let n = idx.len();
            let k = (0..n)
                .min_by_key(|&k| {
                    cross(p[idx[(k + n - 1) % n]], p[idx[k]], p[idx[(k + 1) % n]]).abs()
                })
                .unwrap_or(0);
            let (a, b, c) = (idx[(k + n - 1) % n], idx[k], idx[(k + 1) % n]);
            if cross(p[a], p[b], p[c]) > 0 {
                tris.push([a, b, c]);
            }
            idx.remove(k);
        }
    }
    if idx.len() == 3 && cross(p[idx[0]], p[idx[1]], p[idx[2]]) > 0 {
        tris.push([idx[0], idx[1], idx[2]]);
    }
    tris
}

/// Merges triangles into convex polygons (collinear vertices allowed),
/// longest shared edge first, at most [`MAX_VERTS`] vertices each.
fn merge(p: &[(i64, i64)], tris: Vec<[usize; 3]>) -> Vec<Vec<usize>> {
    let mut polys: Vec<Vec<usize>> = tris.into_iter().map(|t| t.to_vec()).collect();
    loop {
        let mut best: Option<(i64, usize, usize, Vec<usize>)> = None;
        for i in 0..polys.len() {
            for j in i + 1..polys.len() {
                if let Some((len, merged)) = try_merge(p, &polys[i], &polys[j])
                    && best.as_ref().is_none_or(|b| len > b.0)
                {
                    best = Some((len, i, j, merged));
                }
            }
        }
        match best {
            Some((_, i, j, merged)) => {
                polys[i] = merged;
                polys.remove(j);
            }
            None => break,
        }
    }
    polys
}

/// Two polygons sharing an edge, merged when the result is convex and small enough.
fn try_merge(p: &[(i64, i64)], a: &[usize], b: &[usize]) -> Option<(i64, Vec<usize>)> {
    let (na, nb) = (a.len(), b.len());
    if na + nb - 2 > MAX_VERTS {
        return None;
    }
    for i in 0..na {
        let (a0, a1) = (a[i], a[(i + 1) % na]);
        for j in 0..nb {
            let (b0, b1) = (b[j], b[(j + 1) % nb]);
            if !(p[a0] == p[b1] && p[a1] == p[b0]) {
                continue;
            }
            let mut merged = Vec::with_capacity(na + nb - 2);
            for k in 0..na - 1 {
                merged.push(a[(i + 1 + k) % na]);
            }
            for k in 0..nb - 1 {
                merged.push(b[(j + 1 + k) % nb]);
            }
            let m = merged.len();
            let convex = (0..m).all(|k| {
                cross(
                    p[merged[(k + m - 1) % m]],
                    p[merged[k]],
                    p[merged[(k + 1) % m]],
                ) >= 0
            });
            if !convex {
                return None;
            }
            let (dx, dz) = (p[a1].0 - p[a0].0, p[a1].1 - p[a0].1);
            return Some((dx * dx + dz * dz, merged));
        }
    }
    None
}
