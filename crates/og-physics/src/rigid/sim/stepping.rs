use super::*;

/// Every body's linear and angular velocity before a substep, by handle
/// index (with the generation-bearing handle): the motions the solver meets
/// a new contact with. A body a controller moves is taken at the velocity
/// that carries it to its target, as rapier gives it inside the step.
pub(crate) type Motions = Vec<Option<(rp::RigidBodyHandle, Vec3, Vec3)>>;

fn motions_now(w: &rp::PhysicsWorld) -> Motions {
    let inv_dt = w.integration_parameters.inv_dt();
    let mut motions = Vec::new();
    for (h, rb) in w.bodies.iter() {
        let i = h.into_raw_parts().0 as usize;
        if i >= motions.len() {
            motions.resize(i + 1, None);
        }
        let vels = if rb.body_type() == rp::RigidBodyType::KinematicPositionBased {
            rp::RigidBodyPosition {
                position: *rb.position(),
                next_position: *rb.next_position(),
            }
            .interpolate_velocity(inv_dt, rb.local_center_of_mass())
        } else {
            *rb.vels()
        };
        motions[i] = Some((h, unv(vels.linvel), unangvel(vels.angvel)));
    }
    motions
}

/// The pairs touching after the substeps before the last, by their
/// colliders: the normal impulse they took so far and their relative
/// velocity as the first of those substeps began.
type Carried = BTreeMap<(u64, u64), (f32, Vec3)>;

/// What the step's readout needs to report a hit: the motions before the
/// last substep and what the earlier ones carried.
pub(super) struct Hits<'a> {
    pub(super) motions: &'a Motions,
    pub(super) carried: &'a Carried,
}

pub(super) fn pair_key(pair: &rp::ContactPair) -> (u64, u64) {
    (
        pair.collider1.into_raw_parts().0 as u64 | (pair.collider1.into_raw_parts().1 as u64) << 32,
        pair.collider2.into_raw_parts().0 as u64 | (pair.collider2.into_raw_parts().1 as u64) << 32,
    )
}

/// A touching pair's hit in the substep just stepped: the normal impulse
/// the solver gave it (N·s, summed over its contact points) and collider
/// 1's velocity relative to collider 2 at the deepest point, each body's
/// velocity taken from `motions` (before the substep) with its angular
/// part about its centre of mass now. A collider without a body is still.
pub(super) fn hit(
    pair: &rp::ContactPair,
    colliders: &rp::ColliderSet,
    bodies: &rp::RigidBodySet,
    motions: &Motions,
) -> (f32, Vec3) {
    let Some((_, contact)) = pair.find_deepest_contact() else {
        return (0.0, Vec3::ZERO);
    };
    let c1 = &colliders[pair.collider1];
    let point = unv(c1.position().transform_point(contact.local_p1));
    let at = |c: &rp::Collider| {
        let Some(h) = c.parent() else {
            return Vec3::ZERO;
        };
        match motions.get(h.into_raw_parts().0 as usize) {
            Some(Some((handle, linear, angular))) if *handle == h => {
                let com = unv(bodies[h].center_of_mass());
                *linear + angular.cross(point - com)
            }
            _ => Vec3::ZERO,
        }
    };
    let impulse = pair.total_impulse_magnitude().max(0.0);
    (impulse, at(c1) - at(&colliders[pair.collider2]))
}

