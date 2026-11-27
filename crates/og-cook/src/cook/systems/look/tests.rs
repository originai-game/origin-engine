use super::super::test_dir::TempDir;
use super::*;

fn source(root: &Path, rel: &str) -> Source {
    Source {
        rel: rel.to_string(),
        path: root.join(rel),
        kind: &LOOK,
        bytes: std::fs::read(root.join(rel)).unwrap(),
        settings: None,
        settings_files: Vec::new(),
        hash: 0,
        gltf: None,
        error: None,
        encoded: Default::default(),
        whole: Default::default(),
        model: Default::default(),
        video: None,
        timeline: None,
        motion_action: None,
        strings: None,
        font_pack: None,
        fields: Default::default(),
        fx: None,
        terrains: Default::default(),
        navmesh: None,
        lightmap: None,
        tileset: None,
        tilemap: None,
        animgraph: None,
        flipbook: None,
        rig2d: None,
        scene: None,
        authored_model: None,
        conformed: None,
        fracture: None,
    }
}

fn project(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new("og-look");
    for (rel, text) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    dir
}

const HEAD: &str = "type look\nversion 1\n---\n";
const PRESET: &str = "type look\nversion 1\n---\nname \"Cel\"\nantialias msaa4\nMaterialStyle {\n  toon true\n  outlineWidth 2\n}\nPostEffects {\n  vignette true\n}\n";

fn cook(root: &Path, rel: &str, known: &[&str]) -> Result<LookFile, String> {
    let s = source(root, rel);
    let known = |id: &str| known.contains(&id);
    resolve(&s, root, &known, &[("cel", PRESET)]).map(|(l, _)| l)
}

#[test]
fn refuses_a_missing_base_and_a_cycle() {
    let root = project(&[
        ("looks/a.ogt", &format!("{HEAD}extends \"looks/b.ogt\"\n")),
        ("looks/b.ogt", &format!("{HEAD}extends \"a\"\n")),
        ("looks/c.ogt", &format!("{HEAD}extends \"nowhere\"\n")),
    ]);
    assert!(
        cook(&root, "looks/a.ogt", &[])
            .unwrap_err()
            .contains("circle")
    );
    let e = cook(&root, "looks/c.ogt", &[]).unwrap_err();
    assert!(
        e.contains("neither assets/looks/nowhere.ogt nor a built-in preset (cel)"),
        "{e}"
    );
}

#[test]
fn the_presets_are_the_template_looks() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/looks");
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
        .map(|d| {
            d.filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(str::to_string))
                .filter_map(|n| n.strip_suffix(".ogt").map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    on_disk.sort();
    let mut built_in: Vec<String> = PRESETS.iter().map(|(s, _)| s.to_string()).collect();
    built_in.sort();
    assert_eq!(built_in, on_disk, "PRESETS lists templates/looks/*.ogt");
    for (slug, text) in PRESETS {
        LookFile::parse(text).unwrap_or_else(|e| panic!("{slug}: {e}"));
    }
    // Every variant file is compiled in, each a variant of a preset.
    let mut on_disk: Vec<String> = Vec::new();
    for slug in std::fs::read_dir(dir.join("variants")).unwrap().flatten() {
        if !slug.path().is_dir() {
            continue;
        }
        let slug_name = slug.file_name().to_str().unwrap().to_string();
        assert!(
            PRESETS.iter().any(|(s, _)| *s == slug_name),
            "variants/{slug_name}: no such preset"
        );
        for v in std::fs::read_dir(slug.path()).unwrap().flatten() {
            let v = v.file_name().to_str().unwrap().to_string();
            on_disk.push(format!("{slug_name}/{}", v.strip_suffix(".ogt").unwrap()));
        }
    }
    on_disk.sort();
    let mut built_in: Vec<String> = VARIANT_LAYERS
        .iter()
        .map(|(s, v, _)| format!("{s}/{v}"))
        .collect();
    built_in.sort();
    assert_eq!(
        built_in, on_disk,
        "VARIANT_LAYERS lists templates/looks/variants/*/*.ogt"
    );
}
#[test]
fn the_built_ins_are_the_template_assets() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/looks/assets");
    let mut on_disk = Vec::new();
    for kind in ["canvas", "palettes"] {
        for e in std::fs::read_dir(dir.join(kind)).unwrap() {
            let name = e.unwrap().file_name().to_string_lossy().to_string();
            on_disk.push(format!("@{kind}/{name}"));
        }
    }
    on_disk.sort();
    let ids: Vec<String> = BUILTINS.iter().map(|(id, _)| id.to_string()).collect();
    assert_eq!(
        ids, on_disk,
        "BUILTINS lists templates/looks/assets, sorted"
    );
    for (id, bytes) in BUILTINS {
        if id.ends_with(".png") {
            assert!(bytes.len() <= 64 * 1024, "{id}: {} B", bytes.len());
            let img = image::load_from_memory(bytes).unwrap();
            assert_eq!((img.width(), img.height()), (256, 256), "{id}");
        } else {
            let colours = crate::look::parse_palette(id, std::str::from_utf8(bytes).unwrap())
                .unwrap_or_else(|e| panic!("{id}: {e}"));
            let digits: String = id.chars().filter(char::is_ascii_digit).collect();
            let want: usize = if id.starts_with("@palettes/1bit") {
                2
            } else {
                digits.parse().unwrap()
            };
            assert_eq!(colours.len(), want, "{id}");
        }
    }
    assert_eq!(
        builtin("@canvas/paper").map(|b| b.0),
        Some("@canvas/paper.png")
    );
    assert_eq!(
        builtin("@canvas/paper.png").map(|b| b.0),
        Some("@canvas/paper.png")
    );
    assert_eq!(builtin("@canvas/pap"), None);
    assert_eq!(builtin("canvas/paper"), None);
}

