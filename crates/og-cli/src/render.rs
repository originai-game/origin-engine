//! `og render [project] --commands "..." | --timeline <name> [--fps F] [--from S] [--to S]
//! [--size WxH] [--out DIR | out.mp4 | out.webm] [--frames png|jpg] [--audio]
//! [--contact-sheet COLS] [--jobs N] [--seed S] [--json] [--no-check]`
//!
//! The video producer (docs/design.md §2.9). The game runs headless on the
//! native host at its own tick (`packages/engine/src/node/render.ts`); the
//! world is drawn once every `rate / fps` ticks (the fps must divide the
//! tick rate, else the error names the nearest that do) and that frame is
//! read back through the capture hook (og-probe's `Capture::frame`: no
//! second draw, no PNG, no JSON-RPC) and streamed here, where it is
//!
//! - written as `frames/00000.png` (or `.jpg`) when `--out` is a directory
//!   or `--frames` is given;
//! - piped to `ffmpeg` as raw RGBA when `--out` names an `.mp4` (H.264,
//!   yuv420p, CRF 18) or a `.webm` (VP9, CRF 32), with the capture mixer's
//!   WAV muxed in with `--audio` (AAC or Opus). Without ffmpeg (`og doctor`
//!   says where it was looked for; `OG_FFMPEG` names one) the frames are
//!   written beside the file instead and the run says so;
//! - hashed: `frames.json` lists every frame's tick, time and the SHA-256 of
//!   its RGBA bytes, and the audio's SHA-256. Two runs of one script give
//!   the same list (the engine is deterministic and the frame is read, not
//!   encoded, before hashing).
//!
//! `--contact-sheet COLS` writes a grid PNG of frames spaced over the range
//! with their times and ticks (`contact_sheet.rs`): the review surface.
//!
//! `--jobs N` splits the range in N parts. A leader process simulates the
//! script without drawing and writes a render snapshot (the world exactly,
//! og-probe's `render_snapshot`) a few frames before each part; each part
//! restores its snapshot, draws those warm-up frames (so the renderer's
//! history, TAA and adapted exposure, has settled) and renders its frames;
//! ffmpeg concatenates the parts. The leader also renders the audio.
//!
//! `--timeline <name>` is `--commands "0:timeline:play:<name>"` (before any
//! `--commands` given too) with the range `--to` defaulting to the
//! timeline's `duration`: it works once timelines exist (stream S2,
//! branch depth/timeline); until then the action is reported unknown.

use crate::contact_sheet;
use crate::project::{self, Project};
use serde_json::{Value, json};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitCode, Stdio};

const USAGE: &str = "usage: og render [project] --commands \"t:action[:args];...\" | --timeline <name> [--fps F] [--from S] [--to S] [--size WxH] [--out DIR|out.mp4|out.webm] [--frames png|jpg] [--audio] [--contact-sheet COLS] [--jobs N] [--seed S] [--json] [--no-check]";

/// Frames each job of `--jobs` draws before its range, unsent.
const WARMUP: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameFormat {
    Png,
    Jpg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Container {
    Mp4,
    Webm,
}

/// What `og render` was asked.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderArgs {
    pub project: String,
    pub commands: Option<String>,
    pub timeline: Option<String>,
    pub fps: u32,
    pub from: f64,
    pub to: Option<f64>,
    pub size: (u32, u32),
    pub out: Option<String>,
    pub frames: Option<FrameFormat>,
    pub audio: bool,
    pub contact_sheet: Option<u32>,
    pub jobs: u32,
    pub seed: Option<String>,
    pub json: bool,
    pub check: bool,
}

