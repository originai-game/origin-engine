use super::*;

pub(super) fn plan_texture<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        options,
        group_of,
        tex_desc,
        ..
    } = *cx;
    let PlanState { early_errors, .. } = state;
    match texture_kind(s).and_then(|k| Ok((k, texture_stream(s, k)?))) {
        Ok((k, stream)) => {
            // A mesh's texture under the art contract: its desc too.
            let mesh = mesh_texture(s, options)
                .map(|c| c.texture_desc())
                .unwrap_or_default();
            let kind = k.to_le_bytes();
            let st = |word: &[u8]| {
                let mut parts: Vec<&[u8]> = vec![b"texture", tex_desc.as_bytes(), &hash, &kind];
                if !mesh.is_empty() {
                    parts.push(mesh.as_bytes());
                }
                if !word.is_empty() {
                    parts.push(word);
                }
                stamp(&parts)
            };
            let source = s.conformed.as_deref().map_or(&s.bytes[..], |b| &b[..]);
            match stream.then(|| not_split(source, k, group_of.get(&s.rel))) {
                Some(None) => {
                    let mips = st(b"mips");
                    outs.push((
                        s.rel.clone(),
                        format::Texture::NAME,
                        st(b"tail"),
                        Work::Texture(s, k, Split::Tail(mips)),
                    ));
                    outs.push((
                        crate::runtime::mips_id(&s.rel),
                        format::Texture::NAME,
                        mips,
                        Work::Texture(s, k, Split::Mips),
                    ));
                }
                why => outs.push((
                    s.rel.clone(),
                    format::Texture::NAME,
                    st(b""),
                    Work::Texture(s, k, Split::Whole(why.flatten())),
                )),
            }
            // Its normal map for 2D lighting (`normal auto` or a file).
            match normal2d::setting(settings_block(s, "texture")) {
                Ok(None) => {}
                Ok(Some(setting)) => {
                    let file = match &setting {
                        normal2d::NormalSetting::Auto => Ok(None),
                        normal2d::NormalSetting::File(name) => {
                            let path = s.path.parent().unwrap_or(Path::new(".")).join(name);
                            std::fs::read(&path)
                                .map(Some)
                                .map_err(|e| format!("normal \"{name}\": {e}"))
                        }
                    };
                    match file {
                        Ok(file) => {
                            let st = stamp(&[
                                b"texture normal",
                                tex_desc.as_bytes(),
                                &hash,
                                file.as_deref().unwrap_or(b"auto"),
                            ]);
                            outs.push((
                                format!("{}#normal", s.rel),
                                format::Texture::NAME,
                                st,
                                Work::TextureNormal(s, file),
                            ));
                        }
                        Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
                    }
                }
                Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
            }
        }
        Err(e) => early_errors.push((s.rel.clone(), "texture", e)),
    }
}
