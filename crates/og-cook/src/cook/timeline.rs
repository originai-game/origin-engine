//! `type timeline` → timeline blobs (depth S2): the file parsed by
//! og-timeline, its caption files (`captions "narration.srt"`, or a
//! word-timestamp `.json`) imported as caption keys, the sounds and models
//! it names checked against the bundle, and the canonical text stored for
//! the runtime to parse again.
//!
//! A caption file is found beside the timeline, else from `assets/`; it is
//! part of the timeline (not an asset of its own) and in its stamp.

use super::terrain::resolve;
use super::{CookError, Source, assets_root, hash_bytes};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;
use og_core::ogt::Document;
use og_timeline::{Timeline, Track};
use std::path::Path;

pub(super) const TIMELINE_TYPE: &str = og_timeline::TIMELINE_TYPE;

/// The caption files a timeline names, asset-relative.
pub(super) fn caption_files(rel: &str, root: &Path, text: &str) -> Vec<String> {
    let Ok(doc) = Document::parse(text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for t in doc.body.get_all("track") {
        if t.values.first().and_then(|v| v.as_word()) != Some("caption") {
            continue;
        }
        let Some(b) = &t.block else { continue };
        for c in b.get_all("captions") {
            if let Some(f) = c.values.first().and_then(|v| v.as_str()) {
                out.push(resolve(rel, root, f));
            }
        }
    }
    out
}

/// The stamp's dependencies: each caption file with its hash.
pub(super) fn dependencies(s: &Source) -> String {
    let root = assets_root(s);
    caption_files(&s.rel, &root, &String::from_utf8_lossy(&s.bytes))
        .into_iter()
        .map(|f| {
            let h = std::fs::read(root.join(&f))
                .map(|b| hash_bytes(&b))
                .unwrap_or(0);
            format!("{f}={h:016x}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn cook_timeline(
    s: &Source,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("a timeline is text (UTF-8)".into()))?;
    let root = assets_root(s);
    let mut read = |file: &str| -> Result<String, String> {
        let rel = resolve(&s.rel, &root, file);
        std::fs::read_to_string(root.join(&rel))
            .map_err(|e| format!("{rel} (beside the timeline or under assets/): {e}"))
    };
    let t = Timeline::parse(text, &mut read).map_err(CookError)?;
    // Every sound and model it names is a cooked asset.
    let mut missing = Vec::new();
    let mut need = |id: &str, what: &str| {
        if !known(id) {
            missing.push(format!("{what} '{id}'"));
        }
    };
    let mut captions = 0;
    for tr in &t.tracks {
        match tr {
            Track::Audio(keys) => keys.iter().for_each(|k| need(&k.sound, "sound")),
            Track::Music(keys) => keys.iter().for_each(|k| need(&k.sound, "music")),
            Track::Animation(a) if !a.model.is_empty() => need(&a.model, "model"),
            Track::Caption(c) => captions += c.keys.len(),
            _ => {}
        }
    }
    if !missing.is_empty() {
        return Err(CookError(format!(
            "the timeline names {} that the bundle lacks (an asset id under assets/, such as \"audio/blip.wav\")",
            missing.join(", ")
        )));
    }
    let canonical = t.to_text();
    let mut w = BlobWriter::new(
        format::Timeline::type_hash(),
        format::Timeline::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::TimelineMeta {
            text_bytes: canonical.len() as u32,
            tracks: t.tracks.len() as u32,
            duration: t.duration,
        }],
        format::row_hash::<format::TimelineMeta>(),
    )
    .raw("text", canonical.into_bytes(), 16);
    let kinds: Vec<&str> = t.tracks.iter().map(Track::kind).collect();
    Ok((
        w.finish(),
        vec![format!(
            "timeline '{}': {} s, {} track(s) ({}), {} path(s), {captions} caption(s)",
            t.name,
            t.duration,
            t.tracks.len(),
            kinds.join(", "),
            t.paths.len()
        )],
    ))
}
