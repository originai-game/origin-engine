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
//! loudness -16               # normalise the sound to this LUFS (EBU R128); off by default
//! aiGenerated true           # the footage was made by a generative model (VideoInfo.aiGenerated)
//! placeholder true           # stand-in footage to replace (og flow list reports it)
//! alpha true                 # keep the source's alpha (ProRes 4444, QuickTime Animation, PNG, VP8/VP9 WebM)
//! ```
//!
//! `alpha true` keeps the source's alpha channel (refused for a source
//! without one); without it every frame is opaque. A clip's layers hold
//! it. A stream's codecs carry no alpha, so each unit is a picture twice
//! the frame's height (packed alpha): the colour above, the alpha below as
//! luma (limited range, neutral chroma), its 4:2:0 chroma rows split
//! cleanly at the even height. rav1d folds the lower half into the frame's
//! alpha natively and for the poster (og-video's `Decoder::packed_alpha`);
//! on the web og-render does it on the GPU after the browser's copy.
//!
//! A stream (`stream true`: long footage, decoded as it plays) cooks to
//!
//! - `<id>`: the `og.Video` blob, `streamed` 1, `codec` AV1;
//! - `<id>#stream`: an `og.VideoStream` ([`format::VideoStreamMeta`]): the
//!   index of the source encoded by ffmpeg to AV1 (8-bit 4:2:0, BT.709
//!   limited range, one temporal unit per frame, a key frame every
//!   `keyframe` seconds), read from the IVF ffmpeg writes
//!   ([`og_video::ivf`]) into the engine's own blobs, so no container is
//!   read at run time: each frame's place in its chunk and the codec's
//!   configuration. Its units are in its chunks `<id>#stream.<n>`
//!   (`og.VideoChunk`, [`format::VideoChunkMeta`]), each `chunk` seconds
//!   from a key frame on. Neither is in the startup bundle: the runtime
//!   fetches the index when a player first wants a frame, and each chunk
//!   ahead of the players that need it;
//! - `<id>#stream720`, beside a stream larger than 1280x720 (720x1280 portrait), with its chunks: the same at
//!   that size or less (its aspect kept), which the web's software
//!   fallback decoder plays instead (it holds 30 fps at 720p, not 1080p);
//! - `<id>#streamhevc` and `<id>#streamh264` with their chunks, as
//!   `renditions` names them (HEVC by default): the same frames and chunks in HEVC (ffmpeg's `libx265`) or
//!   H.264 (`libx264`), with no B-frames, read from the MP4 ffmpeg writes
//!   ([`og_video::mp4`]), for a browser whose WebCodecs decodes no AV1 but
//!   decodes these (Safari on devices without AV1 hardware). Only the web
//!   host reads them; an executor always decodes the AV1;
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
//! chunk 6                    # seconds per chunk (default 6, whole key frame intervals): what a player fetches at a time
//! crf 32                     # AV1 quality, 0 (best) to 63 (default 32)
//! crf720 38                  # the 720p rendition's (default crf + 6, at most 63)
//! renditions hevc h264       # beside the AV1: hevc, h264, both, or none (default hevc)
//! loudness -16               # normalise the sound to this LUFS (EBU R128); off by default
//! aiGenerated true           # the footage was made by a generative model (VideoInfo.aiGenerated)
//! ```
//!
//! `loudness` runs ffmpeg's `loudnorm` on the sound so AI-generated clips,
//! whose levels vary, all play at one target (−16 LUFS for streaming, −14
//! louder). `aiGenerated` sets VideoInfo.aiGenerated, which a game or store
//! reads to label the footage.
//!
//! The encoder is ffmpeg's `libsvtav1`, else `libaom-av1` (`og doctor`
//! names the one it finds); the stamp covers which, as they write different
//! bytes. A rendition's quality follows `crf` ([`Rendition::crf`]: x264's
//! 23 and x265's 26 at AV1's 32). An ffmpeg without `libx265` cooks no
//! HEVC rendition by default and says so; one a file names is refused.
//!
//! The frame count is `ceil(min(duration, maxSeconds) · fps)`, at most 256
//! (a texture array's layers): frame n is the source at `n / fps` seconds
//! (ffmpeg's `fps` filter), resized with bicubic filtering. The cook reads
//! the source's duration, size and streams with `ffprobe` when it plans and
//! decodes once per source when it cooks (`OG_FFMPEG`, `OG_FFPROBE`, else
//! PATH; `og doctor` reports them). The frames are what this ffmpeg decodes:
//! another ffmpeg version may round a pixel differently, so a reference
//! frame is recorded with the cook it was made with.

use super::texture;
use super::{
    Action, CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, audio, hash_bytes,
    stamp, texture_array,
};
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
    /// A stream's seconds per chunk (`chunk`, rounded to whole key frame
    /// intervals): what a player fetches at a time.
    pub chunk: f64,
    /// A stream's AV1 constant rate factor, 0..=63.
    pub crf: u32,
    /// Its 720p rendition's (`crf720`); `None`: [`RENDITION_CRF`] more than
    /// `crf`, at most 63.
    pub crf720: Option<u32>,
    /// The renditions beside a stream's AV1 (`renditions`); `None`: HEVC
    /// where this ffmpeg encodes it.
    pub renditions: Option<Vec<Rendition>>,
    /// Integrated loudness target in LUFS (`loudness`), applied to the
    /// sound with ffmpeg's `loudnorm` (EBU R128) so every clip plays at
    /// the same level; `None`: the source's own. −16 to 0, typically −16
    /// (streaming) or −14.
    pub loudness: Option<f64>,
    /// `aiGenerated true`: the footage was made by a generative model.
    /// VideoInfo.aiGenerated carries it, so a game or store can label it.
    pub ai_generated: bool,
    /// `placeholder true`: stand-in footage to replace (the skeleton's
    /// clip). Cooked as any other; `og flow list` reports it.
    pub placeholder: bool,
    /// `alpha true`: the frames keep the source's alpha (a stream's units
    /// pack it below the colour); otherwise they are opaque.
    pub alpha: bool,
    /// Maximum longest side in pixels for a stream's `#poster` thumbnail
    /// (default 480; 0 keeps full resolution).
    pub poster_size: u32,
    /// Alternate localized audio tracks (`audioLocale "<tag>" "<path>"`).
    pub audio_locales: Vec<(String, String)>,
}

/// A stream's copy in a codec other than AV1, for browsers whose WebCodecs
/// decodes no AV1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rendition {
    Hevc,
    H264,
}

impl Rendition {
    /// Its word in `renditions`, and its output's suffix (`<id>#stream<word>`).
    pub fn word(self) -> &'static str {
        match self {
            Rendition::Hevc => "hevc",
            Rendition::H264 => "h264",
        }
    }

    /// The ffmpeg encoder that writes it.
    pub fn encoder(self) -> &'static str {
        match self {
            Rendition::Hevc => "libx265",
            Rendition::H264 => "libx264",
        }
    }

    /// Its [`format::video_codec`].
    pub fn codec(self) -> u32 {
        match self {
            Rendition::Hevc => format::video_codec::HEVC,
            Rendition::H264 => format::video_codec::H264,
        }
    }

    /// Its encoder's crf for a stream of AV1 `crf`: linear, x264's 23 and
    /// x265's 26 (their defaults' look) at AV1's default 32, at most 51.
    pub fn crf(self, crf: u32) -> u32 {
        let at32 = match self {
            Rendition::Hevc => 26,
            Rendition::H264 => 23,
        };
        ((crf * at32 + 16) / 32).min(51)
    }
}

