//! `type timeline` → timeline blobs (depth S2): the file parsed by
//! og-timeline, its caption files (`captions "narration.srt"`, or a
//! word-timestamp `.json`) imported as caption keys, the sounds, models,
//! videos, effects, timelines and scenes it names checked against the
//! bundle (and recorded as what it refers to, so it needs the engine
//! modules they need: `pipeline/needs.rs`), and the canonical text stored
//! for the runtime to parse again.
//!
//! A caption file is found beside the timeline, else from `assets/`; it is
//! part of the timeline (not an asset of its own) and in its stamp.

use super::terrain::resolve;
use super::{
    CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, assets, assets_root,
    hash_bytes, stamp,
};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;
use og_core::ogt::Document;
use og_timeline::{FieldValue, FieldsKind, FieldsTrack, NEST_DEPTH, Timeline, Track};
use std::path::Path;

pub(crate) const TIMELINE_TYPE: &str = og_timeline::TIMELINE_TYPE;

/// The caption files a timeline names, asset-relative.
/// Its translations found beside them included (a new one recooks it).
pub(crate) fn caption_files(rel: &str, root: &Path, text: &str) -> Vec<String> {
    let Ok(doc) = Document::parse(&with_translations(rel, root, text)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for t in doc.body.get_all("track") {
        if t.values.first().and_then(|v| v.as_word()) != Some("caption") {
            continue;
        }
        let Some(b) = &t.block else { continue };
        let locales = b.get_all("locale").filter_map(|l| l.block.as_ref());
        for block in std::iter::once(b).chain(locales) {
            for c in block.get_all("captions") {
                if let Some(f) = c.values.first().and_then(|v| v.as_str()) {
                    out.push(resolve(rel, root, f));
                }
            }
        }
    }
    out
}

/// A timeline's text with each caption file's translations beside it
/// (`kitchen.zh.srt`, `kitchen.pt-BR.srt` beside `kitchen.srt`) as
/// `locale "<tag>" { captions "<file>" [offset] }` in its track, unless the
/// track writes that locale itself. Unchanged without any.
pub(crate) fn with_translations(rel: &str, root: &Path, text: &str) -> String {
    let Ok(mut doc) = Document::parse(text) else {
        return text.to_string();
    };
    let tag_ok = |t: &str| {
        let mut parts = t.split('-');
        parts.next().is_some_and(|l| {
            (2..=3).contains(&l.len()) && l.chars().all(|c| c.is_ascii_alphabetic())
        }) && parts
            .all(|p| (1..=8).contains(&p.len()) && p.chars().all(|c| c.is_ascii_alphanumeric()))
    };
    let mut changed = false;
    for t in doc.body.entries.iter_mut().filter(|e| e.key == "track") {
        if t.values.first().and_then(|v| v.as_word()) != Some("caption") {
            continue;
        }
        let Some(b) = &mut t.block else { continue };
        let mut have: Vec<String> = b
            .get_all("locale")
            .filter_map(|l| l.values.first().and_then(|v| v.as_str()))
            .map(str::to_ascii_lowercase)
            .collect();
        let mut add = Vec::new();
        for c in b.get_all("captions") {
            let Some(f) = c.values.first().and_then(|v| v.as_str()) else {
                continue;
            };
            let file = root.join(resolve(rel, root, f));
            let (Some(dir), Some(stem), Some(ext)) = (
                file.parent(),
                file.file_stem().and_then(|s| s.to_str()),
                file.extension().and_then(|s| s.to_str()),
            ) else {
                continue;
            };
            let Ok(entries) = std::fs::read_dir(dir) else {
                continue;
            };
            let mut found: Vec<String> = entries
                .filter_map(|e| e.ok()?.file_name().into_string().ok())
                .collect();
            found.sort();
            for name in found {
                let Some(tag) = name
                    .strip_prefix(&format!("{stem}."))
                    .and_then(|r| r.strip_suffix(&format!(".{ext}")))
                else {
                    continue;
                };
                if !tag_ok(tag) || have.contains(&tag.to_ascii_lowercase()) {
                    continue;
                }
                have.push(tag.to_ascii_lowercase());
                // The authored path with the translation's name, so it
                // resolves as the base file does.
                let path = match f.rsplit_once('/') {
                    Some((d, _)) => format!("{d}/{name}"),
                    None => name.clone(),
                };
                let mut captions = c.clone();
                captions.values[0] = og_core::ogt::Value::String(path);
                let mut e = og_core::ogt::Entry::new("locale")
                    .with(og_core::ogt::Value::String(tag.to_string()));
                e.block = Some(og_core::ogt::Block::new().with(captions));
                add.push(e);
            }
        }
        changed |= !add.is_empty();
        b.entries.extend(add);
    }
    match changed.then(|| doc.to_text()) {
        Some(Ok(t)) => t,
        _ => text.to_string(),
    }
}

/// The timelines a timeline nests (`track timeline { key 0 { play ... } }`),
/// as written (asset ids).
fn nested_files(text: &str) -> Vec<String> {
    let Ok(doc) = Document::parse(text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for t in doc.body.get_all("track") {
        if t.values.first().and_then(|v| v.as_word()) != Some("timeline") {
            continue;
        }
        let Some(b) = &t.block else { continue };
        for k in b.get_all("key") {
            let Some(kb) = &k.block else { continue };
            for p in kb.get_all("play") {
                if let Some(f) = p.values.first().and_then(|v| v.as_str()) {
                    out.push(f.to_string());
                }
            }
        }
    }
    out
}

/// The assets a timeline's tracks name, as written (asset ids): its sounds
/// and music, models, effects, videos, scenes and nested timelines. None
/// for a text that does not parse: the cook reports that.
fn named_assets(s: &Source) -> Vec<String> {
    let Ok(t) = parse(s) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for tr in t.tracks {
        match tr {
            Track::Audio(keys) => out.extend(keys.into_iter().map(|k| k.sound)),
            Track::Music(keys) => out.extend(keys.into_iter().map(|k| k.sound)),
            Track::Animation(a) => out.push(a.model),
            Track::Timeline(keys) => out.extend(keys.into_iter().map(|k| k.play)),
            Track::Vfx(keys) => out.extend(keys.into_iter().map(|k| k.effect)),
            Track::Video(v) => out.extend(v.keys.into_iter().map(|k| k.video)),
            Track::Scene(keys) => out.extend(keys.into_iter().map(|k| k.scene)),
            _ => {}
        }
    }
    out.retain(|id| !id.is_empty());
    out
}

/// The stamp's dependencies: each caption file with its hash, and every
/// timeline it nests, to any depth (a cycle made by editing a child is
/// found by cooking the parent again).
pub(crate) fn dependencies(s: &Source) -> String {
    let root = assets_root(s);
    let mut nested = Vec::new();
    let mut todo = nested_files(&String::from_utf8_lossy(&s.bytes));
    while let Some(f) = todo.pop() {
        if nested.contains(&f) || nested.len() > 64 {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(root.join(&f)) {
            todo.extend(nested_files(&text));
        }
        nested.push(f);
    }
    caption_files(&s.rel, &root, &String::from_utf8_lossy(&s.bytes))
        .into_iter()
        .chain(nested)
        .map(|f| {
            let h = std::fs::read(root.join(&f))
                .map(|b| hash_bytes(&b))
                .unwrap_or(0);
            format!("{f}={h:016x}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A timeline source parsed, its caption files (and their translations,
/// [`with_translations`]) read beside it or from `assets/`.
pub(crate) fn parse(s: &Source) -> Result<Timeline, CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("a timeline is text (UTF-8)".into()))?;
    let root = assets_root(s);
    let text = &with_translations(&s.rel, &root, text);
    let mut read = |file: &str| -> Result<String, String> {
        let rel = resolve(&s.rel, &root, file);
        std::fs::read_to_string(root.join(&rel))
            .map_err(|e| format!("{rel} (beside the timeline or under assets/): {e}"))
    };
    Timeline::parse(text, &mut read).map_err(CookError)
}

pub(crate) fn cook_timeline(
    s: &Source,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let t = parse(s)?;
    let root = assets_root(s);
    // Every sound, model, timeline, video and scene it names is a cooked asset.
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
            Track::Fields(f) => check_fields(f).map_err(CookError)?,
            Track::Timeline(keys) => keys.iter().for_each(|k| need(&k.play, "timeline")),
            Track::Vfx(keys) => keys.iter().for_each(|k| need(&k.effect, "vfx")),
            Track::Video(v) => v.keys.iter().for_each(|k| need(&k.video, "video")),
            Track::Scene(keys) => keys
                .iter()
                .filter(|k| !k.scene.is_empty())
                .for_each(|k| need(&k.scene, "scene")),
            _ => {}
        }
    }
    check_nesting(&root, &t, &mut vec![s.rel.clone()]).map_err(CookError)?;
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

/// The engine records a field track may write, by its kind.
fn records_of(kind: FieldsKind) -> &'static [&'static str] {
    match kind {
        FieldsKind::Light => &[
            "og.DirectionalLight",
            "og.PointLight",
            "og.SpotLight",
            "og.RectLight",
            "og.Light2D",
        ],
        FieldsKind::Post => &["og.PostEffects"],
        FieldsKind::Blendshape => &["og.Blendshapes"],
        _ => &[],
    }
}

/// Whether `value` fits a field of `column` (with its enum variants):
/// `Ok(())`, else why not.
fn fits(column: &str, variants: &[&str], value: &FieldValue) -> Result<(), String> {
    let lanes = match column {
        "f32" | "f64" | "u32" | "i32" => 1,
        "Vector2" => 2,
        "Vector3" | "Color" => 3,
        "Quaternion" => 4,
        "bool" => {
            return match value {
                FieldValue::Word(w) if w == "true" || w == "false" => Ok(()),
                _ => Err("takes true or false".into()),
            };
        }
        _ if !variants.is_empty() => {
            return match value {
                FieldValue::Word(w) if variants.contains(&w.as_str()) => Ok(()),
                _ => Err(format!("takes one of {}", variants.join(", "))),
            };
        }
        other => {
            return Err(format!(
                "is a {other}, which a timeline cannot ease or step (it eases numbers, vectors, colours and quaternions and steps bools, integers and enums)"
            ));
        }
    };
    match value {
        FieldValue::Numbers(n) if n.len() == lanes => Ok(()),
        _ => Err(format!(
            "is a {column}: {lanes} number{}",
            if lanes == 1 { "" } else { "s" }
        )),
    }
}

/// A field track's fields against the engine's records: an engine
/// record's field it lacks, a field a timeline cannot drive, or a value of
/// the wrong shape is refused naming the field. A game's own component
/// (not `og.`) is checked when it plays.
fn check_fields(f: &FieldsTrack) -> Result<(), String> {
    let kind = f.kind.name();
    let find = |schema: &str| og_records::RECORDS.iter().find(|r| r.schema == schema);
    for k in &f.keys {
        for (name, value) in &k.values {
            let at = format!("the {kind} track's key at {} s: '{name}'", k.time);
            match f.kind {
                FieldsKind::Property => {
                    // The longest engine record the path starts with.
                    let found = name
                        .rmatch_indices('.')
                        .map(|(i, _)| (&name[..i], &name[i + 1..]))
                        .find_map(|(r, field)| find(r).map(|rec| (rec, field)));
                    let Some((rec, field)) = found else {
                        if name.starts_with("og.") {
                            return Err(format!("{at}: no engine record of that name"));
                        }
                        continue;
                    };
                    let want = if f.entity.is_empty() {
                        "resource"
                    } else {
                        "component"
                    };
                    if rec.kind != want {
                        return Err(format!(
                            "{at}: {} is a {}, so the track {}",
                            rec.schema,
                            rec.kind,
                            if rec.kind == "resource" {
                                "names no entity"
                            } else {
                                "names its entity: `track property \"crate\" {`"
                            }
                        ));
                    }
                    let decl = rec.fields.iter().find(|d| d.name == field).ok_or_else(|| {
                        format!(
                            "{at}: {} has no field '{field}' (its fields: {})",
                            rec.schema,
                            rec.fields
                                .iter()
                                .map(|d| d.name)
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    })?;
                    if decl.readonly {
                        return Err(format!("{at} is read only"));
                    }
                    fits(decl.column, decl.variants, value).map_err(|e| format!("{at} {e}"))?;
                }
                FieldsKind::Light | FieldsKind::Post | FieldsKind::Blendshape => {
                    let recs: Vec<_> = records_of(f.kind).iter().filter_map(|r| find(r)).collect();
                    let decls: Vec<_> = recs
                        .iter()
                        .filter_map(|r| r.fields.iter().find(|d| d.name == name))
                        .collect();
                    if decls.is_empty() {
                        let mut names: Vec<&str> = recs
                            .iter()
                            .flat_map(|r| r.fields.iter())
                            .filter(|d| !d.readonly)
                            .map(|d| d.name)
                            .collect();
                        names.sort_unstable();
                        names.dedup();
                        return Err(format!(
                            "{at}: no {} field of that name (its fields: {})",
                            records_of(f.kind).join(", "),
                            names.join(", ")
                        ));
                    }
                    if let Some(d) = decls.iter().find(|d| d.readonly) {
                        return Err(format!("{at} is read only ({})", d.name));
                    }
                    if decls
                        .iter()
                        .all(|d| fits(d.column, d.variants, value).is_err())
                    {
                        let e = fits(decls[0].column, decls[0].variants, value).unwrap_err();
                        return Err(format!("{at} {e}"));
                    }
                }
                FieldsKind::Visibility | FieldsKind::Material => {}
            }
        }
    }
    Ok(())
}

/// Refuses a timeline that nests itself (at any depth) or nests deeper
/// than [`NEST_DEPTH`], naming the chain. `chain` holds the asset ids from
/// the cooked timeline down to `t`.
fn check_nesting(root: &Path, t: &Timeline, chain: &mut Vec<String>) -> Result<(), String> {
    for tr in &t.tracks {
        let Track::Timeline(keys) = tr else { continue };
        for k in keys {
            let id = k.play.clone();
            if chain.contains(&id) || chain.first().is_some_and(|c| asset_id(c) == asset_id(&id)) {
                chain.push(id);
                return Err(format!(
                    "the timelines nest in a cycle: {}",
                    chain.join(" → ")
                ));
            }
            if chain.len() > NEST_DEPTH {
                chain.push(id);
                return Err(format!(
                    "timelines nest at most {NEST_DEPTH} deep: {}",
                    chain.join(" → ")
                ));
            }
            let Ok(text) = std::fs::read_to_string(root.join(&id)) else {
                continue;
            };
            let mut read = |file: &str| -> Result<String, String> {
                let rel = resolve(&id, root, file);
                std::fs::read_to_string(root.join(&rel)).map_err(|e| format!("{rel}: {e}"))
            };
            let Ok(child) = Timeline::parse(&text, &mut read) else {
                continue;
            };
            chain.push(id);
            check_nesting(root, &child, chain)?;
            chain.pop();
        }
    }
    Ok(())
}

/// An asset path without the `assets/` a source's rel may carry.
fn asset_id(p: &str) -> &str {
    p.strip_prefix("assets/").unwrap_or(p)
}

pub(crate) static TIMELINE: KindDef = KindDef {
    name: "timeline",
    extensions: &[],
    types: &[TIMELINE_TYPE],
    by_content: false,
    streams: false,
    plan: plan_timeline,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&TIMELINE];

fn plan_timeline<'a>(
    _cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let PlanState { parsed_refs, .. } = state;
    // The stamp covers every caption file it imports.
    let deps = dependencies(s);
    assets::record_dependencies(parsed_refs, &s.rel, &deps);
    // What its tracks name is what it refers to: a timeline needs the
    // engine modules its videos and scenes need.
    parsed_refs
        .entry(s.rel.clone())
        .or_default()
        .extend(named_assets(s));
    let st = stamp(&[b"timeline 1", &hash, deps.as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::Timeline::NAME,
        st,
        Box::new(TimelineJob(s)),
    ));
}

/// A timeline's one output.
struct TimelineJob<'a>(&'a Source);

impl Job for TimelineJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_timeline(self.0, &|id| ex.known.contains(id), p.stamp)
    }
}
