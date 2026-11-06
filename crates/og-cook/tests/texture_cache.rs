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

    // Changing the shared file's owner must not overwrite the surviving alias.
    image::RgbaImage::from_pixel(8, 8, image::Rgba([200, 0, 0, 255]))
        .save(root.join("assets/a.png"))
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
                .filter(|n| !n.contains("shared texture cache hit"))
                .collect()
        };
        assert_eq!(said(&r), said(&first), "the hit keeps the cook's notes");
        let (r, _) = cook(&second, Some(&cache), &options);
        assert!(!hit(&r));
        assert!(r.cooked_outputs().is_empty(), "project manifest still wins");

        // Flip a payload byte without changing either blob's header or length.
        for e in std::fs::read_dir(&cache).unwrap() {
            let path = e.unwrap().path();
            let mut bytes = std::fs::read(&path).unwrap();
            let blob = og_blob::Blob::parse(&bytes, "entry").unwrap();
            let section = blob.section("result").unwrap();
            let offset = (section.offset + section.size - 1) as usize;
            bytes[offset] ^= 1;
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
    }
    std::fs::remove_dir_all(root).unwrap();
}
