use super::*;

/// The texture cache's key for the project's outputs.
pub(super) fn project_key(
    cook_index: &index::Index,
    sources: &[Source],
    texture_cache: Option<&Path>,
) -> Option<String> {
    texture_cache
        .and_then(|_| cook_index.content_key())
        .map(|input| {
            let dependencies: Vec<_> = sources
                .iter()
                .flat_map(|s| {
                    s.gltf.iter().flat_map(move |g| {
                        g.external.iter().map(move |(name, bytes)| {
                            (&s.rel, name, blake3::hash(bytes).to_hex().to_string())
                        })
                    })
                })
                .collect();
            cache::key(
                "outputs",
                &[
                    input.as_bytes(),
                    &serde_json::to_vec(&dependencies).expect("dependency strings"),
                ],
            )
        })
}

/// An output's cook: its bytes and notes, how long it took, when it began
/// and the thread that ran it.
type Done = (Result<(Vec<u8>, Vec<String>), CookError>, f64, f64, u32);

/// Cooks what is stale, in parallel; results land by plan index.
pub(super) fn execute(
    cx: &Cx,
    plan: &[Planned],
    threads: usize,
    texture_cache: Option<&Path>,
    project_key: Option<&str>,
    started: Instant,
) -> Vec<Mutex<Option<Done>>> {
    let Cx {
        options,
        known,
        prefab_texts,
        schemas,
        budgets,
        ..
    } = *cx;
    let results: Vec<Mutex<Option<Done>>> = plan.iter().map(|_| Mutex::new(None)).collect();
    let todo: Vec<usize> = (0..plan.len())
        .filter(|&i| plan[i].work.is_some())
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for worker in 0..threads.min(todo.len()) {
            let (next, results, plan, schemas, known, todo, prefab_texts) =
                (&next, &results, plan, schemas, known, &todo, prefab_texts);
            scope.spawn(move || {
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(&i) = todo.get(k) else { break };
                    let p = &plan[i];
                    let t = Instant::now();
                    let start = t.duration_since(started).as_secs_f64() * 1000.0;
                    let r = action_cache::run(texture_cache, p, options, project_key, || {
                        run(p, options, threads, schemas, known, prefab_texts, budgets)
                    });
                    *results[i].lock().expect("no poisoning") =
                        Some((r, t.elapsed().as_secs_f64() * 1000.0, start, worker as u32));
                }
            });
        }
    });
    results
}

/// The reports and each source's outputs, with the aliases resolved.
pub(super) struct Assembled {
    pub(super) reports: Vec<AssetReport>,
    pub(super) new_outputs: BTreeMap<usize, Vec<NewOutput>>,
    // The 3D pass's lines per output: this cook's, or for an output that
    // was up to date the ones its last cook recorded.
    pub(super) log: ConformLog,
}

