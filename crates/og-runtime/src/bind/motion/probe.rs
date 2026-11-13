//! The motion dump and its probe commands.

use super::*;

/// The probe's `dump_motion`: every agent's goals (the simulation's) and
/// what the model is doing for it (the presenter's: frames generated
/// ahead, the batches, their timings), settled as a capture is (natively
/// every result due by the tick waited for; on the web as far as has come
/// back).
pub(crate) fn dump(engine_id: u32) -> Value {
    let eng = match engine(engine_id) {
        Ok(eng) => eng,
        Err(e) => return json!({ "error": e }),
    };
    present::settle(&eng);
    let goals = lock(&shared(&eng)).clone();
    let entities = eng
        .world()
        .ok()
        .and_then(|world| {
            let state = eng.state.borrow();
            let layout = state.layouts.get(AGENT)?;
            Some(crate::present::extract::rows(world, layout))
        })
        .unwrap_or_default();
    let agents: Vec<Value> = entities
        .into_iter()
        .map(|(e, _)| {
            let a = goals.agents.get(&e).cloned().unwrap_or_default();
            json!({
                "entity": e as f64,
                "presentation": present::presentation(&eng, e),
                "goal": a.current.map_or("none", |c| KIND_NAMES[c.goal.kind as usize]),
                "elapsed": a.current.map_or(0.0, |c| c.elapsed),
                "queued": a.queue.len(),
                "completed": a.completed,
                "hold": POSES[a.hold as usize],
                // A jump's state: its stage, seconds in the air and the
                // predicted seconds to its landing.
                "jump": a.current.and_then(|c| c.jump).map(|j| json!({
                    "stage": (["prepare", "air", "landed"])[j.stage as usize],
                    "air": j.air,
                    "landingIn": j.eta,
                })),
            })
        })
        .collect();
    let model = present::dump(&eng);
    json!({ "agents": agents, "model": model })
}

pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine: u32) {
    let _ = probe.register_command(
        "motion_backend",
        og_probe::CommandDesc {
            usage: "motion_backend:<cpu|gpu>".into(),
            doc: "og.motion.setBackend from the probe: gpu (the default, the same poses on one host and GPU) or cpu (the opt-in, the same poses on every host)".into(),
        },
        move |_world, args| {
            let eng = crate::engine::engine(engine)?;
            match args.first().map(String::as_str) {
                Some("cpu") => present::set_backend(&eng, false),
                Some("gpu") => present::set_backend(&eng, true),
                _ => return Err("write motion_backend:cpu or motion_backend:gpu".into()),
            }
            Ok(())
        },
    );
    let _ = probe.register_dump(
        "dump_motion",
        og_probe::CommandDesc {
            usage: "dump_motion[:path]".into(),
            doc: "og.motion: every agent's goals as the simulation keeps them, and the motion model's side (presentation only): whether its weights are resident and on which backend, each agent's frames generated ahead and shown, the batches run and their GPU times".into(),
        },
        0,
        move |_world, _args| Ok(dump(engine)),
    );
}
