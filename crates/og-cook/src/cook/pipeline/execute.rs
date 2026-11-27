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

/// The bytes of the asset at `rel` under `assets/` beside `s`.
pub(crate) fn read_sibling_bytes(s: &Source, rel: &str) -> Result<Vec<u8>, String> {
    let root = crate::cook::assets_root(s);
    if rel.split('/').any(|c| c == ".." || c.is_empty()) {
        return Err(format!("'{rel}' is not a path under assets/"));
    }
    std::fs::read(root.join(rel)).map_err(|e| format!("{e}"))
}
