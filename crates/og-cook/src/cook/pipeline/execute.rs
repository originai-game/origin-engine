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
