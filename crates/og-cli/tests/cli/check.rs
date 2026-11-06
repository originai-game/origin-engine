use super::*;

/// A project with a tsconfig.json of its own, in a scratch root.
fn check_project() -> (PathBuf, PathBuf) {
    let dir = scratch_root();
    let game = dir.join("game");
    std::fs::create_dir_all(game.join("src")).unwrap();
    std::fs::write(game.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(game.join("tsconfig.json"), "{}\n").unwrap();
    std::fs::write(
        game.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    (dir, game)
}

const TS2322: &str =
    "src/main.ts(1,7): error TS2322: Type 'string' is not assignable to type 'number'.\n";

/// `og check` finds tsrs by OG_TSRS, then the engine package's
/// node_modules/@tsrs/<platform>, then PATH, and reports its diagnostics
/// as one `check` event; without any it falls back to tsgo through bun x.
#[cfg(unix)]
#[test]
fn check_finds_tsrs_by_its_chain_and_reports_diagnostics_as_values() {
    let plat = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x64-gnu",
        ("linux", "aarch64") => "linux-arm64-gnu",
        ("macos", "aarch64") => "darwin-arm64",
        _ => return,
    };
    let (dir, game) = check_project();
    // The engine package in the project's node_modules, its tsrs beside it.
    let engine = game.join("node_modules/@origingame/engine");
    std::fs::create_dir_all(engine.join("src/node")).unwrap();
    std::fs::write(engine.join("src/node/run.ts"), "").unwrap();
    fake_tsrs(
        &game.join("node_modules/@tsrs").join(plat),
        "1.0.0-modules",
        TS2322,
        2,
        true,
    );
    let on_path = dir.join("path");
    fake_tsrs(&on_path, "1.0.0-path", "", 0, true);
    let named = fake_tsrs(&dir.join("named"), "1.0.0-named", "", 0, true);
    let path = format!("{}:{}", on_path.display(), std::env::var("PATH").unwrap());
    let check = |env: &[(&str, &std::ffi::OsStr)]| {
        let mut c = og();
        c.args(["check", "--json"])
            .arg(&game)
            .current_dir(&dir)
            .env_remove("OG_TSRS")
            .env_remove("OG_ENGINE_PACKAGE");
        for (k, v) in env {
            c.env(k, v);
        }
        let out = c.output().unwrap();
        let line = String::from_utf8_lossy(&out.stdout).to_string();
        let v: serde_json::Value =
            serde_json::from_str(line.trim()).unwrap_or_else(|e| panic!("{e}: {}", text(&out)));
        (out.status.code(), v)
    };

    let (code, v) = check(&[("OG_TSRS", named.as_os_str()), ("PATH", path.as_ref())]);
    assert_eq!(
        (code, v["version"].as_str()),
        (Some(0), Some("1.0.0-named")),
        "{v}"
    );
    assert_eq!(v["ok"], true);
    assert_eq!(v["diagnostics"], serde_json::json!([]));

    let (code, v) = check(&[("PATH", path.as_ref())]);
    assert_eq!(
        (code, v["version"].as_str()),
        (Some(1), Some("1.0.0-modules")),
        "{v}"
    );
    assert_eq!(v["tool"], "tsrs");
    assert_eq!(v["ok"], false);
    assert_eq!(
        v["config"], "tsconfig.json",
        "the engine is in node_modules: its own config"
    );
    assert_eq!(
        v["diagnostics"],
        serde_json::json!([{ "file": "src/main.ts", "line": 1, "col": 7, "code": 2322,
            "message": "Type 'string' is not assignable to type 'number'." }])
    );
    let args = std::fs::read_to_string(game.join("node_modules/@tsrs").join(plat).join("args.txt"))
        .unwrap();
    assert!(
        args.contains("-p")
            && args.contains("--noEmit --incremental false --pretty false --checkers"),
        "{args}"
    );

    std::fs::remove_dir_all(game.join("node_modules/@tsrs")).unwrap();
    let (code, v) = check(&[("PATH", path.as_ref())]);
    assert_eq!(
        (code, v["version"].as_str()),
        (Some(0), Some("1.0.0-path")),
        "{v}"
    );

    // No tsrs anywhere: tsgo through `bun x` (a stand-in bun here), the same JSON.
    std::fs::remove_dir_all(&on_path).unwrap();
    let bun = dir.join("bun-only");
    fake_tsrs(&bun, "unused", TS2322, 1, false);
    std::fs::rename(bun.join("tsrs"), bun.join("bun")).unwrap();
    let out = og()
        .args(["check", "--json"])
        .arg(&game)
        .current_dir(&dir)
        .env_remove("OG_TSRS")
        .env("PATH", &bun)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(out.stdout.trim_ascii()).unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert_eq!(
        (v["tool"].as_str(), v["version"].as_str()),
        (Some("tsgo"), Some("7.0.2"))
    );
    assert_eq!(v["diagnostics"][0]["code"], 2322);
    assert!(
        text(&out).contains("no tsrs") && text(&out).contains("checking with tsgo 7.0.2"),
        "{}",
        text(&out)
    );
    let args = std::fs::read_to_string(bun.join("args.txt")).unwrap();
    assert!(
        args.starts_with("x --package typescript@7.0.2 tsc -p "),
        "{args}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// `og run`, `og test` and `og build` type-check before they bundle: a
/// type error stops each before the engine package's bundler, test runner
/// or exporter is started; `--no-check` runs as before, and a publish
/// build refuses it.
#[test]
fn run_test_and_build_stop_on_a_type_error_before_bundling() {
    assert!(
        cfg!(windows)
            || std::env::var_os("PATH")
                .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join("tsrs").is_file())),
        "this test needs tsrs on PATH (scripts/install-tools.sh)"
    );
    let dir = scratch_root();
    std::fs::remove_dir_all(dir.join("crates")).unwrap();
    // A stand-in engine package: each step it would run leaves a mark.
    let engine = dir.join("engine");
    std::fs::create_dir_all(engine.join("src/node")).unwrap();
    let marks = dir.join("marks");
    std::fs::create_dir_all(&marks).unwrap();
    for (script, extra) in [
        (
            "build.ts",
            "const i = process.argv.indexOf('--out'); await Bun.write(process.argv[i + 1], 'export default 0;');",
        ),
        ("run.ts", ""),
        ("playtest-run.ts", ""),
        ("web-build.ts", ""),
    ] {
        std::fs::write(
            engine.join("src/node").join(script),
            format!(
                "await Bun.write({:?}, '');\n{extra}\n",
                marks.join(script).to_str().unwrap()
            ),
        )
        .unwrap();
    }
    let game = dir.join("game");
    std::fs::create_dir_all(game.join("src")).unwrap();
    std::fs::create_dir_all(game.join("tests")).unwrap();
    std::fs::write(
        game.join("src/main.ts"),
        "export const n: number = \"one\";\nexport default 0;\n",
    )
    .unwrap();
    std::fs::write(game.join("tests/smoke.og.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        game.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let og_in = |args: &[&str]| {
        let out = og()
            .args(args)
            .arg(&game)
            .env("OG_ENGINE_PACKAGE", &engine)
            .env_remove("OG_TSRS")
            .output()
            .unwrap();
        let ran: Vec<String> = std::fs::read_dir(&marks)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        for e in std::fs::read_dir(&marks).unwrap().flatten() {
            std::fs::remove_file(e.path()).unwrap();
        }
        let mut ran = ran;
        ran.sort();
        (out.status.code(), text(&out), ran)
    };
    for cmd in [&["run", "--headless"][..], &["test"], &["build"]] {
        let (code, t, ran) = og_in(cmd);
        assert_eq!(code, Some(1), "og {cmd:?}: {t}");
        assert!(
            t.contains("src/main.ts(1,14): error TS2322"),
            "og {cmd:?}: {t}"
        );
        assert!(
            t.contains("1 type error(s); fix them or pass --no-check"),
            "og {cmd:?}: {t}"
        );
        assert!(
            ran.is_empty(),
            "og {cmd:?} started {ran:?} past a type error"
        );
    }
    assert!(!game.join(".og/game.native.js").exists());
    let (_, t, _) = og_in(&["test"]);
    assert!(
        t.contains("the 1 playtest(s) did not run: the type check failed"),
        "{t}"
    );

    let (code, t, ran) = og_in(&["run", "--headless", "--no-check"]);
    assert_eq!(
        (code, ran.clone()),
        (Some(0), vec!["build.ts".to_string(), "run.ts".into()]),
        "{t}"
    );
    let (code, _, ran) = og_in(&["test", "--no-check"]);
    assert_eq!(
        (code, ran),
        (
            Some(0),
            vec!["build.ts".to_string(), "playtest-run.ts".into()]
        )
    );
    let (code, _, ran) = og_in(&["build", "--no-check"]);
    assert_eq!((code, ran), (Some(0), vec!["web-build.ts".to_string()]));
    let (code, t, ran) = og_in(&["build", "--publish", "--no-check"]);
    assert_eq!(code, Some(2), "{t}");
    assert!(
        t.contains("always type-checked; drop --no-check") && ran.is_empty(),
        "{t}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
