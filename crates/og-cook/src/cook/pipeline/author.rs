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
    ("navmesh", &navmesh::NAVMESH),
    ("shader", &shader_check::SHADER),
    ("post", &shader_check::POST),
    ("ui-material", &shader_check::UI_MATERIAL),
    ("motion-action", &anim::MOTION_ACTION),
    ("strings", &strings::STRINGS),
    ("font-extension", &font_pack::FONT_PACK),
    ("rig", &anim::RIG),
    ("lightmap", &lightmap::LIGHTMAP),
    ("tileset", &tilemap::TILESET),
    ("tilemap", &tilemap::TILEMAP),
    ("grammar", &district::GRAMMAR),
    ("animgraph", &animgraph::GRAPH),
    ("vfx2d", &flipbook::FLIPBOOK),
    ("rig2d", &rig2d::RIG2D),
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
            &font::FONT,
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
        // Derived outputs can retain their producer's path or a synthetic
        // sibling path. Only real original scan entries define this catalogue.
        if source.rel != file || !source.path.is_file() {
            continue;
        }
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

/// Rig maps are inputs of model preparation, which runs before terrain and
/// navigation can consume imported meshes. Retain their checked content for
/// the graph so the author entry is never evaluated again during this cook.
pub(crate) fn rigs(
    root: &Path,
    project: &source_project::Project,
) -> Result<(BTreeMap<String, source_content::Content>, anim::Rigs), CookError> {
    let index = source_index::Index::open(root, project).map_err(|e| CookError(e.to_string()))?;
    let mut evaluator = source_content::Evaluator::default();
    let mut content = BTreeMap::new();
    let mut rigs = anim::Rigs::new();
    for entry in index.entries.values().filter(|e| e.factory == "rig") {
        let checked = evaluator
            .evaluate(root, project, entry, &BTreeMap::new(), None)
            .map_err(|e| CookError(e.to_string()))?;
        let file: crate::cook::source_schemas::File =
            serde_json::from_value(checked.data().root().into_json())
                .map_err(|e| CookError(e.to_string()))?;
        let input: crate::cook::source_schemas::media::clips::Rig =
            serde_json::from_value(file.value).map_err(|e| CookError(e.to_string()))?;
        let asset = og_anim::RigAsset {
            scheme: input.scheme.filter(|s| s != "auto"),
            foot_height: input.foot_height,
            bones: input
                .bones
                .into_iter()
                .map(|(bone, node)| {
                    let at = source_content::diagnostic(
                        checked.source(),
                        &entry.file,
                        &format!("/bones/{bone}"),
                        "",
                    );
                    (
                        og_anim::human_bone(&bone).expect("schema checked bone name"),
                        node,
                        at.location.line,
                    )
                })
                .collect(),
        };
        let at = source_content::diagnostic(checked.source(), &entry.file, "", "");
        asset
            .validate()
            .map_err(|e| CookError(format!("{at}{}", e.reason())))?;
        rigs.insert(
            entry.id.clone(),
            anim::PreparedRig {
                asset,
                hash: hash_bytes(checked.data().as_bytes()),
                diagnostic: at.to_string(),
            },
        );
        content.insert(entry.id.clone(), checked);
    }
    Ok((content, rigs))
}

