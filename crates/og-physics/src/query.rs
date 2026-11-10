use super::*;

/// Scene queries over a fixed set of colliders (see [`PhysicsSystem::query`]).
/// Results depend only on the colliders, so they are as deterministic as the
/// world they were built from.
pub struct PhysicsQuery {
    pub(super) inner: Arc<QueryInner>,
    pub(super) shared: Arc<Mutex<Shared>>,
}

pub(crate) enum QueryInner {
    #[cfg(feature = "physics3d")]
    D3(dim3::QueryWorld),
    #[cfg(feature = "physics2d")]
    D2(dim2::QueryWorld),
}

impl QueryInner {
    fn build(profile: u32, scene: &Scene, shared: &mut Shared) -> Result<QueryInner> {
        Ok(match profile {
            #[cfg(feature = "physics3d")]
            PROFILE_3D => QueryInner::D3(dim3::QueryWorld::build(scene, shared)?),
            #[cfg(feature = "physics2d")]
            PROFILE_2D => QueryInner::D2(dim2::QueryWorld::build(scene, shared)?),
            other => return Err(check_profile(other).unwrap_err()),
        })
    }

    /// [`Self::build`] outside the step, through [`outside`].
    pub(super) fn build_outside(
        profile: u32,
        scene: &Scene,
        shared: &Mutex<Shared>,
    ) -> Result<QueryInner> {
        outside(|| Self::build(profile, scene, &mut lock(shared)))
    }
}

fn max_distance(d: f32) -> f32 {
    if d > 0.0 { d } else { f32::MAX }
}

impl PhysicsQuery {
    /// The nearest collider along a ray.
    pub fn raycast(&self, desc: &RaycastDesc) -> Result<Option<RaycastHit>> {
        let max = max_distance(desc.max_distance);
        outside(|| match &*self.inner {
            #[cfg(feature = "physics3d")]
            QueryInner::D3(q) => q.raycast(desc.origin, desc.direction, max, &desc.filter),
            #[cfg(feature = "physics2d")]
            QueryInner::D2(q) => q.raycast(desc.origin, desc.direction, max, &desc.filter),
        })
    }

    /// Every collider a ray meets within its reach, nearest first (ties by
    /// collider entity, then body), at most `max_hits`: one hit per
    /// collider, at its first crossing.
    pub fn raycast_all(&self, desc: &RaycastAllDesc) -> Result<Vec<RaycastHit>> {
        let r = &desc.ray;
        let max = max_distance(r.max_distance);
        let cap = match desc.max_hits {
            0 => MAX_RAYCAST_HITS,
            n => n.min(MAX_RAYCAST_HITS),
        } as usize;
        outside(|| match &*self.inner {
            #[cfg(feature = "physics3d")]
            QueryInner::D3(q) => q.raycast_all(r.origin, r.direction, max, &r.filter, cap),
            #[cfg(feature = "physics2d")]
            QueryInner::D2(q) => q.raycast_all(r.origin, r.direction, max, &r.filter, cap),
        })
    }

    /// The nearest point on any collider to a point, within its reach.
    pub fn project_point(&self, desc: &ProjectPointDesc) -> Result<Option<PointProjection>> {
        let max = max_distance(desc.max_distance);
        outside(|| match &*self.inner {
            #[cfg(feature = "physics3d")]
            QueryInner::D3(q) => q.project_point(desc, max),
            #[cfg(feature = "physics2d")]
            QueryInner::D2(q) => q.project_point(desc, max),
        })
    }

    /// The first collider a shape meets when swept along a direction.
    pub fn shape_cast(&self, desc: &ShapeCastDesc) -> Result<Option<ShapeCastHit>> {
        let max = max_distance(desc.max_distance);
        outside(|| {
            let mut shared = lock(&self.shared);
            match &*self.inner {
                #[cfg(feature = "physics3d")]
                QueryInner::D3(q) => q.shape_cast(desc, max, &mut shared),
                #[cfg(feature = "physics2d")]
                QueryInner::D2(q) => q.shape_cast(desc, max, &mut shared),
            }
        })
    }

    /// Every collider overlapping a shape, sorted by entity id.
    pub fn overlap(&self, desc: &OverlapDesc) -> Result<Vec<Entity>> {
        outside(|| {
            let mut shared = lock(&self.shared);
            match &*self.inner {
                #[cfg(feature = "physics3d")]
                QueryInner::D3(q) => q.overlap(desc, &mut shared),
                #[cfg(feature = "physics2d")]
                QueryInner::D2(q) => q.overlap(desc, &mut shared),
            }
        })
    }
}

/// [`PhysicsQuery::raycast`] on a view built for this one call.
pub fn raycast(
    world: &World,
    physics: &PhysicsSystem,
    desc: &RaycastDesc,
) -> Result<Option<RaycastHit>> {
    physics.query(world)?.raycast(desc)
}

/// [`PhysicsQuery::shape_cast`] on a view built for this one call.
pub fn shape_cast(
    world: &World,
    physics: &PhysicsSystem,
    desc: &ShapeCastDesc,
) -> Result<Option<ShapeCastHit>> {
    physics.query(world)?.shape_cast(desc)
}

/// [`PhysicsQuery::overlap`] on a view built for this one call.
pub fn overlap(world: &World, physics: &PhysicsSystem, desc: &OverlapDesc) -> Result<Vec<Entity>> {
    physics.query(world)?.overlap(desc)
}
