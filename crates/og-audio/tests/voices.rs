//! Resampling, voice virtualisation and retrigger, through the public
//! boundary (a world with `og_audio::install`, its stream read from the link
//! or rendered by a capture sink).
//!
//! Ways it could fail, and the test that catches each:
//! - the windowed sinc is not used, leaks images, dulls the highs, or the
//!   linear option is not kept → `the_windowed_sinc_resamples_without_images_and_linear_is_kept`
//! - a virtual voice restarts instead of resuming, is heard while virtual,
//!   loses its playhead, never comes back, or a virtual one-shot ends on
//!   another tick → `a_virtual_voice_resumes_where_it_would_have_been`
//! - retrigger needs a level change, keeps the old voice, does not restart
//!   from the top, leaves the flag set, or a stopped source ignores it →
//!   `retrigger_restarts_a_sound_without_a_level_change`
//! - the new state is outside the world, so a second run or a restore
//!   mid-run differs → `a_run_with_sinc_virtual_voices_and_retriggers_is_the_same_twice_and_across_a_restore`

use crate::support;

use og_audio::command::Op;
use og_audio::schema::resampler;
use og_audio::{
    Audio, AudioPlayback, AudioSource, AudioStream, HeadlessMode, HeadlessSink, SoundBank,
    SoundEnded, SoundStarted, end_reason,
};
use og_ecs::{Entity, World};
use std::f64::consts::TAU;
use support::*;

/// Goertzel magnitude of `freq` in a channel at `rate`, as an amplitude.
fn tone(ch: &[f32], freq: f64, rate: f64) -> f64 {
    let n = ch.len() as f64;
    let w = TAU * freq / rate;
    let c = 2.0 * w.cos();
    let (mut s1, mut s2) = (0.0f64, 0.0f64);
    for &x in ch {
        let s0 = f64::from(x) + c * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - c * s1 * s2).max(0.0).sqrt() * 2.0 / n
}

fn db(x: f64) -> f64 {
    20.0 * x.max(1e-12).log10()
}

fn run(world: &mut World, sink: &mut HeadlessSink, ticks: u64) {
    for _ in 0..ticks {
        world.step().unwrap();
        sink.pump();
    }
}

