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
    /// By collider handle index: its body kind bit (`QUERY_DYNAMIC`, `QUERY_FIXED`, `QUERY_KINEMATIC`).
    kind: Vec<u8>,
    /// Contact hooks snapshot for symmetric `ContactHook.ignore` filtering
    /// when a query specifies `QueryFilter.exclude`.
    hooks: IdMap<u64, ContactHook>,
}

fn hook_ignores(hooks: &IdMap<u64, ContactHook>, a: u64, b: u64) -> bool {
    hooks
        .get(&a)
        .is_some_and(|h| h.ignore.as_slice().iter().any(|&x| x != 0 && x == b))
}

fn body_kind_bit(b: &Body) -> u8 {
    if b.control.is_some() {
        QUERY_KINEMATIC as u8
    } else {
        match b.body.kind {
            BODY_DYNAMIC => QUERY_DYNAMIC as u8,
            BODY_KINEMATIC => QUERY_KINEMATIC as u8,
            _ => QUERY_FIXED as u8,
        }
    }
}

fn keep(
    h: rp::ColliderHandle,
    c: &rp::Collider,
    f: &QueryFilter,
    kinds: &[u8],
    hooks: &IdMap<u64, ContactHook>,
) -> bool {
    if !c.is_enabled() {
        return false;
    }
    if f.kind_flags != 0 {
        let bit = kinds
            .get(h.into_raw_parts().0 as usize)
            .copied()
            .unwrap_or(QUERY_FIXED as u8) as u32;
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
            kind: Vec::new(),
            hooks: scene.hooks.clone(),
        };
        let mut placed = Vec::with_capacity(scene.bodies.len() + scene.statics.len());
        for b in &scene.bodies {
            if !b.body.enabled {
                continue;
            }
            let kb = body_kind_bit(b);
            let body_pose = pose(b.position, b.quaternion);
            if let Some(c) = &b.collider
                && c.enabled
            {
                let at = body_pose * local_pose(c, b.scale);
                placed.push((b.entity, None, at, b.scale, c, kb));
            }
            for child in &b.children {
                if !child.collider.enabled {
                    continue;
                }
                let at = body_pose
                    * pose(child.position, child.quaternion)
                    * local_pose(&child.collider, child.scale);
                placed.push((
                    child.entity,
                    Some(b.entity),
                    at,
                    child.scale,
                    &child.collider,
                    kb,
                ));
            }
        }
        for s in &scene.statics {
            if !s.collider.enabled {
                continue;
            }
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
                    if q.kind.len() <= i {
                        q.kind.resize(i + 1, QUERY_FIXED as u8);
                    }
                    q.kind[i] = QUERY_FIXED as u8;
                    chunks.push(h);
                }
                q.fields.push(chunks);
                continue;
            }
            let at = pose(s.position, s.quaternion) * local_pose(&s.collider, s.scale);
            placed.push((s.entity, None, at, s.scale, &s.collider, QUERY_FIXED as u8));
        }
        for (e, body, at, scale, c, kb) in placed {
            let mut built = collider(c, scale, at, e, 0.0, shared)?.build();
            if let Some(body) = body {
                built.user_data |= (body.to_bits() as u128) << 64;
            }
            let aabb = built.compute_aabb();
            let h = q.colliders.insert(built);
            q.broad.set_aabb(&params, h, aabb);
            let i = h.into_raw_parts().0 as usize;
            if q.kind.len() <= i {
                q.kind.resize(i + 1, QUERY_FIXED as u8);
            }
            q.kind[i] = kb;
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
        let predicate = |h: rp::ColliderHandle, c: &rp::Collider| {
            !chunk(h) && keep(h, c, f, &self.kind, &self.hooks)
        };
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
        for chunks in &self.fields {
            let h0 = chunks[0];
            let c = &self.colliders[h0];
            if !keep(h0, c, f, &self.kind, &self.hooks) || (c.is_sensor() && !f.include_sensors) {
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
                .map(|(h, hit)| {
                    let c = &self.colliders[h];
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
                    && keep(h, c, &d.filter, &self.kind, &self.hooks)
            };
            let mut filter = rp::QueryFilter::new().predicate(&predicate);
            if !d.filter.include_sensors {
                filter = filter.exclude_sensors();
            }
            let pipeline = self.broad.as_query_pipeline(
                self.narrow.query_dispatcher(),
                &self.bodies,
                &self.colliders,
                filter,
            );
            let Some((h, hit)) = pipeline.cast_shape(&at, v(dir), shape.as_ref(), options) else {
                break;
            };
            let c = &self.colliders[h];
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
