use super::*;

/// `og upgrade` against the repository's engine package and its
/// changelog.json: a patch is done (exit 0), crossing 0.3.0 rewrites
/// `og.load.skin` and lists the `ThemeColor` line to edit by hand (exit 3),
/// and a downgrade writes nothing (exit 1).
#[test]
fn upgrade_rewrites_the_pin_applies_codemods_and_lists_hand_edits() {
    let dir = scratch_root();
    std::fs::remove_dir_all(dir.join("crates")).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let source = "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'upgrade',entry:'src/main.ts'});\n";
    std::fs::write(dir.join("project.ts"), source).unwrap();
    let manifest = |engine: &str| {
        format!("{}\n", serde_json::to_string_pretty(&serde_json::json!({"version":1,"name":"u","engine":engine,"look":{"$ogRef":"base/looks/main"}})).unwrap())
    };
    let write = |engine: &str| {
        std::fs::write(dir.join("og-project.json"), manifest(engine)).unwrap();
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
    let pinned = std::fs::read_to_string(dir.join("og-project.json")).unwrap();
    assert_eq!(pinned, manifest("0.6.1"));
    assert_eq!(
        std::fs::read_to_string(dir.join("project.ts")).unwrap(),
        source
    );
    // og-project.json is the one pin: a package.json is left as it is.
    let pkg = std::fs::read_to_string(dir.join("package.json")).unwrap();
    assert!(pkg.contains("\"@origingame/engine\": \"^0.6.0\""), "{pkg}");
    // doctor's pin: an older patch of the installed minor is compatible, an
    // older minor is not.
    let pin = |engine: &str| {
        std::fs::write(dir.join("og-project.json"), manifest(engine)).unwrap();
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
        std::fs::write(dir.join("og-project.json"), manifest(engine)).unwrap();
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
    std::fs::write(dir.join("og-project.json"), manifest("0.6.1")).unwrap();

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
        std::fs::read_to_string(dir.join("og-project.json")).unwrap(),
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
        std::fs::read_to_string(dir.join("og-project.json")).unwrap(),
        manifest("0.6.1")
    );
    let _ = std::fs::remove_dir_all(&dir);
}
