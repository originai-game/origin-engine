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
            texture::split_texture(s, *image, split, p, cook)
        }
        Work::Fracture(s) => Ok(fracture::cook(s, p.stamp)),
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