/// How much coarser the 720p rendition is encoded than its stream by
/// default: it is the web fallback's copy, fetched only where the browser
/// cannot decode AV1, so it is kept small (a 1080p stream's rendition at
/// the same crf was 70% of the stream's bytes).
pub const RENDITION_CRF: u32 = 6;

/// Default maximum longest side for a stream's `#poster` texture.
pub const DEFAULT_POSTER_SIZE: u32 = 480;

impl VideoDecl {
    /// The 720p rendition's crf.
    pub fn rendition_crf(&self) -> u32 {
        self.crf720
            .unwrap_or_else(|| (self.crf + RENDITION_CRF).min(63))
    }

    /// The 480p low-quality ABR rendition's crf.
    pub fn low_crf(&self) -> u32 {
        (self.rendition_crf() + 4).min(63)
    }
}

/// What `ffprobe` says of a source.
#[derive(Clone, Debug, PartialEq)]
pub struct Probe {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub has_audio: bool,
    /// The video stream's codec (`ffprobe`'s `codec_name`).
    pub codec: String,
    /// Whether its pixels carry alpha: an alpha pixel format, or a WebM's
    /// `alpha_mode` tag (VP8 and VP9 keep alpha beside the picture).
    pub has_alpha: bool,
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
    /// A rendition's plan: its encoder (`libx265` or `libx264`).
    pub encoder: String,
    /// A stream's renditions this ffmpeg encodes (set when the cook plans).
    pub renditions: Vec<Rendition>,
    /// The default renditions this ffmpeg cannot encode (said, not refused).
    pub missing: Vec<Rendition>,
    /// Set in a rendition's own plan ([`rendition_plan`]).
    pub rendition: Option<Rendition>,
}

impl VideoPlan {
    /// What the stamp covers besides the source's bytes.
    pub fn describe(&self) -> String {
        let mut d = format!(
            "video 3 fps {} max {} size {}x{} frames {} kind {} audio {}",
            self.decl.fps,
            self.decl.max_seconds,
            self.size.0,
            self.size.1,
            self.frames,
            self.decl.kind,
            self.audio
        );
        if let Some(r) = self.rendition {
            d.push_str(&format!(
                " stream {} {} keyframe {} crf {}",
                r.word(),
                self.encoder,
                self.decl.keyframe,
                r.crf(self.decl.crf)
            ));
        } else if self.decl.stream {
            d.push_str(&format!(
                " stream av1 {} keyframe {} crf {} crf720 {} poster {}",
                self.encoder,
                self.decl.keyframe,
                self.decl.crf,
                self.decl.rendition_crf(),
                self.decl.poster_size
            ));
        }
        if self.decl.stream {
            d.push_str(&format!(" chunk {}", self.chunk_frames()));
        }
        if let Some(i) = self.decl.loudness {
            d.push_str(&format!(" loudness {i}"));
        }
        if self.decl.ai_generated {
            d.push_str(" ai");
        }
        if self.decl.alpha {
            d.push_str(" alpha");
        }
        for (tag, path) in &self.decl.audio_locales {
            d.push_str(&format!(" audioLocale {tag}={path}"));
        }
        d
    }

    /// Frames from one key frame to the next.
    pub fn key_interval(&self) -> u32 {
        ((self.decl.keyframe * f64::from(self.decl.fps)).round() as u32).max(1)
    }

    /// Frames in each of a stream's chunks but the last: whole key
    /// intervals, about `chunk` seconds of footage.
    pub fn chunk_frames(&self) -> u32 {
        let k = self.key_interval();
        let per = (self.decl.chunk * f64::from(self.decl.fps) / f64::from(k)).round() as u32;
        k * per.max(1)
    }

    /// A stream's chunks (`<stream id>.<n>`).
    pub fn chunks(&self) -> u32 {
        self.frames.div_ceil(self.chunk_frames())
    }
}

