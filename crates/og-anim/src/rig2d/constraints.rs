//! Rig IK, transform and path constraint solving.

use super::*;

/// FABRIK's passes over a chain of three or more bones: at most this many.
const FABRIK_PASSES: usize = 64;
/// ... stopping when the tip is this close to the target (pixels).
const FABRIK_TOLERANCE: f64 = 1e-4;

/// Turns `chain` (three or more bones, each the previous one's child) so
/// the last one's tip reaches `t`, by FABRIK over the joints (each bone's
/// origin, then the tip): from the animated pose, its interior joints
/// first moved to the asked side of the line from the root to the target
/// (mirrored across it when they lie on the other side, pushed off it when
/// the chain is straight), then backward and forward passes that keep each
/// segment's length; each bone then turns, root first, towards its solved
/// joint, by `w` of the way. Out of reach the chain points straight at it.
pub(super) fn fabrik(
    bones: &[Bone2d],
    local: &mut [[f32; 6]],
    chain: &[usize],
    t: [f64; 2],
    w: f64,
    counterclockwise: bool,
) {
    let joint = |world: &[Affine2], i: usize| -> [f64; 2] {
        match chain.get(i) {
            Some(&b) => [f64::from(world[b].tx), f64::from(world[b].ty)],
            None => {
                let last = chain[chain.len() - 1];
                world[last].apply(bones[last].length, 0.0).map(f64::from)
            }
        }
    };
    let world = world_of(bones, local);
    let mut p: Vec<[f64; 2]> = (0..=chain.len()).map(|i| joint(&world, i)).collect();
    let n = chain.len();
    let lengths: Vec<f64> = (0..n)
        .map(|i| sq(p[i + 1][0] - p[i][0], p[i + 1][1] - p[i][1]))
        .collect();
    let total: f64 = lengths.iter().sum();
    let root = p[0];
    let (dx, dy) = (t[0] - root[0], t[1] - root[1]);
    let dist = sq(dx, dy);
    if dist >= total {
        let (ux, uy) = if dist > 0.0 {
            (dx / dist, dy / dist)
        } else {
            (1.0, 0.0)
        };
        for i in 1..=n {
            p[i] = [
                p[i - 1][0] + ux * lengths[i - 1],
                p[i - 1][1] + uy * lengths[i - 1],
            ];
        }
    } else {
        if dist > 0.0 {
            let (ux, uy) = (dx / dist, dy / dist);
            let side = if counterclockwise { 1.0 } else { -1.0 };
            // Left of the line (counter-clockwise) is a positive cross product.
            let bent: f64 = p[1..n]
                .iter()
                .map(|q| ux * (q[1] - root[1]) - uy * (q[0] - root[0]))
                .sum();
            if bent * side < 0.0 {
                for q in &mut p[1..] {
                    let (rx, ry) = (q[0] - root[0], q[1] - root[1]);
                    let along = rx * ux + ry * uy;
                    *q = [
                        root[0] + 2.0 * along * ux - rx,
                        root[1] + 2.0 * along * uy - ry,
                    ];
                }
            } else if bent.abs() <= 1e-9 * total * total {
                for q in &mut p[1..n] {
                    *q = [
                        q[0] - side * uy * total * 0.05,
                        q[1] + side * ux * total * 0.05,
                    ];
                }
            }
        }
        for _ in 0..FABRIK_PASSES {
            if sq(p[n][0] - t[0], p[n][1] - t[1]) <= FABRIK_TOLERANCE {
                break;
            }
            p[n] = t;
            for i in (0..n).rev() {
                let (ex, ey) = (p[i][0] - p[i + 1][0], p[i][1] - p[i + 1][1]);
                let l = sq(ex, ey);
                if l > 1e-12 {
                    let k = lengths[i] / l;
                    p[i] = [p[i + 1][0] + ex * k, p[i + 1][1] + ey * k];
                }
            }
            p[0] = root;
            for i in 1..=n {
                let (ex, ey) = (p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]);
                let l = sq(ex, ey);
                if l > 1e-12 {
                    let k = lengths[i - 1] / l;
                    p[i] = [p[i - 1][0] + ex * k, p[i - 1][1] + ey * k];
                }
            }
        }
    }
    // Root first: each bone turns its next joint towards the solved one.
    for (i, &b) in chain.iter().enumerate() {
        let world = world_of(bones, local);
        let o = joint(&world, i);
        let next = joint(&world, i + 1);
        let now = atan2(next[1] - o[1], next[0] - o[0]);
        let want = atan2(p[i + 1][1] - o[1], p[i + 1][0] - o[0]);
        local[b][2] += (wrap(want - now) * w / DEG) as f32;
    }
}