pub fn parse(args: &[&str]) -> Result<RenderArgs, String> {
    let mut r = RenderArgs {
        project: ".".into(),
        commands: None,
        timeline: None,
        fps: 30,
        from: 0.0,
        to: None,
        size: (1280, 720),
        out: None,
        frames: None,
        audio: false,
        contact_sheet: None,
        jobs: 1,
        seed: None,
        json: false,
        check: true,
    };
    let mut i = 0;
    while i < args.len() {
        let flag = args[i];
        let mut value = || -> Result<&str, String> {
            i += 1;
            args.get(i)
                .copied()
                .ok_or_else(|| format!("{flag} needs a value. {USAGE}"))
        };
        let seconds = |v: &str| -> Result<f64, String> {
            v.parse::<f64>()
                .ok()
                .filter(|s| s.is_finite() && *s >= 0.0)
                .ok_or_else(|| format!("{flag} takes seconds, 0 or more, not `{v}`"))
        };
        let count = |v: &str| -> Result<u32, String> {
            v.parse::<u32>()
                .ok()
                .filter(|n| *n > 0)
                .ok_or_else(|| format!("{flag} takes a whole number above 0, not `{v}`"))
        };
        match flag {
            "--commands" => r.commands = Some(value()?.to_string()),
            "--timeline" => r.timeline = Some(value()?.to_string()),
            "--fps" => r.fps = count(value()?)?,
            "--from" => r.from = seconds(value()?)?,
            "--to" => r.to = Some(seconds(value()?)?),
            "--size" => {
                let v = value()?;
                r.size = v
                    .split_once('x')
                    .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
                    .filter(|&(w, h): &(u32, u32)| w > 0 && h > 0 && w <= 8192 && h <= 8192)
                    .ok_or_else(|| {
                        format!("--size takes WIDTHxHEIGHT, such as 1280x720, not `{v}`")
                    })?;
            }
            "--out" => r.out = Some(value()?.to_string()),
            "--frames" => {
                r.frames = Some(match value()? {
                    "png" => FrameFormat::Png,
                    "jpg" | "jpeg" => FrameFormat::Jpg,
                    v => return Err(format!("--frames takes png or jpg, not `{v}`")),
                })
            }
            "--audio" => r.audio = true,
            "--contact-sheet" => r.contact_sheet = Some(count(value()?)?),
            "--jobs" => r.jobs = count(value()?)?,
            "--seed" => r.seed = Some(value()?.to_string()),
            "--json" => r.json = true,
            "--no-check" => r.check = false,
            f if f.starts_with('-') => return Err(format!("unknown option `{f}`. {USAGE}")),
            p => r.project = p.to_string(),
        }
        i += 1;
    }
    if r.commands.is_none() && r.timeline.is_none() {
        return Err(format!("say what plays: --commands or --timeline. {USAGE}"));
    }
    if r.timeline.is_none() && r.to.is_none() {
        return Err("--to names where the render ends, in seconds (--timeline defaults it to the timeline's duration)".into());
    }
    if let Some(to) = r.to
        && to <= r.from
    {
        return Err(format!("--to {to} is not after --from {}", r.from));
    }
    Ok(r)
}

/// Where a render writes.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    /// The video, when `--out` named one.
    pub video: Option<(PathBuf, Container)>,
    /// The directory of frames.json, the sheet, the WAV and the script's outputs.
    pub dir: PathBuf,
    /// Prefix of the files beside a video (`intro.` of `intro.mp4`), empty for a directory.
    pub prefix: String,
    /// The frames' directory and format, when frames are written.
    pub frames: Option<(PathBuf, FrameFormat)>,
}

impl Layout {
    pub fn of(out: &Path, frames: Option<FrameFormat>, ffmpeg: bool) -> Layout {
        let container = match out.extension().and_then(|e| e.to_str()) {
            Some(e) if e.eq_ignore_ascii_case("mp4") => Some(Container::Mp4),
            Some(e) if e.eq_ignore_ascii_case("webm") => Some(Container::Webm),
            _ => None,
        };
        match container {
            None => Layout {
                video: None,
                dir: out.to_path_buf(),
                prefix: String::new(),
                frames: Some((out.join("frames"), frames.unwrap_or(FrameFormat::Png))),
            },
            Some(c) => {
                let dir = out.parent().unwrap_or(Path::new(".")).to_path_buf();
                let stem = out.file_stem().unwrap_or_default().to_string_lossy();
                // Without ffmpeg the frames are what the render leaves.
                let frames = match (frames, ffmpeg) {
                    (Some(f), _) => Some(f),
                    (None, false) => Some(FrameFormat::Png),
                    (None, true) => None,
                };
                Layout {
                    video: ffmpeg.then(|| (out.to_path_buf(), c)),
                    prefix: format!("{stem}."),
                    frames: frames.map(|f| (dir.join(format!("{stem}-frames")), f)),
                    dir,
                }
            }
        }
    }

