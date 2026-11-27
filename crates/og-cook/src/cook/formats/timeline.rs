//! Checked TS timelines become native tracks. Text exists only in the internal
//! blob codec shared with the runtime; author OGT and path fallbacks are gone.
use super::{
    CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, hash_bytes, stamp,
};
use crate::cook::source_content::Graph;
use crate::format::{self, Area, BlobKind};
use og_blob::BlobWriter;
use og_timeline::{FieldValue, FieldsKind, FieldsTrack, NEST_DEPTH, Timeline, Track};
use std::collections::BTreeMap;
use std::path::Path;

mod input;
pub(crate) use input::Prepared;

pub(crate) fn parse(s: &Source) -> Result<Timeline, CookError> {
    s.timeline
        .as_ref()
        .map(|p| p.timeline.clone())
        .ok_or_else(|| CookError(format!("{}: not a prepared TS timeline", s.rel)))
}

pub(crate) fn refs(t: &Timeline) -> Vec<String> {
    let mut out = Vec::new();
    for tr in &t.tracks {
        match tr {
            Track::Audio(keys) => out.extend(keys.iter().map(|k| k.sound.clone())),
            Track::Music(keys) => out.extend(keys.iter().map(|k| k.sound.clone())),
            Track::Animation(a) => out.push(a.model.clone()),
            Track::Timeline(keys) => out.extend(keys.iter().map(|k| k.play.clone())),
            Track::Vfx(keys) => out.extend(keys.iter().map(|k| k.effect.clone())),
            Track::Video(v) => out.extend(v.keys.iter().map(|k| k.video.clone())),
            Track::Scene(keys) => out.extend(keys.iter().map(|k| k.scene.clone())),
            _ => {}
        }
    }
    out.retain(|id| !id.is_empty());
    out
}

/// Resolve checked logical references to their producer's one actual output.
/// The transitional scene catalogue disappears with the scene cutover.
pub(crate) fn bind(sources: &mut [Source], graph: &Graph) -> Result<(), CookError> {
    for s in sources.iter_mut() {
        let Some(p) = &mut s.timeline else { continue };
        let resolve = |id: &mut String| {
            if let Some(output) = graph.outputs.get(id) {
                *id = output.asset.clone();
            }
        };
        for track in &mut p.timeline.tracks {
            match track {
                Track::Audio(keys) => keys.iter_mut().for_each(|k| resolve(&mut k.sound)),
                Track::Music(keys) => keys.iter_mut().for_each(|k| resolve(&mut k.sound)),
                Track::Animation(t) => resolve(&mut t.model),
                Track::Timeline(keys) => keys.iter_mut().for_each(|k| resolve(&mut k.play)),
                Track::Vfx(keys) => keys.iter_mut().for_each(|k| resolve(&mut k.effect)),
                Track::Video(t) => t.keys.iter_mut().for_each(|k| resolve(&mut k.video)),
                Track::Scene(keys) => keys.iter_mut().for_each(|k| resolve(&mut k.scene)),
                _ => {}
            }
        }
    }
    let timelines: BTreeMap<_, _> = sources
        .iter()
        .filter_map(|s| s.timeline.as_ref().map(|p| (s.rel.as_str(), &p.timeline)))
        .collect();
    fn visit(
        id: &str,
        timelines: &BTreeMap<&str, &Timeline>,
        chain: &mut Vec<String>,
        depths: &mut BTreeMap<String, usize>,
    ) -> Result<usize, CookError> {
        if chain.iter().any(|i| i == id) {
            chain.push(id.into());
            return Err(CookError(format!(
                "timelines nest in a cycle: {}",
                chain.join(" → ")
            )));
        }
        if chain.len() + depths.get(id).copied().unwrap_or(0) > NEST_DEPTH {
            return Err(CookError(format!(
                "timelines nest at most {NEST_DEPTH} deep: {} → {id}",
                chain.join(" → ")
            )));
        }
        if let Some(depth) = depths.get(id) {
            return Ok(*depth);
        }
        let t = timelines
            .get(id)
            .ok_or_else(|| CookError(format!("{id}: nested timeline is not prepared")))?;
        chain.push(id.into());
        let mut depth = 0;
        for track in &t.tracks {
            if let Track::Timeline(keys) = track {
                for k in keys {
                    depth = depth.max(1 + visit(&k.play, timelines, chain, depths)?);
                }
            }
        }
        chain.pop();
        depths.insert(id.into(), depth);
        Ok(depth)
    }
    let mut depths = BTreeMap::new();
    for id in timelines.keys() {
        visit(id, &timelines, &mut Vec::new(), &mut depths)?;
    }
    Ok(())
}

