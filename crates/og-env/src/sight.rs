//! Fog of war: which cells of a ground map each team sees now and has
//! seen before. A [`SightMap`] is a grid over a rectangle of the ground
//! (x, z); each tick its blockers are laid on it, every viewer's sight is
//! cast over it, and what a team sees is added to what it has explored.
//! The simulation asks it about a point ([`SightMap::visible`],
//! [`SightMap::explored`]) and the renderer draws its
//! [`SightMap::levels`], so the two cannot disagree.
//!
//! Sight is symmetric shadow casting (Albert Ford, *Symmetric
//! Shadowcasting*, 2020, CC0): between two open cells, one sees the other
//! exactly when the other sees it, and a wall's face is seen from its
//! open side. Slopes are whole-number fractions, so every host casts the
//! same cells.

use og_core::glam::Vec2;
use std::collections::BTreeMap;

/// The most cells a map has along a side: a larger region takes a larger
/// cell.
pub const MAX_SIDE: u32 = 512;
/// The teams a map keeps cells for are 0 to this.
pub const MAX_TEAM: u32 = 31;

/// A cell's level for a team ([`SightMap::levels`]).
pub const UNSEEN: u8 = 0;
pub const EXPLORED: u8 = 1;
pub const VISIBLE: u8 = 2;

/// Something that sees: a team's eyes at a point of the ground, reaching
/// `radius` metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewer {
    pub team: u32,
    pub at: Vec2,
    pub radius: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Team {
    visible: Vec<u64>,
    explored: Vec<u64>,
}

/// One map: its grid, the cells sight does not pass, and each team's.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SightMap {
    columns: u32,
    rows: u32,
    cell: f32,
    /// Its min corner on the ground (x, z).
    min: Vec2,
    opaque: Vec<u64>,
    blocked: bool,
    teams: BTreeMap<u32, Team>,
    /// What every team saw before this tick's sight was cast.
    before: Vec<(u32, Vec<u64>)>,
    revision: u64,
}

fn words(cells: usize) -> usize {
    cells.div_ceil(64)
}

fn get(bits: &[u64], i: usize) -> bool {
    bits[i / 64] >> (i % 64) & 1 == 1
}

fn set(bits: &mut [u64], i: usize) {
    bits[i / 64] |= 1 << (i % 64);
}

impl SightMap {
    /// Lays the map over the rectangle of `size` (x, z) centred on
    /// `centre`, in cells of `cell` metres (at most [`MAX_SIDE`] a side: a
    /// smaller cell is widened to fit). A map whose grid changes is another
    /// map: what its teams explored is forgotten.
    pub fn fit(&mut self, centre: Vec2, size: Vec2, cell: f32) {
        let size = size.max(Vec2::splat(1e-3));
        let cell = cell.max(size.max_element() / MAX_SIDE as f32).max(1e-3);
        let side = |s: f32| ((s / cell).ceil() as u32).clamp(1, MAX_SIDE);
        let (columns, rows) = (side(size.x), side(size.y));
        if (columns, rows, cell.to_bits()) != (self.columns, self.rows, self.cell.to_bits()) {
            *self = SightMap {
                columns,
                rows,
                cell,
                opaque: vec![0; words((columns * rows) as usize)],
                revision: self.revision + 1,
                ..SightMap::default()
            };
        }
        self.min = centre - Vec2::new(columns as f32, rows as f32) * cell * 0.5;
    }

    /// Cells a side (x, z).
    pub fn size(&self) -> (u32, u32) {
        (self.columns, self.rows)
    }

    /// A cell's side (m).
    pub fn cell(&self) -> f32 {
        self.cell
    }

    /// The min corner on the ground (x, z).
    pub fn min(&self) -> Vec2 {
        self.min
    }

    /// Moves the map with the world's zero.
    pub fn shift(&mut self, d: Vec2) {
        self.min += d;
    }

