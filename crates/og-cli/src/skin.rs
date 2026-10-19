//! `og skin pack <dir> [--out <dir>]`: a skin drawn by hand, one PNG per
//! part, packed into the atlas a skin file cuts from.
//!
//! The input directory holds:
//!
//! - `<part>.9.png` for each nine-slice: `panel`, `card`, `button`,
//!   `buttonHover`, `buttonPressed`, `buttonDisabled`, `field`, `slot`,
//!   `tooltip`, `capsule`, `track`, `fill`, and `soft` (the blurred
//!   rectangle shadows and glows use). A nine-patch, as Android draws them:
//!   the art inside a one-pixel border whose top row marks the columns that
//!   stretch and whose left column marks the rows that stretch (opaque dark
//!   pixels); the corner insets are what lies outside the marks;
//! - any other `<name>.png` (`corner`, `divider`, `flourish`, `tile`, a
//!   picture of the game's own): packed as it is.
//!
//! It writes `<out>/atlas.png` (the images on shelves, tallest first, two
//! pixels apart with their edge pixels extruded into the gap so filtering
//! never reads a neighbour), `<out>/atlas.png.ogt` when missing, and
//! `<out>/skin.ogt`: an existing one keeps everything but its `atlas` line,
//! its `slice` and `soft` lines (rewritten from the nine-patches) and the
//! regions of its `corner`, `divider`, `flourish` and `tile` lines (moved to
//! where those images now sit); without one it starts a skin of the slices.
//! `--out` defaults to the input directory.

use image::{Rgba, RgbaImage};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og skin pack <dir> [--out <dir>]";

/// The parts a nine-patch can dress (og-ui's `SkinPart` names) and `soft`.
const SLICES: [&str; 13] = [
    "panel",
    "card",
    "button",
    "buttonHover",
    "buttonPressed",
    "buttonDisabled",
    "field",
    "slot",
    "tooltip",
    "capsule",
    "track",
    "fill",
    "soft",
];

/// Pixels between packed images (each image's edge is extruded into it).
const GAP: u32 = 2;

/// One image to pack, with its nine-slice insets when it is a nine-patch.
struct Piece {
    name: String,
    image: RgbaImage,
    /// left, top, right, bottom in texels.
    insets: Option<[u32; 4]>,
    at: (u32, u32),
}

