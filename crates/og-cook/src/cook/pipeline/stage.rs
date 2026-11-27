use super::*;

/// The project's checked bootstrap and where its assets are:
/// read once, before the scan.
pub(crate) struct Project {
    pub(crate) assets_dir: PathBuf,
    pub(crate) cooked_dir: PathBuf,
    pub(crate) source: source_project::Project,
    pub(crate) budgets: stats::Budgets,
    pub(crate) budget_desc: String,
    pub(crate) story_functions: BTreeMap<String, og_story::yarn::types::Function>,
    pub(crate) story_functions_stamp: Vec<u8>,
    /// The creation form's profile (`look::project_profile`): the
    /// manifest's.
    pub(crate) profile: crate::look::Profile,
    /// The boot scene's project ref: project.ts's `scene` (else `main`)
    /// through its `scenes` block; `None` when the block does not name it
    /// (the game's GameDesc does, which the cook does not read).
    pub(crate) boot: Option<String>,
    /// project.ts's `flow`: its dialogue's project ref, which the engine runs
    /// from the first tick, so the bundle needs what it needs.
    pub(crate) flow: Option<String>,
    /// `cook { uses }`: the assets other cooked bundles provide.
    pub(crate) provided: provided::Provided,
}

impl Project {
    pub(crate) fn read(root: &Path) -> Result<Project, CookError> {
        let source = source_project::Project::open(root).map_err(|e| CookError(e.to_string()))?;
        let story_functions =
            story_functions::read(&source.input.story_functions).map_err(CookError)?;
        let story_functions_stamp =
            serde_json::to_vec(&story_functions).map_err(|e| CookError(e.to_string()))?;
        let mut budgets = stats::Budgets::default();
        if let Some(input) = &source.input.budgets {
            macro_rules! apply { ($($field:ident),* $(,)?) => { $(
                if let Some(value) = input.$field { budgets.$field = value; }
            )* }; }
            apply!(
                triangles,
                vertices,
                texture_size,
                bones,
                clips,
                asset_bytes,
                total_bytes,
                sound_seconds,
                video_seconds,
                nav_polygons,
                scene_entities,
                emitter_particles,
                bvh_share,
                gpu_bytes
            );
        }
        let profile = if matches!(
            source.input.create.as_ref().and_then(|c| c.form.as_ref()),
            Some(source_schemas::project::Form::TwoD)
        ) {
            crate::look::Profile::TwoD
        } else {
            crate::look::Profile::ThreeD
        };
        let provided = provided::Provided::read(
            root,
            source
                .input
                .cook
                .as_ref()
                .map_or(&[], |c| c.uses.as_slice()),
        )
        .map_err(CookError)?;
        let boot = source
            .input
            .scenes
            .get(source.input.scene.as_deref().unwrap_or("main"))
            .map(|r| r.id.clone());
        let flow = source.input.flow.as_ref().map(|f| f.source.id.clone());
        let budget_desc = format!(
            "budgets triangles {} vertices {}",
            budgets.triangles, budgets.vertices
        );
        Ok(Project {
            assets_dir: root.join("assets"),
            cooked_dir: root.join("cooked"),
            source,
            budgets,
            budget_desc,
            story_functions,
            story_functions_stamp,
            profile,
            boot,
            flow,
            provided,
        })
    }
}

