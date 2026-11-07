//! The audio contract, through the public boundary: a world with
//! `og_audio::install`, stepped tick by tick, its command stream read
//! through a sink, its PCM rendered by the mixer.
//!
//! Ways it could fail, and the test that catches each:
//! - the command stream depends on something outside the world (a sink,
//!   iteration over a hash map, state in the system closure), or is left out
//!   of `world_hash`, or does not survive snapshot/restore →
//!   `command_stream_is_deterministic_and_in_the_world_hash`
//! - a one-shot ends a tick early or late, emits `SoundEnded` twice or never,
//!   keeps `playing`, or the mixer cuts its tail → `a_one_shot_ends_on_its_tick`
//! - a loop wraps to the wrong frame, interpolates across the seam into the
//!   frames after `loopEnd`, or a streamed QOA frame is decoded wrongly at a
//!   frame boundary → `a_loop_crosses_its_seam_without_a_discontinuity`
//! - pitch is ignored by the tick side or by the resampler →
//!   `pitch_two_halves_the_duration`
//! - pan has the wrong sign or does not follow the Transform →
//!   `a_spatial_source_moving_left_to_right_pans_left_to_right`
//! - summed sources clip → `the_limiter_keeps_summed_peaks_at_or_below_one`
//! - the mixer is too slow for real time → `one_second_of_a_busy_mix_renders_in_bounded_time`
//! - a sink attached mid-game misses the music, or a despawned source's
//!   voice plays on → `a_late_sink_hears_the_playing_loop_and_a_despawn_silences_it`

use crate::support;

use og_audio::{
    AudioSource, AudioStream, HeadlessMode, HeadlessSink, Mixer, Op, SoundBank, Transform,
};
use og_core::glam::Vec3;
use og_ecs::World;
use std::time::Instant;
use support::*;

#[test]
fn presentation_feedback_is_audible_without_world_state_or_voice_collisions() {
    let bank = SoundBank::new();
    let sound = insert(&bank, "feedback", 1, &synth(24_000, 1, sine(731.0, 0.2)));
    let (mut world, link) = world_with(&bank, |_| {});
    let (mut control, _) = world_with(&bank, |_| {});
    world.spawn((AudioSource::new(sound),)).unwrap();
    control.spawn((AudioSource::new(sound),)).unwrap();
    link.attach();
    let mut mixer = Mixer::for_link(&link, RATE, og_audio::Clock::Driven);
    let before = world.world_hash();
    link.play_feedback(0, sound, 0.7, 1.3).unwrap();
    assert_eq!(world.world_hash(), before);
    world.step().unwrap();
    control.step().unwrap();
    assert_eq!(world.world_hash(), control.world_hash());
    mixer.pump(&link);
    let mut pcm = vec![0.0; 1602];
    mixer.render(&mut pcm);
    assert!(pcm.iter().any(|sample| sample.abs() > 0.01));
    assert_eq!(
        mixer.voice_count(),
        2,
        "world voice 1 and feedback voice 1 coexist, including past Live"
    );

    link.play_feedback(1, sound, 1.0, 1.0).unwrap();
    link.clear_feedback();
    world.step().unwrap();
    control.step().unwrap();
    assert_eq!(world.world_hash(), control.world_hash());
    mixer.pump(&link);
    mixer.render(&mut pcm);
    assert_eq!(
        mixer.voice_count(),
        2,
        "restore discarded the unheard third voice"
    );
}

/// Ticks a world, pumping the sink after each tick.
fn run(world: &mut World, sink: &mut HeadlessSink, ticks: u64) {
    for _ in 0..ticks {
        world.step().expect("step");
        sink.pump();
    }
}

#[derive(Debug, PartialEq)]
struct Trace {
    stream_hash: u64,
    commands: u64,
    world_hash: u64,
    received_hash: u64,
    /// (tick, stream hash) every 60 ticks.
    checkpoints: Vec<(u64, u64)>,
}

fn trace(options: SceneOptions, ticks: u64, attach: bool) -> Trace {
    let bank = SoundBank::new();
    let sounds = scene_sounds(&bank);
    let (mut world, link) = scene(&bank, sounds, options);
    let mut sink = attach.then(|| HeadlessSink::new(&link, HeadlessMode::Null));
    let mut checkpoints = Vec::new();
    for _ in 0..ticks {
        world.step().unwrap();
        if let Some(s) = &mut sink {
            s.pump();
        }
        let t = world.time().tick;
        if t % 60 == 0 {
            checkpoints.push((t, world.resource::<AudioStream>().unwrap().hash));
        }
    }
    let stream = *world.resource::<AudioStream>().unwrap();
    Trace {
        stream_hash: stream.hash,
        commands: stream.commands,
        world_hash: world.world_hash(),
        received_hash: sink.map_or(0, |s| s.stats().received_hash),
        checkpoints,
    }
}

