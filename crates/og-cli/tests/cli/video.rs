use super::*;

/// `og video edges` writes the first and last frame of a clip as PNG, so
/// an author seeds the next AI clip from the last frame. Ways this could
/// fail: both frames come out the same (the last not sought), the files
/// are not written, a `type video` .ogt's source is not resolved.
#[test]
fn video_edges_writes_the_first_and_last_frame() {
    if which_ffmpeg().is_none() {
        eprintln!("skipped: no ffmpeg");
        return;
    }
    let clip = repo().join("packages/engine/tests/fixtures/video/assets/clips/count.mp4");
    let dir = TempDir::new("og-cli-video-edges");
    let out = og()
        .args(["video", "edges", clip.to_str().unwrap(), "--out"])
        .arg(&*dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let first = dir.join("count.first.png");
    let last = dir.join("count.last.png");
    let a = image::open(&first)
        .unwrap_or_else(|e| panic!("{}: {e}", first.display()))
        .to_rgba8();
    let b = image::open(&last)
        .unwrap_or_else(|e| panic!("{}: {e}", last.display()))
        .to_rgba8();
    assert_eq!(a.dimensions(), b.dimensions(), "the same size");
    // This fixture has 120 frames at 30 Hz. Compare the decoded frame
    // numbered 119, not merely a different frame (frame 90 is different too).
    assert_eq!(a, decoded_frame(&clip, 0, "rgb24"));
    assert_eq!(b, decoded_frame(&clip, 119, "rgb24"));

    // A `type video` .ogt resolves its source beside it.
    let ogt_dir = TempDir::new("og-cli-video-edges-ogt");
    std::fs::copy(&clip, ogt_dir.join("count.mp4")).unwrap();
    std::fs::write(
        ogt_dir.join("shot.ogt"),
        "type video\nversion 1\n---\nsource \"count.mp4\"\nmaxSeconds 1\n",
    )
    .unwrap();
    let out = og()
        .args(["video", "edges"])
        .arg(ogt_dir.join("shot.ogt"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    assert!(ogt_dir.join("shot.first.png").exists(), "{}", text(&out));
    assert_eq!(
        image::open(ogt_dir.join("shot.last.png"))
            .unwrap()
            .to_rgba8(),
        decoded_frame(&clip, 29, "rgb24"),
        "maxSeconds 1 ends before frame 30"
    );
}

/// Frame `index` of `clip` converted to `pix_fmt` as the command under
/// test converts it (edges writes ffmpeg's RGB PNG, study reads RGBA):
/// ffmpeg 7's YUV to RGB24 and to RGBA paths differ by a few levels.
fn decoded_frame(clip: &Path, index: u32, pix_fmt: &str) -> image::RgbaImage {
    let out = Command::new(which_ffmpeg().unwrap())
        .args(["-v", "error", "-nostdin", "-i"])
        .arg(clip)
        .args([
            "-vf",
            &format!("select=eq(n\\,{index})"),
            "-frames:v",
            "1",
            "-pix_fmt",
            pix_fmt,
            "-f",
            "image2pipe",
            "-c:v",
            "png",
            "-",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    image::load_from_memory(&out.stdout).unwrap().to_rgba8()
}

#[test]
fn video_study_samples_displayed_frames_and_is_byte_repeatable() {
    if which_ffmpeg().is_none() {
        eprintln!("skipped: no ffmpeg");
        return;
    }
    let clip = repo().join("packages/engine/tests/fixtures/video/assets/clips/count.mp4");
    let a = TempDir::new("og-cli-study-a");
    let b = TempDir::new("og-cli-study-b");
    for dir in [&a, &b] {
        let out = og()
            .args(["video", "study"])
            .arg(&clip)
            .args(["--at", "0.251,1.01,3.99", "--window", "1", "--out"])
            .arg(&**dir)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
    }
    let doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(a.join("study.json")).unwrap()).unwrap();
    assert_eq!(doc["duration"], 4.0);
    assert_eq!(doc["fps"], 30.0);
    assert_eq!(doc["width"], 192);
    assert_eq!(doc["height"], 108);
    assert_eq!(doc["source_frames"].as_array().unwrap().len(), 120);
    assert_eq!(doc["samples"].as_array().unwrap().len(), 16);
    for (i, index) in [7, 30, 119].into_iter().enumerate() {
        let frame = &doc["at"][i]["frame"];
        assert_eq!(frame["index"], index);
        let image = image::open(a.join(frame["file"].as_str().unwrap()))
            .unwrap()
            .to_rgba8();
        assert!(
            image == decoded_frame(&clip, index, "rgba"),
            "wrong displayed frame at request {i}"
        );
        assert!(
            frame["file"]
                .as_str()
                .unwrap()
                .contains(&format!("{:.9}s", index as f64 / 30.0))
        );
    }
    // The expected comparisons come from independently chosen fixture
    // frames, not the command's change series. In particular comparison 2
    // must be 7 -> 15, not 0 -> 15 (last keyframe, rather than last sample).
    let frames: Vec<_> = [0, 7, 15, 22, 30]
        .into_iter()
        .map(|i| {
            let image = decoded_frame(&clip, i, "rgba");
            og_probe::frame::Rgba {
                width: 192,
                height: 108,
                pixels: image.into_raw(),
            }
        })
        .collect();
    let mut values = Vec::new();
    for (i, pair) in frames.windows(2).enumerate() {
        let change =
            og_probe::frame::flip::flip(&pair[0], &pair[1], og_probe::frame::flip::DEFAULT_PPD)
                .unwrap()
                .stats()
                .mean as f64;
        assert!((doc["changes"][i]["mean_flip"].as_f64().unwrap() - change).abs() < 1e-7);
        values.push(change);
    }
    let mean = values.iter().sum::<f64>() / 4.0;
    values.sort_by(f64::total_cmp);
    let median = (values[1] + values[2]) / 2.0;
    assert_eq!(doc["windows"].as_array().unwrap().len(), 13);
    assert_eq!(doc["windows"][0]["count"], 4);
    assert!((doc["windows"][0]["mean"].as_f64().unwrap() - mean).abs() < 1e-7);
    assert!((doc["windows"][0]["median"].as_f64().unwrap() - median).abs() < 1e-7);
    // JSON excludes output paths and clocks; PNGs and the labelled sheet
    // must also be reproducible across directories and invocations.
    for file in ["study.json", "contact-sheet.png"].into_iter().chain(
        doc["frames"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["file"].as_str().unwrap()),
    ) {
        assert_eq!(
            std::fs::read(a.join(file)).unwrap(),
            std::fs::read(b.join(file)).unwrap(),
            "{file}"
        );
    }
    // Select all 120 frames at their midpoints, beyond ffmpeg's flat-sum
    // expression depth limit. Keep every PNG, but only 24 thumbnails:
    // dropping selections to cap the sheet would lose the proof frames.
    let dense = TempDir::new("og-cli-study-dense");
    let at = (0..120)
        .map(|i| ((i as f64 + 0.5) / 30.0).to_string())
        .collect::<Vec<_>>()
        .join(",");
    let out = og()
        .args(["video", "study"])
        .arg(&clip)
        .args(["--interval", "0.1", "--at", &at, "--out"])
        .arg(&*dense)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dense.join("study.json")).unwrap()).unwrap();
    assert_eq!(doc["frames"].as_array().unwrap().len(), 120);
    assert_eq!(
        std::fs::read_dir(dense.join("frames")).unwrap().count(),
        120
    );
    let sheet = doc["sheet_frames"].as_array().unwrap();
    assert_eq!(sheet.len(), 24);
    assert_eq!(sheet.first().unwrap(), 0);
    assert_eq!(sheet.last().unwrap(), 119);
    assert!(sheet.windows(2).all(|p| p[0].as_u64() < p[1].as_u64()));
    for index in sheet {
        assert!(
            doc["frames"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| &f["index"] == index)
        );
    }
    assert!(image::open(dense.join(doc["frames"][119]["file"].as_str().unwrap())).is_ok());
    // 4 columns x 6 rows of 192x108 cells, 6px gaps and 20px labels.
    assert_eq!(
        image::image_dimensions(dense.join("contact-sheet.png")).unwrap(),
        (798, 810)
    );
}

#[test]
fn video_study_preserves_variable_pts_and_nonzero_origin() {
    let Some(ffmpeg) = which_ffmpeg() else {
        eprintln!("skipped: no ffmpeg");
        return;
    };
    let dir = TempDir::new("og-cli-study-vfr");
    let clip = repo().join("packages/engine/tests/fixtures/video/assets/clips/count.mp4");
    // Keep source frames 0,3,21,45 without retiming: intervals .1,.6,.8.
    // PNG-in-MOV is lossless; a 2-second timestamp offset tests origin
    // handling. Even with a 1/30 s encoder time base, passthrough must not
    // fill the gaps with CFR frames (ffmpeg 7 refuses -r with passthrough).
    // The muxer chooses the final packet's duration.
    let out = Command::new(ffmpeg)
        .args(["-v", "error", "-nostdin", "-i"])
        .arg(&clip)
        .args([
            "-map",
            "0:v:0",
            "-vf",
            "select='eq(n,0)+eq(n,3)+eq(n,21)+eq(n,45)'",
            "-fps_mode",
            "passthrough",
            "-c:v",
            "png",
            "-enc_time_base",
            "1/30",
            "-output_ts_offset",
            "2",
        ])
        .arg(dir.join("vfr.mov"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let out = og()
        .args(["video", "study"])
        .arg(dir.join("vfr.mov"))
        .args(["--at", "0.69,0.7,1.49,1.5", "--interval", "0.2", "--out"])
        .arg(dir.join("study"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("study/study.json")).unwrap()).unwrap();
    let frames = doc["source_frames"].as_array().unwrap();
    assert_eq!(frames.len(), 4, "no CFR duplication");
    for (frame, time) in frames.iter().zip([0.0, 0.1, 0.7, 1.5]) {
        assert!((frame["time"].as_f64().unwrap() - time).abs() < 1e-9);
        assert!((frame["pts"].as_f64().unwrap() - (time + 2.0)).abs() < 1e-9);
    }
    assert!(
        doc["duration"].as_f64().unwrap() > 1.5,
        "includes the final frame's hold"
    );
    for (i, index) in [1, 2, 2, 3].into_iter().enumerate() {
        assert_eq!(doc["at"][i]["frame"]["index"], index);
    }
    assert_eq!(
        doc["changes"][1]["mean_flip"], 0.0,
        "same displayed frame at .2 and .4"
    );
    assert!(
        doc["windows"].as_array().unwrap().is_empty(),
        "no full 3-second window"
    );
    // An out-of-range time is an error, not a silently clamped tail.
    let out = og()
        .args(["video", "study"])
        .arg(dir.join("vfr.mov"))
        .args(["--at", "2", "--out"])
        .arg(dir.join("invalid"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(!dir.join("invalid").exists());
}

#[test]
fn video_study_cuts_have_separate_normalized_palettes() {
    let Some(ffmpeg) = which_ffmpeg() else {
        eprintln!("skipped: no ffmpeg");
        return;
    };
    let dir = TempDir::new("og-cli-study-cuts");
    // Exactly 2/3 red, 1/3 blue for .6 s, then solid green for .6 s.
    // Generate in RGB and encode losslessly: fractions have a spatial
    // derivation, independent of the palette implementation and YUV loss.
    let out = Command::new(ffmpeg).args(["-v", "error", "-nostdin", "-f", "lavfi", "-i",
        "color=black:s=24x16:r=10:d=1.2,format=rgb24,drawbox=x=0:y=0:w=16:h=16:color=red:t=fill:enable='lt(t,0.6)',drawbox=x=16:y=0:w=8:h=16:color=blue:t=fill:enable='lt(t,0.6)',drawbox=x=0:y=0:w=24:h=16:color=lime:t=fill:enable='gte(t,0.6)'",
        "-c:v", "png"]).arg(dir.join("cuts.mov")).output().unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let out = og()
        .args(["video", "study"])
        .arg(dir.join("cuts.mov"))
        .args(["--out"])
        .arg(dir.join("study"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("study/study.json")).unwrap()).unwrap();
    assert_eq!(doc["cuts"].as_array().unwrap().len(), 1);
    assert_eq!(doc["cuts"][0]["time"], 0.6);
    assert!(doc["cuts"][0]["scene_score"].as_f64().unwrap() > 0.35);
    assert_eq!(doc["segments"].as_array().unwrap().len(), 2);
    let fraction = |pal: &serde_json::Value, hex: &str| {
        pal["colors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["hex"] == hex)
            .unwrap()["fraction"]
            .as_f64()
            .unwrap()
    };
    let first = &doc["segments"][0]["palette"];
    let second = &doc["segments"][1]["palette"];
    assert!((fraction(first, "#ff0000") - 2.0 / 3.0).abs() < 1e-9);
    assert!((fraction(first, "#0000ff") - 1.0 / 3.0).abs() < 1e-9);
    assert_eq!(second["colors"].as_array().unwrap().len(), 1);
    assert_eq!(fraction(second, "#00ff00"), 1.0);
    for pal in [&doc["palette"], first, second] {
        let sum: f64 = pal["colors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["fraction"].as_f64().unwrap())
            .sum();
        assert!(
            (sum - 1.0).abs() < 1e-9,
            "normalized pixel fractions, got {sum}"
        );
    }
}

fn which_ffmpeg() -> Option<PathBuf> {
    let bin = std::env::var_os("OG_FFMPEG").unwrap_or_else(|| "ffmpeg".into());
    Command::new(&bin)
        .arg("-version")
        .output()
        .ok()
        .and_then(|o| o.status.success().then(|| PathBuf::from(bin)))
}
