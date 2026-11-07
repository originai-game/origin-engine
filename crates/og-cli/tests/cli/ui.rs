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

/// `og vfx add <directory>` installs an effect pack (`kind vfx`) into
/// assets/vfx/<name>/, where its composed effects cook with the emitters
/// they name by the installed path; a second add refuses, and a pack with
/// code (`entry`) is not a valid vfx pack.
///
/// Ways it could fail: the pack copied into a kit's or a style's folder,
/// an existing installation replaced, or a vfx pack allowed to ship code.
#[test]
fn a_vfx_pack_installs_into_assets_vfx_and_cooks() {
    let dir = scratch_root();
    let project = dir.join("game");
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(project.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        project.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let pack = dir.join("hits");
    std::fs::create_dir_all(&pack).unwrap();
    let manifest = |extra: &str| {
        format!(
            "{{\"format\":1,\"kind\":\"vfx\",\"name\":\"hits\",\"version\":\"1.0.0\",\"description\":\"Hit effects\"{extra}}}"
        )
    };
    std::fs::write(
        pack.join("extension.json"),
        manifest(",\"minEngine\":\"0.1.0\",\"keywords\":[\"spark\"],\"tags\":{\"style\":\"stylised\",\"dim\":\"3d\",\"category\":[\"hit-spark\"],\"element\":[]}"),
    )
    .unwrap();
    std::fs::create_dir_all(pack.join("preview")).unwrap();
    std::fs::write(pack.join("preview/hit.png"), b"not installed").unwrap();
    std::fs::write(
        pack.join("sparks.ogt"),
        "type particles\nversion 1\n---\nrate 10\n",
    )
    .unwrap();
    std::fs::write(
        pack.join("hit.ogt"),
        "type vfx\nversion 1\n---\nduration 0.5\nimpact 0.1\nemitter \"vfx/hits/sparks.ogt\" {\n  at 0\n}\n",
    )
    .unwrap();
    let add = || {
        let out = og()
            .args(["vfx", "add"])
            .arg(&pack)
            .arg(&project)
            .output()
            .unwrap();
        (out.status.code(), text(&out))
    };
    let (code, out) = add();
    assert_eq!(code, Some(0), "{out}");
    assert!(project.join("assets/vfx/hits/hit.ogt").is_file(), "{out}");
    assert!(!project.join("assets/vfx/hits/preview").exists(), "{out}");
    std::fs::remove_dir_all(project.join("assets/vfx")).unwrap();
    // A pack for an engine newer than this one installs only on request.
    std::fs::write(
        pack.join("extension.json"),
        manifest(",\"minEngine\":\"99.0.0\""),
    )
    .unwrap();
    let (code, out) = add();
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("needs engine 99.0.0"), "{out}");
    assert!(!project.join("assets/vfx/hits").exists(), "{out}");
    let out = og()
        .args(["vfx", "add"])
        .arg(&pack)
        .arg(&project)
        .arg("--any-engine")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::write(pack.join("extension.json"), manifest(",\"minEngine\":\"0.1.0\",\"keywords\":[\"spark\"],\"tags\":{\"style\":\"stylised\",\"dim\":\"3d\",\"category\":[\"hit-spark\"],\"element\":[]}")).unwrap();
    let (code, out) = add();
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("already exists"), "{out}");

    let out = og().arg("cook").current_dir(&project).output().unwrap();
    assert!(out.status.success(), "{}", text(&out));

    std::fs::remove_dir_all(project.join("assets/vfx")).unwrap();
    for bad in [
        ",\"minEngine\":\"unreleased\"",
        ",\"keywords\":[\"Spark\"]",
        ",\"tags\":{\"style\":\"anime\",\"dim\":\"3d\"}",
    ] {
        std::fs::write(pack.join("extension.json"), manifest(bad)).unwrap();
        let (code, out) = add();
        assert_eq!(code, Some(1), "{bad}: {out}");
        assert!(out.contains("invalid extension manifest"), "{bad}: {out}");
    }
    std::fs::write(
        pack.join("extension.json"),
        manifest(",\"entry\":\"index.ts\",\"types\":\"index.d.ts\""),
    )
    .unwrap();
    std::fs::write(pack.join("index.ts"), "").unwrap();
    std::fs::write(pack.join("index.d.ts"), "").unwrap();
    let (code, out) = add();
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("invalid extension manifest"), "{out}");
}

/// `og check` types each screen and component from its document (`og ui
/// types`): a viewmodel against the props, a list's items as its fixtures
/// show them, the events' names (a used component's included), a
/// component's props; a name no document has stays untyped; an edit to a
/// document retypes the next check.
///
/// Ways it could fail: the engine's string overload chosen before the
/// game's documents (nothing caught), a used component's emit missing
/// (valid code refused), an unknown screen refused, or types written once
/// and left stale.
#[test]
fn og_check_types_viewmodels_and_events_from_the_documents() {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("assets/ui")).unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let shop = |gold: &str| {
        format!(
            "type screen\nversion 2\n---\nname \"shop\"\nback emit closeShop\nprops {{\n  gold {gold} \"Gold.\"\n  items list \"For sale.\"\n}}\n\
             fixture rich \"{{\\\"gold\\\": 1, \\\"items\\\": [{{\\\"name\\\": \\\"Sword\\\", \\\"price\\\": 40}}, {{\\\"name\\\": \\\"Shield\\\", \\\"price\\\": 30, \\\"sale\\\": true}}]}}\"\n\
             column Root {{\n  button Buy {{\n    text \"Buy\"\n    on click emit buy\n  }}\n  use Badge Gold {{\n    value 1\n  }}\n}}\n"
        )
    };
    std::fs::write(dir.join("assets/ui/shop.ogt"), shop("number")).unwrap();
    std::fs::write(
        dir.join("assets/ui/badge.ogt"),
        "type component\nversion 2\n---\nname \"Badge\"\nprops {\n  value number \"The number.\"\n}\nrow Root {\n  button Tap {\n    text \"{value}\"\n    on click emit tapped\n  }\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/main.ts"),
        r#"import { og } from "@origingame/engine";
og.ui.screen("shop", { gold: 1, items: [{ name: "a", price: 1 }] });
og.ui.screen("shop", { gold: "1", items: [] });
og.ui.screen("shop", { gold: 1, items: [{ name: "a", cost: 1 }] });
og.ui.screen("elsewhere", { anything: 1 });
for (const e of og.ui.screen("shop", null).events) {
  if (e.kind === "emit" && (e.name === "buy" || e.name === "tapped")) {}
  if (e.kind === "back" && e.name === "closeShop") {}
  if (e.kind === "emit" && e.name === "buyy") {}
}
og.ui.component("b", "Badge", { value: "no" });
export default og.game({});
"#,
    )
    .unwrap();
    let errors = || -> Vec<(u64, u64)> {
        let out = og().args(["check", "--json"]).arg(&dir).output().unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {}", text(&out)));
        report["diagnostics"]
            .as_array()
            .unwrap_or_else(|| panic!("{report}"))
            .iter()
            .map(|d| (d["line"].as_u64().unwrap(), d["code"].as_u64().unwrap()))
            .collect()
    };
    // The string gold, the item's unknown field, the misspelt event and
    // the component's string prop; nothing else.
    assert_eq!(errors(), [(3, 2322), (4, 2353), (9, 2367), (11, 2322)]);
    std::fs::write(dir.join("assets/ui/shop.ogt"), shop("string")).unwrap();
    assert_eq!(
        errors(),
        [(2, 2322), (4, 2322), (4, 2353), (9, 2367), (11, 2322)],
        "gold is a string now: the numbers are the wrong ones"
    );
}
