//! Procedural generation on the engine's seeded [`Rng`]: weighted picks and
//! permutations, gradient noise (Perlin, simplex, FBM), Poisson-disk
//! points, BSP rooms and wave function collapse.
//!
//! Determinism: every generator takes a seed (idiomatically
//! og.Rng's `rng.fork("label").seed`) and draws only from an [`Rng`] on it, in
//! a fixed order, so the main stream never moves and the same seed gives
//! the same output on every host. The arithmetic is `+ - * /`, `floor` and
//! `sqrt` on f32 (correctly rounded on wasm and native alike), never a
//! transcendental function, and every loop walks a fixed order; ties are
//! broken by index.

use crate::Rng;
use crate::{Error, ErrorKind, Result};
use std::cell::RefCell;
use std::sync::Arc;

// ---- picks -----------------------------------------------------------------

/// The index of a weight drawn in proportion to it (one draw), among the
/// positive finite weights; `None` when there is none.
pub fn weighted(rng: &mut Rng, weights: &[f64]) -> Option<usize> {
    let ok = |w: f64| w.is_finite() && w > 0.0;
    let total: f64 = weights.iter().copied().filter(|w| ok(*w)).sum();
    if total <= 0.0 {
        return None;
    }
    let mut at = rng.next_f64() * total;
    let mut last = None;
    for (i, &w) in weights.iter().enumerate() {
        if !ok(w) {
            continue;
        }
        last = Some(i);
        if at < w {
            return Some(i);
        }
        at -= w;
    }
    // Rounding left a sliver past the last weight: it is the last one's.
    last
}

/// `0..n` shuffled (Fisher-Yates from the end, `n - 1` draws): apply it to
/// a list to shuffle the list.
pub fn permutation(rng: &mut Rng, n: u32) -> Vec<u32> {
    let mut p: Vec<u32> = (0..n).collect();
    for i in (1..n as usize).rev() {
        let j = rng.integer(0, i as i64) as usize;
        p.swap(i, j);
    }
    p
}

// ---- noise -----------------------------------------------------------------

/// A seed's gradient table: 0..255 shuffled by its stream, twice over.
struct Table([u8; 512]);

impl Table {
    fn new(seed: u32) -> Table {
        let p = permutation(&mut Rng::new(seed), 256);
        let mut t = [0u8; 512];
        for i in 0..512 {
            t[i] = p[i & 255] as u8;
        }
        Table(t)
    }
    #[inline]
    fn at(&self, i: i32) -> usize {
        self.0[(i & 511) as usize] as usize
    }
}

thread_local! {
    /// The last few seeds' tables (building one is 255 draws).
    static TABLES: RefCell<Vec<(u32, Arc<Table>)>> = const { RefCell::new(Vec::new()) };
}

fn table(seed: u32) -> Arc<Table> {
    TABLES.with(|t| {
        let mut t = t.borrow_mut();
        if let Some((_, tab)) = t.iter().find(|(s, _)| *s == seed) {
            return tab.clone();
        }
        let tab = Arc::new(Table::new(seed));
        if t.len() >= 8 {
            t.remove(0);
        }
        t.push((seed, tab.clone()));
        tab
    })
}

#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

#[inline]
fn grad3(h: usize, x: f32, y: f32, z: f32) -> f32 {
    // Ken Perlin's improved noise: twelve edge gradients (sixteen slots).
    let h = h & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

#[inline]
fn grad2(h: usize, x: f32, y: f32) -> f32 {
    // Eight gradients: the axes and the diagonals.
    match h & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        _ => -y,
    }
}

/// Improved Perlin noise at (x, y), about -1 to 1, 0 on every lattice point.
pub fn perlin2(seed: u32, x: f32, y: f32) -> f32 {
    let t = table(seed);
    let (fx, fy) = (x.floor(), y.floor());
    let (xi, yi) = (fx as i32, fy as i32);
    let (x, y) = (x - fx, y - fy);
    let (u, v) = (fade(x), fade(y));
    let a = t.at(xi) as i32 + yi;
    let b = t.at(xi + 1) as i32 + yi;
    let (aa, ab, ba, bb) = (t.at(a), t.at(a + 1), t.at(b), t.at(b + 1));
    let n = lerp(
        lerp(grad2(aa, x, y), grad2(ba, x - 1.0, y), u),
        lerp(grad2(ab, x, y - 1.0), grad2(bb, x - 1.0, y - 1.0), u),
        v,
    );
    // The diagonal gradients reach about ±1.4 between lattice points.
    n * 0.7
}

/// Improved Perlin noise at (x, y, z), about -1 to 1.
pub fn perlin3(seed: u32, x: f32, y: f32, z: f32) -> f32 {
    let t = table(seed);
    let (fx, fy, fz) = (x.floor(), y.floor(), z.floor());
    let (xi, yi, zi) = (fx as i32, fy as i32, fz as i32);
    let (x, y, z) = (x - fx, y - fy, z - fz);
    let (u, v, w) = (fade(x), fade(y), fade(z));
    let a = t.at(xi) as i32 + yi;
    let aa = t.at(a) as i32 + zi;
    let ab = t.at(a + 1) as i32 + zi;
    let b = t.at(xi + 1) as i32 + yi;
    let ba = t.at(b) as i32 + zi;
    let bb = t.at(b + 1) as i32 + zi;
    lerp(
        lerp(
            lerp(grad3(t.at(aa), x, y, z), grad3(t.at(ba), x - 1.0, y, z), u),
            lerp(
                grad3(t.at(ab), x, y - 1.0, z),
                grad3(t.at(bb), x - 1.0, y - 1.0, z),
                u,
            ),
            v,
        ),
        lerp(
            lerp(
                grad3(t.at(aa + 1), x, y, z - 1.0),
                grad3(t.at(ba + 1), x - 1.0, y, z - 1.0),
                u,
            ),
            lerp(
                grad3(t.at(ab + 1), x, y - 1.0, z - 1.0),
                grad3(t.at(bb + 1), x - 1.0, y - 1.0, z - 1.0),
                u,
            ),
            v,
        ),
        w,
    )
}

