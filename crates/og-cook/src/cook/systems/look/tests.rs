use super::*;

#[test]
fn embedded_looks_cover_the_template_catalogue() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/looks");
    let mut files = Vec::new();
    fn walk(dir: &Path, root: &Path, files: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                if dir != root
                    || ["moods", "variants"].contains(&entry.file_name().to_str().unwrap())
                {
                    walk(&path, root, files);
                }
            } else if path.extension().is_some_and(|e| e == "ts") {
                files.push(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                );
            }
        }
    }
    walk(&dir, &dir, &mut files);
    let mut embedded: Vec<_> = PRESETS
        .iter()
        .map(|(s, _)| format!("{s}.ts"))
        .chain(
            VARIANT_LAYERS
                .iter()
                .map(|(s, v, _)| format!("variants/{s}/{v}.ts")),
        )
        .chain(MOOD_LAYERS.iter().map(|(m, k, _)| match k {
            Some(k) => format!("moods/{m}/{k}.ts"),
            None => format!("moods/{m}.ts"),
        }))
        .collect();
    files.sort();
    embedded.sort();
    assert_eq!(files, embedded);
    let known = |id: &str| {
        builtin(id).is_some() || id.starts_with("@styles/") || id.starts_with("@themes/")
    };
    for (file, text) in preset_documents().unwrap() {
        if file == "palettes.json" {
            continue;
        }
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(!json["looks"].as_array().unwrap().is_empty(), "{file}");
    }
    for slug in PRESETS.iter().map(|(s, _)| s) {
        let mut look = flatten(
            "@check",
            &BTreeMap::from([(
                "@check".into(),
                LookFile {
                    extends: vec![slug.to_string()],
                    ..Default::default()
                },
            )]),
        )
        .unwrap();
        check(&mut look, &known).unwrap_or_else(|e| panic!("{slug}: {e}"));
        let bytes = look.to_bytes().unwrap();
        let decoded = LookFile::from_bytes(&bytes).unwrap();
        assert_eq!(look_json(&look), look_json(&decoded));
        assert!(LookFile::from_bytes(b"type look\nversion 1\n---\n").is_err());
    }
}

#[test]
fn embedded_art_covers_the_template_assets() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/looks/assets");
    let mut on_disk = Vec::new();
    for kind in ["canvas", "palettes"] {
        for e in std::fs::read_dir(dir.join(kind)).unwrap() {
            on_disk.push(format!(
                "@{kind}/{}",
                e.unwrap().file_name().to_string_lossy()
            ));
        }
    }
    on_disk.sort();
    let ids: Vec<_> = BUILTINS.iter().map(|(id, _)| id.to_string()).collect();
    assert_eq!(ids, on_disk);
    assert!(builtin("@canvas/paper.png").is_some());
    assert!(builtin("@canvas/paper").is_none());
}
