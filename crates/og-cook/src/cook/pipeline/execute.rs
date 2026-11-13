use super::*;

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
        Work::Fracture(s) => Ok(fracture::cook(s, p.stamp)),
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
