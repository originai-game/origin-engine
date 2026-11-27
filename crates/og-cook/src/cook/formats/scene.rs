//! Checked scene trees become packed component/resource rows. Prefab expansion,
//! streaming, paths and HLOD consume the same captured native document.
use super::{
    CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, hash_bytes, prefab, stamp,
};
use crate::format::{self, *};
use og_blob::BlobWriter;
use og_core::Schema;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod input;
mod scatter;
pub use input::Document;
pub(crate) use scatter::expand;

/// Direct placements plus those inherited from every reached prefab.
pub fn subscenes(doc: &Document, documents: &BTreeMap<String, Arc<Document>>) -> Vec<String> {
    let mut out: BTreeSet<String> = doc.direct_subscenes().into_iter().collect();
    for id in prefab::reached(doc, documents) {
        if let Some(doc) = documents.get(&id) {
            out.extend(doc.direct_subscenes());
        }
    }
    out.into_iter().collect()
}

fn subscene_cycle(
    source: &str,
    placed: &str,
    documents: &BTreeMap<String, Arc<Document>>,
) -> Option<Vec<String>> {
    fn go(
        at: &str,
        target: &str,
        documents: &BTreeMap<String, Arc<Document>>,
        path: &mut Vec<String>,
        seen: &mut BTreeSet<String>,
    ) -> bool {
        path.push(at.into());
        if at == target {
            return true;
        }
        if seen.insert(at.into())
            && let Some(doc) = documents.get(at)
        {
            for next in subscenes(doc, documents) {
                if go(&next, target, documents, path, seen) {
                    return true;
                }
            }
        }
        path.pop();
        false
    }
    let mut path = vec![source.into()];
    go(placed, source, documents, &mut path, &mut BTreeSet::new()).then_some(path)
}

pub struct Prefabs<'a> {
    pub documents: &'a BTreeMap<String, Arc<Document>>,
    pub publish: bool,
}

