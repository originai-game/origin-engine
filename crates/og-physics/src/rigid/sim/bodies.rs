use super::*;

/// Each body's rapier handle, in scene order, and by entity for the
/// passes that look bodies up (joints, vehicles, water), built on the
/// first lookup.
pub(crate) struct Handles<'s> {
    scene: &'s Scene,
    pub(crate) by_body: Vec<rp::RigidBodyHandle>,
    by_entity: std::sync::OnceLock<IdMap<u64, rp::RigidBodyHandle>>,
}

impl Handles<'_> {
    /// The handle of the body of entity `key`.
    pub(crate) fn get(&self, key: &u64) -> Option<&rp::RigidBodyHandle> {
        self.by_entity
            .get_or_init(|| {
                self.scene
                    .bodies
                    .iter()
                    .map(|b| b.entity.to_bits())
                    .zip(self.by_body.iter().copied())
                    .collect()
            })
            .get(key)
    }
}

/// `f` on the ranges of the scene's bodies, by [`crate::Grains::bodies`] a
/// task on the lanes from two of them, the ranges' results joined in body
/// order.
pub(super) fn by_bodies<R: Send>(
    n: usize,
    f: impl Fn(std::ops::Range<usize>) -> Vec<R> + Sync + Send,
) -> Vec<R> {
    let mut parts = crate::lanes::map_ranges(n, crate::lanes::now().bodies, f);
    if parts.len() == 1 {
        parts.pop().unwrap_or_default()
    } else {
        parts.into_iter().flatten().collect()
    }
}

pub(super) fn simulated(b: &Body) -> bool {
    b.body.kind == BODY_DYNAMIC && b.control.is_none()
}

/// A kinematic body with a nonzero unlocked og.Velocity in this profile:
/// the step moves it by it and writes its Transform back.
pub(super) fn moves_itself(b: &Body) -> bool {
    if b.body.kind != BODY_KINEMATIC || b.control.is_some() {
        return false;
    }
    let Some(v) = b.velocity else {
        return false;
    };
    let u = unlocked(v, b.body.lock);
    let lin = flat(u.linear);
    let ang = angvel(u.angular);
    lin != Vec3::ZERO || ang != angvel(Vec3::ZERO)
}

/// The dynamic bodies' poses before the step, for the rest rule, in body
/// order (by ranges of bodies on the lanes).
pub(super) fn rest_poses(scene: &Scene, sim: &Sim, handles: &Handles<'_>) -> Vec<rp::Pose> {
    let bodies = &sim.world.bodies;
    by_bodies(scene.bodies.len(), |r| {
        scene.bodies[r.clone()]
            .iter()
            .zip(&handles.by_body[r])
            .filter(|(b, _)| simulated(b))
            .map(|(_, &h)| *bodies[h].position())
            .collect()
    })
}

/// What [`sync`] does to a body, decided from the body and what physics
/// last left it at (read only, by ranges of bodies on the lanes), then
/// done to the rapier world in body order.
enum Plan {
    /// Still the body topology it was: patch non-structural body/collider
    /// parameters in place (preserving warm starts and attached joints),
    /// move where game code or a controller put it, and update velocity.
    Keep {
        handle: rp::RigidBodyHandle,
        patch_body: bool,
        patch_own_collider: bool,
        patch_children: bool,
        next: Option<rp::Pose>,
        position: Option<rp::Pose>,
        velocity: Option<(rp::Vector, rp::AngVector)>,
    },
    /// New, or its kind, mass/COM, collider shape/topology or scale changed:
    /// built afresh.
    Build,
}

/// Whether two colliders have the same structural topology (shape, size,
/// offset, density, sensor flag); friction/restitution/layers/mask can be
/// patched in place without rebuilding the collider or body.
fn same_collider_topology(a: &Collider, b: &Collider) -> bool {
    let topo = |x: &Collider| Collider {
        friction: 0.0,
        restitution: 0.0,
        layers: 0,
        mask: 0,
        ..*x
    };
    topo(a) == topo(b)
}

