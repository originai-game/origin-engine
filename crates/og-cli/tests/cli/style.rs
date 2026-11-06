use super::*;

/// A fresh project whose one look names the UI style `ui`, optionally with a
/// Chinese string table.
fn look_project(ui: &str, chinese: bool) -> PathBuf {
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

/// `og style eject scifi keeper` writes the built-in style's whole pack
/// (templates/styles/scifi, byte for byte) into the game's
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
    let mine = look_project("styles/keeper/style.ogt", true);
    let out = og()
        .args(["style", "eject", "scifi", "keeper", "--project"])
        .arg(&mine)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert!(
        text(&out).contains("ui \"styles/keeper/style.ogt\""),
        "{}",
        text(&out)
    );
    let template = repo().join("templates/styles/scifi");
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
            "{f} differs from templates/styles/scifi"
        );
    }

    // An existing pack is kept unless --force.
    std::fs::write(ejected.join("style.ogt"), "edited").unwrap();
    let again = og()
        .args(["style", "eject", "scifi", "keeper", "--project"])
        .arg(&mine)
        .output()
        .unwrap();
    assert_eq!(again.status.code(), Some(1), "{}", text(&again));
    assert!(text(&again).contains("--force"), "{}", text(&again));
    assert_eq!(
        std::fs::read_to_string(ejected.join("style.ogt")).unwrap(),
        "edited"
    );
    let forced = og()
        .args(["style", "eject", "scifi", "keeper", "--force", "--project"])
        .arg(&mine)
        .output()
        .unwrap();
    assert_eq!(forced.status.code(), Some(0), "{}", text(&forced));

    let builtin = look_project("scifi", true);
    for dir in [&mine, &builtin] {
        let out = og().arg("cook").arg(dir).output().unwrap();
        assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    }
    let a = og_cook::runtime::Bundle::open(&mine.join("cooked")).unwrap();
    let b = og_cook::runtime::Bundle::open(&builtin.join("cooked")).unwrap();
    assert!(
        a.output("@styles/scifi/style.ogt").is_err(),
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
    for f in [
        "style.ogt",
        "atlas.png",
        "icons.png",
        "IBMPlexSans-Regular.ttf",
        "IBMPlexSans-Light.ttf",
        "IBMPlexMono-Regular.ttf",
    ] {
        assert!(
            unstamped(&a, &format!("styles/keeper/{f}"))
                == unstamped(&b, &format!("@styles/scifi/{f}")),
            "{f}: the ejected copy cooks differently from the built-in"
        );
    }
    // scifi names one CJK face for body and display alike.
    let id = "@styles/cjk/NotoSansSC-Regular.ttf";
    assert!(
        unstamped(&a, id) == unstamped(&b, id),
        "{id}: the ejected style's Chinese companion"
    );
    let _ = std::fs::remove_dir_all(&mine);
    let _ = std::fs::remove_dir_all(&builtin);
}

