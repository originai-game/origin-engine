//! G2-5: the bus graph, mixer snapshots, bus effects, audio events and
//! Opus sounds, through the public boundary: a world with
//! `og_audio::install`, its command stream rendered by a capture sink.
//!
//! Ways they could fail, and the test that catches each:
//! - a user bus ignores its parent or its volume, or a source on it plays
//!   on its built-in bus → `a_user_bus_plays_through_its_parents_gains`
//! - a snapshot's blend steps per tick instead of ramping, ramps over
//!   another length, starts on another sample than its tick's, or does not
//!   blend back → `a_snapshot_blends_linearly_across_each_tick`
//! - the equaliser, the echo or the waveshaper is not applied, applied to
//!   another bus, or the echo lands off its frame →
//!   `eq_delay_and_distortion_shape_their_bus`
//! - an event's choice or jitter draws on anything but the seeded stream
//!   (two runs differ, another seed does not), perturbs the game's og.Rng,
//!   or an order other than random is not kept →
//!   `audio_events_choose_the_same_with_the_same_seed`
//! - a cooldown or a voice limit is not kept, steals the wrong voice, or a
//!   finished voice is not despawned → `cooldowns_and_voice_limits`
//! - an Opus sound decodes off its source (late, the wrong gain, the
//!   blocks' seams) → `an_opus_sound_round_trips_within_its_quality_bound`
//! - a long Opus track streamed through the frame cache has a gap, or its
//!   mix depends on whether and how the helper decoded ahead →
//!   `a_long_opus_track_streams_without_gaps`

use crate::support;

use og_audio::opus;
use og_audio::{
    Audio, AudioBusFx, AudioBusNode, AudioEvent, AudioEventVoice, AudioSnapshot, AudioSource, Bus,
    HeadlessMode, HeadlessSink, SoundBank, event_order,
};
use og_core::{List, Rng};
use og_ecs::{Entity, QueryDesc, World, WorldDesc};
use std::f64::consts::TAU;
use support::*;

const TICK: usize = 800;

/// Renders `ticks` ticks of a world `setup` filled, calling `each` before
/// every step with the tick about to run (1 for the first).
fn render(
    bank: &SoundBank,
    setup: impl FnOnce(&mut World),
    ticks: u64,
    mut each: impl FnMut(&mut World, u64),
) -> Vec<f32> {
    let (mut world, link) = world_with(bank, |_| {});
    setup(&mut world);
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    for t in 1..=ticks {
        each(&mut world, t);
        world.step().unwrap();
        sink.pump();
    }
    sink.take_pcm()
}

fn left(pcm: &[f32]) -> Vec<f32> {
    pcm.iter().step_by(2).copied().collect()
}

/// A constant (DC) mono sound, resident: every output sample is its gain.
fn dc(bank: &SoundBank, value: f64) -> u64 {
    bank.insert_pcm("dc", RATE, 1, &synth(RATE as usize * 2, 1, |_, _| value))
        .unwrap()
}

fn looped(sound: u64) -> AudioSource {
    AudioSource {
        looping: true,
        ..AudioSource::new(sound)
    }
}

/// Goertzel magnitude of `freq` in a channel, as an amplitude.
fn tone(ch: &[f32], freq: f64) -> f64 {
    let n = ch.len() as f64;
    let w = TAU * freq / f64::from(RATE);
    let c = 2.0 * w.cos();
    let (mut s1, mut s2) = (0.0f64, 0.0f64);
    for &x in ch {
        let s0 = f64::from(x) + c * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - c * s1 * s2).max(0.0).sqrt() * 2.0 / n
}