/// Assembles the report and each output's manifest entry and bytes.
pub(super) fn assemble(
    cx: &Cx,
    plan: &[Planned],
    results: Vec<Mutex<Option<Done>>>,
    early_errors: Vec<EarlyError>,
    reused_fractures: &BTreeSet<String>,
) -> Result<Assembled, CookError> {
    let Cx {
        sources,
        cooked_dir,
        group_of,
        old_outputs,
        ..
    } = *cx;
    let mut reports: Vec<AssetReport> = sources
        .iter()
        .map(|s| AssetReport {
            source: s.rel.clone(),
            kind: s.kind.name().to_string(),
            source_bytes: s.bytes.len() as u64
                + s.settings.as_ref().map_or(0, |(t, _)| t.len() as u64)
                + s.gltf
                    .as_ref()
                    .map_or(0, |g| g.external.iter().map(|e| e.1.len() as u64).sum()),
            outputs: Vec::new(),
            notes: if reused_fractures.contains(&s.rel) {
                vec!["fracture: inputs unchanged, reused prefracture".into()]
            } else {
                Vec::new()
            },
            error: s.error.clone(),
            cook_ms: 0.0,
        })
        .collect();
    for (rel, kind, e) in early_errors {
        match reports.iter_mut().find(|r| r.source == rel) {
            Some(r) => r.error = Some(e),
            None => reports.push(AssetReport {
                source: rel,
                kind: kind.to_string(),
                source_bytes: 0,
                outputs: Vec::new(),
                notes: Vec::new(),
                error: Some(e),
                cook_ms: 0.0,
            }),
        }
    }
    let mut new_outputs: BTreeMap<usize, Vec<NewOutput>> = BTreeMap::new();
    // The 3D pass's lines per output: this cook's, or for an output that
    // was up to date the ones its last cook recorded.
    let old_log = read_conform_log(cooked_dir);
    let mut log = ConformLog::new();
    for (i, p) in plan.iter().enumerate() {
        let r = &mut reports[p.source];
        let key = (p.id.clone(), p.stamp);
        let (bytes, cooked, ms, start_ms, worker) =
            match results[i].lock().expect("no poisoning").take() {
                Some((Ok((bytes, notes)), ms, start, worker)) => {
                    let mut mine = Vec::new();
                    for n in notes {
                        match conform_note(&p.id, &n) {
                            Some(what) => mine.push(what.to_string()),
                            None => r.notes.push(n),
                        }
                    }
                    if !mine.is_empty() {
                        log.insert(key, mine);
                    }
                    (Some(OutputBytes::from(bytes)), true, ms, start, worker)
                }
                Some((Err(e), ..)) => {
                    r.error = Some(e.0);
                    continue;
                }
                None => {
                    if let Some(w) = old_log.get(&key) {
                        log.insert(key, w.clone());
                    }
                    (None, false, 0.0, 0.0, 0)
                }
            };
        let (size, hash) = match &bytes {
            Some(b) => (b.len() as u64, hash_bytes(b)),
            None => {
                let o = old_outputs[p.id.as_str()];
                (o.bytes, o.hash)
            }
        };
        new_outputs.entry(p.source).or_default().push((
            ManifestOutput {
                id: p.id.clone(),
                ty: p.ty.to_string(),
                file: p.file.clone(),
                stamp: p.stamp,
                hash,
                bytes: size,
                group: output_group(&p.id, &sources[p.source].rel, group_of),
            },
            bytes,
            OutputReport {
                id: p.id.clone(),
                file: p.file.clone(),
                ty: p.ty.to_string(),
                bytes: size,
                cooked,
                written: false,
                ms,
                start_ms,
                worker,
            },
        ));
    }
    // Resolve all aliases before writing: a changed canonical source may
    // overwrite the file an unchanged alias still reads. Compare the bytes,
    // not just their hash, and choose filenames in id order.
    let mut shared: BTreeMap<(u64, u64), Vec<(String, OutputBytes)>> = BTreeMap::new();
    let mut ordered: Vec<_> = new_outputs
        .iter_mut()
        .filter(|(si, _)| reports[**si].error.is_none())
        .flat_map(|(_, outs)| outs.iter_mut())
        .collect();
    ordered.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    for (m, bytes, rep) in ordered {
        let data = match bytes.take() {
            Some(data) => data,
            None => {
                let file = &old_outputs[m.id.as_str()].file;
                std::fs::read(cooked_dir.join(file))
                    .map_err(|e| CookError(format!("cooked/{file}: {e}")))?
                    .into()
            }
        };
        let bucket = shared.entry((m.hash, m.bytes)).or_default();
        if let Some((file, _)) = bucket.iter().find(|(_, b)| *b == data) {
            m.file = file.clone();
        } else {
            bucket.push((m.file.clone(), data.clone()));
            *bytes = Some(data);
        }
        rep.file = m.file.clone();
    }
    Ok(Assembled {
        reports,
        new_outputs,
        log,
    })
}

/// The manifest and the files it keeps.
pub(super) struct Written {
    manifest: Manifest,
    keep: BTreeSet<String>,
}

