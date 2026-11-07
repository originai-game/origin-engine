//! Frame comparison: a per-pixel tolerance and an RMSE threshold.
//!
//! One reference set, any real GPU backend (docs/ci.md §Reference frames):
//! references are recorded on [`RECORDING_TIER`] and kept under
//! `references/<tier>/`; a frame is judged against its own tier's reference
//! when one is committed, else the recording tier's. [`tolerance_for`] is the
//! rule every harness applies: a frame compared with a reference recorded on
//! its own backend gets the scene's tolerance (the tight one, so the
//! recording platform catches subtle regressions); compared with another
//! backend's, the wider of that and the scene's cross-platform tolerance
//! ([`Tolerance::CROSS_PLATFORM`] unless the scene names its own). A software
//! rasterizer (lavapipe, WARP) never gets the wider one.

use std::path::Path;

/// The backend tier new reference frames are recorded on. One platform
/// records, so a frame never flip-flops between hosts' renderings; the
/// others are held to it under the cross-platform tolerance.
pub const RECORDING_TIER: &str = "metal";

/// When two frames count as the same picture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance {
    /// A pixel is an outlier when any RGB channel differs by more than this (0..255).
    pub channel: u8,
    /// The largest allowed fraction of outlier pixels (0..1).
    pub max_outlier_fraction: f64,
    /// The largest allowed root-mean-square error over RGB, normalized to 0..1.
    pub rmse: f64,
    /// The largest allowed mean signed difference of any one RGB channel,
    /// normalized to 0..1: a whole frame brighter or darker. 1.0 is no limit;
    /// only [Tolerance::CROSS_PLATFORM] sets one, the guard that lets its
    /// RMSE widen without letting a dimmer light through.
    pub max_bias: f64,
}

impl Default for Tolerance {
    fn default() -> Self {
        Tolerance {
            channel: 32,
            max_outlier_fraction: 0.005,
            rmse: 0.01,
            max_bias: 1.0,
        }
    }
}

impl Tolerance {
    /// What a frame from another GPU backend may differ by: the default's
    /// outliers, its RMSE widened to 0.016, and no channel's mean off by more
    /// than 0.008. Set from the measured spread with a margin (RMSE 1.5x,
    /// bias 1.4x; 2026-11-08, Metal references, RTX 5060 on DX12, on
    /// Vulkan and in Chrome 150 on D3D12): rasterisation, threshold and
    /// sampling ties along edges, RMSE at most 0.0102 (terrain-grass) and
    /// outliers at most 0.08 % (shadows-soft), a channel's mean bias at most
    /// 0.0057 (dof-high) outside the four scenes whose
    /// spread is wider by nature and that name their own (scenes.rs:
    /// visual-garment, stylise-print, sprites-array, tilemap-lit). A darker
    /// light, a lost shadow, a moved or recoloured element still exceed it
    /// (tests/gfx.rs `the_cross_platform_tolerance_still_catches_small_regressions`:
    /// a 10 % dimmer sun is RMSE 0.0142 but a bias of 0.0102).
    pub const CROSS_PLATFORM: Tolerance = Tolerance {
        channel: 32,
        max_outlier_fraction: 0.005,
        rmse: 0.016,
        max_bias: 0.008,
    };

    /// The looser of the two per-pixel, outlier and RMSE limits, and the
    /// tighter bias limit: the bias limit is what a cross-platform tolerance
    /// trades for its wider RMSE.
    pub fn widened(self, other: Tolerance) -> Tolerance {
        Tolerance {
            channel: self.channel.max(other.channel),
            max_outlier_fraction: self.max_outlier_fraction.max(other.max_outlier_fraction),
            rmse: self.rmse.max(other.rmse),
            max_bias: self.max_bias.min(other.max_bias),
        }
    }
}