#[test]
fn command_stream_is_deterministic_and_in_the_world_hash() {
    let a = trace(SceneOptions::default(), 600, true);
    let b = trace(SceneOptions::default(), 600, true);
    assert_eq!(a, b, "two runs of the scripted scene");
    assert!(
        a.commands > 150,
        "the scene emits a stream: {} commands",
        a.commands
    );

    // The stream does not depend on whether a sink listens.
    let silent = trace(SceneOptions::default(), 600, false);
    assert_eq!(
        (silent.stream_hash, silent.world_hash),
        (a.stream_hash, a.world_hash)
    );

    // One different command (the music's volume at tick 300) changes the
    // stream hash from that tick on, and the world hash.
    let changed = trace(
        SceneOptions {
            music_change_at: Some(300),
        },
        600,
        true,
    );
    let split = a.checkpoints.iter().position(|c| c.0 > 300).unwrap();
    assert_eq!(
        a.checkpoints[..split],
        changed.checkpoints[..split],
        "identical before tick 300"
    );
    assert_ne!(
        a.checkpoints[split], changed.checkpoints[split],
        "different after tick 300"
    );
    assert_ne!(
        a.world_hash, changed.world_hash,
        "world_hash covers the stream"
    );

    // A snapshot at tick 300 restored into a fresh world continues the same stream.
    let bank = SoundBank::new();
    let sounds = scene_sounds(&bank);
    let (mut first, _) = scene(&bank, sounds, SceneOptions::default());
    for _ in 0..300 {
        first.step().unwrap();
    }
    let snap = first.snapshot();
    let (mut second, link) = scene(&bank, sounds, SceneOptions::default());
    second.restore(&snap).unwrap();
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Null);
    run(&mut second, &mut sink, 300);
    assert_eq!(second.world_hash(), a.world_hash, "restored halfway");
    assert_eq!(
        second.resource::<AudioStream>().unwrap().hash,
        a.stream_hash
    );

    let report = serde_like(&[
        ("ticks", "600".into()),
        ("commands", a.commands.to_string()),
        ("stream_hash", format!("\"{:016x}\"", a.stream_hash)),
        ("world_hash", format!("\"{:016x}\"", a.world_hash)),
        ("received_hash", format!("\"{:016x}\"", a.received_hash)),
        (
            "second_run_world_hash",
            format!("\"{:016x}\"", b.world_hash),
        ),
        (
            "no_sink_world_hash",
            format!("\"{:016x}\"", silent.world_hash),
        ),
        (
            "restored_at_300_world_hash",
            format!("\"{:016x}\"", second.world_hash()),
        ),
        (
            "music_volume_changed_at_300_world_hash",
            format!("\"{:016x}\"", changed.world_hash),
        ),
        (
            "stream_hash_every_60_ticks",
            format!(
                "[{}]",
                a.checkpoints
                    .iter()
                    .map(|(t, h)| format!("[{t}, \"{h:016x}\"]"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ),
    ]);
    std::fs::write(evidence_dir().join("audio-determinism.json"), report).unwrap();
}

/// A flat JSON object from pre-rendered values.
fn serde_like(fields: &[(&str, String)]) -> String {
    let body: Vec<String> = fields
        .iter()
        .map(|(k, v)| format!("  \"{k}\": {v}"))
        .collect();
    format!("{{\n{}\n}}\n", body.join(",\n"))
}

#[test]
#[ignore = "perf lane"]
fn perf_one_second_of_a_busy_mix_renders_in_bounded_time() {
    // 32 voices: 16 streamed stereo QOA loops at assorted pitches, 16
    // resident mono loops, half of all spatial; rendered in 128-frame
    // quanta like an AudioWorklet pulls them.
    let streamed = SoundBank::with_resident_seconds(0.0);
    let music = synth(10 * RATE as usize, 2, |i, c| {
        0.3 * (i as f64 * if c == 0 { 0.031 } else { 0.029 }).sin()
    });
    let blip = synth(RATE as usize / 2, 1, sine(660.0, 0.3));
    let long = insert(&streamed, "music", 2, &music);
    let short = streamed.insert_pcm("blip", RATE, 1, &blip).unwrap();
    let (mut world, link) = world_with(&streamed, |_| {});
    world
        .spawn((Transform::default(), og_audio::AudioListener::default()))
        .unwrap();
    for i in 0..32u32 {
        world
            .spawn((
                Transform::at(Vec3::new(i as f32 - 16.0, 0.0, -3.0)),
                AudioSource {
                    looping: true,
                    volume: 0.1,
                    pitch: 0.75 + 0.03 * i as f32,
                    spatial: i % 2 == 1,
                    ..AudioSource::new(if i < 16 { long } else { short })
                },
            ))
            .unwrap();
    }
    let mut mixer = Mixer::for_link(&link, RATE, og_audio::Clock::Driven);
    link.attach();
    for _ in 0..300 {
        world.step().unwrap();
        mixer.pump(&link);
    }
    assert!(mixer.frames_until_tick(link.next_tick()) >= 5 * 48_000);
    // Five seconds, timed one at a time; the fastest is the cost (the
    // others include whatever else the machine was doing).
    let mut block = vec![0.0f32; 256];
    let ms = (0..5)
        .map(|_| {
            let t0 = Instant::now();
            for _ in 0..375 {
                mixer.render(&mut block);
            }
            t0.elapsed().as_secs_f64() * 1000.0
        })
        .fold(f64::INFINITY, f64::min);
    assert_eq!(mixer.stats().voices, 32);
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    // Real time is 1000 ms; a release build must leave the Worker nearly all of it.
    let bound = if cfg!(debug_assertions) { 1000.0 } else { 50.0 };
    std::fs::write(
        evidence_dir().join(format!("audio-mix-timing-{profile}.json")),
        serde_like(&[
            ("profile", format!("\"{profile}\"")),
            ("frames", "48000".into()),
            ("sample_rate", "48000".into()),
            ("channels", "2".into()),
            ("voices", "32".into()),
            ("streamed_qoa_voices", "16".into()),
            ("block_frames", "128".into()),
            ("ms", format!("{ms:.3}")),
            ("realtime_fraction", format!("{:.5}", ms / 1000.0)),
            ("bound_ms", format!("{bound}")),
        ]),
    )
    .unwrap();
    assert!(
        ms < bound,
        "1 s of 32 voices took {ms:.1} ms ({profile}; bound {bound} ms)"
    );
}

/// The plays of `sound` in `packets`, and the last packet's live voices.
fn plays(packets: &[og_audio::TickPacket], sound: u64) -> usize {
    packets
        .iter()
        .flat_map(|p| &p.commands)
        .filter(|c| c.op == Op::Play && c.sound == sound)
        .count()
}

/// G2-7: a rollback rewinds the world and runs its ticks again; the sink
/// heard them once. Ways it could fail: the replay plays a one-shot a
/// second time, a play only the mispredicted run made keeps sounding, or
/// the dedupe reaches the world (the stream hash, which lockstep compares,
/// would then depend on whether a sink listened).
#[test]
fn a_rollback_replay_plays_each_sound_once_stops_a_mispredicted_one_and_leaves_the_world_alone() {
    let bank = SoundBank::new();
    let sound = insert(&bank, "hit", 1, &synth(48_000, 1, sine(440.0, 0.4)));
    // The run without a sink, and without any rollback: the reference world.
    let straight = {
        let (mut world, _link) = world_with(&bank, |_| {});
        for t in 0..20 {
            if t == 8 {
                world.spawn((AudioSource::new(sound),)).unwrap();
            }
            world.step().unwrap();
        }
        (
            world.resource::<AudioStream>().unwrap().hash,
            world.world_hash(),
        )
    };
    for predicted_right in [true, false] {
        let (mut world, link) = world_with(&bank, |_| {});
        link.attach();
        let mut heard = Vec::new();
        for _ in 0..5 {
            world.step().unwrap();
            heard.extend(link.drain());
        }
        let back = world.snapshot();
        // The first run: the sound starts at tick 8 (in the mispredicted
        // case, a play the corrected run will not make).
        for t in 5..12 {
            if t == 8 {
                world.spawn((AudioSource::new(sound),)).unwrap();
            }
            world.step().unwrap();
            heard.extend(link.drain());
        }
        assert_eq!(plays(&heard, sound), 1);
        // The rewind to tick 5 and the replay to 12, then on to 20.
        world.restore(&back).unwrap();
        og_audio::restored(&mut world).unwrap();
        link.rewound(5, 12, 6);
        for t in 5..20 {
            if t == 8 && predicted_right {
                world.spawn((AudioSource::new(sound),)).unwrap();
            }
            world.step().unwrap();
            let packets = link.drain();
            assert!(
                packets.iter().all(|p| !p.resync),
                "a rewind is no gap: tick {t}"
            );
            if t == 11 {
                let live = packets
                    .last()
                    .and_then(|p| p.live.clone())
                    .expect("the replay's last tick names the live voices");
                assert_eq!(live.is_empty(), !predicted_right, "{live:?}");
            }
            heard.extend(packets);
        }
        assert_eq!(
            plays(&heard, sound),
            1,
            "predicted right: {predicted_right}"
        );
        if predicted_right {
            assert_eq!(
                (
                    world.resource::<AudioStream>().unwrap().hash,
                    world.world_hash()
                ),
                straight,
                "the world's audio is the straight run's, sink or not"
            );
        }
    }
}
