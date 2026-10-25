//! What each propagation and voice feature costs the mixer, on one fixed
//! scene: 16 spatial loops circling a listener (moving, so every tick
//! sends their gain and pan) and 8 non-spatial loops, 24 voices, at 48 kHz
//! in 128-frame quanta. Each variant turns one feature on over the base
//! (every feature off, the linear resampler); milliseconds of mixing per
//! second of audio, the fastest of five seconds, and the world step's
//! microseconds per tick.
//!
//!     cargo run -p og-audio --example feature_costs
//!
//! The numbers in tests/evidence/r4/audio-propagation/NOTES.md are this
//! example in the dev profile (og-audio at opt-level 0), so they are
//! ratios to the base more than real-time costs; the release mixer is
//! measured at a wave's end.

use og_audio::schema::panning_model;
use og_audio::{
    AudioDesc, AudioListener, AudioSource, Clock, Mixer, SoundBank, Transform, install, qoa,
};
use og_core::glam::Vec3;
use og_ecs::{Entity, World, WorldDesc};
use std::f64::consts::TAU;
use std::time::Instant;

const RATE: u32 = 48_000;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Variant {
    Base,
    Doppler,
    Cones,
    Hrtf,
    /// The windowed sinc, with Doppler so the spatial voices resample.
    Sinc,
    /// maxVoices 12, every source virtualize: 12 heard, 12 virtual.
    Virtual,
    All,
}

fn sound(bank: &SoundBank, id: &str, freq: f64) -> u64 {
    let samples: Vec<i16> = (0..RATE as usize * 2)
        .map(|i| (0.3 * (TAU * freq * i as f64 / f64::from(RATE)).sin() * 32767.0) as i16)
        .collect();
    bank.insert_qoa(id, &qoa::encode(&samples, 1, RATE).unwrap())
        .unwrap()
}

/// (mixer ms per second of audio, tick-side µs per tick).
fn measure(v: Variant) -> (f64, f64) {
    let bank = SoundBank::new();
    let tones: Vec<u64> = (0..4)
        .map(|k| sound(&bank, &format!("t{k}"), 220.0 * (k + 1) as f64))
        .collect();
    let mut world = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 3,
    })
    .unwrap();
    world.register::<Transform>().unwrap();
    world.register::<AudioListener>().unwrap();
    let link = install(
        &mut world,
        &AudioDesc {
            bank: Some(bank),
            ..AudioDesc::default()
        },
    )
    .unwrap();
    let all = v == Variant::All;
    {
        let a = world.resource_mut::<og_audio::Audio>().unwrap();
        a.resampler = if v == Variant::Sinc || all {
            og_audio::schema::resampler::SINC
        } else {
            og_audio::schema::resampler::LINEAR
        };
        if v == Variant::Virtual {
            a.max_voices = 12;
        }
    }
    world
        .spawn((
            Transform::default(),
            AudioListener {
                panning_model: if v == Variant::Hrtf || all {
                    panning_model::HRTF
                } else {
                    panning_model::EQUALPOWER
                },
                ..AudioListener::default()
            },
        ))
        .unwrap();
    let spatial: Vec<Entity> = (0..16)
        .map(|k| {
            let cones = v == Variant::Cones || all;
            world
                .spawn((
                    Transform::at(Vec3::new(8.0, 0.0, 0.0)),
                    AudioSource {
                        looping: true,
                        spatial: true,
                        volume: 0.1,
                        ref_distance: 2.0,
                        virtualize: v == Variant::Virtual,
                        doppler_factor: if matches!(v, Variant::Doppler | Variant::Sinc) || all {
                            1.0
                        } else {
                            0.0
                        },
                        cone_inner_angle: if cones { 90.0 } else { 360.0 },
                        cone_outer_angle: if cones { 240.0 } else { 360.0 },
                        cone_outer_gain: 0.2,
                        ..AudioSource::new(tones[k % 4])
                    },
                ))
                .unwrap()
        })
        .collect();
    for k in 0..8 {
        world
            .spawn((AudioSource {
                looping: true,
                volume: 0.05,
                pan: k as f32 / 4.0 - 1.0,
                virtualize: v == Variant::Virtual,
                ..AudioSource::new(tones[k % 4])
            },))
            .unwrap();
    }
    let mut mixer = Mixer::for_link(&link, RATE, Clock::Driven);
    link.attach();
    let mut ticking = 0.0;
    for t in 0..330u64 {
        for (k, &e) in spatial.iter().enumerate() {
            let a = t as f32 / 60.0 * (0.5 + k as f32 * 0.1) + k as f32;
            world
                .insert(
                    e,
                    Transform::at(Vec3::new(8.0 * a.cos(), 0.0, 8.0 * a.sin())),
                )
                .unwrap();
        }
        let t0 = Instant::now();
        world.step().unwrap();
        ticking += t0.elapsed().as_secs_f64();
        mixer.pump(&link);
    }
    let mut block = vec![0.0f32; 256];
    let mix = (0..5)
        .map(|_| {
            let t0 = Instant::now();
            for _ in 0..375 {
                mixer.render(&mut block);
            }
            t0.elapsed().as_secs_f64() * 1000.0
        })
        .fold(f64::INFINITY, f64::min);
    (mix, ticking / 330.0 * 1e6)
}

fn main() {
    let profile = if cfg!(debug_assertions) {
        "dev"
    } else {
        "release"
    };
    let (base, base_tick) = measure(Variant::Base);
    println!("{{\n  \"profile\": \"{profile}\",\n  \"voices\": 24,\n  \"spatial\": 16,");
    println!("  \"base\": {{ \"ms_per_s\": {base:.2}, \"tick_us\": {base_tick:.1} }},");
    let variants = [
        ("doppler", Variant::Doppler),
        ("cones", Variant::Cones),
        ("hrtf", Variant::Hrtf),
        ("sinc", Variant::Sinc),
        ("virtual", Variant::Virtual),
        ("all", Variant::All),
    ];
    for (i, (name, v)) in variants.iter().enumerate() {
        let (ms, tick) = measure(*v);
        let comma = if i + 1 < variants.len() { "," } else { "" };
        println!(
            "  \"{name}\": {{ \"ms_per_s\": {ms:.2}, \"over_base\": {:.3}, \"tick_us\": {tick:.1} }}{comma}",
            ms / base
        );
    }
    println!("}}");
}
