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
    assert_eq!(
        std::fs::read_to_string(ejected.join("race-results.ogt")).unwrap(),
        edited
    );
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
        edited
    );
}

/// `og ui add materials` installs the material presets (each a `type
/// ui-material` naming its WGSL beside it), and the game cooks every one:
/// the source is read relative to the material file, as a kit's relative
/// references are promised to survive installation (docs/ui.md §17.1). Each
/// cooked material carries its code and its parameters in `param` order.
///
/// Ways it could fail: `source` read only from `assets/` (every preset
/// missing its WGSL once installed), a preset whose WGSL does not compile,
/// or a preset past the four parameters a layer carries.
#[test]
fn the_material_presets_install_and_cook_with_their_wgsl_beside_them() {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let out = og()
        .args(["ui", "add", "materials"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let out = og().arg("cook").arg(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    for (preset, first) in [
        ("foil", "amount"),
        ("holo", "amount"),
        ("burn", "progress"),
        ("rarity", "color"),
    ] {
        let id = format!("ui-kits/materials/{preset}.ogt");
        let m = bundle
            .ui_material(&id, bundle.read(&id).unwrap().as_slice())
            .unwrap_or_else(|e| panic!("{id}: {e}"));
        assert!(m.code.contains("fn material("), "{id}");
        assert_eq!(m.params.len(), 4, "{id}");
        assert_eq!(m.params[0].name, first, "{id}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