/// The tier a reference frame was recorded on: the name of its directory
/// (`references/<tier>/<name>.png`).
pub fn reference_tier(reference: &Path) -> String {
    reference
        .parent()
        .and_then(Path::file_name)
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The tolerance a frame rendered on `tier` gets against `reference`: `same`
/// when the reference was recorded on `tier` or the adapter is a software
/// rasterizer, else `same` widened to `cross` (module docs).
pub fn tolerance_for(
    same: Tolerance,
    cross: Tolerance,
    tier: &str,
    software: bool,
    reference: &Path,
) -> Tolerance {
    if software || reference_tier(reference) == tier {
        same
    } else {
        same.widened(cross)
    }
}

/// How two frames differ.
#[derive(Clone, Debug, PartialEq)]
pub struct Comparison {
    /// RMSE over RGB, normalized to 0..1.
    pub rmse: f64,
    /// The largest channel difference anywhere (0..255).
    pub max_channel_diff: u8,
    /// Pixels with a channel difference above the tolerance.
    pub outliers: usize,
    /// The mean signed difference (actual - reference) of each RGB channel,
    /// normalized to -1..1.
    pub bias: [f64; 3],
    pub pixels: usize,
    /// An RGBA8 visualisation: differences amplified 4x, outliers in pure red.
    pub diff_image: Vec<u8>,
}

impl Comparison {
    pub fn outlier_fraction(&self) -> f64 {
        self.outliers as f64 / self.pixels.max(1) as f64
    }

    /// `Ok` when within `tolerance`, otherwise every limit that was exceeded.
    pub fn check(&self, tolerance: &Tolerance) -> Result<(), String> {
        let mut failures = Vec::new();
        if self.rmse > tolerance.rmse {
            failures.push(format!("RMSE {:.5} > {:.5}", self.rmse, tolerance.rmse));
        }
        let bias = self.bias.iter().fold(0.0f64, |m, b| m.max(b.abs()));
        if bias > tolerance.max_bias {
            failures.push(format!(
                "mean channel bias {:+.5} {:+.5} {:+.5} > {:.5}",
                self.bias[0], self.bias[1], self.bias[2], tolerance.max_bias
            ));
        }
        if self.outlier_fraction() > tolerance.max_outlier_fraction {
            failures.push(format!(
                "{} pixels ({:.3}%) differ by more than {} > {:.3}%",
                self.outliers,
                self.outlier_fraction() * 100.0,
                tolerance.channel,
                tolerance.max_outlier_fraction * 100.0
            ));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("; "))
        }
    }
}

/// Compares two RGBA8 frames of the same size (alpha is ignored).
pub fn compare(
    actual: &[u8],
    reference: &[u8],
    width: u32,
    height: u32,
    tolerance: &Tolerance,
) -> Result<Comparison, String> {
    let expected = width as usize * height as usize * 4;
    if actual.len() != expected || reference.len() != expected {
        return Err(format!(
            "size mismatch: {}x{} needs {expected} bytes, got {} and {}",
            width,
            height,
            actual.len(),
            reference.len()
        ));
    }
    let mut sum_sq = 0.0f64;
    let mut max_channel_diff = 0u8;
    let mut outliers = 0usize;
    let mut signed = [0i64; 3];
    let mut diff_image = Vec::with_capacity(expected);
    for (a, r) in actual
        .as_chunks::<4>()
        .0
        .iter()
        .zip(reference.as_chunks::<4>().0.iter())
    {
        let mut pixel_max = 0u8;
        let mut amplified = [0u8; 3];
        for c in 0..3 {
            signed[c] += a[c] as i64 - r[c] as i64;
            let d = a[c].abs_diff(r[c]);
            pixel_max = pixel_max.max(d);
            sum_sq += (d as f64 / 255.0).powi(2);
            amplified[c] = d.saturating_mul(4);
        }
        max_channel_diff = max_channel_diff.max(pixel_max);
        if pixel_max > tolerance.channel {
            outliers += 1;
            diff_image.extend_from_slice(&[255, 0, 0, 255]);
        } else {
            diff_image.extend_from_slice(&[amplified[0], amplified[1], amplified[2], 255]);
        }
    }
    let pixels = (width * height) as usize;
    let rmse = (sum_sq / (pixels * 3).max(1) as f64).sqrt();
    Ok(Comparison {
        rmse,
        max_channel_diff,
        outliers,
        bias: signed.map(|s| s as f64 / 255.0 / pixels.max(1) as f64),
        pixels,
        diff_image,
    })
}