/// A problem found before the cook ran: the asset, its kind and the error.
pub(crate) type EarlyError = (String, &'static str, String);

/// The scan stage's sources, with the sources expanded from others and
/// the files those consumed.
pub(crate) struct Scanned {
    pub(crate) sources: Vec<Source>,
    pub(crate) ignored: Vec<String>,
    pub(crate) early_errors: Vec<EarlyError>,
    pub(crate) consumed: BTreeSet<String>,
    /// The scan's warnings ([`scan::Scan::warnings`]).
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn scan_sources(
    project: &Project,
    cook_index: &index::Index,
    threads: usize,
    code: &CodeGraph,
) -> Result<Scanned, CookError> {
    let assets_dir = &project.assets_dir;
    let Scan {
        mut sources,
        ignored,
        early_errors,
        external,
        warnings,
    } = scan::scan_with(
        assets_dir,
        &|path| cook_index.read(path),
        threads,
        code,
        project.source.metadata.look.as_ref(),
    )?;
    cook_index.note_external(&external);
    let mut consumed: BTreeSet<String> = BTreeSet::new();
    crate::cook::materials::expand(&mut sources);
    // Typed script results enter the ordinary asset pipelines before planning.
    effect::expand(&mut sources);
    model_text::expand(&mut sources);
    // Effekseer projects become composed effects and what they need (`efk/`).
    efk::expand(&mut sources, &mut consumed);
    Ok(Scanned {
        sources,
        ignored,
        early_errors,
        consumed,
        warnings,
    })
}

/// What the model import stage keeps beside the sources it rewrote.
pub(crate) struct Imported {
    // The `type model` texts, built again under the art contract's face
    // budget once the look is read (below).
    pub(crate) model_texts: BTreeMap<String, String>,
    /// The fractured models whose prefracture came from the cache.
    pub(crate) reused_fractures: BTreeSet<String>,
}

/// What importing one source made beside the source itself.
#[derive(Default)]
struct Made {
    /// A `type model`'s text.
    text: Option<String>,
    /// Its prefracture came from the cache.
    reused_fracture: bool,
    /// The files its glTF read, which are part of it.
    consumed: Vec<String>,
}

/// Every model is made a glb and loaded (vox, FBX and OBJ, text models,
/// fracture), and every source hashed. Independent sources import in
/// parallel; modelGeometry consumers wait for their imported operands.
pub(crate) fn import(
    sources: &mut [Source],
    consumed: &mut BTreeSet<String>,
    texture_cache: Option<&Path>,
    threads: usize,
    rigs: &anim::Rigs,
) -> Imported {
    // Virtual map paths cannot collide with physical files (checked at expansion).
    let textures: BTreeMap<_, _> = sources
        .iter()
        .filter(|s| s.kind == &texture::TEXTURE && !s.path.exists())
        .map(|s| (s.rel.clone(), s.bytes.clone()))
        .collect();
    let ids: BTreeMap<_, _> = sources
        .iter()
        .enumerate()
        .map(|(i, s)| (s.rel.clone(), i))
        .collect();
    let operands: Vec<_> = sources
        .iter()
        .map(|s| {
            if s.kind == &model::MODEL && ext(&s.rel) == "ogt" {
                model_text::operand_references(&s.bytes)
            } else {
                Vec::new()
            }
        })
        .collect();
    let transfers: Vec<_> = sources
        .iter()
        .map(|s| {
            if s.kind != &model::MODEL {
                return None;
            }
            let text = (ext(&s.rel) == "ogt").then(|| String::from_utf8_lossy(&s.bytes));
            model::process::settings(text.as_deref(), settings_block(s, "model"))
                .ok()
                .flatten()
                .and_then(|b| model::process::transfer::source(&b).ok().flatten())
        })
        .collect();
    let dependencies: Vec<_> = operands
        .iter()
        .zip(&transfers)
        .map(|(ops, transfer)| ops.iter().chain(transfer).cloned().collect::<Vec<_>>())
        .collect();
    let mut pending: BTreeSet<_> = (0..sources.len()).collect();
    let mut made: Vec<Option<Made>> = (0..sources.len()).map(|_| None).collect();
    while !pending.is_empty() {
        let ready: BTreeSet<_> = pending
            .iter()
            .copied()
            .filter(|&i| {
                dependencies[i]
                    .iter()
                    .all(|id| ids.get(id).is_none_or(|j| !pending.contains(j)))
            })
            .collect();
        if ready.is_empty() {
            let mut chain = Vec::new();
            let mut i = *pending.first().unwrap();
            loop {
                chain.push(sources[i].rel.clone());
                i = dependencies[i]
                    .iter()
                    .filter_map(|id| ids.get(id))
                    .find(|j| pending.contains(j))
                    .copied()
                    .unwrap();
                if chain.contains(&sources[i].rel) {
                    chain.push(sources[i].rel.clone());
                    break;
                }
            }
            let reason = format!("model dependency cycle: {}", chain.join(" -> "));
            for i in pending {
                sources[i].error = Some(reason.clone());
            }
            break;
        }
        // Resolve from already imported sources, so every supported model
        // format, source settings and referenced generator has one path.
        let expanded: Vec<_> = ready
            .iter()
            .filter(|&&i| !operands[i].is_empty())
            .map(|&i| {
                let bytes = model_text::expand_operands(&sources[i].bytes, |id, node| {
                    let s = ids
                        .get(id)
                        .map(|&i| &sources[i])
                        .ok_or_else(|| format!("modelGeometry source '{id}' is not an asset"))?;
                    let gltf = s.gltf.as_ref().ok_or_else(|| {
                        format!(
                            "modelGeometry source '{id}' did not import as a model: {}",
                            s.error.as_deref().unwrap_or("not a model")
                        )
                    })?;
                    model::operand_geometry(gltf, node)
                });
                (i, bytes)
            })
            .collect();
        for (i, bytes) in expanded {
            match bytes {
                Ok(bytes) => sources[i].bytes = bytes,
                Err(e) => sources[i].error = Some(e),
            }
        }
        let bases: BTreeMap<_, _> = ready
            .iter()
            .filter_map(|&i| transfers[i].as_ref())
            .filter_map(|id| {
                let s = &sources[*ids.get(id)?];
                let mut files = s.gltf.as_ref()?.external.clone();
                files.sort_by(|a, b| a.0.cmp(&b.0));
                Some((
                    id.clone(),
                    model::process::TransferInput {
                        bytes: s.bytes.clone(),
                        files,
                    },
                ))
            })
            .collect();
        let slots: Vec<_> = sources
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| ready.contains(i))
            .map(|(i, s)| (i, Mutex::new(s)))
            .collect();
        for (i, result) in each(&slots, threads, |(i, slot)| {
            let s: &mut Source = &mut slot.lock().expect("no poisoning");
            let result = if s.error.is_some() {
                None
            } else {
                import_source(s, texture_cache, &textures, &bases, rigs)
            };
            s.hash = source_hash(s);
            (*i, result)
        }) {
            made[i] = result;
        }
        pending.retain(|i| !ready.contains(i));
    }
    let mut model_texts: BTreeMap<String, String> = BTreeMap::new();
    let mut reused_fractures = BTreeSet::new();
    for (s, made) in sources.iter().zip(made) {
        let Some(made) = made else { continue };
        if let Some(text) = made.text {
            model_texts.insert(s.rel.clone(), text);
        }
        if made.reused_fracture {
            reused_fractures.insert(s.rel.clone());
        }
        consumed.extend(made.consumed);
    }
    Imported {
        model_texts,
        reused_fractures,
    }
}

/// A source's hash: its own bytes', a byte at a time, with its settings
/// and every file its glTF read.
pub(crate) fn source_hash(s: &Source) -> u64 {
    let mut h = StableHasher::new();
    h.write(&s.bytes);
    if let Some((text, _)) = &s.settings {
        h.write_str(text);
    }
    if let Some(g) = &s.gltf {
        if let Some(hash) = g.skin_rig_hash {
            h.write_u64(hash);
        }
        let mut ext: Vec<_> = g.external.iter().collect();
        ext.sort_by(|a, b| a.0.cmp(&b.0));
        for (p, b) in ext {
            h.write_str(p);
            h.write(b);
        }
    }
    h.finish()
}

/// One source's import; `None` when it failed, with its error on the
/// source (which then has no hash and plans nothing).
fn import_source(
    s: &mut Source,
    texture_cache: Option<&Path>,
    textures: &BTreeMap<String, Vec<u8>>,
    bases: &BTreeMap<String, model::process::TransferInput>,
    rigs: &anim::Rigs,
) -> Option<Made> {
    let mut made = Made::default();
    let dir = s.path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let base = s
        .rel
        .rsplit_once('/')
        .map_or(String::new(), |(p, _)| format!("{p}/"));
    let read = |file: &str| {
        textures
            .get(&normalise_rel(&format!("{base}{file}")))
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| std::fs::read(dir.join(file)).map_err(|e| format!("{file}: {e}")))
    };
    let processing = if s.kind == &model::MODEL {
        let text = (ext(&s.rel) == "ogt").then(|| String::from_utf8_lossy(&s.bytes));
        match model::process::settings(text.as_deref(), settings_block(s, "model")) {
            Ok(v) => v,
            Err(e) => {
                s.error = Some(e);
                return None;
            }
        }
    } else {
        None
    };
    // What a model says it is (`conform character`, `conform effect`),
    // read from its text or import settings before a text becomes a glb.
    let role = if s.kind == &crate::cook::model::MODEL {
        match conform::model_role(s) {
            Ok(c) => c,
            Err(e) => {
                s.error = Some(e);
                return None;
            }
        }
    } else {
        conform::ModelRole::Prop
    };
    // A MagicaVoxel file is a model: meshed into a glTF binary here (its
    // import settings' `model { voxScale voxOrigin }` applied), so the
    // rest of the cook sees a .glb.
    if s.kind == &crate::cook::model::MODEL && ext(&s.rel) == "vox" {
        match vox::VoxSettings::parse(settings_block(s, "model"))
            .and_then(|v| vox::to_glb(&s.bytes, &v))
        {
            Ok(glb) => s.bytes = glb,
            Err(e) => {
                s.error = Some(e);
                return None;
            }
        }
    }
    // An FBX or OBJ is a model: read by ufbx and written as a glTF
    // binary here (`fbx.rs`), so the rest of the cook sees a .glb; the
    // files it read beside the glb's own (an OBJ's .mtl, the sources
    // of packed images) join its hash and are consumed with it.
    let mut imported = None;
    if s.kind == &crate::cook::model::MODEL && fbx::handles(&ext(&s.rel)) {
        match fbx::to_glb(&s.bytes, &s.path, &ext(&s.rel)) {
            Ok(i) => {
                s.bytes = i.glb;
                imported = Some((i.read, i.notes));
            }
            Err(e) => {
                s.error = Some(e);
                return None;
            }
        }
    }
    // A model whose source is an og-shape program (the program, or a
    // `type model` naming it with its values): replayed by the kernel
    // and written as a glTF binary here (`model_program.rs`); the
    // program and the files it read join its hash.
    if s.kind == &crate::cook::model::MODEL
        && (model_program::is_program(&s.rel)
            || (ext(&s.rel) == "ogt"
                && Document::parse(&String::from_utf8_lossy(&s.bytes))
                    .is_ok_and(|d| model_program::names_program(&d))))
    {
        match model_program::parse(&s.rel, &s.bytes, &read).and_then(|p| p.glb(&read)) {
            Ok(w) => {
                s.bytes = w.glb;
                imported = Some((w.read, Vec::new()));
            }
            Err(e) => {
                s.error = Some(e);
                return None;
            }
        }
    }
    // A model written as text (`type model`): its parts built and
    // written as a glTF binary here, so the rest of the cook sees a .glb.
    else if s.kind == &crate::cook::model::MODEL && ext(&s.rel) == "ogt" {
        let text = String::from_utf8_lossy(&s.bytes).to_string();
        match model_text::to_glb(&text) {
            Ok(glb) => {
                s.bytes = glb;
                made.text = Some(text);
            }
            Err(e) => {
                s.error = Some(e);
                return None;
            }
        }
    }
    // Its stated front and up (`model { front "-x" up "+y" }`): turned
    // to +Z and +Y before the fracture cut and the load read it
    // (`model/orient.rs`).
    if s.kind == &crate::cook::model::MODEL {
        match model::orient::settings(settings_block(s, "model")).and_then(|o| {
            o.map(|(front, up)| model::orient::apply(&s.bytes, front, up))
                .transpose()
        }) {
            Ok(Some(turned)) => s.bytes = turned,
            Ok(None) => {}
            Err(e) => {
                s.error = Some(e);
                return None;
            }
        }
    }
    if let Some(processing) = &processing {
        let processed = model::process::transfer::source(processing).and_then(|id| {
            let base = id
                .as_ref()
                .map(|id| {
                    bases
                        .get(id)
                        .ok_or_else(|| format!("transfer.source '{id}' did not import as a model"))
                })
                .transpose()?;
            model::process::apply(&s.bytes, &read, processing, texture_cache, base)
        });
        match processed {
            Ok((bytes, read)) => {
                s.bytes = bytes;
                imported.get_or_insert_with(Default::default).0.extend(read);
            }
            Err(e) => {
                s.error = Some(format!("process: {e}"));
                return None;
            }
        }
    }
    // A `fracture` block: the glb cut into its chunks (`fracture.rs`).
    if s.kind == &crate::cook::model::MODEL {
        let text = made.text.as_deref();
        let input_stamp = stamp(
            &[Area::Fracture],
            &[
                &s.bytes,
                text.unwrap_or_default().as_bytes(),
                s.settings.as_ref().map_or("", |(text, _)| text).as_bytes(),
            ],
        );
        match fracture::Settings::of(text, s.settings.as_ref().map(|(_, d)| d)).and_then(|f| {
            f.map(|f| {
                let key = cache::key(
                    &[crate::format::Area::Fracture],
                    "fracture",
                    &[
                        &s.bytes,
                        text.unwrap_or_default().as_bytes(),
                        s.settings.as_ref().map_or("", |(text, _)| text).as_bytes(),
                    ],
                );
                fracture_cache::run(texture_cache, &key, input_stamp, &read, || {
                    fracture::fracture_glb(&s.bytes, &f, &read)
                })
            })
            .transpose()
        }) {
            Ok(Some((f, cached))) => {
                s.bytes = f.glb;
                s.fracture = Some(f.asset);
                // Its images now live in the binary; the files stay
                // part of the source (its hash, what it consumed).
                let (read, notes) = imported.take().unwrap_or_default();
                if cached {
                    made.reused_fracture = true;
                }
                let mut all = read;
                all.extend(f.read);
                imported = Some((all, notes));
            }
            Ok(None) => {}
            Err(e) => {
                s.error = Some(e);
                return None;
            }
        }
    }
    if s.kind == &crate::cook::model::MODEL {
        match model::load_with(&s.bytes, &read) {
            Ok(mut g) => {
                let relaxed = (|| {
                    let enabled = settings_block(s, "model")
                        .and_then(|b| b.get("relaxSkin"))
                        .map(|e| {
                            if e.values.len() == 1 {
                                e.values[0].as_bool()
                            } else {
                                None
                            }
                            .ok_or_else(|| {
                                format!(
                                    "line {}: relaxSkin takes true or false (the default)",
                                    e.line
                                )
                            })
                        })
                        .transpose()?
                        .unwrap_or(false);
                    if enabled {
                        let settings = anim_settings(s)?;
                        model::skin_report::repair(&mut g, rigs, settings.rig.as_deref())?;
                    }
                    Ok::<_, String>(())
                })();
                if let Err(error) = relaxed {
                    s.error = Some(error);
                    return None;
                }
                g.role = role;
                if let Some((read, notes)) = imported {
                    g.external.extend(read);
                    g.warnings.extend(notes);
                }
                let base = s
                    .rel
                    .rsplit_once('/')
                    .map_or(String::new(), |(d, _)| format!("{d}/"));
                for (p, _) in &g.external {
                    // As the asset it names: `cells/../textures/a.png` is
                    // `textures/a.png` (s59). A program reads itself, and
                    // a model is never part of itself.
                    let file = normalise_rel(&format!("{base}{p}"));
                    if file != s.rel {
                        made.consumed.push(file);
                    }
                }
                s.gltf = Some(g);
            }
            Err(e) => s.error = Some(e),
        }
    }
    // Rebuilding procedural primitives under an art face budget would erase
    // the authored preparation. The normal imported-mesh budget applies instead.
    if processing.is_some() {
        made.text = None;
    }
    Some(made)
}

