//! og.CharacterController, moved after the step against the world as it now is.

use super::*;

/// Moves every og.CharacterController after the step, against the world as
/// it now is: ride the platform it stood on, take the input and gravity,
/// jump on the rising edge of `jump`, slide, climb, step and snap.
pub(crate) fn characters(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    sim: &mut Sim,
    poses: &BodyPoses,
    out: &mut StepOut,
) -> Result<()> {
    for (k, b) in scene.bodies.iter().enumerate() {
        let Some(crate::gather::Control::Character(mut c)) = b.control else {
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
        let local = *collider
            .position_wrt_parent()
            .unwrap_or(&rp::Pose::IDENTITY);
        let body_at = pose(b.position, b.quaternion);
        let at = body_at * local;
        // What blocks it: the solids its layers and mask interact with.
        let filter = rp::QueryFilter::new()
            .exclude_rigid_body(handle)
            .exclude_sensors()
            .groups(collider.collision_groups());
        // Ride what it stood on.
        let foot = unv(at.translation);
        let ride = match body_of(sim, c.ground) {
            Some(g) if c.grounded => carried(poses, &sim.world, g, foot),
            _ => Vec3::ZERO,
        };
        let g = flat(settings.gravity) * c.gravity_scale;
        let mut vertical = if c.gravity_scale > 0.0 {
            c.velocity.y
        } else {
            0.0
        };
        c.coyote = if c.grounded {
            c.coyote_time
        } else {
            (c.coyote - dt).max(0.0)
        };
        let mut jumped = false;
        if c.jump && !c.jump_was_held && c.coyote > 0.0 && c.gravity_scale > 0.0 {
            vertical = c.jump_speed;
            c.coyote = 0.0;
            jumped = true;
        }
        if c.gravity_scale > 0.0 {
            vertical = (vertical + g.y * dt).max(-c.max_fall_speed);
            if c.grounded && !jumped && vertical < 0.0 {
                // Pressed into the ground, so snapping and slopes hold.
                vertical = vertical.max(g.y * dt);
            }
        }
        let across = if c.gravity_scale > 0.0 {
            Vec3::new(c.movement.x, 0.0, c.movement.z)
        } else {
            c.movement
        };
        let desired = flat((across + Vec3::Y * vertical) * dt);
        let start = foot + ride;
        let mover = Mover {
            qp: sim.world.query_pipeline_with_filter(filter),
            shape: &*shape,
            rot: at.rotation,
            skin: c.skin.max(0.001),
            walkable: og_core::math::cos(c.max_slope.clamp(0.0, 89.0).to_radians()),
            step_height: c.step_height.max(0.0),
        };
        let moved = if c.gravity_scale > 0.0 {
            // Snapping holds a character that stood on the ground down
            // slopes and steps; one in the air (falling from a jump or off
            // a ledge) lands where it meets the ground, not pulled down to
            // ground within snapDistance in one tick (rapier's own
            // controller snaps only when grounded at the start, too).
            let snap = if jumped || vertical > 0.0 || !c.grounded {
                0.0
            } else {
                c.snap_distance
            };
            mover.travel(start, desired, snap, c.grounded && !jumped)
        } else {
            mover.travel_free(start, desired)
        };
        let end = start + moved.translation;
        let was_grounded = c.grounded;
        let grounded = moved.ground.is_some() && !jumped && vertical <= 0.0;
        if moved.translation.y < desired.y - 1e-4 && vertical > 0.0 {
            vertical = 0.0; // a ceiling
        }
        if grounded && vertical < 0.0 {
            vertical = 0.0;
        }
        let ground = if grounded {
            moved
                .ground
                .map(|(h, n)| (entity_of(&sim.world.colliders[h]), n))
        } else {
            None
        };
        let speed_in = c.velocity.y;
        c.grounded = grounded;
        c.sliding = moved.sliding;
        c.velocity = flat(Vec3::new(
            moved.translation.x / dt,
            vertical,
            moved.translation.z / dt,
        )) + if c.gravity_scale > 0.0 {
            Vec3::ZERO
        } else {
            Vec3::new(0.0, moved.translation.y / dt, 0.0)
        };
        c.ground_normal = ground.map_or(Vec3::ZERO, |g| g.1);
        c.ground = ground.map_or(0, |g| g.0);
        c.jump_was_held = c.jump;
        // The body's new pose: its collider's displacement applied to the body.
        let new_body = rp::Pose::from_parts(v(end), at.rotation) * local.inverse();
        let (position, quaternion) = unpose(&new_body, b.position);
        out.moved[k] = Some((position, quaternion, None));
        if c.push_bodies {
            push(
                &mut sim.world,
                &moved.hits,
                desired / dt,
                c.push_strength,
                dt,
            );
        }
        if jumped {
            out.events.push(CharacterEvent {
                entity: b.entity.to_bits(),
                kind: CHARACTER_JUMPED,
                speed: vertical,
            });
        } else if grounded && !was_grounded {
            out.events.push(CharacterEvent {
                entity: b.entity.to_bits(),
                kind: CHARACTER_LANDED,
                speed: speed_in,
            });
        } else if !grounded && was_grounded {
            out.events.push(CharacterEvent {
                entity: b.entity.to_bits(),
                kind: CHARACTER_LEFT_GROUND,
                speed: vertical,
            });
        }
        out.controls.push((b.entity, ControlOut::Character(c)));
    }
    Ok(())
}
