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
    pub(crate) style_tokens_hash: [u8; 8],
    pub(crate) story_functions: &'a BTreeMap<String, og_story::yarn::types::Function>,
    pub(crate) story_functions_stamp: &'a [u8],
    pub(crate) document_themes: &'a [og_ui::Theme],
    /// Each component name by the id of the document defining it: the
    /// game's own, else a built-in `@ui/` one (the scan's).
    pub(crate) components: &'a BTreeMap<String, String>,
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
    let Cx {
        sources,
        cooked_dir,
        pack_faces,
        old_outputs,
        ..
    } = *cx;
    let mut state = PlanState {
        early_errors,
        parsed_refs: BTreeMap::new(),
        planned_textures: std::collections::HashSet::new(),
    };
    let mut plan: Vec<Planned> = Vec::new();
    for (si, s) in sources.iter().enumerate() {
        if s.error.is_some() {
            continue;
        }
        // These cookers consume only bytes/options. The other kinds resolve
        // relative references or write source-derived names/sub-asset ids;
        // their location remains part of the action, not the content hash.
        let hash = if s.kind.by_content {
            stamp(&[&s.hash.to_le_bytes(), ext(&s.rel).as_bytes()])
        } else {
            stamp(&[&s.hash.to_le_bytes(), s.rel.as_bytes()])
        }
        .to_le_bytes();
        let mut outs: Vec<Out> = Vec::new();
        (s.kind.plan)(cx, &mut state, s, hash, &mut outs);
        for (id, ty, st, work) in outs {
            let file = if pack_faces.contains(&id) {
                format!("{id}.{}", og_bundle::runtime::FONT_PACK_FACE)
            } else {
                output_file(&id)
            };
            plan.push(Planned {
                source: si,
                id,
                file,
                ty,
                stamp: st,
                work: Some(work),
            });
        }
    }
    // Up to date: the manifest's stamp and file agree, and the file still
    // holds the type and schema version the loader expects today (a schema
    // change without a cooker bump would otherwise keep a blob the loader
    // refuses). Each output's file is read for it, all at once.
    let fresh = each(&plan, threads, |p| {
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
    (plan, state)
}