#[test]
fn a_user_bus_plays_through_its_parents_gains() {
    let bank = SoundBank::new();
    let s = dc(&bank, 0.25);
    // Reference: the same source on master, whose gain is 1.
    let direct = left(&render(
        &bank,
        |w| {
            w.spawn((AudioSource {
                bus: Bus::Master as u32,
                ..looped(s)
            },))
                .unwrap();
        },
        20,
        |_, _| {},
    ));
    // Bus 5 (0.5) into bus 4 (0.6) into music (Audio.music 0.5): 0.15. Its
    // `bus` (ui, 1) must not count.
    let routed = left(&render(
        &bank,
        |w| {
            w.resource_mut::<Audio>().unwrap().music = 0.5;
            w.spawn((AudioBusNode {
                index: 4,
                parent: Bus::Music as u32,
                volume: 0.6,
            },))
                .unwrap();
            w.spawn((AudioBusNode {
                index: 5,
                parent: 4,
                volume: 0.5,
            },))
                .unwrap();
            w.spawn((AudioSource {
                bus: Bus::Ui as u32,
                user_bus: 5,
                ..looped(s)
            },))
                .unwrap();
        },
        20,
        |_, _| {},
    ));
    let start = direct.iter().position(|x| x.abs() > 1e-4).expect("plays");
    assert!(direct[start] > 0.1, "the direct source at full gain");
    // After the graph's first 256-frame ramp, every sample is 0.15 of the direct one.
    for n in start + TICK..direct.len() {
        let want = 0.15 * direct[n];
        assert!(
            (routed[n] - want).abs() < 2e-6,
            "frame {n}: routed {} want {want}",
            routed[n]
        );
    }
}

#[test]
fn a_snapshot_blends_linearly_across_each_tick() {
    let bank = SoundBank::new();
    let s = dc(&bank, 0.25);
    const ON: u64 = 20;
    const OFF: u64 = 40;
    const TICKS: u64 = 60;
    let setup = |snapshot: bool| {
        move |w: &mut World| {
            w.spawn((AudioBusNode {
                index: 4,
                parent: Bus::Sfx as u32,
                volume: 1.0,
            },))
                .unwrap();
            w.spawn((AudioSource {
                user_bus: 4,
                ..looped(s)
            },))
                .unwrap();
            if snapshot {
                // Underwater: sfx to half, bus 4 to a quarter, over 0.1 s (6 ticks).
                w.spawn((AudioSnapshot {
                    fade: 0.1,
                    volume: List::from_slice(&[1.0, 1.0, 0.5, 1.0, 0.25]).unwrap(),
                    ..AudioSnapshot::default()
                },))
                    .unwrap();
            }
        }
    };
    let toggle = |w: &mut World, t: u64| {
        if t == ON || t == OFF {
            let e = w
                .query(&QueryDesc::new().read::<AudioSnapshot>())
                .unwrap()
                .flat_map(|a| a.entities().to_vec())
                .next()
                .unwrap();
            let snap = w.get::<AudioSnapshot>(e).unwrap();
            w.insert(
                e,
                AudioSnapshot {
                    active: t == ON,
                    ..snap
                },
            )
            .unwrap();
        }
    };
    let dry = left(&render(&bank, setup(false), TICKS, |_, _| {}));
    let wet = left(&render(&bank, setup(true), TICKS, toggle));

    // The offline expectation, independent of the mixer: the weight moves
    // 1/6 a tick; at weight w the voice's gain is (1 - 0.5 w)(1 - 0.75 w),
    // and between ticks it is the straight line from one tick's to the
    // next's, frame i of the tick at (i + 1) / 800 of the way.
    let product = |w: f64| (1.0 - 0.5 * w) * (1.0 - 0.75 * w);
    let mut target = vec![1.0f64; TICKS as usize + 2];
    let mut w = 0.0f64;
    for t in 1..=TICKS {
        let active = (ON..OFF).contains(&t);
        w = if active {
            (w + 1.0 / 6.0).min(1.0)
        } else {
            (w - 1.0 / 6.0).max(0.0)
        };
        target[t as usize] = product(w);
    }
    // The source's first sample is tick 1's first frame.
    let first = dry.iter().position(|x| x.abs() > 1e-4).expect("plays");
    let mut checked = 0;
    for t in 2..=TICKS as usize {
        for i in 0..TICK {
            let n = first + (t - 1) * TICK + i;
            if n >= dry.len() {
                break;
            }
            let (from, to) = (target[t - 1], target[t]);
            let gain = from + (to - from) * (i as f64 + 1.0) / TICK as f64;
            let want = f64::from(dry[n]) * gain;
            assert!(
                (f64::from(wet[n]) - want).abs() < 1e-5,
                "tick {t} frame {i}: {} want {want} (gain {gain})",
                wet[n]
            );
            checked += 1;
        }
    }
    assert!(checked > 50 * TICK);
    // It did blend: down to 0.5 x 0.25 of the dry level, and back to it.
    let mid = first + (OFF as usize - 2) * TICK;
    assert!((wet[mid] / dry[mid] - 0.125).abs() < 1e-5);
    let end = dry.len() - 1;
    assert!((wet[end] / dry[end] - 1.0).abs() < 1e-5);
}

