use super::*;

/// The colliders of a scene, placed for queries (no bodies, no stepping).
pub(crate) struct QueryWorld {
    bodies: rp::RigidBodySet,
    colliders: rp::ColliderSet,
    broad: rp::BroadPhaseBvh,
    narrow: rp::NarrowPhase,
    /// Each static height field built in chunks ([`crate::FIELD_CHUNK`]):
    /// its chunks' colliders, which [`QueryWorld::raycast`] casts itself
    /// in the order the ray meets them.
    fields: Vec<Vec<rp::ColliderHandle>>,
    /// By collider handle index: whether it is one of those chunks.
    chunk: Vec<bool>,
    /// Contact hooks snapshot for symmetric `ContactHook.ignore` filtering
    /// when a query specifies `QueryFilter.exclude`.
    hooks: IdMap<u64, ContactHook>,
}

fn hook_ignores(hooks: &IdMap<u64, ContactHook>, a: u64, b: u64) -> bool {
    hooks
        .get(&a)
        .is_some_and(|h| h.ignore.as_slice().iter().any(|&x| x != 0 && x == b))
}

fn keep(c: &rp::Collider, f: &QueryFilter, hooks: &IdMap<u64, ContactHook>) -> bool {
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

impl QueryWorld {
    pub(crate) fn build(scene: &Scene, shared: &mut Shared) -> Result<QueryWorld> {
        let params = rp::IntegrationParameters::default();
        let mut q = QueryWorld {
            bodies: rp::RigidBodySet::new(),
            colliders: rp::ColliderSet::new(),
            broad: rp::BroadPhaseBvh::new(),
            narrow: rp::NarrowPhase::new(),
            fields: Vec::new(),
            chunk: Vec::new(),
            hooks: scene.hooks.clone(),
        };
        let mut placed = Vec::with_capacity(scene.bodies.len() + scene.statics.len());
        for b in &scene.bodies {
            let body_pose = pose(b.position, b.quaternion);
            if let Some(c) = &b.collider {
                let at = body_pose * local_pose(c, b.scale);
                placed.push((b.entity, None, at, b.scale, c));
            }
            for child in &b.children {
                let at = body_pose
                    * pose(child.position, child.quaternion)
                    * local_pose(&child.collider, child.scale);
                placed.push((
                    child.entity,
                    Some(b.entity),
                    at,
                    child.scale,
                    &child.collider,
                ));
            }
        }
        for s in &scene.statics {
            // A field built in chunks is queried as its chunks, as stepped.
            if let Some(f) = chunked(&s.collider, shared) {
                let at = pose(s.position, s.quaternion) * local_pose(&s.collider, s.scale);
                let (cx, cz) = crate::field_chunks(f.columns, f.rows);
                let mut chunks = Vec::with_capacity((cx * cz) as usize);
                for k in 0..cx * cz {
                    let built =
                        chunk_collider(&s.collider, &f, k, s.scale, at, s.entity, shared)?.build();
                    let aabb = built.compute_aabb();
                    let h = q.colliders.insert(built);
                    q.broad.set_aabb(&params, h, aabb);
                    let i = h.into_raw_parts().0 as usize;
                    if q.chunk.len() <= i {
                        q.chunk.resize(i + 1, false);
                    }
                    q.chunk[i] = true;
                    chunks.push(h);
                }
                q.fields.push(chunks);
                continue;
            }
            let at = pose(s.position, s.quaternion) * local_pose(&s.collider, s.scale);
            placed.push((s.entity, None, at, s.scale, &s.collider));
        }
        for (e, body, at, scale, c) in placed {
            let mut built = collider(c, scale, at, e, 0.0, shared)?.build();
            if let Some(body) = body {
                built.user_data |= (body.to_bits() as u128) << 64;
            }
            let aabb = built.compute_aabb();
            let h = q.colliders.insert(built);
            q.broad.set_aabb(&params, h, aabb);
        }
        Ok(q)
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
            no_chunks
                && self
                    .chunk
                    .get(h.into_raw_parts().0 as usize)
                    .is_some_and(|&c| c)
        };
        let predicate =
            |h: rp::ColliderHandle, c: &rp::Collider| !chunk(h) && keep(c, f, &self.hooks);
        let mut filter = rp::QueryFilter::new().predicate(&predicate);
        if !f.include_sensors {
            filter = filter.exclude_sensors();
        }
        let pipeline = self.broad.as_query_pipeline(
            self.narrow.query_dispatcher(),
            &self.bodies,
            &self.colliders,
            filter,
        );
        run(&pipeline)
    }

    pub(crate) fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max: f32,
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
        let mut best = self.pipeline(f, true, |p| p.cast_ray_and_get_normal(&ray, max, true));
        // A field's chunks in the order the ray enters them, each cast only
        // as far as the nearest hit so far.
        let mut order: Vec<(f32, rp::ColliderHandle)> = Vec::new();
        for chunks in &self.fields {
            let c = &self.colliders[chunks[0]];
            if !keep(c, f, &self.hooks) || (c.is_sensor() && !f.include_sensors) {
                continue;
            }
            for &h in chunks {
                if let Some((enter, _)) = self.colliders[h].compute_aabb().clip_ray_parameters(&ray)
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
            let c = &self.colliders[h];
            if let Some(hit) = c
                .shape()
                .cast_ray_and_get_normal(c.position(), &ray, limit, true)
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
            entity: entity_of(&self.colliders[h]),
            body: body_of(&self.colliders[h]),
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
            p.intersect_ray(ray, max, true)
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
        if !self.fields.is_empty() && hits.len() > 1 {
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
            let c = &self.colliders[h];
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
                .map(|(h, hit)| ShapeCastHit {
                    entity: entity_of(&self.colliders[h]),
                    distance: hit.time_of_impact,
                    point: unv(hit.witness1),
                    normal: unv(hit.normal1),
                })
        }))
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
}

// ---- forces and impulses --------------------------------------------------------
