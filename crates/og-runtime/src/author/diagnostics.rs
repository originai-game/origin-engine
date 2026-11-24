//! `dump_physics` and `dump_audio`: read-only diagnostic snapshots of the
//! physics world (bodies, colliders, joints, active contacts/sensors,
//! characters, vehicles, soft bodies, fluids) and the audio subsystem (active
//! voices, buses, snapshots, music clocks, reverb zones, sound bank, recent
//! listen peak/RMS, and mixer stats). Never registered in `publish` builds.

use crate::engine::engine;
use glam::Vec3;
use og_ecs::{Entity, World};
use serde_json::{Value, json};

fn vec3_json(v: Vec3) -> Value {
    let r = |x: f32| (f64::from(x) * 1e4).round() / 1e4;
    json!([r(v.x), r(v.y), r(v.z)])
}

fn round4(x: f32) -> f64 {
    (f64::from(x) * 1e4).round() / 1e4
}

fn entity_id(world: &World, bits: u64) -> Value {
    if bits == 0 {
        return Value::Null;
    }
    let e = Entity::from_bits(bits);
    if world.is_alive(e) {
        Value::String(og_probe::Names.id_of(world, e))
    } else {
        json!(bits)
    }
}

/// Snapshot of the physics simulation and character controllers (`dump_physics`).
pub(crate) fn dump_physics(engine_id: u32, world: &World) -> Result<Value, String> {
    let eng = engine(engine_id)?;
    let tick = world.time().tick;
    let phys = eng.state.borrow().physics.clone();

    let mut characters = Vec::new();
    for e in world.entities() {
        if let Ok(c) = world.get::<og_physics::CharacterController>(e) {
            characters.push(json!({
                "entity": og_probe::Names.id_of(world, e),
                "kind": "character",
                "grounded": c.grounded,
                "sliding": c.sliding,
                "velocity": vec3_json(c.velocity),
                "groundNormal": vec3_json(c.ground_normal),
                "coyote": round4(c.coyote),
                "ground": entity_id(world, c.ground),
            }));
        }
        if let Ok(p) = world.get::<og_physics::PlatformerController>(e) {
            characters.push(json!({
                "entity": og_probe::Names.id_of(world, e),
                "kind": "platformer",
                "grounded": p.grounded,
                "velocity": vec3_json(p.velocity),
                "coyote": round4(p.coyote),
                "ground": entity_id(world, p.ground),
            }));
        }
    }

    let Some(phys) = phys else {
        return Ok(json!({
            "kind": "physics",
            "tick": tick,
            "installed": false,
            "stepped": false,
            "characters": characters,
        }));
    };

    let Some(d) = phys.dump() else {
        return Ok(json!({
            "kind": "physics",
            "tick": tick,
            "installed": true,
            "stepped": false,
            "characters": characters,
        }));
    };

    let bodies: Vec<Value> = d
        .bodies
        .iter()
        .map(|b| {
            json!({
                "entity": entity_id(world, b.entity),
                "kind": b.kind,
                "sleeping": b.sleeping,
                "position": vec3_json(b.position),
                "linearVelocity": vec3_json(b.linear_velocity),
                "angularVelocity": vec3_json(b.angular_velocity),
                "colliders": b.colliders,
            })
        })
        .collect();

    let joints: Vec<Value> = d
        .joints
        .iter()
        .map(|j| {
            json!({
                "entity": entity_id(world, j.entity),
                "kind": j.kind,
                "other": entity_id(world, j.other),
            })
        })
        .collect();

    let contacts: Vec<Value> = d
        .contacts
        .iter()
        .map(|c| {
            json!({
                "a": entity_id(world, c.a),
                "b": entity_id(world, c.b),
                "sensor": c.sensor,
                "manifolds": c.manifolds,
                "points": c.points,
                "deepestDist": round4(c.deepest_dist),
                "point": vec3_json(c.point),
                "normal": vec3_json(c.normal),
            })
        })
        .collect();

    let vehicles: Vec<Value> = d
        .vehicles
        .iter()
        .map(|v| {
            json!({
                "entity": entity_id(world, v.entity),
                "wheels": v.wheels,
                "groundedWheels": v.grounded_wheels,
            })
        })
        .collect();

    let ragdolls: Vec<Value> = d
        .ragdolls
        .iter()
        .map(|r| {
            json!({
                "entity": entity_id(world, r.entity),
                "bones": r.bones,
                "dynamic": r.dynamic,
            })
        })
        .collect();

    let map_counts = |items: &[(u64, usize)]| -> Vec<Value> {
        items
            .iter()
            .map(|&(e, n)| json!({ "entity": entity_id(world, e), "count": n }))
            .collect()
    };

    Ok(json!({
        "kind": "physics",
        "tick": tick,
        "installed": true,
        "stepped": true,
        "profile": d.profile,
        "gravity": vec3_json(d.gravity),
        "counts": {
            "trackedBodies": d.body_counts.tracked,
            "rapierBodies": d.body_counts.rapier,
            "dynamic": d.body_counts.dynamic,
            "fixed": d.body_counts.fixed,
            "kinematic": d.body_counts.kinematic,
            "awake": d.body_counts.awake,
            "sleeping": d.body_counts.sleeping,
            "colliders": d.collider_counts.total,
            "statics": d.collider_counts.statics,
            "sensors": d.collider_counts.sensors,
            "compounds": d.collider_counts.compounds,
            "joints": joints.len(),
            "contacts": contacts.len(),
            "characters": characters.len(),
            "vehicles": vehicles.len(),
            "ragdolls": ragdolls.len(),
            "ropes": d.ropes.len(),
            "cloths": d.cloths.len(),
            "softBodies": d.soft_bodies.len(),
            "garments": d.garments.len(),
            "fluids2d": d.fluids2d.len(),
            "fluids3d": d.fluids3d.len(),
            "waterSurfaces": d.water_surfaces,
            "waterLines": d.water_lines,
        },
        "stats": {
            "tick": d.stats.tick,
            "bodies": d.stats.bodies,
            "threads": d.stats.threads,
            "pool": d.stats.pool,
            "ms": (d.stats.ms * 1000.0).round() / 1000.0,
        },
        "bodies": bodies,
        "joints": joints,
        "contacts": contacts,
        "characters": characters,
        "vehicles": vehicles,
        "ragdolls": ragdolls,
        "ropes": map_counts(&d.ropes),
        "cloths": map_counts(&d.cloths),
        "softBodies": map_counts(&d.soft_bodies),
        "garments": map_counts(&d.garments),
        "fluids2d": map_counts(&d.fluids2d),
        "fluids3d": map_counts(&d.fluids3d),
    }))
}