/// Simplex noise at (x, y) (Gustavson's construction), about -1 to 1.
pub fn simplex2(seed: u32, x: f32, y: f32) -> f32 {
    // (sqrt(3) - 1) / 2 and (3 - sqrt(3)) / 6, as f32 literals.
    const F2: f32 = 0.366_025_42;
    const G2: f32 = 0.211_324_87;
    let t = table(seed);
    let s = (x + y) * F2;
    let (i, j) = ((x + s).floor(), (y + s).floor());
    let tt = (i + j) * G2;
    let (x0, y0) = (x - (i - tt), y - (j - tt));
    let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
    let (x1, y1) = (x0 - i1 as f32 + G2, y0 - j1 as f32 + G2);
    let (x2, y2) = (x0 - 1.0 + 2.0 * G2, y0 - 1.0 + 2.0 * G2);
    let (ii, jj) = (i as i32 & 255, j as i32 & 255);
    let corner = |dx: f32, dy: f32, h: usize| {
        let r = 0.5 - dx * dx - dy * dy;
        if r < 0.0 {
            0.0
        } else {
            let r2 = r * r;
            r2 * r2 * grad2(h, dx, dy)
        }
    };
    let n0 = corner(x0, y0, t.at(ii + t.at(jj) as i32));
    let n1 = corner(x1, y1, t.at(ii + i1 + t.at(jj + j1) as i32));
    let n2 = corner(x2, y2, t.at(ii + 1 + t.at(jj + 1) as i32));
    // Scaled so the range is about -1 to 1.
    45.23 * (n0 + n1 + n2)
}

/// Simplex noise at (x, y, z), about -1 to 1.
pub fn simplex3(seed: u32, x: f32, y: f32, z: f32) -> f32 {
    const F3: f32 = 1.0 / 3.0;
    const G3: f32 = 1.0 / 6.0;
    let t = table(seed);
    let s = (x + y + z) * F3;
    let (i, j, k) = ((x + s).floor(), (y + s).floor(), (z + s).floor());
    let tt = (i + j + k) * G3;
    let (x0, y0, z0) = (x - (i - tt), y - (j - tt), z - (k - tt));
    let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
        if y0 >= z0 {
            (1, 0, 0, 1, 1, 0)
        } else if x0 >= z0 {
            (1, 0, 0, 1, 0, 1)
        } else {
            (0, 0, 1, 1, 0, 1)
        }
    } else if y0 < z0 {
        (0, 0, 1, 0, 1, 1)
    } else if x0 < z0 {
        (0, 1, 0, 0, 1, 1)
    } else {
        (0, 1, 0, 1, 1, 0)
    };
    let offs = [
        (0, 0, 0, 0.0),
        (i1, j1, k1, G3),
        (i2, j2, k2, 2.0 * G3),
        (1, 1, 1, 3.0 * G3),
    ];
    let (ii, jj, kk) = (i as i32 & 255, j as i32 & 255, k as i32 & 255);
    let mut n = 0.0;
    for (oi, oj, ok, g) in offs {
        let (dx, dy, dz) = (x0 - oi as f32 + g, y0 - oj as f32 + g, z0 - ok as f32 + g);
        let r = 0.6 - dx * dx - dy * dy - dz * dz;
        if r > 0.0 {
            let h = t.at(ii + oi + t.at(jj + oj + t.at(kk + ok) as i32) as i32);
            let r2 = r * r;
            n += r2 * r2 * grad3(h, dx, dy, dz);
        }
    }
    32.0 * n
}

/// Which noise an [`Fbm`] sums.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseKind {
    Perlin,
    Simplex,
}

/// Fractal Brownian motion: `octaves` layers of noise, each `lacunarity`
/// times the frequency and `gain` times the amplitude of the one before,
/// normalised by the amplitudes' sum (so about -1 to 1). Each octave reads
/// its own table (the seed plus its index), so octaves do not line up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fbm {
    pub seed: u32,
    pub kind: NoiseKind,
    pub octaves: u32,
    pub frequency: f32,
    pub lacunarity: f32,
    pub gain: f32,
}

impl Default for Fbm {
    fn default() -> Self {
        Fbm {
            seed: 0,
            kind: NoiseKind::Perlin,
            octaves: 5,
            frequency: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        }
    }
}

impl Fbm {
    /// The sum at (x, y, z); `flat` reads 2D noise and ignores z.
    pub fn at(&self, x: f32, y: f32, z: f32, flat: bool) -> f32 {
        let (mut sum, mut amp, mut norm, mut f) = (0.0f32, 1.0f32, 0.0f32, self.frequency);
        for o in 0..self.octaves.clamp(1, 16) {
            let s = self.seed.wrapping_add(o.wrapping_mul(0x9e37_79b9));
            let n = match (self.kind, flat) {
                (NoiseKind::Perlin, true) => perlin2(s, x * f, y * f),
                (NoiseKind::Perlin, false) => perlin3(s, x * f, y * f, z * f),
                (NoiseKind::Simplex, true) => simplex2(s, x * f, y * f),
                (NoiseKind::Simplex, false) => simplex3(s, x * f, y * f, z * f),
            };
            sum += n * amp;
            norm += amp;
            amp *= self.gain;
            f *= self.lacunarity;
        }
        if norm > 0.0 { sum / norm } else { 0.0 }
    }

