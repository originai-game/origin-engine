use super::*;
use std::path::PathBuf;

fn source(root: &Path, rel: &str) -> Source {
    Source {
        rel: rel.to_string(),
        path: root.join(rel),
        kind: super::super::Kind::Look,
        bytes: std::fs::read(root.join(rel)).unwrap(),
        settings: None,
        hash: 0,
        gltf: None,
        error: None,
        encoded: Default::default(),
        conformed: None,
    }
}

fn project(files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "og-look-{}-{}",
        std::process::id(),
        super::super::hash_bytes(format!("{files:?}").as_bytes())
    ));
    let _ = std::fs::remove_dir_all(&dir);
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
fn extends_a_preset_and_the_games_own_look_wins() {
    let root = project(&[(
        "looks/main.ogt",
        &format!(
            "{HEAD}extends \"cel\"\nStylise {{\n  posterize 5\n}}\nMaterialStyle {{\n  outlineWidth 3\n}}\n"
        ),
    )]);
    let l = cook(&root, "looks/main.ogt", &[]).unwrap();
    assert_eq!(l.extends, vec!["cel"]);
    assert_eq!(l.name, None);
    assert_eq!(l.antialias.as_deref(), Some("msaa4"));
    let m = l
        .blocks
        .iter()
        .find(|b| b.resource == "MaterialStyle")
        .unwrap();
    assert_eq!(
        m.get("outlineWidth").unwrap()[0]
            .as_number()
            .unwrap()
            .text(),
        "3"
    );
    assert_eq!(m.get("toon").unwrap()[0].as_word(), Some("true"));
    assert_eq!(
        l.resources(),
        vec!["PostEffects", "Stylise", "MaterialStyle"]
    );

    // The game's own looks/cel.ogt overrides the preset of that slug,
    // and itself extends the preset.
    let root = project(&[
        (
            "looks/cel.ogt",
            &format!("{HEAD}extends \"cel\"\nname \"Our cel\"\n"),
        ),
        ("looks/main.ogt", &format!("{HEAD}extends \"cel\"\n")),
    ]);
    let l = cook(&root, "looks/main.ogt", &[]).unwrap();
    assert_eq!(l.extends, vec!["looks/cel.ogt", "cel"]);
    // A look is not called after its base.
    assert_eq!(l.name, None);
}

