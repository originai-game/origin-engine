use super::*;

// ---- the system ------------------------------------------------------------------

/// A controller's component after its move.
pub(crate) enum ControlOut {
    Character(CharacterController),
    /// Only a 2D world moves a platformer controller.
    Platformer(PlatformerController),
}

/// What sleep made of the tick: the `og.RigidBody` values to write (their
/// `sleeping` changed) and the bodies that fell asleep and woke, each in
/// entity order.
#[derive(Default)]
pub(crate) struct SleepOut {
    pub(crate) writes: Vec<(Entity, RigidBody)>,
    pub(crate) slept: Vec<u64>,
    pub(crate) woke: Vec<u64>,
}

/// What one step produced.
#[derive(Default)]
pub(crate) struct StepOut {
    /// Per `Scene::bodies` index: the new pose of dynamic bodies (with
    /// their velocity) and of controllers (without).
    pub moved: Vec<Option<(Vec3, Quat, Option<Velocity>)>>,
    /// Every touching pair, sorted by (a, b), with the event it starts as
    /// (point, normal from a to b and the hit; zero for sensor overlaps).
    pub touching: Vec<(Contact, CollisionStarted)>,
    /// Solid pairs that collided during an earlier substep of this tick and
    /// separated before the final substep (emits Started + Ended when not
    /// already touching on the prior tick).
    pub transient_started: Vec<CollisionStarted>,
    /// The components a step writes back.
    pub controls: Vec<(Entity, ControlOut)>,
    /// Dynamic bodies receiving virtual-controller impulses this tick.
    pub controller_impulses: Vec<u64>,
    pub joints: Vec<(Entity, Joint)>,
    /// The joints that broke, in the order they did.
    pub broken: Vec<JointBroken>,
    pub vehicles: Vec<(Entity, Vehicle)>,
    pub ragdolls: Vec<(Entity, Ragdoll)>,
    pub events: Vec<CharacterEvent>,
    pub splashes: Vec<Splash>,
    /// The soft volumes' written fields.
    pub softs: Vec<(Entity, SoftBody)>,
    /// The floating bodies' written fields (og.Buoyancy.submerged).
    pub floats: Vec<(Entity, Buoyancy)>,
    /// What sleep changed (og.Physics.sleepTicks).
    pub sleep: SleepOut,
    /// The step's phases, timed on the thread it ran on.
    pub laps: Option<stats::Laps>,
}

pub(super) fn run(ctx: &mut SystemContext<'_>, ids: &Ids, shared: &Mutex<Shared>) -> Result<()> {
    // The world's own lanes setting, for the whole tick: its gather, step
    // and write-back, and whatever reaches the pool from them.
    let lanes = lock(shared).lanes;
    og_lanes::with_enabled(lanes.on, || {
        crate::lanes::with_grains(lanes.grains, || run_here(ctx, ids, shared))
    })
}

