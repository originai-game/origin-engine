//! Islands: the ones a break freed given their bodies, chunks shattered,
//! fresh pieces committed and the budgets enforced.

use super::*;

/// Each loose body of prop `p` (a support chunk's body with the chunks it
/// carries, or a shattered piece on its own): its mass and place. A body
/// made this run has no transform yet: a piece's place is then `fallback`.
#[allow(clippy::too_many_arguments)]
pub(super) fn loose_bodies(
    ctx: &SystemContext<'_>,
    l: &Ls,
    p: &Prop,
    a: &FractureAsset,
    scale: f32,
    density: f32,
    poses: &[Option<Pose>],
    made: &BTreeMap<u64, (Vec3, Vec3)>,
    fallback: Vec3,
) -> BTreeMap<u64, (f32, Vec3)> {
    // An island's place is its centre of mass (its chunks' centres by
    // their masses), not its root chunk's: a long island whose root lies
    // by a blast is pushed as a whole far from it is.
    let mut bodies: BTreeMap<u64, (f32, Vec3)> = BTreeMap::new();
    let mut placed: BTreeMap<u64, f32> = BTreeMap::new();
    for (c, &b) in p.body.iter().enumerate() {
        if b == 0 || p.entities[c] == 0 {
            continue;
        }
        let m = a.chunks[c].volume * scale * scale * scale * density;
        let e = bodies.entry(b).or_insert((0.0, Vec3::ZERO));
        e.0 += m;
        if let Some(ps) = poses[c] {
            e.1 += ps.0 * m;
            *placed.entry(b).or_default() += m;
        }
    }
    for (b, e) in bodies.iter_mut() {
        if let Some(&m) = placed.get(b).filter(|&&m| m > 0.0) {
            e.1 /= m;
        }
    }
    for c in a.support as usize..a.chunks.len() {
        let e = p.entities[c];
        if e != 0 && !bodies.contains_key(&e) {
            let m = a.chunks[c].volume * scale * scale * scale * density;
            let at = match made.contains_key(&e) {
                true => None,
                false => world_pose(ctx, l, Entity::from_bits(e)).map(|x| x.0),
            };
            bodies.insert(e, (m, at.unwrap_or(fallback)));
        }
    }
    bodies
}

