//! Decoding beside the tick (`src/decode.rs`), through the public boundary:
//! a world with `og_audio::install`, rendered by a capture sink, with the
//! bank's helper run on a schedule of the test's choosing.
//!
//! Ways it could fail, and the test that catches each:
//! - a sound is decoded where it is inserted (the engine thread, inside
//!   `og.load.sound`) → `a_sound_is_indexed_at_insert_and_decoded_by_the_helper`
//! - the mix depends on when, or whether, the helper ran (a frame not
//!   decoded yet plays silence, or a helper's samples differ from a voice's
//!   own decode) → `the_mix_is_the_same_whatever_the_helper_did`
//! - the decode-ahead misses what the mixer reads next (a loop's wrap, a
//!   pitched voice, a fresh start), so the mixing thread decodes after all
//!   → the same test: a helper that keeps up leaves one miss, the start
//!   part-way into a long sound, which nothing could foresee
//! - the decode-ahead is unbounded (a long sound decoded whole, a voice
//!   pitched up planning seconds) or does not cross a loop's wrap →
//!   `the_decode_ahead_is_bounded_and_crosses_the_wrap`
//! - a helper's answer for another frame, or a late one, is taken →
//!   `answers_of_another_length_or_key_are_refused`

mod support;

use og_audio::qoa;
use og_audio::{
    AudioSource, AudioStream, DECODE_AHEAD_FRAMES, DecodeJob, DecodeStats, HeadlessMode,
    HeadlessSink, SoundBank,
};
use og_ecs::World;
use std::collections::VecDeque;
use std::f64::consts::TAU;
use support::*;

const QOA_FRAME: u64 = 5120;

fn encode_at(
    rate: u32,
    channels: usize,
    frames: usize,
    f: impl Fn(usize, usize) -> f64,
) -> Vec<u8> {
    let samples = synth(frames, channels, f);
    qoa::encode(&samples, channels, rate).expect("encode")
}

/// A rich tone (so a wrong frame shows): two partials and a slow sweep.
fn tone(rate: u32, base: f64) -> impl Fn(usize, usize) -> f64 {
    move |i, c| {
        let t = i as f64 / f64::from(rate);
        let f = base * (1.0 + 0.1 * (TAU * 0.3 * t).sin());
        0.4 * (TAU * f * t + c as f64).sin() + 0.2 * (TAU * 3.1 * f * t).sin()
    }
}

/// How the bank's helper runs in a scene.
#[derive(Clone, Copy, Debug)]
enum Helper {
    /// Never: every frame is decoded where a voice reads it.
    Never,
    /// After every tick, before the pump: every queued frame.
    KeepsUp,
    /// The bank's own thread, racing the ticks and the pumps.
    Thread,
    /// Like the web's decoder Worker, slow: four jobs a tick, each answered
    /// three ticks later.
    Late,
}

struct Ran {
    pcm: Vec<f32>,
    stream: u64,
    stats: DecodeStats,
}

/// Ten seconds: a 9 s stereo 44.1 kHz track looping 2.0–8.5 s (it wraps
/// at 8.5 s), a 7 s mono pad pitched 1.37 from tick 30 (to its end), a
/// short blip started every 40 ticks, and at tick 200 a second voice of
/// the track started 3 s into it (`startOffset`).
fn scene(helper: Helper) -> Ran {
    let bank = SoundBank::new();
    if matches!(helper, Helper::Thread) {
        assert!(bank.start_decode_thread());
    }
    let track = bank
        .insert_qoa(
            "track",
            &encode_at(44_100, 2, 9 * 44_100, tone(44_100, 220.0)),
        )
        .unwrap();
    let pad = bank
        .insert_qoa(
            "pad",
            &encode_at(RATE, 1, 7 * RATE as usize, tone(RATE, 330.0)),
        )
        .unwrap();
    let blip = bank
        .insert_qoa("blip", &encode_at(RATE, 1, 9600, tone(RATE, 990.0)))
        .unwrap();
    assert!(bank.get(track).unwrap().streamed() && bank.get(pad).unwrap().streamed());
    assert!(!bank.get(blip).unwrap().streamed());
    let (mut world, link) = world_with(&bank, |_| {});
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    world
        .spawn((AudioSource {
            looping: true,
            loop_start: 2.0,
            loop_end: 8.5,
            ..AudioSource::new(track)
        },))
        .unwrap();
    let mut late: VecDeque<(u64, Vec<DecodeJob>)> = VecDeque::new();
    for tick in 0..600u64 {
        if tick == 30 {
            world
                .spawn((AudioSource {
                    pitch: 1.37,
                    ..AudioSource::new(pad)
                },))
                .unwrap();
        }
        if tick % 40 == 5 {
            world.spawn((AudioSource::new(blip),)).unwrap();
        }
        if tick == 200 {
            world
                .spawn((AudioSource {
                    start_offset: 3.0,
                    volume: 0.5,
                    ..AudioSource::new(track)
                },))
                .unwrap();
        }
        world.step().unwrap();
        match helper {
            Helper::KeepsUp => {
                bank.decode_queued(usize::MAX);
            }
            Helper::Late => {
                late.push_back((tick, bank.take_decode_jobs(4)));
                while late.front().is_some_and(|(t, _)| t + 3 <= tick) {
                    for job in late.pop_front().unwrap().1 {
                        let samples = job.decode().unwrap();
                        assert!(bank.put_decoded(job.key, &samples));
                    }
                }
            }
            Helper::Never | Helper::Thread => {}
        }
        sink.pump();
    }
    let stream = world.resource::<AudioStream>().unwrap().hash;
    Ran {
        pcm: sink.take_pcm(),
        stream,
        stats: bank.decode_stats(),
    }
}

