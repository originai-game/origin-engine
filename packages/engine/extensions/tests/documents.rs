use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-documents")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    root
}

/// The library's screen documents (packages/engine/src/ui/screens, docs/ui.md
/// §11): every one cooks under the name its file gives, its art slots are
/// specs the art brief takes, and the README's table names one document or
/// more for every row of docs/ui.md Appendix B, in its order; no document is
/// left with a slot waiting for a pattern.
#[test]
fn the_library_screens_cook_and_cover_every_screen_of_appendix_b() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = repo.join("packages/engine/src/ui/screens");
    let kits = repo.join("packages/engine/extensions");
    let ui = std::fs::read_to_string(repo.join("docs/ui.md")).unwrap();
    let appendix: Vec<String> = ui
        .split("## Appendix B. Screens")
        .nth(1)
        .unwrap()
        .lines()
        .skip_while(|l| !l.starts_with("|---"))
        .skip(1)
        .take_while(|l| l.starts_with('|'))
        .map(|l| l.split('|').nth(1).unwrap().trim().to_string())
        .collect();
    assert_eq!(appendix.len(), 47, "Appendix B's rows");
    let readme = std::fs::read_to_string(kits.join("tests/screen-catalogue.md")).unwrap();
    // | row | `doc`, `doc` |
    let table: Vec<(String, Vec<String>)> = readme
        .lines()
        .skip_while(|l| !l.starts_with("|---"))
        .skip(1)
        .take_while(|l| l.starts_with('|'))
        .map(|l| {
            let cells: Vec<&str> = l.split('|').map(str::trim).collect();
            let docs = cells[2]
                .split(',')
                .map(|d| d.trim().trim_matches('`').to_string())
                .collect();
            (cells[1].to_string(), docs)
        })
        .collect();
    let rows: Vec<&String> = table.iter().map(|(r, _)| r).collect();
    assert_eq!(
        rows,
        appendix.iter().collect::<Vec<_>>(),
        "README.md's rows are Appendix B's"
    );
    let mut files: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| {
            let name = e.unwrap().file_name().into_string().unwrap();
            name.strip_suffix(".ogt").map(str::to_string)
        })
        .collect();
    for entry in std::fs::read_dir(&kits).unwrap() {
        let kit = entry.unwrap().path();
        if !kit.join("extension.json").is_file() {
            continue;
        }
        for entry in std::fs::read_dir(&kit).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "ogt")
                && std::fs::read_to_string(&path)
                    .unwrap()
                    .starts_with("type screen\n")
            {
                files.push(format!(
                    "{}/{}",
                    kit.file_name().unwrap().to_str().unwrap(),
                    path.file_stem().unwrap().to_str().unwrap()
                ));
            }
        }
    }
    files.sort();
    let mut named: Vec<String> = table.iter().flat_map(|(_, d)| d.clone()).collect();
    named.sort();
    assert_eq!(
        named, files,
        "every document is in the table once, and every name in it is a file"
    );
    let texts: Vec<(String, String)> = files
        .iter()
        .map(|f| {
            let text = std::fs::read_to_string(if f.contains('/') {
                kits.join(format!("{f}.ogt"))
            } else {
                dir.join(format!("{f}.ogt"))
            })
            .unwrap();
            (format!("ui/{f}.ogt"), text)
        })
        .collect();
    let pairs: Vec<(&str, &str)> = texts
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    let root = project("screens", &pairs);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{}", report.errors().join("\n"));
    for (id, text) in &texts {
        let doc = og_ui::UiDocument::parse(id, text).unwrap();
        let stem = id.rsplit('/').next().unwrap().trim_end_matches(".ogt");
        assert_eq!(doc.name, stem, "{id}: its name is its file's");
        assert_eq!(doc.kind, og_ui::doc::DocKind::Screen, "{id}");
        assert!(
            !doc.props.is_empty(),
            "{id}: its viewmodel contract (props) is written"
        );
        // Code opens a slot by `<screen>/<slot id>`, so no two share an id.
        let mut slots: Vec<&str> = doc
            .nodes
            .iter()
            .filter(|n| n.kind == og_ui::doc::NodeKind::Element("slot"))
            .map(|n| n.id.as_str())
            .collect();
        let count = slots.len();
        slots.sort_unstable();
        slots.dedup();
        assert_eq!(
            slots.len(),
            count,
            "{id}: its slots' ids are unique: {slots:?}"
        );
        for s in og_ui::slot::document_slots(&doc) {
            let problems = s.spec.validate();
            assert!(problems.is_empty(), "{id}:{}: {problems:?}", s.line);
        }
        assert!(
            !text.contains("PATTERN PENDING"),
            "{id}: every slot names the og.ui component or pattern that fills it"
        );
    }
}
