//! Shared by the audio tests: synthesised sounds (encoded to QOA, as the
//! cook writes them), the scripted scene, and PCM measurements.
#![allow(dead_code)]

use og_audio::qoa;
use og_audio::{
    AudioDesc, AudioLink, AudioListener, AudioSource, SoundBank, SoundEnded, Transform, install,
};
use og_core::glam::Vec3;
use og_core::{Reflect, Rng};
use og_ecs::{QueryDesc, SystemDesc, World, WorldDesc};
use std::f64::consts::TAU;

pub const RATE: u32 = 48_000;

/// Interleaved i16 from a per-frame function of (frame, channel) in [-1, 1].
pub fn synth(frames: usize, channels: usize, f: impl Fn(usize, usize) -> f64) -> Vec<i16> {
    let mut out = Vec::with_capacity(frames * channels);
    for i in 0..frames {
        for c in 0..channels {
            out.push((f(i, c).clamp(-1.0, 1.0) * 32767.0).round() as i16);
        }
    }
    out
}

/// Encodes and inserts PCM as a cooked sound would arrive.
pub fn insert(bank: &SoundBank, id: &str, channels: usize, samples: &[i16]) -> u64 {
    let bytes = qoa::encode(samples, channels, RATE).expect("encode");
    bank.insert_qoa(id, &bytes).expect("insert")
}

pub fn sine(freq: f64, amp: f64) -> impl Fn(usize, usize) -> f64 {
    move |i, _| amp * (TAU * freq * i as f64 / f64::from(RATE)).sin()
}

/// A world with audio installed (after `systems`, which may add the game's own).
pub fn world_with(bank: &SoundBank, systems: impl FnOnce(&mut World)) -> (World, AudioLink) {
    let mut world = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 7,
    })
    .expect("world");
    world.register::<Transform>().unwrap();
    world.register::<AudioSource>().unwrap();
    world.register::<SoundEnded>().unwrap();
    systems(&mut world);
    let link = install(
        &mut world,
        &AudioDesc {
            bank: Some(bank.clone()),
            ..AudioDesc::default()
        },
    )
    .expect("install");
    (world, link)
}

/// Tags the moving spatial source of the scene.
#[derive(Clone, Copy, Debug, Default, Reflect)]
#[reflect(name = "Drone")]
pub struct Drone {}

/// Tags the scene's one-shots, so the script despawns them when they end.
#[derive(Clone, Copy, Debug, Default, Reflect)]
#[reflect(name = "Blip")]
pub struct Blip {}

/// The sounds of the scripted scene.
#[derive(Clone, Copy, Debug)]
pub struct SceneSounds {
    pub music: u64,
    pub blip: u64,
    pub drone: u64,
}

/// Music (3.2 s, streamed, looped from 0.4 s), a blip and a drone tone.
pub fn scene_sounds(bank: &SoundBank) -> SceneSounds {
    // A two-chord pad; stereo with a slightly different right channel.
    let music = synth((3.2 * f64::from(RATE)) as usize, 2, |i, c| {
        let t = i as f64 / f64::from(RATE);
        let root = if t < 1.6 { 220.0 } else { 196.0 };
        let detune = if c == 0 { 1.0 } else { 1.003 };
        let env = (t * 8.0).min(1.0);
        env * 0.18
            * ((TAU * root * detune * t).sin()
                + (TAU * root * 1.25 * t).sin()
                + (TAU * root * 1.5 * detune * t).sin())
    });
    let blip = synth(6000, 1, |i, _| {
        let t = i as f64 / f64::from(RATE);
        let env = (1.0 - t / 0.125).max(0.0).powi(2);
        0.7 * env * (TAU * 880.0 * t).sin().signum() * 0.5
    });
    let drone = synth(RATE as usize, 1, |i, _| {
        let t = i as f64 / f64::from(RATE);
        0.5 * (TAU * 110.0 * t).sin() + 0.2 * (TAU * 330.0 * t).sin()
    });
    SceneSounds {
        music: insert(bank, "music", 2, &music),
        blip: insert(bank, "blip", 1, &blip),
        drone: insert(bank, "drone", 1, &drone),
    }
}

/// What the scene can be told to do differently.
#[derive(Clone, Copy, Debug, Default)]
pub struct SceneOptions {
    /// At this tick the music's volume changes (a one-command difference).
    pub music_change_at: Option<u64>,
}

