//! The system's run: the tick's clock, the props surveyed, the chunks lost
//! and what the block publishes.

use super::*;

/// A chunk's place in its island's body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Role {
    Static,
    Root,
    /// A compound child of the chunk named.
    Child(u32),
}

/// The og.destruction system, one tick, as its stages in order: bursts
/// expire; the props are surveyed (new ones placed, gone ones forgotten);
/// chunks the game despawned leave their graph; the chunks' poses are
/// read; the hits are gathered (impacts, then the calls); each prop
/// breaks; the blasts and the blows push what broke loose; the bodies made
/// take their velocity; the budgets despawn the oldest; and what the game
/// reads is written (the destructibles' fields, og.Fractured, the bursts).
pub(super) fn run(ctx: &mut SystemContext<'_>, m: &ModuleState, l: &Ls) -> og_core::Result<()> {
    let t = Clock::read(ctx, l)?;
    let assets = lock(&m.assets).clone();
    let mut guard = lock(&m.state);
    let st = &mut *guard;

    expire_bursts(ctx, st, t.tick);
    let settings = survey(ctx, l, st, &assets)?;
    drop_lost_chunks(ctx, st)?;
    let poses = chunk_poses(ctx, l, st, &assets);
    let Gathered {
        mut hits,
        bursts,
        strikes,
        explosions,
    } = gather_hits(ctx, l, st, &assets, &settings)?;

    let mut out = Broken {
        bursts,
        ..Broken::default()
    };
    let mut stress_budget = match t.res.stress_budget {
        0 => u64::MAX,
        n => u64::from(n),
    };
    let keys: Vec<u64> = st.props.keys().copied().collect();
    for k in keys {
        let p = st.props.get_mut(&k).expect("listed");
        let a = assets[&p.asset].clone();
        let at = Breaking {
            key: k,
            a: &a,
            s: &settings[&k],
            pose: &poses[&k],
        };
        let h = hits.remove(&k).unwrap_or_default();
        break_prop(ctx, m, l, &t, at, p, h, &mut stress_budget, &mut out)?;
    }

    push_blasted_bodies(
        ctx,
        l,
        st,
        &assets,
        &settings,
        &poses,
        &explosions,
        &mut out,
    )?;
    push_blasted_pieces(&settings, &explosions, &mut out.debris, &mut out.crumbs);
    push_struck(ctx, l, st, &assets, &settings, &poses, &strikes, &mut out);
    commit_fresh(ctx, l, st, &mut out)?;
    enforce_budgets(ctx, st, &assets, &t)?;
    publish(ctx, l, st, &t, out.events, out.bursts)
}

/// What the tick reads once: its clock, og.Destruction and the gravity.
pub(super) struct Clock {
    pub(super) tick: u64,
    pub(super) dt: f32,
    /// og.Destruction, read.
    pub(super) res: record::Destruction,
    pub(super) debris_volume: f32,
    pub(super) gravity: Vec3,
}

impl Clock {
    pub(super) fn read(ctx: &SystemContext<'_>, l: &Ls) -> og_core::Result<Clock> {
        let time = ctx.time();
        let res: record::Destruction = row(&l.resource.1, &ctx.resource_bytes(l.resource.0)?)?;
        let physics: record::Physics = row(&l.physics.1, &ctx.resource_bytes(l.physics.0)?)?;
        Ok(Clock {
            tick: time.tick,
            dt: time.dt.max(1e-6),
            debris_volume: res.debris_volume,
            gravity: physics.gravity,
            res,
        })
    }

    /// The ticks `s` seconds take, rounded up.
    pub(super) fn ticks_of(&self, s: f32) -> u64 {
        (s.max(0.0) / self.dt).ceil() as u64
    }
}

pub(super) fn alive(ctx: &SystemContext<'_>, b: u64) -> bool {
    b != 0 && ctx.is_alive(Entity::from_bits(b))
}

/// Bursts past their lifetime.
pub(super) fn expire_bursts(ctx: &mut SystemContext<'_>, st: &mut State, tick: u64) {
    let mut fx = std::mem::take(&mut st.fx);
    fx.retain(|&(e, at)| {
        if tick < at {
            return true;
        }
        if alive(ctx, e) {
            let _ = ctx.despawn(Entity::from_bits(e));
        }
        false
    });
    st.fx = fx;
}

