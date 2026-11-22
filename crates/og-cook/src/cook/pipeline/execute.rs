use super::*;

pub(crate) fn run(
    p: &Planned,
    options: &CookOptions,
    schemas: &BTreeMap<String, Schema>,
    known: &BTreeSet<String>,
    prefab_texts: &BTreeMap<String, String>,
    budgets: &stats::Budgets,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    p.work.as_ref().expect("planned work").cook(
        p,
        &Exec {
            options,
            schemas,
            known,
            prefab_texts,
            budgets,
        },
    )
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

pub(crate) fn read_sibling(s: &Source, rel: &str) -> Result<String, String> {
    String::from_utf8(read_sibling_bytes(s, rel)?).map_err(|_| format!("'{rel}' is not UTF-8 text"))
}

/// The bytes of the asset at `rel` under `assets/` beside `s`.
pub(crate) fn read_sibling_bytes(s: &Source, rel: &str) -> Result<Vec<u8>, String> {
    let root = crate::cook::assets_root(s);
    if rel.split('/').any(|c| c == ".." || c.is_empty()) {
        return Err(format!("'{rel}' is not a path under assets/"));
    }
    std::fs::read(root.join(rel)).map_err(|e| format!("{e}"))
}
