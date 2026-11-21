use super::*;

// ── the tick ────────────────────────────────────────────────────────────────

/// At a tick boundary: pending bundles, a scene waiting for its bundle,
/// GameDesc.start. `Err` once GameDesc.start has thrown, at this boundary
/// or an earlier one: no tick runs on the half-built world.
fn tick_boundary(eng: &Engine) -> Result<(), String> {
    let start = trace::start();
    crate::world::assets::resolve_releases(eng);
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    eng.assets.borrow_mut().poll(tick);
    crate::world::uncooked::poll(eng, tick);
    crate::world::data::poll(eng, tick);
    crate::world::scenes::before_boundary(eng);
    super::stream::update(eng);
    super::assets::retranscode(eng);
    super::stream::sound_chunks(eng);
    super::stream::resolve(eng);
    if eng.state.borrow().boot_scene.is_some() {
        boot_scene(eng);
    }
    if let Some(e) = &eng.state.borrow().start_failed {
        return Err(e.clone());
    }
    let result = run_start(eng);
    trace::preparation("tick boundary", start);
    result
}

/// Whether GameDesc.start is still waiting for what it needs (the boot
/// scene, the locale's font faces; run_start): no tick runs before it.
fn start_waiting(eng: &Engine) -> bool {
    eng.state.borrow().start.is_some()
}

/// The wall time since `t0` the ticks took, less the probe's screenshots
/// taken on the way (a 1280x720 PNG is a fifth of a second in a dev build:
/// a step time that counted it read as a stall on every screenshot tick).
fn step_ms(eng: &Engine, t0: f64) -> f64 {
    (rprobe::now_ms() - t0 - eng.render.borrow().capture_ms).max(0.0)
}

/// Up to `steps` ticks through the probe (due command steps run on the
/// way). `Err` names the tick and the failing system.
pub(crate) fn advance_probe(eng: &Engine, steps: u64) -> Result<u64, String> {
    if eng.stepping.get() {
        return Err("advance was called from inside a system".into());
    }
    eng.render.borrow_mut().capture_ms = 0.0;
    let t0 = rprobe::now_ms();
    let mut left = steps;
    let tick = loop {
        // A batch has exactly the boundaries of repeated single advances:
        // host answers and pending assets must reach the next fixed tick.
        pump_platform(eng)?;
        tick_boundary(eng)?;
        if start_waiting(eng) {
            break eng.world()?.time().tick;
        }
        let chunk = left.min(1);
        let stepped = rprobe::with_probe(eng, false, |p, w| p.advance(w, chunk));
        super::modules::end_of_ticks();
        super::stream::sound_chunks(eng);
        pump_platform(eng)?;
        let tick = stepped?.map_err(err)?;
        left -= chunk;
        if left == 0 || eng.probe.borrow().quit_requested() {
            break tick;
        }
    };
    if steps > 0 {
        eng.state.borrow_mut().step_ms = step_ms(eng, t0) / steps as f64;
    }
    Ok(tick)
}

/// `og_engine_advance`.
pub fn advance(id: u32, steps: u32) -> String {
    let eng = match engine(id) {
        Ok(e) => e,
        Err(e) => return refuse(e),
    };
    match advance_probe(&eng, u64::from(steps)) {
        Ok(tick) => ok(json!(tick)),
        Err(e) => refuse(e),
    }
}

/// `og_engine_frame`: a harness's frame, whose interval in the
/// performance record is the engine's clock between frames (a harness's
/// `elapsed_ms` is its own clock, one tick in a scripted run).
pub fn frame(id: u32, elapsed_ms: f64) -> String {
    frame_value(id, elapsed_ms, None).to_string()
}

fn refused(reason: impl Into<String>) -> Value {
    json!({ "ok": false, "reason": reason.into() })
}

