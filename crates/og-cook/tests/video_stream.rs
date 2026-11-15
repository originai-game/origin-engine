//! A streamed video (`stream true`, G3-2) cooks to AV1 in the engine's own
//! `og.VideoStream` blob (skipped without an ffmpeg with an AV1 encoder):
//! one unit per frame with a key frame every `keyframe` seconds, its poster
//! (frame 0 as rav1d decodes it) as an ordinary texture, its sound as Opus,
//! and no texture array; its index (`<id>#stream`) and each chunk
//! (`<id>#stream.<n>`, from a key frame on) are outputs of their own, not
//! ones the bundle loads at boot with the clips, and every unit read
//! through them decodes.
//!
//! Ways it could fail: ffmpeg's scene cuts or a dropped `-g` putting key
//! frames elsewhere (a seek decodes from the wrong place); a unit count off
//! by the last frame; the frames cut at 256 as a clip's are; the stream's
//! sound cooked as QOA; a chunk starting off a key frame (its player could
//! not start there), or its units placed from the stream's start instead
//! of the chunk's.

use og_cook::cook::{CookOptions, cook_project};
use og_cook::format;
use og_cook::runtime::{Bundle, CookedVideoStream};
use std::path::{Path, PathBuf};

/// A stream's index and its units read through every chunk, in frame order.
fn read_stream(b: &Bundle, id: &str) -> (CookedVideoStream, Vec<(bool, Vec<u8>)>) {
    assert_eq!(b.output(id).unwrap().ty, "og.VideoStream");
    let s = b.video_stream(id, b.read(id).unwrap().as_slice()).unwrap();
    let mut units = Vec::new();
    for n in 0..s.meta.chunks {
        let c = og_cook::runtime::video_chunk_id(id, n);
        assert_eq!(b.output(&c).unwrap().ty, "og.VideoChunk");
        let chunk = b.video_chunk(&c, b.read(&c).unwrap().as_slice()).unwrap();
        assert_eq!((chunk.chunk, chunk.first), (n, n * s.meta.chunk_frames));
        units.extend(
            s.split(&chunk)
                .unwrap()
                .into_iter()
                .map(|(k, d)| (k, d.to_vec())),
        );
    }
    assert!(
        b.output(&og_cook::runtime::video_chunk_id(id, s.meta.chunks))
            .is_err()
    );
    (s, units)
}

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
    // 4 s at 60 fps: 240 frames, a key frame every 0.5 s, chunks of 1 s.
    std::fs::write(
        root.join("assets/clips/count.ogt"),
        "type video\nversion 1\n---\nsource \"count.mp4\"\nstream true\nfps 60\nsize 128 72\nkeyframe 0.5\nchunk 1\naiGenerated true\n",
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
    assert_eq!(meta.ai_generated, 1, "aiGenerated true is kept");
    assert!(
        b.output("clips/count.ogt#frames").is_err(),
        "a stream has no texture array"
    );
    let (s, units) = read_stream(&b, "clips/count.ogt#stream");
    assert_eq!((s.meta.chunk_frames, s.meta.chunks), (60, 4));
    assert_eq!(units.len(), 240);
    let keys: Vec<usize> = units
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
    for (n, (_, unit)) in units.iter().enumerate() {
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

/// The H.264 samples of a stream as an Annex B stream ffmpeg reads: the
/// avcC record's parameter sets first, each NAL unit behind a start code.
fn annex_b(config: &[u8], units: &[(bool, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut at = 6;
    let sets = |count: usize, at: &mut usize, out: &mut Vec<u8>| {
        for _ in 0..count {
            let len = usize::from(u16::from_be_bytes([config[*at], config[*at + 1]]));
            out.extend_from_slice(&[0, 0, 0, 1]);
            out.extend_from_slice(&config[*at + 2..*at + 2 + len]);
            *at += 2 + len;
        }
    };
    sets(usize::from(config[5] & 0x1f), &mut at, &mut out);
    let pps = usize::from(config[at]);
    at += 1;
    sets(pps, &mut at, &mut out);
    let len = usize::from(config[4] & 3) + 1;
    for (_, u) in units {
        let mut i = 0;
        while i + len <= u.len() {
            let n = u[i..i + len]
                .iter()
                .fold(0usize, |v, &b| (v << 8) | usize::from(b));
            out.extend_from_slice(&[0, 0, 0, 1]);
            out.extend_from_slice(&u[i + len..i + len + n]);
            i += len + n;
        }
    }
    out
}

#[test]
fn a_stream_cooks_hevc_and_h264_renditions_frame_for_frame_with_its_key_frames() {
    let has = og_cook::cook::video::has_encoder;
    if og_cook::cook::video::av1_encoder().is_none() || !has("libx264") || !has("libx265") {
        eprintln!("skipped: no ffmpeg with AV1, libx264 and libx265");
        return;
    }
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-video-renditions");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/clips")).unwrap();
    std::fs::copy(fixture(), root.join("assets/clips/count.mp4")).unwrap();
    // 2 s at 30 fps, a key frame every half second, chunks of 1 s; the
    // default (HEVC), both, and none.
    for (name, line) in [
        ("default", ""),
        ("both", "renditions h264 hevc\n"),
        ("none", "renditions none\n"),
    ] {
        std::fs::write(
            root.join(format!("assets/clips/{name}.ogt")),
            format!("type video\nversion 1\n---\nsource \"count.mp4\"\nstream true\nfps 30\nmaxSeconds 2\nsize 192 108\nkeyframe 0.5\nchunk 1\naudio false\n{line}"),
        )
        .unwrap();
    }
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let stream = |id: &str| read_stream(&b, id);
    let av1 = stream("clips/both.ogt#stream");
    let keys = |s: &(CookedVideoStream, Vec<(bool, Vec<u8>)>)| {
        s.1.iter()
            .enumerate()
            .filter(|(_, u)| u.0)
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&av1), [0, 15, 30, 45]);
    assert_eq!((av1.0.meta.chunk_frames, av1.0.meta.chunks), (30, 2));
    for (id, codec, record) in [
        (
            "clips/both.ogt#streamhevc",
            format::video_codec::HEVC,
            "hvc1",
        ),
        (
            "clips/both.ogt#streamh264",
            format::video_codec::H264,
            "avc1",
        ),
        (
            "clips/default.ogt#streamhevc",
            format::video_codec::HEVC,
            "hvc1",
        ),
    ] {
        let r = stream(id);
        let s = &r.0;
        assert_eq!(
            (
                s.meta.codec,
                s.meta.width,
                s.meta.height,
                s.meta.fps,
                r.1.len()
            ),
            (codec, 192, 108, 30, 60),
            "{id}"
        );
        // The AV1's key frames and chunks exactly: a seek starts at the
        // same frame on every decoder, and a player fetches the same span.
        assert_eq!(keys(&r), keys(&av1), "{id}");
        assert_eq!(s.meta.key_interval, 15, "{id}");
        assert_eq!((s.meta.chunk_frames, s.meta.chunks), (30, 2), "{id}");
        let kind = if codec == format::video_codec::HEVC {
            og_video::mp4::Codec::Hevc
        } else {
            og_video::mp4::Codec::H264
        };
        let name = og_video::mp4::codec_string(kind, &s.config).unwrap();
        assert!(name.starts_with(record), "{id}: {name}");
    }
    assert!(
        b.output("clips/default.ogt#streamh264").is_err(),
        "H.264 only when named"
    );
    assert!(
        b.output("clips/none.ogt#streamhevc").is_err(),
        "renditions none: none"
    );

    // ffmpeg decodes the H.264 units, in order, to all 60 frames.
    let (h264, units) = stream("clips/both.ogt#streamh264");
    let file = root.join("units.h264");
    std::fs::write(&file, annex_b(&h264.config, &units)).unwrap();
    let out = std::process::Command::new(og_cook::cook::video::ffprobe_bin())
        .args([
            "-v",
            "error",
            "-count_frames",
            "-select_streams",
            "v",
            "-show_entries",
        ])
        .args(["stream=nb_read_frames,width,height", "-of", "csv=p=0"])
        .arg(&file)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "192,108,60");
}