/// The bundle's assets once the files other assets consume are taken out.
pub(crate) struct Resolved {
    pub(crate) known: BTreeSet<String>,
    /// Checked scene and prefab snapshots, keyed by cooked asset identity.
    pub(crate) prefab_texts: BTreeMap<String, std::sync::Arc<scene::Document>>,
}

pub(crate) fn resolve(
    assets_dir: &Path,
    sources: &mut Vec<Source>,
    ignored: &mut Vec<String>,
    mut consumed: BTreeSet<String>,
) -> Resolved {
    // A terrain's heightmap and control map are part of the terrain, an
    // environment's image part of the environment.
    for s in sources.iter() {
        if s.kind == &terrain::TERRAIN
            && let Ok(prepared) = terrain::get(s, &s.rel)
        {
            consumed.extend(prepared.files.iter().filter_map(|path| {
                path.strip_prefix(assets_dir)
                    .ok()
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
            }));
        }
        if let Some(prepared) = &s.rig2d {
            consumed.extend(prepared.files.iter().filter_map(|path| {
                path.strip_prefix(assets_dir)
                    .ok()
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
            }));
        }
        if let Some(timeline) = &s.timeline {
            consumed.extend(timeline.files.values().filter_map(|(path, _)| {
                path.strip_prefix(assets_dir)
                    .ok()
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
            }));
        }
        if s.kind == &environment::ENVIRONMENT {
            consumed.extend(environment::environment_files(s, assets_dir));
        }
    }
    // A file a model references stays an asset of its own as well when it
    // has import settings of its own, not only its directory's (which only a
    // texture cooked on its own reads) or a terrain names it as a layer (by asset id), s59.
    for s in sources.iter() {
        if s.kind == &crate::cook::texture::TEXTURE && s.own_settings() {
            consumed.remove(&s.rel);
        }
        // A model program is a model of its own at its own values, also
        // when a `type model` variant reads it as its part.
        if s.kind == &crate::cook::model::MODEL && model_program::is_program(&s.rel) {
            consumed.remove(&s.rel);
        }
        if s.kind == &terrain::TERRAIN
            && let Ok(prepared) = terrain::get(s, &s.rel)
        {
            for layer in terrain::layer_files(&prepared.doc) {
                consumed.remove(layer);
            }
        }
    }
    // Files a .gltf references are part of that model, not assets of their own.
    sources.retain(|s| !consumed.contains(&s.rel));
    ignored.retain(|r| !consumed.contains(r));
    // TS primary and derived outputs join this set from the actual content
    // graph in the caller, not a guessed list of special-case suffixes.
    let known: BTreeSet<String> = sources.iter().map(|s| s.rel.clone()).collect();
    let prefab_texts = sources
        .iter()
        .filter_map(|s| s.scene.as_ref().map(|doc| (s.rel.clone(), doc.clone())))
        .collect();
    Resolved {
        known,
        prefab_texts,
    }
}