#[test]
fn eq_delay_and_distortion_shape_their_bus() {
    let bank = SoundBank::new();
    let sine = |f: f64| {
        bank.insert_pcm(
            "sine",
            RATE,
            1,
            &synth(RATE as usize, 1, support::sine(f, 0.2)),
        )
        .unwrap()
    };
    let (low, mid, high, a) = (sine(100.0), sine(1000.0), sine(12_000.0), sine(440.0));
    // Each tone on sfx, the same on music (untouched by sfx's effects).
    let run = |fx: Option<AudioBusFx>, sounds: &[u64]| {
        let sounds = sounds.to_vec();
        let pcm = render(
            &bank,
            move |w| {
                for &s in &sounds {
                    w.spawn((looped(s),)).unwrap();
                }
                if let Some(f) = fx {
                    w.spawn((f,)).unwrap();
                }
            },
            30,
            |_, _| {},
        );
        left(&pcm)[12_000..24_000].to_vec()
    };
    let tones = [low, mid, high];
    let dry = run(None, &tones);
    let eq = run(
        Some(AudioBusFx {
            bus: Bus::Sfx as u32,
            eq_low: 12.0,
            eq_low_freq: 400.0,
            eq_mid: -12.0,
            eq_mid_freq: 1000.0,
            eq_mid_q: 2.0,
            eq_high: 6.0,
            eq_high_freq: 3000.0,
            ..AudioBusFx::default()
        }),
        &tones,
    );
    let gain = |f: f64| tone(&eq, f) / tone(&dry, f);
    // Shelves well inside their band reach their gain; the peak's centre is its gain.
    let db = |g: f64| 20.0 * g.log10();
    assert!(
        (db(gain(100.0)) - 12.0).abs() < 1.0,
        "low shelf {:.2} dB",
        db(gain(100.0))
    );
    assert!(
        (db(gain(1000.0)) + 12.0).abs() < 0.5,
        "mid {:.2} dB",
        db(gain(1000.0))
    );
    assert!(
        (db(gain(12_000.0)) - 6.0).abs() < 0.5,
        "high shelf {:.2} dB",
        db(gain(12_000.0))
    );

    // The waveshaper adds odd harmonics to a pure 440 Hz.
    let clean = run(None, &[a]);
    let driven = run(
        Some(AudioBusFx {
            bus: Bus::Sfx as u32,
            distortion: 1.0,
            ..AudioBusFx::default()
        }),
        &[a],
    );
    assert!(
        tone(&clean, 1320.0) < 1e-3,
        "clean {}",
        tone(&clean, 1320.0)
    );
    assert!(
        tone(&driven, 1320.0) > 0.03,
        "third harmonic {}",
        tone(&driven, 1320.0)
    );
    // Full scale stays full scale: the fundamental is louder, not clipped away.
    assert!(tone(&driven, 440.0) > tone(&clean, 440.0));

    // The echo: one click, then copies 0.25 s apart at mix, mix * feedback.
    let mut click = vec![0i16; RATE as usize * 2];
    click[0] = 16_384;
    let c = bank.insert_pcm("click", RATE, 1, &click).unwrap();
    let pcm = left(&render(
        &bank,
        move |w| {
            w.spawn((AudioSource::new(c),)).unwrap();
            w.spawn((AudioBusFx {
                bus: Bus::Sfx as u32,
                delay: 0.25,
                delay_feedback: 0.5,
                delay_mix: 0.4,
                ..AudioBusFx::default()
            },))
                .unwrap();
        },
        60,
        |_, _| {},
    ));
    let hits: Vec<usize> = (0..pcm.len()).filter(|&n| pcm[n].abs() > 1e-4).collect();
    let at = hits[0];
    let d = 12_000;
    assert_eq!(
        hits,
        vec![at, at + d, at + 2 * d, at + 3 * d],
        "the echoes' frames"
    );
    let dry = pcm[at];
    assert!((pcm[at + d] / dry - 0.4).abs() < 1e-5);
    assert!((pcm[at + 2 * d] / dry - 0.2).abs() < 1e-5);
    assert!((pcm[at + 3 * d] / dry - 0.1).abs() < 1e-5);
}

