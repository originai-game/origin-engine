use super::*;

/// Registered mesh `handle`, or the host's mesh source's (built on first
/// use and kept like a registered one).
pub(crate) fn mesh(shared: &mut Shared, handle: u64) -> Option<Arc<crate::MeshDesc>> {
    if let Some(m) = shared.meshes.get(&handle) {
        return Some(m.clone());
    }
    let m = Arc::new(shared.mesh_source.clone().and_then(|f| f(handle))?);
    shared.meshes.insert(handle, m.clone());
    Some(m)
}

/// The shape of a collider at a transform scale: primitives are built, mesh
/// shapes come from the cache (or are built into it).
pub(super) fn shape(c: &Collider, scale: Vec3, shared: &mut Shared) -> Result<rp::SharedShape> {
    for (what, x) in [("radius", c.radius), ("height", c.height)] {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(invalid(what, x));
        }
    }
    for x in c.size.to_array() {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(invalid("size", x));
        }
    }
    let s = scale.abs();
    if !s.is_finite() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("og.Transform.scale must be finite, got {scale}"),
        ));
    }
    if c.shape == SHAPE_HEIGHTFIELD
        && let Some(why) = shared.heightfields_absent
    {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("og.Collider.shape 6 (SHAPE_HEIGHTFIELD, a terrain's height field): {why}"),
        ));
    }
    if let Some(k) = crate::key_chunk(c.shape) {
        return chunk_shape(c.mesh, k, s, shared);
    }
    if let Some(shape) = primitive(c, s)? {
        return Ok(shape);
    }
    let key = ShapeKey {
        shape: c.shape,
        mesh: c.mesh,
        scale: s.to_array().map(f32::to_bits),
    };
    if let Some(shape) = cache(shared).get(&key) {
        return Ok(shape.clone());
    }
    if c.shape == SHAPE_HEIGHTFIELD {
        let field = shared.heightfields.get(&c.mesh).cloned().ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!(
                    "og.Collider.mesh {} is not a height field registered with PhysicsSystem::set_heightfield",
                    c.mesh
                ),
            )
        })?;
        let built = from_heightfield(&field, s)?;
        let cached = cache(shared);
        if cached.len() >= SHAPE_CACHE_LIMIT {
            cached.clear();
        }
        cached.insert(key, built.clone());
        return Ok(built);
    }
    // A mesh nobody registered is asked of the host's mesh source (its
    // geometry, built on first use and kept like a registered one).
    let mesh = mesh(shared, c.mesh).ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            format!(
                "og.Collider.mesh {} was not registered with PhysicsSystem::set_mesh",
                c.mesh
            ),
        )
    })?;
    let built = match baked(shared, c, s, &mesh) {
        Some(b) => b,
        None => from_mesh(c.shape, &mesh, s).map_err(|e| e.context(format!("mesh {}", c.mesh)))?,
    };
    let cached = cache(shared);
    if cached.len() >= SHAPE_CACHE_LIMIT {
        cached.clear();
    }
    cached.insert(key, built.clone());
    Ok(built)
}

/// Chunk `k` of height field `mesh` at scale `s` (positive), from the
/// chunk cache or built into it.
fn chunk_shape(mesh: u64, k: u32, s: Vec3, shared: &mut Shared) -> Result<rp::SharedShape> {
    let field = shared.heightfields.get(&mesh).cloned().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            format!(
                "og.Collider.mesh {mesh} is not a height field registered with PhysicsSystem::set_heightfield"
            ),
        )
    })?;
    let key = crate::chunk_key(
        ShapeKey {
            shape: SHAPE_HEIGHTFIELD,
            mesh,
            scale: s.to_array().map(f32::to_bits),
        },
        k,
    );
    if let Some(shape) = chunk_cache(shared).get(&key) {
        return Ok(shape.clone());
    }
    let built = field_chunk(&field, s, k);
    chunk_cache(shared).insert(key, built.clone());
    Ok(built)
}

