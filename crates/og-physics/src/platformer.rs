//! The 2D platformer controller (the 2D profile only), run after the
//! rigid-body step like the character controller: timers, run
//! acceleration, gravity from jump height and time to apex, jumps (ground,
//! coyote, buffered, air, wall), the move against the world with one-way
//! platforms, corner correction, then ground and wall probes.

use super::ext::{body_of, carried, entity_of, push};
use super::mover::Mover;
use super::sim::{BodyPoses, Sim};
use super::{pose, rp, unpose, unv, v};
use crate::gather::{Control, Scene};
use crate::schema::*;
use crate::schema_ext::*;
use crate::{ControlOut, StepOut};
use og_core::Result;
use og_core::glam::{Vec2, Vec3};

/// `x` moved towards `target` by at most `step`.
fn approach(x: f32, target: f32, step: f32) -> f32 {
    if x < target {
        (x + step).min(target)
    } else {
        (x - step).max(target)
    }
}

/// Whether a one-way platform's collider blocks a character whose feet are
/// at `feet` moving with vertical speed `vy`: only from above, falling, and
/// not while dropping through.
fn one_way_blocks(c: &rp::Collider, feet: f32, vy: f32, dropping: bool) -> bool {
    if dropping || vy > 0.0 {
        return false;
    }
    let top = c.compute_aabb().maxs.y;
    feet >= top - 0.05
}