/// One frame handed `elapsed_ms`, answered as `og_engine_frame` is.
/// `interval_ms` is the presenting host's interval since its last drawn
/// frame (`og_present_frame`), which the performance record takes; `None`
/// for a harness's frame.
pub(crate) fn frame_value(id: u32, elapsed_ms: f64, interval_ms: Option<f64>) -> Value {
    let eng = match engine(id) {
        Ok(e) => e,
        Err(e) => return refused(e),
    };
    if eng.stepping.get() {
        return refused("frame was called from inside a system");
    }
    // A playing video player with `wait` whose frame the last frame could
    // not show holds the clock as a pause does: no tick runs and no time is
    // owed, so the world (and its sound, rendered up to its tick) waits for
    // the footage instead of running ahead of it.
    let held = eng.probe.borrow().paused();
    let paused = held || eng.render.borrow().video_stalled;
    let (mut due, before, tick_ms) = {
        let mut st = eng.state.borrow_mut();
        let tick_ms = 1000.0 / f64::from(st.rate);
        let before = st.accumulator_ms;
        if paused || !elapsed_ms.is_finite() || elapsed_ms < 0.0 {
            (0, before, tick_ms)
        } else {
            let cap = f64::from(st.max_catch_up_ticks) * tick_ms;
            st.accumulator_ms = (st.accumulator_ms + elapsed_ms).min(cap);
            let n = (st.accumulator_ms / tick_ms).floor() as u32;
            st.accumulator_ms -= f64::from(n) * tick_ms;
            (n, before, tick_ms)
        }
    };
    let t0 = rprobe::now_ms();
    // The host's pump at each boundary, as an advance has it: host answers
    // and the TSX UI's ops since the last boundary.
    let pump = || pump_platform(&eng);
    if let Err(e) = pump().and_then(|()| tick_boundary(&eng)) {
        return refused(e);
    }
    // No tick before GameDesc.start has run, and no time owed for the wait.
    if start_waiting(&eng) {
        eng.state.borrow_mut().accumulator_ms = before;
        due = 0;
    }
    {
        let mut rs = eng.render.borrow_mut();
        rs.capture_ms = 0.0;
    }
    crate::bind::net::frame_clock(id, Some(t0 as u64));
    // Each tick of a frame that catches up has its own boundary, as an
    // advance's batch does: what a boundary puts in place (a streaming
    // group) lands at the same tick however the host's frames fall.
    let stepped = (|| -> Result<og_core::Result<u32>, String> {
        let mut taken = 0;
        loop {
            let n = match rprobe::with_probe(&eng, false, |p, w| p.frame(w, u32::from(due > 0)))? {
                Ok(n) => n,
                Err(e) => return Ok(Err(e)),
            };
            taken += n;
            if n == 0 || taken >= due {
                return Ok(Ok(taken));
            }
            super::modules::end_of_ticks();
            pump()?;
            tick_boundary(&eng)?;
        }
    })();
    crate::bind::net::frame_clock(id, None);
    let frame_t0 = t0;
    super::modules::end_of_ticks();
    super::stream::sound_chunks(&eng);
    let stepped = stepped.and_then(|r| pump().map(|()| r));
    let ticks = match stepped {
        Ok(Ok(n)) => n,
        // A system failed: the world's clock stays on the tick it threw in
        // (Time advances only after a tick's systems ran), which a catch-up
        // frame may have reached several ticks past the last answer's.
        Ok(Err(e)) => {
            let tick = eng.world().map(|w| w.time().tick).ok();
            return json!({ "ok": false, "reason": err(e), "tick": tick });
        }
        Err(e) => return refused(e),
    };
    if ticks > 0 {
        eng.state.borrow_mut().step_ms = step_ms(&eng, t0) / f64::from(ticks);
    }
    // The inspector's panel, built between the ticks and the frame: its
    // edits land between ticks (development builds).
    #[cfg(not(feature = "publish"))]
    crate::author::inspector::frame(&eng);
    // Drawn between the last two ticks by the accumulator's fraction
    // (RenderSettings.interpolate), or at the tick on a tick boundary.
    let after = eng.state.borrow().accumulator_ms;
    let alpha = super::interpolate::frame_alpha(before, after, tick_ms, ticks < due);
    let capture_only = eng.render.borrow().display.capture_only;
    let rendered = match if capture_only {
        Ok(false)
    } else {
        render(&eng, alpha, interval_ms, t0)
    } {
        Ok(r) => r,
        Err(e) => return refused(e),
    };
    // A frame that drew nothing (no renderer, capture-only) is a row of the
    // performance record too: its interval and work.
    if !rendered {
        let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
        crate::author::perf::frame(
            id,
            tick,
            interval_ms,
            rprobe::now_ms() - t0,
            0.0,
            &[],
            &[],
            &og_render::FrameStats::default(),
        );
    }
    // Automatic quality reads the drawn frame: its interval, the engine's
    // work on it (ticks and render) and the GPU's newest time.
    let page = {
        let gpu_ms = {
            let st = eng.state.borrow();
            st.gpu_frame_ms
        };
        let mut rs = eng.render.borrow_mut();
        if rendered {
            crate::present::display::after_frame(
                &mut rs,
                elapsed_ms,
                rprobe::now_ms() - frame_t0,
                gpu_ms,
            );
        }
        crate::present::display::page_asks(&rs)
    };
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    let (behind, step_ms, render_ms) = {
        let st = eng.state.borrow();
        (st.accumulator_ms, st.step_ms, st.render_ms)
    };
    let (text, copy) = text_entry(&eng);
    // `cursor`: the pointer's shape the UI resolved (og.ui.frameCursor, a
    // CSS cursor name): the web shell sets the canvas's. `cursorImage`, on
    // the frames where it changed: the first of its `url()` images that
    // loads (`{width, height, rgba: <base64>, hotspot: [x, y]}`), or null.
    let shape = crate::host::ui_cursor::shape(&eng);
    if let Some(b) = eng.render.borrow_mut().backend.as_mut() {
        b.cursor(&shape);
    }
    let cursor = shape.keyword;
    let cursor_image = {
        let mut st = eng.state.borrow_mut();
        let id = shape.id();
        (st.cursor_sent.map(|(_, i)| i) != Some(id.1)).then(|| {
            st.cursor_sent = Some(id);
            shape.image.as_ref().map_or(Value::Null, |i| {
                json!({
                    "width": i.width, "height": i.height,
                    "rgba": og_core::base64::encode(&i.rgba), "hotspot": i.hotspot,
                })
            })
        })
    };
    let a11y = a11y_changed(&eng);
    let pointer_lock = eng
        .world()
        .ok()
        .and_then(|w| w.resource::<og_input::PointerLock>().ok().map(|l| l.wanted))
        .unwrap_or(false);
    let probe = eng.probe.borrow();
    // `paused`: the clock was held for this frame or is held after it (a
    // host pump at one of its boundaries may resume or pause it).
    // `behind`: real time the accumulator holds that no tick has simulated
    // yet, so a host can place an input event within the next tick.
    // `text`: the focused text control (where the page's text target goes,
    // the selection a copy takes), `copy`: text a Ctrl/Cmd+C or X took.
    // `stepMs`: wall milliseconds a tick took (the mean of this frame's, or
    // the last frame's that ticked), `renderMs`: the last render's.
    // `pending`: the command channel's steps still to run (a script has
    // ended when it is 0, wherever a restore moved og.Time).
    // `pointerLock`: og.PointerLock.wanted, the game's ask for the pointer
    // lock (the web shell requests it on the next press, exits it when
    // false, and reports `locked` back as input).
    // `fullscreen`, `lowPower`, `rawMouse`: what the player's settings ask
    // of the page (the web shell enters or leaves fullscreen, keeps the
    // power preference for the next load, asks pointer lock for raw input).
    let mut answer = json!({
        "ok": true, "ticks": ticks, "tick": tick, "rendered": rendered,
        "paused": held || probe.paused(), "quit": probe.quit_requested(), "behind": behind,
        "text": text, "stepMs": step_ms, "renderMs": render_ms,
        "pending": probe.steps_left(), "pointerLock": pointer_lock,
        "fullscreen": page["fullscreen"],
        "lowPower": page["lowPower"], "rawMouse": page["rawMouse"],
        "cursor": cursor,
    });
    if let Some(c) = copy {
        answer["copy"] = json!(c);
    }
    // `buffering`: a playing video player with `wait` could not show its
    // frame, so the clock holds (no tick runs) until it can: the web shell
    // tells its page (ShellOptions.onBuffering) to show it is loading.
    if eng.render.try_borrow().is_ok_and(|rs| rs.video_stalled) {
        answer["buffering"] = json!(true);
    }
    if let Some(image) = cursor_image {
        answer["cursorImage"] = image;
    }
    // `haptics`: the patterns the UI's feedback played since the last
    // frame, at the player's strength (og.ui.setHaptics), each a list of
    // [startMs, ms, low, high] pulses: the web shell rumbles the pad and
    // vibrates the phone; a native host rumbles its pad.
    let haptics = haptics_played(&eng);
    if !haptics.is_empty() {
        answer["haptics"] = json!(haptics);
    }
    // `rumble`: what og.input.rumble asked for since the last frame, each
    // [pad, low, high, ms] (pad -1: every connected pad): output only.
    let rumble = super::api::rumble_played(&eng);
    if !rumble.is_empty() {
        answer["rumble"] = json!(rumble);
    }
    // `a11y`: what changed in the UI's accessibility tree since the last
    // answer (og-ui-next `a11y::web`), for a host that mirrors it (create
    // option `a11y`: the web shell's DOM mirror).
    if let Some(tree) = a11y {
        answer["a11y"] = tree;
    }
    // `saved`: `{scenes: [{source, text}]}`, the scene texts the
    // inspector's Save (or a prefab Apply or Revert) made on a host that
    // writes no files: the web shell sends them to the Stage
    // (`scene_saved`) or downloads them.
    #[cfg(not(feature = "publish"))]
    if let Some(saved) = eng
        .render
        .try_borrow_mut()
        .ok()
        .and_then(|mut rs| crate::author::inspector::take_saved(&mut rs))
    {
        answer["saved"] = saved;
    }
    answer
}