/// `og style art` takes a generated style from its requests to painted
/// atlases: it completes `art-requests.json` (each slot's file, cell size,
/// insets, states, dressings and hash), refuses art that arrives without
/// provenance, packs what arrived at 1×, 2× and 3× (the panel trimmed to its
/// art and drawn edge to edge in its cell, the primary button delivered as
/// one cell given its other states), points each dressing at its piece
/// with the art's tone, reruns to the same bytes, reports a slot whose
/// request changed as stale, and the game cooks the denser atlases but
/// never the art sources.
///
/// Ways it could fail: the transparent margin packed with the art, the
/// denser atlases laid out differently from the 1× one (a region then cuts
/// the wrong texels there), the primary states written to the plain
/// button's dressings, a rerun stacking a second copy below the first, or
/// the cook bundling `art/`.
#[test]
fn style_art_packs_painted_slots_at_three_densities() {
    let dir = look_project("styles/arena/style.ogt", false);
    let out = og()
        .args(["style", "new", "arena", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let pack = dir.join("assets/styles/arena");
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
    assert_eq!(primary["file"], "art/button-primary.png");
    assert_eq!(primary["states"].as_array().unwrap().len(), 5);
    assert_eq!(
        primary["dress"][1], "Button.base[primary][hover]",
        "{primary}"
    );
    let panel = slot("arena/panel");
    assert_eq!(panel["size"], serde_json::json!([96, 96]));
    assert_eq!(panel["pixels"], serde_json::json!([288, 288]));
    assert!(
        !requests()["slots"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == "arena/bar")
    );
    let mut custom = requests();
    custom["slots"].as_array_mut().unwrap().extend([
        serde_json::json!({"name": "arena/bar"}),
        serde_json::json!({
            "name": "arena/selected-tab", "kind": "styleFrame", "size": [80, 32],
            "insets": [0.1, 0.25, 0.1, 0.25], "dress": ["Tabs.tab[selected]"]
        }),
    ]);
    std::fs::write(pack.join("art-requests.json"), custom.to_string()).unwrap();
    let out = art(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let bar = slot("arena/bar");
    let tab = slot("arena/selected-tab");
    assert_eq!(bar["dress"], serde_json::json!(["Meter.fill"]));
    assert_eq!(tab["pixels"], serde_json::json!([240, 96]));

    // The panel: a red 18-px border round a blue middle, in a transparent
    // 20-px margin; the primary button one plain cell.
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
    img.save(art_dir.join("panel.png")).unwrap();
    image::RgbaImage::from_pixel(288, 132, image::Rgba([200, 160, 40, 255]))
        .save(art_dir.join("button-primary.png"))
        .unwrap();
    image::RgbaImage::from_pixel(192, 48, image::Rgba([220, 220, 220, 255]))
        .save(art_dir.join("bar.png"))
        .unwrap();
    image::RgbaImage::from_pixel(240, 96, image::Rgba([40, 120, 200, 255]))
        .save(art_dir.join("selected-tab.png"))
        .unwrap();
    let out = art(&[]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(
        text(&out).contains("no entry for arena/panel"),
        "{}",
        text(&out)
    );
    let provenance = serde_json::json!({
        "arena/panel": { "model": "hand", "prompt": "a test panel", "request": panel["hash"] },
        "arena/button-primary": { "model": "hand", "prompt": "a test button", "request": primary["hash"] },
        "arena/bar": { "model": "hand", "prompt": "a tinted bar", "request": bar["hash"] },
        "arena/selected-tab": { "model": "hand", "prompt": "a selected tab", "request": tab["hash"] },
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
    // The dressing's slice: its region and insets in CSS px, authored.
    let dress = |sel: &str| -> Vec<String> {
        let at = style
            .find(&format!(
                "{} {{",
                og_ui::style_file::css::selector_css(&og_ui::Selector::parse(sel).unwrap())
            ))
            .unwrap_or_else(|| panic!("no dress {sel} in\n{style}"));
        let block = &style[at..];
        let slice = block
            .lines()
            .find(|l| l.trim().starts_with("-og-slice:"))
            .unwrap();
        slice.split_whitespace().map(String::from).collect()
    };
    assert_eq!(
        &dress("Meter.fill")[3..],
        &["64", "16", "12", "4", "12", "4", "tint"]
    );
    assert_eq!(
        &dress("Tabs.tab[selected]")[3..],
        &["80", "32", "8", "8", "8", "8", "authored"]
    );
    let s = dress("Panel.frame");
    let n = |i: usize| -> u32 { s[i].parse().unwrap() };
    assert_eq!(
        (n(3), n(4), &s[5..10]),
        (
            96,
            96,
            &["24", "24", "24", "24", "authored"].map(String::from)[..]
        ),
        "{s:?}"
    );
    let (x, y) = (n(1), n(2));
    // Trimmed: the border's red at the cell's very edge, at every density,
    // and the blue middle at its centre.
    for (k, a) in atlases.iter().enumerate() {
        let k = k as u32 + 1;
        let edge = a.get_pixel(x * k + 1, (y + 48) * k);
        let mid = a.get_pixel((x + 48) * k, (y + 48) * k);
        assert!(edge[0] > 180 && edge[2] < 80, "{k}×: edge {edge:?}");
        assert!(mid[2] > 180 && mid[0] < 80, "{k}×: middle {mid:?}");
    }
    assert!(style.contains("Button::base.primary:disabled {"), "{style}");
    assert!(!style.contains("Button::base.secondary {"), "{style}");
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
    assert!(bundle.output("styles/arena/art/panel.png").is_err());

    // A request that changes leaves its art stale.
    let mut r = requests();
    for s in r["slots"].as_array_mut().unwrap() {
        if s["name"] == "arena/panel" {
            s["size"] = serde_json::json!([120, 96]);
        }
    }
    std::fs::write(pack.join("art-requests.json"), r.to_string()).unwrap();
    let out = art(&["--check"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(
        text(&out).contains("arena/panel was painted for an older request"),
        "{}",
        text(&out)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
