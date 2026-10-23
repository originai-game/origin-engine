//! og-probe mounted on the runtime, per `crates/og-probe/README.md` §What a
//! host provides: the probe is created once per world (after the engine's and
//! the game's types are registered, before the systems), every tick goes
//! through it, its calls arrive between ticks, its outputs and log are handed
//! to the host, and the runtime gives it the hooks it has: `Capture` over
//! og-render, the declared systems, the game's facts, the game's and the
//! modules' commands, the measured timings and every named spawn.
//!
//! The probe is not re-entrant: while it runs (a frame's command steps, a
//! JSON-RPC call), game code it calls (a command, the facts) reaches the
//! world through the runtime as usual, but a second probe call is refused.
//! Names given while the probe runs are queued and reach the probe when it
//! returns, so a scheduled step addresses a name a command gave from the
//! next probe call on.

use super::engine::{DECL, Engine, assemble, call, engine, extract_layouts, hooks, invalidate};
use super::extract::{RenderState, extract};
use og_ecs::{Entity, World};
use og_probe::{Capture, Frame, Probe, Rebuilt, Reloader};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

/// The probe's renderer hook: the frame of the world as handed in, drawn
/// offscreen through the host's backend.
pub(crate) struct RuntimeCapture {
    pub render: Rc<RefCell<RenderState>>,
    pub tier: String,
}

impl Capture for RuntimeCapture {
    fn tier(&self) -> &str {
        &self.tier
    }

    fn target_size(&self) -> (u32, u32) {
        self.render
            .borrow()
            .backend
            .as_ref()
            .map_or((0, 0), |b| b.size())
    }

    fn deferred(&self) -> bool {
        self.render
            .try_borrow()
            .ok()
            .and_then(|rs| rs.backend.as_ref().map(|b| b.deferred()))
            .unwrap_or(false)
    }

    fn save_state(&mut self) -> u64 {
        self.render
            .try_borrow_mut()
            .ok()
            .and_then(|mut rs| rs.backend.as_mut().map(|b| b.save_state()))
            .unwrap_or(0)
    }

    fn restore_state(&mut self, state: u64) {
        if let Ok(mut rs) = self.render.try_borrow_mut()
            && let Some(b) = rs.backend.as_mut()
        {
            b.restore_state(state);
        }
    }

    fn frame(&mut self) -> Option<og_probe::RawFrame> {
        let mut rs = self.render.try_borrow_mut().ok()?;
        let image = rs.backend.as_mut()?.read_frame()?;
        Some(og_probe::RawFrame {
            width: image.width,
            height: image.height,
            rgba: image.pixels,
        })
    }

    fn capture(&mut self, world: &World, width: u32, height: u32) -> Option<Frame> {
        let mut rs = self.render.try_borrow_mut().ok()?;
        // Captured frames show every wanted terrain tile, whatever the decode timing.
        rs.terrain.settle = true;
        extract(world, &mut rs);
        let rs = &mut *rs;
        let t0 = now_ms();
        let image = rs.backend.as_mut()?.capture(&rs.scene, width, height);
        let frame = image.map(|image| Frame {
            width: image.width,
            height: image.height,
            png: png(&image),
        });
        // The probe's work, not the game's: the tick's step time leaves it out.
        rs.capture_ms += now_ms() - t0;
        frame
    }
}

/// A name change made outside the probe, replayed into it.
pub(crate) enum NameOp {
    Set(Entity, String),
    Remove(String),
}

/// Runs `f` on the engine's probe and world between ticks: queued names go
/// in first, views are retired (`moving`: whether `f` may move column
/// memory), the timings are current; afterwards names given meanwhile go in,
/// and the log joins the engine's.
pub(crate) fn with_probe<T>(
    eng: &Engine,
    moving: bool,
    f: impl FnOnce(&mut Probe, &mut World) -> T,
) -> Result<T, String> {
    if eng.stepping.get() {
        return Err("the probe is not available while a system runs; call it between ticks".into());
    }
    let mut probe = eng.probe.try_borrow_mut().map_err(|_| {
        "the probe is busy (a command step or a probe call is running); call it after that returns"
            .to_string()
    })?;
    flush_names(eng);
    {
        let st = eng.state.borrow();
        probe.set_timings(st.step_ms, st.render_ms);
    }
    invalidate(moving);
    super::engine::clear_world_queries();
    let world = eng.world()?;
    let out = f(&mut probe, world);
    flush_names(eng);
    let log = probe.take_log();
    let outputs = probe.take_outputs();
    {
        let mut st = eng.state.borrow_mut();
        st.log.extend(
            log.into_iter()
                .map(|l| json!({ "tick": l.tick, "warning": l.warning, "text": l.text })),
        );
        st.outputs.extend(outputs);
    }
    invalidate(moving);
    super::engine::clear_world_queries();
    Ok(out)
}

