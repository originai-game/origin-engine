use super::*;

#[test]
fn create_and_new_scaffold_from_an_installed_distribution() {
    let dir = TempDir::new("og-create-installed");
    let engine = dir.join("dist/engine");
    std::fs::create_dir_all(&engine).unwrap();
    // Use the real creator, but resolve it next to the installed engine,
    // never relative to the caller's working directory.
    #[cfg(unix)]
    std::os::unix::fs::symlink(repo().join("packages/create"), dir.join("dist/create")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(repo().join("packages/create"), dir.join("dist/create"))
        .unwrap();
    for command in ["create", "new"] {
        let out = og()
            .current_dir(&dir)
            .env("OG_ENGINE_PACKAGE", &engine)
            .args([
                command,
                command,
                "--name",
                "Installed Game",
                "--view",
                "side-2d",
                "--look",
                "pixel",
                "--no-prompt",
            ])
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
        let manifest = std::fs::read_to_string(dir.join(command).join("project.ogt")).unwrap();
        assert!(manifest.contains("name \"Installed Game\""), "{manifest}");
        assert!(manifest.contains("view \"side-2d\""), "{manifest}");
        assert!(dir.join(command).join("src/main.ts").is_file());
        let refused = og()
            .current_dir(&dir)
            .env("OG_ENGINE_PACKAGE", &engine)
            .args([command, command, "--no-prompt"])
            .output()
            .unwrap();
        assert!(!refused.status.success(), "{}", text(&refused));
        assert!(
            text(&refused).contains("is not empty"),
            "{}",
            text(&refused)
        );
    }
}

/// The creation a tool runs: no terminal and no --no-prompt, the chosen
/// look and premise, --force over a nonempty directory (the start guide
/// §1). An unknown look is refused, naming the ones
/// there are, and writes nothing.
#[test]
fn create_without_a_terminal_writes_the_chosen_look() {
    let dir = TempDir::new("og-create-look");
    let engine = dir.join("dist/engine");
    std::fs::create_dir_all(&engine).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(repo().join("packages/create"), dir.join("dist/create")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(repo().join("packages/create"), dir.join("dist/create"))
        .unwrap();
    let new = |args: &[&str]| {
        og().current_dir(&dir)
            .env("OG_ENGINE_PACKAGE", &engine)
            .stdin(std::process::Stdio::null())
            .arg("new")
            .args(args)
            .output()
            .unwrap()
    };
    std::fs::create_dir_all(dir.join("Game Dir")).unwrap();
    std::fs::write(dir.join("Game Dir/notes.txt"), "kept").unwrap();
    let out = new(&[
        "Game Dir",
        "--id",
        "neon-run",
        "--look",
        "neon-vector",
        "--premise",
        "a cozy fishing village",
        "--force",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    let game = dir.join("Game Dir");
    let manifest = std::fs::read_to_string(game.join("project.ogt")).unwrap();
    let package: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo().join("packages/engine/package.json")).unwrap(),
    )
    .unwrap();
    for line in [
        format!("engine \"{}\"", package["version"].as_str().unwrap()),
        "id \"neon-run\"".into(),
        "look \"looks/neon-run.ogt\"".into(),
        "look \"neon-vector\"".into(),
    ] {
        assert!(manifest.contains(&line), "{line}: {manifest}");
    }
    let look = std::fs::read_to_string(game.join("assets/looks/neon-run.ogt")).unwrap();
    assert!(look.contains("extends \"neon-vector\""), "{look}");
    assert_eq!(
        std::fs::read_to_string(game.join("notes.txt")).unwrap(),
        "kept"
    );

    let refused = new(&["other", "--look", "no-such-look", "--no-prompt"]);
    assert_eq!(refused.status.code(), Some(1), "{}", text(&refused));
    let t = text(&refused);
    assert!(
        t.contains("no look preset 'no-such-look'") && t.contains("neon-vector"),
        "{t}"
    );
    assert!(!dir.join("other/project.ogt").exists(), "{t}");
}

/// A game `og new` makes outside the checkout whose `og` runs it (the
/// checkout's og, no OG_ENGINE_PACKAGE) type-checks clean: og links the
/// checkout's engine package into the game's node_modules, so the
/// template's `types` entry (`@origingame/engine/bun-test`) resolves
/// instead of failing with TS2688.
#[cfg(unix)]
#[test]
fn a_new_game_outside_the_checkout_checks_clean() {
    assert!(
        std::env::var_os("PATH")
            .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join("tsrs").is_file())),
        "this test needs tsrs on PATH (scripts/install-tools.sh)"
    );
    // Outside the checkout: its node_modules link must not resolve the engine.
    let mut base = std::env::temp_dir();
    if base.starts_with(repo()) {
        base = PathBuf::from("/tmp");
    }
    let dir = base.join(format!("og-new-check-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    struct Remove(PathBuf);
    impl Drop for Remove {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _remove = Remove(dir.clone());
    let og_here = |args: &[&str]| {
        og().current_dir(&dir)
            .env_remove("OG_ENGINE_PACKAGE")
            .env("OG_SESSION", "0")
            .args(args)
            .output()
            .unwrap()
    };
    let out = og_here(&[
        "new",
        ".",
        "--id",
        "fresh",
        "--premise",
        "a test",
        "--no-prompt",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    let out = og_here(&["check", "."]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(!text(&out).contains("TS2688"), "{}", text(&out));
}
