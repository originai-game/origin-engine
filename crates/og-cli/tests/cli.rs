//! The `og` binary as a user runs it: exit codes and what it prints.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn og() -> Command {
    Command::new(env!("CARGO_BIN_EXE_og"))
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A fresh directory that holds a copy of the repository's IDL, as a root.
fn scratch_root() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "og-cli-test-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let idl = dir.join(og_api_path());
    std::fs::create_dir_all(idl.parent().unwrap()).unwrap();
    std::fs::copy(repo().join(og_api_path()), &idl).unwrap();
    dir
}

fn og_api_path() -> &'static str {
    "crates/og-api/api.ogt"
}

#[test]
fn version_names_og_and_the_engine() {
    let out = og().arg("--version").output().unwrap();
    assert!(out.status.success());
    let v = env!("CARGO_PKG_VERSION");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        format!("og {v} (origin-engine {v})")
    );
}

#[test]
fn api_check_passes_on_the_repository() {
    let out = og()
        .args(["api", "check"])
        .current_dir(repo())
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("match crates/og-api/api.ogt"));
}

#[test]
fn api_check_detects_a_hand_edit_and_a_stray_file_and_gen_repairs_them() {
    let root = scratch_root();
    let root_arg = root.to_str().unwrap();
    let r#gen = og()
        .args(["api", "gen", "--root", root_arg])
        .output()
        .unwrap();
    assert!(r#gen.status.success(), "{}", text(&r#gen));
    let clean = og()
        .args(["api", "check", "--root", root_arg])
        .output()
        .unwrap();
    assert!(clean.status.success(), "{}", text(&clean));

    // A hand edit in the middle of the declarations.
    let dts = root.join("packages/engine/types/index.d.ts");
    let original = std::fs::read_to_string(&dts).unwrap();
    std::fs::write(
        &dts,
        original.replacen("readonly tick: number;", "readonly tick: string;", 1),
    )
    .unwrap();
    // A file nobody generates in a generated directory.
    std::fs::write(root.join("packages/engine/schema/notes.json"), "{}").unwrap();

    let out = og()
        .args(["api", "check", "--root", root_arg])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    let report = text(&out);
    assert!(report.contains("2 file(s) differ"), "{report}");
    assert!(
        report.contains("packages/engine/types/index.d.ts: differs"),
        "{report}"
    );
    assert!(
        report.contains("packages/engine/schema/notes.json: not generated from api.ogt"),
        "{report}"
    );
    assert!(
        !report.contains("bindings.json"),
        "untouched files are not reported: {report}"
    );

    std::fs::remove_file(root.join("packages/engine/schema/notes.json")).unwrap();
    assert!(
        og().args(["api", "gen", "--root", root_arg])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        og().args(["api", "check", "--root", root_arg])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(std::fs::read_to_string(&dts).unwrap(), original);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn api_gen_refuses_an_invalid_idl_and_names_the_line() {
    let root = scratch_root();
    let idl = root.join(og_api_path());
    let text_in = std::fs::read_to_string(&idl).unwrap();
    std::fs::write(
        &idl,
        text_in.replacen("returns Result.Entity", "returns Result.Entty", 1),
    )
    .unwrap();
    let out = og()
        .args(["api", "gen", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out).contains("unknown type `Entty`"),
        "{}",
        text(&out)
    );
    assert!(
        !root.join("packages/engine/types/index.d.ts").exists(),
        "nothing is written for an invalid IDL"
    );
    let _ = std::fs::remove_dir_all(&root);
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

/// The pins as the repository's own files state them.
fn pins() -> (String, String) {
    let toolchain = std::fs::read_to_string(repo().join("rust-toolchain.toml")).unwrap();
    let rust = toolchain
        .lines()
        .find_map(|l| l.strip_prefix("channel = "))
        .unwrap()
        .trim_matches('"')
        .to_string();
    let lock = std::fs::read_to_string(repo().join("Cargo.lock")).unwrap();
    let at = lock.find("name = \"wasm-bindgen\"\n").unwrap();
    let bindgen = lock[at..]
        .lines()
        .nth(1)
        .unwrap()
        .trim_start_matches("version = ")
        .trim_matches('"')
        .to_string();
    (rust, bindgen)
}

// The doctor tests and their environment. `og doctor` reads the shell's
// `OG_TSRS` and `OG_ENGINE_PACKAGE`, and finds tsrs the way og check does
// (check::locate_tsrs): `OG_TSRS`, then `node_modules/@tsrs/<platform>/tsrs`
// beside the engine package (any ancestor of packages/engine, so a `bun
// install` in the package or the repository root), then PATH. The tests
// below take both variables away themselves and, when an installed tsrs
// package sits beside the engine, skip and say so: a doctor that finds it
// is right, and the test cannot take it away. HOME and the locale play no
// part. A failure here is the doctor's, never the environment's.

/// `node_modules/@tsrs/<platform>/tsrs` beside `engine` or one of its
/// ancestors: the tsrs og doctor finds before PATH.
fn tsrs_installed_beside(engine: &Path) -> Option<PathBuf> {
    engine.ancestors().find_map(|d| {
        std::fs::read_dir(d.join("node_modules/@tsrs"))
            .ok()?
            .flatten()
            .map(|e| e.path().join("tsrs"))
            .find(|p| p.is_file())
    })
}

/// The reason a doctor test that expects no tsrs cannot run here, if any.
fn skip_with_installed_tsrs() -> bool {
    match tsrs_installed_beside(&repo().join("packages/engine")) {
        Some(p) => {
            eprintln!(
                "skipped: {} exists (a bun install beside the engine package); og doctor finds it before PATH, so this test cannot show a missing tsrs. Remove that node_modules/@tsrs to run it.",
                p.display()
            );
            true
        }
        None => false,
    }
}

/// `og doctor` with the shell's tsrs and engine overrides taken away: the
/// tests say which they set.
fn doctor() -> Command {
    let mut c = og();
    c.arg("doctor")
        .env_remove("OG_TSRS")
        .env_remove("OG_ENGINE_PACKAGE");
    c
}

#[test]
#[cfg(unix)]
fn doctor_reports_the_pins_and_each_tool() {
    let (rust, _) = pins();
    let out = doctor().current_dir(repo()).output().unwrap();
    let t = text(&out);
    assert!(
        t.contains(&format!("pinned      rust {rust} (rust-toolchain.toml)")),
        "{t}"
    );
    for tool in ["rustc", "cargo", "wasm-bindgen", "bun", "tsrs", "node"] {
        assert!(
            t.lines().any(|l| l.split_whitespace().nth(1) == Some(tool)),
            "no line for {tool}:\n{t}"
        );
    }
    let install = std::fs::read_to_string(repo().join("scripts/install-tools.sh")).unwrap();
    let tsrs = install
        .lines()
        .find_map(|l| l.strip_prefix("TSRS_VERSION=\"${TSRS_VERSION:-"))
        .unwrap()
        .trim_end_matches("}\"");
    assert!(t.contains(&format!("tsrs {tsrs} (TSRS_VERSION)")), "{t}");
    // Inside the repository the version found is held to the pin.
    let dir = std::env::temp_dir().join(format!("og-cli-doctor-tsrs-{}", std::process::id()));
    let fake = fake_tsrs(&dir, "9.9.9", "", 0, true);
    let out = doctor()
        .current_dir(repo())
        .env("OG_TSRS", &fake)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let t = text(&out);
    assert!(
        t.contains(&format!(
            "differs  tsrs          0.0.0-9.9.9, pinned {tsrs}"
        )),
        "{t}"
    );
}

#[test]
fn doctor_names_a_fix_for_every_missing_tool() {
    if skip_with_installed_tsrs() {
        return;
    }
    let empty = std::env::temp_dir().join(format!("og-cli-empty-path-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let (rust, bindgen) = pins();
    let out = doctor()
        .current_dir(repo())
        .env("PATH", &empty)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir(&empty);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    let t = text(&out);
    let rustup = format!("rustup toolchain install {rust}");
    let cli = format!("cargo install wasm-bindgen-cli --version {bindgen} --locked");
    for (tool, fix) in [
        ("rustc", rustup.as_str()),
        ("cargo", rustup.as_str()),
        ("wasm-bindgen", cli.as_str()),
        ("bun", "https://bun.sh/install"),
        ("tsrs", "scripts/install-tools.sh"),
        ("node", "https://nodejs.org"),
    ] {
        if tool == "tsrs" && cfg!(windows) {
            assert!(t.contains("og check uses tsgo 7.0.2 through bun x"), "{t}");
            continue;
        }
        let at = t
            .lines()
            .position(|l| l.starts_with("missing") && l.split_whitespace().nth(1) == Some(tool))
            .unwrap_or_else(|| panic!("{tool} not reported missing:\n{t}"));
        let next = t.lines().nth(at + 1).unwrap_or_default();
        assert!(
            next.contains("fix:") && next.contains(fix),
            "{tool}: `{next}`"
        );
    }
    let problems = if cfg!(windows) { 5 } else { 6 };
    assert!(t.contains(&format!("{problems} problem(s)")), "{t}");
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

    // The dev build has every subsystem; a 3D project's publish keeps them.
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

/// An executor (outside the repository, bun but no node, no Rust): the
/// build toolchain is skipped and node is optional, so neither is a problem.
#[test]
#[cfg(unix)]
fn doctor_outside_the_repository_needs_only_bun_and_the_engine() {
    if skip_with_installed_tsrs() {
        return;
    }
    let Some(bun) = std::env::var_os("PATH").and_then(|p| {
        std::env::split_paths(&p)
            .map(|d| d.join("bun"))
            .find(|b| b.is_file())
    }) else {
        eprintln!(
            "skipped: no bun on PATH (https://bun.sh/install); the test symlinks the shell's bun into an otherwise empty PATH"
        );
        return;
    };
    let dir = scratch_root();
    std::fs::remove_dir_all(dir.join("crates")).unwrap();
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::os::unix::fs::symlink(&bun, bin.join("bun")).unwrap();
    let out = doctor()
        .current_dir(&dir)
        .env("PATH", &bin)
        .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
        .output()
        .unwrap();
    let t = text(&out);
    let _ = std::fs::remove_dir_all(&dir);
    let line = |tool: &str| {
        t.lines()
            .find(|l| l.split_whitespace().nth(1) == Some(tool))
            .unwrap_or_else(|| panic!("no line for {tool}:\n{t}"))
            .to_string()
    };
    for tool in ["rustc", "cargo", "wasm-bindgen"] {
        assert!(line(tool).starts_with("skipped"), "{t}");
    }
    assert!(line("node").starts_with("optional"), "{t}");
    assert!(line("bun").starts_with("ok"), "{t}");
    // No tsrs: the fix outside the repository is the project's bun install.
    assert!(line("tsrs").starts_with("missing"), "{t}");
    assert!(t.contains("fix: `bun install` in the project"), "{t}");
    // Outside the repository a tsrs found is not compared with any pin.
    let fake_dir = std::env::temp_dir().join(format!("og-cli-doctor-out-{}", std::process::id()));
    let fake = fake_tsrs(&fake_dir, "9.9.9", "", 0, true);
    let other = scratch_root();
    std::fs::remove_dir_all(other.join("crates")).unwrap();
    let out = doctor()
        .current_dir(&other)
        .env("PATH", bun.parent().unwrap())
        .env("OG_TSRS", &fake)
        .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&fake_dir);
    let _ = std::fs::remove_dir_all(&other);
    assert!(
        text(&out)
            .lines()
            .any(|l| l
                .starts_with("ok       tsrs          0.0.0-9.9.9 (TypeScript 9.9.9, from OG_TSRS")),
        "{}",
        text(&out)
    );
    // Whatever else fails (an unbuilt addon in a fresh checkout) is engine
    // lines only: every counted problem has a `missing`/`broken` line.
    let counted = t
        .lines()
        .filter(|l| l.starts_with("missing") || l.starts_with("broken") || l.starts_with("differs"))
        .count();
    if counted == 0 {
        assert_eq!(out.status.code(), Some(0), "{t}");
    } else {
        assert!(t.contains(&format!("{counted} problem(s)")), "{t}");
    }
}

/// `og api gen --wiki` writes the wiki pages alone, one per module plus the
/// index, and removes a page whose module the IDL no longer declares.
#[test]
fn api_gen_wiki_writes_only_the_wiki_and_removes_stale_pages() {
    let root = scratch_root();
    let wiki = root.join("packages/knowledge/wiki");
    std::fs::create_dir_all(&wiki).unwrap();
    std::fs::write(wiki.join("retired.md"), "an old module's page\n").unwrap();
    let out = og()
        .args(["api", "gen", "--wiki", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    assert!(!wiki.join("retired.md").exists(), "{}", text(&out));
    let idl = std::fs::read_to_string(root.join(og_api_path())).unwrap();
    let modules: Vec<&str> = idl
        .lines()
        .filter_map(|l| l.strip_prefix("module ")?.strip_suffix(" {"))
        .collect();
    let mut pages: Vec<String> = std::fs::read_dir(&wiki)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    pages.sort();
    let mut want: Vec<String> = modules.iter().map(|m| format!("{m}.md")).collect();
    want.push("index.md".into());
    want.sort();
    assert_eq!(pages, want);
    // Nothing outside the wiki was written.
    assert!(!root.join("packages/engine").exists());
    assert!(!root.join("crates/og-host-web").exists());
    let _ = std::fs::remove_dir_all(&root);
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

/// A copy of tests/fixtures/style-pack (three nine-patches, a corner and a
/// style file) in a fresh directory.
fn style_fixture() -> PathBuf {
    let dir = scratch_root().join("handmade");
    std::fs::create_dir_all(&dir).unwrap();
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/style-pack");
    for e in std::fs::read_dir(src).unwrap() {
        let p = e.unwrap().path();
        std::fs::copy(&p, dir.join(p.file_name().unwrap())).unwrap();
    }
    dir
}

/// `og style pack` packs a style drawn by hand: each nine-patch's art lands in
/// the atlas pixel for pixel, its insets are what lies outside its stretch
/// marks, the corner is packed as drawn, and the style file keeps its own
/// lines (the palette, a slice's tint and repeat words) while its atlas,
/// slices, soft rectangle and corner region move to the new atlas; og-ui
/// parses the result. A nine-patch with no stretch marks is refused by
/// name.
///
/// Ways it could fail: the border is packed with the art, the insets are
/// off by the border, images overlap or bleed into each other, the style
/// file loses lines it does not own, or a part's region points elsewhere.
#[test]
fn style_pack_packs_nine_patches_and_rewrites_the_style_file() {
    let dir = style_fixture();
    let out = og()
        .args(["style", "pack", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let atlas = image::open(dir.join("atlas.png")).unwrap().to_rgba8();
    let style = std::fs::read_to_string(dir.join("style.ogt")).unwrap();
    let line = |key: &str, part: &str| -> Vec<String> {
        style
            .lines()
            .map(|l| l.split_whitespace().map(String::from).collect::<Vec<_>>())
            .find(|w| {
                w.first().map(String::as_str) == Some(key)
                    && (part.is_empty() || w.get(1).map(String::as_str) == Some(part))
            })
            .unwrap_or_else(|| panic!("no {key} {part} line in\n{style}"))
    };
    let num = |w: &[String], i: usize| -> u32 { w[i].parse().unwrap() };
    // Each nine-patch: its region is the art, its insets the marks'.
    for (part, insets) in [("panel", [6, 5, 6, 5]), ("button", [4, 3, 4, 3])] {
        let w = line("slice", part);
        let (x, y, rw, rh) = (num(&w, 2), num(&w, 3), num(&w, 4), num(&w, 5));
        assert_eq!(
            [num(&w, 6), num(&w, 7), num(&w, 8), num(&w, 9)],
            insets,
            "{part}'s insets"
        );
        let src = image::open(dir.join(format!("{part}.9.png")))
            .unwrap()
            .to_rgba8();
        assert_eq!(
            (rw, rh),
            (src.width() - 2, src.height() - 2),
            "{part}'s region is its art"
        );
        for sy in 0..rh {
            for sx in 0..rw {
                assert_eq!(
                    atlas.get_pixel(x + sx, y + sy),
                    src.get_pixel(sx + 1, sy + 1),
                    "{part}'s art at {sx},{sy}"
                );
            }
        }
    }
    assert_eq!(
        line("slice", "button")[10..].to_vec(),
        vec!["tint".to_string(), "repeat".to_string()],
        "the button keeps its words"
    );
    let soft = line("soft", "");
    assert_eq!(
        (num(&soft, 3), num(&soft, 4), num(&soft, 5)),
        (12, 12, 5),
        "the soft rectangle and its inset"
    );
    let corner = line("corner", "");
    let src = image::open(dir.join("corner.png")).unwrap().to_rgba8();
    let (cx, cy) = (num(&corner, 1), num(&corner, 2));
    assert_eq!(
        atlas.get_pixel(cx + 3, cy + 2),
        src.get_pixel(3, 2),
        "the corner is packed as drawn"
    );
    assert_eq!(
        corner[5..].join(" "),
        "12 12 primary 2",
        "the corner keeps its size, ink and offset"
    );
    assert!(
        style.contains("primary \"#ffb000\"") && style.contains("corners panel"),
        "the style's own lines stay:\n{style}"
    );
    assert_eq!(
        line("atlas", "")[2..].to_vec(),
        vec![atlas.width().to_string(), atlas.height().to_string()],
        "the atlas line names the packed size"
    );
    let parsed =
        og_ui::parse_style(&style).unwrap_or_else(|e| panic!("og-ui reads it: {e}\n{style}"));
    assert_eq!(parsed.name, "handmade");
    // A nine-patch without marks is refused, by name.
    let bad = image::RgbaImage::from_pixel(10, 10, image::Rgba([200, 200, 200, 255]));
    bad.save(dir.join("card.9.png")).unwrap();
    let out = og()
        .args(["style", "pack", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out).contains("card.9.png: its top border marks no column"),
        "{}",
        text(&out)
    );
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

/// A stand-in tsrs in `dir`: `--version` names it `version`, a check
/// prints `out` and exits `status`; `lib` puts lib.es2022.d.ts beside it.
#[cfg(unix)]
fn fake_tsrs(dir: &Path, version: &str, out: &str, status: i32, lib: bool) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(dir).unwrap();
    let bin = dir.join("tsrs");
    std::fs::write(
        &bin,
        format!(
            "#!/bin/sh\nPATH=/usr/bin:/bin\nif [ \"$1\" = --version ]; then echo 'Version {version} (tsrs 0.0.0-{version}, test)'; exit 0; fi\necho \"$@\" > \"$(dirname \"$0\")/args.txt\"\ncat \"$(dirname \"$0\")/out.txt\"\nexit {status}\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("out.txt"), out).unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
    if lib {
        std::fs::write(dir.join("lib.es2022.d.ts"), "").unwrap();
    }
    bin
}

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
    for (filter, suites) in [("chosen", 2), ("tests/nested/", 1), ("chosen.test.ts", 1)] {
        let out = og()
            .args(["test", "--no-check", "--json", "--filter", filter])
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
    ] {
        let out = og()
            .arg("test")
            .arg(&game)
            .arg("--no-check")
            .args(args)
            .env("OG_ENGINE_PACKAGE", &engine)
            .output()
            .unwrap();
        assert!(!out.status.success(), "{}", text(&out));
        assert!(text(&out).contains("--filter"), "{}", text(&out));
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
    let (a, b) = (next(), next());
    let status = child.wait().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let (reload, check) = if a["event"] == "reload" {
        (a, b)
    } else {
        (b, a)
    };
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

#[test]
fn compare_prints_the_iou_and_delta_e_and_writes_the_sheet() {
    let dir = scratch_root();
    let square = |x0: u32, w: u32, h: u32| {
        image::RgbaImage::from_fn(w, h, |x, y| {
            let inside = (x0..x0 + 40).contains(&x) && (20..60).contains(&y);
            image::Rgba(if inside {
                [255, 0, 0, 255]
            } else {
                [0, 0, 0, 255]
            })
        })
    };
    square(20, 100, 80).save(dir.join("ref.png")).unwrap();
    square(30, 100, 80).save(dir.join("shot.png")).unwrap();
    let out = og()
        .args(["compare", "ref.png", "shot.png", "--json"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["iou"], serde_json::json!(0.6), "{v}");
    assert_eq!(v["deltaE00"]["mean"], serde_json::json!(0.0));
    let sheet = image::open(dir.join("shot.sheet.png")).unwrap();
    assert_eq!(
        (sheet.width(), sheet.height()),
        (8 + 3 * 108, 8 + 80 + 22 + 8)
    );

    // Another shape is refused, naming the size to render at.
    square(30, 160, 80).save(dir.join("wide.png")).unwrap();
    let out = og()
        .args(["compare", "ref.png", "wide.png"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out).contains("og run --size 100x80"),
        "{}",
        text(&out)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn migrate_three_rewrites_a_file_and_lists_what_is_not_mechanical() {
    let dir = scratch_root();
    std::fs::write(
        dir.join("scene.ts"),
        "import * as THREE from \"three\";\nconst scene = new THREE.Scene();\nconst box = new THREE.Mesh(new THREE.BoxGeometry(1, 2, 3), new THREE.MeshStandardMaterial({ color: 0x336699 }));\nscene.add(box);\nconst clock = new THREE.Clock();\n",
    )
    .unwrap();
    let out = og()
        .args([
            "migrate",
            "three",
            "scene.ts",
            "--out",
            "scene.og.ts",
            "--json",
        ])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["event"], "migrate", "{v}");
    assert_eq!(
        (
            v["rewritten"].as_u64(),
            v["removed"].as_u64(),
            v["manual"].as_u64()
        ),
        (Some(1), Some(3), Some(1)),
        "{v}"
    );
    let manual = v["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["kind"] == "manual")
        .unwrap();
    assert_eq!(
        (manual["line"].as_u64(), manual["section"].as_u64()),
        (Some(5), Some(8)),
        "{v}"
    );
    let written = std::fs::read_to_string(dir.join("scene.og.ts")).unwrap();
    assert!(
        written.starts_with("import { og, color } from \"@origingame/engine\";\n"),
        "{written}"
    );
    assert!(
        written.contains("world.insert(box, og.Mesh, { geometry: must(og.render.boxGeometry(world, { width: 1, height: 2, depth: 3 })), material: must(og.render.standardMaterial(world, { color: color.fromHex(0x336699) })) });"),
        "{written}"
    );
    assert!(written.contains("// og migrate: Clock:"), "{written}");
    assert_eq!(
        std::fs::read_to_string(dir.join("scene.ts"))
            .unwrap()
            .lines()
            .count(),
        5,
        "--out leaves the source alone"
    );

    // A file that imports nothing from three is refused; an unknown kind is a usage error.
    std::fs::write(dir.join("plain.ts"), "export const a = 1;\n").unwrap();
    let out = og()
        .args(["migrate", "three", "plain.ts"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(
        text(&out).contains("imports nothing from three"),
        "{}",
        text(&out)
    );
    let out = og()
        .args(["migrate", "babylon", "scene.ts"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    let _ = std::fs::remove_dir_all(&dir);
}
