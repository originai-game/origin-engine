//! The procedural sound presets (`type sfx`) through the cook and back.
//!
//! Ways this could fail: a preset renders silence, clips, or runs on for
//! seconds; the same preset renders differently twice (machine or run
//! dependent); `seed`, `pitch` or `length` do nothing; an unknown preset or
//! entry is cooked instead of refused with the list; a cooked preset is not
//! an ordinary sound blob the runtime loads.

use og_cook::cook::sfx::{PRESETS, RATE, Variation, parse, preset_ogt, render};
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

fn peak(s: &[i16]) -> f64 {
    s.iter()
        .map(|x| f64::from(x.unsigned_abs()))
        .fold(0.0, f64::max)
        / 32767.0
}

/// Zero crossings per second over the first 50 ms after the attack: a pitch proxy.
fn crossings(s: &[i16]) -> usize {
    let w = &s[(RATE as usize / 200)..(RATE as usize / 200 + RATE as usize / 20).min(s.len())];
    w.windows(2).filter(|p| (p[0] >= 0) != (p[1] >= 0)).count()
}

#[test]
fn every_preset_renders_a_short_audible_unclipped_sound_the_same_twice() {
    assert!(PRESETS.len() >= 16);
    for (name, _) in PRESETS {
        let a = render(name, Variation::default()).unwrap();
        let b = render(name, Variation::default()).unwrap();
        assert_eq!(a, b, "{name} renders the same twice");
        let secs = a.len() as f64 / f64::from(RATE);
        assert!((0.02..=2.0).contains(&secs), "{name} lasts {secs:.2} s");
        let p = peak(&a);
        assert!(p > 0.15 && p < 0.999, "{name} peaks at {p:.3}");
    }
}

#[test]
fn seed_pitch_and_length_vary_a_preset() {
    let base = render("laser", Variation::default()).unwrap();
    let seeded = render(
        "laser",
        Variation {
            seed: 7,
            ..Variation::default()
        },
    )
    .unwrap();
    assert_ne!(base, seeded, "a seed varies it");
    assert_eq!(
        seeded,
        render(
            "laser",
            Variation {
                seed: 7,
                ..Variation::default()
            }
        )
        .unwrap()
    );
    let longer = render(
        "laser",
        Variation {
            length: 2.0,
            ..Variation::default()
        },
    )
    .unwrap();
    assert!(
        (longer.len() as f64 / base.len() as f64 - 2.0).abs() < 0.1,
        "length doubles it"
    );
    let coin = render("coin", Variation::default()).unwrap();
    let high = render(
        "coin",
        Variation {
            pitch: 2.0,
            ..Variation::default()
        },
    )
    .unwrap();
    let ratio = crossings(&high) as f64 / crossings(&coin) as f64;
    assert!(
        (ratio - 2.0).abs() < 0.15,
        "pitch 2 is an octave up: {ratio:.2}"
    );
}

#[test]
fn malformed_presets_are_refused_with_the_fix() {
    let unknown = parse("type sfx\nversion 1\n---\npreset \"kazoo\"\n").unwrap_err();
    assert!(
        unknown.contains("no preset 'kazoo'") && unknown.contains("coin"),
        "{unknown}"
    );
    let entry = parse("type sfx\nversion 1\n---\npreset \"coin\"\nreverb 1\n").unwrap_err();
    assert!(
        entry.contains("'reverb' is not a sound preset entry"),
        "{entry}"
    );
    let (name, v) = parse(&preset_ogt(
        "hit",
        Variation {
            seed: 3,
            pitch: 1.5,
            length: 0.5,
            volume: 0.7,
        },
    ))
    .unwrap();
    assert_eq!(
        (name.as_str(), v.seed, v.pitch, v.length, v.volume),
        ("hit", 3, 1.5, 0.5, 0.7)
    );
}

#[test]
fn a_cooked_preset_is_a_sound_the_runtime_loads() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-sfx");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/sfx")).unwrap();
    std::fs::write(
        root.join("assets/sfx/select.ogt"),
        preset_ogt("select", Variation::default()),
    )
    .unwrap();
    std::fs::write(
        root.join("assets/sfx/bad.ogt"),
        "type sfx\nversion 1\n---\npreset \"kazoo\"\n",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(format!("{errors:?}").contains("kazoo"));
    let sound = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_sound("sfx/select.ogt")
        .unwrap();
    let pcm = render("select", Variation::default()).unwrap();
    assert_eq!(sound.meta.frames as usize, pcm.len());
    assert_eq!(sound.meta.sample_rate, RATE);
    let decoded = sound.decode().unwrap().samples;
    let power = |f: &dyn Fn(usize) -> f64| (0..pcm.len()).map(|i| f(i).powi(2)).sum::<f64>();
    let signal = power(&|i| f64::from(pcm[i]));
    let noise = power(&|i| f64::from(decoded[i]) - f64::from(pcm[i]));
    let snr = 10.0 * (signal / noise).log10();
    assert!(
        snr > 20.0,
        "QOA keeps the preset's waveform: {snr:.1} dB SNR"
    );
}
