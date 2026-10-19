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
            doc: "the lanes the engine got: on, threads, the pool (native, web-workers or none), why they are serial, the parallel calls and items so far, and the last step and render times".into(),
        },
        0,
        move |_world, _args| {
            let r = og_lanes::report();
            let eng = super::engine::engine(engine)?;
            let st = eng.state.borrow();
            Ok(json!({
                "on": r.on,
                "threads": r.threads,
                "pool": r.pool,
                "why": r.why,
                "parallelCalls": r.parallel_calls,
                "parallelItems": r.parallel_items,
                "stepMs": (st.step_ms * 1000.0).round() / 1000.0,
                "renderMs": (st.render_ms * 1000.0).round() / 1000.0,
            }))
        },
    );
}
