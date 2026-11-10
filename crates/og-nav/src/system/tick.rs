use super::*;

#[derive(Clone, Copy)]
struct Row {
    e: Entity,
    agent: NavAgent,
    pos: Vec3,
    steer: Option<Steering>,
    vel: Option<Velocity>,
    filter: Option<NavFilter>,
    crowd: Option<Crowd>,
}

/// The tick's start-of-tick state crowd agents solve from.
struct CrowdView<'a> {
    rows: &'a [Row],
    positions: &'a [Vec2],
    velocities: &'a [Vec2],
    index: &'a og_ecs::SpatialHash,
    at: &'a BTreeMap<u64, usize>,
}

/// Agent `i`'s ORCA velocity for `preferred`: its `c.neighbours` nearest
/// agents within reach of its horizon, and on a navmesh the walls within
/// reach of its obstacle horizon.
#[allow(clippy::too_many_arguments)]
fn crowd_velocity(
    view: &CrowdView<'_>,
    i: usize,
    c: &Crowd,
    preferred: Vec2,
    speed: f32,
    mesh: Option<(&NavMesh, u32, Vec3)>,
    plane: Plane,
    dt: f32,
) -> Vec2 {
    let row = &view.rows[i];
    let p = view.positions[i];
    let radius = row.agent.radius.max(0.0);
    let k = c.neighbours.min(32) as usize;
    let mut neighbours = Vec::with_capacity(k);
    if k > 0 {
        for n in view.index.nearest([p.x, p.y, 0.0], k + 1) {
            let Some(&j) = view.at.get(&n.entity.to_bits()) else {
                continue;
            };
            if j == i {
                continue;
            }
            let other = &view.rows[j];
            let r = other.agent.radius.max(0.0);
            let reach = radius + r + c.horizon * (speed + other.agent.speed.max(0.0));
            if n.distance > f64::from(reach) || neighbours.len() == k {
                continue;
            }
            neighbours.push(crate::orca::Neighbour {
                position: view.positions[j],
                velocity: view.velocities[j],
                radius: r,
                share: other.crowd.map_or(1.0, |_| c.weight.clamp(0.0, 1.0)),
            });
        }
    }
    let walls: Vec<(Vec2, Vec2)> = match mesh {
        Some((m, poly, k)) if c.obstacle_horizon > 0.0 => m
            .walls_near(poly, row.pos - k, radius + c.obstacle_horizon * speed)
            .into_iter()
            .map(|(a, b)| {
                (
                    plane.to_plane(og_core::math::offset(a, k)),
                    plane.to_plane(og_core::math::offset(b, k)),
                )
            })
            .collect(),
        _ => Vec::new(),
    };
    crate::orca::velocity(
        &crate::orca::Agent {
            position: p,
            velocity: view.velocities[i],
            radius,
            preferred,
            max_speed: speed,
            horizon: c.horizon,
            obstacle_horizon: c.obstacle_horizon,
            seed: row.e.to_bits(),
        },
        &walls,
        &neighbours,
        dt,
    )
}