/// After a substep but the last: each touching solid pair's impulse added
/// to what it carried, its first relative velocity kept; also records every
/// active solid contact into `seen` (keyed by canonical entity pair) so
/// transient impacts that separate before the final substep still fire
/// CollisionStarted + CollisionEnded.
fn carry(
    w: &rp::PhysicsWorld,
    motions: &Motions,
    carried: &mut Carried,
    seen: &mut BTreeMap<(u64, u64), CollisionStarted>,
) {
    let mut next = Carried::new();
    for pair in w.narrow_phase.contact_pairs() {
        if !pair.has_any_active_contact() {
            continue;
        }
        let (impulse, velocity) = hit(pair, &w.colliders, &w.bodies, motions);
        let key = pair_key(pair);
        let entry = match carried.get(&key) {
            Some(&(earlier, first)) => (earlier + impulse, first),
            None => (impulse, velocity),
        };
        next.insert(key, entry);

        let (c1, c2) = (&w.colliders[pair.collider1], &w.colliders[pair.collider2]);
        let (e1, e2) = (entity_of(c1).to_bits(), entity_of(c2).to_bits());
        if e1 == e2 {
            continue;
        }
        let (mut point, mut normal) = (Vec3::ZERO, Vec3::ZERO);
        if let Some((manifold, contact)) = pair.find_deepest_contact() {
            point = unv(c1.position().transform_point(contact.local_p1));
            normal = unv(manifold.data.normal);
        }
        let (a, b, normal, rel_vel) = if e1 < e2 {
            (e1, e2, normal, velocity)
        } else {
            (e2, e1, -normal, -velocity)
        };
        match seen.get_mut(&(a, b)) {
            Some(prev) => {
                if impulse > prev.impulse {
                    prev.point = point;
                    prev.normal = normal;
                }
                prev.impulse += impulse;
            }
            None => {
                seen.insert(
                    (a, b),
                    CollisionStarted {
                        a,
                        b,
                        point,
                        normal,
                        impulse,
                        relative_velocity: rel_vel,
                    },
                );
            }
        }
    }
    *carried = next;
}

fn body_kind(b: &Body) -> Result<rp::RigidBodyType> {
    let e = b.entity;
    let kind = match b.body.kind {
        _ if b.control.is_some() => rp::RigidBodyType::KinematicPositionBased,
        BODY_DYNAMIC => rp::RigidBodyType::Dynamic,
        BODY_STATIC => rp::RigidBodyType::Fixed,
        BODY_KINEMATIC => rp::RigidBodyType::KinematicVelocityBased,
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "entity {e}: og.RigidBody.kind {other} is not a kind (0 dynamic, 1 static, 2 kinematic)"
                ),
            ));
        }
    };
    for (what, x) in [
        ("mass", b.body.mass),
        ("linearDamping", b.body.linear_damping),
        ("angularDamping", b.body.angular_damping),
        ("airDrag", b.body.air_drag),
    ] {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("entity {e}: og.RigidBody.{what} must be finite and at least 0, got {x}"),
            ));
        }
    }
    if !b.body.gravity_scale.is_finite() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "entity {e}: og.RigidBody.gravityScale must be finite, got {}",
                b.body.gravity_scale
            ),
        ));
    }
    if !b.body.center_of_mass.is_finite() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "entity {e}: og.RigidBody.centerOfMass must be finite, got {}",
                b.body.center_of_mass
            ),
        ));
    }
    if !b.position.is_finite() || !b.quaternion.is_finite() || !b.scale.is_finite() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("entity {e}: og.Transform must be finite"),
        ));
    }
    if let Some(vel) = b.velocity
        && (!vel.linear.is_finite() || !vel.angular.is_finite())
    {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("entity {e}: og.Velocity must be finite"),
        ));
    }
    Ok(kind)
}