    /// A `width` × `height` grid, row-major from (x0, y0), `step` apart,
    /// at height z (2D noise when `flat`).
    #[allow(clippy::too_many_arguments)]
    pub fn grid(
        &self,
        width: u32,
        height: u32,
        x0: f32,
        y0: f32,
        step: f32,
        z: f32,
        flat: bool,
    ) -> Vec<f32> {
        let mut out = Vec::with_capacity((width * height) as usize);
        for j in 0..height {
            for i in 0..width {
                out.push(self.at(x0 + i as f32 * step, y0 + j as f32 * step, z, flat));
            }
        }
        out
    }
}

// ---- Poisson disk ------------------------------------------------------------

/// Points in [0, width) × [0, height), none closer than `radius` to
/// another, every gap filled to within twice it (Bridson 2007): `attempts`
/// candidates around each active point, the active list taken from by a
/// draw, at most `max` points. Deterministic for a seed. Refused for an
/// area of more than 16,777,216 cells of radius / √2 (a radius too small
/// for the area).
pub fn poisson_disk(
    seed: u32,
    width: f32,
    height: f32,
    radius: f32,
    attempts: u32,
    max: usize,
) -> Result<Vec<[f32; 2]>> {
    if !(width > 0.0 && height > 0.0 && radius > 0.0 && width.is_finite() && height.is_finite()) {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "Poisson disk: width {width}, height {height} and radius {radius} must be finite and above 0"
            ),
        ));
    }
    let cell = radius / std::f32::consts::SQRT_2;
    let (gw, gh) = (
        (width / cell).floor() as usize + 1,
        (height / cell).floor() as usize + 1,
    );
    if gw.saturating_mul(gh) > 1 << 24 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "Poisson disk: radius {radius} over {width} x {height} needs {gw} x {gh} grid cells; at most 16,777,216"
            ),
        ));
    }
    if max == 0 {
        return Ok(Vec::new());
    }
    let mut rng = Rng::new(seed);
    let mut grid: Vec<u32> = vec![u32::MAX; gw * gh];
    let at = |p: [f32; 2]| {
        (
            ((p[0] / cell) as usize).min(gw - 1),
            ((p[1] / cell) as usize).min(gh - 1),
        )
    };
    let mut points: Vec<[f32; 2]> = Vec::new();
    let mut active: Vec<u32> = Vec::new();
    let first = [rng.range_f32(0.0, width), rng.range_f32(0.0, height)];
    let (cx, cy) = at(first);
    grid[cy * gw + cx] = 0;
    points.push(first);
    active.push(0);
    let r2 = radius * radius;
    while !active.is_empty() && points.len() < max {
        let k = rng.integer(0, active.len() as i64 - 1) as usize;
        let base = points[active[k] as usize];
        let mut placed = false;
        for _ in 0..attempts.max(1) {
            // A point in the annulus r..2r: a direction from a unit square
            // draw (rejected outside the disc), a distance from another.
            let (mut dx, mut dy);
            loop {
                dx = rng.range_f32(-1.0, 1.0);
                dy = rng.range_f32(-1.0, 1.0);
                let l = dx * dx + dy * dy;
                if l > 1e-6 && l <= 1.0 {
                    let inv = 1.0 / l.sqrt();
                    dx *= inv;
                    dy *= inv;
                    break;
                }
            }
            let d = radius * (1.0 + rng.range_f32(0.0, 1.0));
            let p = [base[0] + dx * d, base[1] + dy * d];
            if !(p[0] >= 0.0 && p[0] < width && p[1] >= 0.0 && p[1] < height) {
                continue;
            }
            let (px, py) = at(p);
            let mut clear = true;
            'near: for gy in py.saturating_sub(2)..(py + 3).min(gh) {
                for gx in px.saturating_sub(2)..(px + 3).min(gw) {
                    let q = grid[gy * gw + gx];
                    if q != u32::MAX {
                        let o = points[q as usize];
                        let (ex, ey) = (o[0] - p[0], o[1] - p[1]);
                        if ex * ex + ey * ey < r2 {
                            clear = false;
                            break 'near;
                        }
                    }
                }
            }
            if clear {
                grid[py * gw + px] = points.len() as u32;
                active.push(points.len() as u32);
                points.push(p);
                placed = true;
                break;
            }
        }
        if !placed {
            active.swap_remove(k);
        }
    }
    Ok(points)
}

// ---- BSP rooms -----------------------------------------------------------------

/// A rectangle of cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// What [`bsp_rooms`] makes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BspLayout {
    /// One room per leaf, in the tree's order (left before right).
    pub rooms: Vec<CellRect>,
    /// Corridor segments, each a straight run `corridor` cells wide.
    pub corridors: Vec<CellRect>,
    /// `width` × `height`, row-major from the top: 0 wall, 1 room, 2 corridor.
    pub cells: Vec<u8>,
}

/// What [`bsp_rooms`] is asked for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BspDesc {
    pub seed: u32,
    pub width: u32,
    pub height: u32,
    /// The smallest room side.
    pub min_room: u32,
    /// The largest room side.
    pub max_room: u32,
    /// How many times to split at most.
    pub depth: u32,
    /// Wall cells kept between a room and its leaf's edge.
    pub padding: u32,
    /// Corridor width.
    pub corridor: u32,
}

