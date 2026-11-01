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

#[cfg(windows)]
#[test]
fn windows_checks_types_with_the_reported_tsgo_fallback() {
    let (dir, game) = check_project();
    std::fs::write(game.join("tsconfig.json"), r#"{"include":["src/**/*.ts"]}"#).unwrap();
    std::fs::write(game.join("src/main.ts"), "const n: number = 'wrong';\n").unwrap();
    let out = og()
        .arg("check")
        .arg(&game)
        .arg("--json")
        .env_remove("OG_TSRS")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["tool"], "tsgo");
    assert!(text(&out).contains("2322"), "{TS2322}: {}", text(&out));
    std::fs::write(game.join("src/main.ts"), "const n: number = 42;\n").unwrap();
    let out = og()
        .arg("check")
        .arg(&game)
        .arg("--json")
        .env_remove("OG_TSRS")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::remove_dir_all(dir).unwrap();
}

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

/// A checker that crashes, exits with nothing readable, or sits apart from
/// its lib files is a failure, never a clean check.
#[cfg(unix)]
#[test]
fn check_never_reads_a_failing_tsrs_as_clean() {
    let (dir, game) = check_project();
    for (name, out, status, lib, expect) in [
        (
            "crash",
            "thread main panicked",
            101,
            true,
            "tsrs failed: it exited with status 101",
        ),
        (
            "silent",
            "",
            1,
            true,
            "reported nothing it could be read as",
        ),
        ("nolib", "", 0, false, "has no lib.es2022.d.ts beside it"),
    ] {
        let bin = fake_tsrs(&dir.join(name), name, out, status, lib);
        let o = og()
            .args(["check"])
            .arg(&game)
            .current_dir(&dir)
            .env("OG_TSRS", &bin)
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(1), "{name}: {}", text(&o));
        assert!(text(&o).contains(expect), "{name}: {}", text(&o));
        assert!(!text(&o).contains("og check: ok"), "{name}: {}", text(&o));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// tsgo as `bun x` left it after an interrupted install (studio-win's
/// `%TEMP%\bunx-*-typescript@7.0.2`, 26 lib files gone): an empty lib.d.ts
/// and only two other lib files.
fn truncated_bunx_typescript(tree: &Path) {
    let os = match std::env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        os => os,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        arch => arch,
    };
    let modules = tree.join("node_modules");
    let lib = modules.join(format!("@typescript/typescript-{os}-{arch}/lib"));
    std::fs::create_dir_all(modules.join("typescript/lib")).unwrap();
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(modules.join("typescript/package.json"), "{}").unwrap();
    std::fs::write(modules.join("typescript/lib/tsc.js"), "//").unwrap();
    let exe = if cfg!(windows) { "tsc.exe" } else { "tsc" };
    std::fs::write(lib.join(exe), "tsgo").unwrap();
    std::fs::write(lib.join("lib.d.ts"), "").unwrap();
    for name in ["lib.es5.d.ts", "lib.dom.d.ts"] {
        std::fs::write(lib.join(name), "interface Array<T> {}\n").unwrap();
    }
}

/// tsgo 7.0.2's panic on that install, its stack cut short.
#[cfg(unix)]
fn tsgo_panic(tree: &Path) -> String {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };
    format!(
        "panic: bundled: {}/node_modules/@typescript/typescript-{os}-{arch}/lib/lib.d.ts does not exist; this executable may be misplaced [recovered, repanicked]\n\ngoroutine 1 [running]:\ngithub.com/microsoft/typescript-go/internal/bundled.init.OnceValue[...].func8.1.1()\n\tsync/oncefunc.go:63 +0x70\nmain.main()\n\tgithub.com/microsoft/typescript-go/cmd/tsgo/main.go:14 +0x13\n",
        tree.display()
    )
}

