//! The mixer's cost for tests/evidence/E6/audio-costs.json: 32 voices (a
//! looped music bed and stems, spatial loops, one-shots retriggered on the
//! beat) through every bus insert (low- and high-pass, the compressor,
//! ducking, the reverb send) and per-voice low-pass (distance and
//! occlusion), rendered headless for 20 s; milliseconds of mixing per second
//! of audio, and the same without effects.
//!
//!     cargo run --release -p og-audio --example mix_costs > tests/evidence/E6/audio-costs.json

use og_audio::{
    AudioBusFx, AudioDesc, AudioListener, AudioSource, Bus, HeadlessMode, HeadlessSink, SoundBank,
    SoundEnded, Transform, install, qoa,
};
use og_core::glam::Vec3;
use og_ecs::{World, WorldDesc};
use std::f64::consts::TAU;
use std::time::Instant;

const RATE: u32 = 48_000;

fn sound(bank: &SoundBank, id: &str, seconds: f64, freq: f64, decay: f64) -> u64 {
    let frames = (seconds * f64::from(RATE)) as usize;
    let samples: Vec<i16> = (0..frames)
        .map(|i| {
            let t = i as f64 / f64::from(RATE);
            (0.4 * (-t / decay).exp() * (TAU * freq * t).sin() * 32767.0) as i16
        })
        .collect();
    bank.insert_qoa(id, &qoa::encode(&samples, 1, RATE).unwrap())
        .unwrap()
}

fn scene(effects: bool) -> (World, HeadlessSink) {
    let bank = SoundBank::new();
    let bed = sound(&bank, "bed", 4.0, 110.0, 1e9);
    let stem = sound(&bank, "stem", 4.0, 220.0, 1e9);
    let hum = sound(&bank, "hum", 2.0, 330.0, 1e9);
    let hit = sound(&bank, "hit", 0.3, 900.0, 0.08);
    let mut world = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 5,
    })
    .unwrap();
    world.register::<Transform>().unwrap();
    world.register::<AudioSource>().unwrap();
    world.register::<SoundEnded>().unwrap();
    let link = install(
        &mut world,
        &AudioDesc {
            bank: Some(bank),
            ..AudioDesc::default()
        },
    )
    .unwrap();
    world
        .spawn((Transform::default(), AudioListener::default()))
        .unwrap();
    let looped = |sound, bus: Bus, volume| AudioSource {
        looping: true,
        bus: bus as u32,
        volume,
        ..AudioSource::new(sound)
    };
    world.spawn((looped(bed, Bus::Music, 0.5),)).unwrap();
    for _ in 0..3 {
        world.spawn((looped(stem, Bus::Music, 0.2),)).unwrap();
    }
    for k in 0..16 {
        let a = k as f32 * 0.39;
        world
            .spawn((
                Transform {
                    position: Vec3::new(8.0 * a.cos(), 0.0, 8.0 * a.sin()),
                    ..Default::default()
                },
                AudioSource {
                    spatial: true,
                    distance_cutoff: if effects { 2500.0 } else { 0.0 },
                    lowpass: if effects && k % 4 == 0 { 1200.0 } else { 0.0 },
                    ..looped(hum, Bus::Sfx, 0.1)
                },
            ))
            .unwrap();
    }
    for k in 0..12 {
        world
            .spawn((AudioSource {
                looping: true,
                loop_start: 0.0,
                loop_end: 0.5 + k as f32 * 0.01,
                ..looped(hit, Bus::Sfx, 0.3)
            },))
            .unwrap();
    }
    if effects {
        world
            .spawn((AudioBusFx {
                bus: Bus::Music as u32,
                lowpass: 4000.0,
                highpass: 60.0,
                duck: true,
                reverb: 0.1,
                ..AudioBusFx::default()
            },))
            .unwrap();
        world
            .spawn((AudioBusFx {
                bus: Bus::Sfx as u32,
                threshold: -18.0,
                ratio: 4.0,
                reverb: 0.3,
                ..AudioBusFx::default()
            },))
            .unwrap();
        world
            .spawn((AudioBusFx {
                bus: Bus::Master as u32,
                threshold: -1.0,
                ratio: 20.0,
                attack: 0.001,
                ..AudioBusFx::default()
            },))
            .unwrap();
    }
    let sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    (world, sink)
}

/// ms of tick + mix per second of audio, over `seconds`.
fn cost(effects: bool, seconds: u32) -> (f64, f64) {
    let (mut world, mut sink) = scene(effects);
    let (mut tick, mut mix) = (0.0, 0.0);
    for _ in 0..seconds * 60 {
        let t = Instant::now();
        world.step().unwrap();
        tick += t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        sink.pump();
        mix += t.elapsed().as_secs_f64() * 1000.0;
    }
    (tick / f64::from(seconds), mix / f64::from(seconds))
}

fn main() {
    let (tick_fx, mix_fx) = cost(true, 20);
    let (tick_dry, mix_dry) = cost(false, 20);
    println!(
        "{{\n  \"voices\": 32,\n  \"rate\": {RATE},\n  \"seconds\": 20,\n  \"tickMsPerSecond.effects\": {tick_fx:.3},\n  \"mixMsPerSecond.effects\": {mix_fx:.3},\n  \"tickMsPerSecond.dry\": {tick_dry:.3},\n  \"mixMsPerSecond.dry\": {mix_dry:.3},\n  \"mixShareOfRealTime.effects\": {:.4}\n}}",
        mix_fx / 1000.0
    );
}
