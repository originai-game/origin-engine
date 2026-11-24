//! Foot locks, joint limits, spring bones and IK evaluation.

use super::*;

/// One foot lock this tick: whether the clips plant the foot, the lock's
/// next state from `prev`, and the leg bent to hold the foot there.
pub(super) fn foot_lock(
    sh: &Shared,
    w: &mut Work,
    fl: &mut FootLock,
    prev: Lock,
    dt: f32,
    limits: &Limits,
) -> Result<Lock> {
    let foot = joint(w, fl.foot, "og.FootLock.foot")?;
    let Some(leg) = ik::chain(&w.asset, foot, 2) else {
        return Err(invalid(format!(
            "og.FootLock: joint {} has fewer than 2 bones above it",
            fl.foot
        )));
    };
    // Planted when the clips planting it hold at least half the stance.
    let (mut on, mut total) = (0.0f32, 0.0f32);
    for &(clip, t, weight) in &w.stance {
        if weight <= 0.0 {
            continue;
        }
        let Some(p) = sh.plants(w.skeleton.asset, clip, foot) else {
            continue;
        };
        let Some(c) = sh.clips.get(&clip.0) else {
            continue;
        };
        total += weight;
        if p.planted_at(c, t) {
            on += weight;
        }
    }
    let planted = total > 0.0 && on >= 0.5 * total;
    let frame = w.world_matrix();
    let model = w.asset.model_space(&w.pose.nodes);
    let animated = frame.transform_point3(model[foot as usize].translation);
    let ray_at = match prev {
        Lock::Held(at) | Lock::Releasing(at, _) => at,
        Lock::Free => animated,
    };
    let ground = if fl.ground_reach > 0.0 {
        sh.physics
            .as_ref()
            .map(|p| p.tick_query())
            .transpose()?
            .map(|q| {
                q.raycast(&og_physics::RaycastDesc {
                    origin: ray_at + Vec3::Y * fl.ground_reach,
                    direction: -Vec3::Y,
                    max_distance: 2.0 * fl.ground_reach,
                    filter: og_physics::QueryFilter {
                        exclude: Some(w.entity),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            })
            .transpose()?
            .flatten()
    } else {
        None
    };
    let planted_at = ground.as_ref().map_or(animated, |h| h.point);
    let release = fl.release.max(0.0);
    let lifted = |at: Vec3| {
        if release > 0.0 {
            Lock::Releasing(at, 0.0)
        } else {
            Lock::Free
        }
    };
    let lock = match prev {
        Lock::Free if planted => Lock::Held(planted_at),
        Lock::Free => Lock::Free,
        // A teleport must drop the old anchor, not fade towards it.
        Lock::Held(at) | Lock::Releasing(at, _)
            if fl.max_distance > 0.0 && (at - animated).length() > fl.max_distance =>
        {
            Lock::Free
        }
        Lock::Held(at) if !planted => lifted(at),
        Lock::Held(at) => Lock::Held(at),
        Lock::Releasing(from, t) => {
            let t = t + dt;
            if t >= release {
                Lock::Free
            } else {
                Lock::Releasing(from, t)
            }
        }
    };
    let target = match lock {
        Lock::Free => None,
        Lock::Held(at) => Some(at),
        Lock::Releasing(from, t) => Some(from + (animated - from) * (t / release)),
    };
    if let Some(target) = target {
        let target = ground
            .as_ref()
            .map_or(target, |h| Vec3::new(target.x, h.point.y, target.z));
        let to = frame.inverse().transform_point3(target);
        let turned = model[foot as usize].rotation;
        if fl.pelvis_drop > 0.0 {
            let pelvis = w.asset.nodes[leg[0] as usize].parent;
            if pelvis != NONE {
                let drop =
                    (animated.y - target.y).clamp(0.0, fl.pelvis_drop) * fl.weight.clamp(0.0, 1.0);
                let parent = w.asset.nodes[pelvis as usize].parent;
                let pm = if parent == NONE {
                    Mat4::IDENTITY
                } else {
                    w.asset.model_matrices(&w.pose.nodes)[parent as usize]
                };
                w.pose.nodes[pelvis as usize].translation +=
                    (frame * pm).inverse().transform_vector3(-Vec3::Y * drop);
            }
        }
        let lim = chain_limits(w, &leg, limits);
        ik::two_bone(
            &w.asset,
            &mut w.pose.nodes,
            &leg,
            to,
            None,
            fl.weight.clamp(0.0, 1.0),
            &lim,
        );
        // The foot keeps its animated orientation.
        let parent = w.asset.nodes[foot as usize].parent as usize;
        let m = w.asset.model_space(&w.pose.nodes);
        let turned = ground.as_ref().map_or(turned, |h| {
            let normal = frame
                .transpose()
                .transform_vector3(h.normal)
                .normalize_or(Vec3::Y);
            unit(rotation_arc(turned * Vec3::Y, normal) * turned)
        });
        let local = unit(m[parent].rotation.conjugate() * turned);
        w.pose.nodes[foot as usize].rotation = nlerp(
            w.pose.nodes[foot as usize].rotation,
            local,
            fl.weight.clamp(0.0, 1.0),
        );
    }
    fl.planted = planted;
    fl.locked = matches!(lock, Lock::Held(_));
    Ok(lock)
}

/// One hang this tick: `w`'s Transform turned so its +Y runs from the grip
/// towards the anchor (by `lay`, from upright) with its front towards
/// `facing`, and moved so the grip joint, as this tick's clips pose it,
/// sits at `point`; by `weight` from the Transform it had. Its limbs'
/// targets go into `reaches`, by chain entity.
pub(super) fn hold(w: &mut Work, hang: &mut Hang, reaches: &mut BTreeMap<u64, Vec3>) -> Result<()> {
    let grip = joint(w, hang.grip, "og.Hang.grip")?;
    let Some(t) = w.transform.as_mut() else {
        return Ok(());
    };
    let weight = hang.weight.clamp(0.0, 1.0);
    let rope = hang.anchor - hang.point;
    let along = rope.normalize_or_zero();
    let lay = hang.lay.clamp(0.0, 1.0);
    let up = (Vec3::Y * (1.0 - lay) + along * lay)
        .try_normalize()
        .unwrap_or(Vec3::Y);
    // The front: `facing`, else the entity's own, less its part along up.
    let own = t.quaternion * Vec3::Z;
    let want = if hang.facing.length_squared() > 0.0 {
        hang.facing
    } else {
        own
    };
    let front = (want - up * want.dot(up))
        .try_normalize()
        .or_else(|| (own - up * own.dot(up)).try_normalize())
        .unwrap_or_else(|| up.any_orthonormal_vector());
    let side = up.cross(front);
    let turned = unit(Quat::from_mat3(&og_core::glam::Mat3::from_cols(
        side, up, front,
    )));
    let model = w.asset.model_space(&w.pose.nodes);
    let g = model[grip as usize].translation;
    let from = t.position + t.quaternion * (t.scale * g);
    let rotation = nlerp(t.quaternion, turned, weight);
    let at = from + (hang.point - from) * weight;
    t.quaternion = rotation;
    t.position = at - rotation * (t.scale * g);
    for (i, &chain) in hang.limbs.as_slice().iter().enumerate() {
        if chain == 0 {
            continue;
        }
        let reach = hang.reach.as_slice().get(i).copied().unwrap_or(0.0);
        reaches.insert(chain, hang.point + along * reach);
    }
    hang.held = weight > 0.0;
    Ok(())
}

/// Joint limits by (skeleton entity, node).
pub(super) type Limits = BTreeMap<(u64, u32), JointLimit>;

/// The node of joint `joint` of `w`'s skin, or an error naming `field`.
pub(super) fn joint(w: &Work, joint: u32, field: &str) -> Result<u32> {
    let skin = w.skeleton.skin;
    w.asset
        .joint_node(skin, joint)
        .ok_or_else(|| invalid(format!("{field} {joint} is not a joint of skin {skin}")))
}

/// The limits of `nodes` on `w`'s skeleton, aligned with them (empty when
/// none is limited, so the solvers take their unlimited path).
fn chain_limits(w: &Work, nodes: &[u32], limits: &Limits) -> Vec<Option<JointLimit>> {
    let key = w.entity.to_bits();
    let out: Vec<Option<JointLimit>> = nodes
        .iter()
        .map(|&n| limits.get(&(key, n)).copied())
        .collect();
    if out.iter().any(Option::is_some) {
        out
    } else {
        Vec::new()
    }
}

/// A point of `space` in the skeleton's model space (world: the inverse
/// of the entity's Transform).
fn model_point(space: u32, frame: Mat4, p: Vec3) -> Vec3 {
    match space {
        IK_WORLD => frame.inverse().transform_point3(p),
        _ => p,
    }
}

/// A direction of `space` in the skeleton's model space.
fn model_direction(space: u32, frame: Mat4, v: Vec3) -> Vec3 {
    match space {
        IK_WORLD => frame.inverse().transform_vector3(v),
        _ => v,
    }
}

/// Where joint `n`'s tail is in its own frame: the next joint of `chain`
/// after it, else its first child, else its parent bone continued at the
/// same length (the rest pose's).
fn tail_of(asset: &SkeletonAsset, pose: &[Trs], chain: &[u32], i: usize) -> Vec3 {
    let n = chain[i];
    let child = chain.get(i + 1).copied().or_else(|| {
        asset
            .nodes
            .iter()
            .position(|c| c.parent == n)
            .map(|c| c as u32)
    });
    match child {
        Some(c) => pose[c as usize].translation,
        None => {
            let rest = &asset.nodes[n as usize].rest;
            let s = rest.scale;
            let along = rest.rotation.conjugate() * rest.translation;
            if s.x != 0.0 && s.y != 0.0 && s.z != 0.0 {
                along / s
            } else {
                Vec3::ZERO
            }
        }
    }
}

/// A spring collider placed for one chain: in the spring's frame, a
/// segment (a point for a sphere) and the distance a tail keeps from it.
struct Placed {
    a: Vec3,
    b: Vec3,
    reach: f32,
    normal: Option<Vec3>,
}

/// Closest points, including parallel and zero-length collider segments.
fn closest_segments(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> (Vec3, Vec3) {
    let (u, v, r) = (b - a, d - c, a - c);
    let (uu, vv, uv, ur, vr) = (u.dot(u), v.dot(v), u.dot(v), u.dot(r), v.dot(r));
    if vv == 0.0 {
        return (og_core::math::closest_on_segment(c, a, b), c);
    }
    if uu == 0.0 {
        return (a, og_core::math::closest_on_segment(a, c, d));
    }
    let denom = uu * vv - uv * uv;
    let mut s = if denom > 0.0 {
        ((uv * vr - ur * vv) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let t = (uv * s + vr) / vv;
    if t < 0.0 {
        s = (-ur / uu).clamp(0.0, 1.0);
    }
    if t > 1.0 {
        s = ((uv - ur) / uu).clamp(0.0, 1.0);
    }
    (a + u * s, c + v * t.clamp(0.0, 1.0))
}

/// One spring chain this tick: each joint's tail, top first, stepped
/// towards where the pose (with the joints above already swung) puts it,
/// held at the bone's length, pushed out of the colliders it meets (and
/// held at the length again), and the joint turned to point at it.
pub(super) fn spring(
    w: &mut Work,
    sb: &mut SpringBone,
    prev: Option<&SpringState>,
    colliders: &[(u32, SpringCollider)],
    limits: &Limits,
    physics: Option<&og_physics::PhysicsSystem>,
    dt: f32,
) -> Result<SpringState> {
    let node = joint(w, sb.joint, "og.SpringBone.joint")?;
    let bones = sb.bones.clamp(1, 16);
    let Some(chain) = ik::chain(&w.asset, node, bones - 1) else {
        return Err(invalid(format!(
            "og.SpringBone: joint {} has fewer than {} joints above it",
            sb.joint,
            bones - 1
        )));
    };
    let frame = match sb.space {
        SPRING_WORLD => w.world_matrix(),
        _ => Mat4::IDENTITY,
    };
    let to_model = frame.inverse();
    let matrices = w.asset.model_matrices(&w.pose.nodes);
    let root = (frame * matrices[chain[0] as usize]).transform_point3(Vec3::ZERO);
    let prev = prev.filter(|p| {
        p.tails.len() == chain.len()
            && (sb.teleport_distance <= 0.0 || p.root.distance(root) <= sb.teleport_distance)
    });
    let (k, c) = (sb.stiffness.max(0.0), sb.drag.max(0.0));
    let weight = sb.weight.clamp(0.0, 1.0);
    // The colliders of this skeleton the chain is kept out of, placed in
    // the spring's frame from the pose as it is when the chain starts.
    let scale = |m: Mat4| {
        m.x_axis
            .truncate()
            .length()
            .max(m.y_axis.truncate().length())
            .max(m.z_axis.truncate().length())
    };
    let radius = sb.radius.max(0.0);
    let placed: Vec<Placed> = colliders
        .iter()
        .filter(|(_, c)| c.skeleton == sb.skeleton && c.layers & sb.colliders != 0)
        .map(|&(node, c)| {
            let m = frame * matrices[node as usize];
            let a = m.transform_point3(c.offset);
            let b = if c.shape == SPRING_CAPSULE {
                m.transform_point3(c.tail)
            } else {
                a
            };
            Placed {
                a,
                b,
                reach: c.radius.max(0.0) * scale(m),
                normal: (c.shape == SPRING_PLANE).then(|| {
                    m.inverse()
                        .transpose()
                        .transform_vector3(c.tail)
                        .normalize_or_zero()
                }),
            }
        })
        .collect();
    let mut parent = match w.asset.nodes[chain[0] as usize].parent {
        NONE => Mat4::IDENTITY,
        p => matrices[p as usize],
    };
    let world_query = if sb.world_colliders != 0 {
        physics.map(|p| p.tick_query()).transpose()?
    } else {
        None
    };
    let mut tails = Vec::with_capacity(chain.len());
    for (i, &n) in chain.iter().enumerate() {
        let tail = if i + 1 == chain.len() && sb.tail_offset != Vec3::ZERO {
            sb.tail_offset
        } else {
            tail_of(&w.asset, &w.pose.nodes, &chain, i)
        };
        let local = w.pose.nodes[n as usize];
        let m = parent * local.to_mat4();
        let head = frame.transform_point3(m.transform_point3(Vec3::ZERO));
        let target = frame.transform_point3(m.transform_point3(tail));
        let segment_radius = radius * scale(frame * m);
        let length = (target - head).length();
        let (mut x, mut v) = prev.map_or((target, Vec3::ZERO), |p| p.tails[i]);
        if length < 1e-6 {
            // No bone to swing.
            tails.push((target, Vec3::ZERO));
            parent = m;
            continue;
        }
        let x0 = x;
        if dt > 0.0 {
            let wind = physics.map_or(Vec3::ZERO, |p| {
                let at = if sb.space == SPRING_WORLD {
                    x
                } else {
                    w.world_matrix().transform_point3(x)
                };
                let wind = p.air_at(at);
                if sb.space == SPRING_WORLD {
                    wind
                } else {
                    w.world_matrix().inverse().transform_vector3(wind)
                }
            });
            v = (v + ((target - x) * k + sb.gravity + wind * sb.wind) * dt)
                / (1.0 + dt * (c + sb.wind.max(0.0)) + dt * dt * k);
            if sb.max_speed > 0.0 {
                v = v.clamp_length_max(sb.max_speed);
            }
            x += v * dt;
        }
        // Held at the bone's length from the head.
        let hold = |x: Vec3| {
            let d = x - head;
            let l = d.length();
            if l > 1e-9 {
                head + d * (length / l)
            } else {
                target
            }
        };
        x = hold(x);
        // Out of every collider it meets, in order, still at the length.
        for _ in 0..4 {
            for p in &placed {
                let reach = p.reach + segment_radius;
                if let Some(normal) = p.normal {
                    let distance = (x - p.a).dot(normal);
                    if distance < reach {
                        x = hold(x + normal * (reach - distance));
                    }
                    continue;
                }
                // Closest points of the whole bone and collider segments.
                let (on_bone, near) = closest_segments(head, x, p.a, p.b);
                if on_bone.distance(near) < reach {
                    let fraction = ((on_bone - head).length() / length).max(1e-4);
                    let normal = (on_bone - near)
                        .normalize_or((target - head).cross(Vec3::X).normalize_or(Vec3::Z));
                    x = hold(x + normal * ((reach - on_bone.distance(near)) / fraction));
                }
            }
            if let Some(q) = &world_query {
                let world = if sb.space == SPRING_WORLD {
                    Mat4::IDENTITY
                } else {
                    w.world_matrix()
                };
                let a = world.transform_point3(head);
                let b = world.transform_point3(x);
                let reach = radius * scale(w.world_matrix() * m);
                let hit = q.shape_cast(&og_physics::ShapeCastDesc {
                    shape: og_physics::ShapeDesc {
                        shape: og_physics::SHAPE_SPHERE,
                        radius: reach,
                        ..Default::default()
                    },
                    position: a,
                    direction: b - a,
                    max_distance: a.distance(b),
                    filter: og_physics::QueryFilter {
                        mask: sb.world_colliders,
                        exclude: Some(w.entity),
                        ..Default::default()
                    },
                    ..Default::default()
                })?;
                if let Some(hit) = hit {
                    let penetration = (hit.point + hit.normal * reach - b).dot(hit.normal);
                    if penetration > 0.0 {
                        x = hold(
                            world
                                .inverse()
                                .transform_point3(b + hit.normal * penetration),
                        );
                    }
                }
            }
        }
        let from = local.rotation * (local.scale * tail);
        let to = parent
            .inverse()
            .transform_point3(to_model.transform_point3(x))
            - local.translation;
        let mut swung = unit(rotation_arc(from, to) * local.rotation);
        if let Some(limit) = limits.get(&(sb.skeleton, n)) {
            swung = limit.clamp(w.asset.nodes[n as usize].rest.rotation, swung);
        }
        w.pose.nodes[n as usize].rotation = nlerp(local.rotation, swung, weight);
        parent *= w.pose.nodes[n as usize].to_mat4();
        // Constraints and blend weight can change the actual tail. Persist
        // that tail rather than integrating from an unreachable position.
        let actual = frame.transform_point3(parent.transform_point3(tail));
        sb.lag = sb.lag.max((actual - target).length());
        if dt > 0.0 {
            v = (actual - x0) / dt;
        }
        if sb.max_speed > 0.0 {
            v = v.clamp_length_max(sb.max_speed);
        }
        tails.push((actual, v));
    }
    Ok(SpringState {
        skeleton: sb.skeleton,
        space: sb.space,
        root,
        tails,
    })
}

pub(super) fn aim(w: &mut Work, look: &mut LookAt, limits: &Limits) -> Result<()> {
    let node = joint(w, look.joint, "og.LookAt.joint")?;
    let bones = look.bones.clamp(1, 8);
    let Some(nodes) = ik::chain(&w.asset, node, bones - 1) else {
        return Err(invalid(format!(
            "og.LookAt: joint {} has fewer than {} joints above it",
            look.joint,
            bones - 1
        )));
    };
    let frame = w.transform.map_or(Mat4::IDENTITY, |t| t.matrix());
    let target = model_point(look.space, frame, look.target);
    let up = (look.up != Vec3::ZERO).then(|| model_direction(look.space, frame, look.up));
    let lim = chain_limits(w, &nodes, limits);
    look.clamped = ik::look_at(&w.asset, &mut w.pose.nodes, &nodes, target, up, look, &lim);
    Ok(())
}

pub(super) fn solve(w: &mut Work, chain: &mut IkChain, limits: &Limits) -> Result<()> {
    let tip = joint(w, chain.tip, "og.IkChain.tip")?;
    let bones = if chain.solver == IK_TWO_BONE {
        2
    } else {
        chain.bones.max(1)
    };
    let Some(nodes) = ik::chain(&w.asset, tip, bones) else {
        return Err(invalid(format!(
            "og.IkChain: joint {} has fewer than {bones} bones above it",
            chain.tip
        )));
    };
    let frame = w.transform.map_or(Mat4::IDENTITY, |t| t.matrix());
    let to_model = |p: Vec3| model_point(chain.space, frame, p);
    let target = to_model(chain.target);
    let weight = chain.weight.clamp(0.0, 1.0);
    let lim = chain_limits(w, &nodes, limits);
    let result = if chain.solver == IK_TWO_BONE {
        let pole = (chain.pole != Vec3::ZERO).then(|| to_model(chain.pole));
        ik::two_bone(
            &w.asset,
            &mut w.pose.nodes,
            &nodes,
            target,
            pole,
            weight,
            &lim,
        )
    } else {
        ik::fabrik(
            &w.asset,
            &mut w.pose.nodes,
            &nodes,
            target,
            chain.iterations,
            chain.tolerance,
            weight,
            &lim,
        )
    };
    chain.error = result.error;
    chain.reached = result.reachable && result.error <= chain.tolerance;
    Ok(())
}
