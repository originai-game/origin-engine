//! ORCA, optimal reciprocal collision avoidance (van den Berg et al.,
//! the RVO2 library's construction): the velocity nearest an agent's
//! preferred one that keeps it clear of every neighbour for a time
//! horizon, each neighbour taking its share of the avoidance.
//!
//! Deterministic by construction: plain `f32` arithmetic (`+ − × ÷` and
//! `sqrt`, all correctly rounded on every host; no trigonometry), the
//! neighbours in the order the caller gives (distance, then entity), and
//! every agent solved from the same start-of-tick state (a Jacobi step: no
//! agent sees another's new velocity), so the order agents are solved in
//! does not matter.

use og_core::glam::Vec2;

/// Below this a determinant is taken as parallel lines.
const EPSILON: f32 = 1e-5;

/// A half-plane of allowed velocities: those left of `direction` through `point`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Line {
    point: Vec2,
    direction: Vec2,
}

/// Another agent, as the solving one sees it at the start of the tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Neighbour {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    /// The solving agent's share of avoiding it: its own weight against
    /// another crowd agent, 1 against one that does not avoid.
    pub share: f32,
}

/// What one agent solves for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Agent {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    /// The velocity it wants (its path's, steering's).
    pub preferred: Vec2,
    pub max_speed: f32,
    /// Seconds ahead it keeps clear of other agents, and of walls.
    pub horizon: f32,
    pub obstacle_horizon: f32,
    /// Picks its symmetry-breaking margin (the caller's entity).
    pub seed: u64,
}

