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
