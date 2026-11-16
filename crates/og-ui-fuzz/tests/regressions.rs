//! Every input a fuzz target once failed on (`regressions/<target>/`, each
//! minimised with `cargo fuzz tmin`) runs clean: no panic. Fails when a
//! fixed crash comes back.

#[test]
fn every_document_that_once_crashed_runs_clean() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("regressions/document");
    let mut failed = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "no inputs in {}", dir.display());
    for path in entries {
        let bytes = std::fs::read(&path).unwrap();
        if std::panic::catch_unwind(|| og_ui_fuzz::document(&bytes)).is_err() {
            failed.push(path.file_name().unwrap().to_string_lossy().into_owned());
        }
    }
    assert!(failed.is_empty(), "these inputs panic: {failed:?}");
}