impl Default for BspDesc {
    fn default() -> Self {
        BspDesc {
            seed: 0,
            width: 48,
            height: 32,
            min_room: 4,
            max_room: 12,
            depth: 5,
            padding: 1,
            corridor: 1,
        }
    }
}

/// A dungeon by binary space partition: the area is split in two (along
/// its longer side, at a drawn position leaving both halves room for a
/// room) to `depth` levels or until a half would be too small; each leaf
/// gets a room of drawn size and place inside its padding; the two halves
/// of every split are joined by an L-shaped corridor between the centres
/// of a room on each side (the horizontal or vertical leg first by a
/// draw). Draws in depth-first order, left half first.
pub fn bsp_rooms(desc: &BspDesc) -> Result<BspLayout> {
    let (w, h) = (desc.width, desc.height);
    if w == 0 || h == 0 || w > 4096 || h > 4096 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("BSP rooms: the area is {w} x {h}; each side must be 1 to 4096 cells"),
        ));
    }
    let min_room = desc.min_room.max(1);
    let max_room = desc.max_room.max(min_room);
    let pad = desc.padding;
    let leaf_min = min_room + 2 * pad;
    if w < leaf_min || h < leaf_min {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "BSP rooms: a {w} x {h} area cannot hold a {min_room}-cell room with {pad} padding"
            ),
        ));
    }
    let mut rng = Rng::new(desc.seed);
    let mut out = BspLayout {
        rooms: Vec::new(),
        corridors: Vec::new(),
        cells: vec![0; (w * h) as usize],
    };
    split(
        &mut rng,
        desc,
        CellRect {
            x: 0,
            y: 0,
            width: w,
            height: h,
        },
        desc.depth.min(16),
        min_room,
        max_room,
        leaf_min,
        &mut out,
    );
    for r in &out.rooms {
        paint(&mut out.cells, w, *r, 1);
    }
    for c in &out.corridors {
        paint(&mut out.cells, w, *c, 2);
    }
    // Rooms over corridors where they cross.
    for r in &out.rooms {
        paint(&mut out.cells, w, *r, 1);
    }
    Ok(out)
}

fn paint(cells: &mut [u8], w: u32, r: CellRect, v: u8) {
    for y in r.y..r.y + r.height {
        for x in r.x..r.x + r.width {
            cells[(y * w + x) as usize] = v;
        }
    }
}

/// Splits `area`, returning the indices of its rooms in `out.rooms`.
#[allow(clippy::too_many_arguments)]
fn split(
    rng: &mut Rng,
    desc: &BspDesc,
    area: CellRect,
    depth: u32,
    min_room: u32,
    max_room: u32,
    leaf_min: u32,
    out: &mut BspLayout,
) -> std::ops::Range<usize> {
    let start = out.rooms.len();
    let along_x = area.width >= area.height;
    let side = if along_x { area.width } else { area.height };
    let roomy = side >= 2 * leaf_min;
    let big = side > max_room + 2 * desc.padding;
    if depth > 0 && roomy && (big || rng.next_f64() < 0.5) {
        let at = rng.integer(i64::from(leaf_min), i64::from(side - leaf_min)) as u32;
        let (a, b) = if along_x {
            (
                CellRect { width: at, ..area },
                CellRect {
                    x: area.x + at,
                    width: area.width - at,
                    ..area
                },
            )
        } else {
            (
                CellRect { height: at, ..area },
                CellRect {
                    y: area.y + at,
                    height: area.height - at,
                    ..area
                },
            )
        };
        let left = split(rng, desc, a, depth - 1, min_room, max_room, leaf_min, out);
        let right = split(rng, desc, b, depth - 1, min_room, max_room, leaf_min, out);
        // Join the room of each side nearest the split (ties: the first).
        let key = |r: &CellRect| {
            if along_x {
                r.x + r.width / 2
            } else {
                r.y + r.height / 2
            }
        };
        let ia = left
            .clone()
            .max_by_key(|&i| (key(&out.rooms[i]), std::cmp::Reverse(i)))
            .expect("every split side holds a room");
        let ib = right
            .clone()
            .min_by_key(|&i| (key(&out.rooms[i]), i))
            .expect("every split side holds a room");
        let (ra, rb) = (out.rooms[ia], out.rooms[ib]);
        let (ax, ay) = (ra.x + ra.width / 2, ra.y + ra.height / 2);
        let (bx, by) = (rb.x + rb.width / 2, rb.y + rb.height / 2);
        let cw = desc.corridor.max(1);
        let hseg = |y: u32, x0: u32, x1: u32| CellRect {
            x: x0.min(x1),
            y: y.min(area.y + area.height - cw),
            width: x0.abs_diff(x1) + cw,
            height: cw,
        };
        let vseg = |x: u32, y0: u32, y1: u32| CellRect {
            x: x.min(area.x + area.width - cw),
            y: y0.min(y1),
            width: cw,
            height: y0.abs_diff(y1) + cw,
        };
        let clip = |r: CellRect| CellRect {
            width: r.width.min(desc.width - r.x),
            height: r.height.min(desc.height - r.y),
            ..r
        };
        if rng.next_f64() < 0.5 {
            out.corridors.push(clip(hseg(ay, ax, bx)));
            out.corridors.push(clip(vseg(bx, ay, by)));
        } else {
            out.corridors.push(clip(vseg(ax, ay, by)));
            out.corridors.push(clip(hseg(by, ax, bx)));
        }
        return start..right.end;
    }
    // A leaf: one room inside its padding.
    let pad = desc.padding;
    let room_w = rng.integer(
        i64::from(min_room),
        i64::from(max_room.min(area.width - 2 * pad)),
    ) as u32;
    let room_h = rng.integer(
        i64::from(min_room),
        i64::from(max_room.min(area.height - 2 * pad)),
    ) as u32;
    let x = area.x + pad + rng.integer(0, i64::from(area.width - 2 * pad - room_w)) as u32;
    let y = area.y + pad + rng.integer(0, i64::from(area.height - 2 * pad - room_h)) as u32;
    out.rooms.push(CellRect {
        x,
        y,
        width: room_w,
        height: room_h,
    });
    start..start + 1
}

