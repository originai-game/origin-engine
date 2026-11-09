//! The gfx reference harness: render a named scene headless, compare it with
//! `crates/og-tests/references/<backend-tier>/<scene>.png`, and on failure
//! write the actual frame and a diff image under the output directory.
//!
//! `OG_GFX_CAPTURE=1` (re)records every reference instead of comparing;
//! `OG_GFX_CAPTURE=scene-a,scene-b` records only those scenes.

use crate::compare::{Comparison, compare, tolerance_for};
use crate::scenes::{self, GfxScene};
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
            "gfx {} [{} on {}, aa {:?}]: draws={} batches={} instances={} culled={} triangles={} shadow_passes={} shadow_casters={} lights={} sprites={} glyphs={} sprite_draws={}",
            self.scene,
            self.tier,
            self.info.adapter,
            self.info.antialias,
            self.stats.draw_calls,
            self.stats.batches,
            self.stats.instances,
            self.stats.culled,
            self.stats.triangles,
            self.stats.shadow_passes,
            self.stats.shadow_casters,
            self.stats.lights,
            self.stats.sprites,
            self.stats.glyphs,
            self.stats.sprite_draws
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

/// The directory committed reference frames live in, or `OG_TEST_REFERENCES`
/// (the sandbox run compares against frames it captured itself).
pub fn references_dir() -> PathBuf {
    match std::env::var_os("OG_TEST_REFERENCES") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("references"),
    }
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