#[test]
fn the_windowed_sinc_resamples_without_images_and_linear_is_kept() {
    // A 15 kHz sine recorded at 44.1 kHz, played at 48 kHz: resampled at a
    // step of 0.919. Linear interpolation leaves its image at 44.1 − 15 =
    // 29.1 kHz, which folds to 48 − 29.1 = 18.9 kHz, and dulls the tone.
    let src_rate = 44_100u32;
    let samples: Vec<i16> = (0..src_rate as usize * 2)
        .map(|i| {
            (0.5 * (TAU * 15_000.0 * i as f64 / f64::from(src_rate)).sin() * 32767.0).round() as i16
        })
        .collect();
    let hear = |mode: u32, pitch: f32, bank: &SoundBank, sound: u64| {
        let (mut world, link) = world_with(bank, |_| {});
        world.resource_mut::<Audio>().unwrap().resampler = mode;
        world
            .spawn((AudioSource {
                looping: true,
                pan: -1.0,
                pitch,
                ..AudioSource::new(sound)
            },))
            .unwrap();
        let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
        run(&mut world, &mut sink, 60);
        let (l, _) = split(sink.pcm());
        l[9_600..48_000].to_vec()
    };
    let bank = SoundBank::new();
    let sound = bank.insert_pcm("hf", src_rate, 1, &samples).unwrap();
    let rate = f64::from(RATE);
    let sinc = hear(resampler::SINC, 1.0, &bank, sound);
    let linear = hear(resampler::LINEAR, 1.0, &bank, sound);
    let (s_tone, s_image) = (tone(&sinc, 15_000.0, rate), tone(&sinc, 18_900.0, rate));
    let (l_tone, l_image) = (tone(&linear, 15_000.0, rate), tone(&linear, 18_900.0, rate));
    // pan −1 puts the mono source on the left at full gain.
    assert!(
        (db(s_tone) - db(0.5)).abs() < 0.5,
        "sinc keeps the tone: {:.2} dB",
        db(s_tone / 0.5)
    );
    assert!(
        db(s_image / s_tone) < -60.0,
        "sinc: the image at {:.1} dB",
        db(s_image / s_tone)
    );
    assert!(
        db(l_tone / 0.5) < -2.0 && db(l_image / l_tone) > -30.0,
        "linear is kept as it was: tone {:.2} dB, image {:.1} dB",
        db(l_tone / 0.5),
        db(l_image / l_tone)
    );

    // Pitched up 2.5×: a 6 kHz tone at 48 kHz becomes 15 kHz, and its
    // linear image (48 − 6 = 42 kHz, ×2.5 = 105 kHz) folds to 105 − 96 =
    // 9 kHz; the widened sinc keeps it out.
    let six: Vec<i16> = (0..RATE as usize * 2)
        .map(|i| (0.5 * (TAU * 6_000.0 * i as f64 / rate).sin() * 32767.0).round() as i16)
        .collect();
    let up = bank.insert_pcm("six", RATE, 1, &six).unwrap();
    let sinc = hear(resampler::SINC, 2.5, &bank, up);
    let linear = hear(resampler::LINEAR, 2.5, &bank, up);
    let s = db(tone(&sinc, 9_000.0, rate) / tone(&sinc, 15_000.0, rate));
    let l = db(tone(&linear, 9_000.0, rate) / tone(&linear, 15_000.0, rate));
    assert!(s < -50.0, "sinc pitched up: the folded image at {s:.1} dB");
    assert!(l > -40.0, "linear pitched up: {l:.1} dB");

    // Unpitched at the output's rate, both play the sound's own samples.
    let same = bank.insert_pcm("same", RATE, 1, &six).unwrap();
    assert_eq!(
        hear(resampler::SINC, 1.0, &bank, same),
        hear(resampler::LINEAR, 1.0, &bank, same),
        "on its own frames the sinc is the frame itself"
    );
}

/// A 2 s, 500 Hz sine whose amplitude rises linearly from 0 to 0.8: its
/// level says where in the sound a window is.
fn swell() -> Vec<i16> {
    synth(2 * RATE as usize, 1, |i, _| {
        let t = i as f64 / f64::from(RATE);
        0.4 * t * (TAU * 500.0 * t).sin()
    })
}