    fn file(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{}{name}", self.prefix))
    }
}

/// The `ffmpeg` og render pipes to: `OG_FFMPEG`, else `ffmpeg` on PATH;
/// `None` when it does not answer `-version`. Its version line, for og doctor.
pub fn ffmpeg() -> Option<(PathBuf, String)> {
    let bin = std::env::var_os("OG_FFMPEG")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("ffmpeg"));
    let out = Command::new(&bin).arg("-version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let version = text
        .lines()
        .next()?
        .split_whitespace()
        .nth(2)
        .unwrap_or("?")
        .to_string();
    Some((bin, version))
}

/// The `duration` of timeline `name` (`assets/timelines/<name>.ogt`, or any
/// `.ogt` of `type timeline` named so under assets/).
fn timeline_duration(root: &Path, name: &str) -> Result<f64, String> {
    let direct = root.join("assets/timelines").join(format!("{name}.ogt"));
    let mut candidates = vec![direct];
    let mut stack = vec![root.join("assets")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_stem().is_some_and(|s| s == name)
                && p.extension().is_some_and(|x| x == "ogt")
            {
                candidates.push(p);
            }
        }
    }
    for p in candidates {
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        let Ok(doc) = og_core::ogt::parse(&text) else {
            continue;
        };
        if doc.ty != "timeline" {
            continue;
        }
        return doc
            .body
            .get("duration")
            .and_then(|d| d.number(0).ok())
            .map(|n| n.as_f64())
            .filter(|d| *d > 0.0)
            .ok_or_else(|| format!("{}: no `duration <seconds>` line", p.display()));
    }
    Err(format!(
        "no timeline `{name}` under {}/assets (an .ogt of `type timeline`; timelines are stream S2's, docs/depth.md §2)",
        root.display()
    ))
}

/// One frame's line of frames.json.
#[derive(Clone, Debug)]
struct FrameMeta {
    index: u64,
    tick: u64,
    time: f64,
    sha256: String,
    file: Option<String>,
}

/// What one bun process of the render reported.
struct JobOut {
    frames: Vec<FrameMeta>,
    thumbs: Vec<(usize, image::RgbaImage, String)>,
    end: Value,
}

/// Reads one record (length, JSON, payload) of render.ts's stdout.
fn read_record(r: &mut impl Read) -> Result<Option<(Value, Vec<u8>)>, String> {
    let mut len = [0u8; 4];
    match r.read_exact(&mut len) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.to_string()),
    }
    let mut head = vec![0u8; u32::from_le_bytes(len) as usize];
    r.read_exact(&mut head).map_err(|e| e.to_string())?;
    let head: Value = serde_json::from_slice(&head).map_err(|e| format!("a bad record: {e}"))?;
    let mut payload = vec![0u8; head["bytes"].as_u64().unwrap_or(0) as usize];
    r.read_exact(&mut payload).map_err(|e| e.to_string())?;
    Ok(Some((head, payload)))
}

fn write_frame(path: &Path, w: u32, h: u32, rgba: &[u8], f: FrameFormat) -> Result<(), String> {
    let img = image::RgbaImage::from_raw(w, h, rgba.to_vec()).ok_or("a short frame")?;
    let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = std::io::BufWriter::new(file);
    match f {
        FrameFormat::Png => {
            let enc = image::codecs::png::PngEncoder::new_with_quality(
                &mut out,
                image::codecs::png::CompressionType::Fast,
                image::codecs::png::FilterType::Adaptive,
            );
            img.write_with_encoder(enc)
        }
        FrameFormat::Jpg => {
            let rgb = image::DynamicImage::ImageRgba8(img).to_rgb8();
            rgb.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut out, 92,
            ))
        }
    }
    .map_err(|e| format!("{}: {e}", path.display()))
}