#[test]
fn checks_fields_against_the_records() {
    let one = |body: &str, known: &[&str]| {
        let root = project(&[("looks/a.ogt", &format!("{HEAD}{body}"))]);
        cook(&root, "looks/a.ogt", known)
    };
    let l = one("PostEffects {\n  outlineColor \"#ffffff\"\n  aoMethod gtao\n}\nStylise {\n  palette \"palettes/p.hex\"\n  dither bayer8\n}\n", &["palettes/p.hex"]).unwrap();
    let p = l
        .blocks
        .iter()
        .find(|b| b.resource == "PostEffects")
        .unwrap();
    assert_eq!(p.get("outlineColor").unwrap().len(), 3);
    assert!(
        one("Stylise {\n  posterize\n}\n", &[])
            .unwrap_err()
            .contains("line 5: Stylise.posterize has no value")
    );
    assert!(
        one("Stylise {\n  posterise 4\n}\n", &[])
            .unwrap_err()
            .contains("Stylise has no field 'posterise'")
    );
    assert!(
        one("Stylise {\n  posterize 100\n}\n", &[])
            .unwrap_err()
            .contains("outside 0 to 64")
    );
    assert!(
        one("Stylise {\n  posterize 2.5\n}\n", &[])
            .unwrap_err()
            .contains("whole number")
    );
    assert!(
        one("Stylise {\n  dither bayer3\n}\n", &[])
            .unwrap_err()
            .contains("one of none, bayer2")
    );
    assert!(
        one("Stylise {\n  palette \"palettes/p.png\"\n}\n", &[])
            .unwrap_err()
            .contains("not a Palette")
    );
    assert!(
        one("Stylise {\n  palette \"palettes/p.hex\"\n}\n", &[])
            .unwrap_err()
            .contains("not under assets/: palettes/p.hex")
    );
    // A list on its one line: a handle per element by its asset id,
    // at most the capacity, the key alone for the empty list.
    let l = one(
        "PostEffects {\n  custom \"post/tint.ogt\" \"post/crt.ogt\"\n}\n",
        &["post/tint.ogt", "post/crt.ogt"],
    )
    .unwrap();
    let p = l
        .blocks
        .iter()
        .find(|b| b.resource == "PostEffects")
        .unwrap();
    let ids: Vec<&str> = p
        .get("custom")
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert_eq!(ids, ["post/tint.ogt", "post/crt.ogt"]);
    let empty = one("PostEffects {\n  custom\n}\n", &[]);
    assert!(empty.is_ok(), "{:?}", empty.err());
    let nine = format!(
        "PostEffects {{\n  custom{}\n}}\n",
        " \"post/a.ogt\"".repeat(9)
    );
    assert!(
        one(&nine, &["post/a.ogt"])
            .unwrap_err()
            .contains("PostEffects.custom holds at most 8, got 9")
    );
    assert!(
        one("PostEffects {\n  custom \"post/a.png\"\n}\n", &[])
            .unwrap_err()
            .contains("not a Post (.ogt)")
    );
    assert!(
        one("PostEffects {\n  outline 1\n}\n", &[])
            .unwrap_err()
            .contains("true or false")
    );
    assert!(
        one("Fog {\n  color \"#12\"\n}\n", &[])
            .unwrap_err()
            .contains("not an sRGB")
    );
    // `ui`: an asset id the bundle must have; a slug names the game's
    // own style first, else a built-in; neither names both.
    assert!(
        one("ui \"styles/x/style.ogt\"\n", &[])
            .unwrap_err()
            .contains("styles/x/style.ogt")
    );
    let l = one("ui \"pixel\"\n", &["@styles/pixel/style.ogt"]).unwrap();
    assert_eq!(l.ui.as_deref(), Some("@styles/pixel/style.ogt"));
    let l = one(
        "ui \"pixel\"\n",
        &["styles/pixel/style.ogt", "@styles/pixel/style.ogt"],
    )
    .unwrap();
    assert_eq!(
        l.ui.as_deref(),
        Some("styles/pixel/style.ogt"),
        "the game's own wins"
    );
    let e = one("ui \"velvet\"\n", &[]).unwrap_err();
    assert!(e.contains("velvet") && e.contains("pixel, scifi"), "{e}");
    assert!(
        one("skin \"skins/x/skin.ogt\"\n", &[])
            .unwrap_err()
            .contains("'skin' is now 'ui'")
    );
}

