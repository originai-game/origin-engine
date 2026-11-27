use super::*;

/// The texture cache's key for the project's outputs.
pub(crate) fn project_key(
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
            // No area's revision: each output's stamp, which its key in
            // the cache holds beside this one, has its own areas'.
            cache::key(
                &[],
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
pub(crate) fn execute(
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
    // What is stale, each with its cost, the least work first (results
    // land by plan index, so the order changes no byte). A cook of many
    // outputs keeps every core on one output each; as the queue empties,
    // the large encodes left take the cores the others no longer use
    // (`lanes`), where the largest, started first on one thread, would
    // hold the whole cook to its own time.
    let mut todo: Vec<(usize, u64)> = (0..plan.len())
        .filter_map(|i| Some((i, plan[i].work.as_ref()?.cost(options))))
        .collect();
    todo.sort_by_key(|&(_, cost)| cost);
    // The cost of everything not yet cooked.
    let left = std::sync::atomic::AtomicU64::new(todo.iter().map(|t| t.1).sum());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let workers = threads.min(todo.len());
    // What each worker is cooking, and since when, for the progress lines.
    let running: Vec<Mutex<Option<(usize, Instant)>>> =
        (0..workers).map(|_| Mutex::new(None)).collect();
    let finished = std::sync::atomic::AtomicUsize::new(0);
    // Every worker holds a sender; the last one to finish wakes the ticker.
    let (alive, ended) = std::sync::mpsc::channel::<()>();
    std::thread::scope(|scope| {
        for worker in 0..workers {
            let (next, results, plan, schemas, known, todo, prefab_texts) =
                (&next, &results, plan, schemas, known, &todo, prefab_texts);
            let left = &left;
            let (running, finished, alive) = (&running, &finished, alive.clone());
            scope.spawn(move || {
                let _alive = alive;
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(&(i, cost)) = todo.get(k) else { break };
                    let p = &plan[i];
                    let lanes = lanes(
                        threads,
                        cost,
                        left.load(std::sync::atomic::Ordering::Relaxed),
                    );
                    let t = Instant::now();
                    *running[worker].lock().expect("no poisoning") = Some((i, t));
                    let start = t.duration_since(started).as_secs_f64() * 1000.0;
                    let r = action_cache::run(texture_cache, p, options, project_key, || {
                        texture::with_encode_threads(lanes, || {
                            run(p, options, schemas, known, prefab_texts, budgets)
                        })
                    });
                    left.fetch_sub(cost, std::sync::atomic::Ordering::Relaxed);
                    let took = t.elapsed();
                    let done = finished.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    if options.progress && took >= SLOW {
                        eprintln!(
                            "og cook: {done}/{} {} took {:.1} s",
                            todo.len(),
                            p.id,
                            took.as_secs_f64()
                        );
                    }
                    *running[worker].lock().expect("no poisoning") = None;
                    *results[i].lock().expect("no poisoning") =
                        Some((r, took.as_secs_f64() * 1000.0, start, worker as u32));
                }
            });
        }
        drop(alive);
        if options.progress && !todo.is_empty() {
            let (running, finished, total) = (&running, &finished, todo.len());
            scope.spawn(move || {
                let begun = Instant::now();
                while let Err(std::sync::mpsc::RecvTimeoutError::Timeout) = ended.recv_timeout(TICK)
                {
                    let longest = running
                        .iter()
                        .filter_map(|r| *r.lock().expect("no poisoning"))
                        .min_by_key(|&(_, since)| since);
                    let longest = longest
                        .map(|(i, since)| {
                            format!(
                                "; longest running: {} ({:.0} s)",
                                plan[i].id,
                                since.elapsed().as_secs_f64()
                            )
                        })
                        .unwrap_or_default();
                    eprintln!(
                        "og cook: {}/{total} outputs cooked in {:.0} s{longest}",
                        finished.load(std::sync::atomic::Ordering::Relaxed),
                        begun.elapsed().as_secs_f64()
                    );
                }
            });
        }
    });
    results
}

/// The plan and its results with the derived outputs and theirs
/// (`plan::derive`), each source's after its own: one plan again, in the
/// order a cook that planned them all from the start would have had.
pub(crate) fn with_derived<'a>(
    plan: Vec<Planned<'a>>,
    results: Vec<Mutex<Option<Done>>>,
    derived: Vec<Planned<'a>>,
    derived_results: Vec<Mutex<Option<Done>>>,
) -> (Vec<Planned<'a>>, Vec<Mutex<Option<Done>>>) {
    if derived.is_empty() {
        return (plan, results);
    }
    let mut merged = (
        Vec::with_capacity(plan.len() + derived.len()),
        Vec::with_capacity(plan.len() + derived.len()),
    );
    // Both are in source order: a source's outputs are one run of each.
    let mut own = plan.into_iter().zip(results).peekable();
    let mut more = derived.into_iter().zip(derived_results).peekable();
    while let Some((p, r)) = own.next() {
        let source = p.source;
        merged.0.push(p);
        merged.1.push(r);
        if own.peek().is_none_or(|(next, _)| next.source != source) {
            while let Some((p, r)) = more.next_if(|(p, _)| p.source == source) {
                merged.0.push(p);
                merged.1.push(r);
            }
        }
    }
    merged
}