/// The new core's accessibility update for a host that mirrors it, when
/// what the tree shows changed since the last frame answer.
fn a11y_changed(eng: &Engine) -> Option<Value> {
    if !eng.state.borrow().a11y_mirror {
        return None;
    }
    eng.render.try_borrow_mut().ok()?.ui_next.as_mut()?.a11y()
}

/// The haptic patterns the UI played since the last frame answer.
fn haptics_played(eng: &Engine) -> Vec<Vec<[f32; 4]>> {
    let Some(ui) = eng.state.borrow().ui.clone() else {
        return Vec::new();
    };
    let mut ui = ui.lock();
    ui.take_haptics()
        .iter()
        .map(|h| {
            h.pulses()
                .iter()
                .map(|p| [p.start_ms, p.ms, p.low, p.high])
                .collect()
        })
        .collect()
}

/// The UI's focused text control as the web shell takes it (`null`: none),
/// and the text a copy or cut took since the last frame.
fn text_entry(eng: &Engine) -> (Value, Option<String>) {
    let Some(ui) = eng.state.borrow().ui.clone() else {
        return (Value::Null, None);
    };
    let mut ui = ui.lock();
    let copy = ui.take_clipboard();
    // The page places its input method's window in CSS px.
    let u = ui.frame().unit.max(1.0);
    let text = match &ui.frame().text_input {
        Some(t) => json!({
            "path": t.path,
            "rect": [t.rect.x * u, t.rect.y * u, t.rect.width * u, t.rect.height * u],
            "multiline": t.multiline,
            "selection": t.selection,
            "composing": t.composing,
            "type": t.input_type.name(),
            "inputmode": t.inputmode.name(),
            "autocomplete": t.autocomplete,
            "secret": t.secret,
        }),
        None => Value::Null,
    };
    (text, copy)
}

