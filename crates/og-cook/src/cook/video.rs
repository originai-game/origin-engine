//! Footage: an `.ogt` of `type video` naming an `.mp4`, `.webm` or `.mov`,
//! cooked by `ffmpeg` (docs/design.md §2.9) into
//!
//! - `<id>`: an `og.Video` blob ([`format::VideoMeta`]: size, frames, fps,
//!   seconds, whether it has sound);
//! - `<id>#frames`: an `og.TextureArray` of its frames, one layer per frame
//!   (`<id>#frames#layer<n>`, each a KTX2 texture of the video's `kind`, one
//!   mip level), which `og.load.video` uploads as one 2D-array texture;
//! - `<id>#audio`: its sound track as an `og.Sound` (QOA, 48 kHz stereo),
//!   when it has one and `audio` is not `false`.
//!
//! ```text
//! type video
//! version 1
//! ---
//! source "clips/intro.mp4"   # .mp4, .webm or .mov, beside this file or under assets/
//! fps 30                     # frames per second kept (default 30)
//! maxSeconds 8               # what is kept from the start (default 8)
//! size 320 180               # each frame's size (default the source's, at most 1024 wide)
//! kind color                 # color (default) or pixel (nearest, lossless)
//! audio true                 # its sound track as <id>#audio (default true)
//! ```
//!
//! The frame count is `ceil(min(duration, maxSeconds) · fps)`, at most 256
//! (a texture array's layers): frame n is the source at `n / fps` seconds
//! (ffmpeg's `fps` filter), resized with bicubic filtering. The cook reads
//! the source's duration, size and streams with `ffprobe` when it plans and
//! decodes once per source when it cooks (`OG_FFMPEG`, `OG_FFPROBE`, else
//! PATH; `og doctor` reports them). The frames are what this ffmpeg decodes:
//! another ffmpeg version may round a pixel differently, so a reference
//! frame is recorded with the cook it was made with.

use super::CookError;
use super::texture;
use crate::format::{self, BlobKind, VideoMeta, row_hash};
use og_blob::BlobWriter;
use og_core::ogt::Document;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};

/// The `.ogt` type of a video.
pub const VIDEO_TYPE: &str = "video";

/// The most frames a video keeps: a texture array's layers.
pub const MAX_FRAMES: u32 = super::texture_array::MAX_LAYERS as u32;

/// Past this many bytes of cooked frames the cook warns.
pub const BUDGET_BYTES: u64 = 64 << 20;

/// The widest frame a video keeps unless `size` says otherwise.
const DEFAULT_MAX_WIDTH: u32 = 1024;

/// A parsed video file.
#[derive(Clone, Debug, PartialEq)]
pub struct VideoDecl {
    /// The source's path under `assets/`.
    pub source: String,
    pub fps: u32,
    pub max_seconds: f64,
    /// `None`: the source's own (capped at [`DEFAULT_MAX_WIDTH`] wide).
    pub size: Option<(u32, u32)>,
    pub kind: u32,
    pub audio: bool,
}

/// What `ffprobe` says of a source.
#[derive(Clone, Debug, PartialEq)]
pub struct Probe {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub has_audio: bool,
}

/// What a video cooks to, decided when the cook plans.
#[derive(Clone, Debug, PartialEq)]
pub struct VideoPlan {
    pub decl: VideoDecl,
    pub probe: Probe,
    pub size: (u32, u32),
    pub frames: u32,
    /// Whether `<id>#audio` is cooked.
    pub audio: bool,
}

impl VideoPlan {
    /// What the stamp covers besides the source's bytes.
    pub fn describe(&self) -> String {
        format!(
            "video 1 fps {} max {} size {}x{} frames {} kind {} audio {}",
            self.decl.fps,
            self.decl.max_seconds,
            self.size.0,
            self.size.1,
            self.frames,
            self.decl.kind,
            self.audio
        )
    }
}

