//! `png_best`, the encoder of every reference frame, and the guard that
//! keeps the committed references written by it (or an encoder as good):
//! a frame recorded by copying a run's fast PNG (the probe's fdeflate, the
//! `image` crate's default `Fast`) is several times larger, and 309 such
//! frames made 167 MB of the repository before 2026-11-01. Recompress with
//! `bun scripts/recompress-references.ts`.

use og_render::{RgbaImage, png_best};
use std::path::{Path, PathBuf};

fn decode(bytes: &[u8]) -> (image::ColorType, image::RgbaImage) {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).expect("a PNG");
    (img.color(), img.to_rgba8())
}

/// A frame with gradients, noise and flat areas, every pixel opaque or not.
fn frame(width: u32, height: u32, opaque: bool) -> RgbaImage {
    let mut pixels = Vec::new();
    let mut seed = 0x9e37_79b9u32;
    for y in 0..height {
        for x in 0..width {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let noise = if x > width / 2 { (seed & 7) as u8 } else { 0 };
            let a = if opaque { 255 } else { (x * 255 / width) as u8 };
            pixels.extend_from_slice(&[
                (x * 3) as u8 ^ noise,
                (y * 5) as u8,
                if y > height / 2 { 40 } else { noise },
                a,
            ]);
        }
    }
    RgbaImage {
        width,
        height,
        pixels,
    }
}

#[test]
fn png_best_decodes_to_the_same_pixels_rgb_when_opaque() {
    for (opaque, color) in [
        (true, image::ColorType::Rgb8),
        (false, image::ColorType::Rgba8),
    ] {
        let img = frame(97, 41, opaque);
        let (got, back) = decode(&png_best(&img));
        assert_eq!(got, color, "opaque {opaque}");
        assert_eq!((back.width(), back.height()), (97, 41));
        assert!(
            back.as_raw() == &img.pixels,
            "opaque {opaque}: the pixels decode exactly"
        );
        let fast = og_render::png(&img);
        assert!(
            png_best(&img).len() < fast.len(),
            "smaller than the capture's fast PNG"
        );
    }
}

/// Every PNG under a `references/` directory of the repository.
fn references() -> Vec<PathBuf> {
    fn walk(dir: &Path, in_refs: bool, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let path = e.path();
            let name = e.file_name();
            let name = name.to_string_lossy();
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if !matches!(
                    &*name,
                    "target" | "node_modules" | "cooked" | "og-out" | ".git"
                ) {
                    walk(&path, in_refs || name == "references", out);
                }
            } else if in_refs && name.ends_with(".png") {
                out.push(path);
            }
        }
    }
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut out = Vec::new();
    for root in ["crates", "templates", "examples", "packages"] {
        walk(&repo.join(root), false, &mut out);
    }
    out.sort();
    out
}

/// A PNG's width × height (IHDR).
fn area(png: &[u8]) -> u64 {
    let dim = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().expect("IHDR")) as u64;
    dim(16) * dim(20)
}

#[test]
fn a_sample_of_reference_frames_is_no_larger_than_png_best_writes() {
    // The smallest frame of each references directory, by pixels, the
    // smallest first until four megapixels (png_best unoptimised takes
    // about 0.6 s a 1280×720 frame): the check reads the encoding as well
    // as the zlib level (unfiltered rows at level 9 pass the other test).
    let mut sample: std::collections::BTreeMap<PathBuf, (u64, PathBuf, Vec<u8>)> =
        Default::default();
    for f in references() {
        let bytes = std::fs::read(&f).expect("read");
        let a = area(&bytes);
        let e = sample
            .entry(f.parent().expect("dir").to_path_buf())
            .or_insert((u64::MAX, PathBuf::new(), Vec::new()));
        if a < e.0 {
            *e = (a, f, bytes);
        }
    }
    let mut sample: Vec<_> = sample.into_values().collect();
    sample.sort_by_key(|(a, f, _)| (*a, f.clone()));
    let mut budget = 4_000_000u64;
    let mut large = Vec::new();
    for (a, f, bytes) in &sample {
        if *a > budget {
            break;
        }
        budget -= a;
        let (_, img) = decode(bytes);
        let best = png_best(&RgbaImage {
            width: img.width(),
            height: img.height(),
            pixels: img.into_raw(),
        })
        .len();
        // zlib's and miniz_oxide's level 9 differ by a few percent.
        if bytes.len() as f64 > best as f64 * 1.1 {
            large.push(format!(
                "{}: {} bytes, png_best {best}",
                f.display(),
                bytes.len()
            ));
        }
    }
    assert!(
        large.is_empty(),
        "larger than png_best writes (re-encode with `bun scripts/recompress-references.ts`): {large:#?}"
    );
}