#[test]
fn every_preset_checks_against_the_records_with_the_built_ins() {
    let builtin_ids: Vec<&str> = BUILTINS
        .iter()
        .map(|(id, _)| *id)
        .chain(styles().iter().map(StyleFile::id))
        .collect();
    // A built-in UI pack is named by its directory (`@styles/<name>`).
    let packs: Vec<String> = style_slugs()
        .iter()
        .map(|s| format!("@styles/{s}"))
        .collect();
    let looks = PRESETS
        .iter()
        .map(|(s, _)| format!("extends \"{s}\""))
        .chain(
            VARIANT_LAYERS
                .iter()
                .map(|(s, v, _)| format!("extends \"{s}\"\nvariant \"{v}\"")),
        );
    for slug in looks {
        let root = project(&[("looks/x.ogt", &format!("{HEAD}{slug}\n"))]);
        let s = source(&root, "looks/x.ogt");
        // An engine theme cooks to `@themes/<name>/pack.json`.
        let theme = |id: &str| {
            id.strip_prefix("@themes/")
                .and_then(|t| t.strip_suffix("/pack.json"))
                .is_some_and(|t| crate::cook::ui_themes::builtin_recipe(t).is_some())
        };
        let known =
            |id: &str| builtin_ids.contains(&id) || packs.iter().any(|p| p == id) || theme(id);
        let (look, _) =
            resolve(&s, &root, &known, PRESETS).unwrap_or_else(|e| panic!("{slug}: {e}"));
        let cap = number_of(&look, "MaterialStyle", "outlineNearCap");
        if slug.starts_with("extends \"anime-cel-3d\"") {
            assert_eq!(cap, Some(vec![1.0]), "{slug}");
        } else {
            assert_eq!(cap, None, "{slug} keeps the default 4× near growth");
        }
        let text = String::from_utf8_lossy(&s.bytes).to_string();
        for (id, _) in builtin_files(&s.rel, &text, &root) {
            // A theme's files are the ones its recipe just cooked.
            assert!(
                id.starts_with("@themes/") || builtin_ids.contains(&id),
                "{slug}: {id}"
            );
        }
    }
}

fn number_of(l: &LookFile, resource: &str, field: &str) -> Option<Vec<f64>> {
    l.blocks
        .iter()
        .find(|b| b.resource == resource)?
        .get(field)?
        .iter()
        .map(|v| v.as_number().map(Number::as_f64))
        .collect()
}
