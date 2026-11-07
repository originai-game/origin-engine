use super::*;

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
    // in whatever UI style the game draws (the default one included).
    let (code, t, exported) = build(&["build"]);
    assert_eq!((code, exported), (Some(0), true), "{t}");
    project("3d");
    let (code, t, exported) = build(&["build", "--publish"]);
    assert_eq!((code, exported), (Some(0), true), "{t}");
    // A 2D project without them publishes as before.
    project("2d");
    scene(false);
    let (code, t, exported) = build(&["build", "--publish"]);
    assert_eq!((code, exported), (Some(0), true), "{t}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `og dev` checks a source change beside the rebuild and never refuses the
/// reload over it. An error and its correction each reload with their own
/// check, including its exact duration, rather than the last check of the
/// whole session. The fixture releases each reload after its check lands.
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
    // The game acknowledges each bundle, then waits for the test to release
    // its reload after that source's check. No timing race decides which
    // check is available when the reload is reported.
    std::fs::write(
        engine.join("src/node/run.ts"),
        r#"import { statSync, existsSync, readFileSync } from 'node:fs';
const f = process.argv[process.argv.indexOf('--watch') + 1];
const release = process.env.RELEASE_RELOAD;
let m = statSync(f).mtimeMs, bundled = 0, reloaded = 0;
console.log(JSON.stringify({ event: 'ready' }));
setInterval(() => {
  const t = statSync(f).mtimeMs;
  if (t !== m) {
    m = t;
    console.log(JSON.stringify({ event: 'bundled', n: ++bundled }));
  }
  if (reloaded < bundled && existsSync(release) && Number(readFileSync(release, 'utf8')) > reloaded) {
    console.log(JSON.stringify({ event: 'reload', ok: true, latencyMs: 1000, report: {}, tick: ++reloaded }));
  }
}, 10);
"#,
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
        .args(["dev", "--json", "--exit-after", "2", "--port", "47997"])
        .arg(&game)
        .env("OG_ENGINE_PACKAGE", &engine)
        .env("RELEASE_RELOAD", dir.join("release"))
        .env_remove("OG_TSRS")
        .stdout(std::process::Stdio::piped())
        .stderr(std::fs::File::create(dir.join("og-dev.err")).unwrap())
        .spawn()
        .unwrap();
    // Each event within a minute (one takes about a second): a missing one
    // fails with what og dev said instead of holding the run until
    // nextest's 360 s timeout (it did once on a loaded gate, 2026-11-07).
    let (tx, rx) = std::sync::mpsc::channel();
    let stdout = child.stdout.take().unwrap();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let _ = tx.send(line.unwrap());
        }
    });
    let mut seen = Vec::<String>::new();
    let pid = child.id();
    let mut next_event = || -> Option<serde_json::Value> {
        match rx.recv_timeout(std::time::Duration::from_secs(60)) {
            Ok(line) => {
                seen.push(line.clone());
                Some(serde_json::from_str(&line).unwrap())
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => None,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // og dev stops its game on a signal; left alone it runs on.
                let _ = std::process::Command::new("kill")
                    .arg(pid.to_string())
                    .status();
                panic!(
                    "og dev printed nothing for 60 s; its events: {seen:?}; its stderr: {}",
                    std::fs::read_to_string(dir.join("og-dev.err")).unwrap_or_default()
                )
            }
        }
    };
    let (mut ready, mut checked) = (false, false);
    while !ready || !checked {
        let event = next_event().expect("og dev ended");
        ready |= event["event"] == "ready";
        if event["event"] == "check" {
            assert_eq!(event["ok"], true, "{event}");
            checked = true;
        }
    }
    let mut events = Vec::<serde_json::Value>::new();
    for (i, source) in [
        "export const n: number = \"one\";\nexport default 0;\n",
        "export const n: number = 1;\nexport default 0;\n",
    ]
    .iter()
    .enumerate()
    {
        // Rename a complete file into place: one source modification per
        // phase, rather than a truncate and a write observed separately.
        let staged = dir.join("saved.ts");
        std::fs::write(&staged, source).unwrap();
        std::fs::rename(staged, game.join("src/main.ts")).unwrap();
        let (mut bundled, mut checked) = (false, false);
        while !bundled || !checked {
            let event = next_event().expect("og dev ended");
            bundled |= event["event"] == "bundled";
            checked |= event["event"] == "check";
            events.push(event);
        }
        std::fs::write(dir.join("release"), (i + 1).to_string()).unwrap();
        loop {
            let event = next_event().expect("og dev ended");
            let reloaded = event["event"] == "reload";
            events.push(event);
            if reloaded {
                break;
            }
        }
    }
    while let Some(event) = next_event() {
        events.push(event);
    }
    let status = child.wait().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let mut pending = None;
    let mut reloads = 0;
    for event in &events {
        if event["event"] == "check" {
            assert!(pending.replace(event).is_none(), "{events:?}");
            assert_eq!(event["ok"], reloads == 1, "{event}");
            if reloads == 0 {
                assert_eq!(event["diagnostics"][0]["code"], 2322, "{event}");
                assert_eq!(event["diagnostics"][0]["file"], "src/main.ts", "{event}");
            } else {
                assert_eq!(event["diagnostics"], serde_json::json!([]), "{event}");
            }
        } else if event["event"] == "reload" {
            let check = pending
                .take()
                .expect("each reload has its own landed check");
            assert_eq!(event["ok"], true, "the reload went through: {event}");
            assert_eq!(event["swapMs"].as_f64(), Some(1000.0), "{event}");
            assert_eq!(event["withinBudget"], false, "latency is reported: {event}");
            assert_eq!(
                event["check"],
                serde_json::json!({
                    "ok": check["ok"], "ms": check["ms"],
                    "errors": check["diagnostics"].as_array().unwrap().len(),
                }),
                "{events:?}"
            );
            reloads += 1;
        }
    }
    assert_eq!(reloads, 2, "{events:?}");
    assert!(pending.is_none(), "{events:?}");
    assert!(status.success(), "{status:?}");
}
