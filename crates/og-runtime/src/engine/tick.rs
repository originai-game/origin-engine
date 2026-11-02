use super::*;

// ── the tick ────────────────────────────────────────────────────────────────

/// At a tick boundary: pending bundles, a scene waiting for its bundle,
/// GameDesc.start. `Err` once GameDesc.start has thrown, at this boundary
/// or an earlier one: no tick runs on the half-built world.
fn tick_boundary(eng: &Engine) -> Result<(), String> {
    eng.assets.borrow_mut().poll();
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    crate::uncooked::poll(eng, tick);
    super::stream::update(eng);
    super::stream::sound_chunks(eng);
    super::stream::resolve(eng);
    if eng.state.borrow().boot_scene.is_some() {
        boot_scene(eng);
    }
    if let Some(e) = &eng.state.borrow().start_failed {
        return Err(e.clone());
    }
    run_start(eng)
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
    tick_boundary(eng)?;
    eng.render.borrow_mut().capture_ms = 0.0;
    let t0 = rprobe::now_ms();
    let mut left = steps;
    let tick = loop {
        let chunk = left.min(og_probe::PLAYTEST_MAX_TICKS);
        let stepped = rprobe::with_probe(eng, false, |p, w| p.advance(w, chunk));
        super::modules::end_of_ticks();
        super::stream::sound_chunks(eng);
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
    if eng.stepping.get() {
        return refuse("advance was called from inside a system");
    }
    match advance_probe(&eng, u64::from(steps)) {
        Ok(tick) => ok(json!(tick)),
        Err(e) => refuse(e),
    }
}

/// `og_engine_frame`.
pub fn frame(id: u32, elapsed_ms: f64) -> String {
    let eng = match engine(id) {
        Ok(e) => e,
        Err(e) => return refuse(e),
    };
    if eng.stepping.get() {
        return refuse("frame was called from inside a system");
    }
    let paused = eng.probe.borrow().paused();
    let (due, before, tick_ms) = {
        let mut st = eng.state.borrow_mut();
        let tick_ms = 1000.0 / f64::from(st.rate);
        let before = st.accumulator_ms;
        if paused || !elapsed_ms.is_finite() || elapsed_ms < 0.0 {
            (0, before, tick_ms)
        } else {
            let cap = f64::from(st.max_catch_up) * tick_ms;
            st.accumulator_ms = (st.accumulator_ms + elapsed_ms).min(cap);
            let n = (st.accumulator_ms / tick_ms).floor() as u32;
            st.accumulator_ms -= f64::from(n) * tick_ms;
            (n, before, tick_ms)
        }
    };
    if let Err(e) = tick_boundary(&eng) {
        return refuse(e);
    }
    eng.render.borrow_mut().capture_ms = 0.0;
    let t0 = rprobe::now_ms();
    let stepped = rprobe::with_probe(&eng, false, |p, w| p.frame(w, due));
    let frame_t0 = t0;
    super::modules::end_of_ticks();
    super::stream::sound_chunks(&eng);
    let ticks = match stepped {
        Ok(Ok(n)) => n,
        Ok(Err(e)) => return refuse(err(e)),
        Err(e) => return refuse(e),
    };
    if ticks > 0 {
        eng.state.borrow_mut().step_ms = step_ms(&eng, t0) / f64::from(ticks);
    }
    // The inspector's panel, built between the ticks and the frame: its
    // edits land between ticks (development builds).
    #[cfg(not(feature = "publish"))]
    crate::inspector::frame(&eng);
    // Drawn between the last two ticks by the accumulator's fraction
    // (RenderSettings.interpolate), or at the tick on a tick boundary.
    let after = eng.state.borrow().accumulator_ms;
    let alpha = super::interpolate::frame_alpha(before, after, tick_ms, ticks < due);
    let rendered = match render(&eng, alpha) {
        Ok(r) => r,
        Err(e) => return refuse(e),
    };
    // Automatic quality reads the drawn frame: its interval, the engine's
    // work on it (ticks and render) and the GPU's newest time.
    let (max_fps, page) = {
        let gpu_ms = {
            let st = eng.state.borrow();
            (!st.gpu_ms.is_empty()).then(|| st.gpu_ms.iter().map(|(_, ms)| *ms).sum::<f64>())
        };
        let mut rs = eng.render.borrow_mut();
        if rendered {
            crate::display::after_frame(&mut rs, elapsed_ms, rprobe::now_ms() - frame_t0, gpu_ms);
        }
        (crate::display::max_fps(&rs), crate::display::page_asks(&rs))
    };
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    let (behind, step_ms, render_ms) = {
        let st = eng.state.borrow();
        (st.accumulator_ms, st.step_ms, st.render_ms)
    };
    let (text, copy) = text_entry(&eng);
    // `cursor`: the pointer's shape the UI resolved (og.ui.frameCursor, a
    // CSS cursor name): the web shell sets the canvas's, a native host
    // with a window its own.
    let cursor =
        eng.state
            .borrow()
            .ui
            .clone()
            .map_or("default", |ui| match ui.lock().frame().cursor {
                og_ui::Cursor::Auto => "default",
                c => c.css(),
            });
    let a11y = a11y_changed(&eng);
    let pointer_lock = eng
        .world()
        .ok()
        .and_then(|w| w.resource::<og_input::PointerLock>().ok().map(|l| l.wanted))
        .unwrap_or(false);
    let probe = eng.probe.borrow();
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
    // `maxFps`: the frame cap resolved (og.render.setDisplay; 0: the
    // display's rate), which the web Worker keeps by skipping display frames.
    // `fullscreen`, `lowPower`, `rawMouse`: what the player's settings ask
    // of the page (the web shell enters or leaves fullscreen, keeps the
    // power preference for the next load, asks pointer lock for raw input).
    let mut answer = json!({
        "ok": true, "ticks": ticks, "tick": tick, "rendered": rendered,
        "paused": probe.paused(), "quit": probe.quit_requested(), "behind": behind,
        "text": text, "stepMs": step_ms, "renderMs": render_ms,
        "pending": probe.steps_left(), "pointerLock": pointer_lock,
        "maxFps": max_fps, "fullscreen": page["fullscreen"],
        "lowPower": page["lowPower"], "rawMouse": page["rawMouse"],
        "cursor": cursor,
    });
    if let Some(c) = copy {
        answer["copy"] = json!(c);
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
    // `a11y`: the UI's accessibility tree (as og.ui.a11y answers it), on
    // the frames where it changed, for a host that mirrors it (create
    // option `a11y`: the web shell's DOM mirror).
    if let Some(tree) = a11y {
        answer["a11y"] = tree;
    }
    answer.to_string()
}

/// The UI's accessibility tree when it differs from the one the last frame
/// answer carried (by [`og_ui::A11yTree::digest`]).
fn a11y_changed(eng: &Engine) -> Option<Value> {
    if !eng.state.borrow().a11y_mirror {
        return None;
    }
    let ui = eng.state.borrow().ui.clone()?;
    let ui = ui.lock();
    let tree = ui.a11y();
    let digest = tree.digest().max(1);
    let mut st = eng.state.borrow_mut();
    if st.a11y_digest == digest {
        return None;
    }
    st.a11y_digest = digest;
    Some(super::modules::a11y_json(tree, ui.frame().unit.max(1.0)))
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
        }),
        None => Value::Null,
    };
    (text, copy)
}