/// [`world_of`] with the bones a constraint placed held where it put them
/// (their children following).
fn world_fixed(bones: &[Bone2d], local: &[[f32; 6]], fixed: &[Option<Affine2>]) -> Vec<Affine2> {
    let mut world: Vec<Affine2> = Vec::with_capacity(bones.len());
    for (i, (b, l)) in bones.iter().zip(local).enumerate() {
        let m = match fixed[i] {
            Some(m) => m,
            None => {
                let m = Affine2::trsk(l[0], l[1], l[2], l[5], l[3], l[4]);
                match b.parent {
                    Some(p) => world[p].mul(&m),
                    None => m,
                }
            }
        };
        world.push(m);
    }
    world
}

/// `m`'s axes turned by `r` radians.
fn turned(m: &Affine2, r: f64) -> Affine2 {
    let (s, c) = sincos(r);
    let (a, b, cc, d) = (
        f64::from(m.a),
        f64::from(m.b),
        f64::from(m.c),
        f64::from(m.d),
    );
    Affine2 {
        a: (c * a - s * b) as f32,
        b: (s * a + c * b) as f32,
        c: (c * cc - s * d) as f32,
        d: (s * cc + c * d) as f32,
        tx: m.tx,
        ty: m.ty,
    }
}

/// A transform constraint in the armature's space (Spine's
/// `applyAbsoluteWorld`): each bone's rotation, origin, axis lengths and
/// shear moved `mix` of the way to the target's (plus the offsets).
pub(super) fn transform_constraint(
    bones: &[Bone2d],
    local: &[[f32; 6]],
    fixed: &mut [Option<Affine2>],
    world: &mut Vec<Affine2>,
    c: &TransformConstraint2d,
) {
    let t = world[c.target];
    let f = |v: f32| f64::from(v);
    let reflect = if f(t.a) * f(t.d) - f(t.b) * f(t.c) > 0.0 {
        DEG
    } else {
        -DEG
    };
    let (offset_rotation, offset_shear) = (f(c.rotation) * reflect, f(c.shear) * reflect);
    let [mr, mt, ms, mh] = c.mix.map(f64::from);
    // Each bone from its transform before the constraint (a child's is not
    // moved by its constrained parent until the constraint is done).
    let before = world.clone();
    for &b in &c.bones {
        let mut m = before[b];
        if mr != 0.0 {
            let r = wrap(t.angle() - m.angle() + offset_rotation) * mr;
            m = turned(&m, r);
        }
        if mt != 0.0 {
            let p = t.apply(c.at[0], c.at[1]);
            m.tx = (f(m.tx) + (f(p[0]) - f(m.tx)) * mt) as f32;
            m.ty = (f(m.ty) + (f(p[1]) - f(m.ty)) * mt) as f32;
        }
        if ms > 0.0 {
            for (axis, offset) in [(0, c.scale[0]), (1, c.scale[1])] {
                let (x, y, tx, ty) = if axis == 0 {
                    (m.a, m.b, t.a, t.b)
                } else {
                    (m.c, m.d, t.c, t.d)
                };
                let s = sq(f(x), f(y));
                let ts = sq(f(tx), f(ty));
                if s > 1e-5 {
                    let k = ((s + (ts - s + f(offset)) * ms) / s) as f32;
                    if axis == 0 {
                        (m.a, m.b) = (m.a * k, m.b * k);
                    } else {
                        (m.c, m.d) = (m.c * k, m.d * k);
                    }
                }
            }
        }
        if mh > 0.0 {
            // The angle between the axes moves towards the target's.
            let by = atan2(f(m.d), f(m.c));
            let r = atan2(f(t.d), f(t.c)) - t.angle() - (by - m.angle());
            let r = by + (wrap(r) + offset_shear) * mh;
            let s = sq(f(m.c), f(m.d));
            let (sn, cs) = sincos(r);
            (m.c, m.d) = ((cs * s) as f32, (sn * s) as f32);
        }
        fixed[b] = Some(m);
    }
    *world = world_fixed(bones, local, fixed);
}