/// The scripted scene: looping music on the music bus, a spatial drone
/// moving from left to right in front of the listener, and blips at
/// pseudo-random ticks and pitches (the world's Rng), despawned when they end.
pub fn scene(bank: &SoundBank, sounds: SceneSounds, options: SceneOptions) -> (World, AudioLink) {
    let (mut world, link) = world_with(bank, |w| {
        w.register::<Drone>().unwrap();
        w.register::<Blip>().unwrap();
        w.register::<Music>().unwrap();
        w.register::<AudioListener>().unwrap();
        w.add_system(
            SystemDesc::new("script")
                .write::<AudioSource>()
                .write::<Transform>()
                .write::<Blip>()
                .read::<Music>()
                .read::<Drone>()
                .read::<SoundEnded>()
                .write::<Rng>()
                .query(QueryDesc::new().read::<Drone>().write::<Transform>())
                .query(QueryDesc::new().write::<AudioSource>().read::<Music>()),
            move |ctx| {
                let tick = ctx.time().tick;
                for a in ctx.query(0)? {
                    for &e in a.entities() {
                        let x = -8.0 + 16.0 * (tick as f32 / 180.0).min(1.0);
                        ctx.set(e, Transform::at(Vec3::new(x, 0.0, -2.0)))?;
                    }
                }
                if let Some(at) = options.music_change_at
                    && tick == at
                {
                    let music: Vec<_> = ctx.query(1)?.flat_map(|a| a.entities().to_vec()).collect();
                    for e in music {
                        let s = ctx.get::<AudioSource>(e)?;
                        ctx.set(e, AudioSource { volume: 0.45, ..s })?;
                    }
                }
                let fire = ctx.resource_mut::<Rng>()?.next_f64() < 0.06;
                if fire {
                    let pitch = ctx.resource_mut::<Rng>()?.range_f32(0.75, 1.5);
                    let pan = ctx.resource_mut::<Rng>()?.range_f32(-0.6, 0.6);
                    ctx.spawn((
                        AudioSource {
                            pitch,
                            pan,
                            volume: 0.6,
                            ..AudioSource::new(sounds.blip)
                        },
                        Blip {},
                    ))?;
                }
                for ended in ctx.events::<SoundEnded>()? {
                    let e = og_ecs::Entity::from_bits(ended.entity);
                    if ctx.is_alive(e)
                        && ctx
                            .get::<AudioSource>(e)
                            .is_ok_and(|s| s.sound == sounds.blip)
                    {
                        ctx.despawn(e)?;
                    }
                }
                Ok(())
            },
        )
        .unwrap();
    });
    world
        .spawn((Transform::default(), AudioListener::default()))
        .unwrap();
    world
        .spawn((
            AudioSource {
                bus: og_audio::Bus::Music as u32,
                volume: 0.5,
                looping: true,
                loop_start: 0.4,
                ..AudioSource::new(sounds.music)
            },
            Music {},
        ))
        .unwrap();
    world
        .spawn((
            Transform::at(Vec3::new(-8.0, 0.0, -2.0)),
            AudioSource {
                looping: true,
                spatial: true,
                volume: 0.8,
                ref_distance: 2.0,
                ..AudioSource::new(sounds.drone)
            },
            Drone {},
        ))
        .unwrap();
    (world, link)
}

/// Tags the scene's music.
#[derive(Clone, Copy, Debug, Default, Reflect)]
#[reflect(name = "Music")]
pub struct Music {}

/// Left and right channels of interleaved stereo.
pub fn split(pcm: &[f32]) -> (Vec<f32>, Vec<f32>) {
    (
        pcm.iter().step_by(2).copied().collect(),
        pcm.iter().skip(1).step_by(2).copied().collect(),
    )
}

/// The largest jump between consecutive samples of one channel.
pub fn max_step(ch: &[f32]) -> (f32, usize) {
    ch.windows(2)
        .enumerate()
        .map(|(i, w)| ((w[1] - w[0]).abs(), i))
        .fold((0.0, 0), |a, b| if b.0 > a.0 { b } else { a })
}

/// Index of the last sample louder than `threshold` (either channel), in frames.
pub fn last_sound(pcm: &[f32], threshold: f32) -> Option<usize> {
    pcm.as_chunks::<2>()
        .0
        .iter()
        .rposition(|f| f[0].abs() > threshold || f[1].abs() > threshold)
}

/// Index of the first sample louder than `threshold`, in frames.
pub fn first_sound(pcm: &[f32], threshold: f32) -> Option<usize> {
    pcm.as_chunks::<2>()
        .0
        .iter()
        .position(|f| f[0].abs() > threshold || f[1].abs() > threshold)
}

pub fn rms(ch: &[f32]) -> f32 {
    (ch.iter().map(|x| x * x).sum::<f32>() / ch.len().max(1) as f32).sqrt()
}

/// Where scratch evidence goes: `$OG_EVIDENCE_DIR/M6/` (default target/evidence/M6).
pub fn evidence_dir() -> std::path::PathBuf {
    let base = std::env::var_os("OG_EVIDENCE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/evidence")
        });
    let dir = base.join("M6");
    std::fs::create_dir_all(&dir).expect("evidence dir");
    dir
}
