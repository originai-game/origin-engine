use super::*;

/// `og ui add racing` installs an editable copy of the kit; `og ui eject
/// racing/race-results` then copies the installed copy's document (the
/// game's edit with it) into assets/ui/racing/, with no second copy of the
/// kit's code, manifest, skill or sheet, and refuses to overwrite it
/// without --force.
///
/// Ways it could fail: the bundled document ejected over the game's edit,
/// the kit's TypeScript or sheet copied beside the document (a stray second
/// kit in the game's assets), or an existing ejected edit overwritten.
#[test]
fn ui_eject_takes_the_installed_kit_document_and_only_its_companions() {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let run = |args: &[&str]| {
        let out = og().args(args).arg(&dir).output().unwrap();
        (out.status.code(), text(&out))
    };
    let (code, out) = run(&["ui", "add", "racing"]);
    assert_eq!(code, Some(0), "{out}");
    let installed = dir.join("assets/ui-kits/racing/race-results.ogt");
    let edited = std::fs::read_to_string(&installed)
        .unwrap()
        .replace("text \"Pos\"", "text \"Position\"");
    std::fs::write(&installed, &edited).unwrap();

    let (code, out) = run(&["ui", "eject", "racing/race-results"]);
    assert_eq!(code, Some(0), "{out}");
    let ejected = dir.join("assets/ui/racing");
    let copy = std::fs::read_to_string(ejected.join("race-results.ogt")).unwrap();
    let (head, body) = copy.split_once("\n---\n").unwrap();
    let (provenance, rest) = body.split_once('\n').unwrap();
    assert!(
        provenance.starts_with("# og:ejected from the racing kit's race-results.ogt, fingerprint "),
        "{provenance}"
    );
    assert_eq!(format!("{head}\n---\n{rest}"), edited);
    for kit_only in [
        "index.ts",
        "types.ts",
        "extension.json",
        "src",
        "sheet",
        "fixtures",
    ] {
        assert!(
            !ejected.join(kit_only).exists(),
            "{kit_only} was ejected beside the document"
        );
    }
    let (code, out) = run(&["ui", "eject", "racing/race-results"]);
    assert_eq!(code, Some(1), "{out}");
    assert_eq!(
        std::fs::read_to_string(ejected.join("race-results.ogt")).unwrap(),
        copy
    );
}