/// A truncated bunx TypeScript install is named, with its fix, instead of
/// tsgo's Go stack trace: found where bun x keeps it before tsgo runs, and
/// from tsgo's panic when it is somewhere og does not look.
#[test]
fn check_names_a_truncated_tsgo_install_and_its_fix() {
    let (dir, game) = check_project();
    // Where bun 1.4's `bun x` keeps typescript@7.0.2.
    let base = dir.join(if cfg!(windows) { "tmp" } else { "cache" });
    std::fs::create_dir_all(&base).unwrap();
    #[cfg(unix)]
    let tree = {
        use std::os::unix::fs::MetadataExt;
        let uid = std::fs::metadata(&base).unwrap().uid();
        base.join(format!(".bunx-{uid}/typescript@7.0.2"))
    };
    #[cfg(windows)]
    let tree = base.join("bunx-S-1-5-21-test-typescript@7.0.2");
    truncated_bunx_typescript(&tree);
    // The stand-in bun prints tsgo's panic and exits as tsgo does.
    let bun = dir.join("bun");
    std::fs::create_dir_all(&bun).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(bun.join("panic.txt"), tsgo_panic(&tree)).unwrap();
        std::fs::write(
            bun.join("bun"),
            "#!/bin/sh\nPATH=/usr/bin:/bin\ncat \"$(dirname \"$0\")/panic.txt\" >&2\nexit 2\n",
        )
        .unwrap();
        std::fs::set_permissions(bun.join("bun"), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let check = |cache: &Path| {
        let o = og()
            .args(["check", "--tool", "tsgo"])
            .arg(&game)
            .current_dir(&dir)
            .env("PATH", &bun)
            .env("BUN_INSTALL_CACHE_DIR", cache)
            .env("TMPDIR", cache)
            .env("TMP", cache)
            .env("TEMP", cache)
            .output()
            .unwrap();
        let t = text(&o);
        assert_eq!(o.status.code(), Some(1), "{t}");
        for want in [
            format!(
                "og check: the TypeScript checker (tsgo 7.0.2) that bun x installed in {} is incomplete",
                tree.display()
            ),
            "106 of its 111 files missing or empty (lib.d.ts, lib.decorators.d.ts, lib.decorators.legacy.d.ts and 103 more)".into(),
            "Delete that directory, or clear bun's cache with `bun pm cache rm`, then rerun".into(),
        ] {
            assert!(t.contains(&want), "wanted `{want}` in:\n{t}");
        }
        for unwanted in ["panic:", "goroutine", "og check: ok"] {
            assert!(!t.contains(unwanted), "`{unwanted}` in:\n{t}");
        }
    };
    // Found before tsgo runs.
    check(&base);
    // Out of og's sight (a bunfig.toml cache directory): read from the panic.
    #[cfg(unix)]
    check(&dir.join("elsewhere"));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Without a tsconfig.json of its own, a bare entry and a project are
/// checked with the template's options and `paths` to the engine package
/// og resolved (here the checkout's): a type error in the entry, and one
/// in a playtest the entry never imports, are found; fixed, both pass.
#[test]
fn check_synthesises_the_configuration_a_project_does_not_have() {
    assert!(
        cfg!(windows)
            || std::env::var_os("PATH")
                .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join("tsrs").is_file())),
        "this test needs tsrs on PATH (scripts/install-tools.sh)"
    );
    let dir = std::env::temp_dir().join(format!("og-check-synth-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("game/src")).unwrap();
    std::fs::create_dir_all(dir.join("game/tests")).unwrap();
    let entry = dir.join("bare.ts");
    let bad = "import { og, Vector3 } from \"@origingame/engine\";\nvoid og; void Vector3;\nexport const n: number = \"one\";\n";
    std::fs::write(&entry, bad).unwrap();
    let check = |path: &Path| {
        let out = og()
            .args(["check", "--json"])
            .arg(path)
            .current_dir(&dir)
            .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
            .env_remove("OG_TSRS")
            .output()
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(out.stdout.trim_ascii())
            .unwrap_or_else(|e| panic!("{e}: {}", text(&out)));
        (out.status.code(), v)
    };
    let (code, v) = check(&entry);
    assert_eq!(code, Some(1), "{v}");
    assert_eq!(v["tool"], if cfg!(windows) { "tsgo" } else { "tsrs" });
    assert_eq!(
        (
            v["diagnostics"][0]["file"].as_str(),
            v["diagnostics"][0]["line"].as_u64(),
            v["diagnostics"][0]["code"].as_u64()
        ),
        (Some("bare.ts"), Some(3), Some(2322)),
        "{v}"
    );
    assert_eq!(
        v["diagnostics"].as_array().unwrap().len(),
        1,
        "the engine resolved: {v}"
    );
    std::fs::write(&entry, bad.replace("\"one\"", "1")).unwrap();
    assert_eq!(check(&entry).0, Some(0));

    let game = dir.join("game");
    std::fs::write(
        game.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    std::fs::write(
        game.join("src/main.ts"),
        "import { og } from \"@origingame/engine\";\nexport default og.game({});\n",
    )
    .unwrap();
    std::fs::write(
        game.join("tests/smoke.og.ts"),
        "import { playtest } from \"@origingame/engine/playtest\";\nexport default playtest({ name: \"n\", commands: \"0.1:quit\", check(run, t) { t.ok(run.quit); } });\n",
    )
    .unwrap();
    let (code, v) = check(&game);
    assert_eq!(code, Some(1), "{v}");
    assert_eq!(v["config"], ".og/tsconfig.check.json");
    assert_eq!(v["diagnostics"][0]["file"], "tests/smoke.og.ts", "{v}");
    assert_eq!(v["diagnostics"][0]["code"], 2554, "{v}");
    let written = std::fs::read_to_string(game.join(".og/tsconfig.check.json")).unwrap();
    assert!(
        written.starts_with("// Written by og check")
            && written.contains("exactOptionalPropertyTypes"),
        "{written}"
    );
    std::fs::write(
        game.join("tests/smoke.og.ts"),
        std::fs::read_to_string(game.join("tests/smoke.og.ts"))
            .unwrap()
            .replace("t.ok(run.quit)", "t.ok(run.quit, \"quit\")"),
    )
    .unwrap();
    assert_eq!(check(&game).0, Some(0));
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
