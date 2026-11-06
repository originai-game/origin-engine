//! Opus sounds (`sound { codec opus }`): music and long sounds cook to Opus
//! in self-contained blocks (og-bundle's `opus`), resampled to 48 kHz; Ogg
//! Opus files are read as sources.
//!
//! Ways it could fail: the codec setting ignored (QOA cooked), the source's
//! rate kept or resampled to the wrong length, the encoder's delay left in
//! (the decoded tone a few samples late: the SNR collapses), the blocks'
//! seams stepping, the bitrate ignored (no saving over QOA), `stream true`
//! accepted for a sound that is never chunked, or an `.opus` source not
//! read (or read without its pre-skip trimmed).

use og_cook::cook::{CookOptions, audio, cook_project};
use og_cook::format::sound_codec;
use og_cook::runtime::Bundle;
use std::f64::consts::TAU;
use std::path::{Path, PathBuf};

/// A chord per channel: 440 + 660 Hz left, 550 + 880 Hz right, at `rate`.
fn chord(i: usize, c: u16, rate: u32) -> f64 {
    let t = i as f64 / f64::from(rate);
    let (a, b) = if c == 0 {
        (440.0, 660.0)
    } else {
        (550.0, 880.0)
    };
    0.3 * (TAU * a * t).sin() + 0.2 * (TAU * b * t).sin()
}

fn wav(path: &Path, rate: u32, channels: u16, seconds: f64) {
    let spec = hound::WavSpec {
        channels,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).unwrap();
    for i in 0..(seconds * f64::from(rate)) as usize {
        for c in 0..channels {
            w.write_sample((chord(i, c, rate) * 32767.0).round() as i16)
                .unwrap();
        }
    }
    w.finalize().unwrap();
}

fn project(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-sound-opus")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/audio")).unwrap();
    root
}

fn settings(root: &Path, file: &str, body: &str) {
    std::fs::write(
        root.join("assets/audio").join(format!("{file}.ogt")),
        format!("type import\nversion 1\n---\nsound {{\n{body}\n}}\n"),
    )
    .unwrap();
}

fn cooked_bytes(root: &Path, id: &str) -> u64 {
    let b = Bundle::open(&root.join("cooked")).unwrap();
    b.read(id).unwrap().as_slice().len() as u64
}

#[test]
fn a_sound_cooks_to_opus_at_48_khz_within_its_quality_bound() {
    let root = project("opus");
    // 7 s of a 44.1 kHz stereo chord: 15 blocks of 0.48 s at 48 kHz.
    wav(&root.join("assets/audio/theme.wav"), 44_100, 2, 7.0);
    wav(&root.join("assets/audio/same.wav"), 44_100, 2, 7.0);
    settings(&root, "theme.wav", "  codec opus\n  bitrate 96");
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let s = b.load_sound("audio/theme.wav").unwrap();
    assert_eq!(s.meta.codec, sound_codec::OPUS);
    assert_eq!((s.meta.sample_rate, s.meta.channels), (48_000, 2));
    // 7 s at 48 kHz exactly: 308,700 source frames times 48,000 / 44,100.
    assert_eq!(s.meta.frames, 336_000);
    assert_eq!(s.meta.chunk_count, 0, "an Opus sound is never chunked");
    assert!(s.qoa.is_empty() && !s.opus.is_empty());

    // The round trip against the chord synthesised at 48 kHz (not the
    // resampler's output: an independent expectation). Skip the first and
    // last 10 ms, where the source starts and stops abruptly.
    let d = s.decode().unwrap();
    assert_eq!(d.samples.len(), 336_000 * 2);
    let (mut signal, mut noise) = (0f64, 0f64);
    for i in 480..336_000 - 480 {
        for c in 0..2u16 {
            let want = chord(i, c, 48_000) * 32767.0;
            let got = f64::from(d.samples[i * 2 + c as usize]);
            signal += want * want;
            noise += (want - got) * (want - got);
        }
    }
    let snr = 10.0 * (signal / noise).log10();
    assert!(snr > 30.0, "round trip SNR {snr:.1} dB");

    // No block seam steps: the largest change between neighbouring samples
    // at a seam stays within the chord's own largest step (2π·880·0.5/48k
    // of full scale on the right channel, ~0.06) plus the codec's noise.
    let step = |i: usize, c: usize| {
        (f64::from(d.samples[i * 2 + c]) - f64::from(d.samples[(i - 1) * 2 + c])).abs() / 32767.0
    };
    for k in 1..14 {
        let i = k * 23_040;
        for c in 0..2 {
            assert!(step(i, c) < 0.08, "block {k} seam steps {:.3}", step(i, c));
        }
    }

    // About a fifth of the QOA bytes (96 kb/s against QOA's 3.2 bits per
    // sample at 44.1 kHz stereo, 282 kb/s).
    let (opus, qoa) = (
        cooked_bytes(&root, "audio/theme.wav"),
        cooked_bytes(&root, "audio/same.wav"),
    );
    assert!(opus * 2 < qoa, "Opus {opus} B against QOA {qoa} B");
    let note = report
        .assets
        .iter()
        .find(|a| a.source == "audio/theme.wav")
        .map(|a| a.notes.join("\n"))
        .unwrap_or_default();
    assert!(note.contains("to Opus at 96 kb/s"), "{note}");
}

