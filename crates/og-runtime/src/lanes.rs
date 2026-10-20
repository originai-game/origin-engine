//! The probe's view of lanes (og-lanes): `lanes:<on|off>` switches them at
//! run time and `dump_lanes` reports what the engine got, so a page or a
//! test can compare a run with lanes on against one with them off.

use serde_json::json;

/// Registers `lanes:<on|off>` and `dump_lanes[:path]`. Lanes are one switch
/// for the process (every engine in it shares the pool).
pub(crate) fn register(probe: &mut og_probe::Probe, engine: u32) {
    let _ = probe.register_command(
        "lanes",
        og_probe::CommandDesc {
            usage: "lanes:<on|off>".into(),
            doc: "run the parallel systems on the lanes pool (on, the default) or on the engine's thread (off); results are the same either way".into(),
        },
        move |_world, args| match args.first().map(String::as_str) {
            Some("on") => {
                og_lanes::set_enabled(true);
                Ok(())
            }
            Some("off") => {
                og_lanes::set_enabled(false);
                Ok(())
            }
            _ => Err("write lanes:on or lanes:off".into()),
        },
    );
    let _ = probe.register_dump(
        "dump_lanes",
        og_probe::CommandDesc {
            usage: "dump_lanes[:path]".into(),
            doc: "the lanes the engine got: on, threads, the pool (native, web-workers or none), why they are serial, the parallel calls and items so far (in all and per site: anim, extract, cull, physics), and the last step and render times".into(),
        },
        0,
        move |_world, _args| {
            let eng = super::engine::engine(engine)?;
            let st = eng.state.borrow();
            let mut out = report();
            out["stepMs"] = json!((st.step_ms * 1000.0).round() / 1000.0);
            out["renderMs"] = json!((st.render_ms * 1000.0).round() / 1000.0);
            Ok(out)
        },
    );
}

/// The lanes the process got (og_lanes::report) as the probe and the web
/// host's `og_web_lanes` report them: on, threads, pool, why, parallelCalls,
/// parallelItems, and `sites`: the same calls and items per call site
/// (`anim`, `extract`, `cull`, `physics`).
pub fn report() -> serde_json::Value {
    let r = og_lanes::report();
    let sites: serde_json::Map<String, serde_json::Value> = og_lanes::Site::ALL
        .iter()
        .map(|&s| {
            let c = r.sites[s as usize];
            (
                s.name().to_string(),
                json!({ "calls": c.calls, "items": c.items }),
            )
        })
        .collect();
    json!({
        "on": r.on,
        "threads": r.threads,
        "pool": r.pool,
        "why": r.why,
        "parallelCalls": r.parallel_calls,
        "parallelItems": r.parallel_items,
        "sites": sites,
    })
}

/// Records a Rust panic for the host's crash report (a panic hook calls it on
/// whichever thread panicked): its message and location, the tick the world
/// was running and, inside a lane, the lane's site (og_lanes::record_panic).
pub fn record_panic(info: &std::panic::PanicHookInfo<'_>) {
    og_lanes::record_panic_info(info, og_ecs::last_tick());
}

/// The first panic recorded, as the web host's `og_web_panic` answers it:
/// `{ kind: "panic", message, location, tick, lane }` (`lane` the site's
/// name, or null on the engine's own thread), or null.
pub fn panic_report() -> serde_json::Value {
    match og_lanes::panic_report() {
        None => serde_json::Value::Null,
        Some(p) => json!({
            "kind": "panic",
            "message": p.message,
            "location": p.location,
            "tick": p.tick,
            "lane": p.lane.map(|s| s.name()),
        }),
    }
}
