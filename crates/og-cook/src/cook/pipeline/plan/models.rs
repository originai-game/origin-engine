use super::*;

pub(super) fn plan_model<'a>(
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
        mesh_desc,
        budget_desc,
        ..
    } = *cx;
    let PlanState {
        early_errors,
        parsed_refs,
        planned_textures,
        ..
    } = state;
    let g = s.gltf.as_ref().expect("loaded");
    // The art contract's part of each output's stamp: empty
    // (and left out) when it changes nothing here.
    let contract = options
        .mesh
        .conform
        .as_ref()
        .filter(|_| !conform_off(s, options));
    let model_desc = contract
        .map(|c| {
            let mut d = if c.applies(g) {
                c.model_desc()
            } else {
                String::new()
            };
            // An effect's mesh is held to no face budget.
            if !d.is_empty() && g.role == conform::ModelRole::Effect {
                d.push_str(" role effect");
            }
            // Its colour textures' kind is in the blob.
            if c.filter == conform::mesh::Filter::Nearest {
                d.push_str(" textures nearest");
            }
            d
        })
        .unwrap_or_default();
    let texture_desc = contract.map(|c| c.texture_desc()).unwrap_or_default();
    let with = |parts: &[&[u8]], desc: &str, extra: &[u8]| {
        let mut v: Vec<&[u8]> = parts.to_vec();
        if !desc.is_empty() {
            v.push(desc.as_bytes());
            v.push(extra);
        }
        stamp(&v)
    };
    let stream_textures = match model_stream_textures(s) {
        Ok(b) => b,
        Err(e) => {
            early_errors.push((s.rel.clone(), "model", e));
            return;
        }
    };
    let mut ids = Vec::new();
    let grouped = group_of.contains_key(&s.rel);
    for (i, u) in g.textures.iter().enumerate() {
        let image_hash = hash_bytes(&g.images[u.image]).to_le_bytes();
        // One cooked texture per image and use: its kind, and
        // whether it streams its levels (a model's
        // `stream_textures` splits it into a tail and a chain).
        // A model's own unless an art contract makes it depend
        // on the model (a density is per metre of the model) or
        // it is in a streaming group's pack.
        let shared = !grouped && texture_desc.is_empty();
        let id = if shared {
            crate::runtime::gltf_texture_id(
                u64::from_le_bytes(image_hash),
                u.kind,
                stream_textures,
                u.cutout,
            )
        } else {
            format!("{}#tex{i}", s.rel)
        };
        parsed_refs
            .entry(s.rel.clone())
            .or_default()
            .insert(id.clone());
        if shared && !planned_textures.insert(id.clone()) {
            ids.push(id);
            continue;
        }
        // A density is per metre of the model: its hash too.
        let kind = u.kind.to_le_bytes();
        let st = |word: &[u8]| {
            let mut parts: Vec<&[u8]> = vec![b"texture", tex_desc.as_bytes(), &image_hash, &kind];
            // A cut-out's mips are its own, kept at its cut-off
            // (texture.rs `cutout_mips`, `keep_coverage`).
            if let Some(c) = &u.cutout {
                parts.push(b"cutout");
                parts.push(std::slice::from_ref(c));
            }
            if !word.is_empty() {
                parts.push(word);
            }
            with(&parts, &texture_desc, &hash)
        };
        let not_split =
            stream_textures.then(|| not_split(&g.images[u.image], u.kind, group_of.get(&s.rel)));
        let gltf_texture = |split| Work::GltfTexture(s, u.image, u.kind, u.cutout, split);
        match not_split {
            Some(None) => {
                let mips = st(b"mips");
                outs.push((
                    id.clone(),
                    format::Texture::NAME,
                    st(b"tail"),
                    gltf_texture(Split::Tail(mips)),
                ));
                outs.push((
                    crate::runtime::mips_id(&id),
                    format::Texture::NAME,
                    mips,
                    gltf_texture(Split::Mips),
                ));
            }
            why => outs.push((
                id.clone(),
                format::Texture::NAME,
                st(b""),
                gltf_texture(Split::Whole(why.flatten())),
            )),
        }
        ids.push(id);
    }
    let joined = ids.join("\n");
    // A model written as text is refused past the budgets, so
    // they are among its inputs.
    let st = if ext(&s.rel) == "ogt" {
        with(
            &[
                b"model",
                NODE_ROWS,
                mesh_desc.as_bytes(),
                &hash,
                joined.as_bytes(),
                budget_desc.as_bytes(),
            ],
            &model_desc,
            &[],
        )
    } else {
        with(
            &[
                b"model",
                NODE_ROWS,
                mesh_desc.as_bytes(),
                &hash,
                joined.as_bytes(),
            ],
            &model_desc,
            &[],
        )
    };
    outs.insert(
        0,
        (s.rel.clone(), format::Model::NAME, st, Work::Model(s, ids)),
    );
    if s.fracture.is_some() {
        outs.push((
            format!("{}#fracture", s.rel),
            format::Fracture::NAME,
            // The asset's layout too: a new og_destruct version
            // recooks what the runtime would refuse.
            stamp(&[
                b"fracture",
                &og_destruct::ASSET_VERSION.to_le_bytes(),
                &hash,
            ]),
            Work::Fracture(s),
        ));
    }
    if group_of.contains_key(&s.rel) {
        let st = with(
            &[b"proxy", NODE_ROWS, mesh_desc.as_bytes(), &hash],
            &model_desc,
            &[],
        );
        outs.push((
            format!("{}#proxy", s.rel),
            format::Model::NAME,
            st,
            Work::Proxy(s),
        ));
    }
    // The impostor's three atlases (`model { impostor true }`),
    // baked from the model's source.
    if let Some(b) = mesh_for(s, options)
        .ok()
        .and_then(|m| m.impostor)
        .map(|b| b.resolved(g))
    {
        let size = [b.grid.to_le_bytes(), b.frame.to_le_bytes()].concat();
        for (kind, id) in model::impostor_ids(&s.rel).into_iter().enumerate() {
            // `6`: the atlas's layout version (2, the
            // hemi-octahedron; 3, 16 samples a texel, filtered
            // textures and the material atlas; 4, the leaves' dome
            // normals and the mean normal's length; 6, the sun's
            // share in the material's alpha, 5 an unlanded bake of
            // it), so an atlas baked by an older layout is baked
            // again.
            let st = with(
                &[
                    b"impostor",
                    &[6, kind as u8],
                    &size,
                    tex_desc.as_bytes(),
                    &hash,
                ],
                &model_desc,
                &[],
            );
            outs.push((
                id,
                format::Texture::NAME,
                st,
                Work::Impostor(s, b, kind as u8),
            ));
        }
    }
    if anim::wanted(g) {
        match anim_settings(s) {
            Ok(a) => {
                let deps = anim::dependencies(&assets_root(s), &a);
                let motion = contract
                    .and_then(|c| c.motion.as_ref())
                    .map(|m| m.describe())
                    .unwrap_or_default();
                let st = with(
                    &[
                        b"animation",
                        NODE_ROWS,
                        mesh_desc.as_bytes(),
                        a.describe().as_bytes(),
                        &hash,
                        deps.as_bytes(),
                    ],
                    &format!("{model_desc}{motion}"),
                    &[],
                );
                outs.insert(
                    1,
                    (
                        format!("{}#anim", s.rel),
                        format::Animation::NAME,
                        st,
                        Work::Animation(s, a),
                    ),
                );
            }
            Err(e) => early_errors.push((s.rel.clone(), "model", e)),
        }
    }
}