/// Writes what changed and builds the manifest.
pub(super) fn write(
    cx: &Cx,
    new_outputs: BTreeMap<usize, Vec<NewOutput>>,
    reports: &mut [AssetReport],
    old: &Manifest,
) -> Result<Written, CookError> {
    let Cx {
        sources,
        cooked_dir,
        font_desc,
        mesh_desc,
        tex_desc,
        ..
    } = *cx;
    let mut manifest = Manifest {
        cooker: COOKER_VERSION.to_string(),
        options: BTreeMap::from([
            ("font".to_string(), font_desc.to_string()),
            ("mesh".to_string(), mesh_desc.to_string()),
            ("texture".to_string(), tex_desc.to_string()),
        ]),
        assets: Vec::new(),
        streams: Vec::new(),
        motion: None,
    };
    let mut keep: BTreeSet<String> = BTreeSet::new();
    for (si, outs) in new_outputs {
        let s = &sources[si];
        if reports[si].error.is_some() {
            continue;
        }
        let mut asset = ManifestAsset {
            source: s.rel.clone(),
            kind: s.kind.name().to_string(),
            source_hash: s.hash,
            source_bytes: reports[si].source_bytes,
            outputs: Vec::new(),
        };
        for (m, bytes, mut rep) in outs {
            if let Some(b) = bytes {
                let path = cooked_dir.join(&m.file);
                if std::fs::read(&path).ok().as_deref() != Some(&b[..]) {
                    if let Some(dir) = path.parent() {
                        std::fs::create_dir_all(dir)
                            .map_err(|e| CookError(format!("{}: {e}", dir.display())))?;
                    }
                    std::fs::write(&path, &b)
                        .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
                    rep.written = true;
                }
            }
            keep.insert(m.file.clone());
            asset.outputs.push(m);
            reports[si].outputs.push(rep);
        }
        manifest.assets.push(asset);
    }
    // Baked reflection probes (`og bake probes`, probes.rs) have no source:
    // kept while their files are.
    for a in old.assets.iter().filter(|a| a.kind == probes::KIND) {
        if a.outputs.iter().all(|o| cooked_dir.join(&o.file).is_file()) {
            keep.extend(a.outputs.iter().map(|o| o.file.clone()));
            manifest.assets.push(a.clone());
        }
    }
    manifest.assets.sort_by(|a, b| a.source.cmp(&b.source));
    for r in reports.iter_mut().filter(|r| r.error.is_some()) {
        r.outputs.clear();
    }
    Ok(Written { manifest, keep })
}

// Each group's outputs, packed into one file the runtime fetches.
pub(super) fn pack(
    cx: &Cx,
    groups: &BTreeMap<String, stream::GroupDecl>,
    written: &mut Written,
    reports: &mut [AssetReport],
) -> Result<(), CookError> {
    let cooked_dir = cx.cooked_dir;
    let Written { manifest, keep } = written;
    for (name, g) in groups {
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut memory = 0u64;
        let mut sounds = Vec::new();
        for o in manifest
            .outputs()
            .filter(|o| o.group.as_deref() == Some(name))
        {
            let path = cooked_dir.join(&o.file);
            let bytes =
                std::fs::read(&path).map_err(|e| CookError(format!("{}: {e}", path.display())))?;
            memory += stream::memory_of(&o.ty, &bytes);
            if o.ty == "og.Sound" {
                sounds.extend(stream::sound_of(&o.id, &bytes));
            }
            files.push((o.file.clone(), bytes));
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        files.dedup_by(|a, b| a.0 == b.0);
        let mut h = StableHasher::new();
        h.write_str(name);
        for (f, b) in &files {
            h.write_str(f);
            h.write(&hash_bytes(b).to_le_bytes());
        }
        let pack = stream::pack(&files, h.finish());
        let file = format!("stream/{name}.ogb");
        let path = cooked_dir.join(&file);
        if std::fs::read(&path).ok().as_deref() != Some(&pack[..]) {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)
                    .map_err(|e| CookError(format!("{}: {e}", dir.display())))?;
            }
            std::fs::write(&path, &pack)
                .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
        }
        keep.insert(file.clone());
        let entry = stream::manifest_entry(
            g,
            file,
            pack.len() as u64,
            hash_bytes(&pack),
            memory,
            sounds,
        );
        if let Some(r) = reports.iter_mut().find(|r| r.source == g.scene) {
            r.notes.push(format!(
                "stream group '{name}': {} asset(s), {} file(s), {} bytes packed, {} bytes once loaded",
                g.assets.len(),
                files.len(),
                entry.bytes,
                memory
            ));
        }
        manifest.streams.push(entry);
    }
    Ok(())
}