/// A path attachment's points in the armature's space (`deform` offsets
/// each influence, as a mesh's).
fn path_points(
    points: &[Vec<Influence>],
    world: &[Affine2],
    deform: Option<&[f32]>,
) -> Vec<[f64; 2]> {
    let mut k = 0usize;
    points
        .iter()
        .map(|v| {
            let mut p = [0.0f64; 2];
            for i in v {
                let (dx, dy) = deform.map_or((0.0, 0.0), |d| {
                    (
                        d.get(2 * k).copied().unwrap_or(0.0),
                        d.get(2 * k + 1).copied().unwrap_or(0.0),
                    )
                });
                k += 1;
                let q = world[i.bone].apply(i.x + dx, i.y + dy);
                p[0] += f64::from(q[0]) * f64::from(i.weight);
                p[1] += f64::from(q[1]) * f64::from(i.weight);
            }
            p
        })
        .collect()
}

/// The point (and the tangent's angle) at `t` of the cubic from `p[0]`
/// through the handles `p[1]`, `p[2]` to `p[3]`.
fn cubic_at(t: f64, p: [[f64; 2]; 4]) -> [f64; 3] {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    let at = |i: usize| a * p[0][i] + b * p[1][i] + c * p[2][i] + d * p[3][i];
    // The derivative's direction: (1-t)² (p1-p0) + 2(1-t)t (p2-p1) + t² (p3-p2).
    let dv = |i: usize| {
        u * u * (p[1][i] - p[0][i])
            + 2.0 * u * t * (p[2][i] - p[1][i])
            + t * t * (p[3][i] - p[2][i])
    };
    let (mut dx, mut dy) = (dv(0), dv(1));
    if dx * dx + dy * dy < 1e-18 {
        (dx, dy) = (p[3][0] - p[0][0], p[3][1] - p[0][1]);
    }
    [at(0), at(1), atan2(dy, dx)]
}

/// The point `d` from `from` along the direction from `a` to `b`, and that direction's angle.
fn along_line(a: [f64; 2], b: [f64; 2], from: [f64; 2], d: f64) -> [f64; 3] {
    let r = atan2(b[1] - a[1], b[0] - a[0]);
    let (s, c) = sincos(r);
    [from[0] + d * c, from[1] + d * s, r]
}

