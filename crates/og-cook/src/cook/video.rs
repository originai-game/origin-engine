//! Footage: an `.ogt` of `type video` naming an `.mp4`, `.webm` or `.mov`,
//! cooked by `ffmpeg` (docs/design.md §2.9). A clip (the default, for
//! short loops) cooks to
//!
//! - `<id>`: an `og.Video` blob ([`format::VideoMeta`]: size, frames, fps,
//!   seconds, whether it has sound, `streamed` 0);
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
//! A stream (`stream true`: long footage, decoded as it plays) cooks to
//!
//! - `<id>`: the `og.Video` blob, `streamed` 1, `codec` AV1;
//! - `<id>#stream`: an `og.VideoStream` ([`format::VideoStreamMeta`]): the
//!   source encoded by ffmpeg to AV1 (8-bit 4:2:0, BT.709 limited range,
//!   one temporal unit per frame, a key frame every `keyframe` seconds),
//!   read from the IVF ffmpeg writes ([`og_video::ivf`]) into the engine's
//!   own blob, so no container is read at run time; the runtime fetches it
//!   when `og.load.video` runs, not with the startup bundle;
//! - `<id>#stream720`, beside a stream larger than 1280x720: the same at
//!   that size or less (its aspect kept), which the web's software
//!   fallback decoder plays instead (it holds 30 fps at 720p, not 1080p);
//! - `<id>#poster`: frame 0 as rav1d decodes it, a texture of the video's
//!   kind, shown until the stream's first frame and on a host with no AV1
//!   decoder;
//! - `<id>#audio`: its sound track as an Opus `og.Sound` (48 kHz).
//!
//! ```text
//! stream true                # decoded as it plays; no frame limit
//! maxSeconds 600             # default: the whole source
//! size 1280 720              # default the source's, at most 1920 wide
//! keyframe 1                 # seconds between key frames (default 1): a seek decodes at most this much
//! crf 32                     # AV1 quality, 0 (best) to 63 (default 32)
//! ```
//!
//! The encoder is ffmpeg's `libsvtav1`, else `libaom-av1` (`og doctor`
//! names the one it finds); the stamp covers which, as they write different
//! bytes.
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

/// The widest frame a clip keeps unless `size` says otherwise.
const DEFAULT_MAX_WIDTH: u32 = 1024;

/// The widest frame a stream keeps unless `size` says otherwise.
const DEFAULT_STREAM_WIDTH: u32 = 1920;

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
    /// Streamed: AV1 decoded as it plays, not a texture array.
    pub stream: bool,
    /// A stream's seconds from one key frame to the next.
    pub keyframe: f64,
    /// A stream's AV1 constant rate factor, 0..=63.
    pub crf: u32,
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
    /// A stream's AV1 encoder (`libsvtav1` or `libaom-av1`); empty for a clip.
    pub encoder: String,
}

impl VideoPlan {
    /// What the stamp covers besides the source's bytes.
    pub fn describe(&self) -> String {
        let mut d = format!(
            "video 2 fps {} max {} size {}x{} frames {} kind {} audio {}",
            self.decl.fps,
            self.decl.max_seconds,
            self.size.0,
            self.size.1,
            self.frames,
            self.decl.kind,
            self.audio
        );
        if self.decl.stream {
            d.push_str(&format!(
                " stream av1 {} keyframe {} crf {}",
                self.encoder, self.decl.keyframe, self.decl.crf
            ));
        }
        d
    }