/// Writes the manifest, the motion model, the asset index, the publish plan,
/// the cook times and the conform log, and deletes what is no longer an
/// output. Returns the motion model's warnings.
pub(super) fn finish(
    cx: &Cx,
    old: &Manifest,
    written: Written,
    parsed_refs: &BTreeMap<String, BTreeSet<String>>,
    reports: &mut [AssetReport],
    log: &ConformLog,
) -> Result<(Vec<String>, Vec<String>), CookError> {
    let Cx {
        root,
        sources,
        cooked_dir,
        ..
    } = *cx;
    let Written {
        mut manifest,
        mut keep,
    } = written;
    // og.motion's model, for a project whose code uses it (in the manifest,
    // outside its outputs: never in the boot bundle).
    let mut motion_warnings = Vec::new();
    manifest.motion = motion::cook(root, cooked_dir, &mut keep, &mut motion_warnings)?;
    let mut removed = Vec::new();
    let old_files = old
        .outputs()
        .map(|o| &o.file)
        .chain(old.streams.iter().map(|g| &g.file))
        .chain(old.motion.iter().map(|m| &m.file));
    for f in old_files {
        if !keep.contains(f) && cooked_dir.join(f).exists() {
            std::fs::remove_file(cooked_dir.join(f))
                .map_err(|e| CookError(format!("cooked/{f}: {e}")))?;
            removed.push(f.clone());
        }
    }
    std::fs::create_dir_all(cooked_dir)
        .map_err(|e| CookError(format!("{}: {e}", cooked_dir.display())))?;
    let text = manifest.to_text();
    let manifest_path = cooked_dir.join(MANIFEST);
    if std::fs::read_to_string(&manifest_path).ok().as_deref() != Some(text.as_str()) {
        std::fs::write(&manifest_path, &text)
            .map_err(|e| CookError(format!("{}: {e}", manifest_path.display())))?;
    }
    let index = assets::build(root, &manifest, sources, parsed_refs)?;
    index.write(cooked_dir)?;
    index.write_publish_plan(root, &manifest)?;
    // Each asset's cook time: this run's (its outputs' summed) when it
    // cooked, else the one recorded; written only when one changed.
    let old_times = CookTimes::read(cooked_dir);
    let mut times = CookTimes::default();
    for r in reports.iter_mut().filter(|r| r.error.is_none()) {
        let now: Vec<f64> = r
            .outputs
            .iter()
            .filter(|o| o.cooked)
            .map(|o| o.ms)
            .collect();
        r.cook_ms = if now.is_empty() {
            old_times.assets.get(&r.source).copied().unwrap_or(0.0)
        } else {
            now.iter().sum()
        };
        if !r.outputs.is_empty() {
            times.assets.insert(r.source.clone(), r.cook_ms);
        }
    }
    let log_path = cooked_dir.join(CONFORM_LOG);
    if log.is_empty() {
        if log_path.exists() {
            std::fs::remove_file(&log_path)
                .map_err(|e| CookError(format!("{}: {e}", log_path.display())))?;
        }
    } else {
        let text = conform_log_text(log);
        if std::fs::read_to_string(&log_path).ok().as_deref() != Some(text.as_str()) {
            std::fs::write(&log_path, text)
                .map_err(|e| CookError(format!("{}: {e}", log_path.display())))?;
        }
    }
    if times != old_times {
        let path = cooked_dir.join(COOK_TIMES);
        std::fs::write(&path, times.to_text())
            .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
    }
    Ok((removed, motion_warnings))
}