/// The scene at the world's tick as a frame draws it: between the last two
/// ticks at `alpha` (`None`: at the tick), its baked probes loaded and the
/// display's scale and quality applied. `display`: a frame that goes on
/// from the one before (per-object motion blur moves from it, and the
/// streamed mips count it); a reference capture is a still. The one
/// extraction a display frame and a probe capture share.
pub(crate) fn extract_frame(
    eng: &Engine,
    world: &og_ecs::World,
    alpha: Option<f32>,
    display: bool,
) -> Result<(), String> {
    world.parent_frames().map_err(err)?;
    let texel_paint = super::art::texel_paint(eng);
    let mut rs = eng.render.borrow_mut();
    rs.art.contract = super::art::contract(eng);
    rs.art.texel_paint = texel_paint;
    rs.interp.alpha = alpha;
    rs.interp.display = display;
    super::extract::extract(world, &mut rs);
    if display {
        super::stream::mips::frame(eng, &mut rs);
    }
    rs.interp.alpha = None;
    rs.interp.display = false;
    super::probes::load_bakes(eng, &mut rs);
    crate::present::display::before_render(&mut rs);
    Ok(())
}

/// Draws the frame, between the last two ticks at `alpha`
/// (`super::interpolate`) or at the tick (`None`). For the performance
/// record: `interval_ms` is the presenting host's time since its last drawn
/// frame (`None`: a harness's frame), `began` when the frame's work began.
fn render(
    eng: &Engine,
    alpha: Option<f32>,
    interval_ms: Option<f64>,
    began: f64,
) -> Result<bool, String> {
    if eng.render.borrow().backend.is_none() {
        return Ok(false);
    }
    let t0 = rprobe::now_ms();
    // A lost device is replaced before the frame is extracted, and what the
    // new one takes in other encodings is decoded again for it first.
    if let Some(b) = eng.render.borrow_mut().backend.as_mut() {
        b.recover()?;
    }
    super::assets::retranscode(eng);
    // The tick this frame shows: its notes (and a missing glyph) are this tick's.
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    // The async stages' work due by this tick, as the frame's kind asks:
    // a recorded frame settles it, any other draws what is ready.
    crate::present::barrier::frame(eng);
    let mut passes = Vec::new();
    let mut gpu = Vec::new();
    let mut frame = og_render::FrameStats::default();
    let notes = {
        extract_frame(eng, eng.world()?, alpha, true)?;
        let mut rs = eng.render.borrow_mut();
        let rs = &mut *rs;
        if let Some(b) = rs.backend.as_mut() {
            b.set_clock(Some(rprobe::now_ms));
            b.render(&rs.scene)?;
            // The decoded models' pairs were started by this frame.
            rs.scene.objects.prepare.clear();
            passes = b.pass_timings();
            gpu = b.gpu_timings(false);
            frame = b.frame_stats();
            crate::author::perf::stages(eng.id, &**b);
            // The fades held for pipelines this frame made ahead.
            rs.reveal.drawn(frame.compiling_ahead);
            crate::author::perf::held(eng.id, frame.compiling_ahead, rs.reveal.held());
        }
        crate::present::loading::compile_step(rs);
        super::assets::release_uploaded_textures(eng, rs);
        std::mem::take(&mut rs.notes)
    };
    if std::mem::take(&mut eng.render.borrow_mut().warm_tick) {
        crate::present::loading::warm_tick(eng.id);
    }
    let mut st = eng.state.borrow_mut();
    st.render_ms = rprobe::now_ms() - t0;
    super::trace::frame(eng.id, tick, t0, st.render_ms, &passes);
    super::trace::gpu_frames(eng.id, &gpu);
    crate::author::perf::frame(
        eng.id,
        tick,
        interval_ms,
        rprobe::now_ms() - began,
        st.render_ms,
        &passes,
        &gpu,
        &frame,
    );
    if let Some(newest) = gpu.last() {
        // Each timed pass's own time; one the GPU could not time is left
        // out, not reported as 0. Passes that ran at once each count their
        // whole time, so the frame's GPU time is its span.
        st.gpu_ms = newest
            .passes
            .iter()
            .filter_map(|p| Some((p.name.clone(), p.ms()?)))
            .collect();
        st.gpu_frame_ms = Some(newest.ms());
    }
    st.pass_ms = passes;
    st.frame = frame;
    for n in notes {
        if st.ui_warned.insert(n.clone()) {
            st.log
                .push(json!({ "tick": tick, "warning": true, "text": n }));
        }
    }
    Ok(true)
}