fn bits(pcm: &[f32]) -> Vec<u32> {
    pcm.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn the_mix_is_the_same_whatever_the_helper_did() {
    let never = scene(Helper::Never);
    let keeps_up = scene(Helper::KeepsUp);
    let thread = scene(Helper::Thread);
    let late = scene(Helper::Late);
    assert_eq!(never.pcm.len(), 600 * 800 * 2);
    let peak = never.pcm.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    assert!(peak > 0.2, "the scene is heard: peak {peak}");
    for (name, ran) in [
        ("keeps up", &keeps_up),
        ("thread", &thread),
        ("late", &late),
    ] {
        assert_eq!(ran.stream, never.stream, "{name}: the command stream");
        assert!(
            bits(&ran.pcm) == bits(&never.pcm),
            "{name}: the PCM differs from the mix decoded where it was read"
        );
    }
    // Without a helper every frame is decoded where it is read.
    assert_eq!(never.stats.ahead, 0);
    assert!(never.stats.misses > 100, "{:?}", never.stats);
    // A helper that keeps up leaves only the start part-way into the
    // track: its first frame is not planned before the tick that starts it.
    assert_eq!(keeps_up.stats.misses, 1, "{:?}", keeps_up.stats);
    assert!(keeps_up.stats.ahead > 100, "{:?}", keeps_up.stats);
    assert!(keeps_up.stats.plans >= 599, "{:?}", keeps_up.stats);
    assert_eq!(keeps_up.stats.queued, 0);
    // The slow helper decoded some, and voices decoded the rest themselves.
    assert!(
        late.stats.ahead > 0 && late.stats.misses > 1,
        "{:?}",
        late.stats
    );
    assert!(thread.stats.ahead > 0, "{:?}", thread.stats);
}

#[test]
fn a_sound_is_indexed_at_insert_and_decoded_by_the_helper() {
    let bank = SoundBank::new();
    // Six seconds of stereo at 48 kHz: short, so it stays decoded.
    let frames = 6 * RATE as usize;
    let file = encode_at(RATE, 2, frames, tone(RATE, 440.0));
    let sound = bank.insert_qoa("six", &file).unwrap();
    let qoa_frames = (frames as u64).div_ceil(QOA_FRAME);
    let stats = bank.decode_stats();
    assert_eq!(stats.queued, qoa_frames, "every frame queued, none decoded");
    assert_eq!(stats.ahead + stats.misses, 0);
    assert_eq!(bank.resident_bytes(), file.len(), "the QOA file alone");
    // A long one queues its first frame.
    let long = encode_at(RATE, 1, 7 * RATE as usize, tone(RATE, 220.0));
    bank.insert_qoa("seven", &long).unwrap();
    assert_eq!(bank.decode_stats().queued, qoa_frames + 1);
    assert_eq!(bank.decode_queued(usize::MAX) as u64, qoa_frames + 1);
    assert_eq!(
        bank.resident_bytes(),
        file.len() + frames * 2 * 2 + long.len() + QOA_FRAME as usize * 2
    );
    // Decoded by the helper, read by a voice: the mix of the whole file
    // decoded at once, sample for sample.
    let decoded = mix(&bank, sound, 330);
    assert_eq!(bank.decode_stats().misses, 0);
    assert!(bits(&decoded) == bits(&mix_whole(&file, 330)));
    assert!(decoded.iter().any(|x| x.abs() > 0.3));
}

/// `ticks` of `sound` played at pitch 1.1, captured.
fn mix(bank: &SoundBank, sound: u64, ticks: u64) -> Vec<f32> {
    let (mut world, link) = world_with(bank, |_| {});
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    world
        .spawn((AudioSource {
            pitch: 1.1,
            ..AudioSource::new(sound)
        },))
        .unwrap();
    for _ in 0..ticks {
        world.step().unwrap();
        sink.pump();
    }
    sink.take_pcm()
}

/// The same of a QOA file decoded whole first (inserted as PCM).
fn mix_whole(file: &[u8], ticks: u64) -> Vec<f32> {
    let bank = SoundBank::new();
    let d = qoa::decode(file).unwrap();
    let sound = bank
        .insert_pcm("whole", d.sample_rate, d.channels, &d.samples)
        .unwrap();
    mix(&bank, sound, ticks)
}

#[test]
fn the_decode_ahead_is_bounded_and_crosses_the_wrap() {
    // Everything streams: only first frames are pinned.
    let bank = SoundBank::with_resident_seconds(0.0);
    let file = encode_at(RATE, 1, 20 * RATE as usize, tone(RATE, 300.0));
    let fast = bank.insert_qoa("fast", &file).unwrap();
    bank.decode_queued(usize::MAX);
    let (mut world, link) = world_with(&bank, |_| {});
    let _sink = HeadlessSink::new(&link, HeadlessMode::Null);
    // Pitched 16: a quarter second is 37 QOA frames; at most the cursor's
    // and DECODE_AHEAD_FRAMES more are planned.
    let e = world
        .spawn((AudioSource {
            pitch: 16.0,
            ..AudioSource::new(fast)
        },))
        .unwrap();
    world.step().unwrap(); // starts it: 12 800 frames a tick
    world.step().unwrap(); // plans from frame 12 800 (QOA frame 2)
    let frames: Vec<u32> = bank
        .take_decode_jobs(usize::MAX)
        .iter()
        .map(|j| j.frame)
        .collect();
    let want: Vec<u32> = (2..=2 + DECODE_AHEAD_FRAMES as u32).collect();
    assert_eq!(frames, want);
    world.despawn(e).unwrap();

    // Pitch 1, looping 1.0–1.5 s: the loop start's frames are planned
    // before the wrap, nothing past the loop's end ever is, and what is
    // decoded ahead stays within the window.
    let lp = bank.insert_qoa("loop", &file).unwrap();
    world
        .spawn((AudioSource {
            looping: true,
            loop_start: 1.0,
            loop_end: 1.5,
            ..AudioSource::new(lp)
        },))
        .unwrap();
    bank.decode_queued(usize::MAX); // its first frame (a pin)
    let (loop_start, loop_end) = (RATE as u64, RATE as u64 * 3 / 2);
    let start_frame = (loop_start / QOA_FRAME) as u32;
    let last_frame = ((loop_end - 1) / QOA_FRAME) as u32;
    let mut first_wrap_planned_at = None;
    let before = bank.decode_stats().misses;
    let base = bank.resident_bytes();
    for tick in 0..150u64 {
        world.step().unwrap();
        let jobs = bank.take_decode_jobs(usize::MAX);
        for j in &jobs {
            assert!(
                j.frame <= last_frame,
                "tick {tick}: frame {} past the loop",
                j.frame
            );
            // The playhead passes the loop start's frame on its way in
            // (ticks 57–64); after that only the wrap asks for it.
            if j.frame == start_frame && tick > 66 && first_wrap_planned_at.is_none() {
                first_wrap_planned_at = Some(tick);
            }
            assert!(bank.put_decoded(j.key, &j.decode().unwrap()));
        }
        // The decode-ahead holds at most its window: the cursor's frame and
        // DECODE_AHEAD_FRAMES more, on each side of the wrap.
        let ahead = bank.resident_bytes() - base;
        assert!(
            ahead <= 2 * (DECODE_AHEAD_FRAMES as usize + 1) * QOA_FRAME as usize * 2,
            "tick {tick}: {ahead} B decoded ahead"
        );
    }
    // The playhead first wraps at 1.5 s (tick 90); the loop start's frame
    // was asked for a quarter second before.
    let at = first_wrap_planned_at.expect("the loop start was planned");
    assert!((70..90).contains(&at), "planned at tick {at}");
    let p = world
        .get::<og_audio::AudioPlayback>(world_entity_of(&world, lp))
        .unwrap();
    assert!(p.cursor >= loop_start as f64 && p.cursor < loop_end as f64);
    assert_eq!(
        bank.decode_stats().misses,
        before,
        "nothing pumped the sink"
    );
}

/// The entity playing `sound`.
fn world_entity_of(world: &World, sound: u64) -> og_ecs::Entity {
    let rows = world
        .query(&og_ecs::QueryDesc::new().read::<og_audio::AudioPlayback>())
        .unwrap();
    rows.flat_map(|a| a.entities().to_vec())
        .find(|&e| world.get::<og_audio::AudioPlayback>(e).unwrap().sound == sound)
        .expect("a playhead")
}

#[test]
fn answers_of_another_length_or_key_are_refused() {
    let bank = SoundBank::new();
    let file = encode_at(RATE, 2, 12_000, tone(RATE, 500.0));
    let sound = bank.insert_qoa("short", &file).unwrap();
    let jobs = bank.take_decode_jobs(usize::MAX);
    assert_eq!(jobs.len(), 3);
    assert_eq!(bank.decode_stats().in_flight, 3);
    let decoded: Vec<Vec<i16>> = jobs.iter().map(|j| j.decode().unwrap()).collect();
    // The last frame is 1760 samples a channel; a full frame's answer is not it.
    assert_eq!(decoded[2].len(), 1760 * 2);
    assert!(!bank.put_decoded(jobs[2].key, &decoded[0]));
    assert!(!bank.put_decoded(jobs[2].key, &decoded[2]), "answered once");
    assert!(!bank.put_decoded(9999, &decoded[0]), "no such job");
    assert!(bank.put_decoded(jobs[0].key, &decoded[0]));
    assert!(bank.put_decoded(jobs[1].key, &decoded[1]));
    assert_eq!(bank.decode_stats().in_flight, 0);
    // The refused frame is decoded where it is read, to the same samples.
    let decoded = mix(&bank, sound, 16);
    assert_eq!(bank.decode_stats().misses, 1);
    assert!(bits(&decoded) == bits(&mix_whole(&file, 16)));
}

/// The QOA file both decoders (this crate's and the web host's decoder
/// Worker, `packages/engine/src/web/audio-decoder.ts`) are checked against:
/// the same samples, frame by frame, or a voice would hear its own decode
/// and the Worker's differ. Two full frames and a short one of stereo:
/// full-scale square waves with noise (the clamp, the largest residuals),
/// a sweep, silence and impulses. `OG_WRITE_FIXTURE=1` writes it.
fn fixture_samples() -> Vec<i16> {
    let mut seed = 0x2545_f491_u32;
    let mut noise = move || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f64::from(seed >> 8) / f64::from(1u32 << 24) * 2.0 - 1.0
    };
    let mut out = Vec::new();
    for i in 0..12_000usize {
        let t = i as f64 / f64::from(RATE);
        let square = if (i / 37) % 2 == 0 { 1.0 } else { -1.0 };
        let l = if i < 7000 {
            square + 0.3 * noise()
        } else {
            0.05 * noise()
        };
        let r = match i {
            0..4000 => 0.9 * (TAU * (200.0 + 4000.0 * t) * t).sin(),
            4000..6000 => 0.0,
            _ if i % 500 == 0 => 1.0,
            _ => -0.4 * noise(),
        };
        for v in [l, r] {
            out.push((v.clamp(-1.0, 1.0) * 32767.0).round() as i16);
        }
    }
    out
}