/// Whether `b`'s child colliders have the same topology as built, returning
/// `(same_topology, any_param_changed)`.
fn compare_children(b: &Body, built: Option<&Vec<TrackedChild>>) -> (bool, bool) {
    match built {
        None => (b.children.is_empty(), false),
        Some(built) => {
            if built.len() != b.children.len() {
                return (false, false);
            }
            let mut param_changed = false;
            for (t, c) in built.iter().zip(&b.children) {
                if t.entity != c.entity.to_bits()
                    || t.position != c.position
                    || t.quaternion != c.quaternion
                    || t.scale != c.scale
                    || !same_collider_topology(&t.collider, &c.collider)
                {
                    return (false, false);
                }
                if t.collider != c.collider {
                    param_changed = true;
                }
            }
            (true, param_changed)
        }
    }
}

/// Whether two og.RigidBody values have the same structural topology
/// (kind, mass, center_of_mass); damping, gravity_scale, lock and ccd are
/// patched in place without losing warm starts or rebuilding joints.
fn same_body_topology(a: &RigidBody, b: &RigidBody) -> bool {
    a.kind == b.kind && a.mass == b.mass && a.center_of_mass == b.center_of_mass
}

fn body_params_differ(a: &RigidBody, b: &RigidBody) -> bool {
    a.linear_damping != b.linear_damping
        || a.angular_damping != b.angular_damping
        || a.gravity_scale != b.gravity_scale
        || a.lock != b.lock
        || a.ccd != b.ccd
}

fn plan(
    b: &Body,
    tracked: &IdMap<u64, Tracked>,
    compounds: &IdMap<u64, Vec<TrackedChild>>,
    stale: &BTreeSet<u64>,
) -> Plan {
    let stale_mesh = |c: &Collider| {
        matches!(c.shape, SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD)
            && stale.contains(&c.mesh)
    };
    let key = b.entity.to_bits();
    match tracked.get(&key) {
        Some(t) if same_body_topology(&t.body, &b.body) && t.scale == b.scale => {
            let own_ok = match (&t.collider, &b.collider) {
                (None, None) => true,
                (Some(tc), Some(bc)) => same_collider_topology(tc, bc) && !stale_mesh(bc),
                _ => false,
            };
            if !own_ok || b.children.iter().any(|c| stale_mesh(&c.collider)) {
                return Plan::Build;
            }
            let (children_ok, patch_children) = compare_children(b, compounds.get(&key));
            if !children_ok {
                return Plan::Build;
            }
            let patch_body = body_params_differ(&t.body, &b.body);
            let patch_own_collider = t.collider != b.collider;
            let kind = b.body.kind;
            let (mut next, mut position) = (None, None);
            if b.control.is_some() {
                // A controller moved it after the last step: the step
                // takes it there, pushing what is in the way.
                next = Some(pose(b.position, b.quaternion));
            } else if t.position != b.position || t.quaternion != b.quaternion {
                // Game code moved it (a kinematic body moving by its
                // velocity is where physics wrote it, so it is not moved).
                position = Some(pose(b.position, b.quaternion));
            }
            // Only push velocity updates when velocity changed or when a
            // moving kinematic body is integrating its velocity (avoids
            // waking motionless kinematic bodies every tick).
            let vel_needed = b.control.is_none()
                && match kind {
                    BODY_DYNAMIC => t.velocity != b.velocity,
                    BODY_KINEMATIC => t.velocity != b.velocity || moves_itself(b),
                    _ => false,
                };
            let velocity = vel_needed.then(|| {
                let vel = unlocked(b.velocity.unwrap_or_default(), b.body.lock);
                (v(flat(vel.linear)), angvel(vel.angular))
            });
            Plan::Keep {
                handle: t.handle,
                patch_body,
                patch_own_collider,
                patch_children,
                next,
                position,
                velocity,
            }
        }
        _ => Plan::Build,
    }
}