fn det(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn dot(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

fn abs_sq(a: Vec2) -> f32 {
    dot(a, a)
}

fn normalize(a: Vec2) -> Vec2 {
    let l = abs_sq(a).sqrt();
    if l > 0.0 { a / l } else { Vec2::ZERO }
}

/// The new velocity for `agent`: the one nearest its preferred velocity
/// that stays clear of `walls` (segments, end to end, kept as hard
/// limits) for its obstacle horizon and of `neighbours` for its horizon,
/// at most its speed. When no velocity satisfies every neighbour, the one
/// that violates them least (walls stay hard). `dt` is the tick: agents
/// already overlapping are pushed apart within it.
pub fn velocity(agent: &Agent, walls: &[(Vec2, Vec2)], neighbours: &[Neighbour], dt: f32) -> Vec2 {
    let agent = &Agent {
        radius: agent.radius * (1.0 + margin(agent.seed)),
        ..*agent
    };
    let mut lines: Vec<Line> = Vec::with_capacity(walls.len() + neighbours.len());
    let inv_dt = if dt > 0.0 { 1.0 / dt } else { 0.0 };
    // Walls: RVO2's obstacle lines for each segment (hard limits, wholly
    // this agent's to keep), skipping one the lines so far already cover.
    if agent.obstacle_horizon > 0.0 {
        let inv = 1.0 / agent.obstacle_horizon;
        for &(a, b) in walls {
            // The agent on the segment's outer (right) side, as RVO2's
            // counter-clockwise obstacles have it.
            let (a, b) = if det(b - a, agent.position - a) > 0.0 {
                (b, a)
            } else {
                (a, b)
            };
            let r = agent.radius;
            let (rel1, rel2) = (a - agent.position, b - agent.position);
            let covered = lines.iter().any(|l| {
                det(rel1 * inv - l.point, l.direction) - inv * r >= -EPSILON
                    && det(rel2 * inv - l.point, l.direction) - inv * r >= -EPSILON
            });
            if covered || !(rel1.is_finite() && rel2.is_finite()) || a == b {
                continue;
            }
            if let Some(l) = wall_line(agent.velocity, rel1, rel2, r, inv) {
                lines.push(l);
            }
        }
    }
    let hard = lines.len();
    if agent.horizon > 0.0 {
        for n in neighbours {
            if let Some(l) = pair_line(
                agent,
                n.position - agent.position,
                agent.velocity - n.velocity,
                agent.radius + n.radius,
                n.share,
                agent.horizon,
                inv_dt,
            ) {
                lines.push(l);
            }
        }
    }
    let max = agent.max_speed.max(0.0);
    let mut result = Vec2::ZERO;
    let fail = program2(&lines, max, agent.preferred, false, &mut result);
    if fail < lines.len() {
        program3(&lines, hard, fail, max, &mut result);
    }
    result
}

/// The margin an agent keeps beyond its radius, from 0 to [`MARGIN`] of
/// it, picked by `seed` (its entity): agents meeting perfectly
/// symmetrically (a ring swapping places, a formation turning about) hold
/// each other still in a ring ORCA alone keeps exactly symmetric, and the
/// unequal margins break it as RVO2's demos do with noise, but the same on
/// every host and every run.
fn margin(seed: u64) -> f32 {
    // splitmix64's finaliser: neighbouring entities spread over [0, 1).
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    (z >> 40) as f32 / (1u64 << 24) as f32 * MARGIN
}

/// The largest margin, as a share of the radius. A thousandth left an
/// eight-agent ring locked; a twentieth breaks it on every seed tried
/// while a crowd still looks evenly spaced.
const MARGIN: f32 = 0.05;

/// The half-plane keeping an agent of radius `r` and velocity `v` off the
/// wall from `rel1` to `rel2` (relative to it, it on the right) for
/// `1 / inv` seconds: RVO2's obstacle construction for one segment whose
/// ends are both convex and with no neighbouring segments.
fn wall_line(v: Vec2, rel1: Vec2, rel2: Vec2, r: f32, inv: f32) -> Option<Line> {
    let r_sq = r * r;
    let along = rel2 - rel1;
    let unit = normalize(along);
    let (d1, d2) = (abs_sq(rel1), abs_sq(rel2));
    let s = dot(-rel1, along) / abs_sq(along);
    let dist_line = abs_sq(-rel1 - along * s);
    let at_zero = |dir: Vec2| Line {
        point: Vec2::ZERO,
        direction: dir,
    };
    // Already touching an end or the segment: straight away from it.
    if s < 0.0 && d1 <= r_sq {
        return Some(at_zero(normalize(Vec2::new(-rel1.y, rel1.x))));
    }
    if s > 1.0 && d2 <= r_sq {
        return Some(at_zero(normalize(Vec2::new(-rel2.y, rel2.x))));
    }
    if (0.0..1.0).contains(&s) && dist_line <= r_sq {
        return Some(at_zero(-unit));
    }
    let left_leg = |rel: Vec2, d: f32| {
        let leg = (d - r_sq).sqrt();
        Vec2::new(rel.x * leg - rel.y * r, rel.x * r + rel.y * leg) / d
    };
    let right_leg = |rel: Vec2, d: f32| {
        let leg = (d - r_sq).sqrt();
        Vec2::new(rel.x * leg + rel.y * r, -rel.x * r + rel.y * leg) / d
    };
    // Seen end on, both legs come from the nearer end.
    let (p1, p2, left, right) = if s < 0.0 && dist_line <= r_sq {
        (rel1, rel1, left_leg(rel1, d1), right_leg(rel1, d1))
    } else if s > 1.0 && dist_line <= r_sq {
        (rel2, rel2, left_leg(rel2, d2), right_leg(rel2, d2))
    } else {
        (rel1, rel2, left_leg(rel1, d1), right_leg(rel2, d2))
    };
    let point = p1 == p2;
    let left_cut = p1 * inv;
    let right_cut = p2 * inv;
    let cut = right_cut - left_cut;
    let t = if point {
        0.5
    } else {
        dot(v - left_cut, cut) / abs_sq(cut)
    };
    let t_left = dot(v - left_cut, left);
    let t_right = dot(v - right_cut, right);
    let around = |c: Vec2| {
        let w = normalize(v - c);
        Line {
            point: c + w * (r * inv),
            direction: Vec2::new(w.y, -w.x),
        }
    };
    if (t < 0.0 && t_left < 0.0) || (point && t_left < 0.0 && t_right < 0.0) {
        return Some(around(left_cut));
    }
    if t > 1.0 && t_right < 0.0 {
        return Some(around(right_cut));
    }
    let far = f32::INFINITY;
    let d_cut = if t < 0.0 || t > 1.0 || point {
        far
    } else {
        abs_sq(v - (left_cut + cut * t))
    };
    let d_left = if t_left < 0.0 {
        far
    } else {
        abs_sq(v - (left_cut + left * t_left))
    };
    let d_right = if t_right < 0.0 {
        far
    } else {
        abs_sq(v - (right_cut + right * t_right))
    };
    let offset = |base: Vec2, dir: Vec2| Line {
        point: base + Vec2::new(-dir.y, dir.x) * (r * inv),
        direction: dir,
    };
    Some(if d_cut <= d_left && d_cut <= d_right {
        offset(left_cut, -unit)
    } else if d_left <= d_right {
        offset(left_cut, left)
    } else {
        offset(right_cut, -right)
    })
}

/// The ORCA half-plane against one neighbour at relative position `p`
/// with relative velocity `v` (this agent's minus the other's), combined
/// radius `r` and this agent's `share`; None when the share is 0 or the
/// inputs are not finite.
fn pair_line(
    agent: &Agent,
    p: Vec2,
    v: Vec2,
    r: f32,
    share: f32,
    tau: f32,
    inv_dt: f32,
) -> Option<Line> {
    if share <= 0.0 || !(p.is_finite() && v.is_finite() && r.is_finite()) {
        return None;
    }
    let dist_sq = abs_sq(p);
    let r_sq = r * r;
    let (direction, u) = if dist_sq > r_sq {
        let inv_tau = 1.0 / tau;
        // From the cut-off circle's centre to the relative velocity.
        let w = v - p * inv_tau;
        let w_len_sq = abs_sq(w);
        let d1 = dot(w, p);
        if d1 < 0.0 && d1 * d1 > r_sq * w_len_sq {
            // Onto the cut-off circle.
            let w_len = w_len_sq.sqrt();
            let unit = if w_len > 0.0 { w / w_len } else { Vec2::ZERO };
            (Vec2::new(unit.y, -unit.x), unit * (r * inv_tau - w_len))
        } else {
            // Onto a leg of the cone.
            let leg = (dist_sq - r_sq).sqrt();
            let dir = if det(p, w) > 0.0 {
                Vec2::new(p.x * leg - p.y * r, p.x * r + p.y * leg) / dist_sq
            } else {
                -Vec2::new(p.x * leg + p.y * r, -p.x * r + p.y * leg) / dist_sq
            };
            let d2 = dot(v, dir);
            (dir, dir * d2 - v)
        }
    } else {
        // Already overlapping: apart within this tick.
        if inv_dt <= 0.0 {
            return None;
        }
        let w = v - p * inv_dt;
        let w_len = abs_sq(w).sqrt();
        let unit = if w_len > 0.0 { w / w_len } else { Vec2::X };
        (Vec2::new(unit.y, -unit.x), unit * (r * inv_dt - w_len))
    };
    Some(Line {
        point: agent.velocity + u * share.min(1.0),
        direction,
    })
}

/// The best velocity on line `n` within the speed circle and left of every
/// earlier line; false when there is none.
fn program1(
    lines: &[Line],
    n: usize,
    radius: f32,
    opt: Vec2,
    direction_opt: bool,
    result: &mut Vec2,
) -> bool {
    let line = lines[n];
    let d = dot(line.point, line.direction);
    let disc = d * d + radius * radius - abs_sq(line.point);
    if disc < 0.0 {
        return false;
    }
    let s = disc.sqrt();
    let (mut left, mut right) = (-d - s, -d + s);
    for other in &lines[..n] {
        let den = det(line.direction, other.direction);
        let num = det(other.direction, line.point - other.point);
        if den.abs() <= EPSILON {
            if num < 0.0 {
                return false;
            }
            continue;
        }
        let t = num / den;
        if den >= 0.0 {
            right = right.min(t);
        } else {
            left = left.max(t);
        }
        if left > right {
            return false;
        }
    }
    let t = if direction_opt {
        if dot(opt, line.direction) > 0.0 {
            right
        } else {
            left
        }
    } else {
        dot(line.direction, opt - line.point).clamp(left, right)
    };
    *result = line.point + line.direction * t;
    true
}

/// The velocity nearest `opt` (or furthest along it, with `direction_opt`)
/// inside every line and the speed circle; the index of the first line it
/// could not satisfy, or `lines.len()`.
fn program2(
    lines: &[Line],
    radius: f32,
    opt: Vec2,
    direction_opt: bool,
    result: &mut Vec2,
) -> usize {
    *result = if direction_opt {
        opt * radius
    } else if abs_sq(opt) > radius * radius {
        normalize(opt) * radius
    } else {
        opt
    };
    for i in 0..lines.len() {
        if det(lines[i].direction, lines[i].point - *result) > 0.0 {
            let keep = *result;
            if !program1(lines, i, radius, opt, direction_opt, result) {
                *result = keep;
                return i;
            }
        }
    }
    lines.len()
}

/// From line `begin` on, the velocity that violates the soft lines least
/// (the first `hard` stay satisfied).
fn program3(lines: &[Line], hard: usize, begin: usize, radius: f32, result: &mut Vec2) {
    let mut distance = 0.0f32;
    for i in begin..lines.len() {
        if det(lines[i].direction, lines[i].point - *result) <= distance {
            continue;
        }
        let mut projected: Vec<Line> = lines[..hard].to_vec();
        for j in hard..i {
            let d = det(lines[i].direction, lines[j].direction);
            let point = if d.abs() <= EPSILON {
                if dot(lines[i].direction, lines[j].direction) > 0.0 {
                    continue;
                }
                (lines[i].point + lines[j].point) * 0.5
            } else {
                lines[i].point
                    + lines[i].direction
                        * (det(lines[j].direction, lines[i].point - lines[j].point) / d)
            };
            projected.push(Line {
                point,
                direction: normalize(lines[j].direction - lines[i].direction),
            });
        }
        let keep = *result;
        let opt = Vec2::new(-lines[i].direction.y, lines[i].direction.x);
        if program2(&projected, radius, opt, true, result) < projected.len() {
            *result = keep;
        }
        distance = det(lines[i].direction, lines[i].point - *result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(x: f32, vx: f32) -> Agent {
        Agent {
            position: Vec2::new(x, 0.0),
            velocity: Vec2::new(vx, 0.0),
            radius: 0.5,
            preferred: Vec2::new(vx, 0.0),
            max_speed: 2.0,
            horizon: 2.0,
            obstacle_horizon: 0.5,
            seed: 7,
        }
    }

    #[test]
    fn a_free_agent_keeps_its_preferred_velocity_and_its_speed_cap() {
        let a = agent(0.0, 1.5);
        assert_eq!(velocity(&a, &[], &[], 1.0 / 60.0), Vec2::new(1.5, 0.0));
        let fast = Agent {
            preferred: Vec2::new(0.0, 5.0),
            ..a
        };
        assert!((velocity(&fast, &[], &[], 1.0 / 60.0).length() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn head_on_agents_each_turn_aside_by_half_and_a_wall_stays_hard() {
        let a = agent(-2.0, 1.0);
        let b = Neighbour {
            position: Vec2::new(2.0, 0.0),
            velocity: Vec2::new(-1.0, 0.0),
            radius: 0.5,
            share: 0.5,
        };
        let v = velocity(&a, &[], &[b], 1.0 / 60.0);
        assert!(v.y.abs() > 0.05, "it sidesteps: {v}");
        // A wall right beside the side it would take: it slows instead of
        // going through it.
        let side = if v.y > 0.0 { 0.55 } else { -0.55 };
        let wall = (Vec2::new(-10.0, side), Vec2::new(10.0, side));
        let c = velocity(&a, &[wall], &[b], 1.0 / 60.0);
        // Clear of it by 0.05 m, it closes at most that in the 0.5 s horizon.
        assert!(c.y * side.signum() <= 0.1 + 1e-4, "into the wall: {c}");
    }
}
