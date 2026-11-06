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

/// `og style new arena --look "sports broadcast" ...` writes a generated
/// version 2 style into the game (its tokens, procedural dressings, sound
/// sets with their `type sfx` presets, and the art slots it asks for),
/// with the flags over the nearest seed's choices; the game cooks with it,
/// the packs' fonts and icons it shares bundled; a bad choice is refused
/// with the names it takes.
///
/// Ways it could fail: a flag ignored, a sound the style names left
/// unwritten, or a shared font the cook leaves out of the bundle.
#[test]
fn style_new_generates_a_style_the_game_cooks() {
    let dir = look_project("styles/arena/style.ogt", false);
    let out = og()
        .args([
            "style",
            "new",
            "arena",
            "--look",
            "sports broadcast",
            "--premise",
            "a five-a-side game",
            "--shape",
            "pill",
            "--motion",
            "bouncy",
            "--project",
        ])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let style_dir = dir.join("assets/styles/arena");
    let style = std::fs::read_to_string(style_dir.join("style.ogt")).unwrap();
    let file = og_ui::parse_style(&style).unwrap_or_else(|e| panic!("{e}\n{style}"));
    assert_eq!((file.version, file.name.as_str()), (2, "arena"));
    let listed = og()
        .args(["ui", "list"])
        .arg(&dir)
        .arg("--json")
        .output()
        .unwrap();
    assert!(listed.status.success(), "{}", text(&listed));
    let extensions: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert!(
        extensions
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["manifest"]["name"] == "arena" && e["installed"] == true)
    );
    // The pill shape language and the bouncy press over broadcast's seed.
    assert!(style.contains("  button round pill\n"), "{style}");
    assert!(style.contains("  press 1.06 0.9 3\n"), "{style}");
    assert!(
        style.contains("@styles/shared/ChakraPetch-BoldItalic.ttf"),
        "{style}"
    );
    for sounds in &file.feedback_sounds {
        for s in sounds {
            assert!(style_dir.join(s).is_file(), "{s}");
        }
    }
    let requests: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(style_dir.join("art-requests.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(requests["choices"]["shape"], "pill");
    assert!(
        requests["slots"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == "arena/button-primary" && s["kind"] == "styleButton"),
        "{requests}"
    );
    let cooked = og().arg("cook").arg(&dir).output().unwrap();
    assert_eq!(cooked.status.code(), Some(0), "{}", text(&cooked));
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    assert!(
        bundle
            .output("@styles/shared/ChakraPetch-BoldItalic.ttf")
            .is_ok()
    );

    let bad = og()
        .args(["style", "new", "other", "--shape", "blob", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(1), "{}", text(&bad));
    assert!(text(&bad).contains("squircle"), "{}", text(&bad));
    let _ = std::fs::remove_dir_all(&dir);
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

/// A copy of tests/fixtures/style-pack (three nine-patches, a corner and a
/// style file) in a fresh directory.
fn style_fixture() -> PathBuf {
    let dir = scratch_root().join("handmade");
    std::fs::create_dir_all(&dir).unwrap();
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/style-pack");
    for e in std::fs::read_dir(src).unwrap() {
        let p = e.unwrap().path();
        std::fs::copy(&p, dir.join(p.file_name().unwrap())).unwrap();
    }
    dir
}

/// `og style pack` packs a style drawn by hand: each nine-patch's art lands in
/// the atlas pixel for pixel, its insets are what lies outside its stretch
/// marks, the corner is packed as drawn, and the style file keeps its own
/// lines (the palette, a slice's tint and repeat words) while its atlas,
/// slices, soft rectangle and corner region move to the new atlas; og-ui
/// parses the result. A nine-patch with no stretch marks is refused by
/// name.
///
/// Ways it could fail: the border is packed with the art, the insets are
/// off by the border, images overlap or bleed into each other, the style
/// file loses lines it does not own, or a part's region points elsewhere.
#[test]
fn style_pack_packs_nine_patches_and_rewrites_the_style_file() {
    let dir = style_fixture();
    let out = og()
        .args(["style", "pack", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let atlas = image::open(dir.join("atlas.png")).unwrap().to_rgba8();
    let style = std::fs::read_to_string(dir.join("style.ogt")).unwrap();
    let line = |key: &str, part: &str| -> Vec<String> {
        style
            .lines()
            .map(|l| l.split_whitespace().map(String::from).collect::<Vec<_>>())
            .find(|w| {
                w.first().map(String::as_str) == Some(key)
                    && (part.is_empty() || w.get(1).map(String::as_str) == Some(part))
            })
            .unwrap_or_else(|| panic!("no {key} {part} line in\n{style}"))
    };
    let num = |w: &[String], i: usize| -> u32 { w[i].parse().unwrap() };
    // Each nine-patch: its region is the art, its insets the marks'.
    for (part, insets) in [("panel", [6, 5, 6, 5]), ("button", [4, 3, 4, 3])] {
        let w = line("slice", part);
        let (x, y, rw, rh) = (num(&w, 2), num(&w, 3), num(&w, 4), num(&w, 5));
        assert_eq!(
            [num(&w, 6), num(&w, 7), num(&w, 8), num(&w, 9)],
            insets,
            "{part}'s insets"
        );
        let src = image::open(dir.join(format!("{part}.9.png")))
            .unwrap()
            .to_rgba8();
        assert_eq!(
            (rw, rh),
            (src.width() - 2, src.height() - 2),
            "{part}'s region is its art"
        );
        for sy in 0..rh {
            for sx in 0..rw {
                assert_eq!(
                    atlas.get_pixel(x + sx, y + sy),
                    src.get_pixel(sx + 1, sy + 1),
                    "{part}'s art at {sx},{sy}"
                );
            }
        }
    }
    assert_eq!(
        line("slice", "button")[10..].to_vec(),
        vec!["tint".to_string(), "repeat".to_string()],
        "the button keeps its words"
    );
    let soft = line("soft", "");
    assert_eq!(
        (num(&soft, 3), num(&soft, 4), num(&soft, 5)),
        (12, 12, 5),
        "the soft rectangle and its inset"
    );
    let corner = line("corner", "");
    let src = image::open(dir.join("corner.png")).unwrap().to_rgba8();
    let (cx, cy) = (num(&corner, 1), num(&corner, 2));
    assert_eq!(
        atlas.get_pixel(cx + 3, cy + 2),
        src.get_pixel(3, 2),
        "the corner is packed as drawn"
    );
    assert_eq!(
        corner[5..].join(" "),
        "12 12 primary 2",
        "the corner keeps its size, ink and offset"
    );
    assert!(
        style.contains("primary \"#ffb000\"") && style.contains("corners panel"),
        "the style's own lines stay:\n{style}"
    );
    assert_eq!(
        line("atlas", "")[2..].to_vec(),
        vec![atlas.width().to_string(), atlas.height().to_string()],
        "the atlas line names the packed size"
    );
    let parsed =
        og_ui::parse_style(&style).unwrap_or_else(|e| panic!("og-ui reads it: {e}\n{style}"));
    assert_eq!(parsed.name, "handmade");
    // A nine-patch without marks is refused, by name.
    let bad = image::RgbaImage::from_pixel(10, 10, image::Rgba([200, 200, 200, 255]));
    bad.save(dir.join("card.9.png")).unwrap();
    let out = og()
        .args(["style", "pack", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out).contains("card.9.png: its top border marks no column"),
        "{}",
        text(&out)
    );
}

/// `og style upgrade` writes the hand-drawn fixture's version 1 style as
/// version 2 in place, og-ui reads it with every part a dressing, and
/// `og style pack` then rewrites the slices inside those dressings (a
/// version 2 file has no top-level `slice` line to rewrite); `--check`
/// fails on a version 1 style and passes once it is upgraded.
///
/// Ways it could fail: the upgrade writes a file og-ui refuses, pack writes
/// version 1 `slice` lines into a version 2 file (which then no longer
/// loads), or a slice's tint and repeat words are lost on the way.
#[test]
fn style_upgrade_writes_version_2_and_pack_rewrites_its_dressings() {
    let dir = style_fixture();
    let style = dir.join("style.ogt");
    let check = og()
        .args(["style", "upgrade", style.to_str().unwrap(), "--check"])
        .output()
        .unwrap();
    assert_eq!(check.status.code(), Some(1), "{}", text(&check));
    assert!(text(&check).contains("is version 1"), "{}", text(&check));
    let out = og()
        .args(["style", "upgrade", style.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let up = std::fs::read_to_string(&style).unwrap();
    let parsed = og_ui::parse_style(&up).unwrap_or_else(|e| panic!("og-ui reads it: {e}\n{up}"));
    assert_eq!((parsed.version, parsed.name.as_str()), (2, "handmade"));
    assert!(up.contains("dress \"Button.base\" {"), "{up}");
    let out = og()
        .args(["style", "pack", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let packed = std::fs::read_to_string(&style).unwrap();
    let file =
        og_ui::parse_style(&packed).unwrap_or_else(|e| panic!("og-ui reads it: {e}\n{packed}"));
    assert!(!packed.lines().any(|l| l.starts_with("slice ")), "{packed}");
    let atlas = image::open(dir.join("atlas.png")).unwrap().to_rgba8();
    let button =
        file.theme.art.slices[og_ui::StylePart::Button.index()].expect("the button's slice");
    // The button's nine-patch is 4 3 4 3 inside a one-pixel border; it keeps tint and repeat.
    let w = atlas.width() as f32;
    assert_eq!((button.border_uv[0] * w).round(), 4.0);
    assert!(
        button.tint && button.fill == og_ui::SliceFill::Repeat,
        "{packed}"
    );
    let again = og()
        .args(["style", "upgrade", style.to_str().unwrap(), "--check"])
        .output()
        .unwrap();
    assert!(again.status.success(), "{}", text(&again));
}

/// A project directory under the temp dir with `files` written in it.
fn style_project(name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("og-style-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, bytes) in files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    dir
}

#[test]
fn style_check_without_a_look_says_there_is_no_contract_and_exits_0() {
    let dir = style_project(
        "nolook",
        &[
            (
                "project.ogt",
                b"type project\nversion 1\n---\nname \"x\"\nentry \"src/main.ts\"\n".to_vec(),
            ),
            (
                "src/main.ts",
                b"import { og } from '@origingame/engine'; export default og.game({});\n".to_vec(),
            ),
        ],
    );
    let out = og().args(["style", "check"]).arg(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert!(text(&out).contains("no art contract"), "{}", text(&out));
    let _ = std::fs::remove_dir_all(&dir);
}

/// A 2D project whose look holds sprites to a strict two-colour palette:
/// `on.png` is drawn on it, `off.png` (opted out with `conform off`, so the
/// cook keeps it) has a quarter of its texels 3 ΔE00 off it (grey 136
/// against the palette's 128). The check measures the cooked texels: the
/// opted-out sprite is listed, measured and off the contract.
#[test]
fn style_check_measures_cooked_sprites_and_lists_opt_outs() {
    let png = |off: bool| {
        let mut img = image::RgbaImage::new(4, 4);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = if x == 0 && off {
                image::Rgba([136, 136, 136, 255])
            } else if y < 2 {
                image::Rgba([128, 128, 128, 255])
            } else {
                image::Rgba([200, 40, 40, 255])
            };
        }
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    };
    let pixel = b"type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n".to_vec();
    let dir = style_project(
        "measure",
        &[
            (
                "project.ogt",
                b"type project\nversion 1\n---\nname \"x\"\nentry \"src/main.ts\"\nprofile \"2d\"\nlook \"looks/main.ogt\"\n".to_vec(),
            ),
            (
                "assets/looks/main.ogt",
                b"type look\nversion 1\n---\nArtPalette {\n  colors \"palettes/two.hex\"\n  strict true\n}\n".to_vec(),
            ),
            ("assets/palettes/two.hex", b"808080\nc82828\n".to_vec()),
            ("assets/on.png", png(false)),
            ("assets/on.png.ogt", pixel.clone()),
            ("assets/off.png", png(true)),
            (
                "assets/off.png.ogt",
                b"type import\nversion 1\n---\nconform off\ntexture {\n  kind pixel\n}\n".to_vec(),
            ),
        ],
    );
    let out = og()
        .args(["style", "check", "--json", "--no-frames"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "{}", text(&out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let asset = |id: &str| {
        v["assets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["asset"] == id)
            .cloned()
            .unwrap_or_else(|| panic!("{id} in {v}"))
    };
    // The contract the frames and assets were judged against, flattened, and what was captured.
    assert_eq!(v["contract"]["art"]["ArtPalette"]["strict"], true, "{v}");
    assert_eq!(
        v["contract"]["art"]["ArtPalette"]["colors"], "palettes/two.hex",
        "{v}"
    );
    assert_eq!(
        v["contract"]["palette"]["hex"],
        serde_json::json!(["#808080", "#c82828"])
    );
    assert_eq!(v["coverage"]["viewports"], serde_json::json!(["desktop"]));
    // The project declares no scenes and no screens: nothing captured, nothing uncovered.
    assert_eq!(v["coverage"]["screens"], serde_json::json!([]));
    assert_eq!(v["coverage"]["uncoveredScenes"], serde_json::json!([]));
    assert_eq!(asset("on.png")["measures"]["paletteHit"], 1.0);
    assert_eq!(asset("on.png")["ok"], true);
    assert_eq!(asset("off.png")["measures"]["paletteHit"], 0.75);
    assert_eq!(asset("off.png")["ok"], false);
    assert!(asset("off.png")["optedOut"].is_string());
    assert!(
        v["optOuts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["asset"] == "off.png"),
        "{v}"
    );
    // A bad --sizes is a usage error before anything is captured.
    let bad = og()
        .args(["style", "check", "--sizes", "desktop,watch", "--json"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(2), "{}", text(&bad));
    let _ = std::fs::remove_dir_all(&dir);
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
            .find(&format!("dress \"{sel}\" {{"))
            .unwrap_or_else(|| panic!("no dress {sel} in\n{style}"));
        let block = &style[at..];
        let slice = block
            .lines()
            .find(|l| l.trim().starts_with("slice"))
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
    assert!(
        style.contains("dress \"Button.base[primary][disabled]\""),
        "{style}"
    );
    assert!(
        !style.contains("dress \"Button.base[secondary]\""),
        "{style}"
    );
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

/// `og frame measure` as a reviewer runs it on a captured frame: a 40×20
/// PNG, a white label ground on the left with grey-119 strokes (4.48:1 by
/// WCAG's formula, by hand) beside a black right half. The region is given
/// in fractions, the ground taken from `--against`; a region off the frame
/// is refused.
///
/// Ways it could fail: fractions read as pixels, the ground taken from the
/// region when --against is given, or an unknown metric accepted.
#[test]
fn frame_measure_reads_contrast_in_a_fractional_region_against_its_ground() {
    let dir = style_project("measure-frame", &[]);
    std::fs::create_dir_all(&dir).unwrap();
    let png = dir.join("frame.png");
    let img = image::RgbImage::from_fn(40, 20, |x, y| {
        if x >= 20 {
            image::Rgb([0, 0, 0])
        } else if (4..16).contains(&y) && x % 4 == 1 {
            image::Rgb([119, 119, 119])
        } else {
            image::Rgb([255, 255, 255])
        }
    });
    img.save(&png).unwrap();
    let out = og()
        .args(["frame", "measure"])
        .arg(&png)
        .args([
            "--units",
            "fraction",
            "--region",
            "0,0.2,0.5,0.6",
            "--against",
            "0,0,0.5,0.2",
            "--metric",
            "contrast,mean",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["region"], serde_json::json!([0, 4, 20, 12]));
    assert_eq!(v["against"], serde_json::json!([0, 0, 20, 4]));
    assert_eq!(v["metrics"]["contrast"]["ratio"], 4.48);
    assert_eq!(v["metrics"]["contrast"]["ground"]["colour"], "#ffffff");
    assert!(v["metrics"].get("highpass").is_none(), "{v}");
    // The whole frame by default, and text output names the ratio.
    let all = og().args(["frame", "measure"]).arg(&png).output().unwrap();
    assert!(text(&all).contains("region 0,0 40×20 px"), "{}", text(&all));
    assert!(text(&all).contains("highpass"), "{}", text(&all));
    let off = og()
        .args(["frame", "measure"])
        .arg(&png)
        .args(["--region", "30,0,20,10"])
        .output()
        .unwrap();
    assert_eq!(off.status.code(), Some(1), "{}", text(&off));
    let bad = og()
        .args(["frame", "measure"])
        .arg(&png)
        .args(["--metric", "sharpness"])
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(2), "{}", text(&bad));
    let _ = std::fs::remove_dir_all(&dir);
}

/// A game's copy of a painted pack follows its own primary: cyber copied
/// with a blue primary cooks a recoloured atlas at each density beside the
/// style, the cooked style draws from it, and the primary button's face
/// (painted acid yellow) is blue there while the near-black panel body
/// keeps its colour; the copy left at cyber's yellow cooks no second atlas.
///
/// Ways it could fail: the cooked style still naming the painted atlas, a
/// density left yellow, the dark body tinted with the accent, or a copy
/// that changed nothing paying for a recoloured atlas.
#[test]
fn a_painted_pack_follows_the_games_primary_at_cook() {
    let cook = |primary: Option<&str>| {
        let dir = look_project("styles/mine/style.ogt", false);
        let pack = dir.join("assets/styles/mine");
        copy_dir(&repo().join("templates/styles/cyber"), &pack);
        if let Some(p) = primary {
            let style = std::fs::read_to_string(pack.join("style.ogt")).unwrap();
            let style = style.replacen("  primary \"#f5e60c\"", &format!("  primary \"{p}\""), 1);
            std::fs::write(pack.join("style.ogt"), style).unwrap();
        }
        let out = og().arg("cook").arg(&dir).output().unwrap();
        assert_eq!(out.status.code(), Some(0), "{}", text(&out));
        (
            dir.clone(),
            og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap(),
        )
    };
    let (dir, bundle) = cook(Some("#3d7eff"));
    for d in ["", "@2x", "@3x"] {
        assert!(
            bundle
                .output(&format!("styles/mine/style.ogt#atlas{d}.png"))
                .is_ok(),
            "no recoloured atlas{d}"
        );
    }
    let id = "styles/mine/style.ogt";
    let style = bundle
        .style_text(id, bundle.read(id).unwrap().as_slice())
        .unwrap();
    assert!(
        style.contains("atlas \"style.ogt#atlas.png\" 512 262 2 3"),
        "{style}"
    );
    let t = bundle
        .load_texture("styles/mine/style.ogt#atlas.png")
        .unwrap()
        .to_texture_data()
        .unwrap();
    let px = |x: u32, y: u32| {
        let i = ((y * t.width + x) * 4) as usize;
        [t.rgba[i], t.rgba[i + 1], t.rgba[i + 2]]
    };
    // Button.base[primary]: slice 146 145 157 44; Panel.frame: 1 1 96 96.
    let face = px(146 + 78, 145 + 22);
    assert!(
        face[2] > 200 && face[0] < 120 && face[1] < 170,
        "face {face:?}"
    );
    let body = px(1 + 48, 1 + 48);
    assert!(body.iter().all(|&c| c < 40), "body {body:?}");
    let _ = std::fs::remove_dir_all(&dir);

    let (dir, bundle) = cook(None);
    assert!(bundle.output("styles/mine/style.ogt#atlas.png").is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else {
            std::fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}
