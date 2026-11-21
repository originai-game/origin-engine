//! `og video`: tools for the footage of an interactive film (og.film's
//! segments).
//!
//! - `og video edges <path> [--out <dir>]` writes the first and last frame
//!   of a clip as PNG (`<stem>.first.png`, `<stem>.last.png`): the last
//!   frame is the seed an image-to-video model continues from for the next
//!   clip, the first lets an author check a cut joins. `path` is a media
//!   file (`.mp4`, `.webm`, `.mov`) or a `type video` `.ogt`, whose
//!   `source`, `maxSeconds` and `fps` it reads.

mod study;

use og_cook::cook::video::ffmpeg_bin;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "usage: og video edges <path> [--out <dir>]";

pub fn run(args: &[&str]) -> ExitCode {
    match args {
        ["study", rest @ ..] => match study::run(rest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("og video study: {e}");
                ExitCode::FAILURE
            }
        },
        ["edges", rest @ ..] => match edges(rest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("og video edges: {e}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("expected og video edges or og video study; see og video --help");
            ExitCode::from(2)
        }
    }
}

/// The media file an argument names: a media file as given, or the
/// `source` of a `type video` `.ogt` (beside it or under assets/), with
/// the seconds kept (`maxSeconds`), if any.
fn resolve(path: &Path) -> Result<(PathBuf, Option<f64>), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    if matches!(ext.as_deref(), Some("mp4" | "webm" | "mov")) {
        return Ok((path.to_path_buf(), None));
    }
    if ext.as_deref() != Some("ogt") {
        return Err(format!(
            "{} is neither a media file (.mp4, .webm, .mov) nor a type video .ogt",
            path.display()
        ));
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut source = None;
    let mut max_seconds = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("source ") {
            source = Some(rest.trim().trim_matches('"').to_string());
        } else if let Some(rest) = line.strip_prefix("maxSeconds ") {
            max_seconds = rest.trim().parse::<f64>().ok();
        }
    }
    let source = source.ok_or_else(|| format!("{} names no source", path.display()))?;
    let dir = path.parent().unwrap_or(Path::new("."));
    let beside = dir.join(&source);
    let file = if beside.exists() {
        beside
    } else {
        PathBuf::from(&source)
    };
    if !file.exists() {
        return Err(format!(
            "source '{source}' is not beside {}",
            path.display()
        ));
    }
    Ok((file, max_seconds))
}

fn edges(args: &[&str]) -> Result<(), String> {
    let mut path = None;
    let mut out = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--out" => {
                out = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--out takes a directory")?,
                ));
                i += 2;
            }
            a if a.starts_with("--") => return Err(format!("unknown option '{a}'; {USAGE}")),
            a => {
                if path.is_some() {
                    return Err(USAGE.into());
                }
                path = Some(PathBuf::from(a));
                i += 1;
            }
        }
    }
    let path = path.ok_or(USAGE)?;
    let (media, max_seconds) = resolve(&path)?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("the path has no name")?;
    let dir = out.unwrap_or_else(|| media.parent().unwrap_or(Path::new(".")).to_path_buf());
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let first = dir.join(format!("{stem}.first.png"));
    let last = dir.join(format!("{stem}.last.png"));
    write_frame(&media, Edge::First, max_seconds, &first)?;
    write_frame(&media, Edge::Last, max_seconds, &last)?;
    println!("{}\n{}", first.display(), last.display());
    Ok(())
}

enum Edge {
    First,
    Last,
}

/// Writes one frame of `media` as a PNG with ffmpeg. The last frame is the
/// one kept (`maxSeconds`, else the source's end): the model continues the
/// film from there.
fn write_frame(
    media: &Path,
    edge: Edge,
    max_seconds: Option<f64>,
    out: &Path,
) -> Result<(), String> {
    let mut cmd = Command::new(ffmpeg_bin());
    cmd.args(["-v", "error", "-nostdin", "-y", "-i"]).arg(media);
    cmd.args(["-map", "0:v:0", "-an", "-fps_mode", "passthrough"]);
    match edge {
        Edge::First => {
            cmd.args(["-frames:v", "1"]);
        }
        // Decode through the kept range and overwrite the same PNG. No
        // frame limit: that would keep its first frame, not its last.
        // Starting at zero also works for short clips and sparse VFR tails
        // where a one-second seek window can contain no frame at all.
        Edge::Last => {
            if let Some(s) = max_seconds.filter(|s| s.is_finite() && *s > 0.0) {
                cmd.args(["-t", &s.to_string()]);
            }
        }
    }
    cmd.args(["-update", "1"]).arg(out);
    let r = cmd
        .output()
        .map_err(|e| format!("cannot run ffmpeg ({}): {e}", ffmpeg_bin().display()))?;
    if !r.status.success() {
        return Err(format!(
            "ffmpeg cannot read {}: {}",
            media.display(),
            String::from_utf8_lossy(&r.stderr).trim()
        ));
    }
    Ok(())
}