/// Renders a built scene headless: `frames` frames (calling its update
/// before each), then reads the last one back.
pub fn render_built(built: &mut GfxScene) -> Result<Rendered, String> {
    let t0 = Instant::now();
    let mut renderer = Renderer::new_blocking(&built.desc).map_err(|e| e.to_string())?;
    renderer.set_render_scale(built.render_scale);
    let mut stats = FrameStats::default();
    for frame in 0..built.frames.max(1) {
        if let Some(update) = built.update {
            update(&mut built.scene, frame);
        }
        stats = renderer
            .render(&built.scene, &RenderTarget::Offscreen)
            .map_err(|e| e.to_string())?;
    }
    // The GPU culling's counts of this frame, not an earlier one's.
    renderer.finish_stats(&mut stats);
    let image = renderer.read_offscreen().map_err(|e| e.to_string())?;
    Ok(Rendered {
        image,
        stats,
        info: renderer.info().clone(),
        setup_ms: t0.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Writes a frame a run produced (an actual, a diff, evidence) with the
/// `image` crate's fast encoder: quick, and large. A frame kept in git is
/// [`save_reference`]'s.
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

/// Records a reference frame (`OG_GFX_CAPTURE`) as og-render's
/// [`og_render::png_best`], the smallest PNG of its pixels.
pub fn save_reference(path: &Path, image: &RgbaImage) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, og_render::png_best(image)).map_err(|e| format!("{}: {e}", path.display()))
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

/// The FLIP of a failing frame against its reference (og-probe's
/// `frame::flip`, NVIDIA's LDR-FLIP): `<scene>.flip.png`, the heat map, and
/// `<scene>.flip-<n>.png`, the three worst regions (`FlipMap::worst`) as
/// reference | actual | heat enlarged, under `base`; answers
/// the lines the failure prints.
pub fn flip_report(
    reference: &RgbaImage,
    actual: &RgbaImage,
    base: &Path,
    scene: &str,
) -> Result<String, String> {
    use og_probe::frame::{Rgba, flip};
    let frame = |i: &RgbaImage| Rgba {
        width: i.width,
        height: i.height,
        pixels: i.pixels.clone(),
    };
    let (r, t) = (frame(reference), frame(actual));
    let map = flip::flip(&r, &t, flip::DEFAULT_PPD)?;
    let heat = map.heat_map();
    let write = |name: String, img: &Rgba| -> Result<String, String> {
        let path = base.join(name);
        std::fs::write(&path, img.png()).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(path.display().to_string())
    };
    let s = map.stats();
    let mut out = format!(
        "  FLIP      mean {:.4}, p95 {:.4}, max {:.4}: {}",
        s.mean,
        s.p95,
        s.max,
        write(format!("{scene}.flip.png"), &heat)?
    );
    let side = flip::region_side(map.width, map.height);
    for (n, region) in map.worst(3, side).iter().enumerate() {
        let long = region.rect.width.max(region.rect.height);
        let strip = flip::region_strip(&r, &t, &heat, region.rect, flip::enlargement(long, 256));
        let file = write(format!("{scene}.flip-{}.png", n + 1), &strip)?;
        out += &format!(
            "\n  worst {}   {}x{} at {},{}: mean {:.4}, max {:.4}: {file}",
            n + 1,
            region.rect.width,
            region.rect.height,
            region.rect.x,
            region.rect.y,
            region.mean,
            region.max
        );
    }
    Ok(out)
}

/// Renders the scene called `scene` headless (see [`render_built`]).
pub fn render(scene: &str) -> Result<Rendered, String> {
    render_built(&mut scenes::build(scene)?)
}

/// Renders `scene` and compares it with its reference, this tier's or else
/// metal's (or records this tier's when capture is requested). On a mismatch, writes
/// `<out_dir>/<tier>/<scene>.actual.png` and `.diff.png` and returns `Err`.
pub fn check(scene: &str, out_dir: &Path) -> Result<GfxReport, String> {
    check_built(scene, &mut scenes::build(scene)?, out_dir)
}

/// [`check`] of a scene already built as `scene`: og-tests-cooked's
/// registry builds its own scenes and checks them here.
pub fn check_built(scene: &str, built: &mut GfxScene, out_dir: &Path) -> Result<GfxReport, String> {
    let rendered = render_built(built)?;
    let tier = rendered.info.backend_tier();
    let own = reference_path(&tier, scene);
    let mut report = GfxReport {
        scene: scene.to_string(),
        tier: tier.clone(),
        info: rendered.info.clone(),
        stats: rendered.stats,
        reference: own.clone(),
        comparison: None,
    };
    if capture_requested(scene) {
        save_reference(&own, &rendered.image)?;
        return Ok(report);
    }
    // Frame by frame, as every reference harness here: a tier without its
    // own reference for this scene is judged against metal's and says so
    // (docs/ci.md §Browser reference tiers), so a scene recorded on metal
    // is held on the other tiers before their own frame is recorded.
    let metal = reference_path("metal", scene);
    let reference = if own.exists() || !metal.exists() {
        own.clone()
    } else {
        println!(
            "gfx {scene}: no {tier} reference at {}; comparing with metal's (record one with OG_GFX_CAPTURE={scene} on that tier)",
            own.display()
        );
        metal
    };
    report.reference = reference.clone();
    if !reference.exists() {
        return Err(format!(
            "{report}: no reference at {} for backend tier '{tier}', nor on metal; record one with OG_GFX_CAPTURE={scene}",
            own.display()
        ));
    }
    let expected = load_png(&reference)?;
    if (expected.width, expected.height) != (rendered.image.width, rendered.image.height) {
        return Err(format!(
            "{report}: reference is {}x{}, frame is {}x{}",
            expected.width, expected.height, rendered.image.width, rendered.image.height
        ));
    }
    let tolerance = tolerance_for(
        built.tolerance,
        built.cross_tolerance,
        &tier,
        rendered.info.software(),
        &reference,
    );
    let comparison = compare(
        &rendered.image.pixels,
        &expected.pixels,
        expected.width,
        expected.height,
        &tolerance,
    )?;
    let verdict = comparison.check(&tolerance);
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
        let flip = flip_report(&expected, &rendered.image, &base, scene)?;
        return Err(format!(
            "{report}: {why}\n  reference {}\n  actual    {}\n  diff      {}\n{flip}",
            reference.display(),
            actual_path.display(),
            diff_path.display()
        ));
    }
    Ok(report)
}
