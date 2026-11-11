use super::*;

/// The project's own settings (`project.ogt`) and where its assets are:
/// read once, before the scan.
pub(super) struct Project {
    root: PathBuf,
    pub(super) assets_dir: PathBuf,
    pub(super) cooked_dir: PathBuf,
    /// `project.ogt`, parsed; `None` when it is missing or malformed.
    doc: Option<Document>,
    pub(super) budgets: stats::Budgets,
    pub(super) budget_desc: String,
    pub(super) story_functions: BTreeMap<String, og_story::yarn::types::Function>,
    pub(super) story_functions_stamp: Vec<u8>,
}

impl Project {
    pub(super) fn read(root: &Path) -> Result<Project, CookError> {
        let story_functions = story_functions::read(root).map_err(CookError)?;
        let story_functions_stamp =
            serde_json::to_vec(&story_functions).map_err(|e| CookError(e.to_string()))?;
        let doc = std::fs::read_to_string(root.join("project.ogt"))
            .ok()
            .and_then(|t| Document::parse(&t).ok());
        // The project's budgets (project.ogt's `budgets { }`, the defaults
        // without one): a `type model` past them is cooked with a warning.
        let budgets = doc
            .as_ref()
            .and_then(|d| stats::Budgets::from_project(&d.body).ok())
            .unwrap_or_default();
        let budget_desc = format!(
            "budgets triangles {} vertices {}",
            budgets.triangles, budgets.vertices
        );
        Ok(Project {
            root: root.to_path_buf(),
            assets_dir: root.join("assets"),
            cooked_dir: root.join("cooked"),
            doc,
            budgets,
            budget_desc,
            story_functions,
            story_functions_stamp,
        })
    }
}

