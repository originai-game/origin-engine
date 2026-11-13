use super::*;

/// Every body's pose at the start of the tick, by handle index (with the
/// handle, so a slot reused by a rebuilt body is not mistaken for it): how
/// far a platform moved this tick.
#[derive(Default)]
pub(crate) struct BodyPoses(Vec<Option<(rp::RigidBodyHandle, rp::Pose)>>);

impl BodyPoses {
    fn record(w: &rp::PhysicsWorld) -> BodyPoses {
        let mut poses = Vec::new();
        for (h, rb) in w.bodies.iter() {
            let i = h.into_raw_parts().0 as usize;
            if i >= poses.len() {
                poses.resize(i + 1, None);
            }
            poses[i] = Some((h, *rb.position()));
        }
        BodyPoses(poses)
    }

    pub(crate) fn get(&self, h: rp::RigidBodyHandle) -> Option<&rp::Pose> {
        match self.0.get(h.into_raw_parts().0 as usize)? {
            Some((handle, pose)) if *handle == h => Some(pose),
            _ => None,
        }
    }
}

/// Every body's linear and angular velocity before a substep, by handle
/// index (with the handle, as [`BodyPoses`]): the motions the solver meets
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
/// to what it carried, its first relative velocity kept; a pair no longer
/// touching drops out (a contact that starts again starts afresh).
fn carry(w: &rp::PhysicsWorld, motions: &Motions, carried: &mut Carried) {
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
    // A compound body's mass is shared among its colliders by their own
    // (density times volume); alone, its collider takes it whole.
    let masses = compound_masses(b, shared)?;
    let own = match &b.collider {
        Some(c) => {
            let local = local_pose(c, b.scale);
            let mass = masses.as_ref().map_or(body.mass, |m| m[0]);
            let built = collider(c, b.scale, local, e, mass, shared)?;
            Some(centred(built, c, body, b.scale, local)?)
        }
        None => None,
    };
    let mut children = Vec::with_capacity(b.children.len());
    for (k, child) in b.children.iter().enumerate() {
        let at = pose(child.position, child.quaternion) * local_pose(&child.collider, child.scale);
        let mass = masses
            .as_ref()
            .map_or(0.0, |m| m[k + usize::from(b.collider.is_some())]);
        let built = collider(&child.collider, child.scale, at, child.entity, mass, shared)?
            // Its body in the high bits (entity_of reads the low ones).
            .user_data(child.entity.to_bits() as u128 | (e.to_bits() as u128) << 64);
        children.push(built);
    }
    let mut builder = rp::RigidBodyBuilder::new(kind)
        .pose(pose(b.position, b.quaternion))
        .linear_damping(body.linear_damping)
        .angular_damping(body.angular_damping)
        .gravity_scale(body.gravity_scale)
        .locked_axes(rp::LockedAxes::from_bits_truncate((body.lock & 63) as u8))
        .ccd_enabled(body.ccd)
        // Resting is decided by the rest rule below, not by rapier's sleep.
        .can_sleep(false)
        .user_data(e.to_bits() as u128);
    if kind != rp::RigidBodyType::Fixed {
        let vel = unlocked(b.velocity.unwrap_or_default(), body.lock);
        builder = builder
            .linvel(v(flat(vel.linear)))
            .angvel(angvel(vel.angular));
    }
    if own.is_none() && children.is_empty() && kind == rp::RigidBodyType::Dynamic {
        builder = builder.additional_mass(if body.mass > 0.0 { body.mass } else { 1.0 });
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
/// its children's), in proportion to their masses at their densities;
/// None when the body is not a compound or takes its mass from them.
fn compound_masses(b: &Body, shared: &mut Shared) -> Result<Option<Vec<f32>>> {
    if b.children.is_empty() || b.body.mass <= 0.0 {
        return Ok(None);
    }
    let mut own = Vec::with_capacity(b.children.len() + 1);
    if let Some(c) = &b.collider {
        own.push(shape(c, b.scale, shared)?.mass_properties(c.density).mass());
    }
    for child in &b.children {
        let c = &child.collider;
        own.push(
            shape(c, child.scale, shared)?
                .mass_properties(c.density)
                .mass(),
        );
    }
    let total: f32 = own.iter().sum();
    let n = own.len() as f32;
    Ok(Some(
        own.iter()
            .map(|&m| {
                if total > 0.0 {
                    b.body.mass * m / total
                } else {
                    b.body.mass / n
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
pub(crate) fn step(
    settings: &Physics,
    dt: f32,
    tick: u64,
    scene: &Scene,
    shared: &mut Shared,
    sim: &mut Sim,
) -> Result<StepOut> {
    // The bodies this tick steps, or last tick's when ragdolls and vehicles
    // added their own.
    let bodies = scene.bodies.len().max(sim.world.bodies.len());
    og_lanes::install_from(
        og_lanes::Site::Physics,
        bodies,
        crate::pool_bodies(og_lanes::threads()),
        |split| step_here(settings, dt, tick, scene, shared, sim, split),
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
    let mut out = StepOut::default();
    let mut laps = Laps::start();
    // Poses before this tick moved anything (game code teleporting
    // kinematic platforms included): how far each body moved this tick.
    let poses = BodyPoses::record(&sim.world);
    let asleep = sleep::asleep(sim);
    let handles = sync(settings, dt, scene, shared, sim)?;
    let pushes = std::mem::take(&mut shared.pushes);
    apply_pushes(sim, &handles, &pushes, dt);
    // A vortex turns about the profile's up.
    shared.air.axis = if sim.flat() { Vec3::Z } else { Vec3::Y };
    ext::air_drag(scene, &shared.air, dt, sim, &handles);
    laps.lap(Phase::Sync);
    ext::sync_joints(scene, sim, &handles)?;
    super::vehicles(scene, dt, sim, &handles, &mut out)?;
    laps.lap(Phase::Joints);
    super::ragdolls(scene, dt, sim, shared, &mut out)?;
    laps.lap(Phase::Ragdolls);
    let cuts = ext::water(
        scene,
        settings,
        dt,
        tick,
        &shared.waters,
        sim,
        &handles,
        &mut out,
    )?;
    ext::buoyancy(
        scene,
        settings,
        dt,
        tick,
        &shared.waters,
        sim,
        &handles,
        &mut out,
        cuts,
    );
    laps.lap(Phase::Water);
    let sleeping = sleep::before_step(settings, scene, sim, &handles);
    let pre = rest_poses(scene, sim, &handles);
    laps.lap(Phase::Readout);

    // Each substep starts from the bodies' motions recorded before it: what
    // the solver met a contact with. The pairs touching after a substep
    // but the last carry their impulse and first relative velocity on.
    let mut carried = BTreeMap::new();
    let mut motions = Vec::new();
    sync_hooks(scene, sim);
    let solver_dt = {
        let p = &sim.world.integration_parameters;
        p.dt / p.num_solver_iterations.max(1) as f32
    };
    for k in 0..substeps {
        motions = motions_now(&sim.world);
        if scene.hooks.is_empty() {
            sim.world.step();
        } else {
            sim.world.step_with_events(&Hooks(&scene.hooks), &());
        }
        ext::break_joints(sim, solver_dt, &mut out);
        if k + 1 < substeps {
            carry(&sim.world, &motions, &mut carried);
        }
    }
    laps.lap(Phase::Rapier);
    if split {
        laps.split(Phase::Rapier);
    }

    let hits = Hits {
        motions: &motions,
        carried: &carried,
    };
    let (touching, moved) =
        after_step(settings, scene, sim, &handles, pre, &sleeping, &hits, split);
    out.touching = touching;
    out.moved = moved;
    out.sleep = sleep::report(scene, &sim.world, &handles, &asleep);
    laps.lap(Phase::Readout);
    ext::characters(scene, settings, dt, &shared.waters, sim, &poses, &mut out)?;
    super::platformers(scene, settings, dt, sim, &poses, &mut out)?;
    laps.lap(Phase::Controllers);
    ext::soft(scene, settings, dt, tick, shared, sim, &mut out, &mut laps)?;
    ext::joint_values(sim, &mut out);
    for (e, j) in &out.joints {
        ext::joint_written(sim, *e, j);
    }
    laps.lap(Phase::Joints);
    out.laps = Some(laps);
    Ok(out)
}

/// The hooks every collider of a hooked entity runs (og.ContactHook).
fn all_hooks() -> rp::ActiveHooks {
    rp::ActiveHooks::FILTER_CONTACT_PAIRS
        | rp::ActiveHooks::FILTER_INTERSECTION_PAIR
        | rp::ActiveHooks::MODIFY_SOLVER_CONTACTS
}

/// Turns the contact hooks on for the colliders of the entities that have
/// an og.ContactHook and off for the others, touching only those that
/// change; nothing while no collider has them.
fn sync_hooks(scene: &Scene, sim: &mut Sim) {
    if scene.hooks.is_empty() && !sim.ext.hooked {
        return;
    }
    let mut change = Vec::new();
    for (h, c) in sim.world.colliders.iter() {
        let want = if scene.hooks.get(&entity_of(c).to_bits()).is_some() {
            all_hooks()
        } else {
            rp::ActiveHooks::empty()
        };
        if c.active_hooks() != want {
            change.push((h, want));
        }
    }
    for (h, want) in change {
        if let Some(c) = sim.world.colliders.get_mut(h) {
            c.set_active_hooks(want);
        }
    }
    sim.ext.hooked = !scene.hooks.is_empty();
}

/// The contact hooks the solver reads, frozen for the step: read only, so
/// the same on any number of threads.
struct Hooks<'a>(&'a IdMap<u64, ContactHook>);

/// How far from its direction a one-way surface still holds (45°).
const ONE_WAY_ANGLE: f32 = std::f32::consts::FRAC_PI_4;

impl Hooks<'_> {
    fn of(&self, c: &rp::Collider) -> Option<&ContactHook> {
        self.0.get(&entity_of(c).to_bits())
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
        let to_local1 = c1.position().rotation.inverse();
        // The normal runs from collider 1 to collider 2: a hook on collider
        // 2 sees its surface from the other side.
        for (h, side) in [(h1, 1.0), (h2, -1.0)] {
            let Some(h) = h else {
                continue;
            };
            let one_way = flat(h.one_way);
            if one_way != Vec3::ZERO {
                let allowed = to_local1 * v(one_way.normalize() * side);
                ctx.update_as_oneway_platform(allowed, ONE_WAY_ANGLE);
            }
            let Some(m) = ctx.rigid_mut() else {
                continue;
            };
            if h.friction >= 0.0 {
                *m.friction = h.friction;
            }
            if h.restitution >= 0.0 {
                *m.restitution = h.restitution;
            }
            let surface = flat(h.surface_velocity);
            if surface != Vec3::ZERO {
                for c in m.solver_contacts.iter_mut() {
                    c.tangent_velocity = v(surface * side);
                }
            }
        }
    }
}