fn flush_names(eng: &Engine) {
    let ops = std::mem::take(&mut eng.state.borrow_mut().name_ops);
    let Ok(world) = eng.world() else {
        return;
    };
    for op in ops {
        // An entity despawned before its name went in has no name to give.
        match op {
            NameOp::Set(e, name) => {
                let _ = world.set_name(e, &name);
            }
            NameOp::Remove(name) => {
                if let Some(e) = world.find(&name) {
                    let _ = world.clear_name(e);
                }
            }
        }
    }
}

/// After a call that may have restored a snapshot, the runtime's names are
/// the world's (a restore brings back the names of its world).
pub(crate) fn resync_names(eng: &Engine) {
    let (Ok(probe), Ok(world)) = (eng.probe.try_borrow(), eng.world()) else {
        return;
    };
    let live = probe.names().live(world);
    let mut st = eng.state.borrow_mut();
    st.names.clear();
    st.name_of.clear();
    for (name, e) in live {
        st.names.insert(name.clone(), e.to_bits());
        st.name_of.insert(e.to_bits(), name);
    }
}

/// Registers one command into the probe; the handler looks its callback up
/// by name when it runs, so a hot reload's new handler takes over.
pub(crate) fn register_one(
    probe: &mut Probe,
    id: u32,
    name: &str,
    usage: &str,
    doc: &str,
) -> Result<(), String> {
    let desc = og_probe::CommandDesc {
        usage: usage.to_string(),
        doc: doc.to_string(),
    };
    let action = name.to_string();
    probe
        .register_command(name, desc, move |_world, args| {
            let eng = engine(id)?;
            let handler = eng
                .state
                .borrow()
                .commands
                .get(&action)
                .map(|c| c.handler)
                .ok_or_else(|| format!("'{action}' is no longer registered by the game"))?;
            call(&eng, 1, handler, json!(args).to_string())
                .map(|_| ())
                .map_err(|e| format!("{action}: {e}"))
        })
        .map_err(|e| format!("command '{name}': {}", e.reason()))
}

/// Registers the game's commands, the declared ones and the facts function
/// into a new engine's probe.
pub(crate) fn install(eng: &Rc<Engine>, facts: Option<u32>) -> Result<(), String> {
    let id = eng.id;
    let commands: Vec<(String, String, String)> = eng
        .state
        .borrow()
        .commands
        .iter()
        .map(|(k, v)| (k.clone(), v.usage.clone(), v.doc.clone()))
        .collect();
    let mut probe = eng.probe.borrow_mut();
    probe.set_engine_version(format!("origin-engine {}", env!("CARGO_PKG_VERSION")));
    // A world.restore asked for during a tick applies as the tick ends.
    probe.set_after_tick(Box::new(move |world| {
        if let Ok(eng) = engine(id) {
            super::save::after_tick(&eng, world);
        }
    }));
    // og.WorldRestored, the tick after a restore.
    probe.set_before_tick(Box::new(move |world| {
        if let Ok(eng) = engine(id) {
            super::save::before_tick(&eng, world);
        }
    }));
    if let Some(ui) = eng.state.borrow().ui.as_ref() {
        probe.set_ui(ui.probe());
    }
    for (name, usage, doc) in commands {
        register_one(&mut probe, id, &name, &usage, &doc)?;
    }
    // `get` answers what og.DayNight makes of the sun, as drawn.
    probe.set_derive(Box::new(move |world, entity, name, fields| {
        if let Ok(eng) = engine(id)
            && let Ok(st) = eng.state.try_borrow()
        {
            super::atmosphere::derive(world, &st.layouts, entity, name, fields);
        }
    }));
    if facts.is_some() {
        probe.set_facts(move |_world| {
            let Ok(eng) = engine(id) else {
                return Value::Null;
            };
            let Some(facts) = eng.state.borrow().facts else {
                return Value::Null;
            };
            call(&eng, 2, facts, String::new())
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(Value::Null)
        });
    }
    Ok(())
}

/// The host's half of a hot reload: the latest declared game, assembled into
/// a fresh world. The runtime's view of it waits in `State::reloaded` until
/// the probe has restored the old world into it ([`after_reload`]).
struct RuntimeReloader {
    engine: u32,
}

