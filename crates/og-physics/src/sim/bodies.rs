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

/// A kinematic body with a nonzero og.Velocity: the step moves it by it
/// and writes its Transform back.
pub(super) fn moves_itself(b: &Body) -> bool {
    b.body.kind == BODY_KINEMATIC
        && b.control.is_none()
        && b.velocity
            .is_some_and(|v| v.linear != Vec3::ZERO || v.angular != Vec3::ZERO)
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
    /// Still the body it was: moved where game code or a controller put it
    /// (`next` kinematic target, or `position` teleport) and given a
    /// `velocity`, each when it changed.
    Keep {
        handle: rp::RigidBodyHandle,
        next: Option<rp::Pose>,
        position: Option<rp::Pose>,
        velocity: Option<(rp::Vector, rp::AngVector)>,
    },
    /// New, or its body, collider or scale changed: built afresh.
    Build,
}

/// Whether `b`'s child colliders are those its body was built with.
fn same_children(b: &Body, built: Option<&Vec<TrackedChild>>) -> bool {
    match built {
        None => b.children.is_empty(),
        Some(built) => {
            built.len() == b.children.len()
                && built
                    .iter()
                    .zip(&b.children)
                    .all(|(t, c)| *t == TrackedChild::of(c))
        }
    }
}

/// Whether two og.RigidBody values build the same rapier body: sleep's
/// fields (`canSleep`, `sleeping`) change none of it, and a save does not
/// keep them in [`Tracked`].
fn same_body(a: &RigidBody, b: &RigidBody) -> bool {
    let shape = |x: &RigidBody| RigidBody {
        can_sleep: true,
        sleeping: false,
        ..*x
    };
    shape(a) == shape(b)
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
        Some(t)
            if same_body(&t.body, &b.body)
                && t.collider == b.collider
                && t.scale == b.scale
                && !b.collider.as_ref().is_some_and(stale_mesh)
                && same_children(b, compounds.get(&key))
                && !b.children.iter().any(|c| stale_mesh(&c.collider)) =>
        {
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
            let velocity = (b.control.is_none()
                && (kind == BODY_KINEMATIC || (kind == BODY_DYNAMIC && t.velocity != b.velocity)))
                .then(|| {
                    let vel = unlocked(b.velocity.unwrap_or_default(), b.body.lock);
                    (v(flat(vel.linear)), angvel(vel.angular))
                });
            Plan::Keep {
                handle: t.handle,
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

/// Brings the persistent rapier world in line with `scene` (bodies and
/// static colliders); returns each body's handle.
pub(super) fn sync<'s>(
    settings: &Physics,
    dt: f32,
    scene: &'s Scene,
    shared: &mut Shared,
    sim: &mut Sim,
) -> Result<Handles<'s>> {
    let substeps = settings.substeps.clamp(1, 16);
    let Sim {
        world: w,
        bodies: tracked,
        statics: tracked_statics,
        ext,
        ..
    } = sim;
    let compounds = &mut ext.compounds;
    w.gravity = v(settings.gravity);
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
                next,
                position,
                velocity,
            } => {
                let rb = &mut w.bodies[handle];
                if let Some(at) = next {
                    rb.set_next_kinematic_position(at);
                } else if let Some(at) = position {
                    rb.set_position(at, true);
                }
                if let Some((linear, angular)) = velocity {
                    rb.set_linvel(linear, true);
                    rb.set_angvel(angular, true);
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
    for s in &scene.statics {
        let key = s.entity.to_bits();
        let at = pose(s.position, s.quaternion) * local_pose(&s.collider, s.scale);
        let field = chunked(&s.collider, shared);
        let fields = &mut ext.fields;
        let same = |t: &TrackedStatic| t.collider == s.collider && t.scale == s.scale;
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
                if same(t)
                    && (partial.is_some() || (field.is_none() && !stale_mesh(&s.collider))) =>
            {
                let moved = t.position != s.position || t.quaternion != s.quaternion;
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
                    }
                    _ if moved => w.colliders[t.handle].set_position(at),
                    _ => {}
                }
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
