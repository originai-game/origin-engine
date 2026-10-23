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

mod support;

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
        (Some(9), Some(10.0))
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
    let style = &asset(&json, "@styles/paper/style.ogt")["style"];
    assert_eq!(style["name"], "paper");
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