#[test]
fn a_virtual_voice_resumes_where_it_would_have_been() {
    let bank = SoundBank::new();
    let rising = insert(&bank, "swell", 1, &swell());
    let hum = insert(&bank, "hum", 1, &synth(RATE as usize, 1, sine(300.0, 0.2)));
    let blip = insert(
        &bank,
        "blip",
        1,
        &synth(RATE as usize / 2, 1, sine(700.0, 0.2)),
    );
    let (mut world, link) = world_with(&bank, |_| {});
    world.resource_mut::<Audio>().unwrap().max_voices = 2;
    let source = |sound, priority, looping, virtualize| AudioSource {
        priority,
        looping,
        virtualize,
        ..AudioSource::new(sound)
    };
    let _a = world.spawn((source(hum, 200, true, false),)).unwrap();
    let b = world.spawn((source(rising, 100, false, true),)).unwrap();
    let c = world.spawn((source(hum, 150, true, false),)).unwrap();
    // A half-second one-shot that never stands within the cap.
    let d = world.spawn((source(blip, 10, false, true),)).unwrap();
    let mut mixer = og_audio::Mixer::for_link(&link, RATE, og_audio::Clock::Driven);
    link.attach();
    let mut pcm = Vec::new();
    let (mut b_voice, mut d_voice, mut d_end) = (0, 0, None);
    let mut heard_b = Vec::new();
    for t in 0..60u64 {
        if t == 30 {
            // B outranks C now: C (no virtualize) is culled, B is heard.
            let s = world.get::<AudioSource>(b).unwrap();
            world.insert(b, AudioSource { priority: 255, ..s }).unwrap();
        }
        world.step().unwrap();
        for p in link.drain() {
            heard_b.extend(
                p.commands
                    .iter()
                    .filter(|c| c.voice == b_voice && c.op == Op::Play)
                    .map(|c| (p.tick, c.position)),
            );
            assert!(
                d_voice == 0 || p.commands.iter().all(|c| c.voice != d_voice),
                "a virtual voice is never sent"
            );
            mixer.push(p);
        }
        mixer.render_until_tick(link.next_tick(), usize::MAX, &mut pcm);
        for s in world.pending_events::<SoundStarted>().unwrap() {
            if s.entity == b.to_bits() {
                b_voice = s.voice;
            }
            if s.entity == d.to_bits() {
                d_voice = s.voice;
            }
        }
        for e in world.pending_events::<SoundEnded>().unwrap() {
            assert_ne!(e.entity, b.to_bits(), "B is never culled: {e:?}");
            if e.entity == d.to_bits() {
                d_end = Some((t, e.reason));
            }
        }
        if t < 30 {
            let p = world.get::<AudioPlayback>(b).unwrap();
            assert!(p.is_virtual, "tick {t}: B is virtual");
            assert_eq!(p.cursor, (t + 1) as f64 * 800.0, "its playhead runs on");
        }
    }
    assert!(b_voice != 0 && d_voice != 0, "both started (virtually)");
    assert_eq!(
        heard_b,
        vec![(30, 24_000.0)],
        "B is played once, at tick 30, from where it would be"
    );
    assert!(!world.get::<AudioPlayback>(b).unwrap().is_virtual);
    assert!(!world.get::<AudioSource>(c).unwrap().playing, "C culled");
    assert_eq!(
        d_end,
        Some((29, end_reason::FINISHED)),
        "a virtual one-shot ends on its tick"
    );
    // Heard from frame 24 000 of its swell (level about 0.2 there, 0 at
    // its top): a window a tick in has the level the sound has there.
    let (l, _) = split(&pcm);
    // Windows of 7680 frames: whole periods of the hum (300 Hz) and the
    // swell (500 Hz), so neither leaks into the other's measure.
    let quiet = tone(&l[12_000..19_680], 500.0, f64::from(RATE));
    let at = tone(&l[25_600..33_280], 500.0, f64::from(RATE));
    let want = 0.4 * (29_440.0 / 48_000.0) * std::f64::consts::FRAC_1_SQRT_2;
    assert!(quiet < 1e-3, "not heard while virtual: {quiet}");
    assert!(
        (at - want).abs() < 0.1 * want,
        "resumed at its playhead: level {at:.4}, the swell has {want:.4} there"
    );
}

#[test]
fn retrigger_restarts_a_sound_without_a_level_change() {
    let bank = SoundBank::new();
    let rising = insert(&bank, "swell", 1, &swell());
    let (mut world, link) = world_with(&bank, |_| {});
    let e = world
        .spawn((AudioSource {
            looping: true,
            ..AudioSource::new(rising)
        },))
        .unwrap();
    link.attach();
    for _ in 0..20 {
        world.step().unwrap();
    }
    link.drain();
    let old = world.get::<AudioPlayback>(e).unwrap().voice;
    let s = world.get::<AudioSource>(e).unwrap();
    world
        .insert(
            e,
            AudioSource {
                retrigger: true,
                ..s
            },
        )
        .unwrap();
    world.step().unwrap();
    let ended = world.pending_events::<SoundEnded>().unwrap();
    let started = world.pending_events::<SoundStarted>().unwrap();
    assert_eq!(
        ended
            .iter()
            .map(|x| (x.voice, x.reason))
            .collect::<Vec<_>>(),
        vec![(old, end_reason::RETRIGGERED)]
    );
    assert_eq!(started.len(), 1);
    let new = started[0].voice;
    assert_ne!(new, old, "a new voice");
    let ops: Vec<(Op, u32, f64)> = link
        .drain()
        .iter()
        .flat_map(|p| p.commands.clone())
        .map(|c| (c.op, c.voice, c.position))
        .collect();
    assert_eq!(
        ops,
        vec![(Op::Stop, old, 0.0), (Op::Play, new, 0.0)],
        "in one tick: the old voice stops, the new plays from the top"
    );
    let s = world.get::<AudioSource>(e).unwrap();
    assert!(
        s.playing && !s.retrigger,
        "the flag is cleared, playing kept"
    );
    assert_eq!(world.get::<AudioPlayback>(e).unwrap().cursor, 800.0);

    // A one-shot that ended (playing false) plays again on retrigger alone.
    let blip = insert(&bank, "blip", 1, &synth(4_000, 1, sine(700.0, 0.2)));
    let f = world.spawn((AudioSource::new(blip),)).unwrap();
    for _ in 0..10 {
        world.step().unwrap();
    }
    assert!(!world.get::<AudioSource>(f).unwrap().playing, "it ended");
    let s = world.get::<AudioSource>(f).unwrap();
    world
        .insert(
            f,
            AudioSource {
                retrigger: true,
                ..s
            },
        )
        .unwrap();
    world.step().unwrap();
    assert!(
        world
            .pending_events::<SoundStarted>()
            .unwrap()
            .iter()
            .any(|x| x.entity == f.to_bits()),
        "started again"
    );
    let s = world.get::<AudioSource>(f).unwrap();
    assert!(s.playing && !s.retrigger);
}

