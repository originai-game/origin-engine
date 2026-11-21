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
    // A patch may carry breaks: against an engine at 0.9.3 whose 0.9.2
    // breaks, a pin of 0.9.1 crosses it and is not compatible, a pin of
    // 0.9.2 (the pin itself is not crossed) is.
    let fake = dir.join("fake-engine");
    std::fs::create_dir_all(fake.join("src/node")).unwrap();
    std::fs::write(fake.join("package.json"), r#"{ "version": "0.9.3" }"#).unwrap();
    std::fs::write(
        fake.join("src/node/doctor.ts"),
        "console.log(JSON.stringify({ version: \"0.9.3\" }));\n",
    )
    .unwrap();
    let entry = |v: &str, breaking: serde_json::Value| serde_json::json!({ "version": v, "kind": "patch", "simulation": "unchanged", "breaking": breaking, "added": [], "fixed": ["x"] });
    let cut = serde_json::json!([{ "member": "og.x", "change": "c", "fix": "f" }]);
    let log = serde_json::json!({
        "schema": 1,
        "unreleased": { "simulation": "unchanged", "breaking": [], "added": [], "fixed": [] },
        "versions": [entry("0.9.3", serde_json::json!([])), entry("0.9.2", cut), entry("0.9.1", serde_json::json!([])), entry("0.9.0", serde_json::json!([]))],
    });
    std::fs::write(fake.join("changelog.json"), log.to_string()).unwrap();
    let cache = dir.join("engine-cache");
    let doctor = |engine: &str, json: bool| {
        std::fs::write(dir.join("project.ogt"), manifest(engine)).unwrap();
        og().arg("doctor")
            .args(json.then_some("--json"))
            .current_dir(&dir)
            .env("OG_ENGINE_PACKAGE", &fake)
            .env("ORIGIN_ENGINE_CACHE", &cache)
            .output()
            .unwrap()
    };
    let compatible = |engine: &str| report(&doctor(engine, true))["pin"]["compatible"].clone();
    assert_eq!(compatible("0.9.1"), false);
    assert_eq!(compatible("0.9.0"), false);
    assert_eq!(compatible("0.9.2"), true);
    let line = text(&doctor("0.9.1", false));
    assert!(
        line.contains("not compatible (0.9.2 carries breaking changes)")
            && line.contains("og upgrade --to 0.9.3"),
        "{line}"
    );
    // A dev pin compares as the release it is based on (its installed
    // dist's build record): dev.20261119.1 over 0.9.1 crosses 0.9.2, over
    // 0.9.2 it does not; its own numbers never make it newer than 0.9.3.
    let based_on = |base: &str| {
        let d = cache.join("engines/dev.20261119.1/x86_64-unknown-linux-gnu");
        std::fs::create_dir_all(&d).unwrap();
        let build = serde_json::json!({ "build": { "version": "dev.20261119.1", "base": base } });
        std::fs::write(d.join("origin-engine.json"), build.to_string()).unwrap();
    };
    based_on("0.9.1");
    assert_eq!(compatible("dev.20261119.1"), false);
    based_on("0.9.2");
    assert_eq!(compatible("dev.20261119.1"), true);
    std::fs::remove_dir_all(&cache).unwrap();
    std::fs::remove_dir_all(&fake).unwrap();
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

/// `og upgrade` across the release where og.film became og.flow (its
/// `flow-rename` codemod): the game's Yarn headers, its segment's markers
/// and its code take og.flow's names, each rewrite is reported with its
/// file, and nothing is left to edit by hand. Ways this could fail: Yarn
/// files not read (the story still says `og-film` and no longer cooks),
/// or the rewrite reported but not written.
#[test]
fn upgrade_renames_og_film_in_a_games_yarn_segments_and_code() {
    let dir = scratch_root();
    std::fs::remove_dir_all(dir.join("crates")).unwrap();
    for d in ["src", "assets/story", "assets/segments"] {
        std::fs::create_dir_all(dir.join(d)).unwrap();
    }
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"u\"\nid \"u\"\nengine \"0.9.2\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let files = [
        (
            "assets/story/film.yarn",
            "title: a\nog-film: segments/a.ogt\nog-film-ending: brave\n---\n===\n",
            "title: a\nog-flow: segments/a.ogt\nog-flow-ending: brave\n---\n===\n",
        ),
        (
            "assets/segments/a.ogt",
            "type timeline\nversion 1\n---\ntrack marker {\n  key 3 {\n    name \"film.open\"\n  }\n}\n",
            "type timeline\nversion 1\n---\ntrack marker {\n  key 3 {\n    name \"flow.open\"\n  }\n}\n",
        ),
        (
            "src/main.ts",
            "og.film.load(world, \"film\");\nconst film = og.film.start(world, { dialogue: \"film\" });\n",
            "og.flow.load(world, \"film\");\nconst film = og.flow.start(world, { dialogue: \"film\" });\n",
        ),
    ];
    for (path, before, _) in files {
        std::fs::write(dir.join(path), before).unwrap();
    }
    let fake = dir.join("fake-engine");
    std::fs::create_dir_all(&fake).unwrap();
    std::fs::write(fake.join("package.json"), r#"{ "version": "0.9.3" }"#).unwrap();
    let entry = |v: &str, breaking: serde_json::Value| serde_json::json!({ "version": v, "kind": "patch", "simulation": "unchanged", "breaking": breaking, "added": [], "fixed": ["x"] });
    let renamed = serde_json::json!([{ "member": "og.film is og.flow", "change": "c", "fix": "f", "codemod": { "kind": "flow-rename" } }]);
    let log = serde_json::json!({
        "schema": 1,
        "unreleased": { "simulation": "unchanged", "breaking": [], "added": [], "fixed": [] },
        "versions": [entry("0.9.3", renamed), entry("0.9.2", serde_json::json!([]))],
    });
    std::fs::write(fake.join("changelog.json"), log.to_string()).unwrap();
    let out = og()
        .args(["upgrade", "--to", "0.9.3", "--json"])
        .current_dir(&dir)
        .env("OG_ENGINE_PACKAGE", &fake)
        .output()
        .unwrap();
    let r: serde_json::Value =
        serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {}", text(&out)));
    assert_eq!(out.status.code(), Some(0), "{r}");
    assert_eq!(r["status"], "done", "{r}");
    for (path, _, after) in files {
        assert_eq!(
            std::fs::read_to_string(dir.join(path)).unwrap(),
            after,
            "{path}"
        );
    }
    let rewrites = |file: &str, from: &str| {
        r["codemods"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["file"] == file && c["from"] == from)
            .map(|c| {
                (
                    c["to"].as_str().unwrap().to_string(),
                    c["count"].as_u64().unwrap(),
                )
            })
    };
    assert_eq!(
        rewrites("assets/story/film.yarn", "og-film"),
        Some(("og-flow".into(), 2)),
        "{r}"
    );
    assert_eq!(
        rewrites("src/main.ts", "og.film"),
        Some(("og.flow".into(), 2)),
        "{r}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// `og upgrade` with a dev build `dev.<date>.<n>` installed: a release pin
/// moves onto it only when --to names it, crossing its base release's
/// versions and its own unreleased entry, and never from a release newer
/// than its base (its numbers never make it newer than a release).
#[test]
fn upgrade_moves_a_release_pin_onto_a_dev_build_only_when_named() {
    let dir = TempDir::new("og-cli-upgrade-dev");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "og.x;\n").unwrap();
    let manifest = |engine: &str| {
        format!(
            "type project\nversion 1\n---\nname \"u\"\nid \"u\"\nengine \"{engine}\"\nentry \"src/main.ts\"\n"
        )
    };
    let report = |out: &Output| -> serde_json::Value {
        serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {}", text(out)))
    };
    let fake = dir.join("fake-engine");
    std::fs::create_dir_all(fake.join("src/node")).unwrap();
    std::fs::write(fake.join("src/node/run.ts"), "").unwrap();
    std::fs::write(
        fake.join("package.json"),
        r#"{ "version": "dev.20261119.1" }"#,
    )
    .unwrap();
    let entry = |v: &str| serde_json::json!({ "version": v, "kind": "patch", "simulation": "unchanged", "breaking": [], "added": [], "fixed": ["x"] });
    let dev_log = serde_json::json!({
        "schema": 1,
        "unreleased": { "simulation": "unchanged", "breaking": [], "added": [{ "member": "og.y", "summary": "s" }], "fixed": [] },
        "versions": [entry("0.9.3"), entry("0.9.2")],
    });
    std::fs::write(fake.join("changelog.json"), dev_log.to_string()).unwrap();
    let upgrade_dev = |engine: &str, args: &[&str]| {
        std::fs::write(dir.join("project.ogt"), manifest(engine)).unwrap();
        let out = og()
            .args(["upgrade", "--json"])
            .args(args)
            .current_dir(&dir)
            .env("OG_ENGINE_PACKAGE", &fake)
            .output()
            .unwrap();
        (
            report(&out),
            std::fs::read_to_string(dir.join("project.ogt")).unwrap(),
        )
    };
    let (r, pinned) = upgrade_dev("0.9.2", &[]);
    assert_eq!(r["status"], "failed", "{r}");
    assert!(
        r["reason"]
            .as_str()
            .unwrap()
            .contains("--to dev.20261119.1"),
        "{r}"
    );
    assert_eq!(pinned, manifest("0.9.2"));
    let (r, pinned) = upgrade_dev("0.9.2", &["--to", "dev.20261119.1"]);
    assert_eq!(
        (r["status"].as_str(), r["kind"].as_str()),
        (Some("done"), Some("patch")),
        "{r}"
    );
    assert_eq!(
        r["versions"],
        serde_json::json!(["0.9.3", "dev.20261119.1"])
    );
    assert_eq!(pinned, manifest("dev.20261119.1"));
    // dev.20261119.1 is based on 0.9.3: moving a 0.9.4 pin onto it is a downgrade.
    let (r, pinned) = upgrade_dev("0.9.4", &["--to", "dev.20261119.1"]);
    assert!(
        r["reason"].as_str().unwrap().contains("never downgrades"),
        "{r}"
    );
    assert_eq!(pinned, manifest("0.9.4"));
}

#[test]
fn fmt_rewrites_a_projects_ogt_files_and_check_only_reports_them() {
    let dir = TempDir::new("og-cli-fmt");
    let write = |file: &str, text: &str| {
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    let read = |file: &str| std::fs::read_to_string(dir.join(file)).unwrap();
    let loose = "type material\nversion 1\n---\n# the base\n\n\ncolor   1 0 0   # red\n";
    let tidy = "type material\nversion 1\n---\n# the base\n\ncolor 1 0 0  # red\n";
    // A style's lines are its own, a pack's files its pack's, and nothing
    // outside project.ogt and assets/ is the project's to format.
    let style = "type style\nversion 1\n---\n.title {\n      color:   red;\n}\n";
    write(
        "project.ogt",
        "type   project\nversion 1\n---\nname \"fmt\"\n",
    );
    write("assets/a.ogt", loose);
    write("assets/done.ogt", tidy);
    write("assets/ui/title.ogt", style);
    write("assets/pack/extension.json", "{}");
    write("assets/pack/b.ogt", loose);
    write("notes/c.ogt", loose);
    write(
        "assets/broken.ogt",
        "type material\nversion 1\n---\ncolor {\n",
    );
    let fmt = |args: &[&str]| {
        og().arg("fmt")
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap()
    };

    // The broken file is named with its line and stops neither run.
    let out = fmt(&["--check", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    let r: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        r["changed"],
        serde_json::json!(["./project.ogt", "./assets/a.ogt"]),
        "{r}"
    );
    assert_eq!(r["errors"][0]["file"], "./assets/broken.ogt", "{r}");
    assert!(
        r["errors"][0]["error"].as_str().unwrap().contains("line 4"),
        "{r}"
    );
    assert_eq!(read("assets/a.ogt"), loose, "--check writes nothing");
    std::fs::remove_file(dir.join("assets/broken.ogt")).unwrap();

    let out = fmt(&[]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(read("assets/a.ogt"), tidy);
    assert_eq!(
        read("project.ogt"),
        "type project\nversion 1\n---\nname \"fmt\"\n"
    );
    for (file, kept) in [
        ("assets/ui/title.ogt", style),
        ("assets/pack/b.ogt", loose),
        ("notes/c.ogt", loose),
    ] {
        assert_eq!(read(file), kept, "{file}");
    }
    let out = fmt(&["--check"]);
    assert!(out.status.success(), "{}", text(&out));
    // A file named outright is formatted wherever it is.
    let out = fmt(&["notes/c.ogt"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(read("notes/c.ogt"), tidy);
}