/// Source inspection uses the exact same catalogue, evaluator and binder as cook.
pub fn read(root: &Path) -> Result<BTreeMap<String, Timeline>, String> {
    let project = super::super::stage::Project::read(root).map_err(|e| e.to_string())?;
    let mut sources = super::super::pipeline::scan::scan(&root.join("assets"))
        .map_err(|e| e.to_string())?
        .sources;
    let mut consumed = Default::default();
    let (mut rig_content, rigs) =
        super::super::pipeline::author::rigs(root, &project.source).map_err(|e| e.to_string())?;
    super::super::imports::prepare(root, &project.source, &mut sources, &mut rig_content)
        .map_err(|e| e.to_string())?;
    super::super::pipeline::author::models(root, &project.source, &mut sources, &mut rig_content)
        .map_err(|e| e.to_string())?;
    let bindings = super::super::pipeline::author::bindings(root, &project.source, &sources)
        .map_err(|e| e.to_string())?;
    super::super::stage::import(&mut sources, &mut consumed, None, 1, &rigs, &bindings);
    super::super::pipeline::author::sources(
        root,
        &project.source,
        &project.story_functions,
        &mut sources,
        &mut consumed,
        &project.provided,
        rig_content,
    )
    .map_err(|e| e.to_string())?;
    Ok(sources
        .into_iter()
        .filter_map(|s| s.timeline.map(|p| (s.rel, p.timeline)))
        .collect())
}

pub(crate) static TIMELINE: KindDef = KindDef {
    name: "timeline",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&TIMELINE];

fn plan<'a>(cx: &Cx<'a>, _: &mut PlanState, s: &'a Source, hash: [u8; 8], outs: &mut Vec<Out<'a>>) {
    let Some(p) = &s.timeline else { return };
    let dependencies: Vec<_> = p.files.values().map(|(path, _)| path.clone()).collect();
    cx.index.note_external(&dependencies);
    let mut deps = String::new();
    for (file, (_, text)) in &p.files {
        deps.push_str(&format!("{file}={:016x}\n", hash_bytes(text.as_bytes())));
    }
    // Native normalized tracks include physical output bindings, not just the
    // logical IR. A changed binding must not reuse yesterday's cached blob.
    let canonical = p.timeline.to_text();
    let st = stamp(
        &[Area::Scene],
        &[
            b"timeline TS 1",
            &hash,
            canonical.as_bytes(),
            deps.as_bytes(),
        ],
    );
    outs.push((
        s.rel.clone(),
        format::Timeline::NAME,
        st,
        Box::new(TimelineJob {
            timeline: &p.timeline,
            canonical,
        }),
    ));
}

struct TimelineJob<'a> {
    timeline: &'a Timeline,
    canonical: String,
}

impl Job for TimelineJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let missing: Vec<_> = refs(self.timeline)
            .into_iter()
            .filter(|id| !ex.known.contains(id))
            .collect();
        if !missing.is_empty() {
            return Err(CookError(format!(
                "timeline names missing cooked assets: {}",
                missing.join(", ")
            )));
        }
        let t = self.timeline;
        let mut w = BlobWriter::new(
            format::Timeline::type_hash(),
            format::Timeline::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::TimelineMeta {
                text_bytes: self.canonical.len() as u32,
                tracks: t.tracks.len() as u32,
                duration: t.duration,
            }],
            format::row_hash::<format::TimelineMeta>(),
        )
        .raw("text", self.canonical.as_bytes().to_vec(), 16);
        Ok((
            w.finish(),
            vec![format!(
                "timeline '{}': {} s, {} track(s), {} path(s)",
                t.name,
                t.duration,
                t.tracks.len(),
                t.paths.len()
            )],
        ))
    }
}

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
                "is a {other}, which a timeline cannot ease or step"
            ));
        }
    };
    match value {
        FieldValue::Numbers(n) if n.len() == lanes => Ok(()),
        _ => Err(format!("is a {column}: {lanes} numbers")),
    }
}

fn check_fields(f: &FieldsTrack) -> Result<(), String> {
    let kind = f.kind.name();
    let find = |schema: &str| og_records::RECORDS.iter().find(|r| r.schema == schema);
    for k in &f.keys {
        for (name, value) in &k.values {
            let at = format!("the {kind} track's key at {} s: '{name}'", k.time);
            match f.kind {
                FieldsKind::Property => {
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
                            "{at}: {} is a {}, but the track targets a {want}",
                            rec.schema, rec.kind
                        ));
                    }
                    let decl =
                        rec.fields.iter().find(|d| d.name == field).ok_or_else(|| {
                            format!("{at}: {} has no field '{field}'", rec.schema)
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
                        return Err(format!(
                            "{at}: no {} field of that name",
                            records_of(f.kind).join(", ")
                        ));
                    }
                    if let Some(d) = decls.iter().find(|d| d.readonly) {
                        return Err(format!("{at} is read only ({})", d.name));
                    }
                    if decls
                        .iter()
                        .all(|d| fits(d.column, d.variants, value).is_err())
                    {
                        return Err(format!(
                            "{at} {}",
                            fits(decls[0].column, decls[0].variants, value).unwrap_err()
                        ));
                    }
                }
                FieldsKind::Visibility | FieldsKind::Material => {}
            }
        }
    }
    Ok(())
}
