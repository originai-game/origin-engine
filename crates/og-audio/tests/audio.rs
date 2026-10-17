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

mod support;

use og_audio::{
    end_reason, AudioPlayback, AudioSource, AudioStream, HeadlessMode, HeadlessSink, Mixer,
    MixerDesc, Op, SoundBank, SoundEnded, SoundStarted, Transform,
};
use og_core::glam::Vec3;
use og_ecs::World;
use std::time::Instant;
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
fn a_one_shot_ends_on_its_tick() {
    let bank = SoundBank::new();
    // 0.5 s at 48 kHz: 24000 frames, 800 per tick at 60 Hz, so 30 ticks.
    let tone = synth(24_000, 1, sine(500.0, 0.5));
    let sound = insert(&bank, "tone", 1, &tone);
    let (mut world, link) = world_with(&bank, |_| {});
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    run(&mut world, &mut sink, 10);
    let e = world
        .spawn((AudioSource {
            pan: -1.0,
            ..AudioSource::new(sound)
        },))
        .unwrap();
    let mut ended_at = None;
    let mut started = 0;
    let mut ended = 0;
    for _ in 0..60 {
        let tick = world.time().tick;
        world.step().unwrap();
        sink.pump();
        started += world.pending_events::<SoundStarted>().unwrap().len();
        let ev = world.pending_events::<SoundEnded>().unwrap();
        if let Some(end) = ev.first() {
            assert_eq!(
                (end.entity, end.reason, end.sound),
                (e.to_bits(), end_reason::FINISHED, sound)
            );
            ended_at.get_or_insert(tick);
        }
        ended += ev.len();
    }
    assert_eq!((started, ended), (1, 1), "one start, one end");
    assert_eq!(
        ended_at,
        Some(10 + 29),
        "started in tick 10, 30 ticks long: ends in tick 39"
    );
    assert!(
        !world.get::<AudioSource>(e).unwrap().playing,
        "playing reset"
    );
    assert!(!world.has::<AudioPlayback>(e).unwrap(), "playhead removed");

    // The PCM: silent before frame 8000 (tick 10), the tone for 24000
    // frames, silent after.
    let pcm = sink.pcm();
    let (first, last) = (
        first_sound(pcm, 1e-3).unwrap(),
        last_sound(pcm, 1e-3).unwrap(),
    );
    assert!(
        (8000..8004).contains(&first),
        "starts at tick 10's frame: {first}"
    );
    assert!(
        (31_990..32_000).contains(&last),
        "ends 24000 frames later: {last}"
    );
    let (l, _) = split(&pcm[8000 * 2..32_000 * 2]);
    assert!(
        rms(&l) > 0.3,
        "the tone plays at full level on the left: {}",
        rms(&l)
    );
}