pub fn cook(
    doc: &Document,
    source: &str,
    schemas: &BTreeMap<String, Schema>,
    known: &dyn Fn(&str) -> bool,
    prefabs: &Prefabs,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let mut flattener = prefab::Flattener::new(prefabs.documents, schemas);
    let flat = if doc.prefab {
        vec![flattener.prefab(source)?]
    } else {
        flattener.scene(source, doc)?
    };
    let mut notes = Vec::new();
    for orphan in flattener.orphans.iter().filter(|o| o.source == source) {
        if prefabs.publish {
            return Err(CookError(format!(
                "a published game never ships an override that does nothing: {orphan}"
            )));
        }
        notes.push(format!("warning: {orphan}"));
    }
    let mut strings = Vec::new();
    let mut add = |s: &str| {
        let at = strings.len() as u32;
        strings.extend_from_slice(s.as_bytes());
        (at, s.len() as u32)
    };
    let name = if doc.prefab {
        flat[0].name.as_str()
    } else {
        doc.scene.name.as_deref().unwrap_or_default()
    };
    let (name_offset, name_len) = add(name);
    let mut entities = Vec::new();
    let mut lines = Vec::new();
    let mut origins = Vec::new();
    let mut overrides = BTreeMap::new();
    let mut refs: Vec<String> = Vec::new();
    let mut subscene_ids: Vec<String> = Vec::new();
    let mut subscene_rows = Vec::new();
    let mut prefab_ids: Vec<String> = Vec::new();
    let mut prefab_rows = Vec::new();
    let mut instance_roots = BTreeMap::new();
    let mut component_data: BTreeMap<String, Vec<(u32, Vec<u8>)>> = BTreeMap::new();
    // Sparse typed relocations: entity -> record -> field -> content token.
    let mut bindings: BTreeMap<u32, BTreeMap<String, Map<String, Value>>> = BTreeMap::new();
    let mut resource_bindings: BTreeMap<String, Map<String, Value>> = BTreeMap::new();
    let mut stack: Vec<_> = flat.iter().rev().map(|n| (n, NONE)).collect();
    while let Some((node, parent)) = stack.pop() {
        let index = entities.len() as u32;
        origins.push(node.file.clone());
        if node.link.is_some() {
            let fields: Vec<_> = node
                .origins
                .iter()
                .filter(|(_, owner)| owner.as_str() == source)
                .map(|((component, field), _)| format!("{component}.{field}"))
                .collect();
            if !fields.is_empty() {
                overrides.insert(index, fields);
            }
        }
        let (name_offset, name_len) = add(&node.name);
        let mut instance = NONE;
        if let Some(model) = &node.model {
            if !known(model) {
                return Err(CookError(format!(
                    "{}:{}: model '{model}' is not an asset of this project",
                    doc.file, node.line
                )));
            }
            instance = refs.iter().position(|r| r == model).unwrap_or_else(|| {
                refs.push(model.clone());
                refs.len() - 1
            }) as u32;
        }
        if let Some(sub) = &node.subscene {
            if !prefabs.documents.get(sub).is_some_and(|d| !d.prefab) {
                return Err(CookError(format!(
                    "{}:{}: '{sub}' is not a checked scene",
                    doc.file, node.line
                )));
            }
            if let Some(chain) = subscene_cycle(source, sub, prefabs.documents) {
                return Err(CookError(format!(
                    "{}:{}: sub-scene places this scene again: {}",
                    doc.file,
                    node.line,
                    chain.join(" -> ")
                )));
            }
            let scene = subscene_ids
                .iter()
                .position(|r| r == sub)
                .unwrap_or_else(|| {
                    subscene_ids.push(sub.clone());
                    subscene_ids.len() - 1
                }) as u32;
            subscene_rows.push(SubsceneRow {
                entity: index,
                scene,
            });
        }
        let mut components = node.components.clone();
        if let Some(flag) = node.static_model {
            if instance == NONE {
                return Err(CookError(format!(
                    "{}:{}: static marks a model for baked lighting: the entity has no model",
                    doc.file, node.line
                )));
            }
            let model = components.entry("og.Model".into()).or_default();
            if model.insert("static".into(), Value::Bool(flag)).is_some() {
                return Err(CookError(format!(
                    "{}:{}: og.Model.static is set twice",
                    doc.file, node.line
                )));
            }
        }
        for (name, patch) in &components {
            let record = doc.records.get(name).ok_or_else(|| {
                CookError(format!(
                    "{}:{}: no declared record {name}",
                    doc.file, node.line
                ))
            })?;
            let (bytes, relocations) = record
                .pack(patch)
                .map_err(|e| CookError(format!("{}:{}: {e}", doc.file, node.line)))?;
            if !relocations.is_empty() {
                bindings
                    .entry(index)
                    .or_default()
                    .insert(name.clone(), relocations);
            }
            component_data
                .entry(name.clone())
                .or_default()
                .push((index, bytes));
        }
        entities.push(EntityRow {
            name_offset,
            name_len,
            parent,
            instance,
        });
        lines.push(SceneLineRow {
            line: node.line as u32,
        });
        let origin = match &node.link {
            None => PrefabRow {
                prefab: NONE,
                node: NONE,
                root: NONE,
            },
            Some(link) => {
                let root = *instance_roots.entry(link.instance).or_insert(index);
                let prefab = prefab_ids
                    .iter()
                    .position(|r| r == &link.prefab)
                    .unwrap_or_else(|| {
                        prefab_ids.push(link.prefab.clone());
                        prefab_ids.len() - 1
                    }) as u32;
                PrefabRow {
                    prefab,
                    node: link.node,
                    root,
                }
            }
        };
        prefab_rows.push(origin);
        stack.extend(node.children.iter().rev().map(|n| (n, index)));
    }
    let mut components = Vec::new();
    let mut payloads = Vec::new();
    for (name, rows) in component_data {
        let schema = &doc.records[&name].schema;
        let hash = schema.layout_hash();
        let size = og_core::Layout::of(schema).size();
        let (name_offset, name_len) = add(&name);
        components.push(ComponentRow {
            name_offset,
            name_len,
            layout_hash_lo: hash as u32,
            layout_hash_hi: (hash >> 32) as u32,
            rows: rows.len() as u32,
            inferred: 0,
        });
        let mut bytes = Vec::with_capacity(rows.len() * (4 + size));
        for (entity, row) in rows {
            bytes.extend_from_slice(&entity.to_le_bytes());
            bytes.extend_from_slice(&row);
        }
        payloads.push((name, bytes, (4 + size) as u32, hash, schema.to_json()));
    }
    let mut resources = Vec::new();
    let mut resource_payloads = Vec::new();
    for (name, patch) in &doc.scene.resources {
        let record = &doc.records[name];
        let (packed, relocations) = record.pack(patch).map_err(CookError)?;
        if !relocations.is_empty() {
            resource_bindings.insert(name.clone(), relocations);
        }
        let layout = og_core::Layout::of(&record.schema);
        let mut schema = record.schema.clone();
        let mut bytes = Vec::new();
        for (i, field) in record.schema.fields.iter().enumerate() {
            if patch.contains_key(&field.name) {
                bytes.extend_from_slice(&packed[layout.range(i)]);
            }
        }
        schema.fields.retain(|f| patch.contains_key(&f.name));
        let hash = schema.layout_hash();
        let (name_offset, name_len) = add(name);
        resources.push(ComponentRow {
            name_offset,
            name_len,
            layout_hash_lo: hash as u32,
            layout_hash_hi: (hash >> 32) as u32,
            rows: 1,
            inferred: 0,
        });
        resource_payloads.push((
            name.clone(),
            bytes,
            og_core::Layout::of(&schema).size() as u32,
            hash,
            schema.to_json(),
        ));
    }
    let mut ref_rows = |ids: &[String]| {
        ids.iter()
            .map(|id| {
                let (id_offset, id_len) = add(id);
                RefRow { id_offset, id_len }
            })
            .collect::<Vec<_>>()
    };
    let models = ref_rows(&refs);
    let subs = ref_rows(&subscene_ids);
    let pref = ref_rows(&prefab_ids);
    let meta = SceneMeta {
        entity_count: entities.len() as u32,
        component_count: components.len() as u32,
        ref_count: models.len() as u32,
        name_offset,
        name_len,
    };
    let mut writer = BlobWriter::new(
        format::Scene::type_hash(),
        format::Scene::schema_version(),
        stamp,
    );
    writer
        .table("meta", &[meta], row_hash::<SceneMeta>())
        .table("entities", &entities, row_hash::<EntityRow>())
        .table("refs", &models, row_hash::<RefRow>())
        .table("components", &components, row_hash::<ComponentRow>())
        .raw("strings", strings, 1);
    for (name, bytes, stride, hash, json) in payloads {
        writer.rows(&format!("c.{name}"), bytes, stride, hash).raw(
            &format!("s.{name}"),
            json.into_bytes(),
            1,
        );
    }
    if !prefab_ids.is_empty() {
        writer
            .table("prefabs", &prefab_rows, row_hash::<PrefabRow>())
            .table("prefab_refs", &pref, row_hash::<RefRow>());
    }
    if !subscene_rows.is_empty() {
        writer
            .table("subscenes", &subscene_rows, row_hash::<SubsceneRow>())
            .table("subscene_refs", &subs, row_hash::<RefRow>());
    }
    if !prefabs.publish {
        writer.table("lines", &lines, row_hash::<SceneLineRow>());
        writer.raw(
            "origins",
            serde_json::to_vec(&(origins, overrides)).map_err(|e| CookError(e.to_string()))?,
            1,
        );
    }
    if !resources.is_empty() {
        writer.table("resources", &resources, row_hash::<ComponentRow>());
        for (name, bytes, stride, hash, json) in resource_payloads {
            writer
                .rows(&format!("r.{name}"), bytes, stride.max(1), hash)
                .raw(&format!("rs.{name}"), json.into_bytes(), 1);
        }
    }
    writer.raw(
        "bindings",
        serde_json::to_vec(&(bindings, resource_bindings)).map_err(|e| CookError(e.to_string()))?,
        1,
    );
    let mut paths = Vec::new();
    for (name, input) in &doc.scene.paths {
        use crate::cook::source_schemas::media::timeline::PathKind as Input;
        let path = og_timeline::PathDesc {
            points: input
                .points
                .iter()
                .copied()
                .map(og_core::glam::Vec3::from_array)
                .collect(),
            closed: input.closed.unwrap_or(false),
            tension: input.tension.unwrap_or(0.5),
            kind: match input.kind.as_ref().unwrap_or(&Input::CatmullRom) {
                Input::CatmullRom => og_timeline::PathKind::CatmullRom,
                Input::Bezier => og_timeline::PathKind::Bezier,
                Input::Linear => og_timeline::PathKind::Linear,
            },
        };
        if name.is_empty() {
            return Err(CookError("a path needs a nonempty name".into()));
        }
        og_timeline::Path::new(path.clone()).map_err(CookError)?;
        paths.push((name.clone(), path));
    }
    if !paths.is_empty() {
        writer.raw("paths", og_timeline::paths_to_text(&paths).into_bytes(), 1);
    }
    notes.push(format!(
        "{} entities, {} component table(s), {} model reference(s)",
        entities.len(),
        components.len(),
        refs.len()
    ));
    Ok((writer.finish(), notes))
}

