//! Flipbook sheets through the cook and back, and pixel-art textures.
//!
//! Ways this could fail: the texture's size is not filled in (every uv is
//! wrong); Aseprite hash frames come in string order ("10" before "2");
//! a tag's direction is ignored; TexturePacker pivots keep their top-left
//! origin; the sheet's own clip loses to a tag; editing the atlas leaves a
//! stale blob; a pixel texture is compressed or mipmapped.

use og_anim::{FlipMode, FlipbookSheet};
use og_cook::runtime::Bundle;
use og_cook::{cook_project, CookOptions};
use std::path::PathBuf;

fn project(name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-flipbook")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

/// A 64x32 checker of 2x2-texel squares in four colours.
fn png() -> Vec<u8> {
    let img = image::RgbaImage::from_fn(64, 32, |x, y| {
        let c = [
            [255, 0, 0, 255],
            [0, 255, 0, 255],
            [0, 0, 255, 255],
            [255, 255, 0, 255],
        ];
        image::Rgba(c[((x / 2 + y / 2) % 4) as usize])
    });
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// Aseprite's hash export: twelve 16x16 frames, numbered in their names.
fn aseprite() -> Vec<u8> {
    let frames: Vec<String> = (0..12)
        .map(|i| {
            format!(
                r#""hero {i}.aseprite": {{ "frame": {{ "x": {}, "y": {}, "w": 16, "h": 16 }}, "rotated": false, "trimmed": false, "duration": {} }}"#,
                (i % 4) * 16,
                (i / 4) * 16 % 32,
                if i == 3 { 200 } else { 100 }
            )
        })
        .collect();
    format!(
        r#"{{ "frames": {{ {} }}, "meta": {{ "image": "hero.png", "size": {{ "w": 64, "h": 32 }}, "frameTags": [
            {{ "name": "walk", "from": 0, "to": 3, "direction": "forward" }},
            {{ "name": "back", "from": 4, "to": 6, "direction": "reverse" }},
            {{ "name": "bob", "from": 8, "to": 11, "direction": "pingpong" }} ] }} }}"#,
        frames.join(",")
    )
    .into_bytes()
}

const SHEET: &str = r#"type flipbook
version 1
---
texture "hero.png"
atlas "hero.json"
pivot 0.5 0
clip bob {
  frames 8 9
  fps 4
  mode once
  event 1 "land"
}
"#;

const PACKED: &str = r#"{ "frames": [
  { "filename": "coin_1", "frame": { "x": 0, "y": 0, "w": 8, "h": 8 }, "rotated": false, "pivot": { "x": 0.5, "y": 1 } },
  { "filename": "coin_2", "frame": { "x": 8, "y": 0, "w": 8, "h": 8 }, "rotated": false, "pivot": { "x": 0.5, "y": 1 } }
], "meta": { "image": "hero.png" } }"#;

const COINS: &str = r#"type flipbook
version 1
---
texture "../sprites/hero.png"
atlas "coins.json"
clip spin {
  frames coin_1 coin_2
  fps 8
}
"#;