/// The threads an output of `cost` may encode on, of the cook's `threads`,
/// when the outputs not yet cooked (itself among them) cost `left`: its
/// share of the work that is left, at least one. A share is taken as the
/// output starts and outputs already running keep theirs, so the cook may
/// run on up to twice its threads for as long as those take to end.
fn lanes(threads: usize, cost: u64, left: u64) -> usize {
    if cost == 0 || left == 0 {
        return 1;
    }
    let share = (threads as u128 * cost as u128).div_ceil(left as u128);
    (share as usize).clamp(1, threads)
}

/// An output or a stage that took this long gets its own progress line.
pub(crate) const SLOW: std::time::Duration = std::time::Duration::from_secs(5);

/// How often a long cook says how far it is.
const TICK: std::time::Duration = std::time::Duration::from_secs(10);

/// The reports and each source's outputs, with the aliases resolved.
pub(crate) struct Assembled {
    pub(crate) reports: Vec<AssetReport>,
    pub(crate) new_outputs: BTreeMap<usize, Vec<NewOutput>>,
    // The 3D pass's lines per output: this cook's, or for an output that
    // was up to date the ones its last cook recorded.
    pub(crate) log: ConformLog,
}

/// Assembles the report and each output's manifest entry and bytes.
pub(crate) fn assemble(
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
            kind: s.kind.name.to_string(),
            source_bytes: s.bytes.len() as u64
                + s.settings.stamp.len() as u64
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
        let (size, hash, sri) = match &bytes {
            Some(b) => (b.len() as u64, hash_bytes(b), integrity(b)),
            None => {
                let o = old_outputs[p.id.as_str()];
                (o.bytes, o.hash, o.integrity.clone())
            }
        };
        new_outputs.entry(p.source).or_default().push((
            ManifestOutput {
                id: p.id.clone(),
                ty: p.ty.to_string(),
                file: p.file.clone(),
                stamp: p.stamp,
                hash,
                integrity: sri,
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
pub(crate) struct Written {
    manifest: Manifest,
    keep: BTreeSet<String>,
}

/// Writes what changed and builds the manifest.
pub(crate) fn write(
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
        profile,
        ..
    } = *cx;
    let mut manifest = Manifest {
        cooker: COOKER.to_string(),
        profile,
        modules: Vec::new(),
        options: BTreeMap::from([
            ("font".to_string(), font_desc.to_string()),
            ("mesh".to_string(), mesh_desc.to_string()),
            ("texture".to_string(), tex_desc.to_string()),
        ]),
        assets: Vec::new(),
        streams: Vec::new(),
        requires: Vec::new(),
        motion: None,
        preload: Vec::new(),
        lightmaps: sources
            .iter()
            .enumerate()
            .filter(|(i, _)| reports[*i].error.is_none())
            .filter_map(|(_, s)| {
                s.lightmap
                    .as_ref()
                    .map(|p| (p.file.scene.clone(), s.rel.clone()))
            })
            .collect(),
    };
    let mut keep: BTreeSet<String> = BTreeSet::new();
    for (si, outs) in new_outputs {
        let s = &sources[si];
        if reports[si].error.is_some() {
            continue;
        }
        let mut asset = ManifestAsset {
            source: s.rel.clone(),
            kind: s.kind.name.to_string(),
            source_hash: s.hash,
            source_bytes: reports[si].source_bytes,
            modules: Vec::new(),
            dependencies: Vec::new(),
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
    // A selected cook must not delete or claim to rebuild another pack's
    // previous payloads. Their old stamps remain available for its next cook.
    let mut positions: BTreeMap<_, _> = manifest
        .assets
        .iter()
        .enumerate()
        .map(|(i, a)| (a.source.clone(), i))
        .collect();
    for asset in old.assets.iter().filter(|a| a.kind != probes::KIND) {
        let mut retained = asset.clone();
        retained.outputs.clear();
        for output in &asset.outputs {
            if !cx.delivery.contains(&output.id, &asset.source)?
                && cooked_dir.join(&output.file).is_file()
            {
                keep.insert(output.file.clone());
                retained.outputs.push(output.clone());
            }
        }
        if !retained.outputs.is_empty() {
            if let Some(&i) = positions.get(&asset.source) {
                manifest.assets[i].outputs.extend(retained.outputs);
            } else {
                positions.insert(asset.source.clone(), manifest.assets.len());
                manifest.assets.push(retained);
            }
        }
    }
    for group in &old.streams {
        if !cx.delivery.contains(&group.scene, &group.scene)?
            && cooked_dir.join(&group.file).is_file()
        {
            keep.insert(group.file.clone());
            manifest.streams.push(group.clone());
        }
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
pub(crate) fn pack(
    cx: &Cx,
    groups: &BTreeMap<String, stream::GroupDecl>,
    written: &mut Written,
    reports: &mut [AssetReport],
) -> Result<(), CookError> {
    let cooked_dir = cx.cooked_dir;
    let Written { manifest, keep } = written;
    for (name, g) in groups {
        if !cx.delivery.contains(&g.scene, &g.scene)? {
            continue;
        }
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
            integrity(&pack),
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
/// output. Returns what it deleted, the motion model's warnings and what it
/// held before it read the project's code.
pub(crate) fn finish(
    cx: &Cx,
    old: &Manifest,
    written: Written,
    reports: &mut [AssetReport],
    log: &ConformLog,
    threads: usize,
) -> Result<(Vec<String>, Vec<String>, BeforeCode), CookError> {
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
    let uses_motion = motion::uses_motion(root);
    manifest.motion = motion::cook(uses_motion, cooked_dir, &mut keep, &mut motion_warnings)?;
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
    let mut index = assets::build(root, &manifest, sources, threads)?;
    let mut runtime_refs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for asset in &index.assets {
        runtime_refs
            .entry(asset.source.clone())
            .or_default()
            .extend(asset.refs.iter().cloned());
    }
    needs::record(
        &mut manifest,
        sources,
        &runtime_refs,
        cx.boot,
        cx.flow,
        cx.profile,
    );
    let before_code = BeforeCode {
        uses_motion,
        roots: index.roots.clone(),
        users: index.users.clone(),
        computed: index.computed.clone(),
    };
    assets::add_code(root, &manifest, &mut index, &cx.options.code.files)?;
    manifest.preload = index.font_preloads();
    let text = manifest.to_text();
    if std::fs::read_to_string(&manifest_path).ok().as_deref() != Some(text.as_str()) {
        std::fs::write(&manifest_path, text)
            .map_err(|e| CookError(format!("{}: {e}", manifest_path.display())))?;
    }
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
    Ok((removed, motion_warnings, before_code))
}

/// What a cook holds once everything but the project's code is read: the
/// asset index's roots, users and computed loads without the code's, and
/// whether the code used og.motion. Code is read by the asset index and
/// the motion pack alone, so a cook after a change to code alone starts
/// from this ([`cook_code`]).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct BeforeCode {
    uses_motion: bool,
    roots: BTreeSet<String>,
    users: BTreeMap<String, BTreeSet<String>>,
    computed: Vec<assets::ComputedLoad>,
}

/// The cook after a change to code alone: the asset index and the publish
/// plan are written again from the last cook's manifest and assets, which
/// are as they were. `false` when it cannot answer and the whole cook must:
/// the code uses og.motion, or did (its model is read from outside the
/// project, and the manifest names it), or the last cook's files do not
/// read back. Code that reaches other built-in UI is another
/// [`CookOptions::code`], which the index never takes for a change to code
/// alone.
pub(crate) fn cook_code(
    root: &Path,
    before: &BeforeCode,
    code: &CodeGraph,
) -> Result<bool, CookError> {
    // Delivery manifests and cross-pack roots must be re-linked after code
    // changes. Native payload stamps still avoid rebuilding those assets.
    if !source_project::Project::open(root)
        .map_err(|e| CookError(e.to_string()))?
        .input
        .packs
        .is_empty()
    {
        return Ok(false);
    }
    let cooked_dir = root.join("cooked");
    let (Some(mut manifest), Ok(last)) = (
        std::fs::read_to_string(cooked_dir.join(MANIFEST))
            .ok()
            .and_then(|t| Manifest::parse(&t).ok()),
        assets::Index::read(root),
    ) else {
        return Ok(false);
    };
    if before.uses_motion || motion::uses_motion(root) {
        return Ok(false);
    }
    let mut index = assets::Index {
        assets: last.assets,
        roots: before.roots.clone(),
        users: before.users.clone(),
        computed: before.computed.clone(),
    };
    assets::add_code(root, &manifest, &mut index, &code.files)?;
    let preload = index.font_preloads();
    if manifest.preload != preload {
        manifest.preload = preload;
        let path = cooked_dir.join(MANIFEST);
        std::fs::write(&path, manifest.to_text())
            .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
    }
    index.write(&cooked_dir)?;
    index.write_publish_plan(root, &manifest)?;
    Ok(true)
}

/// Each model's diagnostics (its skin report and emissive check) as notes
/// on its report, and the warnings among them. They run on cached cooks as
/// well, so a warning does not disappear on a rerun: each model's on its
/// own (its emissive check decodes two of its images), on up to `threads`
/// threads.
pub(crate) fn diagnose_models(cx: &Cx, reports: &mut [AssetReport], threads: usize) -> Vec<String> {
    let Cx { sources, rigs, .. } = *cx;
    let mut warnings = Vec::new();
    let models: Vec<Option<&Source>> = sources
        .iter()
        .zip(reports.iter())
        .map(|(source, report)| report.error.is_none().then_some(source))
        .collect();
    let diagnosed = each(&models, threads, |source| {
        let source = (*source)?;
        let gltf = source.gltf.as_ref()?;
        Some(anim_settings(source).and_then(|settings| {
            let mut notes = model::skin_report::report(gltf, rigs, settings.rig.as_deref())?;
            notes.extend(gltf.emissive_warnings()?);
            Ok(notes)
        }))
    });
    for ((source, report), notes) in sources.iter().zip(reports.iter_mut()).zip(diagnosed) {
        match notes {
            Some(Ok(notes)) => {
                for note in notes {
                    if note.starts_with("warning:") {
                        warnings.push(format!("assets/{}: {note}", source.rel));
                    }
                    report.notes.push(note);
                }
            }
            Some(Err(error)) => {
                let note = format!("model diagnostics unavailable: {error}");
                warnings.push(format!("assets/{}: {note}", source.rel));
                report.notes.push(note);
            }
            None => {}
        }
    }
    warnings
}

/// The cook's warnings and the art contract's entries: diagnostics describe
/// source assets and never change their cooked blobs.
pub(crate) fn diagnose(
    cx: &Cx,
    plan: &[Planned],
    reports: &mut [AssetReport],
    log: &ConformLog,
    contract: Option<&(String, crate::look::LookFile)>,
    mut conformed: Vec<ConformEntry>,
    mut warnings: Vec<String>,
) -> (Vec<ConformEntry>, Vec<String>) {
    let Cx {
        sources, options, ..
    } = *cx;
    // Across the string tables: every locale has every key, with its arguments.
    let tables: Vec<(&str, &og_i18n::StringTable)> = sources
        .iter()
        .filter_map(|s| s.strings.as_ref().map(|t| (s.rel.as_str(), t)))
        .collect();
    // Hyphenation in a locale its pack has no dictionary for, and a
    // locale written without spaces that no pack carries a dictionary for.
    for (rel, what) in font_pack::check_hyphenation(sources, &options.code)
        .into_iter()
        .chain(font_pack::check_segmentation(sources))
    {
        warnings.push(format!("assets/{rel}: {what}"));
        if let Some(r) = reports.iter_mut().find(|r| r.source == rel) {
            r.notes.push(what);
        }
    }
    for (rel, what) in strings::check_strings(&tables) {
        warnings.push(format!("assets/{rel}: {what}"));
        if let Some(r) = reports.iter_mut().find(|r| r.source == rel) {
            r.notes.push(what);
        }
    }
    // Dialogues and quests: every key they show is in the base table.
    let stories: Vec<(&str, &og_story::asset::Asset)> = sources
        .iter()
        .filter_map(|s| s.story.as_ref().map(|asset| (s.rel.as_str(), asset)))
        .collect();
    for (rel, what) in story::check_story_keys(&tables, &stories) {
        warnings.push(format!("assets/{rel}: {what}"));
        if let Some(r) = reports.iter_mut().find(|r| r.source == rel) {
            r.notes.push(what);
        }
    }
    // Dialogues og.flow plays: their segments, defaults and lines. An error
    // of the dialogue's (the cook fails), on a cached cook too; a held
    // node's branch on the variables is a warning (the cook builds).
    let segment = |id: &str| {
        sources
            .iter()
            .find(|s| s.rel == id && s.kind == &timeline::TIMELINE)
            .map(|s| {
                timeline::parse(s)
                    .map_err(|e| e.0)
                    .and_then(|t| t.segment())
            })
    };
    // A video's length and keyframe interval, asked of ffprobe once a
    // video: every node that plays it asks, and a probe is a process.
    let probed: std::cell::RefCell<BTreeMap<String, Option<(f64, f64)>>> = Default::default();
    let video_info = |id: &str| -> Option<(f64, f64)> {
        if let Some(known) = probed.borrow().get(id) {
            return *known;
        }
        let info = sources
            .iter()
            .find(|s| s.rel == id && s.kind == &video::VIDEO)
            .and_then(|s| {
                let prepared = s.video.as_ref()?;
                let decl = &prepared.plan.decl;
                let seconds = prepared.plan.probe.duration.min(decl.max_seconds);
                let keyframe = if decl.stream { decl.keyframe } else { 0.0 };
                Some((seconds, keyframe))
            });
        probed.borrow_mut().insert(id.to_string(), info);
        info
    };
    let (flow_errors, flow_warnings) = story::check_flows(&stories, &segment, &video_info);
    for (rel, what) in flow_warnings {
        warnings.push(format!("assets/{rel}: {what}"));
        if let Some(r) = reports.iter_mut().find(|r| r.source == rel) {
            r.notes.push(what);
        }
    }
    for (rel, what) in flow_errors {
        if let Some(r) = reports.iter_mut().find(|r| r.source == rel) {
            match &mut r.error {
                Some(e) => {
                    e.push_str("; ");
                    e.push_str(&what);
                }
                None => r.error = Some(what),
            }
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
        .filter(|s| s.kind == &tilemap::TILEMAP && s.rel.to_ascii_lowercase().ends_with(".tmj"))
    {
        for what in tilemap::tiled_unread(&String::from_utf8_lossy(&s.bytes)) {
            warnings.push(format!("assets/{}: {what}", s.rel));
            if let Some(r) = reports.iter_mut().find(|r| r.source == s.rel) {
                r.notes.push(what);
            }
        }
    }
    // A name that says one type over a file of another.
    for (s, r) in sources.iter().zip(reports.iter_mut()) {
        if let Some(named) = scan::misnamed(&s.rel, s.kind) {
            let what = format!(
                "named as a {named} (.{named}.ogt) and cooked as a {}: rename it or fix its type line",
                r.kind
            );
            warnings.push(format!("assets/{}: {what}", s.rel));
            r.notes.push(what);
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
                        k if k == &crate::cook::model::MODEL => true,
                        k if k == &flipbook::FLIPBOOK => c.motion.is_some(),
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
pub(crate) fn save(
    cook_index: &index::Index,
    report: &Report,
    before_code: BeforeCode,
    sources: &[Source],
    assets_dir: &Path,
    texture_cache: Option<&Path>,
) {
    let assets_path = assets_dir.canonicalize().ok();
    let bounded = sources.iter().all(|s| {
        s.kind != &video::VIDEO
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
    cook_index.save(report, bounded.then_some(before_code));
    if let Some(path) = texture_cache {
        let _ = cache::Store::new(path.to_owned()).gc_if_due(cache::max_bytes());
    }
}
