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
        whole: Default::default(),
        conformed: None,
        fracture: None,
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

/// A checkout reads its own packs from disk (nothing is compiled into
/// `og`): a pack's cooked files, its densities included, are what a game
/// naming it gets; its licences are `og style eject`'s only; its `art/`
/// sources are neither.
#[test]
fn a_built_in_style_is_its_packs_files_read_from_the_checkout() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    assert_eq!(
        styles_dir().map(|d| d.canonicalize().unwrap()),
        Some(dir.canonicalize().unwrap())
    );
    let ids: Vec<&str> = style_files("@styles/fantasy/style.ogt")
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    for want in [
        "style.ogt",
        "atlas.png",
        "atlas@2x.png",
        "atlas@3x.png",
        "Cinzel-Bold.ttf",
    ] {
        assert!(
            ids.contains(&format!("@styles/fantasy/{want}").as_str()),
            "{ids:?}"
        );
    }
    assert!(
        !ids.iter()
            .any(|id| id.contains("/art/") || id.ends_with(".md") || id.ends_with("OFL.txt")),
        "{ids:?}"
    );
    let style = style_files("@styles/fantasy/style.ogt")
        .into_iter()
        .find(|(id, _)| *id == "@styles/fantasy/style.ogt")
        .unwrap()
        .1;
    assert_eq!(style, std::fs::read(dir.join("fantasy/style.ogt")).unwrap());
    let licences: Vec<&str> = style_licences().iter().map(|f| f.id()).collect();
    assert!(
        licences.contains(&"@styles/fantasy/LICENSES.md"),
        "{licences:?}"
    );
    assert!(
        licences.contains(&"@styles/fantasy/art-requests.json"),
        "{licences:?}"
    );
}

/// A built-in style's Chinese companions join the bundle of a game that
/// shows Chinese (a `zh` string table) and only then: a game in English
/// does not carry megabytes of outlines it never draws.
#[test]
fn a_styles_chinese_companions_ride_with_a_chinese_game_only() {
    let look = ("looks/x.ogt", "type look\nversion 1\n---\nui \"fantasy\"\n");
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
            "@styles/cjk/NotoSerifSC-SemiBold.ttf",
            "@styles/cjk/NotoSerifSC-SemiBold.ttf.ogt",
        ]
    );
}

