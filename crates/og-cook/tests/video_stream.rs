//! A streamed video (`stream true`, G3-2) cooks to AV1 in the engine's own
//! `og.VideoStream` blob (skipped without an ffmpeg with an AV1 encoder):
//! one unit per frame with a key frame every `keyframe` seconds, its poster
//! (frame 0 as rav1d decodes it) as an ordinary texture, its sound as Opus,
//! and no texture array; every unit decodes, and the stream file is its own
//! output, not one the bundle loads at boot with the clips.
//!
//! Ways it could fail: ffmpeg's scene cuts or a dropped `-g` putting key
//! frames elsewhere (a seek decodes from the wrong place); a unit count off
//! by the last frame; the frames cut at 256 as a clip's are; the stream's
//! sound cooked as QOA.

use og_cook::cook::{CookOptions, cook_project};
use og_cook::format;
use og_cook::runtime::Bundle;
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/engine/tests/fixtures/video/assets/clips/count.mp4")
}

#[test]
fn a_stream_cooks_to_av1_units_with_its_key_frames_poster_and_opus_sound() {
    if og_cook::cook::video::av1_encoder().is_none() {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    }
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-video-stream");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/clips")).unwrap();
    std::fs::copy(fixture(), root.join("assets/clips/count.mp4")).unwrap();
    // 4 s at 60 fps: 240 frames, a key frame every 0.5 s.
    std::fs::write(
        root.join("assets/clips/count.ogt"),
        "type video\nversion 1\n---\nsource \"count.mp4\"\nstream true\nfps 60\nsize 128 72\nkeyframe 0.5\n",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let meta = b
        .video(
            "clips/count.ogt",
            b.read("clips/count.ogt").unwrap().as_slice(),
        )
        .unwrap();
    assert_eq!(
        (
            meta.streamed,
            meta.codec,
            meta.frames,
            meta.fps,
            meta.width,
            meta.height
        ),
        (1, format::video_codec::AV1, 240, 60, 128, 72)
    );
    assert!(
        b.output("clips/count.ogt#frames").is_err(),
        "a stream has no texture array"
    );
    let id = "clips/count.ogt#stream";
    assert_eq!(b.output(id).unwrap().ty, "og.VideoStream");
    let s = b.video_stream(id, b.read(id).unwrap().as_slice()).unwrap();
    assert_eq!(s.units.len(), 240);
    let keys: Vec<usize> = s
        .units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.0)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(keys, (0..240).step_by(30).collect::<Vec<_>>());
    assert_eq!(s.meta.key_interval, 30);
    // Every unit decodes to a frame of the stream's size.
    let mut d = og_video::decode::Decoder::new(1).unwrap();
    let mut first = None;
    for (n, (_, unit)) in s.units.iter().enumerate() {
        let p = d
            .decode(unit)
            .unwrap()
            .unwrap_or_else(|| panic!("unit {n} shows no frame"));
        assert_eq!((p.width, p.height), (128, 72));
        first.get_or_insert(p);
    }
    // The poster is frame 0 (lossy as any colour texture, near it).
    let poster = b
        .load_texture("clips/count.ogt#poster")
        .unwrap()
        .to_texture_data()
        .unwrap();
    let f0 = first.unwrap();
    let err: f64 = poster
        .rgba
        .iter()
        .zip(f0.rgba.iter())
        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).abs())
        .sum::<f64>()
        / poster.rgba.len() as f64;
    assert!(err < 12.0, "the poster is frame 0 (mean error {err:.1})");
    let sound = b.load_sound("clips/count.ogt#audio").unwrap();
    assert!(
        sound.meta.codec == og_cook::format::sound_codec::OPUS,
        "a stream's sound is Opus"
    );
}