    /// Frames from one key frame to the next.
    pub fn key_interval(&self) -> u32 {
        ((self.decl.keyframe * f64::from(self.decl.fps)).round() as u32).max(1)
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
        stream: false,
        keyframe: 1.0,
        crf: 32,
    };
    let mut max_given = false;
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
                max_given = true;
            }
            "stream" => {
                out.stream = e
                    .values
                    .first()
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| format!("line {line}: stream is true or false"))?;
            }
            "keyframe" => {
                let v = e.number(0).map_err(err)?.as_f64();
                if !(0.1..=10.0).contains(&v) {
                    return Err(format!("line {line}: keyframe is 0.1 to 10 seconds"));
                }
                out.keyframe = v;
            }
            "crf" => {
                let v = e.number(0).map_err(err)?.as_f64();
                if v.fract() != 0.0 || !(0.0..=63.0).contains(&v) {
                    return Err(format!("line {line}: crf is a whole number from 0 to 63"));
                }
                out.crf = v as u32;
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
                    "line {line}: unknown key '{k}' (source, fps, maxSeconds, size, kind, audio, stream, keyframe, crf)"
                ));
            }
        }
    }
    if out.source.is_empty() {
        return Err("a video names its file: `source \"clips/intro.mp4\"`".into());
    }
    if out.stream && !max_given {
        out.max_seconds = f64::INFINITY;
    }
    if !out.stream
        && doc
            .body
            .entries
            .iter()
            .any(|e| matches!(e.key.as_str(), "keyframe" | "crf"))
    {
        return Err("keyframe and crf are a stream's (`stream true`)".into());
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
pub fn plan(decl: &VideoDecl, probe: Probe, encoder: &str) -> Result<VideoPlan, String> {
    let widest = if decl.stream {
        DEFAULT_STREAM_WIDTH
    } else {
        DEFAULT_MAX_WIDTH
    };
    let size = match decl.size {
        Some(s) => s,
        None => {
            if probe.width == 0 || probe.height == 0 {
                return Err("ffprobe gives no frame size: name one with `size W H`".into());
            }
            let w = probe.width.min(widest);
            // Even, as most codecs keep them.
            let h = ((u64::from(probe.height) * u64::from(w) / u64::from(probe.width)) as u32)
                .max(2)
                & !1;
            (w & !1, h)
        }
    };
    let seconds = probe.duration.min(decl.max_seconds);
    let frames = ((seconds * f64::from(decl.fps)) - 1e-6).ceil().max(1.0) as u32;
    if decl.stream && (size.0 % 2 != 0 || size.1 % 2 != 0 || size.0 < 64 || size.1 < 64) {
        return Err(format!(
            "a stream's size is even (4:2:0) and at least 64x64 (SVT-AV1's least): {}x{}; a smaller video is a clip",
            size.0, size.1
        ));
    }
    if frames > MAX_FRAMES && !decl.stream {
        return Err(format!(
            "{seconds:.2} s at {} fps is {frames} frames; a clip keeps at most {MAX_FRAMES} (a texture array's layers): stream it (`stream true`), or lower fps or maxSeconds (at {} fps, maxSeconds {:.2})",
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
        encoder: if decl.stream {
            encoder.to_string()
        } else {
            String::new()
        },
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
        streamed: u32::from(plan.decl.stream),
        codec: if plan.decl.stream {
            format::video_codec::AV1
        } else {
            0
        },
    };
    let mut w = BlobWriter::new(
        format::Video::type_hash(),
        format::Video::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<VideoMeta>());
    let raw = u64::from(plan.size.0) * u64::from(plan.size.1) * 4 * u64::from(plan.frames);
    if plan.decl.stream {
        return Ok((
            w.finish(),
            vec![format!(
                "video: streamed, {} frames of {}x{} at {} fps ({:.2} s of {:.2} s, {}), AV1 by {}, a key frame every {} frames{}",
                plan.frames,
                plan.size.0,
                plan.size.1,
                plan.decl.fps,
                meta.seconds,
                plan.probe.duration,
                plan.decl.source,
                plan.encoder,
                plan.key_interval(),
                if plan.audio {
                    ", with sound (Opus)"
                } else {
                    ""
                }
            )],
        ));
    }
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

/// The id of a stream's units.
pub fn stream_id(id: &str) -> String {
    format!("{id}#stream")
}

/// The id of a stream's rendition for the web's software fallback (beside
/// a stream larger than 720p: `og_video::stream::fallback_size`).
pub fn stream_fallback_id(id: &str) -> String {
    format!("{id}#stream720")
}

/// The plan of a stream's fallback rendition: the same, at `size`.
pub fn fallback_plan(plan: &VideoPlan, size: (u32, u32)) -> VideoPlan {
    VideoPlan {
        size,
        ..plan.clone()
    }
}

/// The id of a stream's poster frame.
pub fn poster_id(id: &str) -> String {
    format!("{id}#poster")
}

/// The AV1 encoder this ffmpeg has: `libsvtav1`, else `libaom-av1`
/// (`None` without either, or without ffmpeg).
pub fn av1_encoder() -> Option<&'static str> {
    static FOUND: OnceLock<Option<&'static str>> = OnceLock::new();
    *FOUND.get_or_init(|| {
        let out = Command::new(ffmpeg_bin())
            .args(["-hide_banner", "-encoders"])
            .output()
            .ok()?;
        let list = String::from_utf8_lossy(&out.stdout);
        ["libsvtav1", "libaom-av1"]
            .into_iter()
            .find(|e| list.lines().any(|l| l.split_whitespace().nth(1) == Some(e)))
    })
}

/// What a stream needs of its encoder: refused without one.
pub fn stream_encoder() -> Result<&'static str, String> {
    av1_encoder().ok_or_else(|| {
        "a streamed video is encoded to AV1 by ffmpeg's libsvtav1 or libaom-av1, and this ffmpeg has neither (og doctor says what it finds; brew install ffmpeg, or apt install ffmpeg with libsvtav1)".to_string()
    })
}

/// A stream encoded: its IVF read into units.
pub struct Encoded {
    pub ivf: og_video::ivf::Ivf,
}

type EncodedSlot = Arc<OnceLock<Result<Arc<Encoded>, String>>>;

/// Each stream encoded once per process (its meta, units and poster are
/// cooked on several threads), by its path, hash and plan.
fn encoded(path: &Path, hash: u64, plan: &VideoPlan) -> Result<Arc<Encoded>, CookError> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, EncodedSlot>>> = OnceLock::new();
    let key = format!("{}|{hash:016x}|{}", path.display(), plan.describe());
    let slot = {
        let mut map = CACHE
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        map.entry(key).or_default().clone()
    };
    slot.get_or_init(|| encode(path, plan).map(Arc::new))
        .clone()
        .map_err(CookError)
}

/// Runs ffmpeg for the AV1: the `fps` filter, bicubic scale to BT.709
/// limited range 4:2:0, the last frame held so the count is exactly the
/// plan's, one temporal unit per frame, a key frame every key interval.
fn encode(path: &Path, plan: &VideoPlan) -> Result<Encoded, String> {
    let (w, h) = plan.size;
    let tmp = std::env::temp_dir().join(format!(
        "og-video-{}-{:016x}.ivf",
        std::process::id(),
        og_core::hash::hash_bytes(format!("{}|{}", path.display(), plan.describe()).as_bytes())
    ));
    let seconds = f64::from(plan.frames) / f64::from(plan.decl.fps);
    let key = plan.key_interval().to_string();
    let crf = plan.decl.crf.to_string();
    let mut cmd = Command::new(ffmpeg_bin());
    cmd.args(["-v", "error", "-nostdin", "-y", "-i"])
        .arg(path)
        .args(["-t", &format!("{seconds}"), "-an"])
        .args([
            "-vf",
            &format!(
                "fps={},scale={w}:{h}:flags=bicubic:out_color_matrix=bt709:out_range=tv,format=yuv420p,tpad=stop_mode=clone:stop=-1",
                plan.decl.fps
            ),
        ])
        .args(["-frames:v", &plan.frames.to_string()])
        .args(["-c:v", &plan.encoder, "-crf", &crf, "-g", &key]);
    match plan.encoder.as_str() {
        // No scene-cut key frames: one every key interval, no more.
        "libsvtav1" => cmd.args(["-preset", "8", "-svtav1-params", "scd=0"]),
        _ => cmd.args([
            "-b:v",
            "0",
            "-cpu-used",
            "6",
            "-row-mt",
            "1",
            "-keyint_min",
            &key,
        ]),
    };
    cmd.args([
        "-colorspace",
        "bt709",
        "-color_primaries",
        "bt709",
        "-color_trc",
        "bt709",
        "-color_range",
        "tv",
        "-f",
        "ivf",
    ])
    .arg(&tmp);
    let out = cmd.output().map_err(|e| format!("{NO_FFMPEG} ({e})"))?;
    let bytes = std::fs::read(&tmp);
    let _ = std::fs::remove_file(&tmp);
    if !out.status.success() {
        return Err(format!(
            "ffmpeg ({}) cannot encode {}: {}",
            plan.encoder,
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let bytes = bytes.map_err(|e| format!("{}: {e}", tmp.display()))?;
    let ivf = og_video::ivf::read(&bytes).map_err(|e| format!("{}'s AV1: {e}", path.display()))?;
    if ivf.units.len() != plan.frames as usize || (ivf.width, ivf.height) != (w, h) {
        return Err(format!(
            "ffmpeg encoded {} frames of {}x{} from {}, not the {} of {w}x{h} planned",
            ivf.units.len(),
            ivf.width,
            ivf.height,
            path.display(),
            plan.frames
        ));
    }
    Ok(Encoded { ivf })
}

/// The `og.VideoStream` blob of a stream's units.
pub fn cook_stream(
    path: &Path,
    hash: u64,
    plan: &VideoPlan,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let e = encoded(path, hash, plan)?;
    let mut data = Vec::new();
    let mut rows = Vec::with_capacity(e.ivf.units.len());
    for u in &e.ivf.units {
        rows.push(format::VideoUnitRow {
            offset: data.len() as u32,
            size: u.data.len() as u32,
            key: u32::from(u.key),
        });
        data.extend_from_slice(&u.data);
    }
    if data.len() > u32::MAX as usize {
        return Err(CookError(format!(
            "{}: {} bytes of AV1, past the 4 GiB a stream holds: raise crf or lower size",
            path.display(),
            data.len()
        )));
    }
    let keys = rows.iter().filter(|r| r.key != 0).count();
    let longest = {
        let mut gap = 0u32;
        let mut last = 0u32;
        for (i, r) in rows.iter().enumerate().skip(1) {
            if r.key != 0 {
                gap = gap.max(i as u32 - last);
                last = i as u32;
            }
        }
        gap.max(rows.len() as u32 - last)
    };
    let meta = format::VideoStreamMeta {
        codec: format::video_codec::AV1,
        width: plan.size.0,
        height: plan.size.1,
        frames: rows.len() as u32,
        fps: plan.decl.fps,
        key_interval: longest,
    };
    let mut w = BlobWriter::new(
        format::VideoStream::type_hash(),
        format::VideoStream::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<format::VideoStreamMeta>());
    w.table("units", &rows, row_hash::<format::VideoUnitRow>());
    w.raw("config", e.ivf.sequence_header.clone(), 4);
    let kbps =
        data.len() as f64 * 8.0 / 1000.0 / (f64::from(plan.frames) / f64::from(plan.decl.fps));
    let note = format!(
        "the video's AV1: {:.1} MiB ({kbps:.0} kb/s), {keys} key frame(s)",
        data.len() as f64 / f64::from(1u32 << 20)
    );
    w.raw("data", data, 16);
    Ok((w.finish(), vec![note]))
}

/// The poster: frame 0 as rav1d decodes it, a texture of the video's kind.
pub fn cook_poster(
    path: &Path,
    hash: u64,
    plan: &VideoPlan,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Result<Vec<u8>, CookError> {
    let e = encoded(path, hash, plan)?;
    let mut d = og_video::decode::Decoder::new(1).map_err(CookError)?;
    let pic = d
        .decode(&e.ivf.units[0].data)
        .map_err(|x| CookError(format!("{}: frame 0: {x}", path.display())))?
        .ok_or_else(|| CookError(format!("{}: frame 0 shows no picture", path.display())))?;
    let img = texture::Image {
        width: pic.width,
        height: pic.height,
        rgba: pic.rgba.to_vec(),
    };
    texture::cook(&img, plan.decl.kind, &frame_options(opts), stamp)
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
            stream: false,
            keyframe: 1.0,
            crf: 32,
        };
        let probe = Probe {
            duration: 5.0,
            width: 1920,
            height: 1080,
            has_audio: false,
        };
        let p = plan(&decl(30, 4.0), probe.clone(), "").unwrap();
        assert_eq!((p.frames, p.size, p.audio), (120, (1024, 576), false));
        // 5 s at 30 fps is 150 frames exactly, not 151.
        assert_eq!(plan(&decl(30, 8.0), probe.clone(), "").unwrap().frames, 150);
        let e = plan(&decl(60, 8.0), probe.clone(), "").unwrap_err();
        assert!(e.contains("300 frames") && e.contains("at most 256"), "{e}");
        assert!(e.contains("stream true"), "{e}");
        // A stream keeps every frame, up to 1920 wide by default.
        let stream = VideoDecl {
            stream: true,
            max_seconds: f64::INFINITY,
            ..decl(60, 8.0)
        };
        let p = plan(&stream, probe, "libsvtav1").unwrap();
        assert_eq!((p.frames, p.size), (300, (1920, 1080)));
        assert_eq!(p.key_interval(), 60);
        assert!(
            p.describe()
                .contains("stream av1 libsvtav1 keyframe 1 crf 32")
        );
    }

    #[test]
    fn a_stream_reads_its_settings_and_a_clip_refuses_them() {
        let exists = |p: &str| p == "a.mp4";
        let doc = |body: &str| format!("type video\nversion 1\n---\nsource \"a.mp4\"\n{body}\n");
        let d = parse("a.ogt", &doc("stream true\nkeyframe 2\ncrf 40"), &exists).unwrap();
        assert!(d.stream && d.max_seconds.is_infinite());
        assert_eq!((d.keyframe, d.crf), (2.0, 40));
        let d = parse("a.ogt", &doc("stream true\nmaxSeconds 30"), &exists).unwrap();
        assert_eq!(d.max_seconds, 30.0);
        assert!(
            parse("a.ogt", &doc("crf 40"), &exists)
                .unwrap_err()
                .contains("stream true")
        );
        assert!(
            parse("a.ogt", &doc("stream true\ncrf 64"), &exists)
                .unwrap_err()
                .contains("0 to 63")
        );
    }
}
