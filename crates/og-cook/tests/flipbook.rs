//! Flipbook sheets through the cook and back, and pixel-art textures.
//!
//! Ways this could fail: the texture's size is not filled in (every uv is
//! wrong); Aseprite hash frames come in string order ("10" before "2");
//! a tag's direction is ignored; TexturePacker pivots keep their top-left
//! origin; the sheet's own clip loses to a tag; editing the atlas leaves a
//! stale blob; a pixel texture is compressed or mipmapped.

use og_anim::{FlipMode, FlipbookSheet};
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
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

/// `array true`: the frames, cut out of the texture, are the layers of
/// `<sheet>#array` (pixel art under `filter nearest`), the sheet keeps its
/// frames and clips; a sheet whose frames differ in size is refused.
/// Broken once by cutting each layer from the texture's first rows (every
/// layer is frame 0's row): layer 5's texels are frame 1's.
#[test]
fn an_array_sheet_cooks_its_frames_into_a_texture_array() {
    // Each 16x16 cell its own colour, shaded across x and y.
    let cells = image::RgbaImage::from_fn(64, 32, |x, y| {
        let n = (x / 16 + 4 * (y / 16)) as u8;
        image::Rgba([n * 30, (x % 16) as u8 * 16, (y % 16) as u8 * 16, 255])
    });
    let mut cells_png = Vec::new();
    cells
        .write_to(
            &mut std::io::Cursor::new(&mut cells_png),
            image::ImageFormat::Png,
        )
        .unwrap();
    const ARRAY: &str = "type flipbook\nversion 1\n---\ntexture \"hero.png\"\narray true\ngrid 16 16\nclip run {\n  frames 0 1 2 3 4 5 6 7\n  fps 8\n}\n";
    let root = project(
        "array",
        &[
            ("sprites/hero.png", cells_png.clone()),
            ("sprites/run.ogt", ARRAY.as_bytes().to_vec()),
        ],
    );
    let bundle = cooked(&root);
    let sheet = bundle.load_flipbook("sprites/run.ogt").unwrap();
    assert!(sheet.array);
    assert_eq!(sheet.frames.len(), 8);
    let array = bundle.load_texture_array("sprites/run.ogt#array").unwrap();
    assert_eq!(
        (array.meta.width, array.meta.height, array.meta.layers),
        (16, 16, 8)
    );
    assert_eq!(
        array.meta.kind,
        og_cook::format::kind::PIXEL,
        "filter nearest"
    );
    assert_eq!(array.layers[5], "sprites/run.ogt#array#layer5");
    // Layer n is frame n: 16x16 at (16 (n % 4), 16 (n / 4)) of the texture.
    let source = cells;
    for n in [0usize, 5, 7] {
        let layer = bundle
            .load_texture(&array.layers[n])
            .unwrap()
            .to_texture_data()
            .unwrap();
        assert_eq!((layer.width, layer.height), (16, 16));
        let (fx, fy) = (16 * (n as u32 % 4), 16 * (n as u32 / 4));
        for (x, y) in [(0u32, 0u32), (3, 1), (15, 15)] {
            let i = ((y * 16 + x) * 4) as usize;
            assert_eq!(
                &layer.rgba[i..i + 4],
                &source.get_pixel(fx + x, fy + y).0,
                "layer {n} at {x},{y}"
            );
        }
    }
    // Frames of two sizes cannot be one array's layers.
    let root = project(
        "array-sizes",
        &[
            ("sprites/hero.png", png()),
            (
                "sprites/run.ogt",
                ARRAY
                    .replace("grid 16 16\n", "frame a 0 0 16 16\nframe b 16 0 8 8\n")
                    .replace("frames 0 1 2 3 4 5 6 7", "frames a b")
                    .into_bytes(),
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(errors.contains("frames are one size"), "{errors}");
}

/// A 24x20 canvas over `bg`: a 4x6 body of `body` with a 2x2 head on top,
/// its top left at (x, y).
fn figure(x: u32, y: u32, body: [u8; 3], bg: [u8; 4]) -> image::RgbaImage {
    image::RgbaImage::from_fn(24, 20, |px, py| {
        let in_body = (x..x + 4).contains(&px) && (y + 2..y + 8).contains(&py);
        let in_head = (x + 1..x + 3).contains(&px) && (y..y + 2).contains(&py);
        if in_body {
            image::Rgba([body[0], body[1], body[2], 255])
        } else if in_head {
            image::Rgba([0, 0, 200, 255])
        } else {
            image::Rgba(bg)
        }
    })
}

fn gif(frames: &[(image::RgbaImage, u32)]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = image::codecs::gif::GifEncoder::new(&mut out);
        enc.set_repeat(image::codecs::gif::Repeat::Infinite)
            .unwrap();
        enc.encode_frames(frames.iter().map(|(img, ms)| {
            image::Frame::from_parts(img.clone(), 0, 0, image::Delay::from_numer_denom_ms(*ms, 1))
        }))
        .unwrap();
    }
    out
}

fn apng(frames: &[(image::RgbaImage, u16)]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, 24, 20);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_animated(frames.len() as u32, 0).unwrap();
        let mut w = enc.write_header().unwrap();
        for (img, ms) in frames {
            w.set_frame_delay(*ms, 1000).unwrap();
            w.write_image_data(img.as_raw()).unwrap();
        }
        w.finish().unwrap();
    }
    out
}

