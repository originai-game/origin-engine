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
    // count.mp4 counts up, so its first and last frame differ.
    assert_ne!(a.as_raw(), b.as_raw(), "the last frame is not the first");

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
    assert!(ogt_dir.join("shot.last.png").exists(), "{}", text(&out));
}

fn which_ffmpeg() -> Option<PathBuf> {
    let bin = std::env::var_os("OG_FFMPEG").unwrap_or_else(|| "ffmpeg".into());
    Command::new(&bin)
        .arg("-version")
        .output()
        .ok()
        .and_then(|o| o.status.success().then(|| PathBuf::from(bin)))
}