/// The plays of a run: (tick, sound, volume bits, pitch bits) of every
/// voice the event spawned, and og.Rng's state at the end.
fn plays(
    seed: u32,
    order: u32,
    trigger_every: u64,
    ticks: u64,
) -> (Vec<(u64, u64, u32, u32)>, u32) {
    let bank = SoundBank::new();
    let sounds: Vec<u64> = (0..4)
        .map(|k| {
            bank.insert_pcm(
                &format!("s{k}"),
                RATE,
                1,
                &synth(1200, 1, support::sine(300.0 + 100.0 * k as f64, 0.2)),
            )
            .unwrap()
        })
        .collect();
    let mut world = World::new(&WorldDesc {
        tick_rate: 60,
        seed,
    })
    .unwrap();
    let link = og_audio::install(
        &mut world,
        &og_audio::AudioDesc {
            bank: Some(bank.clone()),
            ..og_audio::AudioDesc::default()
        },
    )
    .unwrap();
    let e = world
        .spawn((AudioEvent {
            sounds: List::from_slice(&sounds).unwrap(),
            weights: List::from_slice(&[1.0, 2.0, 1.0, 0.5]).unwrap(),
            order,
            volume: 0.5,
            volume_jitter: 3.0,
            pitch_jitter: 2.0,
            ..AudioEvent::default()
        },))
        .unwrap();
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Null);
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for t in 1..=ticks {
        if t % trigger_every == 1 {
            let ev = world.get::<AudioEvent>(e).unwrap();
            world
                .insert(
                    e,
                    AudioEvent {
                        trigger: true,
                        ..ev
                    },
                )
                .unwrap();
        }
        world.step().unwrap();
        sink.pump();
        let voices: Vec<Entity> = world
            .query(&QueryDesc::new().read::<AudioEventVoice>())
            .unwrap()
            .flat_map(|a| a.entities().to_vec())
            .collect();
        for v in voices {
            if seen.insert(v.to_bits()) {
                let s = world.get::<AudioSource>(v).unwrap();
                let tag = world.get::<AudioEventVoice>(v).unwrap();
                assert_eq!((tag.event, tag.sound), (e.to_bits(), s.sound));
                out.push((t, s.sound, s.volume.to_bits(), s.pitch.to_bits()));
            }
        }
    }
    let rng = world.resource::<Rng>().unwrap().save();
    (out, rng)
}

