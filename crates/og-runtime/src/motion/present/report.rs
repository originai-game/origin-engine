//! What og.motion reports of its presentation: an agent's state, the
//! backend, the capabilities and the dump.

use super::*;

impl Presenter {
    pub(crate) fn presentation(&self, entity: u64) -> Value {
        let (state, reason) = match &self.model {
            Model::Unloaded => ("loading", "the model is read at the next frame"),
            Model::Waiting(why) | Model::Loading(why) => ("loading", why.as_str()),
            Model::Failed(why) => ("failed", why.as_str()),
            Model::Ready(r) => {
                let failure = self.agents.get(&entity).and_then(|s| s.failure.as_ref());
                if let Some(why) = failure.or(r.failure.as_ref()) {
                    ("failed", why.as_str())
                } else if self.generated.contains(&entity) {
                    ("ready", "")
                } else {
                    ("warming", "waiting for generated frames and their palette")
                }
            }
        };
        json!({ "state": state, "reason": reason })
    }
}

pub(crate) fn presentation(eng: &Engine, entity: u64) -> Value {
    eng.render.borrow().motion.presentation(entity)
}

/// og.motion.setBackend: `gpu` for the opt-in. A change drops the loaded
/// model, which loads again on the other backend, and every agent starts
/// again.
pub(crate) fn set_backend(eng: &Engine, gpu: bool) {
    let mut rs = eng.render.borrow_mut();
    let p = &mut rs.motion;
    if p.cpu == !gpu {
        return;
    }
    p.cpu = !gpu;
    if matches!(p.model, Model::Ready(_) | Model::Failed(_)) {
        p.model = Model::Unloaded;
        p.agents.clear();
        p.palettes.clear();
        p.previous = None;
        p.generated.clear();
    }
}

/// The backend chosen: "gpu" or "cpu".
#[cfg(not(feature = "publish"))]
pub(crate) fn backend(eng: &Engine) -> &'static str {
    if eng.render.borrow().motion.cpu {
        "cpu"
    } else {
        "gpu"
    }
}

/// A system read og.motion's presentation on the GPU backend: one warning
/// in the engine's log, the first time.
#[cfg(not(feature = "publish"))]
pub(crate) fn warn_read(eng: &Engine) {
    let tick = eng.world().map_or(0, |w| w.time().tick);
    let first = {
        let mut rs = eng.render.borrow_mut();
        !std::mem::replace(&mut rs.motion.warned_read, true)
    };
    if first {
        eng.state.borrow_mut().log.push(json!({
            "tick": tick,
            "warning": true,
            "text": "og.motion.presentation was read inside a system on the gpu backend: game logic must not read og.motion's presentation or poses (they arrive when the GPU answers and differ run to run); a game that needs poses in its simulation sets og.motion.setBackend(world, \"cpu\")",
        }));
    }
}

/// og.motion.capabilities.
pub(crate) fn capabilities(eng: &Engine) -> Value {
    let mut rs = eng.render.borrow_mut();
    let p = &mut rs.motion;
    p.wanted = true;
    let (backend, reason) = match &p.model {
        Model::Ready(r) => (r.backend(), String::new()),
        Model::Unloaded => (
            "none",
            "loading: the model is read at the next frame".into(),
        ),
        Model::Waiting(why) | Model::Loading(why) => ("none", why.clone()),
        Model::Failed(why) => ("none", why.clone()),
    };
    json!({ "backend": backend, "reason": reason, "modelBytes": p.model_bytes as f64 })
}

/// The presenter's side of `dump_motion`.
pub(crate) fn dump(eng: &Engine) -> Value {
    let rs = eng.render.borrow();
    let p = &rs.motion;
    let (state, reason) = match &p.model {
        Model::Ready(r) => (r.backend(), String::new()),
        Model::Unloaded => ("unloaded", String::new()),
        Model::Waiting(w) | Model::Loading(w) => ("waiting", w.clone()),
        Model::Failed(f) => ("failed", f.clone()),
    };
    let ready = match &p.model {
        Model::Ready(r) => Some(r),
        _ => None,
    };
    let agents: Vec<Value> = p
        .agents
        .iter()
        .map(|(e, s)| {
            json!({
                "entity": *e as f64,
                "presentation": p.presentation(*e),
                "ahead": ready.map_or(0, |r| s.ahead(&r.motion)),
                "modelGoals": ready.map_or(0, |r| r.motion.goals(s.character)),
                "framesShown": s.shown,
                "framesStarved": s.starved,
                "framesWarmup": s.warmup,
                "facing": s.facing,
                "footing": s.footing.json(),
                "plantedSlide": {
                    "frames": s.slide.1,
                    "meanMm": if s.slide.1 > 0 { 1000.0 * s.slide.0 / s.slide.1 as f32 } else { 0.0 },
                    "maxMm": 1000.0 * s.slide.2,
                },
                "rig": s.target.as_ref().map(|(_, m)| m.scheme.clone()),
                "rigMap": s.rig as f64,
                "bones": s.target.as_ref().map(|(asset, m)| m.bones(asset)),
                // The drawn body's lean from upright (hips to head).
                "leanDeg": s.drawn_lean.map(|l| l.to_degrees()),
                "sourceLeanDeg": s.lean.map(|l| l.to_degrees()),
                "feetAheadM": s.feet_ahead,
                // The drawn frame: the pose its model goal asked for, and
                // how far behind the frame clock it is (s).
                "drawnPose": s.next.and_then(|f| s.issued(&f)).map(|g| POSES[g.pose]),
                "lag": s.next.and_then(|f| s.issued(&f).map(|g| p.clock.unwrap_or(0.0) - (g.start + f64::from(f.index) / f64::from(RATE)))),
                "drawn": p.palettes.contains_key(e),
                // A jump's clip: its generated frames, the one drawn, its
                // own flight by its contacts.
                "jumpClip": s.clip.as_ref().map(|c| json!({
                    "frames": c.frames.len(),
                    "drawn": c.drawn,
                    "takeoff": c.flight.map(|f| f.takeoff),
                    "touchdown": c.flight.and_then(|f| f.touchdown),
                })),
                // The drawn frame's lower ankle over its standing height:
                // 0 on the ground, the generated flight's lift in the air.
                "footLiftM": ready.zip(s.prev).map(|(r, f)| {
                    f.pose.positions[LEFT_ANKLE][1].min(f.pose.positions[RIGHT_ANKLE][1]) - r.clearance
                }),
            })
        })
        .collect();
    json!({
        "state": state,
        "reason": reason,
        "modelBytes": p.model_bytes as f64,
        "poses": POSES,
        "inFlight": ready.map_or(0, |r| r.flights.len()),
        "batches": p.batches,
        // [taken, due] ticks of the last batches: their schedule, the
        // same however late their results came back.
        "taken": p.taken,
        // How far the Presenter is behind the world now, the ticks the
        // world ran while it was behind and the most it was: a result
        // that came back after its tick (no tick waits for one).
        "late": { "behind": p.behind.len(), "ticks": p.late.0, "most": p.late.1 },
        "lastBatch": { "characters": p.last_batch.0, "ms": p.last_batch.1 },
        "frameBudgetMs": FRAME_BUDGET_MS,
        "startup": ready.map(|r| &r.startup),
        "gpuGflops": ready.map_or(0.0, |r| r.rate),
        "cpuTicks": ready.map(|r| r.cpu_ticks.json()),
        "generations": ready.map_or(0, |r| r.generations),
        "errors": p.errors,
        "agents": agents,
    })
}
