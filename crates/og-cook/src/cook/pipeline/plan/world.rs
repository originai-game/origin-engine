use super::*;

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