#[test]
fn audio_events_choose_the_same_with_the_same_seed() {
    let (a, rng_a) = plays(11, event_order::RANDOM, 4, 240);
    let (b, rng_b) = plays(11, event_order::RANDOM, 4, 240);
    let (c, _) = plays(12, event_order::RANDOM, 4, 240);
    assert_eq!(a.len(), 60, "a play per trigger");
    assert_eq!(
        a, b,
        "the same seed chooses the same sounds, volumes and pitches"
    );
    assert_ne!(a, c, "another seed chooses otherwise");
    // The draws are the event's own stream: the game's og.Rng is where a
    // world without triggers leaves it.
    let (none, rng_none) = plays(11, event_order::RANDOM, 1_000, 0);
    assert!(none.is_empty());
    assert_eq!(rng_a, rng_b);
    assert_eq!(rng_a, rng_none, "sounds never move the game's og.Rng");
    // Each play on the tick of its trigger, every sound chosen, the
    // weights felt (the second sound, weight 2, most often), and the
    // jitter within ±3 dB and ±2 semitones.
    let mut count = std::collections::BTreeMap::new();
    for (k, &(t, s, v, p)) in a.iter().enumerate() {
        assert_eq!(t, 1 + 4 * k as u64);
        *count.entry(s).or_insert(0) += 1;
        let (v, p) = (f32::from_bits(v), f32::from_bits(p));
        let db = 20.0 * (v / 0.5).log10();
        let semis = 12.0 * p.log2();
        assert!(
            db.abs() <= 3.0 + 1e-4 && semis.abs() <= 2.0 + 1e-4,
            "{db} dB {semis} st"
        );
    }
    assert_eq!(count.len(), 4);
    let most = count.iter().max_by_key(|e| e.1).unwrap().0;
    assert_eq!(*most, a.iter().map(|p| p.1).min().unwrap() + 1, "{count:?}");
    let pitches: std::collections::BTreeSet<u32> = a.iter().map(|p| p.3).collect();
    assert!(pitches.len() > 50, "the pitch varies");

    // Round-robin plays the sounds in turn; shuffle each round's four once.
    let (rr, _) = plays(11, event_order::ROUND_ROBIN, 4, 40);
    let base = rr[0].1;
    let order: Vec<u64> = rr.iter().map(|p| p.1 - base).collect();
    assert_eq!(order, vec![0, 1, 2, 3, 0, 1, 2, 3, 0, 1]);
    let (sh, _) = plays(11, event_order::SHUFFLE, 4, 160);
    let (sh2, _) = plays(11, event_order::SHUFFLE, 4, 160);
    assert_eq!(sh, sh2);
    let ids: Vec<u64> = sh.iter().map(|p| p.1).collect();
    for round in ids.chunks(4) {
        let mut r = round.to_vec();
        r.sort();
        r.dedup();
        assert_eq!(r.len(), 4, "a shuffle round plays each sound once: {ids:?}");
    }
    for pair in ids.windows(2) {
        assert_ne!(pair[0], pair[1], "never the same twice in a row: {ids:?}");
    }
}

/// An event's run with a trigger on each tick `trigger(t)` holds: per
/// tick its (played, voices), and the plays of its voices sounding.
fn limited(
    ev: AudioEvent,
    frames: usize,
    ticks: u64,
    trigger: impl Fn(u64) -> bool,
) -> (Vec<(u32, u32)>, Vec<Vec<u32>>) {
    let bank = SoundBank::new();
    let s = bank
        .insert_pcm(
            "long",
            RATE,
            1,
            &synth(frames, 1, support::sine(220.0, 0.2)),
        )
        .unwrap();
    let (mut world, link) = world_with(&bank, |_| {});
    let e = world
        .spawn((AudioEvent {
            sounds: List::from_slice(&[s]).unwrap(),
            ..ev
        },))
        .unwrap();
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Null);
    let (mut counts, mut sounding) = (Vec::new(), Vec::new());
    for t in 1..=ticks {
        if trigger(t) {
            let ev = world.get::<AudioEvent>(e).unwrap();
            world
                .insert(
                    e,
                    AudioEvent {
                        trigger: true,
                        ..ev
                    },
                )
                .unwrap();
        }
        world.step().unwrap();
        sink.pump();
        let ev = world.get::<AudioEvent>(e).unwrap();
        assert!(!ev.trigger, "the engine clears the trigger");
        counts.push((ev.played, ev.voices));
        let mut playing: Vec<u32> = world
            .query(&QueryDesc::new().read::<AudioEventVoice>())
            .unwrap()
            .flat_map(|a| a.entities().to_vec())
            .filter(|&v| world.get::<AudioSource>(v).unwrap().playing)
            .map(|v| world.get::<AudioEventVoice>(v).unwrap().play)
            .collect();
        playing.sort();
        sounding.push(playing);
    }
    (counts, sounding)
}