impl Reloader for RuntimeReloader {
    fn rebuild(&mut self, current: &World) -> Result<Rebuilt, String> {
        let t0 = now_ms();
        let latest = DECL.with(|d| d.borrow().games.len());
        let game = latest
            .checked_sub(1)
            .ok_or("no game was declared; the bundle must export og.game(...)")?;
        let seed = current.resource::<og_core::Rng>().map(|r| r.seed()).ok();
        let eng = engine(self.engine)?;
        let (profile, ui, project) = {
            let st = eng.state.borrow();
            (st.profile, st.ui.clone(), st.project.clone())
        };
        let link = og_audio::link(self.engine);
        let previous = match (&ui, link) {
            (Some(u), Some(l)) => Some((u, l)),
            _ => None,
        };
        let mut a = assemble(
            game as u32,
            seed,
            self.engine,
            profile,
            project.as_ref(),
            previous,
        )?;
        a.probe = None;
        let world = std::mem::replace(
            &mut a.world,
            World::new(&og_ecs::WorldDesc::default()).map_err(|e| e.reason().to_string())?,
        );
        eng.state.borrow_mut().reloaded = Some(Box::new(a));
        Ok(Rebuilt {
            world,
            ms: now_ms() - t0,
        })
    }
}

pub(crate) fn set_reloader(eng: &Rc<Engine>) {
    eng.probe
        .borrow_mut()
        .set_reloader(Box::new(RuntimeReloader { engine: eng.id }));
}

/// After a reload: the runtime takes the rebuilt bundle's view (layouts,
/// systems, commands, facts) when the probe swapped the world, and drops it
/// when the probe kept the old bundle.
pub(crate) fn after_reload(eng: &Engine, swapped: bool) {
    super::e6::after_reload(eng.id, swapped);
    let Some(a) = eng.state.borrow_mut().reloaded.take() else {
        return;
    };
    if !swapped {
        return;
    }
    let new_commands: Vec<(String, String, String)> = {
        let st = eng.state.borrow();
        a.commands
            .iter()
            .filter(|(k, _)| !st.commands.contains_key(*k))
            .map(|(k, v)| (k.clone(), v.usage.clone(), v.doc.clone()))
            .collect()
    };
    let meshes = {
        let mut st = eng.state.borrow_mut();
        super::engine::bump_layout_epoch();
        st.layouts = a.layouts;
        st.names_by_id = a.names_by_id;
        st.resources = a.resources;
        st.systems = a.systems;
        st.commands = a.commands;
        st.facts = a.game.facts;
        st.warnings.extend(a.warnings);
        st.physics = Some(a.mounted.physics.clone());
        st.ui = Some(a.mounted.ui.clone());
        st.anim = Some(a.anim.clone());
        st.feel = Some(a.feel.clone());
        (st.physics_meshes.clone(), st.physics_heightfields.clone())
    };
    let (meshes, heightfields) = meshes;
    // Animation assets too: the new og.anim gets every skeleton, clip and graph.
    eng.state.borrow().anim_assets.replay(&a.anim);
    eng.state.borrow().feel_assets.replay(&a.feel);
    // Meshes are assets: the new physics gets every one the old one had.
    for (handle, mesh) in meshes {
        let _ = a.mounted.physics.set_mesh(handle, mesh);
    }
    // And terrains' height fields.
    for (handle, field) in heightfields {
        let _ = a.mounted.physics.set_heightfield(handle, field);
    }
    // So are ragdoll rigs.
    for (&e, kept) in &eng.render.borrow().atmo.rigs {
        let _ = a
            .mounted
            .physics
            .set_ragdoll_rig(og_ecs::Entity::from_bits(e), kept.rig.clone());
    }
    og_audio::publish_link(eng.id, a.mounted.audio.clone());
    eng.render.borrow_mut().ui = Some(a.mounted.ui.clone());
    if let Ok(mut probe) = eng.probe.try_borrow_mut() {
        for (name, usage, doc) in new_commands {
            let _ = register_one(&mut probe, eng.id, &name, &usage, &doc);
        }
        probe.set_ui(a.mounted.ui.probe());
    }
    let (camera, settings, draw) = extract_layouts(eng);
    let mut rs = eng.render.borrow_mut();
    rs.camera = camera;
    rs.settings = settings;
    rs.draw = draw;
    rs.fx.anim = Some(a.anim.clone());
    rs.feel.feel = Some((a.feel.clone(), a.feel_records.clone()));
    drop(rs);
    // `og dev` cooks an asset edit before it rebuilds the bundle: a model
    // whose cooked output changed loads again under the handle it had.
    let (_, failed) = super::assets::refresh(eng);
    eng.state.borrow_mut().warnings.extend(failed);
}