/// Seconds of footage in a stream's chunk by default (`chunk`): what a
/// player fetches at a time, ahead of where it plays (og-video's
/// `stream::LEAD`).
pub const CHUNK_SECONDS: f64 = 6.0;

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
        chunk: CHUNK_SECONDS,
        crf: 32,
        crf720: None,
        renditions: None,
        loudness: None,
        ai_generated: false,
        placeholder: false,
        alpha: false,
        poster_size: DEFAULT_POSTER_SIZE,
        audio_locales: Vec::new(),
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
            "chunk" => {
                let v = e.number(0).map_err(err)?.as_f64();
                if !(0.5..=60.0).contains(&v) {
                    return Err(format!("line {line}: chunk is 0.5 to 60 seconds"));
                }
                out.chunk = v;
            }
            "crf" => {
                let v = e.number(0).map_err(err)?.as_f64();
                if v.fract() != 0.0 || !(0.0..=63.0).contains(&v) {
                    return Err(format!("line {line}: crf is a whole number from 0 to 63"));
                }
                out.crf = v as u32;
            }
            "crf720" => {
                let v = e.number(0).map_err(err)?.as_f64();
                if v.fract() != 0.0 || !(0.0..=63.0).contains(&v) {
                    return Err(format!(
                        "line {line}: crf720 is a whole number from 0 to 63"
                    ));
                }
                out.crf720 = Some(v as u32);
            }
            "posterSize" => {
                let v = e.number(0).map_err(err)?.as_f64();
                if v.fract() != 0.0 || (v != 0.0 && !(64.0..=4096.0).contains(&v)) {
                    return Err(format!(
                        "line {line}: posterSize is 0 (full size) or 64 to 4096 pixels"
                    ));
                }
                out.poster_size = v as u32;
            }
            "audioLocale" => {
                let tag = e.string(0).map_err(err)?.to_string();
                let p = e.string(1).map_err(err)?;
                if tag.is_empty()
                    || !tag
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    return Err(format!("line {line}: audioLocale tag '{tag}' is invalid"));
                }
                if out
                    .audio_locales
                    .iter()
                    .any(|(t, _)| t.eq_ignore_ascii_case(&tag))
                {
                    return Err(format!(
                        "line {line}: audioLocale '{tag}' is declared twice"
                    ));
                }
                if p.is_empty() || p.starts_with('/') || p.split('/').any(|c| c == "..") {
                    return Err(format!("line {line}: '{p}' is not a path under assets/"));
                }
                let sibling = if dir.is_empty() {
                    p.to_string()
                } else {
                    format!("{dir}/{p}")
                };
                let resolved = if exists(&sibling) {
                    sibling
                } else if exists(p) {
                    p.to_string()
                } else {
                    return Err(format!(
                        "line {line}: audioLocale '{tag}' file '{p}' is neither beside the video file nor under assets/"
                    ));
                };
                out.audio_locales.push((tag, resolved));
            }
            "renditions" => {
                let mut list = Vec::new();
                for i in 0..e.values.len() {
                    match e.word(i).map_err(err)? {
                        "hevc" => list.push(Rendition::Hevc),
                        "h264" => list.push(Rendition::H264),
                        "none" if e.values.len() == 1 => {}
                        w => {
                            return Err(format!(
                                "line {line}: renditions takes hevc, h264, both, or none alone, not '{w}'"
                            ));
                        }
                    }
                }
                if e.values.is_empty() {
                    return Err(format!(
                        "line {line}: renditions takes hevc, h264, both, or none"
                    ));
                }
                list.dedup();
                out.renditions = Some(list);
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
            "loudness" => {
                let v = e.number(0).map_err(err)?.as_f64();
                if !(-30.0..=0.0).contains(&v) {
                    return Err(format!(
                        "line {line}: loudness is a LUFS target from -30 to 0 (-23 broadcast, -16 streaming, -14 louder)"
                    ));
                }
                out.loudness = Some(v);
            }
            "aiGenerated" => {
                out.ai_generated = e
                    .values
                    .first()
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| format!("line {line}: aiGenerated is true or false"))?;
            }
            "placeholder" => {
                out.placeholder = e
                    .values
                    .first()
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| format!("line {line}: placeholder is true or false"))?;
            }
            "alpha" => {
                out.alpha = e
                    .values
                    .first()
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| format!("line {line}: alpha is true or false"))?;
            }
            k => {
                return Err(format!(
                    "line {line}: unknown key '{k}' (source, fps, maxSeconds, size, kind, audio, audioLocale, aiGenerated, placeholder, alpha, loudness, stream, keyframe, chunk, crf, crf720, posterSize, renditions)"
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
        && doc.body.entries.iter().any(|e| {
            matches!(
                e.key.as_str(),
                "keyframe" | "chunk" | "crf" | "crf720" | "posterSize" | "renditions"
            )
        })
    {
        return Err(
            "keyframe, chunk, crf, crf720, posterSize and renditions are a stream's (`stream true`)"
                .into(),
        );
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
    let pix = video["pix_fmt"].as_str().unwrap_or_default();
    let has_alpha = ["yuva", "gbrap", "ya8", "ya16", "pal8"]
        .iter()
        .any(|p| pix.starts_with(p))
        || ["rgba", "bgra", "argb", "abgr"]
            .iter()
            .any(|p| pix.contains(p))
        || video["tags"]["alpha_mode"].as_str() == Some("1")
        || video["tags"]["ALPHA_MODE"].as_str() == Some("1");
    Ok(Probe {
        duration,
        width: video["width"].as_u64().unwrap_or(0) as u32,
        height: video["height"].as_u64().unwrap_or(0) as u32,
        has_audio: streams.iter().any(|s| s["codec_type"] == "audio"),
        codec: video["codec_name"].as_str().unwrap_or_default().to_string(),
        has_alpha,
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
    if decl.alpha && !probe.has_alpha {
        return Err(format!(
            "`alpha true`, but {} carries no alpha ({}): export it with alpha (ProRes 4444, QuickTime Animation, PNG in .mov, or VP9 WebM with alpha), or leave alpha out",
            decl.source,
            if probe.codec.is_empty() {
                "its codec is unknown"
            } else {
                probe.codec.as_str()
            }
        ));
    }
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
        renditions: Vec::new(),
        missing: Vec::new(),
        rendition: None,
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

/// ffmpeg reading `path`: libvpx for a VP8 or VP9 source whose alpha is
/// kept (ffmpeg's own decoders drop the alpha WebM stores beside the
/// picture).
fn reading(path: &Path, plan: &VideoPlan) -> Command {
    let mut cmd = Command::new(ffmpeg_bin());
    cmd.args(["-v", "error", "-nostdin", "-y"]);
    if plan.decl.alpha {
        match plan.probe.codec.as_str() {
            "vp9" => {
                cmd.args(["-c:v", "libvpx-vp9"]);
            }
            "vp8" => {
                cmd.args(["-c:v", "libvpx"]);
            }
            _ => {}
        }
    }
    cmd.arg("-i").arg(path);
    cmd
}

/// The rows' height a stream's units hold: twice the frame's with packed
/// alpha (the alpha below the colour).
pub fn coded_height(plan: &VideoPlan) -> u32 {
    if plan.decl.alpha {
        plan.size.1 * 2
    } else {
        plan.size.1
    }
}

/// A keyed frame with its transparent texels' colour taken from their
/// visible neighbourhood (texture.rs `dilate_transparent`). A source keeps
/// whatever colour it likes under alpha 0 (the green of a screen, or
/// black), and that colour reaches the visible edge through 4:2:0 chroma
/// and the texture's bilinear sampling as a fringe; the matte is untouched.
fn keyed(width: u32, height: u32, rgba: Vec<u8>) -> Vec<u8> {
    let img = texture::Image {
        width,
        height,
        rgba,
    };
    match texture::dilate_transparent(&img) {
        Some(d) => d.rgba,
        None => img.rgba,
    }
}

/// Encodes a keyed stream: one ffmpeg reads the source's frames as RGBA,
/// each is filled under its matte ([`keyed`]) and piped, frame by frame,
/// into `encoder` (raw RGBA on its stdin), so no stream is held whole.
fn encode_keyed(
    path: &Path,
    plan: &VideoPlan,
    seconds: f64,
    encoder: &mut Command,
) -> Result<std::process::Output, String> {
    use std::io::{Read, Write};
    use std::process::Stdio;
    let (w, h) = plan.size;
    let mut reader = reading(path, plan)
        .args(["-t", &format!("{seconds}"), "-an"])
        .args([
            "-vf",
            &format!("fps={},scale={w}:{h}:flags=bicubic", plan.decl.fps),
        ])
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{NO_FFMPEG} ({e})"))?;
    let mut enc = encoder
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{NO_FFMPEG} ({e})"))?;
    let mut src = reader.stdout.take().expect("piped");
    let mut reader_err = reader.stderr.take().expect("piped");
    let err_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = reader_err.read_to_end(&mut buf);
        buf
    });
    let mut sink = enc.stdin.take().expect("piped");
    let per = (w * h * 4) as usize;
    let mut failed = None;
    loop {
        let mut frame = vec![0u8; per];
        match src.read_exact(&mut frame) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                failed = Some(format!("reading {}: {e}", path.display()));
                break;
            }
        }
        // The encoder stops reading once it has its frames.
        if sink.write_all(&keyed(w, h, frame)).is_err() {
            break;
        }
    }
    drop(sink);
    drop(src);
    let rstatus = reader.wait().ok();
    let rstderr = err_thread.join().unwrap_or_default();
    let mut out = enc
        .wait_with_output()
        .map_err(|e| format!("{NO_FFMPEG} ({e})"))?;
    if !rstderr.is_empty() && (!out.status.success() || rstatus.is_some_and(|s| !s.success())) {
        if !out.stderr.is_empty() {
            out.stderr.extend_from_slice(b"\nreader: ");
        }
        out.stderr.extend_from_slice(&rstderr);
    }
    match failed {
        Some(e) => Err(e),
        None => Ok(out),
    }
}

/// Runs ffmpeg for the frames: `fps` filter, bicubic scale, RGBA8 (opaque
/// unless `alpha true`); the count made exactly the plan's (the last frame
/// held, or the extra dropped).
fn decode(path: &Path, plan: &VideoPlan) -> Result<Decoded, String> {
    let (w, h) = plan.size;
    let out = reading(path, plan)
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
    for f in &mut frames {
        if plan.decl.alpha {
            *f = keyed(w, h, std::mem::take(f));
        } else {
            for a in f.iter_mut().skip(3).step_by(4) {
                *a = 255;
            }
        }
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

/// The id of a video's alternate localized sound (`audioLocale`).
pub fn audio_locale_id(id: &str, locale: &str) -> String {
    format!("{id}#audio.{locale}")
}

/// Opus blocks per audio chunk for streamed videos (4 blocks = 1.92 s, ~22 KB).
pub const STREAM_AUDIO_CHUNK_BLOCKS: u32 = 4;

/// Exact 48 kHz sample frame count matching a video plan's picture duration.
pub fn expected_audio_frames(plan: &VideoPlan) -> u64 {
    (f64::from(plan.frames) * f64::from(crate::opus::RATE) / f64::from(plan.decl.fps))
        .round()
        .max(1.0) as u64
}

/// Chunking `(blocks_per_chunk, chunk_count)` for a streamed video's Opus
/// audio (`None` for clips or very short streams that fit in 1 chunk).
pub fn stream_audio_chunking(plan: &VideoPlan) -> Option<(u32, u32)> {
    if !plan.decl.stream {
        return None;
    }
    let blocks = crate::opus::block_count(expected_audio_frames(plan)) as u64;
    let per = u64::from(STREAM_AUDIO_CHUNK_BLOCKS);
    let count = blocks.div_ceil(per).max(1) as u32;
    (count > 1).then_some((STREAM_AUDIO_CHUNK_BLOCKS, count))
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
        ai_generated: u32::from(plan.decl.ai_generated),
        alpha: u32::from(plan.decl.alpha),
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
                "video: streamed, {} frames of {}x{} at {} fps ({:.2} s of {:.2} s, {}), AV1 by {}{}, a key frame every {} frames{}{}",
                plan.frames,
                plan.size.0,
                plan.size.1,
                plan.decl.fps,
                meta.seconds,
                plan.probe.duration,
                plan.decl.source,
                plan.encoder,
                plan.renditions
                    .iter()
                    .map(|r| format!(", {} by {}", codec_name(r.codec()), r.encoder()))
                    .collect::<String>(),
                plan.key_interval(),
                if plan.audio {
                    ", with sound (Opus)"
                } else {
                    ""
                },
                plan.missing
                    .iter()
                    .map(|r| format!("; no {} rendition: this ffmpeg has no {}, so a browser without AV1 plays the slower software decoder", codec_name(r.codec()), r.encoder()))
                    .collect::<String>()
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

/// Builds the ffmpeg audio filter chain: two-pass linear EBU R128 `loudnorm`
/// when `loudness` is set (measuring integrated loudness, true peak, LRA and
/// threshold in pass 1 before applying exact linear gain in pass 2), followed
/// by `apad`. Falls back cleanly to `apad` on completely silent tracks.
fn loudnorm_filter(path: &Path, seconds: f64, loudness: Option<f64>) -> Result<String, CookError> {
    let Some(target) = loudness else {
        return Ok("apad".into());
    };
    let pass1 = Command::new(ffmpeg_bin())
        .args(["-v", "info", "-nostdin", "-i"])
        .arg(path)
        .args([
            "-t",
            &seconds.to_string(),
            "-vn",
            "-af",
            &format!("loudnorm=I={target}:TP=-1.5:LRA=11:print_format=json"),
            "-f",
            "null",
            "-",
        ])
        .output()
        .map_err(|e| CookError(format!("{NO_FFMPEG} ({e})")))?;
    if !pass1.status.success() {
        return Err(CookError(format!(
            "ffmpeg loudness analysis failed on {}: {}",
            path.display(),
            String::from_utf8_lossy(&pass1.stderr).trim()
        )));
    }
    let stderr = String::from_utf8_lossy(&pass1.stderr);
    let parsed = stderr.rfind('{').and_then(|start| {
        let end = stderr[start..].find('}')?;
        serde_json::from_str::<serde_json::Value>(&stderr[start..=start + end]).ok()
    });
    let num = |v: &serde_json::Value, key: &str| -> Option<f64> {
        v[key]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| v[key].as_f64())
            .filter(|x| x.is_finite())
    };
    if let Some(v) = parsed
        && let (Some(ii), Some(tp), Some(lra), Some(thresh), Some(off)) = (
            num(&v, "input_i"),
            num(&v, "input_tp"),
            num(&v, "input_lra"),
            num(&v, "input_thresh"),
            num(&v, "target_offset"),
        )
    {
        return Ok(format!(
            "loudnorm=I={target}:TP=-1.5:LRA=11:measured_I={ii}:measured_TP={tp}:measured_LRA={lra}:measured_thresh={thresh}:offset={off}:linear=true,apad"
        ));
    }
    Ok("apad".into())
}

/// The sound track as 48 kHz 16-bit stereo WAV bytes (used for short clips),
/// cut at maxSeconds, padded with silence where it ends first.
pub fn audio_wav(path: &Path, plan: &VideoPlan) -> Result<Vec<u8>, CookError> {
    static CALLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let tmp = std::env::temp_dir().join(format!(
        "og-video-{}-{:016x}-{}.wav",
        std::process::id(),
        og_core::hash::hash_bytes(path.display().to_string().as_bytes()),
        CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let seconds = f64::from(plan.frames) / f64::from(plan.decl.fps);
    let filter = loudnorm_filter(path, seconds, plan.decl.loudness)?;
    let out = Command::new(ffmpeg_bin())
        .args(["-v", "error", "-nostdin", "-y", "-i"])
        .arg(path)
        .args(["-t", &seconds.to_string(), "-vn", "-af", &filter])
        .args(["-ac", "2", "-ar", "48000"])
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

type AudioSlot = Arc<OnceLock<Result<Arc<audio::Encoded>, String>>>;

/// Encodes a streamed video's sound track once per process by piping raw 48 kHz
/// stereo `s16le` PCM directly from ffmpeg into `og_bundle::opus::StreamEncoder`
/// with bounded memory (zero temp WAV, exact `expected_audio_frames(plan)` length).
fn stream_audio_encoded(
    path: &Path,
    hash: u64,
    plan: &VideoPlan,
    chunking: Option<(u32, u32)>,
) -> Result<Arc<audio::Encoded>, CookError> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, AudioSlot>>> = OnceLock::new();
    let key = format!(
        "{}|{hash:016x}|{}|{:?}|{:?}",
        path.display(),
        expected_audio_frames(plan),
        plan.decl.loudness,
        chunking
    );
    let slot = {
        let mut map = CACHE
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        map.entry(key).or_default().clone()
    };
    slot.get_or_init(|| encode_stream_audio_pipe(path, plan, chunking).map(Arc::new))
        .clone()
        .map_err(CookError)
}

fn encode_stream_audio_pipe(
    path: &Path,
    plan: &VideoPlan,
    chunking: Option<(u32, u32)>,
) -> Result<audio::Encoded, String> {
    use std::io::Read;
    use std::process::Stdio;
    let want_frames = expected_audio_frames(plan);
    let seconds = f64::from(plan.frames) / f64::from(plan.decl.fps);
    let filter = loudnorm_filter(path, seconds, plan.decl.loudness).map_err(|e| e.0)?;
    let mut child = Command::new(ffmpeg_bin())
        .args(["-v", "error", "-nostdin", "-i"])
        .arg(path)
        .args(["-t", &seconds.to_string(), "-vn", "-af", &filter])
        .args([
            "-ac",
            "2",
            "-ar",
            "48000",
            "-f",
            "s16le",
            "-c:a",
            "pcm_s16le",
            "-bitexact",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{NO_FFMPEG} ({e})"))?;
    let mut stdout = child.stdout.take().expect("piped");
    let mut stderr = child.stderr.take().expect("piped");
    let err_thread = std::thread::spawn(move || {
        let mut b = Vec::new();
        let _ = stderr.read_to_end(&mut b);
        b
    });
    let bitrate = audio::SoundSettings::default().opus_bitrate(2);
    let mut enc = crate::opus::StreamEncoder::new(2, bitrate)?;
    let mut buf = vec![0u8; 65_536];
    let mut carry = Vec::new();
    let mut pushed = 0u64;
    let mut samples = Vec::new();
    loop {
        let n = stdout
            .read(&mut buf)
            .map_err(|e| format!("reading ffmpeg audio pipe for {}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        carry.extend_from_slice(&buf[..n]);
        let whole_frames = (carry.len() / 4) as u64;
        if whole_frames == 0 {
            continue;
        }
        let take_frames = whole_frames.min(want_frames.saturating_sub(pushed));
        if take_frames > 0 {
            let take_bytes = (take_frames * 4) as usize;
            samples.clear();
            samples.reserve((take_frames * 2) as usize);
            for &ch in carry[..take_bytes].as_chunks::<2>().0 {
                samples.push(i16::from_le_bytes(ch));
            }
            enc.push(&samples)?;
            pushed += take_frames;
        }
        let consumed = (whole_frames * 4) as usize;
        carry.drain(..consumed);
        if pushed >= want_frames {
            break;
        }
    }
    drop(stdout);
    let status = child.wait().map_err(|e| format!("{NO_FFMPEG} ({e})"))?;
    let err_bytes = err_thread.join().unwrap_or_default();
    if !status.success() && pushed == 0 {
        return Err(format!(
            "ffmpeg cannot read the sound of {}: {}",
            path.display(),
            String::from_utf8_lossy(&err_bytes).trim()
        ));
    }
    // Pad any trailing samples so total_frames matches the planned duration exactly.
    while pushed < want_frames {
        let pad = (want_frames - pushed).min(16_384);
        samples.clear();
        samples.resize((pad * 2) as usize, 0);
        enc.push(&samples)?;
        pushed += pad;
    }
    let (blocks, total_frames) = enc.finish()?;
    audio::encode_opus_blocks(
        &blocks,
        2,
        total_frames,
        bitrate,
        chunking.map(|c| c.0),
        "streamed 16-bit PCM",
    )
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

/// The id of a stream's low-bitrate ABR rendition (`og_video::stream::low_size`).
pub fn stream_low_id(id: &str) -> String {
    format!("{id}#stream480")
}

/// The plan of a stream's fallback rendition: the same, at `size`.
pub fn fallback_plan(plan: &VideoPlan, size: (u32, u32)) -> VideoPlan {
    let mut decl = plan.decl.clone();
    decl.crf = decl.rendition_crf();
    VideoPlan {
        size,
        decl,
        ..plan.clone()
    }
}

/// The plan of a stream's 480p low-bitrate rendition: the same, at `size`.
pub fn low_plan(plan: &VideoPlan, size: (u32, u32)) -> VideoPlan {
    let mut decl = plan.decl.clone();
    decl.crf = decl.low_crf();
    VideoPlan {
        size,
        decl,
        ..plan.clone()
    }
}

/// The id of a stream's rendition in `r` (`<id>#streamhevc`, `<id>#streamh264`).
pub fn rendition_id(id: &str, r: Rendition) -> String {
    format!("{id}#stream{}", r.word())
}

/// The plan of a stream's rendition in `r`: the same frames by its encoder.
pub fn rendition_plan(plan: &VideoPlan, r: Rendition) -> VideoPlan {
    VideoPlan {
        encoder: r.encoder().to_string(),
        rendition: Some(r),
        renditions: Vec::new(),
        missing: Vec::new(),
        ..plan.clone()
    }
}

/// The renditions a stream cooks, and the default ones this ffmpeg cannot
/// encode; refused when one the file names cannot be encoded.
pub fn renditions(decl: &VideoDecl) -> Result<(Vec<Rendition>, Vec<Rendition>), String> {
    let has = |r: Rendition| has_encoder(r.encoder());
    match &decl.renditions {
        None => Ok(if has(Rendition::Hevc) {
            (vec![Rendition::Hevc], Vec::new())
        } else {
            (Vec::new(), vec![Rendition::Hevc])
        }),
        Some(list) => {
            if let Some(r) = list.iter().find(|r| !has(**r)) {
                return Err(format!(
                    "renditions {}: this ffmpeg has no {} (og doctor says what it finds; brew install ffmpeg, or apt install ffmpeg)",
                    r.word(),
                    r.encoder()
                ));
            }
            Ok((list.clone(), Vec::new()))
        }
    }
}

/// The id of a stream's poster frame.
pub fn poster_id(id: &str) -> String {
    format!("{id}#poster")
}

/// Whether this ffmpeg has the encoder `name` (false without ffmpeg).
pub fn has_encoder(name: &str) -> bool {
    static LIST: OnceLock<String> = OnceLock::new();
    LIST.get_or_init(|| {
        Command::new(ffmpeg_bin())
            .args(["-hide_banner", "-encoders"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default()
    })
    .lines()
    .any(|l| l.split_whitespace().nth(1) == Some(name))
}

/// The AV1 encoder this ffmpeg has: `libsvtav1`, else `libaom-av1`
/// (`None` without either, or without ffmpeg).
pub fn av1_encoder() -> Option<&'static str> {
    ["libsvtav1", "libaom-av1"]
        .into_iter()
        .find(|e| has_encoder(e))
}

/// What a stream needs of its encoder: refused without one.
pub fn stream_encoder() -> Result<&'static str, String> {
    av1_encoder().ok_or_else(|| {
        "a streamed video is encoded to AV1 by ffmpeg's libsvtav1 or libaom-av1, and this ffmpeg has neither (og doctor says what it finds; brew install ffmpeg, or apt install ffmpeg with libsvtav1)".to_string()
    })
}

/// A stream encoded: its IVF (AV1) or MP4 (a rendition) read into units.
pub struct Encoded {
    /// Its [`format::video_codec`].
    pub codec: u32,
    pub units: Vec<og_video::ivf::Unit>,
    /// The decoder's configuration: AV1's sequence header OBU, or the
    /// `avcC` or `hvcC` record.
    pub config: Vec<u8>,
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

/// Runs ffmpeg for the AV1 (or a rendition's H.264 or HEVC): the `fps`
/// filter, bicubic scale to BT.709 limited range 4:2:0, the last frame held
/// so the count is exactly the plan's, one unit per frame in presentation
/// order (no B-frames), a key frame every key interval and nowhere else.
fn encode(path: &Path, plan: &VideoPlan) -> Result<Encoded, String> {
    let (w, h) = plan.size;
    let tmp = std::env::temp_dir().join(format!(
        "og-video-{}-{:016x}.{}",
        std::process::id(),
        og_core::hash::hash_bytes(format!("{}|{}", path.display(), plan.describe()).as_bytes()),
        if plan.rendition.is_some() {
            "mp4"
        } else {
            "ivf"
        }
    ));
    let seconds = f64::from(plan.frames) / f64::from(plan.decl.fps);
    let key = plan.key_interval().to_string();
    let crf = plan.decl.crf.to_string();
    let fps = plan.decl.fps;
    // Packed alpha: keyed frames ([`encode_keyed`]) on stdin; the colour
    // above, the alpha as limited-range luma with neutral chroma below.
    let (mut cmd, filter) = if plan.decl.alpha {
        let mut c = Command::new(ffmpeg_bin());
        c.args(["-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba"])
            .args(["-s", &format!("{w}x{h}"), "-framerate", &fps.to_string()])
            .args(["-i", "-"]);
        (
            c,
            "split[c][m];[c]scale=out_color_matrix=bt709:out_range=tv,format=yuv420p[top];[m]alphaextract,scale=out_range=tv,format=yuv420p[bottom];[top][bottom]vstack,tpad=stop_mode=clone:stop=-1".to_string(),
        )
    } else {
        let mut c = reading(path, plan);
        c.args(["-t", &format!("{seconds}"), "-an"]);
        (
            c,
            format!(
                "fps={fps},scale={w}:{h}:flags=bicubic:out_color_matrix=bt709:out_range=tv,format=yuv420p,tpad=stop_mode=clone:stop=-1"
            ),
        )
    };
    cmd.args(["-vf", &filter])
        .args(["-frames:v", &plan.frames.to_string()]);
    let crf = plan
        .rendition
        .map_or(crf, |r| r.crf(plan.decl.crf).to_string());
    cmd.args(["-c:v", &plan.encoder, "-crf", &crf, "-g", &key]);
    match plan.encoder.as_str() {
        // No scene-cut key frames: one every key interval, no more.
        "libsvtav1" => cmd.args(["-preset", "8", "-svtav1-params", "scd=0"]),
        "libx264" => cmd.args([
            "-preset",
            "medium",
            "-profile:v",
            "high",
            "-bf",
            "0",
            "-keyint_min",
            &key,
            "-sc_threshold",
            "0",
        ]),
        // `hvc1`: the parameter sets in the hvcC record, as Safari wants;
        // no encoder settings SEI in it.
        "libx265" => cmd.args([
            "-preset",
            "fast",
            "-tag:v",
            "hvc1",
            "-x265-params",
            &format!(
                "keyint={key}:min-keyint={key}:scenecut=0:bframes=0:open-gop=0:info=0:log-level=error"
            ),
        ]),
        _ => cmd.args([
            "-b:v",
            "0",
            "-cpu-used",
            "6",
            "-row-mt",
            "1",
            "-aom-params",
            &format!("enable-keyframe-filtering=0:kf-min-dist={key}:kf-max-dist={key}"),
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
        if plan.rendition.is_some() {
            "mp4"
        } else {
            "ivf"
        },
    ])
    .arg(&tmp);
    let out = if plan.decl.alpha {
        encode_keyed(path, plan, seconds, &mut cmd)
    } else {
        cmd.output().map_err(|e| format!("{NO_FFMPEG} ({e})"))
    };
    let out = out.inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
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
    let (e, size) = if let Some(r) = plan.rendition {
        let m = og_video::mp4::read(&bytes)
            .map_err(|e| format!("{}'s {}: {e}", path.display(), r.word()))?;
        (
            Encoded {
                codec: r.codec(),
                units: m.units,
                config: m.config,
            },
            (m.width, m.height),
        )
    } else {
        let ivf =
            og_video::ivf::read(&bytes).map_err(|e| format!("{}'s AV1: {e}", path.display()))?;
        (
            Encoded {
                codec: format::video_codec::AV1,
                units: ivf.units,
                config: ivf.sequence_header,
            },
            (ivf.width, ivf.height),
        )
    };
    let ch = coded_height(plan);
    if e.units.len() != plan.frames as usize || size != (w, ch) {
        return Err(format!(
            "ffmpeg encoded {} frames of {}x{} from {}, not the {} of {w}x{ch} planned",
            e.units.len(),
            size.0,
            size.1,
            path.display(),
            plan.frames
        ));
    }
    Ok(e)
}

/// The `og.VideoStream` blob of a stream's units.
pub fn cook_stream(
    path: &Path,
    hash: u64,
    plan: &VideoPlan,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let e = encoded(path, hash, plan)?;
    let chunk_frames = plan.chunk_frames();
    if e.units.len() != plan.frames as usize {
        return Err(CookError(format!(
            "{}: ffmpeg wrote {} frames of {}, not the {} planned",
            path.display(),
            e.units.len(),
            codec_name(e.codec),
            plan.frames
        )));
    }
    let mut rows = Vec::with_capacity(e.units.len());
    let mut offset = 0u32;
    let mut bytes = 0u64;
    for (i, u) in e.units.iter().enumerate() {
        if (i as u32).is_multiple_of(chunk_frames) {
            if !u.key {
                return Err(CookError(format!(
                    "{}: ffmpeg's {} has no key frame at frame {i}, where chunk {} starts",
                    path.display(),
                    codec_name(e.codec),
                    i as u32 / chunk_frames
                )));
            }
            offset = 0;
        }
        rows.push(format::VideoUnitRow {
            offset,
            size: u.data.len() as u32,
            key: u32::from(u.key),
        });
        offset += u.data.len() as u32;
        bytes += u.data.len() as u64;
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
        codec: e.codec,
        width: plan.size.0,
        height: coded_height(plan),
        frames: rows.len() as u32,
        fps: plan.decl.fps,
        key_interval: longest,
        chunk_frames,
        chunks: plan.chunks(),
    };
    let mut w = BlobWriter::new(
        format::VideoStream::type_hash(),
        format::VideoStream::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<format::VideoStreamMeta>());
    w.table("units", &rows, row_hash::<format::VideoUnitRow>());
    w.raw("config", e.config.clone(), 4);
    let kbps = bytes as f64 * 8.0 / 1000.0 / (f64::from(plan.frames) / f64::from(plan.decl.fps));
    let note = format!(
        "the video's {}: {:.1} MiB ({kbps:.0} kb/s) in {} chunk(s) of {chunk_frames} frames, {keys} key frame(s)",
        codec_name(e.codec),
        bytes as f64 / f64::from(1u32 << 20),
        plan.chunks()
    );
    Ok((w.finish(), vec![note]))
}

/// Chunk `n` of a stream: the units of its frames, from the key frame
/// `n · chunk_frames` on, in the order its index places them.
pub fn cook_chunk(
    path: &Path,
    hash: u64,
    plan: &VideoPlan,
    n: u32,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let e = encoded(path, hash, plan)?;
    let cf = plan.chunk_frames() as usize;
    let first = n as usize * cf;
    let units = e
        .units
        .get(first..(first + cf).min(e.units.len()))
        .unwrap_or_default();
    if units.is_empty() {
        return Err(CookError(format!(
            "{}: chunk {n} is past the {} frames ffmpeg wrote",
            path.display(),
            e.units.len()
        )));
    }
    let mut data = Vec::new();
    for u in units {
        data.extend_from_slice(&u.data);
    }
    let mut w = BlobWriter::new(
        format::VideoChunk::type_hash(),
        format::VideoChunk::schema_version(),
        stamp,
    );
    let meta = format::VideoChunkMeta {
        chunk: n,
        first: first as u32,
        frames: units.len() as u32,
    };
    w.table("meta", &[meta], row_hash::<format::VideoChunkMeta>());
    w.raw("data", data, 16);
    Ok((w.finish(), Vec::new()))
}

/// A [`format::video_codec`]'s name in notes and errors.
fn codec_name(codec: u32) -> &'static str {
    match codec {
        format::video_codec::H264 => "H.264",
        format::video_codec::HEVC => "HEVC",
        _ => "AV1",
    }
}

/// Downscales an RGBA8 image by box/area averaging to fit `max_side`.
fn downscale_poster(width: u32, height: u32, rgba: &[u8], max_side: u32) -> (u32, u32, Vec<u8>) {
    let long = width.max(height);
    if max_side == 0 || long <= max_side {
        return (width, height, rgba.to_vec());
    }
    let scale = f64::from(max_side) / f64::from(long);
    let dw = ((f64::from(width) * scale).round() as u32).max(2);
    let dh = ((f64::from(height) * scale).round() as u32).max(2);
    let mut out = vec![0u8; (dw * dh * 4) as usize];
    for dy in 0..dh {
        let y0 = (dy * height) / dh;
        let y1 = (((dy + 1) * height).div_ceil(dh)).min(height).max(y0 + 1);
        for dx in 0..dw {
            let x0 = (dx * width) / dw;
            let x1 = (((dx + 1) * width).div_ceil(dw)).min(width).max(x0 + 1);
            let mut sum = [0u32; 4];
            let mut cnt = 0u32;
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let i = ((sy * width + sx) * 4) as usize;
                    for c in 0..4 {
                        sum[c] += u32::from(rgba[i + c]);
                    }
                    cnt += 1;
                }
            }
            let o = ((dy * dw + dx) * 4) as usize;
            let half = cnt / 2;
            for c in 0..4 {
                out[o + c] = ((sum[c] + half) / cnt) as u8;
            }
        }
    }
    (dw, dh, out)
}

/// The poster: frame 0 as rav1d decodes it, scaled to `posterSize` if set,
/// cooked as a texture of the video's kind.
pub fn cook_poster(
    path: &Path,
    hash: u64,
    plan: &VideoPlan,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Result<Vec<u8>, CookError> {
    let e = encoded(path, hash, plan)?;
    let mut d = og_video::decode::Decoder::new(1)
        .map_err(CookError)?
        .packed_alpha(plan.decl.alpha);
    let pic = d
        .decode(&e.units[0].data)
        .map_err(|x| CookError(format!("{}: frame 0: {x}", path.display())))?
        .ok_or_else(|| CookError(format!("{}: frame 0 shows no picture", path.display())))?;
    let (width, height, rgba) =
        downscale_poster(pic.width, pic.height, &pic.rgba, plan.decl.poster_size);
    let img = texture::Image {
        width,
        height,
        rgba,
    };
    texture::cook(&img, plan.decl.kind, &frame_options(opts), stamp)
}

pub(crate) static VIDEO: KindDef = KindDef {
    name: "video",
    extensions: &[],
    types: &[VIDEO_TYPE],
    by_content: false,
    streams: false,
    plan: plan_video,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&VIDEO];

fn plan_video<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { tex_desc, .. } = *cx;
    let PlanState { early_errors, .. } = state;
    let text = String::from_utf8_lossy(&s.bytes);
    let root = crate::cook::assets_root(s);
    let planned = parse(&s.rel, &text, &|p| root.join(p).is_file()).and_then(|d| {
        let src = root.join(&d.source);
        let bytes = std::fs::read(&src).map_err(|e| format!("{}: {e}", src.display()))?;
        let encoder = if d.stream { stream_encoder()? } else { "" };
        let mut plan = plan(&d, probe(&src)?, encoder)?;
        if d.stream {
            (plan.renditions, plan.missing) = renditions(&d)?;
        }
        let mut locale_files = Vec::with_capacity(d.audio_locales.len());
        for (tag, rel) in &d.audio_locales {
            let p = root.join(rel);
            let b = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            locale_files.push((tag.clone(), p, hash_bytes(&b)));
        }
        Ok((src, hash_bytes(&bytes), plan, locale_files))
    });
    match planned {
        Ok((src, src_hash, plan, locale_files)) => {
            let described = plan.describe();
            let base = [
                b"video 3".as_slice(),
                tex_desc.as_bytes(),
                &hash,
                &src_hash.to_le_bytes(),
                described.as_bytes(),
            ];
            let plan = Box::new(plan);
            outs.push((
                s.rel.clone(),
                format::Video::NAME,
                stamp(&base),
                Box::new(VideoMetaJob(plan.clone())),
            ));
            if plan.decl.stream {
                // Each stream (the AV1, its 720p & 480p renditions, its HEVC
                // and H.264 ones): its index and its chunks.
                let mut streams = vec![(
                    stream_id(&s.rel),
                    stamp(&[&base[..], &[b"stream".as_slice()]].concat()),
                    plan.clone(),
                )];
                let s720 = og_video::stream::fallback_size(plan.size);
                if let Some(size) = s720 {
                    streams.push((
                        stream_fallback_id(&s.rel),
                        stamp(&[&base[..], &[b"stream720".as_slice()]].concat()),
                        Box::new(fallback_plan(&plan, size)),
                    ));
                }
                if let Some(size480) = og_video::stream::low_size(plan.size)
                    && Some(size480) != s720
                {
                    streams.push((
                        stream_low_id(&s.rel),
                        stamp(&[&base[..], &[b"stream480".as_slice()]].concat()),
                        Box::new(low_plan(&plan, size480)),
                    ));
                }
                for &r in &plan.renditions {
                    let rp = rendition_plan(&plan, r);
                    streams.push((
                        rendition_id(&s.rel, r),
                        stamp(
                            &[&base[..], &[r.word().as_bytes(), rp.describe().as_bytes()]].concat(),
                        ),
                        Box::new(rp),
                    ));
                }
                for (id, st, sp) in streams {
                    for n in 0..sp.chunks() {
                        outs.push((
                            og_bundle::runtime::video_chunk_id(&id, n),
                            format::VideoChunk::NAME,
                            stamp(&[&st.to_le_bytes()[..], &n.to_le_bytes()[..]]),
                            Box::new(VideoChunkJob(src.clone(), src_hash, sp.clone(), n)),
                        ));
                    }
                    outs.push((
                        id,
                        format::VideoStream::NAME,
                        st,
                        Box::new(VideoStreamJob(src.clone(), src_hash, sp)),
                    ));
                }
                outs.push((
                    poster_id(&s.rel),
                    format::Texture::NAME,
                    stamp(&[&base[..], &[b"poster".as_slice()]].concat()),
                    Box::new(VideoPosterJob(src.clone(), src_hash, plan.clone())),
                ));
            } else {
                let frames = frames_id(&s.rel);
                outs.push((
                    frames.clone(),
                    format::TextureArray::NAME,
                    stamp(&[&base[..], &[b"array".as_slice()]].concat()),
                    Box::new(VideoArrayJob(s, plan.clone())),
                ));
                for n in 0..plan.frames as usize {
                    let st = stamp(&[&base[..], &[&(n as u32).to_le_bytes()[..]]].concat());
                    outs.push((
                        texture_array::layer_id(&frames, n),
                        format::Texture::NAME,
                        st,
                        Box::new(VideoFrameJob(src.clone(), src_hash, plan.clone(), n)),
                    ));
                }
            }
            let audio_chunking = stream_audio_chunking(&plan);
            let push_audio =
                |outs: &mut Vec<Out<'a>>, aid: String, asrc: PathBuf, ahash: u64, tag: &[u8]| {
                    let ast = stamp(
                        &[
                            &base[..],
                            &[b"audio-v2".as_slice(), tag, &ahash.to_le_bytes()],
                        ]
                        .concat(),
                    );
                    outs.push((
                        aid.clone(),
                        format::Sound::NAME,
                        ast,
                        Box::new(VideoAudioJob(
                            asrc.clone(),
                            ahash,
                            plan.clone(),
                            audio_chunking,
                        )),
                    ));
                    if let Some((per, count)) = audio_chunking {
                        for n in 1..count {
                            let cst = stamp(&[&ast.to_le_bytes()[..], &n.to_le_bytes()[..]]);
                            outs.push((
                                format!("{aid}#chunk{n}"),
                                format::SoundChunk::NAME,
                                cst,
                                Box::new(VideoAudioChunkJob(
                                    asrc.clone(),
                                    ahash,
                                    plan.clone(),
                                    (per, count),
                                    n,
                                )),
                            ));
                        }
                    }
                };
            if plan.audio {
                push_audio(outs, audio_id(&s.rel), src.clone(), src_hash, b"default");
            }
            for (tag, lpath, lhash) in locale_files {
                push_audio(
                    outs,
                    audio_locale_id(&s.rel, &tag),
                    lpath,
                    lhash,
                    tag.as_bytes(),
                );
            }
        }
        Err(e) => early_errors.push((s.rel.clone(), "video", e)),
    }
}

/// A video's description: its size, rate and outputs.
struct VideoMetaJob(Box<VideoPlan>);

impl Job for VideoMetaJob {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let VideoMetaJob(plan) = self;
        cook_meta(plan, p.stamp)
    }

    /// A video's outputs are cooked from the file each time.
    fn action(&self) -> Action<'_> {
        Action::Never
    }
}

/// A short video's frames as one texture array.
struct VideoArrayJob<'a>(&'a Source, Box<VideoPlan>);

impl Job for VideoArrayJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let VideoArrayJob(s, plan) = self;
        let Exec { options, .. } = ex;
        Ok((
            cook_array(&s.rel, plan, &options.texture, p.stamp),
            Vec::new(),
        ))
    }

    /// A video's outputs are cooked from the file each time.
    fn action(&self) -> Action<'_> {
        Action::Never
    }
}

/// One frame of a short video's array.
struct VideoFrameJob(PathBuf, u64, Box<VideoPlan>, usize);

impl Job for VideoFrameJob {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let VideoFrameJob(src, h, plan, n) = self;
        let Exec { options, .. } = ex;
        cook_frame(src, *h, plan, *n, &options.texture, p.stamp).map(|b| (b, Vec::new()))
    }

    /// A video's outputs are cooked from the file each time.
    fn action(&self) -> Action<'_> {
        Action::Never
    }
}

/// A video's sound track (head chunk for streamed chunked Opus, or whole blob).
struct VideoAudioJob(PathBuf, u64, Box<VideoPlan>, Option<(u32, u32)>);

impl Job for VideoAudioJob {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let VideoAudioJob(src, hash, plan, chunking) = self;
        if plan.decl.stream {
            let e = stream_audio_encoded(src, *hash, plan, *chunking)?;
            return audio::cook(&e, *chunking, p.stamp)
                .map(|(b, n)| (b, vec![format!("the video's sound: {n}")]));
        }
        let wav = audio_wav(src, plan)?;
        let pcm =
            audio::wav(&format!("{}'s sound track", src.display()), &wav).map_err(CookError)?;
        let e = audio::encode(&pcm, None).map_err(CookError)?;
        audio::cook(&e, None, p.stamp).map(|(b, n)| (b, vec![format!("the video's sound: {n}")]))
    }

    /// A video's outputs are cooked from the file each time.
    fn action(&self) -> Action<'_> {
        Action::Never
    }
}

