//! Sounds cooked in chunks (og-cook's `sound { stream true }`): the bank
//! holds a chunked sound's first chunk and whichever others the host puts
//! there as `chunks_wanted` asks for them from the playheads, and drops the
//! rest. Here the host is the test: between ticks it asks, puts what is
//! missing and pumps the mixer, as og-runtime does natively.
//!
//! Ways it could fail, and what catches it:
//! - a chunk decodes other samples than the same frames of the whole file,
//!   a chunk lands at the wrong place, a wanted chunk arrives too late, a
//!   loop wrap or a start part-way reads a chunk that is not there → the
//!   PCM differs from the whole-file path's
//!   (`a_three_minute_track_streams_in_chunks_sample_identical_to_the_whole_file`)
//! - chunks are never dropped → the resident bound fails
//! - the music clock or a crossfade behaves differently → the beats and
//!   the PCM around the crossfade differ
//!
//! Evidence: `$OG_EVIDENCE_DIR/M6/chunks.json` (resident bytes, chunk
//! loads and drops, the PCM hash of both paths).

mod support;

use og_audio::{
    AudioSource, Bus, HeadlessMode, HeadlessSink, Music, MusicBeat, SoundBank, chunks_wanted, qoa,
};
use og_ecs::World;
use std::f64::consts::TAU;
use support::*;

/// QOA frames (5120 samples) per chunk: 3.9 s at 48 kHz, as the cook's
/// default 4 s rounds.
const PER_CHUNK: usize = 37;

/// A three-minute stereo track. Encoding three minutes at -O0 would take
/// minutes, so the track is phrases of ten QOA frames each (1.07 s),
/// encoded once and laid end to end in a sequence that never repeats a
/// pair: a QOA frame carries its predictor's state, so frames of separate
/// encodes concatenate into a valid file.
fn track() -> Vec<u8> {
    const PHRASE: usize = 10 * 5120;
    let roots = [110.0, 130.81, 146.83, 164.81, 196.0, 220.0, 246.94, 261.63];
    let phrases: Vec<Vec<u8>> = roots
        .iter()
        .enumerate()
        .map(|(k, &f)| {
            let pcm = synth(PHRASE, 2, |i, c| {
                let t = i as f64 / f64::from(RATE);
                let env = 0.6 + 0.4 * (TAU * 0.5 * t).sin();
                let detune = if c == 0 { 1.0 } else { 1.003 };
                env * (0.25 * (TAU * f * detune * t).sin()
                    + 0.15 * (TAU * f * 1.5 * t).sin()
                    + 0.1 * (TAU * f * (2.0 + k as f64 * 0.01) * t).sin())
            });
            qoa::encode(&pcm, 2, RATE).unwrap()
        })
        .collect();
    let count = (180.0 * f64::from(RATE) / PHRASE as f64).ceil() as usize; // 169
    let mut out = Vec::new();
    out.extend_from_slice(&0x716f_6166u32.to_be_bytes());
    out.extend_from_slice(&((count * PHRASE) as u32).to_be_bytes());
    let mut prev = 0usize;
    for n in 0..count {
        // A walk over the phrases: never the same twice in a row.
        let k = (prev + 1 + (n * 5 + n / 8) % 7) % phrases.len();
        out.extend_from_slice(&phrases[k][8..]);
        prev = k;
    }
    out
}

struct Run {
    pcm: Vec<f32>,
    beats: usize,
    /// The bank's resident sample bytes, the largest seen after a pump.
    peak: usize,
    loads: usize,
    drops: usize,
}

