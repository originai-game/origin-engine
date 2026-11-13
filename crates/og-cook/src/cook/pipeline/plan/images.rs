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

pub(super) fn plan_rive<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { options, .. } = *cx;
    let PlanState { early_errors, .. } = state;
    match riv::plan(s, &options.font) {
        Ok((names, images, fonts)) => {
            for f in fonts {
                // The font cook's version and settings, and the
                // file's bytes (a referenced font's own).
                let st = stamp(&[
                    b"rive font 1 font 9",
                    &hash,
                    &f.index.to_le_bytes(),
                    f.settings.describe().as_bytes(),
                    &hash_bytes(&f.bytes).to_le_bytes(),
                ]);
                outs.push((
                    format!("{}#font{}", s.rel, f.index),
                    format::Font::NAME,
                    st,
                    Work::RiveFont(Box::new(f)),
                ));
            }
            for n in images {
                let st = stamp(&[b"rive image", &hash, &n.to_le_bytes()]);
                outs.push((
                    format!("{}#image{n}", s.rel),
                    format::Texture::NAME,
                    st,
                    Work::RiveImage(s, n),
                ));
            }
            for name in names {
                let st = stamp(&[b"rive 3", &hash, name.as_bytes()]);
                outs.push((
                    format!("{}#{name}", s.rel),
                    format::RiveArtboard::NAME,
                    st,
                    Work::Rive(s, name),
                ));
            }
        }
        Err(e) => early_errors.push((s.rel.clone(), "rive", e)),
    }
}

pub(super) fn plan_flipbook<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        options, tex_desc, ..
    } = *cx;
    let PlanState { parsed_refs, .. } = state;
    let deps = flipbook::dependency_hashes(s);
    assets::record_dependencies(parsed_refs, &s.rel, &deps);
    let motion = options
        .mesh
        .conform
        .as_ref()
        .and_then(|c| c.motion.as_ref())
        .filter(|_| !conform_off(s, options))
        .map(|m| m.describe());
    let st = match &motion {
        Some(m) => stamp(&[b"flipbook", &hash, deps.as_bytes(), m.as_bytes()]),
        None => stamp(&[b"flipbook", &hash, deps.as_bytes()]),
    };
    outs.push((s.rel.clone(), format::Flipbook::NAME, st, Work::Flipbook(s)));
    // `array true`: the frames as the layers of a texture array.
    if let Ok(sheet) = flipbook::sheet(s)
        && sheet.array
    {
        let base = [
            b"flipbook array 1".as_slice(),
            tex_desc.as_bytes(),
            &hash,
            deps.as_bytes(),
        ];
        let sheet = Box::new(sheet);
        for n in 0..sheet.frames.len() {
            let st = stamp(&[&base[..], &[&(n as u32).to_le_bytes()[..]]].concat());
            outs.push((
                texture_array::layer_id(&flipbook::array_id(&s.rel), n),
                format::Texture::NAME,
                st,
                Work::FlipbookLayer(s, sheet.clone(), n),
            ));
        }
        outs.push((
            flipbook::array_id(&s.rel),
            format::TextureArray::NAME,
            stamp(&base),
            Work::FlipbookArray(s, sheet),
        ));
    }
}

pub(super) fn plan_texture_array<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { tex_desc, .. } = *cx;
    let PlanState {
        early_errors,
        parsed_refs,
        ..
    } = state;
    let text = String::from_utf8_lossy(&s.bytes);
    let root = crate::cook::assets_root(s);
    match texture_array::parse(&s.rel, &text, &|p| root.join(p).is_file()) {
        Ok(decl) => {
            let deps = texture_array::dependencies(&root, &decl);
            assets::record_dependencies(parsed_refs, &s.rel, &deps);
            let base = [
                b"texture-array 1".as_slice(),
                tex_desc.as_bytes(),
                &hash,
                deps.as_bytes(),
            ];
            outs.push((
                s.rel.clone(),
                format::TextureArray::NAME,
                stamp(&base),
                Work::TextureArray(s, decl.clone()),
            ));
            for n in 0..decl.layers.len() {
                let st = stamp(&[&base[..], &[&(n as u32).to_le_bytes()[..]]].concat());
                outs.push((
                    texture_array::layer_id(&s.rel, n),
                    format::Texture::NAME,
                    st,
                    Work::ArrayLayer(s, decl.clone(), n),
                ));
            }
        }
        Err(e) => early_errors.push((s.rel.clone(), "texture-array", e)),
    }
}