pub(crate) fn platformers(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    sim: &mut Sim,
    poses: &BodyPoses,
    out: &mut StepOut,
) -> Result<()> {
    for (k, b) in scene.bodies.iter().enumerate() {
        let Some(Control::Platformer(mut p)) = b.control else {
            continue;
        };
        let Some(t) = sim.bodies.get(&b.entity.to_bits()) else {
            continue;
        };
        let handle = t.handle;
        let rb = &sim.world.bodies[handle];
        let Some(&ch) = rb.colliders().first() else {
            continue;
        };
        let collider = &sim.world.colliders[ch];
        let shape = collider.shared_shape().clone();
        let groups = collider.collision_groups();
        let local = *collider
            .position_wrt_parent()
            .unwrap_or(&rp::Pose::IDENTITY);
        let at = pose(b.position, b.quaternion) * local;
        let half_height = {
            let aabb = shape.compute_aabb(&rp::Pose::IDENTITY);
            (aabb.maxs.y - aabb.mins.y) * 0.5
        };
        // Ride the platform it stood on.
        let centre = unv(at.translation);
        let ride = match body_of(sim, p.ground) {
            Some(g) if p.grounded => carried(poses, &sim.world, g, centre),
            _ => Vec3::ZERO,
        };
        let centre = centre + ride;

        // The game's impulse, taken once: an upward one leaves the ground.
        let was_grounded = p.grounded;
        if p.impulse != Vec2::ZERO {
            p.velocity += Vec3::new(p.impulse.x, p.impulse.y, 0.0);
            if p.impulse.y > 0.0 {
                p.grounded = false;
                p.jumping = false;
                p.coyote = 0.0;
            }
            p.impulse = Vec2::ZERO;
        }

        // Timers and the jump press.
        let pressed = p.jump && !p.jump_was_held;
        p.coyote = if p.grounded {
            p.coyote_time
        } else {
            (p.coyote - dt).max(0.0)
        };
        p.buffer = if pressed {
            p.jump_buffer.max(dt)
        } else {
            (p.buffer - dt).max(0.0)
        };
        p.wall_lock = (p.wall_lock - dt).max(0.0);
        p.drop_time = (p.drop_time - dt).max(0.0);
        if p.grounded {
            p.air_jumps_left = p.air_jumps;
        }
        if p.down && p.grounded && p.one_way_layers != 0 {
            // Standing on a one-way platform? Drop through it.
            if ground_is_one_way(&sim.world, p.ground, p.one_way_layers) {
                p.drop_time = 0.25;
                p.grounded = false;
            }
        }

        // Run.
        let mut vx = p.velocity.x;
        let mut vy = p.velocity.y;
        let input = p.move_x.clamp(-1.0, 1.0);
        if p.wall_lock <= 0.0 {
            let target = input * p.max_speed;
            let (accel, decel) = if p.grounded {
                (p.acceleration, p.deceleration)
            } else {
                (p.air_acceleration, p.air_deceleration)
            };
            let turning = target != 0.0 && vx != 0.0 && target.signum() != vx.signum();
            let rate = if target == 0.0 {
                decel
            } else if turning {
                accel.max(decel)
            } else {
                accel
            };
            vx = approach(vx, target, rate * dt);
        }
        if input != 0.0 {
            p.facing = if input > 0.0 { 1 } else { -1 };
        }

        // Gravity from the jump's height and time to apex.
        let apex = p.time_to_apex.max(0.05);
        let base_g = 2.0 * p.jump_height.max(0.0) / (apex * apex);
        let jump_speed = base_g * apex;
        let world_scale = (-settings.gravity.y / 9.81).max(0.0);
        let mut g = base_g * world_scale;
        if vy < 0.0 {
            g *= p.fall_gravity.max(0.0);
        } else if p.jumping && !p.jump {
            g *= p.jump_cut.max(1.0);
        } else if p.jump && vy.abs() < p.apex_threshold {
            g *= p.apex_gravity.clamp(0.0, 1.0);
        }

        // Jumps: ground (or coyote), wall, air.
        let mut event = None;
        if p.buffer > 0.0 && (p.grounded || p.coyote > 0.0) {
            vy = jump_speed;
            p.buffer = 0.0;
            p.coyote = 0.0;
            p.jumping = true;
            p.grounded = false;
            event = Some(CHARACTER_JUMPED);
        } else if p.buffer > 0.0 && p.wall_jumps && p.on_wall != 0 && !p.grounded {
            vx = -(p.on_wall as f32) * p.wall_jump.x;
            vy = p.wall_jump.y;
            p.facing = -p.on_wall;
            p.buffer = 0.0;
            p.wall_lock = p.wall_jump_lock;
            p.jumping = true;
            event = Some(CHARACTER_WALL_JUMPED);
        } else if pressed && !p.grounded && p.coyote <= 0.0 && p.air_jumps_left > 0 {
            vy = jump_speed;
            p.air_jumps_left -= 1;
            p.buffer = 0.0;
            p.jumping = true;
            event = Some(CHARACTER_AIR_JUMPED);
        } else {
            vy -= g * dt;
        }
        if vy <= 0.0 {
            p.jumping = false;
        }
        vy = vy.max(-p.max_fall_speed);
        let pushing_wall = p.on_wall != 0 && input != 0.0 && input.signum() == p.on_wall as f32;
        if p.wall_jumps && pushing_wall && vy < 0.0 && !p.grounded {
            vy = vy.max(-p.wall_slide_speed);
        }

        // The move, with one-way platforms passable from below.
        let feet = centre.y - half_height;
        let dropping = p.drop_time > 0.0;
        let one_way = p.one_way_layers;
        let rising = vy;
        let predicate = move |_: rp::ColliderHandle, c: &rp::Collider| {
            one_way == 0
                || c.collision_groups().memberships.bits() & one_way == 0
                || one_way_blocks(c, feet, rising, dropping)
        };
        // What blocks it: the solids its layers and mask interact with
        // (one-way platforms only from above).
        let filter = rp::QueryFilter::new()
            .exclude_rigid_body(handle)
            .exclude_sensors()
            .groups(groups)
            .predicate(&predicate);
        let desired = Vec3::new(vx * dt, vy * dt, 0.0);
        let snap = if p.grounded && vy <= 0.0 && !p.jumping {
            p.snap_distance
        } else {
            0.0
        };
        let mover = Mover {
            qp: sim.world.query_pipeline_with_filter(filter),
            shape: &*shape,
            rot: at.rotation,
            skin: p.skin.max(0.001),
            walkable: og_core::math::cos(p.max_slope.clamp(0.0, 89.0).to_radians()),
            step_height: 0.0,
        };
        let mut moved = mover.travel(centre, desired, snap, false);
        let mut nudge = 0.0;
        // Corner correction: a rising jump clipped by a ceiling corner slides past it.
        if vy > 0.0 && moved.translation.y < desired.y - 1e-4 && p.corner_correction > 0.0 {
            let steps = 6;
            'search: for i in 1..=steps {
                let d = p.corner_correction * i as f32 / steps as f32;
                for dir in [1.0f32, -1.0] {
                    let side = rp::Pose::from_parts(v(centre + Vec3::X * (d * dir)), at.rotation);
                    if mover.qp.intersect_shape(side, &*shape).next().is_some() {
                        continue;
                    }
                    let options = super::ShapeCastOptions::with_max_time_of_impact(desired.y);
                    if mover
                        .qp
                        .cast_shape(&side, v(Vec3::Y), &*shape, options)
                        .is_none()
                    {
                        nudge = d * dir;
                        break 'search;
                    }
                }
            }
            if nudge != 0.0 {
                moved = mover.travel(centre + Vec3::X * nudge, desired, snap, false);
            }
        }
        let end = centre + Vec3::X * nudge + moved.translation;
        if vy > 0.0 && moved.translation.y < desired.y - 1e-4 {
            vy = 0.0;
            p.jumping = false;
            event = event.or(Some(CHARACTER_BUMPED));
        }
        if moved.translation.x.abs() < (desired.x.abs() - 1e-4) && desired.x != 0.0 {
            vx = moved.translation.x / dt;
        }
        let landed_speed = vy;
        let grounded = moved.ground.is_some() && vy <= 0.0;
        if grounded {
            vy = 0.0;
        }
        let under = rp::Pose::from_parts(v(end), at.rotation);
        let ground = if grounded {
            moved
                .ground
                .map(|(h, n)| (entity_of(&sim.world.colliders[h]), n))
        } else {
            None
        };
        // Walls: a short probe to each side while in the air.
        let mut on_wall = 0;
        if !grounded {
            let qp = sim.world.query_pipeline_with_filter(filter);
            for dir in [-1i32, 1] {
                let options = super::ShapeCastOptions::with_max_time_of_impact(p.skin * 3.0 + 0.02);
                if let Some((_, hit)) =
                    qp.cast_shape(&under, v(Vec3::X * dir as f32), &*shape, options)
                    && unv(hit.normal1).x * (dir as f32) < -0.7
                {
                    on_wall = dir;
                }
            }
        }
        p.on_wall = on_wall;
        p.grounded = grounded;
        p.ground = ground.map_or(0, |g| g.0);
        p.velocity = Vec3::new(vx, vy, 0.0);
        p.jump_was_held = p.jump;
        if event.is_none() {
            if grounded && !was_grounded {
                event = Some(CHARACTER_LANDED);
            } else if !grounded && was_grounded && !p.jumping {
                event = Some(CHARACTER_LEFT_GROUND);
            }
        }
        let new_body = rp::Pose::from_parts(v(end), at.rotation) * local.inverse();
        let (position, quaternion) = unpose(&new_body, b.position);
        out.moved[k] = Some((position, quaternion, None));
        push(&mut sim.world, &moved.hits, desired / dt);
        if let Some(kind) = event {
            out.events.push(CharacterEvent {
                entity: b.entity.to_bits(),
                kind,
                speed: if kind == CHARACTER_LANDED {
                    landed_speed
                } else {
                    vy
                },
            });
        }
        out.controls.push((b.entity, ControlOut::Platformer(p)));
    }
    Ok(())
}

/// Whether the entity a character stands on has a collider on `layers`.
fn ground_is_one_way(w: &rp::PhysicsWorld, ground: u64, layers: u32) -> bool {
    ground != 0
        && w.colliders.iter().any(|(_, c)| {
            entity_of(c) == ground && c.collision_groups().memberships.bits() & layers != 0
        })
}