pub(crate) static SCENE: KindDef = KindDef {
    name: "scene",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan: plan_scene,
};
pub(crate) static PREFAB: KindDef = KindDef {
    name: "prefab",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan: plan_scene,
};
pub(crate) static KINDS: &[&KindDef] = &[&SCENE, &PREFAB];

fn plan_scene<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Some(doc) = &s.scene else {
        state.early_errors.push((
            s.rel.clone(),
            "scene",
            "missing checked scene document".into(),
        ));
        return;
    };
    let reached = prefab::reached(doc, cx.prefab_texts);
    let mut references: BTreeSet<String> = doc.references().into_iter().collect();
    let mut placed = Vec::new();
    for id in &reached {
        let other = cx.prefab_texts.get(id);
        if let Some(other) = other {
            references.extend(other.references());
        }
        placed.push(format!(
            "{id}#{}",
            other.map_or(0, |d| hash_bytes(
                &serde_json::to_vec(d.as_ref()).expect("native scene")
            ))
        ));
    }
    state
        .parsed_refs
        .entry(s.rel.clone())
        .or_default()
        .extend(references.iter().chain(reached.iter()).cloned());
    let references = references
        .into_iter()
        .map(|id| format!("{id}={}", cx.known.contains(&id)))
        .collect::<Vec<_>>()
        .join("\n");
    let records = serde_json::to_vec(doc.records.as_ref()).expect("record metadata");
    // Scatter/district snapshots include derived placements. Their author IR
    // alone is unchanged when a referenced terrain surface changes.
    let document = serde_json::to_vec(doc.as_ref()).expect("native scene");
    let st = stamp(
        &[Area::Scene],
        &[
            b"native-scene",
            &hash,
            &cx.schema_hash,
            &records,
            &document,
            references.as_bytes(),
            placed.join("\n").as_bytes(),
            &[cx.options.publish as u8],
        ],
    );
    outs.push((
        s.rel.clone(),
        format::Scene::NAME,
        st,
        Box::new(SceneJob(s)),
    ));
    if s.kind == &SCENE {
        super::hlod::plan(cx, state, s, doc, outs);
    }
}

struct SceneJob<'a>(&'a Source);
impl Job for SceneJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let source = self.0;
        let doc = source
            .scene
            .as_ref()
            .ok_or_else(|| CookError("missing checked scene".into()))?;
        cook(
            doc,
            &source.rel,
            ex.schemas,
            &|id| ex.known.contains(id),
            &Prefabs {
                documents: ex.prefab_texts,
                publish: ex.options.publish,
            },
            p.stamp,
        )
    }
}
