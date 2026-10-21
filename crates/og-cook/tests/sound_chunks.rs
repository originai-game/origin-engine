//! Long sounds cooked in chunks (`sound { stream true chunk s }`, or a QOA
//! file above 1 MiB): the sound's blob holds the QOA header and the first
//! chunk, every other chunk is a blob of its own (`<id>#chunk<n>`), and the
//! chunks are the whole file's frames, cut between frames, so they decode
//! to exactly its samples.
//!
//! Ways it could fail: chunks cut inside a frame or re-encoded (other
//! samples), a chunk missing or out of order, the threshold ignored, the
//! settings outside the stamp, a chunked sound's chunks packed with its
//! group (they are read ahead of the playheads, never with the pack).

use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::{Bundle, chunk_id};
use std::path::{Path, PathBuf};

fn wav(path: &Path, rate: u32, channels: u16, seconds: f64) {
    let spec = hound::WavSpec {
        channels,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).unwrap();
    let n = (seconds * f64::from(rate)) as usize;
    for i in 0..n {
        let t = i as f64 / f64::from(rate);
        for c in 0..channels {
            let f = 220.0 * (1.0 + (t * 0.7).floor() * 0.125) + f64::from(c) * 3.0;
            let v = 0.4 * (std::f64::consts::TAU * f * t).sin();
            w.write_sample((v * 32767.0) as i16).unwrap();
        }
    }
    w.finalize().unwrap();
}

fn project(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-sound-chunks")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/audio")).unwrap();
    root
}

fn cook(root: &Path) -> og_cook::Report {
    cook_project(root, &CookOptions::default()).unwrap()
}

#[test]
fn a_long_sound_cooks_in_chunks_that_decode_to_the_whole_files_samples() {
    let root = project("chunks");
    // 12.5 s at 22,050 Hz mono: 54 QOA frames; 1.5 s rounds up to 7 a chunk.
    wav(&root.join("assets/audio/theme.wav"), 22_050, 1, 12.5);
    wav(&root.join("assets/audio/whole.wav"), 22_050, 1, 12.5);
    std::fs::write(
        root.join("assets/audio/theme.wav.ogt"),
        "type import\nversion 1\n---\nsound {\n  stream true\n  chunk 1.5\n}\n",
    )
    .unwrap();
    let report = cook(&root);
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let theme = b.load_sound("audio/theme.wav").unwrap();
    let whole = b.load_sound("audio/whole.wav").unwrap();
    assert!(theme.chunked() && !whole.chunked());
    assert_eq!((theme.meta.chunk_frames, theme.meta.chunk_count), (7, 8));
    assert_eq!(
        (
            theme.meta.sample_rate,
            theme.meta.channels,
            theme.meta.frames
        ),
        (
            whole.meta.sample_rate,
            whole.meta.channels,
            whole.meta.frames
        )
    );
    assert!(
        theme.decode().is_err(),
        "a chunked sound is not decoded whole"
    );
    // The head and the chunks laid end to end are the whole file's bytes.
    let mut joined = theme.qoa.clone();
    for n in 1..theme.meta.chunk_count {
        let id = chunk_id("audio/theme.wav", n);
        assert_eq!(b.output(&id).unwrap().ty, "og.SoundChunk");
        let bytes = b.read(&id).unwrap();
        let (k, frames) = b.sound_chunk(&id, bytes.as_slice()).unwrap();
        assert_eq!(k, n);
        joined.extend_from_slice(&frames);
    }
    assert!(
        b.output(&chunk_id("audio/theme.wav", 8)).is_err(),
        "no ninth chunk"
    );
    assert_eq!(
        joined, whole.qoa,
        "the chunks are the whole file, cut between frames"
    );
    // Through og-audio's bank: the same samples, chunk by chunk.
    let bank = og_audio::SoundBank::with_resident_seconds(0.0);
    let h = bank
        .insert_chunked(
            "theme",
            &theme.qoa,
            theme.meta.chunk_frames,
            theme.meta.chunk_count,
        )
        .unwrap();
    for n in 1..theme.meta.chunk_count {
        let id = chunk_id("audio/theme.wav", n);
        let (_, frames) = b.sound_chunk(&id, b.read(&id).unwrap().as_slice()).unwrap();
        assert!(bank.put_chunk(h, n, &frames).unwrap());
    }
    assert_eq!(bank.get(h).unwrap().resident_chunks().len(), 8);

    // Up to date: nothing re-cooks; a chunk length change re-cooks only theme.
    let again = cook(&root);
    assert!(
        again
            .assets
            .iter()
            .all(|a| a.outputs.iter().all(|o| !o.cooked))
    );
    std::fs::write(
        root.join("assets/audio/theme.wav.ogt"),
        "type import\nversion 1\n---\nsound {\n  stream true\n  chunk 3\n}\n",
    )
    .unwrap();
    let edit = cook(&root);
    let cooked: Vec<&str> = edit
        .assets
        .iter()
        .flat_map(|a| a.outputs.iter().filter(|o| o.cooked).map(|o| o.id.as_str()))
        .collect();
    assert!(
        cooked.iter().all(|id| id.starts_with("audio/theme.wav")),
        "{cooked:?}"
    );
    let b = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(b.load_sound("audio/theme.wav").unwrap().meta.chunk_count, 5);
    assert!(
        !root.join("cooked/audio/theme.wav.chunk5.ogb").exists(),
        "the old chunks past the new last are removed"
    );
}

