//! Source preparation, before the one environment prefilter.
//!
//! Inverse tone mapping: Reinhard et al., "Photographic Tone Reproduction
//! for Digital Images" (2002), inverse global curve; Banterle et al.,
//! "A framework for inverse tone mapping" (2007), doi:10.1007/s00371-007-0124-9,
//! linearisation followed by expansion and a separate highlight estimate.
//! Outdoor cues: Lalonde et al., "Estimating Natural Illumination from a
//! Single Outdoor Image" (ICCV 2009), and Hold-Geoffroy et al., "Deep Outdoor
//! Illumination Estimation" (CVPR 2017). These motivate using the disc and
//! circumsolar gradient, not treating clipped white as measured radiance.
//! This is an analytic, relative-radiance estimator, not their learned model
//! or a calibrated lux measurement. No implementation code is ported.
//! Sky fit: Preetham, Shirley & Smits, "A Practical Analytic Model for
//! Daylight" (SIGGRAPH 1999), Perez luminance distribution and atmospheric
//! attenuation. Fit turbidity and exposure to unclipped sky (the panorama
//! fitting stage of Lalonde/Hold-Geoffroy, without their image priors/CNN).

use super::{CookError, EnvironmentDoc};
use glam::Vec3;
use image::Rgb32FImage;
use std::f32::consts::{PI, TAU};

const LUMA: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);
// Below an upper-only sky, preserve the column's horizon before easing
// into the author-selected (or horizon-averaged) ground, in linear light.
const HORIZON_BLEND_DEGREES: f32 = 6.0;

pub(super) fn expand(rgb: &mut [f32], amount: f32) {
    for p in rgb.as_chunks_mut::<3>().0 {
        let y = Vec3::from_slice(p).dot(LUMA);
        // Bounded inverse Reinhard; one scale preserves chromaticity.
        let gain = 1.0 / (1.0 - amount * y.clamp(0.0, 1.0));
        p.iter_mut().for_each(|c| *c *= gain);
    }
}

pub(super) fn panorama(
    name: &str,
    img: Rgb32FImage,
    doc: &EnvironmentDoc,
) -> Result<Rgb32FImage, CookError> {
    let (w, h) = img.dimensions();
    if w < 4 || h < 2 || (!doc.upper && (w as f64 / (2.0 * h as f64) - 1.0).abs() > 0.05) {
        return Err(CookError(format!(
            "{name} is {w} × {h}: a full equirectangular panorama must be within 5% of 2:1; hemisphere upper instead takes a zenith-to-horizon image"
        )));
    }
    // Resample in linear light, never crop away a strip of the panorama.
    let height = w.div_ceil(2);
    let width = 2 * height;
    let upper_height = if doc.upper {
        height.div_ceil(2)
    } else {
        height
    };
    let img = image::imageops::resize(
        &img,
        width,
        upper_height,
        image::imageops::FilterType::Triangle,
    );
    if !doc.upper {
        return Ok(img);
    }
    let mut ground = Vec3::ZERO;
    let rows = upper_height.min(4);
    for y in upper_height - rows..upper_height {
        for x in 0..width {
            ground += Vec3::from_array(img.get_pixel(x, y).0);
        }
    }
    ground /= (width * rows) as f32;
    if let Some(color) = doc.ground {
        ground = color;
    }
    Ok(Rgb32FImage::from_fn(width, height, |x, y| {
        if y < upper_height {
            *img.get_pixel(x, y)
        } else {
            let horizon = Vec3::from_array(img.get_pixel(x, upper_height - 1).0);
            let t = ((y - upper_height) as f32 * 180.0 / (height as f32 * HORIZON_BLEND_DEGREES))
                .min(1.0);
            let eased = t * t * (3.0 - 2.0 * t);
            image::Rgb(horizon.lerp(ground, eased).to_array())
        }
    }))
}

pub(super) fn facts(img: &Rgb32FImage, upper: bool) -> serde_json::Value {
    let (w, h) = img.dimensions();
    let (mut sum, mut max) = (0.0, 0.0f32);
    for y in 0..h {
        for c in 0..3 {
            let d = (img.get_pixel(0, y)[c] - img.get_pixel(w - 1, y)[c]).abs();
            sum += d;
            max = max.max(d);
        }
    }
    let clipped = img
        .pixels()
        .filter(|p| p.0.iter().any(|c| *c >= 1.0))
        .count() as f32
        / (w * h) as f32;
    let mut facts = serde_json::json!({
        "space": "linear RGB", "width": w, "height": h,
        "wrapMean": sum / (3*h) as f32, "wrapMax": max,
        "clippedFraction": clipped, "lowerMirrorMean": null,
        "lowerBlueFraction": null,
        "lowerHemisphere": "hemisphere upper: the source has no lower hemisphere; ground is filled by the cook"
    });
    if upper {
        return facts;
    }
    // A reproducible cue, not a semantic classifier: mirror error and blue
    // fraction are exposed so the platform can decide whether it is sky.
    let mut mirror = 0.0;
    let mut blue = 0;
    for y in h / 2..h {
        for x in 0..w {
            let a = Vec3::from_array(img.get_pixel(x, y).0);
            let b = Vec3::from_array(img.get_pixel(x, h - 1 - y).0);
            mirror += (a - b).abs().element_sum() / 3.0;
            blue += usize::from(a.z > a.x && a.z > a.y);
        }
    }
    let lower = (w * (h - h / 2)) as f32;
    facts["lowerMirrorMean"] = serde_json::json!(mirror / lower);
    facts["lowerBlueFraction"] = serde_json::json!(blue as f32 / lower);
    facts["lowerHemisphere"] = serde_json::json!(
        "mirror error and blue fraction are cues, not a sky/ground classification"
    );
    facts
}

