//! The Presenter's tick: the inputs it keeps while behind the world, and
//! the step that asks for goals, runs batches and draws poses.

use super::*;

/// og.motion at the world's tick: from the probe's tick gate (before the
/// next tick) and before a frame is drawn. Takes the tick's [`Input`] and
/// steps every tick behind it can (generated motion, then authored
/// performance), stopping where a batch it needs has not come back: the
/// world goes on and a later call goes on from there, the results applied
/// at the ticks they were due as soon as they arrive. True: no tick waits
/// for the model; a frame meanwhile draws the poses of the last tick
/// stepped.
pub(crate) fn step(eng: &Engine) -> bool {
    catch_up(eng, false);
    true
}

/// [`step`], then, natively, every batch the ticks behind need waited for,
/// so the Presenter is at the world's tick: before a capture, a
/// presentation hash or a fork check, whose frames then reproduce run to
/// run. On the web, which cannot wait for the GPU, false while it is still
/// behind: ask again.
pub(crate) fn settle(eng: &Engine) -> bool {
    catch_up(eng, true)
}

/// Takes the world's tick and steps the ticks behind in order, each until
/// it needs what has not come (with `wait`, natively, waited for).
pub(crate) fn catch_up(eng: &Engine, wait: bool) -> bool {
    let Ok((tick, dt)) = eng.world().map(|w| (w.time().tick, w.time().dt)) else {
        return true;
    };
    if eng.render.borrow().backend.is_none() {
        return true;
    }
    take(eng, tick, dt);
    loop {
        let Some(input) = eng.render.borrow_mut().motion.behind.pop_front() else {
            return true;
        };
        if !step_tick(eng, &input, wait) {
            eng.render.borrow_mut().motion.behind.push_front(input);
            return false;
        }
    }
}

/// The world's tick as [`Input`], once a tick.
pub(crate) fn take(eng: &Engine, tick: u64, dt: f32) {
    if eng.render.borrow().motion.taken_tick == Some(tick) {
        return;
    }
    let agents = seen(eng);
    let goals = lock(&shared(eng)).clone();
    let sys = eng.state.borrow().anim.clone();
    // og.anim's pose where `fallback` may draw it: an agent looking or reacting.
    let poses = sys.map_or_else(BTreeMap::new, |sys| {
        agents
            .iter()
            .filter(|a| {
                goals
                    .agents
                    .get(&a.entity)
                    .is_some_and(|g| g.performance.looking() || g.performance.reacting())
            })
            .filter_map(|a| Some((a.entity, sys.pose(og_ecs::Entity::from_bits(a.entity))?)))
            .collect()
    });
    let mut rs = eng.render.borrow_mut();
    let p = &mut rs.motion;
    if !p.behind.is_empty() {
        p.late.0 += 1;
        p.late.1 = p.late.1.max(p.behind.len());
    }
    p.behind.push_back(Input {
        tick,
        dt,
        agents,
        goals,
        poses,
    });
    p.taken_tick = Some(tick);
}