#[test]
fn a_loop_crosses_its_seam_without_a_discontinuity() {
    // A 375 Hz sine has a period of exactly 128 frames, so frames
    // [15360, 53760) are 300 whole periods: the loop region is continuous
    // across its seam. Before it, the same sine (an intro); after it, DC at
    // 0.9 that a correct loop never plays.
    let (loop_start, loop_end) = (15_360usize, 53_760usize);
    let samples = synth(loop_end + 9_600, 1, |i, _| {
        if i < loop_end {
            0.5 * (std::f64::consts::TAU * i as f64 / 128.0).sin()
        } else {
            0.9
        }
    });
    for (streamed, pitch) in [(true, 1.0f32), (false, 1.0), (true, 1.37)] {
        // A bank that streams everything: the loop wraps across QOA frames
        // (loopEnd is 10.5 frames of 5120 in).
        let bank = if streamed {
            SoundBank::with_resident_seconds(0.0)
        } else {
            SoundBank::new()
        };
        let sound = insert(&bank, "loop", 1, &samples);
        assert_eq!(bank.get(sound).unwrap().streamed(), streamed);
        let (mut world, link) = world_with(&bank, |_| {});
        world
            .spawn((AudioSource {
                looping: true,
                loop_start: loop_start as f32 / RATE as f32,
                loop_end: loop_end as f32 / RATE as f32,
                pan: -1.0,
                pitch,
                ..AudioSource::new(sound)
            },))
            .unwrap();
        let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
        run(&mut world, &mut sink, 240); // 4 s: three wraps at pitch 1, four at 1.37
        let (l, r) = split(sink.pcm());
        assert!(
            r.iter().all(|x| x.abs() < 1e-6),
            "pan -1 puts a mono source on the left only"
        );
        // The steepest a 0.5-amplitude sine at this rate and pitch moves,
        // plus QOA's quantisation noise.
        let slope = (std::f64::consts::TAU / 128.0 * 0.5 * f64::from(pitch)) as f32;
        let (step, at) = max_step(&l);
        assert!(
            step < slope * 1.35,
            "streamed {streamed}, pitch {pitch}: a jump of {step} at frame {at} (slope {slope})"
        );
        assert!(
            l.iter().all(|x| x.abs() < 0.6),
            "the DC after loopEnd never plays"
        );
        if pitch == 1.0 {
            // Every frame is where an unbroken sine would be: a wrap one
            // frame off accumulates a phase error the tolerance does not allow.
            let worst = l
                .iter()
                .enumerate()
                .map(|(n, x)| (x - 0.5 * (std::f32::consts::TAU * n as f32 / 128.0).sin()).abs())
                .fold(0.0f32, f32::max);
            assert!(
                worst < 0.03,
                "streamed {streamed}: {worst} from the ideal sine"
            );
        }
        assert!(world.pending_events::<SoundEnded>().unwrap().is_empty());
    }
}

#[test]
fn pitch_two_halves_the_duration() {
    let bank = SoundBank::new();
    let sound = insert(&bank, "tone", 1, &synth(24_000, 1, sine(300.0, 0.5)));
    let mut measured = Vec::new();
    for pitch in [1.0f32, 2.0] {
        let (mut world, link) = world_with(&bank, |_| {});
        let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
        world
            .spawn((AudioSource {
                pitch,
                ..AudioSource::new(sound)
            },))
            .unwrap();
        let mut end_tick = None;
        for _ in 0..60 {
            let tick = world.time().tick;
            world.step().unwrap();
            sink.pump();
            if !world.pending_events::<SoundEnded>().unwrap().is_empty() {
                end_tick.get_or_insert(tick);
            }
        }
        let pcm = sink.pcm();
        let len = last_sound(pcm, 1e-3).unwrap() + 1 - first_sound(pcm, 1e-3).unwrap();
        measured.push((end_tick.unwrap() + 1, len));
    }
    assert_eq!(measured[0].0, 30, "pitch 1: 30 ticks");
    assert_eq!(measured[1].0, 15, "pitch 2: 15 ticks");
    assert!(
        (23_990..=24_000).contains(&measured[0].1),
        "pitch 1 lasts 24000 frames: {:?}",
        measured
    );
    assert!(
        (11_990..=12_000).contains(&measured[1].1),
        "pitch 2 lasts 12000 frames: {:?}",
        measured
    );
}