/// `og_engine_call`: the probe's JSON-RPC entry.
pub fn call_rpc(id: u32, frame: &str) -> String {
    match og_probe::rpc::parse(frame) {
        Ok(call) => answer_call(id, &call),
        Err(error) => error,
    }
}

/// `og_engine_take_outputs`.
pub fn take_outputs(id: u32) -> String {
    match engine(id) {
        Ok(eng) => {
            let outputs = std::mem::take(&mut eng.state.borrow_mut().outputs);
            rprobe::outputs_json(outputs).to_string()
        }
        Err(_) => "[]".into(),
    }
}

/// `og_engine_take_log`.
pub fn take_log(id: u32) -> String {
    match engine(id) {
        Ok(eng) => {
            let mut st = eng.state.borrow_mut();
            let mut log = std::mem::take(&mut st.log);
            for w in std::mem::take(&mut eng.assets.borrow_mut().warnings) {
                log.push(json!({ "tick": 0, "warning": true, "text": w }));
            }
            for w in super::agents::take_warnings(id) {
                log.push(json!({ "tick": 0, "warning": true, "text": w }));
            }
            Value::Array(log).to_string()
        }
        Err(_) => "[]".into(),
    }
}

/// Answers one parsed JSON-RPC call (a host's own transport: the native
/// WebSocket server's `Pending::call`), exactly as [`call_rpc`] does.
pub fn answer_call(id: u32, call: &og_probe::rpc::Call) -> String {
    let eng = match engine(id) {
        Ok(e) => e,
        Err(e) => {
            return json!({ "jsonrpc": "2.0", "id": call.id, "error": { "code": og_probe::rpc::INTERNAL_ERROR, "message": e } }).to_string()
        }
    };
    #[cfg(not(feature = "publish"))]
    if call.method == "advance" {
        use og_probe::rpc::{INTERNAL_ERROR, error_frame, result_frame, steps};
        return match steps(&call.params) {
            Ok(count) => match advance_probe(&eng, count) {
                Ok(tick) => result_frame(&call.id, json!(tick)),
                Err(error) => error_frame(&call.id, INTERNAL_ERROR, &format!("advance: {error}")),
            },
            Err(error) => error_frame(&call.id, error.code, &error.message),
        };
    }
    // `hello` only says who is speaking. The runtime asks it while creating
    // an engine (packages/engine/src/runtime.ts learnBuild), so with
    // `deferStart` it must not run GameDesc.start before the host has
    // attached its renderer and handed over its textures.
    if call.method != "hello"
        && let Err(e) = tick_boundary(&eng)
    {
        return json!({ "jsonrpc": "2.0", "id": call.id, "error": { "code": og_probe::rpc::INTERNAL_ERROR, "message": e } }).to_string();
    }
    // An external call may report the last frame's GPU counts
    // (`state().timings.triangles`, dump_render): read them now.
    rprobe::read_counts(&eng);
    let answer = rprobe::with_probe(&eng, true, |p, w| rprobe::answer(p, w, call));
    let swapped = answer.as_ref().is_ok_and(|a| a.1);
    let registered = rprobe::after_reload(&eng, swapped);
    rprobe::resync_names(&eng);
    match answer.and_then(|answer| registered.map(|()| answer)) {
        Ok(a) => a.0,
        Err(e) => json!({ "jsonrpc": "2.0", "id": call.id, "error": { "code": og_probe::rpc::INTERNAL_ERROR, "message": e } }).to_string(),
    }
}

/// The host's pump at a tick boundary, with the world's time: host answers
/// and the TSX UI's ops since the last boundary.
fn pump_platform(eng: &Engine) -> Result<(), String> {
    let Some(host) = hooks() else {
        return Ok(());
    };
    let time = eng.world()?.time();
    host.pump_platform(&Dispatch::platform_pump(eng.id, &time))
}