const ANIMATED: &str = r#"type flipbook
version 1
---
source "walk.gif" {
  event 2 "step"
}
source "idle.png" {
  clip "idle"
  mode pingPong
}
key auto
align feet
"#;

/// Animated sources: a GIF on white (keyed out) whose figure drifts and
/// whose delays include one of 10 ms (shown as 100 ms, as browsers do),
/// and an APNG with real alpha. Ways this could fail: the frames keep the
/// drift (a cell's figure moves within it); the crop differs per frame;
/// the background stays (no alpha); identical frames take two cells; the
/// delays are lost; the GIF or the APNG are cooked as assets of their own;
/// an edited GIF leaves a stale sheet.
#[test]
fn animated_sources_cook_into_a_packed_sheet() {
    let white = [255, 255, 255, 255];
    let walk = [
        (figure(4, 8, [200, 0, 0], white), 100),
        (figure(5, 7, [190, 0, 0], white), 200),
        (figure(7, 9, [180, 0, 0], white), 10),
        (figure(5, 7, [190, 0, 0], white), 100),
    ];
    let clear = [0, 0, 0, 0];
    let idle = [
        (figure(10, 10, [0, 160, 0], clear), 250),
        (figure(10, 10, [0, 140, 0], clear), 250),
    ];
    let root = project(
        "animated",
        &[
            ("sprites/walk.gif", gif(&walk)),
            ("sprites/idle.png", apng(&idle)),
            ("sprites/hero.ogt", ANIMATED.as_bytes().to_vec()),
        ],
    );
    let bundle = cooked(&root);
    let sheet = bundle.load_flipbook("sprites/hero.ogt").unwrap();
    assert_eq!(sheet.texture, "sprites/hero.ogt#sheet");
    let walk_clip = &sheet.clips[sheet.clip("walk").unwrap() as usize];
    assert_eq!(walk_clip.durations, vec![0.1, 0.2, 0.1, 0.1]);
    assert_eq!(walk_clip.events, vec![(2, "step".to_string())]);
    let idle_clip = &sheet.clips[sheet.clip("idle").unwrap() as usize];
    assert_eq!(idle_clip.mode, FlipMode::PingPong);
    assert_eq!(idle_clip.durations, vec![0.25, 0.25]);
    assert_eq!(sheet.frames.len(), 6);
    // One size: the 4x8 figure and a texel of padding all round.
    for f in &sheet.frames {
        assert_eq!([f.rect[2], f.rect[3]], [6, 10], "frame {}", f.name);
        assert_eq!(f.pivot, [0.5, 0.1], "the feet, a texel above the bottom");
    }
    assert_eq!(
        sheet.frames[1].rect, sheet.frames[3].rect,
        "identical frames share a cell"
    );
    let cells: std::collections::BTreeSet<[u32; 2]> = sheet
        .frames
        .iter()
        .map(|f| [f.rect[0], f.rect[1]])
        .collect();
    assert_eq!(cells.len(), 5);
    // Every cell holds its figure at one place: body at (1..5, 3..9).
    let tex = bundle.load_texture("sprites/hero.ogt#sheet").unwrap();
    assert!(tex.is_pixel_art());
    let data = tex.to_texture_data().unwrap();
    assert_eq!([data.width, data.height], sheet.size);
    let at = |f: &og_anim::SheetFrame, x: u32, y: u32| {
        let i = (((f.rect[1] + y) * data.width + f.rect[0] + x) * 4) as usize;
        [
            data.rgba[i],
            data.rgba[i + 1],
            data.rgba[i + 2],
            data.rgba[i + 3],
        ]
    };
    let bodies = [200, 190, 180, 190];
    for (f, red) in sheet.frames.iter().zip(bodies) {
        assert_eq!(at(f, 0, 0)[3], 0, "{}: the background is keyed", f.name);
        assert_eq!(
            at(f, 1, 3),
            [red, 0, 0, 255],
            "{}: the body's corner",
            f.name
        );
        assert_eq!(at(f, 4, 8), [red, 0, 0, 255], "{}", f.name);
        assert_eq!(at(f, 2, 1), [0, 0, 200, 255], "{}: the head", f.name);
        assert_eq!(at(f, 5, 9)[3], 0, "{}: padding", f.name);
    }
    assert_eq!(
        at(&sheet.frames[4], 1, 3),
        [0, 160, 0, 255],
        "the APNG's alpha"
    );
    // The sources are part of the sheet, not assets of their own.
    assert!(bundle.load_texture("sprites/idle.png").is_err());

    // Editing the GIF recooks the sheet.
    let mut slower = walk.clone();
    slower[1].1 = 300;
    std::fs::write(root.join("assets/sprites/walk.gif"), gif(&slower)).unwrap();
    let sheet = cooked(&root).load_flipbook("sprites/hero.ogt").unwrap();
    assert_eq!(sheet.clips[0].durations[1], 0.3);

    // A texture and animated sources do not mix.
    std::fs::write(
        root.join("assets/sprites/hero.ogt"),
        format!("{ANIMATED}texture \"walk.png\"\n"),
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(
        report.errors().join("\n").contains("do not mix"),
        "{:?}",
        report.errors()
    );
}

/// A source drawn twice the size of the others: `scale 0.5` in its block
/// brings its figure to theirs, so one crop holds both. Ways this could
/// fail: the source's scale is ignored (the crop grows to the big figure),
/// or it replaces the sheet's instead of multiplying it.
#[test]
fn a_source_scale_brings_a_larger_drawing_to_the_others() {
    let white = [255, 255, 255, 255];
    let small = figure(4, 8, [200, 0, 0], white);
    let big = image::imageops::resize(&small, 48, 40, image::imageops::FilterType::Nearest);
    let root = project(
        "source-scale",
        &[
            ("sprites/run.gif", gif(&[(small, 100)])),
            ("sprites/idle.gif", gif(&[(big, 100)])),
            (
                "sprites/hero.ogt",
                b"type flipbook\nversion 1\n---\nsource \"run.gif\"\nsource \"idle.gif\" {\n  scale 0.5\n}\nkey auto\nalign feet\nscale 1\n"
                    .to_vec(),
            ),
        ],
    );
    let sheet = cooked(&root).load_flipbook("sprites/hero.ogt").unwrap();
    assert_eq!(sheet.frames.len(), 2);
    for f in &sheet.frames {
        assert_eq!([f.rect[2], f.rect[3]], [6, 10], "frame {}", f.name);
    }
}

/// Loose stills of two sizes in one sheet with `crop each`: each keeps a
/// cell of its own size (its drawn bounds and the padding) with its feet
/// as the pivot, and the cells do not overlap. Ways this could fail: both
/// take the union's size (the default `crop all`), a pivot is measured in
/// the union instead of its own cell, the shelves overlap.
#[test]
fn crop_each_packs_loose_stills_in_cells_of_their_own_size() {
    let still = |w: u32, h: u32, c: [u8; 4]| {
        let img = image::RgbaImage::from_fn(24, 20, |x, y| {
            if (2..2 + w).contains(&x) && (20 - h - 2..18).contains(&y) {
                image::Rgba(c)
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        });
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    };
    let root = project(
        "crop-each",
        &[
            ("props/seed.png", still(4, 4, [200, 0, 0, 255])),
            ("props/crate.png", still(10, 6, [0, 0, 200, 255])),
            (
                "props/props.ogt",
                b"type flipbook\nversion 1\n---\nsource \"seed.png\"\nsource \"crate.png\"\nalign feet\ncrop each\n"
                    .to_vec(),
            ),
        ],
    );
    let sheet = cooked(&root).load_flipbook("props/props.ogt").unwrap();
    let frame = |clip: &str| {
        let c = &sheet.clips[sheet.clip(clip).unwrap() as usize];
        sheet.frames[c.frames[0] as usize].clone()
    };
    let (seed, crate_) = (frame("seed"), frame("crate"));
    assert_eq!([seed.rect[2], seed.rect[3]], [6, 6]);
    assert_eq!([crate_.rect[2], crate_.rect[3]], [12, 8]);
    assert_eq!(seed.pivot, [0.5, 1.0 / 6.0]);
    assert_eq!(crate_.pivot, [0.5, 0.125]);
    let overlap = |a: [u32; 4], b: [u32; 4]| {
        a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
    };
    assert!(
        !overlap(seed.rect, crate_.rect),
        "{:?} {:?}",
        seed.rect,
        crate_.rect
    );
}