/// ffmpeg reading raw RGBA frames on stdin into `out` (video only).
fn encoder(
    ffmpeg: &Path,
    size: (u32, u32),
    fps: u32,
    container: Container,
    out: &Path,
) -> Result<(Child, ChildStdin), String> {
    let mut c = Command::new(ffmpeg);
    c.args(["-hide_banner", "-loglevel", "error", "-y", "-f", "rawvideo"])
        .args(["-pix_fmt", "rgba", "-s", &format!("{}x{}", size.0, size.1)])
        .args(["-r", &fps.to_string(), "-i", "-", "-an"]);
    match container {
        Container::Mp4 => c.args([
            "-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p",
        ]),
        Container::Webm => c.args([
            "-c:v",
            "libvpx-vp9",
            "-b:v",
            "0",
            "-crf",
            "32",
            "-row-mt",
            "1",
            "-pix_fmt",
            "yuv420p",
        ]),
    };
    c.arg(out).stdin(Stdio::piped());
    let mut child = c
        .spawn()
        .map_err(|e| format!("cannot start {}: {e}", ffmpeg.display()))?;
    let stdin = child.stdin.take().expect("piped");
    Ok((child, stdin))
}

fn ffmpeg_run(ffmpeg: &Path, args: &[&str]) -> Result<(), String> {
    let out = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(args)
        .output()
        .map_err(|e| format!("cannot start {}: {e}", ffmpeg.display()))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "ffmpeg {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Everything a job thread needs.
struct JobPlan {
    cmd: Command,
    layout: Layout,
    picks: Vec<usize>,
    segment: Option<(PathBuf, PathBuf, Container)>,
    size: (u32, u32),
    fps: u32,
}

/// Runs one bun process of the render: its frames written, encoded and hashed.
fn run_job(mut p: JobPlan) -> Result<JobOut, String> {
    p.cmd.stdout(Stdio::piped());
    let mut child = p
        .cmd
        .spawn()
        .map_err(|e| format!("{} ({e})", project::NO_BUN))?;
    let mut stdout = BufReader::with_capacity(1 << 20, child.stdout.take().expect("piped"));
    let mut enc = match &p.segment {
        Some((ff, out, c)) => Some(encoder(ff, p.size, p.fps, *c, out)?),
        None => None,
    };
    let mut out = JobOut {
        frames: Vec::new(),
        thumbs: Vec::new(),
        end: Value::Null,
    };
    let result = (|| -> Result<(), String> {
        while let Some((head, rgba)) = read_record(&mut stdout)? {
            match head["kind"].as_str() {
                Some("frame") => {
                    let (w, h) = (
                        head["width"].as_u64().unwrap_or(0) as u32,
                        head["height"].as_u64().unwrap_or(0) as u32,
                    );
                    let index = head["index"].as_u64().unwrap_or(0);
                    let mut meta = FrameMeta {
                        index,
                        tick: head["tick"].as_u64().unwrap_or(0),
                        time: head["time"].as_f64().unwrap_or(0.0),
                        sha256: head["sha256"].as_str().unwrap_or("").to_string(),
                        file: None,
                    };
                    if let Some((dir, f)) = &p.layout.frames {
                        let ext = if *f == FrameFormat::Png { "png" } else { "jpg" };
                        let name = format!("{index:05}.{ext}");
                        write_frame(&dir.join(&name), w, h, &rgba, *f)?;
                        meta.file = Some(
                            dir.strip_prefix(&p.layout.dir)
                                .unwrap_or(dir)
                                .join(&name)
                                .to_string_lossy()
                                .into_owned(),
                        );
                    }
                    if let Some((_, stdin)) = enc.as_mut() {
                        stdin
                            .write_all(&rgba)
                            .map_err(|e| format!("ffmpeg stopped taking frames: {e}"))?;
                    }
                    if p.picks.binary_search(&(index as usize)).is_ok() {
                        let label = contact_sheet::label(meta.time, meta.tick);
                        out.thumbs.push((
                            index as usize,
                            contact_sheet::thumbnail(w, h, rgba),
                            label,
                        ));
                    }
                    out.frames.push(meta);
                }
                Some("end") => out.end = head,
                other => return Err(format!("an unknown record `{other:?}`")),
            }
        }
        Ok(())
    })();
    let status = child.wait().map_err(|e| e.to_string())?;
    if let Some((mut c, stdin)) = enc {
        drop(stdin);
        let s = c.wait().map_err(|e| e.to_string())?;
        if !s.success() && result.is_ok() {
            return Err(format!("ffmpeg failed encoding a segment ({s})"));
        }
    }
    result?;
    if !status.success() {
        return Err(format!(
            "the game process exited with {status} (its stderr is above)"
        ));
    }
    if out.end.is_null() {
        return Err("the game process ended without its end record".into());
    }
    Ok(out)
}

pub fn run(args: &[&str]) -> ExitCode {
    match render(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, e)) => {
            eprintln!("og render: {e}");
            ExitCode::from(code)
        }
    }
}