/// Draws the frame, between the last two ticks at `alpha`
/// (`super::interpolate`) or at the tick (`None`).
fn render(eng: &Engine, alpha: Option<f32>) -> Result<bool, String> {
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
    crate::motion::present::present(eng, alpha);
    let mut passes = Vec::new();
    let mut gpu = Vec::new();
    let mut triangles = 0;
    let notes = {
        let world = eng.world()?;
        let mut rs = eng.render.borrow_mut();
        rs.art.contract = super::art::contract(eng);
        rs.interp.alpha = alpha;
        rs.interp.display = true;
        super::extract::extract(world, &mut rs);
        rs.interp.alpha = None;
        rs.interp.display = false;
        super::probes::load_bakes(eng, &mut rs);
        crate::display::before_render(&mut rs);
        let rs = &mut *rs;
        if let Some(b) = rs.backend.as_mut() {
            b.set_clock(Some(rprobe::now_ms));
            b.render(&rs.scene)?;
            passes = b.pass_timings();
            gpu = b.gpu_timings(false);
            triangles = b.triangles();
        }
        std::mem::take(&mut rs.notes)
    };
    let mut st = eng.state.borrow_mut();
    st.render_ms = rprobe::now_ms() - t0;
    super::trace::frame(eng.id, tick, t0, st.render_ms, &passes);
    super::trace::gpu_frames(eng.id, &gpu);
    if let Some(newest) = gpu.last() {
        st.gpu_ms = newest
            .passes
            .iter()
            .map(|p| (p.name.clone(), p.ms()))
            .collect();
    }
    st.pass_ms = passes;
    st.triangles = triangles;
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
            for w in super::e6::take_warnings(id) {
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
    // `hello` only says who is speaking. The runtime asks it while creating
    // an engine (packages/engine/src/runtime.ts learnBuild), so with
    // `deferStart` it must not run GameDesc.start before the host has
    // attached its renderer and handed over its textures.
    if call.method != "hello"
        && let Err(e) = tick_boundary(&eng)
    {
        return json!({ "jsonrpc": "2.0", "id": call.id, "error": { "code": og_probe::rpc::INTERNAL_ERROR, "message": e } }).to_string();
    }
    let answer = rprobe::with_probe(&eng, true, |p, w| rprobe::answer(p, w, call));
    let swapped = answer.as_ref().is_ok_and(|a| a.1);
    rprobe::after_reload(&eng, swapped);
    rprobe::resync_names(&eng);
    match answer {
        Ok(a) => a.0,
        Err(e) => json!({ "jsonrpc": "2.0", "id": call.id, "error": { "code": og_probe::rpc::INTERNAL_ERROR, "message": e } }).to_string(),
    }
}