/// Where each of `spaces` (added up from `position`) falls on the path
/// (`points` in the armature's space), with the tangent's angle there:
/// DragonBones' `_computeBezierCurve`. By the `lengths` (a position within
/// a curve is its parameter), or by arc length when `constant` (each
/// curve's length by four forward-difference steps, then ten within the
/// curve). A closed path wraps; an open one extends past its ends along
/// its end handles (Spine; DragonBones leaves the last tick's value).
#[allow(clippy::too_many_arguments)]
fn path_positions(
    points: &[[f64; 2]],
    closed: bool,
    constant: bool,
    lengths: &[f32],
    spaces: &mut [f64],
    percent_position: bool,
    percent_spacing: bool,
    mut position: f64,
) -> Vec<[f64; 3]> {
    let knots = points.len() / 3;
    let mut out = Vec::with_capacity(spaces.len());
    // The curves' four points each, in order.
    let curves: Vec<[[f64; 2]; 4]> = (0..knots - usize::from(!closed))
        .map(|c| {
            let i = |k: usize| points[k % points.len()];
            [i(3 * c + 1), i(3 * c + 2), i(3 * c + 3), i(3 * c + 4)]
        })
        .collect();
    let first = curves[0];
    let last = curves[curves.len() - 1];
    let place = |p: f64, total: f64, find: &dyn Fn(f64) -> [f64; 3], out: &mut Vec<[f64; 3]>| {
        let p = if closed {
            let w = p % total;
            if w < 0.0 { w + total } else { w }
        } else {
            p
        };
        out.push(if !closed && p < 0.0 {
            along_line(first[0], first[1], first[0], p)
        } else if !closed && p > total {
            along_line(last[2], last[3], last[3], p - total)
        } else {
            find(p)
        });
    };
    if !constant {
        let ends: Vec<f64> = lengths.iter().map(|l| f64::from(*l)).collect();
        let total = ends.last().copied().unwrap_or(0.0);
        if percent_position {
            position *= total;
        }
        if percent_spacing {
            spaces.iter_mut().for_each(|s| *s *= total);
        }
        let find = |p: f64| -> [f64; 3] {
            let c = ends.iter().position(|e| p <= *e).unwrap_or(ends.len() - 1);
            let (lo, hi) = (if c == 0 { 0.0 } else { ends[c - 1] }, ends[c]);
            let t = if hi > lo { (p - lo) / (hi - lo) } else { 0.0 };
            cubic_at(t.clamp(0.0, 1.0), curves[c])
        };
        for s in spaces.iter() {
            position += s;
            place(position, total, &find, &mut out);
        }
        return out;
    }
    // Each curve's length by four forward-difference steps (DragonBones' and Spine's).
    let mut ends = Vec::with_capacity(curves.len());
    let mut total = 0.0f64;
    for q in &curves {
        let [p0, p1, p2, p3] = *q;
        let tmp = [0, 1].map(|i| (p0[i] - p1[i] * 2.0 + p2[i]) * 0.1875);
        let ddd = [0, 1].map(|i| ((p1[i] - p2[i]) * 3.0 - p0[i] + p3[i]) * 0.09375);
        let mut dd = [0, 1].map(|i| tmp[i] * 2.0 + ddd[i]);
        let mut d = [0, 1].map(|i| (p1[i] - p0[i]) * 0.75 + tmp[i] + ddd[i] * 0.166_666_67);
        total += sq(d[0], d[1]);
        for _ in 0..2 {
            d = [0, 1].map(|i| d[i] + dd[i]);
            dd = [0, 1].map(|i| dd[i] + ddd[i]);
            total += sq(d[0], d[1]);
        }
        d = [0, 1].map(|i| d[i] + dd[i] + ddd[i]);
        total += sq(d[0], d[1]);
        ends.push(total);
    }
    if percent_position {
        position *= total;
    }
    if percent_spacing {
        spaces.iter_mut().for_each(|s| *s *= total);
    }
    let find = |p: f64| -> [f64; 3] {
        let c = ends.iter().position(|e| p <= *e).unwrap_or(ends.len() - 1);
        let (lo, hi) = (if c == 0 { 0.0 } else { ends[c - 1] }, ends[c]);
        let mut p = if hi > lo { (p - lo) / (hi - lo) } else { 0.0 };
        // Ten forward-difference steps within the curve, to find the parameter.
        let [p0, p1, p2, p3] = curves[c];
        let tmp = [0, 1].map(|i| (p0[i] - p1[i] * 2.0 + p2[i]) * 0.03);
        let ddd = [0, 1].map(|i| ((p1[i] - p2[i]) * 3.0 - p0[i] + p3[i]) * 0.006);
        let mut dd = [0, 1].map(|i| tmp[i] * 2.0 + ddd[i]);
        let mut d = [0, 1].map(|i| (p1[i] - p0[i]) * 0.3 + tmp[i] + ddd[i] * 0.166_666_67);
        let mut segments = [0.0f64; 10];
        let mut length = sq(d[0], d[1]);
        segments[0] = length;
        for s in segments.iter_mut().take(8).skip(1) {
            d = [0, 1].map(|i| d[i] + dd[i]);
            dd = [0, 1].map(|i| dd[i] + ddd[i]);
            length += sq(d[0], d[1]);
            *s = length;
        }
        d = [0, 1].map(|i| d[i] + dd[i]);
        length += sq(d[0], d[1]);
        segments[8] = length;
        d = [0, 1].map(|i| d[i] + dd[i] + ddd[i]);
        length += sq(d[0], d[1]);
        segments[9] = length;
        p *= length;
        let k = segments.iter().position(|e| p <= *e).unwrap_or(9);
        let t = if k == 0 {
            if segments[0] > 0.0 {
                p / segments[0]
            } else {
                0.0
            }
        } else {
            let (lo, hi) = (segments[k - 1], segments[k]);
            k as f64 + if hi > lo { (p - lo) / (hi - lo) } else { 0.0 }
        };
        cubic_at((t * 0.1).clamp(0.0, 1.0), curves[c])
    };
    for s in spaces.iter() {
        position += s;
        place(position, total, &find, &mut out);
    }
    out
}

