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
//! Brackets: Debevec & Malik, "Recovering High Dynamic Range Radiance Maps
//! from Photographs" (SIGGRAPH 1997), weighted log-radiance merge, with the
//! known sRGB response instead of estimating a camera response curve.

use super::{CookError, EnvironmentDoc};
use glam::Vec3;
use image::Rgb32FImage;
use std::f32::consts::{PI, TAU};

const LUMA: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);

pub(super) fn expand(rgb: &mut [f32], amount: f32) {
    for p in rgb.chunks_exact_mut(3) {
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
            image::Rgb(ground.to_array())
        }
    }))
}

/// Log merge excludes clipped and black channels. If every bracket is at
/// an endpoint, use the shortest clipped exposure (a lower bound), or black.
pub(super) fn merge(images: &[(Rgb32FImage, f32)]) -> Result<Rgb32FImage, CookError> {
    let (w, h) = images[0].0.dimensions();
    if images.iter().any(|(i, _)| i.dimensions() != (w, h)) {
        return Err(CookError(
            "exposures must have exactly the source's dimensions and registration".into(),
        ));
    }
    Ok(Rgb32FImage::from_fn(w, h, |x, y| {
        image::Rgb(std::array::from_fn(|c| {
            let (mut sum, mut weights) = (0.0, 0.0);
            let mut fallback: f32 = 0.0;
            for (img, ev) in images {
                let linear = img.get_pixel(x, y)[c];
                let encoded = if linear <= 0.0031308 {
                    12.92 * linear
                } else {
                    1.055 * linear.powf(1.0 / 2.4) - 0.055
                };
                let weight = if encoded <= 1.0 / 255.0 || encoded >= 254.0 / 255.0 {
                    0.0
                } else {
                    encoded.min(1.0 - encoded)
                };
                let radiance = linear / ev.exp2();
                fallback = fallback.max(radiance);
                if weight > 0.0 {
                    sum += weight * radiance.ln();
                    weights += weight;
                }
            }
            if weights > 0.0 {
                (sum / weights).exp()
            } else {
                fallback
            }
        }))
    }))
}

pub(super) fn facts(img: &Rgb32FImage) -> String {
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
    format!(
        "environment facts (linear RGB): wrapMean {:.6}, wrapMax {:.6}, clippedFraction {:.6}, lowerMirrorMean {:.6}, lowerBlueFraction {:.6} (sky/ground cues, not a classification)",
        sum / (3 * h) as f32,
        max,
        clipped,
        mirror / lower,
        blue as f32 / lower
    )
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
    for y in (0..h/2).step_by((h/32).max(1) as usize) {
        for x in (0..w).step_by((w/64).max(1) as usize) {
            let d = direction(w,h,(y*w+x) as usize);
            let p = Vec3::from_array(img.get_pixel(x,y).0);
            if d.y > 0.08 && p.max_element() < 0.95 && p.dot(LUMA) > 0.005 {
                samples.push((d, p.dot(LUMA)));
            }
        }
    }
    if samples.len() < 16 { return None; }
    let mut best = (f32::INFINITY, 0.0, 2.0);
    for step in 0..=36 {
        let t = 1.0 + step as f32 * 0.25;
        let a = 0.1787*t - 1.4630;
        let b = -0.3554*t + 0.4275;
        let c = -0.0227*t + 5.3251;
        let d = 0.1206*t - 2.5771;
        let e = -0.0670*t + 0.3703;
        let perez = |mu: f32, gamma: f32| (1.0+a*(b/mu.max(0.01)).exp()) * (1.0+c*(d*gamma).exp()+e*gamma.cos().powi(2));
        let yz = ((4.0453*t-4.9710)*((4.0/9.0-t/120.0)*(PI-2.0*zenith)).tan()-0.2155*t+2.4192).max(0.01)*1000.0;
        let model = |dir: Vec3| yz * perez(dir.y, dir.dot(sun).clamp(-1.0,1.0).acos()) / perez(1.0,zenith);
        let (mut xy, mut xx) = (0.0, 0.0);
        for &(dir, value) in &samples { let m = model(dir); xy += m*value; xx += m*m; }
        let scale = xy/xx.max(1e-12);
        let error = samples.iter().map(|&(dir,value)| (model(dir)*scale-value).powi(2)).sum::<f32>();
        if error < best.0 { best = (error,scale,t); }
    }
    let (_,scale,t) = best;
    // Preetham's optical air mass and wavelength-dependent Rayleigh and
    // aerosol transmittance; 127.5 klux extraterrestrial direct illuminance.
    let degrees = zenith.to_degrees();
    let mass = 1.0/(sun.y.max(0.0)+0.15*(93.885-degrees).powf(-1.253));
    let beta = 0.04608*t-0.04586;
    let attenuation = |lambda: f32| (-mass*(0.008735*lambda.powf(-4.08)+beta*lambda.powf(-1.3))).exp();
    let color = Vec3::new(attenuation(0.68),attenuation(0.55),attenuation(0.44));
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
    let lower_bound = (peak_rgb / (1.0 - 0.98 * peak_rgb.dot(LUMA)).max(0.02) - outer).max(Vec3::ZERO) * area;
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
