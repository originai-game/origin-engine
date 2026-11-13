//! Pixel effects (`type vfx2d`): Pixel VFX Forge recipes cooked into
//! flipbook sheets.
//!
//! Ways this could fail: the port drifts from the Forge's renderer (a
//! changed operation order, host trig, heat kept in f64 instead of the
//! Forge's f32), so the library's frames are no longer the Forge's; the
//! impact event lands on another frame than the Forge reports; a locked
//! palette lets other colours through; the override lines are ignored or
//! rounded to the wrong frame; editing the recipe file leaves a stale sheet.

use og_anim::FlipMode;
use og_anim::cooked::BundleAnim;
use og_cook::cook::vfx2d;
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

/// Two recipes of the Forge's library (`interpret(prompt, seed).recipe`,
/// as the Forge saves them), ten layer types between them.
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vfx2d")
}

fn project(name: &str, files: &[(&str, String)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-vfx2d")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    root
}

fn cooked(root: &Path) -> Bundle {
    let report = cook_project(root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    Bundle::open(&root.join("cooked")).unwrap()
}

/// Library recipes render the Forge's frames: the FNV-1a of the RGBA that
/// Node gives running the Forge's engine.js (V8 14.6) on them, in file-name
/// order (level-up: rays, pillar, ring, particles, text; meteor-strike:
/// comet, puffs, cracks, burst, ring, particles, flash).
#[test]
fn library_recipes_render_the_forges_frames() {
    let mut fnv: u64 = 0xcbf2_9ce4_8422_2325;
    let mut texels = 0;
    for name in ["level-up", "meteor-strike"] {
        let text = std::fs::read_to_string(fixtures().join(format!("{name}.json"))).unwrap();
        for f in vfx2d::render_json(&text).unwrap().frames {
            texels += f.len() / 4;
            for b in f {
                fnv = (fnv ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
            }
        }
    }
    assert_eq!(texels, 362_496);
    assert_eq!(
        format!("{fnv:016x}"),
        "db3e83dceb9cbd67",
        "the frames are no longer the Forge's: render the recipes with its engine.js and compare"
    );
}

#[test]
fn a_recipe_cooks_into_a_sheet_with_its_impact() {
    let recipe = std::fs::read_to_string(fixtures().join("meteor-strike.json")).unwrap();
    let locked = r#"{ "w": 32, "h": 32, "frames": 8, "fps": 12, "loop": true, "palette": "gameboy",
      "outline": "outside", "dither": 0.5,
      "layers": [ { "type": "orb", "ramp": "hue:200" }, { "type": "particles", "mode": "orbit", "count": 6 } ] }"#;
    let files = [
        ("fx/meteor.json", recipe.clone()),
        (
            "fx/meteor.ogt",
            "type vfx2d\nversion 1\n---\nrecipe \"meteor.json\"\n".to_string(),
        ),
        (
            "fx/meteor-timed.ogt",
            "type vfx2d\nversion 1\n---\nduration 1.3\nloop true\nimpact 0.11\nevent 0.5 \"dust\"\nrecipe \"fx/meteor.json\"\n"
                .to_string(),
        ),
        (
            "fx/orb.ogt",
            format!("type vfx2d\nversion 1\n---\n# inline\n{locked}\n"),
        ),
    ];
    let root = project("cook", &files);
    let bundle = cooked(&root);

    // The recipe's own: 26 frames at 15 fps, once, the Forge's impact frame 9.
    let sheet = bundle.load_flipbook("fx/meteor.ogt").unwrap();
    assert_eq!(sheet.texture, "fx/meteor.ogt#sheet");
    assert_eq!(sheet.frames.len(), 26);
    assert_eq!(sheet.size, [26 * 96, 96], "the Forge's sheet: one row");
    let clip = &sheet.clips[sheet.clip("meteor").unwrap() as usize];
    assert_eq!(clip.mode, FlipMode::Once);
    assert_eq!(clip.durations, vec![1.0 / 15.0; 26]);
    assert_eq!(clip.events, vec![(9, "impact".to_string())]);
    let tex = bundle.load_texture("fx/meteor.ogt#sheet").unwrap();
    assert!(tex.is_pixel_art());
    let data = tex.to_texture_data().unwrap();
    let r = vfx2d::render_json(&recipe).unwrap();
    for (i, f) in sheet.frames.iter().enumerate() {
        assert_eq!(f.rect, [i as u32 * 96, 0, 96, 96]);
        let row = |y: u32| {
            let s = ((y * data.width + f.rect[0]) * 4) as usize;
            &data.rgba[s..s + 96 * 4]
        };
        assert!(
            (0..96).all(|y| row(y) == &r.frames[i][(y * 96 * 4) as usize..((y + 1) * 96 * 4) as usize]),
            "frame {i} is the rendered frame"
        );
    }

    // The override lines: 1.3 s over 26 frames (0.05 s each), the impact on
    // the frame that starts at or after 0.11 s, the event at 0.5 s on frame 10.
    let timed = bundle.load_flipbook("fx/meteor-timed.ogt").unwrap();
    let clip = &timed.clips[0];
    assert_eq!(clip.mode, FlipMode::Loop);
    assert!(clip.durations.iter().all(|d| (d - 0.05).abs() < 1e-6));
    assert_eq!(
        clip.events,
        vec![(3, "impact".to_string()), (10, "dust".to_string())]
    );

    // Locked to the Game Boy's four colours, outline and dither included.
    let orb = bundle.load_flipbook("fx/orb.ogt").unwrap();
    assert_eq!(orb.clips[0].mode, FlipMode::Loop);
    assert_eq!(orb.clips[0].durations, vec![1.0 / 12.0; 8]);
    let data = bundle
        .load_texture("fx/orb.ogt#sheet")
        .unwrap()
        .to_texture_data()
        .unwrap();
    let gameboy = [
        [0x0f, 0x38, 0x0f],
        [0x30, 0x62, 0x30],
        [0x8b, 0xac, 0x0f],
        [0x9b, 0xbc, 0x0f],
    ];
    let mut seen = std::collections::BTreeSet::new();
    for p in data.rgba.chunks(4) {
        match p[3] {
            0 => {}
            255 => {
                assert!(
                    gameboy.contains(&[p[0], p[1], p[2]]),
                    "{p:?} is not in the palette"
                );
                seen.insert([p[0], p[1], p[2]]);
            }
            a => panic!("alpha {a}: pixel art is opaque or clear"),
        }
    }
    assert!(seen.len() >= 2, "{seen:?}");

    // Determinism: a second render is the same bytes; an edit to the recipe
    // file cooks the sheet again.
    assert_eq!(vfx2d::render_json(&recipe).unwrap(), r);
    std::fs::write(
        root.join("assets/fx/meteor.json"),
        recipe.replace("\"frames\": 26", "\"frames\": 20"),
    )
    .unwrap();
    let bundle = cooked(&root);
    assert_eq!(
        bundle.load_flipbook("fx/meteor.ogt").unwrap().frames.len(),
        20
    );
}

#[test]
fn a_bad_effect_is_its_assets_error() {
    let root = project(
        "errors",
        &[
            (
                "fx/none.ogt",
                "type vfx2d\nversion 1\n---\nimpact 0.1\n".to_string(),
            ),
            (
                "fx/late.ogt",
                "type vfx2d\nversion 1\n---\nevent 9 \"late\"\n{ \"layers\": [ { \"type\": \"flash\" } ] }\n"
                    .to_string(),
            ),
            (
                "fx/missing.ogt",
                "type vfx2d\nversion 1\n---\nrecipe \"gone.json\"\n".to_string(),
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(errors.contains("no recipe"), "{errors}");
    assert!(errors.contains("after the duration"), "{errors}");
    assert!(errors.contains("gone.json"), "{errors}");
}
