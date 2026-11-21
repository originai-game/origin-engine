//! Directional focus: the control nearest the focus in a direction, by
//! Android's FocusFinder (moved from og-ui's `interaction/nav.rs`).

use og_ui::{Nav, Rect};

/// The control nearest `from` in direction `dir` among `cands`, wrapping
/// (also the new core's, `og_ui::nav_neighbour`).
pub fn neighbour(from: &Rect, cands: &[(u64, Rect)], dir: Nav) -> Option<u64> {
    neighbour_in(from, cands, dir, false).or_else(|| neighbour_in(from, cands, dir, true))
}

/// The nearest control that way (`wrap` false, [`toward`]), or the
/// farthest the other way, in the same row or column first (`wrap` true).
fn neighbour_in(from: &Rect, cands: &[(u64, Rect)], dir: Nav, wrap: bool) -> Option<u64> {
    let (fx, fy) = from.center();
    // (along, across): along > 0 is in the direction.
    let axes = |r: &Rect| -> (f32, f32) {
        let (cx, cy) = r.center();
        let gap = |a0: f32, a1: f32, b0: f32, b1: f32| (b0 - a1).max(a0 - b1).max(0.0);
        match dir {
            Nav::Down => (cy - fy, gap(from.x, from.right(), r.x, r.right())),
            Nav::Up => (fy - cy, gap(from.x, from.right(), r.x, r.right())),
            Nav::Right => (cx - fx, gap(from.y, from.bottom(), r.y, r.bottom())),
            _ => (fx - cx, gap(from.y, from.bottom(), r.y, r.bottom())),
        }
    };
    if !wrap {
        return toward(from, cands, dir);
    }
    let mut best: Option<(f32, u64)> = None;
    for (k, r) in cands {
        let (along, across) = axes(r);
        if along < -0.5 {
            let score = along + 3.0 * across;
            if best.is_none_or(|(s, _)| score < s) {
                best = Some((score, *k));
            }
        }
    }
    best.map(|b| b.1)
}

/// Where a candidate stands from the focus, for one direction.
struct Toward {
    /// It overlaps the focus across the move (its row or column).
    beam: bool,
    /// The gap from the focus's leading edge to its near edge (0 touching).
    major: f32,
    /// From the focus's leading edge to its far edge.
    far: f32,
    /// Its centre's offset from the focus's across the move.
    minor: f32,
}

/// The nearest control that lies in direction `dir` from `from` at both of
/// its edges, by Android's FocusFinder (the W3C css-nav rule is the same in
/// substance): a control in the focus's beam (overlapping it across the
/// move) beats one outside it (going up or down, unless that one is wholly
/// nearer), then the least `13·major² + minor²` with `major` measured edge
/// to edge along the move and `minor` centre to centre across it. Scored
/// from centres, a tab diagonally below beat the button beside the focus in
/// its own row, and nothing led to that button.
fn toward(from: &Rect, cands: &[(u64, Rect)], dir: Nav) -> Option<u64> {
    let (fx, fy) = from.center();
    let horizontal = matches!(dir, Nav::Left | Nav::Right);
    let at = |r: &Rect| -> Option<Toward> {
        let (cx, cy) = r.center();
        let overlap = |a0: f32, a1: f32, b0: f32, b1: f32| a1.min(b1) - a0.max(b0) > 0.5;
        // That way at both of its edges (FocusFinder's isCandidate): a row
        // around the focus, whose centre lies past it, is no move out of
        // it (it won at no distance, and a link's neighbour in the same
        // row was never reached).
        let (that_way, major, far) = match dir {
            Nav::Down => (
                (from.y < r.y || from.bottom() <= r.y) && from.bottom() < r.bottom(),
                r.y - from.bottom(),
                r.bottom() - from.bottom(),
            ),
            Nav::Up => (
                (from.bottom() > r.bottom() || from.y >= r.bottom()) && from.y > r.y,
                from.y - r.bottom(),
                from.y - r.y,
            ),
            Nav::Right => (
                (from.x < r.x || from.right() <= r.x) && from.right() < r.right(),
                r.x - from.right(),
                r.right() - from.right(),
            ),
            _ => (
                (from.right() > r.right() || from.x >= r.right()) && from.x > r.x,
                from.x - r.right(),
                from.x - r.x,
            ),
        };
        // A control inside the focus (a row's own Remove button, a chat
        // line's link) whose centre lies that way from the focus's: the move
        // steps into it, at no distance. No edge of it lies past the focus,
        // so nothing led to it at all.
        let inside = r.x >= from.x - 0.5
            && r.y >= from.y - 0.5
            && r.right() <= from.right() + 0.5
            && r.bottom() <= from.bottom() + 0.5
            && match dir {
                Nav::Down => cy > fy + 0.5,
                Nav::Up => cy < fy - 0.5,
                Nav::Right => cx > fx + 0.5,
                _ => cx < fx - 0.5,
            };
        if inside {
            return Some(Toward {
                beam: true,
                major: 0.0,
                far: 0.0,
                minor: if horizontal {
                    (cy - fy).abs()
                } else {
                    (cx - fx).abs()
                },
            });
        }
        that_way.then(|| Toward {
            beam: if horizontal {
                overlap(from.y, from.bottom(), r.y, r.bottom())
            } else {
                overlap(from.x, from.right(), r.x, r.right())
            },
            major: major.max(0.0),
            far: far.max(0.0),
            minor: if horizontal {
                (cy - fy).abs()
            } else {
                (cx - fx).abs()
            },
        })
    };
    let weighted = |t: &Toward| 13.0 * t.major * t.major + t.minor * t.minor;
    // Whether `a` beats `b` (FocusFinder's isBetterCandidate and beamBeats).
    let beats = |a: &Toward, b: &Toward| {
        if a.beam != b.beam {
            let (inside, outside) = if a.beam { (a, b) } else { (b, a) };
            if horizontal || inside.major < outside.far {
                return a.beam;
            }
        }
        weighted(a) < weighted(b)
    };
    let mut best: Option<(Toward, u64)> = None;
    for (k, r) in cands {
        if let Some(t) = at(r)
            && best.as_ref().is_none_or(|(b, _)| beats(&t, b))
        {
            best = Some((t, *k));
        }
    }
    best.map(|b| b.1)
}