#[test]
fn the_art_contract_checks_like_the_frames_resources() {
    let one = |body: &str, known: &[&str]| {
        let root = project(&[("looks/b.ogt", &format!("{HEAD}{body}"))]);
        cook(&root, "looks/b.ogt", known)
    };
    let l = one(
        "Stylise {\n  posterize 4\n}\nArtPalette {\n  colors \"@palettes/dusk16\"\n  maxColors 16\n  accents 2 5\n  dither bayer4\n}\nArtPixels {\n  unit 16\n  spriteSizes 16 32\n  rotation cleanEdge\n}\nArtLine {\n  width 1\n  color \"#ffffff\"\n  place inside\n}\nArtCamera {\n  projection orthographic\n  pixelPerfect true\n}\n",
        &["@palettes/dusk16.hex"],
    )
    .unwrap();
    let block = |r: &str| l.blocks.iter().find(|b| b.resource == r).unwrap();
    assert_eq!(
        block("ArtPalette").get("colors").unwrap()[0].as_str(),
        Some("@palettes/dusk16.hex")
    );
    let line: Vec<f64> = block("ArtLine")
        .get("color")
        .unwrap()
        .iter()
        .map(|v| v.as_number().unwrap().as_f64())
        .collect();
    assert_eq!(line, [1.0, 1.0, 1.0], "an sRGB colour is stored linear");
    assert_eq!(
        l.resources(),
        ["Stylise", "ArtPalette", "ArtPixels", "ArtLine", "ArtCamera"]
    );
    let bad = |body: &str| one(body, &[]).unwrap_err();
    let e = bad("ArtPixels {\n  rotation spin\n}\n");
    assert!(
        e.contains("one of free, nearest, cleanEdge, prerotated"),
        "{e}"
    );
    let e = bad("ArtPalette {\n  maxColors 300\n}\n");
    assert!(e.contains("outside 0 to 256"), "{e}");
    let e = bad("ArtPalette {\n  accents 1.5\n}\n");
    assert!(e.contains("not a u32"), "{e}");
    let e = bad("ArtPixels {\n  spriteSizes 1 2 3 4 5 6 7 8 9\n}\n");
    assert!(e.contains("holds at most 8, got 9"), "{e}");
    let e = bad("ArtShape {\n  voxel -0.1\n}\n");
    assert!(e.contains("outside 0 to"), "{e}");
    let e = bad("ArtPalette {\n  colors \"palettes/p.png\"\n}\n");
    assert!(e.contains("not a Palette"), "{e}");
    let e = bad("ArtLine {\n  thickness 2\n}\n");
    assert!(e.contains("ArtLine has no field 'thickness'"), "{e}");

    // The contract is the look's art blocks alone, through its chain.
    let root = project(&[
        (
            "looks/base.ogt",
            &format!("{HEAD}Fog {{\n  enabled true\n}}\nArtShape {{\n  normals flat\n}}\n"),
        ),
        (
            "looks/main.ogt",
            &format!(
                "{HEAD}extends \"looks/base.ogt\"\nStylise {{\n  posterize 4\n}}\nArtMotion {{\n  stepFps 12\n}}\n"
            ),
        ),
    ]);
    let c = contract(&source(&root, "looks/main.ogt"), &root, &|_| false).unwrap();
    assert_eq!(c.resources(), ["ArtShape", "ArtMotion"]);
    let root = project(&[(
        "looks/plain.ogt",
        &format!("{HEAD}Stylise {{\n  posterize 4\n}}\n"),
    )]);
    let c = contract(&source(&root, "looks/plain.ogt"), &root, &|_| false).unwrap();
    assert!(
        c.blocks.is_empty(),
        "a look without art blocks constrains nothing"
    );
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
}
#[test]
fn names_the_built_in_assets_by_their_reserved_ids() {
    let one = |body: &str, known: &[&str]| {
        let root = project(&[("looks/b.ogt", &format!("{HEAD}{body}"))]);
        cook(&root, "looks/b.ogt", known)
    };
    let l = one(
        "Stylise {\n  paintCanvas \"@canvas/paper\"\n  palette \"@palettes/handheld4.hex\"\n}\n",
        &["@canvas/paper.png", "@palettes/handheld4.hex"],
    )
    .unwrap();
    let st = l.blocks.iter().find(|b| b.resource == "Stylise").unwrap();
    assert_eq!(
        st.get("paintCanvas").unwrap()[0].as_str(),
        Some("@canvas/paper.png")
    );
    assert_eq!(
        st.get("palette").unwrap()[0].as_str(),
        Some("@palettes/handheld4.hex")
    );
    let e = one("Stylise {\n  paintCanvas \"@canvas/burlap\"\n}\n", &[]).unwrap_err();
    assert!(
        e.contains(
            "\"@canvas/burlap\" is not one of the engine's built-in Textures (@canvas/linen, @canvas/paper, @canvas/watercolor)"
        ),
        "{e}"
    );
    let e = one("Stylise {\n  paintCanvas \"@palettes/1bit\"\n}\n", &[]).unwrap_err();
    assert!(e.contains("is not a Texture"), "{e}");
    // Named but not in the bundle: the cook adds the built-ins a look
    // names before it checks the look, so this is a cook without them.
    let e = one("Stylise {\n  palette \"@palettes/lcd2\"\n}\n", &[]).unwrap_err();
    assert!(e.contains("not under assets/: @palettes/lcd2.hex"), "{e}");
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
            let want: usize = if *id == "@palettes/1bit.hex" {
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

/// Every file of a built-in style's directory is compiled in, as a
/// cooked file or a licence, so `og style eject` writes the whole pack.
#[test]
fn every_built_in_style_file_is_compiled_in() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    for slug in style_slugs() {
        let mut on_disk: Vec<String> = std::fs::read_dir(dir.join(slug))
            .unwrap()
            .map(|e| {
                format!(
                    "@styles/{slug}/{}",
                    e.unwrap().file_name().to_string_lossy()
                )
            })
            .collect();
        on_disk.sort();
        let prefix = format!("@styles/{slug}/");
        let mut ids: Vec<String> = STYLES
            .iter()
            .chain(STYLE_LICENCES)
            .filter(|(id, _)| id.starts_with(&prefix))
            .map(|(id, _)| id.to_string())
            .collect();
        ids.sort();
        assert_eq!(ids, on_disk, "{slug}");
    }
}

/// A built-in style's Chinese companions join the bundle of a game that
/// shows Chinese (a `zh` string table) and only then: a game in English
/// does not carry megabytes of outlines it never draws.
#[test]
fn a_styles_chinese_companions_ride_with_a_chinese_game_only() {
    let look = ("looks/x.ogt", "type look\nversion 1\n---\nui \"paper\"\n");
    let cjk = |files: &[(&str, &str)]| -> Vec<&'static str> {
        let root = project(files);
        builtin_files(look.0, look.1, &root)
            .into_iter()
            .map(|(id, _)| id)
            .filter(|id| id.starts_with("@styles/cjk/"))
            .collect()
    };
    let english = cjk(&[
        look,
        (
            "strings/en.ogt",
            "type strings\nversion 1\n---\nlocale \"en\"\n",
        ),
    ]);
    assert!(english.is_empty(), "{english:?}");
    let chinese = cjk(&[
        look,
        (
            "strings/zh.ogt",
            "type strings\nversion 1\n---\nlocale \"zh-CN\"\n",
        ),
    ]);
    assert_eq!(
        chinese,
        [
            "@styles/cjk/NotoSansSC-Regular.ttf",
            "@styles/cjk/NotoSansSC-Regular.ttf.ogt",
            "@styles/cjk/NotoSerifSC-SemiBold.ttf",
            "@styles/cjk/NotoSerifSC-SemiBold.ttf.ogt",
        ]
    );
}