pub(crate) fn sources(
    root: &Path,
    project: &source_project::Project,
    sources: &mut Vec<Source>,
    consumed: &mut BTreeSet<String>,
    provided: &provided::Provided,
    mut prepared: BTreeMap<String, source_content::Content>,
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
    let mut grammars = BTreeMap::new();
    for entry in index.entries.values() {
        let content = if let Some(content) = prepared.remove(&entry.id) {
            if content.entry() != entry {
                return Err(CookError(format!(
                    "{}: source changed during cook",
                    entry.file
                )));
            }
            content
        } else {
            evaluator
                .evaluate(
                    root,
                    project,
                    entry,
                    &BTreeMap::new(),
                    records.contexts[&entry.module].schema(&entry.factory),
                )
                .map_err(|e| CookError(e.to_string()))?
        };
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
            motion_action: None,
            strings: None,
            font_pack: None,
            fields: Default::default(),
            terrains: Default::default(),
            navmesh: None,
            lightmap: None,
            tileset: None,
            tilemap: None,
            animgraph: None,
            flipbook: None,
            rig2d: None,
            conformed: None,
            fracture: None,
        };
        let derived = (|| match entry.factory.as_str() {
            "rig2d" => {
                let prepared =
                    rig2d::Prepared::new(source.input()?, &assets_root(&source), &source.rel)?;
                let derived = if prepared.atlas.is_some() {
                    BTreeMap::from([("atlas".into(), "texture".into())])
                } else {
                    BTreeMap::new()
                };
                source.rig2d = Some(prepared);
                Ok(derived)
            }
            "grammar" => {
                let grammar = district::grammar::declaration(source.input()?, |name| {
                    let escaped = name.replace('~', "~0").replace('/', "~1");
                    source_content::diagnostic(
                        content.source(),
                        &entry.file,
                        &format!("/rules/{escaped}"),
                        "",
                    )
                    .location
                    .line
                })?;
                grammars.insert(entry.id.clone(), grammar);
                Ok(BTreeMap::new())
            }
            "tilemap" => {
                source.tilemap = Some(tilemap::map_declaration(source.input()?)?);
                Ok(BTreeMap::new())
            }
            "animgraph" => {
                source.animgraph = Some(animgraph::declaration(source.input()?)?);
                Ok(BTreeMap::new())
            }
            "vfx2d" => {
                sources.push(vfx2d::prepare(&mut source)?);
                Ok(BTreeMap::from([("sheet".into(), "texture".into())]))
            }
            "tileset" => {
                source.tileset = Some(tilemap::declaration(source.input()?)?);
                Ok(BTreeMap::new())
            }
            "lightmap" => {
                let prepared = lightmap::Prepared::new(
                    source.input()?,
                    &assets_root(&source),
                    source_content::diagnostic(content.source(), &entry.file, "/scene", "")
                        .to_string(),
                )?;
                if let Ok(path) = prepared.path.strip_prefix(root.join("assets")) {
                    consumed.insert(path.to_string_lossy().replace('\\', "/"));
                }
                source.lightmap = Some(prepared);
                Ok(BTreeMap::new())
            }
            "font-extension" => {
                source.font_pack = Some(font_pack::declaration(
                    source.input()?,
                    &assets_root(&source),
                )?);
                Ok(BTreeMap::new())
            }
            "strings" => {
                source.strings = Some(strings::declaration(source.input()?)?);
                Ok(BTreeMap::new())
            }
            "motion-action" => {
                source.motion_action = Some(anim::motion_action(source.input()?)?);
                Ok(BTreeMap::new())
            }
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
            "navmesh" => {
                source.navmesh = Some(navmesh::Prepared::new(
                    source.input()?,
                    source_content::diagnostic(content.source(), &entry.file, "", ""),
                ));
                Ok(BTreeMap::new())
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
    let mut lightmap_scenes = BTreeSet::new();
    for source in sources.iter_mut() {
        if let Some(map) = &mut source.tilemap {
            for tileset in &mut map.tilesets {
                if let og_tilemap::TilesetRef::Asset(id) = tileset {
                    *id = graph.outputs[id.as_str()].asset.clone();
                }
            }
        }
        if let Some(graph_source) = &mut source.animgraph {
            for b in &mut graph_source.bindings {
                b.source = graph.outputs[&b.source].asset.clone();
            }
        }
        if let Some(t) = &mut source.tileset
            && !t.texture.is_empty()
        {
            t.texture = graph.outputs[&t.texture].asset.clone();
        }
        if let Some(lightmap) = &mut source.lightmap {
            lightmap.bind(&graph.outputs);
            if !lightmap_scenes.insert(lightmap.file.scene.clone()) {
                return Err(CookError(format!(
                    "{}scene '{}' has more than one lightmap",
                    lightmap.at, lightmap.file.scene
                )));
            }
        }
        if let Some(table) = &mut source.strings {
            for font in &mut table.fonts {
                *font = graph.outputs[font.as_str()].asset.clone();
            }
        }
        if let Some(action) = &mut source.motion_action {
            for id in [&mut action.source, &mut action.rig] {
                if let Some(output) = graph.outputs.get(id.as_str()) {
                    *id = output.asset.clone();
                }
            }
        }
    }
    rig2d::bind(sources, &graph)?;
    timeline::bind(sources, &graph)?;
    font_pack::bind(sources, &graph)?;
    field::bind(sources, fields);
    for grammar in grammars.values_mut() {
        for rule in grammar.rules.values_mut() {
            for op in &mut rule.ops {
                if let og_shape::generators::grammar::Op::Insert(id) = op {
                    *id = graph.outputs[id.as_str()].asset.clone();
                }
            }
        }
    }
    // Street carves address actual canonical terrain IDs, discovered above.
    district::expand(sources, consumed, &grammars);
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
