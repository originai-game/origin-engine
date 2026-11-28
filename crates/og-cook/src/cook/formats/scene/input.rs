//! A checked scene snapshot shared by flattening, packing, HLOD and streaming.
//! References are bound once, before consumers run. This is native data, not an
//! OGT AST or a second author parser.
use crate::cook::{source_content, source_schemas};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub use source_schemas::world::scene::{EntityInput, PrefabEdit, PrefabInput, SceneInput};

/// A checked declaration's original TypeScript expression, never a runtime override.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SourceSite {
    pub file: String,
    pub path: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Document {
    pub scene: SceneInput,
    #[serde(skip)]
    pub records: std::sync::Arc<BTreeMap<String, crate::cook::source_records::PackedRecord>>,
    pub base: Option<String>,
    pub prefab: bool,
    pub file: String,
    pub lines: BTreeMap<String, usize>,
    pub sites: BTreeMap<String, SourceSite>,
}

impl Document {
    pub fn scene(scene: SceneInput) -> Self {
        Self {
            scene,
            records: Default::default(),
            base: None,
            prefab: false,
            file: String::new(),
            lines: BTreeMap::new(),
            sites: BTreeMap::new(),
        }
    }

    pub fn prefab(input: PrefabInput) -> Self {
        let mut doc = Self::scene(SceneInput {
            name: None,
            entities: vec![input.entity],
            resources: BTreeMap::new(),
            streams: BTreeMap::new(),
            hlod: None,
            paths: BTreeMap::new(),
        });
        doc.base = input.base.map(|r| r.id);
        doc.prefab = true;
        doc
    }

    pub fn line(&self, path: &str) -> usize {
        self.lines.get(path).copied().unwrap_or(1)
    }

    /// Captured declaration locations, including imported/computed entities
    /// whose nearest writable source is the entry's expression.
    pub(crate) fn locate(&mut self, content: &source_content::Content) {
        self.file = content.entry().file.clone();
        fn walk(paths: &mut Vec<String>, e: &EntityInput, path: String) {
            fields(paths, &e.components, &path);
            for (i, edit) in e.edits.iter().enumerate() {
                if let PrefabEdit::Override { components, .. } = edit {
                    fields(paths, components, &format!("{path}/edits/{i}"));
                }
            }
            for (i, child) in e.children.iter().enumerate() {
                walk(paths, child, format!("{path}/children/{i}"));
            }
            paths.push(path);
        }
        fn fields(
            paths: &mut Vec<String>,
            components: &BTreeMap<String, Map<String, Value>>,
            path: &str,
        ) {
            let escape = |s: &str| s.replace('~', "~0").replace('/', "~1");
            for (component, values) in components {
                for field in values.keys() {
                    paths.push(format!(
                        "{path}/components/{}/{}",
                        escape(component),
                        escape(field)
                    ));
                }
            }
        }
        let mut paths = vec![String::new()];
        for (i, e) in self.scene.entities.iter().enumerate() {
            let path = if self.prefab {
                "/entity".into()
            } else {
                format!("/entities/{i}")
            };
            walk(&mut paths, e, path);
        }
        if let Ok(locations) = content
            .source()
            .locations(&paths.iter().map(String::as_str).collect::<Vec<_>>())
        {
            self.lines = paths
                .iter()
                .cloned()
                .zip(locations.iter().map(|l| l.line))
                .collect();
            self.sites = paths
                .into_iter()
                .zip(locations)
                .map(|(path, l)| {
                    (
                        path.clone(),
                        SourceSite {
                            file: self.file.clone(),
                            path,
                            line: l.line,
                            column: l.column,
                        },
                    )
                })
                .collect();
        }
    }

    pub fn site(&self, path: &str) -> Option<SourceSite> {
        self.sites.get(path).cloned()
    }

    pub fn field_site(&self, path: &str, component: &str, field: &str) -> Option<SourceSite> {
        let escape = |s: &str| s.replace('~', "~0").replace('/', "~1");
        self.site(&format!(
            "{path}/components/{}/{}",
            escape(component),
            escape(field)
        ))
    }