pub fn run(args: &[&str]) -> ExitCode {
    match args {
        ["pack", rest @ ..] => match pack(rest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("og skin pack: {e}");
                ExitCode::from(1)
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn pack(args: &[&str]) -> Result<(), String> {
    let (dir, out) = match args {
        [dir] => (PathBuf::from(dir), PathBuf::from(dir)),
        [dir, "--out", out] => (PathBuf::from(dir), PathBuf::from(out)),
        _ => return Err(USAGE.into()),
    };
    let mut pieces = read_pieces(&dir)?;
    if pieces.is_empty() {
        return Err(format!(
            "{} holds no PNG: draw one nine-patch per part (panel.9.png, button.9.png, ...)",
            dir.display()
        ));
    }
    let (w, h) = shelve(&mut pieces);
    let atlas = compose(&pieces, w, h);
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let atlas_path = out.join("atlas.png");
    atlas
        .save(&atlas_path)
        .map_err(|e| format!("{}: {e}", atlas_path.display()))?;
    let import = out.join("atlas.png.ogt");
    if !import.exists() {
        std::fs::write(
            &import,
            "type import\nversion 1\n---\n# A skin atlas drawn by hand (og skin pack).\ntexture {\n  kind color\n}\n",
        )
        .map_err(|e| format!("{}: {e}", import.display()))?;
    }
    let skin_path = out.join("skin.ogt");
    let old = std::fs::read_to_string(&skin_path).ok();
    let name = out
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("skin")
        .to_string();
    let atlas_id = atlas_id(&out);
    let text = skin_text(old.as_deref(), &name, &atlas_id, &pieces, (w, h));
    std::fs::write(&skin_path, text).map_err(|e| format!("{}: {e}", skin_path.display()))?;
    let slices = pieces.iter().filter(|p| p.insets.is_some()).count();
    println!(
        "og skin pack: {} images ({slices} nine-slices) into {} ({w}×{h}); {} {}",
        pieces.len(),
        atlas_path.display(),
        skin_path.display(),
        if old.is_some() { "updated" } else { "written" }
    );
    Ok(())
}

/// The asset id the skin file names its atlas by: the path under the
/// project's `assets/` when there is one, else the file name.
fn atlas_id(out: &Path) -> String {
    let full = std::fs::canonicalize(out).unwrap_or_else(|_| out.to_path_buf());
    let parts: Vec<String> = full
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    match parts.iter().rposition(|p| p == "assets") {
        Some(i) => {
            let mut id = parts[i + 1..].join("/");
            id.push_str("/atlas.png");
            id
        }
        None => "atlas.png".into(),
    }
}

fn read_pieces(dir: &Path) -> Result<Vec<Piece>, String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension().is_some_and(|x| x == "png")
                && p.file_name().is_some_and(|n| n != "atlas.png")
        })
        .collect();
    files.sort();
    let mut out = Vec::new();
    for path in files {
        let file = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let image = image::open(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .to_rgba8();
        if let Some(name) = file.strip_suffix(".9.png") {
            if !SLICES.contains(&name) {
                return Err(format!(
                    "{file}: a nine-patch is named for a part ({}), not '{name}'",
                    SLICES.join(", ")
                ));
            }
            let (art, insets) = nine_patch(&image).map_err(|e| format!("{file}: {e}"))?;
            out.push(Piece {
                name: name.into(),
                image: art,
                insets: Some(insets),
                at: (0, 0),
            });
        } else {
            let name = file.trim_end_matches(".png");
            if SLICES.contains(&name) {
                return Err(format!(
                    "{file}: a slice's image is a nine-patch ({name}.9.png) whose one-pixel border marks what stretches"
                ));
            }
            out.push(Piece {
                name: name.into(),
                image,
                insets: None,
                at: (0, 0),
            });
        }
    }
    Ok(out)
}

/// A marker pixel of a nine-patch's border: opaque and dark.
fn marked(p: &Rgba<u8>) -> bool {
    p[3] >= 128 && (p[0] as u32 + p[1] as u32 + p[2] as u32) < 3 * 64
}

/// The art inside a nine-patch's border, and its insets (left, top, right,
/// bottom): the columns and rows outside the stretch marks.
fn nine_patch(img: &RgbaImage) -> Result<(RgbaImage, [u32; 4]), String> {
    let (w, h) = img.dimensions();
    if w < 3 || h < 3 {
        return Err("too small for a nine-patch (a one-pixel border around the art)".into());
    }
    let span = |marks: Vec<bool>| -> Option<(u32, u32)> {
        let first = marks.iter().position(|m| *m)? as u32;
        let last = marks.iter().rposition(|m| *m)? as u32;
        Some((first, last))
    };
    let cols: Vec<bool> = (1..w - 1).map(|x| marked(img.get_pixel(x, 0))).collect();
    let rows: Vec<bool> = (1..h - 1).map(|y| marked(img.get_pixel(0, y))).collect();
    let (iw, ih) = (w - 2, h - 2);
    let (c0, c1) = span(cols).ok_or(
        "its top border marks no column that stretches (opaque dark pixels in the first row)",
    )?;
    let (r0, r1) = span(rows).ok_or(
        "its left border marks no row that stretches (opaque dark pixels in the first column)",
    )?;
    let art = image::imageops::crop_imm(img, 1, 1, iw, ih).to_image();
    Ok((art, [c0, r0, iw - 1 - c1, ih - 1 - r1]))
}

/// Places the pieces on shelves, tallest first, in an atlas as wide as the
/// next power of two over the square of their area (or the widest piece).
fn shelve(pieces: &mut [Piece]) -> (u32, u32) {
    let area: u64 = pieces
        .iter()
        .map(|p| u64::from(p.image.width() + GAP) * u64::from(p.image.height() + GAP))
        .sum();
    let widest = pieces
        .iter()
        .map(|p| p.image.width() + GAP)
        .max()
        .unwrap_or(1);
    let side = ((area as f64).sqrt().ceil() as u32).max(widest);
    let width = side.next_power_of_two();
    let mut order: Vec<usize> = (0..pieces.len()).collect();
    order.sort_by_key(|&i| {
        (
            std::cmp::Reverse(pieces[i].image.height()),
            pieces[i].name.clone(),
        )
    });
    let (mut x, mut y, mut shelf) = (GAP / 2, GAP / 2, 0);
    for i in order {
        let (w, h) = pieces[i].image.dimensions();
        if x + w + GAP / 2 > width {
            x = GAP / 2;
            y += shelf + GAP;
            shelf = 0;
        }
        pieces[i].at = (x, y);
        x += w + GAP;
        shelf = shelf.max(h);
    }
    (width, y + shelf + GAP / 2)
}

/// The atlas: each piece at its place, its edge pixels extruded one pixel
/// into the gap around it.
fn compose(pieces: &[Piece], w: u32, h: u32) -> RgbaImage {
    let mut atlas = RgbaImage::new(w, h);
    for p in pieces {
        let (px, py) = p.at;
        let (iw, ih) = p.image.dimensions();
        let e = GAP / 2;
        for y in 0..ih + 2 * e {
            for x in 0..iw + 2 * e {
                let sx = (x as i64 - e as i64).clamp(0, iw as i64 - 1) as u32;
                let sy = (y as i64 - e as i64).clamp(0, ih as i64 - 1) as u32;
                let (ax, ay) = (px + x - e, py + y - e);
                if ax < w && ay < h {
                    atlas.put_pixel(ax, ay, *p.image.get_pixel(sx, sy));
                }
            }
        }
    }
    atlas
}

fn region(p: &Piece) -> String {
    format!(
        "{} {} {} {}",
        p.at.0,
        p.at.1,
        p.image.width(),
        p.image.height()
    )
}

/// The skin file: `old` with its atlas, slices and ornament regions
/// replaced, or a new skin of the slices.
fn skin_text(
    old: Option<&str>,
    name: &str,
    atlas_id: &str,
    pieces: &[Piece],
    size: (u32, u32),
) -> String {
    let atlas_line = format!("atlas \"{atlas_id}\" {} {}", size.0, size.1);
    // An existing slice line's words (tint or authored, stretch, repeat or
    // round) stay with its part.
    let modes = |part: &str| -> String {
        old.into_iter()
            .flat_map(str::lines)
            .map(str::split_whitespace)
            .filter_map(|mut w| {
                (w.next() == Some("slice") && w.next() == Some(part)).then(|| {
                    w.filter(|v| v.parse::<f64>().is_err())
                        .map(|v| format!(" {v}"))
                        .collect::<String>()
                })
            })
            .next()
            .unwrap_or_default()
    };
    let slice_line = |p: &Piece| {
        let [l, t, r, b] = p.insets.expect("a nine-patch");
        if p.name == "soft" {
            // The soft rectangle takes one inset (its blur).
            format!("soft {} {}", region(p), l.max(t).max(r).max(b))
        } else {
            format!(
                "slice {} {} {l} {t} {r} {b}{}",
                p.name,
                region(p),
                modes(&p.name)
            )
        }
    };
    let mut slices: Vec<&Piece> = pieces.iter().filter(|p| p.insets.is_some()).collect();
    slices.sort_by_key(|p| SLICES.iter().position(|s| *s == p.name));
    let Some(old) = old else {
        let mut l = vec![
            "type skin".to_string(),
            "version 1".into(),
            "---".into(),
            "# Packed by og skin pack from the parts drawn by hand; add the palette, fonts and motion."
                .into(),
            format!("name \"{name}\""),
            "base dark".into(),
            atlas_line,
        ];
        l.extend(slices.iter().map(|p| slice_line(p)));
        return l.join("\n") + "\n";
    };
    let mut out: Vec<String> = Vec::new();
    let mut placed = false;
    for line in old.lines() {
        let t = line.trim_start();
        let key = t.split_whitespace().next().unwrap_or("");
        match key {
            "atlas" => out.push(atlas_line.clone()),
            "slice" | "soft" => {
                // Rewritten from the nine-patches, where the first one stood.
                if !placed {
                    out.extend(slices.iter().map(|p| slice_line(p)));
                    placed = true;
                }
            }
            "corner" | "divider" | "flourish" | "tile" => {
                match pieces.iter().find(|p| p.name == key && p.insets.is_none()) {
                    Some(p) => {
                        let rest: Vec<&str> = t.split_whitespace().skip(5).collect();
                        out.push(format!("{key} {} {}", region(p), rest.join(" ")));
                    }
                    None => out.push(line.to_string()),
                }
            }
            _ => out.push(line.to_string()),
        }
    }
    if !placed {
        out.extend(slices.iter().map(|p| slice_line(p)));
    }
    out.join("\n") + "\n"
}
