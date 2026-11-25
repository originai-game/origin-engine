use super::*;

fn hook_ignores(hooks: &IdMap<u64, ContactHook>, a: u64, b: u64) -> bool {
    hooks
        .get(&a)
        .is_some_and(|h| h.ignore.as_slice().iter().any(|&x| x != 0 && x == b))
}

fn keep(
    c: &rp::Collider,
    f: &QueryFilter,
    bodies: &rp::RigidBodySet,
    hooks: &IdMap<u64, ContactHook>,
) -> bool {
    if !c.is_enabled()
        || c.parent()
            .and_then(|h| bodies.get(h))
            .is_some_and(|b| !b.is_enabled())
    {
        return false;
    }
    if f.kind_flags != 0 {
        let bit = match c.parent().and_then(|h| bodies.get(h)) {
            Some(b) if b.is_dynamic() => QUERY_DYNAMIC,
            Some(b) if b.is_kinematic() => QUERY_KINEMATIC,
            _ => QUERY_FIXED,
        };
        if bit & f.kind_flags == 0 {
            return false;
        }
    }
    if f.mask != 0 && c.collision_groups().memberships.bits() & f.mask == 0 {
        return false;
    }
    if let Some(ex) = f.exclude {
        let ex_bits = ex.to_bits();
        let ce = entity_of(c).to_bits();
        let cb = body_of(c).to_bits();
        if ex_bits == ce || ex_bits == cb {
            return false;
        }
        if !hooks.is_empty()
            && (hook_ignores(hooks, ex_bits, ce)
                || hook_ignores(hooks, ex_bits, cb)
                || hook_ignores(hooks, ce, ex_bits)
                || (cb != ce && hook_ignores(hooks, cb, ex_bits)))
        {
            return false;
        }
    }
    true
}

pub(crate) fn query_shape(d: &ShapeDesc, shared: &mut Shared) -> Result<rp::SharedShape> {
    shape(&d.collider(), Vec3::ONE, shared)
}

impl Sim {
    /// Populate the one collider world without advancing time. Existing
    /// simulations (especially restored ones) never pass through this path:
    /// syncing them here would consume the step's kinematic pose delta.
    pub(crate) fn initialize(
        settings: &Physics,
        dt: f32,
        scene: &Scene,
        shared: &mut Shared,
    ) -> Result<Self> {
        let mut sim = Self::default();
        let handles = sync(settings, dt, scene, shared, &mut sim)?;
        // No step/readback runs here to record these bodies. Keep their
        // ownership now, or the first step builds a second copy and orphans the first.
        for (b, &handle) in scene.bodies.iter().zip(&handles.by_body) {
            sim.bodies.insert(
                b.entity.to_bits(),
                Tracked {
                    handle,
                    body: b.body,
                    collider: b.collider,
                    scale: b.scale,
                    position: b.position,
                    quaternion: b.quaternion,
                    velocity: b.velocity,
                },
            );
        }
        let mut out = StepOut::default();
        super::super::vehicles(scene, 0.0, &mut sim, &handles, &mut out)?;
        super::super::ragdolls(scene, 0.0, &mut sim, shared, &mut out)?;
        sim.ext.hooked = !scene.hooks.is_empty();
        sim.ext.last_hooks = scene.hooks.clone();
        for (_, body) in sim.world.bodies.iter_mut() {
            body.recompute_mass_properties_from_colliders(&sim.world.colliders);
        }
        sim.world
            .bodies
            .propagate_modified_body_positions_to_colliders(&mut sim.world.colliders);
        // CollisionPipeline consumes the bodies' pending changes without
        // registering them with the dynamics islands. Seed only the live
        // BVH; the first step must still process every insertion and pair.
        sim.refresh_query_bounds();
        Ok(sim)
    }

    pub(super) fn refresh_query_bounds(&mut self) {
        let w = &mut self.world;
        for (handle, collider) in w.colliders.iter().filter(|(_, c)| c.is_enabled()) {
            w.broad_phase.set_aabb(
                &w.integration_parameters,
                handle,
                collider.compute_broad_phase_aabb(&w.integration_parameters, &w.bodies),
            );
        }
    }