/// The destructibles and their settings; props whose entity lost its
/// og.Destructible (or died) are forgotten, their static chunks with them,
/// and a new one is placed (a standing prop's chunks are its colliders
/// from the start).
pub(super) fn survey(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    st: &mut State,
    assets: &Assets,
) -> og_core::Result<BTreeMap<u64, Settings>> {
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let mut settings: BTreeMap<u64, Settings> = BTreeMap::new();
    let mut current: BTreeMap<u64, (u64, Option<Pose>, bool)> = BTreeMap::new();
    for &e in &rows {
        let d: record::Destructible = row(&l.destructible.1, &ctx.get_bytes(e, l.destructible.0)?)?;
        let Ok(mb) = ctx.get_bytes(e, l.model.0) else {
            continue;
        };
        let asset = row::<record::Model>(&l.model.1, &mb)?.asset;
        // A prop that moves (a dynamic or kinematic body) breaks free whole.
        // og.RigidBody.kind is an enum, which the hand read this replaced
        // always took as 0 (dynamic): any body counts as moving, a fixed
        // one too (kept as it was).
        let moving = ctx.get_bytes(e, l.rigid.0).is_ok();
        settings.insert(e.to_bits(), settings_of(&d));
        current.insert(e.to_bits(), (asset, world_pose(ctx, l, e), moving));
    }
    let gone: Vec<u64> = st
        .props
        .iter()
        .filter(|(k, p)| {
            current
                .get(k)
                .is_none_or(|c| c.0 != p.asset || !assets.contains_key(&c.0))
        })
        .map(|(k, _)| *k)
        .collect();
    for k in gone {
        let p = st.props.remove(&k).expect("listed");
        for (i, &e) in p.entities.iter().enumerate() {
            if p.body[i] == 0 && alive(ctx, e) {
                ctx.despawn(Entity::from_bits(e))?;
            }
        }
    }
    for (&k, &(asset, pose, moving)) in &current {
        let Some(a) = assets.get(&asset) else {
            continue;
        };
        let Some(pose) = pose else {
            continue;
        };
        let s = &settings[&k];
        let p = st.props.entry(k).or_insert_with(|| Prop {
            asset,
            inst: Instance::new(a),
            frame: frame_of_pose(pose),
            entities: vec![0; a.chunks.len()],
            body: vec![0; a.chunks.len()],
            loose: vec![u64::MAX; a.chunks.len()],
            collider: None,
            placed: false,
            anchors: false,
            fractured: false,
        });
        if !p.placed {
            p.frame = frame_of_pose(pose);
            p.anchors = !moving && s.anchored;
            // A standing prop's chunks are its colliders from the start.
            if !moving {
                place(ctx, l, k, p, a)?;
            }
        }
    }
    Ok(settings)
}

/// Chunks despawned by the game or a budget: gone from the graph; a body
/// takes its compound children with it.
pub(super) fn drop_lost_chunks(ctx: &mut SystemContext<'_>, st: &mut State) -> og_core::Result<()> {
    for p in st.props.values_mut() {
        let dead: Vec<u64> = p
            .entities
            .iter()
            .copied()
            .filter(|&e| e != 0 && !alive(ctx, e))
            .collect();
        if dead.is_empty() {
            continue;
        }
        for (i, e) in p.entities.iter_mut().enumerate() {
            let lost = dead.contains(e) || (p.body[i] != 0 && dead.contains(&p.body[i]));
            if *e != 0 && lost {
                if alive(ctx, *e) {
                    ctx.despawn(Entity::from_bits(*e))?;
                }
                *e = 0;
                p.body[i] = 0;
                if let Some(sh) = p.inst.shattered.get_mut(i) {
                    *sh = true;
                }
                p.inst.dirty = true;
            }
        }
    }
    Ok(())
}

/// Each prop's chunks where they stand now, by prop.
pub(super) type Poses = BTreeMap<u64, Vec<Option<Pose>>>;

pub(super) fn chunk_poses(ctx: &SystemContext<'_>, l: &Ls, st: &State, assets: &Assets) -> Poses {
    let mut poses: Poses = BTreeMap::new();
    for (&k, p) in &st.props {
        let a = &assets[&p.asset];
        let frame = pose_of_frame(&p.frame);
        let list = (0..a.chunks.len())
            .map(|i| {
                let e = p.entities[i];
                if e != 0 {
                    return world_pose(ctx, l, Entity::from_bits(e));
                }
                // Not spawned: where the model places it.
                let c = a.chunks[i].centroid;
                Some((place_on(&frame, Vec3::ZERO, c), frame.1, frame.2))
            })
            .collect();
        poses.insert(k, list);
    }
    poses
}

/// What the game reads: each destructible's written fields, og.Fractured,
/// and breakFx and hitFx as one-shot bursts, despawned after fxLifetime.
pub(super) fn publish(
    ctx: &mut SystemContext<'_>,
    l: &Ls,
    st: &mut State,
    t: &Clock,
    events: Vec<Vec<u8>>,
    bursts: Vec<(u64, Vec3)>,
) -> og_core::Result<()> {
    let tick = t.tick;
    for (&k, p) in &st.props {
        let e = Entity::from_bits(k);
        let Ok(b) = ctx.get_bytes(e, l.destructible.0) else {
            continue;
        };
        let mut d = l.destructible.1.decode(&b);
        let broken = p.inst.bonds.iter().filter(|&&h| h <= 0.0).count();
        let n = (0..p.entities.len())
            .filter(|&c| p.entities[c] != 0 && p.loose[c] != u64::MAX)
            .count();
        d["fractured"] = json!(p.fractured);
        d["broken"] = json!(broken);
        d["loose"] = json!(n);
        let out = l.destructible.1.enc(Some(&b), Some(&d))?;
        if out != b {
            ctx.set_bytes(e, l.destructible.0, &out)?;
        }
    }
    for ev in events {
        ctx.send_bytes(l.fractured.0, &ev)?;
    }
    let fx_life = t.ticks_of(t.res.fx_lifetime).max(1);
    for (i, (emitter, at)) in bursts.into_iter().take(MAX_BURSTS).enumerate() {
        let tr = l
            .transform
            .1
            .enc(None, Some(&json!({ "position": jv(at) })))?;
        let seed = (tick as u32).wrapping_mul(2_654_435_761) ^ i as u32;
        let ps = l.particles.1.enc(
            None,
            Some(&json!({ "emitter": emitter as f64, "seed": seed, "startTick": tick as u32 })),
        )?;
        let e = ctx.spawn_bytes(vec![(l.transform.0, tr), (l.particles.0, ps)])?;
        st.fx.push((e.to_bits(), tick + fx_life));
    }
    Ok(())
}