// ---- wave function collapse ---------------------------------------------------

/// A tile of a [`Wfc`]: its weight and the socket on each edge (up, right,
/// down, left); two tiles may sit side by side when the sockets they turn
/// to each other are equal. With `rotations` it also comes turned 90°,
/// 180° and 270° clockwise (its sockets turned with it).
#[derive(Clone, Debug, PartialEq)]
pub struct WfcTile {
    pub weight: f32,
    pub sockets: [u32; 4],
    pub rotations: bool,
}

/// What [`wfc`] is asked for.
#[derive(Clone, Debug, PartialEq)]
pub struct Wfc {
    pub seed: u32,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<WfcTile>,
    /// Cells fixed before the first draw: (x, y, tile, rotation).
    pub fixed: Vec<(u32, u32, u32, u32)>,
    /// Edges wrap around (a tiling texture).
    pub wrap: bool,
    /// Tries before refusing, each on the seed's fork `attempt-<n>`.
    pub attempts: u32,
}

/// What [`wfc`] made: per cell (row-major from the top) its tile and rotation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WfcOut {
    pub tiles: Vec<u32>,
    pub rotations: Vec<u32>,
    /// The try that succeeded, from 1.
    pub attempt: u32,
}

/// Wave function collapse, the simple tiled model: every cell starts as
/// every variant (a tile and a rotation); the cell with the fewest left
/// (ties: the lowest index) collapses to one drawn by weight, and the
/// choice propagates to every neighbour until nothing changes. A cell left
/// with none is a contradiction: the try restarts on a fork of the seed, up
/// to `attempts` tries, and then it is refused naming the cell.
pub fn wfc(desc: &Wfc) -> Result<WfcOut> {
    let (w, h) = (desc.width as usize, desc.height as usize);
    if w == 0 || h == 0 || w * h > 1 << 20 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "WFC: the grid is {} x {}; it must have 1 to 1,048,576 cells",
                desc.width, desc.height
            ),
        ));
    }
    if desc.tiles.is_empty() {
        return Err(Error::new(ErrorKind::Invalid, "WFC: no tiles"));
    }
    // Variants: (tile, rotation, sockets turned, weight).
    let mut variants: Vec<(u32, u32, [u32; 4], f64)> = Vec::new();
    for (i, t) in desc.tiles.iter().enumerate() {
        let turns = if t.rotations { 4 } else { 1 };
        let mut seen: Vec<[u32; 4]> = Vec::new();
        for r in 0..turns {
            // Turned r quarter turns clockwise: the up edge shows what the
            // left edge showed, and so on round.
            let s = t.sockets;
            let turned = [
                s[(4 - r as usize) % 4],
                s[(5 - r as usize) % 4],
                s[(6 - r as usize) % 4],
                s[(7 - r as usize) % 4],
            ];
            if seen.contains(&turned) {
                continue;
            }
            seen.push(turned);
            variants.push((i as u32, r, turned, f64::from(t.weight.max(0.0))));
        }
    }
    let n = variants.len();
    if n > 1024 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("WFC: {n} tile variants; at most 1,024"),
        ));
    }
    // compat[d][a] = the variants b may sit in direction d of a.
    let words = n.div_ceil(64);
    let mut compat = vec![vec![vec![0u64; words]; n]; 4];
    for (d, rows) in compat.iter_mut().enumerate() {
        for (a, row) in rows.iter_mut().enumerate() {
            for (b, vb) in variants.iter().enumerate() {
                if variants[a].2[d] == vb.2[(d + 2) % 4] {
                    row[b / 64] |= 1 << (b % 64);
                }
            }
        }
    }
    let mut fixed: Vec<(usize, usize)> = Vec::new();
    for &(x, y, tile, rot) in &desc.fixed {
        if x >= desc.width || y >= desc.height {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("WFC: fixed cell ({x}, {y}) is outside the grid"),
            ));
        }
        let v = variants
            .iter()
            .position(|v| v.0 == tile && v.1 == rot % 4)
            .or_else(|| variants.iter().position(|v| v.0 == tile))
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Invalid,
                    format!("WFC: fixed cell ({x}, {y}) names tile {tile}, past the last"),
                )
            })?;
        fixed.push((y as usize * w + x as usize, v));
    }
    let root = Rng::new(desc.seed);
    let mut last = (0, 0);
    for attempt in 1..=desc.attempts.max(1) {
        let mut rng = if attempt == 1 {
            root
        } else {
            root.fork(&format!("attempt-{attempt}"))
        };
        match collapse(&mut rng, w, h, desc.wrap, &variants, &compat, &fixed) {
            Ok(cells) => {
                return Ok(WfcOut {
                    tiles: cells.iter().map(|&v| variants[v].0).collect(),
                    rotations: cells.iter().map(|&v| variants[v].1).collect(),
                    attempt,
                });
            }
            Err(cell) => last = (cell % w, cell / w),
        }
    }
    Err(Error::new(
        ErrorKind::Invalid,
        format!(
            "WFC: no tiling in {} tries; the last ran out of tiles at cell ({}, {}): the sockets may not fit",
            desc.attempts.max(1),
            last.0,
            last.1
        ),
    ))
}