/// Each island that is not anchored and not already one body becomes
/// one: its root chunk a dynamic body with the velocity it had, the rest
/// its compound children; og.Fractured says where it broke.
#[allow(clippy::too_many_arguments)]
pub(super) fn free_islands(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    t: &Clock,
    at: Breaking<'_>,
    frame: &Pose,
    p: &mut Prop,
    scale: f32,
    inherit: (Vec3, Vec3, Vec3),
    broken_now: &[bool],
    cause: u32,
    out: &mut Broken,
) -> og_core::Result<()> {
    let (a, s, k, pose, tick) = (at.a, at.s, at.key, at.pose, t.tick);
    let islands = islands_of(&p.inst, a, p.anchors);
    // Each support chunk's role now.
    let role_of = |p: &Prop, c: usize| -> Role {
        let (e, b) = (p.entities[c], p.body[c]);
        if b == 0 {
            Role::Static
        } else if b == e {
            Role::Root
        } else {
            match p.entities.iter().position(|&x| x == b) {
                Some(r) => Role::Child(r as u32),
                None => Role::Static,
            }
        }
    };
    let roles: Vec<Role> = (0..a.support as usize).map(|c| role_of(p, c)).collect();
    for isl in &islands {
        if isl.anchored {
            continue;
        }
        let root_now = |c: u32| match roles[c as usize] {
            Role::Root => Some(c),
            Role::Child(r) => Some(r),
            Role::Static => None,
        };
        let first = root_now(isl.chunks[0]);
        let same = first.is_some()
            && isl.chunks.iter().all(|&c| root_now(c) == first)
            && isl.chunks.contains(&first.expect("some"))
            && (0..a.support).all(|c| isl.chunks.contains(&c) || root_now(c) != first);
        if same {
            continue;
        }
        let root = isl
            .chunks
            .iter()
            .copied()
            .find(|&c| roles[c as usize] == Role::Root)
            .unwrap_or(isl.chunks[0]);
        let root_pose = pose[root as usize].expect("a placed chunk has a pose");
        let root_entity = p.entities[root as usize];
        // The velocity it starts with: its old body's at its place.
        let (lin, ang) = match roles[root as usize] {
            Role::Root => (Vec3::ZERO, Vec3::ZERO),
            Role::Child(r) => {
                let b = Entity::from_bits(p.entities[r as usize]);
                match ctx.get_bytes(b, l.velocity.0) {
                    Ok(vb) => {
                        let v: record::Velocity = row(&l.velocity.1, &vb)?;
                        let (lin, ang) = (v.linear, v.angular);
                        let at = pose[r as usize].map_or(root_pose.0, |x| x.0);
                        (lin + ang.cross(root_pose.0 - at), ang)
                    }
                    Err(_) => (Vec3::ZERO, Vec3::ZERO),
                }
            }
            Role::Static => (
                inherit.0 + inherit.1.cross(root_pose.0 - inherit.2),
                inherit.1,
            ),
        };
        let mut mass = 0.0;
        let mut volume = 0.0;
        let mut centre = Vec3::ZERO;
        for &c in &isl.chunks {
            let target = if c == root {
                Role::Root
            } else {
                Role::Child(root)
            };
            let was = roles[c as usize];
            let e = Entity::from_bits(p.entities[c as usize]);
            let cp = pose[c as usize].expect("a placed chunk has a pose");
            let v = a.chunks[c as usize].volume * scale * scale * scale;
            volume += v;
            mass += v * s.mat.density;
            centre += cp.0 * v;
            if p.loose[c as usize] == u64::MAX {
                p.loose[c as usize] = tick;
            }
            p.body[c as usize] = root_entity;
            set_chunk(ctx, l, e, k, c, 1, root_entity, p.loose[c as usize])?;
            if was == target {
                continue;
            }
            match target {
                Role::Root => {
                    if matches!(was, Role::Child(_)) {
                        ctx.remove_id(e, l.parent.0)?;
                        set_transform(ctx, l, e, &cp)?;
                    }
                    let rb = l.rigid.1.enc(None, Some(&json!({ "kind": "dynamic" })))?;
                    ctx.insert_bytes(e, l.rigid.0, &rb)?;
                    out.made.insert(root_entity, (lin, ang));
                    out.fresh_bodies.push(root_entity);
                }
                Role::Child(_) => {
                    if was == Role::Root {
                        ctx.remove_id(e, l.rigid.0)?;
                        ctx.remove_id(e, l.velocity.0)?;
                    }
                    let pb = l
                        .parent
                        .1
                        .enc(None, Some(&json!({ "entity": root_entity as f64 })))?;
                    ctx.insert_bytes(e, l.parent.0, &pb)?;
                    set_transform(ctx, l, e, &relative(&root_pose, &cp))?;
                }
                Role::Static => {}
            }
        }
        // Where it broke: the bonds broken this run along its edge.
        let mut at_ = Vec3::ZERO;
        let mut n = 0.0;
        for (b, bond) in a.bonds.iter().enumerate() {
            let touches = isl.chunks.contains(&bond.a) || isl.chunks.contains(&bond.b);
            if touches && broken_now[b] {
                at_ += place_chunk(at, frame, bond.a, bond.centroid);
                n += 1.0;
            }
        }
        let point = if n > 0.0 {
            at_ / n
        } else {
            centre / volume.max(1e-12)
        };
        out.events.push(l.fractured.1.enc(
            None,
            Some(&json!({
                "entity": k as f64, "body": root_entity as f64,
                "chunks": isl.chunks.len(), "point": jv(point),
                "volume": volume, "mass": mass, "cause": cause,
            })),
        )?);
        if s.break_fx != 0 {
            out.bursts.push((s.break_fx, point));
        }
    }
    Ok(())
}