#[test]
fn a_spatial_source_moving_left_to_right_pans_left_to_right() {
    let bank = SoundBank::new();
    let sound = insert(&bank, "hum", 1, &synth(48_000, 1, sine(220.0, 0.5)));
    let (mut world, link) = world_with(&bank, |_| {});
    world
        .spawn((Transform::default(), og_audio::AudioListener::default()))
        .unwrap();
    let e = world
        .spawn((
            Transform::at(Vec3::new(-10.0, 0.0, -2.0)),
            AudioSource {
                looping: true,
                spatial: true,
                ref_distance: 20.0,
                ..AudioSource::new(sound)
            },
        ))
        .unwrap();
    let mut mixer = Mixer::for_link(&link, RATE, og_audio::Clock::Driven);
    link.attach();
    let mut pcm = Vec::new();
    let mut pans = Vec::new();
    for t in 0..120u64 {
        let x = -10.0 + 20.0 * t as f32 / 119.0;
        world
            .insert(e, Transform::at(Vec3::new(x, 0.0, -2.0)))
            .unwrap();
        world.step().unwrap();
        for p in link.drain() {
            pans.extend(
                p.commands
                    .iter()
                    .filter(|c| c.voice != 0 && matches!(c.op, Op::Play | Op::Params))
                    .map(|c| c.pan),
            );
            mixer.push(p);
        }
        mixer.render_until_tick(link.next_tick(), usize::MAX, &mut pcm);
    }
    assert!(
        pans.len() >= 119,
        "a moving source sends params every tick: {}",
        pans.len()
    );
    assert!(
        pans.windows(2).all(|w| w[1] >= w[0]),
        "pan only moves right"
    );
    assert!(
        pans[0] < -0.95 && *pans.last().unwrap() > 0.95,
        "from hard left to hard right: {:?}",
        (pans[0], pans.last())
    );
    let (l, r) = split(&pcm);
    let q = l.len() / 4;
    let (l0, r0) = (rms(&l[..q]), rms(&r[..q]));
    let (l3, r3) = (rms(&l[3 * q..]), rms(&r[3 * q..]));
    let (lm, rm) = (rms(&l[q * 3 / 2..q * 5 / 2]), rms(&r[q * 3 / 2..q * 5 / 2]));
    assert!(l0 > 3.0 * r0, "first quarter on the left: L {l0} R {r0}");
    assert!(r3 > 3.0 * l3, "last quarter on the right: L {l3} R {r3}");
    assert!(
        (lm / rm - 1.0).abs() < 0.35,
        "the middle is centred: L {lm} R {rm}"
    );
}

#[test]
fn the_limiter_keeps_summed_peaks_at_or_below_one() {
    let bank = SoundBank::new();
    let freqs = [110.0, 165.0, 220.0, 275.0, 330.0, 440.0, 550.0, 660.0];
    let sounds: Vec<u64> = freqs
        .iter()
        .map(|f| insert(&bank, "full", 1, &synth(48_000, 1, sine(*f, 1.0))))
        .collect();
    let mut peaks = Vec::new();
    for limiter in [true, false] {
        let (mut world, link) = world_with(&bank, |_| {});
        for &s in &sounds {
            world
                .spawn((AudioSource {
                    looping: true,
                    volume: 1.0,
                    ..AudioSource::new(s)
                },))
                .unwrap();
        }
        let mut mixer = Mixer::new(
            MixerDesc {
                limiter,
                ..MixerDesc::default()
            },
            bank.clone(),
        );
        link.attach();
        let mut pcm = Vec::new();
        for _ in 0..60 {
            world.step().unwrap();
            mixer.pump(&link);
            mixer.render_until_tick(link.next_tick(), usize::MAX, &mut pcm);
        }
        let peak = pcm.iter().fold(0.0f32, |m, x| m.max(x.abs()));
        peaks.push(peak);
        if limiter {
            assert!(peak <= 1.0, "limited peak {peak}");
            assert!(mixer.stats().limited > 0);
            let (l, _) = split(&pcm);
            assert!(rms(&l) > 0.25, "limited, not silenced: rms {}", rms(&l));
        }
    }
    assert!(
        peaks[1] > 2.0,
        "the sum without the limiter peaks at {}",
        peaks[1]
    );
}