/// Adds `b` to the rapier world at its transform and velocity.
pub(super) fn insert_body(
    w: &mut rp::PhysicsWorld,
    b: &Body,
    shared: &mut Shared,
) -> Result<rp::RigidBodyHandle> {
    let (e, body) = (b.entity, &b.body);
    let kind = body_kind(b)?;
    // A compound body's mass is shared among its non-sensor colliders by
    // their density-weighted volume; sensor colliders never add mass.
    let masses = compound_masses(b, shared)?;
    let custom_com = body.center_of_mass != Vec3::ZERO;
    let mut total_mp = rp::MassProperties::default();
    let mut solid_count = 0usize;

    let own = match &b.collider {
        Some(c) => {
            let local = local_pose(c, b.scale);
            let share = if c.sensor {
                0.0
            } else {
                solid_count += 1;
                masses.as_ref().map_or(body.mass, |m| m[0])
            };
            let pass_mass = if custom_com && !c.sensor { -1.0 } else { share };
            let built = collider(c, b.scale, local, e, pass_mass, shared)?;
            if custom_com && !c.sensor {
                total_mp +=
                    collider_mass_props(&built.shape, c.density, share).transform_by(&local);
            }
            Some(built)
        }
        None => None,
    };
    let mut children = Vec::with_capacity(b.children.len());
    for (k, child) in b.children.iter().enumerate() {
        let at = pose(child.position, child.quaternion) * local_pose(&child.collider, child.scale);
        let share = if child.collider.sensor {
            0.0
        } else {
            solid_count += 1;
            masses
                .as_ref()
                .map_or(0.0, |m| m[k + usize::from(b.collider.is_some())])
        };
        let pass_mass = if custom_com && !child.collider.sensor {
            -1.0
        } else {
            share
        };
        let built = collider(
            &child.collider,
            child.scale,
            at,
            child.entity,
            pass_mass,
            shared,
        )?
        // Its body in the high bits (entity_of reads the low ones).
        .user_data(child.entity.to_bits() as u128 | (e.to_bits() as u128) << 64);
        if custom_com && !child.collider.sensor {
            total_mp +=
                collider_mass_props(&built.shape, child.collider.density, share).transform_by(&at);
        }
        children.push(built);
    }
    let mut builder = rp::RigidBodyBuilder::new(kind)
        .pose(pose(b.position, b.quaternion))
        .linear_damping(body.linear_damping)
        .angular_damping(body.angular_damping)
        .gravity_scale(body.gravity_scale)
        .locked_axes(rp::LockedAxes::from_bits_truncate((body.lock & 63) as u8))
        .ccd_enabled(body.ccd)
        .dominance_group(body.dominance.clamp(-127, 127) as i8)
        .enabled(body.enabled)
        .additional_solver_iterations(body.additional_solver_iterations.min(32) as usize)
        // Resting and sleep are driven by og-physics' deterministic sleep pass.
        .can_sleep(false)
        .user_data(e.to_bits() as u128);
    builder = gyroscopic_builder(builder, body.gyroscopic);
    if kind != rp::RigidBodyType::Fixed {
        let vel = unlocked(b.velocity.unwrap_or_default(), body.lock);
        builder = builder
            .linvel(v(flat(vel.linear)))
            .angvel(angvel(vel.angular));
    }
    if kind == rp::RigidBodyType::Dynamic {
        if solid_count == 0 {
            builder = builder.additional_mass(if body.mass > 0.0 { body.mass } else { 1.0 });
        } else if custom_com {
            if total_mp.mass() <= 0.0 {
                let m = if body.mass > 0.0 { body.mass } else { 1.0 };
                total_mp.set_mass(m, false);
            }
            total_mp.local_com += v(flat(body.center_of_mass * b.scale));
            builder = builder.additional_mass_properties(total_mp);
        }
    }
    let h = w.bodies.insert(builder);
    if let Some(c) = own {
        w.colliders.insert_with_parent(c, h, &mut w.bodies);
    }
    for c in children {
        w.colliders.insert_with_parent(c, h, &mut w.bodies);
    }
    Ok(h)
}

/// Each collider's share of a compound body's `mass` (its own first, then
/// its children's), in proportion to their solid masses at their densities;
/// sensor colliders always receive 0. None when the body is not a compound
/// or takes its mass from densities.
fn compound_masses(b: &Body, shared: &mut Shared) -> Result<Option<Vec<f32>>> {
    if b.children.is_empty() || b.body.mass <= 0.0 {
        return Ok(None);
    }
    let mut own = Vec::with_capacity(b.children.len() + 1);
    let mut is_solid = Vec::with_capacity(b.children.len() + 1);
    if let Some(c) = &b.collider {
        is_solid.push(!c.sensor);
        own.push(if c.sensor {
            0.0
        } else {
            shape(c, b.scale, shared)?.mass_properties(c.density).mass()
        });
    }
    for child in &b.children {
        let c = &child.collider;
        is_solid.push(!c.sensor);
        own.push(if c.sensor {
            0.0
        } else {
            shape(c, child.scale, shared)?
                .mass_properties(c.density)
                .mass()
        });
    }
    let total: f32 = own.iter().sum();
    let solids = is_solid.iter().filter(|&&s| s).count() as f32;
    Ok(Some(
        own.iter()
            .zip(is_solid)
            .map(|(&m, solid)| {
                if !solid {
                    0.0
                } else if total > 0.0 {
                    b.body.mass * m / total
                } else if solids > 0.0 {
                    b.body.mass / solids
                } else {
                    0.0
                }
            })
            .collect(),
    ))
}

/// The entity of the body a collider belongs to: a compound child's body
/// (the user data's high bits), else the collider's own entity.
pub(super) fn body_of(c: &rp::Collider) -> Entity {
    match (c.user_data >> 64) as u64 {
        0 => entity_of(c),
        body => Entity::from_bits(body),
    }
}