    fn with<T>(&self, f: &QueryFilter, run: impl FnOnce(&rp::QueryPipeline<'_>) -> T) -> T {
        self.pipeline(f, false, run)
    }

    /// [`Self::with`], without the height fields' chunks when `no_chunks`.
    fn pipeline<T>(
        &self,
        f: &QueryFilter,
        no_chunks: bool,
        run: impl FnOnce(&rp::QueryPipeline<'_>) -> T,
    ) -> T {
        let chunk = |h: rp::ColliderHandle| {
            no_chunks && self.ext.fields.values().any(|chunks| chunks.contains(&h))
        };
        let predicate = |h: rp::ColliderHandle, c: &rp::Collider| {
            !chunk(h) && keep(c, f, &self.world.bodies, &self.ext.last_hooks)
        };
        let mut filter = rp::QueryFilter::new().predicate(&predicate);
        if !f.include_sensors {
            filter = filter.exclude_sensors();
        }
        let pipeline = self.world.broad_phase.as_query_pipeline(
            self.world.narrow_phase.query_dispatcher(),
            &self.world.bodies,
            &self.world.colliders,
            filter,
        );
        run(&pipeline)
    }

    pub(crate) fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max: f32,
        solid: bool,
        f: &QueryFilter,
    ) -> Result<Option<RaycastHit>> {
        if !origin.is_finite() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("RaycastDesc.origin must be finite, got {origin}"),
            ));
        }
        let dir = crate::direction(flat(direction), "RaycastDesc", PROFILE_NAME)?;
        let ray = rp::Ray::new(v(origin), v(dir));
        let mut best = self.pipeline(f, true, |p| p.cast_ray_and_get_normal(&ray, max, solid));
        // A field's chunks in the order the ray enters them, each cast only
        // as far as the nearest hit so far.
        let mut order: Vec<(f32, rp::ColliderHandle)> = Vec::new();
        for chunks in self.ext.fields.values() {
            let h0 = chunks[0];
            let c = &self.world.colliders[h0];
            if !keep(c, f, &self.world.bodies, &self.ext.last_hooks)
                || (c.is_sensor() && !f.include_sensors)
            {
                continue;
            }
            for &h in chunks {
                if let Some((enter, _)) = self.world.colliders[h]
                    .compute_aabb()
                    .clip_ray_parameters(&ray)
                {
                    order.push((enter, h));
                }
            }
        }
        order.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (enter, h) in order {
            let limit = best.map_or(max, |(_, hit)| hit.time_of_impact);
            if enter > limit || (best.is_some() && enter >= limit) {
                break;
            }
            let c = &self.world.colliders[h];
            if let Some(hit) = c
                .shape()
                .cast_ray_and_get_normal(c.position(), &ray, limit, solid)
                .filter(|hit| {
                    if best.is_some() {
                        hit.time_of_impact < limit
                    } else {
                        hit.time_of_impact <= limit
                    }
                })
            {
                best = Some((h, hit));
            }
        }
        Ok(best.map(|(h, hit)| RaycastHit {
            entity: entity_of(&self.world.colliders[h]),
            body: body_of(&self.world.colliders[h]),
            distance: hit.time_of_impact,
            point: unv(ray.point_at(hit.time_of_impact)),
            normal: unv(hit.normal),
        }))
    }

    pub(crate) fn raycast_all(
        &self,
        origin: Vec3,
        direction: Vec3,
        max: f32,
        solid: bool,
        f: &QueryFilter,
        cap: usize,
    ) -> Result<Vec<RaycastHit>> {
        if !origin.is_finite() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("RaycastDesc.origin must be finite, got {origin}"),
            ));
        }
        let dir = crate::direction(flat(direction), "RaycastDesc", PROFILE_NAME)?;
        let ray = rp::Ray::new(v(origin), v(dir));
        let mut hits = self.with(f, |p| {
            p.intersect_ray(ray, max, solid)
                .map(|(_, c, hit)| RaycastHit {
                    entity: entity_of(c),
                    body: body_of(c),
                    distance: hit.time_of_impact,
                    point: unv(ray.point_at(hit.time_of_impact)),
                    normal: unv(hit.normal),
                })
                .collect::<Vec<_>>()
        });
        // Nearest first, ties by entity bits, so every host and run lists the same.
        hits.sort_unstable_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then(a.entity.to_bits().cmp(&b.entity.to_bits()))
                .then(a.body.to_bits().cmp(&b.body.to_bits()))
        });
        // A chunked height field splits one entity into multiple colliders:
        // keep only its nearest crossing so a caller never sees duplicate hits.
        if !self.ext.fields.is_empty() && hits.len() > 1 {
            let mut seen = BTreeSet::new();
            hits.retain(|h| seen.insert((h.entity.to_bits(), h.body.to_bits())));
        }
        hits.truncate(cap);
        Ok(hits)
    }

    pub(crate) fn project_point(
        &self,
        d: &crate::ProjectPointDesc,
        max: f32,
    ) -> Result<Option<crate::PointProjection>> {
        if !d.point.is_finite() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("ProjectPointDesc.point must be finite, got {}", d.point),
            ));
        }
        let at = flat(d.point);
        Ok(self.with(&d.filter, |p| {
            let (h, proj) = p.project_point(v(at), max, d.solid)?;
            let c = &self.world.colliders[h];
            let point = unv(proj.point);
            let to = at - point;
            let distance = to.length();
            let normal = if distance > 0.0 {
                let n = to / distance;
                if proj.is_inside { -n } else { n }
            } else {
                Vec3::ZERO
            };
            Some(crate::PointProjection {
                entity: entity_of(c),
                body: body_of(c),
                point,
                normal,
                distance,
                inside: proj.is_inside,
            })
        }))
    }

    pub(crate) fn shape_cast_with(
        &self,
        d: &ShapeCastDesc,
        max: f32,
        shape: &rp::SharedShape,
    ) -> Result<Option<ShapeCastHit>> {
        if !d.position.is_finite() || !d.rotation.is_finite() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "ShapeCastDesc.position and rotation must be finite, got {} / {}",
                    d.position, d.rotation
                ),
            ));
        }
        let dir = crate::direction(flat(d.direction), "ShapeCastDesc", PROFILE_NAME)?;
        let at = pose(d.position, d.rotation);
        let options = ShapeCastOptions::with_max_time_of_impact(max);
        Ok(self.with(&d.filter, |p| {
            p.cast_shape(&at, v(dir), shape.as_ref(), options)
                .map(|(h, hit)| {
                    let c = &self.world.colliders[h];
                    ShapeCastHit {
                        entity: entity_of(c),
                        body: body_of(c),
                        distance: hit.time_of_impact,
                        point: unv(hit.witness1),
                        normal: unv(hit.normal1),
                    }
                })
        }))
    }

    pub(crate) fn shape_cast_all_with(
        &self,
        d: &ShapeCastDesc,
        max: f32,
        shape: &rp::SharedShape,
        cap: usize,
    ) -> Result<Vec<ShapeCastHit>> {
        if !d.position.is_finite() || !d.rotation.is_finite() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "ShapeCastDesc.position and rotation must be finite, got {} / {}",
                    d.position, d.rotation
                ),
            ));
        }
        let dir = crate::direction(flat(d.direction), "ShapeCastDesc", PROFILE_NAME)?;
        let at = pose(d.position, d.rotation);
        let options = ShapeCastOptions::with_max_time_of_impact(max);
        let mut hits = Vec::new();
        let mut seen_entities: BTreeSet<(u64, u64)> = BTreeSet::new();
        let mut excluded: Vec<rp::ColliderHandle> = Vec::new();
        for _ in 0..cap {
            let predicate = |h: rp::ColliderHandle, c: &rp::Collider| {
                !excluded.contains(&h)
                    && !seen_entities.contains(&(entity_of(c).to_bits(), body_of(c).to_bits()))
                    && keep(c, &d.filter, &self.world.bodies, &self.ext.last_hooks)
            };
            let mut filter = rp::QueryFilter::new().predicate(&predicate);
            if !d.filter.include_sensors {
                filter = filter.exclude_sensors();
            }
            let pipeline = self.world.broad_phase.as_query_pipeline(
                self.world.narrow_phase.query_dispatcher(),
                &self.world.bodies,
                &self.world.colliders,
                filter,
            );
            let Some((h, hit)) = pipeline.cast_shape(&at, v(dir), shape.as_ref(), options) else {
                break;
            };
            let c = &self.world.colliders[h];
            let entity = entity_of(c);
            let body = body_of(c);
            excluded.push(h);
            seen_entities.insert((entity.to_bits(), body.to_bits()));
            hits.push(ShapeCastHit {
                entity,
                body,
                distance: hit.time_of_impact,
                point: unv(hit.witness1),
                normal: unv(hit.normal1),
            });
        }
        hits.sort_unstable_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then(a.entity.to_bits().cmp(&b.entity.to_bits()))
                .then(a.body.to_bits().cmp(&b.body.to_bits()))
        });
        Ok(hits)
    }

    pub(crate) fn overlap_with(
        &self,
        d: &OverlapDesc,
        shape: &rp::SharedShape,
    ) -> Result<Vec<Entity>> {
        if !d.position.is_finite() || !d.rotation.is_finite() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "OverlapDesc.position and rotation must be finite, got {} / {}",
                    d.position, d.rotation
                ),
            ));
        }
        let at = pose(d.position, d.rotation);
        let mut hits = self.with(&d.filter, |p| {
            p.intersect_shape(at, shape.as_ref())
                .map(|(_, c)| entity_of(c))
                .collect::<Vec<_>>()
        });
        og_core::sort::sort_by_key(&mut hits, |e| e.to_bits());
        hits.dedup();
        Ok(hits)
    }

    pub(crate) fn overlap_point(&self, d: &PointOverlapDesc) -> Result<Vec<Entity>> {
        if !d.point.is_finite() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("PointOverlapDesc.point must be finite, got {}", d.point),
            ));
        }
        let at = v(flat(d.point));
        let mut hits = self.with(&d.filter, |p| {
            p.intersect_point(at)
                .map(|(_, c)| entity_of(c))
                .collect::<Vec<_>>()
        });
        og_core::sort::sort_by_key(&mut hits, |e| e.to_bits());
        hits.dedup();
        Ok(hits)
    }
}

// ---- forces and impulses --------------------------------------------------------