/// A run with a pitched streamed loop (the sinc), a voice cap with
/// virtual voices moving in and out of it, and retriggers every 25 ticks;
/// the PCM hash, stream hash and world hash at the end, restoring the
/// snapshot taken at `restore_at` on the way when given.
fn busy_run(restore_at: Option<u64>) -> (u64, u64, u64) {
    let bank = SoundBank::with_resident_seconds(0.0);
    let rising = insert(&bank, "swell", 1, &swell());
    let hum = insert(&bank, "hum", 1, &synth(RATE as usize, 1, sine(310.0, 0.2)));
    let (mut world, link) = world_with(&bank, |_| {});
    world.resource_mut::<Audio>().unwrap().max_voices = 3;
    let voices: Vec<Entity> = (0..6u32)
        .map(|k| {
            world
                .spawn((AudioSource {
                    looping: k % 2 == 0,
                    virtualize: k != 5,
                    priority: 100 + k * 10,
                    pitch: 0.83 + 0.21 * k as f32,
                    volume: 0.2,
                    ..AudioSource::new(if k % 2 == 0 { hum } else { rising })
                },))
                .unwrap()
        })
        .collect();
    let script = |world: &mut World, t: u64| {
        for (k, &e) in voices.iter().enumerate() {
            let mut s = world.get::<AudioSource>(e).unwrap();
            s.priority = 100 + ((t / 20 + k as u64 * 7) % 6) as u32 * 10;
            if (t + k as u64 * 3).is_multiple_of(25) {
                s.retrigger = true;
            }
            world.insert(e, s).unwrap();
        }
    };
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    let mut saved = None;
    for t in 0..200u64 {
        if Some(t) == restore_at {
            saved = Some(world.snapshot());
        }
        script(&mut world, t);
        world.step().unwrap();
        sink.pump();
    }
    let mut h = og_core::hash::StableHasher::new();
    for x in sink.pcm() {
        h.write_u32(x.to_bits());
    }
    let end = (
        h.finish(),
        world.resource::<AudioStream>().unwrap().hash,
        world.world_hash(),
    );
    if let (Some(bytes), Some(at)) = (saved, restore_at) {
        world.restore(&bytes).unwrap();
        for t in at..200 {
            script(&mut world, t);
            world.step().unwrap();
        }
        assert_eq!(
            (
                world.resource::<AudioStream>().unwrap().hash,
                world.world_hash()
            ),
            (end.1, end.2),
            "restored at tick {at}, the run goes on as it did"
        );
    }
    end
}

#[test]
fn a_run_with_sinc_virtual_voices_and_retriggers_is_the_same_twice_and_across_a_restore() {
    let a = busy_run(None);
    assert_eq!(busy_run(None), a, "the same samples and stream twice");
    assert_eq!(busy_run(Some(87)), a);
}