    pub(crate) fn bind(&mut self, graph: &source_content::Graph) {
        fn value(v: &mut Value, graph: &source_content::Graph) {
            match v {
                Value::Object(m) if m.len() == 1 && m.contains_key("$ogRef") => {
                    let id = m["$ogRef"].as_str().expect("checked ref");
                    m.insert(
                        "$ogRef".into(),
                        Value::String(graph.outputs[id].asset.clone()),
                    );
                }
                Value::Object(m) => m.values_mut().for_each(|v| value(v, graph)),
                Value::Array(a) => a.iter_mut().for_each(|v| value(v, graph)),
                _ => {}
            }
        }
        fn fields(m: &mut BTreeMap<String, Map<String, Value>>, graph: &source_content::Graph) {
            for values in m.values_mut() {
                for v in values.values_mut() {
                    value(v, graph);
                }
            }
        }
        fn entity(e: &mut EntityInput, graph: &source_content::Graph) {
            for r in [&mut e.model, &mut e.scene, &mut e.prefab]
                .into_iter()
                .flatten()
            {
                r.id = graph.outputs[&r.id].asset.clone();
            }
            fields(&mut e.components, graph);
            for edit in &mut e.edits {
                if let PrefabEdit::Override {
                    model, components, ..
                } = edit
                {
                    if let Some(r) = model {
                        r.id = graph.outputs[&r.id].asset.clone();
                    }
                    fields(components, graph);
                }
            }
            e.children.iter_mut().for_each(|e| entity(e, graph));
        }
        if let Some(base) = &mut self.base {
            *base = graph.outputs[base.as_str()].asset.clone();
        }
        self.scene
            .entities
            .iter_mut()
            .for_each(|e| entity(e, graph));
        fields(&mut self.scene.resources, graph);
        for stream in self.scene.streams.values_mut() {
            for r in &mut stream.assets {
                r.id = graph.outputs[&r.id].asset.clone();
            }
        }
    }

    pub fn references(&self) -> Vec<String> {
        fn value(v: &Value, out: &mut BTreeSet<String>) {
            match v {
                Value::Object(m) => {
                    if let Some(id) = m.get("$ogRef").and_then(Value::as_str) {
                        out.insert(id.into());
                    } else {
                        m.values().for_each(|v| value(v, out));
                    }
                }
                Value::Array(a) => a.iter().for_each(|v| value(v, out)),
                _ => {}
            }
        }
        fn fields(records: &BTreeMap<String, Map<String, Value>>, out: &mut BTreeSet<String>) {
            records
                .values()
                .flat_map(|r| r.values())
                .for_each(|v| value(v, out));
        }
        fn refs(e: &EntityInput, out: &mut BTreeSet<String>) {
            out.extend(
                [&e.model, &e.scene]
                    .into_iter()
                    .flatten()
                    .map(|r| r.id.clone()),
            );
            fields(&e.components, out);
            for edit in &e.edits {
                if let PrefabEdit::Override {
                    model, components, ..
                } = edit
                {
                    out.extend(model.iter().map(|r| r.id.clone()));
                    fields(components, out);
                }
            }
            e.children.iter().for_each(|e| refs(e, out));
        }
        let mut out = BTreeSet::new();
        fields(&self.scene.resources, &mut out);
        self.scene.entities.iter().for_each(|e| refs(e, &mut out));
        out.into_iter().collect()
    }

    pub fn placed(&self) -> Vec<String> {
        fn refs(e: &EntityInput, out: &mut BTreeSet<String>) {
            out.extend(e.prefab.iter().map(|r| r.id.clone()));
            e.children.iter().for_each(|e| refs(e, out));
        }
        let mut out: BTreeSet<String> = self.base.iter().cloned().collect();
        self.scene.entities.iter().for_each(|e| refs(e, &mut out));
        out.into_iter().collect()
    }

    pub fn direct_subscenes(&self) -> Vec<String> {
        fn refs(e: &EntityInput, out: &mut BTreeSet<String>) {
            out.extend(e.scene.iter().map(|r| r.id.clone()));
            e.children.iter().for_each(|e| refs(e, out));
        }
        let mut out = BTreeSet::new();
        self.scene.entities.iter().for_each(|e| refs(e, &mut out));
        out.into_iter().collect()
    }
}
