//! Frame comparison: a per-pixel tolerance and an RMSE threshold.

/// When two frames count as the same picture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance {
    /// A pixel is an outlier when any RGB channel differs by more than this (0..255).
    pub channel: u8,
    /// The largest allowed fraction of outlier pixels (0..1).
    pub max_outlier_fraction: f64,
    /// The largest allowed root-mean-square error over RGB, normalized to 0..1.
    pub rmse: f64,
}

impl Default for Tolerance {
    fn default() -> Self {
        Tolerance {
            channel: 32,
            max_outlier_fraction: 0.005,
            rmse: 0.01,
        }
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
    let mut diff_image = Vec::with_capacity(expected);
    for (a, r) in actual.chunks_exact(4).zip(reference.chunks_exact(4)) {
        let mut pixel_max = 0u8;
        let mut amplified = [0u8; 3];
        for c in 0..3 {
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
        pixels,
        diff_image,
    })
}