#[test]
fn cooldowns_and_voice_limits() {
    // A trigger every tick with a 0.21 s cooldown: a play every 13 ticks
    // (12 ticks is 0.2 s, still cooling).
    let (counts, _) = limited(
        AudioEvent {
            cooldown: 0.21,
            ..AudioEvent::default()
        },
        2400,
        60,
        |_| true,
    );
    let plays: Vec<usize> = (0..counts.len())
        .filter(|&i| counts[i].0 > if i == 0 { 0 } else { counts[i - 1].0 })
        .map(|i| i + 1)
        .collect();
    assert_eq!(plays, vec![1, 14, 27, 40, 53]);

    // Three voices at most, five triggers on ticks 1 to 5 of a 10 s sound.
    let ev = AudioEvent {
        max_voices: 3,
        ..AudioEvent::default()
    };
    let (counts, sounding) = limited(ev, RATE as usize * 10, 8, |t| t <= 5);
    assert_eq!(counts[7], (5, 3));
    assert_eq!(sounding[7], vec![3, 4, 5], "the oldest are stolen");
    assert!(counts.iter().all(|c| c.1 <= 3));
    let (counts, sounding) = limited(
        AudioEvent { steal: false, ..ev },
        RATE as usize * 10,
        8,
        |t| t <= 5,
    );
    assert_eq!(
        counts[7],
        (3, 3),
        "without stealing the triggers past the limit are ignored"
    );
    assert_eq!(sounding[7], vec![1, 2, 3]);

    // A finished voice is despawned and no longer counts: a 0.05 s sound
    // ends on tick 3, its entity is gone the tick after.
    let (counts, sounding) = limited(AudioEvent::default(), 2400, 6, |t| t == 1);
    assert_eq!(counts[0], (1, 1));
    assert_eq!(counts[5], (1, 0));
    assert!(sounding[5].is_empty());
}

/// A two-chord pad with a sweep, stereo at 48 kHz.
fn music(frames: usize) -> Vec<i16> {
    synth(frames, 2, |i, c| {
        let t = i as f64 / f64::from(RATE);
        let root = if (t as u64).is_multiple_of(2) {
            220.0
        } else {
            196.0
        };
        let d = if c == 0 { 1.0 } else { 1.004 };
        0.15 * ((TAU * root * d * t).sin()
            + (TAU * root * 1.25 * t).sin()
            + (TAU * root * 1.5 * d * t).sin()
            + 0.5 * (TAU * (900.0 + 300.0 * (TAU * 0.2 * t).sin()) * t).sin())
    })
}

/// Renders one sound alone on master for `ticks`.
fn alone(bank: &SoundBank, sound: u64, ticks: u64, after: impl FnMut(&mut World, u64)) -> Vec<f32> {
    let mut after = after;
    let mut prev = 0;
    render(
        bank,
        |w| {
            w.spawn((AudioSource {
                bus: Bus::Master as u32,
                ..AudioSource::new(sound)
            },))
                .unwrap();
        },
        ticks,
        |w, t| {
            // The helper runs after the tick before this one, as a host's does.
            if t > 1 {
                after(w, prev);
            }
            prev = t;
        },
    )
}

