//! Compressed sound sources (MP3, Ogg Vorbis, FLAC) cook like a WAV: decoded
//! to 16-bit PCM, then the same QOA path, under their own asset id.
//!
//! Ways it could fail: the extension not taken as a sound (the file is
//! ignored), the decoder's rate or channel count lost, encoder delay or
//! padding left in (the length is off), channels interleaved wrongly (the
//! pitch moves), silence cooked, or an undecodable file refused without
//! naming the file and its format.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/audio")
        .join(name)
}

fn rms(s: &[i16]) -> f64 {
    (s.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>() / s.len().max(1) as f64).sqrt() / 32767.0
}

#[test]
fn mp3_and_ogg_sounds_cook_to_qoa_like_a_wav() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-sound-formats");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/audio")).unwrap();
    // tone.mp3: 1 s, 44.1 kHz stereo, 440 Hz left and 660 Hz right, 64 kbps.
    // tone.ogg: 0.5 s, 22.05 kHz mono, 330 Hz (ffmpeg, libmp3lame and libvorbis).
    for f in ["tone.mp3", "tone.ogg"] {
        std::fs::copy(fixture(f), root.join("assets/audio").join(f)).unwrap();
    }
    std::fs::write(root.join("assets/audio/broken.mp3"), b"not an mp3 at all").unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("audio/broken.mp3") && errors[0].contains("as MP3"),
        "the error names the file and the format: {}",
        errors[0]
    );

    let b = Bundle::open(&root.join("cooked")).unwrap();
    // (id, rate, channels, seconds, zero crossings a second of channel 0)
    for (id, rate, channels, seconds, crossings) in [
        ("audio/tone.mp3", 44_100, 2, 1.0, 880.0),
        ("audio/tone.ogg", 22_050, 1, 0.5, 660.0),
    ] {
        let s = b.load_sound(id).unwrap_or_else(|e| panic!("{id}: {e:?}"));
        assert_eq!(
            (s.meta.sample_rate, s.meta.channels),
            (rate, channels),
            "{id}"
        );
        let expected = seconds * f64::from(rate);
        assert!(
            (f64::from(s.meta.frames) - expected).abs() < expected * 0.03,
            "{id}: {} frames for {expected}",
            s.meta.frames
        );
        let d = s.decode().unwrap();
        assert_eq!((d.sample_rate, d.channels), (rate, channels as usize));
        assert_eq!(d.samples.len(), s.meta.frames as usize * channels as usize);
        // A 0.5-amplitude sine: RMS 0.35.
        assert!(rms(&d.samples) > 0.2, "{id}: rms {}", rms(&d.samples));
        let left: Vec<i16> = d
            .samples
            .iter()
            .step_by(channels as usize)
            .copied()
            .collect();
        let n = left
            .windows(2)
            .filter(|p| (p[0] >= 0) != (p[1] >= 0))
            .count() as f64;
        let per_second = n / (left.len() as f64 / f64::from(rate));
        assert!(
            (per_second - crossings).abs() < crossings * 0.05,
            "{id}: {per_second} zero crossings a second, a {crossings} tone"
        );
    }
}