fn bus_name(bus: u32) -> Value {
    match bus {
        0 => json!("master"),
        1 => json!("music"),
        2 => json!("sfx"),
        3 => json!("ui"),
        n => json!(format!("bus{n}")),
    }
}

/// Snapshot of the audio system (`dump_audio`).
pub(crate) fn dump_audio(engine_id: u32, world: &World) -> Result<Value, String> {
    let tick = world.time().tick;
    let link = og_audio::link(engine_id);

    let settings = world.resource::<og_audio::Audio>().ok().map(|a| {
        json!({
            "master": round4(a.master),
            "music": round4(a.music),
            "sfx": round4(a.sfx),
            "ui": round4(a.ui),
            "maxVoices": a.max_voices,
        })
    });

    let stream = world.resource::<og_audio::AudioStream>().ok().map(|s| {
        json!({
            "hash": format!("{:016x}", s.hash),
            "commands": s.commands,
            "nextVoice": s.next_voice,
            "voices": s.voices,
            "tickRate": s.tick_rate,
        })
    });

    let mut voices = Vec::new();
    let mut user_buses = Vec::new();
    let mut snapshots = Vec::new();
    let mut music = Vec::new();
    let mut reverb_zones = Vec::new();
    let mut listeners = Vec::new();

    for e in world.entities() {
        let id = og_probe::Names.id_of(world, e);
        if let Ok(p) = world.get::<og_audio::AudioPlayback>(e) {
            let sound_info = link
                .as_ref()
                .and_then(|l| l.bank().get(p.sound))
                .map(|sd| (sd.id.clone(), sd.sample_rate, sd.duration(), sd.streamed()));
            let (sound_id, rate, duration, streamed) = match sound_info {
                Some((sid, r, d, st)) => (Value::String(sid), r, Some(d), st),
                None => (Value::Null, 0, None, false),
            };
            let cursor_sec = if rate > 0 {
                (p.cursor / f64::from(rate) * 1e4).round() / 1e4
            } else {
                0.0
            };
            voices.push(json!({
                "entity": id,
                "voice": p.voice,
                "sound": p.sound,
                "soundId": sound_id,
                "bus": bus_name(p.bus),
                "busIndex": p.bus,
                "gain": round4(p.gain),
                "pan": round4(p.pan),
                "pitch": round4(p.pitch),
                "cursorSec": cursor_sec,
                "durationSec": duration.map(|d| (d * 1e4).round() / 1e4),
                "streamed": streamed,
                "virtual": p.is_virtual,
                "lowpass": round4(p.lowpass),
                "flags": p.flags,
            }));
        }
        if let Ok(b) = world.get::<og_audio::AudioBusNode>(e) {
            user_buses.push(json!({
                "entity": id,
                "index": b.index,
                "parent": b.parent,
                "volume": round4(b.volume),
            }));
        }
        if let Ok(s) = world.get::<og_audio::AudioSnapshot>(e) {
            snapshots.push(json!({
                "entity": id,
                "active": s.active,
                "weight": round4(s.weight),
                "fade": round4(s.fade),
            }));
        }
        if let Ok(m) = world.get::<og_audio::Music>(e) {
            music.push(json!({
                "entity": id,
                "bpm": round4(m.bpm),
                "beatsPerBar": m.beats_per_bar,
                "bar": m.bar,
                "beat": m.beat,
                "nextBeat": (m.next_beat * 1e4).round() / 1e4,
                "nextBar": (m.next_bar * 1e4).round() / 1e4,
            }));
        }
        if let Ok(z) = world.get::<og_audio::ReverbZone>(e) {
            reverb_zones.push(json!({
                "entity": id,
                "bus": bus_name(z.bus),
                "weight": round4(z.weight),
                "send": round4(z.send),
                "room": round4(z.room),
                "damping": round4(z.damping),
            }));
        }
        if let Ok(l) = world.get::<og_audio::AudioListener>(e) {
            listeners.push(json!({
                "entity": id,
                "volume": round4(l.volume),
                "dopplerFactor": round4(l.doppler_factor),
                "panningModel": if l.panning_model == 1 { "hrtf" } else { "equalpower" },
            }));
        }
    }

    let link_json = match &link {
        Some(l) => {
            let decode = l.bank().decode_stats();
            let recent = l.listen().last(1.0).map(|(pcm, rate)| {
                let peak = pcm.iter().fold(0.0f32, |m, x| m.max(x.abs()));
                let rms = (pcm.iter().map(|x| x * x).sum::<f32>() / pcm.len().max(1) as f32).sqrt();
                json!({
                    "rate": rate,
                    "frames": (pcm.len() / 2) as u64,
                    "peak": round4(peak),
                    "rms": round4(rms),
                })
            });
            let mix = l.mix_stats().map(|ms| {
                json!({
                    "frames": ms.frames,
                    "commands": ms.commands,
                    "late": ms.late,
                    "reanchors": ms.reanchors,
                    "voices": ms.voices,
                    "voicesPeak": ms.voices_peak,
                    "limited": ms.limited,
                })
            });
            json!({
                "attached": l.attached(),
                "tickRate": l.tick_rate(),
                "nextTick": l.next_tick(),
                "dropped": l.dropped(),
                "bank": {
                    "sounds": l.bank().len(),
                    "residentBytes": l.bank().resident_bytes(),
                    "decode": {
                        "ahead": decode.ahead,
                        "misses": decode.misses,
                        "queued": decode.queued,
                    },
                },
                "listen": recent,
                "mix": mix,
            })
        }
        None => Value::Null,
    };

    Ok(json!({
        "kind": "audio",
        "tick": tick,
        "settings": settings,
        "stream": stream,
        "link": link_json,
        "voices": voices,
        "buses": user_buses,
        "snapshots": snapshots,
        "music": music,
        "reverbZones": reverb_zones,
        "listeners": listeners,
    }))
}

/// Registers `dump_physics[:path]` and `dump_audio[:path]`.
pub(crate) fn register(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_dump(
        "dump_physics",
        og_probe::CommandDesc {
            usage: "dump_physics[:path]".into(),
            doc: "physics state: gravity, body/collider counts (awake/sleeping/static/sensor), tracked bodies with positions and velocities, joints, active solid contacts and sensor overlaps, characters, vehicles, ragdolls, soft bodies, fluids and step timings".into(),
        },
        0,
        move |world, _args| dump_physics(engine_id, world),
    );
    let _ = probe.register_dump(
        "dump_audio",
        og_probe::CommandDesc {
            usage: "dump_audio[:path]".into(),
            doc: "audio state: bus gains, stream hash, active voices with playhead and gain, user buses, snapshots, music clocks, reverb zones, listeners, sound bank residency, recent listen peak/RMS and mixer render stats".into(),
        },
        0,
        move |world, _args| dump_audio(engine_id, world),
    );
}