#[test]
fn every_preset_checks_against_the_records_with_the_built_ins() {
    let builtin_ids: Vec<&str> = BUILTINS
        .iter()
        .map(|(id, _)| *id)
        .chain(styles().iter().map(StyleFile::id))
        .collect();
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

fn number_of(l: &LookFile, resource: &str, field: &str) -> Option<Vec<f64>> {
    l.blocks
        .iter()
        .find(|b| b.resource == resource)?
        .get(field)?
        .iter()
        .map(|v| v.as_number().map(Number::as_f64))
        .collect()
}

#[test]
fn a_mood_is_laid_over_the_preset_and_the_games_fields_still_win() {
    // A preset drawn warm with its own exposure and a vignette at 0.6.
    let preset = "type look\nversion 1\n---\nmood \"warm\"\nRenderSettings {\n  exposure 0.8\n}\nPostEffects {\n  vignette true\n  vignetteIntensity 0.6\n  contrast 0.9\n}\n";
    let presets = [("p", preset)];
    let cook_in = |game: &str| {
        let root = project(&[("looks/g.ogt", &format!("{HEAD}extends \"p\"\n{game}"))]);
        let s = source(&root, "looks/g.ogt");
        resolve(&s, &root, &|_: &str| true, &presets).map(|(l, _)| l)
    };
    // Its own mood, asked or not: the preset as drawn.
    for game in ["", "mood \"warm\"\n"] {
        let l = cook_in(game).unwrap();
        assert_eq!(number_of(&l, "RenderSettings", "exposure"), Some(vec![0.8]));
        assert_eq!(l.mood.as_deref(), Some("warm"));
    }
    // Dark: exposure 0.72 over the default 1 moves 0.8 to 0.52; the
    // vignette 0.55 over its default 0.35 moves 0.6 to 0.8; contrast 0.12
    // over 0 clamps 0.9 + 0.12 at the field's 1; a field the preset leaves
    // unset moves from its default (temperature -0.06, balanceShadows).
    let l = cook_in("mood \"dark\"\n").unwrap();
    assert_eq!(l.mood.as_deref(), Some("dark"));
    assert_eq!(
        number_of(&l, "RenderSettings", "exposure"),
        Some(vec![0.52])
    );
    assert_eq!(
        number_of(&l, "PostEffects", "vignetteIntensity"),
        Some(vec![0.8])
    );
    assert_eq!(number_of(&l, "PostEffects", "contrast"), Some(vec![1.0]));
    assert_eq!(
        number_of(&l, "PostEffects", "temperature"),
        Some(vec![-0.06])
    );
    assert_eq!(
        number_of(&l, "PostEffects", "balanceShadows"),
        Some(vec![-0.02, 0.0, 0.04])
    );
    // The game's own field is laid after the mood and wins whole.
    let l = cook_in("mood \"dark\"\nRenderSettings {\n  exposure 1.3\n}\n").unwrap();
    assert_eq!(number_of(&l, "RenderSettings", "exposure"), Some(vec![1.3]));
    // A flashback in another mood extends the game's look: its mood wins
    // for the whole chain, laid over the preset once.
    let root = project(&[
        (
            "looks/g.ogt",
            &format!("{HEAD}extends \"p\"\nmood \"cold\"\n"),
        ),
        (
            "looks/f.ogt",
            &format!("{HEAD}extends \"looks/g.ogt\"\nmood \"dark\"\n"),
        ),
    ]);
    let s = source(&root, "looks/f.ogt");
    let (l, _) = resolve(&s, &root, &|_: &str| true, &presets).unwrap();
    assert_eq!(l.mood.as_deref(), Some("dark"));
    assert_eq!(
        number_of(&l, "RenderSettings", "exposure"),
        Some(vec![0.52])
    );
    assert_eq!(
        number_of(&l, "PostEffects", "temperature"),
        Some(vec![-0.06])
    );
    // A chain without a preset takes the layer under its root.
    let root = project(&[("looks/solo.ogt", &format!("{HEAD}mood \"cold\"\n"))]);
    let l = cook(&root, "looks/solo.ogt", &[]).unwrap();
    assert_eq!(
        number_of(&l, "PostEffects", "temperature"),
        Some(vec![-0.3])
    );
    assert!(LookFile::parse(&l.to_text()).unwrap().mood.as_deref() == Some("cold"));
}

#[test]
fn a_palette_bound_preset_swaps_its_palette_for_the_moods() {
    let root = project(&[(
        "looks/g.ogt",
        &format!("{HEAD}extends \"monochrome-pixel\"\nmood \"warm\"\n"),
    )]);
    let s = source(&root, "looks/g.ogt");
    let (l, bases) = resolve(&s, &root, &|_: &str| true, PRESETS).unwrap();
    let palette = |r: &str, f: &str| {
        l.blocks
            .iter()
            .find(|b| b.resource == r)
            .unwrap()
            .get(f)
            .unwrap()[0]
            .as_str()
            .map(str::to_string)
    };
    assert_eq!(
        palette("Stylise", "palette").as_deref(),
        Some("@palettes/handheld4-warm.hex")
    );
    assert_eq!(
        palette("ArtPalette", "colors").as_deref(),
        Some("@palettes/handheld4-warm.hex")
    );
    // Its own layer replaces the generic one: no grading from warm.ogt.
    assert_eq!(number_of(&l, "PostEffects", "temperature"), None);
    assert!(bases.iter().any(|b| b.id == "@moods/warm/monochrome-pixel"));
    // The cook bundles the palette the mood names.
    let files = builtin_files(
        "looks/g.ogt",
        &std::fs::read_to_string(root.join("looks/g.ogt")).unwrap(),
        &root,
    );
    assert!(
        files
            .iter()
            .any(|(id, _)| *id == "@palettes/handheld4-warm.hex")
    );
}

#[test]
fn every_preset_names_its_mood_and_every_mood_lays_on_every_preset() {
    for (slug, text) in PRESETS {
        let preset = LookFile::parse(text).unwrap();
        let own = preset
            .mood
            .as_deref()
            .unwrap_or_else(|| panic!("{slug}: no mood line"));
        for mood in crate::look::MOODS {
            let root = project(&[(
                "looks/g.ogt",
                &format!("{HEAD}extends \"{slug}\"\nmood \"{mood}\"\n"),
            )]);
            let s = source(&root, "looks/g.ogt");
            let (l, _) = resolve(&s, &root, &|_: &str| true, PRESETS)
                .unwrap_or_else(|e| panic!("{slug} in {mood}: {e}"));
            assert_eq!(l.mood.as_deref(), Some(mood));
            if mood == own {
                assert_eq!(
                    l.blocks.len(),
                    preset.blocks.len(),
                    "{slug}: its own mood lays nothing"
                );
            }
        }
    }
    // Every layer sets only what a mood may, and names fields that exist.
    for (mood, slug, _) in MOOD_LAYERS {
        assert!(crate::look::MOODS.contains(mood), "{mood}");
        lay_mood(&LookFile::default(), mood, *slug).unwrap_or_else(|e| panic!("{e}"));
        if let Some(slug) = slug {
            assert!(
                PRESETS.iter().any(|(s, _)| s == slug),
                "{mood}/{slug}: no such preset"
            );
        }
    }
}

#[test]
fn a_mood_layer_never_changes_the_art_contracts_sizes() {
    assert!(mood_sets("PostEffects", "contrast"));
    assert!(mood_sets("ArtPalette", "colors"));
    assert!(!mood_sets("ArtPixels", "unit"));
    assert!(!mood_sets("Stylise", "posterize"));
}
