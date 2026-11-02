//! rav1d decodes what the cook's ffmpeg encodes (skipped without an ffmpeg
//! with an AV1 encoder): every frame of a 3 s AV1 of the video fixture
//! (testsrc2 with its frame count burned in), against ffmpeg's own decode
//! of the same IVF; a seek through the decode thread gives the same bytes
//! as decoding through from the start.
//!
//! Ways this could fail: a hidden or a shown-existing frame counted as a
//! frame (every later frame one off); the chroma planes read at the luma
//! stride; BT.601 used for a BT.709 stream (a tint); a seek that decodes
//! from the frame wanted instead of its key frame (garbage until the next).

use og_video::decode::Decoder;
use og_video::ivf;
use og_video::stream::{Player, Units};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

fn ffmpeg() -> PathBuf {
    std::env::var_os("OG_FFMPEG").map_or_else(|| "ffmpeg".into(), PathBuf::from)
}

fn encoder() -> Option<&'static str> {
    let out = Command::new(ffmpeg())
        .args(["-hide_banner", "-encoders"])
        .output()
        .ok()?;
    let list = String::from_utf8_lossy(&out.stdout);
    ["libsvtav1", "libaom-av1"]
        .into_iter()
        .find(|e| list.contains(e))
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/engine/tests/fixtures/video/assets/clips/count.mp4")
}

fn rmse(a: &[u8], b: &[u8]) -> f64 {
    let s: f64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| (f64::from(*x) - f64::from(*y)).powi(2))
        .sum();
    (s / a.len() as f64).sqrt() / 255.0
}

#[test]
fn rav1d_decodes_the_cooks_av1_and_a_seek_matches_decoding_through() {
    let Some(enc) = encoder() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    let dir = std::env::temp_dir().join(format!("og-video-decode-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let ivf_path = dir.join("count.ivf");
    let run = |args: &[&str]| {
        let out = Command::new(ffmpeg()).args(args).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        out.stdout
    };
    let src = fixture();
    let mut args = vec![
        "-v",
        "error",
        "-y",
        "-i",
        src.to_str().unwrap(),
        "-t",
        "3",
        "-an",
    ];
    args.extend([
        "-vf", "fps=30", "-pix_fmt", "yuv420p", "-c:v", enc, "-g", "30",
    ]);
    args.extend([
        "-colorspace",
        "bt709",
        "-color_primaries",
        "bt709",
        "-color_trc",
        "bt709",
        "-color_range",
        "tv",
    ]);
    args.extend(["-f", "ivf", ivf_path.to_str().unwrap()]);
    run(&args);
    let file = ivf::read(&std::fs::read(&ivf_path).unwrap()).unwrap();
    assert_eq!((file.width, file.height), (192, 108));
    assert_eq!(file.units.len(), 90, "one unit per frame");
    let keys: Vec<usize> = file
        .units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.key)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(keys, [0, 30, 60], "a key frame every 30");

    // ffmpeg's own decode of the IVF, BT.709 limited range to RGBA.
    let theirs = run(&[
        "-v",
        "error",
        "-i",
        ivf_path.to_str().unwrap(),
        "-vf",
        "scale=in_color_matrix=bt709:in_range=tv:out_range=pc:flags=accurate_rnd+full_chroma_int,format=rgba",
        "-f",
        "rawvideo",
        "-",
    ]);
    let per = 192 * 108 * 4;
    assert_eq!(theirs.len(), per * 90);
    let mut d = Decoder::new(1).unwrap();
    let mut ours = Vec::new();
    for u in &file.units {
        let p = d
            .decode(&u.data)
            .unwrap()
            .expect("every unit shows a frame");
        assert_eq!((p.width, p.height), (192, 108));
        ours.push(p.rgba.to_vec());
    }
    for n in [0, 29, 30, 45, 89] {
        let e = rmse(&ours[n], &theirs[n * per..(n + 1) * per]);
        // No channel is biased (the matrix and range are right); edges
        // differ by the chroma upsampling (nearest here, ffmpeg's filter there).
        let t = &theirs[n * per..(n + 1) * per];
        for c in 0..3 {
            let bias = ours[n]
                .iter()
                .zip(t)
                .skip(c)
                .step_by(4)
                .map(|(a, b)| f64::from(*a) - f64::from(*b))
                .sum::<f64>()
                / (192.0 * 108.0);
            assert!(
                bias.abs() < 1.0,
                "frame {n}: channel {c} off by {bias:.2} on average"
            );
        }
        assert!(e < 0.05, "frame {n}: rmse {e:.4} against ffmpeg's decode");
        // Each frame is its own, not its neighbour's (the count changes).
        let other = rmse(
            &ours[n],
            &theirs[((n + 7) % 90) * per..((n + 7) % 90 + 1) * per],
        );
        assert!(other > e, "frame {n} nearer frame {}", (n + 7) % 90);
    }

    // Through the decode thread: a seek to 70 (key 60) and back to 10 (key 0).
    let player = Player::new(Arc::new(Units::new(file.units)), 2).unwrap();
    for (frame, looped) in [
        (0, false),
        (70, false),
        (71, false),
        (10, true),
        (89, true),
        (0, true),
    ] {
        player.want(frame, looped);
        let p = player.take(frame, true).unwrap().unwrap();
        assert!(
            *p.rgba == ours[frame as usize],
            "frame {frame} through the thread differs from decoding through"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