/// One try; `Err(cell)` names the cell left with no variant.
fn collapse(
    rng: &mut Rng,
    w: usize,
    h: usize,
    wrap: bool,
    variants: &[(u32, u32, [u32; 4], f64)],
    compat: &[Vec<Vec<u64>>],
    fixed: &[(usize, usize)],
) -> std::result::Result<Vec<usize>, usize> {
    let n = variants.len();
    let words = n.div_ceil(64);
    let mut full = vec![!0u64; words];
    if !n.is_multiple_of(64) {
        full[words - 1] = (1u64 << (n % 64)) - 1;
    }
    let cells = w * h;
    let mut wave: Vec<u64> = Vec::with_capacity(cells * words);
    for _ in 0..cells {
        wave.extend_from_slice(&full);
    }
    let count = |wave: &[u64], c: usize| -> u32 {
        wave[c * words..(c + 1) * words]
            .iter()
            .map(|x| x.count_ones())
            .sum()
    };
    let neighbour = |c: usize, d: usize| -> Option<usize> {
        let (x, y) = ((c % w) as i64, (c / w) as i64);
        let (dx, dy) = [(0, -1), (1, 0), (0, 1), (-1, 0)][d];
        let (mut nx, mut ny) = (x + dx, y + dy);
        if wrap {
            nx = nx.rem_euclid(w as i64);
            ny = ny.rem_euclid(h as i64);
        } else if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
            return None;
        }
        Some(ny as usize * w + nx as usize)
    };
    let mut stack: Vec<usize> = Vec::new();
    let mut allowed = vec![0u64; words];
    let propagate = |wave: &mut Vec<u64>,
                     stack: &mut Vec<usize>,
                     allowed: &mut Vec<u64>|
     -> std::result::Result<(), usize> {
        while let Some(c) = stack.pop() {
            for (d, cd) in compat.iter().enumerate() {
                let Some(nb) = neighbour(c, d) else { continue };
                allowed.fill(0);
                for v in 0..n {
                    if wave[c * words + v / 64] >> (v % 64) & 1 == 1 {
                        for (a, b) in allowed.iter_mut().zip(&cd[v]) {
                            *a |= *b;
                        }
                    }
                }
                let mut changed = false;
                let mut any = false;
                for k in 0..words {
                    let old = wave[nb * words + k];
                    let new = old & allowed[k];
                    if new != old {
                        changed = true;
                    }
                    any |= new != 0;
                    wave[nb * words + k] = new;
                }
                if !any {
                    return Err(nb);
                }
                if changed {
                    stack.push(nb);
                }
            }
        }
        Ok(())
    };
    // Every cell's constraints on its neighbours apply before the first
    // draw: with a single variant every cell starts decided.
    stack.extend((0..cells).rev());
    for &(c, v) in fixed {
        for k in 0..words {
            wave[c * words + k] = 0;
        }
        wave[c * words + v / 64] |= 1 << (v % 64);
        stack.push(c);
    }
    propagate(&mut wave, &mut stack, &mut allowed)?;
    let mut weights = vec![0f64; n];
    loop {
        // The undecided cell with the fewest variants left, lowest index first.
        let mut best: Option<(u32, usize)> = None;
        for c in 0..cells {
            let k = count(&wave, c);
            if k > 1 && best.is_none_or(|b| k < b.0) {
                best = Some((k, c));
            }
        }
        let Some((_, c)) = best else { break };
        for (v, wt) in weights.iter_mut().enumerate() {
            *wt = if wave[c * words + v / 64] >> (v % 64) & 1 == 1 {
                // A zero weight still stands in when nothing else is left.
                variants[v].3.max(1e-9)
            } else {
                0.0
            };
        }
        let v = weighted(rng, &weights).ok_or(c)?;
        for k in 0..words {
            wave[c * words + k] = 0;
        }
        wave[c * words + v / 64] |= 1 << (v % 64);
        stack.push(c);
        propagate(&mut wave, &mut stack, &mut allowed)?;
    }
    Ok((0..cells)
        .map(|c| {
            (0..n)
                .find(|&v| wave[c * words + v / 64] >> (v % 64) & 1 == 1)
                .expect("a decided cell has one variant")
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weighted_draws_follow_their_weights_and_skip_nonpositive_ones() {
        let mut rng = Rng::new(7);
        let mut seen = [0u32; 4];
        for _ in 0..40_000 {
            seen[weighted(&mut rng, &[1.0, 0.0, 3.0, -2.0]).unwrap()] += 1;
        }
        assert_eq!((seen[1], seen[3]), (0, 0));
        let share = f64::from(seen[2]) / 40_000.0;
        assert!((share - 0.75).abs() < 0.01, "{share}");
        assert_eq!(weighted(&mut rng, &[0.0, f64::NAN]), None);
        assert_eq!(weighted(&mut rng, &[]), None);
    }

    #[test]
    fn a_permutation_holds_every_index_once_and_varies_with_the_seed() {
        let p = permutation(&mut Rng::new(1), 100);
        let mut s = p.clone();
        s.sort_unstable();
        assert_eq!(s, (0..100).collect::<Vec<_>>());
        assert_ne!(p, permutation(&mut Rng::new(2), 100));
        assert_ne!(p, (0..100).collect::<Vec<_>>());
        assert!(permutation(&mut Rng::new(1), 0).is_empty());
        assert_eq!(permutation(&mut Rng::new(1), 1), [0]);
    }

    #[test]
    fn noise_is_zero_on_the_lattice_continuous_and_in_range() {
        for seed in [0, 1, 99] {
            assert_eq!(perlin2(seed, 3.0, -7.0), 0.0);
            assert_eq!(perlin3(seed, 3.0, -7.0, 2.0), 0.0);
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for i in 0..4000 {
                let (x, y) = (i as f32 * 0.137, i as f32 * 0.071 - 50.0);
                for n in [
                    perlin2(seed, x, y),
                    perlin3(seed, x, y, 0.3),
                    simplex2(seed, x, y),
                    simplex3(seed, x, y, 0.3),
                ] {
                    lo = lo.min(n);
                    hi = hi.max(n);
                }
                // Continuous: a tiny step moves it a tiny amount.
                let d = (simplex2(seed, x, y) - simplex2(seed, x + 1e-3, y)).abs();
                assert!(d < 0.02, "simplex jumps {d} at {x} {y}");
                let d = (perlin2(seed, x, y) - perlin2(seed, x + 1e-3, y)).abs();
                assert!(d < 0.02, "perlin jumps {d} at {x} {y}");
            }
            assert!(lo > -1.05 && hi < 1.05, "{lo} {hi}");
            assert!(lo < -0.5 && hi > 0.5, "it uses its range: {lo} {hi}");
        }
        assert_ne!(perlin2(1, 0.5, 0.5), perlin2(2, 0.5, 0.5));
    }

    #[test]
    fn fbm_sums_octaves_and_a_grid_reads_the_same_points() {
        let f = Fbm {
            seed: 4,
            octaves: 4,
            ..Fbm::default()
        };
        let g = f.grid(8, 4, 1.0, 2.0, 0.25, 0.0, true);
        assert_eq!(g.len(), 32);
        assert_eq!(g[9], f.at(1.25, 2.25, 0.0, true));
        let one = Fbm { octaves: 1, ..f };
        assert_eq!(one.at(0.3, 0.6, 0.0, true), perlin2(4, 0.3, 0.6));
    }

    #[test]
    fn poisson_points_keep_their_distance_and_fill_the_area() {
        let pts = poisson_disk(3, 40.0, 30.0, 2.0, 30, 10_000).unwrap();
        for (i, a) in pts.iter().enumerate() {
            assert!(a[0] >= 0.0 && a[0] < 40.0 && a[1] >= 0.0 && a[1] < 30.0);
            for b in &pts[i + 1..] {
                let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
                let d = (dx * dx + dy * dy).sqrt();
                assert!(d >= 2.0, "{a:?} {b:?} {d}");
            }
        }
        // A maximal 2-disc set covers 1,200 m² at between 0.866 r²... and
        // well above the 75 a square grid of 4 m would give.
        assert!(pts.len() > 150 && pts.len() < 320, "{}", pts.len());
        assert_eq!(pts, poisson_disk(3, 40.0, 30.0, 2.0, 30, 10_000).unwrap());
        assert_ne!(pts, poisson_disk(4, 40.0, 30.0, 2.0, 30, 10_000).unwrap());
        assert!(poisson_disk(3, 1e6, 1e6, 0.01, 30, 10).is_err());
        assert_eq!(poisson_disk(3, 40.0, 30.0, 2.0, 30, 10).unwrap().len(), 10);
    }

    #[test]
    fn bsp_rooms_fit_their_leaves_do_not_overlap_and_are_all_joined() {
        let d = BspDesc {
            seed: 5,
            width: 64,
            height: 40,
            ..BspDesc::default()
        };
        let out = bsp_rooms(&d).unwrap();
        assert!(out.rooms.len() >= 6, "{}", out.rooms.len());
        for (i, a) in out.rooms.iter().enumerate() {
            assert!(a.width >= 4 && a.height >= 4 && a.width <= 12 && a.height <= 12);
            assert!(a.x + a.width <= 64 && a.y + a.height <= 40);
            for b in &out.rooms[i + 1..] {
                let apart = a.x + a.width <= b.x
                    || b.x + b.width <= a.x
                    || a.y + a.height <= b.y
                    || b.y + b.height <= a.y;
                assert!(apart, "{a:?} {b:?}");
            }
        }
        // Every floor cell is reachable from the first room's.
        let (w, h) = (64usize, 40usize);
        let r0 = out.rooms[0];
        let mut seen = vec![false; w * h];
        let mut todo = vec![(r0.y * 64 + r0.x) as usize];
        seen[todo[0]] = true;
        while let Some(c) = todo.pop() {
            let (x, y) = (c % w, c / w);
            for (nx, ny) in [
                (x + 1, y),
                (x.wrapping_sub(1), y),
                (x, y + 1),
                (x, y.wrapping_sub(1)),
            ] {
                if nx < w && ny < h && out.cells[ny * w + nx] != 0 && !seen[ny * w + nx] {
                    seen[ny * w + nx] = true;
                    todo.push(ny * w + nx);
                }
            }
        }
        for (i, c) in out.cells.iter().enumerate() {
            assert!(*c == 0 || seen[i], "cell {} {} is cut off", i % w, i / w);
        }
        assert_eq!(out, bsp_rooms(&d).unwrap());
        assert_ne!(out, bsp_rooms(&BspDesc { seed: 6, ..d }).unwrap());
        assert!(bsp_rooms(&BspDesc { width: 3, ..d }).is_err());
    }

    fn pipes() -> Vec<WfcTile> {
        // Sockets: 0 empty, 1 pipe. A blank, a straight and a corner.
        vec![
            WfcTile {
                weight: 2.0,
                sockets: [0, 0, 0, 0],
                rotations: false,
            },
            WfcTile {
                weight: 1.0,
                sockets: [1, 0, 1, 0],
                rotations: true,
            },
            WfcTile {
                weight: 1.0,
                sockets: [1, 1, 0, 0],
                rotations: true,
            },
        ]
    }

    #[test]
    fn wfc_tiles_fit_their_neighbours_and_hold_fixed_cells() {
        let desc = Wfc {
            seed: 9,
            width: 20,
            height: 12,
            tiles: pipes(),
            fixed: vec![(3, 4, 2, 1)],
            wrap: false,
            attempts: 5,
        };
        let out = wfc(&desc).unwrap();
        let sockets = |c: usize| {
            let t = &desc.tiles[out.tiles[c] as usize];
            let r = out.rotations[c] as usize;
            let s = t.sockets;
            [
                s[(4 - r) % 4],
                s[(5 - r) % 4],
                s[(6 - r) % 4],
                s[(7 - r) % 4],
            ]
        };
        for y in 0..12usize {
            for x in 0..20usize {
                let c = y * 20 + x;
                if x + 1 < 20 {
                    assert_eq!(sockets(c)[1], sockets(c + 1)[3], "({x},{y}) right");
                }
                if y + 1 < 12 {
                    assert_eq!(sockets(c)[2], sockets(c + 20)[0], "({x},{y}) down");
                }
            }
        }
        assert_eq!((out.tiles[4 * 20 + 3], out.rotations[4 * 20 + 3]), (2, 1));
        assert!(out.tiles.contains(&1), "some straights");
        assert_eq!(out, wfc(&desc).unwrap());
        assert_ne!(
            out.tiles,
            wfc(&Wfc {
                seed: 10,
                ..desc.clone()
            })
            .unwrap()
            .tiles
        );
    }

    #[test]
    fn wfc_refuses_tiles_that_cannot_fit_naming_a_cell() {
        // A tile whose right edge matches no left edge cannot have a right neighbour.
        let desc = Wfc {
            seed: 1,
            width: 3,
            height: 1,
            tiles: vec![WfcTile {
                weight: 1.0,
                sockets: [0, 1, 0, 2],
                rotations: false,
            }],
            fixed: vec![],
            wrap: false,
            attempts: 3,
        };
        let e = wfc(&desc).unwrap_err();
        assert!(
            e.reason().contains("3 tries") && e.reason().contains("cell ("),
            "{}",
            e.reason()
        );
    }

    /// FNV-1a over the generators' outputs: the numbers a game builds a
    /// level from. Any change to a draw order, a hash table or the float
    /// arithmetic moves them, and a moved one means every seeded world in
    /// every game changes; the web host must give the same (the bun test
    /// in packages/engine/tests/modules.test.ts pins the same values).
    #[test]
    fn every_generator_gives_its_pinned_output_for_a_seed() {
        fn fnv(bytes: impl IntoIterator<Item = u8>) -> u64 {
            bytes.into_iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
                (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
            })
        }
        let mut r = Rng::new(42);
        let picks: Vec<u8> = (0..64)
            .map(|_| weighted(&mut r, &[1.0, 0.0, 3.0, 0.5]).unwrap() as u8)
            .collect();
        let perm = permutation(&mut Rng::new(42), 32);
        let f32s = |v: &[f32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
        let grid = |kind, flat| {
            Fbm {
                seed: 7,
                kind,
                octaves: 4,
                frequency: 0.13,
                ..Fbm::default()
            }
            .grid(16, 16, -3.5, 2.25, 0.7, 1.3, flat)
        };
        let poisson = poisson_disk(5, 40.0, 30.0, 3.0, 30, 10_000).unwrap();
        let bsp = bsp_rooms(&BspDesc {
            seed: 3,
            ..BspDesc::default()
        })
        .unwrap();
        let w = wfc(&Wfc {
            seed: 9,
            width: 20,
            height: 12,
            tiles: pipes(),
            fixed: vec![],
            wrap: true,
            attempts: 5,
        })
        .unwrap();
        let got = [
            fnv(picks),
            fnv(perm.iter().flat_map(|x| x.to_le_bytes())),
            fnv(f32s(&grid(NoiseKind::Perlin, true))),
            fnv(f32s(&grid(NoiseKind::Perlin, false))),
            fnv(f32s(&grid(NoiseKind::Simplex, true))),
            fnv(f32s(&grid(NoiseKind::Simplex, false))),
            fnv(poisson.iter().flat_map(|p| f32s(p))),
            fnv(bsp.cells.iter().copied()),
            fnv(w
                .tiles
                .iter()
                .zip(&w.rotations)
                .map(|(t, r)| (t * 4 + r) as u8)),
        ];
        let text: Vec<String> = got.iter().map(|h| format!("{h:016x}")).collect();
        assert_eq!(text, PINNED, "{text:?}");
    }

    const PINNED: [&str; 9] = [
        "23c772a8d720158b",
        "9fe65e8b3d98afa5",
        "f408080a80a69ae1",
        "c99bab4bf56e2da4",
        "f87052e661c5c7be",
        "7988b7773f1605ed",
        "8c0c32d7c29b84cd",
        "4c25af53c3dbd445",
        "c3d44cb51b6ea25a",
    ];
}