#[test]
fn the_threshold_and_the_settings_choose_chunks() {
    use og_cook::cook::audio::SoundSettings;
    let auto = SoundSettings::default();
    // 26.7 s of 22 kHz mono (the idle archetype's music) is 0.23 MiB: whole.
    assert_eq!(auto.chunking(22_050, 1, 588_000), None);
    // Three minutes of 48 kHz stereo is 6.7 MiB: 4 s chunks of 38 frames.
    assert_eq!(auto.chunking(48_000, 2, 180 * 48_000), Some((38, 45)));
    let never = SoundSettings {
        stream: Some(false),
        ..auto
    };
    assert_eq!(never.chunking(48_000, 2, 180 * 48_000), None);
    let parse = |body: &str| {
        let d = og_core::ogt::Document::parse(&format!(
            "type import\nversion 1\n---\nsound {{\n{body}\n}}\n"
        ))
        .unwrap();
        SoundSettings::parse(d.body.get("sound").and_then(|e| e.block.as_ref()))
    };
    assert_eq!(
        parse("  stream true\n  chunk 2").unwrap().stream,
        Some(true)
    );
    assert!(parse("  chunk 0.5").unwrap_err().contains("1..=60"));
    assert!(
        parse("  loud true")
            .unwrap_err()
            .contains("unknown sound setting")
    );
}

#[test]
fn a_chunked_sound_in_a_stream_group_packs_its_head_and_keeps_its_chunks_apart() {
    let root = project("group");
    wav(&root.join("assets/audio/theme.wav"), 22_050, 1, 3.0);
    std::fs::write(
        root.join("assets/audio/theme.wav.ogt"),
        "type import\nversion 1\n---\nsound {\n  stream true\n  chunk 1\n}\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(
        root.join("assets/scenes/main.ogt"),
        "type scene\nversion 1\n---\nname \"Main\"\nstream \"far\" {\n  asset \"audio/theme.wav\"\n}\n",
    )
    .unwrap();
    let report = cook(&root);
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    // The head (the QOA header and the first chunk) goes in the group's pack ...
    let head = b.output("audio/theme.wav").unwrap();
    assert_eq!(head.group.as_deref(), Some("far"));
    // ... and every other chunk stays a file of its own, read ahead of the
    // playheads once the group is resident (none before).
    let chunks: Vec<_> = b
        .manifest()
        .outputs()
        .filter(|o| o.ty == "og.SoundChunk")
        .collect();
    assert_eq!(chunks.len(), 2, "3 s in chunks of about 1 s");
    for o in &chunks {
        assert_eq!(o.group, None, "{} is not in the pack", o.id);
        assert!(root.join("cooked").join(&o.file).exists());
    }
    // The group knows the whole sound's shape: its stand-in is silence of that length.
    let far = b.streams().iter().find(|g| g.name == "far").unwrap();
    let shape = far
        .sounds
        .iter()
        .find(|s| s.id == "audio/theme.wav")
        .unwrap();
    assert_eq!(
        (shape.sample_rate, shape.channels, shape.frames),
        (22_050, 1, 66_150)
    );
}