fn render(args: &[&str]) -> Result<(), (u8, String)> {
    let a = parse(args).map_err(|e| (2, e))?;
    let fail = |e: String| (1u8, e);
    let p = Project::open(Path::new(&a.project)).map_err(fail)?;
    let package = project::engine_package(Some(&p.root)).ok_or((1, project::NO_ENGINE.into()))?;
    if let Some(n) = project::pin_notice(&p, &package) {
        eprintln!("og render: {n}");
    }
    // --timeline: the play action first, the range its duration.
    let mut script = a.commands.clone().unwrap_or_default();
    let mut to = a.to;
    if let Some(name) = &a.timeline {
        let play = format!("0:timeline:play:{name}");
        script = if script.is_empty() {
            play
        } else {
            format!("{play};{script}")
        };
        if to.is_none() {
            to = Some(timeline_duration(&p.root, name).map_err(fail)?);
        }
    }
    let to = to.expect("parse requires --to without --timeline");
    if to <= a.from {
        return Err((2, format!("--to {to} is not after --from {}", a.from)));
    }
    let total = ((to - a.from) * f64::from(a.fps)).round().max(1.0) as u32;
    let ff = ffmpeg();
    let out = a
        .out
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| p.root.join("og-out/render"));
    let out = std::path::absolute(&out).map_err(|e| fail(e.to_string()))?;
    let layout = Layout::of(&out, a.frames, ff.is_some());
    if out.extension().is_some_and(|e| e == "mp4" || e == "webm") && ff.is_none() {
        eprintln!(
            "og render: no ffmpeg (OG_FFMPEG or PATH; og doctor): writing the frames to {} instead of {}",
            layout
                .frames
                .as_ref()
                .map_or(String::new(), |f| f.0.display().to_string()),
            out.display()
        );
    }
    if layout.video.is_some() && (a.size.0 % 2 == 1 || a.size.1 % 2 == 1) {
        return Err((
            2,
            format!(
                "--size {}x{}: a video's width and height are even (yuv420p)",
                a.size.0, a.size.1
            ),
        ));
    }
    std::fs::create_dir_all(&layout.dir)
        .map_err(|e| fail(format!("{}: {e}", layout.dir.display())))?;
    if let Some((d, _)) = &layout.frames {
        // A previous render's frames would mix with these.
        let _ = std::fs::remove_dir_all(d);
        std::fs::create_dir_all(d).map_err(|e| fail(format!("{}: {e}", d.display())))?;
    }
    let bundle = crate::run::prepare(&p, &package, a.check, a.json).map_err(fail)?;
    let work = std::env::temp_dir().join(format!("og-render-{}", std::process::id()));
    std::fs::create_dir_all(&work).map_err(|e| fail(e.to_string()))?;

    let base = |extra: &[String]| -> Command {
        let mut c = project::bun(&package, "src/node/render.ts");
        c.arg("--entry").arg(&bundle);
        if p.manifest {
            c.arg("--root").arg(&p.root);
        }
        c.args(["--fps", &a.fps.to_string(), "--from", &a.from.to_string()])
            .args(["--size", &format!("{}x{}", a.size.0, a.size.1)])
            .args(["--commands", &script])
            .args(["--warmup", &WARMUP.to_string()])
            // Reproducible like a playtest: og.platform.now starts at 2026-01-01.
            .args(["--clock", "1767225600000"])
            .arg("--out")
            .arg(layout.file("outputs"));
        if let Some(s) = &a.seed {
            c.args(["--seed", s]);
        }
        c.args(extra);
        c
    };
    let jobs = a.jobs.min(total);
    let per = total.div_ceil(jobs);
    let picks = a
        .contact_sheet
        .map(|cols| contact_sheet::picks(total as usize, cols))
        .unwrap_or_default();
    let wav = a.audio.then(|| layout.file("audio.wav"));
    let t0 = std::time::Instant::now();

    // The leader, for --jobs N: snapshots and the audio.
    let mut leader = None;
    let snapshot = |j: u32| work.join(format!("job{j}.ogrs"));
    if jobs > 1 {
        let mut extra = vec!["--leader".to_string(), "--end".into(), total.to_string()];
        for j in 1..jobs {
            extra.push("--snapshot".into());
            extra.push(format!("{}:{}", j * per, snapshot(j).display()));
        }
        if let Some(w) = &wav {
            extra.push("--audio".into());
            extra.push(w.display().to_string());
        }
        let mut cmd = base(&extra);
        cmd.stdout(Stdio::piped());
        leader = Some(
            cmd.spawn()
                .map_err(|e| fail(format!("{} ({e})", project::NO_BUN)))?,
        );
    }
    let plan = |j: u32| -> JobPlan {
        let first = j * per;
        let count = per.min(total - first);
        let mut extra = vec![
            "--first".to_string(),
            first.to_string(),
            "--count".into(),
            count.to_string(),
        ];
        if j > 0 {
            extra.push("--restore".into());
            extra.push(snapshot(j).display().to_string());
        }
        if jobs == 1
            && let Some(w) = &wav
        {
            extra.push("--audio".into());
            extra.push(w.display().to_string());
        }
        JobPlan {
            cmd: base(&extra),
            layout: layout.clone(),
            picks: picks.clone(),
            segment: match (&ff, &layout.video) {
                (Some((bin, _)), Some((_, c))) => Some((
                    bin.clone(),
                    work.join(format!(
                        "segment{j}.{}",
                        if *c == Container::Mp4 { "mp4" } else { "webm" }
                    )),
                    *c,
                )),
                _ => None,
            },
            size: a.size,
            fps: a.fps,
        }
    };
    // Job 0 starts at tick 0 and needs no snapshot: it runs beside the leader.
    let first_job = {
        let p0 = plan(0);
        std::thread::spawn(move || run_job(p0))
    };
    let mut leader_end = Value::Null;
    if let Some(mut l) = leader {
        let mut stdout = l.stdout.take().expect("piped");
        while let Some((head, _)) = read_record(&mut stdout).map_err(fail)? {
            if head["kind"] == "end" {
                leader_end = head;
            }
        }
        let s = l.wait().map_err(|e| fail(e.to_string()))?;
        if !s.success() || leader_end.is_null() {
            return Err(fail(format!(
                "the leader process failed ({s}); its stderr is above"
            )));
        }
    }
    let rest: Vec<_> = (1..jobs)
        .map(|j| {
            let pj = plan(j);
            std::thread::spawn(move || run_job(pj))
        })
        .collect();
    let mut outs = vec![
        first_job
            .join()
            .map_err(|_| fail("a job thread panicked".into()))?
            .map_err(fail)?,
    ];
    for h in rest {
        outs.push(
            h.join()
                .map_err(|_| fail("a job thread panicked".into()))?
                .map_err(fail)?,
        );
    }
    let elapsed = t0.elapsed().as_secs_f64();

    let end = if jobs > 1 {
        leader_end.clone()
    } else {
        outs[0].end.clone()
    };
    let mut warnings: Vec<String> = Vec::new();
    for o in &outs {
        for w in o.end["warnings"].as_array().into_iter().flatten() {
            if let Some(w) = w.as_str()
                && !warnings.iter().any(|x| x == w)
            {
                warnings.push(w.to_string());
            }
        }
    }
    let frames: Vec<FrameMeta> = outs.iter().flat_map(|o| o.frames.iter().cloned()).collect();

    // The video: segments concatenated, then the audio muxed in.
    let mut video_file = None;
    if let (Some((bin, _)), Some((path, c))) = (&ff, &layout.video) {
        let ext = if *c == Container::Mp4 { "mp4" } else { "webm" };
        let segments: Vec<PathBuf> = (0..jobs)
            .map(|j| work.join(format!("segment{j}.{ext}")))
            .collect();
        let silent = work.join(format!("video.{ext}"));
        if segments.len() == 1 {
            std::fs::rename(&segments[0], &silent).map_err(|e| fail(e.to_string()))?;
        } else {
            let list = work.join("segments.txt");
            let text: String = segments
                .iter()
                .map(|s| format!("file '{}'\n", s.display()))
                .collect();
            std::fs::write(&list, text).map_err(|e| fail(e.to_string()))?;
            ffmpeg_run(
                bin,
                &[
                    "-f",
                    "concat",
                    "-safe",
                    "0",
                    "-i",
                    &list.to_string_lossy(),
                    "-c",
                    "copy",
                    &silent.to_string_lossy(),
                ],
            )
            .map_err(fail)?;
        }
        match &wav {
            Some(w) => {
                let codec = if *c == Container::Mp4 {
                    ["-c:a", "aac", "-b:a", "192k"]
                } else {
                    ["-c:a", "libopus", "-b:a", "160k"]
                };
                let mut args = vec![
                    "-i",
                    silent.to_str().unwrap_or_default(),
                    "-i",
                    w.to_str().unwrap_or_default(),
                    "-map",
                    "0:v",
                    "-map",
                    "1:a",
                    "-c:v",
                    "copy",
                ];
                args.extend(codec);
                args.extend(["-shortest", path.to_str().unwrap_or_default()]);
                ffmpeg_run(bin, &args).map_err(fail)?;
            }
            None => {
                std::fs::copy(&silent, path).map_err(|e| fail(e.to_string()))?;
            }
        }
        video_file = Some(path.clone());
    }

    // The contact sheet.
    let mut sheet_file = None;
    if let Some(cols) = a.contact_sheet {
        let mut thumbs: Vec<_> = outs
            .iter_mut()
            .flat_map(|o| std::mem::take(&mut o.thumbs))
            .collect();
        thumbs.sort_by_key(|t| t.0);
        let cells: Vec<_> = thumbs
            .into_iter()
            .map(|(_, img, label)| (img, label))
            .collect();
        let path = layout.file("contact-sheet.png");
        contact_sheet::sheet(&cells, cols)
            .save(&path)
            .map_err(|e| fail(format!("{}: {e}", path.display())))?;
        sheet_file = Some(path);
    }

    let rel = |p: &Path| {
        p.strip_prefix(&layout.dir)
            .unwrap_or(p)
            .to_string_lossy()
            .into_owned()
    };
    let manifest = json!({
        "fps": a.fps,
        "rate": end["rate"],
        "from": a.from,
        "to": to,
        "fromTick": end["fromTick"],
        "ticksPerFrame": end["step"],
        "width": a.size.0,
        "height": a.size.1,
        "commands": script,
        "jobs": jobs,
        "tier": outs[0].end["tier"],
        "hash": end["hash"],
        "frames": frames.iter().map(|f| {
            let mut v = json!({ "index": f.index, "tick": f.tick, "time": f.time, "sha256": f.sha256 });
            if let Some(file) = &f.file { v["file"] = json!(file); }
            v
        }).collect::<Vec<_>>(),
        "audio": wav.as_ref().map(|w| json!({ "file": rel(w), "sha256": end["wavSha256"] })),
        "video": video_file.as_ref().map(|v| rel(v)),
        "contactSheet": sheet_file.as_ref().map(|s| rel(s)),
        "warnings": warnings,
    });
    let manifest_path = layout.file("frames.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap_or_default() + "\n",
    )
    .map_err(|e| fail(format!("{}: {e}", manifest_path.display())))?;
    let _ = std::fs::remove_dir_all(&work);

    if (frames.len() as u32) < total {
        warnings.push(format!("{} of {total} frames were rendered", frames.len()));
    }
    let summary = json!({
        "event": "render",
        "frames": frames.len(),
        "expected": total,
        "seconds": elapsed,
        "framesPerSecond": frames.len() as f64 / elapsed.max(1e-9),
        "framesJson": manifest_path,
        "video": video_file,
        "contactSheet": sheet_file,
        "audio": wav,
        "ffmpeg": ff.as_ref().map(|f| f.1.clone()),
        "warnings": warnings,
    });
    if a.json {
        println!("{summary}");
    } else {
        println!(
            "og render: {} frames ({}x{} at {} fps, {}s to {}s) in {:.1}s ({:.2} frames/s) on {}",
            frames.len(),
            a.size.0,
            a.size.1,
            a.fps,
            a.from,
            to,
            elapsed,
            frames.len() as f64 / elapsed.max(1e-9),
            outs[0].end["tier"].as_str().unwrap_or("?")
        );
        for (what, path) in [
            ("video", &video_file),
            ("contact sheet", &sheet_file),
            ("audio", &wav),
        ] {
            if let Some(p) = path {
                println!("  {what:<13} {}", p.display());
            }
        }
        if let Some((d, _)) = &layout.frames {
            println!("  {:<13} {}", "frames", d.display());
        }
        println!("  {:<13} {}", "frames.json", manifest_path.display());
        for w in &warnings {
            println!("  {w}");
        }
    }
    if (frames.len() as u32) < total {
        return Err((
            1,
            format!(
                "{} of {total} frames: the script quit before --to",
                frames.len()
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_takes_the_documented_options_and_refuses_a_bad_range() {
        let a = parse(&[
            "templates/game",
            "--commands",
            "0:pause",
            "--to",
            "8",
            "--out",
            "i.mp4",
            "--audio",
            "--contact-sheet",
            "6",
            "--jobs",
            "4",
            "--fps",
            "30",
        ])
        .unwrap();
        assert_eq!(
            (a.fps, a.to, a.jobs, a.contact_sheet, a.audio),
            (30, Some(8.0), 4, Some(6), true)
        );
        assert!(
            parse(&["--commands", "0:pause"])
                .unwrap_err()
                .contains("--to")
        );
        assert!(
            parse(&["--to", "2"])
                .unwrap_err()
                .contains("--commands or --timeline")
        );
        assert!(
            parse(&["--commands", "x", "--from", "3", "--to", "2"])
                .unwrap_err()
                .contains("not after")
        );
        assert!(parse(&["--commands", "x", "--to", "2", "--size", "12"]).is_err());
        // --timeline alone: the range comes from the timeline.
        assert_eq!(parse(&["--timeline", "intro"]).unwrap().to, None);
    }

    #[test]
    fn the_layout_follows_the_out_and_falls_back_to_frames_without_ffmpeg() {
        let d = Layout::of(Path::new("/r/out"), None, true);
        assert_eq!(d.video, None);
        assert_eq!(
            d.frames,
            Some((PathBuf::from("/r/out/frames"), FrameFormat::Png))
        );
        assert_eq!(d.file("frames.json"), PathBuf::from("/r/out/frames.json"));
        let v = Layout::of(Path::new("/r/intro.mp4"), None, true);
        assert_eq!(
            v.video,
            Some((PathBuf::from("/r/intro.mp4"), Container::Mp4))
        );
        assert_eq!(v.frames, None);
        assert_eq!(v.file("frames.json"), PathBuf::from("/r/intro.frames.json"));
        let none = Layout::of(Path::new("/r/intro.webm"), None, false);
        assert_eq!(none.video, None, "no ffmpeg: no video");
        assert_eq!(
            none.frames,
            Some((PathBuf::from("/r/intro-frames"), FrameFormat::Png))
        );
        let jpg = Layout::of(Path::new("/r/intro.webm"), Some(FrameFormat::Jpg), true);
        assert_eq!(jpg.video.map(|v| v.1), Some(Container::Webm));
        assert_eq!(jpg.frames.map(|f| f.1), Some(FrameFormat::Jpg));
    }

    #[test]
    fn a_timeline_s_duration_is_its_range() {
        let dir = std::env::temp_dir().join(format!("og-render-tl-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("assets/timelines")).unwrap();
        std::fs::write(
            dir.join("assets/timelines/intro.ogt"),
            "type timeline\nversion 1\n---\nduration 8\n",
        )
        .unwrap();
        assert_eq!(timeline_duration(&dir, "intro").unwrap(), 8.0);
        assert!(
            timeline_duration(&dir, "outro")
                .unwrap_err()
                .contains("no timeline `outro`")
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
