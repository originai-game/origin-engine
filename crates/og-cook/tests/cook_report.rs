//! `og cook --report` for every asset type (`docs/design.md` §3.4): each
//! output carries its own figures, read back from `cooked/`, and the
//! budgets that make sense for them flag it.
//!
//! Ways this could fail: a sound, font, navmesh, scene, timeline, string
//! table, behaviour or style reports its bytes only (the report before
//! 2026-10-23); a figure is read from the wrong field (seconds from bytes,
//! keys from the text's lines); a budget is parsed but never checked.

use og_cook::cook::stats::{Budgets, cook_report_json};
use og_cook::cook::{CookOptions, cook_project};
use std::path::PathBuf;

use crate::support;

fn asset<'a>(report: &'a serde_json::Value, source: &str) -> &'a serde_json::Value {
    report["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["source"] == source)
        .unwrap_or_else(|| panic!("no {source} in the report"))
}

#[test]
fn every_asset_type_reports_its_figures_and_its_budgets_flag_it() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-report");
    let _ = std::fs::remove_dir_all(&root);
    support::stage_template(&root.join("assets"));
    std::fs::create_dir_all(root.join("assets/core-test")).unwrap();
    std::fs::write(
        root.join("assets/core-test/main.css"),
        ":root { --example: #123456; }",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let cooked = root.join("cooked");
    let json = cook_report_json(&report, &cooked, &Budgets::default());

    // blip.wav: its header says 44100 data bytes of 16-bit mono at 44100 Hz.
    let wav = std::fs::read(root.join("assets/audio/blip.wav")).unwrap();
    let u16_at = |i: usize| u16::from_le_bytes([wav[i], wav[i + 1]]) as u64;
    let u32_at = |i: usize| u32::from_le_bytes(wav[i..i + 4].try_into().unwrap()) as u64;
    let (channels, rate, data) = (u16_at(22), u32_at(24), u32_at(40));
    let frames = data / (channels * 2);
    let s = &asset(&json, "audio/blip.wav")["sound"];
    assert_eq!(s["channels"], channels);
    assert_eq!(s["sampleRate"], rate);
    assert_eq!(s["frames"], frames);
    assert_eq!(s["seconds"], frames as f64 / rate as f64);
    assert_eq!(s["chunks"], 0);

    // strings/en.ogt holds five `string` lines; intro.ogt nine tracks (the
    // music sting among them) over `duration 10`; guard.ogt's tree is a
    // patrol under one sequence.
    let st = &asset(&json, "strings/en.ogt")["strings"];
    assert_eq!(
        (st["locale"].as_str(), st["keys"].as_u64()),
        (Some("en"), Some(5))
    );
    let tl = &asset(&json, "timelines/intro.ogt")["timeline"];
    assert_eq!(
        (tl["tracks"].as_u64(), tl["seconds"].as_f64()),
        (Some(9), Some(10.0)),
        "{}",
        asset(&json, "timelines/intro.ogt")
    );
    assert!(asset(&json, "ai/guard.ogt")["behaviour"]["nodes"].as_u64() > Some(0));

    // The scene's four entities, two of them model instances.
    let sc = &asset(&json, "scenes/main.ogt")["scene"];
    assert_eq!(
        (sc["entities"].as_u64(), sc["instances"].as_u64()),
        (Some(4), Some(2))
    );

    let font = &asset(&json, "fonts/NotoSansSC-Regular-subset.ttf")["font"];
    let glyphs = font["glyphs"].as_u64().unwrap();
    assert!(glyphs > 0 && font["pages"].as_u64() > Some(0), "{font}");
    let nav = &asset(&json, "nav/yard.ogt")["navmesh"];
    let polygons = nav["polygons"].as_u64().unwrap();
    assert!(
        polygons > 0 && nav["vertices"].as_u64() >= Some(polygons * 3 / 2),
        "{nav}"
    );
    let style = &asset(&json, "core-test")["style"];
    assert_eq!(style["name"], "core-test");
    for a in json["assets"].as_array().unwrap() {
        assert!(a.get("unread").is_none(), "{a}");
        assert_eq!(a["flags"], serde_json::json!([]), "{a}");
    }

    // Budgets one under each figure flag exactly those assets.
    let tight = Budgets {
        sound_seconds: 0,
        font_glyphs: glyphs - 1,
        nav_polygons: polygons - 1,
        scene_entities: 3,
        ..Budgets::default()
    };
    let json = cook_report_json(&report, &cooked, &tight);
    let flags = |source: &str| -> Vec<(String, u64)> {
        asset(&json, source)["flags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                (
                    f["budget"].as_str().unwrap().to_string(),
                    f["value"].as_u64().unwrap(),
                )
            })
            .collect()
    };
    assert_eq!(flags("audio/blip.wav"), [("soundSeconds".to_string(), 1)]);
    assert_eq!(
        flags("fonts/NotoSansSC-Regular-subset.ttf"),
        [("fontGlyphs".to_string(), glyphs)]
    );
    assert_eq!(
        flags("nav/yard.ogt"),
        [("navPolygons".to_string(), polygons)]
    );
    assert_eq!(flags("scenes/main.ogt"), [("sceneEntities".to_string(), 4)]);
}