/// The scene: the track loops from 20 s on the music bus with a 120 bpm
/// clock; at 60 s a two-second crossfade to a second voice of the same
/// track started 100 s in (it wraps at 180 s back to 20 s), until 190 s.
/// With `chunks`, the test feeds the bank as og-runtime does (before and
/// after each tick); the mixer is pumped every `pump_every` ticks.
fn run(
    bank: &SoundBank,
    sound: u64,
    chunks: Option<&[Vec<u8>]>,
    pump_every: u64,
    seconds: u64,
) -> Run {
    let (mut world, link) = world_with(bank, |_| {});
    world
        .spawn((Music {
            bpm: 120.0,
            beats_per_bar: 4,
            ..Music::default()
        },))
        .unwrap();
    let looped = |offset: f64, fade: f32| AudioSource {
        looping: true,
        loop_start: 20.0,
        bus: Bus::Music as u32,
        start_offset: offset,
        fade,
        ..AudioSource::new(sound)
    };
    let a = world.spawn((looped(0.0, 0.0),)).unwrap();
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    let mut out = Run {
        pcm: Vec::new(),
        beats: 0,
        peak: 0,
        loads: 0,
        drops: 0,
    };
    let feed = |world: &World, out: &mut Run| {
        let Some(chunks) = chunks else { return };
        let wanted = chunks_wanted(world, bank);
        let before = bank.get(sound).unwrap().resident_chunks().len();
        let missing = bank.keep_chunks(&wanted);
        out.drops += before - bank.get(sound).unwrap().resident_chunks().len();
        for (h, _, c) in missing {
            assert!(bank.put_chunk(h, c, &chunks[c as usize]).unwrap());
            out.loads += 1;
        }
    };
    for tick in 0..seconds * 60 {
        feed(&world, &mut out);
        if tick == 60 * 60 {
            let s = world.get::<AudioSource>(a).unwrap();
            world
                .insert(
                    a,
                    AudioSource {
                        playing: false,
                        fade: 2.0,
                        ..s
                    },
                )
                .unwrap();
            world.spawn((looped(100.0, 2.0),)).unwrap();
        }
        world.step().unwrap();
        out.beats += world.pending_events::<MusicBeat>().unwrap().len();
        // After the tick, before the mixer: a voice started in it (part-way
        // too) gets its chunks before its first samples are rendered.
        feed(&world, &mut out);
        if (tick + 1) % pump_every == 0 {
            sink.pump();
            out.peak = out.peak.max(bank.resident_bytes());
        }
    }
    out.pcm = sink.pcm().to_vec();
    out
}

#[test]
fn a_three_minute_track_streams_in_chunks_sample_identical_to_the_whole_file() {
    let file = track();
    let decoded = qoa::decode(&file).unwrap();
    let frames = decoded.samples.len() / 2;
    assert!(frames as f64 / f64::from(RATE) >= 180.0, "three minutes");
    let parts = qoa::split(&file, PER_CHUNK).unwrap();
    let count = parts.len();

    // The whole-file path: one QOA file, streamed frame by frame.
    let whole_bank = SoundBank::new();
    let whole = whole_bank.insert_qoa("music", &file).unwrap();
    let w = run(&whole_bank, whole, None, 1, 190);

    // The chunked path: the first chunk inline, the others on demand.
    let bank = SoundBank::new();
    let sound = bank
        .insert_chunked("music", &parts[0], PER_CHUNK as u32, count as u32)
        .unwrap();
    assert_eq!(
        bank.info(sound),
        whole_bank.info(whole),
        "the same rate and length"
    );
    let c = run(&bank, sound, Some(&parts), 1, 190);

    assert_eq!(w.beats, c.beats, "the music clock");
    assert_eq!(w.beats, 190 * 2, "a beat every half second");
    assert_eq!(w.pcm.len(), c.pcm.len());
    let first_diff = w
        .pcm
        .iter()
        .zip(&c.pcm)
        .position(|(a, b)| a.to_bits() != b.to_bits());
    assert_eq!(
        first_diff, None,
        "the chunked mix differs from the whole file's at sample {first_diff:?}"
    );
    let hash = pcm_hash(&w.pcm);
    // Not silence: the crossfade is under way at 61 s, both voices sound.
    let at = |t: f64| {
        let f = (t * f64::from(RATE)) as usize;
        rms(&c.pcm[f * 2..(f + 4800) * 2])
    };
    assert!(at(30.0) > 0.05 && at(61.0) > 0.05 && at(150.0) > 0.05 && at(185.0) > 0.05);

    let whole_bytes = file.len();
    let chunk_bytes = parts[1].len();
    // Resident: the first chunk, the ones behind and ahead of two
    // playheads during the crossfade, and the loop start's.
    assert!(
        c.peak <= 8 * chunk_bytes,
        "{} B resident, {} B a chunk",
        c.peak,
        chunk_bytes
    );
    assert!(c.peak * 4 < whole_bytes, "{} B of {whole_bytes} B", c.peak);
    assert!(
        c.drops > 0 && c.loads >= count - 1,
        "{} loads, {} drops",
        c.loads,
        c.drops
    );
    std::fs::write(
        evidence_dir().join("chunks.json"),
        format!(
            "{{\"seconds\": {:.2}, \"channels\": 2, \"rate\": {RATE}, \"qoa_bytes\": {whole_bytes}, \"chunks\": {count}, \"chunk_bytes\": {chunk_bytes}, \"qoa_frames_per_chunk\": {PER_CHUNK}, \"whole_file_resident_peak\": {}, \"chunked_resident_peak\": {}, \"chunk_loads\": {}, \"chunk_drops\": {}, \"beats\": {}, \"pcm_frames\": {}, \"pcm_hash\": \"{hash:016x}\", \"identical\": true}}\n",
            frames as f64 / f64::from(RATE),
            w.peak,
            c.peak,
            c.loads,
            c.drops,
            c.beats,
            c.pcm.len() / 2
        ),
    )
    .unwrap();
}