/// A problem found before the cook ran: the asset, its kind and the error.
pub(super) type EarlyError = (String, &'static str, String);

/// The scan stage's sources, with the sources expanded from others and
/// the files those consumed.
pub(super) struct Scanned {
    pub(super) sources: Vec<Source>,
    pub(super) ignored: Vec<String>,
    pub(super) early_errors: Vec<EarlyError>,
    pub(super) consumed: BTreeSet<String>,
}

pub(super) fn scan_sources(
    assets_dir: &Path,
    cook_index: &index::Index,
    threads: usize,
) -> Result<Scanned, CookError> {
    let Scan {
        mut sources,
        ignored,
        early_errors,
    } = scan::scan_with(assets_dir, &|path| cook_index.read(path), threads)?;
    let mut consumed: BTreeSet<String> = BTreeSet::new();
    // Pixel effects and flipbooks of animated sources become ordinary
    // sheets (`vfx2d.rs`, `animated.rs`).
    vfx2d::expand(&mut sources, &mut consumed);
    animated::expand(&mut sources, &mut consumed);
    // Effekseer projects become composed effects and what they need (`efk/`).
    efk::expand(&mut sources, &mut consumed);
    Ok(Scanned {
        sources,
        ignored,
        early_errors,
        consumed,
    })
}

/// What the model import stage keeps beside the sources it rewrote.
pub(super) struct Imported {
    // The `type model` texts, built again under the art contract's face
    // budget once the look is read (below).
    pub(super) model_texts: BTreeMap<String, String>,
    /// The fractured models whose prefracture came from the cache.
    pub(super) reused_fractures: BTreeSet<String>,
}

/// Every model is made a glb and loaded (vox, FBX and OBJ, text models,
/// fracture), and every source hashed.
pub(super) fn import(
    sources: &mut [Source],
    consumed: &mut BTreeSet<String>,
    texture_cache: Option<&Path>,
    threads: usize,
) -> Imported {
    let mut model_texts: BTreeMap<String, String> = BTreeMap::new();
    let mut reused_fractures = BTreeSet::new();
    // The sources that came through whole: hashed below, all at once.
    let mut whole = Vec::new();
    for (i, s) in sources.iter_mut().enumerate() {
        // What a model says it is (`conform character`, `conform effect`),
        // read from its text or import settings before a text becomes a glb.
        let role = if s.kind == Kind::Model {
            match conform::model_role(s) {
                Ok(c) => c,
                Err(e) => {
                    s.error = Some(e);
                    continue;
                }
            }
        } else {
            conform::ModelRole::Prop
        };
        // A MagicaVoxel file is a model: meshed into a glTF binary here (its
        // import settings' `model { voxScale voxOrigin }` applied), so the
        // rest of the cook sees a .glb.
        if s.kind == Kind::Model && ext(&s.rel) == "vox" {
            match vox::VoxSettings::parse(settings_block(s, "model"))
                .and_then(|v| vox::to_glb(&s.bytes, &v))
            {
                Ok(glb) => s.bytes = glb,
                Err(e) => {
                    s.error = Some(e);
                    continue;
                }
            }
        }
        // An FBX or OBJ is a model: read by ufbx and written as a glTF
        // binary here (`fbx.rs`), so the rest of the cook sees a .glb; the
        // files it read beside the glb's own (an OBJ's .mtl, the sources
        // of packed images) join its hash and are consumed with it.
        let mut imported = None;
        if s.kind == Kind::Model && fbx::handles(&ext(&s.rel)) {
            match fbx::to_glb(&s.bytes, &s.path, &ext(&s.rel)) {
                Ok(i) => {
                    s.bytes = i.glb;
                    imported = Some((i.read, i.notes));
                }
                Err(e) => {
                    s.error = Some(e);
                    continue;
                }
            }
        }
        // A model written as text (`type model`): its parts built and
        // written as a glTF binary here, so the rest of the cook sees a .glb.
        if s.kind == Kind::Model && ext(&s.rel) == "ogt" {
            let text = String::from_utf8_lossy(&s.bytes).to_string();
            match model_text::to_glb(&text) {
                Ok(glb) => {
                    s.bytes = glb;
                    model_texts.insert(s.rel.clone(), text);
                }
                Err(e) => {
                    s.error = Some(e);
                    continue;
                }
            }
        }
        // A `fracture` block: the glb cut into its chunks (`fracture.rs`).
        if s.kind == Kind::Model {
            let dir = s.path.parent().unwrap_or(Path::new("."));
            let text = model_texts.get(&s.rel).map(String::as_str);
            let input_stamp = stamp(&[
                &s.bytes,
                text.unwrap_or_default().as_bytes(),
                s.settings.as_ref().map_or("", |(text, _)| text).as_bytes(),
            ]);
            match fracture::Settings::of(text, s.settings.as_ref().map(|(_, d)| d)).and_then(|f| {
                f.map(|f| {
                    let key = cache::key(
                        "fracture",
                        &[
                            &s.bytes,
                            text.unwrap_or_default().as_bytes(),
                            s.settings.as_ref().map_or("", |(text, _)| text).as_bytes(),
                        ],
                    );
                    fracture_cache::run(texture_cache, &key, input_stamp, dir, || {
                        fracture::fracture_glb(&s.bytes, dir, &f)
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
                        reused_fractures.insert(s.rel.clone());
                    }
                    let mut all = read;
                    all.extend(f.read);
                    imported = Some((all, notes));
                }
                Ok(None) => {}
                Err(e) => {
                    s.error = Some(e);
                    continue;
                }
            }
        }
        if s.kind == Kind::Model {
            let dir = s.path.parent().unwrap_or(Path::new("."));
            match model::load(&s.bytes, dir) {
                Ok(mut g) => {
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
                        // `textures/a.png` (s59).
                        consumed.insert(normalise_rel(&format!("{base}{p}")));
                    }
                    s.gltf = Some(g);
                }
                Err(e) => s.error = Some(e),
            }
        }
        whole.push(i);
    }
    // A source's hash is its own bytes', a byte at a time, over every byte
    // of the project.
    let hashes = each(&whole, threads, |&i| {
        let s = &sources[i];
        let mut h = StableHasher::new();
        h.write(&s.bytes);
        if let Some((text, _)) = &s.settings {
            h.write_str(text);
        }
        if let Some(g) = &s.gltf {
            let mut ext: Vec<_> = g.external.iter().collect();
            ext.sort_by(|a, b| a.0.cmp(&b.0));
            for (p, b) in ext {
                h.write_str(p);
                h.write(b);
            }
        }
        h.finish()
    });
    for (i, hash) in whole.into_iter().zip(hashes) {
        sources[i].hash = hash;
    }
    Imported {
        model_texts,
        reused_fractures,
    }
}

/// The bundle's assets once the files other assets consume are taken out.
pub(super) struct Resolved {
    pub(super) known: BTreeSet<String>,
    // Prefab and scene texts by asset id: scenes and prefabs flatten
    // through the prefabs, and a sub-scene (`scene "<id>"`) is checked
    // against the scenes (a scene, and no cycle).
    pub(super) prefab_texts: BTreeMap<String, String>,
}

pub(super) fn resolve(
    assets_dir: &Path,
    sources: &mut Vec<Source>,
    ignored: &mut Vec<String>,
    mut consumed: BTreeSet<String>,
) -> Resolved {
    // A terrain's heightmap and control map are part of the terrain, an
    // environment's image part of the environment.
    for s in sources.iter() {
        if s.kind == Kind::Terrain {
            consumed.extend(terrain::terrain_files(
                &s.rel,
                assets_dir,
                &String::from_utf8_lossy(&s.bytes),
            ));
        }
        if s.kind == Kind::Rig2d {
            consumed.extend(autorig::part_files(
                &s.rel,
                assets_dir,
                &String::from_utf8_lossy(&s.bytes),
            ));
        }
        if s.kind == Kind::Lightmap
            && let Some(p) = lightmap::pages_file(&s.rel, &String::from_utf8_lossy(&s.bytes))
        {
            consumed.insert(p);
        }
        if s.kind == Kind::Timeline {
            consumed.extend(timeline::caption_files(
                &s.rel,
                assets_dir,
                &String::from_utf8_lossy(&s.bytes),
            ));
        }
        if s.kind == Kind::Environment {
            consumed.extend(environment::environment_files(
                &s.rel,
                assets_dir,
                &String::from_utf8_lossy(&s.bytes),
            ));
        }
    }
    // A file a model references stays an asset of its own as well when it
    // has import settings of its own (which only a texture cooked on its own
    // reads) or a terrain names it as a layer (by asset id), s59.
    for s in sources.iter() {
        if s.kind == Kind::Texture && s.settings.is_some() {
            consumed.remove(&s.rel);
        }
        if s.kind == Kind::Terrain {
            for layer in
                terrain::layer_files(&s.rel, assets_dir, &String::from_utf8_lossy(&s.bytes))
            {
                consumed.remove(&layer);
            }
        }
    }
    // Files a .gltf references are part of that model, not assets of their own.
    sources.retain(|s| !consumed.contains(&s.rel));
    ignored.retain(|r| !consumed.contains(r));
    let known: BTreeSet<String> = sources.iter().map(|s| s.rel.clone()).collect();
    let prefab_texts: BTreeMap<String, String> = sources
        .iter()
        .filter(|s| matches!(s.kind, Kind::Prefab | Kind::Scene))
        .map(|s| {
            (
                s.rel.clone(),
                String::from_utf8_lossy(&s.bytes).into_owned(),
            )
        })
        .collect();
    Resolved {
        known,
        prefab_texts,
    }
}

/// The art contract's stage: the project's look, the 2D conformance and the
/// options the 3D conformance cooks with.
pub(super) struct Contracted {
    pub(super) contract: Option<(String, crate::look::LookFile)>,
    pub(super) conformed: Vec<ConformEntry>,
    pub(super) options: CookOptions,
}

pub(super) fn contract(
    project: &Project,
    sources: &mut Vec<Source>,
    known: &BTreeSet<String>,
    model_texts: &BTreeMap<String, String>,
    options: &CookOptions,
    early_errors: &mut Vec<EarlyError>,
) -> Result<Contracted, CookError> {
    let root = project.root.as_path();
    let assets_dir = project.assets_dir.as_path();
    // The project's look (project.ogt's `look "looks/<id>.ogt"`): its art
    // contract (docs/art.md §2.1), read before anything cooks. A line that
    // names no look of the bundle is that asset's error.
    let mut contract: Option<(String, crate::look::LookFile)> = None;
    // Its `Stylise` paint fields, which the 3D pass paints a model's colour
    // textures with in texel space (painterly 3D).
    let mut mesh_paint = None;
    let project_look = project
        .doc
        .as_ref()
        .map(|d| crate::look::project_look(&d.body))
        .transpose()
        .map_err(|e| CookError(format!("project.ogt:{e}")))?
        .flatten();
    if let Some(id) = project_look {
        match sources.iter().find(|s| s.rel == id) {
            Some(s) if s.kind == Kind::Look => {
                if let Ok(c) = look::contract(s, assets_dir, &|id| known.contains(id)) {
                    if let Ok(full) = look::flattened(s, assets_dir, &|id| known.contains(id)) {
                        match conform::texture2d::paint_of(&full, assets_dir) {
                            Ok(p) => mesh_paint = p.active().then_some(p),
                            Err(e) => early_errors.push((id.clone(), "look", e)),
                        }
                    }
                    contract = Some((id, c));
                }
            }
            Some(_) => early_errors.push((
                id,
                "look",
                "project.ogt's `look` names it, and it is not an .ogt of `type look`".into(),
            )),
            None => early_errors.push((
                id.clone(),
                "look",
                format!("project.ogt's `look` names assets/{id}, which does not exist"),
            )),
        }
    }
    // What the 3D pass keeps as made (docs/art.md §2.3): read before the
    // 2D pass, which takes a sheet's own `conform` line out of its text.
    let project_conform = project
        .doc
        .as_ref()
        .map(|d| conform::ProjectConform::parse(&d.body))
        .transpose()
        .map_err(|e| CookError(format!("project.ogt:{e}")))?
        .unwrap_or_default();
    // The screens `og ui check` visits (project.ogt's `ui { screen ... }`):
    // a malformed block fails the cook with its line and fix.
    if let Some(d) = &project.doc {
        ui_screens::screens(&d.body).map_err(|e| CookError(format!("project.ogt:{e}")))?;
    }
    let mut mesh_off: BTreeMap<String, String> = BTreeMap::new();
    for s in sources
        .iter()
        .filter(|s| matches!(s.kind, Kind::Model | Kind::Flipbook))
    {
        let why = own_conform_off(s).or_else(|| {
            project_conform
                .off_by(&s.rel)
                .map(|p| format!("project.ogt's conform block: off \"{p}\""))
        });
        if let Some(why) = why {
            mesh_off.insert(s.rel.clone(), why);
        }
    }
    // Conformance (docs/art.md §2.3): the 2D textures, to the contract.
    let (conformed, mesh_textures) =
        conform::texture2d::conform(root, assets_dir, sources, known, contract.as_ref())
            .map_err(CookError)?;
    // In a 3D project a texture nothing else claims is a mesh's
    // (`conform::TextureRole::Mesh`): the mesh pass holds it, unless it or
    // project.ogt opts it out.
    for s in sources.iter().filter(|s| mesh_textures.contains(&s.rel)) {
        let why = own_conform_off(s).or_else(|| {
            project_conform
                .off_by(&s.rel)
                .map(|p| format!("project.ogt's conform block: off \"{p}\""))
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
                let mut g = model::load(&glb, dir)?;
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
    }
    Ok(Contracted {
        contract,
        conformed,
        options: with_contract,
    })
}

/// Streaming groups and the font packs' faces: where outputs go.
pub(super) struct Grouped {
    pub(super) groups: BTreeMap<String, stream::GroupDecl>,
    pub(super) group_of: BTreeMap<String, String>,
    pub(super) pack_faces: BTreeSet<String>,
}

pub(super) fn group(
    sources: &[Source],
    known: &BTreeSet<String>,
    early_errors: &mut Vec<EarlyError>,
    threads: usize,
) -> Grouped {
    // Streaming groups (phase 3): every scene's, read before anything cooks.
    let (groups, group_of) = plan_groups(sources, known, early_errors, threads);
    // Font packs' faces (docs/ui.md §3.4): cooked as fonts by their own
    // settings, to `<id>.face.ogb`, which the web reads on demand.
    let pack_faces: BTreeSet<String> = sources
        .iter()
        .filter(|s| s.kind == Kind::FontPack)
        .flat_map(font_pack::faces)
        .collect();
    Grouped {
        groups,
        group_of,
        pack_faces,
    }
}