/// One tick's step from its [`Input`]: false where it stops for a batch
/// that has not landed (or the weights), to go on from there.
pub(crate) fn step_tick(eng: &Engine, input: &Input, wait: bool) -> bool {
    let (tick, dt) = (input.tick, input.dt);
    let (agents, goals) = (&input.agents, &input.goals);
    let now = tick as f64 * f64::from(dt);
    let batch_ticks = if dt > 0.0 {
        (BATCH_SECONDS / dt).ceil() as u64
    } else {
        1
    };
    let due = tick + batch_ticks;
    let sys = eng.state.borrow().anim.clone();
    let mut rs = eng.render.borrow_mut();
    let rs = &mut *rs;
    let p = &mut rs.motion;
    // A held step goes on where it stopped; anything else is the tick's
    // first call.
    let resume = p.held.take().filter(|_| p.stepped == Some(tick));
    let elapsed = match resume {
        Some(h) => h.elapsed,
        None => {
            // What the tick waits for comes first, before anything of it
            // changes. An agent waits for the weights: the tick the model
            // starts at is then the agent's, never the load's wall time.
            let loadable =
                |m: &Model| matches!(m, Model::Unloaded | Model::Waiting(_) | Model::Loading(_));
            let loaded = !agents.is_empty() && loadable(&p.model);
            if loaded {
                p.model = load(p, eng, rs.backend.as_deref(), false);
                if matches!(p.model, Model::Loading(_)) {
                    if !wait || cfg!(target_arch = "wasm32") {
                        return false;
                    }
                    p.model = load(p, eng, rs.backend.as_deref(), true);
                }
            }
            // The batch runs a slice a tick; its frames are taken at its
            // tick, however soon it landed, and waited for then.
            if !agents.is_empty()
                && let Model::Ready(r) = &mut p.model
            {
                run_flights(r, tick, |_| false, false, &mut p.errors);
                if !settle_batches(r, tick, None, wait, &mut p.errors) {
                    return false;
                }
            }
            // The tick before's palettes, which display frames until the
            // next tick blend from.
            p.previous = p.stepped.map(|t| (t, std::mem::take(&mut p.palettes)));
            p.stepped = Some(tick);
            p.generated.clear();
            // The model loads as soon as the bundle declares it (a game
            // that uses og.motion), before any agent needs it.
            if agents.is_empty() && !p.wanted && !declares(eng) {
                p.agents.clear();
                return true;
            }
            if !loaded && loadable(&p.model) {
                p.model = load(p, eng, rs.backend.as_deref(), false);
            }
            if agents.is_empty() {
                p.agents.clear();
                return true;
            }
            let elapsed = p.clock.map_or(0.0, |c| (now - c).clamp(0.0, 0.25)) as f32;
            p.clock = Some(now);
            elapsed
        }
    };
    let Model::Ready(r) = &mut p.model else {
        fallback(eng, p, agents, goals, sys.as_ref(), &input.poses);
        return true;
    };
    let mut stats = (p.batches, p.last_batch, std::mem::take(&mut p.taken));
    if resume.is_none() {
        step_goals(
            eng,
            &mut p.agents,
            r,
            agents,
            goals,
            sys.as_ref(),
            (elapsed, now, tick, due),
            &mut stats,
            &mut p.errors,
        );
    }
    let first = resume.map_or(0, |h| h.agent);
    for (k, a) in agents.iter().enumerate().skip(first) {
        let s = p.agents.get_mut(&a.entity).expect("added above");
        // Resumed: this agent's clock has moved and its frames were taken;
        // the step stopped at its wait.
        let mut waiting = resume.filter(|h| h.agent == k).map(|h| h.tries);
        let mut tries = waiting.unwrap_or(0);
        if waiting.is_none() {
            // Frames played faster or slower than the model's rate are due
            // that much sooner or later: the goals' clocks move with them.
            let warp = s.warp_for(elapsed);
            for g in &mut s.issued {
                g.start += f64::from((1.0 - warp) * elapsed);
            }
            s.phase += elapsed * RATE * warp;
            if s.prev.is_none() {
                s.phase = 0.0;
            }
        }
        while tries < 3 {
            if waiting.take().is_none() {
                while s.clip.is_none() && (s.next.is_none() || s.phase >= 1.0) {
                    let Some(f) = s.pending.pop_front().or_else(|| r.motion.pop(s.character))
                    else {
                        break;
                    };
                    if s.prev.is_none() {
                        s.prev = Some(f);
                    } else if s.next.is_none() {
                        s.next = Some(f);
                    } else {
                        s.prev = s.next;
                        s.next = Some(f);
                        s.phase -= 1.0;
                    }
                }
                let starved = s.prev.is_none() || (s.next.is_none() || s.phase >= 1.0);
                // The generated motion is drawn, never a gap: an agent that
                // ran out waits for the batch outstanding. Until its first
                // frames land it is drawn as og.anim poses it: the fallback
                // covers the model's start, as its load.
                if !(starved && r.awaits(s.character) && s.prev.is_some()) {
                    break;
                }
            }
            if !settle_batches(r, tick, Some(s.character), wait, &mut p.errors) {
                p.held = Some(Held {
                    agent: k,
                    tries,
                    elapsed,
                });
                (p.batches, p.last_batch, p.taken) = stats;
                return false;
            }
            decode(r, &mut stats, tick, Some(s.character));
            start_flights(r, due);
            tries += 1;
        }
        if let Some(prev) = s.prev {
            // Goals before the drawn frame's are done with (ids ascend).
            while s.issued.front().is_some_and(|g| g.id < prev.tag) {
                s.issued.pop_front();
            }
        }
        jump_clip(&mut r.motion, s, goals.agents.get(&a.entity), elapsed);
        s.count_wait();
        if s.next.is_none() || s.phase >= 1.0 {
            s.phase = s.phase.min(1.0);
        }
        if let Some(sys) = sys.as_ref()
            && let Some(m) = palette(r, a, s, goals, sys, elapsed)
        {
            s.shown += 1;
            p.palettes.insert(a.entity, m);
            p.generated.insert(a.entity);
            if r.startup.first_palette_delay_ms.is_none() {
                r.startup.first_palette_delay_ms =
                    r.startup.inference_finished_at.map(|at| now_ms() - at);
            }
        }
    }
    start_flights(r, due);
    (p.batches, p.last_batch, p.taken) = stats;
    fallback(eng, p, agents, goals, sys.as_ref(), &input.poses);
    true
}