#[test]
fn with_a_loader_a_chunk_is_read_when_a_voice_reaches_it() {
    // og-runtime's native loader: the mixer reads a chunk the moment a
    // voice needs it, so pumping ten seconds at once (og run's long
    // advances) still mixes every sample, with nobody feeding the bank.
    let file = track();
    let parts = std::sync::Arc::new(qoa::split(&file, PER_CHUNK).unwrap());
    let whole_bank = SoundBank::new();
    let whole = whole_bank.insert_qoa("music", &file).unwrap();
    let w = run(&whole_bank, whole, None, 600, 70);
    let bank = SoundBank::new();
    let sound = bank
        .insert_chunked("music", &parts[0], PER_CHUNK as u32, parts.len() as u32)
        .unwrap();
    let reads = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (p, r) = (parts.clone(), reads.clone());
    assert!(bank.set_chunk_loader(
        sound,
        std::sync::Arc::new(move |n| {
            r.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            p.get(n as usize).cloned()
        })
    ));
    let c = run(&bank, sound, None, 600, 70);
    assert!(
        w.pcm
            .iter()
            .zip(&c.pcm)
            .all(|(a, b)| a.to_bits() == b.to_bits())
            && w.pcm.len() == c.pcm.len(),
        "the loader's mix differs from the whole file's"
    );
    // 0-62 s and 100-110 s: 17 or 18 of the 45 chunks after the first.
    let reads = reads.load(std::sync::atomic::Ordering::Relaxed);
    assert!((15..20).contains(&reads), "{reads} reads");
}

#[test]
fn a_missing_chunk_plays_silence_and_a_bad_one_is_refused() {
    let file = track();
    let parts = qoa::split(&file, PER_CHUNK).unwrap();
    let bank = SoundBank::new();
    let sound = bank
        .insert_chunked("music", &parts[0], PER_CHUNK as u32, parts.len() as u32)
        .unwrap();
    assert!(bank.get(sound).unwrap().streamed());
    assert_eq!(bank.get(sound).unwrap().resident_chunks(), vec![0]);
    // A chunk of the right length is accepted; one of the wrong length is
    // not, nor one past the last, nor a chunk count the length disagrees with.
    assert!(bank.put_chunk(sound, 1, &parts[1]).unwrap());
    let last = parts.len() as u32 - 1;
    assert!(
        bank.put_chunk(sound, last, &parts[1]).is_err(),
        "the last chunk is shorter"
    );
    assert!(bank.put_chunk(sound, last + 1, &parts[1]).is_err());
    assert!(
        bank.insert_chunked("x", &parts[0], PER_CHUNK as u32, 3)
            .is_err()
    );
    // Dropping keeps the first chunk.
    let missing = bank.keep_chunks(&Default::default());
    assert!(missing.is_empty());
    assert_eq!(bank.get(sound).unwrap().resident_chunks(), vec![0]);
    // A voice past the first chunk with nothing put there hears silence.
    let (mut world, link) = world_with(&bank, |_| {});
    world
        .spawn((AudioSource {
            start_offset: 30.0,
            ..AudioSource::new(sound)
        },))
        .unwrap();
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    for _ in 0..30 {
        world.step().unwrap();
        sink.pump();
    }
    assert!(sink.pcm().iter().all(|x| *x == 0.0));
    // Asked for, put there, it plays.
    let missing = bank.keep_chunks(&chunks_wanted(&world, &bank));
    assert!(!missing.is_empty());
    for (h, id, c) in missing {
        assert_eq!(id, "music");
        bank.put_chunk(h, c, &parts[c as usize]).unwrap();
    }
    for _ in 0..30 {
        world.step().unwrap();
        sink.pump();
    }
    assert!(sink.pcm().iter().any(|x| x.abs() > 0.05));
}

fn pcm_hash(pcm: &[f32]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for x in pcm {
        for b in x.to_bits().to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    h
}