/// Parses a video file; `exists(rel)` says whether a path under `assets/`
/// is a file (the source is looked for beside the file first).
pub fn parse(rel: &str, text: &str, exists: &dyn Fn(&str) -> bool) -> Result<VideoDecl, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != VIDEO_TYPE {
        return Err(format!("a video is `type {VIDEO_TYPE}`"));
    }
    if doc.version != 1 {
        return Err(format!(
            "video version {} (this engine reads 1)",
            doc.version
        ));
    }
    let dir = rel.rsplit_once('/').map_or("", |(d, _)| d);
    let mut out = VideoDecl {
        source: String::new(),
        fps: 30,
        max_seconds: 8.0,
        size: None,
        kind: format::kind::COLOR,
        audio: true,
    };
    for e in &doc.body.entries {
        let err = |x: og_core::Error| format!("line {}: {x}", e.line);
        let line = e.line;
        match e.key.as_str() {
            "source" => {
                let p = e.string(0).map_err(err)?;
                let ext = p.rsplit_once('.').map(|(_, x)| x.to_ascii_lowercase());
                if !matches!(ext.as_deref(), Some("mp4" | "webm" | "mov")) {
                    return Err(format!("line {line}: '{p}' is not an .mp4, .webm or .mov"));
                }
                if p.is_empty() || p.starts_with('/') || p.split('/').any(|c| c == "..") {
                    return Err(format!("line {line}: '{p}' is not a path under assets/"));
                }
                let sibling = if dir.is_empty() {
                    p.to_string()
                } else {
                    format!("{dir}/{p}")
                };
                out.source = if exists(&sibling) {
                    sibling
                } else if exists(p) {
                    p.to_string()
                } else {
                    return Err(format!(
                        "line {line}: source '{p}' is neither beside the video file nor under assets/"
                    ));
                };
            }
            "fps" => {
                let v = e.number(0).map_err(err)?.as_f64();
                if v.fract() != 0.0 || !(1.0..=120.0).contains(&v) {
                    return Err(format!("line {line}: fps is a whole number from 1 to 120"));
                }
                out.fps = v as u32;
            }
            "maxSeconds" => {
                let v = e.number(0).map_err(err)?.as_f64();
                if !(v > 0.0 && v.is_finite()) {
                    return Err(format!("line {line}: maxSeconds is above 0"));
                }
                out.max_seconds = v;
            }
            "size" => {
                let w = e.number(0).map_err(err)?.as_f64();
                let h = e.number(1).map_err(err)?.as_f64();
                let ok = |v: f64| v.fract() == 0.0 && (2.0..=4096.0).contains(&v);
                if !ok(w) || !ok(h) {
                    return Err(format!(
                        "line {line}: size is a width and a height, whole numbers from 2 to 4096"
                    ));
                }
                out.size = Some((w as u32, h as u32));
            }
            "kind" => {
                out.kind = match e.word(0).map_err(err)? {
                    "color" => format::kind::COLOR,
                    "pixel" => format::kind::PIXEL,
                    w => return Err(format!("line {line}: kind '{w}' (color or pixel)")),
                }
            }
            "audio" => {
                out.audio = e
                    .values
                    .first()
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| format!("line {line}: audio is true or false"))?;
            }
            k => {
                return Err(format!(
                    "line {line}: unknown key '{k}' (source, fps, maxSeconds, size, kind, audio)"
                ));
            }
        }
    }
    if out.source.is_empty() {
        return Err("a video names its file: `source \"clips/intro.mp4\"`".into());
    }
    Ok(out)
}

/// The `ffmpeg` the cook runs: `OG_FFMPEG`, else on PATH.
pub fn ffmpeg_bin() -> PathBuf {
    std::env::var_os("OG_FFMPEG")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("ffmpeg"))
}

/// The `ffprobe`: `OG_FFPROBE`, else beside `OG_FFMPEG`, else on PATH.
pub fn ffprobe_bin() -> PathBuf {
    if let Some(p) = std::env::var_os("OG_FFPROBE") {
        return PathBuf::from(p);
    }
    if let Some(f) = std::env::var_os("OG_FFMPEG") {
        let f = PathBuf::from(f);
        let name = f
            .file_name()
            .map(|n| n.to_string_lossy().replacen("ffmpeg", "ffprobe", 1))
            .unwrap_or_else(|| "ffprobe".into());
        return f.with_file_name(name);
    }
    PathBuf::from("ffprobe")
}

const NO_FFMPEG: &str = "a video is cooked by ffmpeg and ffprobe: install ffmpeg (apt install ffmpeg, brew install ffmpeg) or set OG_FFMPEG and OG_FFPROBE (og doctor says what it finds)";

