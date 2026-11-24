//! Clearance-checked capsule edits, shared by native and script callers.

use super::*;

impl PhysicsQuery {
    /// A resized capsule with its feet fixed in the supplied world frame.
    /// None means clearance is blocked; no world state has changed.
    pub fn resized_character_capsule(
        &self,
        entity: Entity,
        frame: Transform,
        collider: Collider,
        height: f32,
        radius: f32,
    ) -> Result<Option<Collider>> {
        if collider.shape != SHAPE_CAPSULE
            || !height.is_finite()
            || height < 0.0
            || !radius.is_finite()
            || radius <= 0.0
        {
            return Err(Error::new(
                ErrorKind::Invalid,
                "resizeCharacter requires a capsule, finite height >= 0 and radius > 0",
            ));
        }
        let scale = frame.scale.abs();
        let radial = match &*self.inner {
            QueryInner::D2(_) => scale.x,
            QueryInner::D3(_) => scale.x.max(scale.z),
        };
        if scale.y <= 0.0 || radial <= 0.0 {
            return Err(Error::new(
                ErrorKind::Invalid,
                "resizeCharacter requires nonzero capsule scale",
            ));
        }
        let rise = (height - collider.height) * 0.5 * scale.y + (radius - collider.radius) * radial;
        let resized = Collider {
            height,
            radius,
            offset: collider.offset + Vec3::Y * (rise / frame.scale.y),
            ..collider
        };
        let q = unit(frame.quaternion);
        let centre = |c: Collider| frame.position + q * (c.offset * frame.scale);
        let shape = ShapeDesc {
            shape: SHAPE_CAPSULE,
            height: height * scale.y,
            radius: radius * radial,
            ..ShapeDesc::default()
        };
        let filter = QueryFilter {
            mask: collider.mask,
            exclude: Some(entity),
            ..QueryFilter::default()
        };
        if height > collider.height {
            let old_top = centre(collider) + q * Vec3::Y * (collider.height * scale.y * 0.5);
            let distance =
                (height - collider.height) * scale.y + (radius - collider.radius) * radial;
            if distance > 0.0
                && self
                    .shape_cast(&ShapeCastDesc {
                        shape: ShapeDesc {
                            shape: SHAPE_SPHERE,
                            radius: radius * radial,
                            ..ShapeDesc::default()
                        },
                        position: old_top,
                        rotation: q,
                        direction: q * Vec3::Y,
                        max_distance: distance,
                        filter,
                    })?
                    .is_some()
            {
                return Ok(None);
            }
        }
        if !self
            .overlap(&OverlapDesc {
                shape,
                position: centre(resized),
                rotation: q,
                filter,
            })?
            .is_empty()
        {
            return Ok(None);
        }
        Ok(Some(resized))
    }
}

/// Resize a controller's capsule, preserving its feet and refusing blocked
/// growth. The next sync edits the existing Rapier collider without rebuilding
/// its body, so contacts, velocity and joints survive a crouch/stand cycle.
pub fn resize_character(
    world: &mut World,
    physics: &PhysicsSystem,
    entity: Entity,
    height: f32,
    radius: f32,
) -> Result<bool> {
    if world.get::<CharacterController>(entity).is_err()
        && world.get::<PlatformerController>(entity).is_err()
    {
        return Err(Error::new(
            ErrorKind::Invalid,
            "resizeCharacter requires a character controller",
        ));
    }
    let t = world.get::<Transform>(entity)?;
    let (position, quaternion, scale) = og_core::math::world_pose(
        entity,
        (t.position, t.quaternion, t.scale),
        |e| {
            world
                .get::<Parent>(e)
                .ok()
                .filter(|p| p.entity != 0)
                .map(|p| Entity::from_bits(p.entity))
        },
        |e| {
            world
                .get::<Transform>(e)
                .ok()
                .map(|t| (t.position, t.quaternion, t.scale))
        },
    );
    let c = world.get::<Collider>(entity)?;
    let Some(c) = physics.query(world)?.resized_character_capsule(
        entity,
        Transform {
            position,
            quaternion,
            scale,
        },
        c,
        height,
        radius,
    )?
    else {
        return Ok(false);
    };
    world.insert(entity, c)?;
    Ok(true)
}