pub(super) fn run(ctx: &mut SystemContext<'_>, sys: &NavSystem, desc: &NavDesc) -> Result<()> {
    let plane = desc.plane;
    let tick = ctx.time().tick;
    let scaled = ctx.scaled_dt();
    // An agent in a time group (og.TimeGroup) moves at its group's pace.
    let groups = ctx.time_groups();
    let mut rows: Vec<Row> = Vec::new();
    // og.NavFilter is read only once a game has registered it.
    let filters = ctx.id_by_name(NavFilter::NAME).is_ok();
    // og.Crowd likewise.
    let crowds = ctx.id_by_name(Crowd::NAME).is_ok();
    for a in ctx.query(0)? {
        for &e in a.entities() {
            let agent = ctx.get::<NavAgent>(e)?;
            let t = ctx.get::<Transform>(e)?;
            rows.push(Row {
                e,
                agent,
                pos: t.position,
                steer: ctx.get::<Steering>(e).ok(),
                vel: ctx.get::<Velocity>(e).ok(),
                filter: if filters {
                    ctx.get::<NavFilter>(e).ok()
                } else {
                    None
                },
                crowd: if crowds {
                    ctx.get::<Crowd>(e).ok()
                } else {
                    None
                },
            });
        }
    }
    if rows.is_empty() {
        return Ok(());
    }
    let mut navs = lock(&sys.navs);
    navs.refresh_flows();
    let mut state = lock(&sys.state);
    // Drop the paths of agents that are gone.
    let alive: std::collections::BTreeSet<u64> =
        og_core::sort::set(rows.iter().map(|r| r.e.to_bits()));
    state.paths.retain(|k, _| alive.contains(k));
    // Positions for separation, bucketed by 2 m cells.
    let cell = 2.0f32;
    let mut buckets: BTreeMap<(i32, i32), Vec<usize>> = BTreeMap::new();
    for (i, r) in rows.iter().enumerate() {
        let p = plane.to_plane(r.pos);
        buckets
            .entry(((p.x / cell).floor() as i32, (p.y / cell).floor() as i32))
            .or_default()
            .push(i);
    }
    let positions: Vec<Vec2> = rows.iter().map(|r| plane.to_plane(r.pos)).collect();
    // For crowd agents: every agent's start-of-tick velocity and an index
    // of their positions (G2-2's spatial hash: neighbours by distance, then
    // entity). Every crowd agent solves from this same state.
    let crowd = if rows.iter().any(|r| r.crowd.is_some()) {
        let velocities: Vec<Vec2> = rows
            .iter()
            .map(|r| plane.to_plane(r.vel.map_or(r.agent.velocity, |v| v.linear)))
            .collect();
        let points = rows
            .iter()
            .zip(&positions)
            .map(|(r, p)| (r.e, [p.x, p.y, 0.0]))
            .collect();
        let index: BTreeMap<u64, usize> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| (r.e.to_bits(), i))
            .collect();
        Some((velocities, og_ecs::SpatialHash::new(points, 0.0)?, index))
    } else {
        None
    };
    let mut scratch = Scratch::default();
    let mut budget = if desc.plans_per_tick == 0 {
        u32::MAX
    } else {
        desc.plans_per_tick
    };
    // Links reached this tick, in agent order (og.NavLinkReached).
    let mut reached: Vec<NavLinkReached> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let Row {
            e,
            mut agent,
            pos,
            steer: steering,
            vel,
            filter,
            crowd: crowding,
        } = *row;
        let dt = ctx.group_dt(groups.as_ref(), e, scaled);
        let filter = filter.map(|f| f.filter()).unwrap_or_default();
        let key = e.to_bits();
        let p2 = plane.to_plane(pos);
        // A body's actual velocity (physics, or the game, may have changed it).
        let mut v2 = plane.to_plane(vel.map_or(agent.velocity, |v| v.linear));
        let mut desired = Vec2::ZERO;
        let mut new_pos = pos;
        // The nav data's frame: a point of it plus `k` is in the world's.
        let k = navs.offset(agent.nav, state.origin);
        let k2 = plane.to_plane(k);
        let ps = state.paths.entry(key).or_default();
        let nav = navs.get(agent.nav);
        // Part way across a link: the crossing goes on whatever the agent is
        // told, and nothing else moves it (the game, for a game crossing).
        if let Some(mut tr) = ps.traverse {
            let over = if tr.game {
                tr.finished
            } else {
                tr.elapsed += dt;
                tr.elapsed >= tr.duration
            };
            let mut t = ctx.get::<Transform>(e)?;
            let at = if over {
                tr.to
            } else if tr.game {
                t.position
            } else {
                tr.at()
            };
            agent.velocity = if dt > 0.0 {
                (at - pos) / dt
            } else {
                Vec3::ZERO
            };
            if over {
                ps.traverse = None;
                ps.next = (tr.corner + 2).min(ps.corners.len().saturating_sub(1) as u32);
                ps.poly = tr.land;
                agent.status = STATUS_MOVING;
            } else {
                ps.traverse = Some(tr);
                agent.status = STATUS_ON_LINK;
            }
            if t.position != at {
                t.position = at;
                ctx.set(e, t)?;
            }
            if let Some(mut v) = vel
                && v.linear != Vec3::ZERO
            {
                v.linear = Vec3::ZERO;
                ctx.set(e, v)?;
            }
            ctx.set(e, agent)?;
            continue;
        }
        let mut status = if agent.seeking {
            STATUS_MOVING
        } else {
            STATUS_IDLE
        };
        let seek_w = steering.map_or(1.0, |s| s.seek);
        if agent.seeking {
            match nav {
                Some(Nav::Flow { field, .. }) => {
                    let d = field.sample(p2 - k2);
                    let cost = field.cost_at(p2 - k2);
                    if cost == Some(0) || d == Vec2::ZERO {
                        status = if cost.is_some() {
                            STATUS_ARRIVED
                        } else {
                            STATUS_UNREACHABLE
                        };
                    } else {
                        desired = d * agent.speed;
                    }
                    agent.waypoint = plane.to_world(p2 + d, plane.height(pos));
                    agent.remaining = cost.map_or(0.0, |c| c as f32 / 1000.0 * field.cell_size);
                }
                _ => {
                    let version = match nav {
                        Some(Nav::Grid(g)) => g.version(),
                        Some(Nav::Mesh(m)) => m.version(),
                        _ => 0,
                    };
                    let moved = ps.target.distance(agent.target) > agent.repath_distance.max(1e-4);
                    let due = agent.repath_ticks > 0 && ps.age >= agent.repath_ticks;
                    // Off its path (teleported, knocked away): plan again from here.
                    let off = ps.corners.len() >= 2 && {
                        let n = (ps.next as usize).clamp(1, ps.corners.len() - 1);
                        let (a, b) = (
                            plane.to_plane(ps.corners[n - 1]),
                            plane.to_plane(ps.corners[n]),
                        );
                        let ab = b - a;
                        let t = if ab.length_squared() > 1e-12 {
                            ((p2 - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        p2.distance(a + ab * t) > OFF_PATH.max(agent.radius * 4.0)
                    };
                    let stale = ps.nav != agent.nav
                        || ps.version != version
                        || ps.corners.is_empty()
                        || off;
                    if (moved || due || stale) && budget > 0 {
                        budget -= 1;
                        let Plan {
                            corners,
                            status: st,
                            poly,
                            links,
                        } = match nav {
                            Some(n) => {
                                let mut p = plan(
                                    n,
                                    plane,
                                    pos - k,
                                    agent.target - k,
                                    &agent,
                                    &filter,
                                    &mut scratch,
                                );
                                p.corners
                                    .iter_mut()
                                    .for_each(|c| *c = og_core::math::offset(*c, k));
                                p
                            }
                            None => (vec![pos, agent.target], STATUS_MOVING, NO_POLY).into(),
                        };
                        ps.corners = corners;
                        ps.links = links;
                        ps.next = 1.min(ps.corners.len().saturating_sub(1) as u32);
                        ps.target = agent.target;
                        ps.nav = agent.nav;
                        ps.version = version;
                        ps.age = 0;
                        ps.status = st;
                        if poly != NO_POLY || ps.poly == NO_POLY {
                            ps.poly = poly;
                        }
                        agent.plans += 1;
                    } else {
                        ps.age += 1;
                    }
                    status = ps.status;
                    if ps.corners.len() >= 2 {
                        let last = ps.corners.len() - 1;
                        // A corner is passed within reach of it, or once the agent is
                        // past it towards the next one (fast agents overshoot).
                        let reach = agent
                            .stop_distance
                            .max(agent.radius * 0.5)
                            .max(agent.speed * dt * 1.5)
                            .max(0.05);
                        while (ps.next as usize) < last {
                            let c = plane.to_plane(ps.corners[ps.next as usize]);
                            let n = plane.to_plane(ps.corners[ps.next as usize + 1]);
                            let d = c.distance(p2);
                            let past = (p2 - c).dot(n - c) > 0.0 && d < reach * 4.0 + agent.radius;
                            if d <= reach || past {
                                ps.next += 1;
                                // Never past a link's start: it is crossed, not walked.
                                if ps.links.iter().any(|l| l.corner + 1 == ps.next) {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        // At a link's start: across it from the next tick.
                        if let Some(pl) = ps.links.iter().find(|l| l.corner + 1 == ps.next).copied()
                            && let Some(Nav::Mesh(m)) = nav
                        {
                            match m.off_mesh_link(pl.link) {
                                Some(l) if l.open => {
                                    let (a, b) = if pl.reverse {
                                        (
                                            og_core::math::offset(l.end, k),
                                            og_core::math::offset(l.start, k),
                                        )
                                    } else {
                                        (
                                            og_core::math::offset(l.start, k),
                                            og_core::math::offset(l.end, k),
                                        )
                                    };
                                    let land = m
                                        .link_ends(pl.link)
                                        .map_or(NO_POLY, |(s, en)| if pl.reverse { s } else { en });
                                    let speed = agent.speed.max(0.1);
                                    let dist = a.distance(b);
                                    let duration = if l.duration > 0.0 {
                                        l.duration
                                    } else {
                                        match l.kind {
                                            LinkKind::Jump => (dist / speed).max(0.3),
                                            LinkKind::Ladder => dist / (speed * 0.5),
                                            LinkKind::Door => dist / speed,
                                            LinkKind::Teleport => 0.0,
                                        }
                                    };
                                    ps.traverse = Some(Traverse {
                                        link: pl.link,
                                        kind: l.kind,
                                        from: pos,
                                        to: b,
                                        elapsed: 0.0,
                                        duration,
                                        game: l.game,
                                        finished: false,
                                        corner: pl.corner,
                                        land,
                                    });
                                    reached.push(NavLinkReached {
                                        agent: key,
                                        mesh: agent.nav,
                                        link: pl.link,
                                        kind: l.kind as u32,
                                        start: a,
                                        end: b,
                                        game: l.game,
                                    });
                                    agent.status = STATUS_ON_LINK;
                                    agent.velocity = Vec3::ZERO;
                                    agent.waypoint = b;
                                    if let Some(mut v) = vel
                                        && v.linear != Vec3::ZERO
                                    {
                                        v.linear = Vec3::ZERO;
                                        ctx.set(e, v)?;
                                    }
                                    ctx.set(e, agent)?;
                                    continue;
                                }
                                // Shut (or gone) since the plan: back to its start to
                                // wait; the re-plan the change asked for finds
                                // another way, or none.
                                _ => ps.next = pl.corner,
                            }
                        }
                        let wp = ps.corners[ps.next as usize];
                        agent.waypoint = wp;
                        let w2 = plane.to_plane(wp);
                        let goal = plane.to_plane(ps.corners[last]);
                        let mut remaining = w2.distance(p2);
                        for k in ps.next as usize..last {
                            remaining += plane
                                .to_plane(ps.corners[k])
                                .distance(plane.to_plane(ps.corners[k + 1]));
                        }
                        agent.remaining = remaining;
                        if (ps.next as usize) == last
                            && goal.distance(p2) <= agent.stop_distance.max(1e-3)
                        {
                            if status == STATUS_MOVING {
                                status = STATUS_ARRIVED;
                            }
                        } else if (ps.next as usize) == last {
                            desired = steer::arrive(
                                p2,
                                w2,
                                agent.speed,
                                agent.slow_radius,
                                agent.stop_distance,
                            );
                        } else {
                            desired = steer::seek(p2, w2, agent.speed);
                        }
                    } else if status != STATUS_UNREACHABLE {
                        status = STATUS_ARRIVED;
                    }
                }
            }
            desired *= seek_w;
        } else {
            ps.corners.clear();
            ps.age = 0;
            agent.remaining = 0.0;
        }
        // Steering behaviours.
        if let Some(mut s) = steering {
            if s.flee > 0.0 {
                desired += steer::flee(p2, plane.to_plane(s.flee_from), agent.speed, s.flee_radius)
                    * s.flee;
            }
            if s.wander > 0.0 {
                let heading = if v2.length_squared() > 1e-6 {
                    v2
                } else {
                    Vec2::X
                };
                let circle = steer::WanderCircle {
                    radius: s.wander_radius,
                    distance: s.wander_distance,
                    jitter: s.wander_jitter,
                };
                let (w, a) = steer::wander(
                    heading,
                    s.wander_angle,
                    noise(key, tick),
                    circle,
                    dt,
                    agent.speed,
                );
                s.wander_angle = a;
                desired += w * s.wander;
            }
            let mover = |target: u64| -> Option<(Vec2, Vec2)> {
                if target == 0 {
                    return None;
                }
                let t = Entity::from_bits(target);
                if !ctx.is_alive(t) {
                    return None;
                }
                let tp = ctx.get::<Transform>(t).ok()?.position;
                let tv = ctx
                    .get::<NavAgent>(t)
                    .ok()
                    .map(|a| a.velocity)
                    .or_else(|| ctx.get::<Velocity>(t).ok().map(|v| v.linear))
                    .unwrap_or(Vec3::ZERO);
                Some((plane.to_plane(tp), plane.to_plane(tv)))
            };
            if let Some((tp, tv)) = mover(s.pursue) {
                desired += steer::pursue(p2, tp, tv, agent.speed) * s.pursue_weight;
                status = STATUS_MOVING;
            }
            if let Some((tp, tv)) = mover(s.evade) {
                desired += steer::evade(p2, tp, tv, agent.speed, s.flee_radius) * s.evade_weight;
            }
            if s.separation > 0.0 {
                let r = s.separation_radius.max(agent.radius * 2.0);
                let (cx, cy) = ((p2.x / cell).floor() as i32, (p2.y / cell).floor() as i32);
                let reachc = (r / cell).ceil() as i32;
                let mut others = Vec::new();
                for by in cy - reachc..=cy + reachc {
                    for bx in cx - reachc..=cx + reachc {
                        if let Some(list) = buckets.get(&(bx, by)) {
                            for &j in list {
                                if j != i {
                                    others.push(positions[j]);
                                }
                            }
                        }
                    }
                }
                desired += steer::separation(p2, others, r, agent.speed) * s.separation;
            }
            if s.avoidance > 0.0
                && let Some(ray) = &desc.raycast
            {
                let heading = if v2.length_squared() > 1e-4 {
                    v2
                } else {
                    desired
                };
                let origin = pos;
                let a = steer::avoid(heading, s.avoid_distance, agent.speed, |d| {
                    let dir = plane.to_world(d, 0.0);
                    ray(origin, dir, s.avoid_distance, s.avoid_mask, e).map(|(dist, n)| {
                        steer::Feeler {
                            distance: dist,
                            normal: plane.to_plane(n),
                        }
                    })
                });
                desired += a * s.avoidance;
            }
            if s != steering.unwrap() {
                ctx.set(e, s)?;
            }
        }
        if !agent.drive {
            agent.velocity = vel.map_or(Vec3::ZERO, |v| v.linear);
            agent.status = status;
            ctx.set(e, agent)?;
            continue;
        }
        // Velocity and position.
        let speed_cap = agent.speed.max(0.0);
        if let (Some(c), Some((velocities, index, at))) = (crowding, &crowd) {
            desired = crowd_velocity(
                &CrowdView {
                    rows: &rows,
                    positions: &positions,
                    velocities,
                    index,
                    at,
                },
                i,
                &c,
                desired.clamp_length_max(speed_cap),
                speed_cap,
                match nav {
                    Some(Nav::Mesh(m)) => Some((m.as_ref(), ps.poly, k)),
                    _ => None,
                },
                plane,
                dt,
            );
        }
        v2 = steer::accelerate(
            v2,
            desired.clamp_length_max(speed_cap),
            agent.acceleration,
            dt,
            speed_cap,
        );
        if desired == Vec2::ZERO && v2.length_squared() < 1e-6 {
            v2 = Vec2::ZERO;
        }
        let height = plane.height(pos);
        match vel {
            Some(mut v) => {
                let keep = plane.height(v.linear);
                v.linear = plane.to_world(v2, keep);
                ctx.set(e, v)?;
            }
            None => {
                let step = p2 + v2 * dt;
                new_pos = plane.to_world(step, height);
                // An agent at rest is left where it stands; one that moves is
                // walked along the surface from the polygon it stands on.
                if let Some(Nav::Mesh(m)) = nav
                    && dt > 0.0
                    && step != p2
                {
                    let poly = locate(m, ps.poly, pos - k, agent.radius);
                    ps.poly = poly;
                    if m.poly(poly).is_some() {
                        let (np, at) = m.move_along(poly as PolyRef, pos - k, new_pos - k);
                        let at = og_core::math::offset(at, k);
                        ps.poly = np;
                        // The motion the surface allowed is the velocity.
                        let moved = plane.to_plane(at) - p2;
                        v2 = if dt > 0.0 { moved / dt } else { v2 };
                        new_pos = at;
                    }
                }
            }
        }
        agent.velocity = plane.to_world(v2, 0.0);
        agent.status = status;
        let mut t = ctx.get::<Transform>(e)?;
        let changed_pos = new_pos != t.position;
        t.position = new_pos;
        let mut changed_rot = false;
        if agent.face_movement && v2.length_squared() > 0.0025 {
            let yaw = match plane {
                Plane::Xz => og_core::math::atan2(v2.x, v2.y),
                Plane::Xy => og_core::math::atan2(v2.y, v2.x),
            };
            let axis = match plane {
                Plane::Xz => Vec3::Y,
                Plane::Xy => Vec3::Z,
            };
            let want = og_core::math::quat_from_axis_angle(axis, yaw);
            let have = t.quaternion.normalize();
            let angle = have.angle_between(want);
            let max = agent.turn_speed * dt;
            let q = if angle <= max || angle < 1e-5 {
                want
            } else {
                // A yaw step about the plane's axis through og-core's
                // deterministic math (glam's slerp uses the host's sinf and
                // drifted an ulp between Windows and Linux): the current
                // heading, turned towards `yaw` by at most `max`.
                use std::f32::consts::{PI, TAU};
                let heading = 2.0 * og_core::math::atan2(have.xyz().dot(axis), have.w);
                let mut delta = yaw - heading;
                while delta > PI {
                    delta -= TAU;
                }
                while delta < -PI {
                    delta += TAU;
                }
                og_core::math::quat_from_axis_angle(axis, heading + delta.clamp(-max, max))
            };
            changed_rot = q != t.quaternion;
            t.quaternion = q;
        }
        if changed_pos || changed_rot {
            ctx.set(e, t)?;
        }
        ctx.set(e, agent)?;
    }
    if !reached.is_empty() && ctx.id_by_name(NavLinkReached::NAME).is_ok() {
        for r in reached {
            ctx.send(r)?;
        }
    }
    Ok(())
}