/// The source's duration, size and whether it has sound.
pub fn probe(path: &Path) -> Result<Probe, String> {
    let out = Command::new(ffprobe_bin())
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .output()
        .map_err(|e| format!("{NO_FFMPEG} ({e})"))?;
    if !out.status.success() {
        return Err(format!(
            "ffprobe cannot read {}: {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("ffprobe's answer: {e}"))?;
    let streams = v["streams"].as_array().cloned().unwrap_or_default();
    let video = streams
        .iter()
        .find(|s| s["codec_type"] == "video")
        .ok_or_else(|| format!("{} has no video stream", path.display()))?;
    let num = |x: &serde_json::Value| {
        x.as_f64()
            .or_else(|| x.as_str().and_then(|s| s.parse().ok()))
    };
    let duration = num(&v["format"]["duration"])
        .or_else(|| num(&video["duration"]))
        .filter(|d| *d > 0.0)
        .ok_or_else(|| format!("{}: ffprobe gives no duration", path.display()))?;
    Ok(Probe {
        duration,
        width: video["width"].as_u64().unwrap_or(0) as u32,
        height: video["height"].as_u64().unwrap_or(0) as u32,
        has_audio: streams.iter().any(|s| s["codec_type"] == "audio"),
    })
}

/// What the video cooks to.
pub fn plan(decl: &VideoDecl, probe: Probe) -> Result<VideoPlan, String> {
    let size = match decl.size {
        Some(s) => s,
        None => {
            if probe.width == 0 || probe.height == 0 {
                return Err("ffprobe gives no frame size: name one with `size W H`".into());
            }
            let w = probe.width.min(DEFAULT_MAX_WIDTH);
            // Even, as most codecs keep them.
            let h = ((u64::from(probe.height) * u64::from(w) / u64::from(probe.width)) as u32)
                .max(2)
                & !1;
            (w & !1, h)
        }
    };
    let seconds = probe.duration.min(decl.max_seconds);
    let frames = ((seconds * f64::from(decl.fps)) - 1e-6).ceil().max(1.0) as u32;
    if frames > MAX_FRAMES {
        return Err(format!(
            "{seconds:.2} s at {} fps is {frames} frames; a video keeps at most {MAX_FRAMES} (a texture array's layers): lower fps or maxSeconds (at {} fps, maxSeconds {:.2})",
            decl.fps,
            decl.fps,
            f64::from(MAX_FRAMES) / f64::from(decl.fps)
        ));
    }
    Ok(VideoPlan {
        audio: decl.audio && probe.has_audio,
        decl: decl.clone(),
        probe,
        size,
        frames,
    })
}

/// A source decoded: every frame as RGBA8 rows.
pub struct Decoded {
    pub frames: Vec<Vec<u8>>,
}

type Slot = Arc<OnceLock<Result<Arc<Decoded>, String>>>;

/// Each source decoded once per process, by its path, its bytes' hash and
/// the plan (the frames and the layers are cooked on several threads).
fn decoded(path: &Path, hash: u64, plan: &VideoPlan) -> Result<Arc<Decoded>, CookError> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, Slot>>> = OnceLock::new();
    let key = format!("{}|{hash:016x}|{}", path.display(), plan.describe());
    let slot = {
        let mut map = CACHE
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        map.entry(key).or_default().clone()
    };
    slot.get_or_init(|| decode(path, plan).map(Arc::new))
        .clone()
        .map_err(CookError)
}

/// Runs ffmpeg for the frames: `fps` filter, bicubic scale, RGBA8; the
/// count made exactly the plan's (the last frame held, or the extra dropped).
fn decode(path: &Path, plan: &VideoPlan) -> Result<Decoded, String> {
    let (w, h) = plan.size;
    let out = Command::new(ffmpeg_bin())
        .args(["-v", "error", "-nostdin", "-i"])
        .arg(path)
        .args(["-t", &plan.decl.max_seconds.to_string(), "-an"])
        .args([
            "-vf",
            &format!("fps={},scale={w}:{h}:flags=bicubic", plan.decl.fps),
        ])
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"])
        .output()
        .map_err(|e| format!("{NO_FFMPEG} ({e})"))?;
    if !out.status.success() {
        return Err(format!(
            "ffmpeg cannot decode {}: {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let per = (w * h * 4) as usize;
    let mut frames: Vec<Vec<u8>> = out.stdout.chunks_exact(per).map(<[u8]>::to_vec).collect();
    if frames.is_empty() {
        return Err(format!("ffmpeg decoded no frames from {}", path.display()));
    }
    let want = plan.frames as usize;
    frames.truncate(want);
    while frames.len() < want {
        frames.push(frames.last().expect("one").clone());
    }
    Ok(Decoded { frames })
}

/// The id of a video's texture array.
pub fn frames_id(id: &str) -> String {
    format!("{id}#frames")
}

/// The id of a video's sound.
pub fn audio_id(id: &str) -> String {
    format!("{id}#audio")
}

/// The texture options frames are cooked with: one level (a video frame
/// is drawn near its size; mips would add a third to every frame).
fn frame_options(opts: &super::TextureOptions) -> super::TextureOptions {
    super::TextureOptions {
        mipmaps: false,
        ..opts.clone()
    }
}

/// The `og.Video` blob.
pub fn cook_meta(plan: &VideoPlan, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let meta = VideoMeta {
        width: plan.size.0,
        height: plan.size.1,
        frames: plan.frames,
        fps: plan.decl.fps,
        seconds: plan.frames as f32 / plan.decl.fps as f32,
        audio: u32::from(plan.audio),
        kind: plan.decl.kind,
    };
    let mut w = BlobWriter::new(
        format::Video::type_hash(),
        format::Video::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<VideoMeta>());
    let raw = u64::from(plan.size.0) * u64::from(plan.size.1) * 4 * u64::from(plan.frames);
    Ok((
        w.finish(),
        vec![format!(
            "video: {} frames of {}x{} at {} fps ({:.2} s of {:.2} s, {}), {:.1} MiB as RGBA8{}",
            plan.frames,
            plan.size.0,
            plan.size.1,
            plan.decl.fps,
            meta.seconds,
            plan.probe.duration,
            plan.decl.source,
            raw as f64 / f64::from(1u32 << 20),
            if plan.audio { ", with sound" } else { "" }
        )],
    ))
}

/// The `og.TextureArray` of the frames.
pub fn cook_array(id: &str, plan: &VideoPlan, opts: &super::TextureOptions, stamp: u64) -> Vec<u8> {
    super::texture_array::array_blob(
        &frames_id(id),
        plan.size,
        plan.frames as usize,
        plan.decl.kind,
        &frame_options(opts),
        stamp,
    )
}

/// Frame `n` as a texture of the video's kind.
pub fn cook_frame(
    path: &Path,
    hash: u64,
    plan: &VideoPlan,
    n: usize,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Result<Vec<u8>, CookError> {
    let d = decoded(path, hash, plan)?;
    let img = texture::Image {
        width: plan.size.0,
        height: plan.size.1,
        rgba: d.frames[n].clone(),
    };
    texture::cook(&img, plan.decl.kind, &frame_options(opts), stamp)
}

/// The sound track as 48 kHz 16-bit stereo WAV bytes, cut at maxSeconds.
pub fn audio_wav(path: &Path, plan: &VideoPlan) -> Result<Vec<u8>, CookError> {
    let tmp = std::env::temp_dir().join(format!(
        "og-video-{}-{:016x}.wav",
        std::process::id(),
        og_core::hash::hash_bytes(path.display().to_string().as_bytes())
    ));
    let seconds = f64::from(plan.frames) / f64::from(plan.decl.fps);
    let out = Command::new(ffmpeg_bin())
        .args(["-v", "error", "-nostdin", "-y", "-i"])
        .arg(path)
        .args([
            "-t",
            &seconds.to_string(),
            "-vn",
            "-ac",
            "2",
            "-ar",
            "48000",
        ])
        .args(["-c:a", "pcm_s16le", "-bitexact"])
        .arg(&tmp)
        .output()
        .map_err(|e| CookError(format!("{NO_FFMPEG} ({e})")))?;
    if !out.status.success() {
        return Err(CookError(format!(
            "ffmpeg cannot read the sound of {}: {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    let bytes = std::fs::read(&tmp).map_err(|e| CookError(format!("{}: {e}", tmp.display())));
    let _ = std::fs::remove_file(&tmp);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reads_the_settings_and_names_what_is_wrong() {
        let exists = |p: &str| p == "clips/a.mp4";
        let d = parse(
            "clips/a.ogt",
            "type video\nversion 1\n---\nsource \"a.mp4\"\nfps 24\nmaxSeconds 3\nsize 160 90\naudio false\n",
            &exists,
        )
        .unwrap();
        assert_eq!(d.source, "clips/a.mp4", "beside the file");
        assert_eq!(
            (d.fps, d.max_seconds, d.size, d.audio),
            (24, 3.0, Some((160, 90)), false)
        );
        let bad = |body: &str| {
            parse(
                "v.ogt",
                &format!("type video\nversion 1\n---\n{body}\n"),
                &exists,
            )
            .unwrap_err()
        };
        assert!(bad("source \"clips/a.gif\"").contains("not an .mp4"));
        assert!(bad("source \"clips/b.mp4\"").contains("neither beside"));
        assert!(bad("fps 2.5").contains("whole number"));
        assert!(bad("fps 30").contains("names its file"));
        assert!(bad("loop true").contains("unknown key 'loop'"));
    }

    #[test]
    fn the_plan_counts_frames_and_refuses_more_than_an_array_holds() {
        let decl = |fps, max| VideoDecl {
            source: "a.mp4".into(),
            fps,
            max_seconds: max,
            size: None,
            kind: format::kind::COLOR,
            audio: true,
        };
        let probe = Probe {
            duration: 5.0,
            width: 1920,
            height: 1080,
            has_audio: false,
        };
        let p = plan(&decl(30, 4.0), probe.clone()).unwrap();
        assert_eq!((p.frames, p.size, p.audio), (120, (1024, 576), false));
        // 5 s at 30 fps is 150 frames exactly, not 151.
        assert_eq!(plan(&decl(30, 8.0), probe.clone()).unwrap().frames, 150);
        let e = plan(&decl(60, 8.0), probe).unwrap_err();
        assert!(e.contains("300 frames") && e.contains("at most 256"), "{e}");
    }
}
