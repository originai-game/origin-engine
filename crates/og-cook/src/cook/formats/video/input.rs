//! Checked video declarations and the immutable media snapshot shared by
//! reference discovery, ffprobe, cache stamps and ffmpeg jobs.
use super::*;
use crate::cook::source_schemas::media::clips::{FrameKind, Video};
use crate::cook::{source_content, source_index, source_path, source_project};
use std::io::{Read, Write};

pub(crate) fn declaration(input: Video) -> Result<VideoDecl, String> {
    let stream = input.stream.unwrap_or(false);
    if !stream
        && (input.keyframe.is_some()
            || input.chunk.is_some()
            || input.crf.is_some()
            || input.crf720.is_some()
            || input.poster_size.is_some()
            || input.renditions.is_some()
            || input.low_rendition.is_some())
    {
        return Err("keyframe, chunk, crf, crf720, posterSize, renditions and lowRendition require stream: true".into());
    }
    let ext = Path::new(&input.source)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(ext.as_str(), "mp4" | "webm" | "mov") {
        return Err("video source must be an .mp4, .webm or .mov".into());
    }
    let mut tags = std::collections::BTreeSet::new();
    for tag in input.audio_locale.keys() {
        if tag.is_empty()
            || !tag
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
            || !tags.insert(tag.to_ascii_lowercase())
        {
            return Err(format!(
                "audioLocale tag {tag:?} must be unique ignoring case and contain only ASCII letters, digits, '-' or '_'"
            ));
        }
    }
    let renditions = input.renditions.map(|list| {
        let mut out = Vec::new();
        for r in list {
            let r = match r {
                crate::cook::source_schemas::media::clips::Rendition::Hevc => Rendition::Hevc,
                crate::cook::source_schemas::media::clips::Rendition::H264 => Rendition::H264,
            };
            if !out.contains(&r) {
                out.push(r);
            }
        }
        out
    });
    Ok(VideoDecl {
        source: input.source,
        fps: input.fps.unwrap_or(30),
        max_seconds: input
            .max_seconds
            .unwrap_or(if stream { f64::INFINITY } else { 8.0 }),
        size: input.size.map(|[w, h]| (w, h)),
        kind: match input.kind.unwrap_or(FrameKind::Color) {
            FrameKind::Color => format::kind::COLOR,
            FrameKind::Pixel => format::kind::PIXEL,
        },
        audio: input.audio.unwrap_or(true),
        stream,
        keyframe: input.keyframe.unwrap_or(1.0),
        chunk: input.chunk.unwrap_or(CHUNK_SECONDS),
        crf: u32::from(input.crf.unwrap_or(32)),
        crf720: input.crf720.map(u32::from),
        renditions,
        loudness: input.loudness,
        ai_generated: input.ai_generated.unwrap_or(false),
        placeholder: input.placeholder.unwrap_or(false),
        alpha: input.alpha.unwrap_or(false),
        poster_size: input.poster_size.unwrap_or(DEFAULT_POSTER_SIZE),
        audio_locales: input.audio_locale.into_iter().collect(),
        low_rendition: input.low_rendition.unwrap_or(false),
    })
}

fn media_path(root: &Path, file: &str) -> Result<PathBuf, String> {
    if !file.starts_with("assets/") && !file.starts_with("sources/") {
        return Err(format!(
            "{file}: video inputs belong under the owning module's assets/ or sources/"
        ));
    }
    let path = source_path::checked(root, file).map_err(|e| format!("{file}: {e}"))?;
    if !path.is_file() {
        return Err(format!("{file}: video input is not a regular file"));
    }
    Ok(path)
}

/// Read a canonical video ID or project-relative declaration for source tools, using the cook's
/// discovery, closed evaluator, schema and module-relative path rules.
/// Returns the owning module's root and normalized declaration, not OGT.
pub fn read(root: &Path, id: &str) -> Result<(PathBuf, VideoDecl), String> {
    let project = source_project::Project::open(root).map_err(|e| e.to_string())?;
    let index = source_index::Index::open(root, &project).map_err(|e| e.to_string())?;
    let entry = index
        .entries
        .get(id)
        .or_else(|| index.entries.values().find(|e| e.file == id))
        .filter(|e| e.factory == "video")
        .ok_or_else(|| format!("{id}: not an indexed video declaration"))?;
    let content = source_content::Evaluator::default()
        .evaluate(root, &project, entry, &BTreeMap::new(), None)
        .map_err(|e| e.to_string())?;
    let file: crate::cook::source_schemas::File =
        serde_json::from_value(content.data().root().into_json()).map_err(|e| e.to_string())?;
    let input: Video = serde_json::from_value(file.value).map_err(|e| e.to_string())?;
    let decl = declaration(input)?;
    let root = root.join(&project.modules[&entry.module].root);
    media_path(&root, &decl.source)?;
    for (_, file) in &decl.audio_locales {
        media_path(&root, file)?;
    }
    Ok((root, decl))
}