fn run_here(ctx: &mut SystemContext<'_>, ids: &Ids, shared: &Mutex<Shared>) -> Result<()> {
    let settings = *ctx.resource::<Physics>()?;
    // Slow motion steps by the scaled tick; a hit stop (scaled dt 0) holds
    // the simulation as it is and carries the touching pairs over, so no
    // contact ends or starts while time stands still.
    let dt = ctx.scaled_dt();
    if dt <= 0.0 {
        for c in ctx.events::<Contact>()? {
            ctx.send(c)?;
        }
        // A force acts over a step: none passes. Impulses wait for one.
        lock(shared)
            .pushes
            .retain(|p| matches!(p.kind, PushKind::Impulse | PushKind::TorqueImpulse));
        return Ok(());
    }
    let started = stats::now_ms();
    let mut laps = stats::Laps::start();
    let lazy = gather::LazyIds::of(|n| ctx.id_by_name(n).ok());
    let frames = ctx.parent_frames()?;
    let mut scene: Scene = gather::gather(
        &|i| ctx.query(i),
        &|e, id| ctx.get_bytes(e, id),
        lazy,
        ids,
        &mut ids.kept(),
        &frames,
        Some(&|d| ctx.query_with(d)),
    )?;
    if let Some(soft) = lazy.soft {
        scene.softs = gather::softs(
            &|d| ctx.query_with(d),
            &|e, id| ctx.get_bytes(e, id),
            soft,
            ids,
            &frames,
        )?;
    }
    if let Some(garment) = lazy.garment {
        scene.garments = gather::garments(
            &|d| ctx.query_with(d),
            &|e, id| ctx.get_bytes(e, id),
            garment,
        )?;
    }
    if let Some(buoyancy) = lazy.buoyancy {
        scene.floats = gather::floats(
            &|d| ctx.query_with(d),
            &|e, id| ctx.get_bytes(e, id),
            buoyancy,
        )?;
    }
    // The world's clock as the sea's (og.physics.waterHeight's, the renderer's): its tick over its rate.
    scene.seconds = ctx.time().elapsed();
    laps.lap(stats::Phase::Gather);
    let tick = ctx.time().tick;
    let threads = og_lanes::threads();
    // The simulation is taken out for the step and put back only when it
    // succeeds: a failed tick leaves none, and the next one rebuilds it.
    let out = {
        let mut shared = lock(shared);
        // A world with nothing to simulate, and no simulation yet, builds
        // none: a game without physics never runs rapier, so its code stays
        // in the physics modules (docs/web-export.md §Modules). Its first
        // body builds the simulation as any first step does.
        if scene.is_empty() && shared.sim3.is_none() && shared.sim2.is_none() {
            shared.pushes.clear();
            return Ok(());
        }
        match settings.profile {
            PROFILE_3D => {
                shared.sim2 = None;
                // Stepped: the restored bytes are behind the simulation now.
                shared.restored = None;
                let sim = shared.sim3.take().unwrap_or_default();
                let out = dim3::step(&settings, dt, tick, &scene, &mut shared, &mut lock(&sim))?;
                shared.sim3 = Some(sim);
                out
            }
            PROFILE_2D => {
                shared.sim3 = None;
                let sim = shared.sim2.take().unwrap_or_default();
                let out = dim2::step(&settings, dt, tick, &scene, &mut shared, &mut lock(&sim))?;
                shared.sim2 = Some(sim);
                out
            }
            other => return Err(check_profile(other).unwrap_err()),
        }
    };
    laps.skip();
    gather::write_back(ctx, ids, &scene, &out)?;
    for (e, c) in &out.controls {
        match c {
            ControlOut::Character(c) => ctx.set(*e, *c)?,
            ControlOut::Platformer(p) => ctx.set(*e, *p)?,
        }
    }
    for (e, j) in &out.joints {
        ctx.set(*e, *j)?;
    }
    for (e, v) in &out.vehicles {
        ctx.set(*e, *v)?;
    }
    for (e, r) in &out.ragdolls {
        ctx.set(*e, *r)?;
    }
    for ev in &out.events {
        ctx.send(*ev)?;
    }
    for s in &out.splashes {
        ctx.send(*s)?;
    }
    for (e, s) in &out.softs {
        ctx.set(*e, *s)?;
    }
    for (e, b) in &out.floats {
        ctx.set(*e, *b)?;
    }
    for (e, b) in &out.sleep.writes {
        ctx.set(*e, *b)?;
    }
    // G2-3's events are registered only once a game names them (LAZY):
    // a world that does not keeps its type table, and nothing could read
    // them, so they are not sent.
    if ctx.id_by_name(JointBroken::NAME).is_ok() {
        for b in &out.broken {
            ctx.send(*b)?;
        }
    }
    if ctx.id_by_name(BodySlept::NAME).is_ok() {
        for &entity in &out.sleep.slept {
            ctx.send(BodySlept { entity })?;
        }
    }
    if ctx.id_by_name(BodyWoke::NAME).is_ok() {
        for &entity in &out.sleep.woke {
            ctx.send(BodyWoke { entity })?;
        }
    }

    // Started/Ended from the pairs the previous step carried over.
    let mut previous = ctx.events::<Contact>()?;
    og_core::sort::sort(&mut previous);
    previous.dedup();
    let current: Vec<Contact> = out.touching.iter().map(|t| t.0).collect();
    let key = |c: &Contact| (c.a, c.b, c.sensor != 0);
    let (mut i, mut j) = (0, 0);
    loop {
        match (previous.get(i), current.get(j)) {
            (None, None) => break,
            (Some(p), Some(c)) if key(p) == key(c) => {
                if p.sensor != c.sensor {
                    // Only emit Exited/Entered for the sensor bits that actually flipped.
                    let exited = p.sensor & !c.sensor;
                    let entered = c.sensor & !p.sensor;
                    if exited != 0 {
                        sensor_events(
                            ctx,
                            &Contact {
                                sensor: exited,
                                ..*p
                            },
                            false,
                        )?;
                    }
                    if entered != 0 {
                        sensor_events(
                            ctx,
                            &Contact {
                                sensor: entered,
                                ..*c
                            },
                            true,
                        )?;
                    }
                }
                i += 1;
                j += 1;
            }
            (Some(p), c) if c.is_none_or(|c| key(p) < key(c)) => {
                if p.sensor == 0 {
                    ctx.send(CollisionEnded { a: p.a, b: p.b })?;
                } else {
                    sensor_events(ctx, p, false)?;
                }
                i += 1;
            }
            (_, Some(c)) => {
                if c.sensor == 0 {
                    ctx.send(out.touching[j].1)?;
                } else {
                    sensor_events(ctx, c, true)?;
                }
                j += 1;
            }
            (Some(_), None) => unreachable!("handled by the arm above"),
        }
    }
    // Transient solid hits that occurred on an earlier substep and separated
    // before tick end: if not already touching on the prior tick, emit both
    // Started and Ended so fast impacts are never lost.
    for cs in &out.transient_started {
        let probe = Contact {
            a: cs.a,
            b: cs.b,
            sensor: 0,
        };
        if previous.binary_search(&probe).is_err() {
            ctx.send(*cs)?;
            ctx.send(CollisionEnded { a: cs.a, b: cs.b })?;
        }
    }
    for c in current {
        ctx.send(c)?;
    }
    laps.lap(stats::Phase::WriteBack);
    if let Some(step) = &out.laps {
        laps.merge(step);
    }
    let pool = laps.phases[stats::Phase::Rapier as usize].split;
    let mut shared = lock(shared);
    let bodies = match () {
        _ if shared.sim3.is_some() => shared
            .sim3
            .as_deref()
            .map_or(0, |s| lock(s).world.bodies.len()),
        _ if shared.sim2.is_some() => shared
            .sim2
            .as_deref()
            .map_or(0, |s| lock(s).world.bodies.len()),
        _ => 0,
    };
    shared.stats = PhysicsStats {
        tick,
        bodies: bodies as u32,
        threads,
        pool,
        ms: (stats::now_ms() - started).max(0.0),
        phases: laps.phases.to_vec(),
    };
    Ok(())
}

