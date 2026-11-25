use super::*;

/// What every stage after the art contract reads.
#[derive(Clone, Copy)]
pub(crate) struct Cx<'a> {
    pub(crate) root: &'a Path,
    pub(crate) assets_dir: &'a Path,
    pub(crate) cooked_dir: &'a Path,
    pub(crate) options: &'a CookOptions,
    pub(crate) sources: &'a [Source],
    pub(crate) known: &'a BTreeSet<String>,
    pub(crate) prefab_texts: &'a BTreeMap<String, String>,
    pub(crate) group_of: &'a BTreeMap<String, String>,
    pub(crate) pack_faces: &'a BTreeSet<String>,
    pub(crate) old_outputs: &'a BTreeMap<&'a str, &'a ManifestOutput>,
    pub(crate) schemas: &'a BTreeMap<String, Schema>,
    pub(crate) budgets: &'a stats::Budgets,
    pub(crate) tex_desc: &'a str,
    pub(crate) mesh_desc: &'a str,
    pub(crate) font_desc: &'a str,
    pub(crate) budget_desc: &'a str,
    pub(crate) schema_hash: [u8; 8],
    pub(crate) story_functions: &'a BTreeMap<String, og_story::yarn::types::Function>,
    pub(crate) story_functions_stamp: &'a [u8],
    /// The project's profile (its creation form's): the manifest's.
    pub(crate) profile: crate::look::Profile,
    /// The boot scene's asset id (project.ogt `scene` through `scenes`), or
    /// `None` when project.ogt names none: the manifest's `modules` are
    /// then every asset's.
    pub(crate) boot: Option<&'a str>,
    /// project.ogt's `flow` dialogue, which the bundle needs beside its boot
    /// scene.
    pub(crate) flow: Option<&'a str>,
    /// The shared action cache and the project's key in it
    /// (`action_cache`), which a plan may read before anything cooks.
    pub(crate) texture_cache: Option<&'a Path>,
    pub(crate) project_key: Option<&'a str>,
    /// project.ogt's `cook { uses }`: the assets other cooked bundles provide.
    pub(crate) provided: &'a provided::Provided,
}

/// What planning writes as it goes: the errors it finds, the references it
/// parses and the glTF textures already planned.
pub(crate) struct PlanState {
    pub(crate) early_errors: Vec<EarlyError>,
    pub(crate) parsed_refs: BTreeMap<String, BTreeSet<String>>,
    // glTF textures of models outside streaming groups are cooked once per
    // image and use, whichever models use them: a scene exported in many
    // glTFs (cells, tiles) repeats its images in every file, and a copy per
    // model multiplied the cooked bytes, the load's transcoding and the
    // GPU's memory (og-runtime's add_gltf shares them by these ids).
    pub(crate) planned_textures: std::collections::HashSet<String>,
}

/// One planned output: its id, type, stamp and work.
pub(crate) type Out<'a> = (String, &'static str, u64, Box<dyn Job + 'a>);

// Plan every output with its stamp.
pub(crate) fn plan<'a>(
    cx: &Cx<'a>,
    early_errors: Vec<EarlyError>,
    threads: usize,
) -> (Vec<Planned<'a>>, PlanState) {
    let sources = cx.sources;
    let mut state = PlanState {
        early_errors,
        parsed_refs: BTreeMap::new(),
        planned_textures: std::collections::HashSet::new(),
    };
    // These cookers consume only bytes/options. The other kinds resolve
    // relative references or write source-derived names/sub-asset ids;
    // their location remains part of the action, not the content hash.
    let hash = |s: &Source| {
        if s.kind.by_content {
            stamp(&[], &[&s.hash.to_le_bytes(), ext(&s.rel).as_bytes()])
        } else {
            stamp(&[], &[&s.hash.to_le_bytes(), s.rel.as_bytes()])
        }
        .to_le_bytes()
    };
    let mut plan: Vec<Planned> = Vec::new();
    for (si, s) in sources.iter().enumerate() {
        if s.error.is_some() {
            continue;
        }
        let mut outs: Vec<Out> = Vec::new();
        (s.kind.plan)(cx, &mut state, s, hash(s), &mut outs);
        plan.extend(outs.into_iter().map(|out| planned(cx, si, out)));
    }
    keep_fresh(cx, &mut plan, threads);
    (plan, state)
}

/// A source's output as the plan holds it.
fn planned<'a>(cx: &Cx<'a>, source: usize, (id, ty, stamp, work): Out<'a>) -> Planned<'a> {
    let file = if cx.pack_faces.contains(&id) {
        format!("{id}.{}", og_bundle::runtime::FONT_PACK_FACE)
    } else {
        output_file(&id)
    };
    Planned {
        source,
        id,
        file,
        ty,
        stamp,
        work: Some(work),
    }
}

/// The outputs only a cook could name ([`Job::derived`]), planned once the
/// first round has cooked: each after its source's own, with what is up to
/// date left without work, as [`plan`] leaves it. An output whose cook
/// failed derives nothing; its error is the first round's.
pub(crate) fn derive<'a>(
    cx: &Cx<'a>,
    plan: &[Planned<'a>],
    parsed_refs: &mut BTreeMap<String, BTreeSet<String>>,
    threads: usize,
) -> Vec<Planned<'a>> {
    let ex = Exec {
        options: cx.options,
        schemas: cx.schemas,
        known: cx.known,
        prefab_texts: cx.prefab_texts,
        budgets: cx.budgets,
    };
    // An output that cooked answers from what its cook kept; one that came
    // from the shared cache cooks for the answer here, so all at once.
    let derived = each(plan, threads, |p| {
        p.work.as_ref().and_then(|work| work.derived(&ex).ok())
    });
    let mut more: Vec<Planned> = Vec::new();
    for (p, outs) in plan.iter().zip(derived) {
        for out in outs.into_iter().flatten() {
            parsed_refs
                .entry(cx.sources[p.source].rel.clone())
                .or_default()
                .insert(out.0.clone());
            more.push(planned(cx, p.source, out));
        }
    }
    keep_fresh(cx, &mut more, threads);
    more
}

/// Takes the work from every planned output that is up to date.
fn keep_fresh(cx: &Cx, plan: &mut [Planned], threads: usize) {
    let Cx {
        cooked_dir,
        old_outputs,
        ..
    } = *cx;
    // Up to date: the manifest's stamp and file agree, and the file still
    // holds the type and schema version the loader expects today (a schema
    // change without a cooker bump would otherwise keep a blob the loader
    // refuses). Each output's file is read for it, all at once.
    let fresh = each(plan, threads, |p| {
        let want = format::expected_header(p.ty);
        old_outputs.get(p.id.as_str()).is_some_and(|o| {
            o.stamp == p.stamp
                && o.ty == p.ty
                && std::fs::read(cooked_dir.join(&o.file)).is_ok_and(|b| {
                    b.len() as u64 == o.bytes
                        && hash_bytes(&b) == o.hash
                        && og_blob::Blob::parse(&b, &p.file).is_ok_and(|blob| {
                            let h = blob.header();
                            want == Some((h.type_hash, h.schema_version))
                        })
                })
        })
    });
    for (p, fresh) in plan.iter_mut().zip(fresh) {
        if fresh {
            p.work = None;
        }
    }
}