/// Entity keys in ascending order (a scene's usually are already: then no
/// sort).
fn sorted_keys(keys: impl Iterator<Item = u64>) -> Vec<u64> {
    let mut keys: Vec<u64> = keys.collect();
    if !keys.is_sorted() {
        keys.sort_unstable();
    }
    keys
}

/// Keeps the entries of `map` whose key is in `live` (ascending), in one
/// walk of both; `gone` sees each other entry, in key order, as it leaves.
fn retain_live<T>(map: &mut IdMap<u64, T>, live: &[u64], mut gone: impl FnMut(&T)) {
    let mut at = 0;
    map.retain(|key, value| {
        while at < live.len() && live[at] < *key {
            at += 1;
        }
        let keep = live.get(at) == Some(key);
        if !keep {
            gone(value);
        }
        keep
    });
}

/// Whether `runs` hold the entities `synced` held, run for run: an
/// archetype whose og.Collider columns have the stamp they had lost and
/// gained no row since, and neither run left a row out.
fn same_statics(synced: &[StaticRun], runs: &[StaticRun]) -> bool {
    synced.len() == runs.len()
        && synced
            .iter()
            .zip(runs)
            .all(|(a, b)| matches!((a.written, b.written), (Some(a), Some(b)) if a.1 == b.1))
}

/// The static colliders of `scene` a sync has to read, each with the index
/// of its run: all of them, less (when `settled`) each archetype's run
/// that is the run `synced` has at its place, which the sync that recorded
/// it left nothing to do for.
fn unsynced<'s>(
    scene: &'s Scene,
    synced: &'s [StaticRun],
    settled: bool,
) -> impl Iterator<Item = (usize, &'s Static)> {
    let mut rest = scene.statics.as_slice();
    let runs = scene.static_runs.iter().enumerate();
    runs.flat_map(move |(i, run)| {
        let (statics, after) = rest.split_at(run.len);
        rest = after;
        let same = settled && run.written.is_some() && synced.get(i) == Some(run);
        let read = if same { &[] } else { statics };
        read.iter().map(move |s| (i, s))
    })
}

/// Whether `s` equals itself. One with a NaN in it never equals what was
/// tracked of it, so every sync moves it or builds it again: its run is
/// never one to pass over.
#[allow(clippy::eq_op)]
fn exact(s: &Static) -> bool {
    s.position == s.position
        && s.quaternion == s.quaternion
        && s.scale == s.scale
        && s.collider == s.collider
}

fn patch_collider_in_place(c: &mut rp::Collider, next: &Collider, entity: Entity) -> Result<()> {
    for (what, x) in [("friction", next.friction), ("restitution", next.restitution)] {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(invalid(what, x).context(format!("entity {entity}")));
        }
    }
    c.set_friction(next.friction);
    c.set_restitution(next.restitution);
    c.set_collision_groups(groups(next));
    Ok(())
}

