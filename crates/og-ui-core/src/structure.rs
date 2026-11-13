//! The direction CSS and the document model depend on each other in:
//! `doc` reads its stylesheet with `css`, and `css` takes from `doc` only
//! the modules its parent file names.

use std::fs;
use std::path::{Path, PathBuf};

/// The one line through which `css` reaches the document model.
const FROM_DOC: &str = "use crate::doc::{build, schema, syntax, view_transition};";

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn css_names_what_it_takes_from_the_document_in_one_place() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let parent = src.join("css.rs");
    let mut files = vec![parent.clone()];
    sources(&src.join("css"), &mut files);
    let mut found = Vec::new();
    for file in &files {
        for (at, line) in fs::read_to_string(file).unwrap().lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") || !code.contains("crate::doc") {
                continue;
            }
            if *file == parent && code == FROM_DOC {
                continue;
            }
            found.push(format!("{}:{}: {code}", file.display(), at + 1));
        }
    }
    assert!(
        found.is_empty(),
        "css reaches the document model outside css.rs's one import:\n{}",
        found.join("\n")
    );
    assert!(
        fs::read_to_string(&parent).unwrap().contains(FROM_DOC),
        "css.rs no longer imports `{FROM_DOC}`; update this test with the new list"
    );
}