/// FNV-1a over the samples' little-endian bytes.
fn fnv(samples: &[i16]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in samples.iter().flat_map(|s| s.to_le_bytes()) {
        h = (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// The decoded fixture's hash, also asserted by `audio-decoder.test.ts`.
const FIXTURE_HASH: u64 = 0x080b_f551_3188_bc60;

#[test]
fn the_shared_fixture_decodes_to_its_hash_frame_by_frame() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packages/engine/tests/fixtures/qoa/frames.qoa"
    );
    let file = qoa::encode(&fixture_samples(), 2, 44_100).unwrap();
    if std::env::var_os("OG_WRITE_FIXTURE").is_some() {
        std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
        std::fs::write(path, &file).unwrap();
    }
    assert_eq!(
        std::fs::read(path).unwrap(),
        file,
        "the encoder still writes the fixture"
    );
    let whole = qoa::decode(&file).unwrap().samples;
    assert_eq!(fnv(&whole), FIXTURE_HASH, "{:016x}", fnv(&whole));
    // The jobs a helper gets are one-frame files that decode to the same.
    let bank = SoundBank::with_resident_seconds(0.0);
    let sound = bank.insert_qoa("fixture", &file).unwrap();
    let (mut world, link) = world_with(&bank, |_| {});
    let _sink = HeadlessSink::new(&link, HeadlessMode::Null);
    world.spawn((AudioSource::new(sound),)).unwrap();
    world.step().unwrap();
    world.step().unwrap();
    let mut jobs = bank.take_decode_jobs(usize::MAX);
    jobs.sort_by_key(|j| j.frame);
    assert_eq!(jobs.iter().map(|j| j.frame).collect::<Vec<_>>(), [0, 1, 2]);
    let frames: Vec<i16> = jobs.iter().flat_map(|j| j.decode().unwrap()).collect();
    assert_eq!(fnv(&frames), FIXTURE_HASH);
}
