use super::*;

/// `assets/`, from a source's path and its path under it.
/// The atlas bytes a style names: a built-in style's from the engine's styles,
/// a game's from its assets.
pub(super) fn style_atlas(id: &str, root: &Path) -> Option<Vec<u8>> {
    if id.starts_with('@') {
        look::styles()
            .iter()
            .find(|f| f.id() == id)
            .map(|f| f.bytes().to_vec())
    } else {
        std::fs::read(root.join(id)).ok()
    }
}

/// `tone` lines for the tinted frames of a style that sets none: the mean
/// of each frame's centre in the atlas, linear grey (premultiplied, over
/// its coverage) and alpha, where it is not white and opaque.
fn style_tones(file: &og_ui::StyleFile, root: &Path) -> Vec<String> {
    let Some(img) = file
        .atlas
        .as_deref()
        .and_then(|id| style_atlas(crate::runtime::asset_id(id), root))
        .and_then(|b| image::load_from_memory(&b).ok())
        .map(|i| i.to_rgba8())
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for part in og_ui::style_file::StylePart::ALL {
        let Some(sl) = file.theme.art.slices[part.index()] else {
            continue;
        };
        if sl.tone != og_ui::style_file::Slice::WHITE {
            continue;
        }
        let t = slice_tone(&img, &sl);
        if t.iter().any(|v| *v < 0.98) {
            // A version 2 style's tone is a line of its part's dressing.
            out.push(file.tone_text(part, t));
        }
    }
    out
}

/// The mean of a slice's centre in `img`: linear RGB over its coverage
/// (premultiplied, then divided by the mean alpha) and the mean alpha.
pub fn slice_tone(img: &image::RgbaImage, sl: &og_ui::style_file::Slice) -> [f32; 4] {
    let (w, h) = (img.width() as f32, img.height() as f32);
    let lin = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let (u0, v0) = (sl.uv[0] + sl.border_uv[0], sl.uv[1] + sl.border_uv[1]);
    let (u1, v1) = (sl.uv[2] - sl.border_uv[2], sl.uv[3] - sl.border_uv[3]);
    let (x0, y0) = ((u0 * w).round() as u32, (v0 * h).round() as u32);
    let x1 = ((u1 * w).round() as u32).max(x0 + 1).min(img.width());
    let y1 = ((v1 * h).round() as u32).max(y0 + 1).min(img.height());
    let (mut rgb, mut alpha, mut n) = ([0.0f64; 3], 0.0f64, 0.0f64);
    for y in y0..y1 {
        for x in x0..x1 {
            let p = img.get_pixel(x, y).0;
            let a = f64::from(p[3]) / 255.0;
            for k in 0..3 {
                rgb[k] += f64::from(lin(f32::from(p[k]) / 255.0)) * a;
            }
            alpha += a;
            n += 1.0;
        }
    }
    if n == 0.0 {
        return og_ui::style_file::Slice::WHITE;
    }
    let c = rgb.map(|v| (v / alpha.max(1e-6)) as f32);
    [c[0], c[1], c[2], (alpha / n) as f32]
}

