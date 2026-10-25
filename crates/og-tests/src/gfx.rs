//! The gfx reference harness: render a named scene headless, compare it with
//! `crates/og-tests/references/<backend-tier>/<scene>.png`, and on failure
//! write the actual frame and a diff image under the output directory.
//! On the vulkan-cpu tier, a scene on [`VULKAN_CPU_KNOWN_DIVERGENCES`] past
//! its tolerance against metal's reference is reported as tolerated.
//!
//! `OG_GFX_CAPTURE=1` (re)records every reference instead of comparing;
//! `OG_GFX_CAPTURE=scene-a,scene-b` records only those scenes.

use crate::compare::{Comparison, compare};
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

/// The CPU Vulkan tier (Mesa lavapipe): the orbs' and CI's adapter, with no
/// references of its own since 2026-10-22, so its frames are judged against
/// metal's (docs/ci.md §The vulkan-cpu tier's known divergences).
pub const VULKAN_CPU: &str = "vulkan-cpu";

/// A scene whose lavapipe frame is known to exceed its tolerance against
/// metal's reference, with the divergence measured when it was listed.
#[derive(Clone, Copy, Debug)]
pub struct KnownDivergence {
    pub scene: &'static str,
    /// RMSE against metal's reference when measured.
    pub rmse: f64,
    /// Outlier pixels against metal's reference when measured.
    pub outliers: usize,
    /// When it was measured, and on what.
    pub measured: &'static str,
}

/// How far past its measured divergence a listed scene stays tolerated: a
/// listed frame that moves further than this on lavapipe fails again.
pub const KNOWN_DIVERGENCE_MARGIN: f64 = 1.25;

impl KnownDivergence {
    /// Whether `c` is within this entry's measured divergence and the margin.
    pub fn covers(&self, c: &Comparison) -> bool {
        c.rmse <= self.rmse * KNOWN_DIVERGENCE_MARGIN
            && c.outliers as f64 <= self.outliers as f64 * KNOWN_DIVERGENCE_MARGIN
    }
}

impl std::fmt::Display for KnownDivergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "listed rmse={:.5} outliers={}, measured {}",
            self.rmse, self.outliers, self.measured
        )
    }
}

/// The orb's lavapipe and CI's, which agree to a few outlier pixels; each
/// entry holds the larger of the two.
const MEASURED_2026_10_25: &str =
    "2026-10-25, llvmpipe of Mesa 25.0.7 with LLVM 15.0.6 (orb) and LLVM 19.1.7 (CI)";

/// The frames known to diverge on the vulkan-cpu tier, the one list of
/// them (docs/ci.md §The vulkan-cpu tier's known divergences). Read only
/// when the adapter's tier is vulkan-cpu and the scene is judged against
/// metal's reference: such a scene past its tolerance passes and is
/// reported as tolerated, with its numbers, up to [`KNOWN_DIVERGENCE_MARGIN`]
/// times its measured divergence; one within tolerance is reported so the
/// list shrinks. Every other tier, and every scene not listed, is judged
/// as before. Measured scene by scene with the tests that render them
/// (tests/evidence/ci/known-frames/NOTES.md); every one passes on metal.
pub const VULKAN_CPU_KNOWN_DIVERGENCES: &[KnownDivergence] = &[
    KnownDivergence {
        scene: "atmosphere-night",
        rmse: 0.01312,
        outliers: 558,
        measured: MEASURED_2026_10_25,
    },
    KnownDivergence {
        scene: "atmosphere-moon",
        rmse: 0.01295,
        outliers: 552,
        measured: MEASURED_2026_10_25,
    },
    KnownDivergence {
        scene: "sprites-array",
        rmse: 0.01255,
        outliers: 434,
        measured: MEASURED_2026_10_25,
    },
    KnownDivergence {
        scene: "tilemap-lit",
        rmse: 0.02653,
        outliers: 17666,
        measured: MEASURED_2026_10_25,
    },
    KnownDivergence {
        scene: "water-underwater",
        rmse: 0.02253,
        outliers: 20805,
        measured: MEASURED_2026_10_25,
    },
];

/// Reports a known divergence: printed with the test's output, and a line
/// in `$OG_EVIDENCE_DIR/gfx-known-divergences.txt` (default
/// `target/evidence/`), which the gate's test steps print, since nextest
/// shows no output of a passing test.
fn note_known_divergence(line: &str) {
    use std::io::Write as _;
    println!("{line}");
    let dir = std::env::var_os("OG_EVIDENCE_DIR").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/evidence"),
        PathBuf::from,
    );
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("gfx-known-divergences.txt"))
    {
        let _ = file.write_all(format!("{line}\n").as_bytes());
    }
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

/// Renders the scene called `scene` headless (see [`render_built`]).
pub fn render(scene: &str) -> Result<Rendered, String> {
    render_built(&mut scenes::build(scene)?)
}

/// Renders `scene` and compares it with its reference, this tier's or else
/// metal's (or records this tier's when capture is requested). On a mismatch, writes
/// `<out_dir>/<tier>/<scene>.actual.png` and `.diff.png` and returns `Err`.
/// A scene that does not apply to the adapter's tier (TAA on GL, MSAA on
/// WebGPU-class backends) is an error here: callers check
/// [`GfxScene::applies`] first and assert something else instead.
pub fn check(scene: &str, out_dir: &Path) -> Result<GfxReport, String> {
    let mut built = scenes::build(scene)?;
    let rendered = render_built(&mut built)?;
    if !built.applies.includes(rendered.info.tier) {
        return Err(format!(
            "gfx {scene} has no reference on the {:?} tier ({})",
            rendered.info.tier,
            rendered.info.backend_tier()
        ));
    }
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
        save_png(&own, &rendered.image)?;
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
    let known = (tier == VULKAN_CPU && reference != own)
        .then(|| {
            VULKAN_CPU_KNOWN_DIVERGENCES
                .iter()
                .find(|k| k.scene == scene)
        })
        .flatten();
    let verdict = match (verdict, known) {
        (Ok(()), None) => Ok(()),
        (Ok(()), Some(k)) => {
            note_known_divergence(&format!(
                "gfx {scene}: on the {VULKAN_CPU} known-divergence list ({k}) but within tolerance now (rmse={:.5} outliers={}): remove it from VULKAN_CPU_KNOWN_DIVERGENCES",
                comparison.rmse, comparison.outliers
            ));
            Ok(())
        }
        (Err(why), Some(k)) if k.covers(&comparison) => {
            note_known_divergence(&format!(
                "gfx {scene}: TOLERATED on {VULKAN_CPU} against metal, a known divergence ({k}): {why} (rmse={:.5} max_diff={} outliers={})",
                comparison.rmse, comparison.max_channel_diff, comparison.outliers
            ));
            Ok(())
        }
        (Err(why), Some(k)) => Err(format!(
            "{why}; beyond its known {VULKAN_CPU} divergence ({k}) by more than {KNOWN_DIVERGENCE_MARGIN}x"
        )),
        (Err(why), None) => Err(why),
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
