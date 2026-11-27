use super::*;

/// The font pack every TSX UI's bundle carries (`templates/fonts/`).
const DEFAULT_CJK_PACK: &str = "chinese-simplified";

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
    /// The files outside the project its built-in assets were read from (a
    /// style pack's, an engine string table's): inputs of the cook as much
    /// as its own assets are.
    pub(crate) external: Vec<PathBuf>,
    /// What the scan found that does not stop the cook (a `nine-<name>`
    /// class whose image no pack has): the report's warnings.
    pub(crate) warnings: Vec<String>,
}

/// The project's assets for a reader of its sources alone (the art
/// contract's roles): no code, so none of the built-in UI it would add.
pub(crate) fn scan(assets_dir: &Path) -> Result<Scan, CookError> {
    scan_with(
        assets_dir,
        &|path| std::fs::read(path),
        1,
        &CodeGraph::default(),
        None,
    )
}

/// The kind of a file whose extension alone says it.
fn kind_by_extension(ext: &str) -> Option<Kind> {
    kinds::by_extension(ext)
}

/// The kind an `.ogt` of this `type` cooks as; `None`: not an asset.
fn ogt_kind(ty: &str) -> Option<Kind> {
    kinds::by_type(ty)
}

/// The type an `.ogt`'s name says and does not hold: `<name>.<type>.ogt`
/// names its type, so `menu.prefab.ogt` cooking as anything but a
/// prefab is reported. A name whose last part is no type says nothing.
pub(crate) fn misnamed(rel: &str, kind: Kind) -> Option<&str> {
    let stem = rel.rsplit('/').next()?.strip_suffix(".ogt")?;
    let (_, named) = stem.rsplit_once('.')?;
    (ogt_kind(named)? != kind).then_some(named)
}

/// Whether an `.ogt` of type `ty` has lines of its own below its header
/// (CSS declarations, a recipe), which `.ogt`'s reader and `og fmt` leave
/// alone.
pub fn own_lines(ty: &str) -> bool {
    ty == "style"
}

/// The extensions of the texts the scan reads more than once.
const TEXTS: [&str; 2] = ["ogt", "json"];