/// Shattered chunks: their children fly, each its own body, or, below
/// debrisVolume with a debris emitter, as fine pieces on the GPU.
#[allow(clippy::too_many_arguments)]
pub(super) fn shatter_chunks(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    t: &Clock,
    at: Breaking<'_>,
    p: &mut Prop,
    scale: f32,
    shatter: &[u32],
    out: &mut Broken,
) -> og_core::Result<()> {
    let (a, s, k, pose, tick) = (at.a, at.s, at.key, at.pose, t.tick);
    for &c in shatter {
        let ch = &a.chunks[c as usize];
        let Some(cp) = pose[c as usize] else {
            continue;
        };
        let (lin, ang) = match p.body[c as usize] {
            0 => (Vec3::ZERO, Vec3::ZERO),
            b => match ctx.get_bytes(Entity::from_bits(b), l.velocity.0) {
                Ok(vb) => {
                    let v: record::Velocity = row(&l.velocity.1, &vb)?;
                    (v.linear, v.angular)
                }
                Err(_) => (Vec3::ZERO, Vec3::ZERO),
            },
        };
        let e = p.entities[c as usize];
        if alive(ctx, e) {
            ctx.despawn(Entity::from_bits(e))?;
        }
        p.entities[c as usize] = 0;
        p.body[c as usize] = 0;
        let (first, count) = ch.children;
        let mut volume = 0.0;
        let mut body = 0;
        let mut fine = 0;
        for i in first..first + count {
            let kid = &a.chunks[i as usize];
            let v = kid.volume * scale * scale * scale;
            volume += v;
            let kp = (place_on(&cp, ch.centroid, kid.centroid), cp.1, cp.2);
            // Pieces below debrisVolume, given a debris emitter, are
            // the GPU's (its burst, and the pieces themselves in the
            // renderer's debris layer): never bodies.
            if s.debris != 0 && v < t.debris_volume {
                fine += 1;
                let at = kp.0;
                out.debris.push((
                    Debris {
                        prop: k,
                        chunk: i,
                        tick,
                        pose: frame_of_pose(kp),
                        velocity: (lin + ang.cross(at - cp.0)).to_array(),
                        angular: ang.to_array(),
                    },
                    v * s.mat.density,
                ));
                continue;
            }
            let ke = spawn_chunk(ctx, l, k, p, i, 2, &kp, Role::Root, 0, tick)?;
            body = if body == 0 { ke } else { body };
            out.made.insert(ke, (lin + ang.cross(kp.0 - cp.0), ang));
            out.fresh_bodies.push(ke);
        }
        out.events.push(l.fractured.1.enc(
            None,
            Some(&json!({
                "entity": k as f64, "body": body as f64, "chunks": count - fine,
                "point": jv(cp.0), "volume": volume,
                "mass": volume * s.mat.density, "cause": CAUSE_SHATTER,
            })),
        )?);
        if s.break_fx != 0 {
            out.bursts.push((s.break_fx, cp.0));
        }
        if fine > 0 {
            out.bursts.push((s.debris, cp.0));
        }
    }
    Ok(())
}

/// The fine pieces and grit broken off join the state; the bodies made
/// this tick take their velocity.
pub(super) fn commit_fresh(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    st: &mut State,
    out: &mut Broken,
) -> og_core::Result<()> {
    st.debris.extend(out.debris.drain(..).map(|(d, _)| d));
    st.crumbs.extend(out.crumbs.drain(..).map(|(c, _)| c));
    for &b in &out.fresh_bodies {
        let (lin, ang) = out.made[&b];
        let vb = l.velocity.1.enc(
            None,
            Some(&json!({ "linear": jv(lin), "angular": jv(ang) })),
        )?;
        ctx.insert_bytes(Entity::from_bits(b), l.velocity.0, &vb)?;
    }
    Ok(())
}

/// Budgets: loose islands (a body and its children) by when they broke
/// loose; debris past its lifetime, then the oldest past maxChunks; fine
/// pieces and grit after debrisLifetime, with their prop, and the oldest
/// past what the renderer holds.
pub(super) fn enforce_budgets(
    ctx: &mut SystemContext<'_>,
    st: &mut State,
    assets: &Assets,
    t: &Clock,
) -> og_core::Result<()> {
    let tick = t.tick;
    let max = t.res.max_chunks as usize;

    let lifetime = t.res.debris_lifetime;
    let order = loose_islands(st, assets);
    let mut total: usize = order.iter().map(|x| x.chunks.len()).sum();
    let life = t.ticks_of(lifetime);
    for isl in &order {
        let old =
            lifetime > 0.0 && isl.volume < t.debris_volume && tick >= isl.tick.saturating_add(life);
        if !old && total <= max {
            continue;
        }
        total -= isl.chunks.len();
        let p = st.props.get_mut(&isl.prop).expect("listed");
        for &c in &isl.chunks {
            let e = p.entities[c];
            if alive(ctx, e) {
                ctx.despawn(Entity::from_bits(e))?;
            }
            p.entities[c] = 0;
            if c < p.body.len() {
                p.body[c] = 0;
            }
            if let Some(sh) = p.inst.shattered.get_mut(c) {
                *sh = true;
            }
        }
    }
    st.trim_pieces(
        tick,
        (lifetime > 0.0).then_some(life),
        og_render::MAX_DEBRIS,
    );
    Ok(())
}