/// One JSON-RPC call through the probe, with the runtime's additions to
/// `hello` (IDL `Hello.modules`). Returns the answer frame and whether a
/// reload swapped the world.
pub(crate) fn answer(
    probe: &mut Probe,
    world: &mut World,
    call: &og_probe::rpc::Call,
) -> (String, bool) {
    let answer = probe.answer(world, call);
    let method = call.method.as_str();
    if method != "reload" && method != "hello" {
        return (answer, false);
    }
    let Ok(mut v) = serde_json::from_str::<Value>(&answer) else {
        return (answer, false);
    };
    if method == "reload" {
        return (answer, v["result"].is_object());
    }
    if v["result"].is_object() {
        v["result"]["modules"] = json!(modules());
    }
    (v.to_string(), false)
}

/// Attaches (or detaches) the backend and the probe's capture hook.
pub(crate) fn attach(eng: &Engine, backend: Option<Box<dyn super::RenderBackend>>) {
    let tier = backend.as_ref().map(|b| b.tier());
    {
        let mut rs = eng.render.borrow_mut();
        rs.backend = backend;
        let (_, _, layouts) = extract_layouts(eng);
        rs.draw = layouts;
    }
    if let (Some(tier), Ok(mut probe)) = (tier, eng.probe.try_borrow_mut()) {
        probe.set_capture(Box::new(RuntimeCapture {
            render: eng.render.clone(),
            tier,
        }));
    }
}

/// The engine modules beyond the core this runtime was built with.
pub(crate) fn modules() -> Vec<&'static str> {
    vec!["audio", "cook", "physics", "render", "ui"]
}

/// Outputs as the `og_engine_take_outputs` JSON.
pub(crate) fn outputs_json(outputs: Vec<og_probe::Output>) -> Value {
    Value::Array(
        outputs
            .into_iter()
            .map(|o| {
                let (kind, data) = match &o.payload {
                    og_probe::Payload::Png { bytes, .. } => ("png", json!(base64(bytes))),
                    og_probe::Payload::Json(_) => (
                        "json",
                        json!(String::from_utf8_lossy(&o.bytes()).into_owned()),
                    ),
                    og_probe::Payload::Text(t) => ("text", json!(t)),
                    og_probe::Payload::Wav(bytes) => ("wav", json!(base64(bytes))),
                };
                json!({ "tick": o.tick, "action": o.action, "file": o.file_name(), "kind": kind, "data": data })
            })
            .collect(),
    )
}

/// The wall clock, for timings only.
pub(crate) fn now_ms() -> f64 {
    hooks().map_or(0.0, |h| h.now_ms())
}

pub(crate) fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for (i, shift) in [18, 12, 6, 0].iter().enumerate() {
            if i <= chunk.len() {
                out.push(T[((n >> shift) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// A PNG of an RGBA8 image: rows filtered with `Up`, deflated (zlib level 6).
pub fn png(image: &og_render::RgbaImage) -> Vec<u8> {
    // CRC-32 a byte at a time from a table made at compile time: the
    // bitwise loop took a fifth of a screenshot's tick unoptimised (dev).
    const CRC: [u32; 256] = {
        let mut t = [0u32; 256];
        let mut n = 0;
        while n < 256 {
            let mut c = n as u32;
            let mut k = 0;
            while k < 8 {
                c = if c & 1 != 0 {
                    0xedb8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
                k += 1;
            }
            t[n] = c;
            n += 1;
        }
        t
    };
    fn crc(data: &[u8]) -> u32 {
        let mut c = 0xffff_ffffu32;
        for &b in data {
            c = CRC[((c ^ u32::from(b)) & 0xff) as usize] ^ (c >> 8);
        }
        !c
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc(&body).to_be_bytes());
    }
    let (w, h) = (image.width as usize, image.height as usize);
    let stride = w * 4;
    let mut raw = Vec::with_capacity((stride + 1) * h);
    for row in 0..h {
        let cur = &image.pixels[row * stride..(row + 1) * stride];
        if row == 0 {
            raw.push(0);
            raw.extend_from_slice(cur);
        } else {
            let prev = &image.pixels[(row - 1) * stride..row * stride];
            raw.push(2);
            raw.extend(cur.iter().zip(prev).map(|(c, p)| c.wrapping_sub(*p)));
        }
    }
    let z = miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6);
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}