/// The art contract's stage: the project's look, the 2D conformance and the
/// options the 3D conformance cooks with.
pub(crate) struct Contracted {
    pub(crate) contract: Option<(String, crate::look::LookFile)>,
    pub(crate) conformed: Vec<ConformEntry>,
    pub(crate) options: CookOptions,
}

pub(crate) fn contract(
    project: &Project,
    content: &source_content::Graph,
    sources: &mut Vec<Source>,
    known: &BTreeSet<String>,
    model_texts: &BTreeMap<String, String>,
    options: &CookOptions,
    early_errors: &mut Vec<EarlyError>,
) -> Result<Contracted, CookError> {
    let assets_dir = project.assets_dir.as_path();
    // The project's look (og-project.json's `look`): its art
    // contract (docs/art.md §2.1), read before anything cooks. A line that
    // names no look of the bundle is that asset's error.
    let mut contract: Option<(String, crate::look::LookFile)> = None;
    // Its `Stylise` paint fields, which the 3D pass paints a model's colour
    // textures with in texel space (painterly 3D).
    let mut mesh_paint = None;
    let project_look = project
        .source
        .metadata
        .look
        .as_ref()
        .map(|look| match look {
            source_schemas::procedural::LookBase::Asset(reference) => {
                content.outputs[&reference.id].asset.clone()
            }
            source_schemas::procedural::LookBase::Preset(name) => format!("@looks/{name}"),
        });
    if let Some(id) = project_look {
        match sources.iter().find(|s| s.rel == id) {
            Some(s) if s.kind == &look::LOOK => {
                // The project's look decides the art contract, the UI pack
                // and what the models' textures cook to: one that fails stops
                // the cook with nothing changed, as a project.ts that fails
                // does, instead of cooking the project without it and again
                // once it is fixed (a mistyped mood recooked every model
                // texture twice).
                let c = look::contract(s, assets_dir, &|id| known.contains(id))
                    .map_err(|e| CookError(format!("assets/{id}: {e} (og-project.json's look)")))?;
                if let Ok(full) = look::flattened(s, assets_dir, &|id| known.contains(id)) {
                    match conform::texture2d::paint_of(&full, assets_dir) {
                        Ok(p) => mesh_paint = p.active().then_some(p),
                        Err(e) => early_errors.push((id.clone(), "look", e)),
                    }
                }
                contract = Some((id, c));
            }
            Some(_) => early_errors.push((
                id,
                "look",
                "og-project.json's `look` names it, and it is not a look output".into(),
            )),
            None => early_errors.push((
                id.clone(),
                "look",
                format!("og-project.json's `look` names assets/{id}, which does not exist"),
            )),
        }
    }
    // What the 3D pass keeps as made (docs/art.md §2.3): read before the
    // 2D pass, which takes a sheet's own `conform` line out of its text.
    let project_conform = conform::ProjectConform {
        off: project
            .source
            .input
            .conform
            .as_ref()
            .map_or_else(Vec::new, |c| c.off.clone()),
    };
    let mut mesh_off: BTreeMap<String, String> = BTreeMap::new();
    for s in sources
        .iter()
        .filter(|s| [&crate::cook::model::MODEL, &flipbook::FLIPBOOK].contains(&s.kind))
    {
        let why = own_conform_off(s).or_else(|| {
            project_conform
                .off_by(&s.rel)
                .map(|p| format!("project.ts's conform.off: \"{p}\""))
        });
        if let Some(why) = why {
            mesh_off.insert(s.rel.clone(), why);
        }
    }
    // Conformance (docs/art.md §2.3): the 2D textures, to the contract.
    let (conformed, mesh_textures) = conform::texture2d::conform(
        &project_conform,
        project.profile,
        assets_dir,
        sources,
        known,
        contract.as_ref(),
    )
    .map_err(CookError)?;
    // In a 3D project a texture nothing else claims is a mesh's
    // (`conform::TextureRole::Mesh`): the mesh pass holds it, unless it or
    // project.ts opts it out.
    for s in sources.iter().filter(|s| mesh_textures.contains(&s.rel)) {
        let why = own_conform_off(s).or_else(|| {
            project_conform
                .off_by(&s.rel)
                .map(|p| format!("project.ts conform.off: \"{p}\""))
        });
        if let Some(why) = why {
            mesh_off.insert(s.rel.clone(), why);
        }
    }
    // The 3D conformance reads the contract (docs/art.md §2.3): every
    // model, the textures it uses, clips and flipbooks are held to it.
    let mut with_contract = options.clone();
    if let Some((id, c)) = &contract {
        let palette = |p: &str| -> Result<Vec<[u8; 3]>, String> {
            let bytes = match look::builtin(p) {
                Some((_, b)) => b.to_vec(),
                None => std::fs::read(assets_dir.join(crate::runtime::asset_id(p)))
                    .map_err(|e| e.to_string())?,
            };
            crate::look::parse_palette(p, &String::from_utf8_lossy(&bytes))
        };
        match conform::mesh::Conform::of(c, &palette) {
            Ok(m) => {
                with_contract.mesh.conform = m.map(|mut m| {
                    m.paint = mesh_paint.clone();
                    m.off = mesh_off.clone();
                    m.textures = mesh_textures.clone();
                    m
                })
            }
            Err(e) => early_errors.push((id.clone(), "look", e)),
        }
    }
    // A `type model` past its face budget is built again at the style's
    // segment counts (`model_text::to_glb_within`) before the 3D pass
    // decimates what is left. Its stamp already carries the budget
    // (`Conform::model_desc`).
    if let Some(c) = &with_contract.mesh.conform {
        for s in sources
            .iter_mut()
            .filter(|s| s.error.is_none() && !c.off.contains_key(&s.rel))
        {
            let (Some(text), Some(g)) = (model_texts.get(&s.rel), &s.gltf) else {
                continue;
            };
            // A fractured model keeps its chunks; the decimation that
            // follows holds it to the budget.
            if s.fracture.is_some() {
                continue;
            }
            if !c.applies(g) {
                continue;
            }
            let (budget, _) = c.budget(g);
            let role = g.role;
            let written = conform::mesh::drawn_triangles(g);
            let rebuilt = model_text::to_glb_within(text, budget).and_then(|(glb, detail)| {
                if detail >= 1.0 {
                    return Ok(None);
                }
                let dir = s.path.parent().unwrap_or(Path::new("."));
                let mut g = model::load_with(&glb, &|file| {
                    g.external
                        .iter()
                        .find(|(name, _)| name == file)
                        .map(|(_, bytes)| bytes.clone())
                        .map(Ok)
                        .unwrap_or_else(|| {
                            std::fs::read(dir.join(file)).map_err(|e| format!("{file}: {e}"))
                        })
                })?;
                g.role = role;
                g.segments = Some((detail, written));
                Ok(Some((glb, g)))
            });
            match rebuilt {
                Ok(Some((glb, g))) => {
                    s.bytes = glb;
                    s.gltf = Some(g);
                }
                Ok(None) => {}
                Err(e) => s.error = Some(e),
            }
        }
        // Atlas relocation precedes both model and animation jobs, so their
        // vertices and every material map see the same authored-chart mapping.
        for s in sources
            .iter_mut()
            .filter(|s| s.error.is_none() && !c.off.contains_key(&s.rel))
        {
            if let Some(g) = s.gltf.as_mut()
                && let Err(e) = conform::mesh::repack(c, g)
            {
                s.error = Some(e);
            }
        }
    }
    Ok(Contracted {
        contract,
        conformed,
        options: with_contract,
    })
}