/// Chunk `n` (1..) of a streamed video's chunked Opus sound track.
struct VideoAudioChunkJob(PathBuf, u64, Box<VideoPlan>, (u32, u32), u32);

impl Job for VideoAudioChunkJob {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let VideoAudioChunkJob(src, hash, plan, chunking, n) = self;
        let e = stream_audio_encoded(src, *hash, plan, Some(*chunking))?;
        audio::cook_chunk(&e, *n, p.stamp).map(|b| (b, Vec::new()))
    }

    fn action(&self) -> Action<'_> {
        Action::Never
    }
}

/// A chunk of a long video's stream.
struct VideoChunkJob(PathBuf, u64, Box<VideoPlan>, u32);

impl Job for VideoChunkJob {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let VideoChunkJob(src, h, plan, n) = self;
        cook_chunk(src, *h, plan, *n, p.stamp)
    }

    /// A video's outputs are cooked from the file each time.
    fn action(&self) -> Action<'_> {
        Action::Never
    }
}

/// A long video's stream: its index.
struct VideoStreamJob(PathBuf, u64, Box<VideoPlan>);

impl Job for VideoStreamJob {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let VideoStreamJob(src, h, plan) = self;
        cook_stream(src, *h, plan, p.stamp)
    }

    /// A video's outputs are cooked from the file each time.
    fn action(&self) -> Action<'_> {
        Action::Never
    }
}

/// A long video's poster frame.
struct VideoPosterJob(PathBuf, u64, Box<VideoPlan>);

impl Job for VideoPosterJob {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let VideoPosterJob(src, h, plan) = self;
        let Exec { options, .. } = ex;
        cook_poster(src, *h, plan, &options.texture, p.stamp).map(|b| (b, Vec::new()))
    }

    /// A video's outputs are cooked from the file each time.
    fn action(&self) -> Action<'_> {
        Action::Never
    }
}