/// Brings the persistent rapier world in line with `scene`, steps it
/// `substeps` times and reads the dynamic bodies and the touching pairs
/// back. On an error `sim` is left half-updated; the caller drops it.
///
/// All of it runs where og-lanes puts it: from [`crate::pool_bodies`] for
/// the pool's thread count, on the lanes pool (natively and in the lanes
/// wasm), where rapier's `parallel` build splits its broad phase, narrow
/// phase and solver and the contact readout splits by pairs; below that,
/// and with lanes off, on the calling thread, where rapier finds one thread
/// and runs serially. Rapier computes the same bits on any number of
/// threads (`enhanced-determinism`, its solver's layout fixed for eight
/// workers whatever the pool), so the world is the same either way.
/// Resolves any collider mesh handles not yet in `shared.meshes` on the
/// calling thread before stepping on the lanes pool (a host's `mesh_source`
/// may read engine-thread state).
fn pre_resolve_meshes(scene: &Scene, shared: &mut Shared) {
    if shared.mesh_source.is_none() {
        return;
    }
    let need = |c: &Collider| matches!(c.shape, SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_POLYLINE);
    for b in &scene.bodies {
        if let Some(c) = &b.collider
            && need(c)
        {
            let _ = mesh(shared, c.mesh);
        }
        for child in &b.children {
            if need(&child.collider) {
                let _ = mesh(shared, child.collider.mesh);
            }
        }
    }
    for s in &scene.statics {
        if need(&s.collider) {
            let _ = mesh(shared, s.collider.mesh);
        }
    }
    for (_, _, _, sb) in &scene.softs {
        if sb.shape == SOFT_MESH && sb.mesh != 0 {
            let _ = mesh(shared, sb.mesh);
        }
    }
}

pub(crate) fn step(
    settings: &Physics,
    dt: f32,
    tick: u64,
    scene: &Scene,
    shared: &mut Shared,
    sim: &mut Sim,
) -> Result<StepOut> {
    pre_resolve_meshes(scene, shared);
    // The bodies this tick steps, or last tick's when ragdolls and vehicles
    // added their own.
    let bodies = scene.bodies.len().max(sim.world.bodies.len());
    // The world's forced grains go with the step onto the pool's thread.
    let forced = crate::lanes::forced();
    og_lanes::install_from(
        og_lanes::Site::Physics,
        bodies,
        crate::pool_bodies(og_lanes::threads()),
        |split| {
            crate::lanes::with_grains(forced, || {
                step_here(settings, dt, tick, scene, shared, sim, split)
            })
        },
    )
}