/// Brings the persistent rapier world in line with `scene` (bodies and
/// static colliders); returns each body's handle.
///
/// The static colliders are read by archetype: one whose og.Transform and
/// og.Collider columns were not written since the last sync holds the
/// statics that sync left tracked and built, and is passed over. What is
/// done to rapier, and in what order, is what reading them all would do:
/// nothing is done for a static that is as it was tracked.
pub(super) fn sync<'s>(
    settings: &Physics,
    dt: f32,
    scene: &'s Scene,
    shared: &mut Shared,
    sim: &mut Sim,
) -> Result<Handles<'s>> {
    if !settings.gravity.is_finite() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("og.Physics.gravity must be finite, got {}", settings.gravity),
        ));
    }
    let substeps = settings.substeps.clamp(1, 16);
    let Sim {
        world: w,
        bodies: tracked,
        statics: tracked_statics,
        static_runs,
        ext,
        ..
    } = sim;
    // Taken: a sync that fails leaves none.
    let synced = std::mem::take(static_runs);
    let compounds = &mut ext.compounds;
    let new_gravity = v(settings.gravity);
    let gravity_changed = w.gravity != new_gravity;
    w.gravity = new_gravity;
    if gravity_changed {
        // Wake all sleeping dynamic bodies when world gravity changes
        // (e.g. anti-gravity, zero-g or rotated gravity).
        let wake: Vec<_> = w
            .bodies
            .iter()
            .filter(|(_, rb)| rb.is_dynamic() && rb.is_sleeping())
            .map(|(h, _)| h)
            .collect();
        for h in wake {
            w.wake_up(h, true);
        }
    }
    w.integration_parameters.dt = dt / substeps as f32;
    w.integration_parameters.num_solver_iterations =
        settings.solver_iterations.clamp(1, 32) as usize;

    // What is gone leaves first, in entity order.
    let live = sorted_keys(scene.bodies.iter().map(|b| b.entity.to_bits()));
    retain_live(tracked, &live, |t| {
        w.remove_body(t.handle);
    });
    if !compounds.is_empty() {
        retain_live(compounds, &live, |_| {});
    }
    // (No static is gone when every archetype holds the entities it did.)
    if !same_statics(&synced, &scene.static_runs) {
        let live = sorted_keys(scene.statics.iter().map(|s| s.entity.to_bits()));
        if !ext.fields.is_empty() {
            retain_live(&mut ext.fields, &live, |chunks| {
                for &h in &chunks[1..] {
                    w.remove_collider(h);
                }
            });
        }
        retain_live(tracked_statics, &live, |t| {
            w.remove_collider(t.handle);
        });
    }

    let stale = std::mem::take(&mut shared.stale_meshes);
    let edits = std::mem::take(&mut shared.field_edits);
    let stale_mesh = |c: &Collider| {
        matches!(c.shape, SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD)
            && stale.contains(&c.mesh)
    };

    // What to do to each body, decided by ranges of bodies on the lanes;
    // then done in body order, so rapier sees its bodies touched, moved,
    // removed and inserted in the order one thread would.
    let plans = {
        let tracked = &*tracked;
        let compounds = &*compounds;
        let stale = &stale;
        by_bodies(scene.bodies.len(), |r| {
            scene.bodies[r]
                .iter()
                .map(|b| plan(b, tracked, compounds, stale))
                .collect()
        })
    };
    let mut by_body = Vec::with_capacity(scene.bodies.len());
    for (b, plan) in scene.bodies.iter().zip(plans) {
        let h = match plan {
            Plan::Keep {
                handle,
                patch_body,
                patch_own_collider,
                patch_children,
                next,
                position,
                velocity,
            } => {
                if !b.body.air_drag.is_finite() || b.body.air_drag < 0.0 {
                    return Err(Error::new(
                        ErrorKind::Invalid,
                        format!(
                            "entity {}: og.RigidBody.airDrag must be finite and at least 0, got {}",
                            b.entity, b.body.air_drag
                        ),
                    ));
                }
                if !b.position.is_finite() || !b.quaternion.is_finite() {
                    return Err(Error::new(
                        ErrorKind::Invalid,
                        format!("entity {}: og.Transform must be finite", b.entity),
                    ));
                }
                if let Some(vel) = b.velocity
                    && (!vel.linear.is_finite() || !vel.angular.is_finite())
                {
                    return Err(Error::new(
                        ErrorKind::Invalid,
                        format!("entity {}: og.Velocity must be finite", b.entity),
                    ));
                }
                if patch_body {
                    for (what, x) in [
                        ("linearDamping", b.body.linear_damping),
                        ("angularDamping", b.body.angular_damping),
                    ] {
                        if !(x >= 0.0 && x.is_finite()) {
                            return Err(Error::new(
                                ErrorKind::Invalid,
                                format!(
                                    "entity {}: og.RigidBody.{what} must be finite and at least 0, got {x}",
                                    b.entity
                                ),
                            ));
                        }
                    }
                    if !b.body.gravity_scale.is_finite() {
                        return Err(Error::new(
                            ErrorKind::Invalid,
                            format!(
                                "entity {}: og.RigidBody.gravityScale must be finite, got {}",
                                b.entity, b.body.gravity_scale
                            ),
                        ));
                    }
                    let rb = &mut w.bodies[handle];
                    rb.set_linear_damping(b.body.linear_damping);
                    rb.set_angular_damping(b.body.angular_damping);
                    rb.set_gravity_scale(b.body.gravity_scale, true);
                    rb.set_locked_axes(
                        rp::LockedAxes::from_bits_truncate((b.body.lock & 63) as u8),
                        true,
                    );
                    rb.enable_ccd(b.body.ccd);
                }
                if patch_own_collider
                    && let Some(bc) = &b.collider
                    && let Some(&ch) = w.bodies[handle].colliders().first()
                {
                    patch_collider_in_place(&mut w.colliders[ch], bc, b.entity)?;
                }
                if patch_children {
                    let skip = usize::from(b.collider.is_some());
                    let handles_copy: Vec<_> =
                        w.bodies[handle].colliders().iter().skip(skip).copied().collect();
                    for (child, ch) in b.children.iter().zip(handles_copy) {
                        patch_collider_in_place(&mut w.colliders[ch], &child.collider, child.entity)?;
                    }
                    compounds.insert(
                        b.entity.to_bits(),
                        b.children.iter().map(TrackedChild::of).collect(),
                    );
                }
                let rb = &mut w.bodies[handle];
                if let Some(at) = next {
                    rb.set_next_kinematic_position(at);
                } else if let Some(at) = position {
                    rb.set_position(at, true);
                }
                if let Some((linear, angular)) = velocity {
                    let wake = b.body.kind == BODY_DYNAMIC
                        || linear != rp::Vector::ZERO
                        || angular != angvel(Vec3::ZERO);
                    rb.set_linvel(linear, wake);
                    rb.set_angvel(angular, wake);
                }
                handle
            }
            Plan::Build => {
                let key = b.entity.to_bits();
                if let Some(t) = tracked.remove(&key) {
                    w.remove_body(t.handle);
                }
                if b.children.is_empty() {
                    compounds.remove(&key);
                } else {
                    compounds.insert(key, b.children.iter().map(TrackedChild::of).collect());
                }
                insert_body(w, b, shared)?
            }
        };
        by_body.push(h);
    }
    // A mesh or height field registered anew rebuilds colliders no write
    // touched: every static is read then.
    let settled = stale.is_empty() && edits.is_empty();
    let mut runs = scene.static_runs.clone();
    for (run, s) in unsynced(scene, &synced, settled) {
        if !exact(s) {
            runs[run].written = None;
        }
        if !s.position.is_finite() || !s.quaternion.is_finite() || !s.scale.is_finite() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("entity {}: og.Transform must be finite", s.entity),
            ));
        }
        let key = s.entity.to_bits();
        let at = pose(s.position, s.quaternion) * local_pose(&s.collider, s.scale);
        let field = chunked(&s.collider, shared);
        let fields = &mut ext.fields;
        let same_topo =
            |t: &TrackedStatic| same_collider_topology(&t.collider, &s.collider) && t.scale == s.scale;
        // A static field whose every change was a region edit rebuilds
        // only the chunks the edits reached, in chunk order.
        let partial = field.as_ref().and_then(|f| {
            let chunks = fields.get(&key)?;
            let (cx, cz) = crate::field_chunks(f.columns, f.rows);
            let edited = edits.get(&s.collider.mesh);
            (chunks.len() == (cx * cz) as usize && (!stale_mesh(&s.collider) || edited.is_some()))
                .then(|| edited.cloned().unwrap_or_default())
        });
        match tracked_statics.get_mut(&key) {
            Some(t)
                if same_topo(t)
                    && (partial.is_some() || (field.is_none() && !stale_mesh(&s.collider))) =>
            {
                let moved = t.position != s.position || t.quaternion != s.quaternion;
                let params_changed = t.collider != s.collider;
                match (&field, fields.get_mut(&key)) {
                    (Some(f), Some(chunks)) => {
                        for k in partial.unwrap_or_default() {
                            let h = &mut chunks[k as usize];
                            w.remove_collider(*h);
                            let c =
                                chunk_collider(&s.collider, f, k, s.scale, at, s.entity, shared)?;
                            *h = w.colliders.insert(c);
                        }
                        t.handle = chunks[0];
                        if moved {
                            for (k, &h) in chunks.iter().enumerate() {
                                let local = at
                                    * pose(
                                        crate::chunk_offset(f, s.scale, k as u32),
                                        Quat::IDENTITY,
                                    );
                                w.colliders[h].set_position(local);
                            }
                        }
                        if params_changed {
                            for &h in chunks.iter() {
                                patch_collider_in_place(&mut w.colliders[h], &s.collider, s.entity)?;
                            }
                        }
                    }
                    _ => {
                        if moved {
                            w.colliders[t.handle].set_position(at);
                        }
                        if params_changed {
                            patch_collider_in_place(&mut w.colliders[t.handle], &s.collider, s.entity)?;
                        }
                    }
                }
                t.collider = s.collider;
                t.position = s.position;
                t.quaternion = s.quaternion;
            }
            _ => {
                if let Some(t) = tracked_statics.remove(&key) {
                    w.remove_collider(t.handle);
                }
                if let Some(chunks) = fields.remove(&key) {
                    for &h in &chunks[1..] {
                        w.remove_collider(h);
                    }
                }
                let handle = match &field {
                    Some(f) => {
                        let (cx, cz) = crate::field_chunks(f.columns, f.rows);
                        let mut chunks = Vec::with_capacity((cx * cz) as usize);
                        for k in 0..cx * cz {
                            let c =
                                chunk_collider(&s.collider, f, k, s.scale, at, s.entity, shared)?;
                            chunks.push(w.colliders.insert(c));
                        }
                        let first = chunks[0];
                        fields.insert(key, chunks);
                        first
                    }
                    None => {
                        let c = collider(&s.collider, s.scale, at, s.entity, 0.0, shared)?;
                        w.colliders.insert(c)
                    }
                };
                tracked_statics.insert(
                    key,
                    TrackedStatic {
                        handle,
                        collider: s.collider,
                        scale: s.scale,
                        position: s.position,
                        quaternion: s.quaternion,
                    },
                );
            }
        }
    }

    *static_runs = runs;

    // Bodies that are no longer bodies (now static colliders or gone) were
    // removed above; a body re-made this tick keeps its joints out until
    // they are rebuilt.
    Ok(Handles {
        scene,
        by_body,
        by_entity: std::sync::OnceLock::new(),
    })
}

/// Applies what game code pushed the bodies with since the last step
/// (og.physics.applyForce and friends), in the order it asked: a force or
/// torque over this step (as the impulse it gives in `dt`), an impulse at
/// once. Only dynamic bodies move; the others ignore it.
pub(crate) fn apply_pushes(sim: &mut Sim, handles: &Handles<'_>, pushes: &[crate::Push], dt: f32) {
    use crate::PushKind;
    for p in pushes {
        let Some(&h) = handles.get(&p.entity) else {
            continue;
        };
        let rb = &mut sim.world.bodies[h];
        if !rb.is_dynamic() {
            continue;
        }
        let scale = match p.kind {
            PushKind::Force | PushKind::Torque => dt,
            PushKind::Impulse | PushKind::TorqueImpulse => 1.0,
        };
        let x = p.vector * scale;
        match p.kind {
            PushKind::Force | PushKind::Impulse => match p.point {
                Some(at) => rb.apply_impulse_at_point(v(flat(x)), v(at), true),
                None => rb.apply_impulse(v(flat(x)), true),
            },
            PushKind::Torque | PushKind::TorqueImpulse => rb.apply_torque_impulse(angvel(x), true),
        }
    }
}

// ---- the saved simulation ------------------------------------------------------
