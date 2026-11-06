//! The terrain that cooks (`terrain-generated`; og-tests' tests/terrain.rs
//! has the others): read from its frame, then its reference.

use og_render::RgbaImage;
use og_tests_cooked::gfx;
use std::path::PathBuf;

#[path = "terrain/streaming_wind.rs"]
mod streaming_wind;

fn check(scene: &str) -> gfx::GfxReport {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx");
    match gfx::check(scene, &out) {
        Ok(report) => {
            println!("{report}");
            report
        }
        Err(e) => panic!("{e}"),
    }
}

/// Writes a frame a test reads to `<target tmp>/og-terrain/<name>.png`
/// (for looking at a failure).
fn keep(name: &str, img: &RgbaImage) {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-terrain")
        .join(format!("{name}.png"));
    let _ = gfx::save_png(&path, img);
}

fn px(img: &RgbaImage, x: u32, y: u32) -> [f32; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [
        img.pixels[i] as f32,
        img.pixels[i + 1] as f32,
        img.pixels[i + 2] as f32,
    ]
}