/// A path constraint (DragonBones' `PathConstraint.update`): the bones'
/// places along the path from their spacing, each bone moved `mix[1]` of
/// the way to its place and turned `mix[0]` of the way along the tangent
/// there or towards the next place (scaled to reach it in `chainscale`).
pub(super) fn path_constraint(
    body: Body<'_>,
    local: &[[f32; 6]],
    fixed: &mut [Option<Affine2>],
    world: &mut Vec<Affine2>,
    c: &PathConstraint2d,
    deform: Option<&[f32]>,
) {
    let AttachmentKind::Path {
        points,
        closed,
        constant,
        lengths,
    } = &body.attachments[c.path].kind
    else {
        return;
    };
    let points = path_points(points, world, deform);
    let tangent = c.rotate == PathRotate::Tangent;
    let chain_scale = c.rotate == PathRotate::ChainScale;
    let count = c.bones.len();
    let spacing = f64::from(c.spacing);
    let mut spaces = vec![spacing; if tangent { count } else { count + 1 }];
    let mut reach = vec![0.0f64; count];
    if chain_scale || c.spacing_mode == PathSpacing::Length {
        spaces[0] = 0.0;
        for i in 0..spaces.len() - 1 {
            let b = c.bones[i];
            let l = f64::from(body.bones[b].length);
            let m = world[b];
            let len = sq(l * f64::from(m.a), l * f64::from(m.b));
            reach[i] = len;
            spaces[i + 1] = if l > 1e-9 {
                (l + spacing) * len / l
            } else {
                0.0
            };
        }
    }
    let at = path_positions(
        &points,
        *closed,
        *constant,
        lengths,
        &mut spaces,
        c.position_unit == PathUnit::Percent,
        c.spacing_mode == PathSpacing::Percent,
        f64::from(c.position),
    );
    let (mut bx, mut by) = (at[0][0], at[0][1]);
    let mut offset = f64::from(c.rotation);
    let tip = offset == 0.0 && c.rotate == PathRotate::Chain;
    if offset != 0.0 {
        let m = world[body.slots[body.attachments[c.path].slot].bone];
        let det = f64::from(m.a) * f64::from(m.d) - f64::from(m.b) * f64::from(m.c);
        offset *= if det > 0.0 { DEG } else { -DEG };
    }
    let [mr, mt] = c.mix.map(f64::from);
    // As DragonBones and Spine: each bone from its transform before the constraint.
    let before = world.clone();
    for (i, &b) in c.bones.iter().enumerate() {
        let mut m = before[b];
        m.tx = (f64::from(m.tx) + (bx - f64::from(m.tx)) * mt) as f32;
        m.ty = (f64::from(m.ty) + (by - f64::from(m.ty)) * mt) as f32;
        let next = at.get(i + 1).copied().unwrap_or(at[i]);
        let (dx, dy) = (next[0] - bx, next[1] - by);
        if chain_scale && reach[i] > 1e-9 {
            let s = ((sq(dx, dy) / reach[i] - 1.0) * mr + 1.0) as f32;
            (m.a, m.b) = (m.a * s, m.b * s);
        }
        (bx, by) = (next[0], next[1]);
        if mr > 0.0 {
            let mut r = if tangent { at[i][2] } else { atan2(dy, dx) } - m.angle();
            if tip {
                let (s, co) = sincos(r);
                let l = f64::from(body.bones[b].length);
                let (a, bb) = (f64::from(m.a), f64::from(m.b));
                bx += (l * (co * a - s * bb) - dx) * mr;
                by += (l * (s * a + co * bb) - dy) * mr;
            } else {
                r += offset;
            }
            m = turned(&m, wrap(r) * mr);
        }
        fixed[b] = Some(m);
    }
    *world = world_fixed(body.bones, local, fixed);
}

/// Each bone's transform in the armature's space, parents first.
pub(super) fn world_of(bones: &[Bone2d], local: &[[f32; 6]]) -> Vec<Affine2> {
    let mut world: Vec<Affine2> = Vec::with_capacity(bones.len());
    for (b, l) in bones.iter().zip(local) {
        let m = Affine2::trsk(l[0], l[1], l[2], l[5], l[3], l[4]);
        world.push(match b.parent {
            Some(p) => world[p].mul(&m),
            None => m,
        });
    }
    world
}

/// The length of (x, y).
pub(super) fn sq(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}
