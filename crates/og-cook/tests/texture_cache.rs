//! Shared results must behave like an uncached cook, across fresh projects.
use og_cook::cook::{CookOptions, Report, cook_project_with_texture_cache};
use std::path::{Path, PathBuf};

fn project(root: &Path, value: u8, gltf: bool, normal: bool) {
    std::fs::create_dir_all(root.join("assets")).unwrap();
    image::RgbaImage::from_pixel(8, 8, image::Rgba([value, 90, 140, 255]))
        .save(root.join("assets/texture.png"))
        .unwrap();
    if gltf {
        let material = if normal {
            r#"{"normalTexture":{"index":0}}"#
        } else {
            r#"{"pbrMetallicRoughness":{"baseColorTexture":{"index":0}}}"#
        };
        std::fs::write(
            root.join("assets/model.gltf"),
            format!(r#"{{"asset":{{"version":"2.0"}},"images":[{{"uri":"texture.png"}}],"textures":[{{"source":0}}],"materials":[{material}]}}"#),
        )
        .unwrap();
    } else if normal {
        std::fs::write(
            root.join("assets/texture.png.ogt"),
            "type import\nversion 1\n---\ntexture {\n  kind normal\n}\n",
        )
        .unwrap();
    }
}

fn cook(root: &Path, cache: Option<&Path>, options: &CookOptions) -> (Report, Vec<u8>) {
    let r = cook_project_with_texture_cache(root, options, cache).unwrap();
    assert!(r.ok(), "{:?}", r.errors());
    let output = r
        .assets
        .iter()
        .flat_map(|a| &a.outputs)
        .find(|o| o.id == "texture.png" || o.id.starts_with("_gltf/"))
        .unwrap();
    let bytes = std::fs::read(root.join("cooked").join(&output.file)).unwrap();
    (r, bytes)
}

fn hit(r: &Report) -> bool {
    r.assets
        .iter()
        .flat_map(|a| &a.notes)
        .any(|n| n.contains("shared texture cache hit"))
}

#[test]
fn asset_move_preserves_sources_and_rewrites_parsed_references_only() {
    use og_cook::cook::asset_move::move_asset;
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("asset-move");
    let _ = std::fs::remove_dir_all(&root);
    project(&root, 30, false, true);
    std::fs::create_dir_all(root.join("src")).unwrap();
    let code = r#"// og.load.texture(w, 'texture.png');
const re = /texture.png/;
const notAnAsset = 'texture.png.backup';
og.load.texture(w, 'texture.\u0070ng');
const fields = { texture: `assets/texture.png`, normal: 'texture.png#normal' };
og.load.texture(w, `tex${name}.png`);
"#;
    std::fs::write(root.join("src/main.ts"), code).unwrap();
    let scene = "type scene\nversion 1\n---\n# keep this comment: \"texture.png\"\nresource Assets {\n  texture \"assets/texture.png\"\n}\n";
    std::fs::write(root.join("assets/main.ogt"), scene).unwrap();
    let source = std::fs::read(root.join("assets/texture.png")).unwrap();
    let settings = std::fs::read(root.join("assets/texture.png.ogt")).unwrap();
    let options = CookOptions {
        schemas: vec![og_core::reflect::Schema {
            name: "Assets".into(),
            version: 1,
            doc: String::new(),
            fields: vec![og_core::reflect::FieldSchema::new(
                "texture",
                og_core::reflect::FieldType::string(64).unwrap(),
            )],
        }],
        ..CookOptions::default()
    };
    let (_, expected) = cook(&root, None, &options);

    assert!(move_asset(&root, "../outside.png", "new.png").is_err());
    std::fs::write(root.join("assets/new.png.ogt"), &settings).unwrap();
    assert!(move_asset(&root, "texture.png", "new.png").is_err());
    std::fs::remove_file(root.join("assets/new.png.ogt")).unwrap();
    std::fs::write(root.join("src/broken.ts"), "const = ;").unwrap();
    assert!(move_asset(&root, "texture.png", "art/new.png").is_err());
    assert_eq!(
        std::fs::read(root.join("assets/texture.png")).unwrap(),
        source
    );
    assert_eq!(
        std::fs::read_to_string(root.join("src/main.ts")).unwrap(),
        code
    );
    assert_eq!(
        std::fs::read_to_string(root.join("assets/main.ogt")).unwrap(),
        scene
    );
    std::fs::remove_file(root.join("src/broken.ts")).unwrap();

    let report = move_asset(&root, "assets/texture.png", "art/new.png").unwrap();
    assert_eq!(report.rewritten, ["assets/main.ogt", "src/main.ts"]);
    assert_eq!(report.warnings.len(), 1);
    assert!(report.warnings[0].contains("computed load"));
    assert!(!root.join("assets/texture.png").exists());
    assert!(!root.join("assets/texture.png.ogt").exists());
    assert!(!root.join(".og/asset-move").exists());
    assert_eq!(
        std::fs::read(root.join("assets/art/new.png")).unwrap(),
        source
    );
    assert_eq!(
        std::fs::read(root.join("assets/art/new.png.ogt")).unwrap(),
        settings
    );
    assert_eq!(
        std::fs::read_to_string(root.join("assets/main.ogt")).unwrap(),
        scene.replace(
            "texture \"assets/texture.png\"",
            "texture \"assets/art/new.png\""
        )
    );
    let changed = std::fs::read_to_string(root.join("src/main.ts")).unwrap();
    assert!(changed.contains("// og.load.texture(w, 'texture.png');"));
    assert!(changed.contains("/texture.png/"));
    assert!(changed.contains("'texture.png.backup'"));
    assert!(changed.contains("og.load.texture(w, \"art/new.png\")"));
    assert!(changed.contains("texture: \"assets/art/new.png\""));
    assert!(changed.contains("normal: \"art/new.png#normal\""));
    assert!(changed.contains("`tex${name}.png`"));
    let report = cook_project_with_texture_cache(&root, &options, None).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    assert!(bundle.output("texture.png").is_err());
    assert_eq!(
        bundle.read("art/new.png").unwrap().as_ref(),
        expected.as_slice()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn asset_index_tracks_shared_model_textures_and_typescript_without_changing_blobs() {
    use og_cook::cook::assets::Index;
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("asset-index");
    let _ = std::fs::remove_dir_all(&root);
    project(&root, 30, true, false);
    std::fs::copy(
        root.join("assets/model.gltf"),
        root.join("assets/used.gltf"),
    )
    .unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\npublish {\n  keep \"descending.*\"\n}\n",
    )
    .unwrap();
    for alias in ["a-copy.png", "z-used.png"] {
        std::fs::copy(
            root.join("assets/texture.png"),
            root.join("assets").join(alias),
        )
        .unwrap();
    }
    std::fs::create_dir_all(root.join("assets/dynamic")).unwrap();
    std::fs::copy(
        root.join("assets/texture.png"),
        root.join("assets/dynamic/tile.png"),
    )
    .unwrap();
    image::RgbaImage::from_fn(9, 8, |x, _| {
        let v = (255 - x * 28) as u8;
        image::Rgba([v, v, v, 255])
    })
    .save(root.join("assets/descending.png"))
    .unwrap();
    std::fs::write(
        root.join("assets/coin.ogt"),
        og_cook::cook::sfx::preset_ogt("coin", Default::default()),
    )
    .unwrap();
    std::fs::write(
        root.join("src/game.ts"),
        r#"
// og.load.gltf(w, "model.gltf");
const re = /og.load.gltf(w, "model.gltf")/;
og.load.defaultBundle(w);
og.load.status(w, handle);
og.load.gltf(w, "used.\u0067ltf");
og.load.texture(w, `dynamic/${name}.png`);
const fields = { sound: 'coin.ogt', texture: 'z-used.png' };
"#,
    )
    .unwrap();
    let options = CookOptions::default();
    cook(&root, None, &options);
    let index = Index::read(&root).unwrap();
    let reachable = index.reachable();
    assert!(reachable.contains("used.gltf"));
    assert!(
        !reachable.contains("model.gltf"),
        "a shared texture must not retain its first model"
    );
    assert!(reachable.contains("dynamic/tile.png"));
    assert_eq!(
        index
            .assets
            .iter()
            .find(|a| a.id == "descending.png")
            .unwrap()
            .perceptual,
        Some(u64::MAX)
    );
    assert!(
        index
            .assets
            .iter()
            .filter(|a| a.kind == "texture")
            .all(|a| a.perceptual.is_some())
    );
    assert!(
        index
            .assets
            .iter()
            .find(|a| a.id == "coin.ogt")
            .unwrap()
            .perceptual
            .is_some()
    );
    let shared = index
        .assets
        .iter()
        .find(|a| a.id.starts_with("_gltf/"))
        .unwrap();
    assert!(reachable.contains(&shared.id));
    assert!(index.referenced_by(&shared.id).contains("used.gltf"));
    assert_eq!(index.computed.len(), 1);
    assert_eq!(index.computed[0].prefix, "dynamic/");
    assert!(
        reachable.contains("descending.png"),
        "publish.keep is a root"
    );
    let plan: og_cook::cook::assets::PublishPlan =
        serde_json::from_slice(&std::fs::read(root.join(".og/publish-assets.json")).unwrap())
            .unwrap();
    assert!(plan.omitted.iter().any(|o| o.id == "model.gltf"));
    assert!(plan.omitted.iter().any(|o| o.id == "a-copy.png"));
    assert!(
        !plan.removed_files.contains("a-copy.png.ogb"),
        "the retained alias still reads the canonical file"
    );
    let published = og_cook::runtime::Manifest::parse(&plan.manifest).unwrap();
    assert!(
        published
            .outputs()
            .any(|o| o.id == "z-used.png" && o.file == "a-copy.png.ogb")
    );
    assert!(!published.outputs().any(|o| o.id == "model.gltf"));
    assert!(root.join("assets/model.gltf").is_file());
    let sidecar = std::fs::read(root.join("cooked/.assets.ogt")).unwrap();
    let manifest = std::fs::read(root.join("cooked/.cook.ogt")).unwrap();
    let (report, _) = cook(&root, None, &options);
    assert!(report.cooked_outputs().is_empty());
    assert_eq!(
        std::fs::read(root.join("cooked/.assets.ogt")).unwrap(),
        sidecar
    );
    std::fs::write(root.join("src/game.ts"), "og.load.gltf(w, name);").unwrap();
    let (report, _) = cook(&root, None, &options);
    assert!(report.cooked_outputs().is_empty());
    assert_eq!(
        std::fs::read(root.join("cooked/.cook.ogt")).unwrap(),
        manifest
    );
    let index = Index::read(&root).unwrap();
    assert_eq!(index.reachable().len(), index.assets.len());
    assert_eq!(index.computed[0].prefix, "");
    let manifest =
        og_cook::runtime::Manifest::parse(&String::from_utf8(manifest).unwrap()).unwrap();
    assert!(index.publish_plan(&manifest).omitted.is_empty());
    std::fs::write(
        root.join("src/game.ts"),
        "const load = og.load.gltf; load(w, name);",
    )
    .unwrap();
    cook(&root, None, &options);
    let index = Index::read(&root).unwrap();
    assert!(
        index.publish_plan(&manifest).omitted.is_empty(),
        "an escaped loader must retain all assets"
    );
    assert_eq!(index.computed.len(), 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn identical_outputs_share_a_file_and_survive_canonical_changes() {
    use og_cook::runtime::Bundle;
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("shared-cooked-files");
    let _ = std::fs::remove_dir_all(&root);
    project(&root, 30, false, false);
    std::fs::copy(root.join("assets/texture.png"), root.join("assets/a.png")).unwrap();
    let options = CookOptions::default();
    let (_, expected) = cook(&root, None, &options);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(bundle.output("texture.png").unwrap().file, "a.png.ogb");
    assert_eq!(
        bundle.asset("assets/texture.png").unwrap().source,
        "texture.png"
    );
    assert!(!root.join("cooked/texture.png.ogb").exists());
    let manifest = std::fs::read(root.join("cooked/.cook.ogt")).unwrap();
    let (r, bytes) = cook(&root, None, &options);
    assert!(r.cooked_outputs().is_empty());
    assert_eq!(bytes, expected);
    assert_eq!(
        std::fs::read(root.join("cooked/.cook.ogt")).unwrap(),
        manifest
    );

    // A stat-index hit must not hide a modified cooked file.
    let file = root.join("cooked/a.png.ogb");
    let mut broken = expected.clone();
    *broken.last_mut().unwrap() ^= 1;
    std::fs::write(&file, broken).unwrap();
    assert_eq!(cook(&root, None, &options).1, expected);

    // Changing the shared file's owner must not overwrite the surviving alias,
    // including when an editor restores the source's previous modification time.
    let modified = std::fs::metadata(root.join("assets/a.png"))
        .unwrap()
        .modified()
        .unwrap();
    image::RgbaImage::from_pixel(8, 8, image::Rgba([200, 0, 0, 255]))
        .save(root.join("assets/a.png"))
        .unwrap();
    std::fs::File::options()
        .write(true)
        .open(root.join("assets/a.png"))
        .unwrap()
        .set_modified(modified)
        .unwrap();
    let (_, bytes) = cook(&root, None, &options);
    assert_eq!(bytes, expected);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle.output("texture.png").unwrap().file,
        "texture.png.ogb"
    );
    assert_ne!(bundle.read("a.png").unwrap().as_ref(), expected.as_slice());

    std::fs::copy(root.join("assets/texture.png"), root.join("assets/a.png")).unwrap();
    cook(&root, None, &options);
    std::fs::remove_file(root.join("assets/a.png")).unwrap();
    let (r, bytes) = cook(&root, None, &options);
    assert!(r.cooked_outputs().is_empty());
    assert_eq!(bytes, expected);
    assert!(!root.join("cooked/a.png.ogb").exists());
    assert!(root.join("cooked/texture.png.ogb").exists());

    std::fs::copy(root.join("assets/texture.png"), root.join("assets/a.png")).unwrap();
    std::fs::write(
        root.join("assets/main.ogt"),
        "type scene\nversion 1\n---\nstream \"later\" {\n asset \"a.png\"\n asset \"texture.png\"\n}\n",
    )
    .unwrap();
    cook(&root, None, &options);
    let pack = std::fs::read(root.join("cooked/stream/later.ogb")).unwrap();
    assert_eq!(
        og_cook::runtime::unpack(&pack).unwrap(),
        [("a.png.ogb".into(), expected)]
    );
    cook(&root, None, &options);
    assert_eq!(
        std::fs::read(root.join("cooked/stream/later.ogb")).unwrap(),
        pack
    );
}

#[test]
fn shared_texture_results_reuse_invalidate_recover_and_match_uncached_bytes() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("texture-cache-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let options = CookOptions {
        threads: 1,
        ..Default::default()
    };
    for (gltf, normal) in [(false, false), (false, true), (true, false), (true, true)] {
        let base = root.join(format!("gltf-{gltf}-normal-{normal}"));
        let cache = base.join("cache");
        let fresh = |name: &str, value| {
            let p = base.join(name);
            project(&p, value, gltf, normal);
            p
        };
        let (_, expected) = cook(&fresh("uncached", 30), None, &options);
        assert!(!cache.exists());
        let (first, bytes) = cook(&fresh("first", 30), Some(&cache), &options);
        assert!(!hit(&first));
        assert_eq!(bytes, expected);
        let second = fresh("second", 30);
        let (r, bytes) = cook(&second, Some(&cache), &options);
        assert!(hit(&r));
        assert_eq!(bytes, expected);
        // A hit says what the cook said (its notes), then that it was a hit.
        let said = |r: &Report| -> Vec<String> {
            r.assets
                .iter()
                .flat_map(|a| a.notes.iter().cloned())
                .filter(|n| {
                    !n.contains("shared texture cache hit") && !n.contains("shared cook cache hit")
                })
                .collect()
        };
        assert_eq!(said(&r), said(&first), "the hit keeps the cook's notes");
        let (r, _) = cook(&second, Some(&cache), &options);
        assert!(!hit(&r));
        assert!(r.cooked_outputs().is_empty(), "project manifest still wins");

        // Corrupt contents without changing their names or lengths.
        for e in std::fs::read_dir(cache.join("cas")).unwrap() {
            let path = e.unwrap().path();
            let mut bytes = std::fs::read(&path).unwrap();
            if let Some(last) = bytes.last_mut() {
                *last ^= 1;
            }
            std::fs::write(path, bytes).unwrap();
        }
        let (r, bytes) = cook(&fresh("recovery", 30), Some(&cache), &options);
        assert!(!hit(&r));
        assert_eq!(bytes, expected);
        let (r, bytes) = cook(&fresh("repaired", 30), Some(&cache), &options);
        assert!(hit(&r));
        assert_eq!(bytes, expected);

        let (r, bytes) = cook(&fresh("changed", 31), Some(&cache), &options);
        assert!(!hit(&r));
        assert_ne!(bytes, expected);
        let mut changed = options.clone();
        changed.texture.mipmaps = false;
        let (r, bytes) = cook(&fresh("options", 30), Some(&cache), &changed);
        assert!(!hit(&r));
        assert_ne!(bytes, expected);

        let blocked = base.join("not-a-directory");
        std::fs::write(&blocked, b"blocked").unwrap();
        let (r, bytes) = cook(&fresh("unwritable", 30), Some(&blocked), &options);
        assert!(!hit(&r));
        assert_eq!(bytes, expected);

        let concurrent_cache = base.join("concurrent-cache");
        let projects = [fresh("concurrent-a", 30), fresh("concurrent-b", 30)];
        let barrier = std::sync::Barrier::new(2);
        std::thread::scope(|s| {
            let handles: Vec<_> = projects
                .iter()
                .map(|p| {
                    s.spawn(|| {
                        barrier.wait();
                        cook(p, Some(&concurrent_cache), &options).1
                    })
                })
                .collect();
            for h in handles {
                assert_eq!(h.join().unwrap(), expected);
            }
        });
        let (r, bytes) = cook(
            &fresh("after-concurrent", 30),
            Some(&concurrent_cache),
            &options,
        );
        assert!(hit(&r));
        assert_eq!(bytes, expected);

        let store = og_cook::cook::cache::Store::new(concurrent_cache.clone());
        let gc = store.gc(0).unwrap();
        assert!(gc.before > 0);
        assert_eq!(gc.after, 0);
        assert!(projects[0].join("assets/texture.png").is_file());
        let (r, bytes) = cook(&fresh("after-gc", 30), Some(&concurrent_cache), &options);
        assert!(!hit(&r));
        assert_eq!(bytes, expected);
    }
    std::fs::remove_dir_all(root).unwrap();
}