/// Resample faces to a panorama for the same solar estimation as a source
/// panorama. Mapping is the inverse of render-data's `face_direction`.
pub(super) fn cube_panorama(size: u32, rgb: &[f32]) -> Rgb32FImage {
    let (w, h) = (4 * size, 2 * size);
    Rgb32FImage::from_fn(w, h, |x, y| {
        let d = direction(w, h, (y * w + x) as usize);
        let a = d.abs();
        let (face, u, v) = if a.x >= a.y && a.x >= a.z {
            if d.x > 0.0 {
                (0, -d.z / a.x, d.y / a.x)
            } else {
                (1, d.z / a.x, d.y / a.x)
            }
        } else if a.y >= a.z {
            if d.y > 0.0 {
                (2, d.x / a.y, -d.z / a.y)
            } else {
                (3, d.x / a.y, d.z / a.y)
            }
        } else if d.z > 0.0 {
            (4, d.x / a.z, d.y / a.z)
        } else {
            (5, -d.x / a.z, d.y / a.z)
        };
        let px = ((u + 1.0) * 0.5 * size as f32 - 0.5).clamp(0.0, (size - 1) as f32);
        let py = ((1.0 - v) * 0.5 * size as f32 - 0.5).clamp(0.0, (size - 1) as f32);
        let (x0, y0) = (px.floor() as u32, py.floor() as u32);
        let sample = |x, y| {
            let at = ((face * size + y) * size + x) as usize * 3;
            Vec3::from_slice(&rgb[at..at + 3])
        };
        image::Rgb(
            sample(x0, y0)
                .lerp(sample((x0 + 1).min(size - 1), y0), px.fract())
                .lerp(
                    sample(x0, (y0 + 1).min(size - 1)).lerp(
                        sample((x0 + 1).min(size - 1), (y0 + 1).min(size - 1)),
                        px.fract(),
                    ),
                    py.fract(),
                )
                .to_array(),
        )
    })
}

fn direction(w: u32, h: u32, k: usize) -> Vec3 {
    let lon = ((k as u32 % w) as f32 + 0.5) / w as f32 * TAU - PI;
    let lat = PI * (0.5 - ((k as u32 / w) as f32 + 0.5) / h as f32);
    Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin())
}

/// Fit the visible, unclipped sky gradient to the Perez daylight family.
/// For each turbidity, exposure is the least-squares scale; the minimum
/// residual chooses turbidity. Samples cover the upper sky, not the ground
/// or the clipped solar disc. The unknown exposure precludes absolute lux.
fn daylight(img: &Rgb32FImage, sun: Vec3) -> Option<Vec3> {
    let (w, h) = img.dimensions();
    let zenith = sun.y.clamp(0.01, 1.0).acos();
    let mut samples = Vec::new();
    for y in (0..h / 2).step_by((h / 32).max(1) as usize) {
        for x in (0..w).step_by((w / 64).max(1) as usize) {
            let d = direction(w, h, (y * w + x) as usize);
            let p = Vec3::from_array(img.get_pixel(x, y).0);
            if d.y > 0.08 && p.max_element() < 0.95 && p.dot(LUMA) > 0.005 {
                samples.push((d, p.dot(LUMA)));
            }
        }
    }
    if samples.len() < 16 {
        return None;
    }
    let mut best = (f32::INFINITY, 0.0, 2.0);
    for step in 0..=36 {
        let t = 1.0 + step as f32 * 0.25;
        let a = 0.1787 * t - 1.4630;
        let b = -0.3554 * t + 0.4275;
        let c = -0.0227 * t + 5.3251;
        let d = 0.1206 * t - 2.5771;
        let e = -0.0670 * t + 0.3703;
        let perez = |mu: f32, gamma: f32| {
            (1.0 + a * (b / mu.max(0.01)).exp())
                * (1.0 + c * (d * gamma).exp() + e * gamma.cos().powi(2))
        };
        let yz = ((4.0453 * t - 4.9710) * ((4.0 / 9.0 - t / 120.0) * (PI - 2.0 * zenith)).tan()
            - 0.2155 * t
            + 2.4192)
            .max(0.01)
            * 1000.0;
        let model = |dir: Vec3| {
            yz * perez(dir.y, dir.dot(sun).clamp(-1.0, 1.0).acos()) / perez(1.0, zenith)
        };
        let (mut xy, mut xx) = (0.0, 0.0);
        for &(dir, value) in &samples {
            let m = model(dir);
            xy += m * value;
            xx += m * m;
        }
        let scale = xy / xx.max(1e-12);
        let error = samples
            .iter()
            .map(|&(dir, value)| (model(dir) * scale - value).powi(2))
            .sum::<f32>();
        if error < best.0 {
            best = (error, scale, t);
        }
    }
    let (_, scale, t) = best;
    // Preetham's optical air mass and wavelength-dependent Rayleigh and
    // aerosol transmittance; 127.5 klux extraterrestrial direct illuminance.
    let degrees = zenith.to_degrees();
    let mass = 1.0 / (sun.y.max(0.0) + 0.15 * (93.885 - degrees).powf(-1.253));
    let beta = 0.04608 * t - 0.04586;
    let attenuation =
        |lambda: f32| (-mass * (0.008735 * lambda.powf(-4.08) + beta * lambda.powf(-1.3))).exp();
    let color = Vec3::new(attenuation(0.68), attenuation(0.55), attenuation(0.44));
    Some(color / color.dot(LUMA).max(1e-6) * (127_500.0 * attenuation(0.55) * scale))
}