/// What is inside `sensor` as one step's pairs (its og.Contact events)
/// have it: every entity overlapping it, each once, in entity order. A pair
/// counts only on the side its `sensor` bits name, so a solid touching a
/// sensor's entity is not in it; empty for an entity that is no sensor.
pub fn sensor_contents(contacts: &[Contact], sensor: u64) -> Vec<u64> {
    let mut inside: Vec<u64> = contacts
        .iter()
        .filter_map(|c| match () {
            _ if c.a == sensor && c.sensor & 1 != 0 => Some(c.b),
            _ if c.b == sensor && c.sensor & 2 != 0 => Some(c.a),
            _ => None,
        })
        .collect();
    og_core::sort::sort(&mut inside);
    inside.dedup();
    inside
}

fn sensor_events(ctx: &SystemContext<'_>, c: &Contact, entered: bool) -> Result<()> {
    let mut pairs = Vec::with_capacity(2);
    if c.sensor & 1 != 0 {
        pairs.push((c.a, c.b));
    }
    if c.sensor & 2 != 0 {
        pairs.push((c.b, c.a));
    }
    for (sensor, other) in pairs {
        if entered {
            ctx.send(SensorEntered { sensor, other })?;
        } else {
            ctx.send(SensorExited { sensor, other })?;
        }
    }
    Ok(())
}