pub(crate) struct Prepared {
    // Keep snapshots alive through every planned job. Inputs are copied with
    // bounded memory, so a large streamed film never becomes one RAM buffer.
    _directory: tempfile::TempDir,
    pub source: PathBuf,
    pub hash: u64,
    pub plan: VideoPlan,
    pub locales: Vec<(String, PathBuf, u64)>,
    pub dependencies: Vec<PathBuf>,
}

impl Prepared {
    pub fn new(root: &Path, decl: VideoDecl) -> Result<Self, String> {
        let directory = tempfile::Builder::new()
            .prefix("og-video-input-")
            .tempdir()
            .map_err(|e| e.to_string())?;
        let mut files = BTreeMap::<String, (PathBuf, u64)>::new();
        let mut dependencies = Vec::new();
        for file in std::iter::once(&decl.source).chain(decl.audio_locales.iter().map(|(_, f)| f)) {
            if files.contains_key(file) {
                continue;
            }
            let path = media_path(root, file)?;
            let snapshot = directory.path().join(files.len().to_string());
            let mut source = std::fs::File::open(&path).map_err(|e| format!("{file}: {e}"))?;
            let mut target = std::fs::File::create(&snapshot).map_err(|e| e.to_string())?;
            let mut hash = og_core::hash::StableHasher::default();
            let mut buffer = [0u8; 64 * 1024];
            loop {
                let n = source
                    .read(&mut buffer)
                    .map_err(|e| format!("{file}: {e}"))?;
                if n == 0 {
                    break;
                }
                target.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
                hash.write(&buffer[..n]);
            }
            files.insert(file.clone(), (snapshot, hash.finish()));
            dependencies.push(path);
        }
        let (source, hash) = files[&decl.source].clone();
        let encoder = if decl.stream { stream_encoder()? } else { "" };
        let mut plan = plan(&decl, probe(&source)?, encoder)?;
        if decl.stream {
            (plan.renditions, plan.missing) = renditions(&decl)?;
        }
        let locales = decl
            .audio_locales
            .iter()
            .map(|(tag, file)| {
                let (path, hash) = files[file].clone();
                (tag.clone(), path, hash)
            })
            .collect();
        Ok(Self {
            _directory: directory,
            source,
            hash,
            plan,
            locales,
            dependencies,
        })
    }

    /// The graph and planner enumerate precisely the same actual outputs.
    pub fn outputs(&self) -> BTreeMap<String, String> {
        output_kinds(&self.plan)
    }
}

pub(super) fn output_kinds(plan: &VideoPlan) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if plan.decl.stream {
        out.insert("poster".into(), "texture".into());
        for (id, p) in streams(plan) {
            out.insert(id.clone(), "video-stream".into());
            for n in 0..p.chunks() {
                out.insert(format!("{id}.{n}"), "video-chunk".into());
            }
        }
    } else {
        out.insert("frames".into(), "texture-array".into());
        for n in 0..plan.frames {
            out.insert(format!("frames#layer{n}"), "texture".into());
        }
    }
    let audio = plan.audio.then_some("audio".to_string()).into_iter().chain(
        plan.decl
            .audio_locales
            .iter()
            .map(|(tag, _)| format!("audio.{tag}")),
    );
    for id in audio {
        out.insert(id.clone(), "sound".into());
        if let Some((_, count)) = stream_audio_chunking(plan) {
            for n in 1..count {
                out.insert(format!("{id}#chunk{n}"), "sound-chunk".into());
            }
        }
    }
    out
}

/// The same rendition list drives reference validation and actual cook jobs.
pub(super) fn streams(plan: &VideoPlan) -> Vec<(String, VideoPlan)> {
    let mut streams = vec![("stream".to_string(), plan.clone())];
    let fallback = og_video::stream::fallback_size(plan.size);
    if let Some(size) = fallback {
        streams.push(("stream720".into(), fallback_plan(plan, size)));
    }
    if let Some(size) = og_video::stream::low_size(plan.size).filter(|_| plan.decl.low_rendition)
        && Some(size) != fallback
    {
        streams.push(("stream480".into(), low_plan(plan, size)));
    }
    for &r in &plan.renditions {
        streams.push((format!("stream{}", r.word()), rendition_plan(plan, r)));
    }
    streams
}
