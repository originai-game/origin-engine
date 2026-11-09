use super::*;

/// A fresh project whose one look names the UI style `ui`, optionally with a
/// Chinese string table.
fn look_project(ui: &str, chinese: bool) -> TempDir {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("assets/looks")).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("assets/looks/x.ogt"),
        format!("type look\nversion 1\n---\nui \"{ui}\"\n"),
    )
    .unwrap();
    if chinese {
        std::fs::create_dir_all(dir.join("assets/strings")).unwrap();
        std::fs::write(
            dir.join("assets/strings/zh.ogt"),
            "type strings\nversion 1\n---\nlocale \"zh-CN\"\n",
        )
        .unwrap();
    }
    dir
}

/// `og style eject fixture keeper` writes a supplied style's whole pack
/// (byte for byte) into the game's
/// assets/styles/keeper/, refuses to overwrite it without --force, and the
/// ejected style cooks to the built-in's blobs (only the cook stamp in the
/// header differs): the same palette, atlas, icons and fonts. Its `cjk`
/// fonts stay the shared built-ins, bundled for a game that shows Chinese.
///
/// Ways it could fail: a file of the pack is left out or altered, an
/// existing file is overwritten, or the ejected copy cooks differently
/// (an id resolved elsewhere, a font's import settings lost).
#[test]
fn style_eject_writes_the_built_in_pack_that_cooks_the_same() {
    let styles = scratch_root();
    let template = styles.join("fixture");
    std::fs::create_dir_all(&template).unwrap();
    let choices =
        og_ui::style_gen::StyleChoices::for_look("fixture", "minimal modern", "eject test");
    let generated = og_ui::style_gen::generate(&choices).unwrap();
    let mut doc = og_ui::read_style_doc(&generated.text).unwrap();
    doc.body.entries.retain(|entry| entry.key != "fonts");
    for (name, text) in &generated.sounds {
        std::fs::write(template.join(name), text).unwrap();
    }
    let style = og_ui::write_style_doc(&doc)
        .unwrap()
        .replace("@styles/shared/ink-icons.png", "icons.png")
        + "fonts {\n  body \"Body.ttf\"\n  cjk \"@styles/cjk/NotoSansSC-Regular.ttf\"\n}\n";
    std::fs::write(template.join("style.ogt"), style).unwrap();
    std::fs::write(template.join("LICENSES.md"), "Fixture font: SIL OFL 1.1").unwrap();
    std::fs::copy(
        repo().join("templates/styles/shared/ink-icons.png"),
        template.join("icons.png"),
    )
    .unwrap();
    std::fs::copy(
        repo().join("templates/styles/shared/InstrumentSans-Regular.ttf"),
        template.join("Body.ttf"),
    )
    .unwrap();
    std::fs::copy(
        repo().join("templates/styles/shared/InstrumentSans-Regular.ttf.ogt"),
        template.join("Body.ttf.ogt"),
    )
    .unwrap();
    std::fs::create_dir_all(styles.join("cjk")).unwrap();
    for name in ["NotoSansSC-Regular.ttf", "NotoSansSC-Regular.ttf.ogt"] {
        std::fs::copy(
            repo().join("templates/styles/cjk").join(name),
            styles.join("cjk").join(name),
        )
        .unwrap();
    }
    let command = || {
        let mut cmd = og();
        cmd.env("OG_STYLES_DIR", &styles);
        cmd
    };
    let mine = look_project("styles/keeper/style.ogt", true);
    let out = command()
        .args(["style", "eject", "fixture", "keeper", "--project"])
        .arg(&mine)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert!(
        text(&out).contains("ui \"styles/keeper/style.ogt\""),
        "{}",
        text(&out)
    );
    let ejected = mine.join("assets/styles/keeper");
    fn names(d: &Path) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(d)
            .unwrap()
            .flat_map(|e| {
                let e = e.unwrap();
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() {
                    names(&e.path())
                        .into_iter()
                        .map(|f| format!("{name}/{f}"))
                        .collect()
                } else {
                    vec![name]
                }
            })
            .collect();
        v.sort();
        v
    }
    assert_eq!(names(&ejected), names(&template));
    for f in names(&template) {
        if matches!(f.as_str(), "extension.json" | "choices.json") {
            let mut expected: serde_json::Value =
                serde_json::from_slice(&std::fs::read(template.join(&f)).unwrap()).unwrap();
            expected["name"] = "keeper".into();
            let actual: serde_json::Value =
                serde_json::from_slice(&std::fs::read(ejected.join(&f)).unwrap()).unwrap();
            assert_eq!(actual, expected);
            continue;
        }
        assert!(
            std::fs::read(ejected.join(&f)).unwrap() == std::fs::read(template.join(&f)).unwrap(),
            "{f} differs from the source pack"
        );
    }

    // An existing pack is kept unless --force.
    std::fs::write(ejected.join("style.ogt"), "edited").unwrap();
    let again = command()
        .args(["style", "eject", "fixture", "keeper", "--project"])
        .arg(&mine)
        .output()
        .unwrap();
    assert_eq!(again.status.code(), Some(1), "{}", text(&again));
    assert!(text(&again).contains("--force"), "{}", text(&again));
    assert_eq!(
        std::fs::read_to_string(ejected.join("style.ogt")).unwrap(),
        "edited"
    );
    let forced = command()
        .args([
            "style",
            "eject",
            "fixture",
            "keeper",
            "--force",
            "--project",
        ])
        .arg(&mine)
        .output()
        .unwrap();
    assert_eq!(forced.status.code(), Some(0), "{}", text(&forced));

    let builtin = look_project("fixture", true);
    for dir in [&mine, &builtin] {
        let out = command().arg("cook").arg(dir).output().unwrap();
        assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    }
    let a = og_cook::runtime::Bundle::open(&mine.join("cooked")).unwrap();
    let b = og_cook::runtime::Bundle::open(&builtin.join("cooked")).unwrap();
    assert!(
        a.output("@styles/fixture/style.ogt").is_err(),
        "the ejected style replaces the built-in in the bundle"
    );
    // The blob header's cook stamp (bytes 32..40) hashes the source id.
    let unstamped = |bundle: &og_cook::runtime::Bundle, id: &str| -> Vec<u8> {
        let mut v = bundle
            .read(id)
            .unwrap_or_else(|e| panic!("{id}: {e}"))
            .to_vec();
        v[32..40].fill(0);
        v
    };
    for f in ["style.ogt", "Body.ttf"] {
        assert!(
            unstamped(&a, &format!("styles/keeper/{f}"))
                == unstamped(&b, &format!("@styles/fixture/{f}")),
            "{f}: the ejected copy cooks differently from the built-in"
        );
    }
    // The fixture names one CJK companion.
    let id = "@styles/cjk/NotoSansSC-Regular.ttf";
    assert!(
        unstamped(&a, id) == unstamped(&b, id),
        "{id}: the ejected style's Chinese companion"
    );
    let _ = std::fs::remove_dir_all(&mine);
    let _ = std::fs::remove_dir_all(&builtin);
    let _ = std::fs::remove_dir_all(&styles);
}