/// [`step`] on the thread `install` chose; `split` when that is the pool.
fn step_here(
    settings: &Physics,
    dt: f32,
    tick: u64,
    scene: &Scene,
    shared: &mut Shared,
    sim: &mut Sim,
    split: bool,
) -> Result<StepOut> {
    let substeps = settings.substeps.clamp(1, 16);
    let sub_dt = dt / substeps as f32;
    let mut out = StepOut::default();
    let mut laps = Laps::start();
    let asleep = sleep::asleep(sim);
    let handles = sync(settings, dt, scene, shared, sim)?;
    let pushes = std::mem::take(&mut shared.pushes);
    let mut pushed_handles: Vec<rp::RigidBodyHandle> = Vec::new();
    apply_impulses(sim, &handles, &pushes, &mut pushed_handles);
    // A vortex turns about the profile's up.
    shared.air.axis = if sim.flat() { Vec3::Z } else { Vec3::Y };
    laps.lap(Phase::Sync);
    ext::sync_joints(scene, sim, &handles)?;
    super::vehicles(scene, dt, sim, &handles, &mut out)?;
    laps.lap(Phase::Joints);
    super::ragdolls(scene, dt, sim, shared, &mut out)?;
    laps.lap(Phase::Ragdolls);
    let mut cuts = ext::water(
        scene,
        settings,
        dt,
        tick,
        &shared.waters,
        sim,
        &handles,
        &mut out,
    )?;
    laps.lap(Phase::Water);
    sync_hooks(scene, sim);
    let _ = sleep::before_step(settings, scene, sim, &handles);
    laps.lap(Phase::Readout);

    // Track every body that received an external force or impulse this tick so
    // the rest rule never glues a body against a gentle continuous force.
    let mut had_force = vec![false; sim.world.bodies.len().max(1)];
    let mark_forced = |flags: &mut Vec<bool>, hs: &[rp::RigidBodyHandle]| {
        for &h in hs {
            let i = h.into_raw_parts().0 as usize;
            if i >= flags.len() {
                flags.resize(i + 1, false);
            }
            flags[i] = true;
        }
    };
    mark_forced(&mut had_force, &pushed_handles);

    // Each substep applies continuous forces through Rapier's `add_force` /
    // `add_torque` (respecting LockedAxes and substep-independent integration),
    // steps the solver, clears those forces, and carries contact impulses and
    // transient hits forward.
    let mut carried = BTreeMap::new();
    let mut substep_seen: BTreeMap<(u64, u64), CollisionStarted> = BTreeMap::new();
    let mut motions = Vec::new();
    let mut forced_bodies: Vec<rp::RigidBodyHandle> = Vec::new();
    let solver_dt = {
        let p = &sim.world.integration_parameters;
        p.dt / p.num_solver_iterations.max(1) as f32
    };
    let controller_paths =
        pre_step_controllers(scene, settings, dt, shared, sim, &handles, &mut out)?;
    // Controller impulses are already applied; preserve gentle pushes through
    // the rest rule, just like the game's explicit impulses.
    for e in &out.controller_impulses {
        if let Some(&h) = handles.get(e) {
            mark_forced(&mut had_force, &[h]);
        }
    }
    laps.lap(Phase::Controllers);
    for k in 0..substeps {
        for &(h, start, end) in &controller_paths {
            let rb = &mut sim.world.bodies[h];
            if k == 0 {
                rb.set_position(start, false);
            }
            rb.set_next_kinematic_position(start.lerp(&end, (k + 1) as f32 / substeps as f32));
        }
        forced_bodies.clear();
        apply_forces(sim, &handles, &pushes, &mut forced_bodies);
        ext::air_drag(
            scene,
            &shared.air,
            sub_dt,
            sim,
            &handles,
            &mut forced_bodies,
        );
        ext::force_fields(scene, &shared.forces, sim, &handles, &mut forced_bodies);
        ext::buoyancy_substep(
            scene,
            settings,
            sub_dt,
            &shared.waters,
            sim,
            &handles,
            &mut out,
            &mut cuts,
            k + 1 == substeps,
            &mut forced_bodies,
        );
        mark_forced(&mut had_force, &forced_bodies);

        motions = motions_now(&sim.world);
        if scene.hooks.is_empty() {
            sim.world.step();
        } else {
            sim.world.step_with_events(&Hooks(&scene.hooks), &());
        }
        // Clear per-substep user forces/torques so they never leak across
        // substeps, ticks, or saved snapshots.
        for &h in &forced_bodies {
            if let Some(rb) = sim.world.bodies.get_mut(h) {
                rb.reset_forces(false);
                rb.reset_torques(false);
            }
        }
        ext::break_joints(sim, solver_dt, &mut out);
        if k + 1 < substeps {
            carry(&sim.world, &motions, &mut carried, &mut substep_seen);
        }
    }
    ext::record_water_wakes(sim, dt, tick, cuts);
    laps.lap(Phase::Rapier);
    if split {
        laps.split(Phase::Rapier);
    }

    let hits = Hits {
        motions: &motions,
        carried: &carried,
    };
    let (touching, moved) = after_step(settings, scene, sim, &handles, &had_force, &hits, split);
    if !substep_seen.is_empty() {
        for ((a, b), cs) in substep_seen {
            let probe = Contact { a, b, sensor: 0 };
            if touching.binary_search_by_key(&probe, |t| t.0).is_err() {
                out.transient_started.push(cs);
            }
        }
    }
    out.touching = touching;
    for (slot, rigid) in out.moved.iter_mut().zip(moved) {
        if slot.is_none() {
            *slot = rigid;
        }
    }
    for (b, movement) in scene.bodies.iter().zip(&out.moved) {
        if b.control.is_some()
            && let Some((p, q, _)) = movement
            && let Some(t) = sim.bodies.get_mut(&b.entity.to_bits())
        {
            t.position = *p;
            t.quaternion = *q;
        }
    }
    out.sleep = sleep::report(scene, &sim.world, &handles, &asleep);
    laps.lap(Phase::Readout);
    ext::soft(scene, settings, dt, tick, shared, sim, &mut out, &mut laps)?;
    // Deferred soft/fluid reaction impulses can wake a body this tick.
    out.sleep = sleep::report(scene, &sim.world, &handles, &asleep);
    ext::joint_values(sim, &mut out);
    for (e, j) in &out.joints {
        ext::joint_written(sim, *e, j);
    }
    laps.lap(Phase::Joints);
    out.laps = Some(laps);
    Ok(out)
}

