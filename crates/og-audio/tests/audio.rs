//! The audio command stream through the public boundary: a world with
//! `og_audio::install`, stepped tick by tick, its stream read through a sink.
//!
//! Ways it could fail: the command stream depends on something outside the
//! world (a sink, iteration over a hash map, state in the system closure),
//! is left out of `world_hash`, or does not survive snapshot/restore.

use crate::support;

use og_audio::{AudioStream, HeadlessMode, HeadlessSink, SoundBank};
use og_ecs::World;
use support::*;

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
fn chunked_opus_stream_encoder_matches_whole_decode() {
    use og_audio::opus;
    let channels = 2usize;
    let total_frames = 80_000usize; // 4 Opus blocks (23,040 frames each) -> 2 chunks of 2 blocks
    let pcm: Vec<i16> = (0..total_frames * channels)
        .map(|i| {
            let f = (i / channels) as f64;
            let c = (i % channels) as f64;
            let t = f / f64::from(opus::RATE);
            (10_000.0 * (std::f64::consts::TAU * (330.0 + 110.0 * c) * t).sin()) as i16
        })
        .collect();

    let whole_bytes = opus::encode(&pcm, channels, 96_000, 48_000).unwrap();
    let mut enc = opus::StreamEncoder::new(channels, 96_000).unwrap();
    for piece in pcm.chunks(1700 * channels) {
        enc.push(piece).unwrap();
    }
    let (blocks, encoded_frames) = enc.finish().unwrap();
    assert_eq!(encoded_frames, total_frames as u64);
    let chunks = opus::split_blocks(&blocks, channels, encoded_frames, 48_000, 96_000, 2).unwrap();
    assert_eq!(chunks.len(), 2);

    let render = |chunked: bool| -> Vec<f32> {
        let bank = SoundBank::new();
        let h = if chunked {
            let id = bank
                .insert_chunked_opus("chunked", &chunks[0], 2, chunks.len() as u32)
                .unwrap();
            assert!(bank.put_chunk(id, 1, &chunks[1]).unwrap());
            id
        } else {
            bank.insert_opus("whole", &whole_bytes).unwrap()
        };
        let mut world = World::new(&og_ecs::WorldDesc::default()).unwrap();
        let link = og_audio::install(
            &mut world,
            &og_audio::AudioDesc {
                bank: Some(bank),
                ..Default::default()
            },
        )
        .unwrap();
        world.spawn((og_audio::AudioSource::new(h),)).unwrap();
        let mut sink = HeadlessSink::new(
            &link,
            HeadlessMode::Capture {
                sample_rate: 48_000,
            },
        );
        for _ in 0..100 {
            world.step().unwrap();
            sink.pump();
        }
        sink.pcm().to_vec()
    };
    let whole_pcm = render(false);
    let chunked_pcm = render(true);
    assert_eq!(whole_pcm.len(), 100 * 800 * 2);
    assert!(
        whole_pcm.iter().any(|&s| s.abs() > 0.05),
        "non-silent output"
    );
    assert_eq!(whole_pcm, chunked_pcm);
}
