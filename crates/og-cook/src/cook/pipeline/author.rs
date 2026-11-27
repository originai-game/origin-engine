//! Project content enters the same source, plan, execute and output stages as
//! file importers. Domain cutovers consume canonical author IR, never OGT text
//! generated from TS. The scaffold catalogue is removed in P1/P2/P3.
use super::*;
use crate::cook::{source_content, source_index, source_project, source_records};

/// Bind project-only canonical refs through actual scanned domain entries.
/// Does not evaluate content or manufacture a second cooked output.
pub fn project_assets(
    root: &Path,
    project: &source_project::Project,
) -> Result<BTreeMap<String, source_content::Output>, CookError> {
    bindings(root, project, &scan::scan(&root.join("assets"))?.sources)
}

pub(crate) fn bindings(
    root: &Path,
    project: &source_project::Project,
    sources: &[Source],
) -> Result<BTreeMap<String, source_content::Output>, CookError> {
    let index = source_index::Index::open(root, project).map_err(|e| CookError(e.to_string()))?;
    let mut graph = source_content::GraphBuilder::new(project, &index);
    catalogue(root, sources, &mut graph)
}

fn catalogue(
    root: &Path,
    sources: &[Source],
    graph: &mut source_content::GraphBuilder<'_>,
) -> Result<BTreeMap<String, source_content::Output>, CookError> {
    let mut bindings = BTreeMap::new();
    for source in sources.iter().filter(|s| {
        [&scene::SCENE, &scene::PREFAB, &look::LOOK, &story::DIALOGUE].contains(&s.kind)
            && s.path.starts_with(root.join("assets"))
    }) {
        let file = source
            .path
            .strip_prefix(root.join("assets"))
            .map_err(|e| CookError(e.to_string()))?
            .to_string_lossy()
            .replace('\\', "/");
        let stem = file
            .rsplit_once('.')
            .map_or(file.as_str(), |(stem, _)| stem);
        let id = format!("base/{stem}");
        let output = graph
            .insert_scanned(
                source_index::Entry {
                    id: id.clone(),
                    module: "base".into(),
                    file: format!("assets/{file}"),
                    factory: source.kind.name.into(),
                    kind: source.kind.name.into(),
                    digest: blake3::hash(&source.bytes).to_hex().to_string(),
                },
                source.rel.clone(),
            )
            .map_err(CookError)?;
        bindings.insert(id, output);
    }
    Ok(bindings)
}

pub(crate) fn sources(
    root: &Path,
    project: &source_project::Project,
    sources: &mut Vec<Source>,
) -> Result<source_content::Graph, CookError> {
    let index = source_index::Index::open(root, project).map_err(|e| CookError(e.to_string()))?;
    let records = source_records::Records::open(root, project).map_err(CookError)?;
    let mut graph = source_content::GraphBuilder::new(project, &index);
    // Only actual scan results, not guessed paths or a second project reader.
    catalogue(root, sources, &mut graph)?;
    let mut evaluator = source_content::Evaluator::default();
    for entry in index.entries.values() {
        let content = evaluator
            .evaluate(
                root,
                project,
                entry,
                &BTreeMap::new(),
                records.contexts[&entry.module].schema(&entry.factory),
            )
            .map_err(|e| CookError(e.to_string()))?;
        // A raw sampled field already has a native typed domain input. Other
        // per-kind adapters land with their parser removal in P1/P2/P3, rather
        // than serializing arbitrary objects into the author DSL here.
        let kind = match entry.factory.as_str() {
            "field" => &field::FIELD,
            _ => {
                return Err(CookError(format!(
                    "{}: {} has not completed its TS domain cutover",
                    entry.file, entry.factory
                )));
            }
        };
        graph
            .insert(&content, &BTreeMap::new())
            .map_err(|e| CookError(e.to_string()))?;
        let bytes = content.data().as_bytes().to_vec();
        sources.push(Source {
            rel: entry.id.clone(),
            path: root.join(&entry.file),
            kind,
            hash: hash_bytes(&bytes),
            bytes,
            settings: None,
            settings_files: Vec::new(),
            gltf: None,
            error: None,
            encoded: Default::default(),
            whole: Default::default(),
            model: Default::default(),
            conformed: None,
            fracture: None,
        });
    }
    graph.finish().map_err(|e| CookError(e.to_string()))
}
