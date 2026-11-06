use super::*;

#[test]
fn asset_queries_filter_references_and_apply_the_callers_duplicate_threshold() {
    use og_cook::cook::assets::{Asset, Index};
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("cooked")).unwrap();
    let mut index = Index::default();
    for (id, file, fingerprint) in [
        ("a.png", "a.ogb", 0),
        ("b.png", "a.ogb", 0),
        ("c.png", "c.ogb", 3),
    ] {
        index.assets.push(Asset {
            id: id.into(),
            source: id.into(),
            kind: "texture".into(),
            file: file.into(),
            group: Some("later".into()),
            bytes: 100,
            hash: file.into(),
            perceptual: Some(fingerprint),
            refs: if id == "a.png" {
                ["b.png".into()].into()
            } else {
                Default::default()
            },
        });
    }
    index.roots.insert("a.png".into());
    index
        .users
        .insert("src/main.ts".into(), ["a.png".into()].into());
    index.write(&dir.join("cooked")).unwrap();
    let run = |args: &[&str]| {
        let out = og()
            .arg("assets")
            .args(args)
            .arg("--json")
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()
    };
    let exact = run(&["--dupes"]);
    assert_eq!(exact["duplicates"].as_array().unwrap().len(), 1);
    assert_eq!(
        run(&["--dupes", "--near", "1"])["duplicates"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        run(&["--dupes", "--near", "2"])["duplicates"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let unused = run(&["--unused", "--kind", "texture", "--group", "later"]);
    assert_eq!(unused["assets"].as_array().unwrap().len(), 1);
    assert_eq!(unused["assets"][0]["id"], "c.png");
    let refs = run(&["--refs", "--by", "A.PNG"]);
    assert_eq!(refs["assets"][0]["refs"], serde_json::json!(["b.png"]));
    assert_eq!(refs["assets"][0]["by"], serde_json::json!(["src/main.ts"]));
    assert_eq!(
        og().args(["assets", "--near", "2"])
            .current_dir(&dir)
            .status()
            .unwrap()
            .code(),
        Some(2)
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn upgrade_migrates_style_typography_without_discarding_legacy_values() {
    let dir = scratch_root();
    let assets = dir.join("assets");
    let package = dir.join("engine");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(&package).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    let manifest = "type project\nversion 1\n---\nname \"migration\"\nengine \"0.7.7\"\nentry \"src/main.ts\"\n";
    std::fs::write(dir.join("project.ogt"), manifest).unwrap();
    let log: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo().join("packages/engine/changelog.json")).unwrap(),
    )
    .unwrap();
    let migration = log["unreleased"]["breaking"]
        .as_array()
        .unwrap()
        .iter()
        .chain(
            log["versions"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|v| v["breaking"].as_array().into_iter().flatten()),
        )
        .find(|b| b["codemod"]["kind"] == "ui-typography")
        .unwrap();
    std::fs::write(package.join("package.json"), r#"{"version":"0.8.0"}"#).unwrap();
    std::fs::write(
        package.join("changelog.json"),
        serde_json::to_vec(&serde_json::json!({
            "versions": [{"version":"0.8.0", "simulation":"unchanged", "breaking":[migration]}]
        }))
        .unwrap(),
    )
    .unwrap();
    let old =
        std::fs::read_to_string(repo().join("crates/og-cli/tests/fixtures/style-pack/style.ogt"))
            .unwrap()
            + "\nramp {\n  body 19 27 31 0.07 0.02\n}\n";
    let v2 = og_ui::upgrade_style(&old).unwrap();
    std::fs::write(assets.join("legacy.ogt"), &old).unwrap();
    std::fs::write(assets.join("modern.ogt"), &v2).unwrap();
    // A version 2 style written before hero existed has no hero line (meridian-loop's):
    // its ramp's own displayXl2, else the old default, is what its hero keeps.
    let mut skip = false;
    let no_hero: String = v2
        .split_inclusive('\n')
        .filter(|l| {
            if l.starts_with("  hero ") {
                skip = l.trim_end().ends_with('{');
                return false;
            }
            if skip {
                skip = l.trim_end() != "  }";
                return false;
            }
            true
        })
        .collect();
    assert_ne!(no_hero, v2, "the upgraded style writes a hero line");
    std::fs::write(assets.join("bare.ogt"), &no_hero).unwrap();
    let titled = no_hero.replacen(
        "  body 19 27 31 0.07 0.02\n",
        "  body 19 27 31 0.07 0.02\n  displayXl2 50 54\n",
        1,
    );
    assert_ne!(titled, no_hero);
    std::fs::write(assets.join("titled.ogt"), &titled).unwrap();
    let scene = "type scene\nversion 1\n---\nramp { body 23 31 }\n";
    std::fs::write(assets.join("scene.ogt"), scene).unwrap();
    let run = |dry: bool| {
        let mut cmd = og();
        cmd.arg("upgrade")
            .arg(&dir)
            .arg("--json")
            .env("OG_ENGINE_PACKAGE", &package);
        if dry {
            cmd.arg("--dry-run");
        }
        let out = cmd.output().unwrap();
        assert_eq!(out.status.code(), Some(3), "{}", text(&out));
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()
    };
    assert_eq!(run(true)["codemods"].as_array().unwrap().len(), 4);
    assert_eq!(
        std::fs::read_to_string(assets.join("legacy.ogt")).unwrap(),
        old
    );
    assert_eq!(
        std::fs::read_to_string(assets.join("modern.ogt")).unwrap(),
        v2
    );
    assert_eq!(run(false)["codemods"].as_array().unwrap().len(), 4);
    // Without a hero line, hero is display × 1.5: a legacy displayXl2 (a
    // countdown's 60 px line) grew to 81. The upgrade keeps what it drew.
    for (name, size, line) in [
        ("legacy", 56.0, 60.0),
        ("bare", 56.0, 60.0),
        ("titled", 50.0, 54.0),
    ] {
        let migrated = std::fs::read_to_string(assets.join(format!("{name}.ogt"))).unwrap();
        let hero = og_ui::parse_style(&migrated)
            .unwrap()
            .theme
            .type_ramp
            .display_xl2;
        assert_eq!(
            (hero.size, hero.line_height),
            (size, line),
            "{name}: {migrated}"
        );
    }
    for name in ["legacy", "modern"] {
        let migrated = std::fs::read_to_string(assets.join(format!("{name}.ogt"))).unwrap();
        assert!(migrated.contains("# ramp {"));
        assert!(migrated.contains("#   body 19 27 31 0.07 0.02"));
        let file = og_ui::parse_style(&migrated).unwrap();
        assert_eq!(file.version, 3);
        assert_eq!(file.theme.type_ramp.body.size, 19.0);
        assert_eq!(file.theme.type_ramp.body.tracking_em, 0.07);
        assert_eq!(file.theme.type_ramp.body.cjk_tracking_em, 0.02);
    }
    assert_eq!(
        std::fs::read_to_string(assets.join("scene.ogt")).unwrap(),
        scene
    );
    std::fs::write(dir.join("project.ogt"), manifest).unwrap();
    assert!(
        run(false)["codemods"].as_array().unwrap().is_empty(),
        "idempotent"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

/// `og upgrade` against the repository's engine package and its
/// changelog.json: a patch is done (exit 0), crossing 0.3.0 rewrites
/// `og.load.skin` and lists the `ThemeColor` line to edit by hand (exit 3),
/// and a downgrade writes nothing (exit 1).
#[test]
fn upgrade_rewrites_the_pin_applies_codemods_and_lists_hand_edits() {
    let dir = scratch_root();
    std::fs::remove_dir_all(dir.join("crates")).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let manifest = |engine: &str| {
        format!(
            "type project\nversion 1\n---\nname \"u\"\nid \"u\"\nengine \"{engine}\"\nentry \"src/main.ts\"\n"
        )
    };
    let write = |engine: &str| {
        std::fs::write(dir.join("project.ogt"), manifest(engine)).unwrap();
        std::fs::write(
            dir.join("package.json"),
            format!("{{\n  \"dependencies\": {{ \"@origingame/engine\": \"^{engine}\" }}\n}}\n"),
        )
        .unwrap();
    };
    let upgrade = |args: &[&str]| {
        og().arg("upgrade")
            .args(args)
            .current_dir(&dir)
            .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
            .output()
            .unwrap()
    };
    let report = |out: &Output| -> serde_json::Value {
        serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {}", text(out)))
    };

    write("0.6.0");
    std::fs::write(dir.join("src/main.ts"), "og.load.style(\"a\");\n").unwrap();
    let out = upgrade(&["--to", "0.6.1", "--json"]);
    let r = report(&out);
    assert_eq!(out.status.code(), Some(0), "{r}");
    assert_eq!(
        (r["status"].as_str(), r["kind"].as_str()),
        (Some("done"), Some("patch")),
        "{r}"
    );
    let pinned = std::fs::read_to_string(dir.join("project.ogt")).unwrap();
    assert_eq!(pinned, manifest("0.6.1"));
    // project.ogt is the one pin: a package.json is left as it is.
    let pkg = std::fs::read_to_string(dir.join("package.json")).unwrap();
    assert!(pkg.contains("\"@origingame/engine\": \"^0.6.0\""), "{pkg}");
    // doctor's pin: an older patch of the installed minor is compatible, an
    // older minor is not.
    let pin = |engine: &str| {
        std::fs::write(dir.join("project.ogt"), manifest(engine)).unwrap();
        let out = og()
            .args(["doctor", "--json"])
            .current_dir(&dir)
            .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
            .output()
            .unwrap();
        report(&out)["pin"].clone()
    };
    let installed = pin("0.5.0")["installed"].as_str().unwrap().to_string();
    let minor = installed.rsplit_once('.').unwrap().0;
    let first_patch = format!("{minor}.0");
    assert_eq!(
        pin(&first_patch),
        serde_json::json!({ "pinned": first_patch, "installed": installed, "compatible": true })
    );
    assert_eq!(pin("0.5.0")["compatible"], false);
    assert_eq!(pin("0.99.0")["compatible"], false);
    std::fs::write(dir.join("project.ogt"), manifest("0.6.1")).unwrap();

    write("0.2.0");
    let main = "const s = og.load.skin(\"ui/a.ogt\");\nconst c: ThemeColor = \"accent\";\n";
    std::fs::write(dir.join("src/main.ts"), main).unwrap();
    // A dry run reports the same and writes nothing.
    let out = upgrade(&["--to", "0.6.1", "--json", "--dry-run"]);
    assert_eq!(out.status.code(), Some(3), "{}", text(&out));
    assert_eq!(
        std::fs::read_to_string(dir.join("src/main.ts")).unwrap(),
        main
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("project.ogt")).unwrap(),
        manifest("0.2.0")
    );
    let out = upgrade(&["--to", "0.6.1", "--json"]);
    let r = report(&out);
    assert_eq!(out.status.code(), Some(3), "{r}");
    assert_eq!(
        (r["status"].as_str(), r["kind"].as_str()),
        (Some("manual"), Some("minor")),
        "{r}"
    );
    assert_eq!(r["simulation"], "changed", "{r}");
    assert_eq!(r["versions"][0], "0.3.0", "{r}");
    let src = std::fs::read_to_string(dir.join("src/main.ts")).unwrap();
    assert!(
        src.starts_with("const s = og.load.style(\"ui/a.ogt\");"),
        "{src}"
    );
    let hand = r["manual"].as_array().unwrap();
    assert!(
        hand.iter()
            .any(|m| m["member"] == "ThemeColor" && m["file"] == "src/main.ts" && m["line"] == 2),
        "{r}"
    );
    assert!(
        hand.iter().all(|m| m["line"] != 1),
        "the rewritten line is not a hand edit: {r}"
    );

    let out = upgrade(&["--to", "0.5.0"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(text(&out).contains("never downgrades"), "{}", text(&out));
    assert_eq!(
        std::fs::read_to_string(dir.join("project.ogt")).unwrap(),
        manifest("0.6.1")
    );
    let _ = std::fs::remove_dir_all(&dir);
}
