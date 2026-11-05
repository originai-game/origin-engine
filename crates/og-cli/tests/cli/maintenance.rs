use super::*;

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
        assert_eq!(file.version, 2);
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

/// The lit-mesh blend break lists the emitters whose look it changes (lit
/// meshes with no blend line), at their render line, and no other: not an
/// unlit mesh, not one that says its blend, not a lit billboard.
#[test]
fn upgrade_lists_lit_mesh_emitters_without_a_blend_line() {
    let dir = scratch_root();
    let fx = dir.join("assets/fx");
    let package = dir.join("engine");
    std::fs::create_dir_all(&fx).unwrap();
    std::fs::create_dir_all(&package).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    let manifest =
        "type project\nversion 1\n---\nname \"blend\"\nengine \"0.7.7\"\nentry \"src/main.ts\"\n";
    std::fs::write(dir.join("project.ogt"), manifest).unwrap();
    let log: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo().join("packages/engine/changelog.json")).unwrap(),
    )
    .unwrap();
    let entry = log["unreleased"]["breaking"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["codemod"]["kind"] == "particles-lit-mesh-blend")
        .unwrap()
        .clone();
    std::fs::write(package.join("package.json"), r#"{"version":"0.8.0"}"#).unwrap();
    std::fs::write(
        package.join("changelog.json"),
        serde_json::to_vec(&serde_json::json!({
            "versions": [{"version":"0.8.0", "simulation":"unchanged", "breaking":[entry]}]
        }))
        .unwrap(),
    )
    .unwrap();
    let head = "type particles\nversion 1\n---\n";
    for (name, body) in [
        (
            "chips",
            "rate 0\nrender mesh tetrahedron\nlit true  # shaded\n",
        ),
        ("unlit", "render mesh box\n"),
        ("said", "lit true\nrender mesh box\nblend additive\n"),
        ("glow", "lit true\n"),
    ] {
        std::fs::write(fx.join(format!("{name}.ogt")), format!("{head}{body}")).unwrap();
    }
    let out = og()
        .arg("upgrade")
        .arg(&dir)
        .arg("--json")
        .arg("--dry-run")
        .env("OG_ENGINE_PACKAGE", &package)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let listed: Vec<(String, u64)> = v["manual"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["file"].as_str().unwrap().to_string(),
                m["line"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(listed, [("assets/fx/chips.ogt".to_string(), 5)], "{v}");
    std::fs::remove_dir_all(dir).unwrap();
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

/// `og <command> … --help` answers that command's usage on stdout and
/// succeeds, as `og --help` does: an agent's shell reported each command's
/// help as a failure (exit 2, usage on stderr) and retried (M7 run 12).
#[test]
fn every_command_answers_help_with_its_usage() {
    for args in [
        &["style", "--help"][..],
        &["test", "--help"],
        &["run", "-h"],
        &["api", "show", "--help"],
        &["bake", "probes", "--help"],
    ] {
        let out = og().args(args).output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{args:?}: {out:?}");
        assert!(out.stderr.is_empty(), "{args:?}: {out:?}");
        let entry = format!("  {} ", args[..args.len() - 1].join(" "));
        assert!(
            stdout.contains(&entry),
            "{args:?} shows its entry: {stdout}"
        );
    }
    // A subcommand narrows the answer to its entry.
    let out = og().args(["bake", "probes", "--help"]).output().unwrap();
    assert!(!String::from_utf8_lossy(&out.stdout).contains("bake lightmaps"));
}

#[test]
fn api_check_passes_on_the_repository() {
    let out = og()
        .args(["api", "check"])
        .current_dir(repo())
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("match crates/og-api/api"));
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
    let idl = root.join(og_api_path()).join("world.ogt");
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
    let line = text_in
        .lines()
        .position(|l| l.trim() == "fn spawn {")
        .unwrap()
        + 1;
    assert!(
        text(&out).contains(&format!("world.ogt:{line}:")),
        "{}",
        text(&out)
    );
    std::fs::write(
        &idl,
        text_in.replacen("skill og-core", "skill \"og-core", 1),
    )
    .unwrap();
    let out = og()
        .args(["api", "gen", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out).contains("world.ogt:2: unterminated string"),
        "{}",
        text(&out)
    );
    assert!(
        !root.join("packages/engine/types/index.d.ts").exists(),
        "nothing is written for an invalid IDL"
    );
    let _ = std::fs::remove_dir_all(&root);
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
    // No tsrs: the fix outside the repository is the launcher's dist.
    assert!(line("tsrs").starts_with("missing"), "{t}");
    assert!(t.contains("fix: run og through its launcher"), "{t}");
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
    let idl = og_idl::read(&root.join(og_api_path())).unwrap();
    let mut pages: Vec<String> = std::fs::read_dir(&wiki)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    pages.sort();
    let mut want: Vec<String> = idl
        .modules
        .iter()
        .map(|m| format!("{}.md", m.name))
        .collect();
    want.push("index.md".into());
    want.sort();
    assert_eq!(pages, want);
    // Nothing outside the wiki was written.
    assert!(!root.join("packages/engine").exists());
    assert!(!root.join("crates/og-host-web").exists());
    let _ = std::fs::remove_dir_all(&root);
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

/// A project's own budgets are numbers `og doctor --project` reports: an asset
/// over one is a warning and the exit status stays 0.
#[test]
fn doctor_project_reports_an_asset_over_a_budget_without_failing() {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("assets/scenes")).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\nbudgets {\n  sceneEntities 1\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("assets/scenes/main.ogt"),
        "type scene\nversion 1\n---\nname \"Main\"\nentity \"a\" {\n  Transform {\n    position 1 2 3\n  }\n}\nentity \"b\" {\n  Transform {\n    position 4 5 6\n  }\n}\n",
    )
    .unwrap();
    let out = og().args(["cook", "--report"]).arg(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let out = og()
        .args(["doctor", "--project"])
        .arg(&dir)
        .arg("--json")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let over: Vec<&serde_json::Value> = doc["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["code"] == "over-budget")
        .collect();
    assert_eq!(over.len(), 1, "{doc}");
    assert_eq!(over[0]["severity"], "warning");
    assert_eq!(over[0]["budget"], "sceneEntities");
}