#[test]
fn one_second_of_a_busy_mix_renders_in_bounded_time() {
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

#[test]
fn a_late_sink_hears_the_playing_loop_and_a_despawn_silences_it() {
    let bank = SoundBank::new();
    let sound = insert(&bank, "pad", 1, &synth(48_000, 1, sine(330.0, 0.4)));
    let (mut world, link) = world_with(&bank, |_| {});
    let e = world
        .spawn((AudioSource {
            looping: true,
            ..AudioSource::new(sound)
        },))
        .unwrap();
    for _ in 0..100 {
        world.step().unwrap();
    }
    let hash_before = world.resource::<AudioStream>().unwrap().hash;
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    run(&mut world, &mut sink, 30);
    assert_eq!(
        sink.stats().resyncs,
        1,
        "the first packet after attach is a resync"
    );
    let pcm = sink.pcm().to_vec();
    let first = first_sound(&pcm, 1e-3).unwrap();
    assert!(
        first < 16,
        "the loop is heard from the start (fading in): first sound at {first}"
    );
    let (l, _) = split(&pcm[..800 * 2]);
    assert!(
        rms(&l) > 0.15,
        "at level within the first tick: rms {}",
        rms(&l)
    );
    assert_eq!(
        world.resource::<AudioStream>().unwrap().hash,
        hash_before,
        "attaching changed nothing in the hashed stream (the loop sent nothing new)"
    );
    world.despawn(e).unwrap();
    run(&mut world, &mut sink, 30);
    let pcm = sink.pcm();
    let after = 30 * 800;
    let last = last_sound(pcm, 1e-3).unwrap();
    assert!(
        last < after + 800 + og_audio::mixer::FADE_FRAMES as usize,
        "silent within a tick and a fade of the despawn: last sound at {last}, despawn at {after}"
    );
    assert_eq!(sink.mixer().unwrap().voice_count(), 0);
}

/// Renders the scripted scene's first 2.5 s to a WAV and draws its waveform
/// (evidence, `$OG_EVIDENCE_DIR/M6/audio-scene.{wav,png}`), and checks it
/// is what the scene is: music from the first frame, the drone crossing
/// from left to right, no clipping.
#[test]
fn the_scripted_scene_renders_to_a_wav() {
    let bank = SoundBank::new();
    let sounds = scene_sounds(&bank);
    let (mut world, link) = scene(&bank, sounds, SceneOptions::default());
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    run(&mut world, &mut sink, 150);
    let pcm = sink.pcm().to_vec();
    assert_eq!(pcm.len(), 150 * 800 * 2);
    assert!(
        first_sound(&pcm, 1e-4).unwrap() < 200,
        "music and drone start at frame 0"
    );
    assert!(pcm.iter().all(|x| x.abs() <= 1.0));
    let stats = sink.stats();
    assert!(stats.plays >= 5, "music, drone and blips: {stats:?}");
    let dir = evidence_dir();
    std::fs::write(dir.join("audio-scene.wav"), og_audio::wav(&pcm, 2, RATE)).unwrap();

    // Waveform: left above, right below; min/max per column.
    let (w, h) = (1200u32, 320u32);
    let mut img = image::RgbImage::from_pixel(w, h, image::Rgb([250, 250, 248]));
    let (l, r) = split(&pcm);
    for (lane, ch) in [&l, &r].iter().enumerate() {
        let mid = (h / 4 + lane as u32 * h / 2) as f32;
        let per = ch.len() / w as usize;
        for x in 0..w {
            let col = &ch[x as usize * per..(x as usize + 1) * per];
            let (lo, hi) = col
                .iter()
                .fold((0.0f32, 0.0f32), |(a, b), v| (a.min(*v), b.max(*v)));
            let y0 = (mid - hi * (h as f32 / 4.0 - 4.0)) as u32;
            let y1 = (mid - lo * (h as f32 / 4.0 - 4.0)) as u32;
            for y in y0..=y1.min(h - 1) {
                img.put_pixel(x, y, image::Rgb([30, 30, 34]));
            }
        }
        for x in 0..w {
            img.put_pixel(x, mid as u32, image::Rgb([180, 180, 176]));
        }
    }
    img.save(dir.join("audio-scene.png")).unwrap();
}
