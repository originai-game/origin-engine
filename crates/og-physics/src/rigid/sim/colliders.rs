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
    for (what, x) in [
        ("radius", c.radius),
        ("height", c.height),
        ("roundRadius", c.round_radius),
    ] {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(invalid(what, x));
        }
    }
    for x in c.size.to_array() {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(invalid("size", x));
        }
    }
    if !scale.is_finite() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("og.Transform.scale must be finite, got {scale}"),
        ));
    }
    let abs_s = scale.abs();
    if let Some(k) = crate::key_chunk(c.shape) {
        return chunk_shape(c.mesh, k, abs_s, shared);
    }
    if let Some(shape) = primitive(c, abs_s)? {
        return Ok(shape);
    }
    let mesh_s = if c.shape == SHAPE_HEIGHTFIELD {
        abs_s
    } else {
        Vec3::new(
            if scale.x == 0.0 { 0.0 } else { scale.x },
            if scale.y == 0.0 { 0.0 } else { scale.y },
            if scale.z == 0.0 { 0.0 } else { scale.z },
        )
    };
    let key = ShapeKey {
        shape: c.shape,
        mesh: c.mesh,
        scale: mesh_s.to_array().map(f32::to_bits),
        round_radius: c.round_radius.max(0.0).to_bits(),
    };
    if let Some(shape) = cache(shared).get(&key) {
        return Ok(shape.clone());
    }
    if c.shape == SHAPE_HEIGHTFIELD {
        if abs_s.x <= 0.0 || (super::DIM == 3 && abs_s.z <= 0.0) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("heightfield collider requires positive horizontal scale, got {scale}"),
            ));
        }
        let field = shared.heightfields.get(&c.mesh).cloned().ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!(
                    "og.Collider.mesh {} is not a height field registered with PhysicsSystem::set_heightfield",
                    c.mesh
                ),
            )
        })?;
        let built = from_heightfield(&field, abs_s)?;
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
    let built = match baked(shared, c, mesh_s, &mesh) {
        Some(b) => b,
        None => from_mesh(c.shape, &mesh, mesh_s, c.round_radius)
            .map_err(|e| e.context(format!("mesh {}", c.mesh)))?,
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
    if s.x <= 0.0 || s.z <= 0.0 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("heightfield collider requires positive x/z scale, got {s}"),
        ));
    }
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
            round_radius: 0,
        },
        k,
    );
    if let Some(shape) = chunk_cache(shared).get(&key) {
        return Ok(shape.clone());
    }
    let built = field_chunk(&field, s, k);
    let cache = chunk_cache(shared);
    if cache.len() >= SHAPE_CACHE_LIMIT * 4 {
        cache.clear();
    }
    cache.insert(key, built.clone());
    Ok(built)
}

/// The chunks a static collider is built as: for a 3D height field of more
/// than one chunk ([`crate::FIELD_CHUNK`]), the field; `None` for every
/// other collider (one rapier collider).
pub(super) fn chunked(c: &Collider, shared: &Shared) -> Option<Arc<crate::HeightfieldDesc>> {
    if super::DIM == 2 || c.shape != SHAPE_HEIGHTFIELD {
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

pub(super) fn groups(c: &Collider) -> rp::InteractionGroups {
    rp::InteractionGroups::new(
        rp::Group::from_bits_retain(c.layers),
        rp::Group::from_bits_retain(c.mask),
        rp::InteractionTestMode::And,
    )
}

pub(super) fn solver_groups(c: &Collider) -> rp::InteractionGroups {
    let mem = if c.solver_layers != 0 {
        c.solver_layers
    } else {
        c.layers
    };
    let fil = if c.solver_mask != 0 {
        c.solver_mask
    } else {
        c.mask
    };
    rp::InteractionGroups::new(
        rp::Group::from_bits_retain(mem),
        rp::Group::from_bits_retain(fil),
        rp::InteractionTestMode::And,
    )
}

pub(super) fn combine_rule(code: u32) -> rp::CoefficientCombineRule {
    match code {
        COMBINE_MIN => rp::CoefficientCombineRule::Min,
        COMBINE_MULTIPLY => rp::CoefficientCombineRule::Multiply,
        COMBINE_MAX => rp::CoefficientCombineRule::Max,
        _ => rp::CoefficientCombineRule::Average,
    }
}

/// A collider builder at `local` (relative to its body, or the world pose of
/// a static collider). `body_mass` > 0 overrides the density; `< 0` forces
/// zero mass (used when mass properties are placed on the body directly).
/// Sensor colliders always carry zero mass.
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
        .friction_combine_rule(combine_rule(c.friction_combine))
        .restitution(c.restitution)
        .restitution_combine_rule(combine_rule(c.restitution_combine))
        .sensor(c.sensor)
        .collision_groups(groups(c))
        .solver_groups(solver_groups(c))
        .enabled(c.enabled)
        .user_data(entity.to_bits() as u128);
    b = if c.sensor || body_mass < 0.0 {
        b.density(0.0)
    } else if body_mass > 0.0 {
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

/// Mass properties of a solid collider shape at `mass` (> 0) or `density`.
pub(super) fn collider_mass_props(
    shape: &rp::SharedShape,
    density: f32,
    mass: f32,
) -> rp::MassProperties {
    if mass > 0.0 {
        let mut mp = shape.mass_properties(1.0);
        mp.set_mass(mass, true);
        mp
    } else {
        shape.mass_properties(density)
    }
}

pub(super) fn local_pose(c: &Collider, scale: Vec3) -> rp::Pose {
    pose(c.offset * scale, c.quaternion)
}
