use super::*;

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