/// Locate the brightest compact connected disc in the upper hemisphere.
/// Its observed solid angle and the unclipped circumsolar gradient determine
/// the estimate. It is deliberately reported as an estimate: clipping has
/// destroyed information, and a white cloud can be ambiguous.
pub(super) fn take_sun(img: &mut Rgb32FImage, strength: f32) -> og_render_data::EnvironmentSun {
    let (w, h) = img.dimensions();
    let pixels = (w * h) as usize;
    let peak = (0..pixels / 2)
        .max_by(|&a, &b| {
            let y = |k| Vec3::from_slice(&img.as_raw()[3 * k..3 * k + 3]).dot(LUMA);
            y(a).total_cmp(&y(b))
        })
        .unwrap_or(0);
    let peak_rgb = Vec3::from_slice(&img.as_raw()[3 * peak..3 * peak + 3]);
    if peak_rgb.min_element() < 0.8 {
        return Default::default();
    }
    let mut visited = vec![false; pixels];
    let mut disc = vec![peak];
    visited[peak] = true;
    let mut at = 0;
    let mut area = 0.0;
    let mut aim = Vec3::ZERO;
    while at < disc.len() {
        let k = disc[at];
        at += 1;
        let d = direction(w, h, k);
        let solid = TAU / w as f32 * PI / h as f32 * (1.0 - d.y * d.y).sqrt();
        area += solid;
        aim += d * solid;
        let (x, y) = (k as u32 % w, k as u32 / w);
        for (nx, ny) in [
            ((x + w - 1) % w, y),
            ((x + 1) % w, y),
            (x, y.saturating_sub(1)),
            (x, (y + 1).min(h / 2 - 1)),
        ] {
            let j = (ny * w + nx) as usize;
            if !visited[j] {
                visited[j] = true;
                if Vec3::from_slice(&img.as_raw()[3 * j..3 * j + 3]).min_element() >= 0.8 {
                    disc.push(j);
                }
            }
        }
    }
    // Large overcast white regions are not a measurable solar disc.
    if area <= 0.0 || area > 0.03 {
        return Default::default();
    }
    let aim = aim.normalize();
    let radius = (1.0 - area / TAU).acos();
    let (mut ring, mut ring_area) = (Vec3::ZERO, 0.0);
    let (mut outer, mut outer_area) = (Vec3::ZERO, 0.0);
    for k in 0..pixels / 2 {
        let d = direction(w, h, k);
        let angle = d.dot(aim).clamp(-1.0, 1.0).acos();
        let solid = (1.0 - d.y * d.y).sqrt();
        let p = Vec3::from_slice(&img.as_raw()[3 * k..3 * k + 3]);
        if angle > radius * 1.5 && angle < radius * 2.5 + 0.02 {
            ring += p * solid;
            ring_area += solid;
        }
        if angle > radius * 3.0 + 0.04 && angle < radius * 4.0 + 0.08 {
            outer += p * solid;
            outer_area += solid;
        }
    }
    if ring_area == 0.0 || outer_area == 0.0 {
        return Default::default();
    }
    ring /= ring_area;
    outer /= outer_area;
    // Fit the unclipped sky while the disc is still present, then remove
    // it from IBL. If fitting has insufficient samples, only the clipping
    // lower bound is knowable; do not invent a daylight fit.
    let lower_bound =
        (peak_rgb / (1.0 - 0.98 * peak_rgb.dot(LUMA)).max(0.02) - outer).max(Vec3::ZERO) * area;
    let light = daylight(img, aim).unwrap_or(lower_bound) * strength;
    for k in disc {
        img.as_mut()[3 * k..3 * k + 3].copy_from_slice(&ring.to_array());
    }
    let intensity = light.max_element();
    if intensity <= 0.0 {
        return Default::default();
    }
    og_render_data::EnvironmentSun {
        direction: aim.to_array(),
        color: (light / intensity).to_array(),
        intensity,
        angular_size: 2.0 * radius,
    }
}
