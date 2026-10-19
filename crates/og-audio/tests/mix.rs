//! The E6 mixer and tick features, through the public boundary: a world
//! with `og_audio::install`, its command stream rendered by a capture sink.
//!
//! Ways they could fail, and the test that catches each:
//! - a bus insert is not applied, applied to the wrong bus, or its cutoff is
//!   ignored → `bus_filters_shape_their_bus_only`
//! - ducking never engages, never releases, or keys off the wrong bus →
//!   `ducking_turns_music_down_under_effects_and_back`
//! - the compressor does not reduce a loud bus → `the_compressor_reduces_a_loud_bus`
//! - the reverb send leaves no tail, or plays without a send →
//!   `a_reverb_send_leaves_a_tail`
//! - a scheduled start lands on the tick instead of its sample, or two stems
//!   started together drift → `scheduled_starts_land_on_their_sample_and_stems_stay_aligned`
//! - the music clock's beats, bars or next-bar times are off →
//!   `the_music_clock_beats_and_a_stinger_lands_on_the_bar_line`
//! - a crossfade cuts instead of ramping, or ignores its length →
//!   `a_crossfade_ramps_over_its_fade`
//! - the voice cap lets too many play, culls the wrong one, or stops a
//!   culled source silently → `the_voice_cap_culls_the_lowest_priority`
//! - occlusion or distance does not dull the sound →
//!   `occlusion_and_distance_dull_a_source`
//! - the mixer's output depends on anything but the stream, or `listen`
//!   returns other frames than the ones rendered →
//!   `the_mix_is_the_same_twice_and_listen_returns_its_last_second`
//! - a source with a startOffset starts from the clip's beginning, a frame
//!   off, fades in over the first samples, loses its startAt inside the
//!   tick, or ends on another tick than the rest of the clip does →
//!   `a_start_part_way_plays_the_clip_from_its_offset_to_the_sample`

mod support;

use og_audio::{
    Audio, AudioBusFx, AudioDesc, AudioLink, AudioSource, AudioStream, Bus, HeadlessMode,
    HeadlessSink, Music, MusicBeat, SoundBank, SoundEnded, SoundStarted, Transform, end_reason,
    install,
};
use og_core::glam::Vec3;
use og_ecs::{Entity, World, WorldDesc};
use std::f64::consts::TAU;
use std::sync::Arc;
use support::*;

const FRAMES_PER_TICK: usize = 800;