/// Refresh queries without consuming Rapier's pending collision changes or
/// contact events. The solver still observes all of them in its next step.
pub(crate) fn refresh_controller_queries(w: &mut rp::PhysicsWorld) {
    w.bodies
        .propagate_modified_body_positions_to_colliders(&mut w.colliders);
    for (h, c) in w.colliders.iter() {
        if c.is_enabled() {
            w.broad_phase
                .set_aabb(&w.integration_parameters, h, c.compute_aabb());
        }
    }
}

/// Current supporting frame, for compound bodies and moved static colliders.
pub(crate) fn support_pose(
    sim: &Sim,
    handles: &Handles<'_>,
    body: u64,
    collider: u64,
) -> Option<rp::Pose> {
    if let Some(&h) = handles.get(&body) {
        return Some(*sim.world.bodies[h].position());
    }
    sim.statics
        .get(&collider)
        .map(|t| *sim.world.colliders[t.handle].position())
}

/// Publish one virtual character immediately, so the next character queries
/// its new pose. Virtual contacts, not the infinite-mass solver, own pushing.
pub(crate) fn write_controller_pose(sim: &mut Sim, handle: rp::RigidBodyHandle, at: rp::Pose) {
    let rb = &mut sim.world.bodies[handle];
    rb.set_position(at, false);
    rb.set_next_kinematic_position(at);
    let colliders = rb.colliders().to_vec();
    sim.world
        .bodies
        .propagate_modified_body_positions_to_colliders(&mut sim.world.colliders);
    for h in colliders {
        sim.world.colliders[h].set_solver_groups(rp::InteractionGroups::new(
            rp::Group::empty(),
            rp::Group::empty(),
            rp::InteractionTestMode::And,
        ));
        let c = &sim.world.colliders[h];
        if c.is_enabled() {
            sim.world
                .broad_phase
                .set_aabb(&sim.world.integration_parameters, h, c.compute_aabb());
        }
    }
}

type ControllerPath = (rp::RigidBodyHandle, rp::Pose, rp::Pose);

/// A game-authored pose edit detaches saved support; it is not platform carry.
/// New or rebuilt bodies have no tracked frame to inherit support from.
pub(crate) fn controller_teleported(sim: &Sim, b: &Body) -> bool {
    sim.bodies
        .get(&b.entity.to_bits())
        .is_none_or(|t| t.position != b.position || t.quaternion != b.quaternion)
}

/// The pre-step controller phase: sync has finished; every controller sees
/// the current broad phase and publishes its pose before the next one moves.
fn pre_step_controllers(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    shared: &Shared,
    sim: &mut Sim,
    handles: &Handles<'_>,
    out: &mut StepOut,
) -> Result<Vec<ControllerPath>> {
    refresh_controller_queries(&mut sim.world);
    let starts: Vec<_> = scene
        .bodies
        .iter()
        .filter(|b| b.control.is_some())
        .map(|b| {
            let h = *handles.get(&b.entity.to_bits()).unwrap();
            (h, *sim.world.bodies[h].position())
        })
        .collect();
    out.moved.resize(scene.bodies.len(), None);
    ext::characters(scene, settings, dt, shared, sim, handles, out)?;
    super::platformers(scene, settings, dt, sim, handles, out)?;
    Ok(starts
        .into_iter()
        .map(|(h, start)| (h, start, *sim.world.bodies[h].position()))
        .collect())
}

/// The hooks every collider of a hooked entity runs (og.ContactHook).
fn all_hooks() -> rp::ActiveHooks {
    rp::ActiveHooks::FILTER_CONTACT_PAIRS
        | rp::ActiveHooks::FILTER_INTERSECTION_PAIR
        | rp::ActiveHooks::MODIFY_SOLVER_CONTACTS
}

