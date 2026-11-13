use super::*;
use og_ui::prelude::*;

pub(crate) fn run(
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
            let (blob, mut n) = model::cook(g, ids, &mesh, p.stamp, false, &s.rel)?;
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
            model::cook(g, &[], &mesh, p.stamp, true, &s.rel).map(|(b, mut n)| {
                n.notes.retain(|x| x.contains("triangles"));
                (
                    b,
                    n.notes.into_iter().map(|x| format!("proxy: {x}")).collect(),
                )
            })
        }
        Work::Impostor(s, b, kind) => {
            let g = s.gltf.as_ref().expect("loaded");
            let baked = g
                .impostor(*b)
                .map_err(|e| CookError(format!("{}: impostor: {e}", s.rel)))?;
            let (img, k, what) = match kind {
                0 => (&baked.albedo, format::kind::COLOR, "albedo"),
                1 => (&baked.normal, format::kind::LINEAR, "normal and depth"),
                _ => (
                    &baked.material,
                    format::kind::LINEAR,
                    "roughness, translucency and metallic",
                ),
            };
            let note = format!(
                "{}: {}x{} impostor {what} ({}×{} frames)",
                p.id, img.width, img.height, b.grid, b.grid
            );
            texture::cook(img, k, &options.texture, p.stamp).map(|t| (t, vec![note]))
        }
        Work::Animation(s, a) => {
            let g = s.gltf.as_ref().expect("loaded");
            let mesh = mesh_for(s, options)?;
            anim::cook(g, &s.bytes, &mesh, a, p.stamp, &assets_root(s))
        }
        Work::GltfTexture(s, image, k, cutout, split) => {
            let id = p.id.strip_suffix("#mips").unwrap_or(&p.id);
            let cook = |stamp| {
                let g = s.gltf.as_ref().expect("loaded");
                let img = texture::decode(&g.images[*image])
                    .map_err(|e| CookError(format!("{id}: image {image}: {e}")))?;
                let mut notes = vec![format!(
                    "{id}: {}x{} image {image}{}",
                    img.width,
                    img.height,
                    cutout
                        .map(|c| format!(
                            " (cut-out at alpha {c}: an alpha-tested material's base colour)"
                        ))
                        .unwrap_or_default()
                )];
                let mut t = options.texture.clone();
                t.cutout = *cutout;
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
                        notes.extend(note.map(|n| format!("{id}: {n}")));
                        (img, k)
                    }
                    None => (img, *k),
                };
                if t.cutout.is_some() && t.mipmaps && (t.force_toktx || !cfg!(feature = "basis")) {
                    notes.push(format!(
                        "{id}: a cut-out with toktx's own mips (the basis-universal encoder, og-cook's `basis` \
                         feature, makes the ones that keep its coverage)"
                    ));
                }
                texture::cook(&img, k, &t, stamp).map(|b| (b, notes))
            };
            split_texture(s, *image, split, p, cook)
        }
        Work::Texture(s, k, split) if s.rel.ends_with(".ktx2") => {
            let explicit = settings_block(s, "texture")
                .and_then(|b| b.get("kind"))
                .map(|_| *k);
            split_texture(s, 0, split, p, |stamp| {
                ktx2_source::cook(&s.bytes, explicit, &options.texture, stamp)
            })
        }
        Work::Texture(s, k, split) => split_texture(s, 0, split, p, |stamp| {
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
                return texture::cook(&img, k, &t, stamp).map(|b| (b, notes));
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
            texture::cook(&img, *k, &options.texture, stamp).map(|b| (b, vec![note]))
        }),
        Work::TextureNormal(s, file) => {
            let img = match file {
                Some(bytes) => texture::decode(bytes).map_err(CookError)?,
                None => super::normal2d::auto(
                    &texture::decode(s.conformed.as_deref().unwrap_or(&s.bytes))
                        .map_err(CookError)?,
                ),
            };
            let note = format!(
                "{}x{} 2D normal map ({})",
                img.width,
                img.height,
                if file.is_some() { "its file" } else { "auto" }
            );
            texture::cook(&img, format::kind::NORMAL, &options.texture, p.stamp)
                .map(|b| (b, vec![note]))
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
            let baked = fx_field::bake(s, text).map_err(CookError)?;
            let text = baked.as_str();
            let missing = std::cell::RefCell::new(Vec::new());
            let check = |id: &str| {
                if !known.contains(crate::runtime::asset_id(id)) {
                    missing.borrow_mut().push(id.to_string());
                }
            };
            let em = crate::fx::parse_emitter_linked(
                text,
                &mut crate::fx::EmitterLinks {
                    texture: &mut |id| {
                        check(id);
                        Ok(None)
                    },
                    array: &mut |id| {
                        check(id);
                        Ok(None)
                    },
                    mesh: &mut |m| {
                        if let crate::fx::ParticleMesh::Asset(id) = m {
                            check(id);
                        }
                        Ok(None)
                    },
                    emitter: &mut |id| {
                        check(id);
                        Ok(None)
                    },
                    effect: &mut |id| {
                        check(id);
                        Ok(None)
                    },
                },
            )
            .map_err(CookError)?;
            let missing = missing.into_inner();
            if !missing.is_empty() {
                return Err(CookError(format!(
                    "the emitter names asset(s) not under assets/ (a texture, texture array, mesh model or sub-emitter): {}",
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
        Work::Document(s, uses) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a document is text (UTF-8)".into()))?;
            let imports = document_imports(s, text).map_err(CookError)?;
            let doc = og_ui::UiDocument::parse_with_imports(&s.rel, text, imports.clone())
                .map_err(|e| CookError(e.join("\n")))?;
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
            if !imports.is_empty() {
                let bytes = serde_json::to_vec(&imports).expect("strings serialize");
                w.raw("imports", bytes, 16);
            }
            if !uses.is_empty() {
                let bytes = serde_json::to_vec(uses).expect("strings serialize");
                w.raw("uses", bytes, 16);
            }
            let slots = doc
                .nodes
                .iter()
                .filter(|n| matches!(n.kind, og_ui::doc::NodeKind::Render { .. }))
                .count();
            let mut notes = doc.warnings.clone();
            notes.push(format!(
                "{} '{}', {} node(s), {} timeline(s), {slots} content render(s)",
                match doc.kind {
                    og_ui::doc::DocKind::Screen => "screen",
                    og_ui::doc::DocKind::Component => "component",
                },
                doc.name,
                doc.nodes.len(),
                doc.timelines.len()
            ));
            Ok((w.finish(), notes))
        }
        Work::Style(s) => {
            let files = serde_json::from_slice(&s.bytes)
                .map_err(|e| CookError(format!("{}: {e}", s.rel)))?;
            let pack = og_ui::style_pack::StylePack::parse(&s.rel, files).map_err(CookError)?;
            let missing: Vec<_> = pack
                .assets
                .iter()
                .filter(|r| !known.contains(r.as_str()))
                .collect();
            if !missing.is_empty() {
                return Err(CookError(format!(
                    "{}: CSS url() assets are missing: {} (URLs resolve beside the CSS file)",
                    s.rel,
                    missing
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
            let mut w = og_blob::BlobWriter::new(
                format::Style::type_hash(),
                format::Style::schema_version(),
                p.stamp,
            );
            w.table(
                "meta",
                &[format::StyleMeta {
                    text_bytes: s.bytes.len() as u32,
                }],
                format::row_hash::<format::StyleMeta>(),
            )
            .raw("text", s.bytes.clone(), 16);
            Ok((
                w.finish(),
                vec![format!(
                    "CSS pack '{}', {} asset(s)",
                    pack.name,
                    pack.assets.len()
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
        Work::Rive(s, name) => riv::cook(s, name, p.stamp),
        Work::RiveImage(s, n) => riv::cook_image(s, *n, &options.texture, p.stamp),
        Work::RiveFont(f) => riv::cook_font(f, threads, p.stamp),
        Work::UiMaterial(s) => {
            let text = std::str::from_utf8(&s.bytes)
                .map_err(|_| CookError("a UI material is text (UTF-8)".into()))?;
            let name = s.rel.trim_end_matches(".ogt");
            let material =
                crate::shader::parse_ui_material(name, text, &mut |src| read_beside(s, src))
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
        Work::FontPack(s) => font_pack::cook(s, p.stamp),
        Work::Rig(s) => anim::cook_rig(&s.bytes, p.stamp),
        Work::MotionAction(s) => {
            anim::cook_motion_action(&s.bytes, p.stamp, &|id| known.contains(id))
        }
        Work::Terrain(s) => terrain::cook_terrain(s, &|id| known.contains(id), p.stamp),
        Work::Fracture(s) => Ok(fracture::cook(s, p.stamp)),
        Work::Timeline(s) => timeline::cook_timeline(s, &|id| known.contains(id), p.stamp),
        Work::Effect(s) => effect::cook_effect(s, &|id| known.contains(id), p.stamp),
        Work::Vfx(s) => vfx::cook_vfx(s, &|id| known.contains(id), p.stamp),
        Work::Lightmap(s) => lightmap::cook_lightmap(s, p.stamp),
        Work::Environment(s) => environment::cook_environment(s, p.stamp),
        Work::Look(s) => look::cook_look(s, &assets_root(s), &|id| known.contains(id), p.stamp),
        Work::Palette(s) => look::cook_palette(s, p.stamp),
        Work::TextureArray(s, decl) => {
            let root = crate::cook::assets_root(s);
            texture_array::cook(&s.rel, &root, decl, &options.texture, p.stamp)
        }
        Work::ArrayLayer(s, decl, n) => {
            let root = crate::cook::assets_root(s);
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
        Work::Job(job) => job.cook(
            p,
            &Exec {
                options,
                threads,
                schemas,
                known,
                prefab_texts,
                budgets,
            },
        ),
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

/// A texture output by how it stands to its source's mip chain (G3-3):
/// `cook` makes the whole chain's blob under a stamp and its notes. A
/// streamed texture's tail and `#mips` share one cook of the whole chain
/// (under the `#mips` output's stamp, [`Source::whole_texture`]): `#mips`
/// is that blob as it is, without the notes, which its tail reports with
/// its own line.
fn split_texture(
    s: &Source,
    image: usize,
    split: &Split,
    p: &Planned,
    cook: impl FnOnce(u64) -> Result<(Vec<u8>, Vec<String>), CookError>,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    match split {
        Split::Whole(why) => cook(p.stamp).map(|(b, mut notes)| {
            notes.extend(why.iter().map(|w| format!("{}: {w}", p.id)));
            (b, notes)
        }),
        Split::Mips => s
            .whole_texture(image, p.stamp, || cook(p.stamp))
            .map(|(b, _)| (b, Vec::new())),
        Split::Tail(mips) => {
            let (whole, mut notes) = s.whole_texture(image, *mips, || cook(*mips))?;
            let (tail, note) = texture::tail(&whole, p.stamp)?;
            notes.push(format!("{}: {note}", p.id));
            Ok((tail, notes))
        }
    }
}

/// The text of `rel` (an asset path) under the assets directory `s` came from.
/// A file a source names, read relative to the source's own directory
/// first (`foil.wgsl` beside `materials/materials/foil.ogt`, so a pack's
/// relative references survive installation, docs/ui.md §5), then as an
/// asset path from `assets/` ([`read_sibling`]), as a style resolves its
/// references.
pub(crate) fn read_beside(s: &Source, rel: &str) -> Result<String, String> {
    if let Some((dir, _)) = s.rel.rsplit_once('/')
        && let Ok(text) = read_sibling(s, &format!("{dir}/{rel}"))
    {
        return Ok(text);
    }
    read_sibling(s, rel)
}

/// The stylesheets a document's `@import`s name, read from beside it
/// (and theirs, from beside them), by the path each was written as.
pub(crate) fn document_imports(s: &Source, text: &str) -> Result<Vec<(String, String)>, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut pending: Vec<(String, String)> = og_ui::UiDocument::imports(text)
        .into_iter()
        .map(|p| (s.rel.clone(), p))
        .collect();
    while let Some((from, path)) = pending.pop() {
        let rel = og_ui::doc::at_rules::resolve(&from, &path);
        if out.iter().any(|(p, _)| *p == rel) {
            continue;
        }
        let css = read_sibling(s, &rel).map_err(|e| format!("@import \"{path}\" ({rel}): {e}"))?;
        pending.extend(
            og_ui::UiDocument::imports(&css)
                .into_iter()
                .map(|p| (rel.clone(), p)),
        );
        out.push((rel, css));
    }
    Ok(out)
}

pub(crate) fn read_sibling(s: &Source, rel: &str) -> Result<String, String> {
    String::from_utf8(read_sibling_bytes(s, rel)?).map_err(|_| format!("'{rel}' is not UTF-8 text"))
}

/// The bytes of the asset at `rel` under `assets/` beside `s`.
pub(crate) fn read_sibling_bytes(s: &Source, rel: &str) -> Result<Vec<u8>, String> {
    // A built-in component's files are the engine's, not the game's.
    if rel.starts_with(ui_components::PREFIX) {
        return ui_components::read(rel);
    }
    let root = crate::cook::assets_root(s);
    if rel.split('/').any(|c| c == ".." || c.is_empty()) {
        return Err(format!("'{rel}' is not a path under assets/"));
    }
    std::fs::read(root.join(rel)).map_err(|e| format!("{e}"))
}