/// A tick's first call ([`step`]): the batch due now decoded, every agent
/// added or kept, the model's goals asked for, the next batch started.
#[allow(clippy::too_many_arguments)]
pub(crate) fn step_goals(
    eng: &Engine,
    shown: &mut BTreeMap<u64, Shown>,
    r: &mut Ready,
    agents: &[Seen],
    goals: &Goals,
    sys: Option<&og_anim::AnimationSystem>,
    (elapsed, now, tick, due): (f32, f64, u64, u64),
    stats: &mut Stats,
    errors: &mut Vec<String>,
) {
    decode(r, stats, tick, None);
    shown.retain(|e, _| agents.iter().any(|a| a.entity == *e));
    for a in agents {
        let s = shown.entry(a.entity).or_insert_with(|| {
            let idle = r.library.frames(0);
            let placed = Goal::placed(
                idle,
                [a.position.x, a.position.z],
                facing::heading(a.rotation),
            );
            let mut recent = [placed.targets[0]; 4];
            for (k, t) in placed.targets.iter().take(4).enumerate() {
                recent[k] = *t;
            }
            Shown {
                character: r.motion.add(recent),
                prev: None,
                next: None,
                phase: 0.0,
                serial: u32::MAX,
                requested: None,
                planter: Planter::new(Vec::new()),
                skeleton: 0,
                rig: u64::MAX,
                target: None,
                failure: None,
                last_position: a.position,
                velocity: Vec3::ZERO,
                shown: 0,
                starved: 0,
                warmup: 0,
                facing: 0.0,
                planted: [None; 2],
                slide: (0.0, 0, 0.0),
                lean: None,
                drawn_lean: None,
                warp: 1.0,
                feet: [Foot::default(); 2],
                offset: Vec3::ZERO,
                followed: None,
                footing: Footing::default(),
                feet_ahead: None,
                issued: VecDeque::new(),
                next_id: 0,
                clip: None,
                pending: VecDeque::new(),
            }
        });
        if s.skeleton != a.skeleton || s.rig != a.settings.rig {
            s.skeleton = a.skeleton;
            s.rig = a.settings.rig;
            s.failure = None;
            s.target = sys.and_then(|sys| {
                match crate::bind::anim::humanoid_of(eng, sys, a.skeleton, a.settings.rig, "agent")
                {
                    Ok(target) => Some(target),
                    Err(error) => {
                        s.failure = Some(error.clone());
                        errors.push(format!("agent {}: {error}", a.entity));
                        None
                    }
                }
            });
            let legs = s.target.as_ref().map_or(Vec::new(), |(t, m)| {
                let rest = t.model_space(&t.rest_pose());
                [(14usize, 0usize), (18, 2)]
                    .iter()
                    .filter(|(b, _)| (0..3).all(|k| m.nodes[b + k] != NONE))
                    .map(|&(b, contact)| Leg {
                        chain: [m.nodes[b], m.nodes[b + 1], m.nodes[b + 2]],
                        contact,
                        rest: rest[m.nodes[b + 2] as usize].translation.y,
                    })
                    .collect()
            });
            s.planter = Planter::new(legs);
        }
        if elapsed > 0.0 {
            s.velocity = (a.position - s.last_position) / elapsed;
        }
        s.last_position = a.position;
        let serial = goals.agents.get(&a.entity).map_or(0, |g| g.serial);
        // At least the agent's lookahead, and enough to cover two batches'
        // latency (sliced across frames, 0.5 to 0.6 s for 16 agents): one
        // batch runs at a time, so a goal asked for while another is on its
        // way waits for it, then for its own.
        let ahead = a.settings.lookahead.max(2.0 * BATCH_SECONDS);
        // On the CPU each agent's generation is its own batch: one playing
        // faster than the model's rate asks that much sooner.
        let lookahead = match r.compute {
            Compute::Cpu(_) => (ahead * RATE * s.warp.max(1.0)).ceil() as usize,
            Compute::Gpu { .. } => (ahead * RATE) as usize,
        };
        if serial != s.serial
            || (r.motion.goals(s.character) == 0 && s.ahead(&r.motion) < lookahead.max(1))
        {
            let queued = s.ahead(&r.motion) as f32 / (RATE * s.warp);
            let (at, h, pose, seconds) = predict(goals, a, s, queued);
            let mut goal = Goal::placed(r.library.frames(pose), [at.x, at.z], h);
            // A walk or run keeps time by its distance; forced short, the
            // model crawled into a run. A pose (jump, crouch, wave) has no
            // distance to keep time by: its length is the segment's.
            if !is_gait(pose) {
                goal.token_counts = token_counts(seconds);
            }
            if serial != s.serial {
                s.serial = serial;
                s.reconcile_request(&mut r.motion, &goal);
            }
            if r.motion.goals(s.character) != 0 || s.ahead(&r.motion) >= lookahead.max(1) {
                continue;
            }
            s.requested = Some(goal.clone());
            goal.tag = s.next_id;
            s.issued.push_back(Issued {
                id: s.next_id,
                pose,
                start: now + f64::from(queued),
            });
            s.next_id += 1;
            r.motion.push(s.character, goal);
        }
    }
    start_flights(r, due);
}