/// Turns the contact hooks on for the colliders of the entities (or their
/// compound parent bodies) that have an og.ContactHook and off for the others.
/// When a hook is added, removed or edited, wakes affected bodies and their
/// touching partners so sleeping bodies respond immediately (e.g. a conveyor
/// turning on or an ignore list changing).
fn sync_hooks(scene: &Scene, sim: &mut Sim) {
    if scene.hooks.is_empty() && !sim.ext.hooked {
        return;
    }
    let mut changed_entities: BTreeSet<u64> = BTreeSet::new();
    for (&e, h) in scene.hooks.iter() {
        if sim.ext.last_hooks.get(&e) != Some(h) {
            changed_entities.insert(e);
        }
    }
    for &e in sim.ext.last_hooks.keys() {
        if !scene.hooks.contains_key(&e) {
            changed_entities.insert(e);
        }
    }
    let mut change = Vec::new();
    let mut wake_colliders = Vec::new();
    for (h, c) in sim.world.colliders.iter() {
        let ce = entity_of(c).to_bits();
        let cb = body_of(c).to_bits();
        let want = if scene.hooks.get(&ce).is_some() || scene.hooks.get(&cb).is_some() {
            all_hooks()
        } else {
            rp::ActiveHooks::empty()
        };
        if c.active_hooks() != want {
            change.push((h, want));
        }
        if !changed_entities.is_empty()
            && (changed_entities.contains(&ce) || changed_entities.contains(&cb))
        {
            wake_colliders.push(h);
        }
    }
    for (h, want) in change {
        if let Some(c) = sim.world.colliders.get_mut(h) {
            c.set_active_hooks(want);
        }
    }
    if !wake_colliders.is_empty() {
        let mut wake_bodies = Vec::new();
        for ch in wake_colliders {
            if let Some(p) = sim.world.colliders.get(ch).and_then(|c| c.parent()) {
                wake_bodies.push(p);
            }
            for pair in sim.world.narrow_phase.contact_pairs_with(ch) {
                for other in [pair.collider1, pair.collider2] {
                    if let Some(p) = sim.world.colliders.get(other).and_then(|c| c.parent()) {
                        wake_bodies.push(p);
                    }
                }
            }
        }
        for bh in wake_bodies {
            sim.world.wake_up(bh, true);
        }
    }
    sim.ext.hooked = !scene.hooks.is_empty();
    if scene.hooks.is_empty() {
        sim.ext.last_hooks.clear();
    } else {
        sim.ext.last_hooks = scene.hooks.clone();
    }
}

/// The contact hooks the solver reads, frozen for the step: read only, so
/// the same on any number of threads.
pub(super) struct Hooks<'a>(pub(super) &'a IdMap<u64, ContactHook>);

/// Cosine of the 45° one-way acceptance cone.
const ONE_WAY_COS: f32 = std::f32::consts::FRAC_1_SQRT_2;

impl Hooks<'_> {
    fn of(&self, c: &rp::Collider) -> Option<&ContactHook> {
        self.0
            .get(&entity_of(c).to_bits())
            .or_else(|| self.0.get(&body_of(c).to_bits()))
    }

    /// Whether either collider's hook ignores the other (its entity or its body).
    fn ignored(&self, c1: &rp::Collider, c2: &rp::Collider) -> bool {
        let ignores = |h: &ContactHook, other: &rp::Collider| {
            let (e, b) = (entity_of(other).to_bits(), body_of(other).to_bits());
            h.ignore
                .as_slice()
                .iter()
                .any(|&x| x != 0 && (x == e || x == b))
        };
        self.of(c1).is_some_and(|h| ignores(h, c2)) || self.of(c2).is_some_and(|h| ignores(h, c1))
    }
}