#[test]
fn an_opus_sound_round_trips_within_its_quality_bound() {
    let bank = SoundBank::new();
    let pcm = music(RATE as usize * 3);
    let bytes = opus::encode(&pcm, 2, 96_000, RATE).unwrap();
    let encoded = bank.insert_opus("music.opus", &bytes).unwrap();
    let source = bank.insert_pcm("music.wav", RATE, 2, &pcm).unwrap();
    assert_eq!(
        bank.info(encoded),
        bank.info(source),
        "the same rate and length"
    );
    // Through the same mixer path, only the codec differs.
    let a = alone(&bank, encoded, 170, |_, _| {});
    let b = alone(&bank, source, 170, |_, _| {});
    let (mut signal, mut noise) = (0.0f64, 0.0f64);
    for (x, y) in a.iter().zip(&b) {
        signal += f64::from(*y).powi(2);
        noise += f64::from(x - y).powi(2);
    }
    let snr = 10.0 * (signal / noise).log10();
    assert!(signal > 1.0, "it played");
    assert!(snr > 30.0, "Opus round trip {snr:.1} dB");
    // 96 kbit/s (and the blocks' pre-roll) against QOA's 3.2 bits a
    // sample: under half its size.
    let qoa = og_audio::qoa::encode(&pcm, 2, RATE).unwrap();
    assert!(
        bytes.len() * 2 < qoa.len(),
        "{} vs QOA {}",
        bytes.len(),
        qoa.len()
    );
}

#[test]
fn a_long_opus_track_streams_without_gaps() {
    const SECONDS: usize = 40;
    let pcm = music(RATE as usize * SECONDS);
    let bytes = opus::encode(&pcm, 2, 64_000, RATE).unwrap();
    let whole = opus::decode(&bytes).unwrap();
    assert_eq!(whole.samples.len(), pcm.len());
    let ticks = (SECONDS * 60) as u64 + 30;

    // The reference: the whole decode, resident.
    let reference = {
        let bank = SoundBank::new();
        let s = bank.insert_pcm("whole", RATE, 2, &whole.samples).unwrap();
        alone(&bank, s, ticks, |_, _| {})
    };
    // Streamed: nothing beyond the first block is pinned; the helper never
    // runs (the mixer decodes what it reaches), runs every queued block,
    // or decodes four packets (80 ms) a tick as the web's pump does.
    for helper in 0..3 {
        let bank = SoundBank::with_resident_seconds(0.0);
        let s = bank.insert_opus("track.opus", &bytes).unwrap();
        let mut peak = 0;
        let pcm = alone(&bank, s, ticks, |_, _| {
            match helper {
                1 => {
                    bank.decode_queued(usize::MAX);
                }
                2 => {
                    bank.decode_opus_step(4);
                }
                _ => {}
            }
            peak = peak.max(bank.resident_bytes());
        });
        assert_eq!(pcm.len(), reference.len());
        if let Some(n) = (0..pcm.len()).find(|&n| pcm[n].to_bits() != reference[n].to_bits()) {
            panic!(
                "helper {helper}: sample {n} differs: {} vs {}",
                pcm[n], reference[n]
            );
        }
        // No gap: from the first sample to the track's end, never 10 ms of silence.
        let l = left(&pcm);
        let first = l.iter().position(|x| x.abs() > 1e-4).unwrap();
        let end = first + whole.samples.len() / 2;
        let mut run = 0;
        for &x in &l[first..end - 1] {
            run = if x.abs() < 1e-5 { run + 1 } else { 0 };
            assert!(run < 480, "helper {helper}: a gap of {run} frames");
        }
        let stats = bank.decode_stats();
        if helper == 2 {
            assert!(stats.misses <= 1, "the pump keeps ahead: {stats:?}");
        }
        // The track was never resident whole: at most a few blocks decoded.
        let whole_bytes = whole.samples.len() * 2;
        assert!(
            peak * 8 < whole_bytes,
            "helper {helper}: {peak} of {whole_bytes} bytes resident"
        );
    }
}