#[test]
fn opus_settings_are_checked() {
    let parse = |body: &str| {
        let text = format!("type import\nversion 1\n---\nsound {{\n{body}\n}}\n");
        let doc = og_core::ogt::Document::parse(&text).unwrap();
        audio::SoundSettings::parse(doc.body.get("sound").and_then(|e| e.block.as_ref()))
    };
    let s = parse("  codec opus").unwrap();
    assert_eq!(s.codec, audio::Codec::Opus);
    assert_eq!((s.opus_bitrate(2), s.opus_bitrate(1)), (96_000, 64_000));
    assert_eq!(s.chunking(48_000, 2, 48_000 * 600), None, "never chunked");
    assert!(parse("  codec opus\n  stream true").is_err());
    assert!(parse("  codec mp3").is_err());
    assert!(parse("  codec opus\n  bitrate 8").is_err());
    assert_eq!(parse("  bitrate 128").unwrap().opus_bitrate(2), 128_000);
}

#[test]
fn an_ogg_opus_source_cooks_like_any_sound() {
    use opus_pure::{Application, OggOpusWriter, OpusEncoder, OpusHead};
    let root = project("ogg-opus");
    // 1 s of a mono 48 kHz chord through libopus's framing: pre-skip and a
    // final granule that trims the last packet's padding.
    let mut enc = OpusEncoder::new(48_000, 1, Application::Audio).unwrap();
    let head = OpusHead::for_encoder(&enc, 48_000);
    let pre = usize::from(head.pre_skip);
    let mut w = OggOpusWriter::new(Vec::new(), head).unwrap();
    let total = 48_000;
    let packets = (total + pre).div_ceil(960);
    let mut out = vec![0u8; 1500];
    for p in 0..packets {
        let pcm: Vec<f32> = (0..960)
            .map(|j| {
                let i = p * 960 + j;
                if i < total {
                    chord(i, 0, 48_000) as f32
                } else {
                    0.0
                }
            })
            .collect();
        let n = enc.encode(&pcm, 960, &mut out).unwrap();
        if p + 1 == packets {
            let end = (pre + total) as i64 - w.granule();
            w.write_packet_with_duration(&out[..n], end as u32).unwrap();
        } else {
            w.write_packet(&out[..n]).unwrap();
        }
    }
    std::fs::write(root.join("assets/audio/chord.opus"), w.finish().unwrap()).unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let s = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_sound("audio/chord.opus")
        .unwrap();
    // Read, trimmed to its 48,000 samples, and cooked to QOA (the default).
    assert_eq!(s.meta.codec, sound_codec::QOA);
    assert_eq!(
        (s.meta.sample_rate, s.meta.channels, s.meta.frames),
        (48_000, 1, 48_000)
    );
    let d = s.decode().unwrap();
    // The pre-skip trimmed: the chord is in phase with the source.
    let (mut signal, mut noise) = (0f64, 0f64);
    for (i, &sample) in d.samples.iter().enumerate().take(46_000).skip(2_000) {
        let want = chord(i, 0, 48_000) * 32767.0;
        let got = f64::from(sample);
        signal += want * want;
        noise += (want - got) * (want - got);
    }
    let snr = 10.0 * (signal / noise).log10();
    assert!(
        snr > 20.0,
        "an .opus source decoded {snr:.1} dB from its chord"
    );
}

#[test]
fn the_resampler_keeps_a_tone_and_the_length() {
    // 1 s of 1 kHz at 44.1 kHz to 48 kHz: 48,000 frames of the same tone.
    let src: Vec<i16> = (0..44_100)
        .map(|i| (16_000.0 * (TAU * 1000.0 * i as f64 / 44_100.0).sin()).round() as i16)
        .collect();
    let out = audio::resample(&src, 1, 44_100, 48_000);
    assert_eq!(out.len(), 48_000);
    let worst = (100..47_900)
        .map(|i| (f64::from(out[i]) - 16_000.0 * (TAU * 1000.0 * i as f64 / 48_000.0).sin()).abs())
        .fold(0.0, f64::max);
    assert!(worst < 4.0, "worst error {worst} of 16,000");
}
