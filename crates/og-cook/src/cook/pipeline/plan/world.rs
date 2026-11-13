use super::*;

pub(super) fn plan_tileset<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { known, .. } = *cx;
    let PlanState { parsed_refs, .. } = state;
    let deps = tilemap::references_of_tileset(&s.bytes, &|id| known.contains(id));
    assets::record_dependencies(parsed_refs, &s.rel, &deps);
    let st = stamp(&[b"tileset", &hash, deps.as_bytes()]);
    outs.push((s.rel.clone(), format::Tileset::NAME, st, Work::Tileset(s)));
}

pub(super) fn plan_tilemap<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { known, .. } = *cx;
    let PlanState {
        early_errors,
        parsed_refs,
        ..
    } = state;
    match tilemap::MapSettings::parse(settings_block(s, "tilemap")) {
        Ok(m) => {
            let refs = tilemap::references(&s.rel, &s.bytes, &m, &|id| known.contains(id));
            assets::record_dependencies(parsed_refs, &s.rel, &refs.join("\n"));
            let st = stamp(&[
                b"tilemap",
                &hash,
                m.describe().as_bytes(),
                refs.join("\n").as_bytes(),
            ]);
            outs.push((
                s.rel.clone(),
                format::Tilemap::NAME,
                st,
                Work::Tilemap(s, m),
            ));
        }
        Err(e) => early_errors.push((s.rel.clone(), "tilemap", e)),
    }
}

pub(super) fn plan_rig<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[b"rig", &hash]);
    outs.push((s.rel.clone(), format::Rig::NAME, st, Work::Rig(s)));
}

pub(super) fn plan_motion_action<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[b"motion-action", &hash]);
    outs.push((
        s.rel.clone(),
        format::MotionAction::NAME,
        st,
        Work::MotionAction(s),
    ));
}

pub(super) fn plan_look<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        known, assets_dir, ..
    } = *cx;
    let PlanState { parsed_refs, .. } = state;
    let deps = look::look_dependencies(s, assets_dir, &|id| known.contains(id));
    assets::record_dependencies(parsed_refs, &s.rel, &deps);
    let st = stamp(&[b"look 1", &hash, deps.as_bytes()]);
    outs.push((s.rel.clone(), format::Look::NAME, st, Work::Look(s)));
}

pub(super) fn plan_palette<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[b"palette 1", &hash]);
    outs.push((s.rel.clone(), format::Palette::NAME, st, Work::Palette(s)));
}

pub(super) fn plan_scene<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        options,
        known,
        prefab_texts,
        schema_hash,
        ..
    } = *cx;
    let PlanState { parsed_refs, .. } = state;
    let text = String::from_utf8_lossy(&s.bytes);
    // Every prefab it reaches, transitively, with its text's
    // hash: editing a prefab re-cooks what places it.
    let texts = |id: &str| prefab_texts.get(id).cloned();
    let reached = prefab::reached(&text, &texts);
    let mut refs: BTreeSet<String> = scene::references(&text).into_iter().collect();
    let mut placed = Vec::new();
    for id in &reached {
        let t = prefab_texts.get(id);
        if let Some(t) = t {
            refs.extend(scene::references(t));
        }
        placed.push(format!(
            "{id}#{}",
            t.map_or(0, |t| hash_bytes(t.as_bytes()))
        ));
    }
    parsed_refs
        .entry(s.rel.clone())
        .or_default()
        .extend(refs.iter().chain(reached.iter()).cloned());
    let refs: Vec<String> = refs
        .into_iter()
        .map(|r| format!("{r}={}", known.contains(&r)))
        .collect();
    // A scene that places no prefab keeps the stamp it had.
    let st = if placed.is_empty() && s.kind == Kind::Scene {
        stamp(&[b"scene", &hash, &schema_hash, refs.join("\n").as_bytes()])
    } else {
        stamp(&[
            b"scene",
            &hash,
            &schema_hash,
            refs.join("\n").as_bytes(),
            placed.join("\n").as_bytes(),
            &[options.publish as u8],
        ])
    };
    outs.push((s.rel.clone(), format::Scene::NAME, st, Work::Scene(s)));
}