    /// Counts every change of a team's cells: a drawer uploads when it moves.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn cells(&self) -> usize {
        (self.columns * self.rows) as usize
    }

    /// The cell holding `p`, `None` outside the map.
    fn cell_at(&self, p: Vec2) -> Option<usize> {
        let q = ((p - self.min) / self.cell).floor();
        let inside =
            q.x >= 0.0 && q.y >= 0.0 && q.x < self.columns as f32 && q.y < self.rows as f32;
        inside.then(|| q.y as usize * self.columns as usize + q.x as usize)
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let inside = x >= 0 && y >= 0 && x < i64::from(self.columns) && y < i64::from(self.rows);
        inside.then(|| y as usize * self.columns as usize + x as usize)
    }

    /// Whether the map covers `p`.
    pub fn covers(&self, p: Vec2) -> bool {
        self.cell_at(p).is_some()
    }

    /// Starts a tick: no blockers, and nobody sees anything yet.
    pub fn begin(&mut self) {
        self.opaque.iter_mut().for_each(|w| *w = 0);
        self.blocked = false;
        self.before = self
            .teams
            .iter_mut()
            .map(|(t, team)| {
                let none = vec![0; team.visible.len()];
                (*t, std::mem::replace(&mut team.visible, none))
            })
            .collect();
    }

    /// Lays a blocker: the cells between `lo` and `hi` whose centre
    /// `inside` holds stop sight (their near face is still seen).
    pub fn block(&mut self, lo: Vec2, hi: Vec2, inside: impl Fn(Vec2) -> bool) {
        let a = ((lo - self.min) / self.cell).floor().max(Vec2::ZERO);
        let b = ((hi - self.min) / self.cell).floor();
        if b.x < 0.0 || b.y < 0.0 {
            return;
        }
        let (x1, y1) = (
            (b.x as u32).min(self.columns.saturating_sub(1)),
            (b.y as u32).min(self.rows.saturating_sub(1)),
        );
        for y in a.y as u32..=y1 {
            for x in a.x as u32..=x1 {
                let centre = self.min + (Vec2::new(x as f32, y as f32) + 0.5) * self.cell;
                if inside(centre) {
                    set(&mut self.opaque, (y * self.columns + x) as usize);
                    self.blocked = true;
                }
            }
        }
    }

    fn team(&mut self, team: u32) -> Option<&mut Team> {
        if team > MAX_TEAM || self.cells() == 0 {
            return None;
        }
        let n = words(self.cells());
        Some(self.teams.entry(team).or_insert_with(|| Team {
            visible: vec![0; n],
            explored: vec![0; n],
        }))
    }

    /// Casts a viewer's sight: its team sees every cell within its radius
    /// that no blocker hides.
    pub fn see(&mut self, v: Viewer) {
        if v.radius.is_nan() || v.radius <= 0.0 || self.cells() == 0 {
            return;
        }
        let o = ((v.at - self.min) / self.cell).floor();
        let reach = v.radius / self.cell;
        // A viewer whose reach misses the map sees none of it.
        if o.x + reach < 0.0
            || o.y + reach < 0.0
            || o.x - reach > self.columns as f32
            || o.y - reach > self.rows as f32
        {
            return;
        }
        let (ox, oy) = (o.x as i64, o.y as i64);
        let depth = reach.ceil() as i64;
        let reach2 = (reach * reach).floor() as i64;
        let mut seen = Vec::new();
        if self.blocked {
            cast(self, ox, oy, depth, reach2, &mut seen);
        } else {
            for dy in -depth..=depth {
                for dx in -depth..=depth {
                    if dx * dx + dy * dy <= reach2 {
                        seen.extend(self.index(ox + dx, oy + dy));
                    }
                }
            }
        }
        let Some(team) = self.team(v.team) else {
            return;
        };
        for i in seen {
            set(&mut team.visible, i);
        }
    }

    /// Ends a tick: what each team sees is explored.
    pub fn finish(&mut self) {
        let mut changed = self.before.len() != self.teams.len();
        for (t, team) in &mut self.teams {
            for (e, v) in team.explored.iter_mut().zip(&team.visible) {
                *e |= v;
            }
            changed |= self
                .before
                .iter()
                .find(|(b, _)| b == t)
                .is_none_or(|(_, b)| *b != team.visible);
        }
        self.before.clear();
        self.revision += u64::from(changed);
    }

    /// Whether `team` sees the ground at `p` this tick (false outside the
    /// map).
    pub fn visible(&self, team: u32, p: Vec2) -> bool {
        self.cell_at(p)
            .zip(self.teams.get(&team))
            .is_some_and(|(i, t)| get(&t.visible, i))
    }

    /// Whether `team` has ever seen the ground at `p` (false outside the
    /// map).
    pub fn explored(&self, team: u32, p: Vec2) -> bool {
        self.cell_at(p)
            .zip(self.teams.get(&team))
            .is_some_and(|(i, t)| get(&t.explored, i))
    }

    /// Marks the ground within `radius` of `p` as explored by `team`
    /// without anyone seeing it (a map bought, a scout's report).
    pub fn reveal(&mut self, team: u32, p: Vec2, radius: f32) {
        let o = ((p - self.min) / self.cell).floor();
        let reach = radius.max(0.0) / self.cell;
        let (ox, oy, depth) = (o.x as i64, o.y as i64, reach.ceil() as i64);
        let reach2 = (reach * reach).floor() as i64;
        let mut cells = Vec::new();
        for dy in -depth..=depth {
            for dx in -depth..=depth {
                if dx * dx + dy * dy <= reach2 {
                    cells.extend(self.index(ox + dx, oy + dy));
                }
            }
        }
        let Some(t) = self.team(team) else {
            return;
        };
        for i in cells {
            set(&mut t.explored, i);
        }
        self.revision += 1;
    }

    /// Forgets what `team` explored (every team's with `None`); what is
    /// seen this tick stays seen and explored.
    pub fn forget(&mut self, team: Option<u32>) {
        for (t, cells) in &mut self.teams {
            if team.is_none_or(|k| k == *t) {
                cells.explored.clone_from(&cells.visible);
            }
        }
        self.revision += 1;
    }

    /// Every cell's level for `team`, row by row from the min corner
    /// ([`UNSEEN`], [`EXPLORED`], [`VISIBLE`]).
    pub fn levels(&self, team: u32) -> Vec<u8> {
        let Some(t) = self.teams.get(&team) else {
            return vec![UNSEEN; self.cells()];
        };
        (0..self.cells())
            .map(|i| {
                if get(&t.visible, i) {
                    VISIBLE
                } else if get(&t.explored, i) {
                    EXPLORED
                } else {
                    UNSEEN
                }
            })
            .collect()
    }

    /// Appends the map's state: equal maps, equal bytes.
    pub fn save(&self, out: &mut Vec<u8>) {
        for v in [self.columns, self.rows, self.cell.to_bits()] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for v in [self.min.x, self.min.y] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&(self.teams.len() as u32).to_le_bytes());
        for (t, cells) in &self.teams {
            out.extend_from_slice(&t.to_le_bytes());
            for w in cells.visible.iter().chain(&cells.explored) {
                out.extend_from_slice(&w.to_le_bytes());
            }
        }
    }

    /// Reads [`SightMap::save`]'s bytes from the front of `bytes`: the map
    /// and how many bytes it took.
    pub fn load(bytes: &[u8]) -> Result<(SightMap, usize), String> {
        let short = || "a sight map's bytes end early".to_string();
        let mut at = 0;
        let u32_at = |at: &mut usize| -> Result<u32, String> {
            let b = bytes.get(*at..*at + 4).ok_or_else(short)?;
            *at += 4;
            Ok(u32::from_le_bytes(b.try_into().unwrap()))
        };
        let (columns, rows) = (u32_at(&mut at)?, u32_at(&mut at)?);
        let cell = f32::from_bits(u32_at(&mut at)?);
        let min = Vec2::new(
            f32::from_bits(u32_at(&mut at)?),
            f32::from_bits(u32_at(&mut at)?),
        );
        if columns > MAX_SIDE
            || rows > MAX_SIDE
            || (cell.is_nan() || cell <= 0.0) && columns * rows > 0
        {
            return Err("a sight map's grid is out of range".into());
        }
        let n = words((columns * rows) as usize);
        let count = u32_at(&mut at)?;
        let mut teams = BTreeMap::new();
        for _ in 0..count {
            let t = u32_at(&mut at)?;
            if t > MAX_TEAM {
                return Err(format!("a sight map holds team {t}"));
            }
            let read = |at: &mut usize| -> Result<Vec<u64>, String> {
                let b = bytes.get(*at..*at + n * 8).ok_or_else(short)?;
                *at += n * 8;
                Ok(b.as_chunks::<8>()
                    .0
                    .iter()
                    .map(|c| u64::from_le_bytes(*c))
                    .collect())
            };
            let visible = read(&mut at)?;
            let explored = read(&mut at)?;
            teams.insert(t, Team { visible, explored });
        }
        let map = SightMap {
            columns,
            rows,
            cell,
            min,
            opaque: vec![0; n],
            teams,
            ..SightMap::default()
        };
        Ok((map, at))
    }
}