pub(crate) fn scan_with(
    assets_dir: &Path,
    read_file: &(impl Fn(&Path) -> std::io::Result<Vec<u8>> + Sync),
    threads: usize,
    code: &CodeGraph,
    project_look: Option<&crate::cook::source_schemas::procedural::LookBase>,
) -> Result<Scan, CookError> {
    let mut files = Vec::new();
    if assets_dir.exists() {
        walk(assets_dir, "", &mut files)
            .map_err(|e| CookError(format!("{}: {e}", assets_dir.display())))?;
    }
    // Every file the scan reads is read here, all at once. What follows
    // takes them from `ahead` (a text as often as it is read, any other
    // file once) and reads for itself what is not there, with its error.
    let reads: Vec<PathBuf> = files
        .iter()
        .filter(|(rel, _)| {
            let ext = ext(rel);
            TEXTS.contains(&ext.as_str())
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
    if let Some(crate::cook::source_schemas::procedural::LookBase::Preset(name)) = project_look {
        let text = look::PRESETS
            .iter()
            .find(|(id, _)| *id == name)
            .map(|(_, text)| *text)
            .ok_or_else(|| CookError(format!("og-project.json: unknown look preset {name}")))?;
        let id = format!("@looks/{name}");
        builtins.insert(id.clone(), text.as_bytes());
        for (id, bytes) in look::builtin_files(&id, text, assets_dir) {
            builtins.insert(id.to_string(), bytes);
        }
    }
    let mut early_errors: Vec<(String, &'static str, String)> = Vec::new();
    // A game's theme recipe (`styles/<name>/theme.json`) cooks as the
    // pack.json it compiles to, with its drawn art, in its place.
    let recipes: Vec<(String, PathBuf)> = files
        .iter()
        .filter(|(rel, _)| crate::cook::ui_themes::is_recipe(rel))
        .cloned()
        .collect();
    files.retain(|(rel, _)| !crate::cook::ui_themes::is_recipe(rel));
    for (rel, path) in recipes {
        let dir = rel.trim_end_matches(crate::cook::ui_themes::RECIPE);
        let pack = format!("{dir}pack.json");
        if files.iter().any(|(r, _)| *r == pack) {
            early_errors.push((
                rel.clone(),
                "theme",
                format!("{dir} has a theme.json and a pack.json: a style is one or the other (a theme's own pack entries go in its `pack` key)"),
            ));
            continue;
        }
        let made = read_file(&path)
            .map_err(|e| e.to_string())
            .and_then(|b| String::from_utf8(b).map_err(|_| "not UTF-8".to_owned()))
            .and_then(|text| {
                crate::cook::ui_themes::files(
                    &text,
                    &rel,
                    &crate::cook::ui_themes::cache_dir(assets_dir),
                )
            });
        match made {
            Ok(made) => {
                for (file, bytes) in made {
                    builtins.insert(format!("{dir}{file}"), intern(bytes));
                }
            }
            Err(e) => early_errors.push((rel.clone(), "theme", e)),
        }
    }
    // The packs the recipes made name built-in assets as a game's pack does.
    let made: Vec<(String, &'static [u8])> = builtins
        .iter()
        .filter(|(id, _)| crate::cook::ui_packs::is_pack(id))
        .map(|(id, b)| (id.clone(), *b))
        .collect();
    for (id, bytes) in made {
        for (f, b) in look::pack_files(&id, &String::from_utf8_lossy(bytes)) {
            builtins.insert(f.to_string(), b);
        }
    }
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
        // A game's own pack.json can name them.
        if crate::cook::ui_packs::is_pack(rel)
            && let Ok(text) =
                read_file(path).and_then(|b| String::from_utf8(b).map_err(std::io::Error::other))
        {
            for (id, bytes) in look::pack_files(rel, &text) {
                builtins.insert(id.to_string(), bytes);
            }
        }
    }
    let mut warnings: Vec<String> = Vec::new();
    // The engine string tables and emitters the bundle holds (its
    // components'), read from the declarations the bundler resolved.
    let mut table_inputs = Vec::new();
    for (id, path) in &code.tables {
        match std::fs::read(path) {
            Ok(bytes) => {
                builtins.insert(id.clone(), intern(bytes));
                table_inputs.push(path.clone());
            }
            Err(e) => {
                early_errors.push((id.clone(), "strings", format!("{}: {e}", path.display())))
            }
        }
    }
    // TSX UIs can use the default UI without naming a font
    // or a look. Keep its outline-only face separate and lazy like a
    // font-pack face; projects with no UI do not carry it at all.
    if code.tsx {
        for f in look::styles()
            .iter()
            .filter(|f| f.id().strip_suffix(".ts") == Some(og_ui_next::pack::DEFAULT_FONT_ASSET))
        {
            builtins.insert(f.id().to_string(), f.bytes());
        }
    }
    // The default pack's fonts, icons and sounds join a project with a TSX
    // UI, with their import settings, read from the built-in styles. Of its
    // images (its shapes, frames and effect textures) only those a
    // `nine-<name>` class names (in the code or a pack the bundle holds),
    // or the code or a text names by id (an emitter's `texture`), join:
    // the runtime draws no other.
    if code.tsx {
        let pack = og_ui_next::pack::Layer::default_pack();
        let mut nine = code.nine.clone();
        let texts: Vec<String> = files
            .iter()
            .filter(|(rel, _)| TEXTS.contains(&ext(rel).as_str()))
            .filter_map(|(_, path)| read_file(path).ok())
            .chain(builtins.values().map(|b| b.to_vec()))
            .filter_map(|b| String::from_utf8(b).ok())
            .collect();
        // The packs the bundle holds (the game's own, a look's built-in
        // one): the classes their variants write, and the images they have.
        let layers: Vec<og_ui_next::pack::Layer> = files
            .iter()
            .filter(|(rel, _)| crate::cook::ui_packs::is_pack(rel))
            .filter_map(|(_, path)| read_file(path).ok())
            .chain(
                builtins
                    .iter()
                    .filter(|(id, _)| crate::cook::ui_packs::is_pack(id))
                    .map(|(_, b)| b.to_vec()),
            )
            .filter_map(|b| og_ui_next::pack::Layer::parse(&String::from_utf8(b).ok()?).ok())
            .collect();
        // The default pack's own slots too: a component that names a slot
        // (`packClass("Ring", "fill")`) draws the shapes its default
        // dressing names, which its code never writes.
        nine.extend(super::graph::nine_names(
            std::iter::once(&pack)
                .chain(&layers)
                .flat_map(|l| l.variants.values().map(String::as_str)),
        ));
        // A class whose image no pack the bundle holds has draws nothing:
        // the cook says so, rather than leave the art missing silently.
        let have: BTreeSet<&str> = std::iter::once(&pack)
            .chain(&layers)
            .flat_map(|l| l.images.iter().flat_map(|m| m.keys().map(String::as_str)))
            .collect();
        for name in &nine {
            if !name.is_empty()
                && name != og_ui_next::pack::NINE_NONE
                && !have.iter().any(|image| image.starts_with(name.as_str()))
            {
                warnings.push(format!(
                    "`nine-{name}` names no image in the default pack or a pack the game holds: it draws nothing"
                ));
            }
        }
        let images = || pack.images.iter().flat_map(|m| m.iter());
        let kept: BTreeSet<&str> = images()
            .filter(|(name, image)| {
                super::graph::names_image(&nine, name)
                    || code.ids.contains(&image.src)
                    || texts.iter().any(|t| t.contains(image.src.as_str()))
            })
            .map(|(_, image)| image.src.as_str())
            .collect();
        let left_out: BTreeSet<&str> = images()
            .map(|(_, image)| image.src.as_str())
            .filter(|src| !kept.contains(src))
            .collect();
        let wanted: Vec<&str> = pack
            .assets()
            .into_iter()
            .filter(|src| !left_out.contains(src))
            .collect();
        let srcs: BTreeSet<String> = wanted
            .iter()
            .flat_map(|src| [src.to_string(), format!("{src}.ts")])
            .collect();
        for f in look::styles().iter().filter(|f| srcs.contains(f.id())) {
            builtins.insert(f.id().to_string(), f.bytes());
        }
        // An engine whose styles predate an image would cook without it and
        // every `nine-<name>` naming it would draw nothing at run time.
        for src in wanted {
            if !builtins.contains_key(src) && !builtins.contains_key(&format!("{src}.ts")) {
                let dir = look::styles_dir().map_or_else(
                    || "no built-in styles directory was found".to_owned(),
                    |d| format!("not in the built-in styles at {}", d.display()),
                );
                early_errors.push((
                    src.to_owned(),
                    "image",
                    format!("the default pack's image is {dir}: the engine's styles are older than og (OG_STYLES_DIR or the engine package's styles/)"),
                ));
            }
        }
    }
    // Every UI draws Chinese with nothing installed: the bundled
    // `chinese-simplified` font pack joins a TSX UI's bundle unless the
    // project installs its own. Like every font pack, its face is read
    // only when a frame first draws a character only it covers.
    let mut at: BTreeMap<String, PathBuf> = BTreeMap::new();
    if code.tsx
        && !files
            .iter()
            .any(|(r, _)| r.ends_with(&format!("/{DEFAULT_CJK_PACK}/extension.json")))
        && let Some(dir) = look::fonts_dir().map(|d| d.join(DEFAULT_CJK_PACK))
    {
        let mut entries = Vec::new();
        walk(&dir, "", &mut entries).map_err(|e| CookError(format!("{}: {e}", dir.display())))?;
        for (file, path) in entries {
            let name = file
                .strip_prefix(&format!("assets/fonts/{DEFAULT_CJK_PACK}/"))
                .unwrap_or(&file);
            let id = format!("@fonts/{DEFAULT_CJK_PACK}/{name}");
            builtins.insert(
                id.clone(),
                intern(
                    std::fs::read(&path)
                        .map_err(|e| CookError(format!("{}: {e}", path.display())))?,
                ),
            );
            at.insert(id, path);
        }
    }
    for id in builtins.keys() {
        // A built-in style file is read where the styles are, so a file
        // that names its neighbours (an icon set's SVGs) finds them.
        let path = at
            .get(id)
            .cloned()
            .or_else(|| code.tables.get(id).cloned())
            .or_else(|| look::style_path(id).map(Path::to_path_buf))
            .unwrap_or_else(|| assets_dir.join(id));
        files.push((id.clone(), path));
    }
    let mut external: Vec<PathBuf> = builtins
        .keys()
        .filter(|id| !code.tables.contains_key(*id))
        .filter_map(|id| look::style_path(id).map(Path::to_path_buf))
        .collect();
    external.extend(at.into_values());
    external.extend(table_inputs);
    let read = |rel: &str, path: &Path| -> Result<Vec<u8>, CookError> {
        match builtins.get(rel) {
            Some(b) => Ok(b.to_vec()),
            None => read_file(path).map_err(|e| CookError(format!("{}: {e}", path.display()))),
        }
    };

    // Project binaries are inputs of explicit file.import declarations, never
    // assets merely because a directory walk found their extension.
    let mut sources: Vec<Source> = Vec::new();
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
        let mut author_bytes = None;
        let mut imported_path = None;
        let mut imported_settings = imports::Settings::default();
        let kind = match (kind_by_extension(&extension), extension.as_str()) {
            _ if rel.starts_with("@looks/") && builtins.contains_key(rel) => &look::LOOK,
            (Some(kind), _)
                if (builtins.contains_key(rel)
                    && !rel.starts_with("@styles/")
                    && !rel.starts_with("@fonts/"))
                    || ![
                        &model::MODEL,
                        &texture::TEXTURE,
                        &font::FONT,
                        &audio::SOUND,
                        &riv::RIVE,
                        &tilemap::TILEMAP,
                    ]
                    .contains(&kind) =>
            {
                kind
            }
            // Shipped declarations use the same closed TS evaluator, schemas
            // and canonical IR as project content, never an OGT bridge. The
            // reserved @styles/ prefix names the engine's styles root.
            (None, "ts")
                if (rel.starts_with("@styles/")
                    || rel.starts_with("@fonts/")
                    || rel.starts_with("@ui/")
                        && (rel.ends_with(".strings.ts") || rel.ends_with(".particles.ts")))
                    && builtins.contains_key(rel) =>
            {
                let file = rel
                    .strip_prefix("@styles/")
                    .or_else(|| rel.strip_prefix("@ui/"))
                    .or_else(|| {
                        rel.strip_prefix("@fonts/")
                            .and_then(|s| s.split_once('/').map(|(_, file)| file))
                    })
                    .expect("built-in declaration id");
                let mut root = path.clone();
                for _ in file.split('/') {
                    root.pop();
                }
                let program = generator::Program::prepare_value(&root, file)
                    .map_err(|e| CookError(format!("{rel}: {e}")))?;
                external.extend(program.dependencies.iter().map(|file| root.join(file)));
                let data = program
                    .run_ir()
                    .map_err(|e| CookError(format!("{rel}: {e}")))?;
                let file: source_schemas::File = serde_json::from_value(data.root().into_json())
                    .map_err(|e| CookError(format!("{rel}: {e}")))?;
                file.check(None)
                    .map_err(|e| CookError(format!("{rel}: {e}")))?;
                if file.kind == "import" {
                    let input = serde_json::from_value(file.value)
                        .map_err(|e| CookError(format!("{rel}: {e}")))?;
                    imported_settings =
                        imports::Settings::new(input, data.as_bytes().to_vec(), &|file| {
                            let path = source_path::checked(&root, file)
                                .map_err(|e| format!("{file}: {e}"))?;
                            std::fs::read(path).map_err(|e| format!("{file}: {e}"))
                        })
                        .map_err(|e| CookError(format!("{rel}: {e}")))?;
                    let input = imported_settings.source().expect("import input");
                    let kind = imported_settings.kind();
                    if !kind.extensions.contains(&ext(input).as_str()) {
                        return Err(CookError(format!(
                            "{rel}: {input}: not a supported {} input",
                            kind.name
                        )));
                    }
                    external.extend(imported_settings.files.keys().map(|p| root.join(p)));
                    author_bytes = Some(imported_settings.read(input).map_err(CookError)?);
                    imported_path = Some(root.join(input));
                    kind
                } else {
                    let kind = super::author::KINDS
                        .iter()
                        .find(|(name, _)| *name == file.kind)
                        .map(|(_, kind)| *kind)
                        .ok_or_else(|| {
                            CookError(format!(
                                "{rel}: {} has not completed its TS domain cutover",
                                file.kind
                            ))
                        })?;
                    author_bytes = Some(data.as_bytes().to_vec());
                    kind
                }
            }
            (None, "json") if ui_packs::is_pack(rel) => &ui_packs::PACK,
            (None, "svelte") => {
                early_errors.push((rel.clone(), "document", ".svelte documents are removed: write the screen as a TSX component and mount it with mount(world, Screen, props, { layer }) from @origingame/engine/ui".into()));
                continue;
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
                        early_errors.push((rel.clone(), "import", "OGT import settings and directory defaults are removed; declare content/**/*.ts with file.import({ kind, source, ...settings })".into()));
                        continue;
                    }
                    Ok(doc) if doc.ty == "style" => {
                        early_errors.push((rel.clone(), "style", "style.ogt is removed; author a directory with a pack.json and load its directory with og.load.style".into()));
                        continue;
                    }
                    Ok(doc) if super::author::KINDS.iter().any(|(name, _)| *name == doc.ty) => {
                        early_errors.push((
                            rel.clone(),
                            "author",
                            format!("{} OGT authoring is removed; write content/**/*.ts exporting file[{:?}] from @origingame/engine/author", doc.ty, doc.ty),
                        ));
                        continue;
                    }
                    Ok(doc) if let Some(kind) = ogt_kind(&doc.ty) => kind,
                    Ok(doc) => {
                        early_errors.push((
                            rel.clone(),
                            "scene",
                            format!(
                                "an .ogt of type '{}' is not a cookable asset; use og api list --kind file to find the author entry for each domain",
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
        let bytes = match author_bytes {
            Some(bytes) => bytes,
            None => read(rel, path)?,
        };
        let mut source = Source {
            rel: if imported_path.is_some() {
                let id = rel
                    .strip_suffix(".ts")
                    .expect("import declaration")
                    .to_string();
                if let Some((pack, file)) =
                    id.strip_prefix("@fonts/").and_then(|s| s.split_once('/'))
                {
                    format!(
                        "@fonts/{pack}/{}",
                        file.strip_prefix(&format!("content/fonts/{pack}/"))
                            .unwrap_or(file)
                    )
                } else {
                    id
                }
            } else {
                rel.clone()
            },
            path: imported_path.unwrap_or_else(|| path.clone()),
            kind,
            bytes,
            settings: imported_settings,
            hash: 0,
            gltf: None,
            error: None,
            encoded: std::sync::OnceLock::new(),
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
        };
        if source.kind == &strings::STRINGS {
            source.strings =
                Some(strings::declaration(source.input().map_err(CookError)?).map_err(CookError)?);
        }
        if source.kind == &font_pack::FONT_PACK {
            source.font_pack = Some(
                font_pack::declaration(source.input().map_err(CookError)?, &assets_root(&source))
                    .map_err(CookError)?,
            );
        }
        sources.push(source);
    }
    let captured: BTreeSet<_> = sources
        .iter()
        .flat_map(|s| {
            let root = assets_root(s);
            s.settings.files.keys().map(move |p| root.join(p))
        })
        .collect();
    let captured: BTreeSet<_> = files
        .iter()
        .filter(|(_, path)| captured.contains(path))
        .map(|(rel, _)| rel)
        .collect();
    ignored.retain(|rel| !captured.contains(rel));
    Ok(Scan {
        sources,
        ignored,
        early_errors,
        external,
        warnings,
    })
}

/// `bytes` for the process, each distinct content kept once (the cook
/// lends its built-in files as `&'static`): a long-running cook keeps what
/// it read, and only an edit adds a copy.
fn intern(bytes: Vec<u8>) -> &'static [u8] {
    use std::collections::HashMap;
    type Kept = HashMap<u64, Vec<&'static [u8]>>;
    static KEPT: std::sync::Mutex<Option<Kept>> = std::sync::Mutex::new(None);
    let key = og_core::hash::hash_bytes(&bytes);
    let mut kept = KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let same = kept
        .get_or_insert_with(HashMap::new)
        .entry(key)
        .or_default();
    if let Some(k) = same.iter().find(|k| ***k == *bytes) {
        return k;
    }
    let k: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    same.push(k);
    k
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
    // A scene's HLOD stays too: its proxies draw while a cell group's
    // sub-scene is away.
    if id.ends_with("#proxy") || id.ends_with("#anim") || chunk || crate::hlod::is_hlod_output(id) {
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
    delivery: &super::delivery::Delivery<'_>,
) -> stream::Groups {
    let mut groups: BTreeMap<String, stream::GroupDecl> = BTreeMap::new();
    let mut group_of: BTreeMap<String, String> = BTreeMap::new();
    // Every scene some scene or prefab places (`scene "<id>"`): only one
    // of them may declare a cell group, since only a cell loads its blob.
    let placing: Vec<&Source> = sources
        .iter()
        .filter(|s| [&scene::SCENE, &scene::PREFAB].contains(&s.kind))
        .collect();
    let placed: BTreeSet<String> = each(&placing, threads, |s| {
        s.scene
            .as_ref()
            .map_or_else(Vec::new, |doc| doc.direct_subscenes())
    })
    .into_iter()
    .flatten()
    .collect();
    // Each scene's groups are its own text's: every scene is parsed for
    // them at once, and they are taken in the scenes' order.
    let scenes: Vec<&Source> = sources.iter().filter(|s| s.kind == &scene::SCENE).collect();
    let declared = each(&scenes, threads, |s| {
        let doc = s
            .scene
            .as_ref()
            .ok_or_else(|| format!("{}: missing checked scene", s.rel))?;
        stream::groups_of(&s.rel, doc, &|id| known.contains(id))
    });
    for (s, decls) in scenes.into_iter().zip(declared) {
        let mut decls = match decls {
            Ok(d) => d,
            Err(e) => {
                errors.push((s.rel.clone(), "scene", e));
                continue;
            }
        };
        if delivery.split() {
            match delivery.owner(&s.rel, &s.rel) {
                Ok(pack) => {
                    for group in &mut decls {
                        group.name = format!("{pack}/{}", group.name);
                    }
                }
                Err(e) => {
                    errors.push((s.rel.clone(), "scene", e.to_string()));
                    continue;
                }
            }
        }
        // A scene with a refused group adds none of its groups.
        let mut refused = None;
        let mut mine: BTreeMap<String, stream::GroupDecl> = BTreeMap::new();
        let mut mine_of: BTreeMap<String, String> = BTreeMap::new();
        'groups: for g in &decls {
            if g.name.len() > 64 {
                refused = Some(format!(
                    "stream {:?}: pack-qualified name exceeds the 64-byte StreamEvent.group limit; shorten the pack or group name",
                    g.name
                ));
                break;
            }
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
                let fits = src.kind.streams;
                if !fits || skinned {
                    refused = Some(format!(
                        "stream \"{}\": {a} is {}; a group holds glTF models without skins or morph targets, textures, texture arrays, flipbooks, sounds, environment maps, palettes, looks and colour LUTs",
                        g.name,
                        if skinned {
                            "a skinned or morphed model".to_string()
                        } else {
                            format!("a {}", src.kind.name)
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