/// Streaming groups and the font packs' faces: where outputs go.
pub(crate) struct Grouped {
    pub(crate) groups: BTreeMap<String, stream::GroupDecl>,
    pub(crate) group_of: BTreeMap<String, String>,
    pub(crate) pack_faces: BTreeSet<String>,
}

pub(crate) fn group(
    sources: &[Source],
    known: &BTreeSet<String>,
    early_errors: &mut Vec<EarlyError>,
    threads: usize,
) -> Grouped {
    // Streaming groups (phase 3): every scene's, read before anything cooks.
    let (groups, group_of) = plan_groups(sources, known, early_errors, threads);
    // Font packs' faces and every face a style pack's `fonts` names
    // (docs/ui.md §3.4): cooked as fonts by their own settings, to
    // `<id>.face.ogb`, which the web reads after the boot; text draws
    // from the next face of its fallbacks until one lands.
    let mut pack_faces: BTreeSet<String> = sources
        .iter()
        .filter(|s| s.kind == &font_pack::FONT_PACK)
        .flat_map(font_pack::faces)
        .collect();
    for s in sources
        .iter()
        .filter(|s| crate::cook::ui_packs::is_pack(&s.rel))
    {
        let Ok(text) = std::str::from_utf8(&s.bytes) else {
            continue;
        };
        let Ok(mut layer) = og_ui_next::pack::Layer::parse(text) else {
            continue;
        };
        layer.resolve(s.rel.rsplit_once('/').map_or("", |(dir, _)| dir));
        pack_faces.extend(
            layer
                .fonts
                .iter()
                .map(|f| crate::runtime::asset_id(f).to_owned()),
        );
    }
    if sources
        .iter()
        .any(|s| s.rel == og_ui_next::pack::DEFAULT_FONT_ASSET)
    {
        pack_faces.insert(og_ui_next::pack::DEFAULT_FONT_ASSET.to_string());
    }
    Grouped {
        groups,
        group_of,
        pack_faces,
    }
}
