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
        assert!(text(&refused).contains("is not empty"), "{}", text(&refused));
    }
}