/// The cook's warnings and the art contract's entries: diagnostics describe
/// source assets and never change their cooked blobs.
pub(super) fn diagnose(
    cx: &Cx,
    plan: &[Planned],
    reports: &mut [AssetReport],
    log: &ConformLog,
    contract: Option<&(String, crate::look::LookFile)>,
    mut conformed: Vec<ConformEntry>,
    mut warnings: Vec<String>,
) -> (Vec<ConformEntry>, Vec<String>) {
    let Cx {
        sources,
        assets_dir,
        options,
        story_functions,
        ..
    } = *cx;
    // Across the string tables: every locale has every key, with its arguments.
    let tables: Vec<(&str, &[u8])> = sources
        .iter()
        .filter(|s| s.kind == Kind::Strings)
        .map(|s| (s.rel.as_str(), s.bytes.as_slice()))
        .collect();
    // Diagnostics describe source assets, never change their cooked blobs.
    // Run on cached cooks as well so a warning does not disappear on rerun.
    for (source, report) in sources.iter().zip(reports.iter_mut()) {
        if report.error.is_some() {
            continue;
        }
        if let Some(gltf) = &source.gltf {
            let notes = anim_settings(source).and_then(|settings| {
                let mut notes =
                    model::skin_report::report(gltf, assets_dir, settings.rig.as_deref())?;
                notes.extend(gltf.emissive_warnings()?);
                Ok(notes)
            });
            match notes {
                Ok(notes) => {
                    for note in notes {
                        if note.starts_with("warning:") {
                            warnings.push(format!("assets/{}: {note}", source.rel));
                        }
                        report.notes.push(note);
                    }
                }
                Err(error) => {
                    let note = format!("model diagnostics unavailable: {error}");
                    warnings.push(format!("assets/{}: {note}", source.rel));
                    report.notes.push(note);
                }
            }
        }
    }
    for (rel, what) in e6::check_strings(&tables) {
        warnings.push(format!("assets/{rel}: {what}"));
        if let Some(r) = reports.iter_mut().find(|r| r.source == rel) {
            r.notes.push(what);
        }
    }
    // Dialogues and quests: every key they show is in the base table.
    let stories: Vec<(&str, &[u8])> = sources
        .iter()
        .filter(|s| matches!(s.kind, Kind::Dialogue | Kind::Quest))
        .map(|s| (s.rel.as_str(), s.bytes.as_slice()))
        .collect();
    for (rel, what) in e6::check_story_keys(&tables, &stories, story_functions) {
        warnings.push(format!("assets/{rel}: {what}"));
        if let Some(r) = reports.iter_mut().find(|r| r.source == rel) {
            r.notes.push(what);
        }
    }
    // A text model past the project's triangle or vertex budget: cooked, and said.
    for r in reports.iter().filter(|r| r.kind == "model") {
        for n in r
            .notes
            .iter()
            .filter(|n| n.starts_with("warning: over budget: "))
        {
            warnings.push(format!("assets/{}: {n}", r.source));
        }
    }
    // A prefab override whose target is gone: cooked (dropped), and said
    // (a publish's cook refuses it).
    for r in reports
        .iter()
        .filter(|r| r.kind == "scene" || r.kind == "prefab")
    {
        for n in r.notes.iter().filter(|n| n.contains("orphaned override")) {
            warnings.push(format!("assets/{}: {n}", r.source));
        }
    }
    // A video's frames past the budget: footage is the heaviest asset a game has.
    for r in reports.iter_mut().filter(|r| r.kind == "video") {
        let frames = format!("{}#frames#layer", r.source);
        let bytes: u64 = r
            .outputs
            .iter()
            .filter(|o| o.id.starts_with(&frames))
            .map(|o| o.bytes)
            .sum();
        if bytes > video::BUDGET_BYTES {
            let what = format!(
                "its {} frames take {:.1} MiB cooked, past the {} MiB budget for a video: lower its size, fps or maxSeconds",
                r.outputs
                    .iter()
                    .filter(|o| o.id.starts_with(&frames))
                    .count(),
                bytes as f64 / f64::from(1u32 << 20),
                video::BUDGET_BYTES >> 20
            );
            warnings.push(format!("assets/{}: {what}", r.source));
            r.notes.push(what);
        }
    }
    // A Tiled map's custom properties the importer does not read.
    for s in sources
        .iter()
        .filter(|s| s.kind == Kind::Tilemap && s.rel.to_ascii_lowercase().ends_with(".tmj"))
    {
        for what in tilemap::tiled_unread(&String::from_utf8_lossy(&s.bytes)) {
            warnings.push(format!("assets/{}: {what}", s.rel));
            if let Some(r) = reports.iter_mut().find(|r| r.source == s.rel) {
                r.notes.push(what);
            }
        }
    }
    efk::warnings(sources, reports, &mut warnings);
    // The project's look carries the art contract: say which of its
    // resources it sets (conformance to them is docs/art.md §2.3).
    if let Some((id, c)) = contract
        && let Some(r) = reports.iter_mut().find(|r| &r.source == id)
    {
        let set = c.resources();
        r.notes.push(if set.is_empty() {
            "the project's look (project.ogt): it sets no art resource, so the art contract constrains nothing".into()
        } else {
            format!("the project's look (project.ogt): its art contract sets {}", set.join(", "))
        });
    }
    // The 3D pass's entries (docs/art.md §2.3): per asset, what its
    // outputs' lines say, and what it keeps as made; its warnings (frames
    // dropped past ArtMotion.maxFrames) are the cook's too.
    let failed = |rel: &str| reports.iter().any(|r| r.source == rel && r.error.is_some());
    let mut mesh: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for p in plan {
        if let Some(w) = log.get(&(p.id.clone(), p.stamp)) {
            mesh.entry(sources[p.source].rel.as_str())
                .or_default()
                .extend(w.iter().map(String::as_str));
        }
    }
    let mut mine: Vec<ConformEntry> = Vec::new();
    for (rel, lines) in mesh {
        if failed(rel) {
            continue;
        }
        let mut changed: Vec<&str> = Vec::new();
        for l in lines {
            if l != conform::mesh::UNCHANGED && !changed.contains(&l) {
                changed.push(l);
            }
            if l.starts_with("warning: ") {
                warnings.push(format!("assets/{rel}: {l}"));
            }
        }
        mine.push(ConformEntry {
            source: rel.to_string(),
            action: if changed.is_empty() {
                ConformAction::Unchanged
            } else {
                ConformAction::Changed
            },
            what: if changed.is_empty() {
                conform::mesh::UNCHANGED.to_string()
            } else {
                changed.join("; ")
            },
        });
    }
    if let Some(c) = &options.mesh.conform {
        for (rel, why) in &c.off {
            let applies = sources.iter().any(|s| {
                &s.rel == rel
                    && match s.kind {
                        Kind::Model => true,
                        Kind::Flipbook => c.motion.is_some(),
                        _ => !c.texture_desc().is_empty(),
                    }
            });
            let listed = conformed
                .iter()
                .any(|e| &e.source == rel && e.action == ConformAction::Off);
            if applies && !listed && !failed(rel) {
                mine.push(ConformEntry {
                    source: rel.clone(),
                    action: ConformAction::Off,
                    what: why.clone(),
                });
            }
        }
    }
    mine.sort_by(|a, b| a.source.cmp(&b.source));
    conformed.extend(mine);
    // What conformance changed and what opted out, on each asset too.
    for e in &conformed {
        if let Some(r) = reports.iter_mut().find(|r| r.source == e.source) {
            r.notes
                .push(format!("art contract: {}: {}", e.action.name(), e.what));
        }
    }
    (conformed, warnings)
}

/// Saves the cook index and collects the texture cache's garbage.
pub(super) fn save(
    cook_index: &index::Index,
    report: &Report,
    sources: &[Source],
    assets_dir: &Path,
    texture_cache: Option<&Path>,
) {
    let assets_path = assets_dir.canonicalize().ok();
    let bounded = sources.iter().all(|s| {
        s.kind != Kind::Video
            && s.gltf.as_ref().is_none_or(|g| {
                g.external.iter().all(|(name, _)| {
                    s.path
                        .parent()
                        .unwrap_or(assets_dir)
                        .join(name)
                        .canonicalize()
                        .ok()
                        .zip(assets_path.as_ref())
                        .is_some_and(|(path, assets)| path.starts_with(assets))
                })
            })
    });
    cook_index.save(report, bounded);
    if let Some(path) = texture_cache {
        let _ = cache::Store::new(path.to_owned()).gc_if_due(cache::max_bytes());
    }
}
