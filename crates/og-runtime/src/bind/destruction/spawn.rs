//! Chunk entities: their mass, transform, record and spawning.

use super::*;

/// A body's mass as a hit weighs it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Mass {
    Finite(f32),
    /// Static, kinematic or no body: it does not give.
    Infinite,
    /// A shape whose volume is not known here (a mesh): the hit is the
    /// solver's impulse alone.
    Unknown,
}

/// The mass of the body entity `bits` belongs to: a chunk's island, an
/// unbroken prop's chunks, or another body from its og.RigidBody.mass or
/// its og.Collider's shape and density.
pub(super) fn mass_of(
    ctx: &SystemContext<'_>,
    l: &Ls,
    props: &BTreeMap<u64, Prop>,
    assets: &Assets,
    settings: &BTreeMap<u64, Settings>,
    bits_: u64,
) -> Mass {
    let e = Entity::from_bits(bits_);
    let scale_of = |p: &Prop| uniform(pose_of_frame(&p.frame).2);
    for (k, p) in props {
        let Some(a) = assets.get(&p.asset) else {
            continue;
        };
        let density = settings.get(k).map_or(1000.0, |s| s.mat.density);
        let s3 = {
            let k = scale_of(p);
            k * k * k
        };
        if let Some(c) = p.entities.iter().position(|&x| x == bits_) {
            let body = p.body[c];
            if body == 0 {
                return Mass::Infinite;
            }
            let v: f32 = (0..p.entities.len())
                .filter(|&i| p.entities[i] != 0 && p.body[i] == body)
                .map(|i| a.chunks[i].volume)
                .sum();
            return Mass::Finite(v * s3 * density);
        }
        if *k == bits_ && !p.placed {
            let v: f32 = a.chunks[..a.support as usize]
                .iter()
                .map(|c| c.volume)
                .sum();
            return Mass::Finite(v * s3 * density);
        }
    }
    let Ok(rb) = ctx.get_bytes(e, l.rigid.0) else {
        return Mass::Infinite;
    };
    let Ok(rb) = row::<record::RigidBody>(&l.rigid.1, &rb) else {
        return Mass::Unknown;
    };
    // og.RigidBody.kind is an enum, which the hand read this replaced
    // always took as 0 (dynamic): a fixed or kinematic body is weighed as
    // a dynamic one (kept as it was).
    let m = rb.mass;
    if m > 0.0 {
        return Mass::Finite(m);
    }
    let Ok(cb) = ctx.get_bytes(e, l.collider.0) else {
        return Mass::Unknown;
    };
    let Ok(c) = row::<record::Collider>(&l.collider.1, &cb) else {
        return Mass::Unknown;
    };
    let s = world_pose(ctx, l, e).map_or(Vec3::ONE, |p| p.2.abs());
    let (r, h) = (c.radius, c.height);
    let pi = std::f32::consts::PI;
    let volume = match c.shape {
        og_physics::SHAPE_BOX => {
            let z = c.size;
            z.x * z.y * z.z * s.x * s.y * s.z
        }
        og_physics::SHAPE_SPHERE => 4.0 / 3.0 * pi * r * r * r * s.x * s.y * s.z,
        og_physics::SHAPE_CAPSULE => {
            (pi * r * r * h + 4.0 / 3.0 * pi * r * r * r) * s.x * s.y * s.z
        }
        og_physics::SHAPE_CYLINDER => pi * r * r * h * s.x * s.y * s.z,
        _ => return Mass::Unknown,
    };
    Mass::Finite(volume * c.density.max(0.0))
}