#[test]
fn a_stream_wider_than_720p_gets_a_720p_rendition_for_the_webs_fallback() {
    if og_cook::cook::video::av1_encoder().is_none() {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    }
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-video-rendition");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/clips")).unwrap();
    std::fs::copy(fixture(), root.join("assets/clips/count.mp4")).unwrap();
    // 1440x810 (wider than 1280): 10 frames, a key frame every half second.
    std::fs::write(
        root.join("assets/clips/wide.ogt"),
        "type video\nversion 1\n---\nsource \"count.mp4\"\nstream true\nfps 10\nmaxSeconds 1\nsize 1440 810\nkeyframe 0.5\naudio false\n",
    )
    .unwrap();
    // The same at crf720 32, the stream's own crf.
    std::fs::write(
        root.join("assets/clips/same.ogt"),
        "type video\nversion 1\n---\nsource \"count.mp4\"\nstream true\nfps 10\nmaxSeconds 1\nsize 1440 810\nkeyframe 0.5\naudio false\ncrf720 32\n",
    )
    .unwrap();
    std::fs::write(
        root.join("assets/clips/small.ogt"),
        "type video\nversion 1\n---\nsource \"count.mp4\"\nstream true\nfps 10\nmaxSeconds 1\nsize 640 360\naudio false\n",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let id = "clips/wide.ogt#stream720";
    let (s, units) = read_stream(&b, id);
    assert_eq!((s.meta.width, s.meta.height, units.len()), (1280, 720, 10));
    let keys: Vec<bool> = units.iter().map(|u| u.0).collect();
    assert_eq!(
        keys.iter().filter(|k| **k).count(),
        2,
        "the main stream's key frames: 0 and 5"
    );
    assert!(keys[0] && keys[5]);
    let mut d = og_video::decode::Decoder::new(1).unwrap();
    let p = d.decode(&units[0].1).unwrap().unwrap();
    assert_eq!((p.width, p.height), (1280, 720));
    assert!(
        b.output("clips/small.ogt#stream720").is_err(),
        "a stream that fits has none"
    );
    // The rendition is coarser than its stream by default (crf + 6), and
    // crf720 overrides that.
    let bytes = |id: &str| -> usize { read_stream(&b, id).1.iter().map(|u| u.1.len()).sum() };
    assert!(
        bytes(id) < bytes("clips/same.ogt#stream720"),
        "default {} bytes, at the stream's crf {}",
        bytes(id),
        bytes("clips/same.ogt#stream720")
    );
}
