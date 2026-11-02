use super::*;

/// An entry file named relative to the project directory opens its
/// project: `og cook src/main.ts` there cooks it (the walk up for
/// project.ogt once ended at the empty path, M7 run 13).
#[test]
fn an_entry_relative_to_the_project_opens_its_project() {
    let dir = std::env::temp_dir().join(format!("og-cli-entry-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"entry\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/main.ts"), "export {};\n").unwrap();
    let out = og()
        .args(["cook", "src/main.ts"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("og cook: "),
        "{out:?}"
    );
}

/// Project commands outside a project name the fix and do nothing; usage
/// and unknown commands exit 2. (A real cook and playtest of the template,
/// which needs the native addon, is `packages/engine/tests/template.test.ts`.)
#[test]
fn project_commands_without_a_project_name_the_fix() {
    let empty = scratch_root();
    for cmd in ["build", "cook", "test"] {
        let out = og().arg(cmd).current_dir(&empty).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "og {cmd}");
        let t = text(&out);
        assert!(t.contains("no project.ogt"), "og {cmd}: {t}");
        assert!(t.contains("npm create @origingame/game"), "og {cmd}: {t}");
        assert!(!empty.join("cooked").exists() && !empty.join("dist").exists());
    }
    let probe = og().arg("probe").output().unwrap();
    assert_eq!(probe.status.code(), Some(2));
    assert!(
        text(&probe).contains("usage: og probe <method>"),
        "{}",
        text(&probe)
    );

    let missing = og()
        .args(["run", "--headless", "/nonexistent/game.ts", "--ticks", "1"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(
        text(&missing).contains("neither a game entry nor a project directory"),
        "{}",
        text(&missing)
    );

    let unknown = og().arg("frobnicate").output().unwrap();
    assert_eq!(unknown.status.code(), Some(2));
    assert!(text(&unknown).contains("unknown command `frobnicate`"));
    let _ = std::fs::remove_dir_all(&empty);
}

/// When the game process dies, `og run` says so at once with its exit
/// status (or signal) and its last stderr lines, and exits with that code,
/// so a supervisor waiting for the probe stops waiting.
#[test]
#[cfg(unix)]
fn run_reports_a_dead_game_process_at_once() {
    let dir = scratch_root();
    std::fs::remove_dir_all(dir.join("crates")).unwrap();
    let engine = dir.join("engine");
    std::fs::create_dir_all(engine.join("src/node")).unwrap();
    // A stand-in engine package: the bundle step writes the bundle, the run
    // step prints and is killed by a signal.
    std::fs::write(
        engine.join("src/node/build.ts"),
        "const i = process.argv.indexOf('--out'); await Bun.write(process.argv[i + 1], 'export default 0;');\n",
    )
    .unwrap();
    std::fs::write(
        engine.join("src/node/run.ts"),
        "console.error('probe: listening'); console.error('the addon faulted here'); process.kill(process.pid, 'SIGKILL');\n",
    )
    .unwrap();
    std::fs::write(dir.join("game.ts"), "export default 0;\n").unwrap();
    let t0 = std::time::Instant::now();
    let out = og()
        .args(["run", "--headless", "--port", "47999"])
        .arg(dir.join("game.ts"))
        .env("OG_ENGINE_PACKAGE", &engine)
        .output()
        .unwrap();
    let t = text(&out);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(t0.elapsed() < std::time::Duration::from_secs(20), "{t}");
    assert_eq!(out.status.code(), Some(137), "{t}");
    assert!(t.contains("the game process was killed by signal 9"), "{t}");
    assert!(t.contains("  | the addon faulted here"), "{t}");
}

/// project.ogt's publish block is checked before anything is built: a bad
/// value names its line and the values it takes.
#[test]
fn build_refuses_a_bad_publish_setting_by_line() {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    let manifest = |publish: &str| {
        format!(
            "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\npublish {{\n{publish}\n}}\n"
        )
    };
    for (publish, expect) in [
        (
            "  fit stretch",
            "project.ogt:7: publish.fit is `stretch`; use one of fill, letterbox",
        ),
        (
            "  dpr_cap 9",
            "publish.dpr_cap is 9; use a number from 0.5 to 4",
        ),
        ("  aspect \"wide\"", "write width:height, such as \"16:9\""),
        ("  fit letterbox", "add `aspect \"16:9\"`"),
        ("  color red", "publish has no `color`"),
    ] {
        std::fs::write(dir.join("project.ogt"), manifest(publish)).unwrap();
        let out = og().arg("build").arg(&dir).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{publish}: {}", text(&out));
        assert!(text(&out).contains(expect), "{publish}: {}", text(&out));
        assert!(!dir.join("dist").exists());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A 2D project's publish build refuses the 3D world in its scenes (the 2D
/// publish module leaves terrain, planar water, scattered copies and 3D
/// fluids out: RenderCapabilities.world3d), naming each by scene and line,
/// before anything is cooked or exported; a 2D project without them, its
/// dev build, and a 3D project with them build as before.
#[test]
fn a_2d_publish_build_refuses_the_3d_world_by_scene_and_line() {
    let dir = scratch_root();
    std::fs::remove_dir_all(dir.join("crates")).unwrap();
    // A stand-in engine package: the exporter leaves a mark when it runs.
    let engine = dir.join("engine");
    std::fs::create_dir_all(engine.join("src/node")).unwrap();
    let mark = dir.join("exported");
    std::fs::write(
        engine.join("src/node/web-build.ts"),
        format!("await Bun.write({:?}, '');\n", mark.to_str().unwrap()),
    )
    .unwrap();
    let game = dir.join("game");
    std::fs::create_dir_all(game.join("src")).unwrap();
    std::fs::create_dir_all(game.join("assets/scenes")).unwrap();
    std::fs::write(game.join("src/main.ts"), "export default 0;\n").unwrap();
    let project = |profile: &str| {
        std::fs::write(
            game.join("project.ogt"),
            format!(
                "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\nprofile \"{profile}\"\n"
            ),
        )
        .unwrap();
    };
    let scene = |with_3d: bool| {
        let extra = if with_3d {
            "  Terrain {\n    castShadows false\n  }\n  entity \"pond\" {\n    Water {\n    }\n  }\n"
        } else {
            ""
        };
        std::fs::write(
            game.join("assets/scenes/main.ogt"),
            format!(
                "type scene\nversion 1\n---\nname \"Main\"\nentity \"ground\" {{\n  Transform {{\n    position 0 0 0\n  }}\n{extra}}}\n"
            ),
        )
        .unwrap();
    };
    let build = |args: &[&str]| {
        let _ = std::fs::remove_file(&mark);
        let out = og()
            .args(args)
            .arg(&game)
            .env("OG_ENGINE_PACKAGE", &engine)
            .output()
            .unwrap();
        (out.status.code(), text(&out), mark.exists())
    };

    project("2d");
    scene(true);
    let (code, t, exported) = build(&["build", "--publish"]);
    assert_eq!(code, Some(1), "{t}");
    assert!(!exported, "the exporter ran past the refusal: {t}");
    // Nothing was cooked or exported: the project holds what it was given.
    let mut made: Vec<String> = std::fs::read_dir(&game)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    made.sort();
    assert_eq!(made, ["assets", "project.ogt", "src"], "{t}");
    let assets: Vec<_> = std::fs::read_dir(game.join("assets")).unwrap().collect();
    assert_eq!(assets.len(), 1, "{t}");
    for line in [
        "assets/scenes/main.ogt:9: Terrain (a terrain)",
        "assets/scenes/main.ogt:13: Water (planar water)",
    ] {
        assert!(t.contains(line), "{line}: {t}");
    }
    assert!(t.contains("RenderCapabilities.world3d is false"), "{t}");
    assert!(t.contains("profile \"3d\""), "{t}");

    // The dev build has every subsystem; a 3D project's publish keeps them,
    // once the game is out of the default style (docs/ui.md §12.1): a
    // look that picks none is named, and one whose `ui` line picks a
    // style publishes.
    let (code, t, exported) = build(&["build"]);
    assert_eq!((code, exported), (Some(0), true), "{t}");
    project("3d");
    std::fs::create_dir_all(game.join("assets/looks")).unwrap();
    let look = |ui: &str| {
        std::fs::write(
            game.join("assets/looks/main.ogt"),
            format!("type look\nversion 1\n---\nname \"Main\"\n{ui}"),
        )
        .unwrap();
    };
    look("");
    let (code, t, exported) = build(&["build", "--publish"]);
    assert_eq!((code, exported), (Some(1), false), "{t}");
    assert!(t.contains("still draws its UI in the default style"), "{t}");
    assert!(t.contains("assets/looks/main.ogt"), "{t}");
    look("ui \"@styles/paper/style.ogt\"\n");
    let (code, t, exported) = build(&["build", "--publish"]);
    assert_eq!((code, exported), (Some(0), true), "{t}");
    // A 2D project without them publishes as before.
    project("2d");
    scene(false);
    let (code, t, exported) = build(&["build", "--publish"]);
    assert_eq!((code, exported), (Some(0), true), "{t}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A scene row that omits a field of an engine component gets the IDL's
/// default (og.Transform's quaternion identity, scale one), not zero: a
/// sprite on an entity that only states its position must not vanish.
#[test]
fn cook_defaults_the_engine_component_fields_a_scene_omits() {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("assets/scenes")).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("assets/scenes/main.ogt"),
        "type scene\nversion 1\n---\nname \"Main\"\nentity \"probe\" {\n  Transform {\n    position 1 2 3\n  }\n}\nentity \"turned\" {\n  Transform {\n    scale 2 2 2\n  }\n}\n",
    )
    .unwrap();
    let out = og().arg("cook").arg(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    let id = "scenes/main.ogt";
    let scene = bundle
        .scene(id, bundle.read(id).unwrap().as_slice())
        .unwrap();
    let table = scene
        .components
        .iter()
        .find(|t| t.name == "Transform")
        .expect("a Transform table");
    assert!(!table.inferred, "packed with the engine's schema");
    let layout = og_core::reflect::Layout::of(&table.schema);
    let lanes = |row: usize, field: &str| -> Vec<f32> {
        let i = table.schema.field_index(field).unwrap();
        table.get(row).unwrap()[layout.range(i)]
            .chunks(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect()
    };
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(lanes(0, "position"), [1.0, 2.0, 3.0]);
    assert_eq!(lanes(0, "quaternion"), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(lanes(0, "scale"), [1.0, 1.0, 1.0]);
    assert_eq!(lanes(1, "position"), [0.0, 0.0, 0.0]);
    assert_eq!(lanes(1, "scale"), [2.0, 2.0, 2.0]);
}

/// `og dev` stopped from outside (SIGTERM from a supervisor, SIGHUP from a
/// closed terminal, SIGINT) stops its game process and whatever that
/// started (here a stand-in for a browser), then ends by the same signal;
/// it used to die alone and leave its bun child running.
#[cfg(unix)]
#[test]
fn dev_stopped_by_a_signal_stops_its_game_and_what_it_started() {
    let alive = |pid: i32| unsafe { libc::kill(pid, 0) == 0 };
    for (sig, name) in [
        (libc::SIGTERM, "SIGTERM"),
        (libc::SIGHUP, "SIGHUP"),
        (libc::SIGINT, "SIGINT"),
    ] {
        let dir = scratch_root();
        std::fs::remove_dir_all(dir.join("crates")).unwrap();
        let engine = dir.join("engine");
        std::fs::create_dir_all(engine.join("src/node")).unwrap();
        std::fs::write(
            engine.join("src/node/build.ts"),
            "const i = process.argv.indexOf('--out'); await Bun.write(process.argv[i + 1], 'export default 0;');\n",
        )
        .unwrap();
        // The game: records its pid and a child's (a stand-in for a browser
        // it opened), ignores nothing, and runs until stopped.
        let pids = dir.join("pids.txt");
        std::fs::write(
            engine.join("src/node/run.ts"),
            format!(
                "const child = Bun.spawn(['sleep', '600']);\nawait Bun.write({:?}, `${{process.pid}} ${{child.pid}}`);\nsetInterval(() => {{}}, 1000);\n",
                pids.to_str().unwrap()
            ),
        )
        .unwrap();
        let project = dir.join("game");
        std::fs::create_dir_all(project.join("src")).unwrap();
        std::fs::write(project.join("src/main.ts"), "export default 0;\n").unwrap();
        std::fs::write(
            project.join("project.ogt"),
            "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
        )
        .unwrap();
        // A file, not a pipe: a child left running would hold a pipe open.
        let log = dir.join("og-dev.log");
        let mut og_dev = og()
            .args(["dev", "--port", "47998"])
            .arg(&project)
            .env("OG_ENGINE_PACKAGE", &engine)
            .stdout(std::process::Stdio::null())
            .stderr(std::fs::File::create(&log).unwrap())
            .spawn()
            .unwrap();
        let t0 = std::time::Instant::now();
        let (game, browser) = loop {
            if let Ok(t) = std::fs::read_to_string(&pids)
                && let [a, b] = t.split_whitespace().collect::<Vec<_>>()[..]
            {
                break (a.parse::<i32>().unwrap(), b.parse::<i32>().unwrap());
            }
            assert!(
                t0.elapsed() < std::time::Duration::from_secs(30),
                "{name}: the stand-in game never started"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        assert!(alive(game) && alive(browser), "{name}");
        unsafe { libc::kill(og_dev.id() as i32, sig) };
        let t1 = std::time::Instant::now();
        let status = loop {
            if let Some(s) = og_dev.try_wait().unwrap() {
                break s;
            }
            if t1.elapsed() > std::time::Duration::from_secs(10) {
                let _ = og_dev.kill();
                panic!("{name}: og dev did not end within 10 s");
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        let err = std::fs::read_to_string(&log).unwrap_or_default();
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(sig), "{name}: {status:?} {err}");
        // Both are gone (a zombie of the game is reaped by its new parent).
        let t2 = std::time::Instant::now();
        while (alive(game) || alive(browser)) && t2.elapsed() < std::time::Duration::from_secs(5) {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let left = (alive(game), alive(browser));
        if left.1 {
            unsafe { libc::kill(browser, libc::SIGKILL) };
        }
        if left.0 {
            unsafe { libc::kill(game, libc::SIGKILL) };
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(
            left,
            (false, false),
            "{name}: game and browser still running: {err}"
        );
    }
}

/// `og cook --watch` cooks a changed asset by itself and reports its time
/// against the 300 ms budget; `og cook --report` prints the slowest assets
/// by their last cook, kept in cooked/.cook-times.ogt across up-to-date
/// cooks.
#[test]
fn cook_watch_times_one_changed_asset_and_report_prints_the_slowest() {
    use std::io::{BufRead, BufReader, Write};
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("assets/palettes")).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    for (name, colours) in [("dusk", "203040\n405060\n"), ("sea", "102030\n")] {
        std::fs::write(dir.join(format!("assets/palettes/{name}.hex")), colours).unwrap();
    }
    std::fs::write(
        dir.join("assets/grade.cube"),
        "LUT_3D_SIZE 2\n0 0 0\n1 0 0\n0 1 0\n1 1 0\n0 0 1\n1 0 1\n0 1 1\n1 1 1\n",
    )
    .unwrap();

    let mut child = og()
        .arg("cook")
        .arg(&dir)
        .args(["--watch", "--json", "--exit-after", "1"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let first: serde_json::Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
    assert_eq!(first["event"], "watching", "{first}");
    // The first cook is done: one palette changes.
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(dir.join("assets/palettes/sea.hex"))
        .unwrap();
    f.write_all(b"aabbcc\n").unwrap();
    drop(f);
    let cook: serde_json::Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "{cook}");
    assert_eq!(cook["event"], "cook");
    assert_eq!(cook["changed"], serde_json::json!(["palettes/sea.hex"]));
    let assets = cook["assets"].as_array().unwrap();
    assert_eq!(assets.len(), 1, "only the changed asset cooks: {cook}");
    assert_eq!(assets[0]["source"], "palettes/sea.hex");
    assert!(
        assets[0]["ms"].as_f64().is_some_and(|ms| ms >= 0.0),
        "{cook}"
    );
    assert_eq!(cook["budgetMs"], 300.0);
    assert!(cook["withinBudget"].is_boolean() && cook["ms"].as_f64().unwrap() > 0.0);

    // The times survive an up-to-date cook; --report prints them.
    let times = og_cook::CookTimes::read(&dir.join("cooked"));
    assert_eq!(times.assets.len(), 3, "{times:?}");
    let out = og().arg("cook").arg(&dir).arg("--report").output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let printed = text(&out);
    assert!(printed.contains("the slowest 3 asset(s)"), "{printed}");
    for (name, _) in times.slowest(3) {
        assert!(printed.contains(name), "{name}: {printed}");
    }
    assert_eq!(og_cook::CookTimes::read(&dir.join("cooked")), times);
    let md = dir.join("report.md");
    let out = og()
        .arg("cook")
        .arg(&dir)
        .arg("--report")
        .arg(&md)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let report = std::fs::read_to_string(&md).unwrap();
    assert!(report.contains("## The slowest ten") && report.contains("palettes/sea.hex"));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Filtering selects project-relative suite paths, not the project directory
/// or test titles. Unselected failing suites must not run; typos must fail.
#[test]
fn test_filters_unit_and_playtest_paths_and_refuses_no_match() {
    let dir = scratch_root();
    std::fs::remove_dir_all(dir.join("crates")).unwrap();
    let engine = dir.join("engine");
    std::fs::create_dir_all(engine.join("src/node")).unwrap();
    std::fs::write(
        engine.join("src/node/build.ts"),
        "await Bun.write(process.argv[process.argv.indexOf('--out') + 1], 'export default 0;');",
    )
    .unwrap();
    std::fs::write(
        engine.join("src/node/playtest-run.ts"),
        "await import(process.argv[process.argv.indexOf('--file') + 1]);",
    )
    .unwrap();
    let game = dir.join("game");
    std::fs::create_dir_all(game.join("tests/nested")).unwrap();
    std::fs::create_dir_all(game.join("src")).unwrap();
    std::fs::write(game.join("src/main.ts"), "export default 0;").unwrap();
    std::fs::write(
        game.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    for (path, source) in [
        ("tests/nested/chosen.og.ts", "export default 0;"),
        (
            "tests/rejected.og.ts",
            "throw new Error('unselected playtest');",
        ),
        (
            "tests/chosen.test.ts",
            "import { test, expect } from 'bun:test'; test('not a path', () => expect(3).toBe(3));",
        ),
        (
            "tests/rejected.test.ts",
            "import { test, expect } from 'bun:test'; test('chosen', () => expect(3).toBe(4));",
        ),
    ] {
        std::fs::write(game.join(path), source).unwrap();
    }
    // --skip leaves out what it names, after --filter; it may repeat.
    for (args, suites) in [
        (vec!["--filter", "chosen"], 2),
        (vec!["--filter", "tests/nested/"], 1),
        (vec!["--filter", "chosen.test.ts"], 1),
        (vec!["--filter", "chosen", "--skip", ".test.ts"], 1),
        (vec!["--skip", "rejected"], 2),
        (vec!["--skip", "rejected.og", "--skip", "rejected.test"], 2),
    ] {
        let filter = args.join(" ");
        let out = og()
            .args(["test", "--no-check", "--json"])
            .args(&args)
            .arg(&game)
            .env("OG_ENGINE_PACKAGE", &engine)
            .output()
            .unwrap();
        let t = text(&out);
        assert!(out.status.success(), "{filter}: {t}");
        assert!(t.contains(&format!("\"suites\":{suites}")), "{t}");
    }
    for args in [
        vec!["--filter", "missing"],
        vec!["--filter", "game"],
        vec!["--filter"],
        vec!["--filter", "chosen", "--skip", "chosen"],
        vec!["--skip"],
    ] {
        let out = og()
            .arg("test")
            .arg(&game)
            .arg("--no-check")
            .args(&args)
            .env("OG_ENGINE_PACKAGE", &engine)
            .output()
            .unwrap();
        assert!(!out.status.success(), "{}", text(&out));
        let flag = if args.contains(&"--skip") {
            "--skip"
        } else {
            "--filter"
        };
        assert!(text(&out).contains(flag), "{}", text(&out));
    }
    let out = og()
        .arg("test")
        .arg(&game)
        .arg("--no-check")
        .env("OG_ENGINE_PACKAGE", &engine)
        .output()
        .unwrap();
    assert!(!out.status.success(), "unfiltered failing suites must run");
    let _ = std::fs::remove_dir_all(dir);
}

/// `--jobs` runs playtests at once yet prints each one's output whole and
/// in the playtests' order, and still counts every failure: the first
/// playtest ends only once the fourth has started beside it, and the third
/// fails. Run one at a time, the first gives up waiting and fails too.
#[test]
fn test_jobs_runs_playtests_at_once_and_prints_them_in_order() {
    let dir = scratch_root();
    std::fs::remove_dir_all(dir.join("crates")).unwrap();
    let engine = dir.join("engine");
    std::fs::create_dir_all(engine.join("src/node")).unwrap();
    std::fs::write(
        engine.join("src/node/build.ts"),
        "await Bun.write(process.argv[process.argv.indexOf('--out') + 1], 'export default 0;');",
    )
    .unwrap();
    std::fs::write(
        engine.join("src/node/playtest-run.ts"),
        r#"import { basename, dirname, join } from "node:path";
const f = process.argv[process.argv.indexOf("--file") + 1]!;
const n = basename(f, ".og.ts");
const marker = join(dirname(f), "..", ".d-started");
console.log(`start ${n}`);
if (n === "d") await Bun.write(marker, "");
if (n === "a") {
  const t0 = Date.now();
  while (!(await Bun.file(marker).exists())) {
    if (Date.now() - t0 > 3000) { console.log("a: d never started beside it"); process.exit(1); }
    await Bun.sleep(20);
  }
}
console.log(`end ${n}`);
process.exit(n === "c" ? 1 : 0);
"#,
    )
    .unwrap();
    let game = dir.join("game");
    std::fs::create_dir_all(game.join("tests")).unwrap();
    std::fs::create_dir_all(game.join("src")).unwrap();
    std::fs::write(game.join("src/main.ts"), "export default 0;").unwrap();
    std::fs::write(
        game.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    for n in ["a", "b", "c", "d"] {
        std::fs::write(game.join(format!("tests/{n}.og.ts")), "export default 0;").unwrap();
    }
    let run = |jobs: &str| {
        let _ = std::fs::remove_file(game.join(".d-started"));
        let out = og()
            .args(["test", "--no-check", "--jobs", jobs])
            .arg(&game)
            .env("OG_ENGINE_PACKAGE", &engine)
            .output()
            .unwrap();
        (out.status.code(), text(&out))
    };
    let (code, t) = run("3");
    assert_eq!(code, Some(1), "{t}");
    assert!(t.contains("og test: 1 of 4 suite(s) failed"), "{t}");
    let lines: Vec<&str> = t
        .lines()
        .filter(|l| l.starts_with("start ") || l.starts_with("end "))
        .collect();
    assert_eq!(
        lines,
        [
            "start a", "end a", "start b", "end b", "start c", "end c", "start d", "end d"
        ],
        "{t}"
    );
    let (code, t) = run("1");
    assert_eq!(code, Some(1), "{t}");
    assert!(t.contains("og test: 2 of 4 suite(s) failed"), "{t}");
    for bad in ["0", "x"] {
        let (code, t) = run(bad);
        assert_eq!(code, Some(2), "{t}");
        assert!(t.contains("--jobs"), "{t}");
    }
    let _ = std::fs::remove_dir_all(dir);
}

/// `og dev` checks a source change beside the rebuild and never refuses the
/// reload over it: the reload goes through, its event carries the check
/// (or `null` while it runs), the `check` event lists the diagnostic, and
/// `--exit-after` waits for that check to land.
#[test]
fn dev_reports_the_check_of_each_save_and_reloads_anyway() {
    use std::io::{BufRead, BufReader};
    assert!(
        cfg!(windows)
            || std::env::var_os("PATH")
                .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join("tsrs").is_file())),
        "this test needs tsrs on PATH (scripts/install-tools.sh)"
    );
    let dir = scratch_root();
    std::fs::remove_dir_all(dir.join("crates")).unwrap();
    let engine = dir.join("engine");
    std::fs::create_dir_all(engine.join("src/node")).unwrap();
    std::fs::write(
        engine.join("src/node/build.ts"),
        "const i = process.argv.indexOf('--out'); await Bun.write(process.argv[i + 1], `export default ${Date.now()};`);\n",
    )
    .unwrap();
    // The game: a reload event each time the watched bundle changes.
    std::fs::write(
        engine.join("src/node/run.ts"),
        "import { statSync } from 'node:fs';\nconst f = process.argv[process.argv.indexOf('--watch') + 1];\nlet m = statSync(f).mtimeMs, n = 0;\nsetInterval(() => { const t = statSync(f).mtimeMs; if (t !== m) { m = t; console.log(JSON.stringify({ event: 'reload', ok: true, latencyMs: 1, report: {}, tick: ++n })); } }, 10);\n",
    )
    .unwrap();
    let game = dir.join("game");
    std::fs::create_dir_all(game.join("src")).unwrap();
    std::fs::write(game.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        game.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let mut child = og()
        .args(["dev", "--json", "--exit-after", "1", "--port", "47997"])
        .arg(&game)
        .env("OG_ENGINE_PACKAGE", &engine)
        .env_remove("OG_TSRS")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut next = || -> serde_json::Value {
        serde_json::from_str(&lines.next().expect("og dev ended").unwrap()).unwrap()
    };
    // The first bundle's check, then a save with a type error.
    let first = next();
    assert_eq!(
        (first["event"].as_str(), first["ok"].as_bool()),
        (Some("check"), Some(true)),
        "{first}"
    );
    std::thread::sleep(std::time::Duration::from_millis(100));
    std::fs::write(
        game.join("src/main.ts"),
        "export const n: number = \"one\";\nexport default 0;\n",
    )
    .unwrap();
    // A file write can produce more than one watched modification. Read
    // through shutdown: --exit-after waits for the pending check to land.
    let events: Vec<serde_json::Value> = lines
        .map(|line| serde_json::from_str(&line.unwrap()).unwrap())
        .collect();
    let status = child.wait().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let reload = events
        .iter()
        .find(|event| event["event"] == "reload")
        .expect("the saved source reloads");
    let check = events
        .iter()
        .rfind(|event| event["event"] == "check")
        .expect("the saved source is checked before shutdown");
    assert_eq!(reload["event"], "reload", "{reload}");
    assert_eq!(reload["ok"], true, "the reload went through: {reload}");
    assert!(
        reload["check"].is_null()
            || reload["check"]
                == serde_json::json!({ "ok": false, "ms": check["ms"], "errors": 1 }),
        "{reload}"
    );
    assert_eq!(check["event"], "check", "{check}");
    assert_eq!(check["diagnostics"][0]["code"], 2322, "{check}");
    assert_eq!(check["diagnostics"][0]["file"], "src/main.ts", "{check}");
    assert!(status.success(), "{status:?}");
}