/// A generated seed style draws with the packs' fonts and icons
/// (`@styles/fantasy/Cinzel-Bold.ttf`): a game that names it gets
/// them in its bundle, whatever its languages, with their settings.
#[test]
fn a_seed_styles_shared_fonts_and_icons_ride_with_the_game() {
    let look = ("looks/x.ogt", "type look\nversion 1\n---\nui \"gilded\"\n");
    let root = project(&[look]);
    let ids: Vec<&str> = builtin_files(look.0, look.1, &root)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    for id in [
        "@styles/gilded/style.ogt",
        "@styles/gilded/sfx-tap.ogt",
        "@styles/fantasy/Cinzel-Bold.ttf",
        "@styles/fantasy/Cinzel-Bold.ttf.ogt",
        "@styles/fantasy/Newsreader-Medium.ttf",
        "@styles/fantasy/icons.png",
        "@styles/fantasy/icons.png.ogt",
    ] {
        assert!(ids.contains(&id), "{id}: {ids:?}");
    }
    assert!(
        !ids.iter().any(|id| id.starts_with("@styles/cjk/")),
        "{ids:?}"
    );
    assert!(!ids.contains(&"@styles/fantasy/atlas.png"), "{ids:?}");
}

#[test]
fn every_preset_checks_against_the_records_with_the_built_ins() {
    let builtin_ids: Vec<&str> = BUILTINS.iter().chain(STYLES).map(|(id, _)| *id).collect();
    for (slug, _) in PRESETS {
        let root = project(&[("looks/x.ogt", &format!("{HEAD}extends \"{slug}\"\n"))]);
        let s = source(&root, "looks/x.ogt");
        let known = |id: &str| builtin_ids.contains(&id);
        resolve(&s, &root, &known, PRESETS).unwrap_or_else(|e| panic!("{slug}: {e}"));
        let text = String::from_utf8_lossy(&s.bytes).to_string();
        for (id, _) in builtin_files(&s.rel, &text, &root) {
            assert!(builtin_ids.contains(&id), "{slug}: {id}");
        }
    }
}