fn cooked(root: &std::path::Path) -> Bundle {
    let report = cook_project(root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    Bundle::open(&root.join("cooked")).unwrap()
}

#[test]
fn aseprite_and_texturepacker_atlases_cook_into_sheets() {
    let root = project(
        "atlas",
        &[
            ("sprites/hero.png", png()),
            (
                "sprites/hero.png.ogt",
                b"type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n".to_vec(),
            ),
            ("sprites/hero.json", aseprite()),
            ("sprites/hero.ogt", SHEET.as_bytes().to_vec()),
            ("fx/coins.json", PACKED.as_bytes().to_vec()),
            ("fx/coins.ogt", COINS.as_bytes().to_vec()),
        ],
    );
    let bundle = cooked(&root);
    let hero: FlipbookSheet = bundle.load_flipbook("sprites/hero.ogt").unwrap();
    assert_eq!(hero.texture, "sprites/hero.png");
    assert_eq!(hero.size, [64, 32], "the texture's size, from the image");
    assert_eq!(hero.frames.len(), 12);
    assert_eq!(
        hero.frames[10].name, "hero 10.aseprite",
        "natural order, not string order"
    );
    assert_eq!(hero.frames[10].rect, [32, 0, 16, 16]);
    assert_eq!(
        hero.frames[0].pivot,
        [0.5, 0.0],
        "the sheet's pivot for atlas frames"
    );
    let walk = &hero.clips[hero.clip("walk").unwrap() as usize];
    assert_eq!(walk.frames, vec![0, 1, 2, 3]);
    assert_eq!(
        walk.durations,
        vec![0.1, 0.1, 0.1, 0.2],
        "Aseprite's per-frame durations"
    );
    let back = &hero.clips[hero.clip("back").unwrap() as usize];
    assert_eq!(back.frames, vec![6, 5, 4], "a reverse tag plays backwards");
    // The sheet's own `bob` wins over the tag of that name.
    let bob = &hero.clips[hero.clip("bob").unwrap() as usize];
    assert_eq!((bob.frames.clone(), bob.mode), (vec![8, 9], FlipMode::Once));
    assert_eq!(bob.events, vec![(1, "land".to_string())]);
    assert_eq!(hero.clips.len(), 3);

    let coins = bundle.load_flipbook("fx/coins.ogt").unwrap();
    assert_eq!(coins.texture, "sprites/hero.png", "relative to the sheet");
    assert_eq!(
        coins.frames[1].pivot,
        [0.5, 0.0],
        "TexturePacker's top-left pivot, from the bottom left"
    );
    assert_eq!(coins.clips[0].frames, vec![0, 1]);

    // The pixel texture: lossless RGBA8, one level, nearest at runtime.
    let tex = bundle.load_texture("sprites/hero.png").unwrap();
    assert!(tex.is_pixel_art());
    assert_eq!(
        (tex.meta.codec, tex.meta.levels),
        (og_cook::format::codec::RGBA8, 1),
        "lossless, one level"
    );
    let data = tex.to_texture_data().unwrap();
    assert_eq!(data.mips, og_render::Mips::None);
    assert_eq!(
        &data.rgba[..8],
        &[255, 0, 0, 255, 255, 0, 0, 255],
        "exact texels"
    );
    assert_eq!(&data.rgba[8..12], &[0, 255, 0, 255]);

    // Editing the atlas recooks the sheet.
    std::fs::write(
        root.join("assets/sprites/hero.json"),
        aseprite().replace_durations(),
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let sheet = report
        .assets
        .iter()
        .find(|a| a.source == "sprites/hero.ogt")
        .unwrap();
    assert!(sheet.outputs[0].cooked, "the atlas changed");
    let hero = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_flipbook("sprites/hero.ogt")
        .unwrap();
    assert_eq!(hero.clips[0].durations[3], 0.3);
}

trait Durations {
    fn replace_durations(self) -> Vec<u8>;
}
impl Durations for Vec<u8> {
    fn replace_durations(self) -> Vec<u8> {
        String::from_utf8(self)
            .unwrap()
            .replace("\"duration\": 200", "\"duration\": 300")
            .into_bytes()
    }
}

#[test]
fn sheet_mistakes_are_refused_by_name() {
    for (sheet, says) in [
        (
            SHEET.replace("hero.json", "missing.json"),
            "atlas `missing.json` is not in assets/",
        ),
        (
            SHEET.replace("texture \"hero.png\"", "texture \"nope.png\""),
            "texture `nope.png` is not in assets/",
        ),
        (SHEET.replace("frames 8 9", "frames 8 40"), "no frame `40`"),
    ] {
        let root = project(
            "refused",
            &[
                ("sprites/hero.png", png()),
                ("sprites/hero.json", aseprite()),
                ("sprites/hero.ogt", sheet.into_bytes()),
            ],
        );
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        let errors = format!("{:?}", report.errors());
        assert!(errors.contains(says), "{errors}");
    }
}
