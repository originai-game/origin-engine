//! The authoring tool's public contract: no image service call, project
//! look in its requests, deterministic maps, and verified cache reuse.
use super::*;

const LIBRARY: &str = "type materials\nversion 1\n---\nmaterial \"brick\" {\n  label wall\n  tile 2 1\n  brief \"red stock brick\"\n  map \"textures/brick.png\"\n  relief -0.02\n  normalMap \"textures/brick-n.png\"\n  roughnessRange 0.4 0.9\n  metalnessMap \"textures/brick-mr.png\"\n  occlusion 0.5 2\n  aoMap \"textures/brick-ao.png\"\n}\n";

fn project() -> TempDir {
    let dir = TempDir::new("og-materials");
    std::fs::create_dir_all(dir.join("assets/materials")).unwrap();
    std::fs::create_dir_all(dir.join("assets/textures")).unwrap();
    std::fs::create_dir_all(dir.join("assets/looks")).unwrap();
    std::fs::write(
        dir.join("assets/looks/game.ogt"),
        "type look\nversion 1\n---\nextends \"faceted-low-poly\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("assets/materials/town.ogt"), LIBRARY).unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nlook \"looks/game.ogt\"\n",
    )
    .unwrap();
    dir
}

fn materials(dir: &Path, command: &str) -> Output {
    og().current_dir(dir)
        .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
        .args(["materials", command, "assets/materials/town.ogt", "--json"])
        .output()
        .unwrap()
}

#[test]
fn materials_brief_uses_the_project_look_without_needing_source_images() {
    let dir = project();
    let out = materials(&dir, "brief");
    assert!(out.status.success(), "{}", text(&out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let r = &v["requests"][0];
    assert_eq!(r["source"], "textures/brick.source.png");
    assert_eq!(r["brief"]["kind"], "material");
    assert_eq!(r["brief"]["image"]["width"], 1024);
    assert_eq!(r["brief"]["image"]["height"], 512);
    assert!(
        r["brief"]["prompt"]
            .as_str()
            .unwrap()
            .contains("red stock brick")
    );
    let again = materials(&dir, "brief");
    assert_eq!(out.stdout, again.stdout);
    let project = dir.join("assets/looks/game.ogt");
    let original = std::fs::read_to_string(&project).unwrap();
    std::fs::write(&project, original.replace("faceted-low-poly", "toy-3d")).unwrap();
    let different_look = materials(&dir, "brief");
    assert!(different_look.status.success(), "{}", text(&different_look));
    let different_look: serde_json::Value = serde_json::from_slice(&different_look.stdout).unwrap();
    assert_ne!(
        r["brief"]["prompt"],
        different_look["requests"][0]["brief"]["prompt"]
    );
    std::fs::write(project, original).unwrap();
    let changed = LIBRARY.replace("red stock brick", "pale limestone");
    std::fs::write(dir.join("assets/materials/town.ogt"), changed).unwrap();
    let next = materials(&dir, "brief");
    let next: serde_json::Value = serde_json::from_slice(&next.stdout).unwrap();
    assert_ne!(r["requestHash"], next["requests"][0]["requestHash"]);
    assert!(!dir.join("assets/textures/brick.png").exists());
}

#[test]
fn materials_make_recovers_modified_maps_and_invalidates_changed_options() {
    let dir = project();
    let missing = materials(&dir, "make");
    assert_eq!(missing.status.code(), Some(1), "{}", text(&missing));
    let v: serde_json::Value = serde_json::from_slice(&missing.stdout).unwrap();
    assert_eq!(v["missing"][0], "textures/brick.source.png");
    let mut image = image::RgbImage::new(32, 16);
    for (x, y, p) in image.enumerate_pixels_mut() {
        *p = image::Rgb([(30 + x * 5) as u8, (60 + y * 7) as u8, 90]);
    }
    image
        .save(dir.join("assets/textures/brick.source.png"))
        .unwrap();
    let run = || {
        let out = materials(&dir, "make");
        assert!(out.status.success(), "{}", text(&out));
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()
    };
    let first = run();
    assert_eq!(first["made"][0]["written"].as_array().unwrap().len(), 4);
    let normal = dir.join("assets/textures/brick-n.png");
    let original = std::fs::read(&normal).unwrap();
    assert_eq!(run()["made"][0]["cached"], true);
    std::fs::write(&normal, "damaged").unwrap();
    let repaired = run();
    assert_eq!(repaired["made"][0]["cached"], false);
    assert_eq!(
        repaired["made"][0]["written"],
        serde_json::json!(["textures/brick-n.png"])
    );
    assert_eq!(std::fs::read(&normal).unwrap(), original);
    std::fs::write(
        dir.join("assets/materials/town.ogt"),
        LIBRARY.replace("relief -0.02", "relief 0.08"),
    )
    .unwrap();
    assert_eq!(run()["made"][0]["cached"], false);
    assert_ne!(std::fs::read(&normal).unwrap(), original);
    assert_eq!(run()["made"][0]["cached"], true);
    image.put_pixel(8, 8, image::Rgb([255, 0, 0]));
    image
        .save(dir.join("assets/textures/brick.source.png"))
        .unwrap();
    assert_eq!(run()["made"][0]["cached"], false);
    #[cfg(unix)]
    {
        let outside = dir.join("untouched.txt");
        std::fs::write(&outside, "keep").unwrap();
        std::fs::remove_file(&normal).unwrap();
        std::os::unix::fs::symlink(&outside, &normal).unwrap();
        let out = materials(&dir, "make");
        assert_eq!(out.status.code(), Some(2), "{}", text(&out));
        assert!(text(&out).contains("symlink"));
        assert_eq!(std::fs::read_to_string(outside).unwrap(), "keep");
    }
}
