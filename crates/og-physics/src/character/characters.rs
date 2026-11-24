//! Virtual characters run before rigid integration and publish sequentially.

use super::super::mover::{collider_body, collider_entity};
use super::super::sim::{Handles, controller_teleported, support_pose, write_controller_pose};
use super::*;

pub(crate) fn characters(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    shared: &crate::Shared,
    sim: &mut Sim,
    handles: &Handles<'_>,
    out: &mut StepOut,
) -> Result<()> {
    for (k, b) in scene.bodies.iter().enumerate() {
        let Some(crate::gather::Control::Character(mut c)) = b.control else {
            continue;
        };
        let handle = *handles.get(&b.entity.to_bits()).unwrap();
        if controller_teleported(sim, b) {
            c.grounded = false;
            c.ground = 0;
            c.ground_body = 0;
            c.coyote = 0.0;
            c.inherited_velocity = Vec3::ZERO;
        }
        let rb = &sim.world.bodies[handle];
        let Some(&ch) = rb.colliders().first() else {
            continue;
        };
        let collider = &sim.world.colliders[ch];
        let shape = collider.shared_shape().clone();
        let local = *collider
            .position_wrt_parent()
            .unwrap_or(&rp::Pose::IDENTITY);
        let mut at = *rb.position() * local;
        let filter = rp::QueryFilter::new()
            .exclude_rigid_body(handle)
            .exclude_sensors()
            .groups(collider.collision_groups());
        let up = flat(c.up).normalize_or_zero();
        let walking = c.gravity_scale > 0.0 && up != Vec3::ZERO;
        let centre = unv(at.translation);
        let was_grounded = c.grounded;
        let (mut ride, yaw) = if c.grounded {
            support_pose(sim, handles, c.ground_body, c.ground).map_or(
                (Vec3::ZERO, Quat::IDENTITY),
                |after| {
                    carried(
                        pose(c.ground_position, c.ground_quaternion),
                        after,
                        c.ground_point,
                        up,
                    )
                },
            )
        } else {
            (Vec3::ZERO, Quat::IDENTITY)
        };
        // The conveyor belongs to the supporting collider, or its compound body.
        if c.grounded {
            let surface = scene
                .hooks
                .get(&c.ground)
                .or_else(|| scene.hooks.get(&c.ground_body))
                .map_or(Vec3::ZERO, |h| flat(h.surface_velocity));
            ride += surface * dt;
        }
        let (_, rotation) = unpose(&at, b.position);
        at.rotation = pose(Vec3::ZERO, yaw * rotation).rotation;
        c.movement = yaw * c.movement;
        let platform_velocity = ride / dt;
        let added = flat(c.add_velocity);
        c.add_velocity = Vec3::ZERO;
        if added.dot(up) > 0.0 {
            c.grounded = false;
            c.coyote = 0.0;
        }
        let mut vertical = if walking && !was_grounded {
            c.inherited_velocity.dot(up)
        } else {
            0.0
        };
        if walking {
            c.inherited_velocity -= up * c.inherited_velocity.dot(up);
            c.inherited_velocity += added - up * added.dot(up);
            vertical += added.dot(up);
        } else {
            c.inherited_velocity += added;
        }
        c.coyote = if c.grounded {
            c.coyote_time
        } else {
            (c.coyote - dt).max(0.0)
        };
        let pressed = c.jump && !c.jump_was_held;
        c.buffer = if pressed {
            c.jump_buffer
        } else {
            (c.buffer - dt).max(0.0)
        };
        let jumped = walking && (pressed || c.buffer > 0.0) && (c.grounded || c.coyote > 0.0);
        if jumped {
            vertical = c.jump_speed;
            c.buffer = 0.0;
            c.coyote = 0.0;
        }
        let inherited = was_grounded && (jumped || !c.grounded);
        if inherited {
            c.inherited_velocity += platform_velocity * c.platform_inherit.clamp(0.0, 1.0);
        }
        let swim = swims(scene, &shared.waters, sim, handle, b.entity.to_bits());
        let pace = swim.map_or(1.0, |s| 1.0 - (1.0 - c.water_speed.clamp(0.0, 1.0)) * s.1);
        let g = flat(settings.gravity) * c.gravity_scale;
        if walking {
            vertical += (g + shared.forces.at(centre)).dot(up) * dt;
            if let Some((b, under, water)) = swim.filter(|s| s.1 > 0.0) {
                vertical +=
                    b.water_density * under * (-g + water.acceleration).dot(up).max(0.0) * dt;
                vertical -= (vertical - water.velocity.dot(up)) * (b.drag * dt).min(1.0) * under;
                vertical = stroke(
                    vertical,
                    c.movement.dot(up) * pace,
                    &b,
                    under,
                    water.velocity.dot(up),
                    dt,
                );
            }
            if c.max_fall_speed > 0.0 {
                vertical = vertical.max(-c.max_fall_speed);
            }
            if c.grounded && !jumped && vertical < 0.0 {
                vertical = vertical.max(g.dot(up) * dt);
            }
        }
        let input = if walking {
            c.movement - up * c.movement.dot(up)
        } else {
            c.movement
        };
        let flow = swim.map_or(Vec3::ZERO, |(_, under, water)| {
            let velocity = water.velocity - up * water.velocity.dot(up);
            velocity * under.clamp(0.0, 1.0)
        });
        let desired_velocity = flat(input * pace + up * vertical + c.inherited_velocity + flow);
        let mover = Mover {
            qp: sim.world.query_pipeline_with_filter(filter),
            shape: &*shape,
            rot: at.rotation,
            skin: c.skin.max(0.0),
            walkable: og_core::math::cos(c.max_slope.clamp(0.0, 89.0).to_radians()),
            step_height: c.step_height.max(0.0),
            up,
            scene,
            entity: b.entity.to_bits(),
            one_way_mask: 0,
            dropping_through: 0,
        };
        let moved = if walking {
            let snap = if jumped || vertical > 0.0 || !c.grounded {
                0.0
            } else {
                c.snap_distance
            };
            mover.travel(
                centre,
                ride,
                desired_velocity * dt,
                snap,
                c.grounded && !jumped,
            )
        } else {
            mover.travel_free(centre, ride, desired_velocity * dt)
        };
        let grounded = moved.ground.is_some() && !jumped && vertical <= 0.0;
        let impact = desired_velocity.dot(up);
        c.grounded = grounded;
        c.sliding = moved.sliding;
        c.velocity = moved.translation / dt;
        if grounded {
            c.inherited_velocity = Vec3::ZERO;
        }
        if was_grounded && !grounded && !inherited {
            c.inherited_velocity += platform_velocity * c.platform_inherit.clamp(0.0, 1.0);
        }
        if walking && !grounded {
            // Integrate inertia independently of carry and world-position rounding.
            c.inherited_velocity += up * vertical;
            if moved.bumped {
                c.inherited_velocity -= up * c.inherited_velocity.dot(up).max(0.0);
            }
        }
        c.ground = 0;
        c.ground_body = 0;
        c.ground_normal = Vec3::ZERO;
        if let Some(hit) = moved.ground.filter(|_| grounded) {
            let collider = &sim.world.colliders[hit.handle];
            c.ground = collider_entity(collider);
            c.ground_body = collider_body(collider);
            c.ground_point = hit.point;
            c.ground_normal = hit.normal;
            if let Some(frame) = support_pose(sim, handles, c.ground_body, c.ground) {
                (c.ground_position, c.ground_quaternion) = unpose(&frame, Vec3::ZERO);
            }
        }
        c.jump_was_held = c.jump;
        let new_body =
            rp::Pose::from_parts(v(centre + moved.translation), at.rotation) * local.inverse();
        let (position, quaternion) = unpose(&new_body, b.position);
        out.moved[k] = Some((position, quaternion, None));
        write_controller_pose(sim, handle, new_body);
        if c.push_bodies {
            push(
                &mut sim.world,
                &moved.hits,
                moved.ground.filter(|_| grounded),
                desired_velocity + platform_velocity,
                c.mass,
                c.push_strength,
                g,
                dt,
                &mut out.controller_impulses,
            );
        }
        let transition = if jumped {
            Some(CHARACTER_JUMPED)
        } else if grounded && !was_grounded {
            Some(CHARACTER_LANDED)
        } else if !grounded && was_grounded {
            Some(CHARACTER_LEFT_GROUND)
        } else {
            None
        };
        for kind in transition
            .into_iter()
            .chain(moved.bumped.then_some(CHARACTER_BUMPED))
            .chain(moved.crushed.then_some(CHARACTER_CRUSHED))
        {
            out.events.push(CharacterEvent {
                entity: b.entity.to_bits(),
                kind,
                speed: impact,
            });
        }
        out.controls.push((b.entity, ControlOut::Character(c)));
        let limp = sim
            .ragdolls
            .get(&b.entity.to_bits())
            .is_some_and(|r| r.dynamic);
        if let Some((float, submerged, _)) = swim.filter(|_| !limp) {
            out.floats.push((b.entity, Buoyancy { submerged, ..float }));
        }
    }
    Ok(())
}

/// Stroke approaches a water-relative target over time. Entering water at
/// speed retains momentum; a held upward input is never an instant brake.
fn stroke(vertical: f32, wanted: f32, b: &Buoyancy, under: f32, water: f32, dt: f32) -> f32 {
    let floats = (1.0 / b.water_density.max(1e-3)).min(0.9);
    let reach = if wanted > 0.0 {
        ((under - floats) / (1.0 - floats)).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let target = water + wanted * reach;
    if (wanted > 0.0 && target > vertical) || (wanted < 0.0 && target < vertical) {
        vertical + (target - vertical) * (b.drag.max(0.0) * under * dt).min(1.0)
    } else {
        vertical
    }
}