/// One row of a quadrant's scan: its depth and the slopes (numerator over
/// denominator) it lies between.
#[derive(Clone, Copy)]
struct Row {
    depth: i64,
    start: (i64, i64),
    end: (i64, i64),
}

/// Symmetric shadow casting from cell (`ox`, `oy`) out to `depth` rows:
/// the cells seen within `reach2` (cells, squared) into `seen`.
fn cast(map: &SightMap, ox: i64, oy: i64, depth: i64, reach2: i64, seen: &mut Vec<usize>) {
    seen.extend(map.index(ox, oy));
    let wall = |x: i64, y: i64| map.index(x, y).is_some_and(|i| get(&map.opaque, i));
    for quadrant in 0..4 {
        let place = |row: i64, col: i64| match quadrant {
            0 => (ox + col, oy - row),
            1 => (ox + col, oy + row),
            2 => (ox + row, oy + col),
            _ => (ox - row, oy + col),
        };
        let mut rows = vec![Row {
            depth: 1,
            start: (-1, 1),
            end: (1, 1),
        }];
        while let Some(mut row) = rows.pop() {
            if row.depth > depth {
                continue;
            }
            // depth * slope rounded to the nearest column, ties towards
            // the row's inside.
            let d = row.depth;
            let first = (2 * d * row.start.0 + row.start.1).div_euclid(2 * row.start.1);
            let last = -(-(2 * d * row.end.0 - row.end.1)).div_euclid(2 * row.end.1);
            let mut previous: Option<bool> = None;
            for col in first..=last {
                let (x, y) = place(d, col);
                let is_wall = wall(x, y);
                let symmetric =
                    col * row.start.1 >= d * row.start.0 && col * row.end.1 <= d * row.end.0;
                if (is_wall || symmetric) && d * d + col * col <= reach2 {
                    seen.extend(map.index(x, y));
                }
                // The slope of the cell's near edge.
                let edge = (2 * col - 1, 2 * d);
                if previous == Some(true) && !is_wall {
                    row.start = edge;
                }
                if previous == Some(false) && is_wall {
                    rows.push(Row {
                        depth: d + 1,
                        start: row.start,
                        end: edge,
                    });
                }
                previous = Some(is_wall);
            }
            if previous == Some(false) {
                rows.push(Row {
                    depth: d + 1,
                    ..row
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 32 m square of 1 m cells centred on the origin.
    fn field() -> SightMap {
        let mut m = SightMap::default();
        m.fit(Vec2::ZERO, Vec2::splat(32.0), 1.0);
        m
    }

    fn eye(team: u32, x: f32, z: f32, radius: f32) -> Viewer {
        Viewer {
            team,
            at: Vec2::new(x, z),
            radius,
        }
    }

    /// A wall along x = 4..5 from z = -3 to 3.
    fn wall(m: &mut SightMap) {
        m.block(Vec2::new(4.0, -3.0), Vec2::new(4.9, 2.9), |_| true);
    }

    #[test]
    fn an_open_field_is_seen_as_a_disc() {
        let mut m = field();
        m.begin();
        m.see(eye(0, 0.5, 0.5, 5.0));
        m.finish();
        assert!(m.visible(0, Vec2::new(0.5, 0.5)));
        assert!(m.visible(0, Vec2::new(4.5, 3.5)), "3-4-5 lies on the rim");
        assert!(!m.visible(0, Vec2::new(4.5, 4.5)));
        assert!(
            !m.visible(1, Vec2::new(0.5, 0.5)),
            "another team sees nothing"
        );
        let seen = m.levels(0).iter().filter(|&&l| l == VISIBLE).count();
        assert_eq!(seen, 81, "the cells within 5 of a cell");
    }

    #[test]
    fn a_wall_hides_what_is_behind_it_and_shows_its_face() {
        let mut m = field();
        m.begin();
        wall(&mut m);
        m.see(eye(0, 0.5, 0.5, 12.0));
        m.finish();
        assert!(m.visible(0, Vec2::new(4.5, 0.5)), "the wall's face");
        assert!(!m.visible(0, Vec2::new(8.5, 0.5)), "behind the wall");
        assert!(m.visible(0, Vec2::new(8.5, 7.5)), "past its end");
        assert!(m.visible(0, Vec2::new(-8.5, 0.5)), "the open side");
    }

    #[test]
    fn two_open_cells_see_each_other_or_neither_does() {
        let mut blockers = Vec::new();
        let mut s = 12345u32;
        let mut next = |n: u32| {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            (s >> 16) % n
        };
        for _ in 0..60 {
            blockers.push((next(32) as f32 - 16.0, next(32) as f32 - 16.0));
        }
        let lay = |m: &mut SightMap| {
            m.begin();
            for &(x, z) in &blockers {
                m.block(Vec2::new(x, z), Vec2::new(x + 0.9, z + 0.9), |_| true);
            }
        };
        let open = |x: f32, z: f32| !blockers.contains(&(x.floor(), z.floor()));
        let mut pairs = 0;
        for _ in 0..200 {
            let a = (next(32) as f32 - 15.5, next(32) as f32 - 15.5);
            let b = (next(32) as f32 - 15.5, next(32) as f32 - 15.5);
            if !open(a.0, a.1) || !open(b.0, b.1) {
                continue;
            }
            let mut m = field();
            lay(&mut m);
            m.see(eye(0, a.0, a.1, 50.0));
            m.see(eye(1, b.0, b.1, 50.0));
            m.finish();
            assert_eq!(
                m.visible(0, Vec2::new(b.0, b.1)),
                m.visible(1, Vec2::new(a.0, a.1)),
                "{a:?} and {b:?}"
            );
            pairs += 1;
        }
        assert!(pairs > 100);
    }

    #[test]
    fn what_was_seen_stays_explored_and_is_saved() {
        let mut m = field();
        m.begin();
        m.see(eye(2, 0.5, 0.5, 4.0));
        m.finish();
        let seen = m.revision();
        m.begin();
        m.see(eye(2, 10.5, 0.5, 4.0));
        m.finish();
        assert!(m.revision() > seen, "the cells changed");
        let p = Vec2::new(0.5, 0.5);
        assert!(!m.visible(2, p) && m.explored(2, p));
        assert_eq!(m.levels(2)[16 * 32 + 16], EXPLORED);
        let still = m.revision();
        m.begin();
        m.see(eye(2, 10.5, 0.5, 4.0));
        m.finish();
        assert_eq!(m.revision(), still, "nothing moved");

        let mut bytes = Vec::new();
        m.save(&mut bytes);
        let (back, used) = SightMap::load(&bytes).expect("loads");
        assert_eq!(used, bytes.len());
        let mut again = Vec::new();
        back.save(&mut again);
        assert_eq!(bytes, again);
        assert!(back.explored(2, p) && back.visible(2, Vec2::new(10.5, 0.5)));
        assert!(SightMap::load(&bytes[..bytes.len() - 1]).is_err());

        m.reveal(5, Vec2::new(-8.0, -8.0), 2.0);
        assert!(m.explored(5, Vec2::new(-8.5, -8.5)) && !m.visible(5, Vec2::new(-8.5, -8.5)));
        m.forget(Some(2));
        assert!(!m.explored(2, p) && m.explored(2, Vec2::new(10.5, 0.5)));
    }

    #[test]
    fn another_grid_is_another_map() {
        let mut m = field();
        m.begin();
        m.see(eye(0, 0.5, 0.5, 4.0));
        m.finish();
        // Moved, the same grid: the cells go with it.
        m.fit(Vec2::new(100.0, 0.0), Vec2::splat(32.0), 1.0);
        assert!(m.explored(0, Vec2::new(100.5, 0.5)));
        m.fit(Vec2::new(100.0, 0.0), Vec2::splat(32.0), 0.5);
        assert_eq!(m.size(), (64, 64));
        assert!(!m.explored(0, Vec2::new(100.5, 0.5)));
        // A region too large for the cell asked for takes a larger one.
        m.fit(Vec2::ZERO, Vec2::new(2048.0, 100.0), 1.0);
        assert_eq!(m.size(), (MAX_SIDE, 25));
        assert_eq!(m.cell(), 4.0);
        assert!(!m.covers(Vec2::new(0.0, 60.0)) && m.covers(Vec2::new(1000.0, 0.0)));
    }
}