fn step(world: &mut World, sink: &mut HeadlessSink, ticks: u64) {
    for _ in 0..ticks {
        world.step().unwrap();
        sink.pump();
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

/// Frames `a..b` of the left channel.
fn window(pcm: &[f32], a: usize, b: usize) -> Vec<f32> {
    pcm[a * 2..b * 2].iter().step_by(2).copied().collect()
}

fn looped(sound: u64, bus: Bus, volume: f32) -> AudioSource {
    AudioSource {
        looping: true,
        bus: bus as u32,
        volume,
        ..AudioSource::new(sound)
    }
}

fn capture(world: &mut World, link: &AudioLink) -> HeadlessSink {
    let _ = world;
    HeadlessSink::new(link, HeadlessMode::Capture { sample_rate: RATE })
}

#[test]
fn bus_filters_shape_their_bus_only() {
    let bank = SoundBank::new();
    let low = insert(&bank, "low", 1, &synth(RATE as usize, 1, sine(200.0, 0.3)));
    let high = insert(
        &bank,
        "high",
        1,
        &synth(RATE as usize, 1, sine(6000.0, 0.3)),
    );
    let run = |fx: Option<AudioBusFx>| {
        let (mut world, link) = world_with(&bank, |_| {});
        world.spawn((looped(low, Bus::Sfx, 1.0),)).unwrap();
        world.spawn((looped(high, Bus::Sfx, 1.0),)).unwrap();
        // The same pair on music, untouched by sfx's inserts.
        world.spawn((looped(low, Bus::Music, 0.5),)).unwrap();
        world.spawn((looped(high, Bus::Music, 0.5),)).unwrap();
        if let Some(f) = fx {
            world.spawn((f,)).unwrap();
        }
        let mut sink = capture(&mut world, &link);
        step(&mut world, &mut sink, 30);
        let w = window(sink.pcm(), 12_000, 24_000);
        (tone(&w, 200.0), tone(&w, 6000.0))
    };
    let (dry_low, dry_high) = run(None);
    let (lp_low, lp_high) = run(Some(AudioBusFx {
        bus: Bus::Sfx as u32,
        lowpass: 1000.0,
        ..AudioBusFx::default()
    }));
    let (hp_low, hp_high) = run(Some(AudioBusFx {
        bus: Bus::Sfx as u32,
        highpass: 2000.0,
        ..AudioBusFx::default()
    }));
    // Each tone: 0.3 on sfx plus 0.15 on music (equal-power centre: × 0.707).
    assert!(
        (dry_low - 0.45 * 0.707).abs() < 0.03,
        "dry 200 Hz {dry_low}"
    );
    assert!(
        (dry_high - 0.45 * 0.707).abs() < 0.03,
        "dry 6 kHz {dry_high}"
    );
    // The low-pass removes sfx's 6 kHz (music's half stays) and keeps 200 Hz.
    assert!(
        (lp_low - dry_low).abs() < 0.02,
        "low-pass keeps 200 Hz: {lp_low}"
    );
    assert!(
        lp_high < 0.15 * 0.707 + 0.02,
        "low-pass cuts sfx's 6 kHz: {lp_high}"
    );
    assert!(
        lp_high > 0.15 * 0.707 - 0.02,
        "and leaves music's: {lp_high}"
    );
    // The high-pass the other way round.
    assert!(
        (hp_high - dry_high).abs() < 0.02,
        "high-pass keeps 6 kHz: {hp_high}"
    );
    assert!(
        hp_low < 0.15 * 0.707 + 0.02 && hp_low > 0.15 * 0.707 - 0.02,
        "high-pass cuts sfx's 200 Hz: {hp_low}"
    );
}

#[test]
fn ducking_turns_music_down_under_effects_and_back() {
    let bank = SoundBank::new();
    let bed = insert(&bank, "bed", 1, &synth(RATE as usize, 1, sine(220.0, 0.3)));
    let hit = insert(
        &bank,
        "hit",
        1,
        &synth(RATE as usize / 2, 1, sine(3000.0, 0.4)),
    );
    let (mut world, link) = world_with(&bank, |_| {});
    world.spawn((looped(bed, Bus::Music, 1.0),)).unwrap();
    world
        .spawn((AudioBusFx {
            bus: Bus::Music as u32,
            duck: true,
            duck_bus: Bus::Sfx as u32,
            duck_depth: 12.0,
            duck_threshold: -30.0,
            duck_attack: 0.01,
            duck_release: 0.2,
            ..AudioBusFx::default()
        },))
        .unwrap();
    let mut sink = capture(&mut world, &link);
    step(&mut world, &mut sink, 60);
    world.spawn((AudioSource::new(hit),)).unwrap();
    step(&mut world, &mut sink, 120);
    let pcm = sink.pcm();
    // The hit starts at tick 60 (frame 48000) and lasts 0.5 s.
    let before = tone(&window(pcm, 24_000, 48_000), 220.0);
    let during = tone(&window(pcm, 52_800, 72_000), 220.0);
    let after = tone(&window(pcm, 110_000, 144_000), 220.0);
    let depth = 20.0 * (during / before).log10();
    assert!(
        (depth + 12.0).abs() < 1.0,
        "music is 12 dB down under the hit: {depth:.2} dB"
    );
    assert!(
        (after / before - 1.0).abs() < 0.03,
        "and back after the release: {after} vs {before}"
    );
}

#[test]
fn the_compressor_reduces_a_loud_bus() {
    let bank = SoundBank::new();
    let loud = insert(&bank, "loud", 1, &synth(RATE as usize, 1, sine(330.0, 0.7)));
    let level = |fx: bool| {
        let (mut world, link) = world_with(&bank, |_| {});
        world.spawn((looped(loud, Bus::Sfx, 1.0),)).unwrap();
        if fx {
            world
                .spawn((AudioBusFx {
                    bus: Bus::Sfx as u32,
                    threshold: -20.0,
                    ratio: 4.0,
                    ..AudioBusFx::default()
                },))
                .unwrap();
        }
        let mut sink = capture(&mut world, &link);
        step(&mut world, &mut sink, 30);
        let reduction = sink.mixer().unwrap().bus_dynamics(Bus::Sfx as usize).0;
        (tone(&window(sink.pcm(), 12_000, 24_000), 330.0), reduction)
    };
    let (dry, none) = level(false);
    let (wet, reduction) = level(true);
    assert_eq!(none, 0.0);
    // A peak of 0.7 × 0.707 is -6 dB: 14 dB over -20, reduced by 14 × 3/4 = 10.5 dB.
    let got = 20.0 * (wet / dry).log10();
    assert!(
        (got + 10.5).abs() < 1.5,
        "the compressed bus is {got:.2} dB, reduction {reduction:.2}"
    );
    assert!(reduction < -9.0);
}

#[test]
fn a_reverb_send_leaves_a_tail() {
    let bank = SoundBank::new();
    let click = insert(&bank, "click", 1, &synth(480, 1, sine(1000.0, 0.8)));
    let tail = |send: f32| {
        let (mut world, link) = world_with(&bank, |_| {});
        world
            .spawn((AudioBusFx {
                bus: Bus::Sfx as u32,
                reverb: send,
                ..AudioBusFx::default()
            },))
            .unwrap();
        world.spawn((AudioSource::new(click),)).unwrap();
        let mut sink = capture(&mut world, &link);
        step(&mut world, &mut sink, 60);
        let pcm = sink.pcm().to_vec();
        rms(&window(&pcm, 9_600, 24_000))
    };
    let dry = tail(0.0);
    let wet = tail(0.6);
    assert!(dry < 1e-4, "no send, no tail: {dry}");
    assert!(
        wet > 3e-3,
        "the send leaves a tail 0.2–0.5 s after the click: {wet}"
    );
}

#[test]
fn scheduled_starts_land_on_their_sample_and_stems_stay_aligned() {
    let bank = SoundBank::new();
    let a = insert(&bank, "a", 1, &synth(RATE as usize, 1, sine(500.0, 0.25)));
    let b = insert(
        &bank,
        "b",
        1,
        &synth(RATE as usize, 1, |i, _| if i == 0 { 0.5 } else { 0.0 }),
    );
    let (mut world, link) = world_with(&bank, |_| {});
    // 1.0 s + 123 samples: inside tick 60.
    let at = 1.0 + 123.0 / f64::from(RATE);
    let s1 = world
        .spawn((AudioSource {
            start_at: at,
            pan: -1.0,
            ..AudioSource::new(b)
        },))
        .unwrap();
    world
        .spawn((AudioSource {
            start_at: at,
            pan: 1.0,
            ..AudioSource::new(b)
        },))
        .unwrap();
    world
        .spawn((AudioSource {
            start_at: 2.0,
            ..AudioSource::new(a)
        },))
        .unwrap();
    let mut sink = capture(&mut world, &link);
    let mut started_tick = None;
    for _ in 0..150 {
        world.step().unwrap();
        sink.pump();
        if world
            .pending_events::<SoundStarted>()
            .unwrap()
            .iter()
            .any(|s| s.entity == s1.to_bits())
        {
            started_tick.get_or_insert(world.time().tick);
        }
    }
    assert_eq!(
        started_tick,
        Some(61),
        "started in tick 60 (pending after the step)"
    );
    let (l, r) = split(sink.pcm());
    let first = |ch: &[f32]| ch.iter().position(|x| x.abs() > 0.1).unwrap();
    assert_eq!(
        first(&l),
        48_000 + 123,
        "the left stem's impulse lands on its sample"
    );
    assert_eq!(first(&r), first(&l), "both stems start on the same sample");
    let tone_start = sink.pcm().as_chunks::<2>().0[96_000..]
        .iter()
        .position(|f| f[0].abs() > 0.01)
        .unwrap();
    assert!(
        tone_start <= 2,
        "a start on a tick boundary is its first frame: {tone_start}"
    );
}

#[test]
fn the_music_clock_beats_and_a_stinger_lands_on_the_bar_line() {
    let bank = SoundBank::new();
    let sting = insert(
        &bank,
        "sting",
        1,
        &synth(4800, 1, |i, _| if i == 0 { 0.6 } else { 0.0 }),
    );
    let (mut world, link) = world_with(&bank, |_| {});
    // 100 bpm in 3/4 from 0.25 s: a beat every 0.6 s, a bar every 1.8 s.
    let clock = world
        .spawn((Music {
            bpm: 100.0,
            beats_per_bar: 3,
            start: 0.25,
            ..Music::default()
        },))
        .unwrap();
    let mut sink = capture(&mut world, &link);
    let mut beats = Vec::new();
    let mut stinger: Option<Entity> = None;
    for _ in 0..240 {
        world.step().unwrap();
        sink.pump();
        beats.extend(world.pending_events::<MusicBeat>().unwrap());
        let m = world.get::<Music>(clock).unwrap();
        // At 1.0 s, schedule a stinger on the next bar line.
        if world.time().tick == 60 && stinger.is_none() {
            assert!(
                (m.next_bar - 2.05).abs() < 1e-9,
                "the next bar after 1 s is 2.05 s: {}",
                m.next_bar
            );
            stinger = Some(
                world
                    .spawn((AudioSource {
                        start_at: m.next_bar,
                        ..AudioSource::new(sting)
                    },))
                    .unwrap(),
            );
        }
    }
    let times: Vec<(u32, u32, f64)> = beats.iter().map(|b| (b.bar, b.beat, b.time)).collect();
    assert_eq!(times[0], (1, 1, 0.25));
    assert_eq!(times[1], (1, 2, 0.85));
    assert_eq!(times[3], (2, 1, 2.05));
    assert_eq!(times.len(), 7, "beats at 0.25 + 0.6k below 4 s: {times:?}");
    let m = world.get::<Music>(clock).unwrap();
    assert_eq!(
        (m.bar, m.beat),
        (3, 1),
        "at 4 s: bar 3 (from 3.85 s), beat 1"
    );
    let first = sink
        .pcm()
        .as_chunks::<2>()
        .0
        .iter()
        .position(|f| f[0].abs() > 0.1)
        .unwrap();
    assert_eq!(
        first,
        (2.05 * f64::from(RATE)).round() as usize,
        "the stinger hits the bar line's sample"
    );
}

#[test]
fn a_crossfade_ramps_over_its_fade() {
    let bank = SoundBank::new();
    let a = insert(&bank, "a", 1, &synth(RATE as usize, 1, sine(300.0, 0.4)));
    let b = insert(&bank, "b", 1, &synth(RATE as usize, 1, sine(700.0, 0.4)));
    let (mut world, link) = world_with(&bank, |_| {});
    let ea = world.spawn((looped(a, Bus::Music, 1.0),)).unwrap();
    let mut sink = capture(&mut world, &link);
    step(&mut world, &mut sink, 60);
    // A two-second crossfade from a to b.
    let s = world.get::<AudioSource>(ea).unwrap();
    world
        .insert(
            ea,
            AudioSource {
                playing: false,
                fade: 2.0,
                ..s
            },
        )
        .unwrap();
    world
        .spawn((AudioSource {
            fade: 2.0,
            ..looped(b, Bus::Music, 1.0)
        },))
        .unwrap();
    step(&mut world, &mut sink, 180);
    let pcm = sink.pcm();
    let at = |t: f64| {
        let f = (t * f64::from(RATE)) as usize;
        let w = window(pcm, f - 1200, f + 1200);
        (
            tone(&w, 300.0) / (0.4 * 0.707),
            tone(&w, 700.0) / (0.4 * 0.707),
        )
    };
    let (a0, b0) = at(0.5);
    let (a1, b1) = at(2.0);
    let (a2, b2) = at(3.5);
    assert!((a0 - 1.0).abs() < 0.03 && b0 < 0.01, "before: {a0} {b0}");
    assert!(
        (a1 - 0.5).abs() < 0.05 && (b1 - 0.5).abs() < 0.05,
        "halfway (1 s into 2): {a1} {b1}"
    );
    assert!(a2 < 0.01 && (b2 - 1.0).abs() < 0.03, "after: {a2} {b2}");
}

#[test]
fn the_voice_cap_culls_the_lowest_priority() {
    let bank = SoundBank::new();
    let s = insert(&bank, "s", 1, &synth(RATE as usize, 1, sine(400.0, 0.05)));
    let (mut world, link) = world_with(&bank, |_| {});
    world.resource_mut::<Audio>().unwrap().max_voices = 4;
    let prio = [100, 50, 200, 10, 150, 50];
    let es: Vec<Entity> = prio
        .iter()
        .map(|&p| {
            world
                .spawn((AudioSource {
                    priority: p,
                    ..looped(s, Bus::Sfx, 1.0)
                },))
                .unwrap()
        })
        .collect();
    let mut sink = capture(&mut world, &link);
    world.step().unwrap();
    sink.pump();
    let ended = world.pending_events::<SoundEnded>().unwrap();
    let culled: Vec<u64> = ended
        .iter()
        .filter(|e| e.reason == end_reason::CULLED)
        .map(|e| e.entity)
        .collect();
    // Priorities 10 and the older of the two 50s lose.
    assert_eq!(culled, vec![es[1].to_bits(), es[3].to_bits()]);
    assert!(!world.get::<AudioSource>(es[3]).unwrap().playing);
    assert_eq!(world.resource::<AudioStream>().unwrap().voices, 4);
    // A newcomer of priority 120 steals from the 50.
    let late = world
        .spawn((AudioSource {
            priority: 120,
            ..looped(s, Bus::Sfx, 1.0)
        },))
        .unwrap();
    world.step().unwrap();
    sink.pump();
    let ended = world.pending_events::<SoundEnded>().unwrap();
    assert_eq!(
        ended
            .iter()
            .map(|e| (e.entity, e.reason))
            .collect::<Vec<_>>(),
        vec![(es[5].to_bits(), end_reason::CULLED)]
    );
    assert!(world.get::<AudioSource>(late).unwrap().playing);
    assert_eq!(world.resource::<AudioStream>().unwrap().voices, 4);
    step(&mut world, &mut sink, 5);
    assert_eq!(
        sink.mixer().unwrap().voice_count(),
        4,
        "the mixer plays four"
    );
}

#[test]
fn occlusion_and_distance_dull_a_source() {
    let bank = SoundBank::new();
    let noise = synth(RATE as usize, 1, |i, _| {
        // A hash of the frame index: white noise, the same every run.
        let mut x = (i as u32).wrapping_mul(0x9e37_79b9) ^ 0x85eb_ca6b;
        x ^= x >> 15;
        x = x.wrapping_mul(0x2c1b_3c6d);
        x ^= x >> 12;
        (f64::from(x) / f64::from(u32::MAX) - 0.5) * 0.6
    });
    let n = insert(&bank, "n", 1, &noise);
    // (high band over low band, the low band's level) of a spatial source.
    let measure = |occluded: bool, distance: f32, cutoff: f32| {
        let mut world = World::new(&WorldDesc {
            tick_rate: 60,
            seed: 7,
        })
        .unwrap();
        world.register::<Transform>().unwrap();
        let link = install(
            &mut world,
            &AudioDesc {
                bank: Some(bank.clone()),
                occluder: Some(Arc::new(move |_, _| occluded)),
                ..AudioDesc::default()
            },
        )
        .unwrap();
        world
            .spawn((
                Transform::at(Vec3::new(0.0, 0.0, -distance)),
                AudioSource {
                    spatial: true,
                    occlude: true,
                    distance_cutoff: cutoff,
                    ref_distance: 1.0,
                    max_distance: 50.0,
                    rolloff_factor: 0.0,
                    ..looped(n, Bus::Sfx, 1.0)
                },
            ))
            .unwrap();
        let mut sink = capture(&mut world, &link);
        step(&mut world, &mut sink, 30);
        let w = window(sink.pcm(), 12_000, 24_000);
        let hi: f64 = [4000.0, 6000.0, 8000.0, 10000.0]
            .iter()
            .map(|&f| tone(&w, f))
            .sum();
        let lo: f64 = [200.0, 300.0, 400.0, 500.0]
            .iter()
            .map(|&f| tone(&w, f))
            .sum();
        (hi / lo, lo)
    };
    let (clear, clear_level) = measure(false, 2.0, 0.0);
    let (behind, behind_level) = measure(true, 2.0, 0.0);
    let (far, _) = measure(false, 49.0, 1000.0);
    let (near, _) = measure(false, 1.0, 1000.0);
    assert!(
        behind < clear * 0.3,
        "occlusion takes the highs: {behind:.3} vs {clear:.3}"
    );
    let lows = behind_level / clear_level;
    assert!(
        lows > 0.25 && lows < 0.4,
        "and the level, to occlusionGain 0.35 below its cutoff: {lows:.3}"
    );
    assert!(
        far < near * 0.3,
        "distance takes the highs: {far:.3} vs {near:.3}"
    );
    assert!(
        (near / clear - 1.0).abs() < 0.2,
        "at refDistance nothing is filtered: {near} vs {clear}"
    );
}

/// A busy scene with every feature: buses with inserts, ducking, reverb,
/// scheduled stems on a clock, a crossfade and the cap.
fn busy(bank: &SoundBank) -> (World, AudioLink) {
    let bed = insert(
        bank,
        "bed",
        2,
        &synth(RATE as usize, 2, |i, c| {
            0.2 * (TAU * (110.0 + c as f64 * 55.0) * i as f64 / f64::from(RATE)).sin()
        }),
    );
    let hit = insert(
        bank,
        "hit",
        1,
        &synth(9600, 1, |i, _| {
            0.5 * (-(i as f64) / 2000.0).exp() * (TAU * 900.0 * i as f64 / f64::from(RATE)).sin()
        }),
    );
    let (mut world, link) = world_with(bank, |_| {});
    world.resource_mut::<Audio>().unwrap().max_voices = 6;
    world
        .spawn((Music {
            bpm: 128.0,
            ..Music::default()
        },))
        .unwrap();
    world
        .spawn((AudioBusFx {
            bus: Bus::Music as u32,
            lowpass: 3000.0,
            duck: true,
            reverb: 0.2,
            ..AudioBusFx::default()
        },))
        .unwrap();
    world
        .spawn((AudioBusFx {
            bus: Bus::Sfx as u32,
            threshold: -18.0,
            reverb: 0.4,
            ..AudioBusFx::default()
        },))
        .unwrap();
    world
        .spawn((AudioSource {
            start_at: 0.5,
            fade: 0.5,
            ..looped(bed, Bus::Music, 0.8)
        },))
        .unwrap();
    for k in 0..10 {
        world
            .spawn((AudioSource {
                start_at: 0.3 + k as f64 * 60.0 / 128.0,
                priority: 100 + k,
                pitch: 1.0 + k as f32 * 0.05,
                ..AudioSource::new(hit)
            },))
            .unwrap();
    }
    (world, link)
}

fn pcm_hash(pcm: &[f32]) -> u64 {
    let mut h = og_core::hash::StableHasher::new();
    for x in pcm {
        h.write_u32(x.to_bits());
    }
    h.finish()
}

#[test]
fn the_mix_is_the_same_twice_and_listen_returns_its_last_second() {
    let render = || {
        let bank = SoundBank::new();
        let (mut world, link) = busy(&bank);
        let mut sink = capture(&mut world, &link);
        step(&mut world, &mut sink, 300);
        let pcm = sink.pcm().to_vec();
        let (last, rate) = link.listen().last(1.0).unwrap();
        (
            pcm,
            last,
            rate,
            world.resource::<AudioStream>().unwrap().hash,
        )
    };
    let (a, last, rate, stream_a) = render();
    let (b, _, _, stream_b) = render();
    assert_eq!(stream_a, stream_b);
    assert_eq!(a.len(), 300 * FRAMES_PER_TICK * 2);
    assert_eq!(
        pcm_hash(&a),
        pcm_hash(&b),
        "two runs render the same samples"
    );
    assert!(rms(&a) > 0.01);
    assert_eq!(rate, RATE);
    assert_eq!(last.len(), RATE as usize * 2);
    assert_eq!(
        last.as_slice(),
        &a[a.len() - RATE as usize * 2..],
        "listen is the last second rendered"
    );

    // A null sink renders the same into its listen ring.
    let bank = SoundBank::new();
    let (mut world, link) = busy(&bank);
    let mut null = HeadlessSink::new(&link, HeadlessMode::Null);
    step(&mut world, &mut null, 300);
    assert!(null.pcm().is_empty());
    assert_eq!(link.listen().last(1.0).unwrap().0, last);

    // Evidence: the waveform hash and the WAV.
    let dir = evidence_dir();
    std::fs::write(dir.join("e6-mix.wav"), og_audio::wav(&a, 2, RATE)).unwrap();
    std::fs::write(
        dir.join("e6-mix.json"),
        format!(
            "{{\"frames\":{},\"pcmHash\":\"{:016x}\",\"streamHash\":\"{:016x}\",\"runs\":2}}\n",
            a.len() / 2,
            pcm_hash(&a),
            stream_a
        ),
    )
    .unwrap();
}

/// A capture of `source` alone for `ticks` ticks: the PCM and the tick its SoundEnded was sent in.
fn render_alone(bank: &SoundBank, source: AudioSource, ticks: u64) -> (Vec<f32>, Option<u64>) {
    let (mut world, link) = world_with(bank, |_| {});
    world.spawn((source,)).unwrap();
    let mut sink = capture(&mut world, &link);
    let mut ended = None;
    for _ in 0..ticks {
        world.step().unwrap();
        sink.pump();
        if ended.is_none() && !world.pending_events::<SoundEnded>().unwrap().is_empty() {
            ended = Some(world.time().tick);
        }
    }
    (sink.pcm().to_vec(), ended)
}

#[test]
fn a_start_part_way_plays_the_clip_from_its_offset_to_the_sample() {
    let bank = SoundBank::new();
    // Noise, so a frame's offset cannot hide in a period (0.3: under the limiter's knee).
    let mut state = 0x2545_f491_u32;
    let noise: Vec<i16> = (0..RATE as usize)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            ((f64::from(state) / f64::from(u32::MAX) * 2.0 - 1.0) * 0.3 * 32767.0) as i16
        })
        .collect();
    let clip = insert(&bank, "noise", 1, &noise);
    let whole = AudioSource {
        bus: Bus::Music as u32,
        ..AudioSource::new(clip)
    };
    let (full, full_end) = render_alone(&bank, whole, 90);
    // From 0.3125 s in (15,000 frames), started at once.
    let skip = 15_000usize;
    let offset = skip as f64 / f64::from(RATE);
    let (part, part_end) = render_alone(
        &bank,
        AudioSource {
            start_offset: offset,
            ..whole
        },
        90,
    );
    let f = |pcm: &[f32], i: usize| pcm[i * 2];
    assert!(
        f(&full, skip).abs() > 0.0,
        "the clip sounds where the offset lands"
    );
    for i in 0..(RATE as usize - skip) {
        assert_eq!(
            f(&part, i),
            f(&full, i + skip),
            "frame {i} of the part-way start is frame {} of the clip",
            i + skip
        );
    }
    assert!(
        part[(RATE as usize - skip) * 2..].iter().all(|x| *x == 0.0),
        "and it stops where the clip ends"
    );
    // The clip ends in tick 59 (48,000 frames at 800 a tick); from 15,000 in
    // its rest of 33,000 frames ends in tick 41.
    assert_eq!(full_end, Some(60));
    assert_eq!(
        part_end,
        Some(42),
        "its SoundEnded comes with the rest's end"
    );

    // Started inside a tick (startAt 1.0 s + 123 samples) from the same offset.
    let at = 1.0 + 123.0 / f64::from(RATE);
    let (late, _) = render_alone(
        &bank,
        AudioSource {
            start_at: at,
            start_offset: offset,
            ..whole
        },
        150,
    );
    let start = RATE as usize + 123;
    assert!(
        late[..start * 2].iter().all(|x| *x == 0.0),
        "silent until its startAt"
    );
    for i in 0..(RATE as usize - skip) {
        assert_eq!(
            f(&late, start + i),
            f(&full, skip + i),
            "frame {i} after its startAt is frame {} of the clip",
            skip + i
        );
    }
}
