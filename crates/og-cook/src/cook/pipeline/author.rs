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
    ("flipbook", &flipbook::FLIPBOOK),
    ("scene", &scene::SCENE),
    ("prefab", &scene::PREFAB),
    ("scatter", &scene::SCENE),
    ("model", &model::MODEL),
    ("model-program", &model::MODEL),
    ("table", &table::TABLE),
    ("district", &district::DISTRICT),
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
    let mut outputs = catalogue(root, sources, &mut graph)?;
    // The static content index owns primary TS output identities. Bootstrap
    // resolves these without running content; cooking later checks their data.
    for entry in index.entries.values() {
        outputs.insert(
            entry.id.clone(),
            source_content::Output {
                entry: entry.id.clone(),
                kind: entry.kind.clone(),
                asset: entry.id.clone(),
            },
        );
    }
    Ok(outputs)
}

fn catalogue(
    root: &Path,
    sources: &[Source],
    graph: &mut source_content::GraphBuilder<'_>,
) -> Result<BTreeMap<String, source_content::Output>, CookError> {
    let mut bindings = BTreeMap::new();
    for source in sources.iter().filter(|s| {
        [
            &look::LOOK,
            &story::DIALOGUE,
            &district::MATERIALS,
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

fn declaration(root: &Path, entry: &source_index::Entry, kind: Kind, bytes: Vec<u8>) -> Source {
    Source {
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
        table: None,
        font_pack: None,
        fields: Default::default(),
        fx: None,
        terrains: Default::default(),
        navmesh: None,
        lightmap: None,
        tileset: None,
        tilemap: None,
        animgraph: None,
        flipbook: None,
        rig2d: None,
        scene: None,
        authored_model: None,
        conformed: None,
        fracture: None,
    }
}

/// Mesh operands must be imported before terrain/navigation. Evaluate each
/// declaration once and retain its checked content for the complete graph.
pub(crate) fn models(
    root: &Path,
    project: &source_project::Project,
    sources: &mut Vec<Source>,
    prepared: &mut BTreeMap<String, source_content::Content>,
) -> Result<(), CookError> {
    use source_schemas::world::model::{ModelInput, ProgramModel};
    let index = source_index::Index::open(root, project).map_err(|e| CookError(e.to_string()))?;
    let mut evaluator = source_content::Evaluator::default();
    for entry in index
        .entries
        .values()
        .filter(|e| matches!(e.factory.as_str(), "model" | "model-program"))
    {
        let content = evaluator
            .evaluate(root, project, entry, &BTreeMap::new(), None)
            .map_err(|e| CookError(e.to_string()))?;
        prepared.insert(entry.id.clone(), content);
    }
    for entry in index
        .entries
        .values()
        .filter(|e| matches!(e.factory.as_str(), "model" | "model-program"))
    {
        let content = &prepared[&entry.id];
        let file: source_schemas::File = serde_json::from_value(content.data().root().into_json())
            .map_err(|e| CookError(e.to_string()))?;
        let mut source = declaration(
            root,
            entry,
            &model::MODEL,
            content.data().as_bytes().to_vec(),
        );
        let mut program = None;
        let mut hash = source.bytes.clone();
        let input = if entry.factory == "model-program" {
            ModelInput::Program(Box::new(ProgramModel {
                program: source_schemas::ContentRef {
                    id: entry.id.clone(),
                },
                params: BTreeMap::new(),
                materials: Vec::new(),
                atlas: None,
                conform: None,
                process: None,
                options: None,
            }))
        } else {
            serde_json::from_value::<ModelInput>(file.value.clone())
                .map_err(|e| CookError(e.to_string()))?
        };
        if let ModelInput::Program(d) = &input {
            let target = index
                .entries
                .get(&d.program.id)
                .filter(|e| e.factory == "model-program")
                .ok_or_else(|| {
                    CookError(
                        source_content::diagnostic(
                            content.source(),
                            &entry.file,
                            "/desc/program",
                            format!("{} is not a model-program declaration", d.program.id),
                        )
                        .to_string(),
                    )
                })?;
            let checked = &prepared[&target.id];
            let program_file: source_schemas::File =
                serde_json::from_value(checked.data().root().into_json())
                    .map_err(|e| CookError(e.to_string()))?;
            let materials = file
                .value
                .pointer("/desc/materials")
                .cloned()
                .unwrap_or_else(|| serde_json::json!([]));
            let mut p = model_program::Program {
                file: target.id.clone(),
                text: program_file.value.to_string(),
                params: d.params.clone(),
                materials: serde_json::from_value(materials)
                    .map_err(|e| CookError(e.to_string()))?,
                atlas: d.atlas.unwrap_or(0),
                inputs: BTreeMap::new(),
                authoring: file.value.get("desc").cloned(),
            };
            let text = og_shape::model::program::with_params(&p.text, &p.params)
                .map_err(|e| CookError(e.to_string()))?;
            let paths = og_shape::model::program::fonts_named(&text)
                .map_err(|e| CookError(e.to_string()))?
                .into_iter()
                .chain(
                    og_shape::model::program::textures_named(&text)
                        .map_err(|e| CookError(e.to_string()))?,
                );
            let module = &project.modules[&target.module].root;
            for path in paths {
                if !path.starts_with("assets/") && !path.starts_with("sources/") {
                    return Err(CookError(format!(
                        "{path}: program inputs belong under the owning module's assets/ or sources/"
                    )));
                }
                let file = normalise_rel(&format!("{module}/{path}"));
                let checked = crate::cook::source_path::checked(root, &file)
                    .map_err(|e| CookError(e.to_string()))?;
                p.inputs.insert(
                    path,
                    (
                        file,
                        std::fs::read(checked).map_err(|e| CookError(e.to_string()))?,
                    ),
                );
            }
            hash.extend_from_slice(checked.data().as_bytes());
            program = Some(p);
        }
        let module_root = root.join(&project.modules[&entry.module].root);
        let read = |path: &str| {
            let at = module_root.join(path);
            if let Some(source) = sources.iter().find(|s| s.path == at && !s.path.exists()) {
                return Ok(source.bytes.clone());
            }
            let checked =
                crate::cook::source_path::checked(&module_root, path).map_err(|e| e.to_string())?;
            std::fs::read(checked).map_err(|e| format!("{path}: {e}"))
        };
        source.authored_model = Some(std::sync::Arc::new(
            model_text::Prepared::new(input, hash, &read, program).map_err(|e| {
                CookError(
                    source_content::diagnostic(content.source(), &entry.file, "", e).to_string(),
                )
            })?,
        ));
        sources.push(source);
    }
    Ok(())
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
    let packed = std::sync::Arc::new(records.packed().map_err(CookError)?);
    let mut graph = source_content::GraphBuilder::new(project, &index);
    // Only actual scan results, not guessed paths or a second project reader.
    let scanned = catalogue(root, sources, &mut graph)?;
    graph.insert_provided(provided);
    let mut evaluator = source_content::Evaluator::default();
    let mut fields = field::Fields::new();
    let mut terrains = BTreeMap::new();
    let mut grammars = BTreeMap::new();
    let mut flipbooks = BTreeMap::new();
    let mut deferred_flipbooks = BTreeMap::new();
    let mut table_sources = BTreeMap::new();
    let mut districts = BTreeMap::new();
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
        if matches!(entry.factory.as_str(), "model" | "model-program") {
            graph
                .insert(&content, &BTreeMap::new())
                .map_err(|e| CookError(e.to_string()))?;
            continue;
        }
        let mut source = declaration(root, entry, kind, bytes);
        let derived = (|| match entry.factory.as_str() {
            "table" => {
                table_sources.insert(entry.id.clone(), content.source().clone());
                Ok(BTreeMap::new())
            }
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
            "scene" | "prefab" | "scatter" => {
                let mut doc = if entry.factory == "scene" {
                    scene::Document::scene(source.input()?)
                } else if entry.factory == "scatter" {
                    scene::Document::scene(Default::default())
                } else {
                    scene::Document::prefab(source.input()?)
                };
                doc.records = packed.clone();
                doc.locate(&content);
                source.scene = Some(std::sync::Arc::new(doc));
                Ok(BTreeMap::new())
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
            "flipbook" => {
                use source_schemas::media::clips::Flipbook;
                match source.input()? {
                    Flipbook::Texture(input) => {
                        flipbooks.insert(entry.id.clone(), input);
                        Ok(BTreeMap::new())
                    }
                    Flipbook::Animated(input) => {
                        let module = root.join(&project.modules[&entry.module].root);
                        let built = animated::build(&entry.id, &input, &|path| {
                            std::fs::read(module.join(path)).map_err(|e| format!("{path}: {e}"))
                        })?;
                        for path in &built.read {
                            if let Ok(path) = module.join(path).strip_prefix(root.join("assets")) {
                                consumed.insert(path.to_string_lossy().replace('\\', "/"));
                            }
                        }
                        let settings =
                            (built.sheet.filter == og_anim::SheetFilter::Nearest).then(|| {
                                "type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n".into()
                            });
                        let mut outputs = flipbook::outputs(&built.sheet);
                        outputs.insert("sheet".into(), "texture".into());
                        source.flipbook = Some(built.sheet);
                        source.conformed = Some(std::sync::Arc::new(built.png.clone()));
                        sources.push(flipbook::texture(&source, built.png, settings));
                        Ok(outputs)
                    }
                }
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
        if flipbooks.contains_key(&entry.id) {
            deferred_flipbooks.insert(entry.id.clone(), content);
        } else if entry.factory == "district" {
            districts.insert(entry.id.clone(), content);
        } else {
            graph
                .insert(&content, &derived)
                .map_err(|e| CookError(e.to_string()))?;
        }
        sources.push(source);
    }
    for (id, input) in flipbooks {
        let texture = scanned
            .get(&input.texture.id)
            .map_or(&input.texture.id, |output| &output.asset)
            .clone();
        let bytes = &sources
            .iter()
            .find(|s| s.rel == texture)
            .ok_or_else(|| CookError(format!("{id}: texture '{texture}' has no prepared image")))?
            .bytes;
        let sheet = flipbook::declaration(input, texture.clone(), bytes)
            .map_err(|e| CookError(format!("{id}: {e}")))?;
        graph
            .insert(&deferred_flipbooks[&id], &flipbook::outputs(&sheet))
            .map_err(|e| CookError(e.to_string()))?;
        let bytes = std::sync::Arc::new(bytes.clone());
        let source = sources
            .iter_mut()
            .find(|s| s.rel == id)
            .expect("checked flipbook");
        source.flipbook = Some(sheet);
        source.conformed = Some(bytes);
    }
    district::expand(
        root, sources, consumed, &grammars, &records, &districts, &scanned,
    )
    .map_err(CookError)?;
    let mut district_outputs: BTreeMap<_, BTreeMap<_, _>> = BTreeMap::new();
    for source in sources.iter() {
        if let Some((id, suffix)) = source.rel.split_once('#')
            && districts.contains_key(id)
        {
            district_outputs
                .entry(id)
                .or_default()
                .insert(suffix.to_owned(), source.kind.name.to_owned());
        }
    }
    for (id, content) in &districts {
        let derived = district_outputs.remove(id.as_str()).unwrap_or_default();
        graph
            .insert(content, &derived)
            .map_err(|e| CookError(e.to_string()))?;
    }
    let graph = graph.finish().map_err(|e| CookError(e.to_string()))?;
    table::prepare(sources, &graph, &packed, &table_sources)?;
    let mut lightmap_scenes = BTreeSet::new();
    for source in sources.iter_mut() {
        if let Some(doc) = &mut source.scene {
            std::sync::Arc::get_mut(doc)
                .expect("unshared author scene")
                .bind(&graph);
        }
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
