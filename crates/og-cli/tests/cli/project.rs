use super::*;

#[test]
fn tests_default_to_hardware_but_preserve_explicit_renderer_selection() {
    let dir = TempDir::new("og-test-renderer");
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("main.ts"), "export default {};").unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"Test\"\nentry \"main.ts\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("tests/renderer.test.ts"), "import { test, expect } from 'bun:test'; test('renderer policy', () => expect(process.env.OG_RENDERER).toBe(process.env.EXPECT_RENDERER));").unwrap();
    for renderer in [None, Some("software"), Some("none")] {
        let mut command = og();
        command.args(["test", dir.to_str().unwrap(), "--no-check"]);
        command.env("EXPECT_RENDERER", renderer.unwrap_or("hardware"));
        command.env_remove("OG_RENDERER");
        if let Some(renderer) = renderer {
            command.env("OG_RENDERER", renderer);
        }
        let out = command.output().unwrap();
        assert!(out.status.success(), "{}", text(&out));
    }
}

/// Sessions can bind and serve a request even when HOME and runtime hints
/// are longer than a Unix socket address on every supported native host.
#[cfg(unix)]
#[test]
fn session_serves_a_request_with_a_long_home_and_runtime_directory() {
    let dir = TempDir::new("og-session-long-home");
    let home = dir.join("long-home-directory-".repeat(12));
    std::fs::create_dir_all(&home).unwrap();
    let engine = dir.join("engine");
    std::fs::create_dir_all(engine.join("src/node")).unwrap();
    std::fs::write(
        engine.join("src/node/build.ts"),
        "await Bun.write(process.argv[process.argv.indexOf('--out') + 1], '');",
    )
    .unwrap();
    std::fs::write(engine.join("src/node/session.ts"), r#"
import { createServer } from 'node:net';
const server = createServer(socket => socket.once('data', () => {
  for (const event of [{event:'accepted'}, {event:'stdout', text:'long-home session served\n'}, {event:'exit', code:0}])
    socket.write(JSON.stringify(event) + '\n');
  socket.end();
  server.close();
}));
server.listen(process.argv[2]);
"#).unwrap();
    let entry = dir.join("main.ts");
    std::fs::write(&entry, "export default {};").unwrap();
    let addon = dir.join("fixture-addon.node");
    std::fs::write(&addon, "fixture").unwrap();
    let out = og()
        .args(["run", entry.to_str().unwrap(), "--headless", "--no-check"])
        .env("OG_ENGINE_PACKAGE", &engine)
        .env("OG_NATIVE_ADDON", &addon)
        .env("OG_SESSION", "1")
        .env("HOME", &home)
        .env("XDG_RUNTIME_DIR", &home)
        .env("TMPDIR", &home)
        .output()
        .unwrap();
    let t = text(&out);
    assert!(out.status.success(), "{t}");
    assert!(t.contains("long-home session served"), "{t}");
}

/// A failed supervisor reports its own diagnostic, not a readiness timeout.
#[cfg(unix)]
#[test]
fn session_startup_failure_names_the_cause_without_waiting_for_readiness() {
    let dir = TempDir::new("og-session-failure");
    let engine = dir.join("engine");
    std::fs::create_dir_all(engine.join("src/node")).unwrap();
    std::fs::write(
        engine.join("src/node/build.ts"),
        "await Bun.write(process.argv[process.argv.indexOf('--out') + 1], '');",
    )
    .unwrap();
    std::fs::write(
        engine.join("src/node/session.ts"),
        "console.error('fixture startup failure'); process.exit(23);",
    )
    .unwrap();
    let entry = dir.join("main.ts");
    std::fs::write(&entry, "export default {}; ").unwrap();
    let addon = dir.join("fixture-addon.node");
    for present in [false, true] {
        if present {
            std::fs::write(&addon, "fixture").unwrap();
        }
        let started = std::time::Instant::now();
        let out = og()
            .args(["run", entry.to_str().unwrap(), "--headless", "--no-check"])
            .env("OG_ENGINE_PACKAGE", &engine)
            .env("OG_NATIVE_ADDON", &addon)
            .env("OG_SESSION", "1")
            .env("HOME", &*dir)
            .output()
            .unwrap();
        let t = text(&out);
        assert!(!out.status.success(), "{t}");
        assert!(t.contains("session exited before becoming ready"), "{t}");
        assert!(t.contains("no command was sent"), "{t}");
        assert!(
            t.contains(if present {
                "fixture startup failure"
            } else {
                addon.to_str().unwrap()
            }),
            "{t}"
        );
        assert!(started.elapsed().as_secs() < 5, "{t}");
    }
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
    // A stand-in engine package: the exporter leaves a mark when it runs;
    // the bundler writes a bundle and its import graph (the cook's input).
    let engine = dir.join("engine");
    std::fs::create_dir_all(engine.join("src/node")).unwrap();
    std::fs::write(
        engine.join("src/node/build.ts"),
        "const i = process.argv.indexOf('--out'); await Bun.write(process.argv[i + 1], 'export default 0;'); await Bun.write(process.argv[i + 1] + '.graph.json', JSON.stringify({ engine: import.meta.dir, files: [] }));\n",
    )
    .unwrap();
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
                "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\ncreate {{\n  form \"{profile}\"\n}}\n"
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
    assert!(t.contains("create { form \"3d\" }"), "{t}");

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
    std::fs::create_dir_all(engine.join("types")).unwrap();
    std::fs::copy(
        repo().join("packages/engine/types/index.d.ts"),
        engine.join("types/index.d.ts"),
    )
    .unwrap();
    std::fs::write(
        engine.join("src/node/build.ts"),
        // The bundle and its import graph (an entry reaching no engine UI).
        "const i = process.argv.indexOf('--out'); await Bun.write(process.argv[i + 1], `export default ${Date.now()};`); await Bun.write(process.argv[i + 1] + '.graph.json', JSON.stringify({ engine: import.meta.dir, files: [] }));\n",
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
    std::fs::create_dir_all(game.join("assets/ui")).unwrap();
    let document = "<meta name='screen' content='shop'><meta name='prop:gold' content='number' data-doc='Gold.'><div></div>";
    std::fs::write(game.join("assets/ui/shop.svelte"), document).unwrap();
    std::fs::write(game.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        game.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let mut child = og()
        .args(["dev", "--json", "--exit-after", "3", "--port", "47997"])
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
    for (i, (file, source)) in [
        (
            "src/main.ts",
            "export const n: number = \"one\";\nexport default 0;\n".to_string(),
        ),
        (
            "src/main.ts",
            "export const n: number = 1;\nexport default 0;\n".to_string(),
        ),
        (
            "assets/ui/shop.svelte",
            document.replace("content='number'", "content='string'"),
        ),
    ]
    .iter()
    .enumerate()
    {
        // Rename a complete file into place: one source modification per
        // phase, rather than a truncate and a write observed separately.
        let staged = dir.join("saved.ts");
        std::fs::write(&staged, source).unwrap();
        std::fs::rename(staged, game.join(file)).unwrap();
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
    let types = std::fs::read_to_string(game.join(".og/types/ui/shop.d.ts")).unwrap();
    assert!(
        types.contains("gold?: string"),
        "the .svelte save must retype: {types}"
    );
    let _ = std::fs::remove_dir_all(&dir);
    let mut pending = None;
    let mut reloads = 0;
    for event in &events {
        if event["event"] == "check" {
            assert!(pending.replace(event).is_none(), "{events:?}");
            assert_eq!(event["ok"], reloads > 0, "{event}");
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
    assert_eq!(reloads, 3, "{events:?}");
    assert!(pending.is_none(), "{events:?}");
    assert!(status.success(), "{status:?}");
}
