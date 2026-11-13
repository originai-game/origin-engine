use super::*;
use og_ui::prelude::*;

fn walk(dir: &Path, rel: &str, out: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let r = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        if e.file_type()?.is_dir() {
            walk(&e.path(), &r, out)?;
        } else {
            out.push((r, e.path()));
        }
    }
    Ok(())
}

pub(crate) fn ext(rel: &str) -> String {
    rel.rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default()
}

/// The project's assets as the cook first reads them: every file under
/// `assets/` and the built-in ones its looks name, classified, with its
/// import settings attached; the files it ignores; the errors found.
pub(crate) struct Scan {
    pub(crate) sources: Vec<Source>,
    pub(crate) ignored: Vec<String>,
    pub(crate) early_errors: Vec<(String, &'static str, String)>,
    /// Each component name the documents use or declare, by the id of the
    /// document that defines it: the game's own, else a built-in one.
    pub(crate) components: BTreeMap<String, String>,
}

pub(crate) fn scan(assets_dir: &Path) -> Result<Scan, CookError> {
    scan_with(assets_dir, &|path| std::fs::read(path), 1)
}

/// The kind of a file whose extension alone says it.
fn kind_by_extension(ext: &str) -> Option<Kind> {
    Some(match ext {
        "glb" | "gltf" | "vox" | "fbx" | "obj" => Kind::Model,
        "png" | "jpg" | "jpeg" | "ktx2" | "webp" | "tga" | "psd" | "dds" | "svg" => Kind::Texture,
        e if audio::EXTENSIONS.contains(&e) => Kind::Sound,
        "ttf" | "otf" => Kind::Font,
        "riv" => Kind::Rive,
        _ => return kinds::by_extension(ext),
    })
}

/// The `type` of each `.ogt` the cook takes as an asset, with its kind.
const OGT_KINDS: &[(&str, Kind)] = &[
    ("scene", Kind::Scene),
    (prefab::PREFAB_TYPE, Kind::Prefab),
    (sfx::SFX_TYPE, Kind::Sound),
    (texture_array::ARRAY_TYPE, Kind::TextureArray),
    (video::VIDEO_TYPE, Kind::Video),
    (model_text::MODEL_TYPE, Kind::Model),
];

/// The kind an `.ogt` of this `type` cooks as; `None`: not an asset.
fn ogt_kind(ty: &str) -> Option<Kind> {
    OGT_KINDS
        .iter()
        .find(|(t, _)| *t == ty)
        .map(|&(_, k)| k)
        .or_else(|| kinds::by_type(ty))
}

/// The type an `.ogt`'s name says and does not hold: `<name>.<type>.ogt`
/// names its type, so `hero.flipbook.ogt` cooking as anything but a
/// flipbook is reported. A name whose last part is no type says nothing.
pub(crate) fn misnamed(rel: &str, kind: Kind) -> Option<&str> {
    let stem = rel.rsplit('/').next()?.strip_suffix(".ogt")?;
    let (_, named) = stem.rsplit_once('.')?;
    (ogt_kind(named)? != kind).then_some(named)
}

/// Whether an `.ogt` of type `ty` has lines of its own below its header
/// (CSS declarations, a recipe), which `.ogt`'s reader and `og fmt` leave
/// alone.
pub fn own_lines(ty: &str) -> bool {
    [
        "style",
        og_ui::doc::SCREEN_TYPE,
        og_ui::doc::COMPONENT_TYPE,
        vfx2d::VFX2D_TYPE,
    ]
    .contains(&ty)
}

/// The extensions of the texts the scan reads more than once.
const TEXTS: [&str; 4] = ["ogt", "css", "json", "svelte"];

pub(crate) fn scan_with(
    assets_dir: &Path,
    read_file: &(impl Fn(&Path) -> std::io::Result<Vec<u8>> + Sync),
    threads: usize,
) -> Result<Scan, CookError> {
    let mut files = Vec::new();
    walk(assets_dir, "", &mut files)
        .map_err(|e| CookError(format!("{}: {e}", assets_dir.display())))?;
    // Every file the scan reads is read here, all at once. What follows
    // takes them from `ahead` (a text as often as it is read, any other
    // file once) and reads for itself what is not there, with its error.
    let reads: Vec<PathBuf> = files
        .iter()
        .filter(|(rel, _)| {
            let ext = ext(rel);
            TEXTS.contains(&ext.as_str()) || kind_by_extension(&ext).is_some()
        })
        .map(|(_, path)| path.clone())
        .collect();
    let bytes = each(&reads, threads, |path| read_file(path));
    let ahead: std::cell::RefCell<BTreeMap<PathBuf, Vec<u8>>> = std::cell::RefCell::new(
        reads
            .into_iter()
            .zip(bytes)
            .filter_map(|(path, bytes)| Some((path, bytes.ok()?)))
            .collect(),
    );
    let read_file = |path: &Path| -> std::io::Result<Vec<u8>> {
        let text = TEXTS.contains(&ext(&path.to_string_lossy()).as_str());
        let read = if text {
            ahead.borrow().get(path).cloned()
        } else {
            ahead.borrow_mut().remove(path)
        };
        read.map_or_else(|| read_file(path), Ok)
    };
    // The engine's built-in assets the looks name join the bundle under
    // their reserved ids (`@canvas/paper.png`, every file of
    // `@styles/pixel/`), once each, read from the cook itself.
    let mut builtins: BTreeMap<String, &'static [u8]> = BTreeMap::new();
    for (rel, path) in &files {
        if ext(rel) == "ogt"
            && let Ok(text) =
                read_file(path).and_then(|b| String::from_utf8(b).map_err(std::io::Error::other))
            // The header says whether it is a look, before the whole of
            // every text is parsed for it.
            && Document::parse_header(&text).is_ok_and(|h| h.ty == crate::look::LOOK_TYPE)
            && Document::parse(&text).is_ok_and(|d| d.ty == crate::look::LOOK_TYPE)
        {
            for (id, bytes) in look::builtin_files(rel, &text, assets_dir) {
                builtins.insert(id.to_string(), bytes);
            }
        }
        // CSS can explicitly reference the shared asset roots, including CJK.
        if ext(rel) == "css"
            && let Ok(text) =
                read_file(path).and_then(|b| String::from_utf8(b).map_err(std::io::Error::other))
        {
            for (id, bytes) in look::shared_files(rel, &text) {
                builtins.insert(id.to_string(), bytes);
            }
        }
    }
    // The built-in components the game's documents use and the built-in
    // screens its code names, that it does not declare, join it as well
    // (`@ui/components/rpg/backpack.svelte`, with its strings), and those
    // they use in turn; a game's document of the same name wins.
    let mut game = BTreeMap::new();
    let mut used = BTreeSet::new();
    let mut early_errors: Vec<(String, &'static str, String)> = Vec::new();
    for (rel, path) in &files {
        if ext(rel) != "svelte" {
            continue;
        }
        let Ok(text) =
            read_file(path).and_then(|b| String::from_utf8(b).map_err(std::io::Error::other))
        else {
            continue;
        };
        // One that does not parse is the plan's error.
        let imports = disk_imports(assets_dir, rel, &text);
        let Ok(doc) = og_ui::UiDocument::parse_with_imports(rel, &text, imports) else {
            continue;
        };
        game.entry(doc.name.clone()).or_insert_with(|| rel.clone());
        used.extend(ui_components::uses(&doc));
    }
    // The built-in screens the game's code names by id
    // (`og.load.document(world, "@ui/screens/pause.svelte")`).
    if let Some(root) = assets_dir
        .parent()
        .filter(|r| r.join("project.ogt").is_file())
    {
        for path in super::assets::game_typescript(root).unwrap_or_default() {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            // Code that does not parse is the asset index's to report.
            let Ok(refs) = super::ts_refs::parse(&path, &text) else {
                continue;
            };
            used.extend(
                refs.literals
                    .into_iter()
                    .map(|l| l.value)
                    .filter(|v| v.starts_with(ui_components::PREFIX)),
            );
        }
    }
    let components = match ui_components::resolve(game.clone(), used) {
        Ok(r) => {
            builtins.extend(r.files);
            r.components
        }
        Err(e) => {
            early_errors.push(("@ui".into(), "document", e));
            game
        }
    };
    for id in builtins.keys() {
        files.push((id.clone(), assets_dir.join(id)));
    }
    let read = |rel: &str, path: &Path| -> Result<Vec<u8>, CookError> {
        match builtins.get(rel) {
            Some(b) => Ok(b.to_vec()),
            None => read_file(path).map_err(|e| CookError(format!("{}: {e}", path.display()))),
        }
    };

    // Classify; import settings attach to the file they are named after, a
    // directory's (`import.ogt`) to the files under it that it names.
    let names: BTreeSet<&str> = files.iter().map(|(r, _)| r.as_str()).collect();
    let mut sources: Vec<Source> = Vec::new();
    let mut settings: BTreeMap<String, (String, Document)> = BTreeMap::new();
    let mut defaults: Vec<import_defaults::Defaults> = Vec::new();
    let mut ignored = Vec::new();
    for (rel, path) in &files {
        if rel.starts_with('@') && !builtins.contains_key(rel) {
            early_errors.push((
                rel.clone(),
                "asset",
                "paths starting with '@' are the engine's built-in assets (a look names them: \"@canvas/<name>\", \"@palettes/<name>\", a built-in style by its slug); rename the file or its folder".into(),
            ));
            continue;
        }
        let extension = ext(rel);
        let kind = match (kind_by_extension(&extension), extension.as_str()) {
            (Some(kind), _) => kind,
            (None, "json") if font_pack::is_pack(rel, &read(rel, path)?) => {
                Kind::System(&font_pack::FONT_PACK)
            }
            (None, "ogt") => {
                let text = String::from_utf8(read(rel, path)?)
                    .map_err(|_| CookError(format!("{}: not UTF-8 text", path.display())))?;
                // Styles and UI documents have lines of their own (CSS
                // declarations): their header says what they are.
                let own_lines = Document::parse_header(&text).is_ok_and(|h| own_lines(&h.ty));
                let doc = if own_lines {
                    Document::parse_header(&text)
                } else {
                    Document::parse(&text)
                };
                match doc {
                    Ok(doc) if doc.ty == "import" => {
                        // One suffix only: `oak.ogt.ogt` sets how the model
                        // text `oak.ogt` is cooked.
                        let target = rel.strip_suffix(".ogt").unwrap_or(rel);
                        if names.contains(target) {
                            settings.insert(target.to_string(), (text, doc));
                        } else if import_defaults::is_defaults(rel) {
                            match import_defaults::Defaults::parse(rel, &doc) {
                                Ok(d) => defaults.push(d),
                                Err(e) => early_errors.push((rel.clone(), "import", e)),
                            }
                        } else {
                            early_errors.push((
                                rel.clone(),
                                "import",
                                format!("import settings for '{target}', which does not exist"),
                            ));
                        }
                        continue;
                    }
                    Ok(doc) if doc.ty == "style" => {
                        early_errors.push((rel.clone(), "style", "style.ogt is removed; author a directory of .css files and load its directory with og.load.style".into()));
                        continue;
                    }
                    Ok(doc)
                        if doc.ty == og_ui::doc::SCREEN_TYPE
                            || doc.ty == og_ui::doc::COMPONENT_TYPE =>
                    {
                        early_errors.push((rel.clone(), "document", "UI documents use scriptless .svelte markup; replace the old screen/component dialect".into()));
                        continue;
                    }
                    Ok(doc) if let Some(kind) = ogt_kind(&doc.ty) => kind,
                    Ok(doc) => {
                        early_errors.push((
                            rel.clone(),
                            "scene",
                            format!(
                                "an .ogt of type '{}' is not a cookable asset (scene, model, particles, style, icons, tileset, tilemap, flipbook, vfx2d, rig2d, shader, behaviour, navmesh, terrain, timeline, environment, texture-array, video, look, sfx, strings, rig or import)",
                                doc.ty
                            ),
                        ));
                        continue;
                    }
                    Err(e) => {
                        early_errors.push((rel.clone(), "scene", e.to_string()));
                        continue;
                    }
                }
            }
            _ => {
                ignored.push(rel.clone());
                continue;
            }
        };
        let bytes = read(rel, path)?;
        sources.push(Source {
            rel: rel.clone(),
            path: path.clone(),
            kind,
            bytes,
            settings: None,
            settings_files: Vec::new(),
            hash: 0,
            gltf: None,
            error: None,
            encoded: std::sync::OnceLock::new(),
            whole: Default::default(),
            conformed: None,
            fracture: None,
        });
    }
    // Each directory with direct CSS children is a pack. Its cooked source
    // contains those files and their imports, never a generated author manifest.
    let directories: BTreeSet<_> = files
        .iter()
        .filter(|(id, _)| ext(id) == "css")
        .filter_map(|(id, _)| id.rsplit_once('/').map(|(dir, _)| dir.to_string()))
        .collect();
    for dir in directories {
        let mut css = BTreeMap::new();
        let mut pending: Vec<_> = files
            .iter()
            .filter(|(id, _)| {
                ext(id) == "css" && id.rsplit_once('/').is_some_and(|(parent, _)| parent == dir)
            })
            .map(|(id, _)| id.clone())
            .collect();
        while let Some(id) = pending.pop() {
            if css.contains_key(&id) {
                continue;
            }
            let bytes = read(&id, &assets_dir.join(&id))?;
            let text = String::from_utf8(bytes)
                .map_err(|_| CookError(format!("{id}: CSS must be UTF-8")))?;
            pending.extend(
                og_ui::UiDocument::imports(&text)
                    .iter()
                    .map(|path| og_ui::doc::at_rules::resolve(&id, path)),
            );
            css.insert(id, text);
        }
        let css: Vec<_> = css.into_iter().collect();
        let bytes = serde_json::to_vec(&css).expect("CSS sources serialize");
        sources.push(Source {
            path: assets_dir.join(&dir),
            rel: dir,
            kind: Kind::System(&ui_documents::STYLE),
            bytes,
            settings: None,
            settings_files: Vec::new(),
            hash: 0,
            gltf: None,
            error: None,
            encoded: Default::default(),
            whole: Default::default(),
            conformed: None,
            fracture: None,
        });
    }
    ignored.retain(|id| ext(id) != "css");
    // The walk reads a directory's subdirectories before its `import.ogt`.
    defaults.sort_by_key(|d| d.depth());
    for s in sources.iter_mut() {
        (s.settings, s.settings_files) =
            import_defaults::effective(&s.rel, &defaults, settings.remove(&s.rel));
    }
    Ok(Scan {
        sources,
        ignored,
        early_errors,
        components,
    })
}

/// The stylesheets a game document's `@import`s name, read from `assets/`
/// (one that cannot be read is left out: the plan reports it).
fn disk_imports(assets_dir: &Path, rel: &str, text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut pending: Vec<(String, String)> = og_ui::UiDocument::imports(text)
        .into_iter()
        .map(|p| (rel.to_string(), p))
        .collect();
    while let Some((from, path)) = pending.pop() {
        let at = og_ui::doc::at_rules::resolve(&from, &path);
        if out.iter().any(|(p, _)| *p == at) || at.split('/').any(|c| c == ".." || c.is_empty()) {
            continue;
        }
        let Ok(css) = std::fs::read_to_string(assets_dir.join(&at)) else {
            continue;
        };
        pending.extend(
            og_ui::UiDocument::imports(&css)
                .into_iter()
                .map(|p| (at.clone(), p)),
        );
        out.push((at, css));
    }
    out
}

/// The group of a planned output: its asset's, except a glTF's `#proxy`
/// and `#anim`, which stay in the first group (the proxy draws while the
/// group is away; clips are small and bound at load), and the chunks of a
/// sound cooked in chunks after its first (`#chunk<n>`): its head goes in
/// its group's pack and each other chunk stays a file of its own, read
/// ahead of the sound's playheads once the group is resident, as a chunked
/// sound of the first group's are.
pub(crate) fn output_group(
    id: &str,
    rel: &str,
    group_of: &BTreeMap<String, String>,
) -> Option<String> {
    let chunk = id
        .rsplit_once("#chunk")
        .is_some_and(|(_, n)| n.parse::<u32>().is_ok());
    if id.ends_with("#proxy") || id.ends_with("#anim") || chunk {
        return None;
    }
    group_of.get(rel).cloned()
}

/// Every scene's streaming groups, checked across scenes: a group's name is
/// taken once, an asset is in at most one group, and a group holds glTF
/// models without skins or morph targets, textures and sounds (each swapped
/// in behind a stand-in's handle), and texture arrays, flipbooks (an `array
/// true` sheet with its array and layers), environment maps, palettes,
/// looks and LUTs (loaded once the group is resident, let go when it
/// unloads). A refusal is
/// the scene's error, and its scene adds none of its groups.
pub(crate) fn plan_groups(
    sources: &[Source],
    known: &BTreeSet<String>,
    errors: &mut Vec<(String, &'static str, String)>,
    threads: usize,
) -> stream::Groups {
    let mut groups: BTreeMap<String, stream::GroupDecl> = BTreeMap::new();
    let mut group_of: BTreeMap<String, String> = BTreeMap::new();
    // Every scene some scene or prefab places (`scene "<id>"`): only one
    // of them may declare a cell group, since only a cell loads its blob.
    let placing: Vec<&Source> = sources
        .iter()
        .filter(|s| matches!(s.kind, Kind::Scene | Kind::Prefab))
        .collect();
    let placed: BTreeSet<String> = each(&placing, threads, |s| {
        super::scene::subscenes(&String::from_utf8_lossy(&s.bytes), &|_| None)
    })
    .into_iter()
    .flatten()
    .collect();
    // Each scene's groups are its own text's: every scene is parsed for
    // them at once, and they are taken in the scenes' order.
    let scenes: Vec<&Source> = sources.iter().filter(|s| s.kind == Kind::Scene).collect();
    let declared = each(&scenes, threads, |s| {
        stream::groups_of(&s.rel, &String::from_utf8_lossy(&s.bytes), &|id| {
            known.contains(id)
        })
    });
    for (s, decls) in scenes.into_iter().zip(declared) {
        let decls = match decls {
            Ok(d) => d,
            Err(e) => {
                errors.push((s.rel.clone(), "scene", e));
                continue;
            }
        };
        // A scene with a refused group adds none of its groups.
        let mut refused = None;
        let mut mine: BTreeMap<String, stream::GroupDecl> = BTreeMap::new();
        let mut mine_of: BTreeMap<String, String> = BTreeMap::new();
        'groups: for g in &decls {
            if g.trigger == crate::runtime::StreamTrigger::Cell {
                if !placed.contains(&s.rel) {
                    refused = Some(format!(
                        "stream \"{}\" is a cell group, but no scene places {} with a `scene` line; only a sub-scene's cells load one",
                        g.name, s.rel
                    ));
                    break;
                }
                if let Some(other) = mine_of.get(&s.rel) {
                    refused = Some(format!(
                        "stream \"{}\": {} holds its cell group in stream \"{other}\" already; a scene has one",
                        g.name, s.rel
                    ));
                    break;
                }
                // The scene's own blob is the group's first file.
                mine_of.insert(s.rel.clone(), g.name.clone());
            }
            if let Some(other) = groups.get(&g.name).or_else(|| mine.get(&g.name)) {
                refused = Some(format!(
                    "stream \"{}\" is declared by {} too; a group's name is taken once",
                    g.name, other.scene
                ));
                break;
            }
            for a in &g.assets {
                if let Some(other) = group_of.get(a).or_else(|| mine_of.get(a)) {
                    refused = Some(format!(
                        "stream \"{}\": {a} is in stream \"{other}\" already; an asset is in one group",
                        g.name
                    ));
                    break 'groups;
                }
                let Some(src) = sources.iter().find(|x| &x.rel == a) else {
                    continue;
                };
                // A skinned or morphed mesh cannot be swapped in place.
                let skinned = src.gltf.as_ref().is_some_and(|m| {
                    m.doc.skins().next().is_some()
                        || m.doc
                            .meshes()
                            .any(|x| x.primitives().any(|p| p.morph_targets().next().is_some()))
                });
                let fits = matches!(
                    src.kind,
                    Kind::Model | Kind::Texture | Kind::Sound | Kind::TextureArray
                ) || matches!(src.kind, Kind::System(k) if k.streams);
                if !fits || skinned {
                    refused = Some(format!(
                        "stream \"{}\": {a} is {}; a group holds glTF models without skins or morph targets, textures, texture arrays, flipbooks, sounds, environment maps, palettes, looks and colour LUTs",
                        g.name,
                        if skinned {
                            "a skinned or morphed model".to_string()
                        } else {
                            format!("a {}", src.kind.name())
                        }
                    ));
                    break 'groups;
                }
            }
            for a in &g.assets {
                mine_of.insert(a.clone(), g.name.clone());
            }
            mine.insert(g.name.clone(), g.clone());
        }
        if refused.is_none() {
            groups.append(&mut mine);
            group_of.append(&mut mine_of);
        }
        if let Some(e) = refused {
            errors.push((s.rel.clone(), "scene", e));
        }
    }
    (groups, group_of)
}
