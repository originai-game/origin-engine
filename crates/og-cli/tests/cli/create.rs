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
        let snapshot = og()
            .args(["project", dir.join(command).to_str().unwrap(), "--json"])
            .output()
            .unwrap();
        assert!(snapshot.status.success(), "{}", text(&snapshot));
        let manifest: serde_json::Value = serde_json::from_slice(&snapshot.stdout).unwrap();
        assert_eq!(manifest["name"], "Installed Game");
        assert_eq!(manifest["input"]["create"]["view"], "side-2d");
        assert!(!dir.join(command).join("project.ogt").exists());
        assert!(dir.join(command).join("src/main.ts").is_file());
        // One public creation path exercises the connected shell, including
        // repeated New game restores and the first pause after the rewind.
        if command == "new" {
            let run = og()
                .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
                .env("OG_RENDERER", "none")
                .args(["run", dir.join(command).to_str().unwrap(), "--headless", "--json", "--commands",
                    "0.1:key_down:Enter;0.12:key_up:Enter;0.4:expect:ui:text:=New game;0.5:gamepad:a;0.52:gamepad:-;0.8:expect:ui:text:=Now entering;1.6:expect:ui:text:=Pause;1.7:intent:ShellAction:{\"action\":\"menu\"};1.8:intent:ShellAction:{\"action\":\"new\"};2.9:gamepad:start;2.93:gamepad:-;3.2:expect:ui:text:=Paused;3.3:gamepad:a;3.33:gamepad:-;3.6:expect:ui:text:=Pause;3.7:gamepad:start;3.73:gamepad:-;4.1:gamepad:down;4.13:gamepad:-;4.3:gamepad:a;4.33:gamepad:-;4.6:expect:ui:text:=Settings;4.7:gamepad:b;4.73:gamepad:-;5:expect:ui:text:=Paused;5.1:quit"])
                .output()
                .unwrap();
            assert!(run.status.success(), "{}", text(&run));
            assert!(text(&run).contains("\"passed\":7"), "{}", text(&run));
        }
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
    let snapshot = og()
        .args(["project", game.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(snapshot.status.success(), "{}", text(&snapshot));
    let manifest: serde_json::Value = serde_json::from_slice(&snapshot.stdout).unwrap();
    let package: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo().join("packages/engine/package.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["engine"], package["version"]);
    assert_eq!(manifest["id"], "neon-run");
    assert_eq!(manifest["startup"]["look"], "looks/neon-run.ogt");
    assert_eq!(manifest["input"]["create"]["look"], "neon-vector");
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
    assert!(!dir.join("other/og-project.json").exists(), "{t}");
}
