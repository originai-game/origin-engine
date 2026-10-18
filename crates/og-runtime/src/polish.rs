//! `dump_pulse`: one tick's sample of what a player sees and hears change,
//! for `og polish` (packages/engine/src/node/polish.ts), which takes one
//! after every frame of a playtest and measures feel and taste from the
//! series: the time scale and hit stop, camera trauma, punch and kick,
//! floating texts, afterimages, running tweens, particle systems, the
//! playing voices by bus, the named entities' positions, scales and
//! flipbook frames, the UI's elements
//! (bounds and a hash of their text) and the last tick's step and render
//! times. Nothing here changes the world.

use super::codec::{Layout, RecordKind};
use og_ecs::World;
use og_probe::UiTree;
use serde_json::{Value, json};

fn text_hash(s: &str) -> u32 {
    // FNV-1a: cheap and stable across hosts.
    s.bytes()
        .fold(0x811c_9dc5u32, |h, b| (h ^ u32::from(b)).wrapping_mul(0x0100_0193))
}

fn round(x: f32) -> f64 {
    (f64::from(x) * 1e4).round() / 1e4
}

fn camera(world: &World, layouts: &std::collections::BTreeMap<String, Layout>) -> Value {
    let Ok(id) = world.id_by_name("og.Camera") else {
        return Value::Null;
    };
    let Ok(bytes) = world.resource_bytes(id) else {
        return Value::Null;
    };
    let layout = match layouts.get("og.Camera") {
        Some(l) => l.clone(),
        None => match world.schema(id) {
            Ok(s) => Layout::from_schema(s, RecordKind::Resource),
            Err(_) => return Value::Null,
        },
    };
    layout.decode(&bytes)
}

/// The sample, as `dump_pulse` writes it.
pub(crate) fn pulse(engine: u32, world: &World) -> Result<Value, String> {
    let eng = super::engine::engine(engine)?;
    let st = eng.state.borrow();
    let time = world.time();
    let scale = world.time_scale();
    let cam = camera(world, &st.layouts);
    let num = |v: &Value, k: &str| v[k].as_f64().unwrap_or(0.0);
    let kick = &cam["kick"];
    let kick_len = (num(kick, "x").powi(2) + num(kick, "y").powi(2) + num(kick, "z").powi(2)).sqrt();
    let (floating, spawned, texts, ghosts, tweens) = match st.feel.as_ref() {
        Some(f) => {
            let s = f.shared.lock().unwrap_or_else(|e| e.into_inner());
            (
                s.floating.len(),
                s.spawned,
                s.floating.iter().map(|t| t.text.clone()).collect::<Vec<_>>(),
                s.ghosts.values().map(Vec::len).sum::<usize>(),
                f.tweens.count(),
            )
        }
        None => (0, 0, Vec::new(), 0, 0),
    };
    let particles: Vec<u64> = eng
        .render
        .try_borrow()
        .map(|rs| rs.scene.particles.iter().map(|p| p.id).collect())
        .unwrap_or_default();
    let mut voices = Vec::new();
    let mut named = Vec::new();
    for e in world.entities() {
        if let Ok(p) = world.get::<og_audio::AudioPlayback>(e) {
            voices.push(json!([p.voice, p.bus, round(p.gain)]));
        }
    }
    // A flipbook's clip and frame (clip × 1000 + frame; -1 without one).
    let flipbook = world
        .id_by_name("og.Flipbook")
        .ok()
        .zip(st.layouts.get("og.Flipbook"));
    for (e, name) in world.names() {
        if let Ok(t) = world.get::<og_audio::Transform>(e) {
            let anim = flipbook
                .and_then(|(id, layout)| world.get_bytes(e, id).ok().map(|b| layout.decode(&b)))
                .map_or(-1.0, |f| num(&f, "clip") * 1000.0 + num(&f, "frame"));
            named.push(json!([
                name,
                round(t.position.x),
                round(t.position.y),
                round(t.position.z),
                round(t.scale.x),
                round(t.scale.y),
                anim
            ]));
        }
    }
    let ui: Vec<Value> = st
        .ui
        .as_ref()
        .map(|u| {
            og_ui::UiProbe(u.clone())
                .elements(world)
                .iter()
                .map(|el| {
                    json!([
                        el.id,
                        round(el.x),
                        round(el.y),
                        round(el.width),
                        round(el.height),
                        if el.text.is_empty() { 0 } else { text_hash(&el.text) }
                    ])
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(json!({
        "tick": time.tick,
        "scale": round(scale.scale),
        "hitStop": scale.hit_stop,
        "trauma": round(num(&cam, "trauma") as f32),
        "punch": round(num(&cam, "punch") as f32),
        "kick": round(kick_len as f32),
        "floating": floating,
        "spawned": spawned,
        "texts": texts,
        "ghosts": ghosts,
        "tweens": tweens,
        "particles": particles,
        "voices": voices,
        "named": named,
        "ui": ui,
        "stepMs": (st.step_ms * 1000.0).round() / 1000.0,
        "renderMs": (st.render_ms * 1000.0).round() / 1000.0,
    }))
}

/// Registers `dump_pulse[:path]`.
pub(crate) fn register(probe: &mut og_probe::Probe, engine: u32) {
    let _ = probe.register_dump(
        "dump_pulse",
        og_probe::CommandDesc {
            usage: "dump_pulse[:path]".into(),
            doc: "one tick's feel sample for og polish: time scale and hit stop, camera trauma, floating texts, afterimages, tweens, particle systems, voices by bus, named entities' positions, UI bounds, step and render times".into(),
        },
        0,
        move |world, _args| pulse(engine, world),
    );
}
