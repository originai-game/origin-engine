//! Project content enters the same source, plan, execute and output stages as
//! file importers. Domain cutovers consume canonical author IR, never OGT text
//! generated from TS. The scaffold catalogue is removed in P1/P2/P3.
use super::*;
use crate::cook::{source_content, source_index, source_project, source_records};

/// Domains whose sole author entry is now a checked TS declaration. Keep this
/// separate from legacy type claims; the scanner must not accept their OGT.
pub(crate) const KINDS: &[(&str, Kind)] = &[
    ("effect-texture", &effect_texture::TEXTURE),
    ("sound", &audio::SOUND),
    ("texture-array", &texture_array::TEXTURE_ARRAY),
    ("vector", &vector::VECTOR),
    ("sfx", &sfx::SFX),
    ("environment", &environment::ENVIRONMENT),
    ("icons", &icons::ICONS),
    ("video", &video::VIDEO),
    ("timeline", &timeline::TIMELINE),
    ("field", &field::FIELD),
    ("terrain", &terrain::TERRAIN),
];

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
        [
            &scene::SCENE,
            &scene::PREFAB,
            &look::LOOK,
            &story::DIALOGUE,
            &model::MODEL,
            &texture::TEXTURE,
        ]
        .contains(&s.kind)
            && !s.rel.starts_with('@')
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
    consumed: &mut BTreeSet<String>,
    provided: &provided::Provided,
) -> Result<source_content::Graph, CookError> {
    let index = source_index::Index::open(root, project).map_err(|e| CookError(e.to_string()))?;
    let records = source_records::Records::open(root, project).map_err(CookError)?;
    let mut graph = source_content::GraphBuilder::new(project, &index);
    // Only actual scan results, not guessed paths or a second project reader.
    catalogue(root, sources, &mut graph)?;
    graph.insert_provided(provided);
    let mut evaluator = source_content::Evaluator::default();
    let mut fields = field::Fields::new();
    let mut terrains = BTreeMap::new();
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
        // Each domain consumes the canonical IR directly. A cutover removes
        // its legacy type claim instead of serializing data back into OGT.
        let kind = match entry.factory.as_str() {
            name if let Some((_, kind)) = KINDS.iter().find(|(n, _)| *n == name) => kind,
            _ => {
                return Err(CookError(format!(
                    "{}: {} has not completed its TS domain cutover",
                    entry.file, entry.factory
                )));
            }
        };
        let bytes = content.data().as_bytes().to_vec();
        let mut source = Source {
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
            video: None,
            timeline: None,
            fields: Default::default(),
            terrains: Default::default(),
            conformed: None,
            fracture: None,
        };
        let derived = (|| match entry.factory.as_str() {
            "effect-texture" => effect_texture::derived(content.data()),
            "sound" => audio::derived(content.data()),
            "texture-array" => texture_array::derived(content.data()),
            "field" => {
                fields.insert(entry.id.clone(), field::Prepared::new(source.input()?)?);
                Ok(BTreeMap::new())
            }
            "terrain" => {
                let input: source_schemas::procedural::terrain::TerrainInput = source.input()?;
                terrains.insert(
                    entry.id.clone(),
                    (input, content.source().clone(), entry.file.clone()),
                );
                Ok(BTreeMap::new())
            }
            "video" => {
                let decl = video::declaration(source.input()?)?;
                let prepared = video::Prepared::new(&assets_root(&source), decl)?;
                let outputs = prepared.outputs();
                source.video = Some(prepared);
                Ok(outputs)
            }
            "timeline" => {
                source.timeline = Some(timeline::Prepared::new(
                    &assets_root(&source),
                    source.input()?,
                )?);
                Ok(BTreeMap::new())
            }
            _ => Ok(BTreeMap::new()),
        })()
        .map_err(|e| {
            CookError(source_content::diagnostic(content.source(), &entry.file, "", e).to_string())
        })?;
        graph
            .insert(&content, &derived)
            .map_err(|e| CookError(e.to_string()))?;
        sources.push(source);
    }
    let graph = graph.finish().map_err(|e| CookError(e.to_string()))?;
    timeline::bind(sources, &graph)?;
    field::bind(sources, fields);
    // Street carves address actual canonical terrain IDs, discovered above.
    district::expand(sources, consumed);
    let mut prepared = terrain::Terrains::new();
    for (id, (input, captured, file)) in terrains {
        let source = sources
            .iter()
            .find(|s| s.rel == id)
            .expect("checked terrain source");
        let models = |id: &str, node: Option<&str>| {
            let asset = &graph.outputs[id].asset;
            let source = sources
                .iter()
                .find(|s| &s.rel == asset)
                .ok_or_else(|| format!("modelGeometry '{id}' has not been imported"))?;
            let gltf = source
                .gltf
                .as_ref()
                .ok_or_else(|| format!("modelGeometry '{id}' has no checked mesh"))?;
            model::operand_geometry(gltf, node)
        };
        let terrain = terrain::declaration(input, &models).and_then(|mut doc| {
            for (a, n, _) in &mut doc.layers {
                for r in [a, n].into_iter().flatten() {
                    *r = graph.outputs[r.as_str()].asset.clone();
                }
            }
            for r in [&mut doc.albedo_array, &mut doc.normal_array]
                .into_iter()
                .flatten()
            {
                *r = graph.outputs[r.as_str()].asset.clone();
            }
            terrain::prepare(
                source,
                doc,
                source_content::diagnostic(&captured, &file, "", ""),
            )
        });
        prepared.insert(
            id,
            terrain.map_err(|e| {
                CookError(source_content::diagnostic(&captured, &file, "", e).to_string())
            })?,
        );
    }
    let prepared = std::sync::Arc::new(prepared);
    for source in sources {
        source.terrains = prepared.clone();
    }
    Ok(graph)
}