/// The chunks a static collider is built as: for a height field of more
/// than one chunk ([`crate::FIELD_CHUNK`]), the field; `None` for every
/// other collider (one rapier collider).
pub(super) fn chunked(c: &Collider, shared: &Shared) -> Option<Arc<crate::HeightfieldDesc>> {
    if c.shape != SHAPE_HEIGHTFIELD || shared.heightfields_absent.is_some() {
        return None;
    }
    let f = shared.heightfields.get(&c.mesh)?;
    let (cx, cz) = crate::field_chunks(f.columns, f.rows);
    (cx * cz > 1).then(|| f.clone())
}

/// Chunk `k` of a static height-field collider `c` at world pose `at`.
pub(super) fn chunk_collider(
    c: &Collider,
    f: &crate::HeightfieldDesc,
    k: u32,
    scale: Vec3,
    at: rp::Pose,
    entity: Entity,
    shared: &mut Shared,
) -> Result<rp::ColliderBuilder> {
    for (what, x) in [
        ("friction", c.friction),
        ("restitution", c.restitution),
        ("density", c.density),
    ] {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(invalid(what, x).context(format!("entity {entity}")));
        }
    }
    let shape = chunk_shape(c.mesh, k, scale.abs(), shared)
        .map_err(|e| e.context(format!("entity {entity}")))?;
    let local = at * pose(crate::chunk_offset(f, scale, k), Quat::IDENTITY);
    Ok(builder(c, shape, local, entity, 0.0))
}

fn groups(c: &Collider) -> rp::InteractionGroups {
    rp::InteractionGroups::new(
        rp::Group::from_bits_retain(c.layers),
        rp::Group::from_bits_retain(c.mask),
        rp::InteractionTestMode::And,
    )
}

/// A collider builder at `local` (relative to its body, or the world pose of
/// a static collider). `body_mass` > 0 overrides the density.
pub(super) fn collider(
    c: &Collider,
    scale: Vec3,
    local: rp::Pose,
    entity: Entity,
    body_mass: f32,
    shared: &mut Shared,
) -> Result<rp::ColliderBuilder> {
    for (what, x) in [
        ("friction", c.friction),
        ("restitution", c.restitution),
        ("density", c.density),
    ] {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(invalid(what, x).context(format!("entity {entity}")));
        }
    }
    let shape = shape(c, scale, shared).map_err(|e| e.context(format!("entity {entity}")))?;
    Ok(builder(c, shape, local, entity, body_mass))
}

/// [`collider`] with its shape built.
fn builder(
    c: &Collider,
    shape: rp::SharedShape,
    local: rp::Pose,
    entity: Entity,
    body_mass: f32,
) -> rp::ColliderBuilder {
    let mut b = rp::ColliderBuilder::new(shape)
        .position(local)
        .friction(c.friction)
        .restitution(c.restitution)
        .sensor(c.sensor)
        .collision_groups(groups(c))
        .user_data(entity.to_bits() as u128);
    b = if body_mass > 0.0 {
        b.mass(body_mass)
    } else {
        b.density(c.density)
    };
    if c.sensor {
        // A sensor also notices kinematic and static colliders.
        b = b.active_collision_types(rp::ActiveCollisionTypes::all());
    }
    b
}

/// A body's collider with its centre of mass moved by
/// `og.RigidBody.centerOfMass` (in the entity's frame, scaled like the
/// collider's offset): the shape's mass properties at the body's mass (or
/// the collider's density) with their centre shifted, the inertia kept.
/// Unchanged when the offset is zero.
pub(super) fn centred(
    b: rp::ColliderBuilder,
    c: &Collider,
    body: &RigidBody,
    scale: Vec3,
    local: rp::Pose,
) -> Result<rp::ColliderBuilder> {
    let offset = body.center_of_mass;
    if offset == Vec3::ZERO {
        return Ok(b);
    }
    if !offset.is_finite() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("og.RigidBody.centerOfMass must be finite, got {offset}"),
        ));
    }
    let mut mp = if body.mass > 0.0 {
        let mut mp = b.shape.mass_properties(1.0);
        mp.set_mass(body.mass, true);
        mp
    } else {
        b.shape.mass_properties(c.density)
    };
    mp.local_com += local.rotation.inverse() * v(flat(offset * scale));
    Ok(b.mass_properties(mp))
}

pub(super) fn local_pose(c: &Collider, scale: Vec3) -> rp::Pose {
    pose(c.offset * scale, c.rotation)
}