/// Independent 2-bit one-way state for one side of a contact manifold
/// (`shift` = 0 for collider1, 2 for collider2): 0 unknown, 1 allowed, 2 forbidden.
fn check_one_way(
    local_n1: rp::Vector,
    allowed_local_n1: rp::Vector,
    user_data: &mut u32,
    shift: u32,
) -> bool {
    const UNKNOWN: u32 = 0;
    const ALLOWED: u32 = 1;
    const FORBIDDEN: u32 = 2;
    let state = (*user_data >> shift) & 0b11;
    let ok = local_n1.dot(allowed_local_n1) >= ONE_WAY_COS;
    match state {
        UNKNOWN => {
            if ok {
                *user_data = (*user_data & !(0b11 << shift)) | (ALLOWED << shift);
                true
            } else {
                if local_n1.length_squared() > 0.1 {
                    *user_data = (*user_data & !(0b11 << shift)) | (FORBIDDEN << shift);
                }
                false
            }
        }
        FORBIDDEN => false,
        _ => true,
    }
}

impl rp::PhysicsHooks for Hooks<'_> {
    fn filter_contact_pair(&self, ctx: &rp::PairFilterContext) -> Option<rp::SolverFlags> {
        let (c1, c2) = (&ctx.colliders[ctx.collider1], &ctx.colliders[ctx.collider2]);
        (!self.ignored(c1, c2)).then_some(rp::SolverFlags::COMPUTE_RIGID_IMPULSES)
    }

    fn filter_intersection_pair(&self, ctx: &rp::PairFilterContext) -> bool {
        let (c1, c2) = (&ctx.colliders[ctx.collider1], &ctx.colliders[ctx.collider2]);
        !self.ignored(c1, c2)
    }

    fn modify_solver_contacts(&self, ctx: &mut rp::ContactModificationContext) {
        let (c1, c2) = (&ctx.colliders[ctx.collider1], &ctx.colliders[ctx.collider2]);
        let (h1, h2) = (self.of(c1).copied(), self.of(c2).copied());
        if h1.is_none() && h2.is_none() {
            return;
        }
        let to_local1 = c1.position().rotation.inverse();
        let Some(m) = ctx.rigid_mut() else {
            return;
        };
        // Evaluate each side's one-way constraint independently using disjoint
        // 2-bit slots in `user_data` so two one-way hooks never clobber each other.
        let mut forbidden = false;
        if let Some(h) = h1 {
            let ow = flat(h.one_way);
            if ow != Vec3::ZERO && ow.is_finite() {
                let allowed = to_local1 * v(ow.normalize());
                if !check_one_way(m.manifold.local_n1, allowed, m.user_data, 0) {
                    forbidden = true;
                }
            }
        }
        if let Some(h) = h2 {
            let ow = flat(h.one_way);
            if ow != Vec3::ZERO && ow.is_finite() {
                let allowed = to_local1 * v(-ow.normalize());
                if !check_one_way(m.manifold.local_n1, allowed, m.user_data, 2) {
                    forbidden = true;
                }
            }
        }
        if forbidden {
            m.solver_contacts.clear();
            return;
        }
        // Symmetric, handle-order-independent merging of friction and restitution.
        let f1 = h1
            .map(|h| h.friction)
            .filter(|&x| x >= 0.0 && x.is_finite());
        let f2 = h2
            .map(|h| h.friction)
            .filter(|&x| x >= 0.0 && x.is_finite());
        match (f1, f2) {
            (Some(a), Some(b)) => *m.friction = 0.5 * (a + b),
            (Some(a), None) | (None, Some(a)) => *m.friction = a,
            (None, None) => {}
        }
        let r1 = h1
            .map(|h| h.restitution)
            .filter(|&x| x >= 0.0 && x.is_finite());
        let r2 = h2
            .map(|h| h.restitution)
            .filter(|&x| x >= 0.0 && x.is_finite());
        match (r1, r2) {
            (Some(a), Some(b)) => *m.restitution = 0.5 * (a + b),
            (Some(a), None) | (None, Some(a)) => *m.restitution = a,
            (None, None) => {}
        }
        // Additive conveyor velocity: c1 pushes c2 by +s1, c2 pushes c1 by +s2,
        // so the net tangent velocity of c2 relative to c1 is s1 - s2.
        let s1 = h1
            .map(|h| flat(h.surface_velocity))
            .filter(|s| s.is_finite())
            .unwrap_or(Vec3::ZERO);
        let s2 = h2
            .map(|h| flat(h.surface_velocity))
            .filter(|s| s.is_finite())
            .unwrap_or(Vec3::ZERO);
        let net_surface = s1 - s2;
        if net_surface != Vec3::ZERO {
            for c in m.solver_contacts.iter_mut() {
                c.tangent_velocity = v(net_surface);
            }
        }
    }
}