pub(super) fn set_transform(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    e: Entity,
    p: &Pose,
) -> og_core::Result<()> {
    let t = l.transform.1.enc(
        None,
        Some(&json!({ "position": jv(p.0), "quaternion": jq(p.1), "scale": jv(p.2) })),
    )?;
    ctx.insert_bytes(e, l.transform.0, &t)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn set_chunk(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    e: Entity,
    prop: u64,
    chunk: u32,
    level: u32,
    body: u64,
    loose: u64,
) -> og_core::Result<()> {
    let b = l
        .chunk
        .1
        .enc(None, Some(&chunk_value(prop, chunk, level, body, loose)))?;
    ctx.insert_bytes(e, l.chunk.0, &b)
}

pub(super) fn chunk_value(prop: u64, chunk: u32, level: u32, body: u64, loose: u64) -> Value {
    json!({
        "destructible": prop as f64, "chunk": chunk, "level": level,
        "body": body as f64,
        "looseTick": if loose == u64::MAX { ANCHORED } else { loose.min(u64::from(u32::MAX - 1)) as u32 },
    })
}

/// Spawns chunk `c` of prop `k` at `pose` (in the world for a static chunk
/// or a root; relative to its body for a child), as `role`.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_chunk(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    k: u64,
    p: &mut Prop,
    c: u32,
    level: u32,
    pose: &Pose,
    role: Role,
    body: u64,
    tick: u64,
) -> og_core::Result<u64> {
    let mut collider = match &p.collider {
        Some(b) => l.collider.1.decode(b),
        None => json!({}),
    };
    let keep = ["friction", "restitution", "layers", "mask"];
    if let Value::Object(o) = &mut collider {
        o.retain(|k, _| keep.contains(&k.as_str()));
    }
    collider["shape"] = json!(og_physics::SHAPE_CONVEX);
    collider["mesh"] = json!(hull_handle(p.asset, c) as f64);
    let mut parts = vec![
        (
            l.transform.0,
            l.transform.1.enc(
                None,
                Some(&json!({ "position": jv(pose.0), "quaternion": jq(pose.1), "scale": jv(pose.2) })),
            )?,
        ),
        (l.collider.0, l.collider.1.enc(None, Some(&collider))?),
    ];
    let (loose, body_bits) = match role {
        Role::Static => (u64::MAX, 0),
        _ => (tick, body),
    };
    match role {
        Role::Root => parts.push((
            l.rigid.0,
            l.rigid.1.enc(None, Some(&json!({ "kind": "dynamic" })))?,
        )),
        Role::Child(_) => parts.push((
            l.parent.0,
            l.parent
                .1
                .enc(None, Some(&json!({ "entity": body as f64 })))?,
        )),
        Role::Static => {}
    }
    // A root's body is itself, whose id the spawn gives: its og.Chunk is
    // inserted after (a reserved entity takes commands at once).
    let e = ctx.spawn_bytes(parts)?;
    let body_bits = if role == Role::Root {
        e.to_bits()
    } else {
        body_bits
    };
    set_chunk(ctx, l, e, k, c, level, body_bits, loose)?;
    p.entities[c as usize] = e.to_bits();
    p.body[c as usize] = body_bits;
    p.loose[c as usize] = loose;
    Ok(e.to_bits())
}

/// Spawns a prop's support chunks as static colliders where the model
/// stands, and takes its own collider and body away.
pub(super) fn place(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    k: u64,
    p: &mut Prop,
    a: &FractureAsset,
) -> og_core::Result<()> {
    let e = Entity::from_bits(k);
    p.collider = ctx.get_bytes(e, l.collider.0).ok();
    for id in [l.collider.0, l.rigid.0, l.velocity.0] {
        if ctx.get_bytes(e, id).is_ok() {
            ctx.remove_id(e, id)?;
        }
    }
    let frame = pose_of_frame(&p.frame);
    for c in 0..a.support {
        if p.inst.shattered[c as usize] {
            continue;
        }
        let at = (
            place_on(&frame, Vec3::ZERO, a.chunks[c as usize].centroid),
            frame.1,
            frame.2,
        );
        spawn_chunk(ctx, l, k, p, c, 1, &at, Role::Static, 0, 0)?;
    }
    p.placed = true;
    Ok(())
}
