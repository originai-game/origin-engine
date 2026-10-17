//! The gfx reference harness: render a named scene headless, compare it with
//! `crates/og-tests/references/<backend-tier>/<scene>.png`, and on failure
//! write the actual frame and a diff image under the output directory.
//!
//! `OG_GFX_CAPTURE=1` (re)records every reference instead of comparing;
//! `OG_GFX_CAPTURE=scene-a,scene-b` records only those scenes.

use crate::compare::{compare, Comparison};
use crate::scenes;
use og_render::{FrameStats, RenderInfo, RenderTarget, Renderer, RgbaImage};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// What one harness run did.
#[derive(Debug)]
pub struct GfxReport {
    pub scene: String,
    /// The reference directory name (`RenderInfo::backend_tier`).
    pub tier: String,
    pub info: RenderInfo,
    pub stats: FrameStats,
    pub reference: PathBuf,
    /// `None` when the run captured the reference instead of comparing.
    pub comparison: Option<Comparison>,
}

impl std::fmt::Display for GfxReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "gfx {} [{} on {}]: draws={} batches={} instances={} triangles={}",
            self.scene,
            self.tier,
            self.info.adapter,
            self.stats.draw_calls,
            self.stats.batches,
            self.stats.instances,
            self.stats.triangles
        )?;
        match &self.comparison {
            Some(c) => write!(
                f,
                " rmse={:.5} max_diff={} outliers={} ({:.4}%)",
                c.rmse,
                c.max_channel_diff,
                c.outliers,
                c.outlier_fraction() * 100.0
            ),
            None => write!(f, " captured -> {}", self.reference.display()),
        }
    }
}

/// The directory committed reference frames live in.
pub fn references_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("references")
}

pub fn reference_path(tier: &str, scene: &str) -> PathBuf {
    references_dir().join(tier).join(format!("{scene}.png"))
}

/// Whether `OG_GFX_CAPTURE` asks to record `scene`.
pub fn capture_requested(scene: &str) -> bool {
    match std::env::var("OG_GFX_CAPTURE") {
        Ok(v) if v == "1" || v == "all" => true,
        Ok(v) => v.split(',').any(|s| s.trim() == scene),
        Err(_) => false,
    }
}

/// A rendered frame with what produced it.
pub struct Rendered {
    pub image: RgbaImage,
    pub stats: FrameStats,
    pub info: RenderInfo,
    pub setup_ms: f64,
}

/// Renders `scene` once, headless.
pub fn render(scene: &str) -> Result<Rendered, String> {
    let built = scenes::build(scene)?;
    let t0 = Instant::now();
    let mut renderer = Renderer::new_blocking(&built.desc).map_err(|e| e.to_string())?;
    let stats = renderer
        .render(&built.scene, &RenderTarget::Offscreen)
        .map_err(|e| e.to_string())?;
    let image = renderer.read_offscreen().map_err(|e| e.to_string())?;
    Ok(Rendered {
        image,
        stats,
        info: renderer.info().clone(),
        setup_ms: t0.elapsed().as_secs_f64() * 1000.0,
    })
}

pub fn save_png(path: &Path, image: &RgbaImage) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    image::save_buffer(
        path,
        &image.pixels,
        image.width,
        image.height,
        image::ColorType::Rgba8,
    )
    .map_err(|e| format!("{}: {e}", path.display()))
}

pub fn load_png(path: &Path) -> Result<RgbaImage, String> {
    let img = image::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .to_rgba8();
    Ok(RgbaImage {
        width: img.width(),
        height: img.height(),
        pixels: img.into_raw(),
    })
}

/// Renders `scene` and compares it with its reference (or records the
/// reference when capture is requested). On a mismatch, writes
/// `<out_dir>/<tier>/<scene>.actual.png` and `.diff.png` and returns `Err`.
pub fn check(scene: &str, out_dir: &Path) -> Result<GfxReport, String> {
    let built = scenes::build(scene)?;
    let rendered = render(scene)?;
    let tier = rendered.info.backend_tier();
    let reference = reference_path(&tier, scene);
    let mut report = GfxReport {
        scene: scene.to_string(),
        tier: tier.clone(),
        info: rendered.info.clone(),
        stats: rendered.stats,
        reference: reference.clone(),
        comparison: None,
    };
    if capture_requested(scene) {
        save_png(&reference, &rendered.image)?;
        return Ok(report);
    }
    if !reference.exists() {
        return Err(format!("{report}: no reference at {} for backend tier '{tier}'; record one with OG_GFX_CAPTURE={scene}", reference.display()));
    }
    let expected = load_png(&reference)?;
    if (expected.width, expected.height) != (rendered.image.width, rendered.image.height) {
        return Err(format!(
            "{report}: reference is {}x{}, frame is {}x{}",
            expected.width, expected.height, rendered.image.width, rendered.image.height
        ));
    }
    let comparison = compare(
        &rendered.image.pixels,
        &expected.pixels,
        expected.width,
        expected.height,
        &built.tolerance,
    )?;
    let verdict = comparison.check(&built.tolerance);
    let diff = RgbaImage {
        width: expected.width,
        height: expected.height,
        pixels: comparison.diff_image.clone(),
    };
    report.comparison = Some(comparison);
    if let Err(why) = verdict {
        let base = out_dir.join(&tier);
        let actual_path = base.join(format!("{scene}.actual.png"));
        let diff_path = base.join(format!("{scene}.diff.png"));
        save_png(&actual_path, &rendered.image)?;
        save_png(&diff_path, &diff)?;
        return Err(format!(
            "{report}: {why}\n  reference {}\n  actual    {}\n  diff      {}",
            reference.display(),
            actual_path.display(),
            diff_path.display()
        ));
    }
    Ok(report)
}
