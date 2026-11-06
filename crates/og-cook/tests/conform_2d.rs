//! The cook's conformance of 2D textures to the project's art contract
//! (docs/art.md §2.3, `cook/conform/texture2d.rs`), through `cook_project`
//! and the cooked bundle. Every expected texel here is worked out by hand
//! from the fixture and the contract's rules, not taken from the code.
//!
//! Ways this could fail: a bilinear (or mean) downscale blends a cell's
//! colours; the grid is guessed at the wrong size; the palette maps by RGB
//! distance or misses a colour; the line is drawn as a square, outside the
//! cell, or round the cell's edge; a flipbook's frames or a tile set's
//! tiles keep their old rects over a re-laid-out texture; the index
//! counts from 0 with no room for transparency; a rotation turns the wrong
//! way; an opt-out is ignored or not reported; a project without a
//! contract cooks differently; the source image is written.

use og_cook::cook::ConformAction;
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn project(name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-conform-2d")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = if *path == "project.ogt" {
            root.join(path)
        } else {
            root.join("assets").join(path)
        };
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

const T: [u8; 4] = [0, 0, 0, 0];
const K: [u8; 4] = [0, 0, 0, 255];
const W: [u8; 4] = [255, 255, 255, 255];
const R: [u8; 4] = [224, 32, 32, 255];
const B: [u8; 4] = [32, 64, 224, 255];

/// The four-colour palette the fixtures sit on: K W R B, in that order.
const FOUR: &str = "000000\nffffff\ne02020\n2040e0\n";