#[test]
fn css_pack_cooks_svg_images_as_regular_texture_assets() {
    let builtin = std::env::var_os("OG_TEST_BUILTIN_CSS").is_some();
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(if builtin {
        "og-cook-css-svg-builtin"
    } else {
        "og-cook-css-svg"
    });
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join(if builtin {
        "styles/example"
    } else {
        "assets/example"
    });
    let id = if builtin {
        "@styles/example"
    } else {
        "example"
    };
    std::fs::create_dir_all(dir.join("art")).unwrap();
    if builtin {
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(
            root.join("assets/look.ogt"),
            "type look\nversion 1\n---\nui \"@styles/example\"\n",
        )
        .unwrap();
    }
    std::fs::write(
        dir.join("main.css"),
        "button::part(body) { mask-image: url(art/icon.svg); }",
    )
    .unwrap();
    std::fs::write(dir.join("art/icon.svg"), r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="16"><rect width="16" height="16" fill="#ff0000" fill-opacity="0.5"/></svg>"##).unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let text = bundle.style_text(id, &bundle.read(id).unwrap()).unwrap();
    let pack =
        og_ui::style_pack::StylePack::parse(id, serde_json::from_str(&text).unwrap()).unwrap();
    assert_eq!(pack.assets, [format!("{id}/art/icon.svg")]);
    let atlas = bundle
        .load_texture(&format!("{id}/art/icon.svg"))
        .unwrap()
        .to_texture_data()
        .unwrap();
    assert_eq!((atlas.width, atlas.height), (32, 16));
    let pixel = |x: usize, y: usize| &atlas.rgba[(y * 32 + x) * 4..][..4];
    let painted = pixel(4, 4);
    assert!(
        painted[0] > 230 && painted[1] < 25 && (115..=140).contains(&painted[3]),
        "straight alpha: {painted:?}"
    );
    assert!(pixel(28, 4)[3] < 10, "outside the SVG shape is transparent");
    if !builtin {
        // Built-in tables are process-wide; isolate their directory from
        // other cooks instead of changing the test process's environment.
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "cook_report::css_pack_cooks_svg_images_as_regular_texture_assets",
                "--nocapture",
            ])
            .env("OG_TEST_BUILTIN_CSS", "1")
            .env(
                "OG_STYLES_DIR",
                root.with_file_name("og-cook-css-svg-builtin")
                    .join("styles"),
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Each stage of the cook is timed in the order it ran, and a cook the
/// index finds unchanged runs, and lists, only `index`.
///
/// Ways this could fail: a stage is left out of the list or timed twice; the
/// unchanged cook returns the stages its index saved from the cook before.
#[test]
fn the_report_times_each_stage_and_an_unchanged_cook_only_the_index() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-stages");
    let _ = std::fs::remove_dir_all(&root);
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/sprite-2d");
    support::copy_dir(&fixture.join("assets"), &root.join("assets"));
    let cook = || {
        let r =
            og_cook::cook::cook_project_with_texture_cache(&root, &CookOptions::default(), None)
                .unwrap();
        assert!(r.ok(), "{:?}", r.errors());
        assert!(r.stages.iter().all(|s| s.ms.is_finite() && s.ms >= 0.0));
        r.stages.into_iter().map(|s| s.name).collect::<Vec<_>>()
    };
    assert_eq!(
        cook(),
        [
            "index", "project", "scan", "import", "resolve", "contract", "group", "plan",
            "execute", "assemble", "write", "pack", "finish", "diagnose", "save"
        ]
    );
    assert_eq!(cook(), ["index"]);
}