pub(super) fn run(
    p: &Planned,
    options: &CookOptions,
    threads: usize,
    schemas: &BTreeMap<String, Schema>,
    known: &BTreeSet<String>,
    prefab_texts: &BTreeMap<String, String>,
    budgets: &stats::Budgets,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    match p.work.as_ref().expect("planned work") {
        Work::Model(s, ids) => {
            let g = s.gltf.as_ref().expect("loaded");
            let mesh = mesh_for(s, options)?;
            let (blob, mut n) = model::cook(g, ids, &mesh, p.stamp, false)?;
            // A model written as text past the project's budgets is cooked
            // and flagged: the budgets are reported numbers, never a reason
            // a model is not made (indices are u32; nothing breaks past them).
            if ext(&s.rel) == "ogt" {
                for (what, value, limit, key) in [
                    (
                        "LOD 0 triangles",
                        n.triangles,
                        budgets.triangles,
                        "triangles",
                    ),
                    ("vertices", n.vertices, budgets.vertices, "vertices"),
                ] {
                    if value as u64 > limit {
                        n.notes.push(format!(
                            "warning: over budget: {what}: {value} is over the {key} budget of {limit} (project.ogt's budgets.{key}); cooked as written"
                        ));
                    }
                }
            }
            Ok((blob, n.notes))
        }
        Work::Proxy(s) => {
            let g = s.gltf.as_ref().expect("loaded");
            let mesh = mesh_for(s, options)?;
            model::cook(g, &[], &mesh, p.stamp, true).map(|(b, mut n)| {
                n.notes.retain(|x| x.contains("triangles"));
                (
                    b,
                    n.notes.into_iter().map(|x| format!("proxy: {x}")).collect(),
                )
            })
        }
        Work::Animation(s, a) => {
            let g = s.gltf.as_ref().expect("loaded");
            let mesh = mesh_for(s, options)?;
            anim::cook(g, &s.bytes, &mesh, a, p.stamp, &assets_root(s))
        }
        Work::GltfTexture(s, image, k) => {
            let g = s.gltf.as_ref().expect("loaded");
            let img = texture::decode(&g.images[*image])
                .map_err(|e| CookError(format!("{}: image {image}: {e}", p.id)))?;
            let mut notes = vec![format!(
                "{}: {}x{} image {image}",
                p.id, img.width, img.height
            )];
            let mut t = options.texture.clone();
            let (img, k) = match options
                .mesh
                .conform
                .as_ref()
                .filter(|_| !conform_off(s, options))
            {
                Some(c) => {
                    // The span of the image's longest side, from the span of
                    // a side of its square equivalent (the check's measure).
                    let (w, h) = (img.width as f32, img.height as f32);
                    let extent = conform::mesh::texel_span(g, *image).map_or_else(
                        || conform::mesh::extent(g),
                        |s| s * w.max(h) / (w * h).sqrt(),
                    );
                    let (img, k, mips, note) =
                        conform::mesh::texture(c, Some(extent), img, *k, t.mipmaps);
                    t.mipmaps = mips;
                    notes.extend(note.map(|n| format!("{}: {n}", p.id)));
                    (img, k)
                }
                None => (img, *k),
            };
            texture::cook(&img, k, &t, p.stamp).map(|b| (b, notes))
        }
        Work::Texture(s, k) if s.rel.ends_with(".ktx2") => {
            let explicit = settings_block(s, "texture")
                .and_then(|b| b.get("kind"))
                .map(|_| *k);
            ktx2_source::cook(&s.bytes, explicit, &options.texture, p.stamp)
        }
        Work::Texture(s, k) => {
            let img =
                texture::decode(s.conformed.as_deref().unwrap_or(&s.bytes)).map_err(CookError)?;
            if let Some(c) = mesh_texture(s, options) {
                let mut t = options.texture.clone();
                let (img, k, mips, note) = conform::mesh::texture(&c, None, img, *k, t.mipmaps);
                t.mipmaps = mips;
                let mut notes = vec![format!("{}x{} as a mesh's texture", img.width, img.height)];
                notes.extend(note.map(|n| match n.strip_prefix("conform: ") {
                    Some(w) if w != conform::mesh::UNCHANGED => {
                        format!("conform: {}; {w}", conform::mesh::DEFAULT_MESH_TEXTURE)
                    }
                    _ => n,
                }));
                return texture::cook(&img, k, &t, p.stamp).map(|b| (b, notes));
            }
            let note = format!(
                "{}x{} {}",
                img.width,
                img.height,
                match *k {
                    format::kind::NORMAL => "normal (UASTC)",
                    format::kind::LINEAR => "linear (UASTC)",
                    format::kind::PIXEL => "pixel art (RGBA8, no mips)",
                    _ => "colour (ETC1S)",
                }
            );
            texture::cook(&img, *k, &options.texture, p.stamp).map(|b| (b, vec![note]))
        }
        Work::Sound(s, _) if s.rel.ends_with(".ogt") => {
            sfx::cook(&s.bytes, p.stamp).map(|(b, n)| (b, vec![n]))
        }
        Work::Sound(s, chunking) => {
            let e = s.encoded(chunking.map(|c| c.0))?;
            audio::cook(&e, *chunking, p.stamp).map(|(b, n)| (b, vec![n]))
        }
        Work::SoundChunk(s, per, n) => {
            let e = s.encoded(Some(*per))?;
            audio::cook_chunk(&e, *n, p.stamp).map(|b| (b, Vec::new()))
        }
        Work::Lut(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a .cube LUT is text (UTF-8)".into()))?;
            let lut = crate::fx::parse_cube(text).map_err(CookError)?;
            let mut w = og_blob::BlobWriter::new(
                format::Lut::type_hash(),
                format::Lut::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::LutMeta { size: lut.size }],
                format::row_hash::<format::LutMeta>(),
            )
            .raw("rgba", lut.rgba, 16);
            Ok((w.finish(), vec![format!("{0}³ colour table", lut.size)]))
        }
        Work::Emitter(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("an emitter is text (UTF-8)".into()))?;
            let missing = std::cell::RefCell::new(Vec::new());
            let check = |id: &str| {
                if !known.contains(crate::runtime::asset_id(id)) {
                    missing.borrow_mut().push(id.to_string());
                }
            };
            let em = crate::fx::parse_emitter_with(
                text,
                &mut |id| {
                    check(id);
                    Ok(None)
                },
                &mut |id| {
                    check(id);
                    Ok(None)
                },
            )
            .map_err(CookError)?;
            let missing = missing.into_inner();
            if !missing.is_empty() {
                return Err(CookError(format!(
                    "the emitter names texture(s) or texture array(s) not under assets/: {}",
                    missing.join(", ")
                )));
            }
            let mut w = og_blob::BlobWriter::new(
                format::Emitter::type_hash(),
                format::Emitter::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::EmitterMeta {
                    text_bytes: text.len() as u32,
                }],
                format::row_hash::<format::EmitterMeta>(),
            )
            .raw("text", text.as_bytes().to_vec(), 16);
            Ok((
                w.finish(),
                vec![format!(
                    "{} particles, {} burst(s)",
                    em.max_particles,
                    em.bursts.len()
                )],
            ))
        }
        Work::Shader(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a shader is text (UTF-8)".into()))?;
            let name = s.rel.trim_end_matches(".ogt");
            let shader = crate::shader::parse_shader(name, text, &mut |src| read_sibling(s, src))
                .map_err(CookError)?;
            let compiled = shader_check::validate_shader(&shader)
                .map_err(|e| CookError(format!("the shader does not compile: {e}")))?;
            let canonical = crate::shader::shader_text(&shader);
            let mut w = og_blob::BlobWriter::new(
                format::Shader::type_hash(),
                format::Shader::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::ShaderMeta {
                    text_bytes: canonical.len() as u32,
                }],
                format::row_hash::<format::ShaderMeta>(),
            )
            .raw("text", canonical.into_bytes(), 16);
            Ok((
                w.finish(),
                vec![format!(
                    "{} parameter(s); compiled for {}",
                    shader.params.len(),
                    compiled.join(", ")
                )],
            ))
        }
        Work::Post(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a post pass is text (UTF-8)".into()))?;
            let name = s.rel.trim_end_matches(".ogt");
            let post = crate::shader::parse_post(name, text, &mut |src| read_sibling(s, src))
                .map_err(CookError)?;
            let compiled = shader_check::validate_post(&post)
                .map_err(|e| CookError(format!("the post pass does not compile: {e}")))?;
            let canonical = crate::shader::post_text(&post);
            let mut w = og_blob::BlobWriter::new(
                format::Post::type_hash(),
                format::Post::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::PostMeta {
                    text_bytes: canonical.len() as u32,
                }],
                format::row_hash::<format::PostMeta>(),
            )
            .raw("text", canonical.into_bytes(), 16);
            Ok((
                w.finish(),
                vec![format!(
                    "{} parameter(s); compiled for {}",
                    post.params.len(),
                    compiled.join(", ")
                )],
            ))
        }
        Work::Document(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a document is text (UTF-8)".into()))?;
            let doc =
                og_ui::UiDocument::parse(&s.rel, text).map_err(|e| CookError(e.join("\n")))?;
            let mut w = og_blob::BlobWriter::new(
                format::Document::type_hash(),
                format::Document::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::DocumentMeta {
                    text_bytes: text.len() as u32,
                }],
                format::row_hash::<format::DocumentMeta>(),
            )
            .raw("text", text.as_bytes().to_vec(), 16);
            let slots = doc
                .nodes
                .iter()
                .filter(|n| matches!(n.kind, og_ui::doc::NodeKind::Element("slot" | "art")))
                .count();
            Ok((
                w.finish(),
                vec![format!(
                    "{} '{}', {} element(s), {} timeline(s), {slots} slot(s)",
                    match doc.kind {
                        og_ui::doc::DocKind::Screen => "screen",
                        og_ui::doc::DocKind::Component => "component",
                    },
                    doc.name,
                    doc.nodes.len(),
                    doc.timelines.len()
                )],
            ))
        }
        Work::Style(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a style is text (UTF-8)".into()))?;
            let mut file = og_ui::parse_style(text).map_err(CookError)?;
            file.resolve(&s.rel, |id| known.contains(crate::runtime::asset_id(id)));
            let companions: Vec<String> = file.fonts[4..].iter().flatten().cloned().collect();
            let missing: Vec<String> = file
                .references()
                .into_iter()
                // A Chinese companion is cooked in only for a game that shows Chinese.
                .filter(|r| !known.contains(crate::runtime::asset_id(r)) && !companions.contains(r))
                .collect();
            if !missing.is_empty() {
                let dir = s.rel.rsplit_once('/').map_or("", |(d, _)| d);
                return Err(CookError(format!(
                    "the style names asset(s) neither beside it in {dir}/ nor under assets/: {} (an id is read relative to the style file first, then from assets/; a `fallback` font is the project's own: point it at a CJK font the game has, or remove the line, og-taste §1)",
                    missing.join(", ")
                )));
            }
            // Each tinted frame's centre as the atlas draws it, for a style
            // that did not measure it (og-ui picks button label ink by it).
            let tones = style_tones(&file, &assets_root(s));
            let text = if tones.is_empty() {
                text.to_string()
            } else {
                format!(
                    "{text}\n# Measured by og cook from the atlas (each tinted frame's centre).\n{}\n",
                    tones.join("\n")
                )
            };
            let text = text.as_str();
            let mut w = og_blob::BlobWriter::new(
                format::Style::type_hash(),
                format::Style::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::StyleMeta {
                    text_bytes: text.len() as u32,
                }],
                format::row_hash::<format::StyleMeta>(),
            )
            .raw("text", text.as_bytes().to_vec(), 16);
            let parts = file.theme.art.slices.iter().flatten().count();
            Ok((
                w.finish(),
                vec![format!(
                    "style '{}' version {}, {parts} frame(s), {} dressing(s)",
                    file.name,
                    file.version,
                    file.dressings.len()
                )],
            ))
        }
        Work::Tileset(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a tileset is text (UTF-8)".into()))?;
            tilemap::cook_tileset(text, &|id| known.contains(id), p.stamp)
        }
        Work::Tilemap(s, m) => {
            tilemap::cook_tilemap(&s.rel, &s.bytes, m, &|id| known.contains(id), p.stamp)
        }
        Work::Flipbook(s) => flipbook::cook(
            s,
            options
                .mesh
                .conform
                .as_ref()
                .and_then(|c| c.motion.as_ref())
                .filter(|_| !conform_off(s, options)),
            p.stamp,
        ),
        Work::Rig2d(s) => rig2d::cook(s, p.stamp),
        Work::Rig2dAtlas(s) => {
            rig2d::cook_atlas(s, &options.texture, p.stamp).map(|b| (b, Vec::new()))
        }
        Work::FlipbookArray(s, sheet) => flipbook::cook_array(s, sheet, &options.texture, p.stamp),
        Work::FlipbookLayer(s, sheet, n) => {
            flipbook::cook_layer(s, sheet, *n, &options.texture, p.stamp).map(|b| (b, Vec::new()))
        }
        Work::Icons(s) => icons::cook(s, p.stamp),
        Work::Vector(s) => vector::cook(s, p.stamp),
        Work::UiMaterial(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a UI material is text (UTF-8)".into()))?;
            let name = s.rel.trim_end_matches(".ogt");
            let material =
                crate::shader::parse_ui_material(name, text, &mut |src| read_sibling(s, src))
                    .map_err(CookError)?;
            let compiled = shader_check::validate_ui_material(&material)
                .map_err(|e| CookError(format!("the UI material does not compile: {e}")))?;
            let canonical = crate::shader::ui_material_text(&material);
            let mut w = og_blob::BlobWriter::new(
                format::UiMaterial::type_hash(),
                format::UiMaterial::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::UiMaterialMeta {
                    text_bytes: canonical.len() as u32,
                }],
                format::row_hash::<format::UiMaterialMeta>(),
            )
            .raw("text", canonical.into_bytes(), 16);
            Ok((
                w.finish(),
                vec![format!(
                    "{} parameter(s); compiled for {}",
                    material.params.len(),
                    compiled.join(", ")
                )],
            ))
        }
        Work::Behaviour(s) => e6::cook_behaviour(s, p.stamp),
        Work::Dialogue(s) => e6::cook_story(s, p.stamp, false),
        Work::Quest(s) => e6::cook_story(s, p.stamp, true),
        Work::Strings(s) => e6::cook_strings(s, p.stamp),
        Work::Rig(s) => anim::cook_rig(&s.bytes, p.stamp),
        Work::NavMesh(s) => e6::cook_navmesh(s, p.stamp),
        Work::Terrain(s) => terrain::cook_terrain(s, &|id| known.contains(id), p.stamp),
        Work::Timeline(s) => timeline::cook_timeline(s, &|id| known.contains(id), p.stamp),
        Work::Lightmap(s) => lightmap::cook_lightmap(s, p.stamp),
        Work::Environment(s) => environment::cook_environment(s, p.stamp),
        Work::Look(s) => look::cook_look(s, &assets_root(s), &|id| known.contains(id), p.stamp),
        Work::Palette(s) => look::cook_palette(s, p.stamp),
        Work::TextureArray(s, decl) => {
            let root = texture_array::assets_dir(&s.path, &s.rel);
            texture_array::cook(&s.rel, &root, decl, &options.texture, p.stamp)
        }
        Work::ArrayLayer(s, decl, n) => {
            let root = texture_array::assets_dir(&s.path, &s.rel);
            texture_array::cook_layer(&root, decl, *n, &options.texture, p.stamp)
                .map(|b| (b, Vec::new()))
        }
        Work::VideoMeta(plan) => video::cook_meta(plan, p.stamp),
        Work::VideoArray(s, plan) => Ok((
            video::cook_array(&s.rel, plan, &options.texture, p.stamp),
            Vec::new(),
        )),
        Work::VideoFrame(src, h, plan, n) => {
            video::cook_frame(src, *h, plan, *n, &options.texture, p.stamp).map(|b| (b, Vec::new()))
        }
        Work::VideoStream(src, h, plan) => video::cook_stream(src, *h, plan, p.stamp),
        Work::VideoPoster(src, h, plan) => {
            video::cook_poster(src, *h, plan, &options.texture, p.stamp).map(|b| (b, Vec::new()))
        }
        Work::VideoAudio(src, plan) => {
            let wav = video::audio_wav(src, plan)?;
            let pcm =
                audio::wav(&format!("{}'s sound track", src.display()), &wav).map_err(CookError)?;
            // A stream's sound is long: Opus (96 kb/s stereo), a clip's QOA.
            let e = if plan.decl.stream {
                let bitrate = audio::SoundSettings::default().opus_bitrate(pcm.channels);
                audio::encode_opus(&pcm, bitrate).map_err(CookError)?
            } else {
                audio::encode(&pcm, None).map_err(CookError)?
            };
            audio::cook(&e, None, p.stamp)
                .map(|(b, n)| (b, vec![format!("the video's sound: {n}")]))
        }
        Work::Font(s, f) => {
            font::cook(&s.bytes, f, threads, p.stamp).map(|(b, n)| (b, vec![n.note]))
        }
        Work::Scene(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("scene is not UTF-8".into()))?;
            let doc = Document::parse(text).map_err(|e| CookError(e.to_string()))?;
            let texts = |id: &str| prefab_texts.get(id).cloned();
            scene::cook(
                &doc,
                &s.rel,
                schemas,
                &|id| known.contains(id),
                &scene::Prefabs {
                    texts: &texts,
                    publish: options.publish,
                },
                p.stamp,
            )
        }
    }
}

/// The text of `rel` (an asset path) under the assets directory `s` came from.
pub(super) fn read_sibling(s: &Source, rel: &str) -> Result<String, String> {
    let mut root = s.path.clone();
    for _ in s.rel.split('/') {
        root.pop();
    }
    if rel.split('/').any(|c| c == ".." || c.is_empty()) {
        return Err(format!("'{rel}' is not a path under assets/"));
    }
    std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{e}"))
}
