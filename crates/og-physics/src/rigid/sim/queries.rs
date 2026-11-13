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
}

fn keep(c: &rp::Collider, f: &QueryFilter) -> bool {
    (f.mask == 0 || c.collision_groups().memberships.bits() & f.mask != 0)
        && f.exclude.is_none_or(|e| e.to_bits() != c.user_data as u64)
}

fn query_shape(d: &ShapeDesc, shared: &mut Shared) -> Result<rp::SharedShape> {
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
        };
        let mut placed = Vec::with_capacity(scene.bodies.len() + scene.statics.len());
        for b in &scene.bodies {
            if let Some(c) = &b.collider {
                placed.push((b.entity, None, b.position, b.quaternion, b.scale, c));
            }
            for child in &b.children {
                let (p, r) = (
                    b.position + b.quaternion * child.position,
                    b.quaternion * child.quaternion,
                );
                placed.push((
                    child.entity,
                    Some(b.entity),
                    p,
                    r,
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
            placed.push((
                s.entity,
                None,
                s.position,
                s.quaternion,
                s.scale,
                &s.collider,
            ));
        }
        for (e, body, position, quaternion, scale, c) in placed {
            let at = pose(position, quaternion) * local_pose(c, scale);
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
        let predicate = |h: rp::ColliderHandle, c: &rp::Collider| !chunk(h) && keep(c, f);
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
        let dir = crate::direction(flat(direction), "RaycastDesc", PROFILE_NAME)?;
        let ray = rp::Ray::new(v(origin), v(dir));
        let mut best = self.pipeline(f, true, |p| p.cast_ray_and_get_normal(&ray, max, true));
        // A field's chunks in the order the ray enters them, each cast only
        // as far as the nearest hit so far. In the BVH, rapier casts both
        // leaves of a node at once and goes down the nearer subtree first, so
        // a far chunk was walked cell by cell to its edge before a nearer
        // chunk's (or a wall's) hit could cut it short: a ray over a chunked
        // field visited several times the cells one field does.
        let mut order: Vec<(f32, rp::ColliderHandle)> = Vec::new();
        for chunks in &self.fields {
            let c = &self.colliders[chunks[0]];
            if !keep(c, f) || (c.is_sensor() && !f.include_sensors) {
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
            if enter >= limit {
                break;
            }
            let c = &self.colliders[h];
            if let Some(hit) = c
                .shape()
                .cast_ray_and_get_normal(c.position(), &ray, limit, true)
                .filter(|hit| hit.time_of_impact < limit)
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
        // The BVH's leaf order is not an order a game should see: nearest
        // first, ties by entity bits, so every host and run lists the same.
        hits.sort_unstable_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then(a.entity.to_bits().cmp(&b.entity.to_bits()))
                .then(a.body.to_bits().cmp(&b.body.to_bits()))
        });
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

    pub(crate) fn shape_cast(
        &self,
        d: &ShapeCastDesc,
        max: f32,
        shared: &mut Shared,
    ) -> Result<Option<ShapeCastHit>> {
        let dir = crate::direction(flat(d.direction), "ShapeCastDesc", PROFILE_NAME)?;
        let shape = query_shape(&d.shape, shared)?;
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

    pub(crate) fn overlap(&self, d: &OverlapDesc, shared: &mut Shared) -> Result<Vec<Entity>> {
        let shape = query_shape(&d.shape, shared)?;
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
