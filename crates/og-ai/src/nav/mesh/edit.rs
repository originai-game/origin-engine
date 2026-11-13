//! Runtime edits: obstacles, carved shapes and areas, each rebuilding only
//! the tiles it touches.

use super::*;

impl NavMesh {
    /// Sets, moves (`Some` with an id in use) or removes (`None`) a box
    /// obstacle, and rebuilds the tiles under its old and new boxes. The
    /// tiles rebuilt, in order.
    pub fn set_obstacle(&mut self, id: u32, obstacle: Option<Obstacle>) -> Vec<usize> {
        self.carve(
            id,
            obstacle.map(|o| Shape::Box {
                min: o.min,
                max: o.max,
            }),
        )
    }

    /// [`Self::set_obstacle`] for any shape.
    pub fn carve(&mut self, id: u32, shape: Option<Shape>) -> Vec<usize> {
        let mut bounds: Vec<(Vec3, Vec3)> = shape.iter().map(Shape::bounds).collect();
        let old = match shape {
            Some(s) => self.obstacles.insert(id, s),
            None => self.obstacles.remove(&id),
        };
        bounds.splice(0..0, old.iter().map(Shape::bounds));
        self.rebuild_bounds(&bounds)
    }

    /// Sets, moves or removes (`None`) a run-time area: the spans under
    /// `shape` take area `area` (over the baked areas and lower ids), and
    /// the tiles under its old and new shapes rebuild. The tiles rebuilt.
    pub fn set_area(&mut self, id: u32, area: Option<(Shape, u8)>) -> Vec<usize> {
        let mut bounds: Vec<(Vec3, Vec3)> = area.iter().map(|a| a.0.bounds()).collect();
        let old = match area {
            Some(a) => self.areas.insert(id, a),
            None => self.areas.remove(&id),
        };
        bounds.splice(0..0, old.iter().map(|a| a.0.bounds()));
        self.mark_areas();
        self.rebuild_bounds(&bounds)
    }

    /// Rebuilds the tiles that overlap `min..max` (after a change to the
    /// level there): the region rebuild an editor or a procedural level calls.
    pub fn rebuild_region(&mut self, min: Vec3, max: Vec3) -> Vec<usize> {
        self.rebuild_bounds(&[(min, max)])
    }

    fn rebuild_bounds(&mut self, boxes: &[(Vec3, Vec3)]) -> Vec<usize> {
        let p = self.field.params;
        let r = p.agent_radius;
        let grow = Vec3::new(r, 0.0, r);
        // Spans under every obstacle (grown by the radius), from scratch.
        let mut blocked = vec![false; self.field.spans.len()];
        for o in self.obstacles.values() {
            let (min, max) = o.bounds();
            let (x0, z0, x1, z1) = self.cells_of(min - grow, max + grow);
            for z in z0..z1 {
                for x in x0..x1 {
                    // A box blocks every cell it touches; another shape the
                    // cells whose centre it covers, grown by the radius.
                    if !matches!(o, Shape::Box { .. }) {
                        let cx = self.field.origin.x + (x as f32 + 0.5) * p.cell_size;
                        let cz = self.field.origin.z + (z as f32 + 0.5) * p.cell_size;
                        if !o.covers(cx, cz, r) {
                            continue;
                        }
                    }
                    for s in self.field.column(x, z) {
                        let sp = self.field.spans[s];
                        let y = self.field.origin.y + sp.floor as f32 * p.cell_height;
                        if y + p.agent_height >= min.y && y <= max.y + p.agent_climb {
                            blocked[s] = true;
                        }
                    }
                }
            }
        }
        self.blocked = blocked;
        let ts = p.tile_size.max(4) as i32;
        let mut tiles = Vec::new();
        for &(bmin, bmax) in boxes {
            let (x0, z0, x1, z1) = self.cells_of(bmin - grow, bmax + grow);
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
        self.resolve_links();
        self.version += 1;
        rebuild
    }

    /// The cell rectangle `[x0, x1) × [z0, z1)` a world box covers, clamped.
    pub(in crate::nav::mesh) fn cells_of(&self, min: Vec3, max: Vec3) -> (i32, i32, i32, i32) {
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
    pub(in crate::nav::mesh) fn tile_of(&self, x: i32, z: i32) -> usize {
        let ts = self.field.params.tile_size.max(4) as i32;
        ((z / ts) as u32 * self.tiles_x + (x / ts) as u32) as usize
    }
}
