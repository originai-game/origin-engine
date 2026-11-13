use super::*;

pub(super) fn plan_emitter<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { known, .. } = *cx;
    let PlanState { parsed_refs, .. } = state;
    let text = String::from_utf8_lossy(&s.bytes);
    let refs: Vec<String> = crate::fx::emitter_references(&text)
        .into_iter()
        .inspect(|r| {
            parsed_refs
                .entry(s.rel.clone())
                .or_default()
                .insert(r.clone());
        })
        .map(|r| format!("{r}={}", known.contains(crate::runtime::asset_id(&r))))
        .collect();
    let fields = fx_field::stamp(s, &text);
    let st = stamp(&[
        b"emitter",
        &hash,
        refs.join("\n").as_bytes(),
        fields.as_bytes(),
    ]);
    outs.push((s.rel.clone(), format::Emitter::NAME, st, Work::Emitter(s)));
}

pub(super) fn plan_effect<'a>(
    cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { known, .. } = *cx;
    let text = String::from_utf8_lossy(&s.bytes);
    let refs: Vec<String> = og_bundle::effect::effect_references(&text)
        .into_iter()
        .map(|r| format!("{r}={}", known.contains(crate::runtime::asset_id(&r))))
        .collect();
    let st = stamp(&[b"effect", &hash, refs.join("\n").as_bytes()]);
    outs.push((s.rel.clone(), format::Effect::NAME, st, Work::Effect(s)));
}

pub(super) fn plan_vfx<'a>(
    cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { known, .. } = *cx;
    let st = vfx::stamp_inputs(s, &|id| known.contains(id));
    let st = stamp(&[b"vfx", &hash, st.as_bytes()]);
    outs.push((s.rel.clone(), format::Vfx::NAME, st, Work::Vfx(s)));
}

pub(super) fn plan_shader<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // The stamp covers the WGSL file the shader names.
    let text = String::from_utf8_lossy(&s.bytes);
    let wgsl = crate::shader::shader_source(&text)
        .map(|src| read_sibling(s, &src).unwrap_or_default())
        .unwrap_or_default();
    let st = stamp(&[b"shader", &hash, wgsl.as_bytes()]);
    outs.push((s.rel.clone(), format::Shader::NAME, st, Work::Shader(s)));
}

pub(super) fn plan_post<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let text = String::from_utf8_lossy(&s.bytes);
    let wgsl = crate::shader::shader_source(&text)
        .map(|src| read_sibling(s, &src).unwrap_or_default())
        .unwrap_or_default();
    let st = stamp(&[b"post", &hash, wgsl.as_bytes()]);
    outs.push((s.rel.clone(), format::Post::NAME, st, Work::Post(s)));
}

pub(super) fn plan_ui_material<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let text = String::from_utf8_lossy(&s.bytes);
    let wgsl = crate::shader::shader_source(&text)
        .map(|src| execute::read_beside(s, &src).unwrap_or_default())
        .unwrap_or_default();
    let st = stamp(&[b"ui-material", &hash, wgsl.as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::UiMaterial::NAME,
        st,
        Work::UiMaterial(s),
    ));
}
