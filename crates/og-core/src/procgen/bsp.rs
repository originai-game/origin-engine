//! Rooms and corridors from a binary space partition.

use crate::{Error, ErrorKind, Result, Rng};

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
    /// Room-to-room corridor connections as `(room_a, room_b)` index pairs.
    pub edges: Vec<[u32; 2]>,
}

/// What [`bsp_rooms`] is asked for.
#[derive(Clone, Debug, PartialEq)]
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
    /// Chance (0 to 1) of adding extra loop corridors between nearby rooms.
    pub loops: f32,
    /// Optional row-major writable cells; zero excludes rooms/corridors.
    pub mask: Vec<u8>,
    /// Authored rooms, retained in order before generated rooms.
    pub rooms: Vec<CellRect>,
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
            loops: 0.0,
            mask: Vec::new(),
            rooms: Vec::new(),
        }
    }
}

/// A dungeon by binary space partition: the area is split in two (along
/// its longer side, at a drawn position leaving both halves room for a
/// room) to `depth` levels or until a half would be too small; each leaf
/// gets a room of drawn size and place inside its padding; the two halves
/// of every split are joined by an L-shaped corridor between the 2D-nearest
/// pair of rooms across the split (the horizontal or vertical leg first by a
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
    let Some(leaf_min) = pad.checked_mul(2).and_then(|p| min_room.checked_add(p)) else {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("BSP rooms: minRoom {min_room} with padding {pad} overflows"),
        ));
    };
    if w < leaf_min || h < leaf_min {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "BSP rooms: a {w} x {h} area cannot hold a {min_room}-cell room with {pad} padding"
            ),
        ));
    }
    let cell_count = u64::from(w) * u64::from(h);
    if !desc.mask.is_empty() && desc.mask.len() as u64 != cell_count {
        return Err(Error::new(
            ErrorKind::Invalid,
            "BSP mask needs width × height cells",
        ));
    }
    if !desc.mask.is_empty() && desc.corridor.max(1) > w.min(h) {
        return Err(Error::new(
            ErrorKind::Invalid,
            "BSP corridor width exceeds the masked area",
        ));
    }
    for (i, r) in desc.rooms.iter().enumerate() {
        if r.width == 0
            || r.height == 0
            || r.x.checked_add(r.width).is_none_or(|x| x > w)
            || r.y.checked_add(r.height).is_none_or(|y| y > h)
            || !writable(desc, *r)
            || desc.rooms[..i].iter().any(|p| overlaps(*p, *r))
        {
            return Err(Error::new(
                ErrorKind::Invalid,
                "BSP retained rooms must be nonempty, disjoint and inside writable cells",
            ));
        }
    }
    let mut rng = Rng::new(desc.seed);
    let mut out = BspLayout {
        rooms: desc.rooms.clone(),
        corridors: Vec::new(),
        cells: vec![0; cell_count as usize],
        edges: Vec::new(),
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
    if out.rooms.is_empty() {
        return Err(Error::new(
            ErrorKind::Invalid,
            "BSP: no writable region can hold a room",
        ));
    }
    for i in 1..if desc.rooms.is_empty() {
        0
    } else {
        out.rooms.len()
    } {
        if let Some(j) = (0..i).min_by_key(|&j| (room_dist2(&out.rooms[i], &out.rooms[j]), j)) {
            connect_rooms(
                &mut rng,
                desc,
                CellRect {
                    x: 0,
                    y: 0,
                    width: w,
                    height: h,
                },
                i,
                j,
                &mut out,
            );
        }
    }

    // Optional extra loop connections between nearby non-connected rooms.
    let loop_chance = if desc.loops.is_finite() {
        desc.loops.clamp(0.0, 1.0)
    } else {
        0.0
    };
    if loop_chance > 0.0 && out.rooms.len() >= 3 {
        let mut candidates: Vec<(u64, usize, usize)> = Vec::new();
        for i in 0..out.rooms.len() {
            for j in (i + 1)..out.rooms.len() {
                if out.edges.iter().any(|&[a, b]| {
                    (a as usize == i && b as usize == j) || (a as usize == j && b as usize == i)
                }) {
                    continue;
                }
                let d2 = room_dist2(&out.rooms[i], &out.rooms[j]);
                candidates.push((d2, i, j));
            }
        }
        candidates.sort_unstable();
        let max_loops = (out.rooms.len() / 2).max(1);
        let mut added = 0usize;
        for &(_, i, j) in candidates.iter().take(out.rooms.len() * 2) {
            if added >= max_loops {
                break;
            }
            if rng.next_f64() < f64::from(loop_chance) {
                connect_rooms(
                    &mut rng,
                    desc,
                    CellRect {
                        x: 0,
                        y: 0,
                        width: w,
                        height: h,
                    },
                    i,
                    j,
                    &mut out,
                );
                added += 1;
            }
        }
    }

    let mut reached = vec![false; out.rooms.len()];
    let mut queue = std::collections::VecDeque::from([0]);
    reached[0] = true;
    while let Some(i) = queue.pop_front() {
        for &[a, b] in &out.edges {
            let j = if a as usize == i {
                b as usize
            } else if b as usize == i {
                a as usize
            } else {
                continue;
            };
            if !reached[j] {
                reached[j] = true;
                queue.push_back(j);
            }
        }
    }
    if reached.contains(&false) {
        return Err(Error::new(
            ErrorKind::Invalid,
            "BSP: no connection found for every room at the requested corridor width",
        ));
    }

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

#[inline]
fn room_center(r: &CellRect) -> (u32, u32) {
    (r.x + r.width / 2, r.y + r.height / 2)
}

#[inline]
fn room_dist2(a: &CellRect, b: &CellRect) -> u64 {
    let (ax, ay) = room_center(a);
    let (bx, by) = room_center(b);
    let dx = u64::from(ax.abs_diff(bx));
    let dy = u64::from(ay.abs_diff(by));
    dx * dx + dy * dy
}

fn overlaps(a: CellRect, b: CellRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

fn writable(d: &BspDesc, r: CellRect) -> bool {
    d.mask.is_empty()
        || (r.y..r.y + r.height)
            .all(|y| (r.x..r.x + r.width).all(|x| d.mask[(y * d.width + x) as usize] != 0))
}

fn connect_rooms(
    rng: &mut Rng,
    desc: &BspDesc,
    area: CellRect,
    ia: usize,
    ib: usize,
    out: &mut BspLayout,
) {
    if out.edges.iter().any(|&[a, b]| {
        (a as usize == ia && b as usize == ib) || (a as usize == ib && b as usize == ia)
    }) {
        return;
    }
    let (ra, rb) = (out.rooms[ia], out.rooms[ib]);
    let (ax, ay) = room_center(&ra);
    let (bx, by) = room_center(&rb);
    if !desc.mask.is_empty() {
        // A shortest four-neighbour route of corridor-width squares, not
        // a straight segment clipped through an exclusion mask. Start at
        // every writable square overlapping the first room, not an
        // arbitrarily offset centre square which might hit an exclusion.
        let cw = desc.corridor.max(1);
        let side = desc.width as usize;
        let rect = |k: usize| CellRect {
            x: (k % side) as u32,
            y: (k / side) as u32,
            width: cw,
            height: cw,
        };
        let mut previous = vec![usize::MAX; desc.width as usize * desc.height as usize];
        let mut queue = std::collections::VecDeque::new();
        for y in ra.y.saturating_sub(cw - 1)..=(ra.y + ra.height - 1).min(desc.height - cw) {
            for x in ra.x.saturating_sub(cw - 1)..=(ra.x + ra.width - 1).min(desc.width - cw) {
                let k = y as usize * side + x as usize;
                if writable(desc, rect(k)) {
                    previous[k] = k;
                    queue.push_back(k);
                }
            }
        }
        let mut end = None;
        while let Some(k) = queue.pop_front() {
            if overlaps(rect(k), rb) {
                end = Some(k);
                break;
            }
            let [x, y] = [(k % side) as u32, (k / side) as u32];
            for (nx, ny) in [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ] {
                if nx > desc.width - cw || ny > desc.height - cw {
                    continue;
                }
                let next = ny as usize * side + nx as usize;
                if previous[next] == usize::MAX && writable(desc, rect(next)) {
                    previous[next] = k;
                    queue.push_back(next);
                }
            }
        }
        let Some(end) = end else { return };
        let mut k = end;
        loop {
            out.corridors.push(rect(k));
            if previous[k] == k {
                break;
            }
            k = previous[k];
        }
        out.edges.push([ia as u32, ib as u32]);
        return;
    }
    let cw = desc.corridor.max(1).min(area.width).min(area.height);
    let max_x = area.x + area.width.saturating_sub(cw);
    let max_y = area.y + area.height.saturating_sub(cw);
    let hseg = |y: u32, x0: u32, x1: u32| CellRect {
        x: x0.min(x1),
        y: y.clamp(area.y, max_y),
        width: x0.abs_diff(x1) + cw,
        height: cw,
    };
    let vseg = |x: u32, y0: u32, y1: u32| CellRect {
        x: x.clamp(area.x, max_x),
        y: y0.min(y1),
        width: cw,
        height: y0.abs_diff(y1) + cw,
    };
    let clip = |r: CellRect| CellRect {
        width: r.width.min(desc.width.saturating_sub(r.x)),
        height: r.height.min(desc.height.saturating_sub(r.y)),
        ..r
    };
    if rng.next_f64() < 0.5 {
        out.corridors.push(clip(hseg(ay, ax, bx)));
        out.corridors.push(clip(vseg(bx, ay, by)));
    } else {
        out.corridors.push(clip(vseg(ax, ay, by)));
        out.corridors.push(clip(hseg(by, ax, bx)));
    }
    out.edges.push([ia as u32, ib as u32]);
}

fn paint(cells: &mut [u8], w: u32, r: CellRect, v: u8) {
    let w = w as usize;
    for y in r.y..r.y.saturating_add(r.height) {
        let row = y as usize * w;
        for x in r.x..r.x.saturating_add(r.width) {
            if let Some(cell) = cells.get_mut(row + x as usize) {
                *cell = v;
            }
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
    let roomy = side >= leaf_min.saturating_mul(2);
    let big = side > max_room.saturating_add(desc.padding.saturating_mul(2));
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
        // Join the 2D-nearest pair of rooms across the split (ties: lowest indices).
        let mut best: Option<(u64, usize, usize)> = None;
        for ia in left.clone() {
            for ib in right.clone() {
                let d2 = room_dist2(&out.rooms[ia], &out.rooms[ib]);
                if best.is_none_or(|b| (d2, ia, ib) < b) {
                    best = Some((d2, ia, ib));
                }
            }
        }
        if let Some((_, ia, ib)) = best {
            connect_rooms(rng, desc, area, ia, ib, out);
        }
        return start..right.end;
    }
    // A leaf: one room inside its padding.
    let pad = desc.padding;
    let pad2 = pad.saturating_mul(2);
    let room_w = rng.integer(
        i64::from(min_room),
        i64::from(max_room.min(area.width - pad2)),
    ) as u32;
    let room_h = rng.integer(
        i64::from(min_room),
        i64::from(max_room.min(area.height - pad2)),
    ) as u32;
    let x = area.x + pad + rng.integer(0, i64::from(area.width - pad2 - room_w)) as u32;
    let y = area.y + pad + rng.integer(0, i64::from(area.height - pad2 - room_h)) as u32;
    let mut room = CellRect {
        x,
        y,
        width: room_w,
        height: room_h,
    };
    let permitted = |r| writable(desc, r) && !desc.rooms.iter().any(|p| overlaps(*p, r));
    if !permitted(room) {
        let mut found = None;
        'search: for y in area.y + pad..=area.y + area.height - pad - min_room {
            for x in area.x + pad..=area.x + area.width - pad - min_room {
                let r = CellRect {
                    x,
                    y,
                    width: min_room,
                    height: min_room,
                };
                if permitted(r) {
                    found = Some(r);
                    break 'search;
                }
            }
        }
        let Some(r) = found else {
            return start..start;
        };
        room = r;
    }
    out.rooms.push(room);
    start..start + 1
}