/// `rows` of texels, each drawn as an `s` x `s` block.
fn png(rows: &[&[[u8; 4]]], s: u32) -> Vec<u8> {
    let (w, h) = (rows[0].len() as u32, rows.len() as u32);
    let img = image::RgbaImage::from_fn(w * s, h * s, |x, y| {
        image::Rgba(rows[(y / s) as usize][(x / s) as usize])
    });
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn edit_png(bytes: &[u8], f: impl Fn(&mut image::RgbaImage)) -> Vec<u8> {
    let mut img = image::load_from_memory(bytes).unwrap().to_rgba8();
    f(&mut img);
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn look(body: &str) -> Vec<u8> {
    format!("type look\nversion 1\n---\n{body}").into_bytes()
}

fn project_ogt(extra: &str) -> Vec<u8> {
    format!("type project\nversion 1\n---\nname \"t\"\nid \"t\"\nentry \"src/main.ts\"\n{extra}")
        .into_bytes()
}

/// The cooked texture's texels, row by row.
fn texels(b: &Bundle, id: &str) -> (u32, u32, Vec<[u8; 4]>) {
    let t = b.load_texture(id).unwrap().to_texture_data().unwrap();
    (t.width, t.height, t.rgba.as_chunks::<4>().0.to_vec())
}

fn rows(px: &[[u8; 4]], w: usize) -> Vec<Vec<[u8; 4]>> {
    px.chunks(w).map(<[_]>::to_vec).collect()
}

fn cook(root: &Path) -> og_cook::Report {
    let r = cook_project(root, &CookOptions::default()).unwrap();
    assert!(r.ok(), "{:?}", r.errors());
    r
}

/// The 5 x 4 sprite: asymmetric, so a flip or a transpose shows.
fn hero() -> Vec<&'static [[u8; 4]]> {
    vec![
        &[T, T, R, T, T],
        &[T, R, W, B, T],
        &[T, T, R, T, T],
        &[T, T, T, T, T],
    ]
}

const PIXEL_LOOK: &str = "ArtPalette {\n  colors \"palettes/four.hex\"\n  strict true\n}\nArtPixels {\n  spriteSizes 4 5\n}\nArtLine {\n  width 1\n  color \"#000000\"\n}\n";

#[test]
fn a_sprite_drawn_at_2x_comes_back_to_its_grid_on_the_palette_with_its_line() {
    // Drawn at 2x (10 x 8), one texel of the top red block a little off
    // red: three exact reds outvote it in their 2 x 2 box.
    let source = edit_png(&png(&hero(), 2), |img| {
        img.put_pixel(4, 0, image::Rgba([230, 30, 30, 255]))
    });
    let root = project(
        "sprite",
        &[
            ("project.ogt", project_ogt("look \"looks/main.ogt\"\n")),
            ("looks/main.ogt", look(PIXEL_LOOK)),
            ("palettes/four.hex", FOUR.as_bytes().to_vec()),
            ("sprites/hero.png", source.clone()),
            (
                "sprites/hero.png.ogt",
                b"type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n".to_vec(),
            ),
        ],
    );
    let report = cook(&root);
    let b = Bundle::open(&root.join("cooked")).unwrap();
    // spriteSizes 4 5: 5 draws the 10 x 8 cell back exactly, 4 does not.
    // The outside line (diamond, one texel) takes every clear texel with
    // a shape texel left, right, above or below it, inside the 5 x 4 cell.
    let (w, h, px) = texels(&b, "sprites/hero.png");
    assert_eq!((w, h), (5, 4));
    assert_eq!(
        rows(&px, 5),
        vec![
            vec![T, K, R, K, T],
            vec![K, R, W, B, K],
            vec![T, K, R, K, T],
            vec![T, T, K, T, T],
        ]
    );
    // The index: the colour in RGB, the palette index plus one in alpha.
    let (_, _, ix) = texels(&b, "sprites/hero.png#index");
    let alpha: Vec<u8> = ix.iter().map(|p| p[3]).collect();
    #[rustfmt::skip]
    assert_eq!(alpha, vec![
        0, 1, 3, 1, 0,
        1, 3, 2, 4, 1,
        0, 1, 3, 1, 0,
        0, 0, 1, 0, 0,
    ]);
    assert!(
        ix.iter()
            .zip(&px)
            .all(|(i, p)| i[3] == 0 || i[..3] == p[..3])
    );
    // The report says what changed.
    let e = report
        .conform
        .iter()
        .find(|e| e.source == "sprites/hero.png")
        .unwrap();
    assert_eq!(e.action, ConformAction::Changed);
    assert!(
        e.what.contains("10x8 cells to 5x4") && e.what.contains("of palettes/four.hex"),
        "{}",
        e.what
    );
    assert!(
        report.to_markdown().contains("art contract: changed"),
        "{}",
        report.to_markdown()
    );
    // The source is never written.
    assert_eq!(
        std::fs::read(root.join("assets/sprites/hero.png")).unwrap(),
        source
    );

    // Cooked again: nothing is stale. The same project elsewhere: the same
    // bytes. A look edit re-cooks the texture.
    assert!(cook(&root).cooked_outputs().is_empty());
    let twin = project(
        "sprite-twin",
        &[
            ("project.ogt", project_ogt("look \"looks/main.ogt\"\n")),
            ("looks/main.ogt", look(PIXEL_LOOK)),
            ("palettes/four.hex", FOUR.as_bytes().to_vec()),
            ("sprites/hero.png", source),
            (
                "sprites/hero.png.ogt",
                b"type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n".to_vec(),
            ),
        ],
    );
    cook(&twin);
    assert_eq!(files(&root.join("cooked")), files(&twin.join("cooked")));
    std::fs::write(
        root.join("assets/looks/main.ogt"),
        look(&PIXEL_LOOK.replace("width 1", "width 0")),
    )
    .unwrap();
    let again = cook(&root);
    let ids: Vec<&str> = again
        .cooked_outputs()
        .iter()
        .map(|o| o.id.as_str())
        .collect();
    assert!(ids.contains(&"sprites/hero.png"), "{ids:?}");
    let b = Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(texels(&b, "sprites/hero.png").2[1], T, "no line now");
}

#[test]
fn a_flipbook_and_a_tile_set_follow_their_texture_to_its_grid() {
    // Three 4 x 4 frames, drawn at 2x with 2 texels between: 28 x 8.
    let f0: [&[[u8; 4]]; 4] = [&[R, R, T, T], &[R, R, T, T], &[T, T, T, T], &[T, T, T, B]];
    let f1: [&[[u8; 4]]; 4] = [&[T, R, R, T], &[T, R, R, T], &[T, T, T, T], &[B, T, T, T]];
    let f2: [&[[u8; 4]]; 4] = [&[T, T, R, R], &[T, T, R, R], &[W, T, T, T], &[T, T, T, T]];
    let gap: [u8; 4] = T;
    let sheet: Vec<Vec<[u8; 4]>> = (0..4)
        .map(|y| {
            let mut row = Vec::new();
            for (i, f) in [f0, f1, f2].iter().enumerate() {
                if i > 0 {
                    row.push(gap);
                }
                row.extend_from_slice(f[y]);
            }
            row
        })
        .collect();
    let sheet_rows: Vec<&[[u8; 4]]> = sheet.iter().map(Vec::as_slice).collect();
    // 1 texel of gap at 1x is 2 at 2x: (4 + 1 + 4 + 1 + 4) * 2 = 28.
    let flip_png = png(&sheet_rows, 2);
    // Two 4 x 4 tiles at 2x with a 2-texel gap (18 x 8); the second of an
    // accent's near twin, which tiles keep off.
    let twin: [u8; 4] = [200, 40, 40, 255];
    let tiles: Vec<Vec<[u8; 4]>> = (0..4)
        .map(|_| vec![W, W, W, W, T, twin, twin, twin, twin])
        .collect();
    let tile_rows: Vec<&[[u8; 4]]> = tiles.iter().map(Vec::as_slice).collect();
    let root = project(
        "sheets",
        &[
            ("project.ogt", project_ogt("look \"looks/main.ogt\"\n")),
            (
                "looks/main.ogt",
                look("ArtPalette {\n  colors \"palettes/five.hex\"\n  strict true\n  accents 4\n}\nArtPixels {\n  unit 4\n  spriteSizes 4\n}\n"),
            ),
            ("palettes/five.hex", format!("{FOUR}c82828\n").into_bytes()),
            ("sprites/coin.png", flip_png),
            ("sprites/coin.ogt", b"type flipbook\nversion 1\n---\ntexture \"coin.png\"\ngrid 8 8 3 1\nspacing 2\nclip spin {\n  frames 0 1 2\n  fps 8\n}\n".to_vec()),
            ("tiles/ground.png", png(&tile_rows, 2)),
            ("tiles/ground.ogt", b"type tileset\nversion 1\n---\ntexture \"tiles/ground.png\"\ngrid 8 8 2 1\nspacing 2\ntile 1 {\n  collide 0 2 8 6\n}\n".to_vec()),
        ],
    );
    let report = cook(&root);
    let b = Bundle::open(&root.join("cooked")).unwrap();
    // The flipbook: 4 x 4 frames one texel apart, the sheet 14 x 4.
    let bytes = b.read("sprites/coin.ogt").unwrap();
    let sheet = b.flipbook("sprites/coin.ogt", bytes.as_slice()).unwrap();
    assert_eq!(sheet.size, [14, 4]);
    let rects: Vec<[u32; 4]> = sheet.frames.iter().map(|f| f.rect).collect();
    assert_eq!(rects, vec![[0, 0, 4, 4], [5, 0, 4, 4], [10, 0, 4, 4]]);
    let (w, _, px) = texels(&b, "sprites/coin.png");
    assert_eq!(w, 14);
    let frame = |n: u32| -> Vec<Vec<[u8; 4]>> {
        (0..4)
            .map(|y| (0..4).map(|x| px[(y * 14 + n * 5 + x) as usize]).collect())
            .collect()
    };
    assert_eq!(frame(2), f2.iter().map(|r| r.to_vec()).collect::<Vec<_>>());
    assert_eq!(frame(0), f0.iter().map(|r| r.to_vec()).collect::<Vec<_>>());
    assert!((0..4).all(|y| px[y * 14 + 4] == T), "the gap stays clear");

    // The tile set: tiles of unit 4, spacing 1, the collider scaled; its
    // second tile on red, not the accent it is nearest.
    let bytes = b.read("tiles/ground.ogt").unwrap();
    let t = b.tileset("tiles/ground.ogt", bytes.as_slice()).unwrap();
    assert_eq!(
        (t.tile_width, t.tile_height, t.spacing, t.margin),
        (4, 4, 1, 0)
    );
    assert_eq!(
        t.collision(1),
        Some(og_tilemap::Collision::Rect {
            x: 0.0,
            y: 1.0,
            w: 4.0,
            h: 3.0
        })
    );
    let (w, h, px) = texels(&b, "tiles/ground.png");
    assert_eq!((w, h), (9, 4));
    assert_eq!(px[0], W);
    assert_eq!(px[4], T);
    assert_eq!(px[5], R, "an accent's twin goes to red on a tile");
    // Both sheets are listed as following their texture.
    for id in ["sprites/coin.ogt", "tiles/ground.ogt"] {
        let e = report.conform.iter().find(|e| e.source == id).unwrap();
        assert!(e.what.contains("conformed layout"), "{id}: {}", e.what);
    }
}

/// Every file under `dir` with its bytes, by path.
fn files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_name().is_some_and(|n| n != ".cook-times.ogt") {
                out.insert(
                    // `/` on every host, so `outputs` can match `looks/`.
                    p.strip_prefix(dir)
                        .unwrap()
                        .display()
                        .to_string()
                        .replace('\\', "/"),
                    std::fs::read(&p).unwrap(),
                );
            }
        }
    }
    out
}