/// Ornament packing keeps provenance, trimming, density and rerun contracts;
/// part-art requests are left alone rather than converted into style rules.
#[test]
fn style_art_packs_ornaments_at_three_densities() {
    let dir = look_project("styles/arena/style.ogt", false);
    let out = og()
        .args(["style", "new", "arena", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let pack = dir.join("assets/styles/arena");
    std::fs::write(
        pack.join("art-requests.json"),
        serde_json::json!({
            "slots": [
                {"name": "arena/corner", "kind": "ornament", "size": [32, 32]},
                {"name": "arena/button-primary", "kind": "styleButton"}
            ]
        })
        .to_string(),
    )
    .unwrap();
    let art = |args: &[&str]| {
        og().args(["style", "art"])
            .arg(&pack)
            .args(args)
            .output()
            .unwrap()
    };
    let out = art(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let requests = || -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(pack.join("art-requests.json")).unwrap())
            .unwrap()
    };
    let slot = |name: &str| -> serde_json::Value {
        requests()["slots"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["name"] == name)
            .cloned()
            .unwrap_or_else(|| panic!("no {name} in {}", requests()))
    };
    let primary = slot("arena/button-primary");
    assert!(primary.get("file").is_none(), "part art must not be packed");
    assert!(text(&out).contains("arena/button-primary skipped"));
    let corner = slot("arena/corner");
    assert_eq!(corner["pixels"], serde_json::json!([96, 96]));

    // A red border round a blue middle, inside a transparent margin.
    let art_dir = pack.join("art");
    std::fs::create_dir_all(&art_dir).unwrap();
    let mut img = image::RgbaImage::new(328, 328);
    for (x, y, p) in img.enumerate_pixels_mut() {
        let (x, y) = (x as i32 - 20, y as i32 - 20);
        if (0..288).contains(&x) && (0..288).contains(&y) {
            let edge = x < 18 || y < 18 || x >= 270 || y >= 270;
            *p = if edge {
                image::Rgba([220, 30, 30, 255])
            } else {
                image::Rgba([30, 60, 220, 255])
            };
        }
    }
    img.save(art_dir.join("corner.png")).unwrap();
    let out = art(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(
        text(&out).contains("no entry for arena/corner"),
        "{}",
        text(&out)
    );
    let provenance = serde_json::json!({
        "arena/corner": { "model": "hand", "prompt": "a test corner", "request": corner["hash"] },
    });
    std::fs::write(art_dir.join("provenance.json"), provenance.to_string()).unwrap();
    let out = art(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));

    let style = std::fs::read_to_string(pack.join("style.ogt")).unwrap();
    let file = og_ui::parse_style(&style).unwrap_or_else(|e| panic!("{e}\n{style}"));
    assert_eq!(
        file.atlas_dense,
        [Some("atlas@2x.png".into()), Some("atlas@3x.png".into())]
    );
    let atlases: Vec<image::RgbaImage> = ["atlas.png", "atlas@2x.png", "atlas@3x.png"]
        .iter()
        .map(|f| image::open(pack.join(f)).unwrap().to_rgba8())
        .collect();
    let (w, h) = atlases[0].dimensions();
    assert_eq!(atlases[1].dimensions(), (w * 2, h * 2));
    assert_eq!(atlases[2].dimensions(), (w * 3, h * 3));
    let s: Vec<_> = style
        .lines()
        .find(|l| l.starts_with("corner "))
        .unwrap()
        .split_whitespace()
        .collect();
    let n = |i: usize| -> u32 { s[i].parse().unwrap() };
    assert_eq!((n(3), n(4)), (32, 32), "{s:?}");
    let (x, y) = (n(1), n(2));
    // Trimmed: the border's red at the cell's very edge, at every density,
    // and the blue middle at its centre.
    for (k, a) in atlases.iter().enumerate() {
        let k = k as u32 + 1;
        let edge = a.get_pixel(x * k, (y + 16) * k);
        let mid = a.get_pixel((x + 16) * k, (y + 16) * k);
        assert!(edge[0] > 180 && edge[2] < 80, "{k}×: edge {edge:?}");
        assert!(mid[2] > 180 && mid[0] < 80, "{k}×: middle {mid:?}");
    }
    assert!(!style.contains("-og-slice:"), "{style}");
    assert!(!style.contains("dress \""), "{style}");
    // The review sheet; a style with no atlas of its own keeps no base.
    assert!(pack.join("art/sheet.png").is_file());
    assert!(!pack.join("art/base.png").exists());

    // A rerun packs the same bytes.
    let before = std::fs::read(pack.join("atlas@3x.png")).unwrap();
    let out = art(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert_eq!(
        std::fs::read_to_string(pack.join("style.ogt")).unwrap(),
        style
    );
    assert_eq!(std::fs::read(pack.join("atlas@3x.png")).unwrap(), before);

    // The game cooks the densities and leaves the sources out.
    let cooked = og().arg("cook").arg(&dir).output().unwrap();
    assert_eq!(cooked.status.code(), Some(0), "{}", text(&cooked));
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    assert!(bundle.output("styles/arena/atlas@3x.png").is_ok());
    assert!(bundle.output("styles/arena/art/corner.png").is_err());

    // A request that changes leaves its art stale.
    let mut r = requests();
    for s in r["slots"].as_array_mut().unwrap() {
        if s["name"] == "arena/corner" {
            s["size"] = serde_json::json!([40, 32]);
        }
    }
    std::fs::write(pack.join("art-requests.json"), r.to_string()).unwrap();
    let out = art(&["--check"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(
        text(&out).contains("arena/corner was painted for an older request"),
        "{}",
        text(&out)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
